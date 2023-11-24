// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
)

// LevelMetadata contains metadata for all of the files within
// a level of the LSM.
type LevelMetadata struct {
	level     int
	totalSize uint64
	// NumVirtual is the number of virtual sstables in the level.
	NumVirtual uint64
	// VirtualSize is the size of the virtual sstables in the level.
	VirtualSize uint64
	tree        btree
}

// clone makes a copy of the level metadata, implicitly increasing the ref
// count of every file contained within lm.
func (lm *LevelMetadata) clone() LevelMetadata {
	return LevelMetadata{
		level:       lm.level,
		totalSize:   lm.totalSize,
		NumVirtual:  lm.NumVirtual,
		VirtualSize: lm.VirtualSize,
		tree:        lm.tree.Clone(),
	}
}

func (lm *LevelMetadata) release() (obsolete []*FileBacking) {
	return lm.tree.Release()
}

func makeLevelMetadata(cmp Compare, level int, files []*FileMetadata) LevelMetadata {
	bcmp := btreeCmpSeqNum
	if level > 0 {
		bcmp = btreeCmpSmallestKey(cmp)
	}
	var lm LevelMetadata
	lm.level = level
	lm.tree, _ = makeBTree(bcmp, files)
	for _, f := range files {
		lm.totalSize += f.Size
		if f.Virtual {
			lm.NumVirtual++
			lm.VirtualSize += f.Size
		}
	}
	return lm
}

func makeBTree(cmp btreeCmp, files []*FileMetadata) (btree, LevelSlice) {
	var t btree
	t.cmp = cmp
	for _, f := range files {
		t.Insert(f)
	}
	return t, newLevelSlice(t.Iter())
}

func (lm *LevelMetadata) insert(f *FileMetadata) error {
	if err := lm.tree.Insert(f); err != nil {
		return err
	}
	lm.totalSize += f.Size
	if f.Virtual {
		lm.NumVirtual++
		lm.VirtualSize += f.Size
	}
	return nil
}

func (lm *LevelMetadata) remove(f *FileMetadata) bool {
	lm.totalSize -= f.Size
	if f.Virtual {
		lm.NumVirtual--
		lm.VirtualSize -= f.Size
	}
	return lm.tree.Delete(f)
}

// Empty indicates whether there are any files in the level.
func (lm *LevelMetadata) Empty() bool {
	return lm.tree.Count() == 0
}

// Len returns the number of files within the level.
func (lm *LevelMetadata) Len() int {
	return lm.tree.Count()
}

// Size returns the cumulative size of all the files within the level.
func (lm *LevelMetadata) Size() uint64 {
	return lm.totalSize
}

// Iter constructs a LevelIterator over the entire level.
func (lm *LevelMetadata) Iter() LevelIterator {
	return LevelIterator{iter: lm.tree.Iter()}
}

// Slice constructs a slice containing the entire level.
func (lm *LevelMetadata) Slice() LevelSlice {
	return newLevelSlice(lm.tree.Iter())
}

// Find finds the provided file in the level if it exists.
func (lm *LevelMetadata) Find(cmp base.Compare, m *FileMetadata) *LevelFile {
	iter := lm.Iter()
	if lm.level != 0 {
		// If lm holds files for levels >0, we can narrow our search by binary
		// searching by bounds.
		o := overlaps(iter, cmp, m.Smallest.UserKey,
			m.Largest.UserKey, m.Largest.IsExclusiveSentinel())
		iter = o.Iter()
	}
	for f := iter.First(); f != nil; f = iter.Next() {
		if f == m {
			lf := iter.Take()
			return &lf
		}
	}
	return nil
}

// Annotation lazily calculates and returns the annotation defined by
// Annotator. The Annotator is used as the key for pre-calculated
// values, so equal Annotators must be used to avoid duplicate computations
// and cached annotations. Annotation must not be called concurrently, and in
// practice this is achieved by requiring callers to hold DB.mu.
func (lm *LevelMetadata) Annotation(annotator Annotator) interface{} {
	if lm.Empty() {
		return annotator.Zero(nil)
	}
	v, _ := lm.tree.root.Annotation(annotator)
	return v
}

