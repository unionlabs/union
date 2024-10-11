// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"context"
	"fmt"
	"io"
	"text/tabwriter"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/tool/logs"
	"github.com/spf13/cobra"
)

// dbT implements db-level tools, including both configuration state and the
// commands themselves.
type dbT struct {
	Root       *cobra.Command
	Check      *cobra.Command
	Checkpoint *cobra.Command
	Get        *cobra.Command
	Logs       *cobra.Command
	LSM        *cobra.Command
	Properties *cobra.Command
	Scan       *cobra.Command
	Set        *cobra.Command
	Space      *cobra.Command
	IOBench    *cobra.Command

	// Configuration.
	opts            *pebble.Options
	comparers       sstable.Comparers
	mergers         sstable.Mergers
	openErrEnhancer func(error) error

	// Flags.
	comparerName  string
	mergerName    string
	fmtKey        keyFormatter
	fmtValue      valueFormatter
	start         key
	end           key
	count         int64
	allLevels     bool
	ioCount       int
	ioParallelism int
	ioSizes       string
	verbose       bool
}

func newDB(
	opts *pebble.Options,
	comparers sstable.Comparers,
	mergers sstable.Mergers,
	openErrEnhancer func(error) error,
) *dbT {
	d := &dbT{
		opts:            opts,
		comparers:       comparers,
		mergers:         mergers,
		openErrEnhancer: openErrEnhancer,
	}
	d.fmtKey.mustSet("quoted")
	d.fmtValue.mustSet("[%x]")

	d.Root = &cobra.Command{
		Use:   "db",
		Short: "DB introspection tools",
	}
	d.Check = &cobra.Command{
		Use:   "check <dir>",
		Short: "verify checksums and metadata",
		Long: `
Verify sstable, manifest, and WAL checksums. Requires that the specified
database not be in use by another process.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runCheck,
	}
	d.Checkpoint = &cobra.Command{
		Use:   "checkpoint <src-dir> <dest-dir>",
		Short: "create a checkpoint",
		Long: `
Creates a Pebble checkpoint in the specified destination directory. A checkpoint
is a point-in-time snapshot of DB state. Requires that the specified
database not be in use by another process.
`,
		Args: cobra.ExactArgs(2),
		Run:  d.runCheckpoint,
	}
	d.Get = &cobra.Command{
		Use:   "get <dir> <key>",
		Short: "get value for a key",
		Long: `
Gets a value for a key, if it exists in DB. Prints a "not found" error if key
does not exist. Requires that the specified database not be in use by another
process.
`,
		Args: cobra.ExactArgs(2),
		Run:  d.runGet,
	}
	d.Logs = logs.NewCmd()
	d.LSM = &cobra.Command{
		Use:   "lsm <dir>",
		Short: "print LSM structure",
		Long: `
Print the structure of the LSM tree. Requires that the specified database not
be in use by another process.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runLSM,
	}
	d.Properties = &cobra.Command{
		Use:   "properties <dir>",
		Short: "print aggregated sstable properties",
		Long: `
Print SSTable properties, aggregated per level of the LSM.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runProperties,
	}
	d.Scan = &cobra.Command{
		Use:   "scan <dir>",
		Short: "print db records",
		Long: `
Print the records in the DB. Requires that the specified database not be in use
by another process.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runScan,
	}
	d.Set = &cobra.Command{
		Use:   "set <dir> <key> <value>",
		Short: "set a value for a key",
		Long: `
Adds a new key/value to the DB. Requires that the specified database
not be in use by another process.
`,
		Args: cobra.ExactArgs(3),
		Run:  d.runSet,
	}
	d.Space = &cobra.Command{
		Use:   "space <dir>",
		Short: "print filesystem space used",
		Long: `
Print the estimated filesystem space usage for the inclusive-inclusive range
specified by --start and --end. Requires that the specified database not be in
use by another process.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runSpace,
	}
	d.IOBench = &cobra.Command{
		Use:   "io-bench <dir>",
		Short: "perform sstable IO benchmark",
		Long: `
Run a random IO workload with various IO sizes against the sstables in the
specified database.
`,
		Args: cobra.ExactArgs(1),
		Run:  d.runIOBench,
	}

	d.Root.AddCommand(d.Check, d.Checkpoint, d.Get, d.Logs, d.LSM, d.Properties, d.Scan, d.Set, d.Space, d.IOBench)
	d.Root.PersistentFlags().BoolVarP(&d.verbose, "verbose", "v", false, "verbose output")

	for _, cmd := range []*cobra.Command{d.Check, d.Checkpoint, d.Get, d.LSM, d.Properties, d.Scan, d.Set, d.Space} {
		cmd.Flags().StringVar(
			&d.comparerName, "comparer", "", "comparer name (use default if empty)")
		cmd.Flags().StringVar(
			&d.mergerName, "merger", "", "merger name (use default if empty)")
	}

	for _, cmd := range []*cobra.Command{d.Scan, d.Space} {
		cmd.Flags().Var(
			&d.start, "start", "start key for the range")
		cmd.Flags().Var(
			&d.end, "end", "end key for the range")
	}

	d.Scan.Flags().Var(
		&d.fmtKey, "key", "key formatter")
	for _, cmd := range []*cobra.Command{d.Scan, d.Get} {
		cmd.Flags().Var(
			&d.fmtValue, "value", "value formatter")
	}

	d.Scan.Flags().Int64Var(
		&d.count, "count", 0, "key count for scan (0 is unlimited)")

	d.IOBench.Flags().BoolVar(
		&d.allLevels, "all-levels", false, "if set, benchmark all levels (default is only L5/L6)")
	d.IOBench.Flags().IntVar(
		&d.ioCount, "io-count", 10000, "number of IOs (per IO size) to benchmark")
	d.IOBench.Flags().IntVar(
		&d.ioParallelism, "io-parallelism", 16, "number of goroutines issuing IO")
	d.IOBench.Flags().StringVar(
		&d.ioSizes, "io-sizes-kb", "4,16,64,128,256,512,1024", "comma separated list of IO sizes in KB")

	return d
}

func (d *dbT) loadOptions(dir string) error {
	ls, err := d.opts.FS.List(dir)
	if err != nil || len(ls) == 0 {
		// NB: We don't return the error here as we prefer to return the error from
		// pebble.Open. Another way to put this is that a non-existent directory is
		// not a failure in loading the options.
		return nil
	}

	hooks := &pebble.ParseHooks{
		NewComparer: func(name string) (*pebble.Comparer, error) {
			if c := d.comparers[name]; c != nil {
				return c, nil
			}
			return nil, errors.Errorf("unknown comparer %q", errors.Safe(name))
		},
		NewMerger: func(name string) (*pebble.Merger, error) {
			if m := d.mergers[name]; m != nil {
				return m, nil
			}
			return nil, errors.Errorf("unknown merger %q", errors.Safe(name))
		},
		SkipUnknown: func(name, value string) bool {
			return true
		},
	}

	// TODO(peter): RocksDB sometimes leaves multiple OPTIONS files in
	// existence. We parse all of them as the comparer and merger shouldn't be
	// changing. We could parse only the first or the latest. Not clear if this
	// matters.
	var dbOpts pebble.Options
	for _, filename := range ls {
		ft, _, ok := base.ParseFilename(d.opts.FS, filename)
		if !ok {
			continue
		}
		switch ft {
		case base.FileTypeOptions:
			err := func() error {
				f, err := d.opts.FS.Open(d.opts.FS.PathJoin(dir, filename))
				if err != nil {
					return err
				}
				defer f.Close()

				data, err := io.ReadAll(f)
				if err != nil {
					return err
				}

				if err := dbOpts.Parse(string(data), hooks); err != nil {
					return err
				}
				return nil
			}()
			if err != nil {
				return err
			}
		}
	}

	if dbOpts.Comparer != nil {
		d.opts.Comparer = dbOpts.Comparer
	}
	if dbOpts.Merger != nil {
		d.opts.Merger = dbOpts.Merger
	}
	return nil
}

type openOption interface {
	apply(opts *pebble.Options)
}

func (d *dbT) openDB(dir string, openOptions ...openOption) (*pebble.DB, error) {
	db, err := d.openDBInternal(dir, openOptions...)
	if err != nil {
		if d.openErrEnhancer != nil {
			err = d.openErrEnhancer(err)
		}
		return nil, err
	}
	return db, nil
}

func (d *dbT) openDBInternal(dir string, openOptions ...openOption) (*pebble.DB, error) {
	if err := d.loadOptions(dir); err != nil {
		return nil, errors.Wrap(err, "error loading options")
	}
	if d.comparerName != "" {
		d.opts.Comparer = d.comparers[d.comparerName]
		if d.opts.Comparer == nil {
			return nil, errors.Errorf("unknown comparer %q", errors.Safe(d.comparerName))
		}
	}
	if d.mergerName != "" {
		d.opts.Merger = d.mergers[d.mergerName]
		if d.opts.Merger == nil {
			return nil, errors.Errorf("unknown merger %q", errors.Safe(d.mergerName))
		}
	}
	opts := *d.opts
	for _, opt := range openOptions {
		opt.apply(&opts)
	}
	opts.Cache = pebble.NewCache(128 << 20 /* 128 MB */)
	defer opts.Cache.Unref()
	return pebble.Open(dir, &opts)
}

func (d *dbT) closeDB(stderr io.Writer, db *pebble.DB) {
	if err := db.Close(); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
	}
}

func (d *dbT) runCheck(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)

	var stats pebble.CheckLevelsStats
	if err := db.CheckLevels(&stats); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
	}
	fmt.Fprintf(stdout, "checked %d %s and %d %s\n",
		stats.NumPoints, makePlural("point", stats.NumPoints), stats.NumTombstones, makePlural("tombstone", int64(stats.NumTombstones)))
}

type nonReadOnly struct{}

func (n nonReadOnly) apply(opts *pebble.Options) {
	opts.ReadOnly = false
	// Increase the L0 compaction threshold to reduce the likelihood of an
	// unintended compaction changing test output.
	opts.L0CompactionThreshold = 10
}

func (d *dbT) runCheckpoint(cmd *cobra.Command, args []string) {
	stderr := cmd.ErrOrStderr()
	db, err := d.openDB(args[0], nonReadOnly{})
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)
	destDir := args[1]

	if err := db.Checkpoint(destDir); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
	}
}

func (d *dbT) runGet(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)
	var k key
	if err := k.Set(args[1]); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}

	val, closer, err := db.Get(k)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer func() {
		if closer != nil {
			closer.Close()
		}
	}()
	if val != nil {
		fmt.Fprintf(stdout, "%s\n", d.fmtValue.fn(k, val))
	}
}

func (d *dbT) runLSM(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)

	fmt.Fprintf(stdout, "%s", db.Metrics())
}

func (d *dbT) runScan(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)

	// Update the internal formatter if this comparator has one specified.
	if d.opts.Comparer != nil {
		d.fmtKey.setForComparer(d.opts.Comparer.Name, d.comparers)
		d.fmtValue.setForComparer(d.opts.Comparer.Name, d.comparers)
	}

	start := timeNow()
	fmtKeys := d.fmtKey.spec != "null"
	fmtValues := d.fmtValue.spec != "null"
	var count int64

	iter, _ := db.NewIter(&pebble.IterOptions{
		UpperBound: d.end,
	})
	for valid := iter.SeekGE(d.start); valid; valid = iter.Next() {
		if fmtKeys || fmtValues {
			needDelimiter := false
			if fmtKeys {
				fmt.Fprintf(stdout, "%s", d.fmtKey.fn(iter.Key()))
				needDelimiter = true
			}
			if fmtValues {
				if needDelimiter {
					stdout.Write([]byte{' '})
				}
				fmt.Fprintf(stdout, "%s", d.fmtValue.fn(iter.Key(), iter.Value()))
			}
			stdout.Write([]byte{'\n'})
		}

		count++
		if d.count > 0 && count >= d.count {
			break
		}
	}

	if err := iter.Close(); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
	}

	elapsed := timeNow().Sub(start)

	fmt.Fprintf(stdout, "scanned %d %s in %0.1fs\n",
		count, makePlural("record", count), elapsed.Seconds())
}

func (d *dbT) runSpace(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stdout, db)

	bytes, err := db.EstimateDiskUsage(d.start, d.end)
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	fmt.Fprintf(stdout, "%d\n", bytes)
}

func (d *dbT) runProperties(cmd *cobra.Command, args []string) {
	stdout, stderr := cmd.OutOrStdout(), cmd.ErrOrStderr()
	dirname := args[0]
	err := func() error {
		desc, err := pebble.Peek(dirname, d.opts.FS)
		if err != nil {
			return err
		} else if !desc.Exists {
			return oserror.ErrNotExist
		}
		manifestFilename := d.opts.FS.PathBase(desc.ManifestFilename)

		// Replay the manifest to get the current version.
		f, err := d.opts.FS.Open(desc.ManifestFilename)
		if err != nil {
			return errors.Wrapf(err, "pebble: could not open MANIFEST file %q", manifestFilename)
		}
		defer f.Close()

		cmp := base.DefaultComparer
		var bve manifest.BulkVersionEdit
		bve.AddedByFileNum = make(map[base.FileNum]*manifest.FileMetadata)
		rr := record.NewReader(f, 0 /* logNum */)
		for {
			r, err := rr.Next()
			if err == io.EOF {
				break
			}
			if err != nil {
				return errors.Wrapf(err, "pebble: reading manifest %q", manifestFilename)
			}
			var ve manifest.VersionEdit
			err = ve.Decode(r)
			if err != nil {
				return err
			}
			if err := bve.Accumulate(&ve); err != nil {
				return err
			}
			if ve.ComparerName != "" {
				cmp = d.comparers[ve.ComparerName]
				d.fmtKey.setForComparer(ve.ComparerName, d.comparers)
				d.fmtValue.setForComparer(ve.ComparerName, d.comparers)
			}
		}
		v, err := bve.Apply(
			nil /* version */, cmp.Compare, d.fmtKey.fn, d.opts.FlushSplitBytes,
			d.opts.Experimental.ReadCompactionRate, nil, /* zombies */
			manifest.AllowSplitUserKeys,
		)
		if err != nil {
			return err
		}

		objProvider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(d.opts.FS, dirname))
		if err != nil {
			return err
		}
		defer objProvider.Close()

		// Load and aggregate sstable properties.
		tw := tabwriter.NewWriter(stdout, 2, 1, 4, ' ', 0)
		var total props
		var all []props
		for _, l := range v.Levels {
			iter := l.Iter()
			var level props
			for t := iter.First(); t != nil; t = iter.Next() {
				if t.Virtual {
					// TODO(bananabrick): Handle virtual sstables here. We don't
					// really have any stats or properties at this point. Maybe
					// we could approximate some of these properties for virtual
					// sstables by first grabbing properties for the backing
					// physical sstable, and then extrapolating.
					continue
				}
				err := d.addProps(objProvider, t.PhysicalMeta(), &level)
				if err != nil {
					return err
				}
			}
			all = append(all, level)
			total.update(level)
		}
		all = append(all, total)

		fmt.Fprintln(tw, "\tL0\tL1\tL2\tL3\tL4\tL5\tL6\tTOTAL")

		fmt.Fprintf(tw, "count\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n",
			propArgs(all, func(p *props) interface{} { return p.Count })...)

		fmt.Fprintln(tw, "seq num\t\t\t\t\t\t\t\t")
		fmt.Fprintf(tw, "  smallest\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n",
			propArgs(all, func(p *props) interface{} { return p.SmallestSeqNum })...)
		fmt.Fprintf(tw, "  largest\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n",
			propArgs(all, func(p *props) interface{} { return p.LargestSeqNum })...)

		fmt.Fprintln(tw, "size\t\t\t\t\t\t\t\t")
		fmt.Fprintf(tw, "  data\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.DataSize) })...)
		fmt.Fprintf(tw, "    blocks\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n",
			propArgs(all, func(p *props) interface{} { return p.NumDataBlocks })...)
		fmt.Fprintf(tw, "  index\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.IndexSize) })...)
		fmt.Fprintf(tw, "    blocks\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n",
			propArgs(all, func(p *props) interface{} { return p.NumIndexBlocks })...)
		fmt.Fprintf(tw, "    top-level\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.TopLevelIndexSize) })...)
		fmt.Fprintf(tw, "  filter\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.FilterSize) })...)
		fmt.Fprintf(tw, "  raw-key\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.RawKeySize) })...)
		fmt.Fprintf(tw, "  raw-value\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.RawValueSize) })...)
		fmt.Fprintf(tw, "  pinned-key\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.SnapshotPinnedKeySize) })...)
		fmt.Fprintf(tw, "  pinned-value\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.SnapshotPinnedValueSize) })...)
		fmt.Fprintf(tw, "  point-del-key-size\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.RawPointTombstoneKeySize) })...)
		fmt.Fprintf(tw, "  point-del-value-size\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Bytes.Uint64(p.RawPointTombstoneValueSize) })...)

		fmt.Fprintln(tw, "records\t\t\t\t\t\t\t\t")
		fmt.Fprintf(tw, "  set\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} {
				return humanize.Count.Uint64(p.NumEntries - p.NumDeletions - p.NumMergeOperands)
			})...)
		fmt.Fprintf(tw, "  delete\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumDeletions - p.NumRangeDeletions) })...)
		fmt.Fprintf(tw, "  delete-sized\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumSizedDeletions) })...)
		fmt.Fprintf(tw, "  range-delete\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumRangeDeletions) })...)
		fmt.Fprintf(tw, "  range-key-sets\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumRangeKeySets) })...)
		fmt.Fprintf(tw, "  range-key-unsets\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumRangeKeyUnSets) })...)
		fmt.Fprintf(tw, "  range-key-deletes\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumRangeKeyDeletes) })...)
		fmt.Fprintf(tw, "  merge\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.NumMergeOperands) })...)
		fmt.Fprintf(tw, "  pinned\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n",
			propArgs(all, func(p *props) interface{} { return humanize.Count.Uint64(p.SnapshotPinnedKeys) })...)

		if err := tw.Flush(); err != nil {
			return err
		}
		return nil
	}()
	if err != nil {
		fmt.Fprintln(stderr, err)
	}
}

func (d *dbT) runSet(cmd *cobra.Command, args []string) {
	stderr := cmd.ErrOrStderr()
	db, err := d.openDB(args[0], nonReadOnly{})
	if err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	defer d.closeDB(stderr, db)
	var k, v key
	if err := k.Set(args[1]); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}
	if err := v.Set(args[2]); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
		return
	}

	if err := db.Set(k, v, nil); err != nil {
		fmt.Fprintf(stderr, "%s\n", err)
	}
}

func propArgs(props []props, getProp func(*props) interface{}) []interface{} {
	args := make([]interface{}, 0, len(props))
	for _, p := range props {
		args = append(args, getProp(&p))
	}
	return args
}

type props struct {
	Count                      uint64
	SmallestSeqNum             uint64
	LargestSeqNum              uint64
	DataSize                   uint64
	FilterSize                 uint64
	IndexSize                  uint64
	NumDataBlocks              uint64
	NumIndexBlocks             uint64
	NumDeletions               uint64
	NumSizedDeletions          uint64
	NumEntries                 uint64
	NumMergeOperands           uint64
	NumRangeDeletions          uint64
	NumRangeKeySets            uint64
	NumRangeKeyUnSets          uint64
	NumRangeKeyDeletes         uint64
	RawKeySize                 uint64
	RawPointTombstoneKeySize   uint64
	RawPointTombstoneValueSize uint64
	RawValueSize               uint64
	SnapshotPinnedKeys         uint64
	SnapshotPinnedKeySize      uint64
	SnapshotPinnedValueSize    uint64
	TopLevelIndexSize          uint64
}

func (p *props) update(o props) {
	p.Count += o.Count
	if o.SmallestSeqNum != 0 && (o.SmallestSeqNum < p.SmallestSeqNum || p.SmallestSeqNum == 0) {
		p.SmallestSeqNum = o.SmallestSeqNum
	}
	if o.LargestSeqNum > p.LargestSeqNum {
		p.LargestSeqNum = o.LargestSeqNum
	}
	p.DataSize += o.DataSize
	p.FilterSize += o.FilterSize
	p.IndexSize += o.IndexSize
	p.NumDataBlocks += o.NumDataBlocks
	p.NumIndexBlocks += o.NumIndexBlocks
	p.NumDeletions += o.NumDeletions
	p.NumSizedDeletions += o.NumSizedDeletions
	p.NumEntries += o.NumEntries
	p.NumMergeOperands += o.NumMergeOperands
	p.NumRangeDeletions += o.NumRangeDeletions
	p.NumRangeKeySets += o.NumRangeKeySets
	p.NumRangeKeyUnSets += o.NumRangeKeyUnSets
	p.NumRangeKeyDeletes += o.NumRangeKeyDeletes
	p.RawKeySize += o.RawKeySize
	p.RawPointTombstoneKeySize += o.RawPointTombstoneKeySize
	p.RawPointTombstoneValueSize += o.RawPointTombstoneValueSize
	p.RawValueSize += o.RawValueSize
	p.SnapshotPinnedKeySize += o.SnapshotPinnedKeySize
	p.SnapshotPinnedValueSize += o.SnapshotPinnedValueSize
	p.SnapshotPinnedKeys += o.SnapshotPinnedKeys
	p.TopLevelIndexSize += o.TopLevelIndexSize
}

func (d *dbT) addProps(
	objProvider objstorage.Provider, m manifest.PhysicalFileMeta, p *props,
) error {
	ctx := context.Background()
	f, err := objProvider.OpenForReading(ctx, base.FileTypeTable, m.FileBacking.DiskFileNum, objstorage.OpenOptions{})
	if err != nil {
		return err
	}
	r, err := sstable.NewReader(f, sstable.ReaderOptions{}, d.mergers, d.comparers)
	if err != nil {
		_ = f.Close()
		return err
	}
	p.update(props{
		Count:                      1,
		SmallestSeqNum:             m.SmallestSeqNum,
		LargestSeqNum:              m.LargestSeqNum,
		DataSize:                   r.Properties.DataSize,
		FilterSize:                 r.Properties.FilterSize,
		IndexSize:                  r.Properties.IndexSize,
		NumDataBlocks:              r.Properties.NumDataBlocks,
		NumIndexBlocks:             1 + r.Properties.IndexPartitions,
		NumDeletions:               r.Properties.NumDeletions,
		NumSizedDeletions:          r.Properties.NumSizedDeletions,
		NumEntries:                 r.Properties.NumEntries,
		NumMergeOperands:           r.Properties.NumMergeOperands,
		NumRangeDeletions:          r.Properties.NumRangeDeletions,
		NumRangeKeySets:            r.Properties.NumRangeKeySets,
		NumRangeKeyUnSets:          r.Properties.NumRangeKeyUnsets,
		NumRangeKeyDeletes:         r.Properties.NumRangeKeyDels,
		RawKeySize:                 r.Properties.RawKeySize,
		RawPointTombstoneKeySize:   r.Properties.RawPointTombstoneKeySize,
		RawPointTombstoneValueSize: r.Properties.RawPointTombstoneValueSize,
		RawValueSize:               r.Properties.RawValueSize,
		SnapshotPinnedKeySize:      r.Properties.SnapshotPinnedKeySize,
		SnapshotPinnedValueSize:    r.Properties.SnapshotPinnedValueSize,
		SnapshotPinnedKeys:         r.Properties.SnapshotPinnedKeys,
		TopLevelIndexSize:          r.Properties.TopLevelIndexSize,
	})
	return r.Close()
}

func makePlural(singular string, count int64) string {
	if count > 1 {
		return fmt.Sprintf("%ss", singular)
	}
	return singular
}
