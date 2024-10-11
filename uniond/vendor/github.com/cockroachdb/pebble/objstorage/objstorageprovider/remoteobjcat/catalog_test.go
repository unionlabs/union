// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remoteobjcat_test

import (
	"fmt"
	"math/rand"
	"sort"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/vfs"
)

func TestCatalog(t *testing.T) {
	mem := vfs.NewMem()
	var memLog base.InMemLogger

	var cat *remoteobjcat.Catalog
	datadriven.RunTest(t, "testdata/catalog", func(t *testing.T, td *datadriven.TestData) string {
		toUInt64 := func(args ...string) []uint64 {
			t.Helper()
			var res []uint64
			for _, arg := range args {
				n, err := strconv.Atoi(arg)
				if err != nil {
					td.Fatalf(t, "error parsing arg %s as integer: %v", arg, err)
				}
				res = append(res, uint64(n))
			}
			return res
		}

		parseAdd := func(args []string) remoteobjcat.RemoteObjectMetadata {
			t.Helper()
			if len(args) != 3 {
				td.Fatalf(t, "add <file-num> <creator-id> <creator-file-num>")
			}
			vals := toUInt64(args...)
			return remoteobjcat.RemoteObjectMetadata{
				FileNum: base.FileNum(vals[0]).DiskFileNum(),
				// When we support other file types, we should let the test determine this.
				FileType:       base.FileTypeTable,
				CreatorID:      objstorage.CreatorID(vals[1]),
				CreatorFileNum: base.FileNum(vals[2]).DiskFileNum(),
			}
		}

		parseDel := func(args []string) base.DiskFileNum {
			t.Helper()
			if len(args) != 1 {
				td.Fatalf(t, "delete <file-num>")
			}
			return base.FileNum(toUInt64(args[0])[0]).DiskFileNum()
		}

		memLog.Reset()
		switch td.Cmd {
		case "open":
			if len(td.CmdArgs) != 1 {
				td.Fatalf(t, "open <dir>")
			}
			dirname := td.CmdArgs[0].String()
			err := mem.MkdirAll(dirname, 0755)
			if err != nil {
				td.Fatalf(t, "%v", err)
			}
			var contents remoteobjcat.CatalogContents
			cat, contents, err = remoteobjcat.Open(vfs.WithLogging(mem, memLog.Infof), dirname)
			if err != nil {
				return err.Error()
			}
			var buf strings.Builder
			if contents.CreatorID.IsSet() {
				fmt.Fprintf(&buf, "creator-id: %s\n", contents.CreatorID)
			}
			for _, meta := range contents.Objects {
				fmt.Fprintf(&buf, "%s: %d/%s\n", meta.FileNum, meta.CreatorID, meta.CreatorFileNum)
			}

			return buf.String()

		case "set-creator-id":
			if len(td.CmdArgs) != 1 {
				td.Fatalf(t, "set-creator-id <id>")
			}
			id := objstorage.CreatorID(toUInt64(td.CmdArgs[0].String())[0])
			if err := cat.SetCreatorID(id); err != nil {
				return fmt.Sprintf("error setting creator ID: %v", err)
			}
			return memLog.String()

		case "batch":
			var b remoteobjcat.Batch
			for _, cmd := range strings.Split(td.Input, "\n") {
				tokens := strings.Split(cmd, " ")
				if len(tokens) == 0 {
					td.Fatalf(t, "empty batch line")
				}
				switch tokens[0] {
				case "add":
					b.AddObject(parseAdd(tokens[1:]))
				case "delete":
					b.DeleteObject(parseDel(tokens[1:]))
				default:
					td.Fatalf(t, "unknown batch command: %s", tokens[0])
				}
			}
			if err := cat.ApplyBatch(b); err != nil {
				return fmt.Sprintf("error applying batch: %v", err)
			}
			b.Reset()
			return memLog.String()

		case "random-batches":
			n := 1
			size := 1000
			for _, arg := range td.CmdArgs {
				if len(arg.Vals) != 1 {
					td.Fatalf(t, "random-batches n=<val> size=<val>")
				}
				val := toUInt64(arg.Vals[0])[0]
				switch arg.Key {
				case "n":
					n = int(val)
				case "size":
					size = int(val)
				default:
					td.Fatalf(t, "random-batches n=<val> size=<val>")
				}
			}
			var b remoteobjcat.Batch
			for batchIdx := 0; batchIdx < n; batchIdx++ {
				for i := 0; i < size; i++ {
					b.AddObject(remoteobjcat.RemoteObjectMetadata{
						FileNum: base.FileNum(rand.Uint64()).DiskFileNum(),
						// When we support other file types, we should let the test determine this.
						FileType:       base.FileTypeTable,
						CreatorID:      objstorage.CreatorID(rand.Uint64()),
						CreatorFileNum: base.FileNum(rand.Uint64()).DiskFileNum(),
					})
				}
				if err := cat.ApplyBatch(b); err != nil {
					td.Fatalf(t, "error applying batch: %v", err)
				}
				b.Reset()
			}
			return memLog.String()

		case "close":
			if cat == nil {
				return "nil catalog"
			}
			err := cat.Close()
			cat = nil
			if err != nil {
				return fmt.Sprintf("%v", err)
			}
			return memLog.String()

		case "list":
			if len(td.CmdArgs) != 1 {
				td.Fatalf(t, "open <dir>")
			}
			paths, err := mem.List(td.CmdArgs[0].String())
			if err != nil {
				return err.Error()
			}
			sort.Strings(paths)
			return strings.Join(paths, "\n")

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}
