// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"sort"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invalidating"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/rangekey"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/stretchr/testify/require"
)

func TestSnapshotIndex(t *testing.T) {
	testCases := []struct {
		snapshots      []uint64
		seq            uint64
		expectedIndex  int
		expectedSeqNum uint64
	}{
		{[]uint64{}, 1, 0, InternalKeySeqNumMax},
		{[]uint64{1}, 0, 0, 1},
		{[]uint64{1}, 1, 1, InternalKeySeqNumMax},
		{[]uint64{1}, 2, 1, InternalKeySeqNumMax},
		{[]uint64{1, 3}, 1, 1, 3},
		{[]uint64{1, 3}, 2, 1, 3},
		{[]uint64{1, 3}, 3, 2, InternalKeySeqNumMax},
		{[]uint64{1, 3}, 4, 2, InternalKeySeqNumMax},
		{[]uint64{1, 3, 3}, 2, 1, 3},
	}
	for _, c := range testCases {
		t.Run("", func(t *testing.T) {
			idx, seqNum := snapshotIndex(c.seq, c.snapshots)
			if c.expectedIndex != idx {
				t.Fatalf("expected %d, but got %d", c.expectedIndex, idx)
			}
			if c.expectedSeqNum != seqNum {
				t.Fatalf("expected %d, but got %d", c.expectedSeqNum, seqNum)
			}
		})
	}
}

type debugMerger struct {
	buf []byte
}

func (m *debugMerger) MergeNewer(value []byte) error {
	m.buf = append(m.buf, value...)
	return nil
}

func (m *debugMerger) MergeOlder(value []byte) error {
	buf := make([]byte, 0, len(m.buf)+len(value))
	buf = append(buf, value...)
	buf = append(buf, m.buf...)
	m.buf = buf
	return nil
}

func (m *debugMerger) Finish(includesBase bool) ([]byte, io.Closer, error) {
	if includesBase {
		m.buf = append(m.buf, []byte("[base]")...)
	}
	return m.buf, nil, nil
}

