// Copyright 2012 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package flate

import (
	"archive/zip"
	"bytes"
	"compress/flate"
	"fmt"
	"io"
	"math"
	"math/rand"
	"os"
	"runtime"
	"strconv"
	"strings"
	"testing"
)

func TestWriterMemUsage(t *testing.T) {
	testMem := func(t *testing.T, fn func()) {
		var before, after runtime.MemStats
		runtime.GC()
		runtime.ReadMemStats(&before)
		fn()
		runtime.GC()
		runtime.ReadMemStats(&after)
		t.Logf("%s: Memory Used: %dKB, %d allocs", t.Name(), (after.HeapInuse-before.HeapInuse)/1024, after.HeapObjects-before.HeapObjects)
	}
	data := make([]byte, 100000)
	t.Run("stateless", func(t *testing.T) {
		testMem(t, func() {
			StatelessDeflate(io.Discard, data, false, nil)
		})
	})
	for level := HuffmanOnly; level <= BestCompression; level++ {
		t.Run(fmt.Sprint("level-", level), func(t *testing.T) {
			var zr *Writer
			var err error
			testMem(t, func() {
				zr, err = NewWriter(io.Discard, level)
				if err != nil {
					t.Fatal(err)
				}
				zr.Write(data)
			})
			zr.Close()
		})
	}
	for level := HuffmanOnly; level <= BestCompression; level++ {
		t.Run(fmt.Sprint("stdlib-", level), func(t *testing.T) {
			var zr *flate.Writer
			var err error
			testMem(t, func() {
				zr, err = flate.NewWriter(io.Discard, level)
				if err != nil {
					t.Fatal(err)
				}
				zr.Write(data)
			})
			zr.Close()
		})
	}
}

func TestWriterRegression(t *testing.T) {
	data, err := os.ReadFile("testdata/regression.zip")
	if err != nil {
		t.Fatal(err)
	}
	for level := HuffmanOnly; level <= BestCompression; level++ {
		t.Run(fmt.Sprint("level_", level), func(t *testing.T) {
			zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
			if err != nil {
				t.Fatal(err)
			}

			for _, tt := range zr.File {
				if !strings.HasSuffix(t.Name(), "") {
					continue
				}

				t.Run(tt.Name, func(t *testing.T) {
					if testing.Short() && tt.FileInfo().Size() > 10000 {
						t.SkipNow()
					}
					r, err := tt.Open()
					if err != nil {
						t.Error(err)
						return
					}
					in, err := io.ReadAll(r)
					if err != nil {
						t.Error(err)
					}
					msg := "level " + strconv.Itoa(level) + ":"
					buf := new(bytes.Buffer)
					fw, err := NewWriter(buf, level)
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					n, err := fw.Write(in)
					if n != len(in) {
						t.Fatal(msg + "short write")
					}
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					err = fw.Close()
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					fr1 := NewReader(buf)
					data2, err := io.ReadAll(fr1)
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					if !bytes.Equal(in, data2) {
						t.Fatal(msg + "not equal")
					}
					// Do it again...
					msg = "level " + strconv.Itoa(level) + " (reset):"
					buf.Reset()
					fw.Reset(buf)
					n, err = fw.Write(in)
					if n != len(in) {
						t.Fatal(msg + "short write")
					}
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					err = fw.Close()
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					fr1 = NewReader(buf)
					data2, err = io.ReadAll(fr1)
					if err != nil {
						t.Fatal(msg + err.Error())
					}
					if !bytes.Equal(in, data2) {
						t.Fatal(msg + "not equal")
					}
				})
			}
		})
	}
}

func benchmarkEncoder(b *testing.B, testfile, level, n int) {
	b.SetBytes(int64(n))
	buf0, err := os.ReadFile(testfiles[testfile])
	if err != nil {
		b.Fatal(err)
	}
	if len(buf0) == 0 {
		b.Fatalf("test file %q has no data", testfiles[testfile])
	}
	buf1 := make([]byte, n)
	for i := 0; i < n; i += len(buf0) {
		if len(buf0) > n-i {
			buf0 = buf0[:n-i]
		}
		copy(buf1[i:], buf0)
	}
	buf0 = nil
	runtime.GC()
	w, err := NewWriter(io.Discard, level)
	if err != nil {
		b.Fatal(err)
	}
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		w.Reset(io.Discard)
		_, err = w.Write(buf1)
		if err != nil {
			b.Fatal(err)
		}
		err = w.Close()
		if err != nil {
			b.Fatal(err)
		}
	}
}

