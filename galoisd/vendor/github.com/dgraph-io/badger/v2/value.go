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
	"crypto/aes"
	cryptorand "crypto/rand"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"hash"
	"hash/crc32"
	"io"
	"io/ioutil"
	"math"
	"math/rand"
	"os"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/dgraph-io/badger/v2/options"
	"github.com/dgraph-io/badger/v2/pb"
	"github.com/dgraph-io/badger/v2/y"
	"github.com/pkg/errors"
	"golang.org/x/net/trace"
)

// maxVlogFileSize is the maximum size of the vlog file which can be created. Vlog Offset is of
// uint32, so limiting at max uint32.
var maxVlogFileSize uint32 = math.MaxUint32

// Values have their first byte being byteData or byteDelete. This helps us distinguish between
// a key that has never been seen and a key that has been explicitly deleted.
const (
	bitDelete                 byte = 1 << 0 // Set if the key has been deleted.
	bitValuePointer           byte = 1 << 1 // Set if the value is NOT stored directly next to key.
	bitDiscardEarlierVersions byte = 1 << 2 // Set if earlier versions can be discarded.
	// Set if item shouldn't be discarded via compactions (used by merge operator)
	bitMergeEntry byte = 1 << 3
	// The MSB 2 bits are for transactions.
	bitTxn    byte = 1 << 6 // Set if the entry is part of a txn.
	bitFinTxn byte = 1 << 7 // Set if the entry is to indicate end of txn in value log.

	mi int64 = 1 << 20

	// The number of updates after which discard map should be flushed into badger.
	discardStatsFlushThreshold = 100

	// size of vlog header.
	// +----------------+------------------+
	// | keyID(8 bytes) |  baseIV(12 bytes)|
	// +----------------+------------------+
	vlogHeaderSize = 20
)

type logFile struct {
	path string
	// This is a lock on the log file. It guards the fd’s value, the file’s
	// existence and the file’s memory map.
	//
	// Use shared ownership when reading/writing the file or memory map, use
	// exclusive ownership to open/close the descriptor, unmap or remove the file.
	lock        sync.RWMutex
	fd          *os.File
	fid         uint32
	fmap        []byte
	size        uint32
	loadingMode options.FileLoadingMode
	dataKey     *pb.DataKey
	baseIV      []byte
	registry    *KeyRegistry
}

// encodeEntry will encode entry to the buf
// layout of entry
// +--------+-----+-------+-------+
// | header | key | value | crc32 |
// +--------+-----+-------+-------+
func (lf *logFile) encodeEntry(e *Entry, buf *bytes.Buffer, offset uint32) (int, error) {
	h := header{
		klen:      uint32(len(e.Key)),
		vlen:      uint32(len(e.Value)),
		expiresAt: e.ExpiresAt,
		meta:      e.meta,
		userMeta:  e.UserMeta,
	}

	// encode header.
	var headerEnc [maxHeaderSize]byte
	sz := h.Encode(headerEnc[:])
	y.Check2(buf.Write(headerEnc[:sz]))
	// write hash.
	hash := crc32.New(y.CastagnoliCrcTable)
	y.Check2(hash.Write(headerEnc[:sz]))
	// we'll encrypt only key and value.
	if lf.encryptionEnabled() {
		// TODO: no need to allocate the bytes. we can calculate the encrypted buf one by one
		// since we're using ctr mode of AES encryption. Ordering won't changed. Need some
		// refactoring in XORBlock which will work like stream cipher.
		eBuf := make([]byte, 0, len(e.Key)+len(e.Value))
		eBuf = append(eBuf, e.Key...)
		eBuf = append(eBuf, e.Value...)
		var err error
		eBuf, err = y.XORBlock(eBuf, lf.dataKey.Data, lf.generateIV(offset))
		if err != nil {
			return 0, y.Wrapf(err, "Error while encoding entry for vlog.")
		}
		// write encrypted buf.
		y.Check2(buf.Write(eBuf))
		// write the hash.
		y.Check2(hash.Write(eBuf))
	} else {
		// Encryption is disabled so writing directly to the buffer.
		// write key.
		y.Check2(buf.Write(e.Key))
		// write key hash.
		y.Check2(hash.Write(e.Key))
		// write value.
		y.Check2(buf.Write(e.Value))
		// write value hash.
		y.Check2(hash.Write(e.Value))
	}
	// write crc32 hash.
	var crcBuf [crc32.Size]byte
	binary.BigEndian.PutUint32(crcBuf[:], hash.Sum32())
	y.Check2(buf.Write(crcBuf[:]))
	// return encoded length.
	return len(headerEnc[:sz]) + len(e.Key) + len(e.Value) + len(crcBuf), nil
}

func (lf *logFile) decodeEntry(buf []byte, offset uint32) (*Entry, error) {
	var h header
	hlen := h.Decode(buf)
	kv := buf[hlen:]
	if lf.encryptionEnabled() {
		var err error
		// No need to worry about mmap. because, XORBlock allocates a byte array to do the
		// xor. So, the given slice is not being mutated.
		if kv, err = lf.decryptKV(kv, offset); err != nil {
			return nil, err
		}
	}
	e := &Entry{
		meta:      h.meta,
		UserMeta:  h.userMeta,
		ExpiresAt: h.expiresAt,
		offset:    offset,
		Key:       kv[:h.klen],
		Value:     kv[h.klen : h.klen+h.vlen],
	}
	return e, nil
}

func (lf *logFile) decryptKV(buf []byte, offset uint32) ([]byte, error) {
	return y.XORBlock(buf, lf.dataKey.Data, lf.generateIV(offset))
}

// KeyID returns datakey's ID.
func (lf *logFile) keyID() uint64 {
	if lf.dataKey == nil {
		// If there is no datakey, then we'll return 0. Which means no encryption.
		return 0
	}
	return lf.dataKey.KeyId
}

func (lf *logFile) mmap(size int64) (err error) {
	if lf.loadingMode != options.MemoryMap {
		// Nothing to do
		return nil
	}
	lf.fmap, err = y.Mmap(lf.fd, false, size)
	if err == nil {
		err = y.Madvise(lf.fmap, false) // Disable readahead
	}
	return err
}

func (lf *logFile) encryptionEnabled() bool {
	return lf.dataKey != nil
}

func (lf *logFile) munmap() (err error) {
	if lf.loadingMode != options.MemoryMap || len(lf.fmap) == 0 {
		// Nothing to do
		return nil
	}

	if err := y.Munmap(lf.fmap); err != nil {
		return errors.Wrapf(err, "Unable to munmap value log: %q", lf.path)
	}
	// This is important. We should set the map to nil because ummap
	// system call doesn't change the length or capacity of the fmap slice.
	lf.fmap = nil
	return nil
}

// Acquire lock on mmap/file if you are calling this
func (lf *logFile) read(p valuePointer, s *y.Slice) (buf []byte, err error) {
	var nbr int64
	offset := p.Offset
	if lf.loadingMode == options.FileIO {
		buf = s.Resize(int(p.Len))
		var n int
		n, err = lf.fd.ReadAt(buf, int64(offset))
		nbr = int64(n)
	} else {
		// Do not convert size to uint32, because the lf.fmap can be of size
		// 4GB, which overflows the uint32 during conversion to make the size 0,
		// causing the read to fail with ErrEOF. See issue #585.
		size := int64(len(lf.fmap))
		valsz := p.Len
		lfsz := atomic.LoadUint32(&lf.size)
		if int64(offset) >= size || int64(offset+valsz) > size ||
			// Ensure that the read is within the file's actual size. It might be possible that
			// the offset+valsz length is beyond the file's actual size. This could happen when
			// dropAll and iterations are running simultaneously.
			int64(offset+valsz) > int64(lfsz) {
			err = y.ErrEOF
		} else {
			buf = lf.fmap[offset : offset+valsz]
			nbr = int64(valsz)
		}
	}
	y.NumReads.Add(1)
	y.NumBytesRead.Add(nbr)
	return buf, err
}

// generateIV will generate IV by appending given offset with the base IV.
func (lf *logFile) generateIV(offset uint32) []byte {
	iv := make([]byte, aes.BlockSize)
	// baseIV is of 12 bytes.
	y.AssertTrue(12 == copy(iv[:12], lf.baseIV))
	// remaining 4 bytes is obtained from offset.
	binary.BigEndian.PutUint32(iv[12:], offset)
	return iv
}

