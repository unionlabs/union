// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"sort"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func TestCleaner(t *testing.T) {
	dbs := make(map[string]*DB)
	defer func() {
		for _, db := range dbs {
			require.NoError(t, db.Close())
		}
	}()

	mem := vfs.NewMem()
	var memLog base.InMemLogger
	fs := vfs.WithLogging(mem, memLog.Infof)
	datadriven.RunTest(t, "testdata/cleaner", func(t *testing.T, td *datadriven.TestData) string {
		memLog.Reset()
		switch td.Cmd {
		case "batch":
			if len(td.CmdArgs) != 1 {
				return "batch <db>"
			}
			d := dbs[td.CmdArgs[0].String()]
			b := d.NewBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(Sync); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "compact":
			if len(td.CmdArgs) != 1 {
				return "compact <db>"
			}
			d := dbs[td.CmdArgs[0].String()]
			if err := d.Compact(nil, []byte("\xff"), false); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "flush":
			if len(td.CmdArgs) != 1 {
				return "flush <db>"
			}
			d := dbs[td.CmdArgs[0].String()]
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "close":
			if len(td.CmdArgs) != 1 {
				return "close <db>"
			}
			dbDir := td.CmdArgs[0].String()
			d := dbs[dbDir]
			if err := d.Close(); err != nil {
				return err.Error()
			}
			delete(dbs, dbDir)
			return memLog.String()

		case "list":
			if len(td.CmdArgs) != 1 {
				return "list <dir>"
			}
			paths, err := mem.List(td.CmdArgs[0].String())
			if err != nil {
				return err.Error()
			}
			sort.Strings(paths)
			return fmt.Sprintf("%s\n", strings.Join(paths, "\n"))

		case "open":
			if len(td.CmdArgs) < 1 || len(td.CmdArgs) > 3 {
				return "open <dir> [archive] [readonly]"
			}
			dir := td.CmdArgs[0].String()
			opts := (&Options{
				FS:     fs,
				WALDir: dir + "_wal",
			}).WithFSDefaults()

			for i := 1; i < len(td.CmdArgs); i++ {
				switch td.CmdArgs[i].String() {
				case "readonly":
					opts.ReadOnly = true
				case "archive":
					opts.Cleaner = ArchiveCleaner{}
				default:
					return "open <dir> [archive] [readonly]"
				}
			}
			// Asynchronous table stats retrieval makes the output flaky.
			opts.private.disableTableStats = true
			opts.private.testingAlwaysWaitForCleanup = true
			d, err := Open(dir, opts)
			if err != nil {
				return err.Error()
			}
			d.TestOnlyWaitForCleaning()
			dbs[dir] = d
			return memLog.String()

		case "create-bogus-file":
			if len(td.CmdArgs) != 1 {
				return "create-bogus-file <db/file>"
			}
			dst, err := fs.Create(td.CmdArgs[0].String())
			require.NoError(t, err)
			_, err = dst.Write([]byte("bogus data"))
			require.NoError(t, err)
			require.NoError(t, dst.Sync())
			require.NoError(t, dst.Close())
			return memLog.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}
