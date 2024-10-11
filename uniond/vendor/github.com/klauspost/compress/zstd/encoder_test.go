// Copyright 2019+ Klaus Post. All rights reserved.
// License information can be found in the LICENSE file.
// Based on work by Yann Collet, released under BSD License.

package zstd

import (
	"bytes"
	"fmt"
	"io"
	"math/rand"
	"os"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/klauspost/compress/zip"
	"github.com/klauspost/compress/zstd/internal/xxhash"
)

var testWindowSizes = []int{MinWindowSize, 1 << 16, 1 << 22, 1 << 24}

type testEncOpt struct {
	name string
	o    []EOption
}

func getEncOpts(cMax int) []testEncOpt {
	var o []testEncOpt
	for level := speedNotSet + 1; level < speedLast; level++ {
		if isRaceTest && level >= SpeedBestCompression {
			break
		}
		for conc := 1; conc <= 4; conc *= 2 {
			for _, wind := range testWindowSizes {
				addOpt := func(name string, options ...EOption) {
					opts := append([]EOption(nil), WithEncoderLevel(level), WithEncoderConcurrency(conc), WithWindowSize(wind))
					name = fmt.Sprintf("%s-c%d-w%dk-%s", level.String(), conc, wind/1024, name)
					o = append(o, testEncOpt{name: name, o: append(opts, options...)})
				}
				addOpt("default")
				if testing.Short() {
					break
				}
				addOpt("nocrc", WithEncoderCRC(false))
				addOpt("lowmem", WithLowerEncoderMem(true))
				addOpt("alllit", WithAllLitEntropyCompression(true))
				addOpt("nolit", WithNoEntropyCompression(true))
				addOpt("pad1k", WithEncoderPadding(1024))
				addOpt("zerof", WithZeroFrames(true))
				addOpt("1seg", WithSingleSegment(true))
			}
			if testing.Short() && conc == 2 {
				break
			}
			if conc >= cMax {
				break
			}
		}
	}
	return o
}

func TestEncoder_EncodeAllSimple(t *testing.T) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		t.Fatal(err)
	}
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	in = append(in, in...)
	for _, opts := range getEncOpts(4) {
		t.Run(opts.name, func(t *testing.T) {
			runtime.GC()
			e, err := NewWriter(nil, opts.o...)
			if err != nil {
				t.Fatal(err)
			}
			defer e.Close()
			start := time.Now()
			dst := e.EncodeAll(in, nil)
			//t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
			mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
			t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

			decoded, err := dec.DecodeAll(dst, nil)
			if err != nil {
				t.Error(err, len(decoded))
			}
			if !bytes.Equal(decoded, in) {
				os.WriteFile("testdata/"+t.Name()+"-z000028.got", decoded, os.ModePerm)
				os.WriteFile("testdata/"+t.Name()+"-z000028.want", in, os.ModePerm)
				t.Fatal("Decoded does not match")
			}
			//t.Log("Encoded content matched")
		})
	}
}

func TestEncoder_EncodeAllConcurrent(t *testing.T) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		t.Fatal(err)
	}
	in = append(in, in...)

	// When running race no more than 8k goroutines allowed.
	n := 400 / runtime.GOMAXPROCS(0)
	if testing.Short() {
		n = 20 / runtime.GOMAXPROCS(0)
	}
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	for _, opts := range getEncOpts(2) {
		t.Run(opts.name, func(t *testing.T) {
			rng := rand.New(rand.NewSource(0x1337))
			e, err := NewWriter(nil, opts.o...)
			if err != nil {
				t.Fatal(err)
			}
			defer e.Close()
			var wg sync.WaitGroup
			wg.Add(n)
			for i := 0; i < n; i++ {
				in := in[rng.Int()&1023:]
				in = in[:rng.Intn(len(in))]
				go func() {
					defer wg.Done()
					dst := e.EncodeAll(in, nil)
					if len(dst) > e.MaxEncodedSize(len(in)) {
						t.Errorf("max encoded size for %v: got: %d, want max: %d", len(in), len(dst), e.MaxEncodedSize(len(in)))
					}
					//t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
					decoded, err := dec.DecodeAll(dst, nil)
					if err != nil {
						t.Error(err, len(decoded))
					}
					if !bytes.Equal(decoded, in) {
						//os.WriteFile("testdata/"+t.Name()+"-z000028.got", decoded, os.ModePerm)
						//os.WriteFile("testdata/"+t.Name()+"-z000028.want", in, os.ModePerm)
						t.Error("Decoded does not match")
						return
					}
				}()
			}
			wg.Wait()
			//t.Log("Encoded content matched.", n, "goroutines")
		})
	}
}