// InvalidateAnnotation clears any cached annotations defined by Annotator. The
// Annotator is used as the key for pre-calculated values, so equal Annotators
// must be used to clear the appropriate cached annotation. InvalidateAnnotation
// must not be called concurrently, and in practice this is achieved by
// requiring callers to hold DB.mu.
func (lm *LevelMetadata) InvalidateAnnotation(annotator Annotator) {
	if lm.Empty() {
		return
	}
	lm.tree.root.InvalidateAnnotation(annotator)
}

// LevelFile holds a file's metadata along with its position
// within a level of the LSM.
type LevelFile struct {
	*FileMetadata
	slice LevelSlice
}

// Slice constructs a LevelSlice containing only this file.
func (lf LevelFile) Slice() LevelSlice {
	return lf.slice
}

// NewLevelSliceSeqSorted constructs a LevelSlice over the provided files,
// sorted by the L0 sequence number sort order.
// TODO(jackson): Can we improve this interface or avoid needing to export
// a slice constructor like this?
func NewLevelSliceSeqSorted(files []*FileMetadata) LevelSlice {
	tr, slice := makeBTree(btreeCmpSeqNum, files)
	tr.Release()
	slice.verifyInvariants()
	return slice
}

// NewLevelSliceKeySorted constructs a LevelSlice over the provided files,
// sorted by the files smallest keys.
// TODO(jackson): Can we improve this interface or avoid needing to export
// a slice constructor like this?
func NewLevelSliceKeySorted(cmp base.Compare, files []*FileMetadata) LevelSlice {
	tr, slice := makeBTree(btreeCmpSmallestKey(cmp), files)
	tr.Release()
	slice.verifyInvariants()
	return slice
}

// NewLevelSliceSpecificOrder constructs a LevelSlice over the provided files,
// ordering the files by their order in the provided slice. It's used in
// tests.
// TODO(jackson): Update tests to avoid requiring this and remove it.
func NewLevelSliceSpecificOrder(files []*FileMetadata) LevelSlice {
	tr, slice := makeBTree(btreeCmpSpecificOrder(files), files)
	tr.Release()
	slice.verifyInvariants()
	return slice
}

// newLevelSlice constructs a new LevelSlice backed by iter.
func newLevelSlice(iter iterator) LevelSlice {
	s := LevelSlice{iter: iter}
	if iter.r != nil {
		s.length = iter.r.subtreeCount
	}
	s.verifyInvariants()
	return s
}

// newBoundedLevelSlice constructs a new LevelSlice backed by iter and bounded
// by the provided start and end bounds. The provided startBound and endBound
// iterators must be iterators over the same B-Tree. Both start and end bounds
// are inclusive.
func newBoundedLevelSlice(iter iterator, startBound, endBound *iterator) LevelSlice {
	s := LevelSlice{
		iter:  iter,
		start: startBound,
		end:   endBound,
	}
	if iter.valid() {
		s.length = endBound.countLeft() - startBound.countLeft()
		// NB: The +1 is a consequence of the end bound being inclusive.
		if endBound.valid() {
			s.length++
		}
		// NB: A slice that's empty due to its bounds may have an endBound
		// positioned before the startBound due to the inclusive bounds.
		// TODO(jackson): Consider refactoring the end boundary to be exclusive;
		// it would simplify some areas (eg, here) and complicate others (eg,
		// Reslice-ing to grow compactions).
		if s.length < 0 {
			s.length = 0
		}
	}
	s.verifyInvariants()
	return s
}

// LevelSlice contains a slice of the files within a level of the LSM.
// A LevelSlice is immutable once created, but may be used to construct a
// mutable LevelIterator over the slice's files.
//
// LevelSlices should be constructed through one of the existing constructors,
// not manually initialized.
type LevelSlice struct {
	iter   iterator
	length int
	// start and end form the inclusive bounds of a slice of files within a
	// level of the LSM. They may be nil if the entire B-Tree backing iter is
	// accessible.
	start *iterator
	end   *iterator
}

