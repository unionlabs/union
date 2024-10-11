// package gzkp provides gzip compression through github.com/klauspost/compress/gzip.

package gzkp

import (
	"io"
	"sync"

	"github.com/klauspost/compress/gzhttp/writer"
	"github.com/klauspost/compress/gzip"
)

// gzipWriterPools stores a sync.Pool for each compression level for reuse of
// gzip.Writers. Use poolIndex to covert a compression level to an index into
// gzipWriterPools.
var gzipWriterPools [gzip.BestCompression - gzip.StatelessCompression + 1]*sync.Pool

func init() {
	for i := gzip.StatelessCompression; i <= gzip.BestCompression; i++ {
		addLevelPool(i)
	}
}

// poolIndex maps a compression level to its index into gzipWriterPools. It
// assumes that level is a valid gzip compression level.
func poolIndex(level int) int {
	if level > gzip.BestCompression {
		level = gzip.BestCompression
	}
	if level < gzip.StatelessCompression {
		level = gzip.BestSpeed
	}
	return level - gzip.StatelessCompression
}

func addLevelPool(level int) {
	gzipWriterPools[poolIndex(level)] = &sync.Pool{
		New: func() interface{} {
			// NewWriterLevel only returns error on a bad level, we are guaranteeing
			// that this will be a valid level so it is okay to ignore the returned
			// error.
			w, _ := gzip.NewWriterLevel(nil, level)
			return w
		},
	}
}

type pooledWriter struct {
	*gzip.Writer
	index int
}

func (pw *pooledWriter) Close() error {
	err := pw.Writer.Close()
	gzipWriterPools[pw.index].Put(pw.Writer)
	pw.Writer = nil
	return err
}

func NewWriter(w io.Writer, level int) writer.GzipWriter {
	index := poolIndex(level)
	gzw := gzipWriterPools[index].Get().(*gzip.Writer)
	gzw.Reset(w)
	return &pooledWriter{
		Writer: gzw,
		index:  index,
	}
}

// SetHeader will override the gzip header on pw.
func (pw *pooledWriter) SetHeader(h writer.Header) {
	pw.Name = h.Name
	pw.Extra = h.Extra
	pw.Comment = h.Comment
	pw.ModTime = h.ModTime
	pw.OS = h.OS
}

func Levels() (min, max int) {
	return gzip.StatelessCompression, gzip.BestCompression
}

func ImplementationInfo() string {
	return "klauspost/compress/gzip"
}
