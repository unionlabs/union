// Copyright (c) 2012, Suryandaru Triandana <syndtr@gmail.com>
// All rights reserved.
//
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Package cache provides interface and implementation of a cache algorithms.
package cache

import (
	"sort"
	"sync"
	"sync/atomic"
	"unsafe"

	"github.com/syndtr/goleveldb/leveldb/util"
)

// Cacher provides interface to implements a caching functionality.
// An implementation must be safe for concurrent use.
type Cacher interface {
	// Capacity returns cache capacity.
	Capacity() int

	// SetCapacity sets cache capacity.
	SetCapacity(capacity int)

	// Promote promotes the 'cache node'.
	Promote(n *Node)

	// Ban evicts the 'cache node' and prevent subsequent 'promote'.
	Ban(n *Node)

	// Evict evicts the 'cache node'.
	Evict(n *Node)
}

// Value is a 'cache-able object'. It may implements util.Releaser, if
// so the the Release method will be called once object is released.
type Value interface{}

// NamespaceGetter provides convenient wrapper for namespace.
type NamespaceGetter struct {
	Cache *Cache
	NS    uint64
}

// Get simply calls Cache.Get() method.
func (g *NamespaceGetter) Get(key uint64, setFunc func() (size int, value Value)) *Handle {
	return g.Cache.Get(g.NS, key, setFunc)
}

// The hash tables implementation is based on:
// "Dynamic-Sized Nonblocking Hash Tables", by Yujie Liu,
// Kunlong Zhang, and Michael Spear.
// ACM Symposium on Principles of Distributed Computing, Jul 2014.

const (
	mInitialSize           = 1 << 4
	mOverflowThreshold     = 1 << 5
	mOverflowGrowThreshold = 1 << 7
)

const (
	bucketUninitialized = iota
	bucketInitialized
	bucketFrozen
)

type mNodes []*Node

func (x mNodes) Len() int { return len(x) }
func (x mNodes) Less(i, j int) bool {
	a, b := x[i].ns, x[j].ns
	if a == b {
		return x[i].key < x[j].key
	}
	return a < b
}
func (x mNodes) Swap(i, j int) { x[i], x[j] = x[j], x[i] }

func (x mNodes) sort() { sort.Sort(x) }

func (x mNodes) search(ns, key uint64) int {
	return sort.Search(len(x), func(i int) bool {
		a := x[i].ns
		if a == ns {
			return x[i].key >= key
		}
		return a > ns
	})
}

type mBucket struct {
	mu    sync.Mutex
	nodes mNodes
	state int8
}

func (b *mBucket) freeze() mNodes {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.state == bucketInitialized {
		b.state = bucketFrozen
	} else if b.state == bucketUninitialized {
		panic("BUG: freeze uninitialized bucket")
	}
	return b.nodes
}

func (b *mBucket) frozen() bool {
	if b.state == bucketFrozen {
		return true
	}
	if b.state == bucketUninitialized {
		panic("BUG: accessing uninitialized bucket")
	}
	return false
}

func (b *mBucket) get(r *Cache, h *mHead, hash uint32, ns, key uint64, getOnly bool) (done, created bool, n *Node) {
	b.mu.Lock()

	if b.frozen() {
		b.mu.Unlock()
		return
	}

	// Find the node.
	i := b.nodes.search(ns, key)
	if i < len(b.nodes) {
		n = b.nodes[i]
		if n.ns == ns && n.key == key {
			atomic.AddInt32(&n.ref, 1)
			b.mu.Unlock()
			return true, false, n
		}
	}

	// Get only.
	if getOnly {
		b.mu.Unlock()
		return true, false, nil
	}

	// Create node.
	n = &Node{
		r:    r,
		hash: hash,
		ns:   ns,
		key:  key,
		ref:  1,
	}
	// Add node to bucket.
	if i == len(b.nodes) {
		b.nodes = append(b.nodes, n)
	} else {
		b.nodes = append(b.nodes[:i+1], b.nodes[i:]...)
		b.nodes[i] = n
	}
	bLen := len(b.nodes)
	b.mu.Unlock()

	// Update counter.
	grow := atomic.AddInt64(&r.statNodes, 1) >= h.growThreshold
	if bLen > mOverflowThreshold {
		grow = grow || atomic.AddInt32(&h.overflow, 1) >= mOverflowGrowThreshold
	}

	// Grow.
	if grow && atomic.CompareAndSwapInt32(&h.resizeInProgress, 0, 1) {
		nhLen := len(h.buckets) << 1
		nh := &mHead{
			buckets:         make([]mBucket, nhLen),
			mask:            uint32(nhLen) - 1,
			predecessor:     unsafe.Pointer(h),
			growThreshold:   int64(nhLen * mOverflowThreshold),
			shrinkThreshold: int64(nhLen >> 1),
		}
		ok := atomic.CompareAndSwapPointer(&r.mHead, unsafe.Pointer(h), unsafe.Pointer(nh))
		if !ok {
			panic("BUG: failed swapping head")
		}
		atomic.AddInt32(&r.statGrow, 1)
		go nh.initBuckets()
	}

	return true, true, n
}

