// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"sort"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
)

// TODO(peter): describe the MANIFEST file format, independently of the C++
// project.

var errCorruptManifest = base.CorruptionErrorf("pebble: corrupt manifest")

type byteReader interface {
	io.ByteReader
	io.Reader
}

// Tags for the versionEdit disk format.
// Tag 8 is no longer used.
const (
	// LevelDB tags.
	tagComparator     = 1
	tagLogNumber      = 2
	tagNextFileNumber = 3
	tagLastSequence   = 4
	tagCompactPointer = 5
	tagDeletedFile    = 6
	tagNewFile        = 7
	tagPrevLogNumber  = 9

	// RocksDB tags.
	tagNewFile2         = 100
	tagNewFile3         = 102
	tagNewFile4         = 103
	tagColumnFamily     = 200
	tagColumnFamilyAdd  = 201
	tagColumnFamilyDrop = 202
	tagMaxColumnFamily  = 203

	// Pebble tags.
	tagNewFile5            = 104 // Range keys.
	tagCreatedBackingTable = 105
	tagRemovedBackingTable = 106

	// The custom tags sub-format used by tagNewFile4 and above.
	customTagTerminate         = 1
	customTagNeedsCompaction   = 2
	customTagCreationTime      = 6
	customTagPathID            = 65
	customTagNonSafeIgnoreMask = 1 << 6
	customTagVirtual           = 66
)

// DeletedFileEntry holds the state for a file deletion from a level. The file
// itself might still be referenced by another level.
type DeletedFileEntry struct {
	Level   int
	FileNum base.FileNum
}

// NewFileEntry holds the state for a new file or one moved from a different
// level.
type NewFileEntry struct {
	Level int
	Meta  *FileMetadata
	// BackingFileNum is only set during manifest replay, and only for virtual
	// sstables.
	BackingFileNum base.DiskFileNum
}

// VersionEdit holds the state for an edit to a Version along with other
// on-disk state (log numbers, next file number, and the last sequence number).
type VersionEdit struct {
	// ComparerName is the value of Options.Comparer.Name. This is only set in
	// the first VersionEdit in a manifest (either when the DB is created, or
	// when a new manifest is created) and is used to verify that the comparer
	// specified at Open matches the comparer that was previously used.
	ComparerName string

	// MinUnflushedLogNum is the smallest WAL log file number corresponding to
	// mutations that have not been flushed to an sstable.
	//
	// This is an optional field, and 0 represents it is not set.
	MinUnflushedLogNum base.FileNum

	// ObsoletePrevLogNum is a historic artifact from LevelDB that is not used by
	// Pebble, RocksDB, or even LevelDB. Its use in LevelDB was deprecated in
	// 6/2011. We keep it around purely for informational purposes when
	// displaying MANIFEST contents.
	ObsoletePrevLogNum uint64

	// The next file number. A single counter is used to assign file numbers
	// for the WAL, MANIFEST, sstable, and OPTIONS files.
	NextFileNum base.FileNum

	// LastSeqNum is an upper bound on the sequence numbers that have been
	// assigned in flushed WALs. Unflushed WALs (that will be replayed during
	// recovery) may contain sequence numbers greater than this value.
	LastSeqNum uint64

	// A file num may be present in both deleted files and new files when it
	// is moved from a lower level to a higher level (when the compaction
	// found that there was no overlapping file at the higher level).
	DeletedFiles map[DeletedFileEntry]*FileMetadata
	NewFiles     []NewFileEntry
	// CreatedBackingTables can be used to preserve the FileBacking associated
	// with a physical sstable. This is useful when virtual sstables in the
	// latest version are reconstructed during manifest replay, and we also need
	// to reconstruct the FileBacking which is required by these virtual
	// sstables.
	//
	// INVARIANT: The FileBacking associated with a physical sstable must only
	// be added as a backing file in the same version edit where the physical
	// sstable is first virtualized. This means that the physical sstable must
	// be present in DeletedFiles and that there must be at least one virtual
	// sstable with the same FileBacking as the physical sstable in NewFiles. A
	// file must be present in CreatedBackingTables in exactly one version edit.
	// The physical sstable associated with the FileBacking must also not be
	// present in NewFiles.
	CreatedBackingTables []*FileBacking
	// RemovedBackingTables is used to remove the FileBacking associated with a
	// virtual sstable. Note that a backing sstable can be removed as soon as
	// there are no virtual sstables in the latest version which are using the
	// backing sstable, but the backing sstable doesn't necessarily have to be
	// removed atomically with the version edit which removes the last virtual
	// sstable associated with the backing sstable. The removal can happen in a
	// future version edit.
	//
	// INVARIANT: A file must only be added to RemovedBackingTables if it was
	// added to CreateBackingTables in a prior version edit. The same version
	// edit also cannot have the same file present in both CreateBackingTables
	// and RemovedBackingTables. A file must be present in RemovedBackingTables
	// in exactly one version edit.
	RemovedBackingTables []base.DiskFileNum
}

