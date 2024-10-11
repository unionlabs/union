// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"fmt"
	"io"
	"sort"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/record"
	"github.com/spf13/cobra"
)

// remoteCatalogT implements tools for the remote object catalog.
type remoteCatalogT struct {
	Root *cobra.Command
	Dump *cobra.Command

	verbose bool
	opts    *pebble.Options
}

func newRemoteCatalog(opts *pebble.Options) *remoteCatalogT {
	m := &remoteCatalogT{
		opts: opts,
	}

	m.Root = &cobra.Command{
		Use:   "remotecat",
		Short: "remote object catalog introspection tools",
	}

	// Add dump command
	m.Dump = &cobra.Command{
		Use:   "dump <remote-catalog-files>",
		Short: "print remote object catalog contents",
		Long: `
Print the contents of the REMOTE-OBJ-CATALOG files.
`,
		Args: cobra.MinimumNArgs(1),
		Run:  m.runDump,
	}
	m.Dump.Flags().BoolVarP(&m.verbose, "verbose", "v", false, "show each record in the catalog")
	m.Root.AddCommand(m.Dump)

	return m
}

func (m *remoteCatalogT) runDump(cmd *cobra.Command, args []string) {
	for _, arg := range args {
		err := m.runDumpOne(cmd.OutOrStdout(), arg)
		if err != nil {
			fmt.Fprintf(cmd.OutOrStderr(), "%s\n", err)
		}
	}
}

func (m *remoteCatalogT) runDumpOne(stdout io.Writer, filename string) error {
	f, err := m.opts.FS.Open(filename)
	if err != nil {
		return err
	}

	var creatorID objstorage.CreatorID
	objects := make(map[base.DiskFileNum]remoteobjcat.RemoteObjectMetadata)

	fmt.Fprintf(stdout, "%s\n", filename)
	var editIdx int
	rr := record.NewReader(f, 0 /* logNum */)
	for {
		offset := rr.Offset()
		r, err := rr.Next()
		if err == io.EOF {
			break
		} else if err != nil {
			return err
		}

		var ve remoteobjcat.VersionEdit
		err = ve.Decode(r)
		if err != nil {
			return err
		}

		if m.verbose {
			fmt.Fprintf(stdout, "%d/%d\n", offset, editIdx)
			if ve.CreatorID.IsSet() {
				fmt.Fprintf(stdout, "  CreatorID: %s\n", ve.CreatorID)
			}
			if len(ve.NewObjects) > 0 {
				fmt.Fprintf(stdout, "  NewObjects:\n")
				for _, m := range ve.NewObjects {
					fmt.Fprintf(
						stdout, "    %s  CreatorID: %s  CreatorFileNum: %s  Locator: %q CustomObjectName: %q\n",
						m.FileNum, m.CreatorID, m.CreatorFileNum, m.Locator, m.CustomObjectName,
					)
				}
			}
			if len(ve.DeletedObjects) > 0 {
				fmt.Fprintf(stdout, "  DeletedObjects:\n")
				for _, n := range ve.DeletedObjects {
					fmt.Fprintf(stdout, "    %s\n", n)
				}
			}
		}
		editIdx++
		if err := ve.Apply(&creatorID, objects); err != nil {
			return err
		}
	}
	fmt.Fprintf(stdout, "CreatorID: %v\n", creatorID)
	var filenums []base.DiskFileNum
	for n := range objects {
		filenums = append(filenums, n)
	}
	sort.Slice(filenums, func(i, j int) bool {
		return filenums[i].FileNum() < filenums[j].FileNum()
	})
	fmt.Fprintf(stdout, "Objects:\n")
	for _, n := range filenums {
		m := objects[n]
		fmt.Fprintf(
			stdout, "    %s  CreatorID: %s  CreatorFileNum: %s  Locator: %q CustomObjectName: %q\n",
			n, m.CreatorID, m.CreatorFileNum, m.Locator, m.CustomObjectName,
		)
	}
	return nil
}
