// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"
	"strings"
	"sync"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/stretchr/testify/require"
)

func levelMetadata(level int, files ...*FileMetadata) LevelMetadata {
	return makeLevelMetadata(base.DefaultComparer.Compare, level, files)
}

func ikey(s string) InternalKey {
	return base.MakeInternalKey([]byte(s), 0, base.InternalKeyKindSet)
}

func TestIkeyRange(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	testCases := []struct {
		input, want string
	}{
		{
			"",
			"-",
		},
		{
			"a-e",
			"a-e",
		},
		{
			"a-e a-e",
			"a-e",
		},
		{
			"c-g a-e",
			"a-g",
		},
		{
			"a-e c-g a-e",
			"a-g",
		},
		{
			"b-d f-g",
			"b-g",
		},
		{
			"d-e b-d",
			"b-e",
		},
		{
			"e-e",
			"e-e",
		},
		{
			"f-g e-e d-e c-g b-d a-e",
			"a-g",
		},
	}
	for _, tc := range testCases {
		var f []*FileMetadata
		if tc.input != "" {
			for i, s := range strings.Split(tc.input, " ") {
				m := (&FileMetadata{
					FileNum: base.FileNum(i),
				}).ExtendPointKeyBounds(cmp, ikey(s[0:1]), ikey(s[2:3]))
				m.InitPhysicalBacking()
				f = append(f, m)
			}
		}
		levelMetadata := makeLevelMetadata(base.DefaultComparer.Compare, 0, f)

		sm, la := KeyRange(base.DefaultComparer.Compare, levelMetadata.Iter())
		got := string(sm.UserKey) + "-" + string(la.UserKey)
		if got != tc.want {
			t.Errorf("KeyRange(%q) = %q, %q", tc.input, got, tc.want)
		}
	}
}

