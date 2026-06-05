//! A pool of values handed out by index, returned in order, that grows on demand (port of
//! upstream `datastruct/segmented_pool`).
//!
//! Upstream built this for libuv write requests, which must have a stable pointer and are
//! processed in order on a single stream. roastty hands out indices instead of raw pointers —
//! indices are stable across a `Vec` reallocation by construction — so a plain `Vec<T>` backs
//! the pool. The cyclic-ring handout, the `available` count, and the doubling growth match
//! upstream exactly.

/// Returned by `get` when the pool is exhausted (upstream `error.OutOfValues`).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct OutOfValues;

/// A pool of `T` whose slots are handed out by index and returned in order, growing (by
/// doubling) on demand. `prealloc` is the initial slot count (must be >= 1).
pub(crate) struct SegmentedPool<T> {
    items: Vec<T>,
    i: usize,         // wrapping cursor; the next slot is `i % items.len()`
    available: usize, // slots that can still be handed out before `OutOfValues`
}

impl<T: Default> SegmentedPool<T> {
    /// Create a pool with `prealloc` default-initialized slots (`prealloc` must be >= 1).
    pub(crate) fn new(prealloc: usize) -> Self {
        assert!(prealloc >= 1, "SegmentedPool prealloc must be >= 1");
        Self {
            items: (0..prealloc).map(|_| T::default()).collect(),
            i: 0,
            available: prealloc,
        }
    }

    /// Get the index of the next available slot without growing (upstream `get`).
    pub(crate) fn get(&mut self) -> Result<usize, OutOfValues> {
        if self.available == 0 {
            return Err(OutOfValues);
        }
        let idx = self.i % self.items.len();
        self.i = self.i.wrapping_add(1);
        self.available -= 1;
        Ok(idx)
    }

    /// Get the next available slot index, growing the pool if exhausted (upstream `getGrow`).
    pub(crate) fn get_grow(&mut self) -> usize {
        if self.available == 0 {
            self.grow();
        }
        self.get().expect("grow guarantees availability")
    }

    /// Put a slot back. Caller must return slots in `get` order (upstream `put`).
    ///
    /// Slots are handles, not pointers: a caller stores the index from `get` / `get_grow` and
    /// re-resolves it with `at` / `at_mut`. A borrow from `at_mut` must not be held across a
    /// `get_grow` that grows (safe Rust enforces this naturally), but indices stay valid.
    pub(crate) fn put(&mut self) {
        self.available += 1;
        // Upstream's `inlineAssert` traps in release too, so use `assert!` (not `debug_assert!`):
        // an extra `put` past capacity would corrupt later handout, and this is the only guard on
        // the in-order return contract.
        assert!(self.available <= self.items.len());
    }

    /// Borrow a slot by index (handed out by `get` / `get_grow`).
    pub(crate) fn at(&self, idx: usize) -> &T {
        &self.items[idx]
    }

    /// Mutably borrow a slot by index (handed out by `get` / `get_grow`).
    pub(crate) fn at_mut(&mut self, idx: usize) -> &mut T {
        &mut self.items[idx]
    }

    /// Double the slot count, handing out the fresh half first (upstream `grow`).
    fn grow(&mut self) {
        let old_len = self.items.len();
        self.items.extend((0..old_len).map(|_| T::default()));
        self.i = old_len;
        self.available = old_len;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_out_distinct_slots_until_exhausted() {
        let mut pool: SegmentedPool<u8> = SegmentedPool::new(2);
        let a = pool.get().unwrap();
        let b = pool.get().unwrap();
        assert_ne!(a, b);
        assert_eq!(pool.get(), Err(OutOfValues));
    }

    #[test]
    fn put_back_returns_same_slot_in_order() {
        let mut pool: SegmentedPool<u8> = SegmentedPool::new(2);
        let a = pool.get().unwrap();
        pool.get().unwrap();
        assert_eq!(pool.get(), Err(OutOfValues));

        // Write into the first slot, put it back, and confirm the next get hands it back intact.
        *pool.at_mut(a) = 42;
        pool.put();
        let temp = pool.get().unwrap();
        assert_eq!(temp, a);
        assert_eq!(*pool.at(temp), 42);
        assert_eq!(pool.get(), Err(OutOfValues));
    }

    #[test]
    fn grows_handing_out_the_fresh_half_first() {
        let mut pool: SegmentedPool<u8> = SegmentedPool::new(2);
        let a = pool.get().unwrap();
        let b = pool.get().unwrap();
        assert_eq!(pool.get(), Err(OutOfValues));

        // get_grow doubles to length 4 and hands out a fresh slot (distinct from the first two).
        let c = pool.get_grow();
        assert_ne!(c, a);
        assert_ne!(c, b);

        // One more fresh slot, then exhausted again.
        let _d = pool.get().unwrap();
        assert_eq!(pool.get(), Err(OutOfValues));

        // After a put, the cursor wraps back to the first (old) slot.
        pool.put();
        assert_eq!(pool.get().unwrap(), a);
        assert_eq!(pool.get(), Err(OutOfValues));
    }

    #[test]
    fn tracks_capacity_and_availability_across_grow() {
        let mut pool: SegmentedPool<u8> = SegmentedPool::new(2);
        assert_eq!(pool.items.len(), 2);
        pool.get().unwrap();
        pool.get().unwrap();
        pool.get_grow(); // grows to 4, hands out one
        assert_eq!(pool.items.len(), 4);
        // After growing (available reset to old len 2) and one get, one remains.
        assert_eq!(pool.available, 1);
    }

    #[test]
    #[should_panic(expected = "available")]
    fn over_put_panics() {
        let mut pool: SegmentedPool<u8> = SegmentedPool::new(2);
        // Nothing has been handed out, so a put would push `available` past the slot count.
        pool.put();
    }
}
