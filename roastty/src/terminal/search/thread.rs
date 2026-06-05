//! The search-thread aggregator (port of the `Search` struct in upstream `terminal/search/Thread.zig`).
//!
//! This lands the complete search subsystem: the lock-aware, multi-screen `Search` aggregator (a
//! `ViewportSearch` plus a `ScreenSearch` per terminal screen, with `new` / `deinit` /
//! `is_complete` / `tick` / `feed` / `notify`) and the outer `Thread` — its types (`Options` /
//! `Message` / `Mailbox` / `EventCallback`), the `change_needle` / `select` / `drain_mailbox`
//! handlers, and the `thread_main` event loop with `spawn` / `ThreadHandle` (a std-concurrency
//! adaptation — `std::thread` + a `Condvar` wakeup + an `AtomicBool` stop + a `REFRESH_INTERVAL`
//! cadence — of upstream's libxev loop).

use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use super::super::blocking_queue::{BlockingQueue, Timeout};
use super::super::highlight::{Flattened, Untracked};
use super::super::message_data::MessageData;
use super::super::terminal::{Terminal, TerminalScreenKey};
use super::screen::{ScreenSearch, Select, Tick as ScreenTick};
use super::viewport::ViewportSearch;

/// The interval at which the search thread re-feeds the terminal to detect changes (upstream
/// `REFRESH_INTERVAL = 24` ms, 40 FPS).
const REFRESH_INTERVAL: Duration = Duration::from_millis(24);

/// The number of screen kinds (`TerminalScreenKey`: `Primary`, `Alternate`).
const SCREEN_KEY_COUNT: usize = 2;

fn key_index(key: TerminalScreenKey) -> usize {
    match key {
        TerminalScreenKey::Primary => 0,
        TerminalScreenKey::Alternate => 1,
    }
}

/// Per-screen searchers keyed by `TerminalScreenKey` (upstream's `EnumMap<Key, ScreenSearch>`,
/// modelled as a two-slot array since roastty has exactly two screen kinds and no `EnumMap`).
#[derive(Default)]
struct ScreenSearches {
    entries: [Option<ScreenSearch>; SCREEN_KEY_COUNT],
}

impl ScreenSearches {
    fn get(&self, key: TerminalScreenKey) -> Option<&ScreenSearch> {
        self.entries[key_index(key)].as_ref()
    }

    fn get_mut(&mut self, key: TerminalScreenKey) -> Option<&mut ScreenSearch> {
        self.entries[key_index(key)].as_mut()
    }

    /// Insert, returning any replaced searcher so the caller can `deinit` it (avoids leaking the
    /// replaced searcher's tracked pins once `feed`'s reconciliation lands).
    fn insert(&mut self, key: TerminalScreenKey, s: ScreenSearch) -> Option<ScreenSearch> {
        self.entries[key_index(key)].replace(s)
    }

    fn take(&mut self, key: TerminalScreenKey) -> Option<ScreenSearch> {
        self.entries[key_index(key)].take()
    }

    fn iter(&self) -> impl Iterator<Item = &ScreenSearch> {
        self.entries.iter().filter_map(|e| e.as_ref())
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut ScreenSearch> {
        self.entries.iter_mut().filter_map(|e| e.as_mut())
    }
}

/// State captured at the last screen switch (upstream `Search.ScreenState`). Initialized by `new`;
/// read by the deferred `feed` / `notify`.
struct ScreenState {
    key: TerminalScreenKey,
    total: Option<usize>,
    selected: Option<SelectedMatch>,
}

/// The last-notified selected match (upstream `Search.ScreenState.SelectedMatch`).
struct SelectedMatch {
    idx: usize,
    highlight: Untracked,
}

/// The progress of one `tick` across all screens (upstream `Search.Tick`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::terminal) enum Tick {
    /// All searches are complete.
    Complete,
    /// At least one screen made progress.
    Progress,
    /// All incomplete searches are blocked on a feed.
    Blocked,
}

/// Events emitted by the search thread (upstream `Thread.Event`). The caller handles these as it
/// sees fit. `ViewportMatches` borrows a `notify`-local buffer valid only for that callback call.
pub(in crate::terminal) enum Event<'a> {
    /// The search thread is exiting (emitted by the outer `Thread`; unused until it lands).
    Quit,
    /// Search is complete for the needle on all screens.
    Complete,
    /// The active screen's total match count changed.
    TotalMatches(usize),
    /// The selected match changed (or was cleared).
    SelectedMatch(Option<EventSelectedMatch>),
    /// The viewport matches changed (owned by `notify`, valid only during the callback).
    ViewportMatches(&'a [Flattened]),
}

/// A selected match reported to the callback (upstream `Event.SelectedMatch`).
pub(in crate::terminal) struct EventSelectedMatch {
    pub(in crate::terminal) idx: usize,
    pub(in crate::terminal) highlight: Flattened,
}

/// The multi-screen search aggregator owned by the search thread (upstream `Thread.Search`). It
/// drives one `ScreenSearch` per terminal screen plus one `ViewportSearch` for the active screen.
pub(crate) struct Search {
    /// Viewport search for the active screen.
    viewport: ViewportSearch,
    /// The searchers for all the screens.
    screens: ScreenSearches,
    /// State collected at the last screen switch (so a switch invalidates it all at once).
    last_screen: ScreenState,
    /// Whether the "complete" notification has been sent.
    last_complete: bool,
    /// Whether the last viewport matches are stale and need recomputing.
    stale_viewport_matches: bool,
}

impl Search {
    /// Construct an aggregator searching for `needle` (upstream `init`). Active dirty-tracking starts
    /// dirty so the first active-area change re-searches.
    pub(in crate::terminal) fn new(needle: &[u8]) -> Search {
        let mut viewport = ViewportSearch::new(needle);
        viewport.set_active_dirty(Some(true));
        Search {
            viewport,
            screens: ScreenSearches::default(),
            last_screen: ScreenState {
                key: TerminalScreenKey::Primary,
                total: None,
                selected: None,
            },
            last_complete: false,
            stale_viewport_matches: true,
        }
    }

    /// The needle this aggregator is searching for (upstream `s.viewport.needle()`).
    pub(in crate::terminal) fn needle(&self) -> &[u8] {
        self.viewport.needle()
    }

    /// Clear the last-notified selection (upstream `s.last_screen.selected = null`), so the next
    /// `notify` re-emits the selection. Used by the search thread's `select`.
    pub(in crate::terminal) fn reset_last_selected(&mut self) {
        self.last_screen.selected = None;
    }

