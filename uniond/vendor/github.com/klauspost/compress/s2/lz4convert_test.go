// Copyright (c) 2022 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"path/filepath"
	"sort"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
	"github.com/klauspost/compress/internal/lz4ref"
	"github.com/klauspost/compress/internal/snapref"
)

func TestLZ4Converter_ConvertBlock(t *testing.T) {
	for _, tf := range testFiles {
		t.Run(tf.label, func(t *testing.T) {
			if err := downloadBenchmarkFiles(t, tf.filename); err != nil {
				t.Fatalf("failed to download testdata: %s", err)
			}

			bDir := filepath.FromSlash(*benchdataDir)
			data := readFile(t, filepath.Join(bDir, tf.filename))
			if n := tf.sizeLimit; 0 < n && n < len(data) {
				data = data[:n]
			}

			lz4Data := make([]byte, lz4ref.CompressBlockBound(len(data)))
			n, err := lz4ref.CompressBlock(data, lz4Data)
			if err != nil {
				t.Fatal(err)
			}
			if n == 0 {
				t.Skip("incompressible")
				return
			}
			t.Log("input size:", len(data))
			t.Log("lz4 size:", n)
			lz4Data = lz4Data[:n]
			s2Dst := make([]byte, binary.MaxVarintLen32, MaxEncodedLen(len(data)))
			s2Dst = s2Dst[:binary.PutUvarint(s2Dst, uint64(len(data)))]
			hdr := len(s2Dst)

			conv := LZ4Converter{}

			szS := 0
			out, n, err := conv.ConvertBlockSnappy(s2Dst, lz4Data)
			if err != nil {
				t.Fatal(err)
			}
			if n != len(data) {
				t.Fatalf("length mismatch: want %d, got %d", len(data), n)
			}
			szS = len(out) - hdr
			t.Log("lz4->snappy size:", szS)

			decom, err := snapref.Decode(nil, out)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(decom, data) {
				t.Errorf("output mismatch")
			}

			sz := 0
			out, n, err = conv.ConvertBlock(s2Dst, lz4Data)
			if err != nil {
				t.Fatal(err)
			}
			if n != len(data) {
				t.Fatalf("length mismatch: want %d, got %d", len(data), n)
			}
			sz = len(out) - hdr
			t.Log("lz4->s2 size:", sz)

			decom, err = Decode(nil, out)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(decom, data) {
				t.Errorf("output mismatch")
			}

			out2 := Encode(s2Dst[:0], data)
			sz2 := len(out2) - hdr
			t.Log("s2 (default) size:", sz2)

			out2 = EncodeBetter(s2Dst[:0], data)
			sz3 := len(out2) - hdr
			t.Log("s2 (better) size:", sz3)

			t.Log("lz4 -> s2 bytes saved:", len(lz4Data)-sz)
			t.Log("lz4 -> snappy bytes saved:", len(lz4Data)-szS)
			t.Log("data -> s2 (default) bytes saved:", len(lz4Data)-sz2)
			t.Log("data -> s2 (better) bytes saved:", len(lz4Data)-sz3)
			t.Log("direct data -> s2 (default) compared to converted from lz4:", sz-sz2)
			t.Log("direct data -> s2 (better) compared to converted from lz4:", sz-sz3)
		})
	}
}

