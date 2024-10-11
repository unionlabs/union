// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"golang.org/x/exp/rand"
)

const (
	// The metamorphic test exercises range keys, so we cannot use an older
	// FormatMajorVersion than pebble.FormatRangeKeys.
	minimumFormatMajorVersion = pebble.FormatRangeKeys
	// The format major version to use in the default options configurations. We
	// default to the last format major version of Cockroach 22.2 so we exercise
	// the runtime version ratcheting that a cluster upgrading to 23.1 would
	// experience. The randomized options may still use format major versions
	// that are less than defaultFormatMajorVersion but are at least
	// minimumFormatMajorVersion.
	defaultFormatMajorVersion = pebble.FormatPrePebblev1Marked
	// newestFormatMajorVersionToTest is the most recent format major version
	// the metamorphic tests should use. This may be greater than
	// pebble.FormatNewest when some format major versions are marked as
	// experimental.
	newestFormatMajorVersionToTest = pebble.FormatNewest
)

func parseOptions(
	opts *TestOptions, data string, customOptionParsers map[string]func(string) (CustomOption, bool),
) error {
	hooks := &pebble.ParseHooks{
		NewCache:        pebble.NewCache,
		NewFilterPolicy: filterPolicyFromName,
		SkipUnknown: func(name, value string) bool {
			switch name {
			case "TestOptions":
				return true
			case "TestOptions.strictfs":
				opts.strictFS = true
				return true
			case "TestOptions.ingest_using_apply":
				opts.ingestUsingApply = true
				return true
			case "TestOptions.delete_sized":
				opts.deleteSized = true
				return true
			case "TestOptions.replace_single_delete":
				opts.replaceSingleDelete = true
				return true
			case "TestOptions.use_disk":
				opts.useDisk = true
				return true
			case "TestOptions.initial_state_desc":
				opts.initialStateDesc = value
				return true
			case "TestOptions.initial_state_path":
				opts.initialStatePath = value
				return true
			case "TestOptions.threads":
				v, err := strconv.Atoi(value)
				if err != nil {
					panic(err)
				}
				opts.threads = v
				return true
			case "TestOptions.disable_block_property_collector":
				v, err := strconv.ParseBool(value)
				if err != nil {
					panic(err)
				}
				opts.disableBlockPropertyCollector = v
				if v {
					opts.Opts.BlockPropertyCollectors = nil
				}
				return true
			case "TestOptions.enable_value_blocks":
				opts.enableValueBlocks = true
				opts.Opts.Experimental.EnableValueBlocks = func() bool { return true }
				return true
			case "TestOptions.async_apply_to_db":
				opts.asyncApplyToDB = true
				return true
			case "TestOptions.shared_storage_enabled":
				opts.sharedStorageEnabled = true
				opts.Opts.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
					"": remote.NewInMem(),
				})
				if opts.Opts.Experimental.CreateOnShared == remote.CreateOnSharedNone {
					opts.Opts.Experimental.CreateOnShared = remote.CreateOnSharedAll
				}
				return true
			case "TestOptions.secondary_cache_enabled":
				opts.secondaryCacheEnabled = true
				opts.Opts.Experimental.SecondaryCacheSizeBytes = 1024 * 1024 * 32 // 32 MBs
				return true
			case "TestOptions.seed_efos":
				v, err := strconv.ParseUint(value, 10, 64)
				if err != nil {
					panic(err)
				}
				opts.seedEFOS = v
				return true
			case "TestOptions.ingest_split":
				opts.ingestSplit = true
				opts.Opts.Experimental.IngestSplit = func() bool {
					return true
				}
				return true
			default:
				if customOptionParsers == nil {
					return false
				}
				name = strings.TrimPrefix(name, "TestOptions.")
				if p, ok := customOptionParsers[name]; ok {
					if customOpt, ok := p(value); ok {
						opts.CustomOpts = append(opts.CustomOpts, customOpt)
						return true
					}
				}
				return false
			}
		},
	}
	err := opts.Opts.Parse(data, hooks)
	opts.Opts.EnsureDefaults()
	return err
}