func TestEncoder_EncodeAllEncodeXML(t *testing.T) {
	f, err := os.Open("testdata/xml.zst")
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}
	if testing.Short() {
		in = in[:10000]
	}

	for level := speedNotSet + 1; level < speedLast; level++ {
		t.Run(level.String(), func(t *testing.T) {
			if isRaceTest && level >= SpeedBestCompression {
				t.SkipNow()
			}
			e, err := NewWriter(nil, WithEncoderLevel(level))
			if err != nil {
				t.Fatal(err)
			}
			defer e.Close()
			start := time.Now()
			dst := e.EncodeAll(in, nil)
			if len(dst) > e.MaxEncodedSize(len(in)) {
				t.Errorf("max encoded size for %v: got: %d, want max: %d", len(in), len(dst), e.MaxEncodedSize(len(in)))
			}
			//t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
			mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
			t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

			decoded, err := dec.DecodeAll(dst, nil)
			if err != nil {
				t.Error(err, len(decoded))
			}
			if !bytes.Equal(decoded, in) {
				os.WriteFile("testdata/"+t.Name()+"-xml.got", decoded, os.ModePerm)
				t.Error("Decoded does not match")
				return
			}
			//t.Log("Encoded content matched")
		})
	}
}

func TestEncoderRegression(t *testing.T) {
	defer timeout(4 * time.Minute)()
	data, err := os.ReadFile("testdata/comp-crashers.zip")
	if err != nil {
		t.Fatal(err)
	}
	// We can't close the decoder.
	dec, err := NewReader(nil)
	if err != nil {
		t.Error(err)
		return
	}
	defer dec.Close()
	for _, opts := range getEncOpts(2) {
		t.Run(opts.name, func(t *testing.T) {
			zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
			if err != nil {
				t.Fatal(err)
			}
			enc, err := NewWriter(
				nil,
				opts.o...,
			)
			if err != nil {
				t.Fatal(err)
			}
			defer enc.Close()

			for i, tt := range zr.File {
				if !strings.HasSuffix(t.Name(), "") {
					continue
				}
				if testing.Short() && i > 10 {
					break
				}

				t.Run(tt.Name, func(t *testing.T) {
					r, err := tt.Open()
					if err != nil {
						t.Error(err)
						return
					}
					in, err := io.ReadAll(r)
					if err != nil {
						t.Error(err)
					}
					encoded := enc.EncodeAll(in, nil)
					if len(encoded) > enc.MaxEncodedSize(len(in)) {
						t.Errorf("max encoded size for %v: got: %d, want max: %d", len(in), len(encoded), enc.MaxEncodedSize(len(in)))
					}
					// Usually too small...
					got, err := dec.DecodeAll(encoded, make([]byte, 0, len(in)))
					if err != nil {
						t.Logf("error: %v\nwant: %v\ngot:  %v", err, len(in), len(got))
						t.Fatal(err)
					}
					// Use the Writer
					var dst bytes.Buffer
					enc.ResetContentSize(&dst, int64(len(in)))
					_, err = enc.Write(in)
					if err != nil {
						t.Error(err)
					}
					err = enc.Close()
					if err != nil {
						t.Error(err)
					}
					encoded = dst.Bytes()
					if len(encoded) > enc.MaxEncodedSize(len(in)) {
						t.Errorf("max encoded size for %v: got: %d, want max: %d", len(in), len(encoded), enc.MaxEncodedSize(len(in)))
					}
					got, err = dec.DecodeAll(encoded, make([]byte, 0, len(in)/2))
					if err != nil {
						t.Logf("error: %v\nwant: %v\ngot:  %v", err, in, got)
						t.Error(err)
					}
				})
			}
		})
	}
}

