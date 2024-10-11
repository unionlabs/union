// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"context"
	"sort"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/sstable"
)

func sstableKeyCompare(userCmp Compare, a, b InternalKey) int {
	c := userCmp(a.UserKey, b.UserKey)
	if c != 0 {
		return c
	}
	if a.IsExclusiveSentinel() {
		if !b.IsExclusiveSentinel() {
			return -1
		}
	} else if b.IsExclusiveSentinel() {
		return +1
	}
	return 0
}

// KeyRange encodes a key range in user key space. A KeyRange's Start is
// inclusive while its End is exclusive.
type KeyRange struct {
	Start, End []byte
}

// Valid returns true if the KeyRange is defined.
func (k *KeyRange) Valid() bool {
	return k.Start != nil && k.End != nil
}

// Contains returns whether the specified key exists in the KeyRange.
func (k *KeyRange) Contains(cmp base.Compare, key InternalKey) bool {
	v := cmp(key.UserKey, k.End)
	return (v < 0 || (v == 0 && key.IsExclusiveSentinel())) && cmp(k.Start, key.UserKey) <= 0
}

// OverlapsInternalKeyRange checks if the specified internal key range has an
// overlap with the KeyRange. Note that we aren't checking for full containment
// of smallest-largest within k, rather just that there's some intersection
// between the two ranges.
func (k *KeyRange) OverlapsInternalKeyRange(cmp base.Compare, smallest, largest InternalKey) bool {
	v := cmp(k.Start, largest.UserKey)
	return v <= 0 && !(largest.IsExclusiveSentinel() && v == 0) &&
		cmp(k.End, smallest.UserKey) > 0
}

// Overlaps checks if the specified file has an overlap with the KeyRange.
// Note that we aren't checking for full containment of m within k, rather just
// that there's some intersection between m and k's bounds.
func (k *KeyRange) Overlaps(cmp base.Compare, m *fileMetadata) bool {
	return k.OverlapsInternalKeyRange(cmp, m.Smallest, m.Largest)
}

// OverlapsKeyRange checks if this span overlaps with the provided KeyRange.
// Note that we aren't checking for full containment of either span in the other,
// just that there's a key x that is in both key ranges.
func (k *KeyRange) OverlapsKeyRange(cmp Compare, span KeyRange) bool {
	return cmp(k.Start, span.End) < 0 && cmp(k.End, span.Start) > 0
}

func ingestValidateKey(opts *Options, key *InternalKey) error {
	if key.Kind() == InternalKeyKindInvalid {
		return base.CorruptionErrorf("pebble: external sstable has corrupted key: %s",
			key.Pretty(opts.Comparer.FormatKey))
	}
	if key.SeqNum() != 0 {
		return base.CorruptionErrorf("pebble: external sstable has non-zero seqnum: %s",
			key.Pretty(opts.Comparer.FormatKey))
	}
	return nil
}

// ingestSynthesizeShared constructs a fileMetadata for one shared sstable owned
// or shared by another node.
func ingestSynthesizeShared(
	opts *Options, sm SharedSSTMeta, fileNum base.DiskFileNum,
) (*fileMetadata, error) {
	if sm.Size == 0 {
		// Disallow 0 file sizes
		return nil, errors.New("pebble: cannot ingest shared file with size 0")
	}
	// Don't load table stats. Doing a round trip to shared storage, one SST
	// at a time is not worth it as it slows down ingestion.
	meta := &fileMetadata{
		FileNum:      fileNum.FileNum(),
		CreationTime: time.Now().Unix(),
		Virtual:      true,
		Size:         sm.Size,
	}
	meta.InitProviderBacking(fileNum)
	// Set the underlying FileBacking's size to the same size as the virtualized
	// view of the sstable. This ensures that we don't over-prioritize this
	// sstable for compaction just yet, as we do not have a clear sense of what
	// parts of this sstable are referenced by other nodes.
	meta.FileBacking.Size = sm.Size
	if sm.LargestRangeKey.Valid() && sm.LargestRangeKey.UserKey != nil {
		// Initialize meta.{HasRangeKeys,Smallest,Largest}, etc.
		//
		// NB: We create new internal keys and pass them into ExternalRangeKeyBounds
		// so that we can sub a zero sequence number into the bounds. We can set
		// the sequence number to anything here; it'll be reset in ingestUpdateSeqNum
		// anyway. However we do need to use the same sequence number across all
		// bound keys at this step so that we end up with bounds that are consistent
		// across point/range keys.
		smallestRangeKey := base.MakeInternalKey(sm.SmallestRangeKey.UserKey, 0, sm.SmallestRangeKey.Kind())
		largestRangeKey := base.MakeExclusiveSentinelKey(sm.LargestRangeKey.Kind(), sm.LargestRangeKey.UserKey)
		meta.ExtendRangeKeyBounds(opts.Comparer.Compare, smallestRangeKey, largestRangeKey)
	}
	if sm.LargestPointKey.Valid() && sm.LargestPointKey.UserKey != nil {
		// Initialize meta.{HasPointKeys,Smallest,Largest}, etc.
		//
		// See point above in the ExtendRangeKeyBounds call on why we use a zero
		// sequence number here.
		smallestPointKey := base.MakeInternalKey(sm.SmallestPointKey.UserKey, 0, sm.SmallestPointKey.Kind())
		largestPointKey := base.MakeInternalKey(sm.LargestPointKey.UserKey, 0, sm.LargestPointKey.Kind())
		if sm.LargestPointKey.IsExclusiveSentinel() {
			largestPointKey = base.MakeRangeDeleteSentinelKey(sm.LargestPointKey.UserKey)
		}
		meta.ExtendPointKeyBounds(opts.Comparer.Compare, smallestPointKey, largestPointKey)
	}
	if err := meta.Validate(opts.Comparer.Compare, opts.Comparer.FormatKey); err != nil {
		return nil, err
	}
	return meta, nil
}

// ingestLoad1External loads the fileMetadata for one external sstable.
// Sequence number and target level calculation happens during prepare/apply.
func ingestLoad1External(
	opts *Options,
	e ExternalFile,
	fileNum base.DiskFileNum,
	objprovider objstorage.Provider,
	jobID int,
) (*fileMetadata, error) {
	if e.Size == 0 {
		// Disallow 0 file sizes
		return nil, errors.New("pebble: cannot ingest external file with size 0")
	}
	if !e.HasRangeKey && !e.HasPointKey {
		return nil, errors.New("pebble: cannot ingest external file with no point or range keys")
	}
	// Don't load table stats. Doing a round trip to shared storage, one SST
	// at a time is not worth it as it slows down ingestion.
	meta := &fileMetadata{}
	meta.FileNum = fileNum.FileNum()
	meta.CreationTime = time.Now().Unix()
	meta.Virtual = true
	meta.Size = e.Size
	meta.InitProviderBacking(fileNum)

	// Try to resolve a reference to the external file.
	backing, err := objprovider.CreateExternalObjectBacking(e.Locator, e.ObjName)
	if err != nil {
		return nil, err
	}
	metas, err := objprovider.AttachRemoteObjects([]objstorage.RemoteObjectToAttach{{
		FileNum:  fileNum,
		FileType: fileTypeTable,
		Backing:  backing,
	}})
	if err != nil {
		return nil, err
	}
	if opts.EventListener.TableCreated != nil {
		opts.EventListener.TableCreated(TableCreateInfo{
			JobID:   jobID,
			Reason:  "ingesting",
			Path:    objprovider.Path(metas[0]),
			FileNum: fileNum.FileNum(),
		})
	}
	// In the name of keeping this ingestion as fast as possible, we avoid
	// *all* existence checks and synthesize a file metadata with smallest/largest
	// keys that overlap whatever the passed-in span was.
	smallestCopy := make([]byte, len(e.SmallestUserKey))
	copy(smallestCopy, e.SmallestUserKey)
	largestCopy := make([]byte, len(e.LargestUserKey))
	copy(largestCopy, e.LargestUserKey)
	if e.HasPointKey {
		meta.ExtendPointKeyBounds(opts.Comparer.Compare, base.MakeInternalKey(smallestCopy, 0, InternalKeyKindMax),
			base.MakeRangeDeleteSentinelKey(largestCopy))
	}
	if e.HasRangeKey {
		meta.ExtendRangeKeyBounds(opts.Comparer.Compare, base.MakeInternalKey(smallestCopy, 0, InternalKeyKindRangeKeySet),
			base.MakeExclusiveSentinelKey(InternalKeyKindRangeKeyDelete, largestCopy))
	}

	// Set the underlying FileBacking's size to the same size as the virtualized
	// view of the sstable. This ensures that we don't over-prioritize this
	// sstable for compaction just yet, as we do not have a clear sense of
	// what parts of this sstable are referenced by other nodes.
	meta.FileBacking.Size = e.Size

	if err := meta.Validate(opts.Comparer.Compare, opts.Comparer.FormatKey); err != nil {
		return nil, err
	}
	return meta, nil
}

// ingestLoad1 creates the FileMetadata for one file. This file will be owned
// by this store.
func ingestLoad1(
	opts *Options,
	fmv FormatMajorVersion,
	readable objstorage.Readable,
	cacheID uint64,
	fileNum base.DiskFileNum,
) (*fileMetadata, error) {
	cacheOpts := private.SSTableCacheOpts(cacheID, fileNum).(sstable.ReaderOption)
	r, err := sstable.NewReader(readable, opts.MakeReaderOptions(), cacheOpts)
	if err != nil {
		return nil, err
	}
	defer r.Close()

	// Avoid ingesting tables with format versions this DB doesn't support.
	tf, err := r.TableFormat()
	if err != nil {
		return nil, err
	}
	if tf < fmv.MinTableFormat() || tf > fmv.MaxTableFormat() {
		return nil, errors.Newf(
			"pebble: table format %s is not within range supported at DB format major version %d, (%s,%s)",
			tf, fmv, fmv.MinTableFormat(), fmv.MaxTableFormat(),
		)
	}

	meta := &fileMetadata{}
	meta.FileNum = fileNum.FileNum()
	meta.Size = uint64(readable.Size())
	meta.CreationTime = time.Now().Unix()
	meta.InitPhysicalBacking()

	// Avoid loading into the table cache for collecting stats if we
	// don't need to. If there are no range deletions, we have all the
	// information to compute the stats here.
	//
	// This is helpful in tests for avoiding awkwardness around deletion of
	// ingested files from MemFS. MemFS implements the Windows semantics of
	// disallowing removal of an open file. Under MemFS, if we don't populate
	// meta.Stats here, the file will be loaded into the table cache for
	// calculating stats before we can remove the original link.
	maybeSetStatsFromProperties(meta.PhysicalMeta(), &r.Properties)

	{
		iter, err := r.NewIter(nil /* lower */, nil /* upper */)
		if err != nil {
			return nil, err
		}
		defer iter.Close()
		var smallest InternalKey
		if key, _ := iter.First(); key != nil {
			if err := ingestValidateKey(opts, key); err != nil {
				return nil, err
			}
			smallest = (*key).Clone()
		}
		if err := iter.Error(); err != nil {
			return nil, err
		}
		if key, _ := iter.Last(); key != nil {
			if err := ingestValidateKey(opts, key); err != nil {
				return nil, err
			}
			meta.ExtendPointKeyBounds(opts.Comparer.Compare, smallest, key.Clone())
		}
		if err := iter.Error(); err != nil {
			return nil, err
		}
	}

	iter, err := r.NewRawRangeDelIter()
	if err != nil {
		return nil, err
	}
	if iter != nil {
		defer iter.Close()
		var smallest InternalKey
		if s := iter.First(); s != nil {
			key := s.SmallestKey()
			if err := ingestValidateKey(opts, &key); err != nil {
				return nil, err
			}
			smallest = key.Clone()
		}
		if err := iter.Error(); err != nil {
			return nil, err
		}
		if s := iter.Last(); s != nil {
			k := s.SmallestKey()
			if err := ingestValidateKey(opts, &k); err != nil {
				return nil, err
			}
			largest := s.LargestKey().Clone()
			meta.ExtendPointKeyBounds(opts.Comparer.Compare, smallest, largest)
		}
	}

	// Update the range-key bounds for the table.
	{
		iter, err := r.NewRawRangeKeyIter()
		if err != nil {
			return nil, err
		}
		if iter != nil {
			defer iter.Close()
			var smallest InternalKey
			if s := iter.First(); s != nil {
				key := s.SmallestKey()
				if err := ingestValidateKey(opts, &key); err != nil {
					return nil, err
				}
				smallest = key.Clone()
			}
			if err := iter.Error(); err != nil {
				return nil, err
			}
			if s := iter.Last(); s != nil {
				k := s.SmallestKey()
				if err := ingestValidateKey(opts, &k); err != nil {
					return nil, err
				}
				// As range keys are fragmented, the end key of the last range key in
				// the table provides the upper bound for the table.
				largest := s.LargestKey().Clone()
				meta.ExtendRangeKeyBounds(opts.Comparer.Compare, smallest, largest)
			}
			if err := iter.Error(); err != nil {
				return nil, err
			}
		}
	}

	if !meta.HasPointKeys && !meta.HasRangeKeys {
		return nil, nil
	}

	// Sanity check that the various bounds on the file were set consistently.
	if err := meta.Validate(opts.Comparer.Compare, opts.Comparer.FormatKey); err != nil {
		return nil, err
	}

	return meta, nil
}