func (lf *logFile) doneWriting(offset uint32) error {
	// Sync before acquiring lock. (We call this from write() and thus know we have shared access
	// to the fd.)
	if err := lf.fd.Sync(); err != nil {
		return errors.Wrapf(err, "Unable to sync value log: %q", lf.path)
	}

	// Before we were acquiring a lock here on lf.lock, because we were invalidating the file
	// descriptor due to reopening it as read-only. Now, we don't invalidate the fd, but unmap it,
	// truncate it and remap it. That creates a window where we have segfaults because the mmap is
	// no longer valid, while someone might be reading it. Therefore, we need a lock here again.
	lf.lock.Lock()
	defer lf.lock.Unlock()

	// Unmap file before we truncate it. Windows cannot truncate a file that is mmapped.
	if err := lf.munmap(); err != nil {
		return errors.Wrapf(err, "failed to munmap vlog file %s", lf.fd.Name())
	}

	// TODO: Confirm if we need to run a file sync after truncation.
	// Truncation must run after unmapping, otherwise Windows would crap itself.
	if err := lf.fd.Truncate(int64(offset)); err != nil {
		return errors.Wrapf(err, "Unable to truncate file: %q", lf.path)
	}

	// Reinitialize the log file. This will mmap the entire file.
	if err := lf.init(); err != nil {
		return errors.Wrapf(err, "failed to initialize file %s", lf.fd.Name())
	}

	// Previously we used to close the file after it was written and reopen it in read-only mode.
	// We no longer open files in read-only mode. We keep all vlog files open in read-write mode.
	return nil
}

// You must hold lf.lock to sync()
func (lf *logFile) sync() error {
	return lf.fd.Sync()
}

var errStop = errors.New("Stop iteration")
var errTruncate = errors.New("Do truncate")
var errDeleteVlogFile = errors.New("Delete vlog file")

type logEntry func(e Entry, vp valuePointer) error

type safeRead struct {
	k []byte
	v []byte

	recordOffset uint32
	lf           *logFile
}

// hashReader implements io.Reader, io.ByteReader interfaces. It also keeps track of the number
// bytes read. The hashReader writes to h (hash) what it reads from r.
type hashReader struct {
	r         io.Reader
	h         hash.Hash32
	bytesRead int // Number of bytes read.
}

func newHashReader(r io.Reader) *hashReader {
	hash := crc32.New(y.CastagnoliCrcTable)
	return &hashReader{
		r: r,
		h: hash,
	}
}

// Read reads len(p) bytes from the reader. Returns the number of bytes read, error on failure.
func (t *hashReader) Read(p []byte) (int, error) {
	n, err := t.r.Read(p)
	if err != nil {
		return n, err
	}
	t.bytesRead += n
	return t.h.Write(p[:n])
}

// ReadByte reads exactly one byte from the reader. Returns error on failure.
func (t *hashReader) ReadByte() (byte, error) {
	b := make([]byte, 1)
	_, err := t.Read(b)
	return b[0], err
}

// Sum32 returns the sum32 of the underlying hash.
func (t *hashReader) Sum32() uint32 {
	return t.h.Sum32()
}

// Entry reads an entry from the provided reader. It also validates the checksum for every entry
// read. Returns error on failure.
func (r *safeRead) Entry(reader io.Reader) (*Entry, error) {
	tee := newHashReader(reader)
	var h header
	hlen, err := h.DecodeFrom(tee)
	if err != nil {
		return nil, err
	}
	if h.klen > uint32(1<<16) { // Key length must be below uint16.
		return nil, errTruncate
	}
	kl := int(h.klen)
	if cap(r.k) < kl {
		r.k = make([]byte, 2*kl)
	}
	vl := int(h.vlen)
	if cap(r.v) < vl {
		r.v = make([]byte, 2*vl)
	}

	e := &Entry{}
	e.offset = r.recordOffset
	e.hlen = hlen
	buf := make([]byte, h.klen+h.vlen)
	if _, err := io.ReadFull(tee, buf[:]); err != nil {
		if err == io.EOF {
			err = errTruncate
		}
		return nil, err
	}
	if r.lf.encryptionEnabled() {
		if buf, err = r.lf.decryptKV(buf[:], r.recordOffset); err != nil {
			return nil, err
		}
	}
	e.Key = buf[:h.klen]
	e.Value = buf[h.klen:]
	var crcBuf [crc32.Size]byte
	if _, err := io.ReadFull(reader, crcBuf[:]); err != nil {
		if err == io.EOF {
			err = errTruncate
		}
		return nil, err
	}
	crc := y.BytesToU32(crcBuf[:])
	if crc != tee.Sum32() {
		return nil, errTruncate
	}
	e.meta = h.meta
	e.UserMeta = h.userMeta
	e.ExpiresAt = h.expiresAt
	return e, nil
}

// iterate iterates over log file. It doesn't not allocate new memory for every kv pair.
// Therefore, the kv pair is only valid for the duration of fn call.
func (vlog *valueLog) iterate(lf *logFile, offset uint32, fn logEntry) (uint32, error) {
	fi, err := lf.fd.Stat()
	if err != nil {
		return 0, err
	}
	if offset == 0 {
		// If offset is set to zero, let's advance past the encryption key header.
		offset = vlogHeaderSize
	}
	if int64(offset) == fi.Size() {
		// We're at the end of the file already. No need to do anything.
		return offset, nil
	}
	if vlog.opt.ReadOnly {
		// We're not at the end of the file. We'd need to replay the entries, or
		// possibly truncate the file.
		return 0, ErrReplayNeeded
	}

	// We're not at the end of the file. Let's Seek to the offset and start reading.
	if _, err := lf.fd.Seek(int64(offset), io.SeekStart); err != nil {
		return 0, errFile(err, lf.path, "Unable to seek")
	}

	reader := bufio.NewReader(lf.fd)
	read := &safeRead{
		k:            make([]byte, 10),
		v:            make([]byte, 10),
		recordOffset: offset,
		lf:           lf,
	}

	var lastCommit uint64
	var validEndOffset uint32 = offset

loop:
	for {
		e, err := read.Entry(reader)
		switch {
		case err == io.EOF:
			break loop
		case err == io.ErrUnexpectedEOF || err == errTruncate:
			break loop
		case err != nil:
			return 0, err
		case e == nil:
			continue
		}

		var vp valuePointer
		vp.Len = uint32(int(e.hlen) + len(e.Key) + len(e.Value) + crc32.Size)
		read.recordOffset += vp.Len

		vp.Offset = e.offset
		vp.Fid = lf.fid

		switch {
		case e.meta&bitTxn > 0:
			txnTs := y.ParseTs(e.Key)
			if lastCommit == 0 {
				lastCommit = txnTs
			}
			if lastCommit != txnTs {
				break loop
			}

		case e.meta&bitFinTxn > 0:
			txnTs, err := strconv.ParseUint(string(e.Value), 10, 64)
			if err != nil || lastCommit != txnTs {
				break loop
			}
			// Got the end of txn. Now we can store them.
			lastCommit = 0
			validEndOffset = read.recordOffset

		default:
			if lastCommit != 0 {
				// This is most likely an entry which was moved as part of GC.
				// We shouldn't get this entry in the middle of a transaction.
				break loop
			}
			validEndOffset = read.recordOffset
		}

		if err := fn(*e, vp); err != nil {
			if err == errStop {
				break
			}
			return 0, errFile(err, lf.path, "Iteration function")
		}
	}
	return validEndOffset, nil
}

