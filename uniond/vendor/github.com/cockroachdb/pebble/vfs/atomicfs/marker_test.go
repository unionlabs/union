// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package atomicfs

import (
	"bytes"
	"fmt"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync/atomic"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/errorfs"
	"github.com/stretchr/testify/require"
)

func TestMarker_FilenameRoundtrip(t *testing.T) {
	filenames := []string{
		"marker.foo.000003.MANIFEST-000021",
		"marker.bar.000003.MANIFEST-000021",
		"marker.version.000003.1",
		"marker.version.000003.1.2.3.4",
		"marker.current.500000.MANIFEST-000001",
		"marker.current.18446744073709551615.MANIFEST-000001",
	}
	for _, testFilename := range filenames {
		t.Run(testFilename, func(t *testing.T) {
			name, iter, value, err := parseMarkerFilename(testFilename)
			require.NoError(t, err)

			filename := markerFilename(name, iter, value)
			require.Equal(t, testFilename, filename)
		})
	}
}

func TestMarker_Parsefilename(t *testing.T) {
	testCases := map[string]func(require.TestingT, error, ...interface{}){
		"marker.current.000003.MANIFEST-000021":  require.NoError,
		"marker.current.10.MANIFEST-000021":      require.NoError,
		"marker.v.10.1.2.3.4":                    require.NoError,
		"marker.name.18446744073709551615.value": require.NoError,
		"marke.current.000003.MANIFEST-000021":   require.Error,
		"marker.current.foo.MANIFEST-000021":     require.Error,
		"marker.current.ffffff.MANIFEST-000021":  require.Error,
	}
	for filename, assert := range testCases {
		t.Run(filename, func(t *testing.T) {
			_, _, _, err := parseMarkerFilename(filename)
			assert(t, err)
		})
	}
}

func TestMarker(t *testing.T) {
	markers := map[string]*Marker{}
	memFS := vfs.NewMem()

	var buf bytes.Buffer
	datadriven.RunTest(t, "testdata/marker", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "list":
			ls, err := memFS.List(td.CmdArgs[0].String())
			if err != nil {
				return err.Error()
			}
			sort.Strings(ls)
			buf.Reset()
			for _, filename := range ls {
				fmt.Fprintln(&buf, filename)
			}
			return buf.String()

		case "locate":
			var dir, marker string
			td.ScanArgs(t, "dir", &dir)
			td.ScanArgs(t, "marker", &marker)
			m, v, err := LocateMarker(memFS, dir, marker)
			if err != nil {
				return err.Error()
			}
			p := memFS.PathJoin(dir, marker)
			if oldMarker := markers[p]; oldMarker != nil {
				if err := oldMarker.Close(); err != nil {
					return err.Error()
				}
			}

			markers[p] = m
			return v

		case "mkdir-all":
			if len(td.CmdArgs) != 1 {
				return "usage: mkdir-all <dir>"
			}
			if err := memFS.MkdirAll(td.CmdArgs[0].String(), os.ModePerm); err != nil {
				return err.Error()
			}
			return ""

		case "move":
			var dir, marker string
			td.ScanArgs(t, "dir", &dir)
			td.ScanArgs(t, "marker", &marker)
			m := markers[memFS.PathJoin(dir, marker)]
			require.NotNil(t, m)
			err := m.Move(td.Input)
			if err != nil {
				return err.Error()
			}
			return ""

		case "next-iter":
			var dir, marker string
			td.ScanArgs(t, "dir", &dir)
			td.ScanArgs(t, "marker", &marker)
			m := markers[memFS.PathJoin(dir, marker)]
			require.NotNil(t, m)
			return fmt.Sprintf("%d", m.NextIter())

		case "read":
			var dir, marker string
			td.ScanArgs(t, "dir", &dir)
			td.ScanArgs(t, "marker", &marker)
			v, err := ReadMarker(memFS, dir, marker)
			if err != nil {
				return err.Error()
			}
			return v

		case "remove-obsolete":
			var dir, marker string
			td.ScanArgs(t, "dir", &dir)
			td.ScanArgs(t, "marker", &marker)
			m := markers[memFS.PathJoin(dir, marker)]
			require.NotNil(t, m)
			obsoleteCount := len(m.obsoleteFiles)
			require.NoError(t, m.RemoveObsolete())
			removedCount := obsoleteCount - len(m.obsoleteFiles)
			return fmt.Sprintf("Removed %d files.", removedCount)

		case "touch":
			for _, filename := range strings.Split(td.Input, "\n") {
				f, err := memFS.Create(filename)
				if err != nil {
					return err.Error()
				}
				if err := f.Close(); err != nil {
					return err.Error()
				}
			}
			return ""

		default:
			panic(fmt.Sprintf("unknown command %q", td.Cmd))
		}
	})
}

