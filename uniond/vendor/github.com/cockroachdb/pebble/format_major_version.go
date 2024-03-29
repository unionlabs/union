// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"strconv"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/atomicfs"
)

// FormatMajorVersion is a constant controlling the format of persisted
// data. Backwards incompatible changes to durable formats are gated
// behind new format major versions.
//
// At any point, a database's format major version may be bumped.
// However, once a database's format major version is increased,
// previous versions of Pebble will refuse to open the database.
//
// The zero value format is the FormatDefault constant. The exact
// FormatVersion that the default corresponds to may change with time.
type FormatMajorVersion uint64

// SafeValue implements redact.SafeValue.
func (v FormatMajorVersion) SafeValue() {}

// String implements fmt.Stringer.
func (v FormatMajorVersion) String() string {
	// NB: This must not change. It's used as the value for the on-disk
	// version marker file.
	//
	// Specifically, this value must always parse as a base 10 integer
	// that fits in a uint64. We format it as zero-padded, 3-digit
	// number today, but the padding may change.
	return fmt.Sprintf("%03d", v)
}

const (
	// 21.2 versions.

	// FormatDefault leaves the format version unspecified. The
	// FormatDefault constant may be ratcheted upwards over time.
	FormatDefault FormatMajorVersion = iota
	// FormatMostCompatible maintains the most backwards compatibility,
	// maintaining bi-directional compatibility with RocksDB 6.2.1 in
	// the particular configuration described in the Pebble README.
	FormatMostCompatible
	// formatVersionedManifestMarker is the first
	// backwards-incompatible change made to Pebble, introducing the
	// format-version marker file for handling backwards-incompatible
	// changes more broadly, and replacing the `CURRENT` file with a
	// marker file.
	//
	// This format version is intended as an intermediary version state.
	// It is deliberately unexported to discourage direct use of this
	// format major version.  Clients should use FormatVersioned which
	// also ensures earlier versions of Pebble fail to open a database
	// written in a future format major version.
	formatVersionedManifestMarker
	// FormatVersioned is a new format major version that replaces the
	// old `CURRENT` file with a new 'marker' file scheme.  Previous
	// Pebble versions will be unable to open the database unless
	// they're aware of format versions.
	FormatVersioned
	// FormatSetWithDelete is a format major version that introduces a new key
	// kind, base.InternalKeyKindSetWithDelete. Previous Pebble versions will be
	// unable to open this database.
	FormatSetWithDelete

	// 22.1 versions.

	// FormatBlockPropertyCollector is a format major version that introduces
	// BlockPropertyCollectors.
	FormatBlockPropertyCollector
	// FormatSplitUserKeysMarked is a format major version that guarantees that
	// all files that share user keys with neighbors are marked for compaction
	// in the manifest. Ratcheting to FormatSplitUserKeysMarked will block
	// (without holding mutexes) until the scan of the LSM is complete and the
	// manifest has been rotated.
	FormatSplitUserKeysMarked

	// 22.2 versions.

	// FormatSplitUserKeysMarkedCompacted is a format major version that
	// guarantees that all files explicitly marked for compaction in the manifest
	// have been compacted. Combined with the FormatSplitUserKeysMarked format
	// major version, this version guarantees that there are no user keys split
	// across multiple files within a level L1+. Ratcheting to this format version
	// will block (without holding mutexes) until all necessary compactions for
	// files marked for compaction are complete.
	FormatSplitUserKeysMarkedCompacted
	// FormatRangeKeys is a format major version that introduces range keys.
	FormatRangeKeys
	// FormatMinTableFormatPebblev1 is a format major version that guarantees that
	// tables created by or ingested into the DB at or above this format major
	// version will have a table format version of at least Pebblev1 (Block
	// Properties).
	FormatMinTableFormatPebblev1
	// FormatPrePebblev1Marked is a format major version that guarantees that all
	// sstables with a table format version pre-Pebblev1 (i.e. those that are
	// guaranteed to not contain block properties) are marked for compaction in
	// the manifest. Ratcheting to FormatPrePebblev1Marked will block (without
	// holding mutexes) until the scan of the LSM is complete and the manifest has
	// been rotated.
	FormatPrePebblev1Marked

	// 23.1 versions.

	// formatUnusedPrePebblev1MarkedCompacted is an unused format major version.
	// This format major version was originally intended to ship in the 23.1
	// release. It was later decided that this should be deferred until a
	// subsequent release. The original ordering is preserved so as not to
	// introduce breaking changes in Cockroach.
	formatUnusedPrePebblev1MarkedCompacted

	// FormatSSTableValueBlocks is a format major version that adds support for
	// storing values in value blocks in the sstable. Value block support is not
	// necessarily enabled when writing sstables, when running with this format
	// major version.
	//
	// WARNING: In development, so no production code should upgrade to this
	// format, since a DB with this format major version will not actually
	// interoperate correctly with another DB with the same format major
	// version. This format major version is introduced so that tests can start
	// being executed up to this version. Note that these tests succeed despite
	// the incomplete support since they do not enable value blocks and use
	// TableFormatPebblev2.
	FormatSSTableValueBlocks

	// FormatFlushableIngest is a format major version that enables lazy
	// addition of ingested sstables into the LSM structure. When an ingest
	// overlaps with a memtable, a record of the ingest is written to the WAL
	// without waiting for a flush. Subsequent reads treat the ingested files as
	// a level above the overlapping memtable. Once the memtable is flushed, the
	// ingested files are moved into the lowest possible levels.
	//
	// This feature is behind a format major version because it required
	// breaking changes to the WAL format.
	FormatFlushableIngest

	// 23.2 versions.

	// FormatPrePebblev1MarkedCompacted is a format major version that guarantees
	// that all sstables explicitly marked for compaction in the manifest (see
	// FormatPrePebblev1Marked) have been compacted. Ratcheting to this format
	// version will block (without holding mutexes) until all necessary
	// compactions for files marked for compaction are complete.
	FormatPrePebblev1MarkedCompacted

	// FormatDeleteSizedAndObsolete is a format major version that adds support
	// for deletion tombstones that encode the size of the value they're
	// expected to delete. This format major version is required before the
	// associated key kind may be committed through batch applications or
	// ingests. It also adds support for keys that are marked obsolete (see
	// sstable/format.go for details).
	FormatDeleteSizedAndObsolete

	// FormatVirtualSSTables is a format major version that adds support for
	// virtual sstables that can reference a sub-range of keys in an underlying
	// physical sstable. This information is persisted through new,
	// backward-incompatible fields in the Manifest, and therefore requires
	// a format major version.
	FormatVirtualSSTables

	// internalFormatNewest holds the newest format major version, including
	// experimental ones excluded from the exported FormatNewest constant until
	// they've stabilized. Used in tests.
	internalFormatNewest FormatMajorVersion = iota - 1

	// FormatNewest always contains the most recent format major version.
	FormatNewest FormatMajorVersion = internalFormatNewest
)