// Decode decodes an edit from the specified reader.
//
// Note that the Decode step will not set the FileBacking for virtual sstables
// and the responsibility is left to the caller. However, the Decode step will
// populate the NewFileEntry.BackingFileNum in VersionEdit.NewFiles.
func (v *VersionEdit) Decode(r io.Reader) error {
	br, ok := r.(byteReader)
	if !ok {
		br = bufio.NewReader(r)
	}
	d := versionEditDecoder{br}
	for {
		tag, err := binary.ReadUvarint(br)
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		switch tag {
		case tagComparator:
			s, err := d.readBytes()
			if err != nil {
				return err
			}
			v.ComparerName = string(s)

		case tagLogNumber:
			n, err := d.readFileNum()
			if err != nil {
				return err
			}
			v.MinUnflushedLogNum = n

		case tagNextFileNumber:
			n, err := d.readFileNum()
			if err != nil {
				return err
			}
			v.NextFileNum = n

		case tagLastSequence:
			n, err := d.readUvarint()
			if err != nil {
				return err
			}
			v.LastSeqNum = n

		case tagCompactPointer:
			if _, err := d.readLevel(); err != nil {
				return err
			}
			if _, err := d.readBytes(); err != nil {
				return err
			}
			// NB: RocksDB does not use compaction pointers anymore.

		case tagRemovedBackingTable:
			n, err := d.readUvarint()
			if err != nil {
				return err
			}
			v.RemovedBackingTables = append(
				v.RemovedBackingTables, base.FileNum(n).DiskFileNum(),
			)
		case tagCreatedBackingTable:
			dfn, err := d.readUvarint()
			if err != nil {
				return err
			}
			size, err := d.readUvarint()
			if err != nil {
				return err
			}
			fileBacking := &FileBacking{
				DiskFileNum: base.FileNum(dfn).DiskFileNum(),
				Size:        size,
			}
			v.CreatedBackingTables = append(v.CreatedBackingTables, fileBacking)
		case tagDeletedFile:
			level, err := d.readLevel()
			if err != nil {
				return err
			}
			fileNum, err := d.readFileNum()
			if err != nil {
				return err
			}
			if v.DeletedFiles == nil {
				v.DeletedFiles = make(map[DeletedFileEntry]*FileMetadata)
			}
			v.DeletedFiles[DeletedFileEntry{level, fileNum}] = nil

		case tagNewFile, tagNewFile2, tagNewFile3, tagNewFile4, tagNewFile5:
			level, err := d.readLevel()
			if err != nil {
				return err
			}
			fileNum, err := d.readFileNum()
			if err != nil {
				return err
			}
			if tag == tagNewFile3 {
				// The pathID field appears unused in RocksDB.
				_ /* pathID */, err := d.readUvarint()
				if err != nil {
					return err
				}
			}
			size, err := d.readUvarint()
			if err != nil {
				return err
			}
			// We read the smallest / largest key bounds differently depending on
			// whether we have point, range or both types of keys present in the
			// table.
			var (
				smallestPointKey, largestPointKey []byte
				smallestRangeKey, largestRangeKey []byte
				parsedPointBounds                 bool
				boundsMarker                      byte
			)
			if tag != tagNewFile5 {
				// Range keys not present in the table. Parse the point key bounds.
				smallestPointKey, err = d.readBytes()
				if err != nil {
					return err
				}
				largestPointKey, err = d.readBytes()
				if err != nil {
					return err
				}
			} else {
				// Range keys are present in the table. Determine whether we have point
				// keys to parse, in addition to the bounds.
				boundsMarker, err = d.ReadByte()
				if err != nil {
					return err
				}
				// Parse point key bounds, if present.
				if boundsMarker&maskContainsPointKeys > 0 {
					smallestPointKey, err = d.readBytes()
					if err != nil {
						return err
					}
					largestPointKey, err = d.readBytes()
					if err != nil {
						return err
					}
					parsedPointBounds = true
				} else {
					// The table does not have point keys.
					// Sanity check: the bounds must be range keys.
					if boundsMarker&maskSmallest != 0 || boundsMarker&maskLargest != 0 {
						return base.CorruptionErrorf(
							"new-file-4-range-keys: table without point keys has point key bounds: marker=%x",
							boundsMarker,
						)
					}
				}
				// Parse range key bounds.
				smallestRangeKey, err = d.readBytes()
				if err != nil {
					return err
				}
				largestRangeKey, err = d.readBytes()
				if err != nil {
					return err
				}
			}
			var smallestSeqNum uint64
			var largestSeqNum uint64
			if tag != tagNewFile {
				smallestSeqNum, err = d.readUvarint()
				if err != nil {
					return err
				}
				largestSeqNum, err = d.readUvarint()
				if err != nil {
					return err
				}
			}
			var markedForCompaction bool
			var creationTime uint64
			virtualState := struct {
				virtual        bool
				backingFileNum uint64
			}{}
			if tag == tagNewFile4 || tag == tagNewFile5 {
				for {
					customTag, err := d.readUvarint()
					if err != nil {
						return err
					}
					if customTag == customTagTerminate {
						break
					} else if customTag == customTagVirtual {
						virtualState.virtual = true
						n, err := d.readUvarint()
						if err != nil {
							return err
						}
						virtualState.backingFileNum = n
						continue
					}

					field, err := d.readBytes()
					if err != nil {
						return err
					}
					switch customTag {
					case customTagNeedsCompaction:
						if len(field) != 1 {
							return base.CorruptionErrorf("new-file4: need-compaction field wrong size")
						}
						markedForCompaction = (field[0] == 1)

					case customTagCreationTime:
						var n int
						creationTime, n = binary.Uvarint(field)
						if n != len(field) {
							return base.CorruptionErrorf("new-file4: invalid file creation time")
						}

					case customTagPathID:
						return base.CorruptionErrorf("new-file4: path-id field not supported")

					default:
						if (customTag & customTagNonSafeIgnoreMask) != 0 {
							return base.CorruptionErrorf("new-file4: custom field not supported: %d", customTag)
						}
					}
				}
			}
			m := &FileMetadata{
				FileNum:             fileNum,
				Size:                size,
				CreationTime:        int64(creationTime),
				SmallestSeqNum:      smallestSeqNum,
				LargestSeqNum:       largestSeqNum,
				MarkedForCompaction: markedForCompaction,
				Virtual:             virtualState.virtual,
			}
			if tag != tagNewFile5 { // no range keys present
				m.SmallestPointKey = base.DecodeInternalKey(smallestPointKey)
				m.LargestPointKey = base.DecodeInternalKey(largestPointKey)
				m.HasPointKeys = true
				m.Smallest, m.Largest = m.SmallestPointKey, m.LargestPointKey
				m.boundTypeSmallest, m.boundTypeLargest = boundTypePointKey, boundTypePointKey
			} else { // range keys present
				// Set point key bounds, if parsed.
				if parsedPointBounds {
					m.SmallestPointKey = base.DecodeInternalKey(smallestPointKey)
					m.LargestPointKey = base.DecodeInternalKey(largestPointKey)
					m.HasPointKeys = true
				}
				// Set range key bounds.
				m.SmallestRangeKey = base.DecodeInternalKey(smallestRangeKey)
				m.LargestRangeKey = base.DecodeInternalKey(largestRangeKey)
				m.HasRangeKeys = true
				// Set overall bounds (by default assume range keys).
				m.Smallest, m.Largest = m.SmallestRangeKey, m.LargestRangeKey
				m.boundTypeSmallest, m.boundTypeLargest = boundTypeRangeKey, boundTypeRangeKey
				if boundsMarker&maskSmallest == maskSmallest {
					m.Smallest = m.SmallestPointKey
					m.boundTypeSmallest = boundTypePointKey
				}
				if boundsMarker&maskLargest == maskLargest {
					m.Largest = m.LargestPointKey
					m.boundTypeLargest = boundTypePointKey
				}
			}
			m.boundsSet = true
			if !virtualState.virtual {
				m.InitPhysicalBacking()
			}

			nfe := NewFileEntry{
				Level: level,
				Meta:  m,
			}
			if virtualState.virtual {
				nfe.BackingFileNum = base.FileNum(virtualState.backingFileNum).DiskFileNum()
			}
			v.NewFiles = append(v.NewFiles, nfe)

		case tagPrevLogNumber:
			n, err := d.readUvarint()
			if err != nil {
				return err
			}
			v.ObsoletePrevLogNum = n

		case tagColumnFamily, tagColumnFamilyAdd, tagColumnFamilyDrop, tagMaxColumnFamily:
			return base.CorruptionErrorf("column families are not supported")

		default:
			return errCorruptManifest
		}
	}
	return nil
}