type ingestLoadResult struct {
	localMeta, sharedMeta []*fileMetadata
	externalMeta          []*fileMetadata
	localPaths            []string
	sharedLevels          []uint8
	fileCount             int
}

func ingestLoad(
	opts *Options,
	fmv FormatMajorVersion,
	paths []string,
	shared []SharedSSTMeta,
	external []ExternalFile,
	cacheID uint64,
	pending []base.DiskFileNum,
	objProvider objstorage.Provider,
	jobID int,
) (ingestLoadResult, error) {
	meta := make([]*fileMetadata, 0, len(paths))
	newPaths := make([]string, 0, len(paths))
	for i := range paths {
		f, err := opts.FS.Open(paths[i])
		if err != nil {
			return ingestLoadResult{}, err
		}

		readable, err := sstable.NewSimpleReadable(f)
		if err != nil {
			return ingestLoadResult{}, err
		}
		m, err := ingestLoad1(opts, fmv, readable, cacheID, pending[i])
		if err != nil {
			return ingestLoadResult{}, err
		}
		if m != nil {
			meta = append(meta, m)
			newPaths = append(newPaths, paths[i])
		}
	}
	if len(shared) == 0 && len(external) == 0 {
		return ingestLoadResult{localMeta: meta, localPaths: newPaths, fileCount: len(meta)}, nil
	}

	// Sort the shared files according to level.
	sort.Sort(sharedByLevel(shared))

	sharedMeta := make([]*fileMetadata, 0, len(shared))
	levels := make([]uint8, 0, len(shared))
	for i := range shared {
		m, err := ingestSynthesizeShared(opts, shared[i], pending[len(paths)+i])
		if err != nil {
			return ingestLoadResult{}, err
		}
		if shared[i].Level < sharedLevelsStart {
			return ingestLoadResult{}, errors.New("cannot ingest shared file in level below sharedLevelsStart")
		}
		sharedMeta = append(sharedMeta, m)
		levels = append(levels, shared[i].Level)
	}
	externalMeta := make([]*fileMetadata, 0, len(external))
	for i := range external {
		m, err := ingestLoad1External(opts, external[i], pending[len(paths)+len(shared)+i], objProvider, jobID)
		if err != nil {
			return ingestLoadResult{}, err
		}
		externalMeta = append(externalMeta, m)
	}
	result := ingestLoadResult{
		localMeta:    meta,
		sharedMeta:   sharedMeta,
		externalMeta: externalMeta,
		localPaths:   newPaths,
		sharedLevels: levels,
		fileCount:    len(meta) + len(sharedMeta) + len(externalMeta),
	}
	return result, nil
}

// Struct for sorting metadatas by smallest user keys, while ensuring the
// matching path also gets swapped to the same index. For use in
// ingestSortAndVerify.
type metaAndPaths struct {
	meta  []*fileMetadata
	paths []string
	cmp   Compare
}

func (m metaAndPaths) Len() int {
	return len(m.meta)
}

func (m metaAndPaths) Less(i, j int) bool {
	return m.cmp(m.meta[i].Smallest.UserKey, m.meta[j].Smallest.UserKey) < 0
}

func (m metaAndPaths) Swap(i, j int) {
	m.meta[i], m.meta[j] = m.meta[j], m.meta[i]
	if m.paths != nil {
		m.paths[i], m.paths[j] = m.paths[j], m.paths[i]
	}
}

func ingestSortAndVerify(cmp Compare, lr ingestLoadResult, exciseSpan KeyRange) error {
	// Verify that all the shared files (i.e. files in sharedMeta)
	// fit within the exciseSpan.
	for i := range lr.sharedMeta {
		f := lr.sharedMeta[i]
		if !exciseSpan.Contains(cmp, f.Smallest) || !exciseSpan.Contains(cmp, f.Largest) {
			return errors.AssertionFailedf("pebble: shared file outside of excise span, span [%s-%s), file = %s", exciseSpan.Start, exciseSpan.End, f.String())
		}
	}
	if len(lr.externalMeta) > 0 {
		if len(lr.localMeta) > 0 || len(lr.sharedMeta) > 0 {
			// Currently we only support external ingests on their own. If external
			// files are present alongside local/shared files, return an error.
			return errors.AssertionFailedf("pebble: external files cannot be ingested atomically alongside other types of files")
		}
		sort.Sort(&metaAndPaths{
			meta: lr.externalMeta,
			cmp:  cmp,
		})
		for i := 1; i < len(lr.externalMeta); i++ {
			if sstableKeyCompare(cmp, lr.externalMeta[i-1].Largest, lr.externalMeta[i].Smallest) >= 0 {
				return errors.AssertionFailedf("pebble: external sstables have overlapping ranges")
			}
		}
		return nil
	}
	if len(lr.localMeta) <= 1 || len(lr.localPaths) <= 1 {
		return nil
	}

	sort.Sort(&metaAndPaths{
		meta:  lr.localMeta,
		paths: lr.localPaths,
		cmp:   cmp,
	})

	for i := 1; i < len(lr.localPaths); i++ {
		if sstableKeyCompare(cmp, lr.localMeta[i-1].Largest, lr.localMeta[i].Smallest) >= 0 {
			return errors.AssertionFailedf("pebble: local ingestion sstables have overlapping ranges")
		}
	}
	if len(lr.sharedMeta) == 0 {
		return nil
	}
	filesInLevel := make([]*fileMetadata, 0, len(lr.sharedMeta))
	for l := sharedLevelsStart; l < numLevels; l++ {
		filesInLevel = filesInLevel[:0]
		for i := range lr.sharedMeta {
			if lr.sharedLevels[i] == uint8(l) {
				filesInLevel = append(filesInLevel, lr.sharedMeta[i])
			}
		}
		sort.Slice(filesInLevel, func(i, j int) bool {
			return cmp(filesInLevel[i].Smallest.UserKey, filesInLevel[j].Smallest.UserKey) < 0
		})
		for i := 1; i < len(filesInLevel); i++ {
			if sstableKeyCompare(cmp, filesInLevel[i-1].Largest, filesInLevel[i].Smallest) >= 0 {
				return errors.AssertionFailedf("pebble: external shared sstables have overlapping ranges")
			}
		}
	}
	return nil
}

func ingestCleanup(objProvider objstorage.Provider, meta []*fileMetadata) error {
	var firstErr error
	for i := range meta {
		if err := objProvider.Remove(fileTypeTable, meta[i].FileBacking.DiskFileNum); err != nil {
			firstErr = firstError(firstErr, err)
		}
	}
	return firstErr
}

// ingestLink creates new objects which are backed by either hardlinks to or
// copies of the ingested files. It also attaches shared objects to the provider.
func ingestLink(
	jobID int,
	opts *Options,
	objProvider objstorage.Provider,
	lr ingestLoadResult,
	shared []SharedSSTMeta,
) error {
	for i := range lr.localPaths {
		objMeta, err := objProvider.LinkOrCopyFromLocal(
			context.TODO(), opts.FS, lr.localPaths[i], fileTypeTable, lr.localMeta[i].FileBacking.DiskFileNum,
			objstorage.CreateOptions{PreferSharedStorage: true},
		)
		if err != nil {
			if err2 := ingestCleanup(objProvider, lr.localMeta[:i]); err2 != nil {
				opts.Logger.Infof("ingest cleanup failed: %v", err2)
			}
			return err
		}
		if opts.EventListener.TableCreated != nil {
			opts.EventListener.TableCreated(TableCreateInfo{
				JobID:   jobID,
				Reason:  "ingesting",
				Path:    objProvider.Path(objMeta),
				FileNum: lr.localMeta[i].FileNum,
			})
		}
	}
	sharedObjs := make([]objstorage.RemoteObjectToAttach, 0, len(shared))
	for i := range shared {
		backing, err := shared[i].Backing.Get()
		if err != nil {
			return err
		}
		sharedObjs = append(sharedObjs, objstorage.RemoteObjectToAttach{
			FileNum:  lr.sharedMeta[i].FileBacking.DiskFileNum,
			FileType: fileTypeTable,
			Backing:  backing,
		})
	}
	sharedObjMetas, err := objProvider.AttachRemoteObjects(sharedObjs)
	if err != nil {
		return err
	}
	for i := range sharedObjMetas {
		// One corner case around file sizes we need to be mindful of, is that
		// if one of the shareObjs was initially created by us (and has boomeranged
		// back from another node), we'll need to update the FileBacking's size
		// to be the true underlying size. Otherwise, we could hit errors when we
		// open the db again after a crash/restart (see checkConsistency in open.go),
		// plus it more accurately allows us to prioritize compactions of files
		// that were originally created by us.
		if sharedObjMetas[i].IsShared() && !objProvider.IsSharedForeign(sharedObjMetas[i]) {
			size, err := objProvider.Size(sharedObjMetas[i])
			if err != nil {
				return err
			}
			lr.sharedMeta[i].FileBacking.Size = uint64(size)
		}
		if opts.EventListener.TableCreated != nil {
			opts.EventListener.TableCreated(TableCreateInfo{
				JobID:   jobID,
				Reason:  "ingesting",
				Path:    objProvider.Path(sharedObjMetas[i]),
				FileNum: lr.sharedMeta[i].FileNum,
			})
		}
	}
	// We do not need to do anything about lr.externalMetas. Those were already
	// linked in ingestLoad.

	return nil
}

func ingestMemtableOverlaps(cmp Compare, mem flushable, keyRanges []internalKeyRange) bool {
	iter := mem.newIter(nil)
	rangeDelIter := mem.newRangeDelIter(nil)
	rkeyIter := mem.newRangeKeyIter(nil)

	closeIters := func() error {
		err := iter.Close()
		if rangeDelIter != nil {
			err = firstError(err, rangeDelIter.Close())
		}
		if rkeyIter != nil {
			err = firstError(err, rkeyIter.Close())
		}
		return err
	}

	for _, kr := range keyRanges {
		if overlapWithIterator(iter, &rangeDelIter, rkeyIter, kr, cmp) {
			closeIters()
			return true
		}
	}

	// Assume overlap if any iterator errored out.
	return closeIters() != nil
}

