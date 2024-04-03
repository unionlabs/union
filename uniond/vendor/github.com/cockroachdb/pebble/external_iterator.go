// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"context"
	"fmt"
	"sort"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/sstable"
)

// ExternalIterOption provide an interface to specify open-time options to
// NewExternalIter.
type ExternalIterOption interface {
	// iterApply is called on the iterator during opening in order to set internal
	// parameters.
	iterApply(*Iterator)
	// readerOptions returns any reader options added by this iter option.
	readerOptions() []sstable.ReaderOption
}

type externalIterReaderOptions struct {
	opts []sstable.ReaderOption
}

func (e *externalIterReaderOptions) iterApply(iterator *Iterator) {
	// Do nothing.
}

func (e *externalIterReaderOptions) readerOptions() []sstable.ReaderOption {
	return e.opts
}

// ExternalIterReaderOptions returns an ExternalIterOption that specifies
// sstable.ReaderOptions to be applied on sstable readers in NewExternalIter.
func ExternalIterReaderOptions(opts ...sstable.ReaderOption) ExternalIterOption {
	return &externalIterReaderOptions{opts: opts}
}

// ExternalIterForwardOnly is an ExternalIterOption that specifies this iterator
// will only be used for forward positioning operations (First, SeekGE, Next).
// This could enable optimizations that take advantage of this invariant.
// Behaviour when a reverse positioning operation is done on an iterator
// opened with this option is unpredictable, though in most cases it should.
type ExternalIterForwardOnly struct{}

func (e ExternalIterForwardOnly) iterApply(iter *Iterator) {
	iter.forwardOnly = true
}

func (e ExternalIterForwardOnly) readerOptions() []sstable.ReaderOption {
	return nil
}

// NewExternalIter takes an input 2d array of sstable files which may overlap
// across subarrays but not within a subarray (at least as far as points are
// concerned; range keys are allowed to overlap arbitrarily even within a
// subarray), and returns an Iterator over the merged contents of the sstables.
// Input sstables may contain point keys, range keys, range deletions, etc. The
// input files slice must be sorted in reverse chronological ordering. A key in a
// file at a lower index subarray will shadow a key with an identical user key
// contained within a file at a higher index subarray. Each subarray must be
// sorted in internal key order, where lower index files contain keys that sort
// left of files with higher indexes.
//
// Input sstables must only contain keys with the zero sequence number.
//
// Iterators constructed through NewExternalIter do not support all iterator
// options, including block-property and table filters. NewExternalIter errors
// if an incompatible option is set.
func NewExternalIter(
	o *Options,
	iterOpts *IterOptions,
	files [][]sstable.ReadableFile,
	extraOpts ...ExternalIterOption,
) (it *Iterator, err error) {
	return NewExternalIterWithContext(context.Background(), o, iterOpts, files, extraOpts...)
}