func TestEncoder_EncodeAllTwain(t *testing.T) {
	in, err := os.ReadFile("../testdata/Mark.Twain-Tom.Sawyer.txt")
	if err != nil {
		t.Fatal(err)
	}
	testWindowSizes := testWindowSizes
	if testing.Short() {
		testWindowSizes = []int{1 << 20}
	}

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	for level := speedNotSet + 1; level < speedLast; level++ {
		t.Run(level.String(), func(t *testing.T) {
			if isRaceTest && level >= SpeedBestCompression {
				t.SkipNow()
			}
			for _, windowSize := range testWindowSizes {
				t.Run(fmt.Sprintf("window:%d", windowSize), func(t *testing.T) {
					e, err := NewWriter(nil, WithEncoderLevel(level), WithWindowSize(windowSize))
					if err != nil {
						t.Fatal(err)
					}
					defer e.Close()
					start := time.Now()
					dst := e.EncodeAll(in, nil)
					t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
					mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
					t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

					decoded, err := dec.DecodeAll(dst, nil)
					if err != nil {
						t.Error(err, len(decoded))
					}
					if !bytes.Equal(decoded, in) {
						os.WriteFile("testdata/"+t.Name()+"-Mark.Twain-Tom.Sawyer.txt.got", decoded, os.ModePerm)
						t.Fatal("Decoded does not match")
					}
					t.Log("Encoded content matched")
				})
			}
		})
	}
}

func TestEncoder_EncodeRLE(t *testing.T) {
	in := make([]byte, 1<<20)
	testWindowSizes := testWindowSizes
	if testing.Short() {
		testWindowSizes = []int{1 << 20}
	}

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	for level := speedNotSet + 1; level < speedLast; level++ {
		t.Run(level.String(), func(t *testing.T) {
			if isRaceTest && level >= SpeedBestCompression {
				t.SkipNow()
			}
			for _, windowSize := range testWindowSizes {
				t.Run(fmt.Sprintf("window:%d", windowSize), func(t *testing.T) {
					e, err := NewWriter(nil, WithEncoderLevel(level), WithWindowSize(windowSize))
					if err != nil {
						t.Fatal(err)
					}
					defer e.Close()
					start := time.Now()
					dst := e.EncodeAll(in, nil)
					t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
					mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
					t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

					decoded, err := dec.DecodeAll(dst, nil)
					if err != nil {
						t.Error(err, len(decoded))
					}
					if !bytes.Equal(decoded, in) {
						os.WriteFile("testdata/"+t.Name()+"-RLE.got", decoded, os.ModePerm)
						t.Fatal("Decoded does not match")
					}
					t.Log("Encoded content matched")
				})
			}
		})
	}
}

func TestEncoder_EncodeAllPi(t *testing.T) {
	in, err := os.ReadFile("../testdata/pi.txt")
	if err != nil {
		t.Fatal(err)
	}
	testWindowSizes := testWindowSizes
	if testing.Short() {
		testWindowSizes = []int{1 << 20}
	}

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	for level := speedNotSet + 1; level < speedLast; level++ {
		t.Run(level.String(), func(t *testing.T) {
			if isRaceTest && level >= SpeedBestCompression {
				t.SkipNow()
			}
			for _, windowSize := range testWindowSizes {
				t.Run(fmt.Sprintf("window:%d", windowSize), func(t *testing.T) {
					e, err := NewWriter(nil, WithEncoderLevel(level), WithWindowSize(windowSize))
					if err != nil {
						t.Fatal(err)
					}
					defer e.Close()
					start := time.Now()
					dst := e.EncodeAll(in, nil)
					t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
					mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
					t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

					decoded, err := dec.DecodeAll(dst, nil)
					if err != nil {
						t.Error(err, len(decoded))
					}
					if !bytes.Equal(decoded, in) {
						os.WriteFile("testdata/"+t.Name()+"-pi.txt.got", decoded, os.ModePerm)
						t.Fatal("Decoded does not match")
					}
					t.Log("Encoded content matched")
				})
			}
		})
	}
}

