// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
)

func TestFilteringIter(t *testing.T) {
	// makeFilter returns a FilterFunc that will filter out all keys in a Span
	// that are not of the given kind. Empty spans are skipped.
	makeFilter := func(kind base.InternalKeyKind) FilterFunc {
		return func(in *Span, out *Span) (keep bool) {
			out.Start, out.End = in.Start, in.End
			out.Keys = out.Keys[:0]
			for _, k := range in.Keys {
				if k.Kind() != kind {
					continue
				}
				out.Keys = append(out.Keys, k)
			}
			return len(out.Keys) > 0
		}
	}

	cmp := testkeys.Comparer.Compare
	var spans []Span
	datadriven.RunTest(t, "testdata/filtering_iter", func(t *testing.T, td *datadriven.TestData) string {
		switch cmd := td.Cmd; cmd {
		case "define":
			spans = spans[:0]
			lines := strings.Split(strings.TrimSpace(td.Input), "\n")
			for _, line := range lines {
				spans = append(spans, ParseSpan(line))
			}
			return ""

		case "iter":
			var filter FilterFunc
			for _, cmdArg := range td.CmdArgs {
				switch cmdArg.Key {
				case "filter":
					for _, s := range cmdArg.Vals {
						switch s {
						case "no-op":
							filter = nil
						case "key-kind-set":
							filter = makeFilter(base.InternalKeyKindRangeKeySet)
						case "key-kind-unset":
							filter = makeFilter(base.InternalKeyKindRangeKeyUnset)
						case "key-kind-del":
							filter = makeFilter(base.InternalKeyKindRangeKeyDelete)
						default:
							return fmt.Sprintf("unknown filter: %s", s)
						}
					}
				default:
					return fmt.Sprintf("unknown command: %s", cmdArg.Key)
				}
			}
			innerIter := NewIter(cmp, spans)
			iter := Filter(innerIter, filter, cmp)
			defer iter.Close()
			s := runFragmentIteratorCmd(iter, td.Input, nil)
			return s

		default:
			return fmt.Sprintf("unknown command: %s", cmd)
		}
	})
}