func (ls LevelSlice) verifyInvariants() {
	if invariants.Enabled {
		i := ls.Iter()
		var length int
		for f := i.First(); f != nil; f = i.Next() {
			length++
		}
		if ls.length != length {
			panic(fmt.Sprintf("LevelSlice %s has length %d value; actual length is %d", ls, ls.length, length))
		}
	}
}

// Each invokes fn for each element in the slice.
func (ls LevelSlice) Each(fn func(*FileMetadata)) {
	iter := ls.Iter()
	for f := iter.First(); f != nil; f = iter.Next() {
		fn(f)
	}
}

// String implements fmt.Stringer.
func (ls LevelSlice) String() string {
	var buf bytes.Buffer
	fmt.Fprintf(&buf, "%d files: ", ls.length)
	ls.Each(func(f *FileMetadata) {
		if buf.Len() > 0 {
			fmt.Fprintf(&buf, " ")
		}
		fmt.Fprint(&buf, f)
	})
	return buf.String()
}

// Empty indicates whether the slice contains any files.
func (ls *LevelSlice) Empty() bool {
	return emptyWithBounds(ls.iter, ls.start, ls.end)
}

// Iter constructs a LevelIterator that iterates over the slice.
func (ls *LevelSlice) Iter() LevelIterator {
	return LevelIterator{
		start: ls.start,
		end:   ls.end,
		iter:  ls.iter.clone(),
	}
}

// Len returns the number of files in the slice. Its runtime is constant.
func (ls *LevelSlice) Len() int {
	return ls.length
}

// SizeSum sums the size of all files in the slice. Its runtime is linear in
// the length of the slice.
func (ls *LevelSlice) SizeSum() uint64 {
	var sum uint64
	iter := ls.Iter()
	for f := iter.First(); f != nil; f = iter.Next() {
		sum += f.Size
	}
	return sum
}

// NumVirtual returns the number of virtual sstables in the level. Its runtime is
// linear in the length of the slice.
func (ls *LevelSlice) NumVirtual() uint64 {
	var n uint64
	iter := ls.Iter()
	for f := iter.First(); f != nil; f = iter.Next() {
		if f.Virtual {
			n++
		}
	}
	return n
}

// VirtualSizeSum returns the sum of the sizes of the virtual sstables in the
// level.
func (ls *LevelSlice) VirtualSizeSum() uint64 {
	var sum uint64
	iter := ls.Iter()
	for f := iter.First(); f != nil; f = iter.Next() {
		if f.Virtual {
			sum += f.Size
		}
	}
	return sum
}

// Reslice constructs a new slice backed by the same underlying level, with
// new start and end positions. Reslice invokes the provided function, passing
// two LevelIterators: one positioned to i's inclusive start and one
// positioned to i's inclusive end. The resliceFunc may move either iterator
// forward or backwards, including beyond the callee's original bounds to
// capture additional files from the underlying level. Reslice constructs and
// returns a new LevelSlice with the final bounds of the iterators after
// calling resliceFunc.
func (ls LevelSlice) Reslice(resliceFunc func(start, end *LevelIterator)) LevelSlice {
	if ls.iter.r == nil {
		return ls
	}
	var start, end LevelIterator
	if ls.start == nil {
		start.iter = ls.iter.clone()
		start.iter.first()
	} else {
		start.iter = ls.start.clone()
	}
	if ls.end == nil {
		end.iter = ls.iter.clone()
		end.iter.last()
	} else {
		end.iter = ls.end.clone()
	}
	resliceFunc(&start, &end)
	return newBoundedLevelSlice(start.iter.clone(), &start.iter, &end.iter)
}

// KeyType is used to specify the type of keys we're looking for in
// LevelIterator positioning operations. Files not containing any keys of the
// desired type are skipped.
type KeyType int8

