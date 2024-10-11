// Copyright 2013 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"context"
	crand "crypto/rand"
	"fmt"
	"math"
	"math/rand"
	"path/filepath"
	"reflect"
	"regexp"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/errorfs"
	"github.com/stretchr/testify/require"
)

func newVersion(opts *Options, files [numLevels][]*fileMetadata) *version {
	return manifest.NewVersion(
		opts.Comparer.Compare,
		opts.Comparer.FormatKey,
		opts.FlushSplitBytes,
		files)
}

type compactionPickerForTesting struct {
	score         float64
	level         int
	baseLevel     int
	opts          *Options
	vers          *manifest.Version
	maxLevelBytes [7]int64
}

var _ compactionPicker = &compactionPickerForTesting{}

func (p *compactionPickerForTesting) getScores([]compactionInfo) [numLevels]float64 {
	return [numLevels]float64{}
}

func (p *compactionPickerForTesting) getBaseLevel() int {
	return p.baseLevel
}

func (p *compactionPickerForTesting) estimatedCompactionDebt(l0ExtraSize uint64) uint64 {
	return 0
}

func (p *compactionPickerForTesting) forceBaseLevel1() {}

func (p *compactionPickerForTesting) pickAuto(env compactionEnv) (pc *pickedCompaction) {
	if p.score < 1 {
		return nil
	}
	outputLevel := p.level + 1
	if p.level == 0 {
		outputLevel = p.baseLevel
	}
	iter := p.vers.Levels[p.level].Iter()
	iter.First()
	cInfo := candidateLevelInfo{
		level:       p.level,
		outputLevel: outputLevel,
		file:        iter.Take(),
	}
	if cInfo.level == 0 {
		return pickL0(env, p.opts, p.vers, p.baseLevel)
	}
	return pickAutoLPositive(env, p.opts, p.vers, cInfo, p.baseLevel, p.maxLevelBytes)
}

func (p *compactionPickerForTesting) pickElisionOnlyCompaction(
	env compactionEnv,
) (pc *pickedCompaction) {
	return nil
}

func (p *compactionPickerForTesting) pickRewriteCompaction(
	env compactionEnv,
) (pc *pickedCompaction) {
	return nil
}

func (p *compactionPickerForTesting) pickReadTriggeredCompaction(
	env compactionEnv,
) (pc *pickedCompaction) {
	return nil
}

func TestPickCompaction(t *testing.T) {
	fileNums := func(files manifest.LevelSlice) string {
		var ss []string
		files.Each(func(meta *fileMetadata) {
			ss = append(ss, strconv.Itoa(int(meta.FileNum)))
		})
		sort.Strings(ss)
		return strings.Join(ss, ",")
	}

	opts := (*Options)(nil).EnsureDefaults()
	newFileMeta := func(fileNum FileNum, size uint64, smallest, largest base.InternalKey) *fileMetadata {
		m := (&fileMetadata{
			FileNum: fileNum,
			Size:    size,
		}).ExtendPointKeyBounds(opts.Comparer.Compare, smallest, largest)
		m.InitPhysicalBacking()
		return m
	}

	testCases := []struct {
		desc      string
		version   *version
		picker    compactionPickerForTesting
		want      string
		wantMulti bool
	}{
		{
			desc: "no compaction",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("j.SET.102"),
					),
				},
			}),
			want: "",
		},

		{
			desc: "1 L0 file",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("j.SET.102"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100  ",
		},

		{
			desc: "2 L0 files (0 overlaps)",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("j.SET.102"),
					),
					newFileMeta(
						110,
						1,
						base.ParseInternalKey("k.SET.111"),
						base.ParseInternalKey("l.SET.112"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100,110  ",
		},

		{
			desc: "2 L0 files, with ikey overlap",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("p.SET.102"),
					),
					newFileMeta(
						110,
						1,
						base.ParseInternalKey("j.SET.111"),
						base.ParseInternalKey("q.SET.112"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100,110  ",
		},

		{
			desc: "2 L0 files, with ukey overlap",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("i.SET.102"),
					),
					newFileMeta(
						110,
						1,
						base.ParseInternalKey("i.SET.111"),
						base.ParseInternalKey("i.SET.112"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100,110  ",
		},

		{
			desc: "1 L0 file, 2 L1 files (0 overlaps)",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("i.SET.102"),
					),
				},
				1: {
					newFileMeta(
						200,
						1,
						base.ParseInternalKey("a.SET.201"),
						base.ParseInternalKey("b.SET.202"),
					),
					newFileMeta(
						210,
						1,
						base.ParseInternalKey("y.SET.211"),
						base.ParseInternalKey("z.SET.212"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100  ",
		},

		{
			desc: "1 L0 file, 2 L1 files (1 overlap), 4 L2 files (3 overlaps)",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						100,
						1,
						base.ParseInternalKey("i.SET.101"),
						base.ParseInternalKey("t.SET.102"),
					),
				},
				1: {
					newFileMeta(
						200,
						1,
						base.ParseInternalKey("a.SET.201"),
						base.ParseInternalKey("e.SET.202"),
					),
					newFileMeta(
						210,
						1,
						base.ParseInternalKey("f.SET.211"),
						base.ParseInternalKey("j.SET.212"),
					),
				},
				2: {
					newFileMeta(
						300,
						1,
						base.ParseInternalKey("a.SET.301"),
						base.ParseInternalKey("b.SET.302"),
					),
					newFileMeta(
						310,
						1,
						base.ParseInternalKey("c.SET.311"),
						base.ParseInternalKey("g.SET.312"),
					),
					newFileMeta(
						320,
						1,
						base.ParseInternalKey("h.SET.321"),
						base.ParseInternalKey("m.SET.322"),
					),
					newFileMeta(
						330,
						1,
						base.ParseInternalKey("n.SET.331"),
						base.ParseInternalKey("z.SET.332"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     0,
				baseLevel: 1,
			},
			want: "100 210 310,320,330",
		},

		{
			desc: "4 L1 files, 2 L2 files, can grow",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						200,
						1,
						base.ParseInternalKey("i1.SET.201"),
						base.ParseInternalKey("i2.SET.202"),
					),
					newFileMeta(
						210,
						1,
						base.ParseInternalKey("j1.SET.211"),
						base.ParseInternalKey("j2.SET.212"),
					),
					newFileMeta(
						220,
						1,
						base.ParseInternalKey("k1.SET.221"),
						base.ParseInternalKey("k2.SET.222"),
					),
					newFileMeta(
						230,
						1,
						base.ParseInternalKey("l1.SET.231"),
						base.ParseInternalKey("l2.SET.232"),
					),
				},
				2: {
					newFileMeta(
						300,
						1,
						base.ParseInternalKey("a0.SET.301"),
						base.ParseInternalKey("l0.SET.302"),
					),
					newFileMeta(
						310,
						1,
						base.ParseInternalKey("l2.SET.311"),
						base.ParseInternalKey("z2.SET.312"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     1,
				baseLevel: 1,
			},
			want:      "200,210,220 300  ",
			wantMulti: true,
		},

		{
			desc: "4 L1 files, 2 L2 files, can't grow (range)",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						200,
						1,
						base.ParseInternalKey("i1.SET.201"),
						base.ParseInternalKey("i2.SET.202"),
					),
					newFileMeta(
						210,
						1,
						base.ParseInternalKey("j1.SET.211"),
						base.ParseInternalKey("j2.SET.212"),
					),
					newFileMeta(
						220,
						1,
						base.ParseInternalKey("k1.SET.221"),
						base.ParseInternalKey("k2.SET.222"),
					),
					newFileMeta(
						230,
						1,
						base.ParseInternalKey("l1.SET.231"),
						base.ParseInternalKey("l2.SET.232"),
					),
				},
				2: {
					newFileMeta(
						300,
						1,
						base.ParseInternalKey("a0.SET.301"),
						base.ParseInternalKey("j0.SET.302"),
					),
					newFileMeta(
						310,
						1,
						base.ParseInternalKey("j2.SET.311"),
						base.ParseInternalKey("z2.SET.312"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     1,
				baseLevel: 1,
			},
			want:      "200 300  ",
			wantMulti: true,
		},

		{
			desc: "4 L1 files, 2 L2 files, can't grow (size)",
			version: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						200,
						expandedCompactionByteSizeLimit(opts, 1, math.MaxUint64)-1,
						base.ParseInternalKey("i1.SET.201"),
						base.ParseInternalKey("i2.SET.202"),
					),
					newFileMeta(
						210,
						expandedCompactionByteSizeLimit(opts, 1, math.MaxUint64)-1,
						base.ParseInternalKey("j1.SET.211"),
						base.ParseInternalKey("j2.SET.212"),
					),
					newFileMeta(
						220,
						expandedCompactionByteSizeLimit(opts, 1, math.MaxUint64)-1,
						base.ParseInternalKey("k1.SET.221"),
						base.ParseInternalKey("k2.SET.222"),
					),
					newFileMeta(
						230,
						expandedCompactionByteSizeLimit(opts, 1, math.MaxUint64)-1,
						base.ParseInternalKey("l1.SET.231"),
						base.ParseInternalKey("l2.SET.232"),
					),
				},
				2: {
					newFileMeta(
						300,
						expandedCompactionByteSizeLimit(opts, 2, math.MaxUint64)-1,
						base.ParseInternalKey("a0.SET.301"),
						base.ParseInternalKey("l0.SET.302"),
					),
					newFileMeta(
						310,
						expandedCompactionByteSizeLimit(opts, 2, math.MaxUint64)-1,
						base.ParseInternalKey("l2.SET.311"),
						base.ParseInternalKey("z2.SET.312"),
					),
				},
			}),
			picker: compactionPickerForTesting{
				score:     99,
				level:     1,
				baseLevel: 1,
			},
			want: "200 300 ",
		},
	}

	for _, tc := range testCases {
		vs := &versionSet{
			opts:    opts,
			cmp:     DefaultComparer.Compare,
			cmpName: DefaultComparer.Name,
		}
		vs.versions.Init(nil)
		vs.append(tc.version)
		tc.picker.opts = opts
		tc.picker.vers = tc.version
		vs.picker = &tc.picker
		pc, got := vs.picker.pickAuto(compactionEnv{diskAvailBytes: math.MaxUint64}), ""
		if pc != nil {
			c := newCompaction(pc, opts, time.Now(), nil /* provider */)

			gotStart := fileNums(c.startLevel.files)
			gotML := ""
			observedMulti := len(c.extraLevels) > 0
			if observedMulti {
				gotML = " " + fileNums(c.extraLevels[0].files)
			}
			gotOutput := " " + fileNums(c.outputLevel.files)
			gotGrandparents := " " + fileNums(c.grandparents)
			got = gotStart + gotML + gotOutput + gotGrandparents
			if tc.wantMulti != observedMulti {
				t.Fatalf("Expected Multi %t; Observed Multi %t, for %s", tc.wantMulti, observedMulti, got)
			}

		}
		if got != tc.want {
			t.Fatalf("%s:\ngot  %q\nwant %q", tc.desc, got, tc.want)
		}
	}
}

func TestElideTombstone(t *testing.T) {
	opts := (&Options{}).EnsureDefaults().WithFSDefaults()

	newFileMeta := func(smallest, largest base.InternalKey) *fileMetadata {
		m := (&fileMetadata{}).ExtendPointKeyBounds(opts.Comparer.Compare, smallest, largest)
		m.InitPhysicalBacking()
		return m
	}

	type want struct {
		key      string
		expected bool
	}

	testCases := []struct {
		desc    string
		level   int
		version *version
		wants   []want
	}{
		{
			desc:    "empty",
			level:   1,
			version: newVersion(opts, [numLevels][]*fileMetadata{}),
			wants: []want{
				{"x", true},
			},
		},
		{
			desc:  "non-empty",
			level: 1,
			version: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						base.ParseInternalKey("c.SET.801"),
						base.ParseInternalKey("g.SET.800"),
					),
					newFileMeta(
						base.ParseInternalKey("x.SET.701"),
						base.ParseInternalKey("y.SET.700"),
					),
				},
				2: {
					newFileMeta(
						base.ParseInternalKey("d.SET.601"),
						base.ParseInternalKey("h.SET.600"),
					),
					newFileMeta(
						base.ParseInternalKey("r.SET.501"),
						base.ParseInternalKey("t.SET.500"),
					),
				},
				3: {
					newFileMeta(
						base.ParseInternalKey("f.SET.401"),
						base.ParseInternalKey("g.SET.400"),
					),
					newFileMeta(
						base.ParseInternalKey("w.SET.301"),
						base.ParseInternalKey("x.SET.300"),
					),
				},
				4: {
					newFileMeta(
						base.ParseInternalKey("f.SET.201"),
						base.ParseInternalKey("m.SET.200"),
					),
					newFileMeta(
						base.ParseInternalKey("t.SET.101"),
						base.ParseInternalKey("t.SET.100"),
					),
				},
			}),
			wants: []want{
				{"b", true},
				{"c", true},
				{"d", true},
				{"e", true},
				{"f", false},
				{"g", false},
				{"h", false},
				{"l", false},
				{"m", false},
				{"n", true},
				{"q", true},
				{"r", true},
				{"s", true},
				{"t", false},
				{"u", true},
				{"v", true},
				{"w", false},
				{"x", false},
				{"y", true},
				{"z", true},
			},
		},
		{
			desc:  "repeated ukey",
			level: 1,
			version: newVersion(opts, [numLevels][]*fileMetadata{
				6: {
					newFileMeta(
						base.ParseInternalKey("i.SET.401"),
						base.ParseInternalKey("i.SET.400"),
					),
					newFileMeta(
						base.ParseInternalKey("i.SET.301"),
						base.ParseInternalKey("k.SET.300"),
					),
					newFileMeta(
						base.ParseInternalKey("k.SET.201"),
						base.ParseInternalKey("m.SET.200"),
					),
					newFileMeta(
						base.ParseInternalKey("m.SET.101"),
						base.ParseInternalKey("m.SET.100"),
					),
				},
			}),
			wants: []want{
				{"h", true},
				{"i", false},
				{"j", false},
				{"k", false},
				{"l", false},
				{"m", false},
				{"n", true},
			},
		},
	}

	for _, tc := range testCases {
		c := compaction{
			cmp:      DefaultComparer.Compare,
			comparer: DefaultComparer,
			version:  tc.version,
			inputs:   []compactionLevel{{level: tc.level}, {level: tc.level + 1}},
			smallest: base.ParseInternalKey("a.SET.0"),
			largest:  base.ParseInternalKey("z.SET.0"),
		}
		c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
		c.setupInuseKeyRanges()
		for _, w := range tc.wants {
			if got := c.elideTombstone([]byte(w.key)); got != w.expected {
				t.Errorf("%s: ukey=%q: got %v, want %v", tc.desc, w.key, got, w.expected)
			}
		}
	}
}