func ingestUpdateSeqNum(
	cmp Compare, format base.FormatKey, seqNum uint64, loadResult ingestLoadResult,
) error {
	setSeqFn := func(k base.InternalKey) base.InternalKey {
		return base.MakeInternalKey(k.UserKey, seqNum, k.Kind())
	}
	updateMetadata := func(m *fileMetadata) error {
		// NB: we set the fields directly here, rather than via their Extend*
		// methods, as we are updating sequence numbers.
		if m.HasPointKeys {
			m.SmallestPointKey = setSeqFn(m.SmallestPointKey)
		}
		if m.HasRangeKeys {
			m.SmallestRangeKey = setSeqFn(m.SmallestRangeKey)
		}
		m.Smallest = setSeqFn(m.Smallest)
		// Only update the seqnum for the largest key if that key is not an
		// "exclusive sentinel" (i.e. a range deletion sentinel or a range key
		// boundary), as doing so effectively drops the exclusive sentinel (by
		// lowering the seqnum from the max value), and extends the bounds of the
		// table.
		// NB: as the largest range key is always an exclusive sentinel, it is never
		// updated.
		if m.HasPointKeys && !m.LargestPointKey.IsExclusiveSentinel() {
			m.LargestPointKey = setSeqFn(m.LargestPointKey)
		}
		if !m.Largest.IsExclusiveSentinel() {
			m.Largest = setSeqFn(m.Largest)
		}
		// Setting smallestSeqNum == largestSeqNum triggers the setting of
		// Properties.GlobalSeqNum when an sstable is loaded.
		m.SmallestSeqNum = seqNum
		m.LargestSeqNum = seqNum
		// Ensure the new bounds are consistent.
		if err := m.Validate(cmp, format); err != nil {
			return err
		}
		seqNum++
		return nil
	}

	// Shared sstables are required to be sorted by level ascending. We then
	// iterate the shared sstables in reverse, assigning the lower sequence
	// numbers to the shared sstables that will be ingested into the lower
	// (larger numbered) levels first. This ensures sequence number shadowing is
	// correct.
	for i := len(loadResult.sharedMeta) - 1; i >= 0; i-- {
		if i-1 >= 0 && loadResult.sharedLevels[i-1] > loadResult.sharedLevels[i] {
			panic(errors.AssertionFailedf("shared files %s, %s out of order", loadResult.sharedMeta[i-1], loadResult.sharedMeta[i]))
		}
		if err := updateMetadata(loadResult.sharedMeta[i]); err != nil {
			return err
		}
	}
	for i := range loadResult.localMeta {
		if err := updateMetadata(loadResult.localMeta[i]); err != nil {
			return err
		}
	}
	for i := range loadResult.externalMeta {
		if err := updateMetadata(loadResult.externalMeta[i]); err != nil {
			return err
		}
	}
	return nil
}

// Denotes an internal key range. Smallest and largest are both inclusive.
type internalKeyRange struct {
	smallest, largest InternalKey
}

func overlapWithIterator(
	iter internalIterator,
	rangeDelIter *keyspan.FragmentIterator,
	rkeyIter keyspan.FragmentIterator,
	keyRange internalKeyRange,
	cmp Compare,
) bool {
	// Check overlap with point operations.
	//
	// When using levelIter, it seeks to the SST whose boundaries
	// contain keyRange.smallest.UserKey(S).
	// It then tries to find a point in that SST that is >= S.
	// If there's no such point it means the SST ends in a tombstone in which case
	// levelIter.SeekGE generates a boundary range del sentinel.
	// The comparison of this boundary with keyRange.largest(L) below
	// is subtle but maintains correctness.
	// 1) boundary < L,
	//    since boundary is also > S (initial seek),
	//    whatever the boundary's start key may be, we're always overlapping.
	// 2) boundary > L,
	//    overlap with boundary cannot be determined since we don't know boundary's start key.
	//    We require checking for overlap with rangeDelIter.
	// 3) boundary == L and L is not sentinel,
	//    means boundary < L and hence is similar to 1).
	// 4) boundary == L and L is sentinel,
	//    we'll always overlap since for any values of i,j ranges [i, k) and [j, k) always overlap.
	key, _ := iter.SeekGE(keyRange.smallest.UserKey, base.SeekGEFlagsNone)
	if key != nil {
		c := sstableKeyCompare(cmp, *key, keyRange.largest)
		if c <= 0 {
			return true
		}
	}
	// Assume overlap if iterator errored.
	if err := iter.Error(); err != nil {
		return true
	}

	computeOverlapWithSpans := func(rIter keyspan.FragmentIterator) bool {
		// NB: The spans surfaced by the fragment iterator are non-overlapping.
		span := rIter.SeekLT(keyRange.smallest.UserKey)
		if span == nil {
			span = rIter.Next()
		}
		for ; span != nil; span = rIter.Next() {
			if span.Empty() {
				continue
			}
			key := span.SmallestKey()
			c := sstableKeyCompare(cmp, key, keyRange.largest)
			if c > 0 {
				// The start of the span is after the largest key in the
				// ingested table.
				return false
			}
			if cmp(span.End, keyRange.smallest.UserKey) > 0 {
				// The end of the span is greater than the smallest in the
				// table. Note that the span end key is exclusive, thus ">0"
				// instead of ">=0".
				return true
			}
		}
		// Assume overlap if iterator errored.
		if err := rIter.Error(); err != nil {
			return true
		}
		return false
	}

	// rkeyIter is either a range key level iter, or a range key iterator
	// over a single file.
	if rkeyIter != nil {
		if computeOverlapWithSpans(rkeyIter) {
			return true
		}
	}

	// Check overlap with range deletions.
	if rangeDelIter == nil || *rangeDelIter == nil {
		return false
	}
	return computeOverlapWithSpans(*rangeDelIter)
}

// ingestTargetLevel returns the target level for a file being ingested.
// If suggestSplit is true, it accounts for ingest-time splitting as part of
// its target level calculation, and if a split candidate is found, that file
// is returned as the splitFile.
func ingestTargetLevel(
	newIters tableNewIters,
	newRangeKeyIter keyspan.TableNewSpanIter,
	iterOps IterOptions,
	comparer *Comparer,
	v *version,
	baseLevel int,
	compactions map[*compaction]struct{},
	meta *fileMetadata,
	suggestSplit bool,
) (targetLevel int, splitFile *fileMetadata, err error) {
	// Find the lowest level which does not have any files which overlap meta. We
	// search from L0 to L6 looking for whether there are any files in the level
	// which overlap meta. We want the "lowest" level (where lower means
	// increasing level number) in order to reduce write amplification.
	//
	// There are 2 kinds of overlap we need to check for: file boundary overlap
	// and data overlap. Data overlap implies file boundary overlap. Note that it
	// is always possible to ingest into L0.
	//
	// To place meta at level i where i > 0:
	// - there must not be any data overlap with levels <= i, since that will
	//   violate the sequence number invariant.
	// - no file boundary overlap with level i, since that will violate the
	//   invariant that files do not overlap in levels i > 0.
	//   - if there is only a file overlap at a given level, and no data overlap,
	//     we can still slot a file at that level. We return the fileMetadata with
	//     which we have file boundary overlap (must be only one file, as sstable
	//     bounds are usually tight on user keys) and the caller is expected to split
	//     that sstable into two virtual sstables, allowing this file to go into that
	//     level. Note that if we have file boundary overlap with two files, which
	//     should only happen on rare occasions, we treat it as data overlap and
	//     don't use this optimization.
	//
	// The file boundary overlap check is simpler to conceptualize. Consider the
	// following example, in which the ingested file lies completely before or
	// after the file being considered.
	//
	//   |--|           |--|  ingested file: [a,b] or [f,g]
	//         |-----|        existing file: [c,e]
	//  _____________________
	//   a  b  c  d  e  f  g
	//
	// In both cases the ingested file can move to considering the next level.
	//
	// File boundary overlap does not necessarily imply data overlap. The check
	// for data overlap is a little more nuanced. Consider the following examples:
	//
	//  1. No data overlap:
	//
	//          |-|   |--|    ingested file: [cc-d] or [ee-ff]
	//  |*--*--*----*------*| existing file: [a-g], points: [a, b, c, dd, g]
	//  _____________________
	//   a  b  c  d  e  f  g
	//
	// In this case the ingested files can "fall through" this level. The checks
	// continue at the next level.
	//
	//  2. Data overlap:
	//
	//            |--|        ingested file: [d-e]
	//  |*--*--*----*------*| existing file: [a-g], points: [a, b, c, dd, g]
	//  _____________________
	//   a  b  c  d  e  f  g
	//
	// In this case the file cannot be ingested into this level as the point 'dd'
	// is in the way.
	//
	// It is worth noting that the check for data overlap is only approximate. In
	// the previous example, the ingested table [d-e] could contain only the
	// points 'd' and 'e', in which case the table would be eligible for
	// considering lower levels. However, such a fine-grained check would need to
	// be exhaustive (comparing points and ranges in both the ingested existing
	// tables) and such a check is prohibitively expensive. Thus Pebble treats any
	// existing point that falls within the ingested table bounds as being "data
	// overlap".

	// This assertion implicitly checks that we have the current version of
	// the metadata.
	if v.L0Sublevels == nil {
		return 0, nil, errors.AssertionFailedf("could not read L0 sublevels")
	}
	// Check for overlap over the keys of L0 by iterating over the sublevels.
	for subLevel := 0; subLevel < len(v.L0SublevelFiles); subLevel++ {
		iter := newLevelIter(iterOps, comparer, newIters,
			v.L0Sublevels.Levels[subLevel].Iter(), manifest.Level(0), internalIterOpts{})

		var rangeDelIter keyspan.FragmentIterator
		// Pass in a non-nil pointer to rangeDelIter so that levelIter.findFileGE
		// sets it up for the target file.
		iter.initRangeDel(&rangeDelIter)

		levelIter := keyspan.LevelIter{}
		levelIter.Init(
			keyspan.SpanIterOptions{}, comparer.Compare, newRangeKeyIter,
			v.L0Sublevels.Levels[subLevel].Iter(), manifest.Level(0), manifest.KeyTypeRange,
		)

		kr := internalKeyRange{
			smallest: meta.Smallest,
			largest:  meta.Largest,
		}
		overlap := overlapWithIterator(iter, &rangeDelIter, &levelIter, kr, comparer.Compare)
		err := iter.Close() // Closes range del iter as well.
		err = firstError(err, levelIter.Close())
		if err != nil {
			return 0, nil, err
		}
		if overlap {
			return targetLevel, nil, nil
		}
	}

	level := baseLevel
	for ; level < numLevels; level++ {
		levelIter := newLevelIter(iterOps, comparer, newIters,
			v.Levels[level].Iter(), manifest.Level(level), internalIterOpts{})
		var rangeDelIter keyspan.FragmentIterator
		// Pass in a non-nil pointer to rangeDelIter so that levelIter.findFileGE
		// sets it up for the target file.
		levelIter.initRangeDel(&rangeDelIter)

		rkeyLevelIter := &keyspan.LevelIter{}
		rkeyLevelIter.Init(
			keyspan.SpanIterOptions{}, comparer.Compare, newRangeKeyIter,
			v.Levels[level].Iter(), manifest.Level(level), manifest.KeyTypeRange,
		)

		kr := internalKeyRange{
			smallest: meta.Smallest,
			largest:  meta.Largest,
		}
		overlap := overlapWithIterator(levelIter, &rangeDelIter, rkeyLevelIter, kr, comparer.Compare)
		err := levelIter.Close() // Closes range del iter as well.
		err = firstError(err, rkeyLevelIter.Close())
		if err != nil {
			return 0, nil, err
		}
		if overlap {
			return targetLevel, splitFile, nil
		}

		// Check boundary overlap.
		var candidateSplitFile *fileMetadata
		boundaryOverlaps := v.Overlaps(level, comparer.Compare, meta.Smallest.UserKey,
			meta.Largest.UserKey, meta.Largest.IsExclusiveSentinel())
		if !boundaryOverlaps.Empty() {
			// We are already guaranteed to not have any data overlaps with files
			// in boundaryOverlaps, otherwise we'd have returned in the above if
			// statements. Use this, plus boundaryOverlaps.Len() == 1 to detect for
			// the case where we can slot this file into the current level despite
			// a boundary overlap, by splitting one existing file into two virtual
			// sstables.
			if suggestSplit && boundaryOverlaps.Len() == 1 {
				iter := boundaryOverlaps.Iter()
				candidateSplitFile = iter.First()
			} else {
				// We either don't want to suggest ingest-time splits (i.e.
				// !suggestSplit), or we boundary-overlapped with more than one file.
				continue
			}
		}

		// Check boundary overlap with any ongoing compactions. We consider an
		// overlapping compaction that's writing files to an output level as
		// equivalent to boundary overlap with files in that output level.
		//
		// We cannot check for data overlap with the new SSTs compaction will produce
		// since compaction hasn't been done yet. However, there's no need to check
		// since all keys in them will be from levels in [c.startLevel,
		// c.outputLevel], and all those levels have already had their data overlap
		// tested negative (else we'd have returned earlier).
		//
		// An alternative approach would be to cancel these compactions and proceed
		// with an ingest-time split on this level if necessary. However, compaction
		// cancellation can result in significant wasted effort and is best avoided
		// unless necessary.
		overlaps := false
		for c := range compactions {
			if c.outputLevel == nil || level != c.outputLevel.level {
				continue
			}
			if comparer.Compare(meta.Smallest.UserKey, c.largest.UserKey) <= 0 &&
				comparer.Compare(meta.Largest.UserKey, c.smallest.UserKey) >= 0 {
				overlaps = true
				break
			}
		}
		if !overlaps {
			targetLevel = level
			splitFile = candidateSplitFile
		}
	}
	return targetLevel, splitFile, nil
}