func (vlog *valueLog) rewrite(f *logFile, tr trace.Trace) error {
	vlog.filesLock.RLock()
	maxFid := vlog.maxFid
	vlog.filesLock.RUnlock()
	y.AssertTruef(uint32(f.fid) < maxFid, "fid to move: %d. Current max fid: %d", f.fid, maxFid)
	tr.LazyPrintf("Rewriting fid: %d", f.fid)

	wb := make([]*Entry, 0, 1000)
	var size int64

	y.AssertTrue(vlog.db != nil)
	var count, moved int
	fe := func(e Entry) error {
		count++
		if count%100000 == 0 {
			tr.LazyPrintf("Processing entry %d", count)
		}

		vs, err := vlog.db.get(e.Key)
		if err != nil {
			return err
		}
		if discardEntry(e, vs, vlog.db) {
			return nil
		}

		// Value is still present in value log.
		if len(vs.Value) == 0 {
			return errors.Errorf("Empty value: %+v", vs)
		}
		var vp valuePointer
		vp.Decode(vs.Value)

		// If the entry found from the LSM Tree points to a newer vlog file, don't do anything.
		if vp.Fid > f.fid {
			return nil
		}
		// If the entry found from the LSM Tree points to an offset greater than the one
		// read from vlog, don't do anything.
		if vp.Offset > e.offset {
			return nil
		}
		// If the entry read from LSM Tree and vlog file point to the same vlog file and offset,
		// insert them back into the DB.
		// NOTE: It might be possible that the entry read from the LSM Tree points to
		// an older vlog file. See the comments in the else part.
		if vp.Fid == f.fid && vp.Offset == e.offset {
			moved++
			// This new entry only contains the key, and a pointer to the value.
			ne := new(Entry)
			ne.meta = 0 // Remove all bits. Different keyspace doesn't need these bits.
			ne.UserMeta = e.UserMeta
			ne.ExpiresAt = e.ExpiresAt

			// Create a new key in a separate keyspace, prefixed by moveKey. We are not
			// allowed to rewrite an older version of key in the LSM tree, because then this older
			// version would be at the top of the LSM tree. To work correctly, reads expect the
			// latest versions to be at the top, and the older versions at the bottom.
			if bytes.HasPrefix(e.Key, badgerMove) {
				ne.Key = append([]byte{}, e.Key...)
			} else {
				ne.Key = make([]byte, len(badgerMove)+len(e.Key))
				n := copy(ne.Key, badgerMove)
				copy(ne.Key[n:], e.Key)
			}

			ne.Value = append([]byte{}, e.Value...)
			es := int64(ne.estimateSize(vlog.opt.ValueThreshold))
			// Consider size of value as well while considering the total size
			// of the batch. There have been reports of high memory usage in
			// rewrite because we don't consider the value size. See #1292.
			es += int64(len(e.Value))

			// Ensure length and size of wb is within transaction limits.
			if int64(len(wb)+1) >= vlog.opt.maxBatchCount ||
				size+es >= vlog.opt.maxBatchSize {
				tr.LazyPrintf("request has %d entries, size %d", len(wb), size)
				if err := vlog.db.batchSet(wb); err != nil {
					return err
				}
				size = 0
				wb = wb[:0]
			}
			wb = append(wb, ne)
			size += es
		} else {
			// It might be possible that the entry read from LSM Tree points to an older vlog file.
			// This can happen in the following situation. Assume DB is opened with
			// numberOfVersionsToKeep=1
			//
			// Now, if we have ONLY one key in the system "FOO" which has been updated 3 times and
			// the same key has been garbage collected 3 times, we'll have 3 versions of the movekey
			// for the same key "FOO".
			// NOTE: moveKeyi is the moveKey with version i
			// Assume we have 3 move keys in L0.
			// - moveKey1 (points to vlog file 10),
			// - moveKey2 (points to vlog file 14) and
			// - moveKey3 (points to vlog file 15).

			// Also, assume there is another move key "moveKey1" (points to vlog file 6) (this is
			// also a move Key for key "FOO" ) on upper levels (let's say 3). The move key
			//  "moveKey1" on level 0 was inserted because vlog file 6 was GCed.
			//
			// Here's what the arrangement looks like
			// L0 => (moveKey1 => vlog10), (moveKey2 => vlog14), (moveKey3 => vlog15)
			// L1 => ....
			// L2 => ....
			// L3 => (moveKey1 => vlog6)
			//
			// When L0 compaction runs, it keeps only moveKey3 because the number of versions
			// to keep is set to 1. (we've dropped moveKey1's latest version)
			//
			// The new arrangement of keys is
			// L0 => ....
			// L1 => (moveKey3 => vlog15)
			// L2 => ....
			// L3 => (moveKey1 => vlog6)
			//
			// Now if we try to GC vlog file 10, the entry read from vlog file will point to vlog10
			// but the entry read from LSM Tree will point to vlog6. The move key read from LSM tree
			// will point to vlog6 because we've asked for version 1 of the move key.
			//
			// This might seem like an issue but it's not really an issue because the user has set
			// the number of versions to keep to 1 and the latest version of moveKey points to the
			// correct vlog file and offset. The stale move key on L3 will be eventually dropped by
			// compaction because there is a newer versions in the upper levels.
		}
		return nil
	}

	_, err := vlog.iterate(f, 0, func(e Entry, vp valuePointer) error {
		return fe(e)
	})
	if err != nil {
		return err
	}

	tr.LazyPrintf("request has %d entries, size %d", len(wb), size)
	batchSize := 1024
	var loops int
	for i := 0; i < len(wb); {
		loops++
		if batchSize == 0 {
			vlog.db.opt.Warningf("We shouldn't reach batch size of zero.")
			return ErrNoRewrite
		}
		end := i + batchSize
		if end > len(wb) {
			end = len(wb)
		}
		if err := vlog.db.batchSet(wb[i:end]); err != nil {
			if err == ErrTxnTooBig {
				// Decrease the batch size to half.
				batchSize = batchSize / 2
				tr.LazyPrintf("Dropped batch size to %d", batchSize)
				continue
			}
			return err
		}
		i += batchSize
	}
	tr.LazyPrintf("Processed %d entries in %d loops", len(wb), loops)
	tr.LazyPrintf("Total entries: %d. Moved: %d", count, moved)
	tr.LazyPrintf("Removing fid: %d", f.fid)
	var deleteFileNow bool
	// Entries written to LSM. Remove the older file now.
	{
		vlog.filesLock.Lock()
		// Just a sanity-check.
		if _, ok := vlog.filesMap[f.fid]; !ok {
			vlog.filesLock.Unlock()
			return errors.Errorf("Unable to find fid: %d", f.fid)
		}
		if vlog.iteratorCount() == 0 {
			delete(vlog.filesMap, f.fid)
			deleteFileNow = true
		} else {
			vlog.filesToBeDeleted = append(vlog.filesToBeDeleted, f.fid)
		}
		vlog.filesLock.Unlock()
	}

	if deleteFileNow {
		if err := vlog.deleteLogFile(f); err != nil {
			return err
		}
	}

	return nil
}

func (vlog *valueLog) deleteMoveKeysFor(fid uint32, tr trace.Trace) error {
	db := vlog.db
	var result []*Entry
	var count, pointers uint64
	tr.LazyPrintf("Iterating over move keys to find invalids for fid: %d", fid)
	err := db.View(func(txn *Txn) error {
		opt := DefaultIteratorOptions
		opt.InternalAccess = true
		opt.PrefetchValues = false
		itr := txn.NewIterator(opt)
		defer itr.Close()

		for itr.Seek(badgerMove); itr.ValidForPrefix(badgerMove); itr.Next() {
			count++
			item := itr.Item()
			if item.meta&bitValuePointer == 0 {
				continue
			}
			pointers++
			var vp valuePointer
			vp.Decode(item.vptr)
			if vp.Fid == fid {
				e := &Entry{Key: y.KeyWithTs(item.Key(), item.Version()), meta: bitDelete}
				result = append(result, e)
			}
		}
		return nil
	})
	if err != nil {
		tr.LazyPrintf("Got error while iterating move keys: %v", err)
		tr.SetError()
		return err
	}
	tr.LazyPrintf("Num total move keys: %d. Num pointers: %d", count, pointers)
	tr.LazyPrintf("Number of invalid move keys found: %d", len(result))
	batchSize := 10240
	for i := 0; i < len(result); {
		end := i + batchSize
		if end > len(result) {
			end = len(result)
		}
		if err := db.batchSet(result[i:end]); err != nil {
			if err == ErrTxnTooBig {
				batchSize /= 2
				tr.LazyPrintf("Dropped batch size to %d", batchSize)
				continue
			}
			tr.LazyPrintf("Error while doing batchSet: %v", err)
			tr.SetError()
			return err
		}
		i += batchSize
	}
	tr.LazyPrintf("Move keys deletion done.")
	return nil
}

func (vlog *valueLog) incrIteratorCount() {
	atomic.AddInt32(&vlog.numActiveIterators, 1)
}

func (vlog *valueLog) iteratorCount() int {
	return int(atomic.LoadInt32(&vlog.numActiveIterators))
}

func (vlog *valueLog) decrIteratorCount() error {
	num := atomic.AddInt32(&vlog.numActiveIterators, -1)
	if num != 0 {
		return nil
	}

	vlog.filesLock.Lock()
	lfs := make([]*logFile, 0, len(vlog.filesToBeDeleted))
	for _, id := range vlog.filesToBeDeleted {
		lfs = append(lfs, vlog.filesMap[id])
		delete(vlog.filesMap, id)
	}
	vlog.filesToBeDeleted = nil
	vlog.filesLock.Unlock()

	for _, lf := range lfs {
		if err := vlog.deleteLogFile(lf); err != nil {
			return err
		}
	}
	return nil
}

