// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package replay

import (
	"fmt"
	"io"
	"sync"
	"sync/atomic"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/vfs"
)

type workloadCaptureState uint8

const (
	obsolete = workloadCaptureState(1) << iota
	readyForProcessing
	capturedSuccessfully
)

func (wcs workloadCaptureState) is(flag workloadCaptureState) bool { return wcs&flag != 0 }

type manifestDetails struct {
	sourceFilepath string
	sourceFile     vfs.File

	destFile vfs.File
}

// WorkloadCollector is designed to capture workloads by handling manifest
// files, flushed SSTs and ingested SSTs. The collector hooks into the
// pebble.EventListener and pebble.Cleaner in order keep track of file states.
type WorkloadCollector struct {
	mu struct {
		sync.Mutex
		fileState map[string]workloadCaptureState
		// pendingSSTables holds a slice of file paths to sstables that need to
		// be copied but haven't yet. The `copyFiles` goroutine grabs these
		// files, and the flush and ingest event handlers append them.
		pendingSSTables []string
		// manifestIndex is an index into `manifests`, pointing to the
		// manifest currently being copied.
		manifestIndex int
		// appending to manifests requires holding mu. Only the `copyFiles`
		// goroutine is permitted to read or edit the struct contents once
		// appended, so it does not need to hold mu while accessing the structs'
		// fields.
		manifests []*manifestDetails

		// The following condition variable and counts are used in tests to
		// synchronize with the copying goroutine.
		copyCond       sync.Cond
		tablesCopied   int
		tablesEnqueued int
	}
	// Stores the current manifest that is being used by the database.
	curManifest atomic.Uint64
	// Stores whether the workload collector is enabled.
	enabled atomic.Bool
	buffer  []byte
	// config contains information that is only set on the creation of the
	// WorkloadCollector.
	config struct {
		// srcFS and srcDir represent the location from which the workload collector
		// collects the files from.
		srcFS  vfs.FS
		srcDir string
		// destFS and destDir represent the location to which the workload collector
		// sends the files to.
		destFS  vfs.FS
		destDir string
		// cleaner stores the cleaner to use when files become obsolete and need to
		// be cleaned.
		cleaner base.Cleaner
	}
	copier struct {
		sync.Cond
		stop bool
		done chan struct{}
	}
}

// NewWorkloadCollector is used externally to create a New WorkloadCollector.
func NewWorkloadCollector(srcDir string) *WorkloadCollector {
	wc := &WorkloadCollector{}
	wc.buffer = make([]byte, 1<<10 /* 1KB */)
	wc.config.srcDir = srcDir
	wc.mu.copyCond.L = &wc.mu.Mutex
	wc.mu.fileState = make(map[string]workloadCaptureState)
	wc.copier.Cond.L = &wc.mu.Mutex
	return wc
}

// Attach is used to set up the WorkloadCollector by attaching itself to
// pebble.Options EventListener and Cleaner.
func (w *WorkloadCollector) Attach(opts *pebble.Options) {
	opts.AddEventListener(pebble.EventListener{
		FlushEnd:        w.onFlushEnd,
		ManifestCreated: w.onManifestCreated,
		TableIngested:   w.onTableIngest,
	})

	opts.EnsureDefaults()
	// Replace the original Cleaner with the workload collector's implementation,
	// which will invoke the original Cleaner, but only once the collector's copied
	// what it needs.
	c := cleaner{
		name:  fmt.Sprintf("replay.WorkloadCollector(%q)", opts.Cleaner),
		clean: w.clean,
	}
	w.config.cleaner, opts.Cleaner = opts.Cleaner, c
	w.config.srcFS = opts.FS
}

