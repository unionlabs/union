// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
)

func TestSeek(t *testing.T) {
	cmp := base.DefaultComparer.Compare
	fmtKey := base.DefaultComparer.FormatKey
	var iter FragmentIterator
	var buf bytes.Buffer

	datadriven.RunTest(t, "testdata/seek", func(t *testing.T, d *datadriven.TestData) string {
		buf.Reset()
		switch d.Cmd {
		case "build":
			spans := buildSpans(t, cmp, fmtKey, d.Input, base.InternalKeyKindRangeDelete)
			for _, s := range spans {
				fmt.Fprintln(&buf, s)
			}
			iter = NewIter(cmp, spans)
			return buf.String()
		case "seek-ge", "seek-le":
			seek := SeekLE
			if d.Cmd == "seek-ge" {
				seek = func(_ base.Compare, iter FragmentIterator, key []byte) *Span {
					return iter.SeekGE(key)
				}
			}

			for _, line := range strings.Split(d.Input, "\n") {
				parts := strings.Fields(line)
				if len(parts) != 2 {
					return fmt.Sprintf("malformed input: %s", line)
				}
				seq, err := strconv.ParseUint(parts[1], 10, 64)
				if err != nil {
					return err.Error()
				}
				span := seek(cmp, iter, []byte(parts[0]))
				if span != nil {
					visible := span.Visible(seq)
					span = &visible
				}
				fmt.Fprintln(&buf, span)
			}
			return buf.String()
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}