    /// Tear down every screen searcher (upstream `deinit`). `ViewportSearch` frees itself on `Drop`.
    ///
    /// # Safety
    /// Each screen searcher's backing `Screen` must still be live (the `ScreenSearch::deinit`
    /// contract).
    pub(in crate::terminal) unsafe fn deinit(&mut self) {
        for s in self.screens.iter_mut() {
            // SAFETY: caller's contract — the backing screen is live.
            unsafe { s.deinit() };
        }
    }

    /// Whether all screen searches are complete (upstream `isComplete`). Vacuously true with no
    /// screens, matching upstream's empty-iterator behavior.
    pub(in crate::terminal) fn is_complete(&self) -> bool {
        self.screens.iter().all(|s| s.is_state_complete())
    }

    /// Tick every screen forward without taking the big lock (upstream `tick`). `Progress` dominates;
    /// `Blocked` only when every incomplete screen needs a feed and none progressed.
    pub(in crate::terminal) fn tick(&mut self) -> Tick {
        let mut result = Tick::Complete;
        for s in self.screens.iter_mut() {
            match s.tick() {
                ScreenTick::Progressed => result = Tick::Progress,
                ScreenTick::Complete => {}
                ScreenTick::FeedRequired => {
                    if result == Tick::Complete {
                        result = Tick::Blocked;
                    }
                }
            }
        }
        result
    }

    /// Reconcile searchers with the terminal's screens, honor the viewport-dirty flag, update the
    /// viewport search, and feed each searcher that needs it (upstream `feed`). This is the
    /// lock-holding step (it reads/mutates terminal state).
    ///
    /// # Safety
    /// `t` must be live and outlive this `Search`, the `Terminal` must not be moved/reallocated, and
    /// the caller holds the screen lock (no concurrent access). The per-screen searchers cache
    /// `NonNull<Screen>` into `t`'s screens; reconciliation drops a searcher whose screen vanished
    /// or was replaced *without* dereferencing the stale pointer (see step B).
    pub(in crate::terminal) unsafe fn feed(&mut self, t: NonNull<Terminal>) {
        // (A) Active screen switch resets the per-screen notification state.
        // SAFETY: caller's contract — `t` is live.
        let active_key = unsafe { Terminal::search_active_screen_key(t) };
        if active_key != self.last_screen.key {
            self.last_screen = ScreenState {
                key: active_key,
                total: None,
                selected: None,
            };
        }

        // (B) Reconcile searchers with the terminal's screens. Collect the present screen pointers
        // up front; no terminal reference is retained afterwards.
        // SAFETY: caller's contract — `t` is live.
        let present = unsafe { Terminal::present_screen_ptrs(t) };
        for key in [TerminalScreenKey::Primary, TerminalScreenKey::Alternate] {
            let remove = match self.screens.get(key) {
                None => false,
                Some(ss) => match present.iter().find(|(k, _)| *k == key) {
                    None => true,                              // screen gone
                    Some((_, ptr)) => ss.screen_ptr() != *ptr, // screen replaced
                },
            };
            if remove {
                // The backing screen was dropped or replaced, so its pin storage is already gone.
                // Drop the searcher WITHOUT `deinit` (untracking against a freed screen would be
                // use-after-free): roastty's `Screen` owns its tracked pins, so a dropped/replaced
                // screen takes them with it. This is a deliberate divergence from upstream's
                // `entry.value.deinit()`.
                let _ = self.screens.take(key);
            }
        }
        let needle = self.viewport.needle().to_vec();
        for (key, ptr) in &present {
            if self.screens.get(*key).is_some() {
                continue;
            }
            // SAFETY: `ptr` is a live terminal screen (see `# Safety`); no terminal reference held.
            let ss = unsafe { ScreenSearch::new(*ptr, &needle) };
            self.screens.insert(*key, ss);
        }

        // (C) Viewport dirty → re-search the active area.
        // SAFETY: `t` live; raw-pointer flag read, no reference materialized.
        if unsafe { Terminal::search_viewport_dirty(t) } {
            // SAFETY: `t` live.
            unsafe { Terminal::clear_search_viewport_dirty(t) };
            self.viewport.set_active_dirty(Some(true));
            if let Some(ss) = self.screens.get_mut(active_key) {
                // SAFETY: active screen live; no terminal reference held here.
                unsafe { ss.reload_active() };
            }
        }

        // (D) Update the viewport search over the active screen's pages.
        if let Some((_, active_ptr)) = present.iter().find(|(k, _)| *k == active_key) {
            // SAFETY: `active_ptr` is the live active screen; the `&Screen` is used only to read its
            // `&PageList` for `update`, which dereferences no `ScreenSearch` pointer.
            let pages = unsafe { active_ptr.as_ref() }.pages();
            // SAFETY: `pages` is read-only for the call.
            let updated = unsafe { self.viewport.update(pages) };
            if updated {
                self.stale_viewport_matches = true;
            }
        }

        // (E) Feed each searcher that needs more data.
        for ss in self.screens.iter_mut() {
            if ss.needs_feed() {
                // SAFETY: screen live; no terminal reference held here.
                unsafe { ss.feed() };
            }
        }
    }

