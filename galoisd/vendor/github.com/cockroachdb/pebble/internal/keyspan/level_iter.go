// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"fmt"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/manifest"
)

// LevelIter provides a merged view of spans from sstables in a level.
// It takes advantage of level invariants to only have one sstable span block
// open at one time, opened using the newIter function passed in.
type LevelIter struct {
	cmp base.Compare
	// Denotes the kind of key the level iterator should read. If the key type
	// is KeyTypePoint, the level iterator will read range tombstones (which
	// only affect point keys). If the key type is KeyTypeRange, the level
	// iterator will read range keys. It is invalid to configure an iterator
	// with the KeyTypePointAndRange key type.
	//
	// If key type is KeyTypePoint, no straddle spans are emitted between files,
	// and point key bounds are used to find files instead of range key bounds.
	//
	// TODO(bilal): Straddle spans can safely be produced in rangedel mode once
	// we can guarantee that we will never read sstables in a level that split
	// user keys across them. This might be guaranteed in a future release, but
	// as of CockroachDB 22.2 it is not guaranteed, so to be safe disable it when
	// keyType == KeyTypePoint
	keyType manifest.KeyType
	// The LSM level this LevelIter is initialized for. Used in logging.
	level manifest.Level
	// The below fields are used to fill in gaps between adjacent files' range
	// key spaces. This is an optimization to avoid unnecessarily loading files
	// in cases where range keys are sparse and rare. dir is set by every
	// positioning operation, straddleDir is set to dir whenever a straddling
	// Span is synthesized and the last positioning operation returned a
	// synthesized straddle span.
	//
	// Note that when a straddle span is initialized, iterFile is modified to
	// point to the next file in the straddleDir direction. A change of direction
	// on a straddle key therefore necessitates the value of iterFile to be
	// reverted.
	dir         int
	straddle    Span
	straddleDir int
	// The iter for the current file (iterFile). It is nil under any of the
	// following conditions:
	// - files.Current() == nil
	// - err != nil
	// - straddleDir != 0, in which case iterFile is not nil and points to the
	//   next file (in the straddleDir direction).
	// - some other constraint, like the bounds in opts, caused the file at index to not
	//   be relevant to the iteration.
	iter FragmentIterator
	// iterFile holds the current file.
	// INVARIANT: iterFile = files.Current()
	iterFile *manifest.FileMetadata
	newIter  TableNewSpanIter
	files    manifest.LevelIterator
	err      error

	// The options that were passed in.
	tableOpts SpanIterOptions

	// TODO(bilal): Add InternalIteratorStats.
}

// LevelIter implements the keyspan.FragmentIterator interface.
var _ FragmentIterator = (*LevelIter)(nil)

// NewLevelIter returns a LevelIter.
func NewLevelIter(
	opts SpanIterOptions,
	cmp base.Compare,
	newIter TableNewSpanIter,
	files manifest.LevelIterator,
	level manifest.Level,
	keyType manifest.KeyType,
) *LevelIter {
	l := &LevelIter{}
	l.Init(opts, cmp, newIter, files, level, keyType)
	return l
}

// Init initializes a LevelIter.
func (l *LevelIter) Init(
	opts SpanIterOptions,
	cmp base.Compare,
	newIter TableNewSpanIter,
	files manifest.LevelIterator,
	level manifest.Level,
	keyType manifest.KeyType,
) {
	l.err = nil
	l.level = level
	l.tableOpts = opts
	l.cmp = cmp
	l.iterFile = nil
	l.newIter = newIter
	switch keyType {
	case manifest.KeyTypePoint:
		l.keyType = keyType
		l.files = files.Filter(keyType)
	case manifest.KeyTypeRange:
		l.keyType = keyType
		l.files = files.Filter(keyType)
	default:
		panic(fmt.Sprintf("unsupported key type: %v", keyType))
	}
}

func (l *LevelIter) findFileGE(key []byte) *manifest.FileMetadata {
	// Find the earliest file whose largest key is >= key.
	//
	// If the earliest file has its largest key == key and that largest key is a
	// range deletion sentinel, we know that we manufactured this sentinel to convert
	// the exclusive range deletion end key into an inclusive key (reminder: [start, end)#seqnum
	// is the form of a range deletion sentinel which can contribute a largest key = end#sentinel).
	// In this case we don't return this as the earliest file since there is nothing actually
	// equal to key in it.

	m := l.files.SeekGE(l.cmp, key)
	for m != nil {
		largestKey := m.LargestRangeKey
		if l.keyType == manifest.KeyTypePoint {
			largestKey = m.LargestPointKey
		}
		if !largestKey.IsExclusiveSentinel() || l.cmp(largestKey.UserKey, key) != 0 {
			break
		}
		m = l.files.Next()
	}
	return m
}

