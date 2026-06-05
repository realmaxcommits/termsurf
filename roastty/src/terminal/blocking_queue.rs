//! A fixed-capacity single-producer/single-consumer blocking queue (port of upstream
//! `datastruct/blocking_queue`).
//!
//! The producer blocks on a full queue (per `Timeout`); there is no blocking pop — callers use an
//! external "not empty" notifier. Upstream's `std.Thread.Mutex` + `std.Thread.Condition` become
//! `std::sync::{Mutex, Condvar}`, with the ring state moved inside the `Mutex`.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

/// How long `push` waits when the queue is full (upstream `Timeout`).
#[derive(Debug, Clone, Copy)]
pub(crate) enum Timeout {
    /// Fail immediately if full (upstream `.instant`).
    Instant,
    /// Wait until space is available (upstream `.forever`).
    Forever,
    /// Wait up to this many nanoseconds (upstream `.ns`).
    Ns(u64),
}

struct Inner<T, const CAP: usize> {
    data: [Option<T>; CAP],
    write: usize,
    read: usize,
    len: usize,
    not_full_waiters: usize,
}

/// A fixed-capacity SPSC blocking queue (upstream `BlockingQueue`). The producer blocks on a full
/// queue (per `Timeout`); there is no blocking pop (use an external "not empty" notifier).
pub(crate) struct BlockingQueue<T, const CAP: usize> {
    inner: Mutex<Inner<T, CAP>>,
    cond_not_full: Condvar,
}

impl<T, const CAP: usize> BlockingQueue<T, CAP> {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(Inner {
                data: std::array::from_fn(|_| None),
                write: 0,
                read: 0,
                len: 0,
                not_full_waiters: 0,
            }),
            cond_not_full: Condvar::new(),
        }
    }

    /// Push `value`, returning the new queue length (`0` = failed) (upstream `push`).
    pub(crate) fn push(&self, value: T, timeout: Timeout) -> usize {
        let mut inner = self.inner.lock().unwrap();

        if inner.len == CAP {
            match timeout {
                Timeout::Instant => return 0,
                Timeout::Forever => {
                    inner.not_full_waiters += 1;
                    inner = self.cond_not_full.wait(inner).unwrap();
                    inner.not_full_waiters -= 1;
                }
                Timeout::Ns(ns) => {
                    inner.not_full_waiters += 1;
                    let (guard, res) = self
                        .cond_not_full
                        .wait_timeout(inner, Duration::from_nanos(ns))
                        .unwrap();
                    inner = guard;
                    inner.not_full_waiters -= 1;
                    if res.timed_out() {
                        return 0;
                    }
                }
            }
            // Interrupted / spurious wake while still full: fail.
            if inner.len == CAP {
                return 0;
            }
        }

        let w = inner.write;
        inner.data[w] = Some(value);
        inner.write += 1;
        if inner.write >= CAP {
            inner.write -= CAP;
        }
        inner.len += 1;
        inner.len
    }

    /// Pop a value without blocking (upstream `pop`).
    pub(crate) fn pop(&self) -> Option<T> {
        let mut inner = self.inner.lock().unwrap();
        if inner.len == 0 {
            return None;
        }
        let value = Self::take_at_read(&mut inner);
        if inner.not_full_waiters > 0 {
            self.cond_not_full.notify_one();
        }
        Some(value)
    }

    /// Lock and return a draining iterator (upstream `drain`).
    pub(crate) fn drain(&self) -> Drain<'_, T, CAP> {
        Drain {
            guard: self.inner.lock().unwrap(),
            cond: &self.cond_not_full,
        }
    }

    /// Read the value at `read` and advance the cursor (lock held; caller guarantees `len > 0`,
    /// so the slot is always occupied — an empty slot here is an invariant violation).
    fn take_at_read(inner: &mut Inner<T, CAP>) -> T {
        let n = inner.read;
        inner.read += 1;
        if inner.read >= CAP {
            inner.read -= CAP;
        }
        inner.len -= 1;
        inner.data[n].take().expect("occupied slot")
    }
}