const (
	// KeyTypePointAndRange denotes a search among the entire keyspace, including
	// both point keys and range keys. No sstables are skipped.
	KeyTypePointAndRange KeyType = iota
	// KeyTypePoint denotes a search among the point keyspace. SSTables with no
	// point keys will be skipped. Note that the point keyspace includes rangedels.
	KeyTypePoint
	// KeyTypeRange denotes a search among the range keyspace. SSTables with no
	// range keys will be skipped.
	KeyTypeRange
)

type keyTypeAnnotator struct{}

var _ Annotator = keyTypeAnnotator{}

func (k keyTypeAnnotator) Zero(dst interface{}) interface{} {
	var val *KeyType
	if dst != nil {
		val = dst.(*KeyType)
	} else {
		val = new(KeyType)
	}
	*val = KeyTypePoint
	return val
}

func (k keyTypeAnnotator) Accumulate(m *FileMetadata, dst interface{}) (interface{}, bool) {
	v := dst.(*KeyType)
	switch *v {
	case KeyTypePoint:
		if m.HasRangeKeys {
			*v = KeyTypePointAndRange
		}
	case KeyTypePointAndRange:
		// Do nothing.
	default:
		panic("unexpected key type")
	}
	return v, true
}

func (k keyTypeAnnotator) Merge(src interface{}, dst interface{}) interface{} {
	v := dst.(*KeyType)
	srcVal := src.(*KeyType)
	switch *v {
	case KeyTypePoint:
		if *srcVal == KeyTypePointAndRange {
			*v = KeyTypePointAndRange
		}
	case KeyTypePointAndRange:
		// Do nothing.
	default:
		panic("unexpected key type")
	}
	return v
}

// LevelIterator iterates over a set of files' metadata. Its zero value is an
// empty iterator.
type LevelIterator struct {
	iter   iterator
	start  *iterator
	end    *iterator
	filter KeyType
}

func (i LevelIterator) String() string {
	var buf bytes.Buffer
	iter := i.iter.clone()
	iter.first()
	iter.prev()
	if i.iter.pos == -1 {
		fmt.Fprint(&buf, "(<start>)*")
	}
	iter.next()
	for ; iter.valid(); iter.next() {
		if buf.Len() > 0 {
			fmt.Fprint(&buf, "   ")
		}

		if i.start != nil && cmpIter(iter, *i.start) == 0 {
			fmt.Fprintf(&buf, " [ ")
		}
		isCurrentPos := cmpIter(iter, i.iter) == 0
		if isCurrentPos {
			fmt.Fprint(&buf, " ( ")
		}
		fmt.Fprint(&buf, iter.cur().String())
		if isCurrentPos {
			fmt.Fprint(&buf, " )*")
		}
		if i.end != nil && cmpIter(iter, *i.end) == 0 {
			fmt.Fprintf(&buf, " ]")
		}
	}
	if i.iter.n != nil && i.iter.pos >= i.iter.n.count {
		if buf.Len() > 0 {
			fmt.Fprint(&buf, "   ")
		}
		fmt.Fprint(&buf, "(<end>)*")
	}
	return buf.String()
}

// Clone copies the iterator, returning an independent iterator at the same
// position.
func (i *LevelIterator) Clone() LevelIterator {
	if i.iter.r == nil {
		return *i
	}
	// The start and end iterators are not cloned and are treated as
	// immutable.
	return LevelIterator{
		iter:   i.iter.clone(),
		start:  i.start,
		end:    i.end,
		filter: i.filter,
	}
}

// Current returns the item at the current iterator position.
//
// Current is deprecated. Callers should instead use the return value of a
// positioning operation.
func (i *LevelIterator) Current() *FileMetadata {
	if !i.iter.valid() ||
		(i.end != nil && cmpIter(i.iter, *i.end) > 0) ||
		(i.start != nil && cmpIter(i.iter, *i.start) < 0) {
		return nil
	}
	return i.iter.cur()
}

func (i *LevelIterator) empty() bool {
	return emptyWithBounds(i.iter, i.start, i.end)
}

// Filter clones the iterator and sets the desired KeyType as the key to filter
// files on.
func (i *LevelIterator) Filter(keyType KeyType) LevelIterator {
	l := i.Clone()
	l.filter = keyType
	return l
}

