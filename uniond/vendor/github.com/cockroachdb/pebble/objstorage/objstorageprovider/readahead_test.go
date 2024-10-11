// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"fmt"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/stretchr/testify/require"
)

func TestMaybeReadahead(t *testing.T) {
	rs := makeReadaheadState(256 * 1024)
	datadriven.RunTest(t, "testdata/readahead", func(t *testing.T, d *datadriven.TestData) string {
		cacheHit := false
		switch d.Cmd {
		case "reset":
			rs.size = initialReadaheadSize
			rs.limit = 0
			rs.numReads = 0
			return ""

		case "cache-read":
			cacheHit = true
			fallthrough
		case "read":
			args := strings.Split(d.Input, ",")
			if len(args) != 2 {
				return "expected 2 args: offset, size"
			}

			offset, err := strconv.ParseInt(strings.TrimSpace(args[0]), 10, 64)
			require.NoError(t, err)
			size, err := strconv.ParseInt(strings.TrimSpace(args[1]), 10, 64)
			require.NoError(t, err)
			var raSize int64
			if cacheHit {
				rs.recordCacheHit(offset, size)
			} else {
				raSize = rs.maybeReadahead(offset, size)
			}

			var buf strings.Builder
			fmt.Fprintf(&buf, "readahead:  %d\n", raSize)
			fmt.Fprintf(&buf, "numReads:   %d\n", rs.numReads)
			fmt.Fprintf(&buf, "size:       %d\n", rs.size)
			fmt.Fprintf(&buf, "prevSize:   %d\n", rs.prevSize)
			fmt.Fprintf(&buf, "limit:      %d", rs.limit)
			return buf.String()
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}
