const UNIT_BITS: usize = u8::BITS as usize;
const PREALLOC_COLUMNS: usize = 512;
const PREALLOC_COUNT: usize = PREALLOC_COLUMNS / UNIT_BITS;

const MASKS: [u8; UNIT_BITS] = [1, 2, 4, 8, 16, 32, 64, 128];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TabstopError {
    OutOfMemory,
}

#[derive(Debug, Clone)]
pub(super) struct Tabstops {
    cols: usize,
    prealloc_stops: [u8; PREALLOC_COUNT],
    dynamic_stops: Vec<u8>,
}

impl Tabstops {
    pub(super) fn new(cols: usize, interval: usize) -> Result<Self, TabstopError> {
        let mut tabstops = Self::default();
        tabstops.resize(cols)?;
        tabstops.reset(interval);
        Ok(tabstops)
    }

    pub(super) fn resize(&mut self, cols: usize) -> Result<(), TabstopError> {
        if cols <= PREALLOC_COLUMNS {
            self.cols = cols;
            return Ok(());
        }

        let size = cols - PREALLOC_COLUMNS;
        if size < self.dynamic_stops.len() {
            self.cols = cols;
            return Ok(());
        }

        let mut new = Vec::new();
        reserve_dynamic_stops(&mut new, size)?;
        new.resize(size, 0);
        new[..self.dynamic_stops.len()].copy_from_slice(&self.dynamic_stops);

        self.dynamic_stops = new;
        self.cols = cols;
        Ok(())
    }

    pub(super) fn set(&mut self, col: usize) {
        let i = entry(col);
        let idx = index(col);
        if i < PREALLOC_COUNT {
            self.prealloc_stops[i] |= MASKS[idx];
            return;
        }

        let dynamic_i = i - PREALLOC_COUNT;
        assert!(dynamic_i < self.dynamic_stops.len());
        self.dynamic_stops[dynamic_i] |= MASKS[idx];
    }

    pub(super) fn unset(&mut self, col: usize) {
        let i = entry(col);
        let idx = index(col);
        if i < PREALLOC_COUNT {
            self.prealloc_stops[i] ^= MASKS[idx];
            return;
        }

        let dynamic_i = i - PREALLOC_COUNT;
        assert!(dynamic_i < self.dynamic_stops.len());
        self.dynamic_stops[dynamic_i] ^= MASKS[idx];
    }

    pub(super) fn get(&self, col: usize) -> bool {
        let i = entry(col);
        let idx = index(col);
        let mask = MASKS[idx];
        let unit = if i < PREALLOC_COUNT {
            self.prealloc_stops[i]
        } else {
            let dynamic_i = i - PREALLOC_COUNT;
            assert!(dynamic_i < self.dynamic_stops.len());
            self.dynamic_stops[dynamic_i]
        };

        unit & mask == mask
    }

    pub(super) fn capacity(&self) -> usize {
        (PREALLOC_COUNT + self.dynamic_stops.len()) * UNIT_BITS
    }

    pub(super) fn cols(&self) -> usize {
        self.cols
    }

    pub(super) fn reset(&mut self, interval: usize) {
        self.prealloc_stops.fill(0);
        self.dynamic_stops.fill(0);

        if interval > 0 {
            let last_col = self
                .cols
                .checked_sub(1)
                .expect("tabstops reset requires at least one column");
            let mut i = interval;
            while i < last_col {
                self.set(i);
                i += interval;
            }
        }
    }
}

impl Default for Tabstops {
    fn default() -> Self {
        Self {
            cols: 0,
            prealloc_stops: [0; PREALLOC_COUNT],
            dynamic_stops: Vec::new(),
        }
    }
}

fn entry(col: usize) -> usize {
    col / UNIT_BITS
}

fn index(col: usize) -> usize {
    col % UNIT_BITS
}

#[cfg(not(test))]
fn reserve_dynamic_stops(dynamic_stops: &mut Vec<u8>, size: usize) -> Result<(), TabstopError> {
    dynamic_stops
        .try_reserve_exact(size)
        .map_err(|_| TabstopError::OutOfMemory)
}

#[cfg(test)]
fn reserve_dynamic_stops(dynamic_stops: &mut Vec<u8>, size: usize) -> Result<(), TabstopError> {
    if tests::take_fail_next_dynamic_alloc() {
        return Err(TabstopError::OutOfMemory);
    }

    dynamic_stops
        .try_reserve_exact(size)
        .map_err(|_| TabstopError::OutOfMemory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    static FAIL_NEXT_DYNAMIC_ALLOC: AtomicBool = AtomicBool::new(false);

    pub(super) fn take_fail_next_dynamic_alloc() -> bool {
        FAIL_NEXT_DYNAMIC_ALLOC.swap(false, Ordering::SeqCst)
    }

    fn fail_next_dynamic_alloc() {
        FAIL_NEXT_DYNAMIC_ALLOC.store(true, Ordering::SeqCst);
    }

    #[test]
    fn tabstops_basic() {
        let mut t = Tabstops::default();

        assert_eq!(0, entry(4));
        assert_eq!(1, entry(8));
        assert_eq!(0, index(0));
        assert_eq!(1, index(1));
        assert_eq!(1, index(9));

        assert_eq!(0b0000_1000, MASKS[3]);
        assert_eq!(0b0001_0000, MASKS[4]);

        assert!(!t.get(4));
        t.set(4);
        assert!(t.get(4));
        assert!(!t.get(3));

        t.reset(0);
        assert!(!t.get(4));

        t.set(4);
        assert!(t.get(4));
        t.unset(4);
        assert!(!t.get(4));
    }

    #[test]
    fn tabstops_unset_toggles() {
        let mut t = Tabstops::default();

        t.set(4);
        assert!(t.get(4));
        t.unset(4);
        assert!(!t.get(4));
        t.unset(4);
        assert!(t.get(4));
    }

    #[test]
    fn tabstops_dynamic_allocations() {
        let mut t = Tabstops::default();

        let cap = t.capacity();
        t.resize(cap * 2).unwrap();

        t.set(cap + 5);
        assert!(t.get(cap + 5));
        assert!(!t.get(cap + 4));

        assert!(!t.get(5));
    }

    #[test]
    fn tabstops_preserves_upstream_capacity_semantics() {
        let mut t = Tabstops::default();
        let cap = t.capacity();

        t.resize(cap * 2).unwrap();

        assert_eq!(
            (PREALLOC_COUNT + (cap * 2 - PREALLOC_COLUMNS)) * UNIT_BITS,
            t.capacity()
        );
    }

    #[test]
    fn tabstops_interval() {
        let t = Tabstops::new(80, 4).unwrap();

        assert!(!t.get(0));
        assert!(t.get(4));
        assert!(!t.get(5));
        assert!(t.get(8));
    }

    #[test]
    fn tabstops_count_on_80() {
        let t = Tabstops::new(80, 8).unwrap();

        let mut count = 0;
        for i in 0..80 {
            if t.get(i) {
                count += 1;
            }
        }

        assert_eq!(9, count);
    }

    #[test]
    fn tabstops_resize_alloc_failure_preserves_state() {
        let mut t = Tabstops::new(80, 8).unwrap();
        let original_cols = t.cols();

        fail_next_dynamic_alloc();
        let result = t.resize(PREALLOC_COLUMNS * 2);

        assert_eq!(Err(TabstopError::OutOfMemory), result);
        assert_eq!(original_cols, t.cols());
    }
}