func (vlog *valueLog) deleteLogFile(lf *logFile) error {
	if lf == nil {
		return nil
	}
	lf.lock.Lock()
	defer lf.lock.Unlock()

	path := vlog.fpath(lf.fid)
	if err := lf.munmap(); err != nil {
		_ = lf.fd.Close()
		return err
	}
	lf.fmap = nil
	if err := lf.fd.Close(); err != nil {
		return err
	}
	return os.Remove(path)
}

func (vlog *valueLog) dropAll() (int, error) {
	// If db is opened in InMemory mode, we don't need to do anything since there are no vlog files.
	if vlog.db.opt.InMemory {
		return 0, nil
	}
	// We don't want to block dropAll on any pending transactions. So, don't worry about iterator
	// count.
	var count int
	deleteAll := func() error {
		vlog.filesLock.Lock()
		defer vlog.filesLock.Unlock()
		for _, lf := range vlog.filesMap {
			if err := vlog.deleteLogFile(lf); err != nil {
				return err
			}
			count++
		}
		vlog.filesMap = make(map[uint32]*logFile)
		return nil
	}
	if err := deleteAll(); err != nil {
		return count, err
	}

	vlog.db.opt.Infof("Value logs deleted. Creating value log file: 0")
	if _, err := vlog.createVlogFile(0); err != nil { // Called while writes are stopped.
		return count, err
	}
	return count, nil
}

// lfDiscardStats keeps track of the amount of data that could be discarded for
// a given logfile.
type lfDiscardStats struct {
	sync.RWMutex
	m                 map[uint32]int64
	flushChan         chan map[uint32]int64
	closer            *y.Closer
	updatesSinceFlush int
}

type valueLog struct {
	dirPath string

	// guards our view of which files exist, which to be deleted, how many active iterators
	filesLock        sync.RWMutex
	filesMap         map[uint32]*logFile
	maxFid           uint32
	filesToBeDeleted []uint32
	// A refcount of iterators -- when this hits zero, we can delete the filesToBeDeleted.
	numActiveIterators int32

	db                *DB
	writableLogOffset uint32 // read by read, written by write. Must access via atomics.
	numEntriesWritten uint32
	opt               Options

	garbageCh      chan struct{}
	lfDiscardStats *lfDiscardStats
}

func vlogFilePath(dirPath string, fid uint32) string {
	return fmt.Sprintf("%s%s%06d.vlog", dirPath, string(os.PathSeparator), fid)
}

func (vlog *valueLog) fpath(fid uint32) string {
	return vlogFilePath(vlog.dirPath, fid)
}

func (vlog *valueLog) populateFilesMap() error {
	vlog.filesMap = make(map[uint32]*logFile)

	files, err := ioutil.ReadDir(vlog.dirPath)
	if err != nil {
		return errFile(err, vlog.dirPath, "Unable to open log dir.")
	}

	found := make(map[uint64]struct{})
	for _, file := range files {
		if !strings.HasSuffix(file.Name(), ".vlog") {
			continue
		}
		fsz := len(file.Name())
		fid, err := strconv.ParseUint(file.Name()[:fsz-5], 10, 32)
		if err != nil {
			return errFile(err, file.Name(), "Unable to parse log id.")
		}
		if _, ok := found[fid]; ok {
			return errFile(err, file.Name(), "Duplicate file found. Please delete one.")
		}
		found[fid] = struct{}{}

		lf := &logFile{
			fid:         uint32(fid),
			path:        vlog.fpath(uint32(fid)),
			loadingMode: vlog.opt.ValueLogLoadingMode,
			registry:    vlog.db.registry,
		}
		vlog.filesMap[uint32(fid)] = lf
		if vlog.maxFid < uint32(fid) {
			vlog.maxFid = uint32(fid)
		}
	}
	return nil
}

func (lf *logFile) open(path string, flags uint32) error {
	var err error
	if lf.fd, err = y.OpenExistingFile(path, flags); err != nil {
		return y.Wrapf(err, "Error while opening file in logfile %s", path)
	}

	fi, err := lf.fd.Stat()
	if err != nil {
		return errFile(err, lf.path, "Unable to run file.Stat")
	}
	sz := fi.Size()
	y.AssertTruef(
		sz <= math.MaxUint32,
		"file size: %d greater than %d",
		uint32(sz), uint32(math.MaxUint32),
	)
	lf.size = uint32(sz)
	if sz < vlogHeaderSize {
		// Every vlog file should have at least vlogHeaderSize. If it is less than vlogHeaderSize
		// then it must have been corrupted. But no need to handle here. log replayer will truncate
		// and bootstrap the logfile. So ignoring here.
		return nil
	}
	buf := make([]byte, vlogHeaderSize)
	if _, err = lf.fd.Read(buf); err != nil {
		return y.Wrapf(err, "Error while reading vlog file %d", lf.fid)
	}
	keyID := binary.BigEndian.Uint64(buf[:8])
	var dk *pb.DataKey
	// retrieve datakey.
	if dk, err = lf.registry.dataKey(keyID); err != nil {
		return y.Wrapf(err, "While opening vlog file %d", lf.fid)
	}
	lf.dataKey = dk
	lf.baseIV = buf[8:]
	y.AssertTrue(len(lf.baseIV) == 12)
	return nil
}

// bootstrap will initialize the log file with key id and baseIV.
// The below figure shows the layout of log file.
// +----------------+------------------+------------------+
// | keyID(8 bytes) |  baseIV(12 bytes)|	 entry...     |
// +----------------+------------------+------------------+
func (lf *logFile) bootstrap() error {
	var err error
	// delete all the data. because bootstrap is been called while creating vlog and as well
	// as replaying log. While replaying log, there may be any data left. So we need to truncate
	// everything.
	if err = lf.fd.Truncate(0); err != nil {
		return y.Wrapf(err, "Error while bootstraping.")
	}

	if _, err = lf.fd.Seek(0, io.SeekStart); err != nil {
		return y.Wrapf(err, "Error while SeekStart for the logfile %d in logFile.bootstarp", lf.fid)
	}
	// generate data key for the log file.
	var dk *pb.DataKey
	if dk, err = lf.registry.latestDataKey(); err != nil {
		return y.Wrapf(err, "Error while retrieving datakey in logFile.bootstarp")
	}
	lf.dataKey = dk
	// We'll always preserve vlogHeaderSize for key id and baseIV.
	buf := make([]byte, vlogHeaderSize)
	// write key id to the buf.
	// key id will be zero if the logfile is in plain text.
	binary.BigEndian.PutUint64(buf[:8], lf.keyID())
	// generate base IV. It'll be used with offset of the vptr to encrypt the entry.
	if _, err := cryptorand.Read(buf[8:]); err != nil {
		return y.Wrapf(err, "Error while creating base IV, while creating logfile")
	}
	// Initialize base IV.
	lf.baseIV = buf[8:]
	y.AssertTrue(len(lf.baseIV) == 12)
	// write the key id and base IV to the file.
	_, err = lf.fd.Write(buf)
	return err
}

func (vlog *valueLog) createVlogFile(fid uint32) (*logFile, error) {
	path := vlog.fpath(fid)

	lf := &logFile{
		fid:         fid,
		path:        path,
		loadingMode: vlog.opt.ValueLogLoadingMode,
		registry:    vlog.db.registry,
	}
	// writableLogOffset is only written by write func, by read by Read func.
	// To avoid a race condition, all reads and updates to this variable must be
	// done via atomics.
	var err error
	if lf.fd, err = y.CreateSyncedFile(path, vlog.opt.SyncWrites); err != nil {
		return nil, errFile(err, lf.path, "Create value log file")
	}

	removeFile := func() {
		// Remove the file so that we don't get an error when createVlogFile is
		// called for the same fid, again. This could happen if there is an
		// transient error because of which we couldn't create a new file
		// and the second attempt to create the file succeeds.
		y.Check(os.Remove(lf.fd.Name()))
	}

	if err = lf.bootstrap(); err != nil {
		removeFile()
		return nil, err
	}

	if err = syncDir(vlog.dirPath); err != nil {
		removeFile()
		return nil, errFile(err, vlog.dirPath, "Sync value log dir")
	}

	if err = lf.mmap(2 * vlog.opt.ValueLogFileSize); err != nil {
		removeFile()
		return nil, errFile(err, lf.path, "Mmap value log file")
	}

	vlog.filesLock.Lock()
	vlog.filesMap[fid] = lf
	vlog.maxFid = fid
	// writableLogOffset is only written by write func, by read by Read func.
	// To avoid a race condition, all reads and updates to this variable must be
	// done via atomics.
	atomic.StoreUint32(&vlog.writableLogOffset, vlogHeaderSize)
	vlog.numEntriesWritten = 0
	vlog.filesLock.Unlock()

	return lf, nil
}

