// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/testkeys"
)

func TestBoundedIter(t *testing.T) {
	getBounds := func(td *datadriven.TestData) (lower, upper []byte) {
		for _, cmdArg := range td.CmdArgs {
			switch cmdArg.Key {
			case "lower":
				if len(cmdArg.Vals[0]) > 0 {
					lower = []byte(cmdArg.Vals[0])
				}
			case "upper":
				if len(cmdArg.Vals[0]) > 0 {
					upper = []byte(cmdArg.Vals[0])
				}
			}
		}
		return lower, upper
	}

	cmp := testkeys.Comparer.Compare
	split := testkeys.Comparer.Split
	var buf bytes.Buffer
	var iter BoundedIter
	var hasPrefix bool
	var prefix []byte
	datadriven.RunTest(t, "testdata/bounded_iter", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "define":
			var spans []Span
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				spans = append(spans, ParseSpan(line))
			}
			inner := &invalidatingIter{iter: NewIter(cmp, spans)}
			lower, upper := getBounds(td)
			iter.Init(cmp, split, inner, lower, upper, &hasPrefix, &prefix)
			return ""
		case "set-prefix":
			hasPrefix = len(td.CmdArgs) > 0
			if hasPrefix {
				prefix = []byte(td.CmdArgs[0].String())
				return fmt.Sprintf("set prefix to %q\n", prefix)
			}
			return "cleared prefix"
		case "iter":
			buf.Reset()
			lower, upper := getBounds(td)
			iter.SetBounds(lower, upper)
			return runIterCmd(t, td, &iter)
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}
