// Copyright 2010 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gzip

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"math/rand"
	"os"
	"strconv"
	"strings"
	"testing"
	"time"
)

// TestEmpty tests that an empty payload still forms a valid GZIP stream.
func TestEmpty(t *testing.T) {
	buf := new(bytes.Buffer)

	if err := NewWriter(buf).Close(); err != nil {
		t.Fatalf("Writer.Close: %v", err)
	}

	r, err := NewReader(buf)
	if err != nil {
		t.Fatalf("NewReader: %v", err)
	}
	b, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("ReadAll: %v", err)
	}
	if len(b) != 0 {
		t.Fatalf("got %d bytes, want 0", len(b))
	}
	if err := r.Close(); err != nil {
		t.Fatalf("Reader.Close: %v", err)
	}
}

// TestRoundTrip tests that gzipping and then gunzipping is the identity
// function.
func TestRoundTrip(t *testing.T) {
	buf := new(bytes.Buffer)

	w := NewWriter(buf)
	w.Comment = "comment"
	w.Extra = []byte("extra")
	w.ModTime = time.Unix(1e8, 0)
	w.Name = "name"
	if _, err := w.Write([]byte("payload")); err != nil {
		t.Fatalf("Write: %v", err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("Writer.Close: %v", err)
	}

	r, err := NewReader(buf)
	if err != nil {
		t.Fatalf("NewReader: %v", err)
	}
	b, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("ReadAll: %v", err)
	}
	if string(b) != "payload" {
		t.Fatalf("payload is %q, want %q", string(b), "payload")
	}
	if r.Comment != "comment" {
		t.Fatalf("comment is %q, want %q", r.Comment, "comment")
	}
	if string(r.Extra) != "extra" {
		t.Fatalf("extra is %q, want %q", r.Extra, "extra")
	}
	if r.ModTime.Unix() != 1e8 {
		t.Fatalf("mtime is %d, want %d", r.ModTime.Unix(), uint32(1e8))
	}
	if r.Name != "name" {
		t.Fatalf("name is %q, want %q", r.Name, "name")
	}
	if err := r.Close(); err != nil {
		t.Fatalf("Reader.Close: %v", err)
	}
}

// TestLatin1 tests the internal functions for converting to and from Latin-1.
func TestLatin1(t *testing.T) {
	latin1 := []byte{0xc4, 'u', 0xdf, 'e', 'r', 'u', 'n', 'g', 0}
	utf8 := "Äußerung"
	z := Reader{r: bufio.NewReader(bytes.NewReader(latin1))}
	s, err := z.readString()
	if err != nil {
		t.Fatalf("readString: %v", err)
	}
	if s != utf8 {
		t.Fatalf("read latin-1: got %q, want %q", s, utf8)
	}

	buf := bytes.NewBuffer(make([]byte, 0, len(latin1)))
	c := Writer{w: buf}
	if err = c.writeString(utf8); err != nil {
		t.Fatalf("writeString: %v", err)
	}
	s = buf.String()
	if s != string(latin1) {
		t.Fatalf("write utf-8: got %q, want %q", s, string(latin1))
	}
}

// TestLatin1RoundTrip tests that metadata that is representable in Latin-1
// survives a round trip.
func TestLatin1RoundTrip(t *testing.T) {
	testCases := []struct {
		name string
		ok   bool
	}{
		{"", true},
		{"ASCII is OK", true},
		{"unless it contains a NUL\x00", false},
		{"no matter where \x00 occurs", false},
		{"\x00\x00\x00", false},
		{"Látin-1 also passes (U+00E1)", true},
		{"but LĀtin Extended-A (U+0100) does not", false},
		{"neither does 日本語", false},
		{"invalid UTF-8 also \xffails", false},
		{"\x00 as does Látin-1 with NUL", false},
	}
	for _, tc := range testCases {
		buf := new(bytes.Buffer)

		w := NewWriter(buf)
		w.Name = tc.name
		err := w.Close()
		if (err == nil) != tc.ok {
			t.Errorf("Writer.Close: name = %q, err = %v", tc.name, err)
			continue
		}
		if !tc.ok {
			continue
		}

		r, err := NewReader(buf)
		if err != nil {
			t.Errorf("NewReader: %v", err)
			continue
		}
		_, err = io.ReadAll(r)
		if err != nil {
			t.Errorf("ReadAll: %v", err)
			continue
		}
		if r.Name != tc.name {
			t.Errorf("name is %q, want %q", r.Name, tc.name)
			continue
		}
		if err := r.Close(); err != nil {
			t.Errorf("Reader.Close: %v", err)
			continue
		}
	}
}

