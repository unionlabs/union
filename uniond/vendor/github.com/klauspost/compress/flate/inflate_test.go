// Copyright 2014 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package flate

import (
	"bytes"
	"crypto/rand"
	"io"
	"os"
	"strconv"
	"strings"
	"testing"
)

func TestReset(t *testing.T) {
	ss := []string{
		"lorem ipsum izzle fo rizzle",
		"the quick brown fox jumped over",
	}

	deflated := make([]bytes.Buffer, 2)
	for i, s := range ss {
		w, _ := NewWriter(&deflated[i], 1)
		w.Write([]byte(s))
		w.Close()
	}

	inflated := make([]bytes.Buffer, 2)

	f := NewReader(&deflated[0])
	io.Copy(&inflated[0], f)
	f.(Resetter).Reset(&deflated[1], nil)
	io.Copy(&inflated[1], f)
	f.Close()

	for i, s := range ss {
		if s != inflated[i].String() {
			t.Errorf("inflated[%d]:\ngot  %q\nwant %q", i, inflated[i], s)
		}
	}
}

func TestReaderTruncated(t *testing.T) {
	vectors := []struct{ input, output string }{
		{"\x00", ""},
		{"\x00\f", ""},
		{"\x00\f\x00", ""},
		{"\x00\f\x00\xf3\xff", ""},
		{"\x00\f\x00\xf3\xffhello", "hello"},
		{"\x00\f\x00\xf3\xffhello, world", "hello, world"},
		{"\x02", ""},
		{"\xf2H\xcd", "He"},
		{"\xf2H͙0a\u0084\t", "Hel\x90\x90\x90\x90\x90"},
		{"\xf2H͙0a\u0084\t\x00", "Hel\x90\x90\x90\x90\x90"},
	}

	for i, v := range vectors {
		r := strings.NewReader(v.input)
		zr := NewReader(r)
		b, err := io.ReadAll(zr)
		if err != io.ErrUnexpectedEOF {
			t.Errorf("test %d, error mismatch: got %v, want io.ErrUnexpectedEOF", i, err)
		}
		if string(b) != v.output {
			t.Errorf("test %d, output mismatch: got %q, want %q", i, b, v.output)
		}
	}
}

func TestResetDict(t *testing.T) {
	dict := []byte("the lorem fox")
	ss := []string{
		"lorem ipsum izzle fo rizzle",
		"the quick brown fox jumped over",
	}

	deflated := make([]bytes.Buffer, len(ss))
	for i, s := range ss {
		w, _ := NewWriterDict(&deflated[i], DefaultCompression, dict)
		w.Write([]byte(s))
		w.Close()
	}

	inflated := make([]bytes.Buffer, len(ss))

	f := NewReader(nil)
	for i := range inflated {
		f.(Resetter).Reset(&deflated[i], dict)
		io.Copy(&inflated[i], f)
	}
	f.Close()

	for i, s := range ss {
		if s != inflated[i].String() {
			t.Errorf("inflated[%d]:\ngot  %q\nwant %q", i, inflated[i], s)
		}
	}
}

// Tests ported from zlib/test/infcover.c
type infTest struct {
	hex string
	id  string
	n   int
}

