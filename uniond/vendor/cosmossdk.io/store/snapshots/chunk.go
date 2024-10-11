package snapshots

import (
	"io"
	"math"

	"cosmossdk.io/errors"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"
)

// ChunkWriter reads an input stream, splits it into fixed-size chunks, and writes them to a
// sequence of io.ReadClosers via a channel.
type ChunkWriter struct {
	ch        chan<- io.ReadCloser
	pipe      *io.PipeWriter
	chunkSize uint64
	written   uint64
	closed    bool
}

// NewChunkWriter creates a new ChunkWriter. If chunkSize is 0, no chunking will be done.
func NewChunkWriter(ch chan<- io.ReadCloser, chunkSize uint64) *ChunkWriter {
	return &ChunkWriter{
		ch:        ch,
		chunkSize: chunkSize,
	}
}

// chunk creates a new chunk.
func (w *ChunkWriter) chunk() error {
	if w.pipe != nil {
		err := w.pipe.Close()
		if err != nil {
			return err
		}
	}
	pr, pw := io.Pipe()
	w.ch <- pr
	w.pipe = pw
	w.written = 0
	return nil
}

// Close implements io.Closer.
func (w *ChunkWriter) Close() error {
	if !w.closed {
		w.closed = true
		close(w.ch)
		var err error
		if w.pipe != nil {
			err = w.pipe.Close()
		}
		return err
	}
	return nil
}

// CloseWithError closes the writer and sends an error to the reader.
func (w *ChunkWriter) CloseWithError(err error) {
	if !w.closed {
		if w.pipe == nil {
			// create a dummy pipe just to propagate the error to the reader, it always returns nil
			_ = w.chunk()
		}
		w.closed = true
		close(w.ch)
		_ = w.pipe.CloseWithError(err) // CloseWithError always returns nil
	}
}

// Write implements io.Writer.
func (w *ChunkWriter) Write(data []byte) (int, error) {
	if w.closed {
		return 0, errors.Wrap(storetypes.ErrLogic, "cannot write to closed ChunkWriter")
	}
	nTotal := 0
	for len(data) > 0 {
		if w.pipe == nil || (w.written >= w.chunkSize && w.chunkSize > 0) {
			err := w.chunk()
			if err != nil {
				return nTotal, err
			}
		}

		var writeSize uint64
		if w.chunkSize == 0 {
			writeSize = uint64(len(data))
		} else {
			writeSize = w.chunkSize - w.written
		}
		if writeSize > uint64(len(data)) {
			writeSize = uint64(len(data))
		}

		n, err := w.pipe.Write(data[:writeSize])
		w.written += uint64(n)
		nTotal += n
		if err != nil {
			return nTotal, err
		}
		data = data[writeSize:]
	}
	return nTotal, nil
}

// ChunkReader reads chunks from a channel of io.ReadClosers and outputs them as an io.Reader
type ChunkReader struct {
	ch     <-chan io.ReadCloser
	reader io.ReadCloser
}

// NewChunkReader creates a new ChunkReader.
func NewChunkReader(ch <-chan io.ReadCloser) *ChunkReader {
	return &ChunkReader{ch: ch}
}

// next fetches the next chunk from the channel, or returns io.EOF if there are no more chunks.
func (r *ChunkReader) next() error {
	reader, ok := <-r.ch
	if !ok {
		return io.EOF
	}
	r.reader = reader
	return nil
}

// Close implements io.ReadCloser.
func (r *ChunkReader) Close() error {
	var err error
	if r.reader != nil {
		err = r.reader.Close()
		r.reader = nil
	}
	for reader := range r.ch {
		if e := reader.Close(); e != nil && err == nil {
			err = e
		}
	}
	return err
}

// Read implements io.Reader.
func (r *ChunkReader) Read(p []byte) (int, error) {
	if r.reader == nil {
		err := r.next()
		if err != nil {
			return 0, err
		}
	}
	n, err := r.reader.Read(p)
	if errors.IsOf(err, io.EOF) {
		err = r.reader.Close()
		r.reader = nil
		if err != nil {
			return 0, err
		}
		return r.Read(p)
	}
	return n, err
}

// DrainChunks drains and closes all remaining chunks from a chunk channel.
func DrainChunks(chunks <-chan io.ReadCloser) {
	for chunk := range chunks {
		_ = chunk.Close()
	}
}

// ValidRestoreHeight will check height is valid for snapshot restore or not
func ValidRestoreHeight(format uint32, height uint64) error {
	if format != snapshottypes.CurrentFormat {
		return errors.Wrapf(snapshottypes.ErrUnknownFormat, "format %v", format)
	}

	if height == 0 {
		return errors.Wrap(storetypes.ErrLogic, "cannot restore snapshot at height 0")
	}
	if height > uint64(math.MaxInt64) {
		return errors.Wrapf(snapshottypes.ErrInvalidMetadata,
			"snapshot height %v cannot exceed %v", height, int64(math.MaxInt64))
	}

	return nil
}
