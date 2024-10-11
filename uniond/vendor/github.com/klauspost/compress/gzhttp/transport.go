// Copyright (c) 2021 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gzhttp

import (
	"io"
	"net/http"
	"strings"
	"sync"

	"github.com/klauspost/compress/gzip"
	"github.com/klauspost/compress/zstd"
)

// Transport will wrap an HTTP transport with a custom handler
// that will request gzip and automatically decompress it.
// Using this is significantly faster than using the default transport.
func Transport(parent http.RoundTripper, opts ...transportOption) http.RoundTripper {
	g := gzRoundtripper{parent: parent, withZstd: true, withGzip: true}
	for _, o := range opts {
		o(&g)
	}
	var ae []string
	if g.withZstd {
		ae = append(ae, "zstd")
	}
	if g.withGzip {
		ae = append(ae, "gzip")
	}
	g.acceptEncoding = strings.Join(ae, ",")
	return &g
}

type transportOption func(c *gzRoundtripper)

// TransportEnableZstd will send Zstandard as a compression option to the server.
// Enabled by default, but may be disabled if future problems arise.
func TransportEnableZstd(b bool) transportOption {
	return func(c *gzRoundtripper) {
		c.withZstd = b
	}
}

// TransportEnableGzip will send Gzip as a compression option to the server.
// Enabled by default.
func TransportEnableGzip(b bool) transportOption {
	return func(c *gzRoundtripper) {
		c.withGzip = b
	}
}

// TransportCustomEval will send the header of a response to a custom function.
// If the function returns false, the response will be returned as-is,
// Otherwise it will be decompressed based on Content-Encoding field, regardless
// of whether the transport added the encoding.
func TransportCustomEval(fn func(header http.Header) bool) transportOption {
	return func(c *gzRoundtripper) {
		c.customEval = fn
	}
}

// TransportAlwaysDecompress will always decompress the response,
// regardless of whether we requested it or not.
// Default is false, which will pass compressed data through
// if we did not request compression.
func TransportAlwaysDecompress(enabled bool) transportOption {
	return func(c *gzRoundtripper) {
		c.alwaysDecomp = enabled
	}
}

type gzRoundtripper struct {
	parent             http.RoundTripper
	acceptEncoding     string
	withZstd, withGzip bool
	alwaysDecomp       bool
	customEval         func(header http.Header) bool
}

func (g *gzRoundtripper) RoundTrip(req *http.Request) (*http.Response, error) {
	var requestedComp bool
	if req.Header.Get("Accept-Encoding") == "" &&
		req.Header.Get("Range") == "" &&
		req.Method != "HEAD" {
		// Request gzip only, not deflate. Deflate is ambiguous and
		// not as universally supported anyway.
		// See: https://zlib.net/zlib_faq.html#faq39
		//
		// Note that we don't request this for HEAD requests,
		// due to a bug in nginx:
		//   https://trac.nginx.org/nginx/ticket/358
		//   https://golang.org/issue/5522
		//
		// We don't request gzip if the request is for a range, since
		// auto-decoding a portion of a gzipped document will just fail
		// anyway. See https://golang.org/issue/8923
		requestedComp = len(g.acceptEncoding) > 0
		req.Header.Set("Accept-Encoding", g.acceptEncoding)
	}

	resp, err := g.parent.RoundTrip(req)
	if err != nil {
		return resp, err
	}
	decompress := g.alwaysDecomp
	if g.customEval != nil {
		if !g.customEval(resp.Header) {
			return resp, nil
		}
		decompress = true
	} else {
		if !requestedComp && !g.alwaysDecomp {
			return resp, nil
		}
	}
	// Decompress
	if (decompress || g.withGzip) && asciiEqualFold(resp.Header.Get("Content-Encoding"), "gzip") {
		resp.Body = &gzipReader{body: resp.Body}
		resp.Header.Del("Content-Encoding")
		resp.Header.Del("Content-Length")
		resp.ContentLength = -1
		resp.Uncompressed = true
	}
	if (decompress || g.withZstd) && asciiEqualFold(resp.Header.Get("Content-Encoding"), "zstd") {
		resp.Body = &zstdReader{body: resp.Body}
		resp.Header.Del("Content-Encoding")
		resp.Header.Del("Content-Length")
		resp.ContentLength = -1
		resp.Uncompressed = true
	}

	return resp, nil
}

var gzReaderPool sync.Pool

// gzipReader wraps a response body so it can lazily
// call gzip.NewReader on the first call to Read
type gzipReader struct {
	body io.ReadCloser // underlying HTTP/1 response body framing
	zr   *gzip.Reader  // lazily-initialized gzip reader
	zerr error         // any error from gzip.NewReader; sticky
}

func (gz *gzipReader) Read(p []byte) (n int, err error) {
	if gz.zr == nil {
		if gz.zerr == nil {
			zr, ok := gzReaderPool.Get().(*gzip.Reader)
			if ok {
				gz.zr, gz.zerr = zr, zr.Reset(gz.body)
			} else {
				gz.zr, gz.zerr = gzip.NewReader(gz.body)
			}
		}
		if gz.zerr != nil {
			return 0, gz.zerr
		}
	}

	return gz.zr.Read(p)
}

func (gz *gzipReader) Close() error {
	if gz.zr != nil {
		gzReaderPool.Put(gz.zr)
		gz.zr = nil
	}
	return gz.body.Close()
}

// asciiEqualFold is strings.EqualFold, ASCII only. It reports whether s and t
// are equal, ASCII-case-insensitively.
func asciiEqualFold(s, t string) bool {
	if len(s) != len(t) {
		return false
	}
	for i := 0; i < len(s); i++ {
		if lower(s[i]) != lower(t[i]) {
			return false
		}
	}
	return true
}

// lower returns the ASCII lowercase version of b.
func lower(b byte) byte {
	if 'A' <= b && b <= 'Z' {
		return b + ('a' - 'A')
	}
	return b
}

// zstdReaderPool pools zstd decoders.
var zstdReaderPool sync.Pool

// zstdReader wraps a response body so it can lazily
// call gzip.NewReader on the first call to Read
type zstdReader struct {
	body io.ReadCloser // underlying HTTP/1 response body framing
	zr   *zstd.Decoder // lazily-initialized gzip reader
	zerr error         // any error from zstd.NewReader; sticky
}

func (zr *zstdReader) Read(p []byte) (n int, err error) {
	if zr.zerr != nil {
		return 0, zr.zerr
	}
	if zr.zr == nil {
		if zr.zerr == nil {
			reader, ok := zstdReaderPool.Get().(*zstd.Decoder)
			if ok {
				zr.zerr = reader.Reset(zr.body)
				zr.zr = reader
			} else {
				zr.zr, zr.zerr = zstd.NewReader(zr.body, zstd.WithDecoderLowmem(true), zstd.WithDecoderMaxWindow(32<<20), zstd.WithDecoderConcurrency(1))
			}
		}
		if zr.zerr != nil {
			return 0, zr.zerr
		}
	}
	n, err = zr.zr.Read(p)
	if err != nil {
		// Usually this will be io.EOF,
		// stash the decoder and keep the error.
		zr.zr.Reset(nil)
		zstdReaderPool.Put(zr.zr)
		zr.zr = nil
		zr.zerr = err
	}
	return
}

func (zr *zstdReader) Close() error {
	if zr.zr != nil {
		zr.zr.Reset(nil)
		zstdReaderPool.Put(zr.zr)
		zr.zr = nil
	}
	return zr.body.Close()
}