func BenchmarkEncodeDigitsConstant1e4(b *testing.B) { benchmarkEncoder(b, digits, constant, 1e4) }
func BenchmarkEncodeDigitsConstant1e5(b *testing.B) { benchmarkEncoder(b, digits, constant, 1e5) }
func BenchmarkEncodeDigitsConstant1e6(b *testing.B) { benchmarkEncoder(b, digits, constant, 1e6) }
func BenchmarkEncodeDigitsSpeed1e4(b *testing.B)    { benchmarkEncoder(b, digits, speed, 1e4) }
func BenchmarkEncodeDigitsSpeed1e5(b *testing.B)    { benchmarkEncoder(b, digits, speed, 1e5) }
func BenchmarkEncodeDigitsSpeed1e6(b *testing.B)    { benchmarkEncoder(b, digits, speed, 1e6) }
func BenchmarkEncodeDigitsDefault1e4(b *testing.B)  { benchmarkEncoder(b, digits, default_, 1e4) }
func BenchmarkEncodeDigitsDefault1e5(b *testing.B)  { benchmarkEncoder(b, digits, default_, 1e5) }
func BenchmarkEncodeDigitsDefault1e6(b *testing.B)  { benchmarkEncoder(b, digits, default_, 1e6) }
func BenchmarkEncodeDigitsCompress1e4(b *testing.B) { benchmarkEncoder(b, digits, compress, 1e4) }
func BenchmarkEncodeDigitsCompress1e5(b *testing.B) { benchmarkEncoder(b, digits, compress, 1e5) }
func BenchmarkEncodeDigitsCompress1e6(b *testing.B) { benchmarkEncoder(b, digits, compress, 1e6) }
func BenchmarkEncodeDigitsSL1e4(b *testing.B)       { benchmarkStatelessEncoder(b, digits, 1e4) }
func BenchmarkEncodeDigitsSL1e5(b *testing.B)       { benchmarkStatelessEncoder(b, digits, 1e5) }
func BenchmarkEncodeDigitsSL1e6(b *testing.B)       { benchmarkStatelessEncoder(b, digits, 1e6) }
func BenchmarkEncodeTwainConstant1e4(b *testing.B)  { benchmarkEncoder(b, twain, constant, 1e4) }
func BenchmarkEncodeTwainConstant1e5(b *testing.B)  { benchmarkEncoder(b, twain, constant, 1e5) }
func BenchmarkEncodeTwainConstant1e6(b *testing.B)  { benchmarkEncoder(b, twain, constant, 1e6) }
func BenchmarkEncodeTwainSpeed1e4(b *testing.B)     { benchmarkEncoder(b, twain, speed, 1e4) }
func BenchmarkEncodeTwainSpeed1e5(b *testing.B)     { benchmarkEncoder(b, twain, speed, 1e5) }
func BenchmarkEncodeTwainSpeed1e6(b *testing.B)     { benchmarkEncoder(b, twain, speed, 1e6) }
func BenchmarkEncodeTwainDefault1e4(b *testing.B)   { benchmarkEncoder(b, twain, default_, 1e4) }
func BenchmarkEncodeTwainDefault1e5(b *testing.B)   { benchmarkEncoder(b, twain, default_, 1e5) }
func BenchmarkEncodeTwainDefault1e6(b *testing.B)   { benchmarkEncoder(b, twain, default_, 1e6) }
func BenchmarkEncodeTwainCompress1e4(b *testing.B)  { benchmarkEncoder(b, twain, compress, 1e4) }
func BenchmarkEncodeTwainCompress1e5(b *testing.B)  { benchmarkEncoder(b, twain, compress, 1e5) }
func BenchmarkEncodeTwainCompress1e6(b *testing.B)  { benchmarkEncoder(b, twain, compress, 1e6) }
func BenchmarkEncodeTwainSL1e4(b *testing.B)        { benchmarkStatelessEncoder(b, twain, 1e4) }
func BenchmarkEncodeTwainSL1e5(b *testing.B)        { benchmarkStatelessEncoder(b, twain, 1e5) }
func BenchmarkEncodeTwainSL1e6(b *testing.B)        { benchmarkStatelessEncoder(b, twain, 1e6) }

func BenchmarkEncodeTwain1024Win1e4(b *testing.B) { benchmarkEncoder(b, twain, oneK, 1e4) }
func BenchmarkEncodeTwain1024Win1e5(b *testing.B) { benchmarkEncoder(b, twain, oneK, 1e5) }
func BenchmarkEncodeTwain1024Win1e6(b *testing.B) { benchmarkEncoder(b, twain, oneK, 1e6) }