func TestLZ4Converter_ConvertBlockSingle(t *testing.T) {
	// Mainly for analyzing fuzz failures.
	lz4Data := []byte{0x6f, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x1, 0x0, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x30, 0xf, 0x30, 0x30, 0xe4, 0x1f, 0x30, 0x30, 0x30, 0xff, 0xff, 0x30, 0x2f, 0x30, 0x30, 0x30, 0x30, 0xcf, 0x7f, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0xaf, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0xff, 0xff, 0x30, 0xf, 0x30, 0x30, 0x30, 0x1f, 0x30, 0x30, 0x30, 0xff, 0xff, 0x30, 0x30, 0x30, 0x30, 0x30}
	lz4Decoded := make([]byte, 4<<20)
	lzN := lz4ref.UncompressBlock(lz4Decoded, lz4Data)
	data := lz4Decoded
	if lzN < 0 {
		t.Skip(lzN)
	} else {
		data = data[:lzN]
	}
	t.Log("uncompressed size:", lzN)
	t.Log("lz4 size:", len(lz4Data))
	s2Dst := make([]byte, binary.MaxVarintLen32, MaxEncodedLen(len(data)))
	s2Dst = s2Dst[:binary.PutUvarint(s2Dst, uint64(len(data)))]
	hdr := len(s2Dst)

	conv := LZ4Converter{}

	szS := 0
	out, n, err := conv.ConvertBlockSnappy(s2Dst, lz4Data)
	if err != nil {
		t.Fatal(err)
	}
	if n != len(data) {
		t.Fatalf("length mismatch: want %d, got %d", len(data), n)
	}
	szS = len(out) - hdr
	t.Log("lz4->snappy size:", szS)

	decom, err := snapref.Decode(nil, out)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decom, data) {
		t.Errorf("output mismatch")
	}

	sz := 0
	out, n, err = conv.ConvertBlock(s2Dst, lz4Data)
	if err != nil {
		t.Fatal(err)
	}
	if n != len(data) {
		t.Fatalf("length mismatch: want %d, got %d", len(data), n)
	}
	sz = len(out) - hdr
	t.Log("lz4->s2 size:", sz)

	decom, err = Decode(nil, out)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(decom, data) {
		t.Errorf("output mismatch")
	}

	out2 := Encode(s2Dst[:0], data)
	sz2 := len(out2) - hdr
	t.Log("s2 (default) size:", sz2)

	out2 = EncodeBetter(s2Dst[:0], data)
	sz3 := len(out2) - hdr
	t.Log("s2 (better) size:", sz3)

	t.Log("lz4 -> s2 bytes saved:", len(lz4Data)-sz)
	t.Log("lz4 -> snappy bytes saved:", len(lz4Data)-szS)
	t.Log("data -> s2 (default) bytes saved:", len(lz4Data)-sz2)
	t.Log("data -> s2 (better) bytes saved:", len(lz4Data)-sz3)
	t.Log("direct data -> s2 (default) compared to converted from lz4:", sz-sz2)
	t.Log("direct data -> s2 (better) compared to converted from lz4:", sz-sz3)
}

func BenchmarkLZ4Converter_ConvertBlock(b *testing.B) {
	for _, tf := range testFiles {
		b.Run(tf.label, func(b *testing.B) {
			if err := downloadBenchmarkFiles(b, tf.filename); err != nil {
				b.Fatalf("failed to download testdata: %s", err)
			}

			bDir := filepath.FromSlash(*benchdataDir)
			data := readFile(b, filepath.Join(bDir, tf.filename))
			if n := tf.sizeLimit; 0 < n && n < len(data) {
				data = data[:n]
			}

			lz4Data := make([]byte, lz4ref.CompressBlockBound(len(data)))
			n, err := lz4ref.CompressBlock(data, lz4Data)
			if err != nil {
				b.Fatal(err)
			}
			if n == 0 {
				b.Skip("incompressible")
				return
			}
			lz4Data = lz4Data[:n]
			s2Dst := make([]byte, MaxEncodedLen(len(data)))
			conv := LZ4Converter{}
			b.ReportAllocs()
			b.ResetTimer()
			b.SetBytes(int64(len(data)))
			sz := 0
			for i := 0; i < b.N; i++ {
				out, n, err := conv.ConvertBlock(s2Dst[:0], lz4Data)
				if err != nil {
					b.Fatal(err)
				}
				if n != len(data) {
					b.Fatalf("length mismatch: want %d, got %d", len(data), n)
				}
				sz = len(out)
			}
			b.ReportMetric(float64(len(lz4Data)-sz), "b_saved")
		})
	}
}

