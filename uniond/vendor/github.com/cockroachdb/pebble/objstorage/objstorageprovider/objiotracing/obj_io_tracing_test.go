// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objiotracing_test

import (
	"io"
	"strings"
	"testing"
	"unsafe"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/objiotracing"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

type Event = objiotracing.Event

const eventSize = int(unsafe.Sizeof(Event{}))

func TestTracing(t *testing.T) {
	if !objiotracing.Enabled {
		t.Skipf("test can only be run under pebble_obj_io_tracing build tag")
	}
	fs := vfs.NewMem()
	d, err := pebble.Open("", &pebble.Options{FS: fs})
	require.NoError(t, err)

	require.NoError(t, d.Set([]byte("a"), []byte("aaa"), nil))
	require.NoError(t, d.Set([]byte("b"), []byte("bbb"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Set([]byte("c"), []byte("ccc"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Compact([]byte("a"), []byte("z"), false /* parallelize */))
	require.NoError(t, d.Set([]byte("b"), []byte("bbb2"), nil))
	require.NoError(t, d.Set([]byte("c"), []byte("ccc2"), nil))
	require.NoError(t, d.Set([]byte("d"), []byte("ddd"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Compact([]byte("a"), []byte("z"), false /* parallelize */))
	require.NoError(t, d.Close())

	collectEvents := func() []Event {
		t.Helper()

		list, err := fs.List("")
		require.NoError(t, err)

		var events []Event
		for _, f := range list {
			if strings.HasPrefix(f, "IOTRACES-") {
				file, err := fs.Open(f)
				require.NoError(t, err)
				data, err := io.ReadAll(file)
				file.Close()
				require.NoError(t, err)
				// Remove the file so we don't read these events again later.
				fs.Remove(f)
				if len(data) == 0 {
					continue
				}
				require.Equal(t, len(data)%eventSize, 0)
				p := unsafe.Pointer(&data[0])
				asEvents := unsafe.Slice((*Event)(p), len(data)/eventSize)
				events = append(events, asEvents...)
			}
		}
		if testing.Verbose() {
			t.Logf("collected events:")
			for _, e := range events {
				t.Logf("  %#v", e)
			}
		}
		return events
	}
	events := collectEvents()
	num := func(check func(e Event) bool) int {
		res := 0
		for _, e := range events {
			if check(e) {
				res += 1
			}
		}
		return res
	}
	// Check that we saw at least a few reads and writes.
	// TODO(radu): check more fields when they are populated.
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.ReadOp }), 5)
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.WriteOp }), 5)

	// We should see writes at L0 and L7.
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.WriteOp && e.LevelPlusOne == 1 }), 0)
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.WriteOp && e.LevelPlusOne == 7 }), 0)

	// Check that we saw writes for flushing and for compaction.
	require.Greater(t, num(func(e Event) bool { return e.Reason == objiotracing.ForFlush }), 0)
	require.Greater(t, num(func(e Event) bool { return e.Reason == objiotracing.ForCompaction }), 0)

	// Check that offset is set on reads & writes as expected.
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.ReadOp && e.Offset > 0 }), 0)
	require.Greater(t, num(func(e Event) bool { return e.Op == objiotracing.WriteOp && e.Offset > 0 }), 0)

	// Check that the FileNums are set and that we see at least two different files.
	fileNums := make(map[base.FileNum]int)
	for _, e := range events {
		require.NotZero(t, e.FileNum)
		fileNums[e.FileNum] += 1
	}
	require.GreaterOrEqual(t, len(fileNums), 2)

	// Open again and do some reads.
	d, err = pebble.Open("", &pebble.Options{FS: fs})
	require.NoError(t, err)
	for _, k := range []string{"0", "a", "d", "ccc", "b"} {
		_, closer, err := d.Get([]byte(k))
		if err == pebble.ErrNotFound {
			continue
		}
		require.NoError(t, err)
		closer.Close()
	}
	require.NoError(t, d.Close())
	events = collectEvents()
	// Expect L6 data block reads.
	require.Greater(t, num(func(e Event) bool {
		return e.Op == objiotracing.ReadOp && e.BlockType == objiotracing.DataBlock && e.LevelPlusOne == 7
	}), 0)
}
