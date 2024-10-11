// Copyright 2016 The Hashicorp vault Authors.
// This file is part of the Hashicorp vault library.

// Package queue provides Vault plugins with a Priority Queue. It can be used
// as an in-memory list of queue.Item sorted by their priority, and offers
// methods to find or remove items by their key. Internally it uses
// container/heap; see Example Priority Queue:
// https://golang.org/pkg/container/heap/#example__priorityQueue
package queue

import (
	"container/heap"
	"errors"
	"sync"
)

// ErrEmpty is returned for queues with no items
var ErrEmpty = errors.New("queue is empty")

// ErrDuplicateItem is returned when the queue attempts to push an item to a key that
// already exists. The queue does not attempt to update, instead returns this
// error. If an Item needs to be updated or replaced, pop the item first.
var ErrDuplicateItem = errors.New("duplicate item")

// New initializes the internal data structures and returns a new
// PriorityQueue
func New() *PriorityQueue {
	pq := PriorityQueue{
		data:    make(queue, 0),
		dataMap: make(map[string]*Item),
	}
	heap.Init(&pq.data)
	return &pq
}

// PriorityQueue facilitates queue of Items, providing Push, Pop, and
// PopByKey convenience methods. The ordering (priority) is an int64 value
// with the smallest value is the highest priority. PriorityQueue maintains both
// an internal slice for the queue as well as a map of the same items with their
// keys as the index. This enables users to find specific items by key. The map
// must be kept in sync with the data slice.
// See https://golang.org/pkg/container/heap/#example__priorityQueue
type PriorityQueue struct {
	// data is the internal structure that holds the queue, and is operated on by
	// heap functions
	data queue

	// dataMap represents all the items in the queue, with unique indexes, used
	// for finding specific items. dataMap is kept in sync with the data slice
	dataMap map[string]*Item

	// lock is a read/write mutex, and used to facilitate read/write locks on the
	// data and dataMap fields
	lock sync.RWMutex
}

// queue is the internal data structure used to satisfy heap.Interface. This
// prevents users from calling Pop and Push heap methods directly
type queue []*Item

// Item is something managed in the priority queue
type Item struct {
	// Key is a unique string used to identify items in the internal data map
	Key string
	// Value is an unspecified type that implementations can use to store
	// information
	Value interface{}

	// Priority determines ordering in the queue, with the lowest value being the
	// highest priority
	Priority int64

	// index is an internal value used by the heap package, and should not be
	// modified by any consumer of the priority queue
	index int
}

// Len returns the count of items in the Priority Queue
func (pq *PriorityQueue) Len() int {
	pq.lock.RLock()
	defer pq.lock.RUnlock()
	return pq.data.Len()
}

// Pop pops the highest priority item from the queue. This is a
// wrapper/convenience method that calls heap.Pop, so consumers do not need to
// invoke heap functions directly
func (pq *PriorityQueue) Pop() (*Item, error) {
	if pq.Len() == 0 {
		return nil, ErrEmpty
	}

	pq.lock.Lock()
	defer pq.lock.Unlock()

	item, ok := heap.Pop(&pq.data).(*Item)
	if !ok {
		return nil, errors.New("unknown type")
	}
	delete(pq.dataMap, item.Key)
	return item, nil
}

// Push pushes an item on to the queue. This is a wrapper/convenience
// method that calls heap.Push, so consumers do not need to invoke heap
// functions directly. Items must have unique Keys, and Items in the queue
// cannot be updated. To modify an Item, users must first remove it and re-push
// it after modifications. Item does not get copied before pushing on to the queue,
// it's up to the caller to copy the item.
func (pq *PriorityQueue) Push(i *Item) error {
	if i == nil || i.Key == "" {
		return errors.New("error adding item: Item Key is required")
	}

	pq.lock.Lock()
	defer pq.lock.Unlock()

	if _, ok := pq.dataMap[i.Key]; ok {
		return ErrDuplicateItem
	}

	pq.dataMap[i.Key] = i
	heap.Push(&pq.data, i)
	return nil
}

// PopByKey searches the queue for an item with the given key and removes it
// from the queue if found. Returns nil if not found. This method must fix the
// queue after removing any key.
func (pq *PriorityQueue) PopByKey(key string) (*Item, error) {
	pq.lock.Lock()
	defer pq.lock.Unlock()

	item, ok := pq.dataMap[key]
	if !ok {
		return nil, nil
	}

	// Remove the item the heap and delete it from the dataMap
	itemRaw := heap.Remove(&pq.data, item.index)
	delete(pq.dataMap, key)

	if itemRaw != nil {
		if i, ok := itemRaw.(*Item); ok {
			return i, nil
		}
	}

	return nil, nil
}

// RetrieveByKey searches the queue for an item with the given key and returns it
// from the queue if found. Returns nil if not found.
func (pq *PriorityQueue) RetrieveByKey(key string) *Item {
	pq.lock.RLock()
	defer pq.lock.RUnlock()

	item, ok := pq.dataMap[key]
	if !ok {
		return nil
	}

	return pq.data[item.index]
}

// Len returns the number of items in the queue data structure. Do not use this
// method directly on the queue, use PriorityQueue.Len() instead.
func (q queue) Len() int { return len(q) }

// Less returns whether the Item with index i should sort before the Item with
// index j in the queue. This method is used by the queue to determine priority
// internally; the Item with the lower value wins. (priority zero is higher
// priority than 1). The priority of Items with equal values is undetermined.
func (q queue) Less(i, j int) bool {
	return q[i].Priority < q[j].Priority
}

// Swap swaps things in-place; part of sort.Interface
func (q queue) Swap(i, j int) {
	q[i], q[j] = q[j], q[i]
	q[i].index = i
	q[j].index = j
}

// Push is used by heap.Interface to push items onto the heap. This method is
// invoked by container/heap, and should not be used directly.
// See: https://golang.org/pkg/container/heap/#Interface
func (q *queue) Push(x interface{}) {
	n := len(*q)
	item, ok := x.(*Item)
	if !ok {
		return
	}
	item.index = n
	*q = append(*q, item)
}

// Pop is used by heap.Interface to pop items off of the heap. This method is
// invoked by container/heap, and should not be used directly.
// See: https://golang.org/pkg/container/heap/#Interface
func (q *queue) Pop() interface{} {
	old := *q
	n := len(old)
	item := old[n-1]
	old[n-1] = nil  // avoid memory leak
	item.index = -1 // for safety
	*q = old[0 : n-1]
	return item
}