// MaxTableFormat returns the maximum sstable.TableFormat that can be used at
// this FormatMajorVersion.
func (v FormatMajorVersion) MaxTableFormat() sstable.TableFormat {
	switch v {
	case FormatDefault, FormatMostCompatible, formatVersionedManifestMarker,
		FormatVersioned, FormatSetWithDelete:
		return sstable.TableFormatRocksDBv2
	case FormatBlockPropertyCollector, FormatSplitUserKeysMarked,
		FormatSplitUserKeysMarkedCompacted:
		return sstable.TableFormatPebblev1
	case FormatRangeKeys, FormatMinTableFormatPebblev1, FormatPrePebblev1Marked,
		formatUnusedPrePebblev1MarkedCompacted:
		return sstable.TableFormatPebblev2
	case FormatSSTableValueBlocks, FormatFlushableIngest, FormatPrePebblev1MarkedCompacted:
		return sstable.TableFormatPebblev3
	case FormatDeleteSizedAndObsolete, FormatVirtualSSTables:
		return sstable.TableFormatPebblev4
	default:
		panic(fmt.Sprintf("pebble: unsupported format major version: %s", v))
	}
}

// MinTableFormat returns the minimum sstable.TableFormat that can be used at
// this FormatMajorVersion.
func (v FormatMajorVersion) MinTableFormat() sstable.TableFormat {
	switch v {
	case FormatDefault, FormatMostCompatible, formatVersionedManifestMarker,
		FormatVersioned, FormatSetWithDelete, FormatBlockPropertyCollector,
		FormatSplitUserKeysMarked, FormatSplitUserKeysMarkedCompacted,
		FormatRangeKeys:
		return sstable.TableFormatLevelDB
	case FormatMinTableFormatPebblev1, FormatPrePebblev1Marked,
		formatUnusedPrePebblev1MarkedCompacted, FormatSSTableValueBlocks,
		FormatFlushableIngest, FormatPrePebblev1MarkedCompacted,
		FormatDeleteSizedAndObsolete, FormatVirtualSSTables:
		return sstable.TableFormatPebblev1
	default:
		panic(fmt.Sprintf("pebble: unsupported format major version: %s", v))
	}
}