func emptyWithBounds(i iterator, start, end *iterator) bool {
	// If i.r is nil, the iterator was constructed from an empty btree.
	// If the end bound is before the start bound, the bounds represent an
	// empty slice of the B-Tree.
	return i.r == nil || (start != nil && end != nil && cmpIter(*end, *start) < 0)
}

// First seeks to the first file in the iterator and returns it.
func (i *LevelIterator) First() *FileMetadata {
	if i.empty() {
		return nil
	}
	if i.start != nil {
		i.iter = i.start.clone()
	} else {
		i.iter.first()
	}
	if !i.iter.valid() {
		return nil
	}
	return i.skipFilteredForward(i.iter.cur())
}

// Last seeks to the last file in the iterator and returns it.
func (i *LevelIterator) Last() *FileMetadata {
	if i.empty() {
		return nil
	}
	if i.end != nil {
		i.iter = i.end.clone()
	} else {
		i.iter.last()
	}
	if !i.iter.valid() {
		return nil
	}
	return i.skipFilteredBackward(i.iter.cur())
}

// Next advances the iterator to the next file and returns it.
func (i *LevelIterator) Next() *FileMetadata {
	if i.iter.r == nil {
		return nil
	}
	if invariants.Enabled && (i.iter.pos >= i.iter.n.count || (i.end != nil && cmpIter(i.iter, *i.end) > 0)) {
		panic("pebble: cannot next forward-exhausted iterator")
	}
	i.iter.next()
	if !i.iter.valid() {
		return nil
	}
	return i.skipFilteredForward(i.iter.cur())
}

// Prev moves the iterator the previous file and returns it.
func (i *LevelIterator) Prev() *FileMetadata {
	if i.iter.r == nil {
		return nil
	}
	if invariants.Enabled && (i.iter.pos < 0 || (i.start != nil && cmpIter(i.iter, *i.start) < 0)) {
		panic("pebble: cannot prev backward-exhausted iterator")
	}
	i.iter.prev()
	if !i.iter.valid() {
		return nil
	}
	return i.skipFilteredBackward(i.iter.cur())
}

// SeekGE seeks to the first file in the iterator's file set with a largest
// user key greater than or equal to the provided user key. The iterator must
// have been constructed from L1+, because it requires the underlying files to
// be sorted by user keys and non-overlapping.
func (i *LevelIterator) SeekGE(cmp Compare, userKey []byte) *FileMetadata {
	// TODO(jackson): Assert that i.iter.cmp == btreeCmpSmallestKey.
	if i.iter.r == nil {
		return nil
	}
	m := i.seek(func(m *FileMetadata) bool {
		return cmp(m.Largest.UserKey, userKey) >= 0
	})
	if i.filter != KeyTypePointAndRange && m != nil {
		b, ok := m.LargestBound(i.filter)
		if !ok {
			m = i.Next()
		} else if c := cmp(b.UserKey, userKey); c < 0 || c == 0 && b.IsExclusiveSentinel() {
			// This file does not contain any keys of the type ≥ lower. It
			// should be filtered, even though it does contain point keys.
			m = i.Next()
		}
	}
	return i.skipFilteredForward(m)
}

// SeekLT seeks to the last file in the iterator's file set with a smallest
// user key less than the provided user key. The iterator must have been
// constructed from L1+, because it requires the underlying files to be sorted
// by user keys and non-overlapping.
func (i *LevelIterator) SeekLT(cmp Compare, userKey []byte) *FileMetadata {
	// TODO(jackson): Assert that i.iter.cmp == btreeCmpSmallestKey.
	if i.iter.r == nil {
		return nil
	}
	i.seek(func(m *FileMetadata) bool {
		return cmp(m.Smallest.UserKey, userKey) >= 0
	})
	m := i.Prev()
	// Although i.Prev() guarantees that the current file contains keys of the
	// relevant type, it doesn't guarantee that the keys of the relevant type
	// are < userKey.
	if i.filter != KeyTypePointAndRange && m != nil {
		b, ok := m.SmallestBound(i.filter)
		if !ok {
			panic("unreachable")
		}
		if c := cmp(b.UserKey, userKey); c >= 0 {
			// This file does not contain any keys of the type ≥ lower. It
			// should be filtered, even though it does contain point keys.
			m = i.Prev()
		}
	}
	return i.skipFilteredBackward(m)
}

