// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package xz

import (
	"bytes"
	"io"
	"io/ioutil"
	"os"
	"testing"
)

func TestReaderSimple(t *testing.T) {
	const file = "fox.xz"
	xz, err := os.Open(file)
	if err != nil {
		t.Fatalf("os.Open(%q) error %s", file, err)
	}
	r, err := NewReader(xz)
	if err != nil {
		t.Fatalf("NewReader error %s", err)
	}
	var buf bytes.Buffer
	if _, err = io.Copy(&buf, r); err != nil {
		t.Fatalf("io.Copy error %s", err)
	}
}

func TestReaderSingleStream(t *testing.T) {
	data, err := ioutil.ReadFile("fox.xz")
	if err != nil {
		t.Fatalf("ReadFile error %s", err)
	}
	xz := bytes.NewReader(data)
	rc := ReaderConfig{SingleStream: true}
	r, err := rc.NewReader(xz)
	if err != nil {
		t.Fatalf("NewReader error %s", err)
	}
	var buf bytes.Buffer
	if _, err = io.Copy(&buf, r); err != nil {
		t.Fatalf("io.Copy error %s", err)
	}
	buf.Reset()
	data = append(data, 0)
	xz = bytes.NewReader(data)
	r, err = rc.NewReader(xz)
	if err != nil {
		t.Fatalf("NewReader error %s", err)
	}
	if _, err = io.Copy(&buf, r); err != errUnexpectedData {
		t.Fatalf("io.Copy returned %v; want %v", err, errUnexpectedData)
	}
}

func TestReaderMultipleStreams(t *testing.T) {
	data, err := ioutil.ReadFile("fox.xz")
	if err != nil {
		t.Fatalf("ReadFile error %s", err)
	}
	m := make([]byte, 0, 4*len(data)+4*4)
	m = append(m, data...)
	m = append(m, data...)
	m = append(m, 0, 0, 0, 0)
	m = append(m, data...)
	m = append(m, 0, 0, 0, 0)
	m = append(m, 0, 0, 0, 0)
	m = append(m, data...)
	m = append(m, 0, 0, 0, 0)
	xz := bytes.NewReader(m)
	r, err := NewReader(xz)
	if err != nil {
		t.Fatalf("NewReader error %s", err)
	}
	var buf bytes.Buffer
	if _, err = io.Copy(&buf, r); err != nil {
		t.Fatalf("io.Copy error %s", err)
	}
}

func TestCheckNone(t *testing.T) {
	const file = "fox-check-none.xz"
	xz, err := os.Open(file)
	if err != nil {
		t.Fatalf("os.Open(%q) error %s", file, err)
	}
	r, err := NewReader(xz)
	if err != nil {
		t.Fatalf("NewReader error %s", err)
	}
	var buf bytes.Buffer
	if _, err = io.Copy(&buf, r); err != nil {
		t.Fatalf("io.Copy error %s", err)
	}
}

func BenchmarkReader(b *testing.B) {
	const testFile = "testdata/enwik7"
	data, err := os.ReadFile(testFile)
	if err != nil {
		b.Fatalf("os.ReadFile(%q) error %s", testFile, err)
	}
	buf := new(bytes.Buffer)
	uncompressedLen := int64(len(data))
	b.SetBytes(int64(uncompressedLen))
	b.ReportAllocs()
	buf.Reset()
	w, err := NewWriter(buf)
	if err != nil {
		b.Fatalf("NewWriter(buf) error %s", err)
	}
	if _, err = w.Write(data); err != nil {
		b.Fatalf("w.Write(data) error %s", err)
	}
	if err = w.Close(); err != nil {
		b.Fatalf("w.Write(data)")
	}
	data = make([]byte, buf.Len())
	copy(data, buf.Bytes())
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		buf.Reset()
		r, err := NewReader(bytes.NewReader(data))
		if err != nil {
			b.Fatalf("NewReader(data) error %s", err)
		}
		n, err := io.Copy(buf, r)
		if err != nil {
			b.Fatalf("io.Copy(buf, r) error %s", err)
		}
		if n != uncompressedLen {
			b.Fatalf("io.Copy got %d; want %d", n, uncompressedLen)
		}
	}
}