func (b *mBucket) delete(r *Cache, h *mHead, hash uint32, ns, key uint64) (done, deleted bool) {
	b.mu.Lock()

	if b.frozen() {
		b.mu.Unlock()
		return
	}

	// Find the node.
	i := b.nodes.search(ns, key)
	if i == len(b.nodes) {
		b.mu.Unlock()
		return true, false
	}
	n := b.nodes[i]
	var bLen int
	if n.ns == ns && n.key == key {
		if atomic.LoadInt32(&n.ref) == 0 {
			deleted = true

			// Save and clear value.
			if n.value != nil {
				// Call releaser.
				if r, ok := n.value.(util.Releaser); ok {
					r.Release()
				}
				n.value = nil
			}

			// Remove node from bucket.
			b.nodes = append(b.nodes[:i], b.nodes[i+1:]...)
			bLen = len(b.nodes)
		}
	}
	b.mu.Unlock()

	if deleted {
		// Call delete funcs.
		for _, f := range n.delFuncs {
			f()
		}

		// Update counter.
		atomic.AddInt64(&r.statSize, int64(n.size)*-1)
		shrink := atomic.AddInt64(&r.statNodes, -1) < h.shrinkThreshold
		if bLen >= mOverflowThreshold {
			atomic.AddInt32(&h.overflow, -1)
		}

		// Shrink.
		if shrink && len(h.buckets) > mInitialSize && atomic.CompareAndSwapInt32(&h.resizeInProgress, 0, 1) {
			nhLen := len(h.buckets) >> 1
			nh := &mHead{
				buckets:         make([]mBucket, nhLen),
				mask:            uint32(nhLen) - 1,
				predecessor:     unsafe.Pointer(h),
				growThreshold:   int64(nhLen * mOverflowThreshold),
				shrinkThreshold: int64(nhLen >> 1),
			}
			ok := atomic.CompareAndSwapPointer(&r.mHead, unsafe.Pointer(h), unsafe.Pointer(nh))
			if !ok {
				panic("BUG: failed swapping head")
			}
			atomic.AddInt32(&r.statShrink, 1)
			go nh.initBuckets()
		}
	}

	return true, deleted
}

type mHead struct {
	buckets          []mBucket
	mask             uint32
	predecessor      unsafe.Pointer // *mNode
	resizeInProgress int32

	overflow        int32
	growThreshold   int64
	shrinkThreshold int64
}

func (h *mHead) initBucket(i uint32) *mBucket {
	b := &h.buckets[i]
	b.mu.Lock()
	if b.state >= bucketInitialized {
		b.mu.Unlock()
		return b
	}

	p := (*mHead)(atomic.LoadPointer(&h.predecessor))
	if p == nil {
		panic("BUG: uninitialized bucket doesn't have predecessor")
	}

	var nodes mNodes
	if h.mask > p.mask {
		// Grow.
		m := p.initBucket(i & p.mask).freeze()
		// Split nodes.
		for _, x := range m {
			if x.hash&h.mask == i {
				nodes = append(nodes, x)
			}
		}
	} else {
		// Shrink.
		m0 := p.initBucket(i).freeze()
		m1 := p.initBucket(i + uint32(len(h.buckets))).freeze()
		// Merge nodes.
		nodes = make(mNodes, 0, len(m0)+len(m1))
		nodes = append(nodes, m0...)
		nodes = append(nodes, m1...)
		nodes.sort()
	}
	b.nodes = nodes
	b.state = bucketInitialized
	b.mu.Unlock()
	return b
}

