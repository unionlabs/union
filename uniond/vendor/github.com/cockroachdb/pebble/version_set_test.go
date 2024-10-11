// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"io"
	"testing"
	"time"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func writeAndIngest(t *testing.T, mem vfs.FS, d *DB, k InternalKey, v []byte, filename string) {
	path := mem.PathJoin("ext", filename)
	f, err := mem.Create(path)
	require.NoError(t, err)
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	require.NoError(t, w.Add(k, v))
	require.NoError(t, w.Close())
	require.NoError(t, d.Ingest([]string{path}))
}

// d.mu should be help. logLock should not be held.
func checkBackingSize(t *testing.T, d *DB) {
	d.mu.versions.logLock()
	var backingSizeSum uint64
	for _, backing := range d.mu.versions.backingState.fileBackingMap {
		backingSizeSum += backing.Size
	}
	require.Equal(t, backingSizeSum, d.mu.versions.backingState.fileBackingSize)
	d.mu.versions.logUnlock()
}

// TestLatestRefCounting sanity checks the ref counting implementation for
// FileMetadata.latestRefs, and makes sure that the zombie table implementation
// works when the version edit contains virtual sstables. It also checks that
// we're adding the physical sstable to the obsolete tables list iff the file is
// truly obsolete.
func TestLatestRefCounting(t *testing.T) {
	mem := vfs.NewMem()
	require.NoError(t, mem.MkdirAll("ext", 0755))

	opts := &Options{
		FS:                          mem,
		MaxManifestFileSize:         1,
		DisableAutomaticCompactions: true,
		FormatMajorVersion:          FormatVirtualSSTables,
	}
	d, err := Open("", opts)
	require.NoError(t, err)

	err = d.Set([]byte{'a'}, []byte{'a'}, nil)
	require.NoError(t, err)
	err = d.Set([]byte{'b'}, []byte{'b'}, nil)
	require.NoError(t, err)

	err = d.Flush()
	require.NoError(t, err)

	iter := d.mu.versions.currentVersion().Levels[0].Iter()
	var f *fileMetadata = iter.First()
	require.NotNil(t, f)
	require.Equal(t, 1, int(f.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))

	// Grab some new file nums.
	d.mu.Lock()
	f1 := d.mu.versions.nextFileNum
	f2 := f1 + 1
	d.mu.versions.nextFileNum += 2
	d.mu.Unlock()

	m1 := &manifest.FileMetadata{
		FileBacking:    f.FileBacking,
		FileNum:        f1,
		CreationTime:   time.Now().Unix(),
		Size:           f.Size / 2,
		SmallestSeqNum: f.SmallestSeqNum,
		LargestSeqNum:  f.LargestSeqNum,
		Smallest:       base.MakeInternalKey([]byte{'a'}, f.Smallest.SeqNum(), InternalKeyKindSet),
		Largest:        base.MakeInternalKey([]byte{'a'}, f.Smallest.SeqNum(), InternalKeyKindSet),
		HasPointKeys:   true,
		Virtual:        true,
	}

	m2 := &manifest.FileMetadata{
		FileBacking:    f.FileBacking,
		FileNum:        f2,
		CreationTime:   time.Now().Unix(),
		Size:           f.Size - m1.Size,
		SmallestSeqNum: f.SmallestSeqNum,
		LargestSeqNum:  f.LargestSeqNum,
		Smallest:       base.MakeInternalKey([]byte{'b'}, f.Largest.SeqNum(), InternalKeyKindSet),
		Largest:        base.MakeInternalKey([]byte{'b'}, f.Largest.SeqNum(), InternalKeyKindSet),
		HasPointKeys:   true,
		Virtual:        true,
	}

	m1.LargestPointKey = m1.Largest
	m1.SmallestPointKey = m1.Smallest

	m2.LargestPointKey = m2.Largest
	m2.SmallestPointKey = m2.Smallest

	m1.ValidateVirtual(f)
	d.checkVirtualBounds(m1)
	m2.ValidateVirtual(f)
	d.checkVirtualBounds(m2)

	fileMetrics := func(ve *versionEdit) map[int]*LevelMetrics {
		metrics := newFileMetrics(ve.NewFiles)
		for de, f := range ve.DeletedFiles {
			lm := metrics[de.Level]
			if lm == nil {
				lm = &LevelMetrics{}
				metrics[de.Level] = lm
			}
			metrics[de.Level].NumFiles--
			metrics[de.Level].Size -= int64(f.Size)
		}
		return metrics
	}

	d.mu.Lock()
	defer d.mu.Unlock()
	applyVE := func(ve *versionEdit) error {
		d.mu.versions.logLock()
		jobID := d.mu.nextJobID
		d.mu.nextJobID++

		err := d.mu.versions.logAndApply(jobID, ve, fileMetrics(ve), false, func() []compactionInfo {
			return d.getInProgressCompactionInfoLocked(nil)
		})
		d.updateReadStateLocked(nil)
		return err
	}

	// Virtualize f.
	ve := manifest.VersionEdit{}
	d1 := manifest.DeletedFileEntry{Level: 0, FileNum: f.FileNum}
	n1 := manifest.NewFileEntry{Level: 0, Meta: m1}
	n2 := manifest.NewFileEntry{Level: 0, Meta: m2}

	ve.DeletedFiles = make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)
	ve.DeletedFiles[d1] = f
	ve.NewFiles = append(ve.NewFiles, n1)
	ve.NewFiles = append(ve.NewFiles, n2)
	ve.CreatedBackingTables = append(ve.CreatedBackingTables, f.FileBacking)

	require.NoError(t, applyVE(&ve))
	// 2 latestRefs from 2 virtual sstables in the latest version which refer
	// to the physical sstable.
	require.Equal(t, 2, int(m1.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 1, len(d.mu.versions.backingState.fileBackingMap))
	_, ok := d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.True(t, ok)
	require.Equal(t, f.Size, m2.FileBacking.VirtualizedSize.Load())
	checkBackingSize(t, d)

	// Make sure that f is not present in zombie list, because it is not yet a
	// zombie.
	require.Equal(t, 0, len(d.mu.versions.zombieTables))

	// Delete the virtual sstable m1.
	ve = manifest.VersionEdit{}
	d1 = manifest.DeletedFileEntry{Level: 0, FileNum: m1.FileNum}
	ve.DeletedFiles = make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)
	ve.DeletedFiles[d1] = m1
	require.NoError(t, applyVE(&ve))

	// Only one virtual sstable in the latest version, confirm that the latest
	// version ref counting is correct.
	require.Equal(t, 1, int(m2.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.zombieTables))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 1, len(d.mu.versions.backingState.fileBackingMap))
	_, ok = d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.True(t, ok)
	require.Equal(t, m2.Size, m2.FileBacking.VirtualizedSize.Load())
	checkBackingSize(t, d)

	// Move m2 from L0 to L6 to test the move compaction case.
	ve = manifest.VersionEdit{}
	d1 = manifest.DeletedFileEntry{Level: 0, FileNum: m2.FileNum}
	n1 = manifest.NewFileEntry{Level: 6, Meta: m2}
	ve.DeletedFiles = make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)
	ve.DeletedFiles[d1] = m2
	ve.NewFiles = append(ve.NewFiles, n1)
	require.NoError(t, applyVE(&ve))
	checkBackingSize(t, d)

	require.Equal(t, 1, int(m2.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.zombieTables))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 1, len(d.mu.versions.backingState.fileBackingMap))
	_, ok = d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.True(t, ok)
	require.Equal(t, m2.Size, m2.FileBacking.VirtualizedSize.Load())

	// Delete m2 from L6.
	ve = manifest.VersionEdit{}
	d1 = manifest.DeletedFileEntry{Level: 6, FileNum: m2.FileNum}
	ve.DeletedFiles = make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)
	ve.DeletedFiles[d1] = m2
	require.NoError(t, applyVE(&ve))
	checkBackingSize(t, d)

	// All virtual sstables are gone.
	require.Equal(t, 0, int(m2.LatestRefs()))
	require.Equal(t, 1, len(d.mu.versions.zombieTables))
	require.Equal(t, f.Size, d.mu.versions.zombieTables[f.FileBacking.DiskFileNum])
	require.Equal(t, 0, len(d.mu.versions.backingState.fileBackingMap))
	_, ok = d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.False(t, ok)
	require.Equal(t, 0, int(m2.FileBacking.VirtualizedSize.Load()))
	checkBackingSize(t, d)

	// Make sure that the backing file is added to the obsolete tables list.
	require.Equal(t, 1, len(d.mu.versions.obsoleteTables))

}

