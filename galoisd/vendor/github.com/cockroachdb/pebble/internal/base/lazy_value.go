// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import "github.com/cockroachdb/pebble/internal/invariants"

// A value can have user-defined attributes that are a function of the value
// byte slice. For now, we only support "short attributes", which can be
// encoded in 3 bits. We will likely extend this to "long attributes" later
// for values that are even more expensive to access than those in value
// blocks in the same sstable.
//
// When a sstable writer chooses not to store a value together with the key,
// it can call the ShortAttributeExtractor to extract the attribute and store
// it together with the key. This allows for cheap retrieval of
// AttributeAndLen on the read-path, without doing a more expensive retrieval
// of the value. In general, the extraction code may want to also look at the
// key to decide how to treat the value, hence the key* parameters.
//
// Write path performance: The ShortAttributeExtractor func cannot be inlined,
// so we will pay the cost of this function call. However, we will only pay
// this when (a) the value is not being stored together with the key, and (b)
// the key-value pair is being initially written to the DB, or a compaction is
// transitioning the key-value pair from being stored together to being stored
// separately.

// ShortAttribute encodes a user-specified attribute of the value.
type ShortAttribute uint8

// MaxShortAttribute is the maximum value of the short attribute (3 bits).
const MaxShortAttribute = 7

// ShortAttributeExtractor is an extractor that given the value, will return
// the ShortAttribute.
type ShortAttributeExtractor func(
	key []byte, keyPrefixLen int, value []byte) (ShortAttribute, error)

// AttributeAndLen represents the pair of value length and the short
// attribute.
type AttributeAndLen struct {
	ValueLen       int32
	ShortAttribute ShortAttribute
}

