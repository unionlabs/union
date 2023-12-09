package snapshots

import (
	"bufio"
	"compress/zlib"
	"io"

	protoio "github.com/cosmos/gogoproto/io"
	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/errors"
)

const (
	// Do not change chunk size without new snapshot format (must be uniform across nodes)
	snapshotChunkSize  = uint64(10e6)
	snapshotBufferSize = int(snapshotChunkSize)
	// Do not change compression level without new snapshot format (must be uniform across nodes)
	snapshotCompressionLevel = 7
)

// StreamWriter set up a stream pipeline to serialize snapshot nodes:
// Exported Items -> delimited Protobuf -> zlib -> buffer -> chunkWriter -> chan io.ReadCloser
type StreamWriter struct {
	chunkWriter *ChunkWriter
	bufWriter   *bufio.Writer
	zWriter     *zlib.Writer
	protoWriter protoio.WriteCloser
}

// NewStreamWriter set up a stream pipeline to serialize snapshot DB records.
func NewStreamWriter(ch chan<- io.ReadCloser) *StreamWriter {
	chunkWriter := NewChunkWriter(ch, snapshotChunkSize)
	bufWriter := bufio.NewWriterSize(chunkWriter, snapshotBufferSize)
	zWriter, err := zlib.NewWriterLevel(bufWriter, snapshotCompressionLevel)
	if err != nil {
		chunkWriter.CloseWithError(errors.Wrap(err, "zlib failure"))
		return nil
	}
	protoWriter := protoio.NewDelimitedWriter(zWriter)
	return &StreamWriter{
		chunkWriter: chunkWriter,
		bufWriter:   bufWriter,
		zWriter:     zWriter,
		protoWriter: protoWriter,
	}
}

// WriteMsg implements protoio.Write interface
func (sw *StreamWriter) WriteMsg(msg proto.Message) error {
	return sw.protoWriter.WriteMsg(msg)
}

// Close implements io.Closer interface
func (sw *StreamWriter) Close() error {
	if err := sw.protoWriter.Close(); err != nil {
		sw.chunkWriter.CloseWithError(err)
		return err
	}
	if err := sw.bufWriter.Flush(); err != nil {
		sw.chunkWriter.CloseWithError(err)
		return err
	}
	return sw.chunkWriter.Close()
}

// CloseWithError pass error to chunkWriter
func (sw *StreamWriter) CloseWithError(err error) {
	sw.chunkWriter.CloseWithError(err)
}

// StreamReader set up a restore stream pipeline
// chan io.ReadCloser -> chunkReader -> zlib -> delimited Protobuf -> ExportNode
type StreamReader struct {
	chunkReader *ChunkReader
	zReader     io.ReadCloser
	protoReader protoio.ReadCloser
}

// NewStreamReader set up a restore stream pipeline.
func NewStreamReader(chunks <-chan io.ReadCloser) (*StreamReader, error) {
	chunkReader := NewChunkReader(chunks)
	zReader, err := zlib.NewReader(chunkReader)
	if err != nil {
		return nil, errors.Wrap(err, "zlib failure")
	}
	protoReader := protoio.NewDelimitedReader(zReader, snapshotMaxItemSize)
	return &StreamReader{
		chunkReader: chunkReader,
		zReader:     zReader,
		protoReader: protoReader,
	}, nil
}

// ReadMsg implements protoio.Reader interface
func (sr *StreamReader) ReadMsg(msg proto.Message) error {
	return sr.protoReader.ReadMsg(msg)
}

// Close implements io.Closer interface
func (sr *StreamReader) Close() error {
	var err error
	if err1 := sr.protoReader.Close(); err1 != nil {
		err = err1
	}
	if err2 := sr.zReader.Close(); err2 != nil {
		err = err2
	}
	if err3 := sr.chunkReader.Close(); err3 != nil {
		err = err3
	}
	return err
}