// orderingInvariants returns an enum encoding the set of invariants that must
// hold within the receiver format major version. Invariants only get stricter
// as the format major version advances, so it is okay to retrieve the
// invariants from the current format major version and by the time the
// invariants are enforced, the format major version has advanced.
func (v FormatMajorVersion) orderingInvariants() manifest.OrderingInvariants {
	if v < FormatSplitUserKeysMarkedCompacted {
		return manifest.AllowSplitUserKeys
	}
	return manifest.ProhibitSplitUserKeys
}

// formatMajorVersionMigrations defines the migrations from one format
// major version to the next. Each migration is defined as a closure
// which will be invoked on the database before the new format major
// version is committed. Migrations must be idempotent. Migrations are
// invoked with d.mu locked.
//
// Each migration is responsible for invoking finalizeFormatVersUpgrade
// to set the new format major version.  RatchetFormatMajorVersion will
// panic if a migration returns a nil error but fails to finalize the
// new format major version.
var formatMajorVersionMigrations = map[FormatMajorVersion]func(*DB) error{
	FormatMostCompatible: func(d *DB) error { return nil },
	formatVersionedManifestMarker: func(d *DB) error {
		// formatVersionedManifestMarker introduces the use of a marker
		// file for pointing to the current MANIFEST file.

		// Lock the manifest.
		d.mu.versions.logLock()
		defer d.mu.versions.logUnlock()

		// Construct the filename of the currently active manifest and
		// move the manifest marker to that filename. The marker is
		// guaranteed to exist, because we unconditionally locate it
		// during Open.
		manifestFileNum := d.mu.versions.manifestFileNum
		filename := base.MakeFilename(fileTypeManifest, manifestFileNum.DiskFileNum())
		if err := d.mu.versions.manifestMarker.Move(filename); err != nil {
			return errors.Wrap(err, "moving manifest marker")
		}

		// Now that we have a manifest marker file in place and pointing
		// to the current MANIFEST, finalize the upgrade. If we fail for
		// some reason, a retry of this migration is guaranteed to again
		// move the manifest marker file to the latest manifest. If
		// we're unable to finalize the upgrade, a subsequent call to
		// Open will ignore the manifest marker.
		if err := d.finalizeFormatVersUpgrade(formatVersionedManifestMarker); err != nil {
			return err
		}

		// We've finalized the upgrade. All subsequent Open calls will
		// ignore the CURRENT file and instead read the manifest marker.
		// Before we unlock the manifest, we need to update versionSet
		// to use the manifest marker on future rotations.
		d.mu.versions.setCurrent = setCurrentFuncMarker(
			d.mu.versions.manifestMarker,
			d.mu.versions.fs,
			d.mu.versions.dirname)
		return nil
	},
	// The FormatVersioned version is split into two, each with their
	// own migration to ensure the post-migration cleanup happens even
	// if there's a crash immediately after finalizing the version. Once
	// a new format major version is finalized, its migration will never
	// run again. Post-migration cleanup like the one in the migration
	// below must be performed in a separate migration or every time the
	// database opens.
	FormatVersioned: func(d *DB) error {
		// Replace the `CURRENT` file with one that points to the
		// nonexistent `MANIFEST-000000` file. If an earlier Pebble
		// version that does not know about format major versions
		// attempts to open the database, it will error avoiding
		// accidental corruption.
		if err := setCurrentFile(d.mu.versions.dirname, d.mu.versions.fs, base.FileNum(0).DiskFileNum()); err != nil {
			return err
		}
		return d.finalizeFormatVersUpgrade(FormatVersioned)
	},
	// As SetWithDelete is a new key kind, there is nothing to migrate. We can
	// simply finalize the format version and we're done.
	FormatSetWithDelete: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatSetWithDelete)
	},
	FormatBlockPropertyCollector: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatBlockPropertyCollector)
	},
	FormatSplitUserKeysMarked: func(d *DB) error {
		// Mark any unmarked files with split-user keys. Note all format major
		// versions migrations are invoked with DB.mu locked.
		if err := d.markFilesLocked(markFilesWithSplitUserKeys(d.opts.Comparer.Equal)); err != nil {
			return err
		}
		return d.finalizeFormatVersUpgrade(FormatSplitUserKeysMarked)
	},
	FormatSplitUserKeysMarkedCompacted: func(d *DB) error {
		// Before finalizing the format major version, rewrite any sstables
		// still marked for compaction. Note all format major versions
		// migrations are invoked with DB.mu locked.
		if err := d.compactMarkedFilesLocked(); err != nil {
			return err
		}
		return d.finalizeFormatVersUpgrade(FormatSplitUserKeysMarkedCompacted)
	},
	FormatRangeKeys: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatRangeKeys)
	},
	FormatMinTableFormatPebblev1: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatMinTableFormatPebblev1)
	},
	FormatPrePebblev1Marked: func(d *DB) error {
		// Mark any unmarked files that contain only table properties. Note all
		// format major versions migrations are invoked with DB.mu locked.
		if err := d.markFilesLocked(markFilesPrePebblev1(d.tableCache)); err != nil {
			return err
		}
		return d.finalizeFormatVersUpgrade(FormatPrePebblev1Marked)
	},
	formatUnusedPrePebblev1MarkedCompacted: func(d *DB) error {
		// Intentional no-op.
		return d.finalizeFormatVersUpgrade(formatUnusedPrePebblev1MarkedCompacted)
	},
	FormatSSTableValueBlocks: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatSSTableValueBlocks)
	},
	FormatFlushableIngest: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatFlushableIngest)
	},
	FormatPrePebblev1MarkedCompacted: func(d *DB) error {
		// Before finalizing the format major version, rewrite any sstables
		// still marked for compaction. Note all format major versions
		// migrations are invoked with DB.mu locked.
		if err := d.compactMarkedFilesLocked(); err != nil {
			return err
		}
		return d.finalizeFormatVersUpgrade(FormatPrePebblev1MarkedCompacted)
	},
	FormatDeleteSizedAndObsolete: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatDeleteSizedAndObsolete)
	},
	FormatVirtualSSTables: func(d *DB) error {
		return d.finalizeFormatVersUpgrade(FormatVirtualSSTables)
	},
}

