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

import "github.com/cockroachdb/pebble/internal/base"

// flushIterator is an iterator over the skiplist object. Use Skiplist.NewFlushIter
// to construct an iterator. The current state of the iterator can be cloned by
// simply value copying the struct.
type flushIterator struct {
	Iterator
	bytesIterated *uint64
}

// flushIterator implements the base.InternalIterator interface.
var _ base.InternalIterator = (*flushIterator)(nil)

func (it *flushIterator) String() string {
	return "memtable"
}

func (it *flushIterator) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekGE unimplemented")
}

func (it *flushIterator) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekPrefixGE unimplemented")
}

func (it *flushIterator) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("pebble: SeekLT unimplemented")
}

// First seeks position at the first entry in list. Returns the key and value
// if the iterator is pointing at a valid entry, and (nil, nil) otherwise. Note
// that First only checks the upper bound. It is up to the caller to ensure
// that key is greater than or equal to the lower bound.
func (it *flushIterator) First() (*base.InternalKey, base.LazyValue) {
	key, val := it.Iterator.First()
	if key == nil {
		return nil, base.LazyValue{}
	}
	*it.bytesIterated += uint64(it.nd.allocSize)
	return key, val
}

// Next advances to the next position. Returns the key and value if the
// iterator is pointing at a valid entry, and (nil, nil) otherwise.
// Note: flushIterator.Next mirrors the implementation of Iterator.Next
// due to performance. Keep the two in sync.
func (it *flushIterator) Next() (*base.InternalKey, base.LazyValue) {
	it.nd = it.list.getNext(it.nd, 0)
	if it.nd == it.list.tail {
		return nil, base.LazyValue{}
	}
	it.decodeKey()
	*it.bytesIterated += uint64(it.nd.allocSize)
	return &it.key, base.MakeInPlaceValue(it.value())
}

func (it *flushIterator) NextPrefix(succKey []byte) (*base.InternalKey, base.LazyValue) {
	panic("pebble: NextPrefix unimplemented")
}

func (it *flushIterator) Prev() (*base.InternalKey, base.LazyValue) {
	panic("pebble: Prev unimplemented")
}