func errFile(err error, path string, msg string) error {
	return fmt.Errorf("%s. Path=%s. Error=%v", msg, path, err)
}

func (vlog *valueLog) replayLog(lf *logFile, offset uint32, replayFn logEntry) error {
	fi, err := lf.fd.Stat()
	if err != nil {
		return errFile(err, lf.path, "Unable to run file.Stat")
	}

	// Alright, let's iterate now.
	endOffset, err := vlog.iterate(lf, offset, replayFn)
	if err != nil {
		return errFile(err, lf.path, "Unable to replay logfile")
	}
	if int64(endOffset) == fi.Size() {
		return nil
	}

	// End offset is different from file size. So, we should truncate the file
	// to that size.
	if !vlog.opt.Truncate {
		vlog.db.opt.Warningf("Truncate Needed. File %s size: %d Endoffset: %d",
			lf.fd.Name(), fi.Size(), endOffset)
		return ErrTruncateNeeded
	}

	// The entire file should be truncated (i.e. it should be deleted).
	// If fid == maxFid then it's okay to truncate the entire file since it will be
	// used for future additions. Also, it's okay if the last file has size zero.
	// We mmap 2*opt.ValueLogSize for the last file. See vlog.Open() function
	// if endOffset <= vlogHeaderSize && lf.fid != vlog.maxFid {

	if endOffset <= vlogHeaderSize {
		if lf.fid != vlog.maxFid {
			return errDeleteVlogFile
		}
		return lf.bootstrap()
	}

	vlog.db.opt.Infof("Truncating vlog file %s to offset: %d", lf.fd.Name(), endOffset)
	if err := lf.fd.Truncate(int64(endOffset)); err != nil {
		return errFile(err, lf.path, fmt.Sprintf(
			"Truncation needed at offset %d. Can be done manually as well.", endOffset))
	}
	return nil
}

// init initializes the value log struct. This initialization needs to happen
// before compactions start.
func (vlog *valueLog) init(db *DB) {
	vlog.opt = db.opt
	vlog.db = db
	// We don't need to open any vlog files or collect stats for GC if DB is opened
	// in InMemory mode. InMemory mode doesn't create any files/directories on disk.
	if vlog.opt.InMemory {
		return
	}
	vlog.dirPath = vlog.opt.ValueDir

	vlog.garbageCh = make(chan struct{}, 1) // Only allow one GC at a time.
	vlog.lfDiscardStats = &lfDiscardStats{
		m:         make(map[uint32]int64),
		closer:    y.NewCloser(1),
		flushChan: make(chan map[uint32]int64, 16),
	}
}

func (vlog *valueLog) open(db *DB, ptr valuePointer, replayFn logEntry) error {
	// We don't need to open any vlog files or collect stats for GC if DB is opened
	// in InMemory mode. InMemory mode doesn't create any files/directories on disk.
	if db.opt.InMemory {
		return nil
	}

	go vlog.flushDiscardStats()
	if err := vlog.populateFilesMap(); err != nil {
		return err
	}
	// If no files are found, then create a new file.
	if len(vlog.filesMap) == 0 {
		_, err := vlog.createVlogFile(0)
		return y.Wrapf(err, "Error while creating log file in valueLog.open")
	}
	fids := vlog.sortedFids()
	for _, fid := range fids {
		lf, ok := vlog.filesMap[fid]
		y.AssertTrue(ok)
		var flags uint32
		switch {
		case vlog.opt.ReadOnly:
			// If we have read only, we don't need SyncWrites.
			flags |= y.ReadOnly
			// Set sync flag.
		case vlog.opt.SyncWrites:
			flags |= y.Sync
		}

		// We cannot mmap the files upfront here. Windows does not like mmapped files to be
		// truncated. We might need to truncate files during a replay.
		var err error
		if err = lf.open(vlog.fpath(fid), flags); err != nil {
			return errors.Wrapf(err, "Open existing file: %q", lf.path)
		}

		// This file is before the value head pointer. So, we don't need to
		// replay it, and can just open it in readonly mode.
		if fid < ptr.Fid {
			// Mmap the file here, we don't need to replay it.
			if err := lf.init(); err != nil {
				return err
			}
			continue
		}

		var offset uint32
		if fid == ptr.Fid {
			offset = ptr.Offset + ptr.Len
		}
		vlog.db.opt.Infof("Replaying file id: %d at offset: %d\n", fid, offset)
		now := time.Now()
		// Replay and possible truncation done. Now we can open the file as per
		// user specified options.
		if err := vlog.replayLog(lf, offset, replayFn); err != nil {
			// Log file is corrupted. Delete it.
			if err == errDeleteVlogFile {
				delete(vlog.filesMap, fid)
				// Close the fd of the file before deleting the file otherwise windows complaints.
				if err := lf.fd.Close(); err != nil {
					return errors.Wrapf(err, "failed to close vlog file %s", lf.fd.Name())
				}
				path := vlog.fpath(lf.fid)
				if err := os.Remove(path); err != nil {
					return y.Wrapf(err, "failed to delete empty value log file: %q", path)
				}
				continue
			}
			return err
		}
		vlog.db.opt.Infof("Replay took: %s\n", time.Since(now))

		if fid < vlog.maxFid {
			// This file has been replayed. It can now be mmapped.
			// For maxFid, the mmap would be done by the specially written code below.
			if err := lf.init(); err != nil {
				return err
			}
		}
	}
	// Seek to the end to start writing.
	last, ok := vlog.filesMap[vlog.maxFid]
	y.AssertTrue(ok)
	// We'll create a new vlog if the last vlog is encrypted and db is opened in
	// plain text mode or vice versa. A single vlog file can't have both
	// encrypted entries and plain text entries.
	if last.encryptionEnabled() != vlog.db.shouldEncrypt() {
		newid := vlog.maxFid + 1
		_, err := vlog.createVlogFile(newid)
		if err != nil {
			return y.Wrapf(err, "Error while creating log file %d in valueLog.open", newid)
		}
		last, ok = vlog.filesMap[newid]
		y.AssertTrue(ok)
	}
	lastOffset, err := last.fd.Seek(0, io.SeekEnd)
	if err != nil {
		return errFile(err, last.path, "file.Seek to end")
	}
	vlog.writableLogOffset = uint32(lastOffset)

	// Update the head to point to the updated tail. Otherwise, even after doing a successful
	// replay and closing the DB, the value log head does not get updated, which causes the replay
	// to happen repeatedly.
	vlog.db.vhead = valuePointer{Fid: vlog.maxFid, Offset: uint32(lastOffset)}

	// Map the file if needed. When we create a file, it is automatically mapped.
	if err = last.mmap(2 * vlog.opt.ValueLogFileSize); err != nil {
		return errFile(err, last.path, "Map log file")
	}
	if err := vlog.populateDiscardStats(); err != nil {
		// Print the error and continue. We don't want to prevent value log open if there's an error
		// with the fetching discards stats.
		db.opt.Errorf("Failed to populate discard stats: %s", err)
	}
	return nil
}

func (lf *logFile) init() error {
	fstat, err := lf.fd.Stat()
	if err != nil {
		return errors.Wrapf(err, "Unable to check stat for %q", lf.path)
	}
	sz := fstat.Size()
	if sz == 0 {
		// File is empty. We don't need to mmap it. Return.
		return nil
	}
	y.AssertTrue(sz <= math.MaxUint32)
	lf.size = uint32(sz)
	if err = lf.mmap(sz); err != nil {
		_ = lf.fd.Close()
		return errors.Wrapf(err, "Unable to map file: %q", fstat.Name())
	}
	return nil
}

func (vlog *valueLog) stopFlushDiscardStats() {
	if vlog.lfDiscardStats != nil {
		vlog.lfDiscardStats.closer.Signal()
	}
}