func TestElideRangeTombstone(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()

	newFileMeta := func(smallest, largest base.InternalKey) *fileMetadata {
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			opts.Comparer.Compare, smallest, largest,
		)
		m.InitPhysicalBacking()
		return m
	}

	type want struct {
		key      string
		endKey   string
		expected bool
	}

	testCases := []struct {
		desc     string
		level    int
		version  *version
		wants    []want
		flushing flushableList
	}{
		{
			desc:    "empty",
			level:   1,
			version: newVersion(opts, [numLevels][]*fileMetadata{}),
			wants: []want{
				{"x", "y", true},
			},
		},
		{
			desc:  "non-empty",
			level: 1,
			version: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						base.ParseInternalKey("c.SET.801"),
						base.ParseInternalKey("g.SET.800"),
					),
					newFileMeta(
						base.ParseInternalKey("x.SET.701"),
						base.ParseInternalKey("y.SET.700"),
					),
				},
				2: {
					newFileMeta(
						base.ParseInternalKey("d.SET.601"),
						base.ParseInternalKey("h.SET.600"),
					),
					newFileMeta(
						base.ParseInternalKey("r.SET.501"),
						base.ParseInternalKey("t.SET.500"),
					),
				},
				3: {
					newFileMeta(
						base.ParseInternalKey("f.SET.401"),
						base.ParseInternalKey("g.SET.400"),
					),
					newFileMeta(
						base.ParseInternalKey("w.SET.301"),
						base.ParseInternalKey("x.SET.300"),
					),
				},
				4: {
					newFileMeta(
						base.ParseInternalKey("f.SET.201"),
						base.ParseInternalKey("m.SET.200"),
					),
					newFileMeta(
						base.ParseInternalKey("t.SET.101"),
						base.ParseInternalKey("t.SET.100"),
					),
				},
			}),
			wants: []want{
				{"b", "c", true},
				{"c", "d", true},
				{"d", "e", true},
				{"e", "f", false},
				{"f", "g", false},
				{"g", "h", false},
				{"h", "i", false},
				{"l", "m", false},
				{"m", "n", false},
				{"n", "o", true},
				{"q", "r", true},
				{"r", "s", true},
				{"s", "t", false},
				{"t", "u", false},
				{"u", "v", true},
				{"v", "w", false},
				{"w", "x", false},
				{"x", "y", false},
				{"y", "z", true},
			},
		},
		{
			desc:  "flushing",
			level: -1,
			version: newVersion(opts, [numLevels][]*fileMetadata{
				0: {
					newFileMeta(
						base.ParseInternalKey("h.SET.901"),
						base.ParseInternalKey("j.SET.900"),
					),
				},
				1: {
					newFileMeta(
						base.ParseInternalKey("c.SET.801"),
						base.ParseInternalKey("g.SET.800"),
					),
					newFileMeta(
						base.ParseInternalKey("x.SET.701"),
						base.ParseInternalKey("y.SET.700"),
					),
				},
			}),
			wants: []want{
				{"m", "n", false},
			},
			// Pretend one memtable is being flushed
			flushing: flushableList{nil},
		},
	}

	for _, tc := range testCases {
		c := compaction{
			cmp:      DefaultComparer.Compare,
			comparer: DefaultComparer,
			version:  tc.version,
			inputs:   []compactionLevel{{level: tc.level}, {level: tc.level + 1}},
			smallest: base.ParseInternalKey("a.SET.0"),
			largest:  base.ParseInternalKey("z.SET.0"),
			flushing: tc.flushing,
		}
		c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
		c.setupInuseKeyRanges()
		for _, w := range tc.wants {
			if got := c.elideRangeTombstone([]byte(w.key), []byte(w.endKey)); got != w.expected {
				t.Errorf("%s: keys=%q-%q: got %v, want %v", tc.desc, w.key, w.endKey, got, w.expected)
			}
		}
	}
}

func TestCompactionTransform(t *testing.T) {
	datadriven.RunTest(t, "testdata/compaction_transform", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "transform":
			var snapshots []uint64
			var keyRanges []manifest.UserKeyRange
			disableElision := td.HasArg("disable-elision")
			td.MaybeScanArgs(t, "snapshots", &snapshots)
			if arg, ok := td.Arg("in-use-key-ranges"); ok {
				for _, keyRange := range arg.Vals {
					parts := strings.SplitN(keyRange, "-", 2)
					start := []byte(strings.TrimSpace(parts[0]))
					end := []byte(strings.TrimSpace(parts[1]))
					keyRanges = append(keyRanges, manifest.UserKeyRange{
						Start: start,
						End:   end,
					})
				}
			}
			span := keyspan.ParseSpan(td.Input)
			for i := range span.Keys {
				if i > 0 {
					if span.Keys[i-1].Trailer < span.Keys[i].Trailer {
						return "span keys not sorted"
					}
				}
			}
			var outSpan keyspan.Span
			c := compaction{
				cmp:                base.DefaultComparer.Compare,
				comparer:           base.DefaultComparer,
				disableSpanElision: disableElision,
				inuseKeyRanges:     keyRanges,
			}
			transformer := rangeKeyCompactionTransform(base.DefaultComparer.Equal, snapshots, c.elideRangeTombstone)
			if err := transformer.Transform(base.DefaultComparer.Compare, span, &outSpan); err != nil {
				return fmt.Sprintf("error: %s", err)
			}
			return outSpan.String()
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

type cpuPermissionGranter struct {
	// requestCount is used to confirm that every GetPermission function call
	// has a corresponding CPUWorkDone function call.
	requestCount int
	used         bool
	permit       bool
}

type cpuWorkHandle struct {
	permit bool
}

func (c cpuWorkHandle) Permitted() bool {
	return c.permit
}

func (t *cpuPermissionGranter) GetPermission(dur time.Duration) CPUWorkHandle {
	t.requestCount++
	t.used = true
	return cpuWorkHandle{t.permit}
}

func (t *cpuPermissionGranter) CPUWorkDone(_ CPUWorkHandle) {
	t.requestCount--
}

// Simple test to check if compactions are using the granter, and if exactly
// the acquired handles are returned.
func TestCompactionCPUGranter(t *testing.T) {
	mem := vfs.NewMem()
	opts := (&Options{FS: mem}).WithFSDefaults()
	g := &cpuPermissionGranter{permit: true}
	opts.Experimental.CPUWorkPermissionGranter = g
	d, err := Open("", opts)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer d.Close()

	d.Set([]byte{'a'}, []byte{'a'}, nil)
	err = d.Compact([]byte{'a'}, []byte{'b'}, true)
	if err != nil {
		t.Fatalf("Compact: %v", err)
	}
	require.True(t, g.used)
	require.Equal(t, g.requestCount, 0)
}

// Tests that there's no errors or panics when the default CPU granter is used.
func TestCompactionCPUGranterDefault(t *testing.T) {
	mem := vfs.NewMem()
	opts := (&Options{FS: mem}).WithFSDefaults()
	d, err := Open("", opts)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	defer d.Close()

	d.Set([]byte{'a'}, []byte{'a'}, nil)
	err = d.Compact([]byte{'a'}, []byte{'b'}, true)
	if err != nil {
		t.Fatalf("Compact: %v", err)
	}
}

func TestCompaction(t *testing.T) {
	const memTableSize = 10000
	// Tuned so that 2 values can reside in the memtable before a flush, but a
	// 3rd value will cause a flush. Needs to account for the max skiplist node
	// size.
	const valueSize = 3500

	mem := vfs.NewMem()
	opts := &Options{
		FS:                    mem,
		MemTableSize:          memTableSize,
		DebugCheck:            DebugCheckLevels,
		L0CompactionThreshold: 8,
	}
	opts.testingRandomized(t).WithFSDefaults()
	d, err := Open("", opts)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}

	get1 := func(iter internalIterator) (ret string) {
		b := &bytes.Buffer{}
		for key, _ := iter.First(); key != nil; key, _ = iter.Next() {
			b.Write(key.UserKey)
		}
		if err := iter.Close(); err != nil {
			t.Fatalf("iterator Close: %v", err)
		}
		return b.String()
	}
	getAll := func() (gotMem, gotDisk string, err error) {
		d.mu.Lock()
		defer d.mu.Unlock()

		if d.mu.mem.mutable != nil {
			gotMem = get1(d.mu.mem.mutable.newIter(nil))
		}
		ss := []string(nil)
		v := d.mu.versions.currentVersion()
		provider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(mem, "" /* dirName */))
		if err != nil {
			t.Fatalf("%v", err)
		}
		defer provider.Close()
		for _, levelMetadata := range v.Levels {
			iter := levelMetadata.Iter()
			for meta := iter.First(); meta != nil; meta = iter.Next() {
				if meta.Virtual {
					continue
				}
				f, err := provider.OpenForReading(context.Background(), base.FileTypeTable, meta.FileBacking.DiskFileNum, objstorage.OpenOptions{})
				if err != nil {
					return "", "", errors.WithStack(err)
				}
				r, err := sstable.NewReader(f, sstable.ReaderOptions{})
				if err != nil {
					return "", "", errors.WithStack(err)
				}
				defer r.Close()
				iter, err := r.NewIter(nil /* lower */, nil /* upper */)
				if err != nil {
					return "", "", errors.WithStack(err)
				}
				ss = append(ss, get1(iter)+".")
			}
		}
		sort.Strings(ss)
		return gotMem, strings.Join(ss, ""), nil
	}

	value := bytes.Repeat([]byte("x"), valueSize)
	testCases := []struct {
		key, wantMem, wantDisk string
	}{
		{"+A", "A", ""},
		{"+a", "Aa", ""},
		{"+B", "B", "Aa."},
		{"+b", "Bb", "Aa."},
		// The next level-0 table overwrites the B key.
		{"+C", "C", "Aa.Bb."},
		{"+B", "BC", "Aa.Bb."},
		// The next level-0 table deletes the a key.
		{"+D", "D", "Aa.BC.Bb."},
		{"-a", "Da", "Aa.BC.Bb."},
		{"+d", "Dad", "Aa.BC.Bb."},
		{"+E", "E", "Aa.BC.Bb.Dad."},
		{"+e", "Ee", "Aa.BC.Bb.Dad."},
		// The next addition creates the fourth level-0 table, and l0CompactionTrigger == 8,
		// but since the sublevel count is doubled when comparing with l0CompactionTrigger,
		// the addition of the 4th sublevel triggers a non-trivial compaction into one level-1 table.
		// Note that the keys in this one larger table are interleaved from the four smaller ones.
		{"+F", "F", "ABCDEbde."},
	}
	for _, tc := range testCases {
		if key := tc.key[1:]; tc.key[0] == '+' {
			if err := d.Set([]byte(key), value, nil); err != nil {
				t.Errorf("%q: Set: %v", key, err)
				break
			}
		} else {
			if err := d.Delete([]byte(key), nil); err != nil {
				t.Errorf("%q: Delete: %v", key, err)
				break
			}
		}

		// try backs off to allow any writes to the memfs to complete.
		err := try(100*time.Microsecond, 20*time.Second, func() error {
			gotMem, gotDisk, err := getAll()
			if err != nil {
				return err
			}
			if testing.Verbose() {
				fmt.Printf("mem=%s (%s) disk=%s (%s)\n", gotMem, tc.wantMem, gotDisk, tc.wantDisk)
			}

			if gotMem != tc.wantMem {
				return errors.Errorf("mem: got %q, want %q", gotMem, tc.wantMem)
			}
			if gotDisk != tc.wantDisk {
				return errors.Errorf("ldb: got %q, want %q", gotDisk, tc.wantDisk)
			}
			return nil
		})
		if err != nil {
			t.Errorf("%q: %v", tc.key, err)
		}
	}
	if err := d.Close(); err != nil {
		t.Fatalf("db Close: %v", err)
	}
}