// enqueueCopyLocked enqueues the sstable with the provided filenum be copied in
// the background. Requires w.mu.
func (w *WorkloadCollector) enqueueCopyLocked(fileNum base.DiskFileNum) {
	fileName := base.MakeFilename(base.FileTypeTable, fileNum)
	w.mu.fileState[fileName] |= readyForProcessing
	w.mu.pendingSSTables = append(w.mu.pendingSSTables, w.srcFilepath(fileName))
	w.mu.tablesEnqueued++
}

// cleanFile calls the cleaner on the specified path and removes the path from
// the fileState map.
func (w *WorkloadCollector) cleanFile(fileType base.FileType, path string) error {
	err := w.config.cleaner.Clean(w.config.srcFS, fileType, path)
	if err == nil {
		w.mu.Lock()
		delete(w.mu.fileState, w.config.srcFS.PathBase(path))
		w.mu.Unlock()
	}
	return err
}

// clean deletes files only after they have been processed or are not required
// for the workload collection.
func (w *WorkloadCollector) clean(fs vfs.FS, fileType base.FileType, path string) error {
	if !w.IsRunning() {
		return w.cleanFile(fileType, path)
	}
	w.mu.Lock()
	fileName := fs.PathBase(path)
	if fileState, ok := w.mu.fileState[fileName]; !ok || fileState.is(capturedSuccessfully) {
		// Delete the file if it has been captured or the file is not important
		// to capture which means it can be deleted.
		w.mu.Unlock()
		return w.cleanFile(fileType, path)
	}
	w.mu.fileState[fileName] |= obsolete
	w.mu.Unlock()
	return nil
}

// onTableIngest is attached to a pebble.DB as an EventListener.TableIngested
// func. It enqueues all ingested tables to be copied.
func (w *WorkloadCollector) onTableIngest(info pebble.TableIngestInfo) {
	if !w.IsRunning() {
		return
	}
	w.mu.Lock()
	defer w.mu.Unlock()
	for _, table := range info.Tables {
		w.enqueueCopyLocked(table.FileNum.DiskFileNum())
	}
	w.copier.Broadcast()
}

// onFlushEnd is attached to a pebble.DB as an EventListener.FlushEnd func. It
// enqueues all flushed tables to be copied.
func (w *WorkloadCollector) onFlushEnd(info pebble.FlushInfo) {
	if !w.IsRunning() {
		return
	}
	w.mu.Lock()
	defer w.mu.Unlock()
	for _, table := range info.Output {
		w.enqueueCopyLocked(table.FileNum.DiskFileNum())
	}
	w.copier.Broadcast()
}

// onManifestCreated is attached to a pebble.DB as an
// EventListener.ManifestCreated func. It records the the new manifest so that
// it's copied asynchronously in the background.
func (w *WorkloadCollector) onManifestCreated(info pebble.ManifestCreateInfo) {
	w.curManifest.Store(uint64(info.FileNum))
	if !w.enabled.Load() {
		return
	}
	w.mu.Lock()
	defer w.mu.Unlock()

	// mark the manifest file as ready for processing to prevent it from being
	// cleaned before we process it.
	fileName := base.MakeFilename(base.FileTypeManifest, info.FileNum.DiskFileNum())
	w.mu.fileState[fileName] |= readyForProcessing
	w.mu.manifests = append(w.mu.manifests, &manifestDetails{
		sourceFilepath: info.Path,
	})
}

