// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"fmt"
	"io"
	"testing"

	"github.com/cockroachdb/datadriven"
)

func writeBufferPool(w io.Writer, bp *BufferPool) {
	for i := 0; i < cap(bp.pool); i++ {
		if i > 0 {
			fmt.Fprint(w, " ")
		}
		if i >= len(bp.pool) {
			fmt.Fprint(w, "[    ]")
			continue
		}
		sz := len(bp.pool[i].v.Buf())
		if bp.pool[i].b == nil {
			fmt.Fprintf(w, "[%4d]", sz)
		} else {
			fmt.Fprintf(w, "<%4d>", sz)
		}
	}
}

func TestBufferPool(t *testing.T) {
	var bp BufferPool
	var buf bytes.Buffer
	handles := map[string]Buf{}
	drainPool := func() {
		for h, b := range handles {
			b.Release()
			delete(handles, h)
		}
		bp.Release()
	}
	defer drainPool()
	datadriven.RunTest(t, "testdata/buffer_pool", func(t *testing.T, td *datadriven.TestData) string {
		buf.Reset()
		switch td.Cmd {
		case "init":
			if cap(bp.pool) > 0 {
				drainPool()
			}
			var initialSize int
			td.ScanArgs(t, "size", &initialSize)
			bp.Init(initialSize)
			writeBufferPool(&buf, &bp)
			return buf.String()
		case "alloc":
			var n int
			var handle string
			td.ScanArgs(t, "n", &n)
			td.ScanArgs(t, "handle", &handle)
			handles[handle] = bp.Alloc(n)
			writeBufferPool(&buf, &bp)
			return buf.String()
		case "release":
			var handle string
			td.ScanArgs(t, "handle", &handle)
			b := handles[handle]
			b.Release()
			delete(handles, handle)
			writeBufferPool(&buf, &bp)
			return buf.String()
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}