// LazyValue represents a value that may not already have been extracted.
// Currently, it can represent either an in-place value (stored with the key)
// or a value stored in the value section. However, the interface is general
// enough to support values that are stored in separate files.
//
// LazyValue is used in the InternalIterator interface, such that all
// positioning calls return (*InternalKey, LazyValue). It is also exposed via
// the public Iterator for callers that need to remember a recent but not
// necessarily latest LazyValue, in case they need the actual value in the
// future. An example is a caller that is iterating in reverse and looking for
// the latest MVCC version for a key -- it cannot identify the latest MVCC
// version without stepping to the previous key-value pair e.g.
// storage.pebbleMVCCScanner in CockroachDB.
//
// Performance note: It is important for this struct to not exceed a sizeof 32
// bytes, for optimizing the common case of the in-place value. Prior to
// introducing LazyValue, we were passing around a []byte which is 24 bytes.
// Passing a 40 byte or larger struct causes performance to drop by 75% on
// some benchmarks that do tight iteration loops.
//
// Memory management:
// This is subtle, but important for performance.
//
// A LazyValue returned by an InternalIterator or Iterator is unstable in that
// repositioning the iterator will invalidate the memory inside it. A caller
// wishing to maintain that LazyValue needs to call LazyValue.Clone(). Note
// that this does not fetch the value if it is not in-place. Clone() should
// ideally not be called if LazyValue.Value() has been called, since the
// cloned LazyValue will forget the extracted/fetched value, and calling
// Value() on this clone will cause the value to be extracted again. That is,
// Clone() does not make any promise about the memory stability of the
// underlying value.
//
// A user of an iterator that calls LazyValue.Value() wants as much as
// possible for the returned value []byte to point to iterator owned memory.
//
//  1. [P1] The underlying iterator that owns that memory also needs a promise
//     from that user that at any time there is at most one value []byte slice
//     that the caller is expecting it to maintain. Otherwise, the underlying
//     iterator has to maintain multiple such []byte slices which results in
//     more complicated and inefficient code.
//
//  2. [P2] The underlying iterator, in order to make the promise that it is
//     maintaining the one value []byte slice, also needs a way to know when
//     it is relieved of that promise. One way it is relieved of that promise
//     is by being told that it is being repositioned. Typically, the owner of
//     the value []byte slice is a sstable iterator, and it will know that it
//     is relieved of the promise when it is repositioned. However, consider
//     the case where the caller has used LazyValue.Clone() and repositioned
//     the iterator (which is actually a tree of iterators). In this case the
//     underlying sstable iterator may not even be open. LazyValue.Value()
//     will still work (at a higher cost), but since the sstable iterator is
//     not open, it does not have a mechanism to know when the retrieved value
//     is no longer in use. We refer to this situation as "not satisfying P2".
//     To handle this situation, the LazyValue.Value() method accepts a caller
//     owned buffer, that the callee will use if needed. The callee explicitly
//     tells the caller whether the []byte slice for the value is now owned by
//     the caller. This will be true if the callee attempted to use buf and
//     either successfully used it or allocated a new []byte slice.
//
// To ground the above in reality, we consider three examples of callers of
// LazyValue.Value():
//
//   - Iterator: it calls LazyValue.Value for its own use when merging values.
//     When merging during reverse iteration, it may have cloned the LazyValue.
//     In this case it calls LazyValue.Value() on the cloned value, merges it,
//     and then calls LazyValue.Value() on the current iterator position and
//     merges it. So it is honoring P1.
//
//   - Iterator on behalf of Iterator clients: The Iterator.Value() method
//     needs to call LazyValue.Value(). The client of Iterator is satisfying P1
//     because of the inherent Iterator interface constraint, i.e., it is calling
//     Iterator.Value() on the current Iterator position. It is possible that
//     the Iterator has cloned this LazyValue (for the reverse iteration case),
//     which the client is unaware of, so the underlying sstable iterator may
//     not be able to satisfy P2. This is ok because Iterator will call
//     LazyValue.Value with its (reusable) owned buffer.
//
//   - CockroachDB's pebbleMVCCScanner: This will use LazyValues from Iterator
//     since during reverse iteration in order to find the highest version that
//     satisfies a read it needs to clone the LazyValue, step back the iterator
//     and then decide whether it needs the value from the previously cloned
//     LazyValue. The pebbleMVCCScanner will satisfy P1. The P2 story is
//     similar to the previous case in that it will call LazyValue.Value with
//     its (reusable) owned buffer.
//
// Corollary: callers that directly use InternalIterator can know that they
// have done nothing to interfere with promise P2 can pass in a nil buf and be
// sure that it will not trigger an allocation.
//
// Repeated calling of LazyValue.Value:
// This is ok as long as the caller continues to satisfy P1. The previously
// fetched value will be remembered inside LazyValue to avoid fetching again.
// So if the caller's buffer is used the first time the value was fetched, it
// is still in use.
//
// LazyValue fields are visible outside the package for use in
// InternalIterator implementations and in Iterator, but not meant for direct
// use by users of Pebble.
type LazyValue struct {
	// ValueOrHandle represents a value, or a handle to be passed to ValueFetcher.
	// - Fetcher == nil: ValueOrHandle is a value.
	// - Fetcher != nil: ValueOrHandle is a handle and Fetcher.Attribute is
	//   initialized.
	// The ValueOrHandle exposed by InternalIterator or Iterator may not be stable
	// if the iterator is stepped. To make it stable, make a copy using Clone.
	ValueOrHandle []byte
	// Fetcher provides support for fetching an actually lazy value.
	Fetcher *LazyFetcher
}

// LazyFetcher supports fetching a lazy value.
//
// Fetcher and Attribute are to be initialized at creation time. The fields
// are arranged to reduce the sizeof this struct.
type LazyFetcher struct {
	// Fetcher, given a handle, returns the value.
	Fetcher ValueFetcher
	err     error
	value   []byte
	// Attribute includes the short attribute and value length.
	Attribute   AttributeAndLen
	fetched     bool
	callerOwned bool
}

// ValueFetcher is an interface for fetching a value.
type ValueFetcher interface {
	// Fetch returns the value, given the handle. It is acceptable to call the
	// ValueFetcher.Fetch as long as the DB is open. However, one should assume
	// there is a fast-path when the iterator tree has not moved off the sstable
	// iterator that initially provided this LazyValue. Hence, to utilize this
	// fast-path the caller should try to decide whether it needs the value or
	// not as soon as possible, with minimal possible stepping of the iterator.
	//
	// buf will be used if the fetcher cannot satisfy P2 (see earlier comment).
	// If the fetcher attempted to use buf *and* len(buf) was insufficient, it
	// will allocate a new slice for the value. In either case it will set
	// callerOwned to true.
	Fetch(
		handle []byte, valLen int32, buf []byte) (val []byte, callerOwned bool, err error)
}