var infTests = []infTest{
	{"0 0 0 0 0", "invalid stored block lengths", 1},
	{"3 0", "fixed", 0},
	{"6", "invalid block type", 1},
	{"1 1 0 fe ff 0", "stored", 0},
	{"fc 0 0", "too many length or distance symbols", 1},
	{"4 0 fe ff", "invalid code lengths set", 1},
	{"4 0 24 49 0", "invalid bit length repeat", 1},
	{"4 0 24 e9 ff ff", "invalid bit length repeat", 1},
	{"4 0 24 e9 ff 6d", "invalid code -- missing end-of-block", 1},
	{"4 80 49 92 24 49 92 24 71 ff ff 93 11 0", "invalid literal/lengths set", 1},
	{"4 80 49 92 24 49 92 24 f b4 ff ff c3 84", "invalid distances set", 1},
	{"4 c0 81 8 0 0 0 0 20 7f eb b 0 0", "invalid literal/length code", 1},
	{"2 7e ff ff", "invalid distance code", 1},
	{"c c0 81 0 0 0 0 0 90 ff 6b 4 0", "invalid distance too far back", 1},

	// also trailer mismatch just in inflate()
	{"1f 8b 8 0 0 0 0 0 0 0 3 0 0 0 0 1", "incorrect data check", -1},
	{"1f 8b 8 0 0 0 0 0 0 0 3 0 0 0 0 0 0 0 0 1", "incorrect length check", -1},
	{"5 c0 21 d 0 0 0 80 b0 fe 6d 2f 91 6c", "pull 17", 0},
	{"5 e0 81 91 24 cb b2 2c 49 e2 f 2e 8b 9a 47 56 9f fb fe ec d2 ff 1f", "long code", 0},
	{"ed c0 1 1 0 0 0 40 20 ff 57 1b 42 2c 4f", "length extra", 0},
	{"ed cf c1 b1 2c 47 10 c4 30 fa 6f 35 1d 1 82 59 3d fb be 2e 2a fc f c", "long distance and extra", 0},
	{"ed c0 81 0 0 0 0 80 a0 fd a9 17 a9 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 6", "window end", 0},
}

func TestInflate(t *testing.T) {
	for _, test := range infTests {
		hex := strings.Split(test.hex, " ")
		data := make([]byte, len(hex))
		for i, h := range hex {
			b, _ := strconv.ParseInt(h, 16, 32)
			data[i] = byte(b)
		}
		buf := bytes.NewReader(data)
		r := NewReader(buf)

		_, err := io.Copy(io.Discard, r)
		if (test.n == 0 && err == nil) || (test.n != 0 && err != nil) {
			t.Logf("%q: OK:", test.id)
			t.Logf(" - got %v", err)
			continue
		}

		if test.n == 0 && err != nil {
			t.Errorf("%q: Expected no error, but got %v", test.id, err)
			continue
		}

		if test.n != 0 && err == nil {
			t.Errorf("%q:Expected an error, but got none", test.id)
			continue
		}
		t.Fatal(test.n, err)
	}

	for _, test := range infOutTests {
		hex := strings.Split(test.hex, " ")
		data := make([]byte, len(hex))
		for i, h := range hex {
			b, _ := strconv.ParseInt(h, 16, 32)
			data[i] = byte(b)
		}
		buf := bytes.NewReader(data)
		r := NewReader(buf)

		_, err := io.Copy(io.Discard, r)
		if test.err == (err != nil) {
			t.Logf("%q: OK:", test.id)
			t.Logf(" - got %v", err)
			continue
		}

		if test.err == false && err != nil {
			t.Errorf("%q: Expected no error, but got %v", test.id, err)
			continue
		}

		if test.err && err == nil {
			t.Errorf("%q: Expected an error, but got none", test.id)
			continue
		}
		t.Fatal(test.err, err)
	}

}

// Tests ported from zlib/test/infcover.c
// Since zlib inflate is push (writer) instead of pull (reader)
// some of the window size tests have been removed, since they
// are irrelevant.
type infOutTest struct {
	hex    string
	id     string
	step   int
	win    int
	length int
	err    bool
}

