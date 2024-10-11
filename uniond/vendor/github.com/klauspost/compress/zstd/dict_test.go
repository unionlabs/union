package zstd

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"strings"
	"testing"

	"github.com/klauspost/compress/zip"
)

func TestDecoder_SmallDict(t *testing.T) {
	// All files have CRC
	zr := testCreateZipReader("testdata/dict-tests-small.zip", t)
	dicts := readDicts(t, zr)
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		t.Run("decodeall-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				t.Fatal(err)
			}
			_, err = dec.DecodeAll(in, got[:0])
			if err != nil {
				t.Fatal(err)
			}
		})
	}
}

func TestEncoder_SmallDict(t *testing.T) {
	// All files have CRC
	zr := testCreateZipReader("testdata/dict-tests-small.zip", t)
	var dicts [][]byte
	var encs []*Encoder
	var noDictEncs []*Encoder
	var encNames []string

	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			dicts = append(dicts, in)
			for level := SpeedFastest; level < speedLast; level++ {
				if isRaceTest && level >= SpeedBestCompression {
					break
				}
				enc, err := NewWriter(nil, WithEncoderConcurrency(1), WithEncoderDict(in), WithEncoderLevel(level), WithWindowSize(1<<17))
				if err != nil {
					t.Fatal(err)
				}
				encs = append(encs, enc)
				encNames = append(encNames, fmt.Sprint("level-", level.String(), "-dict-", len(dicts)))

				enc, err = NewWriter(nil, WithEncoderConcurrency(1), WithEncoderLevel(level), WithWindowSize(1<<17))
				if err != nil {
					t.Fatal(err)
				}
				noDictEncs = append(noDictEncs, enc)
			}
		}()
	}
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if testing.Short() && i > 100 {
			break
		}
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
		}
		defer r.Close()
		in, err := io.ReadAll(r)
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := dec.DecodeAll(in, nil)
		if err != nil {
			t.Fatal(err)
		}
		if testing.Short() && len(decoded) > 1000 {
			continue
		}

		t.Run("encodeall-"+tt.Name, func(t *testing.T) {
			// Attempt to compress with all dicts
			var b []byte
			var tmp []byte
			for i := range encs {
				i := i
				t.Run(encNames[i], func(t *testing.T) {
					b = encs[i].EncodeAll(decoded, b[:0])
					tmp, err = dec.DecodeAll(in, tmp[:0])
					if err != nil {
						t.Fatal(err)
					}
					if !bytes.Equal(tmp, decoded) {
						t.Fatal("output mismatch")
					}

					tmp = noDictEncs[i].EncodeAll(decoded, tmp[:0])

					if strings.Contains(t.Name(), "dictplain") && strings.Contains(t.Name(), "dict-1") {
						t.Log("reference:", len(in), "no dict:", len(tmp), "with dict:", len(b), "SAVED:", len(tmp)-len(b))
						// Check that we reduced this significantly
						if len(b) > 250 {
							t.Error("output was bigger than expected")
						}
					}
				})
			}
		})
		t.Run("stream-"+tt.Name, func(t *testing.T) {
			// Attempt to compress with all dicts
			var tmp []byte
			for i := range encs {
				i := i
				enc := encs[i]
				t.Run(encNames[i], func(t *testing.T) {
					var buf bytes.Buffer
					enc.ResetContentSize(&buf, int64(len(decoded)))
					_, err := enc.Write(decoded)
					if err != nil {
						t.Fatal(err)
					}
					err = enc.Close()
					if err != nil {
						t.Fatal(err)
					}
					tmp, err = dec.DecodeAll(buf.Bytes(), tmp[:0])
					if err != nil {
						t.Fatal(err)
					}
					if !bytes.Equal(tmp, decoded) {
						t.Fatal("output mismatch")
					}
					var buf2 bytes.Buffer
					noDictEncs[i].Reset(&buf2)
					noDictEncs[i].Write(decoded)
					noDictEncs[i].Close()

					if strings.Contains(t.Name(), "dictplain") && strings.Contains(t.Name(), "dict-1") {
						t.Log("reference:", len(in), "no dict:", buf2.Len(), "with dict:", buf.Len(), "SAVED:", buf2.Len()-buf.Len())
						// Check that we reduced this significantly
						if buf.Len() > 250 {
							t.Error("output was bigger than expected")
						}
					}
				})
			}
		})
	}
}

