package zstd_test

import (
	"bytes"
	"fmt"

	"github.com/klauspost/compress/zstd"
)

func ExampleWithEncoderDictRaw() {
	// "Raw" dictionaries can be used for compressed delta encoding.

	source := []byte(`
		This is the source file. Compression of the target file with
		the source file as the dictionary will produce a compressed
		delta encoding of the target file.`)
	target := []byte(`
		This is the target file. Decompression of the delta encoding with
		the source file as the dictionary will produce this file.`)

	// The dictionary id is arbitrary. We use zero for compatibility
	// with zstd --patch-from, but applications can use any id
	// not in the range [32768, 1<<31).
	const id = 0

	bestLevel := zstd.WithEncoderLevel(zstd.SpeedBestCompression)

	w, _ := zstd.NewWriter(nil, bestLevel,
		zstd.WithEncoderDictRaw(id, source))
	delta := w.EncodeAll(target, nil)

	r, _ := zstd.NewReader(nil, zstd.WithDecoderDictRaw(id, source))
	out, err := r.DecodeAll(delta, nil)
	if err != nil || !bytes.Equal(out, target) {
		panic("decoding error")
	}

	// Ordinary compression, for reference.
	w, _ = zstd.NewWriter(nil, bestLevel)
	compressed := w.EncodeAll(target, nil)

	// Check that the delta is at most half as big as the compressed file.
	fmt.Println(len(delta) < len(compressed)/2)
	// Output:
	// true
}
