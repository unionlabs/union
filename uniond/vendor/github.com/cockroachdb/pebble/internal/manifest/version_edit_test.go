// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/record"
	"github.com/kr/pretty"
	"github.com/stretchr/testify/require"
)

func checkRoundTrip(e0 VersionEdit) error {
	var e1 VersionEdit
	buf := new(bytes.Buffer)
	if err := e0.Encode(buf); err != nil {
		return errors.Wrap(err, "encode")
	}
	if err := e1.Decode(buf); err != nil {
		return errors.Wrap(err, "decode")
	}
	if diff := pretty.Diff(e0, e1); diff != nil {
		return errors.Errorf("%s", strings.Join(diff, "\n"))
	}
	return nil
}

// Version edits with virtual sstables will not be the same after a round trip
// as the Decode function will not set the FileBacking for a virtual sstable.
// We test round trip + bve accumulation here, after which the virtual sstable
// FileBacking should be set.
func TestVERoundTripAndAccumulate(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	m1 := (&FileMetadata{
		FileNum:        810,
		Size:           8090,
		CreationTime:   809060,
		SmallestSeqNum: 9,
		LargestSeqNum:  11,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet),
		base.MakeInternalKey([]byte("m"), 0, base.InternalKeyKindSet),
	).ExtendRangeKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("l"), 0, base.InternalKeyKindRangeKeySet),
		base.MakeExclusiveSentinelKey(base.InternalKeyKindRangeKeySet, []byte("z")),
	)
	m1.InitPhysicalBacking()

	m2 := (&FileMetadata{
		FileNum:        812,
		Size:           8090,
		CreationTime:   809060,
		SmallestSeqNum: 9,
		LargestSeqNum:  11,
		Virtual:        true,
		FileBacking:    m1.FileBacking,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet),
		base.MakeInternalKey([]byte("c"), 0, base.InternalKeyKindSet),
	)

	ve1 := VersionEdit{
		ComparerName:         "11",
		MinUnflushedLogNum:   22,
		ObsoletePrevLogNum:   33,
		NextFileNum:          44,
		LastSeqNum:           55,
		CreatedBackingTables: []*FileBacking{m1.FileBacking},
		NewFiles: []NewFileEntry{
			{
				Level: 4,
				Meta:  m2,
				// Only set for the test.
				BackingFileNum: m2.FileBacking.DiskFileNum,
			},
		},
	}
	var err error
	buf := new(bytes.Buffer)
	if err = ve1.Encode(buf); err != nil {
		t.Error(err)
	}
	var ve2 VersionEdit
	if err = ve2.Decode(buf); err != nil {
		t.Error(err)
	}
	// Perform accumulation to set the FileBacking on the files in the Decoded
	// version edit.
	var bve BulkVersionEdit
	require.NoError(t, bve.Accumulate(&ve2))
	if diff := pretty.Diff(ve1, ve2); diff != nil {
		t.Error(errors.Errorf("%s", strings.Join(diff, "\n")))
	}
}