func (l *LevelIter) findFileLT(key []byte) *manifest.FileMetadata {
	// Find the last file whose smallest key is < key.
	return l.files.SeekLT(l.cmp, key)
}

type loadFileReturnIndicator int8

const (
	noFileLoaded loadFileReturnIndicator = iota
	fileAlreadyLoaded
	newFileLoaded
)

func (l *LevelIter) loadFile(file *manifest.FileMetadata, dir int) loadFileReturnIndicator {
	indicator := noFileLoaded
	if l.iterFile == file {
		if l.err != nil {
			return noFileLoaded
		}
		if l.iter != nil {
			// We are already at the file, but we would need to check for bounds.
			// Set indicator accordingly.
			indicator = fileAlreadyLoaded
		}
		// We were already at file, but don't have an iterator, probably because the file was
		// beyond the iteration bounds. It may still be, but it is also possible that the bounds
		// have changed. We handle that below.
	}

	// Note that LevelIter.Close() can be called multiple times.
	if indicator != fileAlreadyLoaded {
		if err := l.Close(); err != nil {
			return noFileLoaded
		}
	}

	l.iterFile = file
	if file == nil {
		return noFileLoaded
	}
	if indicator != fileAlreadyLoaded {
		l.iter, l.err = l.newIter(file, l.tableOpts)
		indicator = newFileLoaded
	}
	if l.err != nil {
		return noFileLoaded
	}
	return indicator
}

// SeekGE implements keyspan.FragmentIterator.
func (l *LevelIter) SeekGE(key []byte) *Span {
	l.dir = +1
	l.straddle = Span{}
	l.straddleDir = 0
	l.err = nil // clear cached iteration error

	f := l.findFileGE(key)
	if f != nil && l.keyType == manifest.KeyTypeRange && l.cmp(key, f.SmallestRangeKey.UserKey) < 0 {
		// Peek at the previous file.
		prevFile := l.files.Prev()
		l.files.Next()
		if prevFile != nil {
			// We could unconditionally return an empty span between the seek key and
			// f.SmallestRangeKey, however if this span is to the left of all range
			// keys on this level, it could lead to inconsistent behaviour in relative
			// positioning operations. Consider this example, with a b-c range key:
			//
			// SeekGE(a) -> a-b:{}
			// Next() -> b-c{(#5,RANGEKEYSET,@4,foo)}
			// Prev() -> nil
			//
			// Iterators higher up in the iterator stack rely on this sort of relative
			// positioning consistency.
			//
			// TODO(bilal): Investigate ways to be able to return straddle spans in
			// cases similar to the above, while still retaining correctness.
			// Return a straddling key instead of loading the file.
			l.iterFile = f
			if err := l.Close(); err != nil {
				return l.verify(nil)
			}
			l.straddleDir = +1
			l.straddle = Span{
				Start: prevFile.LargestRangeKey.UserKey,
				End:   f.SmallestRangeKey.UserKey,
				Keys:  nil,
			}
			return l.verify(&l.straddle)
		}
	}
	loadFileIndicator := l.loadFile(f, +1)
	if loadFileIndicator == noFileLoaded {
		return l.verify(nil)
	}
	if span := l.iter.SeekGE(key); span != nil {
		return l.verify(span)
	}
	return l.skipEmptyFileForward()
}