const formatVersionMarkerName = `format-version`

func lookupFormatMajorVersion(
	fs vfs.FS, dirname string,
) (FormatMajorVersion, *atomicfs.Marker, error) {
	m, versString, err := atomicfs.LocateMarker(fs, dirname, formatVersionMarkerName)
	if err != nil {
		return 0, nil, err
	}
	if versString == "" {
		return FormatMostCompatible, m, nil
	}
	v, err := strconv.ParseUint(versString, 10, 64)
	if err != nil {
		return 0, nil, errors.Wrap(err, "parsing format major version")
	}
	vers := FormatMajorVersion(v)
	if vers == FormatDefault {
		return 0, nil, errors.Newf("pebble: default format major version should not persisted", vers)
	}
	if vers > internalFormatNewest {
		return 0, nil, errors.Newf("pebble: database %q written in format major version %d", dirname, vers)
	}
	return vers, m, nil
}

// FormatMajorVersion returns the database's active format major
// version. The format major version may be higher than the one
// provided in Options when the database was opened if the existing
// database was written with a higher format version.
func (d *DB) FormatMajorVersion() FormatMajorVersion {
	return FormatMajorVersion(d.mu.formatVers.vers.Load())
}

// RatchetFormatMajorVersion ratchets the opened database's format major
// version to the provided version. It errors if the provided format
// major version is below the database's current version. Once a
// database's format major version is upgraded, previous Pebble versions
// that do not know of the format version will be unable to open the
// database.
func (d *DB) RatchetFormatMajorVersion(fmv FormatMajorVersion) error {
	if err := d.closed.Load(); err != nil {
		panic(err)
	}

	d.mu.Lock()
	defer d.mu.Unlock()
	return d.ratchetFormatMajorVersionLocked(fmv)
}

