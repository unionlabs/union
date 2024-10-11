// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"bytes"
	"fmt"
	"io"
	"sort"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/keyspan"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/internal/rangedel"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/spf13/cobra"
)

type findRef struct {
	key     base.InternalKey
	value   []byte
	fileNum base.FileNum
}

// findT implements the find tool.
//
// TODO(bananabrick): Add support for virtual sstables in this tool. Currently,
// the tool will work because we're parsing files from disk, so virtual sstables
// will never be added to findT.tables. The manifest could contain information
// about virtual sstables. This is fine because the manifest is only used to
// compute the findT.editRefs, and editRefs is only used if a file in
// findT.tables contains a key. Of course, the tool won't be completely
// accurate without dealing with virtual sstable case.
type findT struct {
	Root *cobra.Command

	// Configuration.
	opts      *pebble.Options
	comparers sstable.Comparers
	mergers   sstable.Mergers

	// Flags.
	comparerName string
	fmtKey       keyFormatter
	fmtValue     valueFormatter
	verbose      bool

	// Map from file num to path on disk.
	files map[base.FileNum]string
	// Map from file num to version edit index which references the file num.
	editRefs map[base.FileNum][]int
	// List of version edits.
	edits []manifest.VersionEdit
	// Sorted list of WAL file nums.
	logs []base.FileNum
	// Sorted list of manifest file nums.
	manifests []base.FileNum
	// Sorted list of table file nums.
	tables []base.FileNum
	// Set of tables that contains references to the search key.
	tableRefs map[base.FileNum]bool
	// Map from file num to table metadata.
	tableMeta map[base.FileNum]*manifest.FileMetadata
	// List of error messages for SSTables that could not be decoded.
	errors []string
}

func newFind(
	opts *pebble.Options,
	comparers sstable.Comparers,
	defaultComparer string,
	mergers sstable.Mergers,
) *findT {
	f := &findT{
		opts:      opts,
		comparers: comparers,
		mergers:   mergers,
	}
	f.fmtKey.mustSet("quoted")
	f.fmtValue.mustSet("[%x]")

	f.Root = &cobra.Command{
		Use:   "find <dir> <key>",
		Short: "find references to the specified key",
		Long: `
Find references to the specified key and any range tombstones that contain the
key. This includes references to the key in WAL files and sstables, and the
provenance of the sstables (flushed, ingested, compacted).
`,
		Args: cobra.ExactArgs(2),
		Run:  f.run,
	}

	f.Root.Flags().BoolVarP(
		&f.verbose, "verbose", "v", false, "verbose output")
	f.Root.Flags().StringVar(
		&f.comparerName, "comparer", defaultComparer, "comparer name")
	f.Root.Flags().Var(
		&f.fmtKey, "key", "key formatter")
	f.Root.Flags().Var(
		&f.fmtValue, "value", "value formatter")
	return f
}

func (f *findT) run(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.OutOrStderr()
	var key key
	if err := key.Set(args[1]); err != nil {
		fmt.Fprintf(stdout, "%s\n", err)
		return
	}

	if err := f.findFiles(stdout, stderr, args[0]); err != nil {
		fmt.Fprintf(stdout, "%s\n", err)
		return
	}
	f.readManifests(stdout)

	f.opts.Comparer = f.comparers[f.comparerName]
	if f.opts.Comparer == nil {
		fmt.Fprintf(stderr, "unknown comparer %q", f.comparerName)
		return
	}
	f.fmtKey.setForComparer(f.opts.Comparer.Name, f.comparers)
	f.fmtValue.setForComparer(f.opts.Comparer.Name, f.comparers)

	refs := f.search(stdout, key)
	var lastFileNum base.FileNum
	for i := range refs {
		r := &refs[i]
		if lastFileNum != r.fileNum {
			lastFileNum = r.fileNum
			fmt.Fprintf(stdout, "%s", f.opts.FS.PathBase(f.files[r.fileNum]))
			if m := f.tableMeta[r.fileNum]; m != nil {
				fmt.Fprintf(stdout, " ")
				formatKeyRange(stdout, f.fmtKey, &m.Smallest, &m.Largest)
			}
			fmt.Fprintf(stdout, "\n")
			if p := f.tableProvenance(r.fileNum); p != "" {
				fmt.Fprintf(stdout, "    (%s)\n", p)
			}
		}
		fmt.Fprintf(stdout, "    ")
		formatKeyValue(stdout, f.fmtKey, f.fmtValue, &r.key, r.value)
	}

	for _, errorMsg := range f.errors {
		fmt.Fprint(stdout, errorMsg)
	}
}

