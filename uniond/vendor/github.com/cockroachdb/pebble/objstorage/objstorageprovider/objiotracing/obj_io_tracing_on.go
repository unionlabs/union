// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build pebble_obj_io_tracing
// +build pebble_obj_io_tracing

package objiotracing

import (
	"bufio"
	"context"
	"fmt"
	"math/rand"
	"sync"
	"sync/atomic"
	"time"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/vfs"
)

// Enabled is used to short circuit tracing-related code in regular builds.
const Enabled = true

// Tracer manages the writing of object IO traces to files.
//
// The tracer runs a background worker goroutine which receives trace event
// buffers over a channel and dumps them to IOTRACES- files. Wrapper
// implementations for Readable, ReadHandle, Writable are producers of traces;
// they maintain internal buffers of events which get flushed to the buffered
// channel when they get full. This allows for minimal synchronization per IO
// (as for most of these structures, an instance only allows a single IO at a
// time).
type Tracer struct {
	fs    vfs.FS
	fsDir string

	handleID atomic.Uint64

	workerStopCh chan struct{}
	workerDataCh chan eventBuf
	workerWait   sync.WaitGroup
}

// Open creates a Tracer which generates trace files in the given directory.
// Each trace file contains a series of Events (as they are in memory).
func Open(fs vfs.FS, fsDir string) *Tracer {
	t := &Tracer{
		fs:           fs,
		fsDir:        fsDir,
		workerStopCh: make(chan struct{}),
		workerDataCh: make(chan eventBuf, channelBufSize),
	}

	t.handleID.Store(uint64(rand.NewSource(time.Now().UnixNano()).Int63()))

	t.workerWait.Add(1)
	go t.workerLoop()
	return t
}

// Close the tracer, flushing any remaining events.
func (t *Tracer) Close() {
	if t.workerStopCh == nil {
		return
	}
	// Tell the worker to stop and wait for it to finish up.
	close(t.workerStopCh)
	t.workerWait.Wait()
	t.workerStopCh = nil
}

// WrapWritable wraps an objstorage.Writable with one that generates tracing
// events.
func (t *Tracer) WrapWritable(
	ctx context.Context, w objstorage.Writable, fileNum base.FileNum,
) objstorage.Writable {
	return &writable{
		w:       w,
		fileNum: fileNum,
		g:       makeEventGenerator(ctx, t),
	}
}

type writable struct {
	w         objstorage.Writable
	fileNum   base.FileNum
	curOffset int64
	g         eventGenerator
}

var _ objstorage.Writable = (*writable)(nil)

// Write is part of the objstorage.Writable interface.
func (w *writable) Write(p []byte) error {
	w.g.add(context.Background(), Event{
		Op:      WriteOp,
		FileNum: w.fileNum,
		Offset:  w.curOffset,
		Size:    int64(len(p)),
	})
	// If w.w.Write(p) returns an error, a new writable
	// will be used, so even tho all of p may not have
	// been written to the underlying "file", it is okay
	// to add len(p) to curOffset.
	w.curOffset += int64(len(p))
	return w.w.Write(p)
}

// Finish is part of the objstorage.Writable interface.
func (w *writable) Finish() error {
	w.g.flush()
	return w.w.Finish()
}

// Abort is part of the objstorage.Writable interface.
func (w *writable) Abort() {
	w.g.flush()
	w.w.Abort()
}

// WrapReadable wraps an objstorage.Readable with one that generates tracing
// events.
func (t *Tracer) WrapReadable(
	ctx context.Context, r objstorage.Readable, fileNum base.FileNum,
) objstorage.Readable {
	res := &readable{
		r:       r,
		fileNum: fileNum,
	}
	res.mu.g = makeEventGenerator(ctx, t)
	return res
}

type readable struct {
	r       objstorage.Readable
	fileNum base.FileNum
	mu      struct {
		sync.Mutex
		g eventGenerator
	}
}

var _ objstorage.Readable = (*readable)(nil)