// Ingest ingests a set of sstables into the DB. Ingestion of the files is
// atomic and semantically equivalent to creating a single batch containing all
// of the mutations in the sstables. Ingestion may require the memtable to be
// flushed. The ingested sstable files are moved into the DB and must reside on
// the same filesystem as the DB. Sstables can be created for ingestion using
// sstable.Writer. On success, Ingest removes the input paths.
//
// Two types of sstables are accepted for ingestion(s): one is sstables present
// in the instance's vfs.FS and can be referenced locally. The other is sstables
// present in remote.Storage, referred to as shared or foreign sstables. These
// shared sstables can be linked through objstorageprovider.Provider, and do not
// need to already be present on the local vfs.FS. Foreign sstables must all fit
// in an excise span, and are destined for a level specified in SharedSSTMeta.
//
// All sstables *must* be Sync()'d by the caller after all bytes are written
// and before its file handle is closed; failure to do so could violate
// durability or lead to corrupted on-disk state. This method cannot, in a
// platform-and-FS-agnostic way, ensure that all sstables in the input are
// properly synced to disk. Opening new file handles and Sync()-ing them
// does not always guarantee durability; see the discussion here on that:
// https://github.com/cockroachdb/pebble/pull/835#issuecomment-663075379
//
// Ingestion loads each sstable into the lowest level of the LSM which it
// doesn't overlap (see ingestTargetLevel). If an sstable overlaps a memtable,
// ingestion forces the memtable to flush, and then waits for the flush to
// occur. In some cases, such as with no foreign sstables and no excise span,
// ingestion that gets blocked on a memtable can join the flushable queue and
// finish even before the memtable has been flushed.
//
// The steps for ingestion are:
//
//  1. Allocate file numbers for every sstable being ingested.
//  2. Load the metadata for all sstables being ingested.
//  3. Sort the sstables by smallest key, verifying non overlap (for local
//     sstables).
//  4. Hard link (or copy) the local sstables into the DB directory.
//  5. Allocate a sequence number to use for all of the entries in the
//     local sstables. This is the step where overlap with memtables is
//     determined. If there is overlap, we remember the most recent memtable
//     that overlaps.
//  6. Update the sequence number in the ingested local sstables. (Remote
//     sstables get fixed sequence numbers that were determined at load time.)
//  7. Wait for the most recent memtable that overlaps to flush (if any).
//  8. Add the ingested sstables to the version (DB.ingestApply).
//     8.1.  If an excise span was specified, figure out what sstables in the
//     current version overlap with the excise span, and create new virtual
//     sstables out of those sstables that exclude the excised span (DB.excise).
//  9. Publish the ingestion sequence number.
//
// Note that if the mutable memtable overlaps with ingestion, a flush of the
// memtable is forced equivalent to DB.Flush. Additionally, subsequent
// mutations that get sequence numbers larger than the ingestion sequence
// number get queued up behind the ingestion waiting for it to complete. This
// can produce a noticeable hiccup in performance. See
// https://github.com/cockroachdb/pebble/issues/25 for an idea for how to fix
// this hiccup.
func (d *DB) Ingest(paths []string) error {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.ReadOnly {
		return ErrReadOnly
	}
	_, err := d.ingest(paths, ingestTargetLevel, nil /* shared */, KeyRange{}, nil /* external */)
	return err
}

// IngestOperationStats provides some information about where in the LSM the
// bytes were ingested.
type IngestOperationStats struct {
	// Bytes is the total bytes in the ingested sstables.
	Bytes uint64
	// ApproxIngestedIntoL0Bytes is the approximate number of bytes ingested
	// into L0. This value is approximate when flushable ingests are active and
	// an ingest overlaps an entry in the flushable queue. Currently, this
	// approximation is very rough, only including tables that overlapped the
	// memtable. This estimate may be improved with #2112.
	ApproxIngestedIntoL0Bytes uint64
	// MemtableOverlappingFiles is the count of ingested sstables
	// that overlapped keys in the memtables.
	MemtableOverlappingFiles int
}

// ExternalFile are external sstables that can be referenced through
// objprovider and ingested as remote files that will not be refcounted or
// cleaned up. For use with online restore. Note that the underlying sstable
// could contain keys outside the [Smallest,Largest) bounds; however Pebble
// is expected to only read the keys within those bounds.
type ExternalFile struct {
	// Locator is the shared.Locator that can be used with objProvider to
	// resolve a reference to this external sstable.
	Locator remote.Locator
	// ObjName is the unique name of this sstable on Locator.
	ObjName string
	// Size of the referenced proportion of the virtualized sstable. An estimate
	// is acceptable in lieu of the backing file size.
	Size uint64
	// SmallestUserKey and LargestUserKey are the [smallest,largest) user key
	// bounds of the sstable. Both these bounds are loose i.e. it's possible for
	// the sstable to not span the entirety of this range. However, multiple
	// ExternalFiles in one ingestion must all have non-overlapping
	// [smallest, largest) spans. Note that this Largest bound is exclusive.
	SmallestUserKey, LargestUserKey []byte
	// HasPointKey and HasRangeKey denote whether this file contains point keys
	// or range keys. If both structs are false, an error is returned during
	// ingestion.
	HasPointKey, HasRangeKey bool
}

// IngestWithStats does the same as Ingest, and additionally returns
// IngestOperationStats.
func (d *DB) IngestWithStats(paths []string) (IngestOperationStats, error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.ReadOnly {
		return IngestOperationStats{}, ErrReadOnly
	}
	return d.ingest(paths, ingestTargetLevel, nil /* shared */, KeyRange{}, nil /* external */)
}

// IngestExternalFiles does the same as IngestWithStats, and additionally
// accepts external files (with locator info that can be resolved using
// d.opts.SharedStorage). These files must also be non-overlapping with
// each other, and must be resolvable through d.objProvider.
func (d *DB) IngestExternalFiles(external []ExternalFile) (IngestOperationStats, error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	if d.opts.ReadOnly {
		return IngestOperationStats{}, ErrReadOnly
	}
	if d.opts.Experimental.RemoteStorage == nil {
		return IngestOperationStats{}, errors.New("pebble: cannot ingest external files without shared storage configured")
	}
	return d.ingest(nil, ingestTargetLevel, nil /* shared */, KeyRange{}, external)
}

// IngestAndExcise does the same as IngestWithStats, and additionally accepts a
// list of shared files to ingest that can be read from a remote.Storage through
// a Provider. All the shared files must live within exciseSpan, and any existing
// keys in exciseSpan are deleted by turning existing sstables into virtual
// sstables (if not virtual already) and shrinking their spans to exclude
// exciseSpan. See the comment at Ingest for a more complete picture of the
// ingestion process.
//
// Panics if this DB instance was not instantiated with a remote.Storage and
// shared sstables are present.
func (d *DB) IngestAndExcise(
	paths []string, shared []SharedSSTMeta, exciseSpan KeyRange,
) (IngestOperationStats, error) {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}
	if d.opts.ReadOnly {
		return IngestOperationStats{}, ErrReadOnly
	}
	return d.ingest(paths, ingestTargetLevel, shared, exciseSpan, nil /* external */)
}

// Both DB.mu and commitPipeline.mu must be held while this is called.
func (d *DB) newIngestedFlushableEntry(
	meta []*fileMetadata, seqNum uint64, logNum FileNum,
) (*flushableEntry, error) {
	// Update the sequence number for all of the sstables in the
	// metadata. Writing the metadata to the manifest when the
	// version edit is applied is the mechanism that persists the
	// sequence number. The sstables themselves are left unmodified.
	// In this case, a version edit will only be written to the manifest
	// when the flushable is eventually flushed. If Pebble restarts in that
	// time, then we'll lose the ingest sequence number information. But this
	// information will also be reconstructed on node restart.
	if err := ingestUpdateSeqNum(
		d.cmp, d.opts.Comparer.FormatKey, seqNum, ingestLoadResult{localMeta: meta},
	); err != nil {
		return nil, err
	}

	f := newIngestedFlushable(meta, d.opts.Comparer, d.newIters, d.tableNewRangeKeyIter)

	// NB: The logNum/seqNum are the WAL number which we're writing this entry
	// to and the sequence number within the WAL which we'll write this entry
	// to.
	entry := d.newFlushableEntry(f, logNum, seqNum)
	// The flushable entry starts off with a single reader ref, so increment
	// the FileMetadata.Refs.
	for _, file := range f.files {
		file.Ref()
	}
	entry.unrefFiles = func() []*fileBacking {
		var obsolete []*fileBacking
		for _, file := range f.files {
			if file.Unref() == 0 {
				obsolete = append(obsolete, file.FileMetadata.FileBacking)
			}
		}
		return obsolete
	}

	entry.flushForced = true
	entry.releaseMemAccounting = func() {}
	return entry, nil
}