// SeekLT implements keyspan.FragmentIterator.
func (l *LevelIter) SeekLT(key []byte) *Span {
	l.dir = -1
	l.straddle = Span{}
	l.straddleDir = 0
	l.err = nil // clear cached iteration error

	f := l.findFileLT(key)
	if f != nil && l.keyType == manifest.KeyTypeRange && l.cmp(f.LargestRangeKey.UserKey, key) < 0 {
		// Peek at the next file.
		nextFile := l.files.Next()
		l.files.Prev()
		if nextFile != nil {
			// We could unconditionally return an empty span between f.LargestRangeKey
			// and the seek key, however if this span is to the right of all range keys
			// on this level, it could lead to inconsistent behaviour in relative
			// positioning operations. Consider this example, with a b-c range key:
			//
			// SeekLT(d) -> c-d:{}
			// Prev() -> b-c{(#5,RANGEKEYSET,@4,foo)}
			// Next() -> nil
			//
			// Iterators higher up in the iterator stack rely on this sort of relative
			// positioning consistency.
			//
			// TODO(bilal): Investigate ways to be able to return straddle spans in
			// cases similar to the above, while still retaining correctness.
			// Return a straddling key instead of loading the file.
			l.iterFile = f
			if err := l.Close(); err != nil {
				return l.verify(nil)
			}
			l.straddleDir = -1
			l.straddle = Span{
				Start: f.LargestRangeKey.UserKey,
				End:   nextFile.SmallestRangeKey.UserKey,
				Keys:  nil,
			}
			return l.verify(&l.straddle)
		}
	}
	if l.loadFile(f, -1) == noFileLoaded {
		return l.verify(nil)
	}
	if span := l.iter.SeekLT(key); span != nil {
		return l.verify(span)
	}
	return l.skipEmptyFileBackward()
}

// First implements keyspan.FragmentIterator.
func (l *LevelIter) First() *Span {
	l.dir = +1
	l.straddle = Span{}
	l.straddleDir = 0
	l.err = nil // clear cached iteration error

	if l.loadFile(l.files.First(), +1) == noFileLoaded {
		return l.verify(nil)
	}
	if span := l.iter.First(); span != nil {
		return l.verify(span)
	}
	return l.skipEmptyFileForward()
}

// Last implements keyspan.FragmentIterator.
func (l *LevelIter) Last() *Span {
	l.dir = -1
	l.straddle = Span{}
	l.straddleDir = 0
	l.err = nil // clear cached iteration error

	if l.loadFile(l.files.Last(), -1) == noFileLoaded {
		return l.verify(nil)
	}
	if span := l.iter.Last(); span != nil {
		return l.verify(span)
	}
	return l.skipEmptyFileBackward()
}

// Next implements keyspan.FragmentIterator.
func (l *LevelIter) Next() *Span {
	if l.err != nil || (l.iter == nil && l.iterFile == nil && l.dir > 0) {
		return l.verify(nil)
	}
	if l.iter == nil && l.iterFile == nil {
		// l.dir <= 0
		return l.First()
	}
	l.dir = +1

	if l.iter != nil {
		if span := l.iter.Next(); span != nil {
			return l.verify(span)
		}
	}
	return l.skipEmptyFileForward()
}

// Prev implements keyspan.FragmentIterator.
func (l *LevelIter) Prev() *Span {
	if l.err != nil || (l.iter == nil && l.iterFile == nil && l.dir < 0) {
		return l.verify(nil)
	}
	if l.iter == nil && l.iterFile == nil {
		// l.dir >= 0
		return l.Last()
	}
	l.dir = -1

	if l.iter != nil {
		if span := l.iter.Prev(); span != nil {
			return l.verify(span)
		}
	}
	return l.skipEmptyFileBackward()
}

func (l *LevelIter) skipEmptyFileForward() *Span {
	if l.straddleDir == 0 && l.keyType == manifest.KeyTypeRange &&
		l.iterFile != nil && l.iter != nil {
		// We were at a file that had spans. Check if the next file that has
		// spans is not directly adjacent to the current file i.e. there is a
		// gap in the span keyspace between the two files. In that case, synthesize
		// a "straddle span" in l.straddle and return that.
		//
		// Straddle spans are not created in rangedel mode.
		if err := l.Close(); err != nil {
			l.err = err
			return l.verify(nil)
		}
		startKey := l.iterFile.LargestRangeKey.UserKey
		// Resetting l.iterFile without loading the file into l.iter is okay and
		// does not change the logic in loadFile() as long as l.iter is also nil;
		// which it should be due to the Close() call above.
		l.iterFile = l.files.Next()
		if l.iterFile == nil {
			return l.verify(nil)
		}
		endKey := l.iterFile.SmallestRangeKey.UserKey
		if l.cmp(startKey, endKey) < 0 {
			// There is a gap between the two files. Synthesize a straddling span
			// to avoid unnecessarily loading the next file.
			l.straddle = Span{
				Start: startKey,
				End:   endKey,
			}
			l.straddleDir = +1
			return l.verify(&l.straddle)
		}
	} else if l.straddleDir < 0 {
		// We were at a straddle key, but are now changing directions. l.iterFile
		// was already moved backward by skipEmptyFileBackward, so advance it
		// forward.
		l.iterFile = l.files.Next()
	}
	l.straddle = Span{}
	l.straddleDir = 0
	var span *Span
	for span.Empty() {
		fileToLoad := l.iterFile
		if l.keyType == manifest.KeyTypePoint {
			// We haven't iterated to the next file yet if we're in point key
			// (rangedel) mode.
			fileToLoad = l.files.Next()
		}
		if l.loadFile(fileToLoad, +1) == noFileLoaded {
			return l.verify(nil)
		}
		span = l.iter.First()
		// In rangedel mode, we can expect to get empty files that we'd need to
		// skip over, but not in range key mode.
		if l.keyType == manifest.KeyTypeRange {
			break
		}
	}
	return l.verify(span)
}

