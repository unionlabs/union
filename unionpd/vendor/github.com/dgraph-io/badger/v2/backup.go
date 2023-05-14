/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
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
	"bufio"
	"bytes"
	"context"
	"encoding/binary"
	"io"

	"github.com/dgraph-io/badger/v2/pb"
	"github.com/dgraph-io/badger/v2/y"
	"github.com/golang/protobuf/proto"
)

// flushThreshold determines when a buffer will be flushed. When performing a
// backup/restore, the entries will be batched up until the total size of batch
// is more than flushThreshold or entry size (without the value size) is more
// than the maxBatchSize.
const flushThreshold = 100 << 20

// Backup is a wrapper function over Stream.Backup to generate full and incremental backups of the
// DB. For more control over how many goroutines are used to generate the backup, or if you wish to
// backup only a certain range of keys, use Stream.Backup directly.
func (db *DB) Backup(w io.Writer, since uint64) (uint64, error) {
	stream := db.NewStream()
	stream.LogPrefix = "DB.Backup"
	return stream.Backup(w, since)
}

// Backup dumps a protobuf-encoded list of all entries in the database into the
// given writer, that are newer than the specified version. It returns a
// timestamp indicating when the entries were dumped which can be passed into a
// later invocation to generate an incremental dump, of entries that have been
// added/modified since the last invocation of Stream.Backup().
//
// This can be used to backup the data in a database at a given point in time.
func (stream *Stream) Backup(w io.Writer, since uint64) (uint64, error) {
	stream.KeyToList = func(key []byte, itr *Iterator) (*pb.KVList, error) {
		list := &pb.KVList{}
		for ; itr.Valid(); itr.Next() {
			item := itr.Item()
			if !bytes.Equal(item.Key(), key) {
				return list, nil
			}
			if item.Version() < since {
				// Ignore versions less than given timestamp, or skip older
				// versions of the given key.
				return list, nil
			}

			var valCopy []byte
			if !item.IsDeletedOrExpired() {
				// No need to copy value, if item is deleted or expired.
				var err error
				valCopy, err = item.ValueCopy(nil)
				if err != nil {
					stream.db.opt.Errorf("Key [%x, %d]. Error while fetching value [%v]\n",
						item.Key(), item.Version(), err)
					return nil, err
				}
			}

			// clear txn bits
			meta := item.meta &^ (bitTxn | bitFinTxn)
			kv := &pb.KV{
				Key:       item.KeyCopy(nil),
				Value:     valCopy,
				UserMeta:  []byte{item.UserMeta()},
				Version:   item.Version(),
				ExpiresAt: item.ExpiresAt(),
				Meta:      []byte{meta},
			}
			list.Kv = append(list.Kv, kv)

			switch {
			case item.DiscardEarlierVersions():
				// If we need to discard earlier versions of this item, add a delete
				// marker just below the current version.
				list.Kv = append(list.Kv, &pb.KV{
					Key:     item.KeyCopy(nil),
					Version: item.Version() - 1,
					Meta:    []byte{bitDelete},
				})
				return list, nil

			case item.IsDeletedOrExpired():
				return list, nil
			}
		}
		return list, nil
	}

	var maxVersion uint64
	stream.Send = func(list *pb.KVList) error {
		for _, kv := range list.Kv {
			if maxVersion < kv.Version {
				maxVersion = kv.Version
			}
		}
		return writeTo(list, w)
	}

	if err := stream.Orchestrate(context.Background()); err != nil {
		return 0, err
	}
	return maxVersion, nil
}

func writeTo(list *pb.KVList, w io.Writer) error {
	if err := binary.Write(w, binary.LittleEndian, uint64(proto.Size(list))); err != nil {
		return err
	}
	buf, err := proto.Marshal(list)
	if err != nil {
		return err
	}
	_, err = w.Write(buf)
	return err
}

