package huff0

import (
	"bytes"
	"fmt"
	"io"
	"math/rand"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"

	"github.com/klauspost/compress/flate"
	"github.com/klauspost/compress/zip"
)

type inputFn func() ([]byte, error)

var testfiles = []struct {
	name  string
	fn    inputFn
	err1X error
	err4X error
}{
	// Digits is the digits of the irrational number e. Its decimal representation
	// does not repeat, but there are only 10 possible digits, so it should be
	// reasonably compressible.
	{name: "digits", fn: func() ([]byte, error) { return os.ReadFile("../testdata/e.txt") }},
	// gettysburg.txt is a small plain text.
	{name: "gettysburg", fn: func() ([]byte, error) { return os.ReadFile("../testdata/gettysburg.txt") }},
	// Twain is Project Gutenberg's edition of Mark Twain's classic English novel.
	{name: "twain", fn: func() ([]byte, error) { return os.ReadFile("../testdata/Mark.Twain-Tom.Sawyer.txt") }},
	// Random bytes
	{name: "random", fn: func() ([]byte, error) { return os.ReadFile("../testdata/sharnd.out") }, err1X: ErrIncompressible, err4X: ErrIncompressible},
	// Low entropy
	{name: "low-ent.10k", fn: func() ([]byte, error) { return []byte(strings.Repeat("1221", 10000)), nil }},
	// Super Low entropy
	{name: "superlow-ent-10k", fn: func() ([]byte, error) { return []byte(strings.Repeat("1", 10000) + strings.Repeat("2", 500)), nil }},
	// Zero bytes
	{name: "zeroes", fn: func() ([]byte, error) { return make([]byte, 10000), nil }, err1X: ErrUseRLE, err4X: ErrUseRLE},
	{name: "crash1", fn: func() ([]byte, error) { return os.ReadFile("../testdata/crash1.bin") }, err1X: ErrIncompressible, err4X: ErrIncompressible},
	{name: "crash2", fn: func() ([]byte, error) { return os.ReadFile("../testdata/crash2.bin") }, err4X: ErrIncompressible},
	{name: "crash3", fn: func() ([]byte, error) { return os.ReadFile("../testdata/crash3.bin") }, err1X: ErrIncompressible, err4X: ErrIncompressible},
	{name: "endzerobits", fn: func() ([]byte, error) { return os.ReadFile("../testdata/endzerobits.bin") }, err1X: nil, err4X: ErrIncompressible},
	{name: "endnonzero", fn: func() ([]byte, error) { return os.ReadFile("../testdata/endnonzero.bin") }, err4X: ErrIncompressible},
	{name: "case1", fn: func() ([]byte, error) { return os.ReadFile("../testdata/case1.bin") }, err1X: nil},
	{name: "case2", fn: func() ([]byte, error) { return os.ReadFile("../testdata/case2.bin") }, err1X: nil},
	{name: "case3", fn: func() ([]byte, error) { return os.ReadFile("../testdata/case3.bin") }, err1X: nil},
	{name: "pngdata.001", fn: func() ([]byte, error) { return os.ReadFile("../testdata/pngdata.bin") }, err1X: nil},
	{name: "normcount2", fn: func() ([]byte, error) { return os.ReadFile("../testdata/normcount2.bin") }, err1X: nil},
}

type fuzzInput struct {
	name string
	fn   inputFn
}

// testfilesExtended is used for regression testing the decoder.
// These files are expected to fail, but not crash
var testfilesExtended []fuzzInput