func optionsToString(opts *TestOptions) string {
	var buf bytes.Buffer
	if opts.strictFS {
		fmt.Fprint(&buf, "  strictfs=true\n")
	}
	if opts.ingestUsingApply {
		fmt.Fprint(&buf, "  ingest_using_apply=true\n")
	}
	if opts.deleteSized {
		fmt.Fprint(&buf, "  delete_sized=true\n")
	}
	if opts.replaceSingleDelete {
		fmt.Fprint(&buf, "  replace_single_delete=true\n")
	}
	if opts.useDisk {
		fmt.Fprint(&buf, "  use_disk=true\n")
	}
	if opts.initialStatePath != "" {
		fmt.Fprintf(&buf, "  initial_state_path=%s\n", opts.initialStatePath)
	}
	if opts.initialStateDesc != "" {
		fmt.Fprintf(&buf, "  initial_state_desc=%s\n", opts.initialStateDesc)
	}
	if opts.threads != 0 {
		fmt.Fprintf(&buf, "  threads=%d\n", opts.threads)
	}
	if opts.disableBlockPropertyCollector {
		fmt.Fprintf(&buf, "  disable_block_property_collector=%t\n", opts.disableBlockPropertyCollector)
	}
	if opts.enableValueBlocks {
		fmt.Fprintf(&buf, "  enable_value_blocks=%t\n", opts.enableValueBlocks)
	}
	if opts.asyncApplyToDB {
		fmt.Fprint(&buf, "  async_apply_to_db=true\n")
	}
	if opts.sharedStorageEnabled {
		fmt.Fprint(&buf, "  shared_storage_enabled=true\n")
	}
	if opts.secondaryCacheEnabled {
		fmt.Fprint(&buf, "  secondary_cache_enabled=true\n")
	}
	if opts.seedEFOS != 0 {
		fmt.Fprintf(&buf, "  seed_efos=%d\n", opts.seedEFOS)
	}
	if opts.ingestSplit {
		fmt.Fprintf(&buf, "  ingest_split=%v\n", opts.ingestSplit)
	}
	for _, customOpt := range opts.CustomOpts {
		fmt.Fprintf(&buf, "  %s=%s\n", customOpt.Name(), customOpt.Value())
	}

	s := opts.Opts.String()
	if buf.Len() == 0 {
		return s
	}
	return s + "\n[TestOptions]\n" + buf.String()
}

func defaultTestOptions() *TestOptions {
	return &TestOptions{
		Opts:    defaultOptions(),
		threads: 16,
	}
}

func defaultOptions() *pebble.Options {
	opts := &pebble.Options{
		Comparer:           testkeys.Comparer,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: defaultFormatMajorVersion,
		Levels: []pebble.LevelOptions{{
			FilterPolicy: bloom.FilterPolicy(10),
		}},
		BlockPropertyCollectors: blockPropertyCollectorConstructors,
	}
	// TODO(sumeer): add IneffectualSingleDeleteCallback that panics by
	// supporting a test option that does not generate ineffectual single
	// deletes.
	opts.Experimental.SingleDeleteInvariantViolationCallback = func(
		userKey []byte) {
		panic(errors.AssertionFailedf("single del invariant violations on key %q", userKey))
	}
	return opts
}