// ReadAt is part of the objstorage.Readable interface.
func (r *readable) ReadAt(ctx context.Context, v []byte, off int64) (n int, err error) {
	r.mu.Lock()
	r.mu.g.add(ctx, Event{
		Op:      ReadOp,
		FileNum: r.fileNum,
		Offset:  off,
		Size:    int64(len(v)),
	})
	r.mu.Unlock()
	return r.r.ReadAt(ctx, v, off)
}

// Close is part of the objstorage.Readable interface.
func (r *readable) Close() error {
	r.mu.g.flush()
	return r.r.Close()
}

// Size is part of the objstorage.Readable interface.
func (r *readable) Size() int64 {
	return r.r.Size()
}

// NewReadHandle is part of the objstorage.Readable interface.
func (r *readable) NewReadHandle(ctx context.Context) objstorage.ReadHandle {
	// It's safe to get the tracer from the generator without the mutex since it never changes.
	t := r.mu.g.t
	return &readHandle{
		rh:       r.r.NewReadHandle(ctx),
		fileNum:  r.fileNum,
		handleID: t.handleID.Add(1),
		g:        makeEventGenerator(ctx, t),
	}
}

type readHandle struct {
	rh       objstorage.ReadHandle
	fileNum  base.FileNum
	handleID uint64
	g        eventGenerator
}

var _ objstorage.ReadHandle = (*readHandle)(nil)

// ReadAt is part of the objstorage.ReadHandle interface.
func (rh *readHandle) ReadAt(ctx context.Context, p []byte, off int64) (n int, err error) {
	rh.g.add(ctx, Event{
		Op:       ReadOp,
		FileNum:  rh.fileNum,
		HandleID: rh.handleID,
		Offset:   off,
		Size:     int64(len(p)),
	})
	return rh.rh.ReadAt(ctx, p, off)
}

// Close is part of the objstorage.ReadHandle interface.
func (rh *readHandle) Close() error {
	rh.g.flush()
	return rh.rh.Close()
}

// SetupForCompaction is part of the objstorage.ReadHandle interface.
func (rh *readHandle) SetupForCompaction() {
	rh.g.add(context.Background(), Event{
		Op:       SetupForCompactionOp,
		FileNum:  rh.fileNum,
		HandleID: rh.handleID,
	})
	rh.rh.SetupForCompaction()
}

// RecordCacheHit is part of the objstorage.ReadHandle interface.
func (rh *readHandle) RecordCacheHit(ctx context.Context, offset, size int64) {
	rh.g.add(ctx, Event{
		Op:       RecordCacheHitOp,
		FileNum:  rh.fileNum,
		HandleID: rh.handleID,
		Offset:   offset,
		Size:     size,
	})
	rh.rh.RecordCacheHit(ctx, offset, size)
}

type ctxInfo struct {
	reason       Reason
	blockType    BlockType
	levelPlusOne uint8
}

func mergeCtxInfo(base, other ctxInfo) ctxInfo {
	res := other
	if res.reason == 0 {
		res.reason = base.reason
	}
	if res.blockType == 0 {
		res.blockType = base.blockType
	}
	if res.levelPlusOne == 0 {
		res.levelPlusOne = base.levelPlusOne
	}
	return res
}

type ctxInfoKey struct{}

func withInfo(ctx context.Context, info ctxInfo) context.Context {
	return context.WithValue(ctx, ctxInfoKey{}, info)
}

func infoFromCtx(ctx context.Context) ctxInfo {
	res := ctx.Value(ctxInfoKey{})
	if res == nil {
		return ctxInfo{}
	}
	return res.(ctxInfo)
}

// WithReason creates a context that has an associated Reason (which ends up in
// traces created under that context).
func WithReason(ctx context.Context, reason Reason) context.Context {
	info := infoFromCtx(ctx)
	info.reason = reason
	return withInfo(ctx, info)
}

