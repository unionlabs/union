/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 * Modifications copyright (C) 2017 Andy Kimball and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package arenaskl

import (
	"sync"

	"github.com/cockroachdb/pebble/internal/base"
)

type splice struct {
	prev *node
	next *node
}

func (s *splice) init(prev, next *node) {
	s.prev = prev
	s.next = next
}

// Iterator is an iterator over the skiplist object. Use Skiplist.NewIter
// to construct an iterator. The current state of the iterator can be cloned by
// simply value copying the struct. All iterator methods are thread-safe.
type Iterator struct {
	list  *Skiplist
	nd    *node
	key   base.InternalKey
	lower []byte
	upper []byte
}

// Iterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*Iterator)(nil)

var iterPool = sync.Pool{
	New: func() interface{} {
		return &Iterator{}
	},
}

// Close resets the iterator.
func (it *Iterator) Close() error {
	it.list = nil
	it.nd = nil
	it.lower = nil
	it.upper = nil
	iterPool.Put(it)
	return nil
}

func (it *Iterator) String() string {
	return "memtable"
}

// Error returns any accumulated error.
func (it *Iterator) Error() error {
	return nil
}

// SeekGE moves the iterator to the first entry whose key is greater than or
// equal to the given key. Returns the key and value if the iterator is
// pointing at a valid entry, and (nil, nil) otherwise. Note that SeekGE only
// checks the upper bound. It is up to the caller to ensure that key is greater
// than or equal to the lower bound.
func (it *Iterator) SeekGE(key []byte, flags base.SeekGEFlags) (*base.InternalKey, base.LazyValue) {
	if flags.TrySeekUsingNext() {
		if it.nd == it.list.tail {
			// Iterator is done.
			return nil, base.LazyValue{}
		}
		less := it.list.cmp(it.key.UserKey, key) < 0
		// Arbitrary constant. By measuring the seek cost as a function of the
		// number of elements in the skip list, and fitting to a model, we
		// could adjust the number of nexts based on the current size of the
		// skip list.
		const numNexts = 5
		for i := 0; less && i < numNexts; i++ {
			k, _ := it.Next()
			if k == nil {
				// Iterator is done.
				return nil, base.LazyValue{}
			}
			less = it.list.cmp(it.key.UserKey, key) < 0
		}
		if !less {
			return &it.key, base.MakeInPlaceValue(it.value())
		}
	}
	_, it.nd, _ = it.seekForBaseSplice(key)
	if it.nd == it.list.tail {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.upper != nil && it.list.cmp(it.upper, it.key.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// SeekPrefixGE moves the iterator to the first entry whose key is greater than
// or equal to the given key. This method is equivalent to SeekGE and is
// provided so that an arenaskl.Iterator implements the
// internal/base.InternalIterator interface.
func (it *Iterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	return it.SeekGE(key, flags)
}

// SeekLT moves the iterator to the last entry whose key is less than the given
// key. Returns the key and value if the iterator is pointing at a valid entry,
// and (nil, nil) otherwise. Note that SeekLT only checks the lower bound. It
// is up to the caller to ensure that key is less than the upper bound.
func (it *Iterator) SeekLT(key []byte, flags base.SeekLTFlags) (*base.InternalKey, base.LazyValue) {
	// NB: the top-level Iterator has already adjusted key based on
	// the upper-bound.
	it.nd, _, _ = it.seekForBaseSplice(key)
	if it.nd == it.list.head {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.lower != nil && it.list.cmp(it.lower, it.key.UserKey) > 0 {
		it.nd = it.list.head
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// First seeks position at the first entry in list. Returns the key and value
// if the iterator is pointing at a valid entry, and (nil, nil) otherwise. Note
// that First only checks the upper bound. It is up to the caller to ensure
// that key is greater than or equal to the lower bound (e.g. via a call to SeekGE(lower)).
func (it *Iterator) First() (*base.InternalKey, base.LazyValue) {
	it.nd = it.list.getNext(it.list.head, 0)
	if it.nd == it.list.tail {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.upper != nil && it.list.cmp(it.upper, it.key.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// Last seeks position at the last entry in list. Returns the key and value if
// the iterator is pointing at a valid entry, and (nil, nil) otherwise. Note
// that Last only checks the lower bound. It is up to the caller to ensure that
// key is less than the upper bound (e.g. via a call to SeekLT(upper)).
func (it *Iterator) Last() (*base.InternalKey, base.LazyValue) {
	it.nd = it.list.getPrev(it.list.tail, 0)
	if it.nd == it.list.head {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.lower != nil && it.list.cmp(it.lower, it.key.UserKey) > 0 {
		it.nd = it.list.head
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// Next advances to the next position. Returns the key and value if the
// iterator is pointing at a valid entry, and (nil, nil) otherwise.
// Note: flushIterator.Next mirrors the implementation of Iterator.Next
// due to performance. Keep the two in sync.
func (it *Iterator) Next() (*base.InternalKey, base.LazyValue) {
	it.nd = it.list.getNext(it.nd, 0)
	if it.nd == it.list.tail {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.upper != nil && it.list.cmp(it.upper, it.key.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// NextPrefix advances to the next position with a new prefix. Returns the key
// and value if the iterator is pointing at a valid entry, and (nil, nil)
// otherwise.
func (it *Iterator) NextPrefix(succKey []byte) (*base.InternalKey, base.LazyValue) {
	return it.SeekGE(succKey, base.SeekGEFlagsNone.EnableTrySeekUsingNext())
}

// Prev moves to the previous position. Returns the key and value if the
// iterator is pointing at a valid entry, and (nil, nil) otherwise.
func (it *Iterator) Prev() (*base.InternalKey, base.LazyValue) {
	it.nd = it.list.getPrev(it.nd, 0)
	if it.nd == it.list.head {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	if it.lower != nil && it.list.cmp(it.lower, it.key.UserKey) > 0 {
		it.nd = it.list.head
		return nil, base.LazyValue{}
	}
	return &it.key, base.MakeInPlaceValue(it.value())
}

// value returns the value at the current position.
func (it *Iterator) value() []byte {
	return it.nd.getValue(it.list.arena)
}

// Head true iff the iterator is positioned at the sentinel head node.
func (it *Iterator) Head() bool {
	return it.nd == it.list.head
}

// Tail true iff the iterator is positioned at the sentinel tail node.
func (it *Iterator) Tail() bool {
	return it.nd == it.list.tail
}

// SetBounds sets the lower and upper bounds for the iterator. Note that the
// result of Next and Prev will be undefined until the iterator has been
// repositioned with SeekGE, SeekPrefixGE, SeekLT, First, or Last.
func (it *Iterator) SetBounds(lower, upper []byte) {
	it.lower = lower
	it.upper = upper
}

func (it *Iterator) decodeKey() {
	it.key.UserKey = it.list.arena.getBytes(it.nd.keyOffset, it.nd.keySize)
	it.key.Trailer = it.nd.keyTrailer
}

func (it *Iterator) seekForBaseSplice(key []byte) (prev, next *node, found bool) {
	ikey := base.MakeSearchKey(key)
	level := int(it.list.Height() - 1)

	prev = it.list.head
	for {
		prev, next, found = it.list.findSpliceForLevel(ikey, level, prev)

		if found {
			if level != 0 {
				// next is pointing at the target node, but we need to find previous on
				// the bottom level.
				prev = it.list.getPrev(next, 0)
			}
			break
		}

		if level == 0 {
			break
		}

		level--
	}

	return
}