    /// Emit state-change events to `cb` (upstream `notify`). Reads only internal, already-accumulated
    /// searcher state — no lock and no screen access needed (hence safe).
    pub(in crate::terminal) fn notify(&mut self, cb: &mut dyn FnMut(Event<'_>)) {
        let key = self.last_screen.key;
        // Snapshot everything from the active screen searcher up front, releasing the borrow before
        // the mutations / callbacks below.
        let (total, sel_idx, sel_flattened) = match self.screens.get(key) {
            None => return,
            Some(ss) => (ss.matches_len(), ss.selected_index(), ss.selected_match()),
        };

        // Total matches.
        if Some(total) != self.last_screen.total {
            self.last_screen.total = Some(total);
            cb(Event::TotalMatches(total));
        }

        // Viewport matches. Clear the stale flag first: even a failed/empty drain requires a re-feed
        // to re-search, and the feed makes it stale again.
        if self.stale_viewport_matches {
            self.stale_viewport_matches = false;
            let mut results = Vec::new();
            while let Some(hl) = self.viewport.next() {
                results.push(hl);
            }
            cb(Event::ViewportMatches(&results));
        }

        // Selected match.
        match sel_idx {
            Some(idx) => {
                // A selection exists, but its index may be out of range after a re-search; in that
                // case (`sel_flattened` is `None`) do nothing — and crucially do not clear it.
                if let Some(flattened) = sel_flattened {
                    let untracked = flattened.untracked();
                    let unchanged = matches!(
                        &self.last_screen.selected,
                        Some(prev) if prev.idx == idx && prev.highlight == untracked
                    );
                    if !unchanged {
                        self.last_screen.selected = Some(SelectedMatch {
                            idx,
                            highlight: untracked,
                        });
                        cb(Event::SelectedMatch(Some(EventSelectedMatch {
                            idx,
                            highlight: flattened,
                        })));
                    }
                }
            }
            None => {
                if self.last_screen.selected.is_some() {
                    self.last_screen.selected = None;
                    cb(Event::SelectedMatch(None));
                }
            }
        }

        // Completion (emitted at most once).
        if !self.last_complete && self.is_complete() {
            self.last_complete = true;
            cb(Event::Complete);
        }
    }
}

/// Messages the search thread accepts (upstream `Thread.Message`).
pub(in crate::terminal) enum Message {
    /// Change the search term (start / restart / stop on empty).
    ChangeNeedle(MessageData<'static, u8, 255>),
    /// Select (and scroll to) a search result.
    Select(Select),
}

/// The mailbox for sending the search thread messages (upstream `Mailbox = BlockingQueue(Message,
/// 64)`). Held behind an `Arc` so a producer's handle stays valid once the `Thread` is spawned.
pub(in crate::terminal) type Mailbox = BlockingQueue<Message, 64>;

/// The event callback (upstream `EventCallback` fn-ptr + opaque userdata) as a boxed closure. `Send`
/// is required once the thread is spawned (Exp 615).
pub(in crate::terminal) type EventCallback = Box<dyn FnMut(Event<'_>) + Send>;

/// Embedder-supplied configuration (upstream `Thread.Options`). The lock and terminal are raw
/// pointers to embedder-owned state, accessed only while the lock is held.
pub(in crate::terminal) struct Options {
    /// Guards all access to `terminal`.
    pub(in crate::terminal) lock: NonNull<Mutex<()>>,
    /// The terminal to search.
    pub(in crate::terminal) terminal: NonNull<Terminal>,
    /// Optional event callback.
    pub(in crate::terminal) event_cb: Option<EventCallback>,
}

/// Shared wakeup / stop control between the producer (`ThreadHandle`) and the spawned thread
/// (replacing upstream's `xev.Async` wakeup/stop). The `pending` flag makes `wait` predicate-based
/// so a posted message is handled immediately, not only on the next refresh tick.
struct Control {
    stop: AtomicBool,
    pending: Mutex<bool>,
    cv: Condvar,
}

impl Control {
    fn new() -> Control {
        Control {
            stop: AtomicBool::new(false),
            pending: Mutex::new(false),
            cv: Condvar::new(),
        }
    }

    /// Mark work pending and wake the thread (a posted message or a stop).
    fn signal(&self) {
        *self.pending.lock().unwrap() = true;
        self.cv.notify_all();
    }

    fn request_stop(&self) {
        self.stop.store(true, Ordering::Release);
        self.signal();
    }

    /// Block up to `timeout` for pending work (or the refresh tick), then consume the flag.
    fn wait(&self, timeout: Duration) {
        let mut p = self.pending.lock().unwrap();
        if !*p {
            p = self
                .cv
                .wait_timeout_while(p, timeout, |pending| !*pending)
                .unwrap()
                .0;
        }
        *p = false;
    }
}

/// The search thread (upstream `Thread`). Owns the inner `Search`, the message handlers, and — once
/// `spawn`ed — runs the std-concurrency event loop (`thread_main`).
pub(crate) struct Thread {
    mailbox: Arc<Mailbox>,
    control: Arc<Control>,
    search: Option<Search>,
    opts: Options,
}

// SAFETY: `Thread`'s raw pointers (`Options.terminal` / `lock` and the `Search`'s cached screen
// pointers) refer to embedder-owned state that — per the `spawn` contract — outlives the joined
// thread, is not moved, and is accessed (on every thread, including from the event callback) only
// while `opts.lock` is held. The `EventCallback` is itself `Send`.
unsafe impl Send for Thread {}

impl Thread {
    /// Construct the thread state (upstream `init`, minus the xev loop/async/timer — those are the
    /// `Control` + `thread_main`).
    pub(in crate::terminal) fn new(opts: Options) -> Thread {
        Thread {
            mailbox: Arc::new(Mailbox::new()),
            control: Arc::new(Control::new()),
            search: None,
            opts,
        }
    }

    /// Tear down the active search (upstream `deinit`'s `if (search) |*s| s.deinit()`). The `deinit`
    /// runs under the terminal lock (it untracks pins, mutating terminal state).
    ///
    /// # Safety
    /// The terminal (and lock) the search points into must still be live (the `Search::deinit`
    /// contract).
    pub(in crate::terminal) unsafe fn deinit(&mut self) {
        if let Some(mut s) = self.search.take() {
            // SAFETY: `lock` is live and guards `terminal`.
            let _guard = unsafe { self.opts.lock.as_ref() }.lock().unwrap();
            // SAFETY: terminal live; lock held.
            unsafe { s.deinit() };
        }
    }

    /// An owned handle to the mailbox (so producers can post messages even after the `Thread` is
    /// spawned).
    pub(in crate::terminal) fn mailbox(&self) -> Arc<Mailbox> {
        Arc::clone(&self.mailbox)
    }

    /// Drain and dispatch all pending messages (upstream `drainMailbox`).
    ///
    /// # Safety
    /// As `change_needle`.
    pub(in crate::terminal) unsafe fn drain_mailbox(&mut self) {
        while let Some(message) = self.mailbox.pop() {
            match message {
                // SAFETY: caller's contract.
                Message::ChangeNeedle(v) => unsafe { self.change_needle(v.slice()) },
                // SAFETY: caller's contract.
                Message::Select(sel) => unsafe { self.select(sel) },
            }
        }
    }

    /// Change the search term (upstream `changeNeedle`): unchanged → no-op; otherwise stop the prior
    /// search (emitting reset events), and on a non-empty needle start a new search with an initial
    /// feed under the lock.
    ///
    /// # Safety
    /// `opts.terminal` and `opts.lock` must be live; the terminal must outlive any search.
    pub(in crate::terminal) unsafe fn change_needle(&mut self, needle: &[u8]) {
        // Unchanged needle → no-op (case-insensitive).
        if let Some(s) = self.search.as_ref() {
            if s.needle().eq_ignore_ascii_case(needle) {
                return;
            }
        }
        // Stop the prior search: deinit it UNDER the lock (it untracks pins, mutating terminal
        // state), then emit the reset events AFTER releasing the lock (so callbacks cannot reenter
        // while the terminal is locked).
        if let Some(mut old) = self.search.take() {
            {
                // SAFETY: `lock` is live and guards `terminal`.
                let _guard = unsafe { self.opts.lock.as_ref() }.lock().unwrap();
                // SAFETY: terminal live; lock held.
                unsafe { old.deinit() };
            }
            if let Some(cb) = self.opts.event_cb.as_mut() {
                cb(Event::TotalMatches(0));
                cb(Event::SelectedMatch(None));
                cb(Event::ViewportMatches(&[]));
            }
        }
        if needle.is_empty() {
            return; // empty needle stops the search
        }
        let mut s = Search::new(needle);
        {
            // Initial feed under the terminal lock.
            // SAFETY: `lock` is live and guards `terminal`.
            let _guard = unsafe { self.opts.lock.as_ref() }.lock().unwrap();
            // SAFETY: terminal live; the lock is held.
            unsafe { s.feed(self.opts.terminal) };
        }
        self.search = Some(s);
    }

    /// Make/move the selection on the active screen and scroll it into view if it is not already
    /// visible (upstream `select`). Holds the terminal lock for the whole body.
    ///
    /// # Safety
    /// `opts.terminal` / `opts.lock` must be live; the terminal must outlive the search.
    pub(in crate::terminal) unsafe fn select(&mut self, sel: Select) {
        let Some(s) = self.search.as_mut() else {
            return;
        };
        let key = s.last_screen.key;
        if s.screens.get(key).is_none() {
            return;
        }

        // SAFETY: `lock` is live and guards `terminal`.
        let _guard = unsafe { self.opts.lock.as_ref() }.lock().unwrap();

        // Resolve the active screen pointer and validate the cached searcher's screen against it
        // BEFORE dereferencing the searcher's cached `NonNull<Screen>` (which may be stale if the
        // terminal dropped or replaced the screen since the last feed — the Exp 607 stale-screen
        // class). If gone or replaced, no-op.
        // SAFETY: terminal live; lock held.
        let present = unsafe { Terminal::present_screen_ptrs(self.opts.terminal) };
        let Some((_, screen_ptr)) = present.iter().find(|(k, _)| *k == key).copied() else {
            return;
        };
        if s.screens.get(key).unwrap().screen_ptr() != screen_ptr {
            return;
        }

        // Make the selection, then snapshot the flattened match (dropping the searcher borrow).
        let flattened = {
            let screen_search = s.screens.get_mut(key).unwrap();
            // SAFETY: screen live (validated above); lock held.
            unsafe { screen_search.select(sel) };
            screen_search.selected_match()
        };
        let Some(flattened) = flattened else {
            return;
        };
        s.reset_last_selected(); // re-notify the selection

        // If the match is already visible in the viewport, do nothing. The shared `&Screen` borrow
        // is scoped to this block so it ends before the `&mut Screen` scroll below.
        let already_visible = {
            // SAFETY: screen live; lock held.
            let screen = unsafe { screen_ptr.as_ref() };
            flattened
                .chunks
                .iter()
                .any(|c| screen.viewport_overlaps_chunk(c.node, c.start, c.end))
        };
        if already_visible {
            return;
        }

        // Scroll the match's start pin into view.
        let mut sp = screen_ptr;
        // SAFETY: screen live; lock held; this is the only access to it here.
        unsafe { sp.as_mut() }.scroll_to_pin(flattened.start_pin());
    }

    /// Spawn the OS thread running `thread_main`, returning a handle to post messages / stop / join
    /// (upstream: the caller spawns `threadMain`).
    ///
    /// # Safety
    /// The embedder's terminal + lock (`opts.terminal` / `opts.lock`) must outlive the thread — join
    /// it via `ThreadHandle::stop_and_join` before dropping or moving them — and all access to the
    /// terminal on every thread (including the event callback) must be under `opts.lock`.
    pub(in crate::terminal) unsafe fn spawn(self) -> ThreadHandle {
        let control = Arc::clone(&self.control);
        let mailbox = Arc::clone(&self.mailbox);
        let join = std::thread::spawn(move || {
            let mut thread = self;
            // SAFETY: the embedder's contract (as `spawn`).
            unsafe { thread.thread_main() };
            // Tear down the search (untrack pins) before `thread` drops — the terminal + lock are
            // still live per the contract (the embedder joins only after this returns).
            // SAFETY: as `spawn`.
            unsafe { thread.deinit() };
        });
        ThreadHandle {
            join: Some(join),
            control,
            mailbox,
        }
    }

    /// The thread body (upstream `threadMain_`): drain messages, make search progress, feed under the
    /// lock periodically, and emit notifications until stopped; then emit `Quit`.
    ///
    /// # Safety
    /// As `spawn`.
    unsafe fn thread_main(&mut self) {
        let mut last_refresh = Instant::now();
        loop {
            if self.control.stop.load(Ordering::Acquire) {
                while self.mailbox.pop().is_some() {} // drain remaining + quit
                break;
            }

            // Process pending messages (upstream wakeup → drainMailbox).
            // SAFETY: embedder contract.
            unsafe { self.drain_mailbox() };

            // Periodic refresh tick (upstream refresh timer, 24ms). Reset `last_refresh` on every
            // tick (even with no search) so the idle wait stays ~24ms; feed only when a search runs.
            if last_refresh.elapsed() >= REFRESH_INTERVAL {
                if self.search.is_some() {
                    // SAFETY: embedder contract; feeds under the lock.
                    unsafe { self.feed_under_lock() };
                }
                last_refresh = Instant::now();
            }

            // Notify any state changes.
            if let (Some(cb), Some(s)) = (self.opts.event_cb.as_mut(), self.search.as_mut()) {
                s.notify(cb);
            }

            // Tick the active search (compute the outcome before re-borrowing `self` to feed).
            let tick = match self.search.as_mut() {
                None => None,
                Some(s) if s.is_complete() => None,
                Some(s) => Some(s.tick()),
            };
            if matches!(tick, Some(Tick::Blocked)) {
                // SAFETY: embedder contract.
                unsafe { self.feed_under_lock() };
            }

            // Any tick outcome means active work — loop without waiting (upstream `run(.no_wait)`).
            // `None` (idle / complete) → block until a message/stop or the next refresh tick.
            if tick.is_none() {
                let until_refresh = REFRESH_INTERVAL.saturating_sub(last_refresh.elapsed());
                self.control
                    .wait(until_refresh.max(Duration::from_millis(1)));
            }
        }

        // Emit the quit event (upstream's `defer` on thread exit).
        if let Some(cb) = self.opts.event_cb.as_mut() {
            cb(Event::Quit);
        }
    }

    /// Feed the active search under the terminal lock.
    ///
    /// # Safety
    /// As `change_needle`.
    unsafe fn feed_under_lock(&mut self) {
        if let Some(s) = self.search.as_mut() {
            // SAFETY: `lock` is live and guards `terminal`.
            let _guard = unsafe { self.opts.lock.as_ref() }.lock().unwrap();
            // SAFETY: terminal live; lock held.
            unsafe { s.feed(self.opts.terminal) };
        }
    }
}

/// A handle to a spawned search thread (`Thread::spawn`): post messages, request stop, and join.
pub(crate) struct ThreadHandle {
    join: Option<JoinHandle<()>>,
    control: Arc<Control>,
    mailbox: Arc<Mailbox>,
}

impl ThreadHandle {
    /// Post a message and wake the thread (upstream: push to the mailbox + `wakeup.notify()`).
    pub(in crate::terminal) fn post(&self, message: Message) {
        self.mailbox.push(message, Timeout::Forever);
        self.control.signal();
    }

    /// Request the thread stop, then join it (upstream `stop.notify()` + the caller's join). The
    /// terminal + lock must remain live until this returns (the thread deinits the search on exit).
    pub(in crate::terminal) fn stop_and_join(&mut self) {
        self.control.request_stop();
        if let Some(j) = self.join.take() {
            let _ = j.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::screen::Screen;
    use std::ptr::NonNull;

    /// Build a `ScreenSearch` over `screen` (which must outlive it) searching for `needle`.
    fn screen_search(screen: &mut Screen, needle: &[u8]) -> ScreenSearch {
        let ptr = NonNull::from(screen);
        // SAFETY: `screen` outlives the returned search; this thread holds it exclusively.
        unsafe { ScreenSearch::new(ptr, needle) }
    }

    #[test]
    fn new_starts_empty_and_complete() {
        let search = Search::new(b"Fizz");
        // No screens → vacuously complete; nothing to tick.
        assert!(search.is_complete());
        assert_eq!(search.viewport.needle(), b"Fizz");
    }

    #[test]
    fn tick_on_empty_is_complete() {
        let mut search = Search::new(b"Fizz");
        assert_eq!(search.tick(), Tick::Complete);
    }

    #[test]
    fn tick_reports_progress_then_blocked_then_complete() {
        let mut screen = Screen::init(10, 10, None).unwrap();
        screen.set_text_lines_for_tests(&["Fizz"]);
        let ss = screen_search(&mut screen, b"Fizz");

        let mut search = Search::new(b"Fizz");
        search.screens.insert(TerminalScreenKey::Primary, ss);

        // First tick drains the active area (state History → HistoryFeed): progress.
        assert_eq!(search.tick(), Tick::Progress);
        // Next tick has nothing to do but wait for a feed: blocked.
        assert_eq!(search.tick(), Tick::Blocked);

        // Drive the screen search to completion, then the aggregate is complete.
        // SAFETY: `screen` is alive.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .search_all()
        };
        assert_eq!(search.tick(), Tick::Complete);
        assert!(search.is_complete());

        // SAFETY: `screen` is alive; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn tick_progress_dominates_blocked() {
        // Screen A: advance one tick so its next tick reports FeedRequired (blocked).
        let mut screen_a = Screen::init(10, 10, None).unwrap();
        screen_a.set_text_lines_for_tests(&["Fizz"]);
        let mut ss_a = screen_search(&mut screen_a, b"Fizz");
        let _ = ss_a.tick(); // History → HistoryFeed

        // Screen B: fresh, so its first tick reports Progressed.
        let mut screen_b = Screen::init(10, 10, None).unwrap();
        screen_b.set_text_lines_for_tests(&["Fizz"]);
        let ss_b = screen_search(&mut screen_b, b"Fizz");

        let mut search = Search::new(b"Fizz");
        search.screens.insert(TerminalScreenKey::Primary, ss_a);
        search.screens.insert(TerminalScreenKey::Alternate, ss_b);

        // One screen is blocked, the other progresses → progress dominates regardless of slot order.
        assert_eq!(search.tick(), Tick::Progress);

        // SAFETY: both screens are alive; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn deinit_releases_screen_search_pins() {
        let mut screen = Screen::init(10, 10, None).unwrap();
        screen.set_text_lines_for_tests(&["Fizz"]);
        let baseline = screen.tracked_pin_count();

        let ss = screen_search(&mut screen, b"Fizz");
        let mut search = Search::new(b"Fizz");
        // Constructing the screen search stood up a history searcher (two tracked pins).
        assert_eq!(screen.tracked_pin_count(), baseline + 2);
        search.screens.insert(TerminalScreenKey::Primary, ss);

        // SAFETY: `screen` is alive; called once.
        unsafe { search.deinit() };
        assert_eq!(screen.tracked_pin_count(), baseline);
    }

    #[test]
    fn insert_returns_replaced_searcher() {
        let mut screen = Screen::init(10, 10, None).unwrap();
        screen.set_text_lines_for_tests(&["Fizz"]);

        let mut map = ScreenSearches::default();
        assert!(map
            .insert(
                TerminalScreenKey::Primary,
                screen_search(&mut screen, b"Fizz")
            )
            .is_none());
        // A second insert returns the prior searcher so the caller can deinit it.
        let replaced = map.insert(
            TerminalScreenKey::Primary,
            screen_search(&mut screen, b"Fizz"),
        );
        assert!(replaced.is_some());

        // Deinit both the replaced and the resident searcher to release their pins.
        let mut replaced = replaced.unwrap();
        // SAFETY: `screen` is alive.
        unsafe { replaced.deinit() };
        let mut resident = map.take(TerminalScreenKey::Primary).unwrap();
        // SAFETY: `screen` is alive.
        unsafe { resident.deinit() };
    }

    use crate::terminal::terminal::Terminal;

    /// The primary screen's current tracked-pin count, read through the terminal's raw screen
    /// pointer (the screens are private to `terminal.rs`).
    fn primary_pin_count(t: NonNull<Terminal>) -> usize {
        // SAFETY: `t` is live; the primary screen is always present.
        let ptr = unsafe { Terminal::present_screen_ptrs(t) }[0].1;
        // SAFETY: `ptr` points at the live primary screen.
        unsafe { ptr.as_ref() }.tracked_pin_count()
    }

    #[test]
    fn feed_adds_a_searcher_for_the_active_screen() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();

        let mut search = Search::new(b"Fizz");
        // SAFETY: `terminal` outlives `search`; exclusive access.
        unsafe { search.feed(t) };

        // A searcher exists for the active (Primary) screen, pointing at the terminal's screen.
        // SAFETY: `t` live.
        let present = unsafe { Terminal::present_screen_ptrs(t) };
        let primary_ptr = present
            .iter()
            .find(|(k, _)| *k == TerminalScreenKey::Primary)
            .unwrap()
            .1;
        let ss = search.screens.get(TerminalScreenKey::Primary).unwrap();
        assert_eq!(ss.screen_ptr(), primary_ptr);

        // It finds the needle.
        // SAFETY: the screen is live.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .search_all()
        };
        assert!(
            search
                .screens
                .get(TerminalScreenKey::Primary)
                .unwrap()
                .matches_len()
                >= 1
        );

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn feed_is_idempotent_for_unchanged_screens() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();

        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };
        let after_first = primary_pin_count(t);
        // A second feed neither duplicates the searcher nor tracks new pins.
        // SAFETY: as above.
        unsafe { search.feed(t) };
        assert_eq!(primary_pin_count(t), after_first);
        assert!(search.screens.get(TerminalScreenKey::Primary).is_some());

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn feed_clears_viewport_dirty_flag() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();

        let mut search = Search::new(b"Fizz");
        // SAFETY: `t` live.
        unsafe { Terminal::mark_search_viewport_dirty(t) };
        // SAFETY: `t` live.
        assert!(unsafe { Terminal::search_viewport_dirty(t) });

        // SAFETY: as above.
        unsafe { search.feed(t) };
        // The dirty handler cleared the flag and marked the viewport matches stale.
        // SAFETY: `t` live.
        assert!(!unsafe { Terminal::search_viewport_dirty(t) });
        assert!(search.stale_viewport_matches);

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn feed_then_deinit_releases_pins() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let baseline = primary_pin_count(t);

        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };
        assert!(primary_pin_count(t) > baseline);

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
        assert_eq!(primary_pin_count(t), baseline);
    }

