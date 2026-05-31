//! Highlights are contiguous ranges of cells that should be called out,
//! most commonly for selection, search results, or semantic terminal regions.
//!
//! Within the terminal package, a highlight is a generic range over cells.

use super::page_list::Pin;

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
