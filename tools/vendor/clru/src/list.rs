use std::cmp::Ordering;
use std::ptr::NonNull;

#[derive(Debug)]
struct FixedSizeListNode<T> {
    prev: usize,
    next: usize,
    data: T,
}

#[derive(Debug)]
pub(crate) struct FixedSizeList<T> {
    capacity: usize,
    nodes: Vec<Option<FixedSizeListNode<T>>>,
    // An un-ordered set of indices that are not in use in `nodes`.
    // All `None` entries in `nodes` _must_ be listed in `free`.
    // A `Vec<usize>` was choosen in order to have O(1) complexity
    // for pop and avoid having to go through `nodes` in order to
    // to find a free place.
    free: Vec<usize>,
    front: usize,
    back: usize,
}

impl<T> FixedSizeList<T> {
    #[inline]
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            nodes: Vec::new(),
            free: Vec::new(),
            front: usize::MAX,
            back: usize::MAX,
        }
    }

    #[inline]
    pub(crate) fn with_memory(capacity: usize, mut reserve: usize) -> Self {
        if reserve > capacity {
            reserve = capacity;
        }
        Self {
            capacity,
            nodes: Vec::with_capacity(reserve),
            free: Vec::new(),
            front: usize::MAX,
            back: usize::MAX,
        }
    }

    #[inline]
    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.nodes.len() - self.free.len()
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
        self.free.clear();
        self.front = usize::MAX;
        self.back = usize::MAX;
    }

    fn next(&mut self) -> Option<usize> {
        if self.is_full() {
            None
        } else if self.free.is_empty() {
            let len = self.len();
            self.nodes.push(None);
            Some(len)
        } else {
            self.free.pop()
        }
    }

    #[inline]
    fn node_ref(&self, idx: usize) -> Option<&FixedSizeListNode<T>> {
        self.nodes.get(idx).and_then(|node| node.as_ref())
    }

    #[inline]
    pub(crate) fn get(&self, idx: usize) -> Option<&T> {
        self.node_ref(idx).map(|node| &node.data)
    }

    #[inline]
    fn node_mut(&mut self, idx: usize) -> Option<&mut FixedSizeListNode<T>> {
        self.nodes.get_mut(idx).and_then(|node| node.as_mut())
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.node_mut(idx).map(|node| &mut node.data)
    }

    #[inline]
    pub(crate) fn front(&self) -> Option<&T> {
        self.node_ref(self.front).map(|node| &node.data)
    }

    #[inline]
    pub(crate) fn front_mut(&mut self) -> Option<&mut T> {
        self.node_mut(self.front).map(|node| &mut node.data)
    }

    #[inline]
    pub(crate) fn back_idx(&self) -> usize {
        self.back
    }

    #[inline]
    pub(crate) fn back(&self) -> Option<&T> {
        self.node_ref(self.back).map(|node| &node.data)
    }

    #[inline]
    pub(crate) fn back_mut(&mut self) -> Option<&mut T> {
        self.node_mut(self.back).map(|node| &mut node.data)
    }

    pub(crate) fn push_front(&mut self, data: T) -> Option<(usize, &mut T)> {
        let idx = self.next()?;
        if let Some(front) = self.node_mut(self.front) {
            front.prev = idx;
        }
        if self.node_ref(self.back).is_none() {
            self.back = idx;
        }
        let node = self.nodes.get_mut(idx).unwrap().insert(FixedSizeListNode {
            prev: usize::MAX,
            next: self.front,
            data,
        });
        self.front = idx;
        Some((idx, &mut node.data))
    }

    #[cfg(test)]
    fn push_back(&mut self, data: T) -> Option<(usize, &mut T)> {
        let idx = self.next()?;
        if let Some(back) = self.node_mut(self.back) {
            back.next = idx;
        }
        if self.node_ref(self.front).is_none() {
            self.front = idx;
        }
        let node = self.nodes.get_mut(idx).unwrap().insert(FixedSizeListNode {
            prev: self.back,
            next: usize::MAX,
            data,
        });
        self.back = idx;
        Some((idx, &mut node.data))
    }

    #[inline]
    pub(crate) fn pop_front(&mut self) -> Option<T> {
        self.remove(self.front)
    }

    #[inline]
    pub(crate) fn pop_back(&mut self) -> Option<T> {
        self.remove(self.back)
    }

    pub(crate) fn remove(&mut self, idx: usize) -> Option<T> {
        let node = self.nodes.get_mut(idx)?.take()?;
        if let Some(prev) = self.node_mut(node.prev) {
            prev.next = node.next;
        } else {
            self.front = node.next;
        }
        if let Some(next) = self.node_mut(node.next) {
            next.prev = node.prev;
        } else {
            self.back = node.prev;
        }
        self.free.push(idx);
        Some(node.data)
    }

    #[inline]
    pub(crate) fn iter(&self) -> FixedSizeListIter<'_, T> {
        FixedSizeListIter {
            list: self,
            front: self.front,
            back: self.back,
            len: self.len(),
        }
    }

    #[inline]
    pub(crate) fn iter_mut(&mut self) -> FixedSizeListIterMut<'_, T> {
        let front = self.front;
        let back = self.back;
        let len = self.len();
        FixedSizeListIterMut::new(&mut self.nodes, front, back, len)
    }

    fn reorder(&mut self) {
        if self.is_empty() {
            return;
        }

        let len = self.len();
        let mut current = 0;
        while current < len {
            let front = self.front;
            let front_data = self.pop_front().unwrap();
            if front != current {
                debug_assert!(current < front, "{} < {}", current, front);
                // We need to free self.nodes[current] if its occupied
                if let Some(current_node) = self.nodes[current].take() {
                    if let Some(node) = self.node_mut(current_node.prev) {
                        node.next = front;
                    } else {
                        self.front = front;
                    }
                    if let Some(node) = self.node_mut(current_node.next) {
                        node.prev = front;
                    } else {
                        self.back = front;
                    }
                    self.nodes[front] = Some(current_node);
                }
            }
            // Assign new front node
            self.nodes[current] = Some(FixedSizeListNode {
                prev: current.wrapping_sub(1),
                next: current + 1,
                data: front_data,
            });
            current += 1;
        }
        self.front = 0;
        self.nodes[len - 1].as_mut().unwrap().next = usize::MAX;
        self.back = len - 1;
        self.free.clear();
        self.free.extend((len..self.nodes.len()).rev());
    }

    pub(crate) fn resize(&mut self, capacity: usize) {
        let len = self.len();
        let cap = self.capacity();
        match capacity.cmp(&cap) {
            Ordering::Less => {
                self.reorder();
                self.nodes.truncate(capacity);
                self.free.clear();
                self.free.extend(len..self.nodes.len());
                self.capacity = capacity;
            }
            Ordering::Equal => {}
            Ordering::Greater => {
                self.capacity = capacity;
            }
        };
        debug_assert_eq!(self.len(), len);
        debug_assert_eq!(self.capacity(), capacity);
    }

    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        let mut front = self.front;
        while front != usize::MAX {
            let node = self.node_ref(front).unwrap();
            let next = node.next;
            if !f(&node.data) {
                self.remove(front);
            }
            front = next;
        }
    }

    pub(crate) fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut front = self.front;
        while front != usize::MAX {
            let node = self.node_mut(front).unwrap();
            let next = node.next;
            if !f(&mut node.data) {
                self.remove(front);
            }
            front = next;
        }
    }

    #[inline]
    pub(crate) fn move_front(&mut self, idx: usize) -> Option<&mut T> {
        // TODO: try to optimize this funtion as it is a fairly hot path
        let node = self.nodes.get_mut(idx)?.take()?;
        if let Some(prev) = self.node_mut(node.prev) {
            prev.next = node.next;
        } else {
            self.front = node.next;
        }
        if let Some(next) = self.node_mut(node.next) {
            next.prev = node.prev;
        } else {
            self.back = node.prev;
        }

        if let Some(front) = self.node_mut(self.front) {
            front.prev = idx;
        }
        if self.node_ref(self.back).is_none() {
            self.back = idx;
        }

        let node = self.nodes.get_mut(idx).unwrap().insert(FixedSizeListNode {
            prev: usize::MAX,
            next: self.front,
            data: node.data,
        });
        self.front = idx;
        Some(&mut node.data)
    }
}