func init() {
	data, err := os.ReadFile("testdata/regression.zip")
	if err != nil {
		panic(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		panic(err)
	}
	for _, tt := range zr.File {
		if tt.UncompressedSize64 == 0 {
			continue
		}
		rc, err := tt.Open()
		if err != nil {
			panic(err)
		}
		b, err := io.ReadAll(rc)
		if err != nil {
			panic(err)
		}
		testfilesExtended = append(testfilesExtended, fuzzInput{
			name: filepath.Base(tt.Name),
			fn: func() ([]byte, error) {
				return b, nil
			},
		})
	}
}

func TestCompressRegression(t *testing.T) {
	// Match the fuzz function
	var testInput = func(t *testing.T, data []byte) int {
		var enc Scratch
		enc.WantLogLess = 5
		comp, _, err := Compress1X(data, &enc)
		if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
			return 0
		}
		if err != nil {
			panic(err)
		}
		if len(comp) >= len(data)-len(data)>>enc.WantLogLess {
			panic(fmt.Errorf("too large output provided. got %d, but should be < %d", len(comp), len(data)-len(data)>>enc.WantLogLess))
		}

		dec, remain, err := ReadTable(comp, nil)
		if err != nil {
			panic(err)
		}
		out, err := dec.Decompress1X(remain)
		if err != nil {
			t.Error(err)
		}
		if !bytes.Equal(out, data) {
			t.Error("decompression 1x mismatch")
		}
		// Reuse as 4X
		enc.Reuse = ReusePolicyAllow
		comp, reUsed, err := Compress4X(data, &enc)
		if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
			return 0
		}
		if err != nil {
			panic(err)
		}
		if len(comp) >= len(data)-len(data)>>enc.WantLogLess {
			panic(fmt.Errorf("too large output provided. got %d, but should be < %d", len(comp), len(data)-len(data)>>enc.WantLogLess))
		}

		remain = comp
		if !reUsed {
			dec, remain, err = ReadTable(comp, dec)
			if err != nil {
				panic(err)
			}
		}
		out, err = dec.Decompress4X(remain, len(data))
		if err != nil {
			t.Error(err)
		}
		if !bytes.Equal(out, data) {
			t.Error("decompression 4x with reuse mismatch")
		}

		enc.Reuse = ReusePolicyNone
		comp, reUsed, err = Compress4X(data, &enc)
		if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
			return 0
		}
		if err != nil {
			panic(err)
		}
		if reUsed {
			panic("reused when asked not to")
		}
		if len(comp) >= len(data)-len(data)>>enc.WantLogLess {
			panic(fmt.Errorf("too large output provided. got %d, but should be < %d", len(comp), len(data)-len(data)>>enc.WantLogLess))
		}

		dec, remain, err = ReadTable(comp, dec)
		if err != nil {
			panic(err)
		}
		out, err = dec.Decompress4X(remain, len(data))
		if err != nil {
			t.Error(err)
		}
		if !bytes.Equal(out, data) {
			t.Error("decompression 4x mismatch")
		}

		// Reuse as 1X
		dec.Reuse = ReusePolicyAllow
		comp, reUsed, err = Compress1X(data, &enc)
		if err == ErrIncompressible || err == ErrUseRLE || err == ErrTooBig {
			return 0
		}
		if err != nil {
			panic(err)
		}
		if len(comp) >= len(data)-len(data)>>enc.WantLogLess {
			panic(fmt.Errorf("too large output provided. got %d, but should be < %d", len(comp), len(data)-len(data)>>enc.WantLogLess))
		}

		remain = comp
		if !reUsed {
			dec, remain, err = ReadTable(comp, dec)
			if err != nil {
				panic(err)
			}
		}
		out, err = dec.Decompress1X(remain)
		if err != nil {
			t.Error(err)
		}
		if !bytes.Equal(out, data) {
			t.Error("decompression 1x with reuse mismatch")
		}
		return 1
	}
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			testInput(t, buf0)
		})
	}
	for _, test := range testfilesExtended {
		t.Run(test.name, func(t *testing.T) {
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			testInput(t, buf0)
		})
	}
}

func TestCompress1X(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			var s Scratch
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			tbSz, dSz, reSz, _ := EstimateSizes(buf0, &s)
			b, re, err := Compress1X(buf0, &s)
			if err != test.err1X {
				t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err1X, err, err)
			}
			if err != nil {
				t.Log(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			min := s.minSize(len(buf0))
			if len(s.OutData) < min {
				t.Errorf("output data length (%d) below shannon limit (%d)", len(s.OutData), min)
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}
			t.Logf("Estimate: table %d, got %d, data %d, got %d, reuse: %d", tbSz, len(s.OutTable), dSz, len(s.OutData), reSz)
			t.Logf("%s: %d -> %d bytes (%.2f:1) re:%t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))
			s.Out = nil
			bRe, _, err := Compress1X(b, &s)
			if err == nil {
				t.Log("Could re-compress to", len(bRe))
			}
		})
	}
}