func TestWriterFlush(t *testing.T) {
	buf := new(bytes.Buffer)

	w := NewWriter(buf)
	w.Comment = "comment"
	w.Extra = []byte("extra")
	w.ModTime = time.Unix(1e8, 0)
	w.Name = "name"

	n0 := buf.Len()
	if n0 != 0 {
		t.Fatalf("buffer size = %d before writes; want 0", n0)
	}

	if err := w.Flush(); err != nil {
		t.Fatal(err)
	}

	n1 := buf.Len()
	if n1 == 0 {
		t.Fatal("no data after first flush")
	}

	w.Write([]byte("x"))

	n2 := buf.Len()
	if n1 != n2 {
		t.Fatalf("after writing a single byte, size changed from %d to %d; want no change", n1, n2)
	}

	if err := w.Flush(); err != nil {
		t.Fatal(err)
	}

	n3 := buf.Len()
	if n2 == n3 {
		t.Fatal("Flush didn't flush any data")
	}
}

// Multiple gzip files concatenated form a valid gzip file.
func TestConcat(t *testing.T) {
	var buf bytes.Buffer
	w := NewWriter(&buf)
	w.Write([]byte("hello "))
	w.Close()
	w = NewWriter(&buf)
	w.Write([]byte("world\n"))
	w.Close()

	r, err := NewReader(&buf)
	if err != nil {
		t.Fatal(err)
	}
	data, err := io.ReadAll(r)
	if string(data) != "hello world\n" || err != nil {
		t.Fatalf("ReadAll = %q, %v, want %q, nil", data, err, "hello world")
	}
}

func TestWriterReset(t *testing.T) {
	buf := new(bytes.Buffer)
	buf2 := new(bytes.Buffer)
	z := NewWriter(buf)
	msg := []byte("hello world")
	z.Write(msg)
	z.Close()
	z.Reset(buf2)
	z.Write(msg)
	z.Close()
	if buf.String() != buf2.String() {
		t.Errorf("buf2 %q != original buf of %q", buf2.String(), buf.String())
	}
}

var testbuf []byte

func testFile(i, level int, t *testing.T) {
	dat, _ := os.ReadFile("testdata/test.json")
	dl := len(dat)
	if len(testbuf) != i*dl {
		// Make results predictable
		testbuf = make([]byte, i*dl)
		for j := 0; j < i; j++ {
			copy(testbuf[j*dl:j*dl+dl], dat)
		}
	}

	br := bytes.NewBuffer(testbuf)
	var buf bytes.Buffer
	w, err := NewWriterLevel(&buf, level)
	if err != nil {
		t.Fatal(err)
	}
	n, err := io.Copy(w, br)
	if err != nil {
		t.Fatal(err)
	}
	if int(n) != len(testbuf) {
		t.Fatal("Short write:", n, "!=", testbuf)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	r, err := NewReader(&buf)
	if err != nil {
		t.Fatal(err.Error())
	}
	decoded, err := io.ReadAll(r)
	if err != nil {
		t.Fatal(err.Error())
	}
	if !bytes.Equal(testbuf, decoded) {
		t.Errorf("decoded content does not match.")
	}
}

func TestFile1xM3(t *testing.T) { testFile(1, -3, t) }
func TestFile1xM2(t *testing.T) { testFile(1, -2, t) }
func TestFile1xM1(t *testing.T) { testFile(1, -1, t) }
func TestFile1x0(t *testing.T)  { testFile(1, 0, t) }
func TestFile1x1(t *testing.T)  { testFile(1, 1, t) }
func TestFile1x2(t *testing.T)  { testFile(1, 2, t) }
func TestFile1x3(t *testing.T)  { testFile(1, 3, t) }
func TestFile1x4(t *testing.T)  { testFile(1, 4, t) }
func TestFile1x5(t *testing.T)  { testFile(1, 5, t) }
func TestFile1x6(t *testing.T)  { testFile(1, 6, t) }
func TestFile1x7(t *testing.T)  { testFile(1, 7, t) }
func TestFile1x8(t *testing.T)  { testFile(1, 8, t) }
func TestFile1x9(t *testing.T)  { testFile(1, 9, t) }
func TestFile10(t *testing.T)   { testFile(10, DefaultCompression, t) }

func TestFile50(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping during short test")
	}
	testFile(50, DefaultCompression, t)
}

func TestFile200(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping during short test")
	}
	testFile(200, BestSpeed, t)
}