#[derive(Debug)]
pub(crate) struct FixedSizeListIter<'a, T> {
    list: &'a FixedSizeList<T>,
    front: usize,
    back: usize,
    len: usize,
}

impl<'a, T> Clone for FixedSizeListIter<'a, T> {
    fn clone(&self) -> Self {
        Self {
            list: self.list,
            front: self.front,
            back: self.back,
            len: self.len,
        }
    }
}

impl<'a, T> Iterator for FixedSizeListIter<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let front = self.front;
            let node = self.list.node_ref(front).unwrap();
            self.front = node.next;
            self.len -= 1;
            Some((front, &node.data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for FixedSizeListIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let back = self.back;
            let node = self.list.node_ref(back).unwrap();
            self.back = node.prev;
            self.len -= 1;
            Some((back, &node.data))
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for FixedSizeListIter<'a, T> {
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

pub(crate) struct FixedSizeListIterMut<'a, T> {
    ptr: NonNull<Option<FixedSizeListNode<T>>>,
    front: usize,
    back: usize,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> FixedSizeListIterMut<'a, T> {
    #[allow(unsafe_code)]
    fn new(
        slice: &'a mut [Option<FixedSizeListNode<T>>],
        front: usize,
        back: usize,
        len: usize,
    ) -> Self {
        let ptr = slice.as_mut_ptr();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            front,
            back,
            len,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> Iterator for FixedSizeListIterMut<'a, T> {
    type Item = (usize, &'a mut T);

    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let front = self.front;

            // Safety:
            // * `self.ptr` is a valid non null pointer since it can only be created through `FixedSizeListIterMut::new`.
            // * `front` is guaranteed to be a valid index within the slice pointed to by `self.ptr`.
            // Notes: implementation is inspired by the iterator over mutable slice from the standard rust library
            // * https://doc.rust-lang.org/src/core/slice/iter.rs.html
            // * https://doc.rust-lang.org/src/core/slice/iter/macros.rs.html
            let node_ref = unsafe {
                let ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(front)).as_ptr();
                &mut *ptr
            };

            let node = node_ref.as_mut().unwrap();
            self.front = node.next;
            self.len -= 1;
            Some((front, &mut node.data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for FixedSizeListIterMut<'a, T> {
    #[allow(unsafe_code)]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            let back = self.back;

            // Safety:
            // * `self.ptr` is a valid non null pointer since it can only be created through `FixedSizeListIterMut::new`.
            // * `back` is guaranteed to be a valid index within the slice pointed to by `self.ptr`.
            // Notes: implementation is inspired by the iterator over mutable slice from the standard rust library
            // * https://doc.rust-lang.org/src/core/slice/iter.rs.html
            // * https://doc.rust-lang.org/src/core/slice/iter/macros.rs.html
            let node_ref = unsafe {
                let ptr = NonNull::new_unchecked(self.ptr.as_ptr().add(back)).as_ptr();
                &mut *ptr
            };

            let node = node_ref.as_mut().unwrap();
            self.back = node.prev;
            self.len -= 1;
            Some((back, &mut node.data))
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for FixedSizeListIterMut<'a, T> {
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_size_list() {
        let mut list = FixedSizeList::new(4);

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        assert_eq!(list.front(), None);
        assert_eq!(list.front_mut(), None);

        assert_eq!(list.back(), None);
        assert_eq!(list.back_mut(), None);

        assert_eq!(list.iter().count(), 0);
        assert_eq!(list.iter().rev().count(), 0);

        assert_eq!(list.push_front(7), Some((0, &mut 7)));

        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);

        assert_eq!(list.front(), Some(&7));
        assert_eq!(list.front_mut(), Some(&mut 7));

        assert_eq!(list.back(), Some(&7));
        assert_eq!(list.back_mut(), Some(&mut 7));

        assert_eq!(list.iter().collect::<Vec<_>>(), vec![(0, &7)]);
        assert_eq!(list.iter().rev().collect::<Vec<_>>(), vec![(0, &7)]);

        assert_eq!(list.push_front(5), Some((1, &mut 5)));

        assert!(!list.is_empty());
        assert_eq!(list.len(), 2);

        assert_eq!(list.front(), Some(&5));
        assert_eq!(list.front_mut(), Some(&mut 5));

        assert_eq!(list.back(), Some(&7));
        assert_eq!(list.back_mut(), Some(&mut 7));

        assert_eq!(list.iter().collect::<Vec<_>>(), vec![(1, &5), (0, &7)]);
        assert_eq!(
            list.iter().rev().collect::<Vec<_>>(),
            vec![(0, &7), (1, &5)]
        );

        assert_eq!(list.push_front(3), Some((2, &mut 3)));

        assert!(!list.is_empty());
        assert_eq!(list.len(), 3);

        assert_eq!(list.front(), Some(&3));
        assert_eq!(list.front_mut(), Some(&mut 3));

        assert_eq!(list.back(), Some(&7));
        assert_eq!(list.back_mut(), Some(&mut 7));

        assert_eq!(
            list.iter().collect::<Vec<_>>(),
            vec![(2, &3), (1, &5), (0, &7)]
        );
        assert_eq!(
            list.iter().rev().collect::<Vec<_>>(),
            vec![(0, &7), (1, &5), (2, &3)]
        );

        list.remove(1);

        assert!(!list.is_empty());
        assert_eq!(list.len(), 2);

        assert_eq!(list.front(), Some(&3));
        assert_eq!(list.front_mut(), Some(&mut 3));

        assert_eq!(list.back(), Some(&7));
        assert_eq!(list.back_mut(), Some(&mut 7));

        assert_eq!(list.iter().collect::<Vec<_>>(), vec![(2, &3), (0, &7)]);
        assert_eq!(
            list.iter().rev().collect::<Vec<_>>(),
            vec![(0, &7), (2, &3)]
        );

        list.remove(0);

        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);

        assert_eq!(list.front(), Some(&3));
        assert_eq!(list.front_mut(), Some(&mut 3));

        assert_eq!(list.back(), Some(&3));
        assert_eq!(list.back_mut(), Some(&mut 3));

        assert_eq!(list.iter().collect::<Vec<_>>(), vec![(2, &3)]);
        assert_eq!(list.iter().rev().collect::<Vec<_>>(), vec![(2, &3)]);

        list.remove(2);

        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        assert_eq!(list.front(), None);
        assert_eq!(list.front_mut(), None);

        assert_eq!(list.back(), None);
        assert_eq!(list.back_mut(), None);

        assert_eq!(list.iter().count(), 0);
        assert_eq!(list.iter().rev().count(), 0);
    }

    #[test]
    fn test_fixed_size_list_reorder() {
        let mut list = FixedSizeList::new(4);

        list.push_back('a');
        list.push_front('b');
        list.push_back('c');
        list.push_front('d');

        assert_eq!(
            list.iter().collect::<Vec<_>>(),
            vec![(3, &'d'), (1, &'b'), (0, &'a'), (2, &'c')]
        );

        list.reorder();

        assert_eq!(
            list.iter().collect::<Vec<_>>(),
            vec![(0, &'d'), (1, &'b'), (2, &'a'), (3, &'c')]
        );
    }
}