/// A draining iterator holding the queue lock (upstream `DrainIterator`).
pub(crate) struct Drain<'a, T, const CAP: usize> {
    guard: MutexGuard<'a, Inner<T, CAP>>,
    cond: &'a Condvar,
}

impl<T, const CAP: usize> Iterator for Drain<'_, T, CAP> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.guard.len == 0 {
            return None;
        }
        Some(BlockingQueue::<T, CAP>::take_at_read(&mut self.guard))
    }
}

impl<T, const CAP: usize> Drop for Drain<'_, T, CAP> {
    fn drop(&mut self) {
        // Signal a blocked producer (if any) on the way out (upstream `DrainIterator.deinit`).
        if self.guard.not_full_waiters > 0 {
            self.cond.notify_one();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn basic_push_and_pop() {
        let q: BlockingQueue<u64, 4> = BlockingQueue::new();

        // Empty.
        assert_eq!(q.pop(), None);

        // Push until full.
        assert_eq!(q.push(1, Timeout::Instant), 1);
        assert_eq!(q.push(2, Timeout::Instant), 2);
        assert_eq!(q.push(3, Timeout::Instant), 3);
        assert_eq!(q.push(4, Timeout::Instant), 4);
        assert_eq!(q.push(5, Timeout::Instant), 0); // full

        // Pop in FIFO order.
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.pop(), Some(2));
        assert_eq!(q.pop(), Some(3));
        assert_eq!(q.pop(), Some(4));
        assert_eq!(q.pop(), None);

        // Drain does nothing on an empty queue.
        {
            let mut it = q.drain();
            assert_eq!(it.next(), None);
        }

        // Can still push.
        assert_eq!(q.push(1, Timeout::Instant), 1);
    }

    #[test]
    fn timed_push() {
        let q: BlockingQueue<u64, 1> = BlockingQueue::new();
        assert_eq!(q.push(1, Timeout::Instant), 1);
        assert_eq!(q.push(2, Timeout::Instant), 0);

        // A timed push on a full queue fails after the timeout.
        assert_eq!(q.push(2, Timeout::Ns(1000)), 0);
    }

    #[test]
    fn drain_with_values() {
        let q: BlockingQueue<u64, 4> = BlockingQueue::new();
        q.push(10, Timeout::Instant);
        q.push(20, Timeout::Instant);
        q.push(30, Timeout::Instant);

        {
            let drained: Vec<u64> = q.drain().collect();
            assert_eq!(drained, vec![10, 20, 30]);
        }

        // Empty afterward, and accepts pushes again.
        assert_eq!(q.pop(), None);
        assert_eq!(q.push(40, Timeout::Instant), 1);
    }

    #[test]
    fn ring_wraps_around_preserving_fifo() {
        let q: BlockingQueue<u64, 3> = BlockingQueue::new();
        // Interleave so write/read wrap past capacity.
        for i in 0..10 {
            assert_ne!(q.push(i, Timeout::Instant), 0);
            assert_eq!(q.pop(), Some(i));
            assert_eq!(q.pop(), None);
        }
    }

    #[test]
    fn forever_blocks_until_space() {
        let q: Arc<BlockingQueue<u64, 1>> = Arc::new(BlockingQueue::new());
        assert_eq!(q.push(1, Timeout::Instant), 1); // full

        let producer = {
            let q = Arc::clone(&q);
            thread::spawn(move || q.push(2, Timeout::Forever))
        };

        // Wait until the producer is actually registered as a not-full waiter, so the pop below
        // deterministically exercises the condvar wake path rather than racing past it.
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if q.inner.lock().unwrap().not_full_waiters == 1 {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "producer never registered as a waiter"
            );
            thread::yield_now();
        }

        // Make space: the producer's blocked push wakes and completes.
        assert_eq!(q.pop(), Some(1));
        assert_eq!(producer.join().unwrap(), 1);

        // The producer's value is now queued.
        assert_eq!(q.pop(), Some(2));
    }
}