// TestOptions describes the options configuring an individual run of the
// metamorphic tests.
type TestOptions struct {
	// Opts holds the *pebble.Options for the test.
	Opts *pebble.Options
	// CustomOptions holds custom test options that are defined outside of this
	// package.
	CustomOpts []CustomOption
	useDisk    bool
	strictFS   bool
	threads    int
	// Use Batch.Apply rather than DB.Ingest.
	ingestUsingApply bool
	// Use Batch.DeleteSized rather than Batch.Delete.
	deleteSized bool
	// Replace a SINGLEDEL with a DELETE.
	replaceSingleDelete bool
	// The path on the local filesystem where the initial state of the database
	// exists.  Empty if the test run begins from an empty database state.
	initialStatePath string
	// A human-readable string describing the initial state of the database.
	// Empty if the test run begins from an empty database state.
	initialStateDesc string
	// Disable the block property collector, which may be used by block property
	// filters.
	disableBlockPropertyCollector bool
	// Enable the use of value blocks.
	enableValueBlocks bool
	// Use DB.ApplyNoSyncWait for applies that want to sync the WAL.
	asyncApplyToDB bool
	// Enable the use of shared storage.
	sharedStorageEnabled bool
	// Enable the secondary cache. Only effective if sharedStorageEnabled is
	// also true.
	secondaryCacheEnabled bool
	// If nonzero, enables the use of EventuallyFileOnlySnapshots for
	// newSnapshotOps that are keyspan-bounded. The set of which newSnapshotOps
	// are actually created as EventuallyFileOnlySnapshots is deterministically
	// derived from the seed and the operation index.
	seedEFOS uint64
	// Enables ingest splits. Saved here for serialization as Options does not
	// serialize this.
	ingestSplit bool
}

// CustomOption defines a custom option that configures the behavior of an
// individual test run. Like all test options, custom options are serialized to
// the OPTIONS file even if they're not options ordinarily understood by Pebble.
type CustomOption interface {
	// Name returns the name of the custom option. This is the key under which
	// the option appears in the OPTIONS file, within the [TestOptions] stanza.
	Name() string
	// Value returns the value of the custom option, serialized as it should
	// appear within the OPTIONS file.
	Value() string
	// Close is run after the test database has been closed at the end of the
	// test as well as during restart operations within the test sequence. It's
	// passed a copy of the *pebble.Options. If the custom options hold on to
	// any resources outside, Close should release them.
	Close(*pebble.Options) error
	// Open is run before the test runs and during a restart operation after the
	// test database has been closed and Close has been called. It's passed a
	// copy of the *pebble.Options. If the custom options must acquire any
	// resources before the test continues, it should reacquire them.
	Open(*pebble.Options) error

	// TODO(jackson): provide additional hooks for custom options changing the
	// behavior of a run.
}

func standardOptions() []*TestOptions {
	// The index labels are not strictly necessary, but they make it easier to
	// find which options correspond to a failure.
	stdOpts := []string{
		0: "", // default options
		1: `
[Options]
  cache_size=1
`,
		2: `
[Options]
  disable_wal=true
`,
		3: `
[Options]
  l0_compaction_threshold=1
`,
		4: `
[Options]
  l0_compaction_threshold=1
  l0_stop_writes_threshold=1
`,
		5: `
[Options]
  lbase_max_bytes=1
`,
		6: `
[Options]
  max_manifest_file_size=1
`,
		7: `
[Options]
  max_open_files=1
`,
		8: `
[Options]
  mem_table_size=2000
`,
		9: `
[Options]
  mem_table_stop_writes_threshold=2
`,
		10: `
[Options]
  wal_dir=data/wal
`,
		11: `
[Level "0"]
  block_restart_interval=1
`,
		12: `
[Level "0"]
  block_size=1
`,
		13: `
[Level "0"]
  compression=NoCompression
`,
		14: `
[Level "0"]
  index_block_size=1
`,
		15: `
[Level "0"]
  target_file_size=1
`,
		16: `
[Level "0"]
  filter_policy=none
`,
		// 1GB
		17: `
[Options]
  bytes_per_sync=1073741824
[TestOptions]
  strictfs=true
`,
		18: `
[Options]
  max_concurrent_compactions=2
`,
		19: `
[TestOptions]
  ingest_using_apply=true
`,
		20: `
[TestOptions]
  replace_single_delete=true
`,
		21: `
[TestOptions]
 use_disk=true
`,
		22: `
[Options]
  max_writer_concurrency=2
  force_writer_parallelism=true
`,
		23: `
[TestOptions]
  disable_block_property_collector=true
`,
		24: `
[TestOptions]
  threads=1
`,
		25: `
[TestOptions]
  enable_value_blocks=true
`,
		26: fmt.Sprintf(`
[Options]
  format_major_version=%s
`, newestFormatMajorVersionToTest),
		27: `
[TestOptions]
  shared_storage_enabled=true
  secondary_cache_enabled=true
`,
	}

	opts := make([]*TestOptions, len(stdOpts))
	for i := range opts {
		opts[i] = defaultTestOptions()
		// NB: The standard options by definition can never include custom
		// options, so no need to propagate custom option parsers.
		if err := parseOptions(opts[i], stdOpts[i], nil /* custom option parsers */); err != nil {
			panic(err)
		}
	}
	return opts
}