// skipFilteredForward takes the file metadata at the iterator's current
// position, and skips forward if the current key-type filter (i.filter)
// excludes the file. It skips until it finds an unfiltered file or exhausts the
// level. If lower is != nil, skipFilteredForward skips any files that do not
// contain keys with the provided key-type ≥ lower.
//
// skipFilteredForward also enforces the upper bound, returning nil if at any
// point the upper bound is exceeded.
func (i *LevelIterator) skipFilteredForward(meta *FileMetadata) *FileMetadata {
	for meta != nil && !meta.ContainsKeyType(i.filter) {
		i.iter.next()
		if !i.iter.valid() {
			meta = nil
		} else {
			meta = i.iter.cur()
		}
	}
	if meta != nil && i.end != nil && cmpIter(i.iter, *i.end) > 0 {
		// Exceeded upper bound.
		meta = nil
	}
	return meta
}

// skipFilteredBackward takes the file metadata at the iterator's current
// position, and skips backward if the current key-type filter (i.filter)
// excludes the file. It skips until it finds an unfiltered file or exhausts the
// level. If upper is != nil, skipFilteredBackward skips any files that do not
// contain keys with the provided key-type < upper.
//
// skipFilteredBackward also enforces the lower bound, returning nil if at any
// point the lower bound is exceeded.
func (i *LevelIterator) skipFilteredBackward(meta *FileMetadata) *FileMetadata {
	for meta != nil && !meta.ContainsKeyType(i.filter) {
		i.iter.prev()
		if !i.iter.valid() {
			meta = nil
		} else {
			meta = i.iter.cur()
		}
	}
	if meta != nil && i.start != nil && cmpIter(i.iter, *i.start) < 0 {
		// Exceeded lower bound.
		meta = nil
	}
	return meta
}

func (i *LevelIterator) seek(fn func(*FileMetadata) bool) *FileMetadata {
	i.iter.seek(fn)

	// i.iter.seek seeked in the unbounded underlying B-Tree. If the iterator
	// has start or end bounds, we may have exceeded them. Reset to the bounds
	// if necessary.
	//
	// NB: The LevelIterator and LevelSlice semantics require that a bounded
	// LevelIterator/LevelSlice containing files x0, x1, ..., xn behave
	// identically to an unbounded LevelIterator/LevelSlice of a B-Tree
	// containing x0, x1, ..., xn. In other words, any files outside the
	// LevelIterator's bounds should not influence the iterator's behavior.
	// When seeking, this means a SeekGE that seeks beyond the end bound,
	// followed by a Prev should return the last element within bounds.
	if i.end != nil && cmpIter(i.iter, *i.end) > 0 {
		i.iter = i.end.clone()
		// Since seek(fn) positioned beyond i.end, we know there is nothing to
		// return within bounds.
		i.iter.next()
		return nil
	} else if i.start != nil && cmpIter(i.iter, *i.start) < 0 {
		i.iter = i.start.clone()
	}
	if !i.iter.valid() {
		return nil
	}
	return i.iter.cur()
}

// Take constructs a LevelFile containing the file at the iterator's current
// position. Take panics if the iterator is not currently positioned over a
// file.
func (i *LevelIterator) Take() LevelFile {
	m := i.Current()
	if m == nil {
		panic("Take called on invalid LevelIterator")
	}
	// LevelSlice's start and end fields are immutable and are positioned to
	// the same position for a LevelFile because they're inclusive, so we can
	// share one iterator stack between the two bounds.
	boundsIter := i.iter.clone()
	s := newBoundedLevelSlice(i.iter.clone(), &boundsIter, &boundsIter)
	return LevelFile{
		FileMetadata: m,
		slice:        s,
	}
}