func TestValidateVersionEdit(t *testing.T) {
	const badKey = "malformed-key"

	errValidationFailed := errors.New("validation failed")
	validateFn := func(key []byte) error {
		if string(key) == badKey {
			return errValidationFailed
		}
		return nil
	}

	cmp := DefaultComparer.Compare
	newFileMeta := func(smallest, largest base.InternalKey) *fileMetadata {
		m := (&fileMetadata{}).ExtendPointKeyBounds(cmp, smallest, largest)
		m.InitPhysicalBacking()
		return m
	}

	testCases := []struct {
		desc    string
		ve      *versionEdit
		vFunc   func([]byte) error
		wantErr error
	}{
		{
			desc: "single new file; start key",
			ve: &versionEdit{
				NewFiles: []manifest.NewFileEntry{
					{
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte(badKey)},
							manifest.InternalKey{UserKey: []byte("z")},
						),
					},
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "single new file; end key",
			ve: &versionEdit{
				NewFiles: []manifest.NewFileEntry{
					{
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte("a")},
							manifest.InternalKey{UserKey: []byte(badKey)},
						),
					},
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "multiple new files",
			ve: &versionEdit{
				NewFiles: []manifest.NewFileEntry{
					{
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte("a")},
							manifest.InternalKey{UserKey: []byte("c")},
						),
					},
					{
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte(badKey)},
							manifest.InternalKey{UserKey: []byte("z")},
						),
					},
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "single deleted file; start key",
			ve: &versionEdit{
				DeletedFiles: map[manifest.DeletedFileEntry]*manifest.FileMetadata{
					deletedFileEntry{Level: 0, FileNum: 0}: newFileMeta(
						manifest.InternalKey{UserKey: []byte(badKey)},
						manifest.InternalKey{UserKey: []byte("z")},
					),
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "single deleted file; end key",
			ve: &versionEdit{
				DeletedFiles: map[manifest.DeletedFileEntry]*manifest.FileMetadata{
					deletedFileEntry{Level: 0, FileNum: 0}: newFileMeta(
						manifest.InternalKey{UserKey: []byte("a")},
						manifest.InternalKey{UserKey: []byte(badKey)},
					),
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "multiple deleted files",
			ve: &versionEdit{
				DeletedFiles: map[manifest.DeletedFileEntry]*manifest.FileMetadata{
					deletedFileEntry{Level: 0, FileNum: 0}: newFileMeta(
						manifest.InternalKey{UserKey: []byte("a")},
						manifest.InternalKey{UserKey: []byte("c")},
					),
					deletedFileEntry{Level: 0, FileNum: 1}: newFileMeta(
						manifest.InternalKey{UserKey: []byte(badKey)},
						manifest.InternalKey{UserKey: []byte("z")},
					),
				},
			},
			vFunc:   validateFn,
			wantErr: errValidationFailed,
		},
		{
			desc: "no errors",
			ve: &versionEdit{
				NewFiles: []manifest.NewFileEntry{
					{
						Level: 0,
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte("b")},
							manifest.InternalKey{UserKey: []byte("c")},
						),
					},
					{
						Level: 0,
						Meta: newFileMeta(
							manifest.InternalKey{UserKey: []byte("d")},
							manifest.InternalKey{UserKey: []byte("g")},
						),
					},
				},
				DeletedFiles: map[manifest.DeletedFileEntry]*manifest.FileMetadata{
					deletedFileEntry{Level: 6, FileNum: 0}: newFileMeta(
						manifest.InternalKey{UserKey: []byte("a")},
						manifest.InternalKey{UserKey: []byte("d")},
					),
					deletedFileEntry{Level: 6, FileNum: 1}: newFileMeta(
						manifest.InternalKey{UserKey: []byte("x")},
						manifest.InternalKey{UserKey: []byte("z")},
					),
				},
			},
			vFunc: validateFn,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.desc, func(t *testing.T) {
			err := validateVersionEdit(tc.ve, tc.vFunc, base.DefaultFormatter)
			if tc.wantErr != nil {
				if !errors.Is(err, tc.wantErr) {
					t.Fatalf("got: %s; want: %s", err, tc.wantErr)
				}
				return
			}
			if err != nil {
				t.Fatalf("got %s; wanted no error", err)
			}
		})
	}
}

func TestManualCompaction(t *testing.T) {
	var mem vfs.FS
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
	}()

	seed := time.Now().UnixNano()
	rng := rand.New(rand.NewSource(seed))
	t.Logf("seed: %d", seed)

	randVersion := func(min, max FormatMajorVersion) FormatMajorVersion {
		return FormatMajorVersion(int(min) + rng.Intn(int(max)-int(min)+1))
	}

	var compactionLog bytes.Buffer
	compactionLogEventListener := &EventListener{
		CompactionEnd: func(info CompactionInfo) {
			// Ensure determinism.
			info.JobID = 1
			info.Duration = time.Second
			info.TotalDuration = time.Second
			fmt.Fprintln(&compactionLog, info.String())
		},
	}
	reset := func(minVersion, maxVersion FormatMajorVersion) {
		compactionLog.Reset()
		if d != nil {
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))

		opts := (&Options{
			FS:                          mem,
			DebugCheck:                  DebugCheckLevels,
			DisableAutomaticCompactions: true,
			EventListener:               compactionLogEventListener,
			FormatMajorVersion:          randVersion(minVersion, maxVersion),
		}).WithFSDefaults()

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
	}

	// d.mu must be held when calling.
	createOngoingCompaction := func(start, end []byte, startLevel, outputLevel int) (ongoingCompaction *compaction) {
		ongoingCompaction = &compaction{
			inputs:   []compactionLevel{{level: startLevel}, {level: outputLevel}},
			smallest: InternalKey{UserKey: start},
			largest:  InternalKey{UserKey: end},
		}
		ongoingCompaction.startLevel = &ongoingCompaction.inputs[0]
		ongoingCompaction.outputLevel = &ongoingCompaction.inputs[1]
		// Mark files as compacting.
		curr := d.mu.versions.currentVersion()
		ongoingCompaction.startLevel.files = curr.Overlaps(startLevel, d.cmp, start, end, false)
		ongoingCompaction.outputLevel.files = curr.Overlaps(outputLevel, d.cmp, start, end, false)
		for _, cl := range ongoingCompaction.inputs {
			iter := cl.files.Iter()
			for f := iter.First(); f != nil; f = iter.Next() {
				f.CompactionState = manifest.CompactionStateCompacting
			}
		}
		d.mu.compact.inProgress[ongoingCompaction] = struct{}{}
		d.mu.compact.compactingCount++
		return
	}

	// d.mu must be held when calling.
	deleteOngoingCompaction := func(ongoingCompaction *compaction) {
		for _, cl := range ongoingCompaction.inputs {
			iter := cl.files.Iter()
			for f := iter.First(); f != nil; f = iter.Next() {
				f.CompactionState = manifest.CompactionStateNotCompacting
			}
		}
		delete(d.mu.compact.inProgress, ongoingCompaction)
		d.mu.compact.compactingCount--
	}

	runTest := func(t *testing.T, testData string, minVersion, maxVersion FormatMajorVersion, verbose bool) {
		reset(minVersion, maxVersion)
		var ongoingCompaction *compaction
		datadriven.RunTest(t, testData, func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "reset":
				reset(minVersion, maxVersion)
				return ""

			case "batch":
				b := d.NewIndexedBatch()
				if err := runBatchDefineCmd(td, b); err != nil {
					return err.Error()
				}
				require.NoError(t, b.Commit(nil))
				return ""

			case "build":
				if err := runBuildCmd(td, d, mem); err != nil {
					return err.Error()
				}
				return ""

			case "compact":
				if err := runCompactCmd(td, d); err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				if verbose {
					s = d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
				}
				d.mu.Unlock()
				if td.HasArg("hide-file-num") {
					re := regexp.MustCompile(`([0-9]*):\[`)
					s = re.ReplaceAllString(s, "[")
				}
				return s

			case "define":
				if d != nil {
					if err := closeAllSnapshots(d); err != nil {
						return err.Error()
					}
					if err := d.Close(); err != nil {
						return err.Error()
					}
				}

				mem = vfs.NewMem()
				opts := (&Options{
					FS:                          mem,
					DebugCheck:                  DebugCheckLevels,
					EventListener:               compactionLogEventListener,
					FormatMajorVersion:          randVersion(minVersion, maxVersion),
					DisableAutomaticCompactions: true,
				}).WithFSDefaults()

				var err error
				if d, err = runDBDefineCmd(td, opts); err != nil {
					return err.Error()
				}

				s := d.mu.versions.currentVersion().String()
				if verbose {
					s = d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
				}
				return s

			case "file-sizes":
				return runTableFileSizesCmd(td, d)

			case "flush":
				if err := d.Flush(); err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				if verbose {
					s = d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
				}
				d.mu.Unlock()
				return s

			case "ingest":
				if err := runIngestCmd(td, d, mem); err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				if verbose {
					s = d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
				}
				d.mu.Unlock()
				return s

			case "iter":
				// TODO(peter): runDBDefineCmd doesn't properly update the visible
				// sequence number. So we have to use a snapshot with a very large
				// sequence number, otherwise the DB appears empty.
				snap := Snapshot{
					db:     d,
					seqNum: InternalKeySeqNumMax,
				}
				iter, _ := snap.NewIter(nil)
				return runIterCmd(td, iter, true)

			case "lsm":
				return runLSMCmd(td, d)

			case "populate":
				b := d.NewBatch()
				runPopulateCmd(t, td, b)
				count := b.Count()
				require.NoError(t, b.Commit(nil))
				return fmt.Sprintf("wrote %d keys\n", count)

			case "async-compact":
				var s string
				ch := make(chan error, 1)
				go func() {
					if err := runCompactCmd(td, d); err != nil {
						ch <- err
						close(ch)
						return
					}
					d.mu.Lock()
					s = d.mu.versions.currentVersion().String()
					d.mu.Unlock()
					close(ch)
				}()

				manualDone := func() bool {
					select {
					case <-ch:
						return true
					default:
						return false
					}
				}

				err := try(100*time.Microsecond, 20*time.Second, func() error {
					if manualDone() {
						return nil
					}

					d.mu.Lock()
					defer d.mu.Unlock()
					if len(d.mu.compact.manual) == 0 {
						return errors.New("no manual compaction queued")
					}
					manual := d.mu.compact.manual[0]
					if manual.retries == 0 {
						return errors.New("manual compaction has not been retried")
					}
					return nil
				})
				if err != nil {
					return err.Error()
				}

				if manualDone() {
					return "manual compaction did not block for ongoing\n" + s
				}

				d.mu.Lock()
				deleteOngoingCompaction(ongoingCompaction)
				ongoingCompaction = nil
				d.maybeScheduleCompaction()
				d.mu.Unlock()
				if err := <-ch; err != nil {
					return err.Error()
				}
				return "manual compaction blocked until ongoing finished\n" + s

			case "add-ongoing-compaction":
				var startLevel int
				var outputLevel int
				var start string
				var end string
				td.ScanArgs(t, "startLevel", &startLevel)
				td.ScanArgs(t, "outputLevel", &outputLevel)
				td.ScanArgs(t, "start", &start)
				td.ScanArgs(t, "end", &end)
				d.mu.Lock()
				ongoingCompaction = createOngoingCompaction([]byte(start), []byte(end), startLevel, outputLevel)
				d.mu.Unlock()
				return ""

			case "remove-ongoing-compaction":
				d.mu.Lock()
				deleteOngoingCompaction(ongoingCompaction)
				ongoingCompaction = nil
				d.mu.Unlock()
				return ""

			case "set-concurrent-compactions":
				var concurrentCompactions int
				td.ScanArgs(t, "num", &concurrentCompactions)
				d.opts.MaxConcurrentCompactions = func() int {
					return concurrentCompactions
				}
				return ""

			case "sstable-properties":
				return runSSTablePropertiesCmd(t, td, d)

			case "wait-pending-table-stats":
				return runTableStatsCmd(td, d)

			case "close-snapshots":
				d.mu.Lock()
				// Re-enable automatic compactions if they were disabled so that
				// closing snapshots can trigger elision-only compactions if
				// necessary.
				d.opts.DisableAutomaticCompactions = false

				var ss []*Snapshot
				l := &d.mu.snapshots
				for i := l.root.next; i != &l.root; i = i.next {
					ss = append(ss, i)
				}
				d.mu.Unlock()
				for i := range ss {
					if err := ss[i].Close(); err != nil {
						return err.Error()
					}
				}
				return ""

			case "compaction-log":
				defer compactionLog.Reset()
				return compactionLog.String()

			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
	}

	testCases := []struct {
		testData   string
		minVersion FormatMajorVersion
		maxVersion FormatMajorVersion // inclusive
		verbose    bool
	}{
		{
			testData:   "testdata/manual_compaction",
			minVersion: FormatMostCompatible,
			maxVersion: FormatSetWithDelete - 1,
		},
		{
			testData:   "testdata/manual_compaction_set_with_del",
			minVersion: FormatBlockPropertyCollector,
			// This test exercises split user keys.
			maxVersion: FormatSplitUserKeysMarkedCompacted - 1,
		},
		{
			testData:   "testdata/singledel_manual_compaction",
			minVersion: FormatMostCompatible,
			maxVersion: FormatSetWithDelete - 1,
		},
		{
			testData:   "testdata/singledel_manual_compaction_set_with_del",
			minVersion: FormatSetWithDelete,
			maxVersion: internalFormatNewest,
		},
		{
			testData:   "testdata/manual_compaction_range_keys",
			minVersion: FormatRangeKeys,
			maxVersion: internalFormatNewest,
			verbose:    true,
		},
		{
			testData:   "testdata/manual_compaction_file_boundaries",
			minVersion: FormatBlockPropertyCollector,
			// This test exercises split user keys.
			maxVersion: FormatSplitUserKeysMarkedCompacted - 1,
		},
		{
			testData:   "testdata/manual_compaction_file_boundaries_delsized",
			minVersion: FormatDeleteSizedAndObsolete,
			maxVersion: internalFormatNewest,
		},
		{
			testData:   "testdata/manual_compaction_set_with_del_sstable_Pebblev4",
			minVersion: FormatDeleteSizedAndObsolete,
			maxVersion: internalFormatNewest,
		},
		{
			testData:   "testdata/manual_compaction_multilevel",
			minVersion: FormatMostCompatible,
			maxVersion: internalFormatNewest,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.testData, func(t *testing.T) {
			runTest(t, tc.testData, tc.minVersion, tc.maxVersion, tc.verbose)
		})
	}
}

