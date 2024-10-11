package types

import (
	"bytes"
	"compress/gzip"
	"io"
)

// Copied gzip feature from wasmd
// https://github.com/CosmWasm/wasmd/blob/v0.31.0/x/wasm/ioutils/utils.go

// Note: []byte can never be const as they are inherently mutable

// magic bytes to identify gzip.
// See https://www.ietf.org/rfc/rfc1952.txt
// and https://github.com/golang/go/blob/master/src/net/http/sniff.go#L186
var gzipIdent = []byte("\x1F\x8B\x08")

// IsGzip returns checks if the file contents are gzip compressed
func IsGzip(input []byte) bool {
	return len(input) >= 3 && bytes.Equal(gzipIdent, input[0:3])
}

// Uncompress expects a valid gzip source to unpack or fails. See IsGzip
func Uncompress(gzipSrc []byte, limit uint64) ([]byte, error) {
	if uint64(len(gzipSrc)) > limit {
		return nil, ErrWasmCodeTooLarge
	}
	zr, err := gzip.NewReader(bytes.NewReader(gzipSrc))
	if err != nil {
		return nil, err
	}
	zr.Multistream(false)
	defer zr.Close()
	return io.ReadAll(limitReader(zr, int64(limit)))
}

// limitReader returns a Reader that reads from r
// but stops with types.ErrLimit after n bytes.
// The underlying implementation is a *io.LimitedReader.
func limitReader(r io.Reader, n int64) io.Reader {
	return &limitedReader{r: &io.LimitedReader{R: r, N: n}}
}

type limitedReader struct {
	r *io.LimitedReader
}

func (l *limitedReader) Read(p []byte) (n int, err error) {
	if l.r.N <= 0 {
		return 0, ErrWasmCodeTooLarge
	}
	return l.r.Read(p)
}

// GzipIt compresses the input ([]byte)
func GzipIt(input []byte) ([]byte, error) {
	// Create gzip writer.
	var b bytes.Buffer
	w := gzip.NewWriter(&b)
	_, err := w.Write(input)
	if err != nil {
		return nil, err
	}
	err = w.Close() // You must close this first to flush the bytes to the buffer.
	if err != nil {
		return nil, err
	}

	return b.Bytes(), nil
}
