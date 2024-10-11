// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"strconv"
	"sync"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/atomicfs"
	"github.com/stretchr/testify/require"
)

func TestFormatMajorVersion_MigrationDefined(t *testing.T) {
	for v := FormatMostCompatible; v <= FormatNewest; v++ {
		if _, ok := formatMajorVersionMigrations[v]; !ok {
			t.Errorf("format major version %d has no migration defined", v)
		}
	}
}

func TestRatchetFormat(t *testing.T) {
	fs := vfs.NewMem()
	d, err := Open("", (&Options{FS: fs}).WithFSDefaults())
	require.NoError(t, err)
	require.NoError(t, d.Set([]byte("foo"), []byte("bar"), Sync))
	require.Equal(t, FormatMostCompatible, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatVersioned))
	require.Equal(t, FormatVersioned, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatVersioned))
	require.Equal(t, FormatVersioned, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatSetWithDelete))
	require.Equal(t, FormatSetWithDelete, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatBlockPropertyCollector))
	require.Equal(t, FormatBlockPropertyCollector, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatSplitUserKeysMarked))
	require.Equal(t, FormatSplitUserKeysMarked, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatSplitUserKeysMarkedCompacted))
	require.Equal(t, FormatSplitUserKeysMarkedCompacted, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatRangeKeys))
	require.Equal(t, FormatRangeKeys, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatMinTableFormatPebblev1))
	require.Equal(t, FormatMinTableFormatPebblev1, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatPrePebblev1Marked))
	require.Equal(t, FormatPrePebblev1Marked, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(formatUnusedPrePebblev1MarkedCompacted))
	require.Equal(t, formatUnusedPrePebblev1MarkedCompacted, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatSSTableValueBlocks))
	require.Equal(t, FormatSSTableValueBlocks, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatFlushableIngest))
	require.Equal(t, FormatFlushableIngest, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatPrePebblev1MarkedCompacted))
	require.Equal(t, FormatPrePebblev1MarkedCompacted, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatDeleteSizedAndObsolete))
	require.Equal(t, FormatDeleteSizedAndObsolete, d.FormatMajorVersion())
	require.NoError(t, d.RatchetFormatMajorVersion(FormatVirtualSSTables))
	require.Equal(t, FormatVirtualSSTables, d.FormatMajorVersion())

	require.NoError(t, d.Close())

	// If we Open the database again, leaving the default format, the
	// database should Open using the persisted FormatNewest.
	d, err = Open("", (&Options{FS: fs}).WithFSDefaults())
	require.NoError(t, err)
	require.Equal(t, internalFormatNewest, d.FormatMajorVersion())
	require.NoError(t, d.Close())

	// Move the marker to a version that does not exist.
	m, _, err := atomicfs.LocateMarker(fs, "", formatVersionMarkerName)
	require.NoError(t, err)
	require.NoError(t, m.Move("999999"))
	require.NoError(t, m.Close())

	_, err = Open("", (&Options{
		FS:                 fs,
		FormatMajorVersion: FormatVersioned,
	}).WithFSDefaults())
	require.Error(t, err)
	require.EqualError(t, err, `pebble: database "" written in format major version 999999`)
}

func testBasicDB(d *DB) error {
	key := []byte("a")
	value := []byte("b")
	if err := d.Set(key, value, nil); err != nil {
		return err
	}
	if err := d.Flush(); err != nil {
		return err
	}
	if err := d.Compact(nil, []byte("\xff"), false); err != nil {
		return err
	}

	iter, _ := d.NewIter(nil)
	for valid := iter.First(); valid; valid = iter.Next() {
	}
	if err := iter.Close(); err != nil {
		return err
	}
	return nil
}