func TestEncoder_SmallDictFresh(t *testing.T) {
	// All files have CRC
	zr := testCreateZipReader("testdata/dict-tests-small.zip", t)
	var dicts [][]byte
	var encs []func() *Encoder
	var noDictEncs []*Encoder
	var encNames []string

	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			dicts = append(dicts, in)
			for level := SpeedFastest; level < speedLast; level++ {
				if isRaceTest && level >= SpeedBestCompression {
					break
				}
				level := level
				encs = append(encs, func() *Encoder {
					enc, err := NewWriter(nil, WithEncoderConcurrency(1), WithEncoderDict(in), WithEncoderLevel(level), WithWindowSize(1<<17))
					if err != nil {
						t.Fatal(err)
					}
					return enc
				})
				encNames = append(encNames, fmt.Sprint("level-", level.String(), "-dict-", len(dicts)))

				enc, err := NewWriter(nil, WithEncoderConcurrency(1), WithEncoderLevel(level), WithWindowSize(1<<17))
				if err != nil {
					t.Fatal(err)
				}
				noDictEncs = append(noDictEncs, enc)
			}
		}()
	}
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if testing.Short() && i > 100 {
			break
		}
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
		}
		defer r.Close()
		in, err := io.ReadAll(r)
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := dec.DecodeAll(in, nil)
		if err != nil {
			t.Fatal(err)
		}
		if testing.Short() && len(decoded) > 1000 {
			continue
		}

		t.Run("encodeall-"+tt.Name, func(t *testing.T) {
			// Attempt to compress with all dicts
			var b []byte
			var tmp []byte
			for i := range encs {
				i := i
				t.Run(encNames[i], func(t *testing.T) {
					enc := encs[i]()
					defer enc.Close()
					b = enc.EncodeAll(decoded, b[:0])
					tmp, err = dec.DecodeAll(in, tmp[:0])
					if err != nil {
						t.Fatal(err)
					}
					if !bytes.Equal(tmp, decoded) {
						t.Fatal("output mismatch")
					}

					tmp = noDictEncs[i].EncodeAll(decoded, tmp[:0])

					if strings.Contains(t.Name(), "dictplain") && strings.Contains(t.Name(), "dict-1") {
						t.Log("reference:", len(in), "no dict:", len(tmp), "with dict:", len(b), "SAVED:", len(tmp)-len(b))
						// Check that we reduced this significantly
						if len(b) > 250 {
							t.Error("output was bigger than expected")
						}
					}
				})
			}
		})
		t.Run("stream-"+tt.Name, func(t *testing.T) {
			// Attempt to compress with all dicts
			var tmp []byte
			for i := range encs {
				i := i
				t.Run(encNames[i], func(t *testing.T) {
					enc := encs[i]()
					defer enc.Close()
					var buf bytes.Buffer
					enc.ResetContentSize(&buf, int64(len(decoded)))
					_, err := enc.Write(decoded)
					if err != nil {
						t.Fatal(err)
					}
					err = enc.Close()
					if err != nil {
						t.Fatal(err)
					}
					tmp, err = dec.DecodeAll(buf.Bytes(), tmp[:0])
					if err != nil {
						t.Fatal(err)
					}
					if !bytes.Equal(tmp, decoded) {
						t.Fatal("output mismatch")
					}
					var buf2 bytes.Buffer
					noDictEncs[i].Reset(&buf2)
					noDictEncs[i].Write(decoded)
					noDictEncs[i].Close()

					if strings.Contains(t.Name(), "dictplain") && strings.Contains(t.Name(), "dict-1") {
						t.Log("reference:", len(in), "no dict:", buf2.Len(), "with dict:", buf.Len(), "SAVED:", buf2.Len()-buf.Len())
						// Check that we reduced this significantly
						if buf.Len() > 250 {
							t.Error("output was bigger than expected")
						}
					}
				})
			}
		})
	}
}