func TestCompress1XMustReuse(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			var s Scratch
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, re, err := Compress1X(buf0, &s)
			if err != test.err1X {
				t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err1X, err, err)
			}
			if err != nil {
				t.Log(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}

			min := s.minSize(len(buf0))
			if len(s.OutData) < min {
				t.Errorf("output data length (%d) below shannon limit (%d)", len(s.OutData), min)
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}
			t.Logf("%s: %d -> %d bytes (%.2f:1) re:%t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))
			table := s.OutTable
			prevTable := s.prevTable
			for i, v := range prevTable {
				// Clear unused sections for comparison
				if v.nBits == 0 {
					prevTable[i].val = 0
				}
			}
			b = s.OutData
			actl := s.actualTableLog

			// Use only the table data to recompress.
			s = Scratch{}
			s2 := &s
			s.Reuse = ReusePolicyMust
			s2, _, err = ReadTable(table, s2)
			if err != nil {
				t.Error("Could not read table", err)
				return
			}
			if !reflect.DeepEqual(prevTable, s2.prevTable) {
				t.Errorf("prevtable mismatch.\ngot %v\nwant %v", s2.prevTable, prevTable)
			}
			if actl != s.actualTableLog {
				t.Errorf("tablelog mismatch, want %d, got %d", actl, s.actualTableLog)
			}
			b2, reused, err := Compress1X(buf0, s2)
			if err != nil {
				t.Error("Could not re-compress with prev table", err)
			}
			if !reused {
				t.Error("didn't reuse...")
				return
			}
			if len(b2) != len(b) {
				t.Errorf("recompressed to different size, want %d, got %d", len(b), len(b2))
				return
			}

			if !bytes.Equal(b, b2) {
				for i := range b {
					if b[i] != b2[i] {
						t.Errorf("recompressed to different output. First mismatch at byte %d, (want %x != got %x)", i, b[i], b2[i])
						return
					}
				}
			}
		})
	}
}

func TestCompress4X(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			var s Scratch
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, re, err := Compress4X(buf0, &s)
			if err != test.err4X {
				t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err4X, err, err)
			}
			if err != nil {
				t.Log(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutTable) == 0 {
				t.Error("got no table definition")
			}
			if re {
				t.Error("claimed to have re-used.")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}

			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))
		})
	}
}

func TestCompress4XReuse(t *testing.T) {
	rng := rand.NewSource(0x1337)
	var s Scratch
	s.Reuse = ReusePolicyAllow
	for i := 0; i < 255; i++ {
		if testing.Short() && i > 10 {
			break
		}
		t.Run(fmt.Sprint("test-", i), func(t *testing.T) {
			buf0 := make([]byte, BlockSizeMax)
			for j := range buf0 {
				buf0[j] = byte(int64(i) + (rng.Int63() & 3))
			}
			tbSz, dSz, reSz, _ := EstimateSizes(buf0, &s)
			b, re, err := Compress4X(buf0, &s)
			if err != nil {
				t.Fatal(err)
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}
			if re {
				t.Error("claimed to have re-used. Unlikely.")
			}
			t.Logf("Estimate: table %d, got %d, data %d, got %d, reuse: %d", tbSz, len(s.OutTable), dSz, len(s.OutData), reSz)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", t.Name(), len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))
		})
	}
}

func TestCompress4XReuseActually(t *testing.T) {
	rng := rand.NewSource(0x1337)
	var s Scratch
	s.Reuse = ReusePolicyAllow
	for i := 0; i < 255; i++ {
		if testing.Short() && i > 10 {
			break
		}
		t.Run(fmt.Sprint("test-", i), func(t *testing.T) {
			buf0 := make([]byte, BlockSizeMax)
			for j := range buf0 {
				buf0[j] = byte(rng.Int63() & 7)
			}

			tbSz, dSz, reSz, _ := EstimateSizes(buf0, &s)
			b, re, err := Compress4X(buf0, &s)
			if err != nil {
				t.Fatal(err)
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}
			if re && i == 0 {
				t.Error("Claimed to have re-used on first loop.")
			}
			if !re && i > 0 {
				t.Error("Expected table to be reused")
			}
			t.Logf("Estimate: table %d, got %d, data %d, got %d, reuse: %d", tbSz, len(s.OutTable), dSz, len(s.OutData), reSz)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t (table: %d bytes)", t.Name(), len(buf0), len(b), float64(len(buf0))/float64(len(b)), re, len(s.OutTable))
		})
	}
}
func TestCompress1XReuse(t *testing.T) {
	for _, test := range testfiles {
		t.Run(test.name, func(t *testing.T) {
			var s Scratch
			buf0, err := test.fn()
			if err != nil {
				t.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b, _, err := Compress1X(buf0, &s)
			if err != test.err1X {
				t.Errorf("want error %v (%T), got %v (%T)", test.err1X, test.err1X, err, err)
			}
			if err != nil {
				t.Log(test.name, err.Error())
				return
			}
			if b == nil {
				t.Error("got no output")
				return
			}
			firstData := len(s.OutData)
			s.Reuse = ReusePolicyAllow
			tbSz, dSz, reSz, _ := EstimateSizes(buf0, &s)
			b, re, err := Compress1X(buf0, &s)
			if err != nil {
				t.Errorf("got secondary error %v (%T)", err, err)
				return
			}
			if !re {
				t.Error("Didn't re-use even if data was the same")
			}
			if len(s.OutTable) != 0 {
				t.Error("got table definition, don't want any")
			}
			if len(s.OutData) == 0 {
				t.Error("got no data output")
			}
			if len(b) != firstData {
				t.Errorf("data length did not match first: %d, second:%d", firstData, len(b))
			}
			t.Logf("Estimate: table %d, got %d, data %d, got %d, reuse: %d", tbSz, len(s.OutTable), dSz, len(s.OutData), reSz)
			t.Logf("%s: %d -> %d bytes (%.2f:1) %t", test.name, len(buf0), len(b), float64(len(buf0))/float64(len(b)), re)
		})
	}
}

func BenchmarkDeflate(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			dec, err := flate.NewWriter(io.Discard, flate.HuffmanOnly)
			if err != nil {
				b.Fatal(err)
			}
			if test.err1X != nil {
				b.Skip("skipping")
			}
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				dec.Reset(io.Discard)
				n, err := dec.Write(buf0)
				if err != nil {
					b.Fatal(err)
				}
				if n != len(buf0) {
					b.Fatal("mismatch", n, len(buf0))
				}
				dec.Close()
			}
		})
	}
}