func TestMarker_StrictSync(t *testing.T) {
	// Use an in-memory FS that strictly enforces syncs.
	mem := vfs.NewStrictMem()
	syncDir := func(dir string) {
		fdir, err := mem.OpenDir(dir)
		require.NoError(t, err)
		require.NoError(t, fdir.Sync())
		require.NoError(t, fdir.Close())
	}

	require.NoError(t, mem.MkdirAll("foo", os.ModePerm))
	syncDir("")
	m, v, err := LocateMarker(mem, "foo", "bar")
	require.NoError(t, err)
	require.Equal(t, "", v)
	require.NoError(t, m.Move("hello"))
	require.NoError(t, m.Close())

	// Discard any unsynced writes to make sure we set up the test
	// preconditions correctly.
	mem.ResetToSyncedState()
	m, v, err = LocateMarker(mem, "foo", "bar")
	require.NoError(t, err)
	require.Equal(t, "hello", v)
	require.NoError(t, m.Move("hello-world"))
	require.NoError(t, m.Close())

	// Discard any unsynced writes.
	mem.ResetToSyncedState()
	m, v, err = LocateMarker(mem, "foo", "bar")
	require.NoError(t, err)
	require.Equal(t, "hello-world", v)
	require.NoError(t, m.Close())
}

// TestMarker_FaultTolerance attempts a series of operations on atomic
// markers, injecting errors at successively higher indexed operations.
// It completes when an error is never injected, because the index is
// higher than the number of filesystem operations performed by the
// test.
func TestMarker_FaultTolerance(t *testing.T) {
	done := false
	for i := 1; !done && i < 1000; i++ {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			var count atomic.Int32
			count.Store(int32(i))
			inj := errorfs.InjectorFunc(func(op errorfs.Op, path string) error {
				// Don't inject on Sync errors. They're fatal.
				if op == errorfs.OpFileSync {
					return nil
				}
				if v := count.Add(-1); v == 0 {
					return errorfs.ErrInjected
				}
				return nil
			})

			mem := vfs.NewMem()
			fs := errorfs.Wrap(mem, inj)
			markers := map[string]*Marker{}
			ops := []struct {
				op    string
				name  string
				value string
			}{
				{op: "locate", name: "foo", value: ""},
				{op: "locate", name: "foo", value: ""},
				{op: "locate", name: "bar", value: ""},
				{op: "rm-obsolete", name: "foo"},
				{op: "move", name: "bar", value: "california"},
				{op: "rm-obsolete", name: "bar"},
				{op: "move", name: "bar", value: "california"},
				{op: "move", name: "bar", value: "new-york"},
				{op: "locate", name: "bar", value: "new-york"},
				{op: "move", name: "bar", value: "california"},
				{op: "rm-obsolete", name: "bar"},
				{op: "locate", name: "bar", value: "california"},
				{op: "move", name: "foo", value: "connecticut"},
				{op: "locate", name: "foo", value: "connecticut"},
			}

			for _, op := range ops {
				runOp := func() error {
					switch op.op {
					case "locate":
						m, v, err := LocateMarker(fs, "", op.name)
						if err != nil {
							return err
						}
						require.NotNil(t, m)
						require.Equal(t, op.value, v)
						if existingMarker := markers[op.name]; existingMarker != nil {
							require.NoError(t, existingMarker.Close())
						}
						markers[op.name] = m
						return nil
					case "move":
						m := markers[op.name]
						require.NotNil(t, m)
						return m.Move(op.value)
					case "rm-obsolete":
						m := markers[op.name]
						require.NotNil(t, m)
						return m.RemoveObsolete()
					default:
						panic("unreachable")
					}
				}

				// Run the operation, if it fails with the injected
				// error, retry it exactly once. The retry should always
				// succeed.
				err := runOp()
				if errors.Is(err, errorfs.ErrInjected) {
					err = runOp()
				}
				require.NoError(t, err)
			}

			for _, m := range markers {
				require.NoError(t, m.Close())
			}

			// Stop if the number of operations in the test case is
			// fewer than `i`.
			done = count.Load() > 0
		})
	}
}