func TestOverlaps(t *testing.T) {
	var v *Version
	cmp := testkeys.Comparer.Compare
	fmtKey := testkeys.Comparer.FormatKey
	datadriven.RunTest(t, "testdata/overlaps", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			var err error
			v, err = ParseVersionDebug(cmp, fmtKey, 64>>10 /* flush split bytes */, d.Input)
			if err != nil {
				return err.Error()
			}
			return v.String()
		case "overlaps":
			var level int
			var start, end string
			var exclusiveEnd bool
			d.ScanArgs(t, "level", &level)
			d.ScanArgs(t, "start", &start)
			d.ScanArgs(t, "end", &end)
			d.ScanArgs(t, "exclusive-end", &exclusiveEnd)
			overlaps := v.Overlaps(level, testkeys.Comparer.Compare, []byte(start), []byte(end), exclusiveEnd)
			var buf bytes.Buffer
			fmt.Fprintf(&buf, "%d files:\n", overlaps.Len())
			overlaps.Each(func(f *FileMetadata) {
				fmt.Fprintf(&buf, "%s\n", f.DebugString(base.DefaultFormatter, false))
			})
			return buf.String()
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestContains(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	newFileMeta := func(fileNum base.FileNum, size uint64, smallest, largest base.InternalKey) *FileMetadata {
		m := (&FileMetadata{
			FileNum: fileNum,
			Size:    size,
		}).ExtendPointKeyBounds(cmp, smallest, largest)
		m.InitPhysicalBacking()
		return m
	}
	m00 := newFileMeta(
		700,
		1,
		base.ParseInternalKey("b.SET.7008"),
		base.ParseInternalKey("e.SET.7009"),
	)
	m01 := newFileMeta(
		701,
		1,
		base.ParseInternalKey("c.SET.7018"),
		base.ParseInternalKey("f.SET.7019"),
	)
	m02 := newFileMeta(
		702,
		1,
		base.ParseInternalKey("f.SET.7028"),
		base.ParseInternalKey("g.SET.7029"),
	)
	m03 := newFileMeta(
		703,
		1,
		base.ParseInternalKey("x.SET.7038"),
		base.ParseInternalKey("y.SET.7039"),
	)
	m04 := newFileMeta(
		704,
		1,
		base.ParseInternalKey("n.SET.7048"),
		base.ParseInternalKey("p.SET.7049"),
	)
	m05 := newFileMeta(
		705,
		1,
		base.ParseInternalKey("p.SET.7058"),
		base.ParseInternalKey("p.SET.7059"),
	)
	m06 := newFileMeta(
		706,
		1,
		base.ParseInternalKey("p.SET.7068"),
		base.ParseInternalKey("u.SET.7069"),
	)
	m07 := newFileMeta(
		707,
		1,
		base.ParseInternalKey("r.SET.7078"),
		base.ParseInternalKey("s.SET.7079"),
	)

	m10 := newFileMeta(
		710,
		1,
		base.ParseInternalKey("d.SET.7108"),
		base.ParseInternalKey("g.SET.7109"),
	)
	m11 := newFileMeta(
		711,
		1,
		base.ParseInternalKey("g.SET.7118"),
		base.ParseInternalKey("j.SET.7119"),
	)
	m12 := newFileMeta(
		712,
		1,
		base.ParseInternalKey("n.SET.7128"),
		base.ParseInternalKey("p.SET.7129"),
	)
	m13 := newFileMeta(
		713,
		1,
		base.ParseInternalKey("p.SET.7148"),
		base.ParseInternalKey("p.SET.7149"),
	)
	m14 := newFileMeta(
		714,
		1,
		base.ParseInternalKey("p.SET.7138"),
		base.ParseInternalKey("u.SET.7139"),
	)

	v := Version{
		Levels: [NumLevels]LevelMetadata{
			0: levelMetadata(0, m00, m01, m02, m03, m04, m05, m06, m07),
			1: levelMetadata(1, m10, m11, m12, m13, m14),
		},
	}

	testCases := []struct {
		level int
		file  *FileMetadata
		want  bool
	}{
		// Level 0: m00=b-e, m01=c-f, m02=f-g, m03=x-y, m04=n-p, m05=p-p, m06=p-u, m07=r-s.
		// Note that:
		//   - the slice isn't sorted (e.g. m02=f-g, m03=x-y, m04=n-p),
		//   - m00 and m01 overlap (not just touch),
		//   - m06 contains m07,
		//   - m00, m01 and m02 transitively overlap/touch each other, and
		//   - m04, m05, m06 and m07 transitively overlap/touch each other.
		{0, m00, true},
		{0, m01, true},
		{0, m02, true},
		{0, m03, true},
		{0, m04, true},
		{0, m05, true},
		{0, m06, true},
		{0, m07, true},
		{0, m10, false},
		{0, m11, false},
		{0, m12, false},
		{0, m13, false},
		{0, m14, false},
		{1, m00, false},
		{1, m01, false},
		{1, m02, false},
		{1, m03, false},
		{1, m04, false},
		{1, m05, false},
		{1, m06, false},
		{1, m07, false},
		{1, m10, true},
		{1, m11, true},
		{1, m12, true},
		{1, m13, true},
		{1, m14, true},

		// Level 2: empty.
		{2, m00, false},
		{2, m14, false},
	}

	for _, tc := range testCases {
		got := v.Contains(tc.level, cmp, tc.file)
		if got != tc.want {
			t.Errorf("level=%d, file=%s\ngot %t\nwant %t", tc.level, tc.file, got, tc.want)
		}
	}
}

func TestVersionUnref(t *testing.T) {
	list := &VersionList{}
	list.Init(&sync.Mutex{})
	v := &Version{Deleted: func([]*FileBacking) {}}
	v.Ref()
	list.PushBack(v)
	v.Unref()
	if !list.Empty() {
		t.Fatalf("expected version list to be empty")
	}
}

func TestCheckOrdering(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey
	datadriven.RunTest(t, "testdata/version_check_ordering",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "check-ordering":
				orderingInvariants := ProhibitSplitUserKeys
				if d.HasArg("allow-split-user-keys") {
					orderingInvariants = AllowSplitUserKeys
				}
				v, err := ParseVersionDebug(cmp, fmtKey, 10<<20, d.Input)
				if err != nil {
					return err.Error()
				}
				// L0 files compare on sequence numbers. Use the seqnums from the
				// smallest / largest bounds for the table.
				v.Levels[0].Slice().Each(func(m *FileMetadata) {
					m.SmallestSeqNum = m.Smallest.SeqNum()
					m.LargestSeqNum = m.Largest.SeqNum()
				})
				if err = v.CheckOrdering(cmp, base.DefaultFormatter, orderingInvariants); err != nil {
					return err.Error()
				}
				return "OK"

			default:
				return fmt.Sprintf("unknown command: %s", d.Cmd)
			}
		})
}