func (vlog *valueLog) Close() error {
	if vlog == nil || vlog.db == nil || vlog.db.opt.InMemory {
		return nil
	}
	// close flushDiscardStats.
	vlog.lfDiscardStats.closer.SignalAndWait()

	vlog.opt.Debugf("Stopping garbage collection of values.")

	var err error
	for id, f := range vlog.filesMap {
		f.lock.Lock() // We won’t release the lock.
		if munmapErr := f.munmap(); munmapErr != nil && err == nil {
			err = munmapErr
		}

		maxFid := vlog.maxFid
		// TODO(ibrahim) - Do we need the following truncations on non-windows
		// platforms? We expand the file only on windows and the vlog.woffset()
		// should point to end of file on all other platforms.
		if !vlog.opt.ReadOnly && id == maxFid {
			// truncate writable log file to correct offset.
			if truncErr := f.fd.Truncate(
				int64(vlog.woffset())); truncErr != nil && err == nil {
				err = truncErr
			}
		}

		if closeErr := f.fd.Close(); closeErr != nil && err == nil {
			err = closeErr
		}
	}
	return err
}

// sortedFids returns the file id's not pending deletion, sorted.  Assumes we have shared access to
// filesMap.
func (vlog *valueLog) sortedFids() []uint32 {
	toBeDeleted := make(map[uint32]struct{})
	for _, fid := range vlog.filesToBeDeleted {
		toBeDeleted[fid] = struct{}{}
	}
	ret := make([]uint32, 0, len(vlog.filesMap))
	for fid := range vlog.filesMap {
		if _, ok := toBeDeleted[fid]; !ok {
			ret = append(ret, fid)
		}
	}
	sort.Slice(ret, func(i, j int) bool {
		return ret[i] < ret[j]
	})
	return ret
}

type request struct {
	// Input values
	Entries []*Entry
	// Output values and wait group stuff below
	Ptrs []valuePointer
	Wg   sync.WaitGroup
	Err  error
	ref  int32
}

func (req *request) reset() {
	req.Entries = req.Entries[:0]
	req.Ptrs = req.Ptrs[:0]
	req.Wg = sync.WaitGroup{}
	req.Err = nil
	req.ref = 0
}

func (req *request) IncrRef() {
	atomic.AddInt32(&req.ref, 1)
}

func (req *request) DecrRef() {
	nRef := atomic.AddInt32(&req.ref, -1)
	if nRef > 0 {
		return
	}
	req.Entries = nil
	requestPool.Put(req)
}

func (req *request) Wait() error {
	req.Wg.Wait()
	err := req.Err
	req.DecrRef() // DecrRef after writing to DB.
	return err
}

type requests []*request

func (reqs requests) DecrRef() {
	for _, req := range reqs {
		req.DecrRef()
	}
}

func (reqs requests) IncrRef() {
	for _, req := range reqs {
		req.IncrRef()
	}
}

// sync function syncs content of latest value log file to disk. Syncing of value log directory is
// not required here as it happens every time a value log file rotation happens(check createVlogFile
// function). During rotation, previous value log file also gets synced to disk. It only syncs file
// if fid >= vlog.maxFid. In some cases such as replay(while opening db), it might be called with
// fid < vlog.maxFid. To sync irrespective of file id just call it with math.MaxUint32.
func (vlog *valueLog) sync(fid uint32) error {
	if vlog.opt.SyncWrites || vlog.opt.InMemory {
		return nil
	}

	vlog.filesLock.RLock()
	maxFid := vlog.maxFid
	// During replay it is possible to get sync call with fid less than maxFid.
	// Because older file has already been synced, we can return from here.
	if fid < maxFid || len(vlog.filesMap) == 0 {
		vlog.filesLock.RUnlock()
		return nil
	}
	curlf := vlog.filesMap[maxFid]
	// Sometimes it is possible that vlog.maxFid has been increased but file creation
	// with same id is still in progress and this function is called. In those cases
	// entry for the file might not be present in vlog.filesMap.
	if curlf == nil {
		vlog.filesLock.RUnlock()
		return nil
	}
	curlf.lock.RLock()
	vlog.filesLock.RUnlock()

	err := curlf.sync()
	curlf.lock.RUnlock()
	return err
}

func (vlog *valueLog) woffset() uint32 {
	return atomic.LoadUint32(&vlog.writableLogOffset)
}

// validateWrites will check whether the given requests can fit into 4GB vlog file.
// NOTE: 4GB is the maximum size we can create for vlog because value pointer offset is of type
// uint32. If we create more than 4GB, it will overflow uint32. So, limiting the size to 4GB.
func (vlog *valueLog) validateWrites(reqs []*request) error {
	vlogOffset := uint64(vlog.woffset())
	for _, req := range reqs {
		// calculate size of the request.
		size := estimateRequestSize(req)
		estimatedVlogOffset := vlogOffset + size
		if estimatedVlogOffset > uint64(maxVlogFileSize) {
			return errors.Errorf("Request size offset %d is bigger than maximum offset %d",
				estimatedVlogOffset, maxVlogFileSize)
		}

		if estimatedVlogOffset >= uint64(vlog.opt.ValueLogFileSize) {
			// We'll create a new vlog file if the estimated offset is greater or equal to
			// max vlog size. So, resetting the vlogOffset.
			vlogOffset = 0
			continue
		}
		// Estimated vlog offset will become current vlog offset if the vlog is not rotated.
		vlogOffset = estimatedVlogOffset
	}
	return nil
}

// estimateRequestSize returns the size that needed to be written for the given request.
func estimateRequestSize(req *request) uint64 {
	size := uint64(0)
	for _, e := range req.Entries {
		size += uint64(maxHeaderSize + len(e.Key) + len(e.Value) + crc32.Size)
	}
	return size
}

// write is thread-unsafe by design and should not be called concurrently.
func (vlog *valueLog) write(reqs []*request) error {
	if vlog.db.opt.InMemory {
		return nil
	}
	// Validate writes before writing to vlog. Because, we don't want to partially write and return
	// an error.
	if err := vlog.validateWrites(reqs); err != nil {
		return err
	}

	vlog.filesLock.RLock()
	maxFid := vlog.maxFid
	curlf, ok := vlog.filesMap[maxFid]
	if !ok {
		var fids []uint32
		for fid := range vlog.filesMap {
			fids = append(fids, fid)
		}
		return errors.Errorf("Cannot find MaxFid: %d in filesMap: %+v", maxFid, fids)
	}
	vlog.filesLock.RUnlock()

	var buf bytes.Buffer
	flushWrites := func() error {
		if buf.Len() == 0 {
			return nil
		}
		vlog.opt.Debugf("Flushing buffer of size %d to vlog", buf.Len())
		n, err := curlf.fd.Write(buf.Bytes())
		if err != nil {
			return errors.Wrapf(err, "Unable to write to value log file: %q", curlf.path)
		}
		buf.Reset()
		y.NumWrites.Add(1)
		y.NumBytesWritten.Add(int64(n))
		vlog.opt.Debugf("Done")
		atomic.AddUint32(&vlog.writableLogOffset, uint32(n))
		atomic.StoreUint32(&curlf.size, vlog.writableLogOffset)
		return nil
	}
	toDisk := func() error {
		if err := flushWrites(); err != nil {
			return err
		}
		if vlog.woffset() > uint32(vlog.opt.ValueLogFileSize) ||
			vlog.numEntriesWritten > vlog.opt.ValueLogMaxEntries {
			if err := curlf.doneWriting(vlog.woffset()); err != nil {
				return err
			}

			newid := vlog.maxFid + 1
			y.AssertTruef(newid > 0, "newid has overflown uint32: %v", newid)
			newlf, err := vlog.createVlogFile(newid)
			if err != nil {
				return err
			}
			curlf = newlf
			atomic.AddInt32(&vlog.db.logRotates, 1)
		}
		return nil
	}
	for i := range reqs {
		b := reqs[i]
		b.Ptrs = b.Ptrs[:0]
		var written int
		for j := range b.Entries {
			e := b.Entries[j]
			if e.skipVlog {
				b.Ptrs = append(b.Ptrs, valuePointer{})
				continue
			}
			var p valuePointer

			p.Fid = curlf.fid
			// Use the offset including buffer length so far.
			p.Offset = vlog.woffset() + uint32(buf.Len())
			plen, err := curlf.encodeEntry(e, &buf, p.Offset) // Now encode the entry into buffer.
			if err != nil {
				return err
			}
			p.Len = uint32(plen)
			b.Ptrs = append(b.Ptrs, p)
			written++

			// It is possible that the size of the buffer grows beyond the max size of the value
			// log (this happens when a transaction contains entries with large value sizes) and
			// badger might run into out of memory errors. We flush the buffer here if it's size
			// grows beyond the max value log size.
			if int64(buf.Len()) > vlog.db.opt.ValueLogFileSize {
				if err := flushWrites(); err != nil {
					return err
				}
			}
		}
		vlog.numEntriesWritten += uint32(written)
		// We write to disk here so that all entries that are part of the same transaction are
		// written to the same vlog file.
		writeNow :=
			vlog.woffset()+uint32(buf.Len()) > uint32(vlog.opt.ValueLogFileSize) ||
				vlog.numEntriesWritten > uint32(vlog.opt.ValueLogMaxEntries)
		if writeNow {
			if err := toDisk(); err != nil {
				return err
			}
		}
	}
	return toDisk()
}