func TestFormatMajorVersions(t *testing.T) {
	for vers := FormatMostCompatible; vers <= FormatNewest; vers++ {
		t.Run(fmt.Sprintf("vers=%03d", vers), func(t *testing.T) {
			fs := vfs.NewStrictMem()
			opts := (&Options{
				FS:                 fs,
				FormatMajorVersion: vers,
			}).WithFSDefaults()

			// Create a database at this format major version and perform
			// some very basic operations.
			d, err := Open("", opts)
			require.NoError(t, err)
			require.NoError(t, testBasicDB(d))
			require.NoError(t, d.Close())

			// Re-open the database at this format major version, and again
			// perform some basic operations.
			d, err = Open("", opts)
			require.NoError(t, err)
			require.NoError(t, testBasicDB(d))
			require.NoError(t, d.Close())

			t.Run("upgrade-at-open", func(t *testing.T) {
				for upgradeVers := vers + 1; upgradeVers <= FormatNewest; upgradeVers++ {
					t.Run(fmt.Sprintf("upgrade-vers=%03d", upgradeVers), func(t *testing.T) {
						// We use vfs.MemFS's option to ignore syncs so
						// that we can perform an upgrade on the current
						// database state in fs, and revert it when this
						// subtest is complete.
						fs.SetIgnoreSyncs(true)
						defer fs.ResetToSyncedState()

						// Re-open the database, passing a higher format
						// major version in the Options to automatically
						// ratchet the format major version. Ensure some
						// basic operations pass.
						opts := opts.Clone()
						opts.FormatMajorVersion = upgradeVers
						d, err = Open("", opts)
						require.NoError(t, err)
						require.Equal(t, upgradeVers, d.FormatMajorVersion())
						require.NoError(t, testBasicDB(d))
						require.NoError(t, d.Close())

						// Re-open to ensure the upgrade persisted.
						d, err = Open("", opts)
						require.NoError(t, err)
						require.Equal(t, upgradeVers, d.FormatMajorVersion())
						require.NoError(t, testBasicDB(d))
						require.NoError(t, d.Close())
					})
				}
			})

			t.Run("upgrade-while-open", func(t *testing.T) {
				for upgradeVers := vers + 1; upgradeVers <= FormatNewest; upgradeVers++ {
					t.Run(fmt.Sprintf("upgrade-vers=%03d", upgradeVers), func(t *testing.T) {
						// Ensure the previous tests don't overwrite our
						// options.
						require.Equal(t, vers, opts.FormatMajorVersion)

						// We use vfs.MemFS's option to ignore syncs so
						// that we can perform an upgrade on the current
						// database state in fs, and revert it when this
						// subtest is complete.
						fs.SetIgnoreSyncs(true)
						defer fs.ResetToSyncedState()

						// Re-open the database, still at the current format
						// major version. Perform some basic operations,
						// ratchet the format version up, and perform
						// additional basic operations.
						d, err = Open("", opts)
						require.NoError(t, err)
						require.NoError(t, testBasicDB(d))
						require.Equal(t, vers, d.FormatMajorVersion())
						require.NoError(t, d.RatchetFormatMajorVersion(upgradeVers))
						require.Equal(t, upgradeVers, d.FormatMajorVersion())
						require.NoError(t, testBasicDB(d))
						require.NoError(t, d.Close())

						// Re-open to ensure the upgrade persisted.
						d, err = Open("", opts)
						require.NoError(t, err)
						require.Equal(t, upgradeVers, d.FormatMajorVersion())
						require.NoError(t, testBasicDB(d))
						require.NoError(t, d.Close())
					})
				}
			})
		})
	}
}