func (v *VersionEdit) string(verbose bool, fmtKey base.FormatKey) string {
	var buf bytes.Buffer
	if v.ComparerName != "" {
		fmt.Fprintf(&buf, "  comparer:     %s", v.ComparerName)
	}
	if v.MinUnflushedLogNum != 0 {
		fmt.Fprintf(&buf, "  log-num:       %d\n", v.MinUnflushedLogNum)
	}
	if v.ObsoletePrevLogNum != 0 {
		fmt.Fprintf(&buf, "  prev-log-num:  %d\n", v.ObsoletePrevLogNum)
	}
	if v.NextFileNum != 0 {
		fmt.Fprintf(&buf, "  next-file-num: %d\n", v.NextFileNum)
	}
	if v.LastSeqNum != 0 {
		fmt.Fprintf(&buf, "  last-seq-num:  %d\n", v.LastSeqNum)
	}
	entries := make([]DeletedFileEntry, 0, len(v.DeletedFiles))
	for df := range v.DeletedFiles {
		entries = append(entries, df)
	}
	sort.Slice(entries, func(i, j int) bool {
		if entries[i].Level != entries[j].Level {
			return entries[i].Level < entries[j].Level
		}
		return entries[i].FileNum < entries[j].FileNum
	})
	for _, df := range entries {
		fmt.Fprintf(&buf, "  deleted:       L%d %s\n", df.Level, df.FileNum)
	}
	for _, nf := range v.NewFiles {
		fmt.Fprintf(&buf, "  added:         L%d", nf.Level)
		if verbose {
			fmt.Fprintf(&buf, " %s", nf.Meta.DebugString(fmtKey, true /* verbose */))
		} else {
			fmt.Fprintf(&buf, " %s", nf.Meta.String())
		}
		if nf.Meta.CreationTime != 0 {
			fmt.Fprintf(&buf, " (%s)",
				time.Unix(nf.Meta.CreationTime, 0).UTC().Format(time.RFC3339))
		}
		fmt.Fprintln(&buf)
	}
	return buf.String()
}

