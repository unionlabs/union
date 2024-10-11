// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
)

func runFragmentIteratorCmd(iter FragmentIterator, input string, extraInfo func() string) string {
	var b bytes.Buffer
	for _, line := range strings.Split(input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		var span *Span
		switch parts[0] {
		case "seek-ge":
			if len(parts) != 2 {
				return "seek-ge <key>\n"
			}
			span = iter.SeekGE([]byte(strings.TrimSpace(parts[1])))
		case "seek-lt":
			if len(parts) != 2 {
				return "seek-lt <key>\n"
			}
			span = iter.SeekLT([]byte(strings.TrimSpace(parts[1])))
		case "first":
			span = iter.First()
		case "last":
			span = iter.Last()
		case "next":
			span = iter.Next()
		case "prev":
			span = iter.Prev()
		default:
			return fmt.Sprintf("unknown op: %s", parts[0])
		}
		if span != nil {
			fmt.Fprintf(&b, "%s", span)
			if extraInfo != nil {
				fmt.Fprintf(&b, " (%s)", extraInfo())
			}
			b.WriteByte('\n')
		} else if err := iter.Error(); err != nil {
			fmt.Fprintf(&b, "err=%v\n", err)
		} else {
			fmt.Fprintf(&b, ".\n")
		}
	}
	return b.String()
}

func TestIter(t *testing.T) {
	var spans []Span
	datadriven.RunTest(t, "testdata/iter", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "define":
			spans = nil
			for _, line := range strings.Split(d.Input, "\n") {
				spans = append(spans, ParseSpan(line))
			}
			return ""

		case "iter":
			iter := NewIter(base.DefaultComparer.Compare, spans)
			defer iter.Close()
			return runFragmentIteratorCmd(iter, d.Input, nil)
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}

// invalidatingIter wraps a FragmentIterator and implements FragmentIterator
// itself. Spans surfaced by the inner iterator are copied to buffers that are
// zeroed by sbubsequent iterator positioning calls. This is intended to help
// surface bugs in improper lifetime expectations of Spans.
type invalidatingIter struct {
	iter FragmentIterator
	bufs [][]byte
	keys []Key
	span Span
}

// invalidatingIter implements FragmentIterator.
var _ FragmentIterator = (*invalidatingIter)(nil)

func (i *invalidatingIter) invalidate(s *Span) *Span {
	// Zero the entirety of the byte bufs and the keys slice.
	for j := range i.bufs {
		for k := range i.bufs[j] {
			i.bufs[j][k] = 0x00
		}
		i.bufs[j] = nil
	}
	for j := range i.keys {
		i.keys[j] = Key{}
	}
	if s == nil {
		return nil
	}

	// Copy all of the span's slices into slices owned by the invalidating iter
	// that we can invalidate on a subsequent positioning method.
	i.bufs = i.bufs[:0]
	i.keys = i.keys[:0]
	i.span = Span{
		Start: i.saveBytes(s.Start),
		End:   i.saveBytes(s.End),
	}
	for j := range s.Keys {
		i.keys = append(i.keys, Key{
			Trailer: s.Keys[j].Trailer,
			Suffix:  i.saveBytes(s.Keys[j].Suffix),
			Value:   i.saveBytes(s.Keys[j].Value),
		})
	}
	i.span.Keys = i.keys
	return &i.span
}

func (i *invalidatingIter) saveBytes(b []byte) []byte {
	if b == nil {
		return nil
	}
	saved := append([]byte(nil), b...)
	i.bufs = append(i.bufs, saved)
	return saved
}

func (i *invalidatingIter) SeekGE(key []byte) *Span { return i.invalidate(i.iter.SeekGE(key)) }
func (i *invalidatingIter) SeekLT(key []byte) *Span { return i.invalidate(i.iter.SeekLT(key)) }
func (i *invalidatingIter) First() *Span            { return i.invalidate(i.iter.First()) }
func (i *invalidatingIter) Last() *Span             { return i.invalidate(i.iter.Last()) }
func (i *invalidatingIter) Next() *Span             { return i.invalidate(i.iter.Next()) }
func (i *invalidatingIter) Prev() *Span             { return i.invalidate(i.iter.Prev()) }
func (i *invalidatingIter) Close() error            { return i.iter.Close() }
func (i *invalidatingIter) Error() error            { return i.iter.Error() }