func TestVersionEditRoundTrip(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	m1 := (&FileMetadata{
		FileNum:      805,
		Size:         8050,
		CreationTime: 805030,
	}).ExtendPointKeyBounds(
		cmp,
		base.DecodeInternalKey([]byte("abc\x00\x01\x02\x03\x04\x05\x06\x07")),
		base.DecodeInternalKey([]byte("xyz\x01\xff\xfe\xfd\xfc\xfb\xfa\xf9")),
	)
	m1.InitPhysicalBacking()

	m2 := (&FileMetadata{
		FileNum:             806,
		Size:                8060,
		CreationTime:        806040,
		SmallestSeqNum:      3,
		LargestSeqNum:       5,
		MarkedForCompaction: true,
	}).ExtendPointKeyBounds(
		cmp,
		base.DecodeInternalKey([]byte("A\x00\x01\x02\x03\x04\x05\x06\x07")),
		base.DecodeInternalKey([]byte("Z\x01\xff\xfe\xfd\xfc\xfb\xfa\xf9")),
	)
	m2.InitPhysicalBacking()

	m3 := (&FileMetadata{
		FileNum:      807,
		Size:         8070,
		CreationTime: 807050,
	}).ExtendRangeKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("aaa"), 0, base.InternalKeyKindRangeKeySet),
		base.MakeExclusiveSentinelKey(base.InternalKeyKindRangeKeySet, []byte("zzz")),
	)
	m3.InitPhysicalBacking()

	m4 := (&FileMetadata{
		FileNum:        809,
		Size:           8090,
		CreationTime:   809060,
		SmallestSeqNum: 9,
		LargestSeqNum:  11,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet),
		base.MakeInternalKey([]byte("m"), 0, base.InternalKeyKindSet),
	).ExtendRangeKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("l"), 0, base.InternalKeyKindRangeKeySet),
		base.MakeExclusiveSentinelKey(base.InternalKeyKindRangeKeySet, []byte("z")),
	)
	m4.InitPhysicalBacking()

	m5 := (&FileMetadata{
		FileNum:        810,
		Size:           8090,
		CreationTime:   809060,
		SmallestSeqNum: 9,
		LargestSeqNum:  11,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet),
		base.MakeInternalKey([]byte("m"), 0, base.InternalKeyKindSet),
	).ExtendRangeKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("l"), 0, base.InternalKeyKindRangeKeySet),
		base.MakeExclusiveSentinelKey(base.InternalKeyKindRangeKeySet, []byte("z")),
	)
	m5.InitPhysicalBacking()

	m6 := (&FileMetadata{
		FileNum:        811,
		Size:           8090,
		CreationTime:   809060,
		SmallestSeqNum: 9,
		LargestSeqNum:  11,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("a"), 0, base.InternalKeyKindSet),
		base.MakeInternalKey([]byte("m"), 0, base.InternalKeyKindSet),
	).ExtendRangeKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("l"), 0, base.InternalKeyKindRangeKeySet),
		base.MakeExclusiveSentinelKey(base.InternalKeyKindRangeKeySet, []byte("z")),
	)
	m6.InitPhysicalBacking()

	testCases := []VersionEdit{
		// An empty version edit.
		{},
		// A complete version edit.
		{
			ComparerName:       "11",
			MinUnflushedLogNum: 22,
			ObsoletePrevLogNum: 33,
			NextFileNum:        44,
			LastSeqNum:         55,
			RemovedBackingTables: []base.DiskFileNum{
				base.FileNum(10).DiskFileNum(), base.FileNum(11).DiskFileNum(),
			},
			CreatedBackingTables: []*FileBacking{m5.FileBacking, m6.FileBacking},
			DeletedFiles: map[DeletedFileEntry]*FileMetadata{
				{
					Level:   3,
					FileNum: 703,
				}: nil,
				{
					Level:   4,
					FileNum: 704,
				}: nil,
			},
			NewFiles: []NewFileEntry{
				{
					Level: 4,
					Meta:  m1,
				},
				{
					Level: 5,
					Meta:  m2,
				},
				{
					Level: 6,
					Meta:  m3,
				},
				{
					Level: 6,
					Meta:  m4,
				},
			},
		},
	}
	for _, tc := range testCases {
		if err := checkRoundTrip(tc); err != nil {
			t.Error(err)
		}
	}
}

