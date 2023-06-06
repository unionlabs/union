package ioutils

import (
	"bytes"
	"compress/gzip"
	"io"

	errorsmod "cosmossdk.io/errors"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// Uncompress expects a valid gzip source to unpack or fails. See IsGzip
func Uncompress(gzipSrc []byte, limit uint64) ([]byte, error) {
	if uint64(len(gzipSrc)) > limit {
		return nil, types.ErrLimit.Wrapf("max %d bytes", limit)
	}
	zr, err := gzip.NewReader(bytes.NewReader(gzipSrc))
	if err != nil {
		return nil, err
	}
	zr.Multistream(false)
	defer zr.Close()
	bz, err := io.ReadAll(LimitReader(zr, int64(limit)))
	if types.ErrLimit.Is(err) {
		return nil, errorsmod.Wrapf(err, "max %d bytes", limit)
	}
	return bz, err
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
		return 0, types.ErrLimit
	}
	return l.r.Read(p)
}