func TestWithEncoderPadding(t *testing.T) {
	n := 100
	if testing.Short() {
		n = 2
	}
	rng := rand.New(rand.NewSource(0x1337))
	d, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer d.Close()

	for i := 0; i < n; i++ {
		padding := (rng.Int() & 0xfff) + 1
		src := make([]byte, (rng.Int()&0xfffff)+1)
		for i := range src {
			src[i] = uint8(rng.Uint32()) & 7
		}
		e, err := NewWriter(nil, WithEncoderPadding(padding), WithEncoderCRC(rng.Uint32()&1 == 0))
		if err != nil {
			t.Fatal(err)
		}
		// Test the added padding is invisible.
		dst := e.EncodeAll(src, nil)
		if len(dst)%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, len(dst), len(dst)%padding)
		}
		got, err := d.DecodeAll(dst, nil)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(src, got) {
			t.Fatal("output mismatch")
		}
		// Test when we supply data as well.
		dst = e.EncodeAll(src, make([]byte, rng.Int()&255))
		if len(dst)%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, len(dst), len(dst)%padding)
		}

		// Test using the writer.
		var buf bytes.Buffer
		e.ResetContentSize(&buf, int64(len(src)))
		_, err = io.Copy(e, bytes.NewBuffer(src))
		if err != nil {
			t.Fatal(err)
		}
		err = e.Close()
		if err != nil {
			t.Fatal(err)
		}
		dst = buf.Bytes()
		if len(dst)%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, len(dst), len(dst)%padding)
		}
		// Test the added padding is invisible.
		got, err = d.DecodeAll(dst, nil)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(src, got) {
			t.Fatal("output mismatch")
		}
		// Try after reset
		buf.Reset()
		e.Reset(&buf)
		_, err = io.Copy(e, bytes.NewBuffer(src))
		if err != nil {
			t.Fatal(err)
		}
		err = e.Close()
		if err != nil {
			t.Fatal(err)
		}
		dst = buf.Bytes()
		if len(dst)%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, len(dst), len(dst)%padding)
		}
		// Test the added padding is invisible.
		got, err = d.DecodeAll(dst, nil)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(src, got) {
			t.Fatal("output mismatch")
		}
	}
}
func TestEncoder_EncoderXML(t *testing.T) {
	testEncoderRoundtrip(t, "./testdata/xml.zst", []byte{0x56, 0x54, 0x69, 0x8e, 0x40, 0x50, 0x11, 0xe})
	testEncoderRoundtripWriter(t, "./testdata/xml.zst", []byte{0x56, 0x54, 0x69, 0x8e, 0x40, 0x50, 0x11, 0xe})
}

func TestEncoder_EncoderTwain(t *testing.T) {
	testEncoderRoundtrip(t, "../testdata/Mark.Twain-Tom.Sawyer.txt", []byte{0x12, 0x1f, 0x12, 0x70, 0x79, 0x37, 0x1f, 0xc6})
	testEncoderRoundtripWriter(t, "../testdata/Mark.Twain-Tom.Sawyer.txt", []byte{0x12, 0x1f, 0x12, 0x70, 0x79, 0x37, 0x1f, 0xc6})
}

func TestEncoder_EncoderPi(t *testing.T) {
	testEncoderRoundtrip(t, "../testdata/pi.txt", []byte{0xe7, 0xe5, 0x25, 0x39, 0x92, 0xc7, 0x4a, 0xfb})
	testEncoderRoundtripWriter(t, "../testdata/pi.txt", []byte{0xe7, 0xe5, 0x25, 0x39, 0x92, 0xc7, 0x4a, 0xfb})
}

func TestEncoder_EncoderSilesia(t *testing.T) {
	testEncoderRoundtrip(t, "testdata/silesia.tar", []byte{0xa5, 0x5b, 0x5e, 0xe, 0x5e, 0xea, 0x51, 0x6b})
	testEncoderRoundtripWriter(t, "testdata/silesia.tar", []byte{0xa5, 0x5b, 0x5e, 0xe, 0x5e, 0xea, 0x51, 0x6b})
}

func TestEncoder_EncoderSimple(t *testing.T) {
	testEncoderRoundtrip(t, "testdata/z000028", []byte{0x8b, 0x2, 0x37, 0x70, 0x92, 0xb, 0x98, 0x95})
	testEncoderRoundtripWriter(t, "testdata/z000028", []byte{0x8b, 0x2, 0x37, 0x70, 0x92, 0xb, 0x98, 0x95})
}

func TestEncoder_EncoderHTML(t *testing.T) {
	testEncoderRoundtrip(t, "../testdata/html.txt", []byte{0x35, 0xa9, 0x5c, 0x37, 0x20, 0x9e, 0xc3, 0x37})
	testEncoderRoundtripWriter(t, "../testdata/html.txt", []byte{0x35, 0xa9, 0x5c, 0x37, 0x20, 0x9e, 0xc3, 0x37})
}

