//! Shared ownership set for expensive [`SharedGrid`] instances.
//!
//! This is the ownership/refcount/locking foundation of upstream
//! `font/SharedGridSet.zig`. Roastty does not have the full upstream
//! config-derived font key yet, so the set is generic over the key and accepts a
//! caller-supplied grid constructor.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use crate::font::shared_grid::SharedGrid;

/// A shared grid reference returned by [`SharedGridSet::ref_grid`].
pub(crate) struct SharedGridHandle<K> {
    key: K,
    grid: Arc<Mutex<SharedGrid>>,
}

impl<K> SharedGridHandle<K> {
    pub(crate) fn key(&self) -> &K {
        &self.key
    }

    pub(crate) fn grid(&self) -> &Arc<Mutex<SharedGrid>> {
        &self.grid
    }
}

struct ReffedGrid {
    grid: Arc<Mutex<SharedGrid>>,
    refs: usize,
}

/// Result of releasing a grid reference from the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DerefResult {
    /// No grid existed for the key.
    Missing,
    /// The refcount was decremented and the grid remains cached.
    Decremented,
    /// The final reference was released and the grid was removed.
    Removed,
}

/// A keyed set of shared font grids with explicit set-owned refcounts.
pub(crate) struct SharedGridSet<K> {
    grids: Mutex<HashMap<K, ReffedGrid>>,
}

impl<K> Default for SharedGridSet<K> {
    fn default() -> Self {
        SharedGridSet {
            grids: Mutex::new(HashMap::new()),
        }
    }
}

impl<K> SharedGridSet<K>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn new() -> SharedGridSet<K> {
        SharedGridSet::default()
    }

    /// Returns the number of cached grids.
    pub(crate) fn count(&self) -> usize {
        self.grids
            .lock()
            .expect("shared grid set mutex poisoned")
            .len()
    }

    /// References the grid for `key`, constructing and caching one if needed.
    pub(crate) fn ref_grid<F>(&self, key: K, make_grid: F) -> SharedGridHandle<K>
    where
        F: FnOnce() -> SharedGrid,
    {
        let mut grids = self.grids.lock().expect("shared grid set mutex poisoned");

        if let Some(reffed) = grids.get_mut(&key) {
            reffed.refs = reffed
                .refs
                .checked_add(1)
                .expect("shared grid refcount overflow");
            return SharedGridHandle {
                key,
                grid: Arc::clone(&reffed.grid),
            };
        }

        let grid = Arc::new(Mutex::new(make_grid()));
        grids.insert(
            key.clone(),
            ReffedGrid {
                grid: Arc::clone(&grid),
                refs: 1,
            },
        );

        SharedGridHandle { key, grid }
    }

    /// Releases one reference for `key`.
    pub(crate) fn deref_grid(&self, key: &K) -> DerefResult {
        let mut grids = self.grids.lock().expect("shared grid set mutex poisoned");

        let Some(reffed) = grids.get_mut(key) else {
            return DerefResult::Missing;
        };

        if reffed.refs > 1 {
            reffed.refs -= 1;
            return DerefResult::Decremented;
        }

        grids.remove(key);
        DerefResult::Removed
    }

    #[cfg(test)]
    fn ref_count(&self, key: &K) -> Option<usize> {
        self.grids
            .lock()
            .expect("shared grid set mutex poisoned")
            .get(key)
            .map(|reffed| reffed.refs)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::font::codepoint_resolver::CodepointResolver;
    use crate::font::collection::Collection;
    use crate::font::face::coretext::Face;
    use crate::font::Style;

    fn menlo_grid() -> SharedGrid {
        let mut collection = Collection::new();
        collection
            .add(Face::new("Menlo", 32.0), Style::Regular, false)
            .unwrap();
        collection.update_metrics().unwrap();
        let metrics = *collection.metrics().unwrap();
        SharedGrid::new(CodepointResolver::new(collection), metrics)
    }

    #[test]
    fn ref_grid_reuses_cached_grid_and_counts_refs() {
        let set = SharedGridSet::new();

        let first = set.ref_grid("menlo", menlo_grid);
        assert_eq!(set.count(), 1);
        assert_eq!(set.ref_count(first.key()), Some(1));

        let second = set.ref_grid("menlo", menlo_grid);
        assert_eq!(set.count(), 1);
        assert_eq!(set.ref_count(first.key()), Some(2));
        assert!(Arc::ptr_eq(first.grid(), second.grid()));

        assert_eq!(set.deref_grid(second.key()), DerefResult::Decremented);
        assert_eq!(set.count(), 1);
        assert_eq!(set.ref_count(first.key()), Some(1));

        assert_eq!(set.deref_grid(first.key()), DerefResult::Removed);
        assert_eq!(set.count(), 0);
        assert_eq!(set.ref_count(first.key()), None);
    }

    #[test]
    fn different_keys_create_distinct_grids() {
        let set = SharedGridSet::new();

        let regular = set.ref_grid(("Menlo", 32), menlo_grid);
        let large = set.ref_grid(("Menlo", 40), menlo_grid);

        assert_eq!(set.count(), 2);
        assert_eq!(set.ref_count(regular.key()), Some(1));
        assert_eq!(set.ref_count(large.key()), Some(1));
        assert!(!Arc::ptr_eq(regular.grid(), large.grid()));
    }

    #[test]
    fn deref_missing_key_is_noop() {
        let set: SharedGridSet<&str> = SharedGridSet::new();

        assert_eq!(set.deref_grid(&"missing"), DerefResult::Missing);
        assert_eq!(set.count(), 0);
    }

    #[test]
    fn shared_handle_allows_mutable_grid_access() {
        let set = SharedGridSet::new();
        let handle = set.ref_grid("menlo", menlo_grid);

        let mut grid = handle.grid().lock().unwrap();
        let index = grid
            .get_index('A' as u32, Style::Regular, None)
            .unwrap()
            .expect("Menlo resolves A");

        assert!(grid.has_codepoint(index, 'A' as u32, None));
    }
}