func TestExtendBounds(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	parseBounds := func(line string) (lower, upper InternalKey) {
		parts := strings.Split(line, "-")
		if len(parts) == 1 {
			parts = strings.Split(parts[0], ":")
			start, end := strings.TrimSpace(parts[0]), strings.TrimSpace(parts[1])
			lower = base.ParseInternalKey(start)
			switch k := lower.Kind(); k {
			case base.InternalKeyKindRangeDelete:
				upper = base.MakeRangeDeleteSentinelKey([]byte(end))
			case base.InternalKeyKindRangeKeySet, base.InternalKeyKindRangeKeyUnset, base.InternalKeyKindRangeKeyDelete:
				upper = base.MakeExclusiveSentinelKey(k, []byte(end))
			default:
				panic(fmt.Sprintf("unknown kind %s with end key", k))
			}
		} else {
			l, u := strings.TrimSpace(parts[0]), strings.TrimSpace(parts[1])
			lower, upper = base.ParseInternalKey(l), base.ParseInternalKey(u)
		}
		return
	}
	format := func(m *FileMetadata) string {
		var b bytes.Buffer
		var smallest, largest string
		switch m.boundTypeSmallest {
		case boundTypePointKey:
			smallest = "point"
		case boundTypeRangeKey:
			smallest = "range"
		default:
			return fmt.Sprintf("unknown bound type %d", m.boundTypeSmallest)
		}
		switch m.boundTypeLargest {
		case boundTypePointKey:
			largest = "point"
		case boundTypeRangeKey:
			largest = "range"
		default:
			return fmt.Sprintf("unknown bound type %d", m.boundTypeLargest)
		}
		bounds, err := m.boundsMarker()
		if err != nil {
			panic(err)
		}
		fmt.Fprintf(&b, "%s\n", m.DebugString(base.DefaultFormatter, true))
		fmt.Fprintf(&b, "  bounds: (smallest=%s,largest=%s) (0x%08b)\n", smallest, largest, bounds)
		return b.String()
	}
	m := &FileMetadata{}
	datadriven.RunTest(t, "testdata/file_metadata_bounds", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "reset":
			m = &FileMetadata{}
			return ""
		case "extend-point-key-bounds":
			u, l := parseBounds(d.Input)
			m.ExtendPointKeyBounds(cmp, u, l)
			return format(m)
		case "extend-range-key-bounds":
			u, l := parseBounds(d.Input)
			m.ExtendRangeKeyBounds(cmp, u, l)
			return format(m)
		default:
			return fmt.Sprintf("unknown command %s\n", d.Cmd)
		}
	})
}

func TestFileMetadata_ParseRoundTrip(t *testing.T) {
	testCases := []struct {
		name   string
		input  string
		output string
	}{
		{
			name:  "point keys only",
			input: "000001:[a#0,SET-z#0,DEL] seqnums:[0-0] points:[a#0,SET-z#0,DEL]",
		},
		{
			name:  "range keys only",
			input: "000001:[a#0,RANGEKEYSET-z#0,RANGEKEYDEL] seqnums:[0-0] ranges:[a#0,RANGEKEYSET-z#0,RANGEKEYDEL]",
		},
		{
			name:  "point and range keys",
			input: "000001:[a#0,RANGEKEYSET-d#0,DEL] seqnums:[0-0] points:[b#0,SET-d#0,DEL] ranges:[a#0,RANGEKEYSET-c#0,RANGEKEYDEL]",
		},
		{
			name:  "point and range keys with nonzero senums",
			input: "000001:[a#3,RANGEKEYSET-d#4,DEL] seqnums:[3-7] points:[b#3,SET-d#4,DEL] ranges:[a#3,RANGEKEYSET-c#5,RANGEKEYDEL]",
		},
		{
			name:   "whitespace",
			input:  " 000001 : [ a#0,SET - z#0,DEL] points : [ a#0,SET - z#0,DEL] ",
			output: "000001:[a#0,SET-z#0,DEL] seqnums:[0-0] points:[a#0,SET-z#0,DEL]",
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			m, err := ParseFileMetadataDebug(tc.input)
			require.NoError(t, err)
			err = m.Validate(base.DefaultComparer.Compare, base.DefaultFormatter)
			require.NoError(t, err)
			got := m.DebugString(base.DefaultFormatter, true)
			want := tc.input
			if tc.output != "" {
				want = tc.output
			}
			require.Equal(t, want, got)
		})
	}
}
