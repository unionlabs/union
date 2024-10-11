// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"regexp"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
)

var spanRe = regexp.MustCompile(`(\d+):\s*(\w+)-*(\w+)\w*([^\n]*)`)

func parseSpanSingleKey(t *testing.T, s string, kind base.InternalKeyKind) Span {
	m := spanRe.FindStringSubmatch(s)
	if len(m) != 5 {
		t.Fatalf("expected 5 components, but found %d: %s", len(m), s)
	}
	seqNum, err := strconv.Atoi(m[1])
	require.NoError(t, err)
	return Span{
		Start: []byte(m[2]),
		End:   []byte(m[3]),
		Keys: []Key{
			{
				Trailer: base.MakeTrailer(uint64(seqNum), kind),
				Value:   []byte(strings.TrimSpace(m[4])),
			},
		},
	}
}

func buildSpans(
	t *testing.T, cmp base.Compare, formatKey base.FormatKey, s string, kind base.InternalKeyKind,
) []Span {
	var spans []Span
	f := &Fragmenter{
		Cmp:    cmp,
		Format: formatKey,
		Emit: func(fragmented Span) {
			spans = append(spans, fragmented)
		},
	}
	for _, line := range strings.Split(s, "\n") {
		if strings.HasPrefix(line, "truncate-and-flush-to ") {
			parts := strings.Split(line, " ")
			if len(parts) != 2 {
				t.Fatalf("expected 2 components, but found %d: %s", len(parts), line)
			}
			f.TruncateAndFlushTo([]byte(parts[1]))
			continue
		}

		f.Add(parseSpanSingleKey(t, line, kind))
	}
	f.Finish()
	return spans
}

func formatAlphabeticSpans(spans []Span) string {
	isLetter := func(b []byte) bool {
		if len(b) != 1 {
			return false
		}
		return b[0] >= 'a' && b[0] <= 'z'
	}

	var buf bytes.Buffer
	for _, v := range spans {
		switch {
		case !v.Valid():
			fmt.Fprintf(&buf, "<invalid>\n")
		case v.Empty():
			fmt.Fprintf(&buf, "<empty>\n")
		case !isLetter(v.Start) || !isLetter(v.End) || v.Start[0] == v.End[0]:
			for _, k := range v.Keys {
				fmt.Fprintf(&buf, "%d: %s-%s", k.SeqNum(), v.Start, v.End)
				if len(k.Value) > 0 {
					buf.WriteString(strings.Repeat(" ", int('z'-v.End[0]+1)))
					buf.WriteString(string(k.Value))
				}
				fmt.Fprintln(&buf)
			}
		default:
			for _, k := range v.Keys {
				fmt.Fprintf(&buf, "%d: %s%s%s%s",
					k.SeqNum(),
					strings.Repeat(" ", int(v.Start[0]-'a')),
					v.Start,
					strings.Repeat("-", int(v.End[0]-v.Start[0]-1)),
					v.End)
				if len(k.Value) > 0 {
					buf.WriteString(strings.Repeat(" ", int('z'-v.End[0]+1)))
					buf.WriteString(string(k.Value))
				}
				fmt.Fprintln(&buf)
			}
		}
	}
	return buf.String()
}