func TestFileWindow(t *testing.T) {
	for sz := MinCustomWindowSize; sz <= MaxCustomWindowSize; sz *= 2 {
		t.Run(fmt.Sprint(sz), func(t *testing.T) {
			testFileWindow(1, sz, t)
		})
	}
}

func testFileWindow(i, window int, t *testing.T) {
	dat, _ := os.ReadFile("testdata/test.json")
	dl := len(dat)
	if len(testbuf) != i*dl {
		// Make results predictable
		testbuf = make([]byte, i*dl)
		for j := 0; j < i; j++ {
			copy(testbuf[j*dl:j*dl+dl], dat)
		}
	}

	br := bytes.NewBuffer(testbuf)
	var buf bytes.Buffer
	w, err := NewWriterWindow(&buf, window)
	if err != nil {
		t.Fatal(err)
	}
	n, err := io.Copy(w, br)
	if err != nil {
		t.Fatal(err)
	}
	if int(n) != len(testbuf) {
		t.Fatal("Short write:", n, "!=", testbuf)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("size: %d bytes", buf.Len())
	r, err := NewReader(&buf)
	if err != nil {
		t.Fatal(err.Error())
	}
	decoded, err := io.ReadAll(r)
	if err != nil {
		t.Fatal(err.Error())
	}
	if !bytes.Equal(testbuf, decoded) {
		t.Errorf("decoded content does not match.")
	}
}

func testBigGzip(i int, t *testing.T) {
	if len(testbuf) != i {
		// Make results predictable
		rand.Seed(1337)
		testbuf = make([]byte, i)
		for idx := range testbuf {
			testbuf[idx] = byte(65 + rand.Intn(20))
		}
	}
	c := BestCompression
	if testing.Short() {
		c = BestSpeed
	}

	br := bytes.NewBuffer(testbuf)
	var buf bytes.Buffer
	w, err := NewWriterLevel(&buf, c)
	if err != nil {
		t.Fatal(err)
	}
	n, err := io.Copy(w, br)
	if err != nil {
		t.Fatal(err)
	}
	if int(n) != len(testbuf) {
		t.Fatal("Short write:", n, "!=", len(testbuf))
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err.Error())
	}

	r, err := NewReader(&buf)
	if err != nil {
		t.Fatal(err.Error())
	}
	decoded, err := io.ReadAll(r)
	if err != nil {
		t.Fatal(err.Error())
	}
	if !bytes.Equal(testbuf, decoded) {
		t.Errorf("decoded content does not match.")
	}
}

func TestGzip1K(t *testing.T)   { testBigGzip(1000, t) }
func TestGzip100K(t *testing.T) { testBigGzip(100000, t) }
func TestGzip1M(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping during short test")
	}

	testBigGzip(1000000, t)
}
func TestGzip10M(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping during short test")
	}
	testBigGzip(10000000, t)
}

// Test if two runs produce identical results.
func TestDeterministicLM2(t *testing.T) { testDeterm(-2, t) }

// Level 0 is not deterministic since it depends on the size of each write.
// func TestDeterministicL0(t *testing.T)  { testDeterm(0, t) }
func TestDeterministicL1(t *testing.T) { testDeterm(1, t) }
func TestDeterministicL2(t *testing.T) { testDeterm(2, t) }
func TestDeterministicL3(t *testing.T) { testDeterm(3, t) }
func TestDeterministicL4(t *testing.T) { testDeterm(4, t) }
func TestDeterministicL5(t *testing.T) { testDeterm(5, t) }
func TestDeterministicL6(t *testing.T) { testDeterm(6, t) }
func TestDeterministicL7(t *testing.T) { testDeterm(7, t) }
func TestDeterministicL8(t *testing.T) { testDeterm(8, t) }
func TestDeterministicL9(t *testing.T) { testDeterm(9, t) }

func testDeterm(level int, t *testing.T) {
	var length = 500000
	if testing.Short() {
		length = 100000
	}
	rand.Seed(1337)
	t1 := make([]byte, length)
	for idx := range t1 {
		t1[idx] = byte(65 + rand.Intn(8))
	}

	br := bytes.NewBuffer(t1)
	var b1 bytes.Buffer
	w, err := NewWriterLevel(&b1, level)
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.Copy(w, br)
	if err != nil {
		t.Fatal(err)
	}
	w.Flush()
	w.Close()

	// We recreate the buffer, so we have a goos chance of getting a
	// different memory address.
	rand.Seed(1337)
	t2 := make([]byte, length)
	for idx := range t2 {
		t2[idx] = byte(65 + rand.Intn(8))
	}

	br2 := bytes.NewBuffer(t2)
	var b2 bytes.Buffer
	w2, err := NewWriterLevel(&b2, level)
	if err != nil {
		t.Fatal(err)
	}

	// We write the same data, but with a different size than
	// the default copy.
	for {
		_, err = io.CopyN(w2, br2, 1234)
		if err == io.EOF {
			err = nil
			break
		} else if err != nil {
			break
		}
	}
	if err != nil {
		t.Fatal(err)
	}
	w2.Flush()
	w2.Close()

	b1b := b1.Bytes()
	b2b := b2.Bytes()

	if !bytes.Equal(b1b, b2b) {
		t.Fatalf("Level %d did not produce deterministric result, len(a) = %d, len(b) = %d", level, len(b1b), len(b2b))
	}
}