// WithBlockType creates a context that has an associated BlockType (which ends up in
// traces created under that context).
func WithBlockType(ctx context.Context, blockType BlockType) context.Context {
	info := infoFromCtx(ctx)
	info.blockType = blockType
	return withInfo(ctx, info)
}

// WithLevel creates a context that has an associated level (which ends up in
// traces created under that context).
func WithLevel(ctx context.Context, level int) context.Context {
	info := infoFromCtx(ctx)
	info.levelPlusOne = uint8(level) + 1
	return withInfo(ctx, info)
}

const (
	eventSize            = int(unsafe.Sizeof(Event{}))
	targetEntriesPerFile = 256 * 1024 * 1024 / eventSize // 256MB files
	eventsPerBuf         = 16
	channelBufSize       = 512 * 1024 / eventsPerBuf // 512K events.
	bytesPerFileSync     = 128 * 1024
)

type eventBuf struct {
	events [eventsPerBuf]Event
	num    int
}

type eventGenerator struct {
	t           *Tracer
	baseCtxInfo ctxInfo
	buf         eventBuf
}

func makeEventGenerator(ctx context.Context, t *Tracer) eventGenerator {
	return eventGenerator{
		t:           t,
		baseCtxInfo: infoFromCtx(ctx),
	}
}

func (g *eventGenerator) flush() {
	if g.buf.num > 0 {
		g.t.workerDataCh <- g.buf
		g.buf.num = 0
	}
}

func (g *eventGenerator) add(ctx context.Context, e Event) {
	e.StartUnixNano = time.Now().UnixNano()
	info := infoFromCtx(ctx)
	info = mergeCtxInfo(g.baseCtxInfo, info)
	e.Reason = info.reason
	e.BlockType = info.blockType
	e.LevelPlusOne = info.levelPlusOne
	if g.buf.num == eventsPerBuf {
		g.flush()
	}
	g.buf.events[g.buf.num] = e
	g.buf.num++
}

type workerState struct {
	curFile          vfs.File
	curBW            *bufio.Writer
	numEntriesInFile int
}

func (t *Tracer) workerLoop() {
	defer t.workerWait.Done()
	stopCh := t.workerStopCh
	dataCh := t.workerDataCh
	var state workerState
	t.workerNewFile(&state)
	for {
		select {
		case <-stopCh:
			close(dataCh)
			// Flush any remaining traces.
			for data := range dataCh {
				t.workerWriteTraces(&state, data)
			}
			t.workerCloseFile(&state)
			return

		case data := <-dataCh:
			t.workerWriteTraces(&state, data)
		}
	}
}

func (t *Tracer) workerWriteTraces(state *workerState, data eventBuf) {
	if state.numEntriesInFile >= targetEntriesPerFile {
		t.workerCloseFile(state)
		t.workerNewFile(state)
	}
	state.numEntriesInFile += data.num
	p := unsafe.Pointer(&data.events[0])
	b := unsafe.Slice((*byte)(p), eventSize*data.num)
	if _, err := state.curBW.Write(b); err != nil {
		panic(err)
	}
}

func (t *Tracer) workerNewFile(state *workerState) {
	filename := fmt.Sprintf("IOTRACES-%s", time.Now().UTC().Format(time.RFC3339Nano))

	file, err := t.fs.Create(t.fs.PathJoin(t.fsDir, filename))
	if err != nil {
		panic(err)
	}
	file = vfs.NewSyncingFile(file, vfs.SyncingFileOptions{
		BytesPerSync: bytesPerFileSync,
	})
	state.curFile = file
	state.curBW = bufio.NewWriter(file)
	state.numEntriesInFile = 0
}

func (t *Tracer) workerCloseFile(state *workerState) {
	if state.curFile != nil {
		if err := state.curBW.Flush(); err != nil {
			panic(err)
		}
		if err := state.curFile.Sync(); err != nil {
			panic(err)
		}
		if err := state.curFile.Close(); err != nil {
			panic(err)
		}
		state.curFile = nil
		state.curBW = nil
	}
}