func benchmarkStatelessEncoder(b *testing.B, testfile, n int) {
	b.SetBytes(int64(n))
	buf0, err := os.ReadFile(testfiles[testfile])
	if err != nil {
		b.Fatal(err)
	}
	if len(buf0) == 0 {
		b.Fatalf("test file %q has no data", testfiles[testfile])
	}
	buf1 := make([]byte, n)
	for i := 0; i < n; i += len(buf0) {
		if len(buf0) > n-i {
			buf0 = buf0[:n-i]
		}
		copy(buf1[i:], buf0)
	}
	buf0 = nil
	runtime.GC()
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		w := NewStatelessWriter(io.Discard)
		_, err = w.Write(buf1)
		if err != nil {
			b.Fatal(err)
		}
		err = w.Close()
		if err != nil {
			b.Fatal(err)
		}
	}
}

// A writer that fails after N writes.
type errorWriter struct {
	N int
}

func (e *errorWriter) Write(b []byte) (int, error) {
	if e.N <= 0 {
		return 0, io.ErrClosedPipe
	}
	e.N--
	return len(b), nil
}

// Test if errors from the underlying writer is passed upwards.
func TestWriteError(t *testing.T) {
	buf := new(bytes.Buffer)
	n := 65536
	if !testing.Short() {
		n *= 4
	}
	for i := 0; i < n; i++ {
		fmt.Fprintf(buf, "asdasfasf%d%dfghfgujyut%dyutyu\n", i, i, i)
	}
	in := buf.Bytes()
	// We create our own buffer to control number of writes.
	copyBuf := make([]byte, 128)
	for l := 0; l < 10; l++ {
		for fail := 1; fail <= 256; fail *= 2 {
			// Fail after 'fail' writes
			ew := &errorWriter{N: fail}
			w, err := NewWriter(ew, l)
			if err != nil {
				t.Fatalf("NewWriter: level %d: %v", l, err)
			}
			n, err := copyBuffer(w, bytes.NewBuffer(in), copyBuf)
			if err == nil {
				t.Fatalf("Level %d: Expected an error, writer was %#v", l, ew)
			}
			n2, err := w.Write([]byte{1, 2, 2, 3, 4, 5})
			if n2 != 0 {
				t.Fatal("Level", l, "Expected 0 length write, got", n)
			}
			if err == nil {
				t.Fatal("Level", l, "Expected an error")
			}
			err = w.Flush()
			if err == nil {
				t.Fatal("Level", l, "Expected an error on flush")
			}
			err = w.Close()
			if err == nil {
				t.Fatal("Level", l, "Expected an error on close")
			}

			w.Reset(io.Discard)
			n2, err = w.Write([]byte{1, 2, 3, 4, 5, 6})
			if err != nil {
				t.Fatal("Level", l, "Got unexpected error after reset:", err)
			}
			if n2 == 0 {
				t.Fatal("Level", l, "Got 0 length write, expected > 0")
			}
			if testing.Short() {
				return
			}
		}
	}
}

// Test if errors from the underlying writer is passed upwards.
func TestWriter_Reset(t *testing.T) {
	buf := new(bytes.Buffer)
	n := 65536
	if !testing.Short() {
		n *= 4
	}
	for i := 0; i < n; i++ {
		fmt.Fprintf(buf, "asdasfasf%d%dfghfgujyut%dyutyu\n", i, i, i)
	}
	in := buf.Bytes()
	for l := 0; l < 10; l++ {
		l := l
		if testing.Short() && l > 1 {
			continue
		}
		t.Run(fmt.Sprintf("level-%d", l), func(t *testing.T) {
			t.Parallel()
			offset := 1
			if testing.Short() {
				offset = 256
			}
			for ; offset <= 256; offset *= 2 {
				// Fail after 'fail' writes
				w, err := NewWriter(io.Discard, l)
				if err != nil {
					t.Fatalf("NewWriter: level %d: %v", l, err)
				}
				if w.d.fast == nil {
					t.Skip("Not Fast...")
					return
				}
				for i := 0; i < (bufferReset-len(in)-offset-maxMatchOffset)/maxMatchOffset; i++ {
					// skip ahead to where we are close to wrap around...
					w.d.fast.Reset()
				}
				w.d.fast.Reset()
				_, err = w.Write(in)
				if err != nil {
					t.Fatal(err)
				}
				for i := 0; i < 50; i++ {
					// skip ahead again... This should wrap around...
					w.d.fast.Reset()
				}
				w.d.fast.Reset()

				_, err = w.Write(in)
				if err != nil {
					t.Fatal(err)
				}
				for i := 0; i < (math.MaxUint32-bufferReset)/maxMatchOffset; i++ {
					// skip ahead to where we are close to wrap around...
					w.d.fast.Reset()
				}

				_, err = w.Write(in)
				if err != nil {
					t.Fatal(err)
				}
				err = w.Close()
				if err != nil {
					t.Fatal(err)
				}
			}
		})
	}
}

