//! The search-thread aggregator (port of the `Search` struct in upstream `terminal/search/Thread.zig`).
//!
//! This lands the lock-aware, multi-screen orchestration core: a `ViewportSearch` plus a
//! `ScreenSearch` per terminal screen, with `new` / `deinit` / `is_complete` / `tick`. The
//! aggregator's `feed` (terminal integration: the `search_viewport_dirty` flag and screen
//! reconciliation) and `notify` (the `Event` / `EventCallback` machinery), and the outer libxev
//! event-loop `Thread`, are deferred to later slices — roastty has no libxev port yet.

use super::super::highlight::Untracked;
use super::super::terminal::TerminalScreenKey;
use super::screen::{ScreenSearch, Tick as ScreenTick};
use super::viewport::ViewportSearch;

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
}