func BenchmarkLZ4Converter_ConvertBlockSnappy(b *testing.B) {
	for _, tf := range testFiles {
		b.Run(tf.label, func(b *testing.B) {
			if err := downloadBenchmarkFiles(b, tf.filename); err != nil {
				b.Fatalf("failed to download testdata: %s", err)
			}

			bDir := filepath.FromSlash(*benchdataDir)
			data := readFile(b, filepath.Join(bDir, tf.filename))
			if n := tf.sizeLimit; 0 < n && n < len(data) {
				data = data[:n]
			}

			lz4Data := make([]byte, lz4ref.CompressBlockBound(len(data)))
			n, err := lz4ref.CompressBlock(data, lz4Data)
			if err != nil {
				b.Fatal(err)
			}
			if n == 0 {
				b.Skip("incompressible")
				return
			}
			lz4Data = lz4Data[:n]
			s2Dst := make([]byte, MaxEncodedLen(len(data)))
			conv := LZ4Converter{}
			b.ReportAllocs()
			b.ResetTimer()
			b.SetBytes(int64(len(data)))
			sz := 0
			for i := 0; i < b.N; i++ {
				out, n, err := conv.ConvertBlockSnappy(s2Dst[:0], lz4Data)
				if err != nil {
					b.Fatal(err)
				}
				if n != len(data) {
					b.Fatalf("length mismatch: want %d, got %d", len(data), n)
				}
				sz = len(out)
			}
			b.ReportMetric(float64(len(lz4Data)-sz), "b_saved")
		})
	}
}

func BenchmarkLZ4Converter_ConvertBlockParallel(b *testing.B) {
	sort.Slice(testFiles, func(i, j int) bool {
		return testFiles[i].filename < testFiles[j].filename
	})
	for _, tf := range testFiles {
		b.Run(tf.filename, func(b *testing.B) {
			if err := downloadBenchmarkFiles(b, tf.filename); err != nil {
				b.Fatalf("failed to download testdata: %s", err)
			}

			bDir := filepath.FromSlash(*benchdataDir)
			data := readFile(b, filepath.Join(bDir, tf.filename))

			lz4Data := make([]byte, lz4ref.CompressBlockBound(len(data)))
			n, err := lz4ref.CompressBlock(data, lz4Data)
			if err != nil {
				b.Fatal(err)
			}
			if n == 0 {
				b.Skip("incompressible")
				return
			}
			lz4Data = lz4Data[:n]
			conv := LZ4Converter{}
			b.ReportAllocs()
			b.ResetTimer()
			b.SetBytes(int64(len(data)))
			b.RunParallel(func(pb *testing.PB) {
				s2Dst := make([]byte, MaxEncodedLen(len(data)))
				for pb.Next() {
					_, n, err := conv.ConvertBlock(s2Dst[:0], lz4Data)
					if err != nil {
						b.Fatal(err)
					}
					if n != len(data) {
						b.Fatalf("length mismatch: want %d, got %d", len(data), n)
					}
				}
			})
		})
	}
}
func BenchmarkCompressBlockReference(b *testing.B) {
	b.Skip("Only reference for BenchmarkLZ4Converter_ConvertBlock")
	for _, tf := range testFiles {
		b.Run(tf.label, func(b *testing.B) {
			if err := downloadBenchmarkFiles(b, tf.filename); err != nil {
				b.Fatalf("failed to download testdata: %s", err)
			}
			bDir := filepath.FromSlash(*benchdataDir)
			data := readFile(b, filepath.Join(bDir, tf.filename))
			if n := tf.sizeLimit; 0 < n && n < len(data) {
				data = data[:n]
			}

			lz4Data := make([]byte, lz4ref.CompressBlockBound(len(data)))
			n, err := lz4ref.CompressBlock(data, lz4Data)
			if err != nil {
				b.Fatal(err)
			}
			if n == 0 {
				b.Skip("incompressible")
				return
			}
			s2Dst := make([]byte, MaxEncodedLen(len(data)))

			b.Run("default", func(b *testing.B) {
				b.ReportAllocs()
				b.ResetTimer()
				b.SetBytes(int64(len(data)))
				for i := 0; i < b.N; i++ {
					_ = Encode(s2Dst, data)
				}
			})
			b.Run("better", func(b *testing.B) {
				b.ReportAllocs()
				b.ResetTimer()
				b.SetBytes(int64(len(data)))
				for i := 0; i < b.N; i++ {
					_ = EncodeBetter(s2Dst, data)
				}
			})
		})
	}
}