// Gets the logFile and acquires and RLock() for the mmap. You must call RUnlock on the file
// (if non-nil)
func (vlog *valueLog) getFileRLocked(vp valuePointer) (*logFile, error) {
	vlog.filesLock.RLock()
	defer vlog.filesLock.RUnlock()
	ret, ok := vlog.filesMap[vp.Fid]
	if !ok {
		// log file has gone away, will need to retry the operation.
		return nil, ErrRetry
	}

	// Check for valid offset if we are reading from writable log.
	maxFid := vlog.maxFid
	if vp.Fid == maxFid {
		currentOffset := vlog.woffset()
		if vp.Offset >= currentOffset {
			return nil, errors.Errorf(
				"Invalid value pointer offset: %d greater than current offset: %d",
				vp.Offset, currentOffset)
		}
	}

	ret.lock.RLock()
	return ret, nil
}

// Read reads the value log at a given location.
// TODO: Make this read private.
func (vlog *valueLog) Read(vp valuePointer, s *y.Slice) ([]byte, func(), error) {
	buf, lf, err := vlog.readValueBytes(vp, s)
	// log file is locked so, decide whether to lock immediately or let the caller to
	// unlock it, after caller uses it.
	cb := vlog.getUnlockCallback(lf)
	if err != nil {
		return nil, cb, err
	}

	if vlog.opt.VerifyValueChecksum {
		hash := crc32.New(y.CastagnoliCrcTable)
		if _, err := hash.Write(buf[:len(buf)-crc32.Size]); err != nil {
			runCallback(cb)
			return nil, nil, errors.Wrapf(err, "failed to write hash for vp %+v", vp)
		}
		// Fetch checksum from the end of the buffer.
		checksum := buf[len(buf)-crc32.Size:]
		if hash.Sum32() != y.BytesToU32(checksum) {
			runCallback(cb)
			return nil, nil, errors.Wrapf(y.ErrChecksumMismatch, "value corrupted for vp: %+v", vp)
		}
	}
	var h header
	headerLen := h.Decode(buf)
	kv := buf[headerLen:]
	if lf.encryptionEnabled() {
		kv, err = lf.decryptKV(kv, vp.Offset)
		if err != nil {
			return nil, cb, err
		}
	}
	if uint32(len(kv)) < h.klen+h.vlen {
		vlog.db.opt.Logger.Errorf("Invalid read: vp: %+v", vp)
		return nil, nil, errors.Errorf("Invalid read: Len: %d read at:[%d:%d]",
			len(kv), h.klen, h.klen+h.vlen)
	}
	return kv[h.klen : h.klen+h.vlen], cb, nil
}

// getUnlockCallback will returns a function which unlock the logfile if the logfile is mmaped.
// otherwise, it unlock the logfile and return nil.
func (vlog *valueLog) getUnlockCallback(lf *logFile) func() {
	if lf == nil {
		return nil
	}
	if vlog.opt.ValueLogLoadingMode == options.MemoryMap {
		return lf.lock.RUnlock
	}
	lf.lock.RUnlock()
	return nil
}

// readValueBytes return vlog entry slice and read locked log file. Caller should take care of
// logFile unlocking.
func (vlog *valueLog) readValueBytes(vp valuePointer, s *y.Slice) ([]byte, *logFile, error) {
	lf, err := vlog.getFileRLocked(vp)
	if err != nil {
		return nil, nil, err
	}

	buf, err := lf.read(vp, s)
	return buf, lf, err
}

func (vlog *valueLog) pickLog(head valuePointer, tr trace.Trace) (files []*logFile) {
	vlog.filesLock.RLock()
	defer vlog.filesLock.RUnlock()
	fids := vlog.sortedFids()
	switch {
	case len(fids) <= 1:
		tr.LazyPrintf("Only one or less value log file.")
		return nil
	case head.Fid == 0:
		tr.LazyPrintf("Head pointer is at zero.")
		return nil
	}

	// Pick a candidate that contains the largest amount of discardable data
	candidate := struct {
		fid     uint32
		discard int64
	}{math.MaxUint32, 0}
	vlog.lfDiscardStats.RLock()
	for _, fid := range fids {
		if fid >= head.Fid {
			break
		}
		if vlog.lfDiscardStats.m[fid] > candidate.discard {
			candidate.fid = fid
			candidate.discard = vlog.lfDiscardStats.m[fid]
		}
	}
	vlog.lfDiscardStats.RUnlock()

	if candidate.fid != math.MaxUint32 { // Found a candidate
		tr.LazyPrintf("Found candidate via discard stats: %v", candidate)
		files = append(files, vlog.filesMap[candidate.fid])
	} else {
		tr.LazyPrintf("Could not find candidate via discard stats. Randomly picking one.")
	}

	// Fallback to randomly picking a log file
	var idxHead int
	for i, fid := range fids {
		if fid == head.Fid {
			idxHead = i
			break
		}
	}
	if idxHead == 0 { // Not found or first file
		tr.LazyPrintf("Could not find any file.")
		return nil
	}
	idx := rand.Intn(idxHead) // Don’t include head.Fid. We pick a random file before it.
	if idx > 0 {
		idx = rand.Intn(idx + 1) // Another level of rand to favor smaller fids.
	}
	tr.LazyPrintf("Randomly chose fid: %d", fids[idx])
	files = append(files, vlog.filesMap[fids[idx]])
	return files
}

func discardEntry(e Entry, vs y.ValueStruct, db *DB) bool {
	if vs.Version != y.ParseTs(e.Key) {
		// Version not found. Discard.
		return true
	}
	if isDeletedOrExpired(vs.Meta, vs.ExpiresAt) {
		return true
	}
	if (vs.Meta & bitValuePointer) == 0 {
		// Key also stores the value in LSM. Discard.
		return true
	}
	if (vs.Meta & bitFinTxn) > 0 {
		// Just a txn finish entry. Discard.
		return true
	}
	if bytes.HasPrefix(e.Key, badgerMove) {
		// Verify the actual key entry without the badgerPrefix has not been deleted.
		// If this is not done the badgerMove entry will be kept forever moving from
		// vlog to vlog during rewrites.
		avs, err := db.get(e.Key[len(badgerMove):])
		if err != nil {
			return false
		}
		return avs.Version == 0
	}
	return false
}