func BenchmarkGzipLM3(b *testing.B) { benchmarkGzipN(b, -3) }
func BenchmarkGzipLM2(b *testing.B) { benchmarkGzipN(b, -2) }
func BenchmarkGzipL1(b *testing.B)  { benchmarkGzipN(b, 1) }
func BenchmarkGzipL2(b *testing.B)  { benchmarkGzipN(b, 2) }
func BenchmarkGzipL3(b *testing.B)  { benchmarkGzipN(b, 3) }
func BenchmarkGzipL4(b *testing.B)  { benchmarkGzipN(b, 4) }
func BenchmarkGzipL5(b *testing.B)  { benchmarkGzipN(b, 5) }
func BenchmarkGzipL6(b *testing.B)  { benchmarkGzipN(b, 6) }
func BenchmarkGzipL7(b *testing.B)  { benchmarkGzipN(b, 7) }
func BenchmarkGzipL8(b *testing.B)  { benchmarkGzipN(b, 8) }
func BenchmarkGzipL9(b *testing.B)  { benchmarkGzipN(b, 9) }

func benchmarkGzipN(b *testing.B, level int) {
	dat, _ := os.ReadFile("testdata/test.json")
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	b.SetBytes(int64(len(dat)))
	w, _ := NewWriterLevel(io.Discard, level)
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		w.Reset(io.Discard)
		n, err := w.Write(dat)
		if n != len(dat) {
			panic("short write")
		}
		if err != nil {
			panic(err)
		}
		err = w.Close()
		if err != nil {
			panic(err)
		}
	}
}

/*
func BenchmarkOldGzipL1(b *testing.B) { benchmarkOldGzipN(b, 1) }
func BenchmarkOldGzipL2(b *testing.B) { benchmarkOldGzipN(b, 2) }
func BenchmarkOldGzipL3(b *testing.B) { benchmarkOldGzipN(b, 3) }
func BenchmarkOldGzipL4(b *testing.B) { benchmarkOldGzipN(b, 4) }
func BenchmarkOldGzipL5(b *testing.B) { benchmarkOldGzipN(b, 5) }
func BenchmarkOldGzipL6(b *testing.B) { benchmarkOldGzipN(b, 6) }
func BenchmarkOldGzipL7(b *testing.B) { benchmarkOldGzipN(b, 7) }
func BenchmarkOldGzipL8(b *testing.B) { benchmarkOldGzipN(b, 8) }
func BenchmarkOldGzipL9(b *testing.B) { benchmarkOldGzipN(b, 9) }

func benchmarkOldGzipN(b *testing.B, level int) {
	dat, _ := os.ReadFile("testdata/test.json")
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)
	dat = append(dat, dat...)

	b.SetBytes(int64(len(dat)))
	w, _ := oldgz.NewWriterLevel(io.Discard, level)
	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		w.Reset(io.Discard)
		n, err := w.Write(dat)
		if n != len(dat) {
			panic("short write")
		}
		if err != nil {
			panic(err)
		}
		err = w.Close()
		if err != nil {
			panic(err)
		}
	}
}

*/

func BenchmarkCompressAllocations(b *testing.B) {
	payload := []byte(strings.Repeat("Tiny payload", 20))
	for j := -2; j <= 9; j++ {
		b.Run("level("+strconv.Itoa(j)+")", func(b *testing.B) {
			b.Run("gzip", func(b *testing.B) {
				b.ReportAllocs()

				for i := 0; i < b.N; i++ {
					w, err := NewWriterLevel(io.Discard, j)
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

	b.Run("gzip", func(b *testing.B) {
		b.ReportAllocs()

		for i := 0; i < b.N; i++ {
			w, err := NewWriterLevel(io.Discard, level)
			if err != nil {
				b.Fatal(err)
			}
			w.Write(payload)
			w.Close()
		}
	})
}