// DebugString is a more verbose version of String(). Use this in tests.
func (v *VersionEdit) DebugString(fmtKey base.FormatKey) string {
	return v.string(true /* verbose */, fmtKey)
}

// String implements fmt.Stringer for a VersionEdit.
func (v *VersionEdit) String() string {
	return v.string(false /* verbose */, base.DefaultFormatter)
}

// Encode encodes an edit to the specified writer.
func (v *VersionEdit) Encode(w io.Writer) error {
	e := versionEditEncoder{new(bytes.Buffer)}

	if v.ComparerName != "" {
		e.writeUvarint(tagComparator)
		e.writeString(v.ComparerName)
	}
	if v.MinUnflushedLogNum != 0 {
		e.writeUvarint(tagLogNumber)
		e.writeUvarint(uint64(v.MinUnflushedLogNum))
	}
	if v.ObsoletePrevLogNum != 0 {
		e.writeUvarint(tagPrevLogNumber)
		e.writeUvarint(v.ObsoletePrevLogNum)
	}
	if v.NextFileNum != 0 {
		e.writeUvarint(tagNextFileNumber)
		e.writeUvarint(uint64(v.NextFileNum))
	}
	for _, dfn := range v.RemovedBackingTables {
		e.writeUvarint(tagRemovedBackingTable)
		e.writeUvarint(uint64(dfn.FileNum()))
	}
	for _, fileBacking := range v.CreatedBackingTables {
		e.writeUvarint(tagCreatedBackingTable)
		e.writeUvarint(uint64(fileBacking.DiskFileNum.FileNum()))
		e.writeUvarint(fileBacking.Size)
	}
	// RocksDB requires LastSeqNum to be encoded for the first MANIFEST entry,
	// even though its value is zero. We detect this by encoding LastSeqNum when
	// ComparerName is set.
	if v.LastSeqNum != 0 || v.ComparerName != "" {
		e.writeUvarint(tagLastSequence)
		e.writeUvarint(v.LastSeqNum)
	}
	for x := range v.DeletedFiles {
		e.writeUvarint(tagDeletedFile)
		e.writeUvarint(uint64(x.Level))
		e.writeUvarint(uint64(x.FileNum))
	}
	for _, x := range v.NewFiles {
		customFields := x.Meta.MarkedForCompaction || x.Meta.CreationTime != 0 || x.Meta.Virtual
		var tag uint64
		switch {
		case x.Meta.HasRangeKeys:
			tag = tagNewFile5
		case customFields:
			tag = tagNewFile4
		default:
			tag = tagNewFile2
		}
		e.writeUvarint(tag)
		e.writeUvarint(uint64(x.Level))
		e.writeUvarint(uint64(x.Meta.FileNum))
		e.writeUvarint(x.Meta.Size)
		if !x.Meta.HasRangeKeys {
			// If we have no range keys, preserve the original format and write the
			// smallest and largest point keys.
			e.writeKey(x.Meta.SmallestPointKey)
			e.writeKey(x.Meta.LargestPointKey)
		} else {
			// When range keys are present, we first write a marker byte that
			// indicates if the table also contains point keys, in addition to how the
			// overall bounds for the table should be reconstructed. This byte is
			// followed by the keys themselves.
			b, err := x.Meta.boundsMarker()
			if err != nil {
				return err
			}
			if err = e.WriteByte(b); err != nil {
				return err
			}
			// Write point key bounds (if present).
			if x.Meta.HasPointKeys {
				e.writeKey(x.Meta.SmallestPointKey)
				e.writeKey(x.Meta.LargestPointKey)
			}
			// Write range key bounds.
			e.writeKey(x.Meta.SmallestRangeKey)
			e.writeKey(x.Meta.LargestRangeKey)
		}
		e.writeUvarint(x.Meta.SmallestSeqNum)
		e.writeUvarint(x.Meta.LargestSeqNum)
		if customFields {
			if x.Meta.CreationTime != 0 {
				e.writeUvarint(customTagCreationTime)
				var buf [binary.MaxVarintLen64]byte
				n := binary.PutUvarint(buf[:], uint64(x.Meta.CreationTime))
				e.writeBytes(buf[:n])
			}
			if x.Meta.MarkedForCompaction {
				e.writeUvarint(customTagNeedsCompaction)
				e.writeBytes([]byte{1})
			}
			if x.Meta.Virtual {
				e.writeUvarint(customTagVirtual)
				e.writeUvarint(uint64(x.Meta.FileBacking.DiskFileNum.FileNum()))
			}
			e.writeUvarint(customTagTerminate)
		}
	}
	_, err := w.Write(e.Bytes())
	return err
}