func BenchmarkCompress1XReuseNone(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress1X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress1X(buf0, &s)
				if re {
					b.Fatal("reused")
				}
			}
		})
	}
}

func BenchmarkCompress1XReuseAllow(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyAllow
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress1X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress1X(buf0, &s)
				if !re {
					b.Fatal("not reused")
				}
			}
		})
	}
}

func BenchmarkCompress1XReusePrefer(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err1X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyPrefer
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress1X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress1X(buf0, &s)
				if !re {
					b.Fatal("not reused")
				}
			}
		})
	}
}

func BenchmarkCompress4XReuseNone(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress4X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress4X(buf0, &s)
				if re {
					b.Fatal("reused")
				}
			}
		})
	}
}

func BenchmarkCompress4XReuseAllow(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyAllow
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress4X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress4X(buf0, &s)
				if !re {
					b.Fatal("not reused")
				}
			}
		})
	}
}

func BenchmarkCompress4XReusePrefer(b *testing.B) {
	for _, tt := range testfiles {
		test := tt
		if test.err4X != nil {
			continue
		}
		b.Run(test.name, func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyPrefer
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			if len(buf0) > BlockSizeMax {
				buf0 = buf0[:BlockSizeMax]
			}
			_, _, err = Compress4X(buf0, &s)
			if err != test.err4X {
				b.Fatal("unexpected error:", err)
			}
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress4X(buf0, &s)
				if !re {
					b.Fatal("not reused")
				}
			}
		})
	}
}

func BenchmarkCompress1XSizes(b *testing.B) {
	test := testfiles[0]
	sizes := []int{1e2, 2e2, 5e2, 1e3, 5e3, 1e4, 5e4}
	for _, size := range sizes {
		b.Run(test.name+"-"+fmt.Sprint(size), func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			buf0 = buf0[:size]
			_, _, err = Compress1X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			//b.Log("Size:", len(o))
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress1X(buf0, &s)
				if re {
					b.Fatal("reused")
				}
			}
		})
	}
}

func BenchmarkCompress4XSizes(b *testing.B) {
	test := testfiles[0]
	sizes := []int{1e2, 2e2, 5e2, 1e3, 5e3, 1e4, 5e4}
	for _, size := range sizes {
		b.Run(test.name+"-"+fmt.Sprint(size), func(b *testing.B) {
			var s Scratch
			s.Reuse = ReusePolicyNone
			buf0, err := test.fn()
			if err != nil {
				b.Fatal(err)
			}
			buf0 = buf0[:size]
			_, _, err = Compress4X(buf0, &s)
			if err != test.err1X {
				b.Fatal("unexpected error:", err)
			}
			//b.Log("Size:", len(o))
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(buf0)))
			for i := 0; i < b.N; i++ {
				_, re, _ := Compress4X(buf0, &s)
				if re {
					b.Fatal("reused")
				}
			}
		})
	}
}
