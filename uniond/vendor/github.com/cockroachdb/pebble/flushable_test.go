package pebble

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

// Simple sanity tests for the flushable interface implementation for ingested
// sstables.
func TestIngestedSSTFlushableAPI(t *testing.T) {
	var mem vfs.FS
	var d *DB
	defer func() {
		require.NoError(t, d.Close())
	}()
	var flushable flushable

	reset := func() {
		if d != nil {
			require.NoError(t, d.Close())
		}

		mem = vfs.NewMem()
		require.NoError(t, mem.MkdirAll("ext", 0755))
		opts := &Options{
			FS:                    mem,
			L0CompactionThreshold: 100,
			L0StopWritesThreshold: 100,
			DebugCheck:            DebugCheckLevels,
			FormatMajorVersion:    internalFormatNewest,
		}
		// Disable automatic compactions because otherwise we'll race with
		// delete-only compactions triggered by ingesting range tombstones.
		opts.DisableAutomaticCompactions = true

		var err error
		d, err = Open("", opts)
		require.NoError(t, err)
		flushable = nil
	}
	reset()

	loadFileMeta := func(paths []string) []*fileMetadata {
		d.mu.Lock()
		pendingOutputs := make([]base.DiskFileNum, len(paths))
		for i := range paths {
			pendingOutputs[i] = d.mu.versions.getNextFileNum().DiskFileNum()
		}
		jobID := d.mu.nextJobID
		d.mu.nextJobID++
		d.mu.Unlock()

		// We can reuse the ingestLoad function for this test even if we're
		// not actually ingesting a file.
		lr, err := ingestLoad(d.opts, d.FormatMajorVersion(), paths, nil, nil, d.cacheID, pendingOutputs, d.objProvider, jobID)
		if err != nil {
			panic(err)
		}
		meta := lr.localMeta
		if len(meta) == 0 {
			// All of the sstables to be ingested were empty. Nothing to do.
			panic("empty sstable")
		}
		// The table cache requires the *fileMetadata to have a positive
		// reference count. Fake a reference before we try to load the file.
		for _, f := range meta {
			f.Ref()
		}

		// Verify the sstables do not overlap.
		if err := ingestSortAndVerify(d.cmp, lr, KeyRange{}); err != nil {
			panic("unsorted sstables")
		}

		// Hard link the sstables into the DB directory. Since the sstables aren't
		// referenced by a version, they won't be used. If the hard linking fails
		// (e.g. because the files reside on a different filesystem), ingestLink will
		// fall back to copying, and if that fails we undo our work and return an
		// error.
		if err := ingestLink(jobID, d.opts, d.objProvider, lr, nil /* shared */); err != nil {
			panic("couldn't hard link sstables")
		}

		// Fsync the directory we added the tables to. We need to do this at some
		// point before we update the MANIFEST (via logAndApply), otherwise a crash
		// can have the tables referenced in the MANIFEST, but not present in the
		// directory.
		if err := d.dataDir.Sync(); err != nil {
			panic("Couldn't sync data directory")
		}

		return meta
	}

	datadriven.RunTest(t, "testdata/ingested_flushable_api", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			reset()
			return ""
		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""
		case "flushable":
			// Creates an ingestedFlushable over the input files.
			paths := make([]string, 0, len(td.CmdArgs))
			for _, arg := range td.CmdArgs {
				paths = append(paths, arg.String())
			}

			meta := loadFileMeta(paths)
			flushable = newIngestedFlushable(
				meta, d.opts.Comparer, d.newIters, d.tableNewRangeKeyIter,
			)
			return ""
		case "iter":
			iter := flushable.newIter(nil)
			var buf bytes.Buffer
			for x, _ := iter.First(); x != nil; x, _ = iter.Next() {
				buf.WriteString(x.String())
				buf.WriteString("\n")
			}
			iter.Close()
			return buf.String()
		case "rangekeyIter":
			iter := flushable.newRangeKeyIter(nil)
			var buf bytes.Buffer
			if iter != nil {
				for span := iter.First(); span != nil; span = iter.Next() {
					buf.WriteString(span.String())
					buf.WriteString("\n")
				}
				iter.Close()
			}
			return buf.String()
		case "rangedelIter":
			iter := flushable.newRangeDelIter(nil)
			var buf bytes.Buffer
			if iter != nil {
				for span := iter.First(); span != nil; span = iter.Next() {
					buf.WriteString(span.String())
					buf.WriteString("\n")
				}
				iter.Close()
			}
			return buf.String()
		case "readyForFlush":
			if flushable.readyForFlush() {
				return "true"
			}
			return "false"
		case "containsRangeKey":
			if flushable.containsRangeKeys() {
				return "true"
			}
			return "false"
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}