func TestEncoder_EncoderEnwik9(t *testing.T) {
	//testEncoderRoundtrip(t, "./testdata/enwik9.zst", []byte{0x28, 0xfa, 0xf4, 0x30, 0xca, 0x4b, 0x64, 0x12})
	//testEncoderRoundtripWriter(t, "./testdata/enwik9.zst", []byte{0x28, 0xfa, 0xf4, 0x30, 0xca, 0x4b, 0x64, 0x12})
}

// test roundtrip using io.ReaderFrom interface.
func testEncoderRoundtrip(t *testing.T, file string, wantCRC []byte) {
	for _, opt := range getEncOpts(1) {
		t.Run(opt.name, func(t *testing.T) {
			opt := opt
			//t.Parallel()
			f, err := os.Open(file)
			if err != nil {
				if os.IsNotExist(err) {
					t.Skip("No input file:", file)
					return
				}
				t.Fatal(err)
			}
			defer f.Close()
			if stat, err := f.Stat(); testing.Short() && err == nil {
				if stat.Size() > 10000 {
					t.SkipNow()
				}
			}
			input := io.Reader(f)
			if strings.HasSuffix(file, ".zst") {
				dec, err := NewReader(f)
				if err != nil {
					t.Fatal(err)
				}
				input = dec
				defer dec.Close()
			}

			pr, pw := io.Pipe()
			dec2, err := NewReader(pr)
			if err != nil {
				t.Fatal(err)
			}
			defer dec2.Close()

			enc, err := NewWriter(pw, opt.o...)
			if err != nil {
				t.Fatal(err)
			}
			defer enc.Close()
			var wantSize int64
			start := time.Now()
			go func() {
				n, err := enc.ReadFrom(input)
				if err != nil {
					t.Error(err)
					return
				}
				wantSize = n
				err = enc.Close()
				if err != nil {
					t.Error(err)
					return
				}
				pw.Close()
			}()
			var gotSize int64

			// Check CRC
			d := xxhash.New()
			if true {
				gotSize, err = io.Copy(d, dec2)
			} else {
				fout, err := os.Create(file + ".got")
				if err != nil {
					t.Fatal(err)
				}
				gotSize, err = io.Copy(io.MultiWriter(fout, d), dec2)
				if err != nil {
					t.Fatal(err)
				}
			}
			if wantSize != gotSize {
				t.Errorf("want size (%d) != got size (%d)", wantSize, gotSize)
			}
			if err != nil {
				t.Fatal(err)
			}
			if gotCRC := d.Sum(nil); len(wantCRC) > 0 && !bytes.Equal(gotCRC, wantCRC) {
				t.Errorf("crc mismatch %#v (want) != %#v (got)", wantCRC, gotCRC)
			} else if len(wantCRC) != 8 {
				t.Logf("Unable to verify CRC: %#v", gotCRC)
			} else {
				t.Logf("CRC Verified: %#v", gotCRC)
			}
			t.Log("Encoder len", wantSize)
			mbpersec := (float64(wantSize) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
			t.Logf("Encoded+Decoded %d bytes with %.2f MB/s", wantSize, mbpersec)
		})
	}
}

type writerWrapper struct {
	w io.Writer
}

func (w writerWrapper) Write(p []byte) (n int, err error) {
	return w.w.Write(p)
}

// test roundtrip using plain io.Writer interface.
func testEncoderRoundtripWriter(t *testing.T, file string, wantCRC []byte) {
	f, err := os.Open(file)
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("No input file:", file)
			return
		}
		t.Fatal(err)
	}
	defer f.Close()
	if stat, err := f.Stat(); testing.Short() && err == nil {
		if stat.Size() > 10000 {
			t.SkipNow()
		}
	}
	input := io.Reader(f)
	if strings.HasSuffix(file, ".zst") {
		dec, err := NewReader(f)
		if err != nil {
			t.Fatal(err)
		}
		input = dec
		defer dec.Close()
	}

	pr, pw := io.Pipe()
	dec2, err := NewReader(pr)
	if err != nil {
		t.Fatal(err)
	}
	defer dec2.Close()

	enc, err := NewWriter(pw, WithEncoderCRC(true))
	if err != nil {
		t.Fatal(err)
	}
	defer enc.Close()
	encW := writerWrapper{w: enc}
	var wantSize int64
	start := time.Now()
	go func() {
		n, err := io.CopyBuffer(encW, input, make([]byte, 1337))
		if err != nil {
			t.Error(err)
			return
		}
		wantSize = n
		err = enc.Close()
		if err != nil {
			t.Error(err)
			return
		}
		pw.Close()
	}()
	var gotSize int64

	// Check CRC
	d := xxhash.New()
	if true {
		gotSize, err = io.Copy(d, dec2)
	} else {
		fout, err := os.Create(file + ".got")
		if err != nil {
			t.Fatal(err)
		}
		gotSize, err = io.Copy(io.MultiWriter(fout, d), dec2)
		if err != nil {
			t.Fatal(err)
		}
	}
	if wantSize != gotSize {
		t.Errorf("want size (%d) != got size (%d)", wantSize, gotSize)
	}
	if err != nil {
		t.Fatal(err)
	}
	if gotCRC := d.Sum(nil); len(wantCRC) > 0 && !bytes.Equal(gotCRC, wantCRC) {
		t.Errorf("crc mismatch %#v (want) != %#v (got)", wantCRC, gotCRC)
	} else if len(wantCRC) != 8 {
		t.Logf("Unable to verify CRC: %#v", gotCRC)
	} else {
		t.Logf("CRC Verified: %#v", gotCRC)
	}
	t.Log("Fast Encoder len", wantSize)
	mbpersec := (float64(wantSize) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Encoded+Decoded %d bytes with %.2f MB/s", wantSize, mbpersec)
}