// copyFiles is run in a separate goroutine, copying sstables and manifests.
func (w *WorkloadCollector) copyFiles() {
	w.mu.Lock()
	defer w.mu.Unlock()
	// NB: This loop must hold w.mu at the beginning of each iteration. It may
	// drop w.mu at times, but it must reacquire it before the next iteration.
	for !w.copier.stop {
		// The following performs the workload capture. It waits on a condition
		// variable (fileListener) to let it know when new files are available to be
		// collected.
		if len(w.mu.pendingSSTables) == 0 {
			w.copier.Wait()
		}
		// Grab the manifests to copy.
		index := w.mu.manifestIndex
		pendingManifests := w.mu.manifests[index:]
		var pending []string
		pending, w.mu.pendingSSTables = w.mu.pendingSSTables, nil
		func() {
			// Note the unusual lock order; Temporarily unlock the
			// mutex, but re-acquire it before returning.
			w.mu.Unlock()
			defer w.mu.Lock()

			// Copy any updates to the manifests files.
			w.copyManifests(index, pendingManifests)
			// Copy the SSTables provided in pending. copySSTables takes
			// ownership of the pending slice.
			w.copySSTables(pending)
		}()

		// This helps in tests; Tests can wait on the copyCond condition
		// variable until the necessary bits have been copied.
		w.mu.tablesCopied += len(pending)
		w.mu.copyCond.Broadcast()
	}

	for idx := range w.mu.manifests {
		if f := w.mu.manifests[idx].sourceFile; f != nil {
			if err := f.Close(); err != nil {
				panic(err)
			}
			w.mu.manifests[idx].sourceFile = nil
		}
		if f := w.mu.manifests[idx].destFile; f != nil {
			if err := f.Close(); err != nil {
				panic(err)
			}
			w.mu.manifests[idx].destFile = nil
		}
	}
	close(w.copier.done)
}

// copyManifests copies any un-copied portions of the source manifests.
func (w *WorkloadCollector) copyManifests(startAtIndex int, manifests []*manifestDetails) {
	destFS := w.config.destFS

	for index, manifest := range manifests {
		if manifest.destFile == nil && manifest.sourceFile == nil {
			// This is the first time we've read from this manifest, and we
			// don't yet have open file descriptors for the src or dst files. It
			// is safe to write to manifest.{destFile,sourceFile} without
			// holding d.mu, because the copyFiles goroutine is the only
			// goroutine that accesses the fields of the `manifestDetails`
			// struct.
			var err error
			manifest.destFile, err = destFS.Create(w.destFilepath(destFS.PathBase(manifest.sourceFilepath)))
			if err != nil {
				panic(err)
			}
			manifest.sourceFile, err = w.config.srcFS.Open(manifest.sourceFilepath)
			if err != nil {
				panic(err)
			}
		}

		numBytesRead, err := io.CopyBuffer(manifest.destFile, manifest.sourceFile, w.buffer)
		if err != nil {
			panic(err)
		}

		// Read 0 bytes from the current manifest and this is not the
		// latest/newest manifest which means we have read its entirety. No new
		// data will be written to it, because only the latest manifest may
		// receive edits. Close the current source and destination files and
		// move the manifest to start at the next index in w.mu.manifests.
		if numBytesRead == 0 && index != len(manifests)-1 {
			// Rotating the manifests so we can close the files.
			if err := manifests[index].sourceFile.Close(); err != nil {
				panic(err)
			}
			manifests[index].sourceFile = nil
			if err := manifests[index].destFile.Close(); err != nil {
				panic(err)
			}
			manifests[index].destFile = nil
			w.mu.Lock()
			w.mu.manifestIndex = startAtIndex + index + 1
			w.mu.Unlock()
		}
	}
}

// copySSTables copies the provided sstables to the stored workload. If a file
// has already been marked as obsolete, then file will be cleaned by the
// w.config.cleaner after it is copied. The provided slice will be mutated and
// should not be used following the call to this function.
func (w *WorkloadCollector) copySSTables(pending []string) {
	for _, filePath := range pending {
		err := vfs.CopyAcrossFS(w.config.srcFS,
			filePath,
			w.config.destFS,
			w.destFilepath(w.config.srcFS.PathBase(filePath)))
		if err != nil {
			panic(err)
		}
	}

	// Identify the subset of `pending` files that should now be cleaned. The
	// WorkloadCollector intercepts Cleaner.Clean calls to defer cleaning until
	// copying has completed. If Cleaner.Clean has already been invoked for any
	// of the files that copied, we can now actually Clean them.
	pendingClean := pending[:0]
	w.mu.Lock()
	for _, filePath := range pending {
		fileName := w.config.srcFS.PathBase(filePath)
		if w.mu.fileState[fileName].is(obsolete) {
			pendingClean = append(pendingClean, filePath)
		} else {
			w.mu.fileState[fileName] |= capturedSuccessfully
		}
	}
	w.mu.Unlock()

	for _, path := range pendingClean {
		_ = w.cleanFile(base.FileTypeTable, path)
	}
}

