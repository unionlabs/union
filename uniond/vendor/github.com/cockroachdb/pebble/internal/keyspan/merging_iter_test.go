// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"math/rand"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/stretchr/testify/require"
)

func TestMergingIter(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	var iter MergingIter

	datadriven.RunTest(t, "testdata/merging_iter", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			snapshot := base.InternalKeySeqNumMax
			for _, cmdArg := range td.CmdArgs {
				switch cmdArg.Key {
				case "snapshot":
					var err error
					snapshot, err = strconv.ParseUint(cmdArg.Vals[0], 10, 64)
					require.NoError(t, err)
				default:
					return fmt.Sprintf("unrecognized arg %q", cmdArg.Key)
				}
			}

			var iters []FragmentIterator
			var spans []Span
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				if line == "--" {
					iters = append(iters, NewIter(cmp, spans))
					spans = nil
					continue
				}
				spans = append(spans, ParseSpan(line))
			}
			if len(spans) > 0 {
				iters = append(iters, &invalidatingIter{iter: NewIter(cmp, spans)})
			}
			iter.Init(cmp, VisibleTransform(snapshot), new(MergingBuffers), iters...)
			return fmt.Sprintf("%d levels", len(iters))
		case "iter":
			return runIterCmd(t, td, &iter)

		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

// TestMergingIter_FragmenterEquivalence tests for equivalence between the
// fragmentation performed on-the-fly by the MergingIter and the fragmentation
// performed by the Fragmenter.
//
// It does this by producing 1-10 levels of well-formed fragments. Generated
// fragments may overlap other levels arbitrarily, but within their level
// generated fragments may only overlap other fragments that share the same user
// key bounds.
//
// The test then feeds all the fragments, across all levels, into a Fragmenter
// and produces a Iter over those fragments. The test also constructs a
// MergingIter with a separate Iter for each level. It runs a random
// series of operations, applying each operation to both. It asserts that each
// operation has identical results on both iterators.
func TestMergingIter_FragmenterEquivalence(t *testing.T) {
	seed := time.Now().UnixNano()
	for i := int64(0); i < 10; i++ {
		testFragmenterEquivalenceOnce(t, seed+i)
	}
}

func TestMergingIter_FragmenterEquivalence_Seed(t *testing.T) {
	// This test uses a fixed seed. It's useful to manually edit its seed when
	// debugging a test failure of the variable-seed test.
	const seed = 1644517830186873000
	testFragmenterEquivalenceOnce(t, seed)
}

func testFragmenterEquivalenceOnce(t *testing.T, seed int64) {
	cmp := testkeys.Comparer.Compare
	rng := rand.New(rand.NewSource(seed))
	t.Logf("seed = %d", seed)

	// Use a key space of alphanumeric strings, with a random max length between
	// 1-3. Repeat keys are more common at the lower max lengths.
	ks := testkeys.Alpha(rng.Intn(3) + 1)

	// Generate between 1 and 10 levels of fragment iterators.
	levels := make([][]Span, rng.Intn(10)+1)
	iters := make([]FragmentIterator, len(levels))
	var allSpans []Span
	var buf bytes.Buffer
	for l := 0; l < len(levels); l++ {
		fmt.Fprintf(&buf, "level %d: ", l)
		for keyspaceStartIdx := int64(0); keyspaceStartIdx < ks.Count(); {
			// Generate spans of lengths of up to a third of the keyspace.
			spanStartIdx := keyspaceStartIdx + rng.Int63n(ks.Count()/3)
			spanEndIdx := spanStartIdx + rng.Int63n(ks.Count()/3) + 1

			if spanEndIdx < ks.Count() {
				keyCount := uint64(rng.Intn(3) + 1)
				s := Span{
					Start: testkeys.Key(ks, spanStartIdx),
					End:   testkeys.Key(ks, spanEndIdx),
					Keys:  make([]Key, 0, keyCount),
				}
				for k := keyCount; k > 0; k-- {
					seqNum := uint64((len(levels)-l)*3) + k
					s.Keys = append(s.Keys, Key{
						Trailer: base.MakeTrailer(seqNum, base.InternalKeyKindRangeKeySet),
					})
				}
				if len(levels[l]) > 0 {
					fmt.Fprint(&buf, ", ")
				}
				fmt.Fprintf(&buf, "%s", s)

				levels[l] = append(levels[l], s)
				allSpans = append(allSpans, s)
			}
			keyspaceStartIdx = spanEndIdx
		}
		iters[l] = &invalidatingIter{iter: NewIter(cmp, levels[l])}
		fmt.Fprintln(&buf)
	}

	// Fragment the spans across the levels.
	var allFragmented []Span
	f := Fragmenter{
		Cmp:    cmp,
		Format: testkeys.Comparer.FormatKey,
		Emit: func(span Span) {
			allFragmented = append(allFragmented, span)
		},
	}
	Sort(f.Cmp, allSpans)
	for _, s := range allSpans {
		f.Add(s)
	}
	f.Finish()

	// Log all the levels and their fragments, as well as the fully-fragmented
	// spans produced by the Fragmenter.
	fmt.Fprintln(&buf, "Fragmenter fragments:")
	for i, s := range allFragmented {
		if i > 0 {
			fmt.Fprint(&buf, ", ")
		}
		fmt.Fprint(&buf, s)
	}
	t.Logf("%d levels:\n%s\n", len(levels), buf.String())

	fragmenterIter := NewIter(f.Cmp, allFragmented)
	mergingIter := &MergingIter{}
	mergingIter.Init(f.Cmp, VisibleTransform(base.InternalKeySeqNumMax), new(MergingBuffers), iters...)

	// Position both so that it's okay to perform relative positioning
	// operations immediately.
	mergingIter.First()
	fragmenterIter.First()

	type opKind struct {
		weight int
		fn     func() (str string, f *Span, m *Span)
	}
	ops := []opKind{
		{weight: 2, fn: func() (string, *Span, *Span) {
			return "First()", fragmenterIter.First(), mergingIter.First()
		}},
		{weight: 2, fn: func() (string, *Span, *Span) {
			return "Last()", fragmenterIter.Last(), mergingIter.Last()
		}},
		{weight: 5, fn: func() (string, *Span, *Span) {
			k := testkeys.Key(ks, rng.Int63n(ks.Count()))
			return fmt.Sprintf("SeekGE(%q)", k),
				fragmenterIter.SeekGE(k),
				mergingIter.SeekGE(k)
		}},
		{weight: 5, fn: func() (string, *Span, *Span) {
			k := testkeys.Key(ks, rng.Int63n(ks.Count()))
			return fmt.Sprintf("SeekLT(%q)", k),
				fragmenterIter.SeekLT(k),
				mergingIter.SeekLT(k)
		}},
		{weight: 50, fn: func() (string, *Span, *Span) {
			return "Next()", fragmenterIter.Next(), mergingIter.Next()
		}},
		{weight: 50, fn: func() (string, *Span, *Span) {
			return "Prev()", fragmenterIter.Prev(), mergingIter.Prev()
		}},
	}
	var totalWeight int
	for _, op := range ops {
		totalWeight += op.weight
	}

	var fragmenterBuf bytes.Buffer
	var mergingBuf bytes.Buffer
	opCount := rng.Intn(200) + 50
	for i := 0; i < opCount; i++ {
		p := rng.Intn(totalWeight)
		opIndex := 0
		for i, op := range ops {
			if p < op.weight {
				opIndex = i
				break
			}
			p -= op.weight
		}

		opString, fs, ms := ops[opIndex].fn()

		fragmenterBuf.Reset()
		mergingBuf.Reset()
		fmt.Fprint(&fragmenterBuf, fs)
		fmt.Fprint(&mergingBuf, ms)
		if fragmenterBuf.String() != mergingBuf.String() {
			t.Fatalf("seed %d, op %d: %s = %s, fragmenter iterator returned %s",
				seed, i, opString, mergingBuf.String(), fragmenterBuf.String())
		}
		t.Logf("op %d: %s = %s", i, opString, fragmenterBuf.String())
	}
}