// Find all of the manifests, logs, and tables in the specified directory.
func (f *findT) findFiles(stdout, stderr io.Writer, dir string) error {
	f.files = make(map[base.FileNum]string)
	f.editRefs = make(map[base.FileNum][]int)
	f.logs = nil
	f.manifests = nil
	f.tables = nil
	f.tableMeta = make(map[base.FileNum]*manifest.FileMetadata)

	if _, err := f.opts.FS.Stat(dir); err != nil {
		return err
	}

	walk(stderr, f.opts.FS, dir, func(path string) {
		ft, fileNum, ok := base.ParseFilename(f.opts.FS, path)
		if !ok {
			return
		}
		switch ft {
		case base.FileTypeLog:
			f.logs = append(f.logs, fileNum.FileNum())
		case base.FileTypeManifest:
			f.manifests = append(f.manifests, fileNum.FileNum())
		case base.FileTypeTable:
			f.tables = append(f.tables, fileNum.FileNum())
		default:
			return
		}
		f.files[fileNum.FileNum()] = path
	})

	sort.Slice(f.logs, func(i, j int) bool {
		return f.logs[i] < f.logs[j]
	})
	sort.Slice(f.manifests, func(i, j int) bool {
		return f.manifests[i] < f.manifests[j]
	})
	sort.Slice(f.tables, func(i, j int) bool {
		return f.tables[i] < f.tables[j]
	})

	if f.verbose {
		fmt.Fprintf(stdout, "%s\n", dir)
		fmt.Fprintf(stdout, "%5d %s\n", len(f.manifests), makePlural("manifest", int64(len(f.manifests))))
		fmt.Fprintf(stdout, "%5d %s\n", len(f.logs), makePlural("log", int64(len(f.logs))))
		fmt.Fprintf(stdout, "%5d %s\n", len(f.tables), makePlural("sstable", int64(len(f.tables))))
	}
	return nil
}

// Read the manifests and populate the editRefs map which is used to determine
// the provenance and metadata of tables.
func (f *findT) readManifests(stdout io.Writer) {
	for _, fileNum := range f.manifests {
		func() {
			path := f.files[fileNum]
			mf, err := f.opts.FS.Open(path)
			if err != nil {
				fmt.Fprintf(stdout, "%s\n", err)
				return
			}
			defer mf.Close()

			if f.verbose {
				fmt.Fprintf(stdout, "%s\n", path)
			}

			rr := record.NewReader(mf, 0 /* logNum */)
			for {
				r, err := rr.Next()
				if err != nil {
					if err != io.EOF {
						fmt.Fprintf(stdout, "%s: %s\n", path, err)
					}
					break
				}

				var ve manifest.VersionEdit
				if err := ve.Decode(r); err != nil {
					fmt.Fprintf(stdout, "%s: %s\n", path, err)
					break
				}
				i := len(f.edits)
				f.edits = append(f.edits, ve)

				if ve.ComparerName != "" {
					f.comparerName = ve.ComparerName
				}
				if num := ve.MinUnflushedLogNum; num != 0 {
					f.editRefs[num] = append(f.editRefs[num], i)
				}
				for df := range ve.DeletedFiles {
					f.editRefs[df.FileNum] = append(f.editRefs[df.FileNum], i)
				}
				for _, nf := range ve.NewFiles {
					// The same file can be deleted and added in a single version edit
					// which indicates a "move" compaction. Only add the edit to the list
					// once.
					refs := f.editRefs[nf.Meta.FileNum]
					if n := len(refs); n == 0 || refs[n-1] != i {
						f.editRefs[nf.Meta.FileNum] = append(refs, i)
					}
					if _, ok := f.tableMeta[nf.Meta.FileNum]; !ok {
						f.tableMeta[nf.Meta.FileNum] = nf.Meta
					}
				}
			}
		}()
	}

	if f.verbose {
		fmt.Fprintf(stdout, "%5d %s\n", len(f.edits), makePlural("edit", int64(len(f.edits))))
	}
}

