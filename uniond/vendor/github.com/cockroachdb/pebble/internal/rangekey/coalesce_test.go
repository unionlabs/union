// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package rangekey

import (
	"bytes"
	"fmt"
	"io"
	"math"
	"math/rand"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/pmezard/go-difflib/difflib"
	"github.com/stretchr/testify/require"
)

func TestCoalesce(t *testing.T) {
	var buf bytes.Buffer
	eq := testkeys.Comparer.Equal
	cmp := testkeys.Comparer.Compare

	datadriven.RunTest(t, "testdata/coalesce", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "coalesce":
			buf.Reset()
			span := keyspan.ParseSpan(td.Input)
			coalesced := keyspan.Span{
				Start: span.Start,
				End:   span.End,
			}
			if err := Coalesce(cmp, eq, span.Keys, &coalesced.Keys); err != nil {
				return err.Error()
			}
			fmt.Fprintln(&buf, coalesced)
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestIter(t *testing.T) {
	eq := testkeys.Comparer.Equal
	cmp := testkeys.Comparer.Compare
	var iter keyspan.MergingIter
	var buf bytes.Buffer

	datadriven.RunTest(t, "testdata/iter", func(t *testing.T, td *datadriven.TestData) string {
		buf.Reset()
		switch td.Cmd {
		case "define":
			visibleSeqNum := base.InternalKeySeqNumMax
			for _, arg := range td.CmdArgs {
				if arg.Key == "visible-seq-num" {
					var err error
					visibleSeqNum, err = strconv.ParseUint(arg.Vals[0], 10, 64)
					require.NoError(t, err)
				}
			}

			var spans []keyspan.Span
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				spans = append(spans, keyspan.ParseSpan(line))
			}
			transform := keyspan.TransformerFunc(func(cmp base.Compare, s keyspan.Span, dst *keyspan.Span) error {
				keysBySuffix := keyspan.KeysBySuffix{
					Cmp:  cmp,
					Keys: dst.Keys[:0],
				}
				if err := coalesce(eq, &keysBySuffix, visibleSeqNum, s.Keys); err != nil {
					return err
				}
				// Update the span with the (potentially reduced) keys slice.  coalesce left
				// the keys in *dst sorted by suffix. Re-sort them by trailer.
				dst.Keys = keysBySuffix.Keys
				keyspan.SortKeysByTrailer(&dst.Keys)
				dst.Start = s.Start
				dst.End = s.End
				return nil
			})
			iter.Init(cmp, transform, new(keyspan.MergingBuffers), keyspan.NewIter(cmp, spans))
			return "OK"
		case "iter":
			buf.Reset()
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				line = strings.TrimSpace(line)
				i := strings.IndexByte(line, ' ')
				iterCmd := line
				if i > 0 {
					iterCmd = string(line[:i])
				}
				var s *keyspan.Span
				switch iterCmd {
				case "first":
					s = iter.First()
				case "last":
					s = iter.Last()
				case "next":
					s = iter.Next()
				case "prev":
					s = iter.Prev()
				case "seek-ge":
					s = iter.SeekGE([]byte(strings.TrimSpace(line[i:])))
				case "seek-lt":
					s = iter.SeekLT([]byte(strings.TrimSpace(line[i:])))
				default:
					return fmt.Sprintf("unrecognized iter command %q", iterCmd)
				}
				require.NoError(t, iter.Error())
				fmt.Fprint(&buf, s)
				if buf.Len() > 0 {
					fmt.Fprintln(&buf)
				}
			}
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestDefragmenting(t *testing.T) {
	cmp := testkeys.Comparer.Compare

	var buf bytes.Buffer
	var spans []keyspan.Span
	var hasPrefix bool
	var prefix []byte
	datadriven.RunTest(t, "testdata/defragmenting_iter", func(t *testing.T, td *datadriven.TestData) string {
		buf.Reset()
		switch td.Cmd {
		case "define":
			spans = spans[:0]
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				spans = append(spans, keyspan.ParseSpan(line))
			}
			return ""
		case "iter":
			var userIterCfg UserIteratorConfig
			iter := userIterCfg.Init(testkeys.Comparer, base.InternalKeySeqNumMax,
				nil /* lower */, nil, /* upper */
				&hasPrefix, &prefix, false /* internalKeys */, new(Buffers),
				keyspan.NewIter(cmp, spans))
			for _, line := range strings.Split(td.Input, "\n") {
				runIterOp(&buf, iter, line)
			}
			return strings.TrimSpace(buf.String())
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestDefragmentingIter_Randomized(t *testing.T) {
	seed := time.Now().UnixNano()
	for i := int64(0); i < 100; i++ {
		testDefragmentingIteRandomizedOnce(t, seed+i)
	}
}

func TestDefragmentingIter_RandomizedFixedSeed(t *testing.T) {
	const seed = 1648173101214881000
	testDefragmentingIteRandomizedOnce(t, seed)
}

func testDefragmentingIteRandomizedOnce(t *testing.T, seed int64) {
	cmp := testkeys.Comparer.Compare
	formatKey := testkeys.Comparer.FormatKey

	rng := rand.New(rand.NewSource(seed))
	t.Logf("seed = %d", seed)

	// Use a key space of alphanumeric strings, with a random max length between
	// 1-2. Repeat keys are more common at the lower max lengths.
	ks := testkeys.Alpha(rng.Intn(2) + 1)

	// Generate between 1-15 range keys.
	const maxRangeKeys = 15
	var original, fragmented []keyspan.Span
	numRangeKeys := 1 + rng.Intn(maxRangeKeys)
	for i := 0; i < numRangeKeys; i++ {
		startIdx := rng.Int63n(ks.Count())
		endIdx := rng.Int63n(ks.Count())
		for startIdx == endIdx {
			endIdx = rng.Int63n(ks.Count())
		}
		if startIdx > endIdx {
			startIdx, endIdx = endIdx, startIdx
		}

		key := keyspan.Key{
			Trailer: base.MakeTrailer(uint64(i), base.InternalKeyKindRangeKeySet),
			Value:   []byte(fmt.Sprintf("v%d", rng.Intn(3))),
		}
		// Generate suffixes 0, 1, 2, or 3 with 0 indicating none.
		if suffix := rng.Int63n(4); suffix > 0 {
			key.Suffix = testkeys.Suffix(suffix)
		}
		original = append(original, keyspan.Span{
			Start: testkeys.Key(ks, startIdx),
			End:   testkeys.Key(ks, endIdx),
			Keys:  []keyspan.Key{key},
		})

		for startIdx < endIdx {
			width := rng.Int63n(endIdx-startIdx) + 1
			fragmented = append(fragmented, keyspan.Span{
				Start: testkeys.Key(ks, startIdx),
				End:   testkeys.Key(ks, startIdx+width),
				Keys:  []keyspan.Key{key},
			})
			startIdx += width
		}
	}

	// Both the original and the deliberately fragmented spans may contain
	// overlaps, so we need to sort and fragment them.
	original = fragment(cmp, formatKey, original)
	fragmented = fragment(cmp, formatKey, fragmented)

	var referenceCfg, fragmentedCfg UserIteratorConfig
	referenceIter := referenceCfg.Init(testkeys.Comparer, base.InternalKeySeqNumMax,
		nil /* lower */, nil, /* upper */
		new(bool), new([]byte), false /* internalKeys */, new(Buffers),
		keyspan.NewIter(cmp, original))
	fragmentedIter := fragmentedCfg.Init(testkeys.Comparer, base.InternalKeySeqNumMax,
		nil /* lower */, nil, /* upper */
		new(bool), new([]byte), false /* internalKeys */, new(Buffers),
		keyspan.NewIter(cmp, fragmented))

	// Generate 100 random operations and run them against both iterators.
	const numIterOps = 100
	type opKind struct {
		weight int
		fn     func() string
	}
	ops := []opKind{
		{weight: 2, fn: func() string { return "first" }},
		{weight: 2, fn: func() string { return "last" }},
		{weight: 50, fn: func() string { return "next" }},
		{weight: 50, fn: func() string { return "prev" }},
		{weight: 5, fn: func() string {
			k := testkeys.Key(ks, rng.Int63n(ks.Count()))
			return fmt.Sprintf("seekge(%s)", k)
		}},
		{weight: 5, fn: func() string {
			k := testkeys.Key(ks, rng.Int63n(ks.Count()))
			return fmt.Sprintf("seeklt(%s)", k)
		}},
	}
	var totalWeight int
	for _, op := range ops {
		totalWeight += op.weight
	}
	var referenceHistory, fragmentedHistory bytes.Buffer
	for i := 0; i < numIterOps; i++ {
		p := rng.Intn(totalWeight)
		opIndex := 0
		if i == 0 {
			// First op is always a First().
		} else {
			for i, op := range ops {
				if p < op.weight {
					opIndex = i
					break
				}
				p -= op.weight
			}
		}
		op := ops[opIndex].fn()
		runIterOp(&referenceHistory, referenceIter, op)
		runIterOp(&fragmentedHistory, fragmentedIter, op)
		if !bytes.Equal(referenceHistory.Bytes(), fragmentedHistory.Bytes()) {
			t.Fatal(debugContext(cmp, formatKey, original, fragmented,
				referenceHistory.String(), fragmentedHistory.String()))
		}
	}
}

func fragment(cmp base.Compare, formatKey base.FormatKey, spans []keyspan.Span) []keyspan.Span {
	keyspan.Sort(cmp, spans)
	var fragments []keyspan.Span
	f := keyspan.Fragmenter{
		Cmp:    cmp,
		Format: formatKey,
		Emit: func(f keyspan.Span) {
			fragments = append(fragments, f)
		},
	}
	for _, s := range spans {
		f.Add(s)
	}
	f.Finish()
	return fragments
}

func debugContext(
	cmp base.Compare,
	formatKey base.FormatKey,
	original, fragmented []keyspan.Span,
	refHistory, fragHistory string,
) string {
	var buf bytes.Buffer
	fmt.Fprintln(&buf, "Reference:")
	for _, s := range original {
		fmt.Fprintln(&buf, s)
	}
	fmt.Fprintln(&buf)
	fmt.Fprintln(&buf, "Fragmented:")
	for _, s := range fragmented {
		fmt.Fprintln(&buf, s)
	}
	fmt.Fprintln(&buf)
	fmt.Fprintln(&buf, "\nOperations diff:")
	diff, err := difflib.GetUnifiedDiffString(difflib.UnifiedDiff{
		A:       difflib.SplitLines(refHistory),
		B:       difflib.SplitLines(fragHistory),
		Context: 5,
	})
	if err != nil {
		panic(err)
	}
	fmt.Fprintln(&buf, diff)
	return buf.String()
}

var iterDelim = map[rune]bool{',': true, ' ': true, '(': true, ')': true, '"': true}

func runIterOp(w io.Writer, it keyspan.FragmentIterator, op string) {
	fields := strings.FieldsFunc(op, func(r rune) bool { return iterDelim[r] })
	var s *keyspan.Span
	switch strings.ToLower(fields[0]) {
	case "first":
		s = it.First()
	case "last":
		s = it.Last()
	case "seekge":
		s = it.SeekGE([]byte(fields[1]))
	case "seeklt":
		s = it.SeekLT([]byte(fields[1]))
	case "next":
		s = it.Next()
	case "prev":
		s = it.Prev()
	default:
		panic(fmt.Sprintf("unrecognized iter op %q", fields[0]))
	}
	fmt.Fprintf(w, "%-10s", op)
	if s == nil {
		fmt.Fprintln(w, ".")
		return
	}
	fmt.Fprintln(w, s)
}

func BenchmarkTransform(b *testing.B) {
	var bufs Buffers
	var ui UserIteratorConfig
	reinit := func() {
		bufs.PrepareForReuse()
		_ = ui.Init(testkeys.Comparer, math.MaxUint64, nil, nil, new(bool), nil, true /* internalKeys */, &bufs)
	}

	for _, shadowing := range []bool{false, true} {
		b.Run(fmt.Sprintf("shadowing=%t", shadowing), func(b *testing.B) {
			for n := 1; n <= 128; n *= 2 {
				b.Run(fmt.Sprintf("keys=%d", n), func(b *testing.B) {
					rng := rand.New(rand.NewSource(233473048763))
					reinit()

					suffixes := make([][]byte, n)
					for s := range suffixes {
						if shadowing {
							suffixes[s] = testkeys.Suffix(int64(rng.Intn(n)))
						} else {
							suffixes[s] = testkeys.Suffix(int64(s))
						}
					}
					rng.Shuffle(len(suffixes), func(i, j int) {
						suffixes[i], suffixes[j] = suffixes[j], suffixes[i]
					})

					var keys []keyspan.Key
					for k := 0; k < n; k++ {
						keys = append(keys, keyspan.Key{
							Trailer: base.MakeTrailer(uint64(n-k), base.InternalKeyKindRangeKeySet),
							Suffix:  suffixes[k],
						})
					}
					dst := keyspan.Span{Keys: make([]keyspan.Key, 0, len(keys))}
					b.ResetTimer()

					for i := 0; i < b.N; i++ {
						err := ui.Transform(testkeys.Comparer.Compare, keyspan.Span{Keys: keys}, &dst)
						if err != nil {
							b.Fatal(err)
						}
						dst.Keys = dst.Keys[:0]
					}
				})
			}
		})
	}
}
