// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
)

type errorIter struct {
	err error
}

// errorIter implements the base.InternalIterator interface.
var _ internalIterator = (*errorIter)(nil)

func newErrorIter(err error) *errorIter {
	return &errorIter{err: err}
}

func (c *errorIter) SeekGE(key []byte, flags base.SeekGEFlags) (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) SeekLT(key []byte, flags base.SeekLTFlags) (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) First() (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) Last() (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) Next() (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) Prev() (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) NextPrefix([]byte) (*InternalKey, base.LazyValue) {
	return nil, base.LazyValue{}
}

func (c *errorIter) Error() error {
	return c.err
}

func (c *errorIter) Close() error {
	return c.err
}

func (c *errorIter) String() string {
	return "error"
}

func (c *errorIter) SetBounds(lower, upper []byte) {}

type errorKeyspanIter struct {
	err error
}

// errorKeyspanIter implements the keyspan.FragmentIterator interface.
var _ keyspan.FragmentIterator = (*errorKeyspanIter)(nil)

func newErrorKeyspanIter(err error) *errorKeyspanIter {
	return &errorKeyspanIter{err: err}
}

func (*errorKeyspanIter) SeekGE(key []byte) *keyspan.Span { return nil }
func (*errorKeyspanIter) SeekLT(key []byte) *keyspan.Span { return nil }
func (*errorKeyspanIter) First() *keyspan.Span            { return nil }
func (*errorKeyspanIter) Last() *keyspan.Span             { return nil }
func (*errorKeyspanIter) Next() *keyspan.Span             { return nil }
func (*errorKeyspanIter) Prev() *keyspan.Span             { return nil }
func (i *errorKeyspanIter) Error() error                  { return i.err }
func (i *errorKeyspanIter) Close() error                  { return i.err }
func (*errorKeyspanIter) String() string                  { return "error" }