    #[test]
    fn feed_drops_searcher_when_alternate_screen_goes_away() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        // Enter the alternate screen (mode 1049): the terminal now has an alternate screen.
        terminal.next_slice(b"\x1b[?1049h").unwrap();
        terminal.next_slice(b"Fizz").unwrap();

        let mut search = Search::new(b"Fizz");
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        // SAFETY: `terminal` outlives `search`.
        unsafe { search.feed(t) };
        assert!(search.screens.get(TerminalScreenKey::Alternate).is_some());

        // A full reset (RIS) returns to the primary screen and DROPS the alternate, dangling the
        // alternate searcher's cached pointer.
        terminal.next_slice(b"\x1bc").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        // Reconciliation drops the now-stale alternate searcher WITHOUT deref (no use-after-free):
        // its backing screen — and pin storage — was freed by the reset.
        // SAFETY: `terminal` live.
        unsafe { search.feed(t) };
        assert!(search.screens.get(TerminalScreenKey::Alternate).is_none());
        // The active primary screen keeps (or is freshly given) a searcher.
        assert!(search.screens.get(TerminalScreenKey::Primary).is_some());

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    /// A simplified, owned snapshot of an `Event` for assertions (avoids the borrow lifetime).
    #[derive(Debug, PartialEq)]
    enum Ev {
        Quit,
        Complete,
        Total(usize),
        Selected(Option<usize>),
        Viewport(usize),
    }

