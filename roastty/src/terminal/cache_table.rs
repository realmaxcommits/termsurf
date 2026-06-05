//! A fixed-bucket cache with within-bucket LRU eviction (port of upstream
//! `datastruct/cache_table`).

/// Provides hashing and equality for a `CacheTable` key (upstream's `Context`).
pub(crate) trait CacheContext<K> {
    fn hash(&self, key: &K) -> u64;
    fn eql(&self, a: &K, b: &K) -> bool;
}

/// An associative cache with `BUCKET_COUNT` fixed-size buckets of `BUCKET_SIZE` entries each;
/// a full bucket evicts its least-recently-used entry on insert (upstream `CacheTable`).
pub(crate) struct CacheTable<
    K: Copy,
    V: Copy,
    C: CacheContext<K>,
    const BUCKET_COUNT: usize,
    const BUCKET_SIZE: usize,
> {
    buckets: [[Option<(K, V)>; BUCKET_SIZE]; BUCKET_COUNT],
    lengths: [u8; BUCKET_COUNT],
    context: C,
}

impl<K: Copy, V: Copy, C: CacheContext<K>, const BUCKET_COUNT: usize, const BUCKET_SIZE: usize>
    CacheTable<K, V, C, BUCKET_COUNT, BUCKET_SIZE>
{
    pub(crate) fn new(context: C) -> Self {
        assert!(
            BUCKET_COUNT.is_power_of_two(),
            "bucket_count must be a power of two"
        );
        assert!(
            BUCKET_SIZE >= 1 && BUCKET_SIZE <= u8::MAX as usize,
            "invalid bucket_size"
        );
        Self {
            buckets: [[None; BUCKET_SIZE]; BUCKET_COUNT],
            lengths: [0; BUCKET_COUNT],
            context,
        }
    }

    /// Insert `(key, value)`. If a full bucket forced an eviction, the removed entry is
    /// returned (upstream's `?KV` / `evicted` hook).
    pub(crate) fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        let idx = (self.context.hash(&key) % BUCKET_COUNT as u64) as usize;
        let len = self.lengths[idx] as usize;

        if len < BUCKET_SIZE {
            self.buckets[idx][len] = Some((key, value));
            self.lengths[idx] += 1;
            return None;
        }

        // Full bucket: evict the front (LRU), shift left, append the new entry at the end.
        let evicted = self.buckets[idx][0].take();
        for i in 1..BUCKET_SIZE {
            self.buckets[idx][i - 1] = self.buckets[idx][i].take();
        }
        self.buckets[idx][BUCKET_SIZE - 1] = Some((key, value));
        evicted
    }

    /// Look up `key`, returning its value (and bumping it to most-recently-used) or `None`.
    pub(crate) fn get(&mut self, key: K) -> Option<V> {
        let idx = (self.context.hash(&key) % BUCKET_COUNT as u64) as usize;
        let len = self.lengths[idx] as usize;

        // Scan from the most-recent (end) toward the start.
        let mut i = len;
        while i > 0 {
            i -= 1;
            let (k, v) = self.buckets[idx][i].expect("slots below the length are populated");
            if self.context.eql(&key, &k) {
                // Bump the found entry to the end (most-recent): rotate [i..len] left once.
                for j in i + 1..len {
                    self.buckets[idx][j - 1] = self.buckets[idx][j].take();
                }
                self.buckets[idx][len - 1] = Some((k, v));
                return Some(v);
            }
        }
        None
    }

    /// Remove all entries.
    pub(crate) fn clear(&mut self) {
        for idx in 0..BUCKET_COUNT {
            for slot in 0..self.lengths[idx] as usize {
                self.buckets[idx][slot] = None;
            }
            self.lengths[idx] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A predictable identity-hash context so the tests can target specific buckets.
    struct IdentityContext;

    impl CacheContext<u32> for IdentityContext {
        fn hash(&self, key: &u32) -> u64 {
            *key as u64
        }

        fn eql(&self, a: &u32, b: &u32) -> bool {
            a == b
        }
    }

    type Table = CacheTable<u32, u32, IdentityContext, 2, 2>;

    #[test]
    fn upstream_fill_and_evict() {
        let mut t = Table::new(IdentityContext);

        // Fill the table (keys 0,2 -> bucket 0; keys 1,3 -> bucket 1).
        assert_eq!(t.put(0, 0), None);
        assert_eq!(t.put(1, 0), None);
        assert_eq!(t.put(2, 0), None);
        assert_eq!(t.put(3, 0), None);

        // It's now full, so an insert into bucket 0 evicts the oldest (key 0).
        assert_eq!(t.put(4, 0), Some((0, 0)));

        // The evicted key is gone.
        assert_eq!(t.get(0), None);
    }

    #[test]
    fn lookup_hit_and_miss() {
        let mut t = Table::new(IdentityContext);
        t.put(0, 42);
        assert_eq!(t.get(0), Some(42));
        assert_eq!(t.get(8), None); // bucket 0, not present
        assert_eq!(t.get(5), None); // bucket 1, empty
    }

    #[test]
    fn get_bumps_to_most_recent() {
        let mut t = Table::new(IdentityContext);
        // Bucket 0 holds [0, 2] (insertion order: 0 is LRU).
        t.put(0, 10);
        t.put(2, 20);

        // Looking up 0 bumps it to most-recent, so the bucket is now [2, 0] and 2 is the LRU.
        assert_eq!(t.get(0), Some(10));

        // Inserting another bucket-0 key evicts 2 (the LRU), not the freshly-bumped 0.
        assert_eq!(t.put(4, 40), Some((2, 20)));
        assert_eq!(t.get(2), None);
        assert_eq!(t.get(0), Some(10));
    }

    #[test]
    fn clear_removes_all() {
        let mut t = Table::new(IdentityContext);
        t.put(0, 1);
        t.put(1, 2);
        t.put(2, 3);

        t.clear();

        assert_eq!(t.get(0), None);
        assert_eq!(t.get(1), None);
        assert_eq!(t.get(2), None);

        // The table is reusable after clear.
        assert_eq!(t.put(7, 7), None);
        assert_eq!(t.get(7), Some(7));
    }
}