func TestDeterministicL1(t *testing.T)  { testDeterministic(1, t) }
func TestDeterministicL2(t *testing.T)  { testDeterministic(2, t) }
func TestDeterministicL3(t *testing.T)  { testDeterministic(3, t) }
func TestDeterministicL4(t *testing.T)  { testDeterministic(4, t) }
func TestDeterministicL5(t *testing.T)  { testDeterministic(5, t) }
func TestDeterministicL6(t *testing.T)  { testDeterministic(6, t) }
func TestDeterministicL7(t *testing.T)  { testDeterministic(7, t) }
func TestDeterministicL8(t *testing.T)  { testDeterministic(8, t) }
func TestDeterministicL9(t *testing.T)  { testDeterministic(9, t) }
func TestDeterministicL0(t *testing.T)  { testDeterministic(0, t) }
func TestDeterministicLM2(t *testing.T) { testDeterministic(-2, t) }

func testDeterministic(i int, t *testing.T) {
	// Test so much we cross a good number of block boundaries.
	var length = maxStoreBlockSize*30 + 500
	if testing.Short() {
		length /= 10
	}

	// Create a random, but compressible stream.
	rng := rand.New(rand.NewSource(1))
	t1 := make([]byte, length)
	for i := range t1 {
		t1[i] = byte(rng.Int63() & 7)
	}

	// Do our first encode.
	var b1 bytes.Buffer
	br := bytes.NewBuffer(t1)
	w, err := NewWriter(&b1, i)
	if err != nil {
		t.Fatal(err)
	}
	// Use a very small prime sized buffer.
	cbuf := make([]byte, 787)
	_, err = copyBuffer(w, br, cbuf)
	if err != nil {
		t.Fatal(err)
	}
	w.Close()

	// We choose a different buffer size,
	// bigger than a maximum block, and also a prime.
	var b2 bytes.Buffer
	cbuf = make([]byte, 81761)
	br2 := bytes.NewBuffer(t1)
	w2, err := NewWriter(&b2, i)
	if err != nil {
		t.Fatal(err)
	}
	_, err = copyBuffer(w2, br2, cbuf)
	if err != nil {
		t.Fatal(err)
	}
	w2.Close()

	b1b := b1.Bytes()
	b2b := b2.Bytes()

	if !bytes.Equal(b1b, b2b) {
		t.Errorf("level %d did not produce deterministic result, result mismatch, len(a) = %d, len(b) = %d", i, len(b1b), len(b2b))
	}

	// Test using io.WriterTo interface.
	var b3 bytes.Buffer
	br = bytes.NewBuffer(t1)
	w, err = NewWriter(&b3, i)
	if err != nil {
		t.Fatal(err)
	}
	_, err = br.WriteTo(w)
	if err != nil {
		t.Fatal(err)
	}
	w.Close()

	b3b := b3.Bytes()
	if !bytes.Equal(b1b, b3b) {
		t.Errorf("level %d (io.WriterTo) did not produce deterministic result, result mismatch, len(a) = %d, len(b) = %d", i, len(b1b), len(b3b))
	}
}

// copyBuffer is a copy of io.CopyBuffer, since we want to support older go versions.
// This is modified to never use io.WriterTo or io.ReaderFrom interfaces.
func copyBuffer(dst io.Writer, src io.Reader, buf []byte) (written int64, err error) {
	if buf == nil {
		buf = make([]byte, 32*1024)
	}
	for {
		nr, er := src.Read(buf)
		if nr > 0 {
			nw, ew := dst.Write(buf[0:nr])
			if nw > 0 {
				written += int64(nw)
			}
			if ew != nil {
				err = ew
				break
			}
			if nr != nw {
				err = io.ErrShortWrite
				break
			}
		}
		if er == io.EOF {
			break
		}
		if er != nil {
			err = er
			break
		}
	}
	return written, err
}

func BenchmarkCompressAllocations(b *testing.B) {
	payload := []byte(strings.Repeat("Tiny payload", 20))
	for j := -2; j <= 9; j++ {
		b.Run("level("+strconv.Itoa(j)+")", func(b *testing.B) {
			b.Run("flate", func(b *testing.B) {
				b.ReportAllocs()

				for i := 0; i < b.N; i++ {
					w, err := NewWriter(io.Discard, j)
					if err != nil {
						b.Fatal(err)
					}
					w.Write(payload)
					w.Close()
				}
			})
		})
	}
}

func BenchmarkCompressAllocationsSingle(b *testing.B) {
	payload := []byte(strings.Repeat("Tiny payload", 20))
	const level = 2
	b.Run("flate", func(b *testing.B) {
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			w, err := NewWriter(io.Discard, level)
			if err != nil {
				b.Fatal(err)
			}
			w.Write(payload)
			w.Close()
		}
	})
}
