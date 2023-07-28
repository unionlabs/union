/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package badger

import (
	"encoding/hex"
	"fmt"
	"math"
	"sync"

	"github.com/dgraph-io/badger/v2/pb"
	"github.com/dgraph-io/badger/v2/table"
	"github.com/dgraph-io/badger/v2/y"
	humanize "github.com/dustin/go-humanize"
	"github.com/pkg/errors"
)

const headStreamId uint32 = math.MaxUint32

// StreamWriter is used to write data coming from multiple streams. The streams must not have any
// overlapping key ranges. Within each stream, the keys must be sorted. Badger Stream framework is
// capable of generating such an output. So, this StreamWriter can be used at the other end to build
// BadgerDB at a much faster pace by writing SSTables (and value logs) directly to LSM tree levels
// without causing any compactions at all. This is way faster than using batched writer or using
// transactions, but only applicable in situations where the keys are pre-sorted and the DB is being
// bootstrapped. Existing data would get deleted when using this writer. So, this is only useful
// when restoring from backup or replicating DB across servers.
//
// StreamWriter should not be called on in-use DB instances. It is designed only to bootstrap new
// DBs.
type StreamWriter struct {
	writeLock  sync.Mutex
	db         *DB
	done       func()
	throttle   *y.Throttle
	maxVersion uint64
	writers    map[uint32]*sortedWriter
	maxHead    valuePointer
}

// NewStreamWriter creates a StreamWriter. Right after creating StreamWriter, Prepare must be
// called. The memory usage of a StreamWriter is directly proportional to the number of streams
// possible. So, efforts must be made to keep the number of streams low. Stream framework would
// typically use 16 goroutines and hence create 16 streams.
func (db *DB) NewStreamWriter() *StreamWriter {
	return &StreamWriter{
		db: db,
		// throttle shouldn't make much difference. Memory consumption is based on the number of
		// concurrent streams being processed.
		throttle: y.NewThrottle(16),
		writers:  make(map[uint32]*sortedWriter),
	}
}

// Prepare should be called before writing any entry to StreamWriter. It deletes all data present in
// existing DB, stops compactions and any writes being done by other means. Be very careful when
// calling Prepare, because it could result in permanent data loss. Not calling Prepare would result
// in a corrupt Badger instance.
func (sw *StreamWriter) Prepare() error {
	sw.writeLock.Lock()
	defer sw.writeLock.Unlock()

	var err error
	sw.done, err = sw.db.dropAll()
	return err
}

// Write writes KVList to DB. Each KV within the list contains the stream id which StreamWriter
// would use to demux the writes. Write is thread safe and can be called concurrently by multiple
// goroutines.
func (sw *StreamWriter) Write(kvs *pb.KVList) error {
	if len(kvs.GetKv()) == 0 {
		return nil
	}

	// closedStreams keeps track of all streams which are going to be marked as done. We are
	// keeping track of all streams so that we can close them at the end, after inserting all
	// the valid kvs.
	closedStreams := make(map[uint32]struct{})
	streamReqs := make(map[uint32]*request)
	for _, kv := range kvs.Kv {
		if kv.StreamDone {
			closedStreams[kv.StreamId] = struct{}{}
			continue
		}

		// Panic if some kv comes after stream has been marked as closed.
		if _, ok := closedStreams[kv.StreamId]; ok {
			panic(fmt.Sprintf("write performed on closed stream: %d", kv.StreamId))
		}

		var meta, userMeta byte
		if len(kv.Meta) > 0 {
			meta = kv.Meta[0]
		}
		if len(kv.UserMeta) > 0 {
			userMeta = kv.UserMeta[0]
		}
		if sw.maxVersion < kv.Version {
			sw.maxVersion = kv.Version
		}
		e := &Entry{
			Key:       y.KeyWithTs(kv.Key, kv.Version),
			Value:     kv.Value,
			UserMeta:  userMeta,
			ExpiresAt: kv.ExpiresAt,
			meta:      meta,
		}
		// If the value can be collocated with the key in LSM tree, we can skip
		// writing the value to value log.
		e.skipVlog = sw.db.shouldWriteValueToLSM(*e)
		req := streamReqs[kv.StreamId]
		if req == nil {
			req = &request{}
			streamReqs[kv.StreamId] = req
		}
		req.Entries = append(req.Entries, e)
	}
	all := make([]*request, 0, len(streamReqs))
	for _, req := range streamReqs {
		all = append(all, req)
	}

	sw.writeLock.Lock()
	defer sw.writeLock.Unlock()

	// We are writing all requests to vlog even if some request belongs to already closed stream.
	// It is safe to do because we are panicking while writing to sorted writer, which will be nil
	// for closed stream. At restart, stream writer will drop all the data in Prepare function.
	if err := sw.db.vlog.write(all); err != nil {
		return err
	}

	for streamID, req := range streamReqs {
		writer, ok := sw.writers[streamID]
		if !ok {
			var err error
			writer, err = sw.newWriter(streamID)
			if err != nil {
				return errors.Wrapf(err, "failed to create writer with ID %d", streamID)
			}
			sw.writers[streamID] = writer
		}

		if writer == nil {
			panic(fmt.Sprintf("write performed on closed stream: %d", streamID))
		}

		writer.reqCh <- req
	}

	// Now we can close any streams if required. We will make writer for
	// the closed streams as nil.
	for streamId := range closedStreams {
		writer, ok := sw.writers[streamId]
		if !ok {
			sw.db.opt.Logger.Warningf("Trying to close stream: %d, but no sorted "+
				"writer found for it", streamId)
			continue
		}

		writer.closer.SignalAndWait()
		if err := writer.Done(); err != nil {
			return err
		}

		if sw.maxHead.Less(writer.head) {
			sw.maxHead = writer.head
		}

		sw.writers[streamId] = nil
	}
	return nil
}