func TestFormatMajorVersions_TableFormat(t *testing.T) {
	// NB: This test is intended to validate the mapping between every
	// FormatMajorVersion and sstable.TableFormat exhaustively. This serves as a
	// sanity check that new versions have a corresponding mapping. The test
	// fixture is intentionally verbose.

	m := map[FormatMajorVersion][2]sstable.TableFormat{
		FormatDefault:                          {sstable.TableFormatLevelDB, sstable.TableFormatRocksDBv2},
		FormatMostCompatible:                   {sstable.TableFormatLevelDB, sstable.TableFormatRocksDBv2},
		formatVersionedManifestMarker:          {sstable.TableFormatLevelDB, sstable.TableFormatRocksDBv2},
		FormatVersioned:                        {sstable.TableFormatLevelDB, sstable.TableFormatRocksDBv2},
		FormatSetWithDelete:                    {sstable.TableFormatLevelDB, sstable.TableFormatRocksDBv2},
		FormatBlockPropertyCollector:           {sstable.TableFormatLevelDB, sstable.TableFormatPebblev1},
		FormatSplitUserKeysMarked:              {sstable.TableFormatLevelDB, sstable.TableFormatPebblev1},
		FormatSplitUserKeysMarkedCompacted:     {sstable.TableFormatLevelDB, sstable.TableFormatPebblev1},
		FormatRangeKeys:                        {sstable.TableFormatLevelDB, sstable.TableFormatPebblev2},
		FormatMinTableFormatPebblev1:           {sstable.TableFormatPebblev1, sstable.TableFormatPebblev2},
		FormatPrePebblev1Marked:                {sstable.TableFormatPebblev1, sstable.TableFormatPebblev2},
		formatUnusedPrePebblev1MarkedCompacted: {sstable.TableFormatPebblev1, sstable.TableFormatPebblev2},
		FormatSSTableValueBlocks:               {sstable.TableFormatPebblev1, sstable.TableFormatPebblev3},
		FormatFlushableIngest:                  {sstable.TableFormatPebblev1, sstable.TableFormatPebblev3},
		FormatPrePebblev1MarkedCompacted:       {sstable.TableFormatPebblev1, sstable.TableFormatPebblev3},
		FormatDeleteSizedAndObsolete:           {sstable.TableFormatPebblev1, sstable.TableFormatPebblev4},
		FormatVirtualSSTables:                  {sstable.TableFormatPebblev1, sstable.TableFormatPebblev4},
	}

	// Valid versions.
	for fmv := FormatMostCompatible; fmv <= internalFormatNewest; fmv++ {
		got := [2]sstable.TableFormat{fmv.MinTableFormat(), fmv.MaxTableFormat()}
		require.Equalf(t, m[fmv], got, "got %s; want %s", got, m[fmv])
		require.True(t, got[0] <= got[1] /* min <= max */)
	}

	// Invalid versions.
	fmv := internalFormatNewest + 1
	require.Panics(t, func() { _ = fmv.MaxTableFormat() })
	require.Panics(t, func() { _ = fmv.MinTableFormat() })
}

func TestSplitUserKeyMigration(t *testing.T) {
	var d *DB
	var opts *Options
	var fs vfs.FS
	var buf bytes.Buffer
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()

	datadriven.RunTest(t, "testdata/format_major_version_split_user_key_migration",
		func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "define":
				if d != nil {
					if err := d.Close(); err != nil {
						return err.Error()
					}
					buf.Reset()
				}
				opts = (&Options{
					FormatMajorVersion: FormatBlockPropertyCollector,
					EventListener: &EventListener{
						CompactionEnd: func(info CompactionInfo) {
							// Fix the job ID and durations for determinism.
							info.JobID = 100
							info.Duration = time.Second
							info.TotalDuration = 2 * time.Second
							fmt.Fprintln(&buf, info)
						},
					},
					DisableAutomaticCompactions: true,
				}).WithFSDefaults()
				var err error
				if d, err = runDBDefineCmd(td, opts); err != nil {
					return err.Error()
				}

				fs = d.opts.FS
				d.mu.Lock()
				defer d.mu.Unlock()
				return d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
			case "reopen":
				if d != nil {
					if err := d.Close(); err != nil {
						return err.Error()
					}
					buf.Reset()
				}
				opts.FS = fs
				opts.DisableAutomaticCompactions = true
				var err error
				d, err = Open("", opts)
				if err != nil {
					return err.Error()
				}
				return "OK"
			case "build":
				if err := runBuildCmd(td, d, fs); err != nil {
					return err.Error()
				}
				return ""
			case "force-ingest":
				if err := runForceIngestCmd(td, d); err != nil {
					return err.Error()
				}
				d.mu.Lock()
				defer d.mu.Unlock()
				return d.mu.versions.currentVersion().DebugString(base.DefaultFormatter)
			case "format-major-version":
				return d.FormatMajorVersion().String()
			case "ratchet-format-major-version":
				v, err := strconv.Atoi(td.CmdArgs[0].String())
				if err != nil {
					return err.Error()
				}
				if err := d.RatchetFormatMajorVersion(FormatMajorVersion(v)); err != nil {
					return err.Error()
				}
				return buf.String()
			case "lsm":
				return runLSMCmd(td, d)
			case "marked-file-count":
				m := d.Metrics()
				return fmt.Sprintf("%d files marked for compaction", m.Compact.MarkedFiles)
			case "disable-automatic-compactions":
				d.mu.Lock()
				defer d.mu.Unlock()
				switch v := td.CmdArgs[0].String(); v {
				case "true":
					d.opts.DisableAutomaticCompactions = true
				case "false":
					d.opts.DisableAutomaticCompactions = false
				default:
					return fmt.Sprintf("unknown value %q", v)
				}
				return ""
			default:
				return fmt.Sprintf("unrecognized command %q", td.Cmd)
			}
		})
}