var infOutTests = []infOutTest{
	{"2 8 20 80 0 3 0", "inflate_fast TYPE return", 0, -15, 258, false},
	{"63 18 5 40 c 0", "window wrap", 3, -8, 300, false},
	{"e5 e0 81 ad 6d cb b2 2c c9 01 1e 59 63 ae 7d ee fb 4d fd b5 35 41 68 ff 7f 0f 0 0 0", "fast length extra bits", 0, -8, 258, true},
	{"25 fd 81 b5 6d 59 b6 6a 49 ea af 35 6 34 eb 8c b9 f6 b9 1e ef 67 49 50 fe ff ff 3f 0 0", "fast distance extra bits", 0, -8, 258, true},
	{"3 7e 0 0 0 0 0", "fast invalid distance code", 0, -8, 258, true},
	{"1b 7 0 0 0 0 0", "fast invalid literal/length code", 0, -8, 258, true},
	{"d c7 1 ae eb 38 c 4 41 a0 87 72 de df fb 1f b8 36 b1 38 5d ff ff 0", "fast 2nd level codes and too far back", 0, -8, 258, true},
	{"63 18 5 8c 10 8 0 0 0 0", "very common case", 0, -8, 259, false},
	{"63 60 60 18 c9 0 8 18 18 18 26 c0 28 0 29 0 0 0", "contiguous and wrap around window", 6, -8, 259, false},
	{"63 0 3 0 0 0 0 0", "copy direct from output", 0, -8, 259, false},
	{"1f 8b 0 0", "bad gzip method", 0, 31, 0, true},
	{"1f 8b 8 80", "bad gzip flags", 0, 31, 0, true},
	{"77 85", "bad zlib method", 0, 15, 0, true},
	{"78 9c", "bad zlib window size", 0, 8, 0, true},
	{"1f 8b 8 1e 0 0 0 0 0 0 1 0 0 0 0 0 0", "bad header crc", 0, 47, 1, true},
	{"1f 8b 8 2 0 0 0 0 0 0 1d 26 3 0 0 0 0 0 0 0 0 0", "check gzip length", 0, 47, 0, true},
	{"78 90", "bad zlib header check", 0, 47, 0, true},
	{"8 b8 0 0 0 1", "need dictionary", 0, 8, 0, true},
	{"63 18 68 30 d0 0 0", "force split window update", 4, -8, 259, false},
	{"3 0", "use fixed blocks", 0, -15, 1, false},
	{"", "bad window size", 0, 1, 0, true},
}

func TestWriteTo(t *testing.T) {
	input := make([]byte, 100000)
	n, err := rand.Read(input)
	if err != nil {
		t.Fatal(err)
	}
	if n != len(input) {
		t.Fatal("did not fill buffer")
	}
	compressed := &bytes.Buffer{}
	w, err := NewWriter(compressed, -2)
	if err != nil {
		t.Fatal(err)
	}
	n, err = w.Write(input)
	if err != nil {
		t.Fatal(err)
	}
	if n != len(input) {
		t.Fatal("did not fill buffer")
	}
	w.Close()
	buf := compressed.Bytes()

	dec := NewReader(bytes.NewBuffer(buf))
	// ReadAll does not use WriteTo, but we wrap it in a NopCloser to be sure.
	readall, err := io.ReadAll(io.NopCloser(dec))
	if err != nil {
		t.Fatal(err)
	}
	if len(readall) != len(input) {
		t.Fatal("did not decompress everything")
	}

	dec = NewReader(bytes.NewBuffer(buf))
	wtbuf := &bytes.Buffer{}
	written, err := dec.(io.WriterTo).WriteTo(wtbuf)
	if err != nil {
		t.Fatal(err)
	}
	if written != int64(len(input)) {
		t.Error("Returned length did not match, expected", len(input), "got", written)
	}
	if wtbuf.Len() != len(input) {
		t.Error("Actual Length did not match, expected", len(input), "got", wtbuf.Len())
	}
	if !bytes.Equal(wtbuf.Bytes(), input) {
		t.Fatal("output did not match input")
	}
}

func TestReaderPartialBlock(t *testing.T) {
	data, err := os.ReadFile("testdata/partial-block")
	if err != nil {
		t.Error(err)
	}

	r := NewReaderOpts(bytes.NewReader(data), WithPartialBlock())
	rb := make([]byte, 32)
	n, err := r.Read(rb)
	if err != nil {
		t.Fatalf("Read: %v", err)
	}

	expected := "hello, world"
	actual := string(rb[:n])
	if expected != actual {
		t.Fatalf("expected: %v, got: %v", expected, actual)
	}
}