// Flush is called once we are done writing all the entries. It syncs DB directories. It also
// updates Oracle with maxVersion found in all entries (if DB is not managed).
func (sw *StreamWriter) Flush() error {
	sw.writeLock.Lock()
	defer sw.writeLock.Unlock()

	defer sw.done()

	for _, writer := range sw.writers {
		if writer != nil {
			writer.closer.SignalAndWait()
		}
	}

	for _, writer := range sw.writers {
		if writer == nil {
			continue
		}
		if err := writer.Done(); err != nil {
			return err
		}
		if sw.maxHead.Less(writer.head) {
			sw.maxHead = writer.head
		}
	}

	// Encode and write the value log head into a new table.
	data := sw.maxHead.Encode()
	headWriter, err := sw.newWriter(headStreamId)
	if err != nil {
		return errors.Wrap(err, "failed to create head writer")
	}
	if err := headWriter.Add(
		y.KeyWithTs(head, sw.maxVersion),
		y.ValueStruct{Value: data}); err != nil {
		return err
	}

	headWriter.closer.SignalAndWait()

	if err := headWriter.Done(); err != nil {
		return err
	}

	if !sw.db.opt.managedTxns {
		if sw.db.orc != nil {
			sw.db.orc.Stop()
		}
		sw.db.orc = newOracle(sw.db.opt)
		sw.db.orc.nextTxnTs = sw.maxVersion
		sw.db.orc.txnMark.Done(sw.maxVersion)
		sw.db.orc.readMark.Done(sw.maxVersion)
		sw.db.orc.incrementNextTs()
	}

	// Wait for all files to be written.
	if err := sw.throttle.Finish(); err != nil {
		return err
	}

	// Sort tables at the end.
	for _, l := range sw.db.lc.levels {
		l.sortTables()
	}

	// Now sync the directories, so all the files are registered.
	if sw.db.opt.ValueDir != sw.db.opt.Dir {
		if err := sw.db.syncDir(sw.db.opt.ValueDir); err != nil {
			return err
		}
	}
	if err := sw.db.syncDir(sw.db.opt.Dir); err != nil {
		return err
	}
	return sw.db.lc.validate()
}

type sortedWriter struct {
	db       *DB
	throttle *y.Throttle

	builder  *table.Builder
	lastKey  []byte
	streamID uint32
	reqCh    chan *request
	head     valuePointer
	// Have separate closer for each writer, as it can be closed at any time.
	closer *y.Closer
}

func (sw *StreamWriter) newWriter(streamID uint32) (*sortedWriter, error) {
	dk, err := sw.db.registry.latestDataKey()
	if err != nil {
		return nil, err
	}

	bopts := buildTableOptions(sw.db.opt)
	bopts.DataKey = dk
	w := &sortedWriter{
		db:       sw.db,
		streamID: streamID,
		throttle: sw.throttle,
		builder:  table.NewTableBuilder(bopts),
		reqCh:    make(chan *request, 3),
		closer:   y.NewCloser(1),
	}

	go w.handleRequests()
	return w, nil
}

func (w *sortedWriter) handleRequests() {
	defer w.closer.Done()

	process := func(req *request) {
		for i, e := range req.Entries {
			// If badger is running in InMemory mode, len(req.Ptrs) == 0.
			if i < len(req.Ptrs) {
				vptr := req.Ptrs[i]
				if !vptr.IsZero() {
					y.AssertTrue(w.head.Less(vptr))
					w.head = vptr
				}
			}
			var vs y.ValueStruct
			if e.skipVlog {
				vs = y.ValueStruct{
					Value:     e.Value,
					Meta:      e.meta,
					UserMeta:  e.UserMeta,
					ExpiresAt: e.ExpiresAt,
				}
			} else {
				vptr := req.Ptrs[i]
				vs = y.ValueStruct{
					Value:     vptr.Encode(),
					Meta:      e.meta | bitValuePointer,
					UserMeta:  e.UserMeta,
					ExpiresAt: e.ExpiresAt,
				}
			}
			if err := w.Add(e.Key, vs); err != nil {
				panic(err)
			}
		}
	}

	for {
		select {
		case req := <-w.reqCh:
			process(req)
		case <-w.closer.HasBeenClosed():
			close(w.reqCh)
			for req := range w.reqCh {
				process(req)
			}
			return
		}
	}
}