// versionEditDecoder should be used to decode version edits.
type versionEditDecoder struct {
	byteReader
}

func (d versionEditDecoder) readBytes() ([]byte, error) {
	n, err := d.readUvarint()
	if err != nil {
		return nil, err
	}
	s := make([]byte, n)
	_, err = io.ReadFull(d, s)
	if err != nil {
		if err == io.ErrUnexpectedEOF {
			return nil, errCorruptManifest
		}
		return nil, err
	}
	return s, nil
}

func (d versionEditDecoder) readLevel() (int, error) {
	u, err := d.readUvarint()
	if err != nil {
		return 0, err
	}
	if u >= NumLevels {
		return 0, errCorruptManifest
	}
	return int(u), nil
}

func (d versionEditDecoder) readFileNum() (base.FileNum, error) {
	u, err := d.readUvarint()
	if err != nil {
		return 0, err
	}
	return base.FileNum(u), nil
}

func (d versionEditDecoder) readUvarint() (uint64, error) {
	u, err := binary.ReadUvarint(d)
	if err != nil {
		if err == io.EOF {
			return 0, errCorruptManifest
		}
		return 0, err
	}
	return u, nil
}

type versionEditEncoder struct {
	*bytes.Buffer
}

func (e versionEditEncoder) writeBytes(p []byte) {
	e.writeUvarint(uint64(len(p)))
	e.Write(p)
}

func (e versionEditEncoder) writeKey(k InternalKey) {
	e.writeUvarint(uint64(k.Size()))
	e.Write(k.UserKey)
	buf := k.EncodeTrailer()
	e.Write(buf[:])
}

func (e versionEditEncoder) writeString(s string) {
	e.writeUvarint(uint64(len(s)))
	e.WriteString(s)
}

func (e versionEditEncoder) writeUvarint(u uint64) {
	var buf [binary.MaxVarintLen64]byte
	n := binary.PutUvarint(buf[:], u)
	e.Write(buf[:n])
}

// BulkVersionEdit summarizes the files added and deleted from a set of version
// edits.
//
// INVARIANTS:
// No file can be added to a level more than once. This is true globally, and
// also true for all of the calls to Accumulate for a single bulk version edit.
//
// No file can be removed from a level more than once. This is true globally,
// and also true for all of the calls to Accumulate for a single bulk version
// edit.
//
// A file must not be added and removed from a given level in the same version
// edit.
//
// A file that is being removed from a level must have been added to that level
// before (in a prior version edit). Note that a given file can be deleted from
// a level and added to another level in a single version edit
type BulkVersionEdit struct {
	Added   [NumLevels]map[base.FileNum]*FileMetadata
	Deleted [NumLevels]map[base.FileNum]*FileMetadata

	// AddedFileBacking is a map to support lookup so that we can populate the
	// FileBacking of virtual sstables during manifest replay.
	AddedFileBacking   map[base.DiskFileNum]*FileBacking
	RemovedFileBacking []base.DiskFileNum

	// AddedByFileNum maps file number to file metadata for all added files
	// from accumulated version edits. AddedByFileNum is only populated if set
	// to non-nil by a caller. It must be set to non-nil when replaying
	// version edits read from a MANIFEST (as opposed to VersionEdits
	// constructed in-memory).  While replaying a MANIFEST file,
	// VersionEdit.DeletedFiles map entries have nil values, because the
	// on-disk deletion record encodes only the file number. Accumulate
	// uses AddedByFileNum to correctly populate the BulkVersionEdit's Deleted
	// field with non-nil *FileMetadata.
	AddedByFileNum map[base.FileNum]*FileMetadata

	// MarkedForCompactionCountDiff holds the aggregated count of files
	// marked for compaction added or removed.
	MarkedForCompactionCountDiff int
}

