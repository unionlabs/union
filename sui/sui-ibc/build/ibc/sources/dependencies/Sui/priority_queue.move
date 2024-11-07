// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Priority queue implemented using a max heap.
module sui::priority_queue;

/// For when heap is empty and there's no data to pop.
const EPopFromEmptyHeap: u64 = 0;

/// Struct representing a priority queue. The `entries` vector represents a max
/// heap structure, where entries[0] is the root, entries[1] and entries[2] are the
/// left child and right child of the root, etc. More generally, the children of
/// entries[i] are at i * 2 + 1 and i * 2 + 2. The max heap should have the invariant
/// that the parent node's priority is always higher than its child nodes' priorities.
public struct PriorityQueue<T: drop> has store, drop {
    entries: vector<Entry<T>>,
}

public struct Entry<T: drop> has store, drop {
    priority: u64, // higher value means higher priority and will be popped first
    value: T,
}

/// Create a new priority queue from the input entry vectors.
public fun new<T: drop>(mut entries: vector<Entry<T>>): PriorityQueue<T> {
    let len = entries.length();
    let mut i = len / 2;
    // Max heapify from the first node that is a parent (node at len / 2).
    while (i > 0) {
        i = i - 1;
        max_heapify_recursive(&mut entries, len, i);
    };
    PriorityQueue { entries }
}

/// Pop the entry with the highest priority value.
public fun pop_max<T: drop>(pq: &mut PriorityQueue<T>): (u64, T) {
    let len = pq.entries.length();
    assert!(len > 0, EPopFromEmptyHeap);
    // Swap the max element with the last element in the entries and remove the max element.
    let Entry { priority, value } = pq.entries.swap_remove(0);
    // Now the max heap property has been violated at the root node, but nowhere else
    // so we call max heapify on the root node.
    max_heapify_recursive(&mut pq.entries, len - 1, 0);
    (priority, value)
}

/// Insert a new entry into the queue.
public fun insert<T: drop>(pq: &mut PriorityQueue<T>, priority: u64, value: T) {
    pq.entries.push_back(Entry { priority, value });
    let index = pq.entries.length() - 1;
    restore_heap_recursive(&mut pq.entries, index);
}

public fun new_entry<T: drop>(priority: u64, value: T): Entry<T> {
    Entry { priority, value }
}

public fun create_entries<T: drop>(mut p: vector<u64>, mut v: vector<T>): vector<Entry<T>> {
    let len = p.length();
    assert!(v.length() == len, 0);
    let mut res = vector[];
    let mut i = 0;
    while (i < len) {
        let priority = p.remove(0);
        let value = v.remove(0);
        res.push_back(Entry { priority, value });
        i = i + 1;
    };
    res
}

// TODO: implement iterative version too and see performance difference.
fun restore_heap_recursive<T: drop>(v: &mut vector<Entry<T>>, i: u64) {
    if (i == 0) {
        return
    };
    let parent = (i - 1) / 2;

    // If new elem is greater than its parent, swap them and recursively
    // do the restoration upwards.
    if (*&v[i].priority > *&v[parent].priority) {
        v.swap(i, parent);
        restore_heap_recursive(v, parent);
    }
}

/// Max heapify the subtree whose root is at index `i`. That means after this function
/// finishes, the subtree should have the property that the parent node has higher priority
/// than both child nodes.
/// This function assumes that all the other nodes in the subtree (nodes other than the root)
/// do satisfy the max heap property.
fun max_heapify_recursive<T: drop>(v: &mut vector<Entry<T>>, len: u64, i: u64) {
    if (len == 0) {
        return
    };
    assert!(i < len, 1);
    let left = i * 2 + 1;
    let right = left + 1;
    let mut max = i;
    // Find the node with highest priority among node `i` and its two children.
    if (left < len && *&v[left].priority > *&v[max].priority) {
        max = left;
    };
    if (right < len && *&v[right].priority > *&v[max].priority) {
        max = right;
    };
    // If the parent node (node `i`) doesn't have the highest priority, we swap the parent with the
    // max priority node.
    if (max != i) {
        v.swap(max, i);
        // After the swap, we have restored the property at node `i` but now the max heap property
        // may be violated at node `max` since this node now has a new value. So we need to now
        // max heapify the subtree rooted at node `max`.
        max_heapify_recursive(v, len, max);
    }
}

public fun priorities<T: drop>(pq: &PriorityQueue<T>): vector<u64> {
    let mut res = vector[];
    let mut i = 0;
    while (i < pq.entries.length()) {
        res.push_back(pq.entries[i].priority);
        i = i +1;
    };
    res
}

#[test]
fun test_pq() {
    let mut h = new(create_entries(vector[3, 1, 4, 2, 5, 2], vector[10, 20, 30, 40, 50, 60]));
    check_pop_max(&mut h, 5, 50);
    check_pop_max(&mut h, 4, 30);
    check_pop_max(&mut h, 3, 10);
    insert(&mut h, 7, 70);
    check_pop_max(&mut h, 7, 70);
    check_pop_max(&mut h, 2, 40);
    insert(&mut h, 0, 80);
    check_pop_max(&mut h, 2, 60);
    check_pop_max(&mut h, 1, 20);
    check_pop_max(&mut h, 0, 80);

    let mut h = new(create_entries(vector[5, 3, 1, 2, 4], vector[10, 20, 30, 40, 50]));
    check_pop_max(&mut h, 5, 10);
    check_pop_max(&mut h, 4, 50);
    check_pop_max(&mut h, 3, 20);
    check_pop_max(&mut h, 2, 40);
    check_pop_max(&mut h, 1, 30);
}

#[test]
fun test_swap_remove_edge_case() {
    // This test would fail if `remove` is used incorrectly instead of `swap_remove` in `pop_max`.
    // It's hard to characterize exactly under what condition this bug is triggered but roughly
    // it happens when the entire tree vector is shifted left by one because of the incorrect usage
    // of `remove`, and the resulting new root and its two children appear to satisfy the heap invariant
    // so we stop max-heapifying there, while the rest of the tree is all messed up because of the shift.
    let priorities = vector[8, 7, 3, 6, 2, 1, 0, 5, 4];
    let values = vector[0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut h = new(create_entries(priorities, values));
    check_pop_max(&mut h, 8, 0);
    check_pop_max(&mut h, 7, 0);
    check_pop_max(&mut h, 6, 0);
    check_pop_max(&mut h, 5, 0);
    check_pop_max(&mut h, 4, 0);
    check_pop_max(&mut h, 3, 0);
    check_pop_max(&mut h, 2, 0);
    check_pop_max(&mut h, 1, 0);
    check_pop_max(&mut h, 0, 0);
}

#[test_only]
fun check_pop_max(h: &mut PriorityQueue<u64>, expected_priority: u64, expected_value: u64) {
    let (priority, value) = pop_max(h);
    assert!(priority == expected_priority);
    assert!(value == expected_value);
}