func TestPebblev1Migration(t *testing.T) {
	var d *DB
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()

	datadriven.RunTest(t, "testdata/format_major_version_pebblev1_migration",
		func(t *testing.T, td *datadriven.TestData) string {
			switch cmd := td.Cmd; cmd {
			case "open":
				var version int
				var err error
				for _, cmdArg := range td.CmdArgs {
					switch cmd := cmdArg.Key; cmd {
					case "version":
						version, err = strconv.Atoi(cmdArg.Vals[0])
						if err != nil {
							return err.Error()
						}
					default:
						return fmt.Sprintf("unknown argument: %s", cmd)
					}
				}
				opts := (&Options{
					FS:                 vfs.NewMem(),
					FormatMajorVersion: FormatMajorVersion(version),
				}).WithFSDefaults()
				d, err = Open("", opts)
				if err != nil {
					return err.Error()
				}
				return ""

			case "format-major-version":
				return d.FormatMajorVersion().String()

			case "min-table-format":
				return d.FormatMajorVersion().MinTableFormat().String()

			case "max-table-format":
				return d.FormatMajorVersion().MaxTableFormat().String()

			case "disable-automatic-compactions":
				d.mu.Lock()
				defer d.mu.Unlock()
				switch v := td.CmdArgs[0].String(); v {
				case "true":
					d.opts.DisableAutomaticCompactions = true
				case "false":
					d.opts.DisableAutomaticCompactions = false
				default:
					return fmt.Sprintf("unknown value %q", v)
				}
				return ""

			case "batch":
				b := d.NewIndexedBatch()
				if err := runBatchDefineCmd(td, b); err != nil {
					return err.Error()
				}
				if err := b.Commit(nil); err != nil {
					return err.Error()
				}
				return ""

			case "flush":
				if err := d.Flush(); err != nil {
					return err.Error()
				}
				return ""

			case "ingest":
				if err := runBuildCmd(td, d, d.opts.FS); err != nil {
					return err.Error()
				}
				// Only the first arg is a filename.
				td.CmdArgs = td.CmdArgs[:1]
				if err := runIngestCmd(td, d, d.opts.FS); err != nil {
					return err.Error()
				}
				return ""

			case "lsm":
				return runLSMCmd(td, d)

			case "tally-table-formats":
				d.mu.Lock()
				defer d.mu.Unlock()
				v := d.mu.versions.currentVersion()
				tally := make([]int, sstable.TableFormatMax+1)
				for _, l := range v.Levels {
					iter := l.Iter()
					for m := iter.First(); m != nil; m = iter.Next() {
						err := d.tableCache.withReader(m.PhysicalMeta(),
							func(r *sstable.Reader) error {
								f, err := r.TableFormat()
								if err != nil {
									return err
								}
								tally[f]++
								return nil
							})
						if err != nil {
							return err.Error()
						}
					}
				}
				var b bytes.Buffer
				for i := 1; i <= int(sstable.TableFormatMax); i++ {
					_, _ = fmt.Fprintf(&b, "%s: %d\n", sstable.TableFormat(i), tally[i])
				}
				return b.String()

			case "ratchet-format-major-version":
				v, err := strconv.Atoi(td.CmdArgs[0].String())
				if err != nil {
					return err.Error()
				}
				if err = d.RatchetFormatMajorVersion(FormatMajorVersion(v)); err != nil {
					return err.Error()
				}
				return ""

			case "marked-file-count":
				m := d.Metrics()
				return fmt.Sprintf("%d files marked for compaction", m.Compact.MarkedFiles)

			default:
				return fmt.Sprintf("unknown command: %s", cmd)
			}
		},
	)
}