func FuzzLZ4Block(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/fuzz/lz4-convert-corpus-raw.zip", fuzz.TypeRaw, false)
	fuzz.AddFromZip(f, "testdata/fuzz/FuzzLZ4Block.zip", fuzz.TypeGoFuzz, false)
	// Fuzzing tweaks:
	const (
		// Max input size:
		maxSize = 1 << 20
	)

	conv := LZ4Converter{}

	f.Fuzz(func(t *testing.T, data []byte) {
		if len(data) > maxSize || len(data) == 0 {
			return
		}

		lz4Decoded := make([]byte, len(data)*2+65536)
		lzN := lz4ref.UncompressBlock(lz4Decoded, data)
		converted := make([]byte, len(data)*2+4096)
		hdr := 0
		if lzN >= 0 {
			hdr = binary.PutUvarint(converted, uint64(lzN))
		}

		cV, cN, cErr := conv.ConvertBlock(converted[:hdr], data)
		if lzN >= 0 && cErr == nil {
			if cN != lzN {
				panic(fmt.Sprintf("uncompressed lz4 size: %d, s2 size: %d", lzN, cN))
			}
			lz4Decoded = lz4Decoded[:lzN]
			// Both success
			s2Dec, err := Decode(nil, cV)
			if err != nil {
				panic(fmt.Sprintf("block: %#v: %v", cV, err))
			}
			if !bytes.Equal(lz4Decoded, s2Dec) {
				panic("output mismatch")
			}
			return
		}
		if lzN >= 0 && cErr != nil {
			panic(fmt.Sprintf("lz4 returned %d, conversion returned %v\n lz4 block: %#v", lzN, cErr, data))
		}
		if lzN < 0 && cErr == nil {
			// We might get an error if there isn't enough space to decompress the LZ4 content.
			// Try with the decompressed size from conversion.
			lz4Decoded = make([]byte, cN)
			lzN = lz4ref.UncompressBlock(lz4Decoded, data)
			if lzN < 0 {
				panic(fmt.Sprintf("lz4 returned %d, conversion returned %v, input: %#v", lzN, cErr, data))
			}
			// Compare now that we have success...
			lz4Decoded = lz4Decoded[:lzN]

			// Re-add correct header.
			tmp := make([]byte, binary.MaxVarintLen32+len(cV))
			hdr = binary.PutUvarint(tmp, uint64(cN))
			cV = append(tmp[:hdr], cV...)

			// Both success
			s2Dec, err := Decode(nil, cV)
			if err != nil {
				panic(fmt.Sprintf("block: %#v: %v\ninput: %#v\n", cV, err, data))
			}
			if !bytes.Equal(lz4Decoded, s2Dec) {
				panic("output mismatch")
			}
		}
		// Snappy....
		hdr = binary.PutUvarint(converted, uint64(lzN))
		cV, cN, cErr = conv.ConvertBlockSnappy(converted[:hdr], data)
		if lzN >= 0 && cErr == nil {
			if cN != lzN {
				panic(fmt.Sprintf("uncompressed lz4 size: %d, s2 size: %d", lzN, cN))
			}
			lz4Decoded = lz4Decoded[:lzN]
			// Both success
			s2Dec, err := snapref.Decode(nil, cV)
			if err != nil {
				panic(fmt.Sprintf("block: %#v: %v", cV, err))
			}
			if !bytes.Equal(lz4Decoded, s2Dec) {
				panic("output mismatch")
			}
			return
		}
		// Snappy can expand a lot due to 64 byte match length limit
		if lzN >= 0 && cErr != ErrDstTooSmall {
			panic(fmt.Sprintf("lz4 returned %d, conversion returned %v\n lz4 block: %#v", lzN, cErr, data))
		}
		if lzN < 0 && cErr == nil {
			panic(fmt.Sprintf("lz4 returned %d, conversion returned %v, input: %#v", lzN, cErr, data))
		}
	})
}