func TestCompactionFindGrandparentLimit(t *testing.T) {
	cmp := DefaultComparer.Compare
	var grandparents []*fileMetadata

	var fileNum base.FileNum
	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		fileNum++
		m := (&fileMetadata{
			FileNum: fileNum,
		}).ExtendPointKeyBounds(
			cmp,
			InternalKey{UserKey: []byte(parts[0])},
			InternalKey{UserKey: []byte(parts[1])},
		)
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/compaction_find_grandparent_limit",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				grandparents = nil
				if len(d.Input) == 0 {
					return ""
				}
				for _, data := range strings.Split(d.Input, "\n") {
					parts := strings.Fields(data)
					if len(parts) != 2 {
						return fmt.Sprintf("malformed test:\n%s", d.Input)
					}

					meta := parseMeta(parts[0])
					var err error
					meta.Size, err = strconv.ParseUint(parts[1], 10, 64)
					if err != nil {
						return err.Error()
					}
					grandparents = append(grandparents, meta)
				}
				return ""

			case "compact":
				c := &compaction{
					cmp:          cmp,
					equal:        DefaultComparer.Equal,
					comparer:     DefaultComparer,
					grandparents: manifest.NewLevelSliceKeySorted(cmp, grandparents),
				}
				if len(d.CmdArgs) != 1 {
					return fmt.Sprintf("%s expects 1 argument", d.Cmd)
				}
				if len(d.CmdArgs[0].Vals) != 1 {
					return fmt.Sprintf("%s expects 1 value", d.CmdArgs[0].Key)
				}
				var err error
				c.maxOverlapBytes, err = strconv.ParseUint(d.CmdArgs[0].Vals[0], 10, 64)
				if err != nil {
					return err.Error()
				}

				var buf bytes.Buffer
				var smallest, largest string
				var grandparentLimit []byte
				for i, key := range strings.Fields(d.Input) {
					if i == 0 {
						smallest = key
						grandparentLimit = c.findGrandparentLimit([]byte(key))
					}
					if grandparentLimit != nil && c.cmp(grandparentLimit, []byte(key)) < 0 {
						fmt.Fprintf(&buf, "%s-%s\n", smallest, largest)
						smallest = key
						grandparentLimit = c.findGrandparentLimit([]byte(key))
					}
					largest = key
				}
				fmt.Fprintf(&buf, "%s-%s\n", smallest, largest)
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionFindL0Limit(t *testing.T) {
	cmp := DefaultComparer.Compare

	fileNumCounter := 1
	parseMeta := func(s string) (*fileMetadata, error) {
		fields := strings.Fields(s)
		parts := strings.Split(fields[0], "-")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{
			FileNum: base.FileNum(fileNumCounter),
		}).ExtendPointKeyBounds(
			cmp,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		fileNumCounter++
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()

		for _, field := range fields[1:] {
			parts := strings.Split(field, "=")
			switch parts[0] {
			case "size":
				size, err := strconv.ParseUint(parts[1], 10, 64)
				if err != nil {
					t.Fatal(err)
				}
				m.Size = size
			}
		}
		m.InitPhysicalBacking()
		return m, nil
	}

	var vers *version
	flushSplitBytes := int64(0)

	datadriven.RunTest(t, "testdata/compaction_find_l0_limit",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				fileMetas := [manifest.NumLevels][]*fileMetadata{}
				baseLevel := manifest.NumLevels - 1
				level := 0
				d.MaybeScanArgs(t, "flush_split_bytes", &flushSplitBytes)

				var err error
				for _, data := range strings.Split(d.Input, "\n") {
					data = strings.TrimSpace(data)
					switch data {
					case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
						level, err = strconv.Atoi(data[1:])
						if err != nil {
							return err.Error()
						}
					default:
						meta, err := parseMeta(data)
						if err != nil {
							return err.Error()
						}
						if level != 0 && level < baseLevel {
							baseLevel = level
						}
						fileMetas[level] = append(fileMetas[level], meta)
					}
				}

				vers = manifest.NewVersion(DefaultComparer.Compare, base.DefaultFormatter, flushSplitBytes, fileMetas)
				flushSplitKeys := vers.L0Sublevels.FlushSplitKeys()

				var buf strings.Builder
				buf.WriteString(vers.String())
				buf.WriteString("flush split keys:\n")
				for _, key := range flushSplitKeys {
					fmt.Fprintf(&buf, "\t%s\n", base.DefaultFormatter(key))
				}

				return buf.String()

			case "flush":
				c := &compaction{
					cmp:      cmp,
					equal:    DefaultComparer.Equal,
					comparer: DefaultComparer,
					version:  vers,
					l0Limits: vers.L0Sublevels.FlushSplitKeys(),
					inputs:   []compactionLevel{{level: -1}, {level: 0}},
				}
				c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]

				var buf bytes.Buffer
				var smallest, largest string
				var l0Limit []byte
				for i, key := range strings.Fields(d.Input) {
					if i == 0 {
						smallest = key
						l0Limit = c.findL0Limit([]byte(key))
					}
					if l0Limit != nil && c.cmp(l0Limit, []byte(key)) < 0 {
						fmt.Fprintf(&buf, "%s-%s\n", smallest, largest)
						smallest = key
						l0Limit = c.findL0Limit([]byte(key))
					}
					largest = key
				}
				fmt.Fprintf(&buf, "%s-%s\n", smallest, largest)
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionOutputLevel(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	version := &version{}

	datadriven.RunTest(t, "testdata/compaction_output_level",
		func(t *testing.T, d *datadriven.TestData) (res string) {
			defer func() {
				if r := recover(); r != nil {
					res = fmt.Sprintln(r)
				}
			}()

			switch d.Cmd {
			case "compact":
				var start, base int
				d.ScanArgs(t, "start", &start)
				d.ScanArgs(t, "base", &base)
				pc := newPickedCompaction(opts, version, start, defaultOutputLevel(start, base), base)
				c := newCompaction(pc, opts, time.Now(), nil /* provider */)
				return fmt.Sprintf("output=%d\nmax-output-file-size=%d\n",
					c.outputLevel.level, c.maxOutputFileSize)

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionAtomicUnitBounds(t *testing.T) {
	cmp := DefaultComparer.Compare
	var files manifest.LevelSlice

	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			cmp,
			base.ParseInternalKey(parts[0]),
			base.ParseInternalKey(parts[1]),
		)
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/compaction_atomic_unit_bounds",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				files = manifest.LevelSlice{}
				if len(d.Input) == 0 {
					return ""
				}
				var ff []*fileMetadata
				for _, data := range strings.Split(d.Input, "\n") {
					meta := parseMeta(data)
					meta.FileNum = FileNum(len(ff))
					ff = append(ff, meta)
				}
				files = manifest.NewLevelSliceKeySorted(cmp, ff)
				return ""

			case "atomic-unit-bounds":
				c := &compaction{
					cmp:      cmp,
					equal:    DefaultComparer.Equal,
					comparer: DefaultComparer,
					inputs:   []compactionLevel{{files: files}, {}},
				}
				c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
				if len(d.CmdArgs) != 1 {
					return fmt.Sprintf("%s expects 1 argument", d.Cmd)
				}
				index, err := strconv.ParseInt(d.CmdArgs[0].String(), 10, 64)
				if err != nil {
					return err.Error()
				}
				iter := files.Iter()
				// Advance iter to `index`.
				_ = iter.First()
				for i := int64(0); i < index; i++ {
					_ = iter.Next()
				}
				atomicUnit, _ := expandToAtomicUnit(c.cmp, iter.Take().Slice(), true /* disableIsCompacting */)
				lower, upper := manifest.KeyRange(c.cmp, atomicUnit.Iter())
				return fmt.Sprintf("%s-%s\n", lower.UserKey, upper.UserKey)

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestCompactionDeleteOnlyHints(t *testing.T) {
	parseUint64 := func(s string) uint64 {
		v, err := strconv.ParseUint(s, 10, 64)
		require.NoError(t, err)
		return v
	}
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
	}()

	var compactInfo *CompactionInfo // protected by d.mu
	reset := func() (*Options, error) {
		if d != nil {
			compactInfo = nil
			if err := closeAllSnapshots(d); err != nil {
				return nil, err
			}
			if err := d.Close(); err != nil {
				return nil, err
			}
		}
		opts := (&Options{
			FS:         vfs.NewMem(),
			DebugCheck: DebugCheckLevels,
			EventListener: &EventListener{
				CompactionEnd: func(info CompactionInfo) {
					if compactInfo != nil {
						return
					}
					compactInfo = &info
				},
			},
			FormatMajorVersion: internalFormatNewest,
		}).WithFSDefaults()

		// Collection of table stats can trigger compactions. As we want full
		// control over when compactions are run, disable stats by default.
		opts.private.disableTableStats = true

		return opts, nil
	}

	compactionString := func() string {
		for d.mu.compact.compactingCount > 0 {
			d.mu.compact.cond.Wait()
		}

		s := "(none)"
		if compactInfo != nil {
			// Fix the job ID and durations for determinism.
			compactInfo.JobID = 100
			compactInfo.Duration = time.Second
			compactInfo.TotalDuration = 2 * time.Second
			s = compactInfo.String()
			compactInfo = nil
		}
		return s
	}

	var err error
	var opts *Options
	datadriven.RunTest(t, "testdata/compaction_delete_only_hints",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "define":
				opts, err = reset()
				if err != nil {
					return err.Error()
				}
				d, err = runDBDefineCmd(td, opts)
				if err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			case "force-set-hints":
				d.mu.Lock()
				defer d.mu.Unlock()
				d.mu.compact.deletionHints = d.mu.compact.deletionHints[:0]
				var buf bytes.Buffer
				for _, data := range strings.Split(td.Input, "\n") {
					parts := strings.FieldsFunc(strings.TrimSpace(data),
						func(r rune) bool { return r == '-' || r == ' ' || r == '.' })

					start, end := []byte(parts[2]), []byte(parts[3])

					var tombstoneFile *fileMetadata
					tombstoneLevel := int(parseUint64(parts[0][1:]))

					// Set file number to the value provided in the input.
					tombstoneFile = &fileMetadata{
						FileNum: base.FileNum(parseUint64(parts[1])),
					}

					var hintType deleteCompactionHintType
					switch typ := parts[7]; typ {
					case "point_key_only":
						hintType = deleteCompactionHintTypePointKeyOnly
					case "range_key_only":
						hintType = deleteCompactionHintTypeRangeKeyOnly
					case "point_and_range_key":
						hintType = deleteCompactionHintTypePointAndRangeKey
					default:
						return fmt.Sprintf("unknown hint type: %s", typ)
					}

					h := deleteCompactionHint{
						hintType:                hintType,
						start:                   start,
						end:                     end,
						fileSmallestSeqNum:      parseUint64(parts[4]),
						tombstoneLevel:          tombstoneLevel,
						tombstoneFile:           tombstoneFile,
						tombstoneSmallestSeqNum: parseUint64(parts[5]),
						tombstoneLargestSeqNum:  parseUint64(parts[6]),
					}
					d.mu.compact.deletionHints = append(d.mu.compact.deletionHints, h)
					fmt.Fprintln(&buf, h.String())
				}
				return buf.String()

			case "get-hints":
				d.mu.Lock()
				defer d.mu.Unlock()

				// Force collection of table stats. This requires re-enabling the
				// collection flag. We also do not want compactions to run as part of
				// the stats collection job, so we disable it temporarily.
				d.opts.private.disableTableStats = false
				d.opts.DisableAutomaticCompactions = true
				defer func() {
					d.opts.private.disableTableStats = true
					d.opts.DisableAutomaticCompactions = false
				}()

				// NB: collectTableStats attempts to acquire the lock. Temporarily
				// unlock here to avoid a deadlock.
				d.mu.Unlock()
				didRun := d.collectTableStats()
				d.mu.Lock()

				if !didRun {
					// If a job was already running, wait for the results.
					d.waitTableStats()
				}

				hints := d.mu.compact.deletionHints
				if len(hints) == 0 {
					return "(none)"
				}
				var buf bytes.Buffer
				for _, h := range hints {
					buf.WriteString(h.String() + "\n")
				}
				return buf.String()

			case "maybe-compact":
				d.mu.Lock()
				d.maybeScheduleCompaction()

				var buf bytes.Buffer
				fmt.Fprintf(&buf, "Deletion hints:\n")
				for _, h := range d.mu.compact.deletionHints {
					fmt.Fprintf(&buf, "  %s\n", h.String())
				}
				if len(d.mu.compact.deletionHints) == 0 {
					fmt.Fprintf(&buf, "  (none)\n")
				}
				fmt.Fprintf(&buf, "Compactions:\n")
				fmt.Fprintf(&buf, "  %s", compactionString())
				d.mu.Unlock()
				return buf.String()

			case "compact":
				if err := runCompactCmd(td, d); err != nil {
					return err.Error()
				}
				d.mu.Lock()
				compactInfo = nil
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			case "close-snapshot":
				seqNum, err := strconv.ParseUint(strings.TrimSpace(td.Input), 0, 64)
				if err != nil {
					return err.Error()
				}
				d.mu.Lock()
				var s *Snapshot
				l := &d.mu.snapshots
				for i := l.root.next; i != &l.root; i = i.next {
					if i.seqNum == seqNum {
						s = i
					}
				}
				d.mu.Unlock()
				if s == nil {
					return "(not found)"
				} else if err := s.Close(); err != nil {
					return err.Error()
				}

				d.mu.Lock()
				// Closing the snapshot may have triggered a compaction.
				str := compactionString()
				d.mu.Unlock()
				return str

			case "iter":
				snap := Snapshot{
					db:     d,
					seqNum: InternalKeySeqNumMax,
				}
				iter, _ := snap.NewIter(nil)
				return runIterCmd(td, iter, true)

			case "reset":
				opts, err = reset()
				if err != nil {
					return err.Error()
				}
				d, err = Open("", opts)
				if err != nil {
					return err.Error()
				}
				return ""

			case "ingest":
				if err = runBuildCmd(td, d, d.opts.FS); err != nil {
					return err.Error()
				}
				if err = runIngestCmd(td, d, d.opts.FS); err != nil {
					return err.Error()
				}
				return "OK"

			case "describe-lsm":
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
}

func TestCompactionTombstones(t *testing.T) {
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
	}()

	var compactInfo *CompactionInfo // protected by d.mu

	compactionString := func() string {
		for d.mu.compact.compactingCount > 0 {
			d.mu.compact.cond.Wait()
		}

		s := "(none)"
		if compactInfo != nil {
			// Fix the job ID and durations for determinism.
			compactInfo.JobID = 100
			compactInfo.Duration = time.Second
			compactInfo.TotalDuration = 2 * time.Second
			s = compactInfo.String()
			compactInfo = nil
		}
		return s
	}

	datadriven.RunTest(t, "testdata/compaction_tombstones",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "define":
				if d != nil {
					compactInfo = nil
					require.NoError(t, closeAllSnapshots(d))
					if err := d.Close(); err != nil {
						return err.Error()
					}
				}
				opts := (&Options{
					FS:         vfs.NewMem(),
					DebugCheck: DebugCheckLevels,
					EventListener: &EventListener{
						CompactionEnd: func(info CompactionInfo) {
							compactInfo = &info
						},
					},
					FormatMajorVersion: internalFormatNewest,
				}).WithFSDefaults()
				var err error
				d, err = runDBDefineCmd(td, opts)
				if err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			case "maybe-compact":
				d.mu.Lock()
				d.opts.DisableAutomaticCompactions = false
				d.maybeScheduleCompaction()
				s := compactionString()
				d.mu.Unlock()
				return s

			case "wait-pending-table-stats":
				return runTableStatsCmd(td, d)

			case "close-snapshot":
				seqNum, err := strconv.ParseUint(strings.TrimSpace(td.Input), 0, 64)
				if err != nil {
					return err.Error()
				}
				d.mu.Lock()
				var s *Snapshot
				l := &d.mu.snapshots
				for i := l.root.next; i != &l.root; i = i.next {
					if i.seqNum == seqNum {
						s = i
					}
				}
				d.mu.Unlock()
				if s == nil {
					return "(not found)"
				} else if err := s.Close(); err != nil {
					return err.Error()
				}

				d.mu.Lock()
				// Closing the snapshot may have triggered a compaction.
				str := compactionString()
				d.mu.Unlock()
				return str

			case "close":
				if err := d.Close(); err != nil {
					return err.Error()
				}
				d = nil
				return ""

			case "version":
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
}

func closeAllSnapshots(d *DB) error {
	d.mu.Lock()
	var ss []*Snapshot
	l := &d.mu.snapshots
	for i := l.root.next; i != &l.root; i = i.next {
		ss = append(ss, i)
	}
	d.mu.Unlock()
	for i := range ss {
		if err := ss[i].Close(); err != nil {
			return err
		}
	}
	return nil
}

func TestCompactionReadTriggeredQueue(t *testing.T) {

	// Convert a read compaction to a string which this test
	// understands.
	showRC := func(rc *readCompaction) string {
		return fmt.Sprintf(
			"L%d: %s-%s %d\n", rc.level, string(rc.start), string(rc.end), rc.fileNum,
		)
	}

	var queue *readCompactionQueue

	datadriven.RunTest(t, "testdata/read_compaction_queue",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "create":
				queue = &readCompactionQueue{}
				return "(success)"
			case "add-compaction":
				for _, line := range strings.Split(td.Input, "\n") {
					if line == "" {
						continue
					}
					parts := strings.Split(line, " ")

					if len(parts) != 3 {
						return "error: malformed data for add-compaction. usage: <level>: <start>-<end> <filenum>"
					}
					if l, err := strconv.Atoi(parts[0][1:2]); err == nil {
						keys := strings.Split(parts[1], "-")
						fileNum, _ := strconv.Atoi(parts[2])
						rc := readCompaction{
							level:   l,
							start:   []byte(keys[0]),
							end:     []byte(keys[1]),
							fileNum: base.FileNum(fileNum),
						}
						queue.add(&rc, DefaultComparer.Compare)
					} else {
						return err.Error()
					}
				}
				return ""
			case "remove-compaction":
				rc := queue.remove()
				if rc == nil {
					return "(nil)"
				}
				return showRC(rc)
			case "print-size":
				// Print the size of the queue.
				return fmt.Sprintf("%d", queue.size)
			case "print-queue":
				// Print each element of the queue on a separate line.
				var sb strings.Builder
				if queue.size == 0 {
					sb.WriteString("(empty)")
				}

				for i := 0; i < queue.size; i++ {
					rc := queue.at(i)
					sb.WriteString(showRC(rc))
				}
				return sb.String()
			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		},
	)
}

func (qu *readCompactionQueue) at(i int) *readCompaction {
	if i >= qu.size {
		return nil
	}

	return qu.queue[i]
}

func TestCompactionReadTriggered(t *testing.T) {
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()

	var compactInfo *CompactionInfo // protected by d.mu

	compactionString := func() string {
		for d.mu.compact.compactingCount > 0 {
			d.mu.compact.cond.Wait()
		}

		s := "(none)"
		if compactInfo != nil {
			// Fix the job ID and durations for determinism.
			compactInfo.JobID = 100
			compactInfo.Duration = time.Second
			compactInfo.TotalDuration = 2 * time.Second
			s = compactInfo.String()
			compactInfo = nil
		}
		return s
	}

	datadriven.RunTest(t, "testdata/compaction_read_triggered",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "define":
				if d != nil {
					compactInfo = nil
					if err := d.Close(); err != nil {
						return err.Error()
					}
				}
				opts := (&Options{
					FS:         vfs.NewMem(),
					DebugCheck: DebugCheckLevels,
					EventListener: &EventListener{
						CompactionEnd: func(info CompactionInfo) {
							compactInfo = &info
						},
					},
				}).WithFSDefaults()
				var err error
				d, err = runDBDefineCmd(td, opts)
				if err != nil {
					return err.Error()
				}
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			case "add-read-compaction":
				d.mu.Lock()
				td.MaybeScanArgs(t, "flushing", &d.mu.compact.flushing)
				for _, line := range strings.Split(td.Input, "\n") {
					if line == "" {
						continue
					}
					parts := strings.Split(line, " ")
					if len(parts) != 3 {
						return "error: malformed data for add-read-compaction. usage: <level>: <start>-<end> <filenum>"
					}
					if l, err := strconv.Atoi(parts[0][:1]); err == nil {
						keys := strings.Split(parts[1], "-")
						fileNum, _ := strconv.Atoi(parts[2])
						rc := readCompaction{
							level:   l,
							start:   []byte(keys[0]),
							end:     []byte(keys[1]),
							fileNum: base.FileNum(fileNum),
						}
						d.mu.compact.readCompactions.add(&rc, DefaultComparer.Compare)
					} else {
						return err.Error()
					}
				}
				d.mu.Unlock()
				return ""

			case "show-read-compactions":
				d.mu.Lock()
				var sb strings.Builder
				if d.mu.compact.readCompactions.size == 0 {
					sb.WriteString("(none)")
				}
				for i := 0; i < d.mu.compact.readCompactions.size; i++ {
					rc := d.mu.compact.readCompactions.at(i)
					sb.WriteString(fmt.Sprintf("(level: %d, start: %s, end: %s)\n", rc.level, string(rc.start), string(rc.end)))
				}
				d.mu.Unlock()
				return sb.String()

			case "maybe-compact":
				d.mu.Lock()
				d.opts.DisableAutomaticCompactions = false
				d.maybeScheduleCompaction()
				s := compactionString()
				d.mu.Unlock()
				return s

			case "version":
				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
}

func TestCompactionInuseKeyRanges(t *testing.T) {
	cmp := DefaultComparer.Compare
	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendRangeKeyBounds(
			cmp,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		m.InitPhysicalBacking()
		return m
	}

	opts := (*Options)(nil).EnsureDefaults()

	var c *compaction
	datadriven.RunTest(t, "testdata/compaction_inuse_key_ranges", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			c = &compaction{
				cmp:       DefaultComparer.Compare,
				equal:     DefaultComparer.Equal,
				comparer:  DefaultComparer,
				formatKey: DefaultComparer.FormatKey,
				inputs:    []compactionLevel{{}, {}},
			}
			c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
			var files [numLevels][]*fileMetadata
			var currentLevel int
			fileNum := FileNum(1)

			for _, data := range strings.Split(td.Input, "\n") {
				switch data {
				case "L0", "L1", "L2", "L3", "L4", "L5", "L6":
					level, err := strconv.Atoi(data[1:])
					if err != nil {
						return err.Error()
					}
					currentLevel = level

				default:
					meta := parseMeta(data)
					meta.FileNum = fileNum
					fileNum++
					files[currentLevel] = append(files[currentLevel], meta)
				}
			}
			c.version = newVersion(opts, files)
			return c.version.String()

		case "inuse-key-ranges":
			var buf bytes.Buffer
			for _, line := range strings.Split(td.Input, "\n") {
				parts := strings.Fields(line)
				if len(parts) != 3 {
					fmt.Fprintf(&buf, "expected <level> <smallest> <largest>: %q\n", line)
					continue
				}
				level, err := strconv.Atoi(parts[0])
				if err != nil {
					fmt.Fprintf(&buf, "expected <level> <smallest> <largest>: %q: %v\n", line, err)
					continue
				}
				c.outputLevel.level = level
				c.smallest.UserKey = []byte(parts[1])
				c.largest.UserKey = []byte(parts[2])

				c.inuseKeyRanges = nil
				c.setupInuseKeyRanges()
				if len(c.inuseKeyRanges) == 0 {
					fmt.Fprintf(&buf, ".\n")
				} else {
					for i, r := range c.inuseKeyRanges {
						if i > 0 {
							fmt.Fprintf(&buf, " ")
						}
						fmt.Fprintf(&buf, "%s-%s", r.Start, r.End)
					}
					fmt.Fprintf(&buf, "\n")
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestCompactionInuseKeyRangesRandomized(t *testing.T) {
	var (
		fileNum     = FileNum(0)
		opts        = (*Options)(nil).EnsureDefaults()
		seed        = int64(time.Now().UnixNano())
		rng         = rand.New(rand.NewSource(seed))
		endKeyspace = 26 * 26
	)
	t.Logf("Using rng seed %d.", seed)

	for iter := 0; iter < 100; iter++ {
		makeUserKey := func(i int) []byte {
			if i >= endKeyspace {
				i = endKeyspace - 1
			}
			return []byte{byte(i/26 + 'a'), byte(i%26 + 'a')}
		}
		makeIK := func(level, i int) InternalKey {
			return base.MakeInternalKey(
				makeUserKey(i),
				uint64(numLevels-level),
				base.InternalKeyKindSet,
			)
		}
		makeFile := func(level, start, end int) *fileMetadata {
			fileNum++
			m := (&fileMetadata{
				FileNum: fileNum,
			}).ExtendPointKeyBounds(
				opts.Comparer.Compare,
				makeIK(level, start),
				makeIK(level, end),
			)
			m.SmallestSeqNum = m.Smallest.SeqNum()
			m.LargestSeqNum = m.Largest.SeqNum()
			m.InitPhysicalBacking()
			return m
		}
		overlaps := func(startA, endA, startB, endB []byte) bool {
			disjoint := opts.Comparer.Compare(endB, startA) < 0 || opts.Comparer.Compare(endA, startB) < 0
			return !disjoint
		}
		var files [numLevels][]*fileMetadata
		for l := 0; l < numLevels; l++ {
			for i := 0; i < rand.Intn(10); i++ {
				s := rng.Intn(endKeyspace)
				maxWidth := rng.Intn(endKeyspace-s) + 1
				e := rng.Intn(maxWidth) + s
				sKey, eKey := makeUserKey(s), makeUserKey(e)
				// Discard the key range if it overlaps any existing files
				// within this level.
				var o bool
				for _, f := range files[l] {
					o = o || overlaps(sKey, eKey, f.Smallest.UserKey, f.Largest.UserKey)
				}
				if o {
					continue
				}
				files[l] = append(files[l], makeFile(l, s, e))
			}
			sort.Slice(files[l], func(i, j int) bool {
				return opts.Comparer.Compare(files[l][i].Smallest.UserKey, files[l][j].Smallest.UserKey) < 0
			})
		}
		v := newVersion(opts, files)
		t.Log(v.DebugString(opts.Comparer.FormatKey))
		for i := 0; i < 1000; i++ {
			l := rng.Intn(numLevels)
			s := rng.Intn(endKeyspace)
			maxWidth := rng.Intn(endKeyspace-s) + 1
			e := rng.Intn(maxWidth) + s
			sKey, eKey := makeUserKey(s), makeUserKey(e)
			keyRanges := calculateInuseKeyRanges(v, opts.Comparer.Compare, l, numLevels-1, sKey, eKey)

			for level := l; level < numLevels; level++ {
				for _, f := range files[level] {
					if !overlaps(sKey, eKey, f.Smallest.UserKey, f.Largest.UserKey) {
						// This file doesn't overlap the queried range. Skip it.
						continue
					}
					// This file does overlap the queried range. The key range
					// [MAX(f.Smallest, sKey), MIN(f.Largest, eKey)] must be fully
					// contained by a key range in keyRanges.
					checkStart, checkEnd := f.Smallest.UserKey, f.Largest.UserKey
					if opts.Comparer.Compare(checkStart, sKey) < 0 {
						checkStart = sKey
					}
					if opts.Comparer.Compare(checkEnd, eKey) > 0 {
						checkEnd = eKey
					}
					var contained bool
					for _, kr := range keyRanges {
						contained = contained ||
							(opts.Comparer.Compare(checkStart, kr.Start) >= 0 &&
								opts.Comparer.Compare(checkEnd, kr.End) <= 0)
					}
					if !contained {
						t.Errorf("Seed %d, iter %d: File %s overlaps %q-%q, but is not fully contained in any of the key ranges.",
							seed, iter, f, sKey, eKey)
					}
				}
			}
		}
	}
}

func TestCompactionAllowZeroSeqNum(t *testing.T) {
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, closeAllSnapshots(d))
			require.NoError(t, d.Close())
		}
	}()

	metaRE := regexp.MustCompile(`^L([0-9]+):([^-]+)-(.+)$`)
	var fileNum base.FileNum
	parseMeta := func(s string) (level int, meta *fileMetadata) {
		match := metaRE.FindStringSubmatch(s)
		if match == nil {
			t.Fatalf("malformed table spec: %s", s)
		}
		level, err := strconv.Atoi(match[1])
		if err != nil {
			t.Fatalf("malformed table spec: %s: %s", s, err)
		}
		fileNum++
		meta = (&fileMetadata{
			FileNum: fileNum,
		}).ExtendPointKeyBounds(
			d.cmp,
			InternalKey{UserKey: []byte(match[2])},
			InternalKey{UserKey: []byte(match[3])},
		)
		meta.InitPhysicalBacking()
		return level, meta
	}

	datadriven.RunTest(t, "testdata/compaction_allow_zero_seqnum",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "define":
				if d != nil {
					require.NoError(t, closeAllSnapshots(d))
					if err := d.Close(); err != nil {
						return err.Error()
					}
				}

				var err error
				if d, err = runDBDefineCmd(td, nil /* options */); err != nil {
					return err.Error()
				}

				d.mu.Lock()
				s := d.mu.versions.currentVersion().String()
				d.mu.Unlock()
				return s

			case "allow-zero-seqnum":
				d.mu.Lock()
				c := &compaction{
					cmp:      d.cmp,
					comparer: d.opts.Comparer,
					version:  d.mu.versions.currentVersion(),
					inputs:   []compactionLevel{{}, {}},
				}
				c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
				d.mu.Unlock()

				var buf bytes.Buffer
				for _, line := range strings.Split(td.Input, "\n") {
					parts := strings.Fields(line)
					if len(parts) == 0 {
						continue
					}
					c.flushing = nil
					c.startLevel.level = -1

					var startFiles, outputFiles []*fileMetadata

					switch {
					case len(parts) == 1 && parts[0] == "flush":
						c.outputLevel.level = 0
						d.mu.Lock()
						c.flushing = d.mu.mem.queue
						d.mu.Unlock()

					default:
						for _, p := range parts {
							level, meta := parseMeta(p)
							if c.startLevel.level == -1 {
								c.startLevel.level = level
							}

							switch level {
							case c.startLevel.level:
								startFiles = append(startFiles, meta)
							case c.startLevel.level + 1:
								outputFiles = append(outputFiles, meta)
							default:
								return fmt.Sprintf("invalid level %d: expected %d or %d",
									level, c.startLevel.level, c.startLevel.level+1)
							}
						}
						c.outputLevel.level = c.startLevel.level + 1
						c.startLevel.files = manifest.NewLevelSliceSpecificOrder(startFiles)
						c.outputLevel.files = manifest.NewLevelSliceKeySorted(c.cmp, outputFiles)
					}

					c.smallest, c.largest = manifest.KeyRange(c.cmp,
						c.startLevel.files.Iter(),
						c.outputLevel.files.Iter())

					c.inuseKeyRanges = nil
					c.setupInuseKeyRanges()
					fmt.Fprintf(&buf, "%t\n", c.allowZeroSeqNum())
				}
				return buf.String()

			default:
				return fmt.Sprintf("unknown command: %s", td.Cmd)
			}
		})
}

func TestCompactionErrorOnUserKeyOverlap(t *testing.T) {
	cmp := DefaultComparer.Compare
	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			cmp,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/compaction_error_on_user_key_overlap",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "error-on-user-key-overlap":
				c := &compaction{
					cmp:       DefaultComparer.Compare,
					comparer:  DefaultComparer,
					formatKey: DefaultComparer.FormatKey,
				}
				var files []manifest.NewFileEntry
				fileNum := FileNum(1)

				for _, data := range strings.Split(d.Input, "\n") {
					meta := parseMeta(data)
					meta.FileNum = fileNum
					fileNum++
					files = append(files, manifest.NewFileEntry{Level: 1, Meta: meta})
				}

				result := "OK"
				ve := &versionEdit{
					NewFiles: files,
				}
				if err := c.errorOnUserKeyOverlap(ve); err != nil {
					result = fmt.Sprint(err)
				}
				return result

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

// TestCompactionErrorCleanup tests an error encountered during a compaction
// after some output tables have been created. It ensures that the pending
// output tables are removed from the filesystem.
func TestCompactionErrorCleanup(t *testing.T) {
	// protected by d.mu
	var (
		initialSetupDone bool
		tablesCreated    []FileNum
	)

	mem := vfs.NewMem()
	ii := errorfs.OnIndex(math.MaxInt32) // start disabled
	opts := (&Options{
		FS:     errorfs.Wrap(mem, ii),
		Levels: make([]LevelOptions, numLevels),
		EventListener: &EventListener{
			TableCreated: func(info TableCreateInfo) {
				t.Log(info)

				// If the initial setup is over, record tables created and
				// inject an error immediately after the second table is
				// created.
				if initialSetupDone {
					tablesCreated = append(tablesCreated, info.FileNum)
					if len(tablesCreated) >= 2 {
						ii.SetIndex(0)
					}
				}
			},
		},
	}).WithFSDefaults()
	for i := range opts.Levels {
		opts.Levels[i].TargetFileSize = 1
	}
	opts.testingRandomized(t)
	d, err := Open("", opts)
	require.NoError(t, err)

	ingest := func(keys ...string) {
		t.Helper()
		f, err := mem.Create("ext")
		require.NoError(t, err)

		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
			TableFormat: d.FormatMajorVersion().MaxTableFormat(),
		})
		for _, k := range keys {
			require.NoError(t, w.Set([]byte(k), nil))
		}
		require.NoError(t, w.Close())
		require.NoError(t, d.Ingest([]string{"ext"}))
	}
	ingest("a", "c")
	ingest("b")

	// Trigger a manual compaction, which will encounter an injected error
	// after the second table is created.
	d.mu.Lock()
	initialSetupDone = true
	d.mu.Unlock()
	err = d.Compact([]byte("a"), []byte("d"), false)
	require.Error(t, err, "injected error")

	d.mu.Lock()
	if len(tablesCreated) < 2 {
		t.Fatalf("expected 2 output tables created by compaction: found %d", len(tablesCreated))
	}
	d.mu.Unlock()

	require.NoError(t, d.Close())
	for _, fileNum := range tablesCreated {
		filename := fmt.Sprintf("%s.sst", fileNum)
		if _, err = mem.Stat(filename); err == nil || !oserror.IsNotExist(err) {
			t.Errorf("expected %q to not exist: %s", filename, err)
		}
	}
}

func TestCompactionCheckOrdering(t *testing.T) {
	cmp := DefaultComparer.Compare
	parseMeta := func(s string) *fileMetadata {
		parts := strings.Split(s, "-")
		if len(parts) != 2 {
			t.Fatalf("malformed table spec: %s", s)
		}
		m := (&fileMetadata{}).ExtendPointKeyBounds(
			cmp,
			base.ParseInternalKey(strings.TrimSpace(parts[0])),
			base.ParseInternalKey(strings.TrimSpace(parts[1])),
		)
		m.SmallestSeqNum = m.Smallest.SeqNum()
		m.LargestSeqNum = m.Largest.SeqNum()
		m.InitPhysicalBacking()
		return m
	}

	datadriven.RunTest(t, "testdata/compaction_check_ordering",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "check-ordering":
				c := &compaction{
					cmp:       DefaultComparer.Compare,
					comparer:  DefaultComparer,
					formatKey: DefaultComparer.FormatKey,
					logger:    panicLogger{},
					inputs:    []compactionLevel{{level: -1}, {level: -1}},
				}
				c.startLevel, c.outputLevel = &c.inputs[0], &c.inputs[1]
				var startFiles, outputFiles []*fileMetadata
				var sublevels []manifest.LevelSlice
				var files *[]*fileMetadata
				var sublevel []*fileMetadata
				var sublevelNum int
				var parsingSublevel bool
				fileNum := FileNum(1)

				switchSublevel := func() {
					if sublevel != nil {
						sublevels = append(
							sublevels, manifest.NewLevelSliceSpecificOrder(sublevel),
						)
						sublevel = nil
					}
					parsingSublevel = false
				}

				for _, data := range strings.Split(d.Input, "\n") {
					if data[0] == 'L' && len(data) == 4 {
						// Format L0.{sublevel}.
						switchSublevel()
						level, err := strconv.Atoi(data[1:2])
						if err != nil {
							return err.Error()
						}
						sublevelNum, err = strconv.Atoi(data[3:])
						if err != nil {
							return err.Error()
						}
						if c.startLevel.level == -1 {
							c.startLevel.level = level
							files = &startFiles
						}
						parsingSublevel = true
					} else if data[0] == 'L' {
						switchSublevel()
						level, err := strconv.Atoi(data[1:])
						if err != nil {
							return err.Error()
						}
						if c.startLevel.level == -1 {
							c.startLevel.level = level
							files = &startFiles
						} else if c.outputLevel.level == -1 {
							if c.startLevel.level >= level {
								return fmt.Sprintf("startLevel=%d >= outputLevel=%d\n", c.startLevel.level, level)
							}
							c.outputLevel.level = level
							files = &outputFiles
						} else {
							return "outputLevel already set\n"
						}
					} else {
						meta := parseMeta(data)
						meta.FileNum = fileNum
						fileNum++
						*files = append(*files, meta)
						if parsingSublevel {
							meta.SubLevel = sublevelNum
							sublevel = append(sublevel, meta)
						}
					}
				}

				switchSublevel()
				c.startLevel.files = manifest.NewLevelSliceSpecificOrder(startFiles)
				c.outputLevel.files = manifest.NewLevelSliceSpecificOrder(outputFiles)
				if c.outputLevel.level == -1 {
					c.outputLevel.level = 0
				}
				if c.startLevel.level == 0 {
					// We don't change the input files for the compaction beyond this point.
					c.startLevel.l0SublevelInfo = generateSublevelInfo(c.cmp, c.startLevel.files)
				}

				newIters := func(
					_ context.Context, _ *manifest.FileMetadata, _ *IterOptions, _ internalIterOpts,
				) (internalIterator, keyspan.FragmentIterator, error) {
					return &errorIter{}, nil, nil
				}
				result := "OK"
				_, err := c.newInputIter(newIters, nil, nil)
				if err != nil {
					result = fmt.Sprint(err)
				}
				return result

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

type mockSplitter struct {
	shouldSplitVal maybeSplit
}

func (m *mockSplitter) shouldSplitBefore(key *InternalKey, tw *sstable.Writer) maybeSplit {
	return m.shouldSplitVal
}

func (m *mockSplitter) onNewOutput(key []byte) []byte {
	return nil
}

func TestCompactionOutputSplitters(t *testing.T) {
	var main, child0, child1 compactionOutputSplitter
	var prevUserKey []byte
	pickSplitter := func(input string) *compactionOutputSplitter {
		switch input {
		case "main":
			return &main
		case "child0":
			return &child0
		case "child1":
			return &child1
		default:
			t.Fatalf("invalid splitter slot: %s", input)
			return nil
		}
	}

	datadriven.RunTest(t, "testdata/compaction_output_splitters",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "reset":
				main = nil
				child0 = nil
				child1 = nil
			case "init":
				if len(d.CmdArgs) < 2 {
					return "expected at least 2 args"
				}
				splitterToInit := pickSplitter(d.CmdArgs[0].Key)
				switch d.CmdArgs[1].Key {
				case "array":
					*splitterToInit = &splitterGroup{
						cmp:       base.DefaultComparer.Compare,
						splitters: []compactionOutputSplitter{child0, child1},
					}
				case "mock":
					*splitterToInit = &mockSplitter{}
				case "userkey":
					*splitterToInit = &userKeyChangeSplitter{
						cmp: base.DefaultComparer.Compare,
						unsafePrevUserKey: func() []byte {
							return prevUserKey
						},
						splitter: child0,
					}
				}
				(*splitterToInit).onNewOutput(nil)
			case "set-should-split":
				if len(d.CmdArgs) < 2 {
					return "expected at least 2 args"
				}
				splitterToSet := (*pickSplitter(d.CmdArgs[0].Key)).(*mockSplitter)
				var val maybeSplit
				switch d.CmdArgs[1].Key {
				case "split-now":
					val = splitNow
				case "no-split":
					val = noSplit
				default:
					t.Fatalf("unexpected value for should-split: %s", d.CmdArgs[1].Key)
				}
				splitterToSet.shouldSplitVal = val
			case "should-split-before":
				if len(d.CmdArgs) < 1 {
					return "expected at least 1 arg"
				}
				key := base.ParseInternalKey(d.CmdArgs[0].Key)
				shouldSplit := main.shouldSplitBefore(&key, nil)
				if shouldSplit == splitNow {
					main.onNewOutput(key.UserKey)
					prevUserKey = nil
				} else {
					prevUserKey = key.UserKey
				}
				return shouldSplit.String()
			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
			return "ok"
		})
}

func TestCompactFlushQueuedMemTableAndFlushMetrics(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("test is flaky on windows")
	}

	// Verify that manual compaction forces a flush of a queued memtable.

	mem := vfs.NewMem()
	d, err := Open("", testingRandomized(t, &Options{
		FS: mem,
	}).WithFSDefaults())
	require.NoError(t, err)

	// Add the key "a" to the memtable, then fill up the memtable with the key
	// prefix "b". The compaction will only overlap with the queued memtable,
	// not the mutable memtable.
	// NB: The initial memtable size is 256KB, which is filled up with random
	// values which typically don't compress well. The test also appends the
	// random value to the "b" key to limit overwriting of the same key, which
	// would get collapsed at flush time since there are no open snapshots.
	value := make([]byte, 50)
	_, err = crand.Read(value)
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("a"), value, nil))
	for {
		_, err = crand.Read(value)
		require.NoError(t, err)
		require.NoError(t, d.Set(append([]byte("b"), value...), value, nil))
		d.mu.Lock()
		done := len(d.mu.mem.queue) == 2
		d.mu.Unlock()
		if done {
			break
		}
	}

	require.NoError(t, d.Compact([]byte("a"), []byte("a\x00"), false))
	d.mu.Lock()
	require.Equal(t, 1, len(d.mu.mem.queue))
	d.mu.Unlock()
	// Flush metrics are updated after and non-atomically with the memtable
	// being removed from the queue.
	func() {
		begin := time.Now()
		for {
			metrics := d.Metrics()
			require.NotNil(t, metrics)
			if int64(50<<10) < metrics.Flush.WriteThroughput.Bytes {
				// The writes (during which the flush is idle) and the flush work
				// should not be so fast as to be unrealistic. If these turn out to be
				// flaky we could instead inject a clock.
				tinyInterval := int64(50 * time.Microsecond)
				require.Less(t, tinyInterval, int64(metrics.Flush.WriteThroughput.WorkDuration))
				require.Less(t, tinyInterval, int64(metrics.Flush.WriteThroughput.IdleDuration))
				return
			}
			if time.Since(begin) > 2*time.Second {
				t.Fatal()
			}
			time.Sleep(time.Millisecond)
		}
	}()
	require.NoError(t, d.Close())
}

func TestCompactFlushQueuedLargeBatch(t *testing.T) {
	// Verify that compaction forces a flush of a queued large batch.

	mem := vfs.NewMem()
	d, err := Open("", testingRandomized(t, &Options{
		FS: mem,
	}).WithFSDefaults())
	require.NoError(t, err)

	// The default large batch threshold is slightly less than 1/2 of the
	// memtable size which makes triggering a problem with flushing queued large
	// batches irritating. Manually adjust the threshold to 1/8 of the memtable
	// size in order to more easily create a situation where a large batch is
	// queued but not automatically flushed.
	d.mu.Lock()
	d.largeBatchThreshold = d.opts.MemTableSize / 8
	require.Equal(t, 1, len(d.mu.mem.queue))
	d.mu.Unlock()

	// Set a record with a large value. This will be transformed into a large
	// batch and placed in the flushable queue.
	require.NoError(t, d.Set([]byte("a"), bytes.Repeat([]byte("v"), int(d.largeBatchThreshold)), nil))
	d.mu.Lock()
	require.Greater(t, len(d.mu.mem.queue), 1)
	d.mu.Unlock()

	require.NoError(t, d.Compact([]byte("a"), []byte("a\x00"), false))
	d.mu.Lock()
	require.Equal(t, 1, len(d.mu.mem.queue))
	d.mu.Unlock()

	require.NoError(t, d.Close())
}

func TestFlushError(t *testing.T) {
	// Error the first five times we try to write a sstable.
	var errorOps atomic.Int32
	errorOps.Store(3)
	fs := errorfs.Wrap(vfs.NewMem(), errorfs.InjectorFunc(func(op errorfs.Op, path string) error {
		if op == errorfs.OpCreate && filepath.Ext(path) == ".sst" && errorOps.Add(-1) >= 0 {
			return errorfs.ErrInjected
		}
		return nil
	}))
	d, err := Open("", testingRandomized(t, &Options{
		FS: fs,
		EventListener: &EventListener{
			BackgroundError: func(err error) {
				t.Log(err)
			},
		},
	}).WithFSDefaults())
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("a"), []byte("foo"), NoSync))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Close())
}

