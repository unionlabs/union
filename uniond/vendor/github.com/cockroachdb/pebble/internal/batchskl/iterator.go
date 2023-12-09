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

package batchskl

import "github.com/cockroachdb/pebble/internal/base"

type splice struct {
	prev uint32
	next uint32
}

// Iterator is an iterator over the skiplist object. Use Skiplist.NewIter
// to construct an iterator. The current state of the iterator can be cloned
// by simply value copying the struct.
type Iterator struct {
	list  *Skiplist
	nd    uint32
	key   base.InternalKey
	lower []byte
	upper []byte
}

// Close resets the iterator.
func (it *Iterator) Close() error {
	it.list = nil
	it.nd = 0
	return nil
}

// SeekGE moves the iterator to the first entry whose key is greater than or
// equal to the given key. Returns true if the iterator is pointing at a valid
// entry and false otherwise. Note that SeekGE only checks the upper bound. It
// is up to the caller to ensure that key is greater than or equal to the lower
// bound.
func (it *Iterator) SeekGE(key []byte, flags base.SeekGEFlags) *base.InternalKey {
	if flags.TrySeekUsingNext() {
		if it.nd == it.list.tail {
			// Iterator is done.
			return nil
		}
		less := it.list.cmp(it.key.UserKey, key) < 0
		// Arbitrary constant. By measuring the seek cost as a function of the
		// number of elements in the skip list, and fitting to a model, we
		// could adjust the number of nexts based on the current size of the
		// skip list.
		const numNexts = 5
		for i := 0; less && i < numNexts; i++ {
			k := it.Next()
			if k == nil {
				// Iterator is done.
				return nil
			}
			less = it.list.cmp(k.UserKey, key) < 0
		}
		if !less {
			return &it.key
		}
	}

	_, it.nd = it.seekForBaseSplice(key, it.list.abbreviatedKey(key))
	if it.nd == it.list.tail {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.upper != nil && it.list.cmp(it.upper, nodeKey.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// SeekLT moves the iterator to the last entry whose key is less the given
// key. Returns true if the iterator is pointing at a valid entry and false
// otherwise. Note that SeekLT only checks the lower bound. It is up to the
// caller to ensure that key is less than the upper bound.
func (it *Iterator) SeekLT(key []byte) *base.InternalKey {
	it.nd, _ = it.seekForBaseSplice(key, it.list.abbreviatedKey(key))
	if it.nd == it.list.head {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.lower != nil && it.list.cmp(it.lower, nodeKey.UserKey) > 0 {
		it.nd = it.list.head
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// First seeks position at the first entry in list. Final state of iterator is
// Valid() iff list is not empty. Note that First only checks the upper
// bound. It is up to the caller to ensure that key is greater than or equal to
// the lower bound (e.g. via a call to SeekGE(lower)).
func (it *Iterator) First() *base.InternalKey {
	it.nd = it.list.getNext(it.list.head, 0)
	if it.nd == it.list.tail {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.upper != nil && it.list.cmp(it.upper, nodeKey.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// Last seeks position at the last entry in list. Final state of iterator is
// Valid() iff list is not empty. Note that Last only checks the lower
// bound. It is up to the caller to ensure that key is less than the upper
// bound (e.g. via a call to SeekLT(upper)).
func (it *Iterator) Last() *base.InternalKey {
	it.nd = it.list.getPrev(it.list.tail, 0)
	if it.nd == it.list.head {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.lower != nil && it.list.cmp(it.lower, nodeKey.UserKey) > 0 {
		it.nd = it.list.head
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// Next advances to the next position. If there are no following nodes, then
// Valid() will be false after this call.
func (it *Iterator) Next() *base.InternalKey {
	it.nd = it.list.getNext(it.nd, 0)
	if it.nd == it.list.tail {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.upper != nil && it.list.cmp(it.upper, nodeKey.UserKey) <= 0 {
		it.nd = it.list.tail
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// Prev moves to the previous position. If there are no previous nodes, then
// Valid() will be false after this call.
func (it *Iterator) Prev() *base.InternalKey {
	it.nd = it.list.getPrev(it.nd, 0)
	if it.nd == it.list.head {
		return nil
	}
	nodeKey := it.list.getKey(it.nd)
	if it.lower != nil && it.list.cmp(it.lower, nodeKey.UserKey) > 0 {
		it.nd = it.list.head
		return nil
	}
	it.key = nodeKey
	return &it.key
}

// Key returns the key at the current position.
func (it *Iterator) Key() *base.InternalKey {
	return &it.key
}

// KeyInfo returns the offset of the start of the record, the start of the key,
// and the end of the key.
func (it *Iterator) KeyInfo() (offset, keyStart, keyEnd uint32) {
	n := it.list.node(it.nd)
	return n.offset, n.keyStart, n.keyEnd
}

// Head true iff the iterator is positioned at the sentinel head node.
func (it *Iterator) Head() bool {
	return it.nd == it.list.head
}

// Tail true iff the iterator is positioned at the sentinel tail node.
func (it *Iterator) Tail() bool {
	return it.nd == it.list.tail
}

// Valid returns nil iff the iterator is positioned at a valid node.
func (it *Iterator) Valid() bool {
	return it.list != nil && it.nd != it.list.head && it.nd != it.list.tail
}

func (it *Iterator) String() string {
	return "batch"
}

// SetBounds sets the lower and upper bounds for the iterator. Note that the
// result of Next and Prev will be undefined until the iterator has been
// repositioned with SeekGE, SeekLT, First, or Last.
func (it *Iterator) SetBounds(lower, upper []byte) {
	it.lower = lower
	it.upper = upper
}

func (it *Iterator) seekForBaseSplice(key []byte, abbreviatedKey uint64) (prev, next uint32) {
	prev = it.list.head
	for level := it.list.height - 1; ; level-- {
		prev, next = it.list.findSpliceForLevel(key, abbreviatedKey, level, prev)
		if level == 0 {
			break
		}
	}

	return
}
