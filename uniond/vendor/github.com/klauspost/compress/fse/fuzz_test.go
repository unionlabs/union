package fse

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
)

func FuzzCompress(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/fse_compress.zip", fuzz.TypeGoFuzz, false)
	f.Fuzz(func(t *testing.T, buf0 []byte) {
		var s, s2 Scratch
		b, err := Compress(buf0, &s)
		if err != nil || b == nil {
			return
		}
		err = s.validateNorm()
		if err != nil {
			return
		}
		//Decompress
		got, err := Decompress(b, &s2)
		if err != nil || len(got) == 0 {
			return
		}
		if !bytes.Equal(buf0, got) {
			t.Fatal(fmt.Sprintln("FuzzCompress output mismatch\n", len(got), "org: \n", len(buf0)))
		}
	})
}

func FuzzDecompress(f *testing.F) {
	// Input is mixed, but TypeGoFuzz will fall back to raw input.
	fuzz.AddFromZip(f, "testdata/fse_decompress.zip", fuzz.TypeGoFuzz, false)
	f.Fuzz(func(t *testing.T, buf0 []byte) {
		var s2 Scratch
		s2.DecompressLimit = 128 << 10
		//Decompress
		got, err := Decompress(buf0, &s2)
		if err != nil || len(got) == 0 {
			return
		}
	})
}