func TestAdjustGrandparentOverlapBytesForFlush(t *testing.T) {
	// 500MB in Lbase
	var lbaseFiles []*manifest.FileMetadata
	const lbaseSize = 5 << 20
	for i := 0; i < 100; i++ {
		m := &manifest.FileMetadata{Size: lbaseSize, FileNum: FileNum(i)}
		m.InitPhysicalBacking()
		lbaseFiles =
			append(lbaseFiles, m)
	}
	const maxOutputFileSize = 2 << 20
	// 20MB max overlap, so flush split into 25 files.
	const maxOverlapBytes = 20 << 20
	ls := manifest.NewLevelSliceSpecificOrder(lbaseFiles)
	testCases := []struct {
		flushingBytes        uint64
		adjustedOverlapBytes uint64
	}{
		// Flushes large enough that 25 files is acceptable.
		{flushingBytes: 128 << 20, adjustedOverlapBytes: 20971520},
		{flushingBytes: 64 << 20, adjustedOverlapBytes: 20971520},
		// Small increase in adjustedOverlapBytes.
		{flushingBytes: 32 << 20, adjustedOverlapBytes: 32768000},
		// Large increase in adjusterOverlapBytes, to limit to 4 files.
		{flushingBytes: 1 << 20, adjustedOverlapBytes: 131072000},
	}
	for _, tc := range testCases {
		t.Run("", func(t *testing.T) {
			c := compaction{
				grandparents:      ls,
				maxOverlapBytes:   maxOverlapBytes,
				maxOutputFileSize: maxOutputFileSize,
			}
			adjustGrandparentOverlapBytesForFlush(&c, tc.flushingBytes)
			require.Equal(t, tc.adjustedOverlapBytes, c.maxOverlapBytes)
		})
	}
}