// TODO(bananabrick): Convert TestLatestRefCounting and this test into a single
// datadriven test.
func TestVirtualSSTableManifestReplay(t *testing.T) {
	mem := vfs.NewMem()
	require.NoError(t, mem.MkdirAll("ext", 0755))

	opts := &Options{
		FormatMajorVersion:          FormatVirtualSSTables,
		FS:                          mem,
		MaxManifestFileSize:         1,
		DisableAutomaticCompactions: true,
	}
	d, err := Open("", opts)
	require.NoError(t, err)

	err = d.Set([]byte{'a'}, []byte{'a'}, nil)
	require.NoError(t, err)
	err = d.Set([]byte{'b'}, []byte{'b'}, nil)
	require.NoError(t, err)

	err = d.Flush()
	require.NoError(t, err)

	iter := d.mu.versions.currentVersion().Levels[0].Iter()
	var f *fileMetadata = iter.First()
	require.NotNil(t, f)
	require.Equal(t, 1, int(f.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))

	// Grab some new file nums.
	d.mu.Lock()
	f1 := d.mu.versions.nextFileNum
	f2 := f1 + 1
	d.mu.versions.nextFileNum += 2
	d.mu.Unlock()

	m1 := &manifest.FileMetadata{
		FileBacking:    f.FileBacking,
		FileNum:        f1,
		CreationTime:   time.Now().Unix(),
		Size:           f.Size / 2,
		SmallestSeqNum: f.SmallestSeqNum,
		LargestSeqNum:  f.LargestSeqNum,
		Smallest:       base.MakeInternalKey([]byte{'a'}, f.Smallest.SeqNum(), InternalKeyKindSet),
		Largest:        base.MakeInternalKey([]byte{'a'}, f.Smallest.SeqNum(), InternalKeyKindSet),
		HasPointKeys:   true,
		Virtual:        true,
	}

	m2 := &manifest.FileMetadata{
		FileBacking:    f.FileBacking,
		FileNum:        f2,
		CreationTime:   time.Now().Unix(),
		Size:           f.Size - m1.Size,
		SmallestSeqNum: f.SmallestSeqNum,
		LargestSeqNum:  f.LargestSeqNum,
		Smallest:       base.MakeInternalKey([]byte{'b'}, f.Largest.SeqNum(), InternalKeyKindSet),
		Largest:        base.MakeInternalKey([]byte{'b'}, f.Largest.SeqNum(), InternalKeyKindSet),
		HasPointKeys:   true,
		Virtual:        true,
	}

	m1.LargestPointKey = m1.Largest
	m1.SmallestPointKey = m1.Smallest
	m1.Stats.NumEntries = 1

	m2.LargestPointKey = m2.Largest
	m2.SmallestPointKey = m2.Smallest
	m2.Stats.NumEntries = 1

	m1.ValidateVirtual(f)
	d.checkVirtualBounds(m1)
	m2.ValidateVirtual(f)
	d.checkVirtualBounds(m2)

	fileMetrics := func(ve *versionEdit) map[int]*LevelMetrics {
		metrics := newFileMetrics(ve.NewFiles)
		for de, f := range ve.DeletedFiles {
			lm := metrics[de.Level]
			if lm == nil {
				lm = &LevelMetrics{}
				metrics[de.Level] = lm
			}
			metrics[de.Level].NumFiles--
			metrics[de.Level].Size -= int64(f.Size)
		}
		return metrics
	}

	d.mu.Lock()
	applyVE := func(ve *versionEdit) error {
		d.mu.versions.logLock()
		jobID := d.mu.nextJobID
		d.mu.nextJobID++

		err := d.mu.versions.logAndApply(jobID, ve, fileMetrics(ve), false, func() []compactionInfo {
			return d.getInProgressCompactionInfoLocked(nil)
		})
		d.updateReadStateLocked(nil)
		return err
	}

	// Virtualize f.
	ve := manifest.VersionEdit{}
	d1 := manifest.DeletedFileEntry{Level: 0, FileNum: f.FileNum}
	n1 := manifest.NewFileEntry{Level: 0, Meta: m1}
	n2 := manifest.NewFileEntry{Level: 0, Meta: m2}

	ve.DeletedFiles = make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)
	ve.DeletedFiles[d1] = f
	ve.NewFiles = append(ve.NewFiles, n1)
	ve.NewFiles = append(ve.NewFiles, n2)
	ve.CreatedBackingTables = append(ve.CreatedBackingTables, f.FileBacking)

	require.NoError(t, applyVE(&ve))
	checkBackingSize(t, d)
	d.mu.Unlock()

	require.Equal(t, 2, int(m1.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 1, len(d.mu.versions.backingState.fileBackingMap))
	_, ok := d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.True(t, ok)
	require.Equal(t, f.Size, m2.FileBacking.VirtualizedSize.Load())

	// Snapshot version edit will be written to a new manifest due to the flush.
	d.Set([]byte{'c'}, []byte{'c'}, nil)
	d.Flush()

	require.NoError(t, d.Close())
	d, err = Open("", opts)
	require.NoError(t, err)

	d.mu.Lock()
	it := d.mu.versions.currentVersion().Levels[0].Iter()
	var virtualFile *fileMetadata
	for f := it.First(); f != nil; f = it.Next() {
		if f.Virtual {
			virtualFile = f
			break
		}
	}

	require.Equal(t, 2, int(virtualFile.LatestRefs()))
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 1, len(d.mu.versions.backingState.fileBackingMap))
	_, ok = d.mu.versions.backingState.fileBackingMap[f.FileBacking.DiskFileNum]
	require.True(t, ok)
	require.Equal(t, f.Size, virtualFile.FileBacking.VirtualizedSize.Load())
	checkBackingSize(t, d)
	d.mu.Unlock()

	// Will cause the virtual sstables to be deleted, and the file backing should
	// also be removed.
	d.Compact([]byte{'a'}, []byte{'z'}, false)

	d.mu.Lock()
	virtualFile = nil
	it = d.mu.versions.currentVersion().Levels[0].Iter()
	for f := it.First(); f != nil; f = it.Next() {
		if f.Virtual {
			virtualFile = f
			break
		}
	}
	require.Nil(t, virtualFile)
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 0, len(d.mu.versions.backingState.fileBackingMap))
	checkBackingSize(t, d)
	d.mu.Unlock()

	// Close and restart to make sure that the new snapshot written during
	// compaction doesn't have the file backing.
	require.NoError(t, d.Close())
	d, err = Open("", opts)
	require.NoError(t, err)

	d.mu.Lock()
	virtualFile = nil
	it = d.mu.versions.currentVersion().Levels[0].Iter()
	for f := it.First(); f != nil; f = it.Next() {
		if f.Virtual {
			virtualFile = f
			break
		}
	}
	require.Nil(t, virtualFile)
	require.Equal(t, 0, len(d.mu.versions.obsoleteTables))
	require.Equal(t, 0, len(d.mu.versions.backingState.fileBackingMap))
	checkBackingSize(t, d)
	d.mu.Unlock()
	require.NoError(t, d.Close())
}

