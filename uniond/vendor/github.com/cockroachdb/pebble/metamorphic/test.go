// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"fmt"
	"io"
	"os"
	"sort"
	"strings"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/pebble/vfs/errorfs"
)

type test struct {
	// The list of ops to execute. The ops refer to slots in the batches, iters,
	// and snapshots slices.
	ops       []op
	opsWaitOn [][]int         // op index -> op indexes
	opsDone   []chan struct{} // op index -> done channel
	idx       int
	// The DB the test is run on.
	dir       string
	db        *pebble.DB
	opts      *pebble.Options
	testOpts  *TestOptions
	writeOpts *pebble.WriteOptions
	tmpDir    string
	// The slots for the batches, iterators, and snapshots. These are read and
	// written by the ops to pass state from one op to another.
	batches   []*pebble.Batch
	iters     []*retryableIter
	snapshots []readerCloser
}

func newTest(ops []op) *test {
	return &test{
		ops: ops,
	}
}

func (t *test) init(h *history, dir string, testOpts *TestOptions) error {
	t.dir = dir
	t.testOpts = testOpts
	t.writeOpts = pebble.NoSync
	if testOpts.strictFS {
		t.writeOpts = pebble.Sync
	}
	t.opts = testOpts.Opts.EnsureDefaults()
	t.opts.Logger = h
	lel := pebble.MakeLoggingEventListener(t.opts.Logger)
	t.opts.EventListener = &lel
	t.opts.DebugCheck = func(db *pebble.DB) error {
		// Wrap the ordinary DebugCheckLevels with retrying
		// of injected errors.
		return withRetries(func() error {
			return pebble.DebugCheckLevels(db)
		})
	}

	t.opsWaitOn, t.opsDone = computeSynchronizationPoints(t.ops)

	defer t.opts.Cache.Unref()

	// If an error occurs and we were using an in-memory FS, attempt to clone to
	// on-disk in order to allow post-mortem debugging. Note that always using
	// the on-disk FS isn't desirable because there is a large performance
	// difference between in-memory and on-disk which causes different code paths
	// and timings to be exercised.
	maybeExit := func(err error) {
		if err == nil || errors.Is(err, errorfs.ErrInjected) || errors.Is(err, pebble.ErrCancelledCompaction) {
			return
		}
		t.maybeSaveData()
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}

	// Exit early on any error from a background operation.
	t.opts.EventListener.BackgroundError = func(err error) {
		t.opts.Logger.Infof("background error: %s", err)
		maybeExit(err)
	}
	t.opts.EventListener.CompactionEnd = func(info pebble.CompactionInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.FlushEnd = func(info pebble.FlushInfo) {
		t.opts.Logger.Infof("%s", info)
		if info.Err != nil && !strings.Contains(info.Err.Error(), "pebble: empty table") {
			maybeExit(info.Err)
		}
	}
	t.opts.EventListener.ManifestCreated = func(info pebble.ManifestCreateInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.ManifestDeleted = func(info pebble.ManifestDeleteInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.TableDeleted = func(info pebble.TableDeleteInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.TableIngested = func(info pebble.TableIngestInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.WALCreated = func(info pebble.WALCreateInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}
	t.opts.EventListener.WALDeleted = func(info pebble.WALDeleteInfo) {
		t.opts.Logger.Infof("%s", info)
		maybeExit(info.Err)
	}

	for i := range t.testOpts.CustomOpts {
		if err := t.testOpts.CustomOpts[i].Open(t.opts); err != nil {
			return err
		}
	}

	var db *pebble.DB
	var err error
	err = withRetries(func() error {
		db, err = pebble.Open(dir, t.opts)
		return err
	})
	if err != nil {
		return err
	}
	h.log.Printf("// db.Open() %v", err)

	if t.testOpts.sharedStorageEnabled {
		err = withRetries(func() error {
			return db.SetCreatorID(1)
		})
		if err != nil {
			return err
		}
		h.log.Printf("// db.SetCreatorID() %v", err)
	}

	t.tmpDir = t.opts.FS.PathJoin(dir, "tmp")
	if err = t.opts.FS.MkdirAll(t.tmpDir, 0755); err != nil {
		return err
	}
	if t.testOpts.strictFS {
		// Sync the whole directory path for the tmpDir, since restartDB() is executed during
		// the test. That would reset MemFS to the synced state, which would make an unsynced
		// directory disappear in the middle of the test. It is the responsibility of the test
		// (not Pebble) to ensure that it can write the ssts that it will subsequently ingest
		// into Pebble.
		for {
			f, err := t.opts.FS.OpenDir(dir)
			if err != nil {
				return err
			}
			if err = f.Sync(); err != nil {
				return err
			}
			if err = f.Close(); err != nil {
				return err
			}
			if len(dir) == 1 {
				break
			}
			dir = t.opts.FS.PathDir(dir)
			// TODO(sbhola): PathDir returns ".", which OpenDir() complains about. Fix.
			if len(dir) == 1 {
				dir = "/"
			}
		}
	}

	t.db = db
	return nil
}

func (t *test) isFMV(fmv pebble.FormatMajorVersion) bool {
	return t.db.FormatMajorVersion() >= fmv
}

func (t *test) restartDB() error {
	if !t.testOpts.strictFS {
		return nil
	}
	t.opts.Cache.Ref()
	// The fs isn't necessarily a MemFS.
	fs, ok := vfs.Root(t.opts.FS).(*vfs.MemFS)
	if ok {
		fs.SetIgnoreSyncs(true)
	}
	if err := t.db.Close(); err != nil {
		return err
	}
	// Release any resources held by custom options. This may be used, for
	// example, by the encryption-at-rest custom option (within the Cockroach
	// repository) to close the file registry.
	for i := range t.testOpts.CustomOpts {
		if err := t.testOpts.CustomOpts[i].Close(t.opts); err != nil {
			return err
		}
	}
	if ok {
		fs.ResetToSyncedState()
		fs.SetIgnoreSyncs(false)
	}

	// TODO(jackson): Audit errorRate and ensure custom options' hooks semantics
	// are well defined within the context of retries.
	err := withRetries(func() (err error) {
		// Reacquire any resources required by custom options. This may be used, for
		// example, by the encryption-at-rest custom option (within the Cockroach
		// repository) to reopen the file registry.
		for i := range t.testOpts.CustomOpts {
			if err := t.testOpts.CustomOpts[i].Open(t.opts); err != nil {
				return err
			}
		}
		t.db, err = pebble.Open(t.dir, t.opts)
		return err
	})
	t.opts.Cache.Unref()
	return err
}

// If an in-memory FS is being used, save the contents to disk.
func (t *test) maybeSaveData() {
	rootFS := vfs.Root(t.opts.FS)
	if rootFS == vfs.Default {
		return
	}
	_ = os.RemoveAll(t.dir)
	if _, err := vfs.Clone(rootFS, vfs.Default, t.dir, t.dir); err != nil {
		t.opts.Logger.Infof("unable to clone: %s: %v", t.dir, err)
	}
}

func (t *test) step(h *history) bool {
	if t.idx >= len(t.ops) {
		return false
	}
	t.ops[t.idx].run(t, h.recorder(-1 /* thread */, t.idx))
	t.idx++
	return true
}

func (t *test) setBatch(id objID, b *pebble.Batch) {
	if id.tag() != batchTag {
		panic(fmt.Sprintf("invalid batch ID: %s", id))
	}
	t.batches[id.slot()] = b
}

func (t *test) setIter(id objID, i *pebble.Iterator) {
	if id.tag() != iterTag {
		panic(fmt.Sprintf("invalid iter ID: %s", id))
	}
	t.iters[id.slot()] = &retryableIter{
		iter:    i,
		lastKey: nil,
	}
}

type readerCloser interface {
	pebble.Reader
	io.Closer
}

func (t *test) setSnapshot(id objID, s readerCloser) {
	if id.tag() != snapTag {
		panic(fmt.Sprintf("invalid snapshot ID: %s", id))
	}
	t.snapshots[id.slot()] = s
}

func (t *test) clearObj(id objID) {
	switch id.tag() {
	case dbTag:
		t.db = nil
	case batchTag:
		t.batches[id.slot()] = nil
	case iterTag:
		t.iters[id.slot()] = nil
	case snapTag:
		t.snapshots[id.slot()] = nil
	}
}

func (t *test) getBatch(id objID) *pebble.Batch {
	if id.tag() != batchTag {
		panic(fmt.Sprintf("invalid batch ID: %s", id))
	}
	return t.batches[id.slot()]
}

func (t *test) getCloser(id objID) io.Closer {
	switch id.tag() {
	case dbTag:
		return t.db
	case batchTag:
		return t.batches[id.slot()]
	case iterTag:
		return t.iters[id.slot()]
	case snapTag:
		return t.snapshots[id.slot()]
	}
	panic(fmt.Sprintf("cannot close ID: %s", id))
}

func (t *test) getIter(id objID) *retryableIter {
	if id.tag() != iterTag {
		panic(fmt.Sprintf("invalid iter ID: %s", id))
	}
	return t.iters[id.slot()]
}

func (t *test) getReader(id objID) pebble.Reader {
	switch id.tag() {
	case dbTag:
		return t.db
	case batchTag:
		return t.batches[id.slot()]
	case snapTag:
		return t.snapshots[id.slot()]
	}
	panic(fmt.Sprintf("invalid reader ID: %s", id))
}

func (t *test) getWriter(id objID) pebble.Writer {
	switch id.tag() {
	case dbTag:
		return t.db
	case batchTag:
		return t.batches[id.slot()]
	}
	panic(fmt.Sprintf("invalid writer ID: %s", id))
}

// Compute the synchronization points between operations. When operating
// with more than 1 thread, operations must synchronize access to shared
// objects. Compute two slices the same length as ops.
//
// opsWaitOn: the value v at index i indicates that operation i must wait
// for the operation at index v to finish before it may run. NB: v < i
//
// opsDone: the channel at index i must be closed when the operation at index i
// completes. This slice is sparse. Operations that are never used as
// synchronization points may have a nil channel.
func computeSynchronizationPoints(ops []op) (opsWaitOn [][]int, opsDone []chan struct{}) {
	opsDone = make([]chan struct{}, len(ops)) // operation index -> done channel
	opsWaitOn = make([][]int, len(ops))       // operation index -> operation index
	lastOpReference := make(map[objID]int)    // objID -> operation index
	for i, o := range ops {
		// Find the last operation that involved the same receiver object. We at
		// least need to wait on that operation.
		receiver := o.receiver()
		waitIndex, ok := lastOpReference[receiver]
		lastOpReference[receiver] = i
		if !ok {
			// Only valid for i=0. For all other operations, the receiver should
			// have been referenced by some other operation before it's used as
			// a receiver.
			if i != 0 {
				panic(fmt.Sprintf("op %d on receiver %s; first reference of %s", i, receiver, receiver))
			}
			continue
		}

		// The last operation that referenced `receiver` is the one at index
		// `waitIndex`. All operations with the same receiver are performed on
		// the same thread. We only need to synchronize on the operation at
		// `waitIndex` if `receiver` isn't also the receiver on that operation
		// too.
		if ops[waitIndex].receiver() != receiver {
			opsWaitOn[i] = append(opsWaitOn[i], waitIndex)
		}

		// In additional to synchronizing on the operation's receiver operation,
		// we may need to synchronize on additional objects. For example,
		// batch0.Commit() must synchronize its receiver, batch0, but also on
		// dbObjID since it mutates database state.
		for _, syncObjID := range o.syncObjs() {
			if vi, vok := lastOpReference[syncObjID]; vok {
				opsWaitOn[i] = append(opsWaitOn[i], vi)
			}
			lastOpReference[syncObjID] = i
		}

		waitIndexes := opsWaitOn[i]
		sort.Ints(waitIndexes)
		for _, waitIndex := range waitIndexes {
			// If this is the first operation that must wait on the operation at
			// `waitIndex`, then there will be no channel for the operation yet.
			// Create one.
			if opsDone[waitIndex] == nil {
				opsDone[waitIndex] = make(chan struct{})
			}
		}
	}
	return opsWaitOn, opsDone
}
