package ioutils

import (
	"bytes"
	"compress/gzip"
)

// Note: []byte can never be const as they are inherently mutable
var (
	// magic bytes to identify gzip.
	// See https://www.ietf.org/rfc/rfc1952.txt
	// and https://github.com/golang/go/blob/master/src/net/http/sniff.go#L186
	gzipIdent = []byte("\x1F\x8B\x08")

	// magic number for Wasm is "\0asm"
	// See https://webassembly.github.io/spec/core/binary/modules.html#binary-module
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