func benchmarkEncodeAllLimitedBySize(b *testing.B, lowerLimit int, upperLimit int) {
	zr := testCreateZipReader("testdata/dict-tests-small.zip", b)
	t := testing.TB(b)

	var dicts [][]byte
	var encs []*Encoder
	var encNames []string

	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			dicts = append(dicts, in)
			for level := SpeedFastest; level < speedLast; level++ {
				enc, err := NewWriter(nil, WithEncoderDict(in), WithEncoderLevel(level))
				if err != nil {
					t.Fatal(err)
				}
				encs = append(encs, enc)
				encNames = append(encNames, fmt.Sprint("level-", level.String(), "-dict-", len(dicts)))
			}
		}()
	}
	const nPer = int(speedLast - SpeedFastest)
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()

	tested := make(map[int]struct{})
	for j, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
		}
		defer r.Close()
		in, err := io.ReadAll(r)
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := dec.DecodeAll(in, nil)
		if err != nil {
			t.Fatal(err)
		}

		// Only test each size once
		if _, ok := tested[len(decoded)]; ok {
			continue
		}
		tested[len(decoded)] = struct{}{}

		if len(decoded) < lowerLimit {
			continue
		}

		if upperLimit > 0 && len(decoded) > upperLimit {
			continue
		}

		for i := range encs {
			// Only do 1 dict (4 encoders) for now.
			if i == nPer-1 {
				break
			}
			// Attempt to compress with all dicts
			encIdx := (i + j*nPer) % len(encs)
			enc := encs[encIdx]
			b.Run(fmt.Sprintf("length-%d-%s", len(decoded), encNames[encIdx]), func(b *testing.B) {
				b.RunParallel(func(pb *testing.PB) {
					dst := make([]byte, 0, len(decoded)+10)
					b.SetBytes(int64(len(decoded)))
					b.ResetTimer()
					b.ReportAllocs()
					for pb.Next() {
						dst = enc.EncodeAll(decoded, dst[:0])
					}
				})
			})
		}
	}
}

func BenchmarkEncodeAllDict0_1024(b *testing.B) {
	benchmarkEncodeAllLimitedBySize(b, 0, 1024)
}

func BenchmarkEncodeAllDict1024_8192(b *testing.B) {
	benchmarkEncodeAllLimitedBySize(b, 1024, 8192)
}

func BenchmarkEncodeAllDict8192_16384(b *testing.B) {
	benchmarkEncodeAllLimitedBySize(b, 8192, 16384)
}

func BenchmarkEncodeAllDict16384_65536(b *testing.B) {
	benchmarkEncodeAllLimitedBySize(b, 16384, 65536)
}

func BenchmarkEncodeAllDict65536_0(b *testing.B) {
	benchmarkEncodeAllLimitedBySize(b, 65536, 0)
}

func TestDecoder_MoreDicts(t *testing.T) {
	// All files have CRC
	// https://files.klauspost.com/compress/zstd-dict-tests.zip
	fn := "testdata/zstd-dict-tests.zip"
	data, err := os.ReadFile(fn)
	if err != nil {
		t.Skip("extended dict test not found.")
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}

	var dicts [][]byte
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			dicts = append(dicts, in)
		}()
	}
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		if testing.Short() && i > 50 {
			continue
		}
		t.Run("decodeall-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				t.Fatal(err)
			}
			_, err = dec.DecodeAll(in, got[:0])
			if err != nil {
				t.Fatal(err)
			}
		})
	}
}

func TestDecoder_MoreDicts2(t *testing.T) {
	// All files have CRC
	// https://files.klauspost.com/compress/zstd-dict-tests.zip
	fn := "testdata/zstd-dict-tests.zip"
	data, err := os.ReadFile(fn)
	if err != nil {
		t.Skip("extended dict test not found.")
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}

	var dicts [][]byte
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			dicts = append(dicts, in)
		}()
	}
	dec, err := NewReader(nil, WithDecoderConcurrency(2), WithDecoderDicts(dicts...))
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		if testing.Short() && i > 50 {
			continue
		}
		t.Run("decodeall-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				t.Fatal(err)
			}
			_, err = dec.DecodeAll(in, got[:0])
			if err != nil {
				t.Fatal(err)
			}
		})
	}
}

func readDicts(tb testing.TB, zr *zip.Reader) [][]byte {
	var dicts [][]byte
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".dict") {
			continue
		}
		func() {
			r, err := tt.Open()
			if err != nil {
				tb.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				tb.Fatal(err)
			}
			dicts = append(dicts, in)
		}()
	}
	return dicts
}

// Test decoding of zstd --patch-from output.
func TestDecoderRawDict(t *testing.T) {
	t.Parallel()

	dict, err := os.ReadFile("testdata/delta/source.txt")
	if err != nil {
		t.Fatal(err)
	}

	delta, err := os.Open("testdata/delta/target.txt.zst")
	if err != nil {
		t.Fatal(err)
	}
	defer delta.Close()

	dec, err := NewReader(delta, WithDecoderDictRaw(0, dict))
	if err != nil {
		t.Fatal(err)
	}

	out, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}

	ref, err := os.ReadFile("testdata/delta/target.txt")
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(out, ref) {
		t.Errorf("mismatch: got %q, wanted %q", out, ref)
	}
}
