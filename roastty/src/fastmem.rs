//! Single-step slice rotations (port of the rotation helpers in upstream `fastmem`).
//!
//! Upstream also wraps libc `memmove` / `memcpy` (`move` / `copy`) purely to prefer them over the
//! Zig builtins for speed. Rust's `slice::copy_within` / `copy_from_slice` already lower to those
//! intrinsics, so only the rotation helpers are ported here.

/// Moves the first item to the end: `0 1 2 3` → `1 2 3 0` (upstream `rotateOnce`). The slice must
/// be non-empty.
pub(crate) fn rotate_once<T>(items: &mut [T]) {
    items.rotate_left(1);
}

/// Moves the last item to the start: `0 1 2 3` → `3 0 1 2` (upstream `rotateOnceR`). The slice must
/// be non-empty.
pub(crate) fn rotate_once_r<T>(items: &mut [T]) {
    items.rotate_right(1);
}

/// Rotates `item` in at the end, returning the displaced first item: rotating `4` into `0 1 2 3`
/// gives `1 2 3 4` and returns `0` (upstream `rotateIn`). The slice must be non-empty.
pub(crate) fn rotate_in<T>(items: &mut [T], item: T) -> T {
    // Put `item` at the front, take the old first out, then rotate it to the end.
    let removed = std::mem::replace(&mut items[0], item);
    items.rotate_left(1);
    removed
}

/// Rotates `item` in at the start, returning the displaced last item: rotating `4` into `0 1 2 3`
/// gives `4 0 1 2` and returns `3` (upstream `rotateInR`). The slice must be non-empty.
pub(crate) fn rotate_in_r<T>(items: &mut [T], item: T) -> T {
    // Put `item` at the back, take the old last out, then rotate it to the front.
    let n = items.len();
    let removed = std::mem::replace(&mut items[n - 1], item);
    items.rotate_right(1);
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_once_moves_first_to_end() {
        let mut items = [0, 1, 2, 3];
        rotate_once(&mut items);
        assert_eq!(items, [1, 2, 3, 0]);
    }

    #[test]
    fn rotate_once_r_moves_last_to_start() {
        let mut items = [0, 1, 2, 3];
        rotate_once_r(&mut items);
        assert_eq!(items, [3, 0, 1, 2]);
    }

    #[test]
    fn rotate_in_appends_and_returns_first() {
        let mut items = [0, 1, 2, 3];
        let removed = rotate_in(&mut items, 4);
        assert_eq!(items, [1, 2, 3, 4]);
        assert_eq!(removed, 0);
    }

    #[test]
    fn rotate_in_r_prepends_and_returns_last() {
        let mut items = [0, 1, 2, 3];
        let removed = rotate_in_r(&mut items, 4);
        assert_eq!(items, [4, 0, 1, 2]);
        assert_eq!(removed, 3);
    }

    #[test]
    fn rotate_once_and_r_are_inverses() {
        let original = [10, 20, 30, 40, 50];

        let mut items = original;
        rotate_once(&mut items);
        rotate_once_r(&mut items);
        assert_eq!(items, original);

        let mut items = original;
        rotate_once_r(&mut items);
        rotate_once(&mut items);
        assert_eq!(items, original);
    }

    #[test]
    fn single_element_is_identity() {
        let mut items = [7];
        rotate_once(&mut items);
        assert_eq!(items, [7]);
        rotate_once_r(&mut items);
        assert_eq!(items, [7]);

        let mut items = [7];
        assert_eq!(rotate_in(&mut items, 9), 7);
        assert_eq!(items, [9]);

        let mut items = [7];
        assert_eq!(rotate_in_r(&mut items, 9), 7);
        assert_eq!(items, [9]);
    }

    #[test]
    fn works_for_non_copy_elements() {
        // The generic (non-`Copy`) surface: rotate owned `String`s.
        let mut items = [String::from("a"), String::from("b"), String::from("c")];
        rotate_once(&mut items);
        assert_eq!(items, ["b", "c", "a"]);

        let removed = rotate_in(&mut items, String::from("d"));
        assert_eq!(items, ["c", "a", "d"]);
        assert_eq!(removed, "b");
    }
}
