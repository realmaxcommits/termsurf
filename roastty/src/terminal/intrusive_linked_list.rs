//! A doubly-linked list (arena port of upstream `datastruct/intrusive_linked_list`).
//!
//! Upstream's list is *intrusive*: the node type is `T` itself (with `prev`/`next` pointers) and
//! the caller owns each node. That raw-pointer design does not translate to safe Rust, so this
//! port owns its nodes in an arena (`Vec<Option<Node>>` + a free list, `NIL` sentinel) and hands
//! out `usize` handles — the analogue of upstream's `*Node`. The list order and operations match
//! upstream exactly.

const NIL: usize = usize::MAX;

struct Node<T> {
    data: T,
    prev: usize, // NIL = none
    next: usize, // NIL = none
}

/// A doubly-linked list owning its nodes in an arena (upstream `DoublyLinkedList`). Handles are
/// `usize` indices returned by the insert methods.
pub(crate) struct DoublyLinkedList<T> {
    nodes: Vec<Option<Node<T>>>,
    free: Vec<usize>,
    first: usize,
    last: usize,
}

impl<T> DoublyLinkedList<T> {
    pub(crate) fn new() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            first: NIL,
            last: NIL,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.first == NIL
    }

    fn node(&self, idx: usize) -> &Node<T> {
        self.nodes[idx].as_ref().expect("occupied slot")
    }

    fn node_mut(&mut self, idx: usize) -> &mut Node<T> {
        self.nodes[idx].as_mut().expect("occupied slot")
    }

    /// Allocate a node (reusing a freed slot if available), returning its handle.
    fn alloc(&mut self, node: Node<T>) -> usize {
        match self.free.pop() {
            Some(idx) => {
                self.nodes[idx] = Some(node);
                idx
            }
            None => {
                self.nodes.push(Some(node));
                self.nodes.len() - 1
            }
        }
    }

    /// Insert `data` after the node at `handle`, returning the new handle (upstream `insertAfter`).
    pub(crate) fn insert_after(&mut self, handle: usize, data: T) -> usize {
        let next = self.node(handle).next;
        let idx = self.alloc(Node {
            data,
            prev: handle,
            next,
        });
        if next != NIL {
            self.node_mut(next).prev = idx;
        } else {
            self.last = idx;
        }
        self.node_mut(handle).next = idx;
        idx
    }

    /// Insert `data` before the node at `handle`, returning the new handle (upstream
    /// `insertBefore`).
    pub(crate) fn insert_before(&mut self, handle: usize, data: T) -> usize {
        let prev = self.node(handle).prev;
        let idx = self.alloc(Node {
            data,
            prev,
            next: handle,
        });
        if prev != NIL {
            self.node_mut(prev).next = idx;
        } else {
            self.first = idx;
        }
        self.node_mut(handle).prev = idx;
        idx
    }

    /// Append `data` at the end, returning its handle (upstream `append`).
    pub(crate) fn append(&mut self, data: T) -> usize {
        if self.last != NIL {
            self.insert_after(self.last, data)
        } else {
            self.prepend(data)
        }
    }

    /// Prepend `data` at the beginning, returning its handle (upstream `prepend`).
    pub(crate) fn prepend(&mut self, data: T) -> usize {
        if self.first != NIL {
            self.insert_before(self.first, data)
        } else {
            let idx = self.alloc(Node {
                data,
                prev: NIL,
                next: NIL,
            });
            self.first = idx;
            self.last = idx;
            idx
        }
    }

    /// Rewire the list past `handle` (upstream `remove`'s pointer fixups; does not free).
    fn unlink(&mut self, handle: usize) {
        let prev = self.node(handle).prev;
        let next = self.node(handle).next;
        if prev != NIL {
            self.node_mut(prev).next = next;
        } else {
            self.first = next;
        }
        if next != NIL {
            self.node_mut(next).prev = prev;
        } else {
            self.last = prev;
        }
    }

    /// Remove the node at `handle` and return its data (upstream `remove`, plus freeing the slot —
    /// the handle is invalid afterward).
    pub(crate) fn remove(&mut self, handle: usize) -> T {
        self.unlink(handle);
        let node = self.nodes[handle].take().expect("occupied slot");
        self.free.push(handle);
        node.data
    }

    /// Remove and return the last node's data (upstream `pop`).
    pub(crate) fn pop(&mut self) -> Option<T> {
        if self.last == NIL {
            None
        } else {
            Some(self.remove(self.last))
        }
    }

    /// Remove and return the first node's data (upstream `popFirst`).
    pub(crate) fn pop_first(&mut self) -> Option<T> {
        if self.first == NIL {
            None
        } else {
            Some(self.remove(self.first))
        }
    }

    /// The first node's handle, or `None` if empty (upstream `first`).
    pub(crate) fn first(&self) -> Option<usize> {
        (self.first != NIL).then_some(self.first)
    }

    /// The last node's handle, or `None` if empty (upstream `last`).
    pub(crate) fn last(&self) -> Option<usize> {
        (self.last != NIL).then_some(self.last)
    }

    /// The handle following `handle`, or `None` at the end (upstream `node.next`).
    pub(crate) fn next(&self, handle: usize) -> Option<usize> {
        let n = self.node(handle).next;
        (n != NIL).then_some(n)
    }

    /// The handle preceding `handle`, or `None` at the start (upstream `node.prev`).
    pub(crate) fn prev(&self, handle: usize) -> Option<usize> {
        let p = self.node(handle).prev;
        (p != NIL).then_some(p)
    }

    /// Borrow the data at `handle`.
    pub(crate) fn get(&self, handle: usize) -> &T {
        &self.node(handle).data
    }

    /// Mutably borrow the data at `handle`.
    pub(crate) fn get_mut(&mut self, handle: usize) -> &mut T {
        &mut self.node_mut(handle).data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Collect the list data forwards (`first` → `next`).
    fn forward(list: &DoublyLinkedList<u32>) -> Vec<u32> {
        let mut out = Vec::new();
        let mut it = list.first();
        while let Some(handle) = it {
            out.push(*list.get(handle));
            it = list.next(handle);
        }
        out
    }

    /// Collect the list data backwards (`last` → `prev`).
    fn backward(list: &DoublyLinkedList<u32>) -> Vec<u32> {
        let mut out = Vec::new();
        let mut it = list.last();
        while let Some(handle) = it {
            out.push(*list.get(handle));
            it = list.prev(handle);
        }
        out
    }

    #[test]
    fn upstream_sequence() {
        let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();

        let two = list.append(2); // {2}
        list.append(5); // {2, 5}
        list.prepend(1); // {1, 2, 5}
                         // {1, 2, 5}; find the handle of 5 (the last) to insert 4 before it.
        let five = list.last().unwrap();
        list.insert_before(five, 4); // {1, 2, 4, 5}
        let three = list.insert_after(two, 3); // {1, 2, 3, 4, 5}

        assert_eq!(forward(&list), vec![1, 2, 3, 4, 5]);
        assert_eq!(backward(&list), vec![5, 4, 3, 2, 1]);

        assert_eq!(list.pop_first(), Some(1)); // {2, 3, 4, 5}
        assert_eq!(list.pop(), Some(5)); // {2, 3, 4}
        assert_eq!(list.remove(three), 3); // {2, 4}

        assert_eq!(*list.get(list.first().unwrap()), 2);
        assert_eq!(*list.get(list.last().unwrap()), 4);
        assert_eq!(forward(&list), vec![2, 4]);
    }

    #[test]
    fn empty_list() {
        let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
        assert!(list.is_empty());
        assert_eq!(list.pop(), None);
        assert_eq!(list.pop_first(), None);
        assert!(list.first().is_none());
        assert!(list.last().is_none());

        list.append(7);
        assert!(!list.is_empty());
    }

    #[test]
    fn single_element() {
        let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
        let h = list.append(42);
        assert_eq!(list.first(), Some(h));
        assert_eq!(list.last(), Some(h));
        assert!(list.next(h).is_none());
        assert!(list.prev(h).is_none());

        assert_eq!(list.pop_first(), Some(42));
        assert!(list.is_empty());
    }

    #[test]
    fn get_mut_mutates_data() {
        let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
        let h = list.append(10);
        *list.get_mut(h) = 99;
        assert_eq!(*list.get(h), 99);
    }

    #[test]
    fn free_slot_is_reused_without_corrupting_order() {
        let mut list: DoublyLinkedList<u32> = DoublyLinkedList::new();
        list.append(1);
        let two = list.append(2);
        list.append(3); // {1, 2, 3}

        // Remove the middle, freeing its slot, then append: the freed slot is reused.
        assert_eq!(list.remove(two), 2); // {1, 3}
        list.append(4); // {1, 3, 4}

        assert_eq!(forward(&list), vec![1, 3, 4]);
        assert_eq!(backward(&list), vec![4, 3, 1]);
    }
}
