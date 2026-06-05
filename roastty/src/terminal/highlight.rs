//! Highlights are contiguous ranges of cells that should be called out,
//! most commonly for selection, search results, or semantic terminal regions.
//!
//! Within the terminal package, a highlight is a generic range over cells.

use std::ptr::NonNull;

use super::page_list::{Node, Pin};
use super::screen::Screen;
use super::size::CellCountInt;

/// An untracked highlight stores its highlighted area as start and end screen
/// pins. Since it is untracked, the pins are only valid for the current
/// terminal state and may not be safe after terminal mutations.
///
/// To simplify operations, `start` must be before or equal to `end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Untracked {
    pub(super) start: Pin,
    pub(super) end: Pin,
}

impl Untracked {
    /// Register this highlight's pins with the screen so they survive terminal mutations (upstream
    /// `track`). `None` if a pin can't be tracked.
    pub(super) fn track(&self, screen: &mut Screen) -> Option<Tracked> {
        Tracked::init(screen, self.start, self.end)
    }
}

/// A tracked highlight stores its highlighted area as tracked screen pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Tracked {
    pub(super) start: NonNull<Pin>,
    pub(super) end: NonNull<Pin>,
}

impl Tracked {
    pub(super) fn init_assume(start: NonNull<Pin>, end: NonNull<Pin>) -> Self {
        Self { start, end }
    }

    /// Track `start` and `end` on `screen`, returning the tracked highlight (upstream
    /// `Tracked.init`). `None` if either pin is invalid; the first is untracked if the second fails.
    pub(super) fn init(screen: &mut Screen, start: Pin, end: Pin) -> Option<Tracked> {
        let start_tracked = screen.track_pin(start)?;
        let Some(end_tracked) = screen.track_pin(end) else {
            screen.untrack_pin(start_tracked);
            return None;
        };
        Some(Tracked {
            start: start_tracked,
            end: end_tracked,
        })
    }

    /// Untrack both pins (upstream `deinit`). Takes `self` by value (a lifecycle signal, matching
    /// upstream; `Tracked` is `Copy`). Do not call on a `Tracked` made via `init_assume`.
    pub(super) fn deinit(self, screen: &mut Screen) {
        screen.untrack_pin(self.start);
        screen.untrack_pin(self.end);
    }
}

/// A flattened highlight stores the highlighted area as serial-stamped page
/// chunks so callers can traverse a highlight without re-reading page-list
/// bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Flattened {
    pub(super) chunks: Vec<Chunk>,
    pub(super) top_x: CellCountInt,
    pub(super) bot_x: CellCountInt,
}

/// A flattened page chunk plus the page serial observed when the highlight was
/// created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Chunk {
    pub(super) node: NonNull<Node>,
    pub(super) serial: u64,
    pub(super) start: CellCountInt,
    pub(super) end: CellCountInt,
}

impl Default for Flattened {
    fn default() -> Self {
        Self {
            chunks: Vec::new(),
            top_x: 0,
            bot_x: 0,
        }
    }
}

impl Flattened {
    const EMPTY_PRECONDITION: &'static str = "flattened highlight must contain at least one chunk";

    pub(super) fn empty() -> Self {
        Self::default()
    }

    pub(super) fn start_pin(&self) -> Pin {
        let chunk = self.chunks.first().expect(Self::EMPTY_PRECONDITION);
        Pin::new(chunk.node, chunk.start, self.top_x)
    }

    pub(super) fn end_pin(&self) -> Pin {
        let chunk = self.chunks.last().expect(Self::EMPTY_PRECONDITION);
        Pin::new(chunk.node, chunk.end - 1, self.bot_x)
    }

    pub(super) fn untracked(&self) -> Untracked {
        Untracked {
            start: self.start_pin(),
            end: self.end_pin(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::screen::Screen;
    use super::*;

    #[test]
    fn track_and_deinit_round_trips_tracked_pin_count() {
        let mut screen = Screen::init(10, 10, None).unwrap();
        let node = screen.first_node_ptr_for_tests();
        let pin = Pin::new(node, 0, 0);
        let untracked = Untracked {
            start: pin,
            end: pin,
        };

        let baseline = screen.tracked_pin_count();
        let tracked = untracked.track(&mut screen).expect("track valid pins");
        assert_eq!(screen.tracked_pin_count(), baseline + 2);

        tracked.deinit(&mut screen);
        assert_eq!(screen.tracked_pin_count(), baseline);
    }

    #[test]
    fn track_rolls_back_on_second_pin_failure() {
        let mut screen = Screen::init(10, 10, None).unwrap();
        let node = screen.first_node_ptr_for_tests();
        let start = Pin::new(node, 0, 0);
        // An invalid end pin makes the second `track_pin` fail; the first must be untracked.
        let invalid_end = Pin::test_invalid_for_tests();

        let baseline = screen.tracked_pin_count();
        assert!(Tracked::init(&mut screen, start, invalid_end).is_none());
        assert_eq!(screen.tracked_pin_count(), baseline);
    }

    #[test]
    fn untracked_equality_is_pin_by_pin() {
        let node = NonNull::dangling();
        let a = Untracked {
            start: Pin::new(node, 0, 0),
            end: Pin::new(node, 1, 1),
        };
        let b = Untracked {
            start: Pin::new(node, 0, 0),
            end: Pin::new(node, 1, 1),
        };
        let c = Untracked {
            start: Pin::new(node, 0, 0),
            end: Pin::new(node, 2, 2),
        };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
