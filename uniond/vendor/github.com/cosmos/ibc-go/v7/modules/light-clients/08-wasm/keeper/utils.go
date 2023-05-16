package keeper

import (
	"bytes"
	"compress/gzip"
	"io"

	"github.com/cosmos/ibc-go/v7/modules/light-clients/08-wasm/types"
)

// Copied gzip feature from wasmd
// https://github.com/CosmWasm/wasmd/blob/v0.31.0/x/wasm/ioutils/utils.go

// Note: []byte can never be const as they are inherently mutable
var (
	// magic bytes to identify gzip.
	// See https://www.ietf.org/rfc/rfc1952.txt
	// and https://github.com/golang/go/blob/master/src/net/http/sniff.go#L186
	gzipIdent = []byte("\x1F\x8B\x08")

	wasmIdent = []byte("\x00\x61\x73\x6D")
)

// IsGzip returns checks if the file contents are gzip compressed
func IsGzip(input []byte) bool {
	return len(input) >= 3 && bytes.Equal(gzipIdent, input[0:3])
}

// IsWasm checks if the file contents are of wasm binary
func IsWasm(input []byte) bool {
	return bytes.Equal(input[:4], wasmIdent)
}

// Uncompress expects a valid gzip source to unpack or fails. See IsGzip
func Uncompress(gzipSrc []byte, limit uint64) ([]byte, error) {
	if uint64(len(gzipSrc)) > limit {
		return nil, types.ErrWasmCodeTooLarge
	}
	zr, err := gzip.NewReader(bytes.NewReader(gzipSrc))
	if err != nil {
		return nil, err
	}
	zr.Multistream(false)
	defer zr.Close()
	return io.ReadAll(LimitReader(zr, int64(limit)))
}

// LimitReader returns a Reader that reads from r
// but stops with types.ErrLimit after n bytes.
// The underlying implementation is a *io.LimitedReader.
func LimitReader(r io.Reader, n int64) io.Reader {
	return &LimitedReader{r: &io.LimitedReader{R: r, N: n}}
}

type LimitedReader struct {
	r *io.LimitedReader
}

func (l *LimitedReader) Read(p []byte) (n int, err error) {
	if l.r.N <= 0 {
		return 0, types.ErrWasmCodeTooLarge
	}
	return l.r.Read(p)
}
