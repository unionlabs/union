// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/rangekey"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/spf13/cobra"
)

// walT implements WAL-level tools, including both configuration state and the
// commands themselves.
type walT struct {
	Root *cobra.Command
	Dump *cobra.Command

	opts     *pebble.Options
	fmtKey   keyFormatter
	fmtValue valueFormatter

	defaultComparer string
	comparers       sstable.Comparers
	verbose         bool
}

func newWAL(opts *pebble.Options, comparers sstable.Comparers, defaultComparer string) *walT {
	w := &walT{
		opts: opts,
	}
	w.fmtKey.mustSet("quoted")
	w.fmtValue.mustSet("size")
	w.comparers = comparers
	w.defaultComparer = defaultComparer

	w.Root = &cobra.Command{
		Use:   "wal",
		Short: "WAL introspection tools",
	}
	w.Dump = &cobra.Command{
		Use:   "dump <wal-files>",
		Short: "print WAL contents",
		Long: `
Print the contents of the WAL files.
`,
		Args: cobra.MinimumNArgs(1),
		Run:  w.runDump,
	}

	w.Root.AddCommand(w.Dump)
	w.Root.PersistentFlags().BoolVarP(&w.verbose, "verbose", "v", false, "verbose output")

	w.Dump.Flags().Var(
		&w.fmtKey, "key", "key formatter")
	w.Dump.Flags().Var(
		&w.fmtValue, "value", "value formatter")
	return w
}

func (w *walT) runDump(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.OutOrStderr()
	w.fmtKey.setForComparer(w.defaultComparer, w.comparers)
	w.fmtValue.setForComparer(w.defaultComparer, w.comparers)

	for _, arg := range args {
		func() {
			// Parse the filename in order to extract the file number. This is
			// necessary in case WAL recycling was used (which it is usually is). If
			// we can't parse the filename or it isn't a log file, we'll plow ahead
			// anyways (which will likely fail when we try to read the file).
			_, fileNum, ok := base.ParseFilename(w.opts.FS, arg)
			if !ok {
				fileNum = base.FileNum(0).DiskFileNum()
			}

			f, err := w.opts.FS.Open(arg)
			if err != nil {
				fmt.Fprintf(stderr, "%s\n", err)
				return
			}
			defer f.Close()

			fmt.Fprintf(stdout, "%s\n", arg)

			var b pebble.Batch
			var buf bytes.Buffer
			rr := record.NewReader(f, fileNum.FileNum())
			for {
				offset := rr.Offset()
				r, err := rr.Next()
				if err == nil {
					buf.Reset()
					_, err = io.Copy(&buf, r)
				}
				if err != nil {
					// It is common to encounter a zeroed or invalid chunk due to WAL
					// preallocation and WAL recycling. We need to distinguish these
					// errors from EOF in order to recognize that the record was
					// truncated, but want to otherwise treat them like EOF.
					switch err {
					case record.ErrZeroedChunk:
						fmt.Fprintf(stdout, "EOF [%s] (may be due to WAL preallocation)\n", err)
					case record.ErrInvalidChunk:
						fmt.Fprintf(stdout, "EOF [%s] (may be due to WAL recycling)\n", err)
					default:
						fmt.Fprintf(stdout, "%s\n", err)
					}
					return
				}

				b = pebble.Batch{}
				if err := b.SetRepr(buf.Bytes()); err != nil {
					fmt.Fprintf(stdout, "corrupt batch within log file %q: %v", arg, err)
					return
				}
				fmt.Fprintf(stdout, "%d(%d) seq=%d count=%d\n",
					offset, len(b.Repr()), b.SeqNum(), b.Count())
				for r, idx := b.Reader(), 0; ; idx++ {
					kind, ukey, value, ok, err := r.Next()
					if !ok {
						if err != nil {
							fmt.Fprintf(stdout, "corrupt batch within log file %q: %v", arg, err)
						}
						break
					}
					fmt.Fprintf(stdout, "    %s(", kind)
					switch kind {
					case base.InternalKeyKindDelete:
						fmt.Fprintf(stdout, "%s", w.fmtKey.fn(ukey))
					case base.InternalKeyKindSet:
						fmt.Fprintf(stdout, "%s,%s", w.fmtKey.fn(ukey), w.fmtValue.fn(ukey, value))
					case base.InternalKeyKindMerge:
						fmt.Fprintf(stdout, "%s,%s", w.fmtKey.fn(ukey), w.fmtValue.fn(ukey, value))
					case base.InternalKeyKindLogData:
						fmt.Fprintf(stdout, "<%d>", len(value))
					case base.InternalKeyKindIngestSST:
						fileNum, _ := binary.Uvarint(ukey)
						fmt.Fprintf(stdout, "%s", base.FileNum(fileNum))
					case base.InternalKeyKindSingleDelete:
						fmt.Fprintf(stdout, "%s", w.fmtKey.fn(ukey))
					case base.InternalKeyKindSetWithDelete:
						fmt.Fprintf(stdout, "%s", w.fmtKey.fn(ukey))
					case base.InternalKeyKindRangeDelete:
						fmt.Fprintf(stdout, "%s,%s", w.fmtKey.fn(ukey), w.fmtKey.fn(value))
					case base.InternalKeyKindRangeKeySet, base.InternalKeyKindRangeKeyUnset, base.InternalKeyKindRangeKeyDelete:
						ik := base.MakeInternalKey(ukey, b.SeqNum()+uint64(idx), kind)
						s, err := rangekey.Decode(ik, value, nil)
						if err != nil {
							fmt.Fprintf(stdout, "%s: error decoding %s", w.fmtKey.fn(ukey), err)
						} else {
							fmt.Fprintf(stdout, "%s", s.Pretty(w.fmtKey.fn))
						}
					case base.InternalKeyKindDeleteSized:
						v, _ := binary.Uvarint(value)
						fmt.Fprintf(stdout, "%s,%d", w.fmtKey.fn(ukey), v)
					}
					fmt.Fprintf(stdout, ")\n")
				}
			}
		}()
	}
}