func (vlog *valueLog) doRunGC(lf *logFile, discardRatio float64, tr trace.Trace) (err error) {
	// Update stats before exiting
	defer func() {
		if err == nil {
			vlog.lfDiscardStats.Lock()
			delete(vlog.lfDiscardStats.m, lf.fid)
			vlog.lfDiscardStats.Unlock()
		}
	}()

	type reason struct {
		total   float64
		discard float64
		count   int
	}

	fi, err := lf.fd.Stat()
	if err != nil {
		tr.LazyPrintf("Error while finding file size: %v", err)
		tr.SetError()
		return err
	}

	// Set up the sampling window sizes.
	sizeWindow := float64(fi.Size()) * 0.1                          // 10% of the file as window.
	sizeWindowM := sizeWindow / (1 << 20)                           // in MBs.
	countWindow := int(float64(vlog.opt.ValueLogMaxEntries) * 0.01) // 1% of num entries.
	tr.LazyPrintf("Size window: %5.2f. Count window: %d.", sizeWindow, countWindow)

	// Pick a random start point for the log.
	skipFirstM := float64(rand.Int63n(fi.Size())) // Pick a random starting location.
	skipFirstM -= sizeWindow                      // Avoid hitting EOF by moving back by window.
	skipFirstM /= float64(mi)                     // Convert to MBs.
	tr.LazyPrintf("Skip first %5.2f MB of file of size: %d MB", skipFirstM, fi.Size()/mi)
	var skipped float64

	var r reason
	start := time.Now()
	y.AssertTrue(vlog.db != nil)
	s := new(y.Slice)
	var numIterations int
	_, err = vlog.iterate(lf, 0, func(e Entry, vp valuePointer) error {
		numIterations++
		esz := float64(vp.Len) / (1 << 20) // in MBs.
		if skipped < skipFirstM {
			skipped += esz
			return nil
		}

		// Sample until we reach the window sizes or exceed 10 seconds.
		if r.count > countWindow {
			tr.LazyPrintf("Stopping sampling after %d entries.", countWindow)
			return errStop
		}
		if r.total > sizeWindowM {
			tr.LazyPrintf("Stopping sampling after reaching window size.")
			return errStop
		}
		if time.Since(start) > 10*time.Second {
			tr.LazyPrintf("Stopping sampling after 10 seconds.")
			return errStop
		}
		r.total += esz
		r.count++

		vs, err := vlog.db.get(e.Key)
		if err != nil {
			return err
		}
		if discardEntry(e, vs, vlog.db) {
			r.discard += esz
			return nil
		}

		// Value is still present in value log.
		y.AssertTrue(len(vs.Value) > 0)
		vp.Decode(vs.Value)

		if vp.Fid > lf.fid {
			// Value is present in a later log. Discard.
			r.discard += esz
			return nil
		}
		if vp.Offset > e.offset {
			// Value is present in a later offset, but in the same log.
			r.discard += esz
			return nil
		}
		if vp.Fid == lf.fid && vp.Offset == e.offset {
			// This is still the active entry. This would need to be rewritten.

		} else {
			vlog.opt.Debugf("Reason=%+v\n", r)
			buf, lf, err := vlog.readValueBytes(vp, s)
			// we need to decide, whether to unlock the lock file immediately based on the
			// loading mode. getUnlockCallback will take care of it.
			cb := vlog.getUnlockCallback(lf)
			if err != nil {
				runCallback(cb)
				return errStop
			}
			ne, err := lf.decodeEntry(buf, vp.Offset)
			if err != nil {
				runCallback(cb)
				return errStop
			}
			ne.print("Latest Entry Header in LSM")
			e.print("Latest Entry in Log")
			runCallback(cb)
			return errors.Errorf("This shouldn't happen. Latest Pointer:%+v. Meta:%v.",
				vp, vs.Meta)
		}
		return nil
	})

	if err != nil {
		tr.LazyPrintf("Error while iterating for RunGC: %v", err)
		tr.SetError()
		return err
	}
	tr.LazyPrintf("Fid: %d. Skipped: %5.2fMB Num iterations: %d. Data status=%+v\n",
		lf.fid, skipped, numIterations, r)

	// If we couldn't sample at least a 1000 KV pairs or at least 75% of the window size,
	// and what we can discard is below the threshold, we should skip the rewrite.
	if (r.count < countWindow && r.total < sizeWindowM*0.75) || r.discard < discardRatio*r.total {
		tr.LazyPrintf("Skipping GC on fid: %d", lf.fid)
		return ErrNoRewrite
	}
	if err = vlog.rewrite(lf, tr); err != nil {
		return err
	}
	tr.LazyPrintf("Done rewriting.")
	return nil
}

func (vlog *valueLog) waitOnGC(lc *y.Closer) {
	defer lc.Done()

	<-lc.HasBeenClosed() // Wait for lc to be closed.

	// Block any GC in progress to finish, and don't allow any more writes to runGC by filling up
	// the channel of size 1.
	vlog.garbageCh <- struct{}{}
}

func (vlog *valueLog) runGC(discardRatio float64, head valuePointer) error {
	select {
	case vlog.garbageCh <- struct{}{}:
		// Pick a log file for GC.
		tr := trace.New("Badger.ValueLog", "GC")
		tr.SetMaxEvents(100)
		defer func() {
			tr.Finish()
			<-vlog.garbageCh
		}()

		var err error
		files := vlog.pickLog(head, tr)
		if len(files) == 0 {
			tr.LazyPrintf("PickLog returned zero results.")
			return ErrNoRewrite
		}
		tried := make(map[uint32]bool)
		for _, lf := range files {
			if _, done := tried[lf.fid]; done {
				continue
			}
			tried[lf.fid] = true
			err = vlog.doRunGC(lf, discardRatio, tr)
			if err == nil {
				return vlog.deleteMoveKeysFor(lf.fid, tr)
			}
		}
		return err
	default:
		return ErrRejected
	}
}

func (vlog *valueLog) updateDiscardStats(stats map[uint32]int64) {
	if vlog.opt.InMemory {
		return
	}

	select {
	case vlog.lfDiscardStats.flushChan <- stats:
	default:
		vlog.opt.Warningf("updateDiscardStats called: discard stats flushChan full, " +
			"returning without pushing to flushChan")
	}
}

func (vlog *valueLog) flushDiscardStats() {
	defer vlog.lfDiscardStats.closer.Done()

	mergeStats := func(stats map[uint32]int64) ([]byte, error) {
		vlog.lfDiscardStats.Lock()
		defer vlog.lfDiscardStats.Unlock()
		for fid, count := range stats {
			vlog.lfDiscardStats.m[fid] += count
			vlog.lfDiscardStats.updatesSinceFlush++
		}

		if vlog.lfDiscardStats.updatesSinceFlush > discardStatsFlushThreshold {
			encodedDS, err := json.Marshal(vlog.lfDiscardStats.m)
			if err != nil {
				return nil, err
			}
			vlog.lfDiscardStats.updatesSinceFlush = 0
			return encodedDS, nil
		}
		return nil, nil
	}

	process := func(stats map[uint32]int64) error {
		encodedDS, err := mergeStats(stats)
		if err != nil || encodedDS == nil {
			return err
		}

		entries := []*Entry{{
			Key:   y.KeyWithTs(lfDiscardStatsKey, 1),
			Value: encodedDS,
		}}
		req, err := vlog.db.sendToWriteCh(entries)
		// No special handling of ErrBlockedWrites is required as err is just logged in
		// for loop below.
		if err != nil {
			return errors.Wrapf(err, "failed to push discard stats to write channel")
		}
		return req.Wait()
	}

	closer := vlog.lfDiscardStats.closer
	for {
		select {
		case <-closer.HasBeenClosed():
			// For simplicity just return without processing already present in stats in flushChan.
			return
		case stats := <-vlog.lfDiscardStats.flushChan:
			if err := process(stats); err != nil {
				vlog.opt.Errorf("unable to process discardstats with error: %s", err)
			}
		}
	}
}

// populateDiscardStats populates vlog.lfDiscardStats.
// This function will be called while initializing valueLog.
func (vlog *valueLog) populateDiscardStats() error {
	key := y.KeyWithTs(lfDiscardStatsKey, math.MaxUint64)
	var statsMap map[uint32]int64
	var val []byte
	var vp valuePointer
	for {
		vs, err := vlog.db.get(key)
		if err != nil {
			return err
		}
		// Value doesn't exist.
		if vs.Meta == 0 && len(vs.Value) == 0 {
			vlog.opt.Debugf("Value log discard stats empty")
			return nil
		}
		vp.Decode(vs.Value)
		// Entry stored in LSM tree.
		if vs.Meta&bitValuePointer == 0 {
			val = y.SafeCopy(val, vs.Value)
			break
		}
		// Read entry from value log.
		result, cb, err := vlog.Read(vp, new(y.Slice))
		runCallback(cb)
		val = y.SafeCopy(val, result)
		// The result is stored in val. We can break the loop from here.
		if err == nil {
			break
		}
		if err != ErrRetry {
			return err
		}
		// If we're at this point it means we haven't found the value yet and if the current key has
		// badger move prefix, we should break from here since we've already tried the original key
		// and the key with move prefix. "val" would be empty since we haven't found the value yet.
		if bytes.HasPrefix(key, badgerMove) {
			break
		}
		// If we're at this point it means the discard stats key was moved by the GC and the actual
		// entry is the one prefixed by badger move key.
		// Prepend existing key with badger move and search for the key.
		key = append(badgerMove, key...)
	}

	if len(val) == 0 {
		return nil
	}
	if err := json.Unmarshal(val, &statsMap); err != nil {
		return errors.Wrapf(err, "failed to unmarshal discard stats")
	}
	vlog.opt.Debugf("Value Log Discard stats: %v", statsMap)
	vlog.lfDiscardStats.flushChan <- statsMap
	return nil
}