func TestCompactionInvalidBounds(t *testing.T) {
	db, err := Open("", testingRandomized(t, &Options{
		FS: vfs.NewMem(),
	}).WithFSDefaults())
	require.NoError(t, err)
	defer db.Close()
	require.NoError(t, db.Compact([]byte("a"), []byte("b"), false))
	require.Error(t, db.Compact([]byte("a"), []byte("a"), false))
	require.Error(t, db.Compact([]byte("b"), []byte("a"), false))
}

func Test_calculateInuseKeyRanges(t *testing.T) {
	opts := (*Options)(nil).EnsureDefaults()
	cmp := base.DefaultComparer.Compare
	newFileMeta := func(fileNum FileNum, size uint64, smallest, largest base.InternalKey) *fileMetadata {
		m := (&fileMetadata{
			FileNum: fileNum,
			Size:    size,
		}).ExtendPointKeyBounds(opts.Comparer.Compare, smallest, largest)
		m.InitPhysicalBacking()
		return m
	}
	tests := []struct {
		name     string
		v        *version
		level    int
		depth    int
		smallest []byte
		largest  []byte
		want     []manifest.UserKeyRange
	}{
		{
			name: "No files in next level",
			v: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						1,
						1,
						base.ParseInternalKey("a.SET.2"),
						base.ParseInternalKey("c.SET.2"),
					),
					newFileMeta(
						2,
						1,
						base.ParseInternalKey("d.SET.2"),
						base.ParseInternalKey("e.SET.2"),
					),
				},
			}),
			level:    1,
			depth:    2,
			smallest: []byte("a"),
			largest:  []byte("e"),
			want: []manifest.UserKeyRange{
				{
					Start: []byte("a"),
					End:   []byte("c"),
				},
				{
					Start: []byte("d"),
					End:   []byte("e"),
				},
			},
		},
		{
			name: "No overlapping key ranges",
			v: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						1,
						1,
						base.ParseInternalKey("a.SET.1"),
						base.ParseInternalKey("c.SET.1"),
					),
					newFileMeta(
						2,
						1,
						base.ParseInternalKey("l.SET.1"),
						base.ParseInternalKey("p.SET.1"),
					),
				},
				2: {
					newFileMeta(
						3,
						1,
						base.ParseInternalKey("d.SET.1"),
						base.ParseInternalKey("i.SET.1"),
					),
					newFileMeta(
						4,
						1,
						base.ParseInternalKey("s.SET.1"),
						base.ParseInternalKey("w.SET.1"),
					),
				},
			}),
			level:    1,
			depth:    2,
			smallest: []byte("a"),
			largest:  []byte("z"),
			want: []manifest.UserKeyRange{
				{
					Start: []byte("a"),
					End:   []byte("c"),
				},
				{
					Start: []byte("d"),
					End:   []byte("i"),
				},
				{
					Start: []byte("l"),
					End:   []byte("p"),
				},
				{
					Start: []byte("s"),
					End:   []byte("w"),
				},
			},
		},
		{
			name: "First few non-overlapping, followed by overlapping",
			v: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						1,
						1,
						base.ParseInternalKey("a.SET.1"),
						base.ParseInternalKey("c.SET.1"),
					),
					newFileMeta(
						2,
						1,
						base.ParseInternalKey("d.SET.1"),
						base.ParseInternalKey("e.SET.1"),
					),
					newFileMeta(
						3,
						1,
						base.ParseInternalKey("n.SET.1"),
						base.ParseInternalKey("o.SET.1"),
					),
					newFileMeta(
						4,
						1,
						base.ParseInternalKey("p.SET.1"),
						base.ParseInternalKey("q.SET.1"),
					),
				},
				2: {
					newFileMeta(
						5,
						1,
						base.ParseInternalKey("m.SET.1"),
						base.ParseInternalKey("q.SET.1"),
					),
					newFileMeta(
						6,
						1,
						base.ParseInternalKey("s.SET.1"),
						base.ParseInternalKey("w.SET.1"),
					),
				},
			}),
			level:    1,
			depth:    2,
			smallest: []byte("a"),
			largest:  []byte("z"),
			want: []manifest.UserKeyRange{
				{
					Start: []byte("a"),
					End:   []byte("c"),
				},
				{
					Start: []byte("d"),
					End:   []byte("e"),
				},
				{
					Start: []byte("m"),
					End:   []byte("q"),
				},
				{
					Start: []byte("s"),
					End:   []byte("w"),
				},
			},
		},
		{
			name: "All overlapping",
			v: newVersion(opts, [numLevels][]*fileMetadata{
				1: {
					newFileMeta(
						1,
						1,
						base.ParseInternalKey("d.SET.1"),
						base.ParseInternalKey("e.SET.1"),
					),
					newFileMeta(
						2,
						1,
						base.ParseInternalKey("n.SET.1"),
						base.ParseInternalKey("o.SET.1"),
					),
					newFileMeta(
						3,
						1,
						base.ParseInternalKey("p.SET.1"),
						base.ParseInternalKey("q.SET.1"),
					),
				},
				2: {
					newFileMeta(
						4,
						1,
						base.ParseInternalKey("a.SET.1"),
						base.ParseInternalKey("c.SET.1"),
					),
					newFileMeta(
						5,
						1,
						base.ParseInternalKey("d.SET.1"),
						base.ParseInternalKey("w.SET.1"),
					),
				},
			}),
			level:    1,
			depth:    2,
			smallest: []byte("a"),
			largest:  []byte("z"),
			want: []manifest.UserKeyRange{
				{
					Start: []byte("a"),
					End:   []byte("c"),
				},
				{
					Start: []byte("d"),
					End:   []byte("w"),
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := calculateInuseKeyRanges(tt.v, cmp, tt.level, tt.depth, tt.smallest, tt.largest); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("calculateInuseKeyRanges() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestMarkedForCompaction(t *testing.T) {
	var mem vfs.FS = vfs.NewMem()
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()

	var buf bytes.Buffer
	opts := (&Options{
		FS:                          mem,
		DebugCheck:                  DebugCheckLevels,
		DisableAutomaticCompactions: true,
		FormatMajorVersion:          internalFormatNewest,
		EventListener: &EventListener{
			CompactionEnd: func(info CompactionInfo) {
				// Fix the job ID and durations for determinism.
				info.JobID = 100
				info.Duration = time.Second
				info.TotalDuration = 2 * time.Second
				fmt.Fprintln(&buf, info)
			},
		},
	}).WithFSDefaults()

	reset := func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
	}
	datadriven.RunTest(t, "testdata/marked_for_compaction", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""

		case "define":
			if d != nil {
				if err := d.Close(); err != nil {
					return err.Error()
				}
			}
			var err error
			if d, err = runDBDefineCmd(td, opts); err != nil {
				return err.Error()
			}
			d.mu.Lock()
			defer d.mu.Unlock()
			t := time.Now()
			d.timeNow = func() time.Time {
				t = t.Add(time.Second)
				return t
			}
			s := d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
			return s

		case "mark-for-compaction":
			d.mu.Lock()
			defer d.mu.Unlock()
			vers := d.mu.versions.currentVersion()
			var fileNum uint64
			td.ScanArgs(t, "file", &fileNum)
			for l, lm := range vers.Levels {
				iter := lm.Iter()
				for f := iter.First(); f != nil; f = iter.Next() {
					if f.FileNum != base.FileNum(fileNum) {
						continue
					}
					f.MarkedForCompaction = true
					vers.Stats.MarkedForCompaction++
					vers.Levels[l].InvalidateAnnotation(markedForCompactionAnnotator{})
					return fmt.Sprintf("marked L%d.%s", l, f.FileNum)
				}
			}
			return "not-found"

		case "maybe-compact":
			d.mu.Lock()
			defer d.mu.Unlock()
			d.opts.DisableAutomaticCompactions = false
			d.maybeScheduleCompaction()
			for d.mu.compact.compactingCount > 0 {
				d.mu.compact.cond.Wait()
			}

			fmt.Fprintln(&buf, d.mu.versions.currentVersion().DebugString(base.DefaultFormatter))
			s := strings.TrimSpace(buf.String())
			buf.Reset()
			opts.DisableAutomaticCompactions = true
			return s

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

// createManifestErrorInjector injects errors (when enabled) into vfs.FS calls
// to create MANIFEST files.
type createManifestErrorInjector struct {
	enabled atomic.Bool
}

// enable enables error injection for the vfs.FS.
func (i *createManifestErrorInjector) enable() {
	i.enabled.Store(true)
}

// MaybeError implements errorfs.Injector.
func (i *createManifestErrorInjector) MaybeError(op errorfs.Op, path string) error {
	if !i.enabled.Load() {
		return nil
	}
	// This necessitates having a MaxManifestSize of 1, to reliably induce
	// logAndApply errors.
	if strings.Contains(path, "MANIFEST") && op == errorfs.OpCreate {
		return errorfs.ErrInjected
	}
	return nil
}

var _ errorfs.Injector = &createManifestErrorInjector{}

// TestCompaction_LogAndApplyFails exercises a flush or ingest encountering an
// unrecoverable error during logAndApply.
//
// Regression test for #1669.
func TestCompaction_LogAndApplyFails(t *testing.T) {
	// flushKeys writes the given keys to the DB, flushing the resulting memtable.
	var key = []byte("foo")
	flushErrC := make(chan error)
	flushKeys := func(db *DB) error {
		b := db.NewBatch()
		err := b.Set(key, nil, nil)
		require.NoError(t, err)
		err = b.Commit(nil)
		require.NoError(t, err)
		// An error from a failing flush is returned asynchronously.
		go func() { _ = db.Flush() }()
		return <-flushErrC
	}

	// ingestKeys adds the given keys to the DB via an ingestion.
	ingestKeys := func(db *DB) error {
		// Create an SST for ingestion.
		const fName = "ext"
		f, err := db.opts.FS.Create(fName)
		require.NoError(t, err)
		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
		require.NoError(t, w.Set(key, nil))
		require.NoError(t, w.Close())
		// Ingest the SST.
		return db.Ingest([]string{fName})
	}

	testCases := []struct {
		name              string
		addFn             func(db *DB) error
		backgroundErrorFn func(*DB, error)
	}{
		{
			name:  "flush",
			addFn: flushKeys,
			backgroundErrorFn: func(db *DB, err error) {
				require.True(t, errors.Is(err, errorfs.ErrInjected))
				flushErrC <- err
				// A flush will attempt to retry in the background. For the purposes of
				// testing this particular scenario, where we would have crashed anyway,
				// drop the memtable on the floor to short circuit the retry loop.
				// NB: we hold db.mu here.
				var cur *flushableEntry
				cur, db.mu.mem.queue = db.mu.mem.queue[0], db.mu.mem.queue[1:]
				cur.readerUnrefLocked(true)
			},
		},
		{
			name:  "ingest",
			addFn: ingestKeys,
		},
	}

	runTest := func(t *testing.T, addFn func(db *DB) error, bgFn func(*DB, error)) {
		var db *DB
		inj := &createManifestErrorInjector{}
		logger := &fatalCapturingLogger{t: t}
		opts := (&Options{
			FS: errorfs.Wrap(vfs.NewMem(), inj),
			// Rotate the manifest after each write. This is required to trigger a
			// file creation, into which errors can be injected.
			MaxManifestFileSize: 1,
			Logger:              logger,
			EventListener: &EventListener{
				BackgroundError: func(err error) {
					if bgFn != nil {
						bgFn(db, err)
					}
				},
			},
			DisableAutomaticCompactions: true,
		}).WithFSDefaults()

		db, err := Open("", opts)
		require.NoError(t, err)
		defer func() { _ = db.Close() }()

		inj.enable()
		err = addFn(db)
		require.True(t, errors.Is(err, errorfs.ErrInjected))

		// Under normal circumstances, such an error in logAndApply would panic and
		// cause the DB to terminate here. Assert that we captured the fatal error.
		require.True(t, errors.Is(logger.err, errorfs.ErrInjected))
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			runTest(t, tc.addFn, tc.backgroundErrorFn)
		})
	}
}

// TestSharedObjectDeletePacing tests that we don't throttle shared object
// deletes (see the TargetBytesDeletionRate option).
func TestSharedObjectDeletePacing(t *testing.T) {
	var opts Options
	opts.FS = vfs.NewMem()
	opts.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
		"": remote.NewInMem(),
	})
	opts.Experimental.CreateOnShared = remote.CreateOnSharedAll
	opts.TargetByteDeletionRate = 1

	d, err := Open("", &opts)
	require.NoError(t, err)
	require.NoError(t, d.SetCreatorID(1))

	randVal := func() []byte {
		res := make([]byte, 1024)
		_, err := crand.Read(res)
		require.NoError(t, err)
		return res
	}

	// We must set up things so that we will have more live bytes than obsolete
	// bytes, otherwise delete pacing will be disabled anyway.
	key := func(i int) string {
		return fmt.Sprintf("k%02d", i)
	}
	const numKeys = 20
	for i := 1; i <= numKeys; i++ {
		require.NoError(t, d.Set([]byte(key(i)), randVal(), nil))
		require.NoError(t, d.Compact([]byte(key(i)), []byte(key(i)+"1"), false))
	}

	done := make(chan struct{})
	go func() {
		err = d.DeleteRange([]byte(key(5)), []byte(key(9)), nil)
		if err == nil {
			err = d.Compact([]byte(key(5)), []byte(key(9)), false)
		}
		// Wait for objects to be deleted.
		for {
			time.Sleep(10 * time.Millisecond)
			if len(d.objProvider.List()) < numKeys-2 {
				break
			}
		}
		close(done)
	}()

	select {
	case <-time.After(60 * time.Second):
		// Don't close the DB in this case (the goroutine above might panic).
		t.Fatalf("compaction timed out, possibly due to incorrect deletion pacing")
	case <-done:
	}
	require.NoError(t, err)
	d.Close()
}