// Start begins collecting a workload. All flushed and ingested sstables, plus
// corresponding manifests are copied to the provided destination path on the
// provided FS.
func (w *WorkloadCollector) Start(destFS vfs.FS, destPath string) {
	w.mu.Lock()
	defer w.mu.Unlock()

	// If the collector not is running then that means w.enabled == 0 so swap it
	// to 1 and continue else return since it is already running.
	if !w.enabled.CompareAndSwap(false, true) {
		return
	}
	w.config.destFS = destFS
	w.config.destDir = destPath

	// Initialize the tracked manifests to the database's current manifest, if
	// the database has already started. Every database Open creates a new
	// manifest. There are two cases:
	//   1. The database has already been opened. Then `w.atomic.curManifest`
	//      contains the file number of the current manifest. We must initialize
	//      the w.mu.manifests slice to contain this first manifest.
	//   2. The database has not yet been opened. Then `w.atomic.curManifest` is
	//      still zero. Once the associated database is opened, it'll invoke
	//      onManifestCreated which will handle enqueuing the manifest on
	//      `w.mu.manifests`.
	fileNum := base.FileNum(w.curManifest.Load())
	if fileNum != 0 {
		fileName := base.MakeFilename(base.FileTypeManifest, fileNum.DiskFileNum())
		w.mu.manifests = append(w.mu.manifests[:0], &manifestDetails{sourceFilepath: w.srcFilepath(fileName)})
		w.mu.fileState[fileName] |= readyForProcessing
	}

	// Begin copying files asynchronously in the background.
	w.copier.done = make(chan struct{})
	w.copier.stop = false
	go w.copyFiles()
}

// WaitAndStop waits for all enqueued sstables to be copied over, and then
// calls Stop. Gracefully ensures that all sstables referenced in the collected
// manifest's latest version edit will exist in the copy directory.
func (w *WorkloadCollector) WaitAndStop() {
	w.mu.Lock()
	for w.mu.tablesEnqueued != w.mu.tablesCopied {
		w.mu.copyCond.Wait()
	}
	w.mu.Unlock()
	w.Stop()
}

// Stop stops collection of the workload.
func (w *WorkloadCollector) Stop() {
	w.mu.Lock()
	// If the collector is running then that means w.enabled == true so swap it to
	// false and continue else return since it is not running.
	if !w.enabled.CompareAndSwap(true, false) {
		w.mu.Unlock()
		return
	}
	w.copier.stop = true
	w.copier.Broadcast()
	w.mu.Unlock()
	<-w.copier.done
}

// IsRunning returns whether the WorkloadCollector is currently running.
func (w *WorkloadCollector) IsRunning() bool {
	return w.enabled.Load()
}

// srcFilepath returns the file path to the named file in the source directory
// on the source filesystem.
func (w *WorkloadCollector) srcFilepath(name string) string {
	return w.config.srcFS.PathJoin(w.config.srcDir, name)
}

// destFilepath returns the file path to the named file in the destination
// directory on the destination filesystem.
func (w *WorkloadCollector) destFilepath(name string) string {
	return w.config.destFS.PathJoin(w.config.destDir, name)
}

type cleaner struct {
	name  string
	clean func(vfs.FS, base.FileType, string) error
}

func (c cleaner) String() string { return c.name }
func (c cleaner) Clean(fs vfs.FS, fileType base.FileType, path string) error {
	return c.clean(fs, fileType, path)
}