func TestEncoder_EncodeAllSilesia(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	in, err := os.ReadFile("testdata/silesia.tar")
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("Missing testdata/silesia.tar")
			return
		}
		t.Fatal(err)
	}

	var e Encoder
	start := time.Now()
	dst := e.EncodeAll(in, nil)
	t.Log("Fast Encoder len", len(in), "-> zstd len", len(dst))
	mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

	dec, err := NewReader(nil, WithDecoderMaxMemory(220<<20))
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	decoded, err := dec.DecodeAll(dst, nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/"+t.Name()+"-silesia.tar.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func TestEncoderReadFrom(t *testing.T) {
	buffer := bytes.NewBuffer(nil)
	encoder, err := NewWriter(buffer)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := encoder.ReadFrom(strings.NewReader("0")); err != nil {
		t.Fatal(err)
	}
	if err := encoder.Close(); err != nil {
		t.Fatal(err)
	}

	dec, _ := NewReader(nil)
	toDec := buffer.Bytes()
	toDec = append(toDec, toDec...)
	decoded, err := dec.DecodeAll(toDec, nil)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal([]byte("00"), decoded) {
		t.Logf("encoded: % x\n", buffer.Bytes())
		t.Fatalf("output mismatch, got %s", string(decoded))
	}
	dec.Close()
}

func TestInterleavedWriteReadFrom(t *testing.T) {
	var encoded bytes.Buffer

	enc, err := NewWriter(&encoded)
	if err != nil {
		t.Fatal(err)
	}

	if _, err := enc.Write([]byte("write1")); err != nil {
		t.Fatal(err)
	}
	if _, err := enc.Write([]byte("write2")); err != nil {
		t.Fatal(err)
	}
	if _, err := enc.ReadFrom(strings.NewReader("readfrom1")); err != nil {
		t.Fatal(err)
	}
	if _, err := enc.Write([]byte("write3")); err != nil {
		t.Fatal(err)
	}

	if err := enc.Close(); err != nil {
		t.Fatal(err)
	}

	dec, err := NewReader(&encoded)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	gotb, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}
	got := string(gotb)

	if want := "write1write2readfrom1write3"; got != want {
		t.Errorf("got decoded %q, want %q", got, want)
	}
}

func TestEncoder_EncodeAllEmpty(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	var in []byte

	for _, opt := range getEncOpts(1) {
		t.Run(opt.name, func(t *testing.T) {
			e, err := NewWriter(nil, opt.o...)
			if err != nil {
				t.Fatal(err)
			}
			defer e.Close()
			dst := e.EncodeAll(in, nil)
			t.Log("Block Encoder len", len(in), "-> zstd len", len(dst), dst)

			dec, err := NewReader(nil, WithDecoderMaxMemory(220<<20))
			if err != nil {
				t.Fatal(err)
			}
			defer dec.Close()
			decoded, err := dec.DecodeAll(dst, nil)
			if err != nil {
				t.Error(err, len(decoded))
			}
			if !bytes.Equal(decoded, in) {
				t.Fatal("Decoded does not match")
			}

			// Test buffer writer.
			var buf bytes.Buffer
			e.Reset(&buf)
			err = e.Close()
			if err != nil {
				t.Fatal(err)
			}
			dst = buf.Bytes()
			t.Log("Buffer Encoder len", len(in), "-> zstd len", len(dst))

			decoded, err = dec.DecodeAll(dst, nil)
			if err != nil {
				t.Error(err, len(decoded))
			}
			if !bytes.Equal(decoded, in) {
				t.Fatal("Decoded does not match")
			}

			t.Log("Encoded content matched")
		})
	}
}