type WriteErrorInjector struct {
	enabled atomic.Bool
}

// enable enables error injection for the vfs.FS.
func (i *WriteErrorInjector) enable() {
	i.enabled.Store(true)
}

// disable disabled error injection for the vfs.FS.
func (i *WriteErrorInjector) disable() {
	i.enabled.Store(false)
}

// MaybeError implements errorfs.Injector.
func (i *WriteErrorInjector) MaybeError(op errorfs.Op, path string) error {
	if !i.enabled.Load() {
		return nil
	}
	// Fail any future write.
	if op == errorfs.OpFileWrite {
		return errorfs.ErrInjected
	}
	return nil
}

var _ errorfs.Injector = &WriteErrorInjector{}

// Cumulative compaction stats shouldn't be updated on compaction error.
func TestCompactionErrorStats(t *testing.T) {
	// protected by d.mu
	var (
		useInjector   bool
		tablesCreated []FileNum
	)

	mem := vfs.NewMem()
	injector := &WriteErrorInjector{}
	opts := (&Options{
		FS:     errorfs.Wrap(mem, injector),
		Levels: make([]LevelOptions, numLevels),
		EventListener: &EventListener{
			TableCreated: func(info TableCreateInfo) {
				t.Log(info)

				if useInjector {
					// We'll write 3 tables during compaction, and we only need
					// the writes to error on the third file write, so only enable
					// the injector after the first two files have been written to.
					tablesCreated = append(tablesCreated, info.FileNum)
					if len(tablesCreated) >= 2 {
						injector.enable()
					}
				}
			},
		},
	}).WithFSDefaults()
	for i := range opts.Levels {
		opts.Levels[i].TargetFileSize = 1
	}
	opts.testingRandomized(t)
	d, err := Open("", opts)
	require.NoError(t, err)

	ingest := func(keys ...string) {
		t.Helper()
		f, err := mem.Create("ext")
		require.NoError(t, err)

		w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
			TableFormat: d.FormatMajorVersion().MaxTableFormat(),
		})
		for _, k := range keys {
			require.NoError(t, w.Set([]byte(k), nil))
		}
		require.NoError(t, w.Close())
		require.NoError(t, d.Ingest([]string{"ext"}))
	}
	ingest("a", "c")
	// Snapshot will preserve the older "a" key during compaction.
	snap := d.NewSnapshot()
	ingest("a", "b")

	// Trigger a manual compaction, which will encounter an injected error
	// after the second table is created.
	d.mu.Lock()
	useInjector = true
	d.mu.Unlock()

	err = d.Compact([]byte("a"), []byte("d"), false)
	require.Error(t, err, "injected error")

	// Due to the error, stats shouldn't have been updated.
	d.mu.Lock()
	require.Equal(t, 0, int(d.mu.snapshots.cumulativePinnedCount))
	require.Equal(t, 0, int(d.mu.snapshots.cumulativePinnedSize))
	useInjector = false
	d.mu.Unlock()

	injector.disable()

	// The following compaction won't error, but snapshot is open, so snapshot
	// pinned stats should update.
	require.NoError(t, d.Compact([]byte("a"), []byte("d"), false))
	require.NoError(t, snap.Close())

	d.mu.Lock()
	require.Equal(t, 1, int(d.mu.snapshots.cumulativePinnedCount))
	require.Equal(t, 9, int(d.mu.snapshots.cumulativePinnedSize))
	d.mu.Unlock()
	require.NoError(t, d.Close())
}