func (d *DB) ratchetFormatMajorVersionLocked(formatVers FormatMajorVersion) error {
	if d.opts.ReadOnly {
		return ErrReadOnly
	}
	if formatVers > internalFormatNewest {
		// Guard against accidentally forgetting to update internalFormatNewest.
		return errors.Errorf("pebble: unknown format version %d", formatVers)
	}
	if currentVers := d.FormatMajorVersion(); currentVers > formatVers {
		return errors.Newf("pebble: database already at format major version %d; cannot reduce to %d",
			currentVers, formatVers)
	}
	if d.mu.formatVers.ratcheting {
		return errors.Newf("pebble: database format major version upgrade is in-progress")
	}
	d.mu.formatVers.ratcheting = true
	defer func() { d.mu.formatVers.ratcheting = false }()

	for nextVers := d.FormatMajorVersion() + 1; nextVers <= formatVers; nextVers++ {
		if err := formatMajorVersionMigrations[nextVers](d); err != nil {
			return errors.Wrapf(err, "migrating to version %d", nextVers)
		}

		// NB: The migration is responsible for calling
		// finalizeFormatVersUpgrade to finalize the upgrade. This
		// structure is necessary because some migrations may need to
		// update in-memory state (without ever dropping locks) after
		// the upgrade is finalized. Here we assert that the upgrade
		// did occur.
		if d.FormatMajorVersion() != nextVers {
			d.opts.Logger.Fatalf("pebble: successful migration to format version %d never finalized the upgrade", nextVers)
		}
	}
	return nil
}

// finalizeFormatVersUpgrade is typically only be called from within a
// format major version migration.
//
// See formatMajorVersionMigrations.
func (d *DB) finalizeFormatVersUpgrade(formatVers FormatMajorVersion) error {
	// We use the marker to encode the active format version in the
	// marker filename. Unlike other uses of the atomic marker, there is
	// no file with the filename `formatVers.String()` on the
	// filesystem.
	if err := d.mu.formatVers.marker.Move(formatVers.String()); err != nil {
		return err
	}
	d.mu.formatVers.vers.Store(uint64(formatVers))
	d.opts.EventListener.FormatUpgrade(formatVers)
	return nil
}