// TestPebblev1MigrationRace exercises the race between a PrePebbleV1Marked
// format major version upgrade that needs to open sstables to read their table
// format, and concurrent compactions that may delete the same files from the
// LSM.
//
// Regression test for #2019.
func TestPebblev1MigrationRace(t *testing.T) {
	// Use a smaller table cache size to slow down the PrePebbleV1Marked
	// migration, ensuring each table read needs to re-open the file.
	cache := NewCache(4 << 20)
	defer cache.Unref()
	tableCache := NewTableCache(cache, 1, 5)
	defer tableCache.Unref()
	d, err := Open("", (&Options{
		Cache:              cache,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: FormatMajorVersion(FormatPrePebblev1Marked - 1),
		TableCache:         tableCache,
		Levels:             []LevelOptions{{TargetFileSize: 1}},
	}).WithFSDefaults())
	require.NoError(t, err)
	defer d.Close()

	ks := testkeys.Alpha(3).EveryN(10)
	var key [3]byte
	for i := int64(0); i < ks.Count(); i++ {
		n := testkeys.WriteKey(key[:], ks, i)
		require.NoError(t, d.Set(key[:n], key[:n], nil))
		require.NoError(t, d.Flush())
	}

	// Asynchronously write and flush range deletes that will cause compactions
	// to delete the existing sstables. These deletes will race with the format
	// major version upgrade's migration will attempt to delete the files.
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		for i := ks.Count() - 1; i > 0; i -= 50 {
			endKey := testkeys.Key(ks, i)
			startIndex := i - 50
			if startIndex < 0 {
				startIndex = 0
			}
			startKey := testkeys.Key(ks, startIndex)

			require.NoError(t, d.DeleteRange(startKey, endKey, nil))
			_, err := d.AsyncFlush()
			require.NoError(t, err)
		}
	}()
	require.NoError(t, d.RatchetFormatMajorVersion(FormatPrePebblev1Marked))
	wg.Wait()
}

// Regression test for #2044, where multiple concurrent compactions can lead
// to an indefinite wait on the compaction goroutine in compactMarkedFilesLocked.
func TestPebblev1MigrationConcurrencyRace(t *testing.T) {
	opts := (&Options{
		Comparer:           testkeys.Comparer,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: FormatSplitUserKeysMarked,
		Levels:             []LevelOptions{{FilterPolicy: bloom.FilterPolicy(10)}},
		MaxConcurrentCompactions: func() int {
			return 4
		},
	}).WithFSDefaults()
	func() {
		d, err := Open("", opts)
		require.NoError(t, err)
		defer func() {
			require.NoError(t, d.Close())
		}()

		ks := testkeys.Alpha(3).EveryN(10)
		var key [3]byte
		for i := int64(0); i < ks.Count(); i++ {
			n := testkeys.WriteKey(key[:], ks, i)
			require.NoError(t, d.Set(key[:n], key[:n], nil))
			if i%100 == 0 {
				require.NoError(t, d.Flush())
			}
		}
		require.NoError(t, d.Flush())
	}()

	opts.FormatMajorVersion = formatUnusedPrePebblev1MarkedCompacted
	d, err := Open("", opts)
	require.NoError(t, err)
	require.NoError(t, d.RatchetFormatMajorVersion(formatUnusedPrePebblev1MarkedCompacted))
	require.NoError(t, d.Close())
}