func TestVersionEditDecode(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	m := (&FileMetadata{
		FileNum:        4,
		Size:           986,
		SmallestSeqNum: 3,
		LargestSeqNum:  5,
	}).ExtendPointKeyBounds(
		cmp,
		base.MakeInternalKey([]byte("bar"), 5, base.InternalKeyKindDelete),
		base.MakeInternalKey([]byte("foo"), 4, base.InternalKeyKindSet),
	)
	m.InitPhysicalBacking()

	testCases := []struct {
		filename     string
		encodedEdits []string
		edits        []VersionEdit
	}{
		// db-stage-1 and db-stage-2 have the same manifest.
		{
			filename: "db-stage-1/MANIFEST-000001",
			encodedEdits: []string{
				"\x02\x00\x03\x02\x04\x00",
			},
			edits: []VersionEdit{
				{
					NextFileNum: 2,
				},
			},
		},
		// db-stage-3 and db-stage-4 have the same manifest.
		{
			filename: "db-stage-3/MANIFEST-000005",
			encodedEdits: []string{
				"\x01\x1aleveldb.BytewiseComparator",
				"\x02\x00",
				"\x02\x04\t\x00\x03\x06\x04\x05d\x00\x04\xda\a\vbar" +
					"\x00\x05\x00\x00\x00\x00\x00\x00\vfoo\x01\x04\x00" +
					"\x00\x00\x00\x00\x00\x03\x05",
			},
			edits: []VersionEdit{
				{
					ComparerName: "leveldb.BytewiseComparator",
				},
				{},
				{
					MinUnflushedLogNum: 4,
					ObsoletePrevLogNum: 0,
					NextFileNum:        6,
					LastSeqNum:         5,
					NewFiles: []NewFileEntry{
						{
							Level: 0,
							Meta:  m,
						},
					},
				},
			},
		},
	}

	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			f, err := os.Open("../../testdata/" + tc.filename)
			if err != nil {
				t.Fatalf("filename=%q: open error: %v", tc.filename, err)
			}
			defer f.Close()
			i, r := 0, record.NewReader(f, 0 /* logNum */)
			for {
				rr, err := r.Next()
				if err == io.EOF {
					break
				}
				if err != nil {
					t.Fatalf("filename=%q i=%d: record reader error: %v", tc.filename, i, err)
				}
				if i >= len(tc.edits) {
					t.Fatalf("filename=%q i=%d: too many version edits", tc.filename, i+1)
				}

				encodedEdit, err := io.ReadAll(rr)
				if err != nil {
					t.Fatalf("filename=%q i=%d: read error: %v", tc.filename, i, err)
				}
				if s := string(encodedEdit); s != tc.encodedEdits[i] {
					t.Fatalf("filename=%q i=%d: got encoded %q, want %q", tc.filename, i, s, tc.encodedEdits[i])
				}

				var edit VersionEdit
				err = edit.Decode(bytes.NewReader(encodedEdit))
				if err != nil {
					t.Fatalf("filename=%q i=%d: decode error: %v", tc.filename, i, err)
				}
				if !reflect.DeepEqual(edit, tc.edits[i]) {
					t.Fatalf("filename=%q i=%d: decode\n\tgot  %#v\n\twant %#v\n%s", tc.filename, i, edit, tc.edits[i],
						strings.Join(pretty.Diff(edit, tc.edits[i]), "\n"))
				}
				if err := checkRoundTrip(edit); err != nil {
					t.Fatalf("filename=%q i=%d: round trip: %v", tc.filename, i, err)
				}

				i++
			}
			if i != len(tc.edits) {
				t.Fatalf("filename=%q: got %d edits, want %d", tc.filename, i, len(tc.edits))
			}
		})
	}
}

func TestVersionEditEncodeLastSeqNum(t *testing.T) {
	testCases := []struct {
		edit    VersionEdit
		encoded string
	}{
		// If ComparerName is unset, LastSeqNum is only encoded if non-zero.
		{VersionEdit{LastSeqNum: 0}, ""},
		{VersionEdit{LastSeqNum: 1}, "\x04\x01"},
		// For compatibility with RocksDB, if ComparerName is set we always encode
		// LastSeqNum.
		{VersionEdit{ComparerName: "foo", LastSeqNum: 0}, "\x01\x03\x66\x6f\x6f\x04\x00"},
		{VersionEdit{ComparerName: "foo", LastSeqNum: 1}, "\x01\x03\x66\x6f\x6f\x04\x01"},
	}
	for _, c := range testCases {
		t.Run("", func(t *testing.T) {
			var buf bytes.Buffer
			require.NoError(t, c.edit.Encode(&buf))
			if result := buf.String(); c.encoded != result {
				t.Fatalf("expected %x, but found %x", c.encoded, result)
			}

			if c.edit.ComparerName != "" {
				// Manually decode the version edit so that we can verify the contents
				// even if the LastSeqNum decodes to 0.
				d := versionEditDecoder{strings.NewReader(c.encoded)}

				// Decode ComparerName.
				tag, err := d.readUvarint()
				require.NoError(t, err)
				if tag != tagComparator {
					t.Fatalf("expected %d, but found %d", tagComparator, tag)
				}
				s, err := d.readBytes()
				require.NoError(t, err)
				if c.edit.ComparerName != string(s) {
					t.Fatalf("expected %q, but found %q", c.edit.ComparerName, s)
				}

				// Decode LastSeqNum.
				tag, err = d.readUvarint()
				require.NoError(t, err)
				if tag != tagLastSequence {
					t.Fatalf("expected %d, but found %d", tagLastSequence, tag)
				}
				val, err := d.readUvarint()
				require.NoError(t, err)
				if c.edit.LastSeqNum != val {
					t.Fatalf("expected %d, but found %d", c.edit.LastSeqNum, val)
				}
			}
		})
	}
}