    /// Run `notify` and collect the emitted events.
    fn collect(search: &mut Search) -> Vec<Ev> {
        let mut evs = Vec::new();
        let mut cb = |e: Event<'_>| {
            evs.push(match e {
                Event::Quit => Ev::Quit,
                Event::Complete => Ev::Complete,
                Event::TotalMatches(n) => Ev::Total(n),
                Event::SelectedMatch(Some(m)) => Ev::Selected(Some(m.idx)),
                Event::SelectedMatch(None) => Ev::Selected(None),
                Event::ViewportMatches(v) => Ev::Viewport(v.len()),
            });
        };
        search.notify(&mut cb);
        evs
    }

    #[test]
    fn notify_emits_total_and_viewport_matches() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let mut search = Search::new(b"Fizz");
        // SAFETY: `terminal` outlives `search`.
        unsafe { search.feed(t) };

        let evs = collect(&mut search);
        assert!(evs.iter().any(|e| matches!(e, Ev::Total(n) if *n == 2)));
        assert!(evs.iter().any(|e| matches!(e, Ev::Viewport(n) if *n == 2)));

        // Nothing changed → a second notify emits no total / viewport events.
        let evs2 = collect(&mut search);
        assert!(!evs2
            .iter()
            .any(|e| matches!(e, Ev::Total(_) | Ev::Viewport(_))));

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn notify_emits_complete_once() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };

        // Drive the active searcher to completion.
        // SAFETY: the screen is live.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .search_all()
        };

        let evs = collect(&mut search);
        assert!(evs.contains(&Ev::Complete));
        // `Complete` is emitted at most once.
        let evs2 = collect(&mut search);
        assert!(!evs2.contains(&Ev::Complete));

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn notify_emits_and_dedups_selected_match() {
        use super::super::screen::Select;
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };

        // Select a match on the active searcher.
        // SAFETY: the screen is live.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .select(Select::Next)
        };

        let evs = collect(&mut search);
        assert!(evs.iter().any(|e| matches!(e, Ev::Selected(Some(_)))));
        // An unchanged selection is not re-emitted.
        let evs2 = collect(&mut search);
        assert!(!evs2.iter().any(|e| matches!(e, Ev::Selected(_))));

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn notify_clears_selection() {
        use super::super::screen::Select;
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };
        // SAFETY: the screen is live.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .select(Select::Next)
        };
        // First notify records the selection.
        let _ = collect(&mut search);

        // Clear the selection; the next notify reports it cleared.
        search
            .screens
            .get_mut(TerminalScreenKey::Primary)
            .unwrap()
            .clear_selection_for_tests();
        let evs = collect(&mut search);
        assert!(evs.contains(&Ev::Selected(None)));

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn notify_with_out_of_range_selection_does_not_clear() {
        use super::super::screen::Select;
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let mut search = Search::new(b"Fizz");
        // SAFETY: as above.
        unsafe { search.feed(t) };
        // SAFETY: the screen is live.
        unsafe {
            search
                .screens
                .get_mut(TerminalScreenKey::Primary)
                .unwrap()
                .select(Select::Next)
        };
        // First notify records the selection.
        let _ = collect(&mut search);

        // Force the selection index out of range: `selected_index()` is `Some`, but
        // `selected_match()` is `None`. `notify` must emit nothing and NOT clear the prior selection.
        search
            .screens
            .get_mut(TerminalScreenKey::Primary)
            .unwrap()
            .set_selected_idx_for_tests(usize::MAX);
        let evs = collect(&mut search);
        assert!(!evs.iter().any(|e| matches!(e, Ev::Selected(_))));

        // SAFETY: `terminal` live; called once.
        unsafe { search.deinit() };
    }

    #[test]
    fn notify_with_no_active_screen_searcher_is_a_noop() {
        // No `feed`, so the aggregator has no searcher for the active key.
        let mut search = Search::new(b"Fizz");
        let evs = collect(&mut search);
        assert!(evs.is_empty());
    }

    // --- Thread foundation (Exp 613) ---

    use crate::terminal::blocking_queue::Timeout;

    fn ev_of(e: Event<'_>) -> Ev {
        match e {
            Event::Quit => Ev::Quit,
            Event::Complete => Ev::Complete,
            Event::TotalMatches(n) => Ev::Total(n),
            Event::SelectedMatch(Some(m)) => Ev::Selected(Some(m.idx)),
            Event::SelectedMatch(None) => Ev::Selected(None),
            Event::ViewportMatches(v) => Ev::Viewport(v.len()),
        }
    }

    /// An event-collecting callback writing into `events`.
    fn collecting_cb(events: &Arc<Mutex<Vec<Ev>>>) -> EventCallback {
        let ev = Arc::clone(events);
        Box::new(move |e| ev.lock().unwrap().push(ev_of(e)))
    }

    fn thread_opts(
        terminal: &mut Terminal,
        lock: &Mutex<()>,
        cb: Option<EventCallback>,
    ) -> Options {
        Options {
            lock: NonNull::from(lock),
            terminal: NonNull::new(core::ptr::addr_of_mut!(*terminal)).unwrap(),
            event_cb: cb,
        }
    }

    #[test]
    fn change_needle_starts_a_search_and_feeds() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // SAFETY: `terminal` and `lock` outlive `thread`.
        unsafe { thread.change_needle(b"Fizz") };
        let s = thread.search.as_ref().expect("a search was started");
        // The initial feed ran under the lock and populated the active screen searcher.
        assert!(
            s.screens
                .get(TerminalScreenKey::Primary)
                .unwrap()
                .matches_len()
                >= 1
        );

        // SAFETY: terminal live; called once.
        unsafe { thread.deinit() };
    }

    #[test]
    fn change_needle_unchanged_is_a_noop() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let lock = Mutex::new(());
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut thread = Thread::new(thread_opts(
            &mut terminal,
            &lock,
            Some(collecting_cb(&events)),
        ));

        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        events.lock().unwrap().clear();
        // A same-needle (any case) change is a no-op: no reset events, search retained.
        // SAFETY: as above.
        unsafe { thread.change_needle(b"FIZZ") };
        assert!(events.lock().unwrap().is_empty());
        assert!(thread.search.is_some());

        // SAFETY: as above.
        unsafe { thread.deinit() };
    }

    #[test]
    fn change_needle_empty_stops_the_search() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let lock = Mutex::new(());
        let events = Arc::new(Mutex::new(Vec::new()));
        let mut thread = Thread::new(thread_opts(
            &mut terminal,
            &lock,
            Some(collecting_cb(&events)),
        ));

        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        events.lock().unwrap().clear();
        // An empty needle stops the search and emits the three reset events.
        // SAFETY: as above.
        unsafe { thread.change_needle(b"") };
        assert!(thread.search.is_none());
        let collected = events.lock().unwrap();
        assert!(collected.contains(&Ev::Total(0)));
        assert!(collected.contains(&Ev::Selected(None)));
        assert!(collected.contains(&Ev::Viewport(0)));
        drop(collected);

        // SAFETY: as above (search is already None, so this is a no-op).
        unsafe { thread.deinit() };
    }

    #[test]
    fn drain_mailbox_processes_change_needle() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        thread.mailbox().push(
            Message::ChangeNeedle(MessageData::init(b"Fizz")),
            Timeout::Instant,
        );
        // SAFETY: as above.
        unsafe { thread.drain_mailbox() };
        assert!(thread.search.is_some());

        // SAFETY: as above.
        unsafe { thread.deinit() };
    }

    #[test]
    fn deinit_releases_the_search() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let lock = Mutex::new(());
        let t = NonNull::new(core::ptr::addr_of_mut!(terminal)).unwrap();
        let baseline = primary_pin_count(t);

        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));
        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        assert!(primary_pin_count(t) > baseline);

        // SAFETY: as above; called once.
        unsafe { thread.deinit() };
        assert_eq!(primary_pin_count(t), baseline);
    }

    // --- Thread `select` handler (Exp 614) ---

    #[test]
    fn select_makes_a_selection() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        // SAFETY: as above.
        unsafe { thread.select(Select::Next) };

        let ss = thread
            .search
            .as_ref()
            .unwrap()
            .screens
            .get(TerminalScreenKey::Primary)
            .unwrap();
        assert!(ss.selected_index().is_some());

        // SAFETY: as above.
        unsafe { thread.deinit() };
    }

    #[test]
    fn select_emits_selection_on_next_notify() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        // SAFETY: as above.
        unsafe { thread.select(Select::Next) };

        // After `select` reset the notify cache, the next `notify` re-emits the selection.
        let mut got = Vec::new();
        {
            let mut cb = |e: Event<'_>| got.push(ev_of(e));
            thread.search.as_mut().unwrap().notify(&mut cb);
        }
        assert!(got.iter().any(|e| matches!(e, Ev::Selected(Some(_)))));

        // SAFETY: as above.
        unsafe { thread.deinit() };
    }

    #[test]
    fn select_with_no_search_is_a_noop() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // No search yet → `select` returns without panicking.
        // SAFETY: as above.
        unsafe { thread.select(Select::Next) };
        assert!(thread.search.is_none());
    }

    #[test]
    fn drain_mailbox_dispatches_select() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // SAFETY: as above.
        unsafe { thread.change_needle(b"Fizz") };
        thread
            .mailbox()
            .push(Message::Select(Select::Next), Timeout::Instant);
        // SAFETY: as above.
        unsafe { thread.drain_mailbox() };

        let ss = thread
            .search
            .as_ref()
            .unwrap()
            .screens
            .get(TerminalScreenKey::Primary)
            .unwrap();
        assert!(ss.selected_index().is_some());

        // SAFETY: as above.
        unsafe { thread.deinit() };
    }

    #[test]
    fn select_no_ops_for_a_dropped_screen() {
        let mut terminal = Terminal::init(10, 10, None).unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        // Enter the alternate screen, so a search there targets `last_screen.key = Alternate`.
        terminal.next_slice(b"\x1b[?1049h").unwrap();
        terminal.next_slice(b"Fizz").unwrap();
        let lock = Mutex::new(());
        let mut thread = Thread::new(thread_opts(&mut terminal, &lock, None));

        // SAFETY: as above. Feeds while the alternate is active → `last_screen.key = Alternate`.
        unsafe { thread.change_needle(b"Fizz") };
        assert!(thread
            .search
            .as_ref()
            .unwrap()
            .screens
            .get(TerminalScreenKey::Alternate)
            .is_some());

        // A full reset (RIS) drops the alternate screen, leaving the alternate searcher's cached
        // pointer stale.
        terminal.next_slice(b"\x1bc").unwrap();

        // `select` resolves the present screens first and finds no alternate, so it no-ops WITHOUT
        // dereferencing the stale cached screen pointer (no use-after-free).
        // SAFETY: as above.
        unsafe { thread.select(Select::Next) };

        // Intentionally no `deinit`: the stale searchers drop without dereferencing their freed
        // screens (the search structs have no `Drop` that touches the screen).
    }

    // --- Spawned thread (Exp 615) ---

    /// Build `Options` over a leaked `Terminal` + `Mutex<()>` (so they outlive the spawned thread),
    /// returning the options and a raw terminal pointer for assertions.
    fn spawn_opts(
        text: &[u8],
        cb: Option<EventCallback>,
    ) -> (Options, NonNull<Terminal>, NonNull<Mutex<()>>) {
        let terminal: &'static mut Terminal =
            Box::leak(Box::new(Terminal::init(10, 10, None).unwrap()));
        terminal.next_slice(text).unwrap();
        let t_ptr = NonNull::new(terminal as *mut Terminal).unwrap();
        let lock: &'static Mutex<()> = Box::leak(Box::new(Mutex::new(())));
        let lock_ptr = NonNull::from(&*lock);
        (
            Options {
                lock: lock_ptr,
                terminal: t_ptr,
                event_cb: cb,
            },
            t_ptr,
            lock_ptr,
        )
    }

    /// A `(done, events)` pair and a callback that records every event and signals `done` on
    /// `Complete`.
    type Recorder = (Arc<(Mutex<bool>, Condvar)>, Arc<Mutex<Vec<Ev>>>);
    fn signalling_cb() -> (EventCallback, Recorder) {
        let done = Arc::new((Mutex::new(false), Condvar::new()));
        let events = Arc::new(Mutex::new(Vec::new()));
        let cb_done = Arc::clone(&done);
        let cb_events = Arc::clone(&events);
        let cb: EventCallback = Box::new(move |e: Event<'_>| {
            let ev = ev_of(e);
            if ev == Ev::Complete {
                let mut d = cb_done.0.lock().unwrap();
                *d = true;
                cb_done.1.notify_all();
            }
            cb_events.lock().unwrap().push(ev);
        });
        (cb, (done, events))
    }

    /// Wait up to 5s for `done`.
    fn wait_done(done: &Arc<(Mutex<bool>, Condvar)>) -> bool {
        let (m, cv) = &**done;
        let mut d = m.lock().unwrap();
        let start = Instant::now();
        while !*d && start.elapsed() < Duration::from_secs(5) {
            d = cv.wait_timeout(d, Duration::from_millis(50)).unwrap().0;
        }
        *d
    }

    #[test]
    fn spawn_searches_and_emits_complete() {
        let (cb, (done, events)) = signalling_cb();
        let (opts, _t, _l) = spawn_opts(b"Fizz Fizz", Some(cb));
        let thread = Thread::new(opts);
        // SAFETY: the leaked terminal + lock outlive the thread; joined before this test returns.
        let mut handle = unsafe { thread.spawn() };

        handle.post(Message::ChangeNeedle(MessageData::init(b"Fizz")));
        assert!(
            wait_done(&done),
            "search did not complete within the timeout"
        );

        handle.stop_and_join();
        let evs = events.lock().unwrap();
        assert!(evs.contains(&Ev::Complete));
        assert!(evs.iter().any(|e| matches!(e, Ev::Total(_))));
        assert!(evs.contains(&Ev::Quit));
    }

    #[test]
    fn spawn_then_stop_emits_quit() {
        let (cb, (_done, events)) = signalling_cb();
        let (opts, _t, _l) = spawn_opts(b"Fizz", Some(cb));
        let thread = Thread::new(opts);
        // SAFETY: as above.
        let mut handle = unsafe { thread.spawn() };

        // Stop immediately, with no search started.
        handle.stop_and_join();
        assert!(events.lock().unwrap().contains(&Ev::Quit));
    }

    #[test]
    fn post_select_after_search_is_a_smoke_test() {
        let (cb, (done, _events)) = signalling_cb();
        let (opts, _t, _l) = spawn_opts(b"Fizz Fizz", Some(cb));
        let thread = Thread::new(opts);
        // SAFETY: as above.
        let mut handle = unsafe { thread.spawn() };

        handle.post(Message::ChangeNeedle(MessageData::init(b"Fizz")));
        assert!(
            wait_done(&done),
            "search did not complete within the timeout"
        );
        // Posting a select should not panic or deadlock.
        handle.post(Message::Select(Select::Next));
        std::thread::sleep(Duration::from_millis(50));

        handle.stop_and_join();
    }
}