func (h *mHead) initBuckets() {
	for i := range h.buckets {
		h.initBucket(uint32(i))
	}
	atomic.StorePointer(&h.predecessor, nil)
}

func (h *mHead) enumerateNodesWithCB(f func([]*Node)) {
	var nodes []*Node
	for x := range h.buckets {
		b := h.initBucket(uint32(x))

		b.mu.Lock()
		nodes = append(nodes, b.nodes...)
		b.mu.Unlock()
		f(nodes)
	}
}

func (h *mHead) enumerateNodesByNS(ns uint64) []*Node {
	var nodes []*Node
	for x := range h.buckets {
		b := h.initBucket(uint32(x))

		b.mu.Lock()
		i := b.nodes.search(ns, 0)
		for ; i < len(b.nodes); i++ {
			n := b.nodes[i]
			if n.ns != ns {
				break
			}
			nodes = append(nodes, n)
		}
		b.mu.Unlock()
	}
	return nodes
}

type Stats struct {
	Buckets     int
	Nodes       int64
	Size        int64
	GrowCount   int32
	ShrinkCount int32
	HitCount    int64
	MissCount   int64
	SetCount    int64
	DelCount    int64
}

// Cache is a 'cache map'.
type Cache struct {
	mu     sync.RWMutex
	mHead  unsafe.Pointer // *mNode
	cacher Cacher
	closed bool

	statNodes  int64
	statSize   int64
	statGrow   int32
	statShrink int32
	statHit    int64
	statMiss   int64
	statSet    int64
	statDel    int64
}

// NewCache creates a new 'cache map'. The cacher is optional and
// may be nil.
func NewCache(cacher Cacher) *Cache {
	h := &mHead{
		buckets:         make([]mBucket, mInitialSize),
		mask:            mInitialSize - 1,
		growThreshold:   int64(mInitialSize * mOverflowThreshold),
		shrinkThreshold: 0,
	}
	for i := range h.buckets {
		h.buckets[i].state = bucketInitialized
	}
	r := &Cache{
		mHead:  unsafe.Pointer(h),
		cacher: cacher,
	}
	return r
}

func (r *Cache) getBucket(hash uint32) (*mHead, *mBucket) {
	h := (*mHead)(atomic.LoadPointer(&r.mHead))
	i := hash & h.mask
	return h, h.initBucket(i)
}

func (r *Cache) enumerateNodesWithCB(f func([]*Node)) {
	h := (*mHead)(atomic.LoadPointer(&r.mHead))
	h.enumerateNodesWithCB(f)
}

func (r *Cache) enumerateNodesByNS(ns uint64) []*Node {
	h := (*mHead)(atomic.LoadPointer(&r.mHead))
	return h.enumerateNodesByNS(ns)
}

func (r *Cache) delete(n *Node) bool {
	for {
		h, b := r.getBucket(n.hash)
		done, deleted := b.delete(r, h, n.hash, n.ns, n.key)
		if done {
			return deleted
		}
	}
}

// GetStats returns cache statistics.
func (r *Cache) GetStats() Stats {
	return Stats{
		Buckets:     len((*mHead)(atomic.LoadPointer(&r.mHead)).buckets),
		Nodes:       atomic.LoadInt64(&r.statNodes),
		Size:        atomic.LoadInt64(&r.statSize),
		GrowCount:   atomic.LoadInt32(&r.statGrow),
		ShrinkCount: atomic.LoadInt32(&r.statShrink),
		HitCount:    atomic.LoadInt64(&r.statHit),
		MissCount:   atomic.LoadInt64(&r.statMiss),
		SetCount:    atomic.LoadInt64(&r.statSet),
		DelCount:    atomic.LoadInt64(&r.statDel),
	}
}

// Nodes returns number of 'cache node' in the map.
func (r *Cache) Nodes() int {
	return int(atomic.LoadInt64(&r.statNodes))
}

// Size returns sums of 'cache node' size in the map.
func (r *Cache) Size() int {
	return int(atomic.LoadInt64(&r.statSize))
}

// Capacity returns cache capacity.
func (r *Cache) Capacity() int {
	if r.cacher == nil {
		return 0
	}
	return r.cacher.Capacity()
}

// SetCapacity sets cache capacity.
func (r *Cache) SetCapacity(capacity int) {
	if r.cacher != nil {
		r.cacher.SetCapacity(capacity)
	}
}