// Add adds key and vs to sortedWriter.
func (w *sortedWriter) Add(key []byte, vs y.ValueStruct) error {
	if len(w.lastKey) > 0 && y.CompareKeys(key, w.lastKey) <= 0 {
		return errors.Errorf("keys not in sorted order (last key: %s, key: %s)",
			hex.Dump(w.lastKey), hex.Dump(key))
	}

	sameKey := y.SameKey(key, w.lastKey)
	// Same keys should go into the same SSTable.
	if !sameKey && w.builder.ReachedCapacity(w.db.opt.MaxTableSize) {
		if err := w.send(false); err != nil {
			return err
		}
	}

	w.lastKey = y.SafeCopy(w.lastKey, key)
	var vp valuePointer
	if vs.Meta&bitValuePointer > 0 {
		vp.Decode(vs.Value)
	}
	w.builder.Add(key, vs, vp.Len)
	return nil
}

func (w *sortedWriter) send(done bool) error {
	if err := w.throttle.Do(); err != nil {
		return err
	}
	go func(builder *table.Builder) {
		err := w.createTable(builder)
		w.throttle.Done(err)
	}(w.builder)
	// If done is true, this indicates we can close the writer.
	// No need to allocate underlying TableBuilder now.
	if done {
		w.builder = nil
		return nil
	}

	dk, err := w.db.registry.latestDataKey()
	if err != nil {
		return y.Wrapf(err, "Error while retriving datakey in sortedWriter.send")
	}
	bopts := buildTableOptions(w.db.opt)
	bopts.DataKey = dk
	w.builder = table.NewTableBuilder(bopts)
	return nil
}

// Done is called once we are done writing all keys and valueStructs
// to sortedWriter. It completes writing current SST to disk.
func (w *sortedWriter) Done() error {
	if w.builder.Empty() {
		// Assign builder as nil, so that underlying memory can be garbage collected.
		w.builder = nil
		return nil
	}

	return w.send(true)
}

func (w *sortedWriter) createTable(builder *table.Builder) error {
	data := builder.Finish()
	if len(data) == 0 {
		return nil
	}
	fileID := w.db.lc.reserveFileID()
	opts := buildTableOptions(w.db.opt)
	opts.DataKey = builder.DataKey()
	opts.BlockCache = w.db.blockCache
	opts.IndexCache = w.db.indexCache
	var tbl *table.Table
	if w.db.opt.InMemory {
		var err error
		if tbl, err = table.OpenInMemoryTable(data, fileID, &opts); err != nil {
			return err
		}
	} else {
		fd, err := y.CreateSyncedFile(table.NewFilename(fileID, w.db.opt.Dir), true)
		if err != nil {
			return err
		}
		if _, err := fd.Write(data); err != nil {
			return err
		}
		if tbl, err = table.OpenTable(fd, opts); err != nil {
			return err
		}
	}
	lc := w.db.lc

	var lhandler *levelHandler
	// We should start the levels from 1, because we need level 0 to set the !badger!head key. We
	// cannot mix up this key with other keys from the DB, otherwise we would introduce a range
	// overlap violation.
	y.AssertTrue(len(lc.levels) > 1)
	for _, l := range lc.levels[1:] {
		ratio := float64(l.getTotalSize()) / float64(l.maxTotalSize)
		if ratio < 1.0 {
			lhandler = l
			break
		}
	}
	if lhandler == nil {
		// If we're exceeding the size of the lowest level, shove it in the lowest level. Can't do
		// better than that.
		lhandler = lc.levels[len(lc.levels)-1]
	}
	if w.streamID == headStreamId {
		// This is a special !badger!head key. We should store it at level 0, separate from all the
		// other keys to avoid an overlap.
		lhandler = lc.levels[0]
	}
	// Now that table can be opened successfully, let's add this to the MANIFEST.
	change := &pb.ManifestChange{
		Id:          tbl.ID(),
		KeyId:       tbl.KeyID(),
		Op:          pb.ManifestChange_CREATE,
		Level:       uint32(lhandler.level),
		Compression: uint32(tbl.CompressionType()),
	}
	if err := w.db.manifest.addChanges([]*pb.ManifestChange{change}); err != nil {
		return err
	}

	// We are not calling lhandler.replaceTables() here, as it sorts tables on every addition.
	// We can sort all tables only once during Flush() call.
	lhandler.addTable(tbl)

	// Release the ref held by OpenTable.
	_ = tbl.DecrRef()
	w.db.opt.Infof("Table created: %d at level: %d for stream: %d. Size: %s\n",
		fileID, lhandler.level, w.streamID, humanize.Bytes(uint64(tbl.Size())))
	return nil
}