// Search the logs and sstables for references to the specified key.
func (f *findT) search(stdout io.Writer, key []byte) []findRef {
	refs := f.searchLogs(stdout, key, nil)
	refs = f.searchTables(stdout, key, refs)

	// For a given file (log or table) the references are already in the correct
	// order. We simply want to order the references by fileNum using a stable
	// sort.
	//
	// TODO(peter): I'm not sure if this is perfectly correct with regards to log
	// files and ingested sstables, but it is close enough and doing something
	// better is onerous. Revisit if this ever becomes problematic (e.g. if we
	// allow finding more than one key at a time).
	//
	// An example of the problem with logs and ingestion (which can only occur
	// with distinct keys). If I write key "a" to a log, I can then ingested key
	// "b" without causing "a" to be flushed. Then I can write key "c" to the
	// log. Ideally, we'd show the key "a" from the log, then the key "b" from
	// the ingested sstable, then key "c" from the log.
	sort.SliceStable(refs, func(i, j int) bool {
		return refs[i].fileNum < refs[j].fileNum
	})
	return refs
}

// Search the logs for references to the specified key.
func (f *findT) searchLogs(stdout io.Writer, searchKey []byte, refs []findRef) []findRef {
	cmp := f.opts.Comparer.Compare
	for _, fileNum := range f.logs {
		_ = func() (err error) {
			path := f.files[fileNum]
			lf, err := f.opts.FS.Open(path)
			if err != nil {
				fmt.Fprintf(stdout, "%s\n", err)
				return
			}
			defer lf.Close()

			if f.verbose {
				fmt.Fprintf(stdout, "%s", path)
				defer fmt.Fprintf(stdout, "\n")
			}
			defer func() {
				switch err {
				case record.ErrZeroedChunk:
					if f.verbose {
						fmt.Fprintf(stdout, ": EOF [%s] (may be due to WAL preallocation)", err)
					}
				case record.ErrInvalidChunk:
					if f.verbose {
						fmt.Fprintf(stdout, ": EOF [%s] (may be due to WAL recycling)", err)
					}
				default:
					if err != io.EOF {
						if f.verbose {
							fmt.Fprintf(stdout, ": %s", err)
						} else {
							fmt.Fprintf(stdout, "%s: %s\n", path, err)
						}
					}
				}
			}()

			var b pebble.Batch
			var buf bytes.Buffer
			rr := record.NewReader(lf, fileNum)
			for {
				r, err := rr.Next()
				if err == nil {
					buf.Reset()
					_, err = io.Copy(&buf, r)
				}
				if err != nil {
					return err
				}

				b = pebble.Batch{}
				if err := b.SetRepr(buf.Bytes()); err != nil {
					fmt.Fprintf(stdout, "%s: corrupt log file: %v", path, err)
					continue
				}
				seqNum := b.SeqNum()
				for r := b.Reader(); ; seqNum++ {
					kind, ukey, value, ok, err := r.Next()
					if !ok {
						if err != nil {
							fmt.Fprintf(stdout, "%s: corrupt log file: %v", path, err)
							break
						}
						break
					}
					ikey := base.MakeInternalKey(ukey, seqNum, kind)
					switch kind {
					case base.InternalKeyKindDelete,
						base.InternalKeyKindDeleteSized,
						base.InternalKeyKindSet,
						base.InternalKeyKindMerge,
						base.InternalKeyKindSingleDelete,
						base.InternalKeyKindSetWithDelete:
						if cmp(searchKey, ikey.UserKey) != 0 {
							continue
						}
					case base.InternalKeyKindRangeDelete:
						// Output tombstones that contain or end with the search key.
						t := rangedel.Decode(ikey, value, nil)
						if !t.Contains(cmp, searchKey) && cmp(t.End, searchKey) != 0 {
							continue
						}
					default:
						continue
					}

					refs = append(refs, findRef{
						key:     ikey.Clone(),
						value:   append([]byte(nil), value...),
						fileNum: fileNum,
					})
				}
			}
		}()
	}
	return refs
}