func TestEncoder_EncodeAllEnwik9(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/enwik9.zst"
	f, err := os.Open(file)
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("To run extended tests, download http://mattmahoney.net/dc/enwik9.zip unzip it \n" +
				"compress it with 'zstd -15 -T0 enwik9' and place it in " + file)
		}
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}

	start := time.Now()
	e, err := NewWriter(nil)
	dst := e.EncodeAll(in, nil)
	if err != nil {
		t.Fatal(err)
	}
	t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
	mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)
	decoded, err := dec.DecodeAll(dst, nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		os.WriteFile("testdata/"+t.Name()+"-enwik9.got", decoded, os.ModePerm)
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")
}

func TestEncoder_EncoderStreamEnwik9(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/enwik9.zst"
	f, err := os.Open(file)
	if err != nil {
		if os.IsNotExist(err) {
			t.Skip("To run extended tests, download http://mattmahoney.net/dc/enwik9.zip unzip it \n" +
				"compress it with 'zstd -15 -T0 enwik9' and place it in " + file)
		}
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	in, err := io.ReadAll(dec)
	if err != nil {
		t.Fatal(err)
	}

	start := time.Now()
	var dst bytes.Buffer
	e, err := NewWriter(&dst)
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.Copy(e, bytes.NewBuffer(in))
	if err != nil {
		t.Fatal(err)
	}
	e.Close()
	t.Log("Full Encoder len", len(in), "-> zstd len", dst.Len())
	mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)
	if false {
		decoded, err := dec.DecodeAll(dst.Bytes(), nil)
		if err != nil {
			t.Error(err, len(decoded))
		}
		if !bytes.Equal(decoded, in) {
			os.WriteFile("testdata/"+t.Name()+"-enwik9.got", decoded, os.ModePerm)
			t.Fatal("Decoded does not match")
		}
		t.Log("Encoded content matched")
	}
}

func BenchmarkEncoder_EncodeAllXML(b *testing.B) {
	f, err := os.Open("testdata/xml.zst")
	if err != nil {
		b.Fatal(err)
	}
	defer f.Close()

	dec, err := NewReader(f)
	if err != nil {
		b.Fatal(err)
	}
	in, err := io.ReadAll(dec)
	if err != nil {
		b.Fatal(err)
	}
	dec.Close()

	enc, _ := NewWriter(nil, WithEncoderConcurrency(1))
	dst := enc.EncodeAll(in, nil)
	wantSize := len(dst)
	//b.Log("Output size:", len(dst))
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(in, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkEncoder_EncodeAllSimple(b *testing.B) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		b.Fatal(err)
	}

	for level := speedNotSet + 1; level < speedLast; level++ {
		b.Run(level.String(), func(b *testing.B) {
			enc, err := NewWriter(nil, WithEncoderConcurrency(1), WithEncoderLevel(level))
			if err != nil {
				b.Fatal(err)
			}
			defer enc.Close()
			dst := enc.EncodeAll(in, nil)
			wantSize := len(dst)
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(in)))
			for i := 0; i < b.N; i++ {
				dst := enc.EncodeAll(in, dst[:0])
				if len(dst) != wantSize {
					b.Fatal(len(dst), "!=", wantSize)
				}
			}
		})
	}
}

func BenchmarkEncoder_EncodeAllSimple4K(b *testing.B) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		b.Fatal(err)
	}
	in = in[:4096]

	for level := speedNotSet + 1; level < speedLast; level++ {
		b.Run(level.String(), func(b *testing.B) {
			enc, err := NewWriter(nil, WithEncoderConcurrency(1), WithEncoderLevel(level))
			if err != nil {
				b.Fatal(err)
			}
			defer enc.Close()
			dst := enc.EncodeAll(in, nil)
			wantSize := len(dst)
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(in)))
			for i := 0; i < b.N; i++ {
				dst := enc.EncodeAll(in, dst[:0])
				if len(dst) != wantSize {
					b.Fatal(len(dst), "!=", wantSize)
				}
			}
		})
	}
}