// compactMarkedFilesLocked performs a migration that schedules rewrite
// compactions to compact away any sstables marked for compaction.
// compactMarkedFilesLocked is run while ratcheting the database's format major
// version to FormatSplitUserKeysMarkedCompacted.
//
// Note that while this method is called with the DB.mu held, and will not
// return until all marked files have been compacted, the mutex is dropped while
// waiting for compactions to complete (or for slots to free up).
func (d *DB) compactMarkedFilesLocked() error {
	curr := d.mu.versions.currentVersion()
	for curr.Stats.MarkedForCompaction > 0 {
		// Attempt to schedule a compaction to rewrite a file marked for
		// compaction.
		d.maybeScheduleCompactionPicker(func(picker compactionPicker, env compactionEnv) *pickedCompaction {
			return picker.pickRewriteCompaction(env)
		})

		// The above attempt might succeed and schedule a rewrite compaction. Or
		// there might not be available compaction concurrency to schedule the
		// compaction.  Or compaction of the file might have already been in
		// progress. In any scenario, wait until there's some change in the
		// state of active compactions.

		// Before waiting, check that the database hasn't been closed. Trying to
		// schedule the compaction may have dropped d.mu while waiting for a
		// manifest write to complete. In that dropped interim, the database may
		// have been closed.
		if err := d.closed.Load(); err != nil {
			return err.(error)
		}

		// Some flush or compaction may have scheduled or completed while we waited
		// for the manifest lock in maybeScheduleCompactionPicker. Get the latest
		// Version before waiting on a compaction.
		curr = d.mu.versions.currentVersion()

		// Only wait on compactions if there are files still marked for compaction.
		// NB: Waiting on this condition variable drops d.mu while blocked.
		if curr.Stats.MarkedForCompaction > 0 {
			if d.mu.compact.compactingCount == 0 {
				panic("expected a compaction of marked files in progress")
			}
			d.mu.compact.cond.Wait()
			// Refresh the current version again.
			curr = d.mu.versions.currentVersion()
		}
	}
	return nil
}

// findFilesFunc scans the LSM for files, returning true if at least one
// file was found. The returned array contains the matched files, if any, per
// level.
type findFilesFunc func(v *version) (found bool, files [numLevels][]*fileMetadata, _ error)

// markFilesWithSplitUserKeys scans the LSM's levels 1 through 6 for adjacent
// files that contain the same user key. Such arrangements of files were
// permitted in RocksDB and in Pebble up to SHA a860bbad.
var markFilesWithSplitUserKeys = func(equal Equal) findFilesFunc {
	return func(v *version) (found bool, files [numLevels][]*fileMetadata, _ error) {
		// Files with split user keys are expected to be rare and performing key
		// comparisons for every file within the LSM is expensive, so drop the
		// database lock while scanning the file metadata.
		for l := numLevels - 1; l > 0; l-- {
			iter := v.Levels[l].Iter()
			var prevFile *fileMetadata
			var prevUserKey []byte
			for f := iter.First(); f != nil; f = iter.Next() {
				if prevUserKey != nil && equal(prevUserKey, f.Smallest.UserKey) {
					// NB: We may append a file twice, once as prevFile and once
					// as f. That's okay, and handled below.
					files[l] = append(files[l], prevFile, f)
					found = true
				}
				if f.Largest.IsExclusiveSentinel() {
					prevUserKey = nil
					prevFile = nil
				} else {
					prevUserKey = f.Largest.UserKey
					prevFile = f
				}
			}
		}
		return
	}
}