func (l *LevelIter) skipEmptyFileBackward() *Span {
	// We were at a file that had spans. Check if the previous file that has
	// spans is not directly adjacent to the current file i.e. there is a
	// gap in the span keyspace between the two files. In that case, synthesize
	// a "straddle span" in l.straddle and return that.
	//
	// Straddle spans are not created in rangedel mode.
	if l.straddleDir == 0 && l.keyType == manifest.KeyTypeRange &&
		l.iterFile != nil && l.iter != nil {
		if err := l.Close(); err != nil {
			l.err = err
			return l.verify(nil)
		}
		endKey := l.iterFile.SmallestRangeKey.UserKey
		// Resetting l.iterFile without loading the file into l.iter is okay and
		// does not change the logic in loadFile() as long as l.iter is also nil;
		// which it should be due to the Close() call above.
		l.iterFile = l.files.Prev()
		if l.iterFile == nil {
			return l.verify(nil)
		}
		startKey := l.iterFile.LargestRangeKey.UserKey
		if l.cmp(startKey, endKey) < 0 {
			// There is a gap between the two files. Synthesize a straddling span
			// to avoid unnecessarily loading the next file.
			l.straddle = Span{
				Start: startKey,
				End:   endKey,
			}
			l.straddleDir = -1
			return l.verify(&l.straddle)
		}
	} else if l.straddleDir > 0 {
		// We were at a straddle key, but are now changing directions. l.iterFile
		// was already advanced forward by skipEmptyFileForward, so move it
		// backward.
		l.iterFile = l.files.Prev()
	}
	l.straddle = Span{}
	l.straddleDir = 0
	var span *Span
	for span.Empty() {
		fileToLoad := l.iterFile
		if l.keyType == manifest.KeyTypePoint {
			fileToLoad = l.files.Prev()
		}
		if l.loadFile(fileToLoad, -1) == noFileLoaded {
			return l.verify(nil)
		}
		span = l.iter.Last()
		// In rangedel mode, we can expect to get empty files that we'd need to
		// skip over, but not in range key mode as the filter on the FileMetadata
		// should guarantee we always get a non-empty file.
		if l.keyType == manifest.KeyTypeRange {
			break
		}
	}
	return l.verify(span)
}

// verify is invoked whenever a span is returned from an iterator positioning
// method to a caller. During invariant builds, it asserts invariants to the
// caller.
func (l *LevelIter) verify(s *Span) *Span {
	// NB: Do not add any logic outside the invariants.Enabled conditional to
	// ensure that verify is always compiled away in production builds.
	if invariants.Enabled {
		if f := l.files.Current(); f != l.iterFile {
			panic(fmt.Sprintf("LevelIter.files.Current (%s) and l.iterFile (%s) diverged",
				f, l.iterFile))
		}
	}
	return s
}

// Error implements keyspan.FragmentIterator.
func (l *LevelIter) Error() error {
	if l.err != nil || l.iter == nil {
		return l.err
	}
	return l.iter.Error()
}

// Close implements keyspan.FragmentIterator.
func (l *LevelIter) Close() error {
	if l.iter != nil {
		l.err = l.iter.Close()
		l.iter = nil
	}
	return l.err
}

// String implements keyspan.FragmentIterator.
func (l *LevelIter) String() string {
	if l.iterFile != nil {
		return fmt.Sprintf("%s: fileNum=%s", l.level, l.iterFile.FileNum)
	}
	return fmt.Sprintf("%s: fileNum=<nil>", l.level)
}