// NewExternalIterWithContext is like NewExternalIter, and additionally
// accepts a context for tracing.
func NewExternalIterWithContext(
	ctx context.Context,
	o *Options,
	iterOpts *IterOptions,
	files [][]sstable.ReadableFile,
	extraOpts ...ExternalIterOption,
) (it *Iterator, err error) {
	if iterOpts != nil {
		if err := validateExternalIterOpts(iterOpts); err != nil {
			return nil, err
		}
	}

	var readers [][]*sstable.Reader

	// Ensure we close all the opened readers if we error out.
	defer func() {
		if err != nil {
			for i := range readers {
				for j := range readers[i] {
					_ = readers[i][j].Close()
				}
			}
		}
	}()
	seqNumOffset := 0
	var extraReaderOpts []sstable.ReaderOption
	for i := range extraOpts {
		extraReaderOpts = append(extraReaderOpts, extraOpts[i].readerOptions()...)
	}
	for _, levelFiles := range files {
		seqNumOffset += len(levelFiles)
	}
	for _, levelFiles := range files {
		var subReaders []*sstable.Reader
		seqNumOffset -= len(levelFiles)
		subReaders, err = openExternalTables(o, levelFiles, seqNumOffset, o.MakeReaderOptions(), extraReaderOpts...)
		readers = append(readers, subReaders)
	}
	if err != nil {
		return nil, err
	}

	buf := iterAllocPool.Get().(*iterAlloc)
	dbi := &buf.dbi
	*dbi = Iterator{
		ctx:                 ctx,
		alloc:               buf,
		merge:               o.Merger.Merge,
		comparer:            *o.Comparer,
		readState:           nil,
		keyBuf:              buf.keyBuf,
		prefixOrFullSeekKey: buf.prefixOrFullSeekKey,
		boundsBuf:           buf.boundsBuf,
		batch:               nil,
		// Add the readers to the Iterator so that Close closes them, and
		// SetOptions can re-construct iterators from them.
		externalReaders: readers,
		newIters: func(
			ctx context.Context, f *manifest.FileMetadata, opts *IterOptions,
			internalOpts internalIterOpts) (internalIterator, keyspan.FragmentIterator, error) {
			// NB: External iterators are currently constructed without any
			// `levelIters`. newIters should never be called. When we support
			// organizing multiple non-overlapping files into a single level
			// (see TODO below), we'll need to adjust this tableNewIters
			// implementation to open iterators by looking up f in a map
			// of readers indexed by *fileMetadata.
			panic("unreachable")
		},
		seqNum: base.InternalKeySeqNumMax,
	}
	if iterOpts != nil {
		dbi.opts = *iterOpts
		dbi.processBounds(iterOpts.LowerBound, iterOpts.UpperBound)
	}
	for i := range extraOpts {
		extraOpts[i].iterApply(dbi)
	}
	finishInitializingExternal(ctx, dbi)
	return dbi, nil
}

func validateExternalIterOpts(iterOpts *IterOptions) error {
	switch {
	case iterOpts.TableFilter != nil:
		return errors.Errorf("pebble: external iterator: TableFilter unsupported")
	case iterOpts.PointKeyFilters != nil:
		return errors.Errorf("pebble: external iterator: PointKeyFilters unsupported")
	case iterOpts.RangeKeyFilters != nil:
		return errors.Errorf("pebble: external iterator: RangeKeyFilters unsupported")
	case iterOpts.OnlyReadGuaranteedDurable:
		return errors.Errorf("pebble: external iterator: OnlyReadGuaranteedDurable unsupported")
	case iterOpts.UseL6Filters:
		return errors.Errorf("pebble: external iterator: UseL6Filters unsupported")
	}
	return nil
}