func TestFragmenter(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey

	var getRe = regexp.MustCompile(`(\w+)#(\d+)`)

	parseGet := func(t *testing.T, s string) (string, int) {
		m := getRe.FindStringSubmatch(s)
		if len(m) != 3 {
			t.Fatalf("expected 3 components, but found %d", len(m))
		}
		seq, err := strconv.Atoi(m[2])
		require.NoError(t, err)
		return m[1], seq
	}

	var iter FragmentIterator

	// Returns true if the specified <key,seq> pair is deleted at the specified
	// read sequence number. Get ignores spans newer than the read sequence
	// number. This is a simple version of what full processing of range
	// tombstones looks like.
	deleted := func(key []byte, seq, readSeq uint64) bool {
		s := Get(cmp, iter, key)
		return s != nil && s.CoversAt(readSeq, seq)
	}

	datadriven.RunTest(t, "testdata/fragmenter", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "build":
			return func() (result string) {
				defer func() {
					if r := recover(); r != nil {
						result = fmt.Sprint(r)
					}
				}()

				spans := buildSpans(t, cmp, fmtKey, d.Input, base.InternalKeyKindRangeDelete)
				iter = NewIter(cmp, spans)
				return formatAlphabeticSpans(spans)
			}()

		case "get":
			if len(d.CmdArgs) != 1 {
				return fmt.Sprintf("expected 1 argument, but found %s", d.CmdArgs)
			}
			if d.CmdArgs[0].Key != "t" {
				return fmt.Sprintf("expected timestamp argument, but found %s", d.CmdArgs[0])
			}
			readSeq, err := strconv.Atoi(d.CmdArgs[0].Vals[0])
			require.NoError(t, err)

			var results []string
			for _, p := range strings.Split(d.Input, " ") {
				key, seq := parseGet(t, p)
				if deleted([]byte(key), uint64(seq), uint64(readSeq)) {
					results = append(results, "deleted")
				} else {
					results = append(results, "alive")
				}
			}
			return strings.Join(results, " ")

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFragmenterCovers(t *testing.T) {
	datadriven.RunTest(t, "testdata/fragmenter_covers", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "build":
			f := &Fragmenter{
				Cmp:    base.DefaultComparer.Compare,
				Format: base.DefaultComparer.FormatKey,
				Emit: func(fragmented Span) {
				},
			}
			var buf bytes.Buffer
			for _, line := range strings.Split(d.Input, "\n") {
				switch {
				case strings.HasPrefix(line, "add "):
					t := parseSpanSingleKey(t, strings.TrimPrefix(line, "add "), base.InternalKeyKindRangeDelete)
					f.Add(t)
				case strings.HasPrefix(line, "deleted "):
					fields := strings.Fields(strings.TrimPrefix(line, "deleted "))
					key := base.ParseInternalKey(fields[0])
					snapshot, err := strconv.ParseUint(fields[1], 10, 64)
					if err != nil {
						return err.Error()
					}
					func() {
						defer func() {
							if r := recover(); r != nil {
								fmt.Fprintf(&buf, "%s: %s\n", key, r)
							}
						}()
						switch f.Covers(key, snapshot) {
						case NoCover:
							fmt.Fprintf(&buf, "%s: none\n", key)
						case CoversInvisibly:
							fmt.Fprintf(&buf, "%s: invisibly\n", key)
						case CoversVisibly:
							fmt.Fprintf(&buf, "%s: visibly\n", key)
						}
					}()
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFragmenterTruncateAndFlushTo(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey

	datadriven.RunTest(t, "testdata/fragmenter_truncate_and_flush_to", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "build":
			return func() (result string) {
				defer func() {
					if r := recover(); r != nil {
						result = fmt.Sprint(r)
					}
				}()

				spans := buildSpans(t, cmp, fmtKey, d.Input, base.InternalKeyKindRangeDelete)
				return formatAlphabeticSpans(spans)
			}()

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFragmenter_Values(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey

	datadriven.RunTest(t, "testdata/fragmenter_values", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "build":
			return func() (result string) {
				defer func() {
					if r := recover(); r != nil {
						result = fmt.Sprint(r)
					}
				}()

				spans := buildSpans(t, cmp, fmtKey, d.Input, base.InternalKeyKindRangeKeySet)
				return formatAlphabeticSpans(spans)
			}()

		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

func TestFragmenter_EmitOrder(t *testing.T) {
	var buf bytes.Buffer

	datadriven.RunTest(t, "testdata/fragmenter_emit_order", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "build":
			buf.Reset()
			f := Fragmenter{
				Cmp:    base.DefaultComparer.Compare,
				Format: base.DefaultComparer.FormatKey,
				Emit: func(span Span) {
					fmt.Fprintf(&buf, "%s %s:",
						base.DefaultComparer.FormatKey(span.Start),
						base.DefaultComparer.FormatKey(span.End))
					for i, k := range span.Keys {
						if i == 0 {
							fmt.Fprint(&buf, " ")
						} else {
							fmt.Fprint(&buf, ", ")
						}
						fmt.Fprintf(&buf, "#%d,%s", k.SeqNum(), k.Kind())
					}
					fmt.Fprintln(&buf, "\n-")
				},
			}
			for _, line := range strings.Split(d.Input, "\n") {
				fields := strings.Fields(line)
				if len(fields) != 2 {
					panic(fmt.Sprintf("datadriven test: expect 2 fields, found %d", len(fields)))
				}
				k := base.ParseInternalKey(fields[0])
				f.Add(Span{
					Start: k.UserKey,
					End:   []byte(fields[1]),
					Keys:  []Key{{Trailer: k.Trailer}},
				})
			}

			f.Finish()
			return buf.String()
		default:
			panic(fmt.Sprintf("unrecognized command %q", d.Cmd))
		}
	})
}