// Accumulate adds the file addition and deletions in the specified version
// edit to the bulk edit's internal state.
//
// INVARIANTS:
// If a file is added to a given level in a call to Accumulate and then removed
// from that level in a subsequent call, the file will not be present in the
// resulting BulkVersionEdit.Deleted for that level.
//
// After accumulation of version edits, the bulk version edit may have
// information about a file which has been deleted from a level, but it may
// not have information about the same file added to the same level. The add
// could've occurred as part of a previous bulk version edit. In this case,
// the deleted file must be present in BulkVersionEdit.Deleted, at the end
// of the accumulation, because we need to decrease the refcount of the
// deleted file in Apply.
func (b *BulkVersionEdit) Accumulate(ve *VersionEdit) error {
	for df, m := range ve.DeletedFiles {
		dmap := b.Deleted[df.Level]
		if dmap == nil {
			dmap = make(map[base.FileNum]*FileMetadata)
			b.Deleted[df.Level] = dmap
		}

		if m == nil {
			// m is nil only when replaying a MANIFEST.
			if b.AddedByFileNum == nil {
				return errors.Errorf("deleted file L%d.%s's metadata is absent and bve.AddedByFileNum is nil", df.Level, df.FileNum)
			}
			m = b.AddedByFileNum[df.FileNum]
			if m == nil {
				return base.CorruptionErrorf("pebble: file deleted L%d.%s before it was inserted", df.Level, df.FileNum)
			}
		}
		if m.MarkedForCompaction {
			b.MarkedForCompactionCountDiff--
		}
		if _, ok := b.Added[df.Level][df.FileNum]; !ok {
			dmap[df.FileNum] = m
		} else {
			// Present in b.Added for the same level.
			delete(b.Added[df.Level], df.FileNum)
		}
	}

	// Generate state for Added backing files. Note that these must be generated
	// before we loop through the NewFiles, because we need to populate the
	// FileBackings which might be used by the NewFiles loop.
	if b.AddedFileBacking == nil {
		b.AddedFileBacking = make(map[base.DiskFileNum]*FileBacking)
	}
	for _, fb := range ve.CreatedBackingTables {
		if _, ok := b.AddedFileBacking[fb.DiskFileNum]; ok {
			// There is already a FileBacking associated with fb.DiskFileNum.
			// This should never happen. There must always be only one FileBacking
			// associated with a backing sstable.
			panic(fmt.Sprintf("pebble: duplicate file backing %s", fb.DiskFileNum.String()))
		}
		b.AddedFileBacking[fb.DiskFileNum] = fb
	}

	for _, nf := range ve.NewFiles {
		// A new file should not have been deleted in this or a preceding
		// VersionEdit at the same level (though files can move across levels).
		if dmap := b.Deleted[nf.Level]; dmap != nil {
			if _, ok := dmap[nf.Meta.FileNum]; ok {
				return base.CorruptionErrorf("pebble: file deleted L%d.%s before it was inserted", nf.Level, nf.Meta.FileNum)
			}
		}
		if nf.Meta.Virtual && nf.Meta.FileBacking == nil {
			// FileBacking for a virtual sstable must only be nil if we're performing
			// manifest replay.
			nf.Meta.FileBacking = b.AddedFileBacking[nf.BackingFileNum]
			if nf.Meta.FileBacking == nil {
				return errors.Errorf("FileBacking for virtual sstable must not be nil")
			}
		} else if nf.Meta.FileBacking == nil {
			return errors.Errorf("Added file L%d.%s's has no FileBacking", nf.Level, nf.Meta.FileNum)
		}

		if b.Added[nf.Level] == nil {
			b.Added[nf.Level] = make(map[base.FileNum]*FileMetadata)
		}
		b.Added[nf.Level][nf.Meta.FileNum] = nf.Meta
		if b.AddedByFileNum != nil {
			b.AddedByFileNum[nf.Meta.FileNum] = nf.Meta
		}
		if nf.Meta.MarkedForCompaction {
			b.MarkedForCompactionCountDiff++
		}
	}

	// Since a file can be removed from backing files in exactly one version
	// edit it is safe to just append without any de-duplication.
	b.RemovedFileBacking = append(b.RemovedFileBacking, ve.RemovedBackingTables...)

	return nil
}

// AccumulateIncompleteAndApplySingleVE should be called if a single version edit
// is to be applied to the provided curr Version and if the caller needs to
// update the versionSet.zombieTables map. This function exists separately from
// BulkVersionEdit.Apply because it is easier to reason about properties
// regarding BulkVersionedit.Accumulate/Apply and zombie table generation, if we
// know that exactly one version edit is being accumulated.
//
// Note that the version edit passed into this function may be incomplete
// because compactions don't have the ref counting information necessary to
// populate VersionEdit.RemovedBackingTables. This function will complete such a
// version edit by populating RemovedBackingTables.
//
// Invariant: Any file being deleted through ve must belong to the curr Version.
// We can't have a delete for some arbitrary file which does not exist in curr.
func AccumulateIncompleteAndApplySingleVE(
	ve *VersionEdit,
	curr *Version,
	cmp Compare,
	formatKey base.FormatKey,
	flushSplitBytes int64,
	readCompactionRate int64,
	backingStateMap map[base.DiskFileNum]*FileBacking,
	addBackingFunc func(*FileBacking),
	removeBackingFunc func(base.DiskFileNum),
	orderingInvariants OrderingInvariants,
) (_ *Version, zombies map[base.DiskFileNum]uint64, _ error) {
	if len(ve.RemovedBackingTables) != 0 {
		panic("pebble: invalid incomplete version edit")
	}
	var b BulkVersionEdit
	err := b.Accumulate(ve)
	if err != nil {
		return nil, nil, err
	}
	zombies = make(map[base.DiskFileNum]uint64)
	v, err := b.Apply(
		curr, cmp, formatKey, flushSplitBytes, readCompactionRate, zombies, orderingInvariants,
	)
	if err != nil {
		return nil, nil, err
	}

	for _, s := range b.AddedFileBacking {
		addBackingFunc(s)
	}

	for fileNum := range zombies {
		if _, ok := backingStateMap[fileNum]; ok {
			// This table was backing some virtual sstable in the latest version,
			// but is now a zombie. We add RemovedBackingTables entries for
			// these, before the version edit is written to disk.
			ve.RemovedBackingTables = append(
				ve.RemovedBackingTables, fileNum,
			)
			removeBackingFunc(fileNum)
		}
	}
	return v, zombies, nil
}