// Search the tables for references to the specified key.
func (f *findT) searchTables(stdout io.Writer, searchKey []byte, refs []findRef) []findRef {
	cache := pebble.NewCache(128 << 20 /* 128 MB */)
	defer cache.Unref()

	f.tableRefs = make(map[base.FileNum]bool)
	for _, fileNum := range f.tables {
		_ = func() (err error) {
			path := f.files[fileNum]
			tf, err := f.opts.FS.Open(path)
			if err != nil {
				fmt.Fprintf(stdout, "%s\n", err)
				return
			}

			m := f.tableMeta[fileNum]
			if f.verbose {
				fmt.Fprintf(stdout, "%s", path)
				if m != nil && m.SmallestSeqNum == m.LargestSeqNum {
					fmt.Fprintf(stdout, ": global seqnum: %d", m.LargestSeqNum)
				}
				defer fmt.Fprintf(stdout, "\n")
			}
			defer func() {
				switch {
				case err != nil:
					if f.verbose {
						fmt.Fprintf(stdout, ": %v", err)
					} else {
						fmt.Fprintf(stdout, "%s: %v\n", path, err)
					}
				}
			}()

			opts := sstable.ReaderOptions{
				Cache:    cache,
				Comparer: f.opts.Comparer,
				Filters:  f.opts.Filters,
			}
			readable, err := sstable.NewSimpleReadable(tf)
			if err != nil {
				return err
			}
			r, err := sstable.NewReader(readable, opts, f.comparers, f.mergers,
				private.SSTableRawTombstonesOpt.(sstable.ReaderOption))
			if err != nil {
				f.errors = append(f.errors, fmt.Sprintf("Unable to decode sstable %s, %s", f.files[fileNum], err.Error()))
				// Ensure the error only gets printed once.
				err = nil
				return
			}
			defer r.Close()

			if m != nil && m.SmallestSeqNum == m.LargestSeqNum {
				r.Properties.GlobalSeqNum = m.LargestSeqNum
			}

			iter, err := r.NewIter(nil, nil)
			if err != nil {
				return err
			}
			defer iter.Close()
			key, value := iter.SeekGE(searchKey, base.SeekGEFlagsNone)

			// We configured sstable.Reader to return raw tombstones which requires a
			// bit more work here to put them in a form that can be iterated in
			// parallel with the point records.
			rangeDelIter, err := func() (keyspan.FragmentIterator, error) {
				iter, err := r.NewRawRangeDelIter()
				if err != nil {
					return nil, err
				}
				if iter == nil {
					return keyspan.NewIter(r.Compare, nil), nil
				}
				defer iter.Close()

				var tombstones []keyspan.Span
				for t := iter.First(); t != nil; t = iter.Next() {
					if !t.Contains(r.Compare, searchKey) {
						continue
					}
					tombstones = append(tombstones, t.ShallowClone())
				}

				sort.Slice(tombstones, func(i, j int) bool {
					return r.Compare(tombstones[i].Start, tombstones[j].Start) < 0
				})
				return keyspan.NewIter(r.Compare, tombstones), nil
			}()
			if err != nil {
				return err
			}

			defer rangeDelIter.Close()
			rangeDel := rangeDelIter.First()

			foundRef := false
			for key != nil || rangeDel != nil {
				if key != nil &&
					(rangeDel == nil || r.Compare(key.UserKey, rangeDel.Start) < 0) {
					if r.Compare(searchKey, key.UserKey) != 0 {
						key, value = nil, base.LazyValue{}
						continue
					}
					v, _, err := value.Value(nil)
					if err != nil {
						return err
					}
					refs = append(refs, findRef{
						key:     key.Clone(),
						value:   append([]byte(nil), v...),
						fileNum: fileNum,
					})
					key, value = iter.Next()
				} else {
					// Use rangedel.Encode to add a reference for each key
					// within the span.
					err := rangedel.Encode(rangeDel, func(k base.InternalKey, v []byte) error {
						refs = append(refs, findRef{
							key:     k.Clone(),
							value:   append([]byte(nil), v...),
							fileNum: fileNum,
						})
						return nil
					})
					if err != nil {
						return err
					}
					rangeDel = rangeDelIter.Next()
				}
				foundRef = true
			}

			if foundRef {
				f.tableRefs[fileNum] = true
			}
			return nil
		}()
	}
	return refs
}