// KVLoader is used to write KVList objects in to badger. It can be used to restore a backup.
type KVLoader struct {
	db          *DB
	throttle    *y.Throttle
	entries     []*Entry
	entriesSize int64
	totalSize   int64
}

// NewKVLoader returns a new instance of KVLoader.
func (db *DB) NewKVLoader(maxPendingWrites int) *KVLoader {
	return &KVLoader{
		db:       db,
		throttle: y.NewThrottle(maxPendingWrites),
		entries:  make([]*Entry, 0, db.opt.maxBatchCount),
	}
}

// Set writes the key-value pair to the database.
func (l *KVLoader) Set(kv *pb.KV) error {
	var userMeta, meta byte
	if len(kv.UserMeta) > 0 {
		userMeta = kv.UserMeta[0]
	}
	if len(kv.Meta) > 0 {
		meta = kv.Meta[0]
	}
	e := &Entry{
		Key:       y.KeyWithTs(kv.Key, kv.Version),
		Value:     kv.Value,
		UserMeta:  userMeta,
		ExpiresAt: kv.ExpiresAt,
		meta:      meta,
	}
	estimatedSize := int64(e.estimateSize(l.db.opt.ValueThreshold))
	// Flush entries if inserting the next entry would overflow the transactional limits.
	if int64(len(l.entries))+1 >= l.db.opt.maxBatchCount ||
		l.entriesSize+estimatedSize >= l.db.opt.maxBatchSize ||
		l.totalSize >= flushThreshold {
		if err := l.send(); err != nil {
			return err
		}
	}
	l.entries = append(l.entries, e)
	l.entriesSize += estimatedSize
	l.totalSize += estimatedSize + int64(len(e.Value))
	return nil
}

func (l *KVLoader) send() error {
	if err := l.throttle.Do(); err != nil {
		return err
	}
	if err := l.db.batchSetAsync(l.entries, func(err error) {
		l.throttle.Done(err)
	}); err != nil {
		return err
	}

	l.entries = make([]*Entry, 0, l.db.opt.maxBatchCount)
	l.entriesSize = 0
	l.totalSize = 0
	return nil
}

// Finish is meant to be called after all the key-value pairs have been loaded.
func (l *KVLoader) Finish() error {
	if len(l.entries) > 0 {
		if err := l.send(); err != nil {
			return err
		}
	}
	return l.throttle.Finish()
}

// Load reads a protobuf-encoded list of all entries from a reader and writes
// them to the database. This can be used to restore the database from a backup
// made by calling DB.Backup(). If more complex logic is needed to restore a badger
// backup, the KVLoader interface should be used instead.
//
// DB.Load() should be called on a database that is not running any other
// concurrent transactions while it is running.
func (db *DB) Load(r io.Reader, maxPendingWrites int) error {
	br := bufio.NewReaderSize(r, 16<<10)
	unmarshalBuf := make([]byte, 1<<10)

	ldr := db.NewKVLoader(maxPendingWrites)
	for {
		var sz uint64
		err := binary.Read(br, binary.LittleEndian, &sz)
		if err == io.EOF {
			break
		} else if err != nil {
			return err
		}

		if cap(unmarshalBuf) < int(sz) {
			unmarshalBuf = make([]byte, sz)
		}

		if _, err = io.ReadFull(br, unmarshalBuf[:sz]); err != nil {
			return err
		}

		list := &pb.KVList{}
		if err := proto.Unmarshal(unmarshalBuf[:sz], list); err != nil {
			return err
		}

		for _, kv := range list.Kv {
			if err := ldr.Set(kv); err != nil {
				return err
			}

			// Update nextTxnTs, memtable stores this
			// timestamp in badger head when flushed.
			if kv.Version >= db.orc.nextTxnTs {
				db.orc.nextTxnTs = kv.Version + 1
			}
		}
	}

	if err := ldr.Finish(); err != nil {
		return err
	}
	db.orc.txnMark.Done(db.orc.nextTxnTs - 1)
	return nil
}