// Apply applies the delta b to the current version to produce a new
// version. The new version is consistent with respect to the comparer cmp.
//
// curr may be nil, which is equivalent to a pointer to a zero version.
//
// On success, if a non-nil zombies map is provided to Apply, the map is updated
// with file numbers and files sizes of deleted files. These files are
// considered zombies because they are no longer referenced by the returned
// Version, but cannot be deleted from disk as they are still in use by the
// incoming Version.
func (b *BulkVersionEdit) Apply(
	curr *Version,
	cmp Compare,
	formatKey base.FormatKey,
	flushSplitBytes int64,
	readCompactionRate int64,
	zombies map[base.DiskFileNum]uint64,
	orderingInvariants OrderingInvariants,
) (*Version, error) {
	addZombie := func(state *FileBacking) {
		if zombies != nil {
			zombies[state.DiskFileNum] = state.Size
		}
	}
	removeZombie := func(state *FileBacking) {
		if zombies != nil {
			delete(zombies, state.DiskFileNum)
		}
	}

	v := new(Version)

	// Adjust the count of files marked for compaction.
	if curr != nil {
		v.Stats.MarkedForCompaction = curr.Stats.MarkedForCompaction
	}
	v.Stats.MarkedForCompaction += b.MarkedForCompactionCountDiff
	if v.Stats.MarkedForCompaction < 0 {
		return nil, base.CorruptionErrorf("pebble: version marked for compaction count negative")
	}

	for level := range v.Levels {
		if curr == nil || curr.Levels[level].tree.root == nil {
			v.Levels[level] = makeLevelMetadata(cmp, level, nil /* files */)
		} else {
			v.Levels[level] = curr.Levels[level].clone()
		}
		if curr == nil || curr.RangeKeyLevels[level].tree.root == nil {
			v.RangeKeyLevels[level] = makeLevelMetadata(cmp, level, nil /* files */)
		} else {
			v.RangeKeyLevels[level] = curr.RangeKeyLevels[level].clone()
		}

		if len(b.Added[level]) == 0 && len(b.Deleted[level]) == 0 {
			// There are no edits on this level.
			if level == 0 {
				// Initialize L0Sublevels.
				if curr == nil || curr.L0Sublevels == nil {
					if err := v.InitL0Sublevels(cmp, formatKey, flushSplitBytes); err != nil {
						return nil, errors.Wrap(err, "pebble: internal error")
					}
				} else {
					v.L0Sublevels = curr.L0Sublevels
					v.L0SublevelFiles = v.L0Sublevels.Levels
				}
			}
			continue
		}

		// Some edits on this level.
		lm := &v.Levels[level]
		lmRange := &v.RangeKeyLevels[level]

		addedFilesMap := b.Added[level]
		deletedFilesMap := b.Deleted[level]
		if n := v.Levels[level].Len() + len(addedFilesMap); n == 0 {
			return nil, base.CorruptionErrorf(
				"pebble: internal error: No current or added files but have deleted files: %d",
				errors.Safe(len(deletedFilesMap)))
		}

		// NB: addedFilesMap may be empty. If a file is present in addedFilesMap
		// for a level, it won't be present in deletedFilesMap for the same
		// level.

		for _, f := range deletedFilesMap {
			if obsolete := v.Levels[level].remove(f); obsolete {
				// Deleting a file from the B-Tree may decrement its
				// reference count. However, because we cloned the
				// previous level's B-Tree, this should never result in a
				// file's reference count dropping to zero.
				err := errors.Errorf("pebble: internal error: file L%d.%s obsolete during B-Tree removal", level, f.FileNum)
				return nil, err
			}
			if f.HasRangeKeys {
				if obsolete := v.RangeKeyLevels[level].remove(f); obsolete {
					// Deleting a file from the B-Tree may decrement its
					// reference count. However, because we cloned the
					// previous level's B-Tree, this should never result in a
					// file's reference count dropping to zero.
					err := errors.Errorf("pebble: internal error: file L%d.%s obsolete during range-key B-Tree removal", level, f.FileNum)
					return nil, err
				}
			}

			// Note that a backing sst will only become a zombie if the
			// references to it in the latest version is 0. We will remove the
			// backing sst from the zombie list in the next loop if one of the
			// addedFiles in any of the levels is referencing the backing sst.
			// This is possible if a physical sstable is virtualized, or if it
			// is moved.
			latestRefCount := f.LatestRefs()
			if latestRefCount <= 0 {
				// If a file is present in deletedFilesMap for a level, then it
				// must have already been added to the level previously, which
				// means that its latest ref count cannot be 0.
				err := errors.Errorf("pebble: internal error: incorrect latestRefs reference counting for file", f.FileNum)
				return nil, err
			} else if f.LatestUnref() == 0 {
				addZombie(f.FileBacking)
			}
		}

		addedFiles := make([]*FileMetadata, 0, len(addedFilesMap))
		for _, f := range addedFilesMap {
			addedFiles = append(addedFiles, f)
		}
		// Sort addedFiles by file number. This isn't necessary, but tests which
		// replay invalid manifests check the error output, and the error output
		// depends on the order in which files are added to the btree.
		sort.Slice(addedFiles, func(i, j int) bool {
			return addedFiles[i].FileNum < addedFiles[j].FileNum
		})

		var sm, la *FileMetadata
		for _, f := range addedFiles {
			// NB: allowedSeeks is used for read triggered compactions. It is set using
			// Options.Experimental.ReadCompactionRate which defaults to 32KB.
			var allowedSeeks int64
			if readCompactionRate != 0 {
				allowedSeeks = int64(f.Size) / readCompactionRate
			}
			if allowedSeeks < 100 {
				allowedSeeks = 100
			}
			f.AllowedSeeks.Store(allowedSeeks)
			f.InitAllowedSeeks = allowedSeeks

			err := lm.insert(f)
			// We're adding this file to the new version, so increment the
			// latest refs count.
			f.LatestRef()
			if err != nil {
				return nil, errors.Wrap(err, "pebble")
			}
			if f.HasRangeKeys {
				err = lmRange.insert(f)
				if err != nil {
					return nil, errors.Wrap(err, "pebble")
				}
			}
			removeZombie(f.FileBacking)
			// Track the keys with the smallest and largest keys, so that we can
			// check consistency of the modified span.
			if sm == nil || base.InternalCompare(cmp, sm.Smallest, f.Smallest) > 0 {
				sm = f
			}
			if la == nil || base.InternalCompare(cmp, la.Largest, f.Largest) < 0 {
				la = f
			}
		}

		if level == 0 {
			if curr != nil && curr.L0Sublevels != nil && len(deletedFilesMap) == 0 {
				// Flushes and ingestions that do not delete any L0 files do not require
				// a regeneration of L0Sublevels from scratch. We can instead generate
				// it incrementally.
				var err error
				// AddL0Files requires addedFiles to be sorted in seqnum order.
				SortBySeqNum(addedFiles)
				v.L0Sublevels, err = curr.L0Sublevels.AddL0Files(addedFiles, flushSplitBytes, &v.Levels[0])
				if errors.Is(err, errInvalidL0SublevelsOpt) {
					err = v.InitL0Sublevels(cmp, formatKey, flushSplitBytes)
				} else if invariants.Enabled && err == nil {
					copyOfSublevels, err := NewL0Sublevels(&v.Levels[0], cmp, formatKey, flushSplitBytes)
					if err != nil {
						panic(fmt.Sprintf("error when regenerating sublevels: %s", err))
					}
					s1 := describeSublevels(base.DefaultFormatter, false /* verbose */, copyOfSublevels.Levels)
					s2 := describeSublevels(base.DefaultFormatter, false /* verbose */, v.L0Sublevels.Levels)
					if s1 != s2 {
						panic(fmt.Sprintf("incremental L0 sublevel generation produced different output than regeneration: %s != %s", s1, s2))
					}
				}
				if err != nil {
					return nil, errors.Wrap(err, "pebble: internal error")
				}
				v.L0SublevelFiles = v.L0Sublevels.Levels
			} else if err := v.InitL0Sublevels(cmp, formatKey, flushSplitBytes); err != nil {
				return nil, errors.Wrap(err, "pebble: internal error")
			}
			if err := CheckOrdering(cmp, formatKey, Level(0), v.Levels[level].Iter(), orderingInvariants); err != nil {
				return nil, errors.Wrap(err, "pebble: internal error")
			}
			continue
		}

		// Check consistency of the level in the vicinity of our edits.
		if sm != nil && la != nil {
			overlap := overlaps(v.Levels[level].Iter(), cmp, sm.Smallest.UserKey,
				la.Largest.UserKey, la.Largest.IsExclusiveSentinel())
			// overlap contains all of the added files. We want to ensure that
			// the added files are consistent with neighboring existing files
			// too, so reslice the overlap to pull in a neighbor on each side.
			check := overlap.Reslice(func(start, end *LevelIterator) {
				if m := start.Prev(); m == nil {
					start.Next()
				}
				if m := end.Next(); m == nil {
					end.Prev()
				}
			})
			if err := CheckOrdering(cmp, formatKey, Level(level), check.Iter(), orderingInvariants); err != nil {
				return nil, errors.Wrap(err, "pebble: internal error")
			}
		}
	}
	return v, nil
}