func randomOptions(
	rng *rand.Rand, customOptionParsers map[string]func(string) (CustomOption, bool),
) *TestOptions {
	testOpts := defaultTestOptions()
	opts := testOpts.Opts

	// There are some private options, which we don't want users to fiddle with.
	// There's no way to set it through the public interface. The only method is
	// through Parse.
	{
		var privateOpts bytes.Buffer
		fmt.Fprintln(&privateOpts, `[Options]`)
		if rng.Intn(3) == 0 /* 33% */ {
			fmt.Fprintln(&privateOpts, `  disable_delete_only_compactions=true`)
		}
		if rng.Intn(3) == 0 /* 33% */ {
			fmt.Fprintln(&privateOpts, `  disable_elision_only_compactions=true`)
		}
		if rng.Intn(5) == 0 /* 20% */ {
			fmt.Fprintln(&privateOpts, `  disable_lazy_combined_iteration=true`)
		}
		if privateOptsStr := privateOpts.String(); privateOptsStr != `[Options]\n` {
			parseOptions(testOpts, privateOptsStr, customOptionParsers)
		}
	}

	opts.BytesPerSync = 1 << uint(rng.Intn(28))     // 1B - 256MB
	opts.Cache = cache.New(1 << uint(rng.Intn(30))) // 1B - 1GB
	opts.DisableWAL = rng.Intn(2) == 0
	opts.FlushDelayDeleteRange = time.Millisecond * time.Duration(5*rng.Intn(245)) // 5-250ms
	opts.FlushDelayRangeKey = time.Millisecond * time.Duration(5*rng.Intn(245))    // 5-250ms
	opts.FlushSplitBytes = 1 << rng.Intn(20)                                       // 1B - 1MB
	opts.FormatMajorVersion = minimumFormatMajorVersion
	n := int(newestFormatMajorVersionToTest - opts.FormatMajorVersion)
	opts.FormatMajorVersion += pebble.FormatMajorVersion(rng.Intn(n + 1))
	opts.Experimental.L0CompactionConcurrency = 1 + rng.Intn(4) // 1-4
	opts.Experimental.LevelMultiplier = 5 << rng.Intn(7)        // 5 - 320
	opts.TargetByteDeletionRate = 1 << uint(20+rng.Intn(10))    // 1MB - 1GB
	opts.Experimental.ValidateOnIngest = rng.Intn(2) != 0
	opts.L0CompactionThreshold = 1 + rng.Intn(100)     // 1 - 100
	opts.L0CompactionFileThreshold = 1 << rng.Intn(11) // 1 - 1024
	opts.L0StopWritesThreshold = 1 + rng.Intn(100)     // 1 - 100
	if opts.L0StopWritesThreshold < opts.L0CompactionThreshold {
		opts.L0StopWritesThreshold = opts.L0CompactionThreshold
	}
	opts.LBaseMaxBytes = 1 << uint(rng.Intn(30)) // 1B - 1GB
	maxConcurrentCompactions := rng.Intn(3) + 1  // 1-3
	opts.MaxConcurrentCompactions = func() int {
		return maxConcurrentCompactions
	}
	opts.MaxManifestFileSize = 1 << uint(rng.Intn(30)) // 1B  - 1GB
	opts.MemTableSize = 2 << (10 + uint(rng.Intn(16))) // 2KB - 256MB
	opts.MemTableStopWritesThreshold = 2 + rng.Intn(5) // 2 - 5
	if rng.Intn(2) == 0 {
		opts.WALDir = "data/wal"
	}
	if rng.Intn(4) == 0 {
		// Enable Writer parallelism for 25% of the random options. Setting
		// MaxWriterConcurrency to any value greater than or equal to 1 has the
		// same effect currently.
		opts.Experimental.MaxWriterConcurrency = 2
		opts.Experimental.ForceWriterParallelism = true
	}
	if rng.Intn(2) == 0 {
		opts.Experimental.DisableIngestAsFlushable = func() bool { return true }
	}
	var lopts pebble.LevelOptions
	lopts.BlockRestartInterval = 1 + rng.Intn(64)  // 1 - 64
	lopts.BlockSize = 1 << uint(rng.Intn(24))      // 1 - 16MB
	lopts.BlockSizeThreshold = 50 + rng.Intn(50)   // 50 - 100
	lopts.IndexBlockSize = 1 << uint(rng.Intn(24)) // 1 - 16MB
	lopts.TargetFileSize = 1 << uint(rng.Intn(28)) // 1 - 256MB

	// We either use no bloom filter, the default filter, or a filter with
	// randomized bits-per-key setting. We zero out the Filters map. It'll get
	// repopulated on EnsureDefaults accordingly.
	opts.Filters = nil
	switch rng.Intn(3) {
	case 0:
		lopts.FilterPolicy = nil
	case 1:
		lopts.FilterPolicy = bloom.FilterPolicy(10)
	default:
		lopts.FilterPolicy = newTestingFilterPolicy(1 << rng.Intn(5))
	}
	opts.Levels = []pebble.LevelOptions{lopts}

	// Explicitly disable disk-backed FS's for the random configurations. The
	// single standard test configuration that uses a disk-backed FS is
	// sufficient.
	testOpts.useDisk = false
	testOpts.strictFS = rng.Intn(2) != 0 // Only relevant for MemFS.
	testOpts.threads = rng.Intn(runtime.GOMAXPROCS(0)) + 1
	if testOpts.strictFS {
		opts.DisableWAL = false
	}
	testOpts.ingestUsingApply = rng.Intn(2) != 0
	testOpts.deleteSized = rng.Intn(2) != 0
	testOpts.replaceSingleDelete = rng.Intn(2) != 0
	testOpts.disableBlockPropertyCollector = rng.Intn(2) == 1
	if testOpts.disableBlockPropertyCollector {
		testOpts.Opts.BlockPropertyCollectors = nil
	}
	testOpts.enableValueBlocks = opts.FormatMajorVersion >= pebble.FormatSSTableValueBlocks &&
		rng.Intn(2) != 0
	if testOpts.enableValueBlocks {
		testOpts.Opts.Experimental.EnableValueBlocks = func() bool { return true }
	}
	testOpts.asyncApplyToDB = rng.Intn(2) != 0
	// 20% of time, enable shared storage.
	if rng.Intn(5) == 0 {
		testOpts.sharedStorageEnabled = true
		testOpts.Opts.Experimental.RemoteStorage = remote.MakeSimpleFactory(map[remote.Locator]remote.Storage{
			"": remote.NewInMem(),
		})
		// If shared storage is enabled, pick between writing all files on shared
		// vs. lower levels only, 50% of the time.
		testOpts.Opts.Experimental.CreateOnShared = remote.CreateOnSharedAll
		if rng.Intn(2) == 0 {
			testOpts.Opts.Experimental.CreateOnShared = remote.CreateOnSharedLower
		}
		// If shared storage is enabled, enable secondary cache 50% of time.
		if rng.Intn(2) == 0 {
			testOpts.secondaryCacheEnabled = true
			// TODO(josh): Randomize various secondary cache settings.
			testOpts.Opts.Experimental.SecondaryCacheSizeBytes = 1024 * 1024 * 32 // 32 MBs
		}
	}
	testOpts.seedEFOS = rng.Uint64()
	testOpts.ingestSplit = rng.Intn(2) == 0
	opts.Experimental.IngestSplit = func() bool { return testOpts.ingestSplit }
	testOpts.Opts.EnsureDefaults()
	return testOpts
}