func TestVersionSetCheckpoint(t *testing.T) {
	mem := vfs.NewMem()
	require.NoError(t, mem.MkdirAll("ext", 0755))

	opts := &Options{
		FS:                  mem,
		MaxManifestFileSize: 1,
	}
	d, err := Open("", opts)
	require.NoError(t, err)

	// Multiple manifest files are created such that the latest one must have a correct snapshot
	// of the preceding state for the DB to be opened correctly and see the written data.
	// Snapshot has no files, so first edit will cause manifest rotation.
	writeAndIngest(t, mem, d, base.MakeInternalKey([]byte("a"), 0, InternalKeyKindSet), []byte("b"), "a")
	// Snapshot has no files, and manifest has an edit from the previous ingest,
	// so this second ingest will cause manifest rotation.
	writeAndIngest(t, mem, d, base.MakeInternalKey([]byte("c"), 0, InternalKeyKindSet), []byte("d"), "c")
	require.NoError(t, d.Close())
	d, err = Open("", opts)
	require.NoError(t, err)
	checkValue := func(k string, expected string) {
		v, closer, err := d.Get([]byte(k))
		require.NoError(t, err)
		require.Equal(t, expected, string(v))
		closer.Close()
	}
	checkValue("a", "b")
	checkValue("c", "d")
	require.NoError(t, d.Close())
}