func createExternalPointIter(ctx context.Context, it *Iterator) (internalIterator, error) {
	// TODO(jackson): In some instances we could generate fewer levels by using
	// L0Sublevels code to organize nonoverlapping files into the same level.
	// This would allow us to use levelIters and keep a smaller set of data and
	// files in-memory. However, it would also require us to identify the bounds
	// of all the files upfront.

	if !it.opts.pointKeys() {
		return emptyIter, nil
	} else if it.pointIter != nil {
		return it.pointIter, nil
	}
	mlevels := it.alloc.mlevels[:0]

	if len(it.externalReaders) > cap(mlevels) {
		mlevels = make([]mergingIterLevel, 0, len(it.externalReaders))
	}
	for _, readers := range it.externalReaders {
		var combinedIters []internalIterator
		for _, r := range readers {
			var (
				rangeDelIter keyspan.FragmentIterator
				pointIter    internalIterator
				err          error
			)
			// We could set hideObsoletePoints=true, since we are reading at
			// InternalKeySeqNumMax, but we don't bother since these sstables should
			// not have obsolete points (so the performance optimization is
			// unnecessary), and we don't want to bother constructing a
			// BlockPropertiesFilterer that includes obsoleteKeyBlockPropertyFilter.
			pointIter, err = r.NewIterWithBlockPropertyFiltersAndContextEtc(
				ctx, it.opts.LowerBound, it.opts.UpperBound, nil, /* BlockPropertiesFilterer */
				false /* hideObsoletePoints */, false, /* useFilterBlock */
				&it.stats.InternalStats, sstable.TrivialReaderProvider{Reader: r})
			if err != nil {
				return nil, err
			}
			rangeDelIter, err = r.NewRawRangeDelIter()
			if err != nil {
				return nil, err
			}
			if rangeDelIter == nil && pointIter != nil && it.forwardOnly {
				// TODO(bilal): Consider implementing range key pausing in
				// simpleLevelIter so we can reduce mergingIterLevels even more by
				// sending all sstable iterators to combinedIters, not just those
				// corresponding to sstables without range deletes.
				combinedIters = append(combinedIters, pointIter)
				continue
			}
			mlevels = append(mlevels, mergingIterLevel{
				iter:         pointIter,
				rangeDelIter: rangeDelIter,
			})
		}
		if len(combinedIters) == 1 {
			mlevels = append(mlevels, mergingIterLevel{
				iter: combinedIters[0],
			})
		} else if len(combinedIters) > 1 {
			sli := &simpleLevelIter{
				cmp:   it.cmp,
				iters: combinedIters,
			}
			sli.init(it.opts)
			mlevels = append(mlevels, mergingIterLevel{
				iter:         sli,
				rangeDelIter: nil,
			})
		}
	}
	if len(mlevels) == 1 && mlevels[0].rangeDelIter == nil {
		// Set closePointIterOnce to true. This is because we're bypassing the
		// merging iter, which turns Close()s on it idempotent for any child
		// iterators. The outer Iterator could call Close() on a point iter twice,
		// which sstable iterators do not support (as they release themselves to
		// a pool).
		it.closePointIterOnce = true
		return mlevels[0].iter, nil
	}

	it.alloc.merging.init(&it.opts, &it.stats.InternalStats, it.comparer.Compare, it.comparer.Split, mlevels...)
	it.alloc.merging.snapshot = base.InternalKeySeqNumMax
	if len(mlevels) <= cap(it.alloc.levelsPositioned) {
		it.alloc.merging.levelsPositioned = it.alloc.levelsPositioned[:len(mlevels)]
	}
	return &it.alloc.merging, nil
}

func finishInitializingExternal(ctx context.Context, it *Iterator) {
	pointIter, err := createExternalPointIter(ctx, it)
	if err != nil {
		it.pointIter = &errorIter{err: err}
	} else {
		it.pointIter = pointIter
	}
	it.iter = it.pointIter

	if it.opts.rangeKeys() {
		it.rangeKeyMasking.init(it, it.comparer.Compare, it.comparer.Split)
		var rangeKeyIters []keyspan.FragmentIterator
		if it.rangeKey == nil {
			// We could take advantage of the lack of overlaps in range keys within
			// each slice in it.externalReaders, and generate keyspan.LevelIters
			// out of those. However, since range keys are expected to be sparse to
			// begin with, the performance gain might not be significant enough to
			// warrant it.
			//
			// TODO(bilal): Explore adding a simpleRangeKeyLevelIter that does not
			// operate on FileMetadatas (similar to simpleLevelIter), and implements
			// this optimization.
			for _, readers := range it.externalReaders {
				for _, r := range readers {
					if rki, err := r.NewRawRangeKeyIter(); err != nil {
						rangeKeyIters = append(rangeKeyIters, &errorKeyspanIter{err: err})
					} else if rki != nil {
						rangeKeyIters = append(rangeKeyIters, rki)
					}
				}
			}
			if len(rangeKeyIters) > 0 {
				it.rangeKey = iterRangeKeyStateAllocPool.Get().(*iteratorRangeKeyState)
				it.rangeKey.init(it.comparer.Compare, it.comparer.Split, &it.opts)
				it.rangeKey.rangeKeyIter = it.rangeKey.iterConfig.Init(
					&it.comparer,
					base.InternalKeySeqNumMax,
					it.opts.LowerBound, it.opts.UpperBound,
					&it.hasPrefix, &it.prefixOrFullSeekKey,
					false /* internalKeys */, &it.rangeKey.internal,
				)
				for i := range rangeKeyIters {
					it.rangeKey.iterConfig.AddLevel(rangeKeyIters[i])
				}
			}
		}
		if it.rangeKey != nil {
			it.rangeKey.iiter.Init(&it.comparer, it.iter, it.rangeKey.rangeKeyIter,
				keyspan.InterleavingIterOpts{
					Mask:       &it.rangeKeyMasking,
					LowerBound: it.opts.LowerBound,
					UpperBound: it.opts.UpperBound,
				})
			it.iter = &it.rangeKey.iiter
		}
	}
}