// Get gets 'cache node' with the given namespace and key.
// If cache node is not found and setFunc is not nil, Get will atomically creates
// the 'cache node' by calling setFunc. Otherwise Get will returns nil.
//
// The returned 'cache handle' should be released after use by calling Release
// method.
func (r *Cache) Get(ns, key uint64, setFunc func() (size int, value Value)) *Handle {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if r.closed {
		return nil
	}

	hash := murmur32(ns, key, 0xf00)
	for {
		h, b := r.getBucket(hash)
		done, created, n := b.get(r, h, hash, ns, key, setFunc == nil)
		if done {
			if created || n == nil {
				atomic.AddInt64(&r.statMiss, 1)
			} else {
				atomic.AddInt64(&r.statHit, 1)
			}

			if n != nil {
				n.mu.Lock()
				if n.value == nil {
					if setFunc == nil {
						n.mu.Unlock()
						n.unRefInternal(false)
						return nil
					}

					n.size, n.value = setFunc()
					if n.value == nil {
						n.size = 0
						n.mu.Unlock()
						n.unRefInternal(false)
						return nil
					}
					atomic.AddInt64(&r.statSet, 1)
					atomic.AddInt64(&r.statSize, int64(n.size))
				}
				n.mu.Unlock()
				if r.cacher != nil {
					r.cacher.Promote(n)
				}
				return &Handle{unsafe.Pointer(n)}
			}

			break
		}
	}
	return nil
}

// Delete removes and ban 'cache node' with the given namespace and key.
// A banned 'cache node' will never inserted into the 'cache tree'. Ban
// only attributed to the particular 'cache node', so when a 'cache node'
// is recreated it will not be banned.
//
// If delFunc is not nil, then it will be executed if such 'cache node'
// doesn't exist or once the 'cache node' is released.
//
// Delete return true is such 'cache node' exist.
func (r *Cache) Delete(ns, key uint64, delFunc func()) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if r.closed {
		return false
	}

	hash := murmur32(ns, key, 0xf00)
	for {
		h, b := r.getBucket(hash)
		done, _, n := b.get(r, h, hash, ns, key, true)
		if done {
			if n != nil {
				if delFunc != nil {
					n.mu.Lock()
					n.delFuncs = append(n.delFuncs, delFunc)
					n.mu.Unlock()
				}
				if r.cacher != nil {
					r.cacher.Ban(n)
				}
				n.unRefInternal(true)
				return true
			}

			break
		}
	}

	if delFunc != nil {
		delFunc()
	}

	return false
}

// Evict evicts 'cache node' with the given namespace and key. This will
// simply call Cacher.Evict.
//
// Evict return true is such 'cache node' exist.
func (r *Cache) Evict(ns, key uint64) bool {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if r.closed {
		return false
	}

	hash := murmur32(ns, key, 0xf00)
	for {
		h, b := r.getBucket(hash)
		done, _, n := b.get(r, h, hash, ns, key, true)
		if done {
			if n != nil {
				if r.cacher != nil {
					r.cacher.Evict(n)
				}
				n.unRefInternal(true)
				return true
			}

			break
		}
	}

	return false
}

// EvictNS evicts 'cache node' with the given namespace. This will
// simply call Cacher.Evict on all nodes with the given namespace.
func (r *Cache) EvictNS(ns uint64) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if r.closed {
		return
	}

	if r.cacher != nil {
		nodes := r.enumerateNodesByNS(ns)
		for _, n := range nodes {
			r.cacher.Evict(n)
		}
	}
}

func (r *Cache) evictAll() {
	r.enumerateNodesWithCB(func(nodes []*Node) {
		for _, n := range nodes {
			r.cacher.Evict(n)
		}
	})
}

// EvictAll evicts all 'cache node'. This will simply call Cacher.EvictAll.
func (r *Cache) EvictAll() {
	r.mu.RLock()
	defer r.mu.RUnlock()
	if r.closed {
		return
	}

	if r.cacher != nil {
		r.evictAll()
	}
}