func TestCompactionIter(t *testing.T) {
	var merge Merge
	var keys []InternalKey
	var rangeKeys []keyspan.Span
	var vals [][]byte
	var snapshots []uint64
	var elideTombstones bool
	var allowZeroSeqnum bool
	var interleavingIter *keyspan.InterleavingIter

	// The input to the data-driven test is dependent on the format major
	// version we are testing against.
	fileFunc := func(formatVersion FormatMajorVersion) string {
		if formatVersion < FormatSetWithDelete {
			return "testdata/compaction_iter"
		}
		if formatVersion < FormatDeleteSizedAndObsolete {
			return "testdata/compaction_iter_set_with_del"
		}
		return "testdata/compaction_iter_delete_sized"
	}

	var ineffectualSingleDeleteKeys []string
	var invariantViolationSingleDeleteKeys []string
	resetSingleDelStats := func() {
		ineffectualSingleDeleteKeys = ineffectualSingleDeleteKeys[:0]
		invariantViolationSingleDeleteKeys = invariantViolationSingleDeleteKeys[:0]
	}
	newIter := func(formatVersion FormatMajorVersion) *compactionIter {
		// To adhere to the existing assumption that range deletion blocks in
		// SSTables are not released while iterating, and therefore not
		// susceptible to use-after-free bugs, we skip the zeroing of
		// RangeDelete keys.
		fi := &fakeIter{keys: keys, vals: vals}
		interleavingIter = &keyspan.InterleavingIter{}
		interleavingIter.Init(
			base.DefaultComparer,
			fi,
			keyspan.NewIter(base.DefaultComparer.Compare, rangeKeys),
			keyspan.InterleavingIterOpts{})
		iter := invalidating.NewIter(interleavingIter, invalidating.IgnoreKinds(InternalKeyKindRangeDelete))
		if merge == nil {
			merge = func(key, value []byte) (base.ValueMerger, error) {
				m := &debugMerger{}
				m.buf = append(m.buf, value...)
				return m, nil
			}
		}
		resetSingleDelStats()
		return newCompactionIter(
			DefaultComparer.Compare,
			DefaultComparer.Equal,
			DefaultComparer.FormatKey,
			merge,
			iter,
			snapshots,
			&keyspan.Fragmenter{},
			&keyspan.Fragmenter{},
			allowZeroSeqnum,
			func([]byte) bool {
				return elideTombstones
			},
			func(_, _ []byte) bool {
				return elideTombstones
			},
			func(userKey []byte) {
				ineffectualSingleDeleteKeys = append(ineffectualSingleDeleteKeys, string(userKey))
			},
			func(userKey []byte) {
				invariantViolationSingleDeleteKeys = append(invariantViolationSingleDeleteKeys, string(userKey))
			},
			formatVersion,
		)
	}

	runTest := func(t *testing.T, formatVersion FormatMajorVersion) {
		datadriven.RunTest(t, fileFunc(formatVersion), func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				merge = nil
				if len(d.CmdArgs) > 0 && d.CmdArgs[0].Key == "merger" &&
					len(d.CmdArgs[0].Vals) > 0 && d.CmdArgs[0].Vals[0] == "deletable" {
					merge = newDeletableSumValueMerger
				}
				keys = keys[:0]
				vals = vals[:0]
				rangeKeys = rangeKeys[:0]
				for _, key := range strings.Split(d.Input, "\n") {
					j := strings.Index(key, ":")
					keys = append(keys, base.ParseInternalKey(key[:j]))

					if strings.HasPrefix(key[j+1:], "varint(") {
						valueStr := strings.TrimSuffix(strings.TrimPrefix(key[j+1:], "varint("), ")")
						v, err := strconv.ParseUint(valueStr, 10, 64)
						require.NoError(t, err)
						encodedValue := binary.AppendUvarint([]byte(nil), v)
						vals = append(vals, encodedValue)
					} else {
						vals = append(vals, []byte(key[j+1:]))
					}
				}
				return ""

			case "define-range-keys":
				for _, key := range strings.Split(d.Input, "\n") {
					s := keyspan.ParseSpan(strings.TrimSpace(key))
					rangeKeys = append(rangeKeys, s)
				}
				return ""

			case "iter":
				snapshots = snapshots[:0]
				elideTombstones = false
				allowZeroSeqnum = false
				printSnapshotPinned := false
				printMissizedDels := false
				printForceObsolete := false
				for _, arg := range d.CmdArgs {
					switch arg.Key {
					case "snapshots":
						for _, val := range arg.Vals {
							seqNum, err := strconv.Atoi(val)
							if err != nil {
								return err.Error()
							}
							snapshots = append(snapshots, uint64(seqNum))
						}
					case "elide-tombstones":
						var err error
						elideTombstones, err = strconv.ParseBool(arg.Vals[0])
						if err != nil {
							return err.Error()
						}
					case "allow-zero-seqnum":
						var err error
						allowZeroSeqnum, err = strconv.ParseBool(arg.Vals[0])
						if err != nil {
							return err.Error()
						}
					case "print-snapshot-pinned":
						printSnapshotPinned = true
					case "print-missized-dels":
						printMissizedDels = true
					case "print-force-obsolete":
						printForceObsolete = true
					default:
						return fmt.Sprintf("%s: unknown arg: %s", d.Cmd, arg.Key)
					}
				}
				sort.Slice(snapshots, func(i, j int) bool {
					return snapshots[i] < snapshots[j]
				})

				iter := newIter(formatVersion)
				var b bytes.Buffer
				for _, line := range strings.Split(d.Input, "\n") {
					parts := strings.Fields(line)
					if len(parts) == 0 {
						continue
					}
					switch parts[0] {
					case "first":
						iter.First()
					case "next":
						iter.Next()
					case "tombstones":
						var key []byte
						if len(parts) == 2 {
							key = []byte(parts[1])
						}
						for _, v := range iter.Tombstones(key) {
							for _, k := range v.Keys {
								fmt.Fprintf(&b, "%s-%s#%d\n", v.Start, v.End, k.SeqNum())
							}
						}
						fmt.Fprintf(&b, ".\n")
						continue
					case "range-keys":
						var key []byte
						if len(parts) == 2 {
							key = []byte(parts[1])
						}
						for _, v := range iter.RangeKeys(key) {
							fmt.Fprintf(&b, "%s\n", v)
						}
						fmt.Fprintf(&b, ".\n")
						continue
					default:
						return fmt.Sprintf("unknown op: %s", parts[0])
					}
					if iter.Valid() {
						snapshotPinned := ""
						if printSnapshotPinned {
							snapshotPinned = " (not pinned)"
							if iter.snapshotPinned {
								snapshotPinned = " (pinned)"
							}
						}
						forceObsolete := ""
						if printForceObsolete {
							forceObsolete = " (not force obsolete)"
							if iter.forceObsoleteDueToRangeDel {
								forceObsolete = " (force obsolete)"
							}
						}
						v := string(iter.Value())
						if iter.Key().Kind() == base.InternalKeyKindDeleteSized && len(iter.Value()) > 0 {
							vn, n := binary.Uvarint(iter.Value())
							if n != len(iter.Value()) {
								v = fmt.Sprintf("err: %0x value not a uvarint", iter.Value())
							} else {
								v = fmt.Sprintf("varint(%d)", vn)
							}
						}
						fmt.Fprintf(&b, "%s:%s%s%s\n", iter.Key(), v, snapshotPinned, forceObsolete)
						if iter.Key().Kind() == InternalKeyKindRangeDelete {
							iter.rangeDelFrag.Add(keyspan.Span{
								Start: append([]byte{}, iter.Key().UserKey...),
								End:   append([]byte{}, iter.Value()...),
								Keys: []keyspan.Key{
									{Trailer: iter.Key().Trailer},
								},
							})
						}
						if rangekey.IsRangeKey(iter.Key().Kind()) {
							iter.rangeKeyFrag.Add(*interleavingIter.Span())
						}
					} else if err := iter.Error(); err != nil {
						fmt.Fprintf(&b, "err=%v\n", err)
					} else {
						fmt.Fprintf(&b, ".\n")
					}
				}
				if printMissizedDels {
					fmt.Fprintf(&b, "missized-dels=%d\n", iter.stats.countMissizedDels)
				}
				if len(ineffectualSingleDeleteKeys) > 0 {
					fmt.Fprintf(&b, "ineffectual-single-deletes: %s\n",
						strings.Join(ineffectualSingleDeleteKeys, ","))
				}
				if len(invariantViolationSingleDeleteKeys) > 0 {
					fmt.Fprintf(&b, "invariant-violation-single-deletes: %s\n",
						strings.Join(invariantViolationSingleDeleteKeys, ","))
				}
				return b.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
	}

	// Rather than testing against all format version, we test against the
	// significant boundaries.
	formatVersions := []FormatMajorVersion{
		FormatMostCompatible,
		FormatSetWithDelete - 1,
		FormatSetWithDelete,
		internalFormatNewest,
	}
	for _, formatVersion := range formatVersions {
		t.Run(fmt.Sprintf("version-%s", formatVersion), func(t *testing.T) {
			runTest(t, formatVersion)
		})
	}
}

func TestFrontiers(t *testing.T) {
	cmp := testkeys.Comparer.Compare
	var keySets [][][]byte
	datadriven.RunTest(t, "testdata/frontiers", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "init":
			// Init configures a frontier per line of input. Each line should
			// contain a sorted whitespace-separated list of keys that the
			// frontier will use.
			//
			// For example, the following input creates two separate monitored
			// frontiers: one that sets its key successively to 'd', 'e', 'j'
			// and one that sets its key to 'a', 'p', 'n', 'z':
			//
			//    init
			//    b e j
			//    a p n z

			keySets = keySets[:0]
			for _, line := range strings.Split(td.Input, "\n") {
				keySets = append(keySets, bytes.Fields([]byte(line)))
			}
			return ""
		case "scan":
			f := &frontiers{cmp: cmp}
			for _, keys := range keySets {
				initTestFrontier(f, keys...)
			}
			var buf bytes.Buffer
			for _, kStr := range strings.Fields(td.Input) {
				k := []byte(kStr)
				f.Advance(k)
				fmt.Fprintf(&buf, "%s : { %s }\n", kStr, f.String())
			}
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

// initTestFrontiers adds a new frontier to f that iterates through the provided
// keys. The keys slice must be sorted.
func initTestFrontier(f *frontiers, keys ...[]byte) *frontier {
	ff := &frontier{}
	var key []byte
	if len(keys) > 0 {
		key, keys = keys[0], keys[1:]
	}
	reached := func(k []byte) (nextKey []byte) {
		if len(keys) > 0 {
			nextKey, keys = keys[0], keys[1:]
		}
		return nextKey
	}
	ff.Init(f, key, reached)
	return ff
}