// Both DB.mu and commitPipeline.mu must be held while this is called. Since
// we're holding both locks, the order in which we rotate the memtable or
// recycle the WAL in this function is irrelevant as long as the correct log
// numbers are assigned to the appropriate flushable.
func (d *DB) handleIngestAsFlushable(meta []*fileMetadata, seqNum uint64) error {
	b := d.NewBatch()
	for _, m := range meta {
		b.ingestSST(m.FileNum)
	}
	b.setSeqNum(seqNum)

	// If the WAL is disabled, then the logNum used to create the flushable
	// entry doesn't matter. We just use the logNum assigned to the current
	// mutable memtable. If the WAL is enabled, then this logNum will be
	// overwritten by the logNum of the log which will contain the log entry
	// for the ingestedFlushable.
	logNum := d.mu.mem.queue[len(d.mu.mem.queue)-1].logNum
	if !d.opts.DisableWAL {
		// We create a new WAL for the flushable instead of reusing the end of
		// the previous WAL. This simplifies the increment of the minimum
		// unflushed log number, and also simplifies WAL replay.
		var prevLogSize uint64
		logNum, prevLogSize = d.rotateWAL()
		// As the rotator of the WAL, we're responsible for updating the
		// previous flushable queue tail's log size.
		d.mu.mem.queue[len(d.mu.mem.queue)-1].logSize = prevLogSize

		d.mu.Unlock()
		err := d.commit.directWrite(b)
		if err != nil {
			d.opts.Logger.Fatalf("%v", err)
		}
		d.mu.Lock()
	}

	entry, err := d.newIngestedFlushableEntry(meta, seqNum, logNum)
	if err != nil {
		return err
	}
	nextSeqNum := seqNum + uint64(b.Count())

	// Set newLogNum to the logNum of the previous flushable. This value is
	// irrelevant if the WAL is disabled. If the WAL is enabled, then we set
	// the appropriate value below.
	newLogNum := d.mu.mem.queue[len(d.mu.mem.queue)-1].logNum
	if !d.opts.DisableWAL {
		// newLogNum will be the WAL num of the next mutable memtable which
		// comes after the ingestedFlushable in the flushable queue. The mutable
		// memtable will be created below.
		//
		// The prevLogSize returned by rotateWAL is the WAL to which the
		// flushable ingest keys were appended. This intermediary WAL is only
		// used to record the flushable ingest and nothing else.
		newLogNum, entry.logSize = d.rotateWAL()
	}

	currMem := d.mu.mem.mutable
	// NB: Placing ingested sstables above the current memtables
	// requires rotating of the existing memtables/WAL. There is
	// some concern of churning through tiny memtables due to
	// ingested sstables being placed on top of them, but those
	// memtables would have to be flushed anyways.
	d.mu.mem.queue = append(d.mu.mem.queue, entry)
	d.rotateMemtable(newLogNum, nextSeqNum, currMem)
	d.updateReadStateLocked(d.opts.DebugCheck)
	d.maybeScheduleFlush()
	return nil
}

// See comment at Ingest() for details on how this works.
func (d *DB) ingest(
	paths []string,
	targetLevelFunc ingestTargetLevelFunc,
	shared []SharedSSTMeta,
	exciseSpan KeyRange,
	external []ExternalFile,
) (IngestOperationStats, error) {
	if len(shared) > 0 && d.opts.Experimental.RemoteStorage == nil {
		panic("cannot ingest shared sstables with nil SharedStorage")
	}
	if (exciseSpan.Valid() || len(shared) > 0 || len(external) > 0) && d.FormatMajorVersion() < FormatVirtualSSTables {
		return IngestOperationStats{}, errors.New("pebble: format major version too old for excise, shared or external sstable ingestion")
	}
	// Allocate file numbers for all of the files being ingested and mark them as
	// pending in order to prevent them from being deleted. Note that this causes
	// the file number ordering to be out of alignment with sequence number
	// ordering. The sorting of L0 tables by sequence number avoids relying on
	// that (busted) invariant.
	d.mu.Lock()
	pendingOutputs := make([]base.DiskFileNum, len(paths)+len(shared)+len(external))
	for i := 0; i < len(paths)+len(shared)+len(external); i++ {
		pendingOutputs[i] = d.mu.versions.getNextFileNum().DiskFileNum()
	}

	jobID := d.mu.nextJobID
	d.mu.nextJobID++
	d.mu.Unlock()

	// Load the metadata for all the files being ingested. This step detects
	// and elides empty sstables.
	loadResult, err := ingestLoad(d.opts, d.FormatMajorVersion(), paths, shared, external, d.cacheID, pendingOutputs, d.objProvider, jobID)
	if err != nil {
		return IngestOperationStats{}, err
	}

	if loadResult.fileCount == 0 {
		// All of the sstables to be ingested were empty. Nothing to do.
		return IngestOperationStats{}, nil
	}

	// Verify the sstables do not overlap.
	if err := ingestSortAndVerify(d.cmp, loadResult, exciseSpan); err != nil {
		return IngestOperationStats{}, err
	}

	// Hard link the sstables into the DB directory. Since the sstables aren't
	// referenced by a version, they won't be used. If the hard linking fails
	// (e.g. because the files reside on a different filesystem), ingestLink will
	// fall back to copying, and if that fails we undo our work and return an
	// error.
	if err := ingestLink(jobID, d.opts, d.objProvider, loadResult, shared); err != nil {
		return IngestOperationStats{}, err
	}

	// Make the new tables durable. We need to do this at some point before we
	// update the MANIFEST (via logAndApply), otherwise a crash can have the
	// tables referenced in the MANIFEST, but not present in the provider.
	if err := d.objProvider.Sync(); err != nil {
		return IngestOperationStats{}, err
	}

	// metaFlushableOverlaps is a slice parallel to meta indicating which of the
	// ingested sstables overlap some table in the flushable queue. It's used to
	// approximate ingest-into-L0 stats when using flushable ingests.
	metaFlushableOverlaps := make([]bool, loadResult.fileCount)
	var mem *flushableEntry
	var mut *memTable
	// asFlushable indicates whether the sstable was ingested as a flushable.
	var asFlushable bool
	prepare := func(seqNum uint64) {
		// Note that d.commit.mu is held by commitPipeline when calling prepare.

		d.mu.Lock()
		defer d.mu.Unlock()

		// Check to see if any files overlap with any of the memtables. The queue
		// is ordered from oldest to newest with the mutable memtable being the
		// last element in the slice. We want to wait for the newest table that
		// overlaps.

		for i := len(d.mu.mem.queue) - 1; i >= 0; i-- {
			m := d.mu.mem.queue[i]
			iter := m.newIter(nil)
			rangeDelIter := m.newRangeDelIter(nil)
			rkeyIter := m.newRangeKeyIter(nil)

			checkForOverlap := func(i int, meta *fileMetadata) {
				if metaFlushableOverlaps[i] {
					// This table already overlapped a more recent flushable.
					return
				}
				kr := internalKeyRange{
					smallest: meta.Smallest,
					largest:  meta.Largest,
				}
				if overlapWithIterator(iter, &rangeDelIter, rkeyIter, kr, d.cmp) {
					// If this is the first table to overlap a flushable, save
					// the flushable. This ingest must be ingested or flushed
					// after it.
					if mem == nil {
						mem = m
					}
					metaFlushableOverlaps[i] = true
				}
			}
			for i := range loadResult.localMeta {
				checkForOverlap(i, loadResult.localMeta[i])
			}
			for i := range loadResult.sharedMeta {
				checkForOverlap(len(loadResult.localMeta)+i, loadResult.sharedMeta[i])
			}
			for i := range loadResult.externalMeta {
				checkForOverlap(len(loadResult.localMeta)+len(loadResult.sharedMeta)+i, loadResult.externalMeta[i])
			}
			if exciseSpan.Valid() {
				kr := internalKeyRange{
					smallest: base.MakeInternalKey(exciseSpan.Start, InternalKeySeqNumMax, InternalKeyKindMax),
					largest:  base.MakeExclusiveSentinelKey(InternalKeyKindRangeDelete, exciseSpan.End),
				}
				if overlapWithIterator(iter, &rangeDelIter, rkeyIter, kr, d.cmp) {
					if mem == nil {
						mem = m
					}
				}
			}
			err := iter.Close()
			if rangeDelIter != nil {
				err = firstError(err, rangeDelIter.Close())
			}
			if rkeyIter != nil {
				err = firstError(err, rkeyIter.Close())
			}
			if err != nil {
				d.opts.Logger.Infof("ingest error reading flushable for log %s: %s", m.logNum, err)
			}
		}

		if mem == nil {
			// No overlap with any of the queued flushables, so no need to queue
			// after them.

			// New writes with higher sequence numbers may be concurrently
			// committed. We must ensure they don't flush before this ingest
			// completes. To do that, we ref the mutable memtable as a writer,
			// preventing its flushing (and the flushing of all subsequent
			// flushables in the queue). Once we've acquired the manifest lock
			// to add the ingested sstables to the LSM, we can unref as we're
			// guaranteed that the flush won't edit the LSM before this ingest.
			mut = d.mu.mem.mutable
			mut.writerRef()
			return
		}
		// The ingestion overlaps with some entry in the flushable queue.
		if d.FormatMajorVersion() < FormatFlushableIngest ||
			d.opts.Experimental.DisableIngestAsFlushable() ||
			len(shared) > 0 || exciseSpan.Valid() || len(external) > 0 ||
			(len(d.mu.mem.queue) > d.opts.MemTableStopWritesThreshold-1) {
			// We're not able to ingest as a flushable,
			// so we must synchronously flush.
			//
			// TODO(bilal): Currently, if any of the files being ingested are shared or
			// there's an excise span present, we cannot use flushable ingests and need
			// to wait synchronously. Either remove this caveat by fleshing out
			// flushable ingest logic to also account for these cases, or remove this
			// comment. Tracking issue: https://github.com/cockroachdb/pebble/issues/2676
			if mem.flushable == d.mu.mem.mutable {
				err = d.makeRoomForWrite(nil)
			}
			// New writes with higher sequence numbers may be concurrently
			// committed. We must ensure they don't flush before this ingest
			// completes. To do that, we ref the mutable memtable as a writer,
			// preventing its flushing (and the flushing of all subsequent
			// flushables in the queue). Once we've acquired the manifest lock
			// to add the ingested sstables to the LSM, we can unref as we're
			// guaranteed that the flush won't edit the LSM before this ingest.
			mut = d.mu.mem.mutable
			mut.writerRef()
			mem.flushForced = true
			d.maybeScheduleFlush()
			return
		}
		// Since there aren't too many memtables already queued up, we can
		// slide the ingested sstables on top of the existing memtables.
		asFlushable = true
		err = d.handleIngestAsFlushable(loadResult.localMeta, seqNum)
	}

	var ve *versionEdit
	apply := func(seqNum uint64) {
		if err != nil || asFlushable {
			// An error occurred during prepare.
			if mut != nil {
				if mut.writerUnref() {
					d.mu.Lock()
					d.maybeScheduleFlush()
					d.mu.Unlock()
				}
			}
			return
		}

		// Update the sequence numbers for all ingested sstables'
		// metadata. When the version edit is applied, the metadata is
		// written to the manifest, persisting the sequence number.
		// The sstables themselves are left unmodified.
		if err = ingestUpdateSeqNum(
			d.cmp, d.opts.Comparer.FormatKey, seqNum, loadResult,
		); err != nil {
			if mut != nil {
				if mut.writerUnref() {
					d.mu.Lock()
					d.maybeScheduleFlush()
					d.mu.Unlock()
				}
			}
			return
		}

		// If we overlapped with a memtable in prepare wait for the flush to
		// finish.
		if mem != nil {
			<-mem.flushed
		}

		// Assign the sstables to the correct level in the LSM and apply the
		// version edit.
		ve, err = d.ingestApply(jobID, loadResult, targetLevelFunc, mut, exciseSpan)
	}

	// Only one ingest can occur at a time because if not, one would block waiting
	// for the other to finish applying. This blocking would happen while holding
	// the commit mutex which would prevent unrelated batches from writing their
	// changes to the WAL and memtable. This will cause a bigger commit hiccup
	// during ingestion.
	d.commit.ingestSem <- struct{}{}
	d.commit.AllocateSeqNum(loadResult.fileCount, prepare, apply)
	<-d.commit.ingestSem

	if err != nil {
		if err2 := ingestCleanup(d.objProvider, loadResult.localMeta); err2 != nil {
			d.opts.Logger.Infof("ingest cleanup failed: %v", err2)
		}
	} else {
		// Since we either created a hard link to the ingesting files, or copied
		// them over, it is safe to remove the originals paths.
		for _, path := range loadResult.localPaths {
			if err2 := d.opts.FS.Remove(path); err2 != nil {
				d.opts.Logger.Infof("ingest failed to remove original file: %s", err2)
			}
		}
	}

	if invariants.Enabled {
		for _, sharedMeta := range loadResult.sharedMeta {
			d.checkVirtualBounds(sharedMeta)
		}
	}

	info := TableIngestInfo{
		JobID:     jobID,
		Err:       err,
		flushable: asFlushable,
	}
	if len(loadResult.localMeta) > 0 {
		info.GlobalSeqNum = loadResult.localMeta[0].SmallestSeqNum
	} else if len(loadResult.sharedMeta) > 0 {
		info.GlobalSeqNum = loadResult.sharedMeta[0].SmallestSeqNum
	} else {
		info.GlobalSeqNum = loadResult.externalMeta[0].SmallestSeqNum
	}
	var stats IngestOperationStats
	if ve != nil {
		info.Tables = make([]struct {
			TableInfo
			Level int
		}, len(ve.NewFiles))
		for i := range ve.NewFiles {
			e := &ve.NewFiles[i]
			info.Tables[i].Level = e.Level
			info.Tables[i].TableInfo = e.Meta.TableInfo()
			stats.Bytes += e.Meta.Size
			if e.Level == 0 {
				stats.ApproxIngestedIntoL0Bytes += e.Meta.Size
			}
			if i < len(metaFlushableOverlaps) && metaFlushableOverlaps[i] {
				stats.MemtableOverlappingFiles++
			}
		}
	} else if asFlushable {
		// NB: If asFlushable == true, there are no shared sstables.
		info.Tables = make([]struct {
			TableInfo
			Level int
		}, len(loadResult.localMeta))
		for i, f := range loadResult.localMeta {
			info.Tables[i].Level = -1
			info.Tables[i].TableInfo = f.TableInfo()
			stats.Bytes += f.Size
			// We don't have exact stats on which files will be ingested into
			// L0, because actual ingestion into the LSM has been deferred until
			// flush time. Instead, we infer based on memtable overlap.
			//
			// TODO(jackson): If we optimistically compute data overlap (#2112)
			// before entering the commit pipeline, we can use that overlap to
			// improve our approximation by incorporating overlap with L0, not
			// just memtables.
			if metaFlushableOverlaps[i] {
				stats.ApproxIngestedIntoL0Bytes += f.Size
				stats.MemtableOverlappingFiles++
			}
		}
	}
	d.opts.EventListener.TableIngested(info)

	return stats, err
}