func BenchmarkEncoder_EncodeAllHTML(b *testing.B) {
	in, err := os.ReadFile("../testdata/html.txt")
	if err != nil {
		b.Fatal(err)
	}

	enc, _ := NewWriter(nil, WithEncoderConcurrency(1))
	dst := enc.EncodeAll(in, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(in, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkEncoder_EncodeAllTwain(b *testing.B) {
	in, err := os.ReadFile("../testdata/Mark.Twain-Tom.Sawyer.txt")
	if err != nil {
		b.Fatal(err)
	}

	enc, _ := NewWriter(nil, WithEncoderConcurrency(1))
	dst := enc.EncodeAll(in, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(in, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkEncoder_EncodeAllPi(b *testing.B) {
	in, err := os.ReadFile("../testdata/pi.txt")
	if err != nil {
		b.Fatal(err)
	}

	enc, _ := NewWriter(nil, WithEncoderConcurrency(1))
	dst := enc.EncodeAll(in, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(in)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(in, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkRandom4KEncodeAllFastest(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 4<<10)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}
	enc, _ := NewWriter(nil, WithEncoderLevel(SpeedFastest), WithEncoderConcurrency(1))
	defer enc.Close()
	dst := enc.EncodeAll(data, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(data)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(data, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkRandom10MBEncodeAllFastest(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 10<<20)
	rng.Read(data)
	enc, _ := NewWriter(nil, WithEncoderLevel(SpeedFastest), WithEncoderConcurrency(2))
	defer enc.Close()
	dst := enc.EncodeAll(data, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(data)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(data, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkRandom4KEncodeAllDefault(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 4<<10)
	rng.Read(data)
	enc, _ := NewWriter(nil, WithEncoderLevel(SpeedDefault), WithEncoderConcurrency(1))
	defer enc.Close()
	dst := enc.EncodeAll(data, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(data)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(data, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkRandomEncodeAllDefault(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 10<<20)
	rng.Read(data)
	enc, _ := NewWriter(nil, WithEncoderLevel(SpeedDefault), WithEncoderConcurrency(1))
	defer enc.Close()
	dst := enc.EncodeAll(data, nil)
	wantSize := len(dst)
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(int64(len(data)))
	for i := 0; i < b.N; i++ {
		dst := enc.EncodeAll(data, dst[:0])
		if len(dst) != wantSize {
			b.Fatal(len(dst), "!=", wantSize)
		}
	}
}

func BenchmarkRandom10MBEncoderFastest(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 10<<20)
	rng.Read(data)
	wantSize := int64(len(data))
	enc, _ := NewWriter(io.Discard, WithEncoderLevel(SpeedFastest))
	defer enc.Close()
	n, err := io.Copy(enc, bytes.NewBuffer(data))
	if err != nil {
		b.Fatal(err)
	}
	if n != wantSize {
		b.Fatal(n, "!=", wantSize)
	}
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(wantSize)
	for i := 0; i < b.N; i++ {
		enc.Reset(io.Discard)
		n, err := io.Copy(enc, bytes.NewBuffer(data))
		if err != nil {
			b.Fatal(err)
		}
		if n != wantSize {
			b.Fatal(n, "!=", wantSize)
		}
	}
}

func BenchmarkRandomEncoderDefault(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 10<<20)
	rng.Read(data)
	wantSize := int64(len(data))
	enc, _ := NewWriter(io.Discard, WithEncoderLevel(SpeedDefault))
	defer enc.Close()
	n, err := io.Copy(enc, bytes.NewBuffer(data))
	if err != nil {
		b.Fatal(err)
	}
	if n != wantSize {
		b.Fatal(n, "!=", wantSize)
	}
	b.ResetTimer()
	b.ReportAllocs()
	b.SetBytes(wantSize)
	for i := 0; i < b.N; i++ {
		enc.Reset(io.Discard)
		n, err := io.Copy(enc, bytes.NewBuffer(data))
		if err != nil {
			b.Fatal(err)
		}
		if n != wantSize {
			b.Fatal(n, "!=", wantSize)
		}
	}
}