func TestVersionEditApply(t *testing.T) {
	parseMeta := func(s string) (*FileMetadata, error) {
		m, err := ParseFileMetadataDebug(s)
		if err != nil {
			return nil, err
		}
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		if m.SmallestSeqNum > m.LargestSeqNum {
			m.SmallestSeqNum, m.LargestSeqNum = m.LargestSeqNum, m.SmallestSeqNum
		}
		m.InitPhysicalBacking()
		return m, nil
	}

	// TODO(bananabrick): Improve the parsing logic in this test.
	datadriven.RunTest(t, "testdata/version_edit_apply",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "apply":
				// TODO(sumeer): move this Version parsing code to utils, to
				// avoid repeating it, and make it the inverse of
				// Version.DebugString().
				var v *Version
				var veList []*VersionEdit
				isVersion := true
				isDelete := true
				var level int
				var err error
				versionFiles := map[base.FileNum]*FileMetadata{}
				for _, data := range strings.Split(d.Input, "\n") {
					data = strings.TrimSpace(data)
					switch data {
					case "edit":
						isVersion = false
						veList = append(veList, &VersionEdit{})
					case "delete":
						isVersion = false
						isDelete = true
					case "add":
						isVersion = false
						isDelete = false
					case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
						level, err = strconv.Atoi(data[1:])
						if err != nil {
							return err.Error()
						}
					default:
						var ve *VersionEdit
						if len(veList) > 0 {
							ve = veList[len(veList)-1]
						}
						if isVersion || !isDelete {
							meta, err := parseMeta(data)
							if err != nil {
								return err.Error()
							}
							if isVersion {
								if v == nil {
									v = new(Version)
									for l := 0; l < NumLevels; l++ {
										v.Levels[l] = makeLevelMetadata(base.DefaultComparer.Compare, l, nil /* files */)
									}
								}
								versionFiles[meta.FileNum] = meta
								v.Levels[level].insert(meta)
								meta.LatestRef()
							} else {
								ve.NewFiles =
									append(ve.NewFiles, NewFileEntry{Level: level, Meta: meta})
							}
						} else {
							fileNum, err := strconv.Atoi(data)
							if err != nil {
								return err.Error()
							}
							dfe := DeletedFileEntry{Level: level, FileNum: base.FileNum(fileNum)}
							if ve.DeletedFiles == nil {
								ve.DeletedFiles = make(map[DeletedFileEntry]*FileMetadata)
							}
							ve.DeletedFiles[dfe] = versionFiles[dfe.FileNum]
						}
					}
				}

				if v != nil {
					if err := v.InitL0Sublevels(base.DefaultComparer.Compare, base.DefaultFormatter, 10<<20); err != nil {
						return err.Error()
					}
				}

				bve := BulkVersionEdit{}
				bve.AddedByFileNum = make(map[base.FileNum]*FileMetadata)
				for _, ve := range veList {
					if err := bve.Accumulate(ve); err != nil {
						return err.Error()
					}
				}
				zombies := make(map[base.DiskFileNum]uint64)
				newv, err := bve.Apply(v, base.DefaultComparer.Compare, base.DefaultFormatter, 10<<20, 32000, zombies, ProhibitSplitUserKeys)
				if err != nil {
					return err.Error()
				}

				zombieFileNums := make([]base.DiskFileNum, 0, len(zombies))
				if len(veList) == 1 {
					// Only care about zombies if a single version edit was
					// being applied.
					for fileNum := range zombies {
						zombieFileNums = append(zombieFileNums, fileNum)
					}
					sort.Slice(zombieFileNums, func(i, j int) bool {
						return zombieFileNums[i].FileNum() < zombieFileNums[j].FileNum()
					})
				}

				return fmt.Sprintf("%szombies %d\n", newv, zombieFileNums)

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}