// Value returns the underlying value.
func (lv *LazyValue) Value(buf []byte) (val []byte, callerOwned bool, err error) {
	if lv.Fetcher == nil {
		return lv.ValueOrHandle, false, nil
	}
	// Do the rest of the work in a separate method to attempt mid-stack
	// inlining of Value(). Unfortunately, this still does not inline since the
	// cost of 85 exceeds the budget of 80.
	//
	// TODO(sumeer): Packing the return values into a struct{[]byte error bool}
	// causes it to be below the budget. Consider this if we need to recover
	// more performance. I suspect that inlining this only matters in
	// micro-benchmarks, and in actual use cases in CockroachDB it will not
	// matter because there is substantial work done with a fetched value.
	return lv.fetchValue(buf)
}

// INVARIANT: lv.Fetcher != nil
func (lv *LazyValue) fetchValue(buf []byte) (val []byte, callerOwned bool, err error) {
	f := lv.Fetcher
	if !f.fetched {
		f.fetched = true
		f.value, f.callerOwned, f.err = f.Fetcher.Fetch(
			lv.ValueOrHandle, lv.Fetcher.Attribute.ValueLen, buf)
	}
	return f.value, f.callerOwned, f.err
}

// InPlaceValue returns the value under the assumption that it is in-place.
// This is for Pebble-internal code.
func (lv *LazyValue) InPlaceValue() []byte {
	if invariants.Enabled && lv.Fetcher != nil {
		panic("value must be in-place")
	}
	return lv.ValueOrHandle
}

// Len returns the length of the value.
func (lv *LazyValue) Len() int {
	if lv.Fetcher == nil {
		return len(lv.ValueOrHandle)
	}
	return int(lv.Fetcher.Attribute.ValueLen)
}

// TryGetShortAttribute returns the ShortAttribute and a bool indicating
// whether the ShortAttribute was populated.
func (lv *LazyValue) TryGetShortAttribute() (ShortAttribute, bool) {
	if lv.Fetcher == nil {
		return 0, false
	}
	return lv.Fetcher.Attribute.ShortAttribute, true
}

// Clone creates a stable copy of the LazyValue, by appending bytes to buf.
// The fetcher parameter must be non-nil and may be over-written and used
// inside the returned LazyValue -- this is needed to avoid an allocation.
// Most callers have at most K cloned LazyValues, where K is hard-coded, so
// they can have a pool of exactly K LazyFetcher structs they can reuse in
// these calls. The alternative of allocating LazyFetchers from a sync.Pool is
// not viable since we have no code trigger for returning to the pool
// (LazyValues are simply GC'd).
//
// NB: It is highly preferable that LazyValue.Value() has not been called,
// since the Clone will forget any previously extracted value, and a future
// call to Value will cause it to be fetched again. We do this since we don't
// want to reason about whether or not to clone an already extracted value
// inside the Fetcher (we don't). Property P1 applies here too: if lv1.Value()
// has been called, and then lv2 is created as a clone of lv1, then calling
// lv2.Value() can invalidate any backing memory maintained inside the fetcher
// for lv1 (even though these are the same values). We initially prohibited
// calling LazyValue.Clone() if LazyValue.Value() has been called, but there
// is at least one complex caller (pebbleMVCCScanner inside CockroachDB) where
// it is not easy to prove this invariant.
func (lv *LazyValue) Clone(buf []byte, fetcher *LazyFetcher) (LazyValue, []byte) {
	var lvCopy LazyValue
	if lv.Fetcher != nil {
		*fetcher = LazyFetcher{
			Fetcher:   lv.Fetcher.Fetcher,
			Attribute: lv.Fetcher.Attribute,
			// Not copying anything that has been extracted.
		}
		lvCopy.Fetcher = fetcher
	}
	vLen := len(lv.ValueOrHandle)
	if vLen == 0 {
		return lvCopy, buf
	}
	bufLen := len(buf)
	buf = append(buf, lv.ValueOrHandle...)
	lvCopy.ValueOrHandle = buf[bufLen : bufLen+vLen]
	return lvCopy, buf
}

// MakeInPlaceValue constructs an in-place value.
func MakeInPlaceValue(val []byte) LazyValue {
	return LazyValue{ValueOrHandle: val}
}