// excise updates ve to include a replacement of the file m with new virtual
// sstables that exclude exciseSpan, returning a slice of newly-created files if
// any. If the entirety of m is deleted by exciseSpan, no new sstables are added
// and m is deleted. Note that ve is updated in-place.
//
// The manifest lock must be held when calling this method.
func (d *DB) excise(
	exciseSpan KeyRange, m *fileMetadata, ve *versionEdit, level int,
) ([]manifest.NewFileEntry, error) {
	numCreatedFiles := 0
	// Check if there's actually an overlap between m and exciseSpan.
	if !exciseSpan.Overlaps(d.cmp, m) {
		return nil, nil
	}
	ve.DeletedFiles[deletedFileEntry{
		Level:   level,
		FileNum: m.FileNum,
	}] = m
	// Fast path: m sits entirely within the exciseSpan, so just delete it.
	if exciseSpan.Contains(d.cmp, m.Smallest) && exciseSpan.Contains(d.cmp, m.Largest) {
		return nil, nil
	}
	var iter internalIterator
	var rangeDelIter keyspan.FragmentIterator
	var rangeKeyIter keyspan.FragmentIterator
	needsBacking := false
	// Create a file to the left of the excise span, if necessary.
	// The bounds of this file will be [m.Smallest, lastKeyBefore(exciseSpan.Start)].
	//
	// We create bounds that are tight on user keys, and we make the effort to find
	// the last key in the original sstable that's smaller than exciseSpan.Start
	// even though it requires some sstable reads. We could choose to create
	// virtual sstables on loose userKey bounds, in which case we could just set
	// leftFile.Largest to an exclusive sentinel at exciseSpan.Start. The biggest
	// issue with that approach would be that it'd lead to lots of small virtual
	// sstables in the LSM that have no guarantee on containing even a single user
	// key within the file bounds. This has the potential to increase both read and
	// write-amp as we will be opening up these sstables only to find no relevant
	// keys in the read path, and compacting sstables on top of them instead of
	// directly into the space occupied by them. We choose to incur the cost of
	// calculating tight bounds at this time instead of creating more work in the
	// future.
	//
	// TODO(bilal): Some of this work can happen without grabbing the manifest
	// lock; we could grab one currentVersion, release the lock, calculate excised
	// files, then grab the lock again and recalculate for just the files that
	// have changed since our previous calculation. Do this optimiaztino as part of
	// https://github.com/cockroachdb/pebble/issues/2112 .
	if d.cmp(m.Smallest.UserKey, exciseSpan.Start) < 0 {
		leftFile := &fileMetadata{
			Virtual:     true,
			FileBacking: m.FileBacking,
			FileNum:     d.mu.versions.getNextFileNum(),
			// Note that these are loose bounds for smallest/largest seqnums, but they're
			// sufficient for maintaining correctness.
			SmallestSeqNum: m.SmallestSeqNum,
			LargestSeqNum:  m.LargestSeqNum,
		}
		if m.HasPointKeys && !exciseSpan.Contains(d.cmp, m.SmallestPointKey) {
			// This file will contain point keys
			smallestPointKey := m.SmallestPointKey
			var err error
			iter, rangeDelIter, err = d.newIters(context.TODO(), m, &IterOptions{level: manifest.Level(level)}, internalIterOpts{})
			if err != nil {
				return nil, err
			}
			var key *InternalKey
			if iter != nil {
				defer iter.Close()
				key, _ = iter.SeekLT(exciseSpan.Start, base.SeekLTFlagsNone)
			} else {
				iter = emptyIter
			}
			if key != nil {
				leftFile.ExtendPointKeyBounds(d.cmp, smallestPointKey, key.Clone())
			}
			// Store the min of (exciseSpan.Start, rdel.End) in lastRangeDel. This
			// needs to be a copy if the key is owned by the range del iter.
			var lastRangeDel []byte
			if rangeDelIter != nil {
				defer rangeDelIter.Close()
				rdel := rangeDelIter.SeekLT(exciseSpan.Start)
				if rdel != nil {
					lastRangeDel = append(lastRangeDel[:0], rdel.End...)
					if d.cmp(lastRangeDel, exciseSpan.Start) > 0 {
						lastRangeDel = exciseSpan.Start
					}
				}
			} else {
				rangeDelIter = emptyKeyspanIter
			}
			if lastRangeDel != nil {
				leftFile.ExtendPointKeyBounds(d.cmp, smallestPointKey, base.MakeExclusiveSentinelKey(InternalKeyKindRangeDelete, lastRangeDel))
			}
		}
		if m.HasRangeKeys && !exciseSpan.Contains(d.cmp, m.SmallestRangeKey) {
			// This file will contain range keys
			var err error
			smallestRangeKey := m.SmallestRangeKey
			rangeKeyIter, err = d.tableNewRangeKeyIter(m, keyspan.SpanIterOptions{})
			if err != nil {
				return nil, err
			}
			// Store the min of (exciseSpan.Start, rkey.End) in lastRangeKey. This
			// needs to be a copy if the key is owned by the range key iter.
			var lastRangeKey []byte
			var lastRangeKeyKind InternalKeyKind
			defer rangeKeyIter.Close()
			rkey := rangeKeyIter.SeekLT(exciseSpan.Start)
			if rkey != nil {
				lastRangeKey = append(lastRangeKey[:0], rkey.End...)
				if d.cmp(lastRangeKey, exciseSpan.Start) > 0 {
					lastRangeKey = exciseSpan.Start
				}
				lastRangeKeyKind = rkey.Keys[0].Kind()
			}
			if lastRangeKey != nil {
				leftFile.ExtendRangeKeyBounds(d.cmp, smallestRangeKey, base.MakeExclusiveSentinelKey(lastRangeKeyKind, lastRangeKey))
			}
		}
		if leftFile.HasRangeKeys || leftFile.HasPointKeys {
			var err error
			leftFile.Size, err = d.tableCache.estimateSize(m, leftFile.Smallest.UserKey, leftFile.Largest.UserKey)
			if err != nil {
				return nil, err
			}
			if leftFile.Size == 0 {
				// On occasion, estimateSize gives us a low estimate, i.e. a 0 file size,
				// such as if the excised file only has range keys/dels and no point
				// keys. This can cause panics in places where we divide by file sizes.
				// Correct for it here.
				leftFile.Size = 1
			}
			if err := leftFile.Validate(d.cmp, d.opts.Comparer.FormatKey); err != nil {
				return nil, err
			}
			leftFile.ValidateVirtual(m)
			d.checkVirtualBounds(leftFile)
			ve.NewFiles = append(ve.NewFiles, newFileEntry{Level: level, Meta: leftFile})
			needsBacking = true
			numCreatedFiles++
		}
	}
	// Create a file to the right, if necessary.
	if exciseSpan.Contains(d.cmp, m.Largest) {
		// No key exists to the right of the excise span in this file.
		if needsBacking && !m.Virtual {
			// If m is virtual, then its file backing is already known to the manifest.
			// We don't need to create another file backing. Note that there must be
			// only one CreatedBackingTables entry per backing sstable. This is
			// indicated by the VersionEdit.CreatedBackingTables invariant.
			ve.CreatedBackingTables = append(ve.CreatedBackingTables, m.FileBacking)
		}
		return ve.NewFiles[len(ve.NewFiles)-numCreatedFiles:], nil
	}
	// Create a new file, rightFile, between [firstKeyAfter(exciseSpan.End), m.Largest].
	//
	// See comment before the definition of leftFile for the motivation behind
	// calculating tight user-key bounds.
	rightFile := &fileMetadata{
		Virtual:     true,
		FileBacking: m.FileBacking,
		FileNum:     d.mu.versions.getNextFileNum(),
		// Note that these are loose bounds for smallest/largest seqnums, but they're
		// sufficient for maintaining correctness.
		SmallestSeqNum: m.SmallestSeqNum,
		LargestSeqNum:  m.LargestSeqNum,
	}
	if m.HasPointKeys && !exciseSpan.Contains(d.cmp, m.LargestPointKey) {
		// This file will contain point keys
		largestPointKey := m.LargestPointKey
		var err error
		if iter == nil && rangeDelIter == nil {
			iter, rangeDelIter, err = d.newIters(context.TODO(), m, &IterOptions{level: manifest.Level(level)}, internalIterOpts{})
			if err != nil {
				return nil, err
			}
			if iter != nil {
				defer iter.Close()
			} else {
				iter = emptyIter
			}
			if rangeDelIter != nil {
				defer rangeDelIter.Close()
			} else {
				rangeDelIter = emptyKeyspanIter
			}
		}
		key, _ := iter.SeekGE(exciseSpan.End, base.SeekGEFlagsNone)
		if key != nil {
			rightFile.ExtendPointKeyBounds(d.cmp, key.Clone(), largestPointKey)
		}
		// Store the max of (exciseSpan.End, rdel.Start) in firstRangeDel. This
		// needs to be a copy if the key is owned by the range del iter.
		var firstRangeDel []byte
		rdel := rangeDelIter.SeekGE(exciseSpan.End)
		if rdel != nil {
			firstRangeDel = append(firstRangeDel[:0], rdel.Start...)
			if d.cmp(firstRangeDel, exciseSpan.End) < 0 {
				firstRangeDel = exciseSpan.End
			}
		}
		if firstRangeDel != nil {
			smallestPointKey := rdel.SmallestKey()
			smallestPointKey.UserKey = firstRangeDel
			rightFile.ExtendPointKeyBounds(d.cmp, smallestPointKey, largestPointKey)
		}
	}
	if m.HasRangeKeys && !exciseSpan.Contains(d.cmp, m.LargestRangeKey) {
		// This file will contain range keys.
		largestRangeKey := m.LargestRangeKey
		if rangeKeyIter == nil {
			var err error
			rangeKeyIter, err = d.tableNewRangeKeyIter(m, keyspan.SpanIterOptions{})
			if err != nil {
				return nil, err
			}
			defer rangeKeyIter.Close()
		}
		// Store the max of (exciseSpan.End, rkey.Start) in firstRangeKey. This
		// needs to be a copy if the key is owned by the range key iter.
		var firstRangeKey []byte
		rkey := rangeKeyIter.SeekGE(exciseSpan.End)
		if rkey != nil {
			firstRangeKey = append(firstRangeKey[:0], rkey.Start...)
			if d.cmp(firstRangeKey, exciseSpan.End) < 0 {
				firstRangeKey = exciseSpan.End
			}
		}
		if firstRangeKey != nil {
			smallestRangeKey := rkey.SmallestKey()
			smallestRangeKey.UserKey = firstRangeKey
			// We call ExtendRangeKeyBounds so any internal boundType fields are
			// set correctly. Note that this is mildly wasteful as we'll be comparing
			// rightFile.{Smallest,Largest}RangeKey with themselves, which can be
			// avoided if we exported ExtendOverallKeyBounds or so.
			rightFile.ExtendRangeKeyBounds(d.cmp, smallestRangeKey, largestRangeKey)
		}
	}
	if rightFile.HasRangeKeys || rightFile.HasPointKeys {
		var err error
		rightFile.Size, err = d.tableCache.estimateSize(m, rightFile.Smallest.UserKey, rightFile.Largest.UserKey)
		if err != nil {
			return nil, err
		}
		if rightFile.Size == 0 {
			// On occasion, estimateSize gives us a low estimate, i.e. a 0 file size,
			// such as if the excised file only has range keys/dels and no point keys.
			// This can cause panics in places where we divide by file sizes. Correct
			// for it here.
			rightFile.Size = 1
		}
		rightFile.ValidateVirtual(m)
		d.checkVirtualBounds(rightFile)
		ve.NewFiles = append(ve.NewFiles, newFileEntry{Level: level, Meta: rightFile})
		needsBacking = true
		numCreatedFiles++
	}

	if needsBacking && !m.Virtual {
		// If m is virtual, then its file backing is already known to the manifest.
		// We don't need to create another file backing. Note that there must be
		// only one CreatedBackingTables entry per backing sstable. This is
		// indicated by the VersionEdit.CreatedBackingTables invariant.
		ve.CreatedBackingTables = append(ve.CreatedBackingTables, m.FileBacking)
	}

	if err := rightFile.Validate(d.cmp, d.opts.Comparer.FormatKey); err != nil {
		return nil, err
	}
	return ve.NewFiles[len(ve.NewFiles)-numCreatedFiles:], nil
}