// Determine the provenance of the specified table. We search the version edits
// for the first edit which created the table, and then analyze the edit to
// determine if it was a compaction, flush, or ingestion. Returns an empty
// string if the provenance of a table cannot be determined.
func (f *findT) tableProvenance(fileNum base.FileNum) string {
	editRefs := f.editRefs[fileNum]
	for len(editRefs) > 0 {
		ve := f.edits[editRefs[0]]
		editRefs = editRefs[1:]
		for _, nf := range ve.NewFiles {
			if fileNum != nf.Meta.FileNum {
				continue
			}

			var buf bytes.Buffer
			switch {
			case len(ve.DeletedFiles) > 0:
				// A version edit with deleted files is a compaction. The deleted
				// files are the inputs to the compaction. We're going to
				// reconstruct the input files and display those inputs that
				// contain the search key (i.e. are list in refs) and use an
				// ellipsis to indicate when there were other inputs that have
				// been elided.
				var sourceLevels []int
				levels := make(map[int][]base.FileNum)
				for df := range ve.DeletedFiles {
					files := levels[df.Level]
					if len(files) == 0 {
						sourceLevels = append(sourceLevels, df.Level)
					}
					levels[df.Level] = append(files, df.FileNum)
				}

				sort.Ints(sourceLevels)
				if sourceLevels[len(sourceLevels)-1] != nf.Level {
					sourceLevels = append(sourceLevels, nf.Level)
				}

				sep := " "
				fmt.Fprintf(&buf, "compacted")
				for _, level := range sourceLevels {
					files := levels[level]
					sort.Slice(files, func(i, j int) bool {
						return files[i] < files[j]
					})

					fmt.Fprintf(&buf, "%sL%d [", sep, level)
					sep = ""
					elided := false
					for _, fileNum := range files {
						if f.tableRefs[fileNum] {
							fmt.Fprintf(&buf, "%s%s", sep, fileNum)
							sep = " "
						} else {
							elided = true
						}
					}
					if elided {
						fmt.Fprintf(&buf, "%s...", sep)
					}
					fmt.Fprintf(&buf, "]")
					sep = " + "
				}

			case ve.MinUnflushedLogNum != 0:
				// A version edit with a min-unflushed log indicates a flush
				// operation.
				fmt.Fprintf(&buf, "flushed to L%d", nf.Level)

			case nf.Meta.SmallestSeqNum == nf.Meta.LargestSeqNum:
				// If the smallest and largest seqnum are the same, the file was
				// ingested. Note that this can also occur for a flushed sstable
				// that contains only a single key, though that would have
				// already been captured above.
				fmt.Fprintf(&buf, "ingested to L%d", nf.Level)

			default:
				// The provenance of the table is unclear. This is usually due to
				// the MANIFEST rolling over and taking a snapshot of the LSM
				// state.
				fmt.Fprintf(&buf, "added to L%d", nf.Level)
			}

			// After a table is created, it can be moved to a different level via a
			// move compaction. This is indicated by a version edit that deletes the
			// table from one level and adds the same table to a different
			// level. Loop over the remaining version edits for the table looking for
			// such moves.
			for len(editRefs) > 0 {
				ve := f.edits[editRefs[0]]
				editRefs = editRefs[1:]
				for _, nf := range ve.NewFiles {
					if fileNum == nf.Meta.FileNum {
						for df := range ve.DeletedFiles {
							if fileNum == df.FileNum {
								fmt.Fprintf(&buf, ", moved to L%d", nf.Level)
								break
							}
						}
						break
					}
				}
			}

			return buf.String()
		}
	}
	return ""
}