func setupInitialState(dataDir string, testOpts *TestOptions) error {
	// Copy (vfs.Default,<initialStatePath>/data) to (testOpts.opts.FS,<dataDir>).
	ok, err := vfs.Clone(
		vfs.Default,
		testOpts.Opts.FS,
		vfs.Default.PathJoin(testOpts.initialStatePath, "data"),
		dataDir,
		vfs.CloneSync,
		vfs.CloneSkip(func(filename string) bool {
			// Skip the archive of historical files, any checkpoints created by
			// operations and files staged for ingest in tmp.
			b := filepath.Base(filename)
			return b == "archive" || b == "checkpoints" || b == "tmp"
		}))
	if err != nil {
		return err
	} else if !ok {
		return os.ErrNotExist
	}

	// Tests with wal_dir set store their WALs in a `wal` directory. The source
	// database (initialStatePath) could've had wal_dir set, or the current test
	// options (testOpts) could have wal_dir set, or both.
	fs := testOpts.Opts.FS
	walDir := fs.PathJoin(dataDir, "wal")
	if err := fs.MkdirAll(walDir, os.ModePerm); err != nil {
		return err
	}

	// Copy <dataDir>/wal/*.log -> <dataDir>.
	src, dst := walDir, dataDir
	if testOpts.Opts.WALDir != "" {
		// Copy <dataDir>/*.log -> <dataDir>/wal.
		src, dst = dst, src
	}
	return moveLogs(fs, src, dst)
}