func TestVersionSetSeqNums(t *testing.T) {
	mem := vfs.NewMem()
	require.NoError(t, mem.MkdirAll("ext", 0755))

	opts := &Options{
		FS:                  mem,
		MaxManifestFileSize: 1,
	}
	d, err := Open("", opts)
	require.NoError(t, err)

	// Snapshot has no files, so first edit will cause manifest rotation.
	writeAndIngest(t, mem, d, base.MakeInternalKey([]byte("a"), 0, InternalKeyKindSet), []byte("b"), "a")
	// Snapshot has no files, and manifest has an edit from the previous ingest,
	// so this second ingest will cause manifest rotation.
	writeAndIngest(t, mem, d, base.MakeInternalKey([]byte("c"), 0, InternalKeyKindSet), []byte("d"), "c")
	require.NoError(t, d.Close())
	d, err = Open("", opts)
	require.NoError(t, err)
	defer d.Close()
	d.TestOnlyWaitForCleaning()

	// Check that the manifest has the correct LastSeqNum, equalling the highest
	// observed SeqNum.
	filenames, err := mem.List("")
	require.NoError(t, err)
	var manifest vfs.File
	for _, filename := range filenames {
		fileType, _, ok := base.ParseFilename(mem, filename)
		if ok && fileType == fileTypeManifest {
			manifest, err = mem.Open(filename)
			require.NoError(t, err)
		}
	}
	require.NotNil(t, manifest)
	defer manifest.Close()
	rr := record.NewReader(manifest, 0 /* logNum */)
	lastSeqNum := uint64(0)
	for {
		r, err := rr.Next()
		if err == io.EOF {
			break
		}
		require.NoError(t, err)
		var ve versionEdit
		err = ve.Decode(r)
		require.NoError(t, err)
		if ve.LastSeqNum != 0 {
			lastSeqNum = ve.LastSeqNum
		}
	}
	// 2 ingestions happened, so LastSeqNum should equal base.SeqNumStart + 1.
	require.Equal(t, uint64(11), lastSeqNum)
	// logSeqNum is always one greater than the last assigned sequence number.
	require.Equal(t, d.mu.versions.logSeqNum.Load(), lastSeqNum+1)
}