type ingestTargetLevelFunc func(
	newIters tableNewIters,
	newRangeKeyIter keyspan.TableNewSpanIter,
	iterOps IterOptions,
	comparer *Comparer,
	v *version,
	baseLevel int,
	compactions map[*compaction]struct{},
	meta *fileMetadata,
	suggestSplit bool,
) (int, *fileMetadata, error)

type ingestSplitFile struct {
	// ingestFile is the file being ingested.
	ingestFile *fileMetadata
	// splitFile is the file that needs to be split to allow ingestFile to slot
	// into `level` level.
	splitFile *fileMetadata
	// The level where ingestFile will go (and where splitFile already is).
	level int
}

// ingestSplit splits files specified in `files` and updates ve in-place to
// account for existing files getting split into two virtual sstables. The map
// `replacedFiles` contains an in-progress map of all files that have been
// replaced with new virtual sstables in this version edit so far, which is also
// updated in-place.
//
// d.mu as well as the manifest lock must be held when calling this method.
func (d *DB) ingestSplit(
	ve *versionEdit,
	updateMetrics func(*fileMetadata, int, []newFileEntry),
	files []ingestSplitFile,
	replacedFiles map[base.FileNum][]newFileEntry,
) error {
	for _, s := range files {
		// replacedFiles can be thought of as a tree, where we start iterating with
		// s.splitFile and run its fileNum through replacedFiles, then find which of
		// the replaced files overlaps with s.ingestFile, which becomes the new
		// splitFile, then we check splitFile's replacements in replacedFiles again
		// for overlap with s.ingestFile, and so on until we either can't find the
		// current splitFile in replacedFiles (i.e. that's the file that now needs to
		// be split), or we don't find a file that overlaps with s.ingestFile, which
		// means a prior ingest split already produced enough room for s.ingestFile
		// to go into this level without necessitating another ingest split.
		splitFile := s.splitFile
		for splitFile != nil {
			replaced, ok := replacedFiles[splitFile.FileNum]
			if !ok {
				break
			}
			updatedSplitFile := false
			for i := range replaced {
				if replaced[i].Meta.Overlaps(d.cmp, s.ingestFile.Smallest.UserKey, s.ingestFile.Largest.UserKey, s.ingestFile.Largest.IsExclusiveSentinel()) {
					if updatedSplitFile {
						// This should never happen because the earlier ingestTargetLevel
						// function only finds split file candidates that are guaranteed to
						// have no data overlap, only boundary overlap. See the comments
						// in that method to see the definitions of data vs boundary
						// overlap. That, plus the fact that files in `replaced` are
						// guaranteed to have file bounds that are tight on user keys
						// (as that's what `d.excise` produces), means that the only case
						// where we overlap with two or more files in `replaced` is if we
						// actually had data overlap all along, or if the ingestion files
						// were overlapping, either of which is an invariant violation.
						panic("updated with two files in ingestSplit")
					}
					splitFile = replaced[i].Meta
					updatedSplitFile = true
				}
			}
			if !updatedSplitFile {
				// None of the replaced files overlapped with the file being ingested.
				// This can happen if we've already excised a span overlapping with
				// this file, or if we have consecutive ingested files that can slide
				// within the same gap between keys in an existing file. For instance,
				// if an existing file has keys a and g and we're ingesting b-c, d-e,
				// the first loop iteration will split the existing file into one that
				// ends in a and another that starts at g, and the second iteration will
				// fall into this case and require no splitting.
				//
				// No splitting necessary.
				splitFile = nil
			}
		}
		if splitFile == nil {
			continue
		}
		// NB: excise operates on [start, end). We're splitting at [start, end]
		// (assuming !s.ingestFile.Largest.IsExclusiveSentinel()). The conflation
		// of exclusive vs inclusive end bounds should not make a difference here
		// as we're guaranteed to not have any data overlap between splitFile and
		// s.ingestFile, so panic if we do see a newly added file with an endKey
		// equalling s.ingestFile.Largest, and !s.ingestFile.Largest.IsExclusiveSentinel()
		added, err := d.excise(KeyRange{Start: s.ingestFile.Smallest.UserKey, End: s.ingestFile.Largest.UserKey}, splitFile, ve, s.level)
		if err != nil {
			return err
		}
		if _, ok := ve.DeletedFiles[deletedFileEntry{
			Level:   s.level,
			FileNum: splitFile.FileNum,
		}]; !ok {
			panic("did not split file that was expected to be split")
		}
		replacedFiles[splitFile.FileNum] = added
		for i := range added {
			if s.ingestFile.Overlaps(d.cmp, added[i].Meta.Smallest.UserKey, added[i].Meta.Largest.UserKey, added[i].Meta.Largest.IsExclusiveSentinel()) {
				panic("ingest-time split produced a file that overlaps with ingested file")
			}
		}
		updateMetrics(splitFile, s.level, added)
	}
	// Flatten the version edit by removing any entries from ve.NewFiles that
	// are also in ve.DeletedFiles.
	newNewFiles := ve.NewFiles[:0]
	for i := range ve.NewFiles {
		fn := ve.NewFiles[i].Meta.FileNum
		deEntry := deletedFileEntry{Level: ve.NewFiles[i].Level, FileNum: fn}
		if _, ok := ve.DeletedFiles[deEntry]; ok {
			delete(ve.DeletedFiles, deEntry)
		} else {
			newNewFiles = append(newNewFiles, ve.NewFiles[i])
		}
	}
	ve.NewFiles = newNewFiles
	return nil
}

