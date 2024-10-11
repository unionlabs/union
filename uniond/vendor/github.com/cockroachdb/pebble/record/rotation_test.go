// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package record

import (
	"fmt"
	"strconv"
	"testing"

	"github.com/cockroachdb/datadriven"
)

func TestRotation(t *testing.T) {
	var rh RotationHelper
	datadriven.RunTest(t, "testdata/rotation", func(t *testing.T, td *datadriven.TestData) string {
		oneIntArg := func() int64 {
			if len(td.CmdArgs) != 1 {
				td.Fatalf(t, "expected one integer argument")
			}
			n, err := strconv.Atoi(td.CmdArgs[0].String())
			if err != nil {
				td.Fatalf(t, "expected one integer argument")
			}
			return int64(n)
		}
		switch td.Cmd {
		case "add":
			size := oneIntArg()
			rh.AddRecord(size)

		case "should-rotate":
			nextSnapshotSize := oneIntArg()
			return fmt.Sprint(rh.ShouldRotate(nextSnapshotSize))

		case "rotate":
			snapshotSize := oneIntArg()
			rh.Rotate(snapshotSize)

		default:
			td.Fatalf(t, "unknown command %s", td.Cmd)
		}

		// For commands with no output, show the debug info.
		a, b := rh.DebugInfo()
		return fmt.Sprintf("last-snapshot-size: %d\nsize-since-last-snapshot: %d", a, b)
	})
}
