//! Foundational types for the split-pane tree (port of the vocabulary of upstream
//! `datastruct/split_tree`). The tree itself — the node arena, view ref-counting, and the
//! spatial normalization / resize logic — is deferred.

use half::f16;

/// A handle into the tree's `nodes` array (upstream `Node.Handle`): a `u16`-backed index, so nodes
/// are referenced by 16-bit handles rather than pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Handle(u16);

impl Handle {
    /// The root node's handle (index 0) (upstream `.root`).
    pub(crate) const ROOT: Handle = Handle(0);

    /// Build a handle from an index (upstream `@enumFromInt`). The full `u16` range is valid —
    /// upstream's `enum(u16)` can represent `u16::MAX`, which the tree iterator uses as an
    /// end sentinel (`@enumFromInt(handle.idx() + 1)`).
    pub(crate) fn from_index(index: usize) -> Handle {
        assert!(index <= u16::MAX as usize, "split tree handle out of range");
        Handle(index as u16)
    }

    /// The index this handle refers to (upstream `idx`).
    pub(crate) fn idx(self) -> usize {
        self.0 as usize
    }

    /// Offset the handle by `v` (upstream `offset`), asserting the result stays below `u16::MAX`
    /// (matching upstream's `final < maxInt(Backing)`).
    pub(crate) fn offset(self, v: usize) -> Handle {
        let result = (self.0 as usize)
            .checked_add(v)
            .expect("split tree handle offset overflow");
        assert!(result < u16::MAX as usize, "split tree handle overflow");
        Handle(result as u16)
    }
}

/// The orientation of a split (upstream `Split.Layout`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Layout {
    Horizontal,
    Vertical,
}

/// The direction a new view is split off in (upstream `Split.Direction`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Direction {
    Left,
    Right,
    Down,
    Up,
}

impl Direction {
    /// The split layout and whether the new view goes on the first (left / top) side, for a split
    /// in this direction (upstream `split`'s direction switch).
    pub(crate) fn split_layout(self) -> (Layout, bool) {
        match self {
            Direction::Left => (Layout::Horizontal, true),
            Direction::Right => (Layout::Horizontal, false),
            Direction::Up => (Layout::Vertical, true),
            Direction::Down => (Layout::Vertical, false),
        }
    }
}

/// The payload of a split node (upstream `Split`): two child handles, the split orientation, and
/// the fraction of space given to the first (left / top) child.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Split {
    pub(crate) layout: Layout,
    pub(crate) ratio: f16,
    pub(crate) left: Handle,
    pub(crate) right: Handle,
}

/// A node's normalized 2D rectangle in the spatial representation (upstream `Spatial.Slot`); all
/// coordinates are in a 1×1 space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Slot {
    pub(crate) x: f16,
    pub(crate) y: f16,
    pub(crate) width: f16,
    pub(crate) height: f16,
}

impl Slot {
    /// The right edge, `x + width` (upstream `maxX`).
    pub(crate) fn max_x(self) -> f16 {
        self.x + self.width
    }

    /// The bottom edge, `y + height` (upstream `maxY`).
    pub(crate) fn max_y(self) -> f16 {
        self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_root_idx_offset() {
        assert_eq!(Handle::ROOT.idx(), 0);
        assert_eq!(Handle::from_index(5).idx(), 5);
        assert_eq!(Handle::ROOT.offset(3).idx(), 3);
        assert_eq!(Handle::from_index(2).offset(4).idx(), 6);
    }

    #[test]
    fn from_index_allows_u16_max() {
        // Upstream's enum can represent u16::MAX (the iterator's end sentinel).
        assert_eq!(
            Handle::from_index(u16::MAX as usize).idx(),
            u16::MAX as usize
        );
    }

    #[test]
    #[should_panic(expected = "out of range")]
    fn from_index_above_u16_max_panics() {
        let _ = Handle::from_index(u16::MAX as usize + 1);
    }

    #[test]
    fn offset_just_below_u16_max_succeeds() {
        assert_eq!(
            Handle::ROOT.offset(u16::MAX as usize - 1).idx(),
            u16::MAX as usize - 1
        );
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn offset_reaching_u16_max_panics() {
        let _ = Handle::ROOT.offset(u16::MAX as usize);
    }

    #[test]
    fn split_layout_mapping() {
        assert_eq!(Direction::Left.split_layout(), (Layout::Horizontal, true));
        assert_eq!(Direction::Right.split_layout(), (Layout::Horizontal, false));
        assert_eq!(Direction::Up.split_layout(), (Layout::Vertical, true));
        assert_eq!(Direction::Down.split_layout(), (Layout::Vertical, false));
    }

    #[test]
    fn enum_variants_are_distinct() {
        assert_ne!(Layout::Horizontal, Layout::Vertical);
        assert_ne!(Direction::Left, Direction::Right);
        assert_ne!(Direction::Up, Direction::Down);
        assert_ne!(Direction::Left, Direction::Up);
    }

    #[test]
    fn split_fields_and_equality() {
        let split = Split {
            layout: Layout::Horizontal,
            ratio: f16::from_f32(0.5),
            left: Handle::from_index(1),
            right: Handle::from_index(2),
        };
        assert_eq!(split.layout, Layout::Horizontal);
        assert_eq!(split.left, Handle::from_index(1));
        assert_eq!(split.right, Handle::from_index(2));

        let same = split;
        assert_eq!(split, same);

        let different = Split {
            ratio: f16::from_f32(0.25),
            ..split
        };
        assert_ne!(split, different);
    }

    #[test]
    fn ratio_round_trips_through_f16() {
        let split = Split {
            layout: Layout::Vertical,
            ratio: f16::from_f32(0.5),
            left: Handle::ROOT,
            right: Handle::from_index(1),
        };
        assert_eq!(split.ratio.to_f32(), 0.5);
    }

    #[test]
    fn slot_max_x_and_max_y() {
        // Binary-exact half fractions, so there is no decimal-rounding ambiguity.
        let slot = Slot {
            x: f16::from_f32(0.25),
            y: f16::from_f32(0.125),
            width: f16::from_f32(0.5),
            height: f16::from_f32(0.25),
        };
        assert_eq!(slot.max_x(), f16::from_f32(0.75));
        assert_eq!(slot.max_y(), f16::from_f32(0.375));
        // Compare like-for-like against the explicit half addition.
        assert_eq!(slot.max_y(), f16::from_f32(0.125) + f16::from_f32(0.25));
    }
}
