// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"context"
	"io"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/sharedcache"
	"github.com/cockroachdb/pebble/objstorage/remote"
)

const remoteMaxReadaheadSize = 1024 * 1024 /* 1MB */

// remoteReadable is a very simple implementation of Readable on top of the
// ReadCloser returned by remote.Storage.CreateObject.
type remoteReadable struct {
	objReader remote.ObjectReader
	size      int64
	fileNum   base.DiskFileNum
	provider  *provider
}

var _ objstorage.Readable = (*remoteReadable)(nil)

func (p *provider) newRemoteReadable(
	objReader remote.ObjectReader, size int64, fileNum base.DiskFileNum,
) *remoteReadable {
	return &remoteReadable{
		objReader: objReader,
		size:      size,
		fileNum:   fileNum,
		provider:  p,
	}
}

// ReadAt is part of the objstorage.Readable interface.
func (r *remoteReadable) ReadAt(ctx context.Context, p []byte, offset int64) error {
	return r.readInternal(ctx, p, offset, false /* forCompaction */)
}

// readInternal performs a read for the object, using the cache when
// appropriate.
func (r *remoteReadable) readInternal(
	ctx context.Context, p []byte, offset int64, forCompaction bool,
) error {
	if cache := r.provider.remote.cache; cache != nil {
		flags := sharedcache.ReadFlags{
			// Don't add data to the cache if this read is for a compaction.
			ReadOnly: forCompaction,
		}
		return r.provider.remote.cache.ReadAt(ctx, r.fileNum, p, offset, r.objReader, r.size, flags)
	}
	return r.objReader.ReadAt(ctx, p, offset)
}

func (r *remoteReadable) Close() error {
	defer func() { r.objReader = nil }()
	return r.objReader.Close()
}

func (r *remoteReadable) Size() int64 {
	return r.size
}

func (r *remoteReadable) NewReadHandle(_ context.Context) objstorage.ReadHandle {
	// TODO(radu): use a pool.
	rh := &remoteReadHandle{readable: r}
	rh.readahead.state = makeReadaheadState(remoteMaxReadaheadSize)
	return rh
}

type remoteReadHandle struct {
	readable  *remoteReadable
	readahead struct {
		state  readaheadState
		data   []byte
		offset int64
	}
	forCompaction bool
}

var _ objstorage.ReadHandle = (*remoteReadHandle)(nil)

// ReadAt is part of the objstorage.ReadHandle interface.
func (r *remoteReadHandle) ReadAt(ctx context.Context, p []byte, offset int64) error {
	readaheadSize := r.maybeReadahead(offset, len(p))

	// Check if we already have the data from a previous read-ahead.
	if rhSize := int64(len(r.readahead.data)); rhSize > 0 {
		if r.readahead.offset <= offset && r.readahead.offset+rhSize > offset {
			n := copy(p, r.readahead.data[offset-r.readahead.offset:])
			if n == len(p) {
				// All data was available.
				return nil
			}
			// Use the data that we had and do a shorter read.
			offset += int64(n)
			p = p[n:]
			readaheadSize -= n
		}
	}

	if readaheadSize > len(p) {
		// Don't try to read past EOF.
		if offset+int64(readaheadSize) > r.readable.size {
			readaheadSize = int(r.readable.size - offset)
			if readaheadSize <= 0 {
				// This shouldn't happen in practice (Pebble should never try to read
				// past EOF).
				return io.EOF
			}
		}
		r.readahead.offset = offset
		// TODO(radu): we need to somehow account for this memory.
		if cap(r.readahead.data) >= readaheadSize {
			r.readahead.data = r.readahead.data[:readaheadSize]
		} else {
			r.readahead.data = make([]byte, readaheadSize)
		}

		if err := r.readable.readInternal(ctx, r.readahead.data, offset, r.forCompaction); err != nil {
			// Make sure we don't treat the data as valid next time.
			r.readahead.data = r.readahead.data[:0]
			return err
		}
		copy(p, r.readahead.data)
		return nil
	}

	return r.readable.readInternal(ctx, p, offset, r.forCompaction)
}

func (r *remoteReadHandle) maybeReadahead(offset int64, len int) int {
	if r.forCompaction {
		return remoteMaxReadaheadSize
	}
	return int(r.readahead.state.maybeReadahead(offset, int64(len)))
}

// Close is part of the objstorage.ReadHandle interface.
func (r *remoteReadHandle) Close() error {
	r.readable = nil
	r.readahead.data = nil
	return nil
}

// SetupForCompaction is part of the objstorage.ReadHandle interface.
func (r *remoteReadHandle) SetupForCompaction() {
	r.forCompaction = true
}

// RecordCacheHit is part of the objstorage.ReadHandle interface.
func (r *remoteReadHandle) RecordCacheHit(_ context.Context, offset, size int64) {
	if !r.forCompaction {
		r.readahead.state.recordCacheHit(offset, size)
	}
}