func (d *DB) ingestApply(
	jobID int,
	lr ingestLoadResult,
	findTargetLevel ingestTargetLevelFunc,
	mut *memTable,
	exciseSpan KeyRange,
) (*versionEdit, error) {
	d.mu.Lock()
	defer d.mu.Unlock()

	ve := &versionEdit{
		NewFiles: make([]newFileEntry, lr.fileCount),
	}
	if exciseSpan.Valid() || (d.opts.Experimental.IngestSplit != nil && d.opts.Experimental.IngestSplit()) {
		ve.DeletedFiles = map[manifest.DeletedFileEntry]*manifest.FileMetadata{}
	}
	metrics := make(map[int]*LevelMetrics)

	// Lock the manifest for writing before we use the current version to
	// determine the target level. This prevents two concurrent ingestion jobs
	// from using the same version to determine the target level, and also
	// provides serialization with concurrent compaction and flush jobs.
	// logAndApply unconditionally releases the manifest lock, but any earlier
	// returns must unlock the manifest.
	d.mu.versions.logLock()

	if mut != nil {
		// Unref the mutable memtable to allows its flush to proceed. Now that we've
		// acquired the manifest lock, we can be certain that if the mutable
		// memtable has received more recent conflicting writes, the flush won't
		// beat us to applying to the manifest resulting in sequence number
		// inversion. Even though we call maybeScheduleFlush right now, this flush
		// will apply after our ingestion.
		if mut.writerUnref() {
			d.maybeScheduleFlush()
		}
	}

	shouldIngestSplit := d.opts.Experimental.IngestSplit != nil &&
		d.opts.Experimental.IngestSplit() && d.FormatMajorVersion() >= FormatVirtualSSTables
	current := d.mu.versions.currentVersion()
	baseLevel := d.mu.versions.picker.getBaseLevel()
	iterOps := IterOptions{logger: d.opts.Logger}
	// filesToSplit is a list where each element is a pair consisting of a file
	// being ingested and a file being split to make room for an ingestion into
	// that level. Each ingested file will appear at most once in this list. It
	// is possible for split files to appear twice in this list.
	filesToSplit := make([]ingestSplitFile, 0)
	checkCompactions := false
	for i := 0; i < lr.fileCount; i++ {
		// Determine the lowest level in the LSM for which the sstable doesn't
		// overlap any existing files in the level.
		var m *fileMetadata
		sharedIdx := -1
		sharedLevel := -1
		externalFile := false
		if i < len(lr.localMeta) {
			// local file.
			m = lr.localMeta[i]
		} else if (i - len(lr.localMeta)) < len(lr.sharedMeta) {
			// shared file.
			sharedIdx = i - len(lr.localMeta)
			m = lr.sharedMeta[sharedIdx]
			sharedLevel = int(lr.sharedLevels[sharedIdx])
		} else {
			// external file.
			externalFile = true
			m = lr.externalMeta[i-(len(lr.localMeta)+len(lr.sharedMeta))]
		}
		f := &ve.NewFiles[i]
		var err error
		if sharedIdx >= 0 {
			f.Level = sharedLevel
			if f.Level < sharedLevelsStart {
				panic("cannot slot a shared file higher than the highest shared level")
			}
			ve.CreatedBackingTables = append(ve.CreatedBackingTables, m.FileBacking)
		} else {
			if externalFile {
				ve.CreatedBackingTables = append(ve.CreatedBackingTables, m.FileBacking)
			}
			var splitFile *fileMetadata
			if exciseSpan.Valid() && exciseSpan.Contains(d.cmp, m.Smallest) && exciseSpan.Contains(d.cmp, m.Largest) {
				// This file fits perfectly within the excise span. We can slot it at
				// L6, or sharedLevelsStart - 1 if we have shared files.
				if len(lr.sharedMeta) > 0 {
					f.Level = sharedLevelsStart - 1
					if baseLevel > f.Level {
						f.Level = 0
					}
				} else {
					f.Level = 6
				}
			} else {
				// TODO(bilal): findTargetLevel does disk IO (reading files for data
				// overlap) even though we're holding onto d.mu. Consider unlocking
				// d.mu while we do this. We already hold versions.logLock so we should
				// not see any version applications while we're at this. The one
				// complication here would be pulling out the mu.compact.inProgress
				// check from findTargetLevel, as that requires d.mu to be held.
				f.Level, splitFile, err = findTargetLevel(
					d.newIters, d.tableNewRangeKeyIter, iterOps, d.opts.Comparer, current, baseLevel, d.mu.compact.inProgress, m, shouldIngestSplit)
			}

			if splitFile != nil {
				if invariants.Enabled {
					if lf := current.Levels[f.Level].Find(d.cmp, splitFile); lf == nil {
						panic("splitFile returned is not in level it should be")
					}
				}
				// We take advantage of the fact that we won't drop the db mutex
				// between now and the call to logAndApply. So, no files should
				// get added to a new in-progress compaction at this point. We can
				// avoid having to iterate on in-progress compactions to cancel them
				// if none of the files being split have a compacting state.
				if splitFile.IsCompacting() {
					checkCompactions = true
				}
				filesToSplit = append(filesToSplit, ingestSplitFile{ingestFile: m, splitFile: splitFile, level: f.Level})
			}
		}
		if err != nil {
			d.mu.versions.logUnlock()
			return nil, err
		}
		f.Meta = m
		levelMetrics := metrics[f.Level]
		if levelMetrics == nil {
			levelMetrics = &LevelMetrics{}
			metrics[f.Level] = levelMetrics
		}
		levelMetrics.NumFiles++
		levelMetrics.Size += int64(m.Size)
		levelMetrics.BytesIngested += m.Size
		levelMetrics.TablesIngested++
	}
	// replacedFiles maps files excised due to exciseSpan (or splitFiles returned
	// by ingestTargetLevel), to files that were created to replace it. This map
	// is used to resolve references to split files in filesToSplit, as it is
	// possible for a file that we want to split to no longer exist or have a
	// newer fileMetadata due to a split induced by another ingestion file, or an
	// excise.
	replacedFiles := make(map[base.FileNum][]newFileEntry)
	updateLevelMetricsOnExcise := func(m *fileMetadata, level int, added []newFileEntry) {
		levelMetrics := metrics[level]
		if levelMetrics == nil {
			levelMetrics = &LevelMetrics{}
			metrics[level] = levelMetrics
		}
		levelMetrics.NumFiles--
		levelMetrics.Size -= int64(m.Size)
		for i := range added {
			levelMetrics.NumFiles++
			levelMetrics.Size += int64(added[i].Meta.Size)
		}
	}
	if exciseSpan.Valid() {
		// Iterate through all levels and find files that intersect with exciseSpan.
		//
		// TODO(bilal): We could drop the DB mutex here as we don't need it for
		// excises; we only need to hold the version lock which we already are
		// holding. However releasing the DB mutex could mess with the
		// ingestTargetLevel calculation that happened above, as it assumed that it
		// had a complete view of in-progress compactions that wouldn't change
		// until logAndApply is called. If we were to drop the mutex now, we could
		// schedule another in-progress compaction that would go into the chosen target
		// level and lead to file overlap within level (which would panic in
		// logAndApply). We should drop the db mutex here, do the excise, then
		// re-grab the DB mutex and rerun just the in-progress compaction check to
		// see if any new compactions are conflicting with our chosen target levels
		// for files, and if they are, we should signal those compactions to error
		// out.
		for level := range current.Levels {
			overlaps := current.Overlaps(level, d.cmp, exciseSpan.Start, exciseSpan.End, true /* exclusiveEnd */)
			iter := overlaps.Iter()

			for m := iter.First(); m != nil; m = iter.Next() {
				newFiles, err := d.excise(exciseSpan, m, ve, level)
				if err != nil {
					return nil, err
				}

				if _, ok := ve.DeletedFiles[deletedFileEntry{
					Level:   level,
					FileNum: m.FileNum,
				}]; !ok {
					// We did not excise this file.
					continue
				}
				replacedFiles[m.FileNum] = newFiles
				updateLevelMetricsOnExcise(m, level, newFiles)
			}
		}
	}
	if len(filesToSplit) > 0 {
		// For the same reasons as the above call to excise, we hold the db mutex
		// while calling this method.
		if err := d.ingestSplit(ve, updateLevelMetricsOnExcise, filesToSplit, replacedFiles); err != nil {
			return nil, err
		}
	}
	if len(filesToSplit) > 0 || exciseSpan.Valid() {
		for c := range d.mu.compact.inProgress {
			if c.versionEditApplied {
				continue
			}
			// Check if this compaction overlaps with the excise span. Note that just
			// checking if the inputs individually overlap with the excise span
			// isn't sufficient; for instance, a compaction could have [a,b] and [e,f]
			// as inputs and write it all out as [a,b,e,f] in one sstable. If we're
			// doing a [c,d) excise at the same time as this compaction, we will have
			// to error out the whole compaction as we can't guarantee it hasn't/won't
			// write a file overlapping with the excise span.
			if exciseSpan.OverlapsInternalKeyRange(d.cmp, c.smallest, c.largest) {
				c.cancel.Store(true)
			}
			// Check if this compaction's inputs have been replaced due to an
			// ingest-time split. In that case, cancel the compaction as a newly picked
			// compaction would need to include any new files that slid in between
			// previously-existing files. Note that we cancel any compaction that has a
			// file that was ingest-split as an input, even if it started before this
			// ingestion.
			if checkCompactions {
				for i := range c.inputs {
					iter := c.inputs[i].files.Iter()
					for f := iter.First(); f != nil; f = iter.Next() {
						if _, ok := replacedFiles[f.FileNum]; ok {
							c.cancel.Store(true)
							break
						}
					}
				}
			}
		}
		// Check for any EventuallyFileOnlySnapshots that could be watching for
		// an excise on this span.
		if exciseSpan.Valid() {
			for s := d.mu.snapshots.root.next; s != &d.mu.snapshots.root; s = s.next {
				if s.efos == nil {
					continue
				}
				efos := s.efos
				// TODO(bilal): We can make this faster by taking advantage of the sorted
				// nature of protectedRanges to do a sort.Search, or even maintaining a
				// global list of all protected ranges instead of having to peer into every
				// snapshot.
				for i := range efos.protectedRanges {
					if efos.protectedRanges[i].OverlapsKeyRange(d.cmp, exciseSpan) {
						efos.excised.Store(true)
						break
					}
				}
			}
		}
	}
	if err := d.mu.versions.logAndApply(jobID, ve, metrics, false /* forceRotation */, func() []compactionInfo {
		return d.getInProgressCompactionInfoLocked(nil)
	}); err != nil {
		return nil, err
	}

	d.mu.versions.metrics.Ingest.Count++

	d.updateReadStateLocked(d.opts.DebugCheck)
	// updateReadStateLocked could have generated obsolete tables, schedule a
	// cleanup job if necessary.
	d.deleteObsoleteFiles(jobID)
	d.updateTableStatsLocked(ve.NewFiles)
	// The ingestion may have pushed a level over the threshold for compaction,
	// so check to see if one is necessary and schedule it.
	d.maybeScheduleCompaction()
	var toValidate []manifest.NewFileEntry
	dedup := make(map[base.DiskFileNum]struct{})
	for _, entry := range ve.NewFiles {
		if _, ok := dedup[entry.Meta.FileBacking.DiskFileNum]; !ok {
			toValidate = append(toValidate, entry)
			dedup[entry.Meta.FileBacking.DiskFileNum] = struct{}{}
		}
	}
	d.maybeValidateSSTablesLocked(toValidate)
	return ve, nil
}

// maybeValidateSSTablesLocked adds the slice of newFileEntrys to the pending
// queue of files to be validated, when the feature is enabled.
//
// Note that if two entries with the same backing file are added twice, then the
// block checksums for the backing file will be validated twice.
//
// DB.mu must be locked when calling.
func (d *DB) maybeValidateSSTablesLocked(newFiles []newFileEntry) {
	// Only add to the validation queue when the feature is enabled.
	if !d.opts.Experimental.ValidateOnIngest {
		return
	}

	d.mu.tableValidation.pending = append(d.mu.tableValidation.pending, newFiles...)
	if d.shouldValidateSSTablesLocked() {
		go d.validateSSTables()
	}
}

// shouldValidateSSTablesLocked returns true if SSTable validation should run.
// DB.mu must be locked when calling.
func (d *DB) shouldValidateSSTablesLocked() bool {
	return !d.mu.tableValidation.validating &&
		d.closed.Load() == nil &&
		d.opts.Experimental.ValidateOnIngest &&
		len(d.mu.tableValidation.pending) > 0
}

// validateSSTables runs a round of validation on the tables in the pending
// queue.
func (d *DB) validateSSTables() {
	d.mu.Lock()
	if !d.shouldValidateSSTablesLocked() {
		d.mu.Unlock()
		return
	}

	pending := d.mu.tableValidation.pending
	d.mu.tableValidation.pending = nil
	d.mu.tableValidation.validating = true
	jobID := d.mu.nextJobID
	d.mu.nextJobID++
	rs := d.loadReadState()

	// Drop DB.mu before performing IO.
	d.mu.Unlock()

	// Validate all tables in the pending queue. This could lead to a situation
	// where we are starving IO from other tasks due to having to page through
	// all the blocks in all the sstables in the queue.
	// TODO(travers): Add some form of pacing to avoid IO starvation.
	for _, f := range pending {
		// The file may have been moved or deleted since it was ingested, in
		// which case we skip.
		if !rs.current.Contains(f.Level, d.cmp, f.Meta) {
			// Assume the file was moved to a lower level. It is rare enough
			// that a table is moved or deleted between the time it was ingested
			// and the time the validation routine runs that the overall cost of
			// this inner loop is tolerably low, when amortized over all
			// ingested tables.
			found := false
			for i := f.Level + 1; i < numLevels; i++ {
				if rs.current.Contains(i, d.cmp, f.Meta) {
					found = true
					break
				}
			}
			if !found {
				continue
			}
		}

		var err error
		if f.Meta.Virtual {
			err = d.tableCache.withVirtualReader(
				f.Meta.VirtualMeta(), func(v sstable.VirtualReader) error {
					return v.ValidateBlockChecksumsOnBacking()
				})
		} else {
			err = d.tableCache.withReader(
				f.Meta.PhysicalMeta(), func(r *sstable.Reader) error {
					return r.ValidateBlockChecksums()
				})
		}

		if err != nil {
			// TODO(travers): Hook into the corruption reporting pipeline, once
			// available. See pebble#1192.
			d.opts.Logger.Fatalf("pebble: encountered corruption during ingestion: %s", err)
		}

		d.opts.EventListener.TableValidated(TableValidatedInfo{
			JobID: jobID,
			Meta:  f.Meta,
		})
	}
	rs.unref()

	d.mu.Lock()
	defer d.mu.Unlock()
	d.mu.tableValidation.validating = false
	d.mu.tableValidation.cond.Broadcast()
	if d.shouldValidateSSTablesLocked() {
		go d.validateSSTables()
	}
}