func openExternalTables(
	o *Options,
	files []sstable.ReadableFile,
	seqNumOffset int,
	readerOpts sstable.ReaderOptions,
	extraReaderOpts ...sstable.ReaderOption,
) (readers []*sstable.Reader, err error) {
	readers = make([]*sstable.Reader, 0, len(files))
	for i := range files {
		readable, err := sstable.NewSimpleReadable(files[i])
		if err != nil {
			return readers, err
		}
		r, err := sstable.NewReader(readable, readerOpts, extraReaderOpts...)
		if err != nil {
			return readers, err
		}
		// Use the index of the file in files as the sequence number for all of
		// its keys.
		r.Properties.GlobalSeqNum = uint64(len(files) - i + seqNumOffset)
		readers = append(readers, r)
	}
	return readers, err
}

// simpleLevelIter is similar to a levelIter in that it merges the points
// from multiple point iterators that are non-overlapping in the key ranges
// they return. It is only expected to support forward iteration and forward
// regular seeking; reverse iteration and prefix seeking is not supported.
// Intended to be a low-overhead, non-FileMetadata dependent option for
// NewExternalIter. To optimize seeking and forward iteration, it maintains
// two slices of child iterators; one of all iterators, and a subset of it that
// contains just the iterators that contain point keys within the current
// bounds.
//
// Note that this levelIter does not support pausing at file boundaries
// in case of range tombstones in this file that could apply to points outside
// of this file (and outside of this level). This is sufficient for optimizing
// the main use cases of NewExternalIter, however for completeness it would make
// sense to build this pausing functionality in.
type simpleLevelIter struct {
	cmp          Compare
	err          error
	lowerBound   []byte
	iters        []internalIterator
	filtered     []internalIterator
	firstKeys    [][]byte
	firstKeysBuf []byte
	currentIdx   int
}

var _ internalIterator = &simpleLevelIter{}

// init initializes this simpleLevelIter.
func (s *simpleLevelIter) init(opts IterOptions) {
	s.currentIdx = 0
	s.lowerBound = opts.LowerBound
	s.resetFilteredIters()
}

func (s *simpleLevelIter) resetFilteredIters() {
	s.filtered = s.filtered[:0]
	s.firstKeys = s.firstKeys[:0]
	s.firstKeysBuf = s.firstKeysBuf[:0]
	s.err = nil
	for i := range s.iters {
		var iterKey *base.InternalKey
		if s.lowerBound != nil {
			iterKey, _ = s.iters[i].SeekGE(s.lowerBound, base.SeekGEFlagsNone)
		} else {
			iterKey, _ = s.iters[i].First()
		}
		if iterKey != nil {
			s.filtered = append(s.filtered, s.iters[i])
			bufStart := len(s.firstKeysBuf)
			s.firstKeysBuf = append(s.firstKeysBuf, iterKey.UserKey...)
			s.firstKeys = append(s.firstKeys, s.firstKeysBuf[bufStart:bufStart+len(iterKey.UserKey)])
		} else if err := s.iters[i].Error(); err != nil {
			s.err = err
		}
	}
}

func (s *simpleLevelIter) SeekGE(
	key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	if s.err != nil {
		return nil, base.LazyValue{}
	}
	// Find the first file that is entirely >= key. The file before that could
	// contain the key we're looking for.
	n := sort.Search(len(s.firstKeys), func(i int) bool {
		return s.cmp(key, s.firstKeys[i]) <= 0
	})
	if n > 0 {
		s.currentIdx = n - 1
	} else {
		s.currentIdx = n
	}
	if s.currentIdx < len(s.filtered) {
		if iterKey, val := s.filtered[s.currentIdx].SeekGE(key, flags); iterKey != nil {
			return iterKey, val
		}
		if err := s.filtered[s.currentIdx].Error(); err != nil {
			s.err = err
		}
		s.currentIdx++
	}
	return s.skipEmptyFileForward(key, flags)
}