// markFilesPrePebblev1 scans the LSM for files that do not support block
// properties (i.e. a table format version pre-Pebblev1).
var markFilesPrePebblev1 = func(tc *tableCacheContainer) findFilesFunc {
	return func(v *version) (found bool, files [numLevels][]*fileMetadata, err error) {
		for l := numLevels - 1; l > 0; l-- {
			iter := v.Levels[l].Iter()
			for f := iter.First(); f != nil; f = iter.Next() {
				if f.Virtual {
					// Any physical sstable which has been virtualized must
					// have already undergone this migration, and we don't
					// need to worry about the virtual sstable themselves.
					panic("pebble: unexpected virtual sstable during migration")
				}
				err = tc.withReader(
					f.PhysicalMeta(), func(r *sstable.Reader) error {
						tf, err := r.TableFormat()
						if err != nil {
							return err
						}
						if tf < sstable.TableFormatPebblev1 {
							found = true
							files[l] = append(files[l], f)
						}
						return nil
					})
				if err != nil {
					return
				}
			}
		}
		return
	}
}

// markFilesLock durably marks the files that match the given findFilesFunc for
// compaction.
func (d *DB) markFilesLocked(findFn findFilesFunc) error {
	jobID := d.mu.nextJobID
	d.mu.nextJobID++

	// Acquire a read state to have a view of the LSM and a guarantee that none
	// of the referenced files will be deleted until we've unreferenced the read
	// state. Some findFilesFuncs may read the files, requiring they not be
	// deleted.
	rs := d.loadReadState()
	var (
		found bool
		files [numLevels][]*fileMetadata
		err   error
	)
	func() {
		defer rs.unrefLocked()
		// Note the unusual locking: unlock, defer Lock(). The scan of the files in
		// the version does not need to block other operations that require the
		// DB.mu. Drop it for the scan, before re-acquiring it.
		d.mu.Unlock()
		defer d.mu.Lock()
		found, files, err = findFn(rs.current)
	}()
	if err != nil {
		return err
	}

	// The database lock has been acquired again by the defer within the above
	// anonymous function.
	if !found {
		// Nothing to do.
		return nil
	}

	// After scanning, if we found files to mark, we fetch the current state of
	// the LSM (which may have changed) and set MarkedForCompaction on the files,
	// and update the version's Stats.MarkedForCompaction count, which are both
	// protected by d.mu.

	// Lock the manifest for a coherent view of the LSM. The database lock has
	// been re-acquired by the defer within the above anonymous function.
	d.mu.versions.logLock()
	vers := d.mu.versions.currentVersion()
	for l, filesToMark := range files {
		if len(filesToMark) == 0 {
			continue
		}
		for _, f := range filesToMark {
			// Ignore files to be marked that have already been compacted or marked.
			if f.CompactionState == manifest.CompactionStateCompacted ||
				f.MarkedForCompaction {
				continue
			}
			// Else, mark the file for compaction in this version.
			vers.Stats.MarkedForCompaction++
			f.MarkedForCompaction = true
		}
		// The compaction picker uses the markedForCompactionAnnotator to
		// quickly find files marked for compaction, or to quickly determine
		// that there are no such files marked for compaction within a level.
		// A b-tree node may be annotated with an annotation recording that
		// there are no files marked for compaction within the node's subtree,
		// based on the assumption that it's static.
		//
		// Since we're marking files for compaction, these b-tree nodes'
		// annotations will be out of date. Clear the compaction-picking
		// annotation, so that it's recomputed the next time the compaction
		// picker looks for a file marked for compaction.
		vers.Levels[l].InvalidateAnnotation(markedForCompactionAnnotator{})
	}

	// The 'marked-for-compaction' bit is persisted in the MANIFEST file
	// metadata. We've already modified the in-memory file metadata, but the
	// manifest hasn't been updated. Force rotation to a new MANIFEST file,
	// which will write every file metadata to the new manifest file and ensure
	// that the now marked-for-compaction file metadata are persisted as marked.
	// NB: This call to logAndApply will unlockthe MANIFEST, which we locked up
	// above before obtaining `vers`.
	return d.mu.versions.logAndApply(
		jobID,
		&manifest.VersionEdit{},
		map[int]*LevelMetrics{},
		true, /* forceRotation */
		func() []compactionInfo { return d.getInProgressCompactionInfoLocked(nil) })
}