// Close closes the 'cache map'.
// All 'Cache' method is no-op after 'cache map' is closed.
// All 'cache node' will be evicted from 'cacher'.
//
// If 'force' is true then all 'cache node' will be forcefully released
// even if the 'node ref' is not zero.
func (r *Cache) Close(force bool) {
	var head *mHead
	// Hold RW-lock to make sure no more in-flight operations.
	r.mu.Lock()
	if !r.closed {
		r.closed = true
		head = (*mHead)(atomic.LoadPointer(&r.mHead))
		atomic.StorePointer(&r.mHead, nil)
	}
	r.mu.Unlock()

	if head != nil {
		head.enumerateNodesWithCB(func(nodes []*Node) {
			for _, n := range nodes {
				// Zeroing ref. Prevent unRefExternal to call finalizer.
				if force {
					atomic.StoreInt32(&n.ref, 0)
				}

				// Evict from cacher.
				if r.cacher != nil {
					r.cacher.Evict(n)
				}

				if force {
					n.callFinalizer()
				}
			}
		})
	}
}

// Node is a 'cache node'.
type Node struct {
	r *Cache

	hash    uint32
	ns, key uint64

	mu    sync.Mutex
	size  int
	value Value

	ref      int32
	delFuncs []func()

	CacheData unsafe.Pointer
}

// NS returns this 'cache node' namespace.
func (n *Node) NS() uint64 {
	return n.ns
}

// Key returns this 'cache node' key.
func (n *Node) Key() uint64 {
	return n.key
}

// Size returns this 'cache node' size.
func (n *Node) Size() int {
	return n.size
}

// Value returns this 'cache node' value.
func (n *Node) Value() Value {
	return n.value
}

// Ref returns this 'cache node' ref counter.
func (n *Node) Ref() int32 {
	return atomic.LoadInt32(&n.ref)
}

// GetHandle returns an handle for this 'cache node'.
func (n *Node) GetHandle() *Handle {
	if atomic.AddInt32(&n.ref, 1) <= 1 {
		panic("BUG: Node.GetHandle on zero ref")
	}
	return &Handle{unsafe.Pointer(n)}
}

func (n *Node) callFinalizer() {
	// Call releaser.
	if n.value != nil {
		if r, ok := n.value.(util.Releaser); ok {
			r.Release()
		}
		n.value = nil
	}

	// Call delete funcs.
	for _, f := range n.delFuncs {
		f()
	}
	n.delFuncs = nil
}

func (n *Node) unRefInternal(updateStat bool) {
	if atomic.AddInt32(&n.ref, -1) == 0 {
		n.r.delete(n)
		if updateStat {
			atomic.AddInt64(&n.r.statDel, 1)
		}
	}
}

func (n *Node) unRefExternal() {
	if atomic.AddInt32(&n.ref, -1) == 0 {
		n.r.mu.RLock()
		if n.r.closed {
			n.callFinalizer()
		} else {
			n.r.delete(n)
			atomic.AddInt64(&n.r.statDel, 1)
		}
		n.r.mu.RUnlock()
	}
}

// Handle is a 'cache handle' of a 'cache node'.
type Handle struct {
	n unsafe.Pointer // *Node
}

// Value returns the value of the 'cache node'.
func (h *Handle) Value() Value {
	n := (*Node)(atomic.LoadPointer(&h.n))
	if n != nil {
		return n.value
	}
	return nil
}

// Release releases this 'cache handle'.
// It is safe to call release multiple times.
func (h *Handle) Release() {
	nPtr := atomic.LoadPointer(&h.n)
	if nPtr != nil && atomic.CompareAndSwapPointer(&h.n, nPtr, nil) {
		n := (*Node)(nPtr)
		n.unRefExternal()
	}
}

func murmur32(ns, key uint64, seed uint32) uint32 {
	const (
		m = uint32(0x5bd1e995)
		r = 24
	)

	k1 := uint32(ns >> 32)
	k2 := uint32(ns)
	k3 := uint32(key >> 32)
	k4 := uint32(key)

	k1 *= m
	k1 ^= k1 >> r
	k1 *= m

	k2 *= m
	k2 ^= k2 >> r
	k2 *= m

	k3 *= m
	k3 ^= k3 >> r
	k3 *= m

	k4 *= m
	k4 ^= k4 >> r
	k4 *= m

	h := seed

	h *= m
	h ^= k1
	h *= m
	h ^= k2
	h *= m
	h ^= k3
	h *= m
	h ^= k4

	h ^= h >> 13
	h *= m
	h ^= h >> 15

	return h
}