func (s *simpleLevelIter) skipEmptyFileForward(
	seekKey []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	var iterKey *base.InternalKey
	var val base.LazyValue
	for s.currentIdx >= 0 && s.currentIdx < len(s.filtered) && s.err == nil {
		if seekKey != nil {
			iterKey, val = s.filtered[s.currentIdx].SeekGE(seekKey, flags)
		} else if s.lowerBound != nil {
			iterKey, val = s.filtered[s.currentIdx].SeekGE(s.lowerBound, flags)
		} else {
			iterKey, val = s.filtered[s.currentIdx].First()
		}
		if iterKey != nil {
			return iterKey, val
		}
		if err := s.filtered[s.currentIdx].Error(); err != nil {
			s.err = err
		}
		s.currentIdx++
	}
	return nil, base.LazyValue{}
}

func (s *simpleLevelIter) SeekPrefixGE(
	prefix, key []byte, flags base.SeekGEFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

func (s *simpleLevelIter) SeekLT(
	key []byte, flags base.SeekLTFlags,
) (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

func (s *simpleLevelIter) First() (*base.InternalKey, base.LazyValue) {
	if s.err != nil {
		return nil, base.LazyValue{}
	}
	s.currentIdx = 0
	return s.skipEmptyFileForward(nil /* seekKey */, base.SeekGEFlagsNone)
}

func (s *simpleLevelIter) Last() (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

func (s *simpleLevelIter) Next() (*base.InternalKey, base.LazyValue) {
	if s.err != nil {
		return nil, base.LazyValue{}
	}
	if s.currentIdx < 0 || s.currentIdx >= len(s.filtered) {
		return nil, base.LazyValue{}
	}
	if iterKey, val := s.filtered[s.currentIdx].Next(); iterKey != nil {
		return iterKey, val
	}
	s.currentIdx++
	return s.skipEmptyFileForward(nil /* seekKey */, base.SeekGEFlagsNone)
}

func (s *simpleLevelIter) NextPrefix(succKey []byte) (*base.InternalKey, base.LazyValue) {
	if s.err != nil {
		return nil, base.LazyValue{}
	}
	if s.currentIdx < 0 || s.currentIdx >= len(s.filtered) {
		return nil, base.LazyValue{}
	}
	if iterKey, val := s.filtered[s.currentIdx].NextPrefix(succKey); iterKey != nil {
		return iterKey, val
	}
	s.currentIdx++
	return s.skipEmptyFileForward(succKey /* seekKey */, base.SeekGEFlagsNone)
}

func (s *simpleLevelIter) Prev() (*base.InternalKey, base.LazyValue) {
	panic("unimplemented")
}

func (s *simpleLevelIter) Error() error {
	if s.currentIdx >= 0 && s.currentIdx < len(s.filtered) {
		s.err = firstError(s.err, s.filtered[s.currentIdx].Error())
	}
	return s.err
}

func (s *simpleLevelIter) Close() error {
	var err error
	for i := range s.iters {
		err = firstError(err, s.iters[i].Close())
	}
	return err
}

func (s *simpleLevelIter) SetBounds(lower, upper []byte) {
	s.currentIdx = -1
	s.lowerBound = lower
	for i := range s.iters {
		s.iters[i].SetBounds(lower, upper)
	}
	s.resetFilteredIters()
}

func (s *simpleLevelIter) String() string {
	if s.currentIdx < 0 || s.currentIdx >= len(s.filtered) {
		return "simpleLevelIter: current=<nil>"
	}
	return fmt.Sprintf("simpleLevelIter: current=%s", s.filtered[s.currentIdx])
}

var _ internalIterator = &simpleLevelIter{}
