// Copyright (c) 2019 Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"runtime"
	"strings"
	"testing"

	"github.com/klauspost/compress/zip"
)

func TestDecodeRegression(t *testing.T) {
	data, err := os.ReadFile("testdata/dec-block-regressions.zip")
	if err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}
	for _, tt := range zr.File {
		if !strings.HasSuffix(t.Name(), "") {
			continue
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
			got, err := Decode(nil, in)
			t.Log("Received:", len(got), err)
		})
	}
}

func TestDecoderMaxBlockSize(t *testing.T) {
	data, err := os.ReadFile("testdata/enc_regressions.zip")
	if err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}
	sizes := []int{4 << 10, 10 << 10, 1 << 20, 4 << 20}
	test := func(t *testing.T, data []byte) {
		for _, size := range sizes {
			t.Run(fmt.Sprintf("%d", size), func(t *testing.T) {
				var buf bytes.Buffer
				dec := NewReader(nil, ReaderMaxBlockSize(size), ReaderAllocBlock(size/2))
				enc := NewWriter(&buf, WriterBlockSize(size), WriterPadding(16<<10), WriterPaddingSrc(zeroReader{}))

				// Test writer.
				n, err := enc.Write(data)
				if err != nil {
					t.Error(err)
					return
				}
				if n != len(data) {
					t.Error(fmt.Errorf("Write: Short write, want %d, got %d", len(data), n))
					return
				}
				err = enc.Close()
				if err != nil {
					t.Error(err)
					return
				}
				// Calling close twice should not affect anything.
				err = enc.Close()
				if err != nil {
					t.Error(err)
					return
				}

				dec.Reset(&buf)
				got, err := io.ReadAll(dec)
				if err != nil {
					t.Error(err)
					return
				}
				if !bytes.Equal(data, got) {
					t.Error("block (reset) decoder mismatch")
					return
				}

				// Test Reset on both and use ReadFrom instead.
				buf.Reset()
				enc.Reset(&buf)
				n2, err := enc.ReadFrom(bytes.NewBuffer(data))
				if err != nil {
					t.Error(err)
					return
				}
				if n2 != int64(len(data)) {
					t.Error(fmt.Errorf("ReadFrom: Short read, want %d, got %d", len(data), n2))
					return
				}
				// Encode twice...
				n2, err = enc.ReadFrom(bytes.NewBuffer(data))
				if err != nil {
					t.Error(err)
					return
				}
				if n2 != int64(len(data)) {
					t.Error(fmt.Errorf("ReadFrom: Short read, want %d, got %d", len(data), n2))
					return
				}

				err = enc.Close()
				if err != nil {
					t.Error(err)
					return
				}
				if enc.pad > 0 && buf.Len()%enc.pad != 0 {
					t.Error(fmt.Errorf("wanted size to be multiple of %d, got size %d with remainder %d", enc.pad, buf.Len(), buf.Len()%enc.pad))
					return
				}
				encoded := buf.Bytes()
				dec.Reset(&buf)
				// Skip first...
				dec.Skip(int64(len(data)))
				got, err = io.ReadAll(dec)
				if err != nil {
					t.Error(err)
					return
				}
				if !bytes.Equal(data, got) {
					t.Error("frame (reset) decoder mismatch")
					return
				}
				// Re-add data, Read concurrent.
				buf.Write(encoded)
				dec.Reset(&buf)
				var doubleB bytes.Buffer
				nb, err := dec.DecodeConcurrent(&doubleB, runtime.GOMAXPROCS(0))
				if err != nil {
					t.Error(err)
					return
				}
				if nb != int64(len(data)*2) {
					t.Errorf("want %d, got %d, err: %v", len(data)*2, nb, err)
					return
				}
				got = doubleB.Bytes()[:len(data)]
				if !bytes.Equal(data, got) {
					t.Error("frame (DecodeConcurrent) decoder mismatch")
					return
				}
				got = doubleB.Bytes()[len(data):]
				if !bytes.Equal(data, got) {
					t.Error("frame (DecodeConcurrent) decoder mismatch")
					return
				}
			})
		}
	}
	for _, tt := range zr.File {
		if !strings.HasSuffix(t.Name(), "") {
			continue
		}
		t.Run(tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			b, err := io.ReadAll(r)
			if err != nil {
				t.Error(err)
				return
			}
			test(t, b[:len(b):len(b)])
		})
	}
}
