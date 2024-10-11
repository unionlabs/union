package huff0

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
)

func FuzzCompress(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/fse_compress.zip", fuzz.TypeRaw, false)
	f.Fuzz(func(t *testing.T, buf0 []byte) {
		//use of Compress1X
		var s Scratch
		if len(buf0) > BlockSizeMax {
			buf0 = buf0[:BlockSizeMax]
		}
		EstimateSizes(buf0, &s)
		b, re, err := Compress1X(buf0, &s)
		s.validateTable(s.cTable)
		s.canUseTable(s.cTable)
		if err != nil || b == nil {
			return
		}

		min := s.minSize(len(buf0))

		if len(s.OutData) < min {
			t.Errorf("FuzzCompress: output data length (%d) below shannon limit (%d)", len(s.OutData), min)
		}
		if len(s.OutTable) == 0 {
			t.Error("FuzzCompress: got no table definition")
		}
		if re {
			t.Error("FuzzCompress: claimed to have re-used.")
		}
		if len(s.OutData) == 0 {
			t.Error("FuzzCompress: got no data output")
		}

		dec, remain, err := ReadTable(b, nil)

		//use of Decompress1X
		out, err := dec.Decompress1X(remain)
		if err != nil || len(out) == 0 {
			return
		}
		if !bytes.Equal(out, buf0) {
			t.Fatal(fmt.Sprintln("FuzzCompressX1 output mismatch\n", len(out), "org: \n", len(buf0)))
		}

		//use of Compress4X
		s.Reuse = ReusePolicyAllow
		b, reUsed, err := Compress4X(buf0, &s)
		if err != nil || b == nil {
			return
		}
		remain = b
		if !reUsed {
			dec, remain, err = ReadTable(b, dec)
			if err != nil {
				return
			}
		}
		//use of Decompress4X
		out, err = dec.Decompress4X(remain, len(buf0))
		if err != nil || out == nil {
			return
		}
		if !bytes.Equal(out, buf0) {
			t.Fatal(fmt.Sprintln("FuzzCompressX4 output mismatch: ", len(out), ", org: ", len(buf0)))
		}
	})
}

func FuzzDecompress1x(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/huff0_decompress1x.zip", fuzz.TypeRaw, false)
	f.Fuzz(func(t *testing.T, buf0 []byte) {
		var s Scratch
		_, remain, err := ReadTable(buf0, &s)
		if err != nil {
			return
		}
		out, err := s.Decompress1X(remain)
		if err != nil || out == nil {
			return
		}
	})
}