func moveLogs(fs vfs.FS, srcDir, dstDir string) error {
	ls, err := fs.List(srcDir)
	if err != nil {
		return err
	}
	for _, f := range ls {
		if filepath.Ext(f) != ".log" {
			continue
		}
		src := fs.PathJoin(srcDir, f)
		dst := fs.PathJoin(dstDir, f)
		if err := fs.Rename(src, dst); err != nil {
			return err
		}
	}
	return nil
}

var blockPropertyCollectorConstructors = []func() pebble.BlockPropertyCollector{
	sstable.NewTestKeysBlockPropertyCollector,
}

// testingFilterPolicy is used to allow bloom filter policies with non-default
// bits-per-key setting. It is necessary because the name of the production
// filter policy is fixed (see bloom.FilterPolicy.Name()); we need to output a
// custom policy name to the OPTIONS file that the test can then parse.
type testingFilterPolicy struct {
	bloom.FilterPolicy
}

var _ pebble.FilterPolicy = (*testingFilterPolicy)(nil)

func newTestingFilterPolicy(bitsPerKey int) *testingFilterPolicy {
	return &testingFilterPolicy{
		FilterPolicy: bloom.FilterPolicy(bitsPerKey),
	}
}

const testingFilterPolicyFmt = "testing_bloom_filter/bits_per_key=%d"

// Name implements the pebble.FilterPolicy interface.
func (t *testingFilterPolicy) Name() string {
	if t.FilterPolicy == 10 {
		return "rocksdb.BuiltinBloomFilter"
	}
	return fmt.Sprintf(testingFilterPolicyFmt, t.FilterPolicy)
}

func filterPolicyFromName(name string) (pebble.FilterPolicy, error) {
	switch name {
	case "none":
		return nil, nil
	case "rocksdb.BuiltinBloomFilter":
		return bloom.FilterPolicy(10), nil
	}
	var bitsPerKey int
	if _, err := fmt.Sscanf(name, testingFilterPolicyFmt, &bitsPerKey); err != nil {
		return nil, errors.Errorf("Invalid filter policy name '%s'", name)
	}
	return newTestingFilterPolicy(bitsPerKey), nil
}
