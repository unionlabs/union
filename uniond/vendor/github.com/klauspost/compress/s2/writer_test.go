// Copyright (c) 2019+ Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"bytes"
	"fmt"
	"io"
	"math/rand"
	"os"
	"runtime"
	"strings"
	"testing"

	"github.com/klauspost/compress/internal/snapref"
	"github.com/klauspost/compress/zip"
)

func testOptions(_ testing.TB) map[string][]WriterOption {
	var testOptions = map[string][]WriterOption{
		"default": {WriterAddIndex()},
		"better":  {WriterBetterCompression()},
		"best":    {WriterBestCompression()},
		"none":    {WriterUncompressed()},
	}

	x := make(map[string][]WriterOption)
	cloneAdd := func(org []WriterOption, add ...WriterOption) []WriterOption {
		y := make([]WriterOption, len(org)+len(add))
		copy(y, org)
		copy(y[len(org):], add)
		return y
	}
	for name, opt := range testOptions {
		x[name] = opt
		x[name+"-c1"] = cloneAdd(opt, WriterConcurrency(1))
	}
	testOptions = x
	x = make(map[string][]WriterOption)
	for name, opt := range testOptions {
		x[name] = opt
		if !testing.Short() {
			x[name+"-4k-win"] = cloneAdd(opt, WriterBlockSize(4<<10))
			x[name+"-4M-win"] = cloneAdd(opt, WriterBlockSize(4<<20))
		}
	}
	testOptions = x
	x = make(map[string][]WriterOption)
	for name, opt := range testOptions {
		x[name] = opt
		x[name+"-pad-min"] = cloneAdd(opt, WriterPadding(2), WriterPaddingSrc(zeroReader{}))
		if !testing.Short() {
			x[name+"-pad-8000"] = cloneAdd(opt, WriterPadding(8000), WriterPaddingSrc(zeroReader{}))
			x[name+"-pad-max"] = cloneAdd(opt, WriterPadding(4<<20), WriterPaddingSrc(zeroReader{}))
		}
	}
	for name, opt := range testOptions {
		x[name] = opt
		x[name+"-snappy"] = cloneAdd(opt, WriterSnappyCompat())
		x[name+"-custom"] = cloneAdd(opt, WriterCustomEncoder(snapref.EncodeBlockInto))
	}
	testOptions = x
	return testOptions
}

type zeroReader struct{}

func (zeroReader) Read(p []byte) (int, error) {
	for i := range p {
		p[i] = 0
	}
	return len(p), nil
}

func TestEncoderRegression(t *testing.T) {
	data, err := os.ReadFile("testdata/enc_regressions.zip")
	if err != nil {
		t.Fatal(err)
	}
	zr, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		t.Fatal(err)
	}
	// Same as fuzz test...
	test := func(t *testing.T, data []byte) {
		if testing.Short() && len(data) > 10000 {
			t.SkipNow()
		}
		var blocksTested bool
		for name, opts := range testOptions(t) {
			t.Run(name, func(t *testing.T) {
				var buf bytes.Buffer
				dec := NewReader(nil)
				enc := NewWriter(&buf, opts...)

				if !blocksTested {
					comp := Encode(make([]byte, MaxEncodedLen(len(data))), data)
					decoded, err := Decode(nil, comp)
					if err != nil {
						t.Error(err)
						return
					}
					if !bytes.Equal(data, decoded) {
						t.Error("block decoder mismatch")
						return
					}
					if mel := MaxEncodedLen(len(data)); len(comp) > mel {
						t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
						return
					}
					comp = EncodeBetter(make([]byte, MaxEncodedLen(len(data))), data)
					decoded, err = Decode(nil, comp)
					if err != nil {
						t.Error(err)
						return
					}
					if !bytes.Equal(data, decoded) {
						t.Error("block decoder mismatch")
						return
					}
					if mel := MaxEncodedLen(len(data)); len(comp) > mel {
						t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
						return
					}

					comp = EncodeBest(make([]byte, MaxEncodedLen(len(data))), data)
					decoded, err = Decode(nil, comp)
					if err != nil {
						t.Error(err)
						return
					}
					if !bytes.Equal(data, decoded) {
						t.Error("block decoder mismatch")
						return
					}
					if mel := MaxEncodedLen(len(data)); len(comp) > mel {
						t.Error(fmt.Errorf("MaxEncodedLen Exceed: input: %d, mel: %d, got %d", len(data), mel, len(comp)))
						return
					}
					blocksTested = true
				}

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
				comp := buf.Bytes()
				if enc.pad > 0 && len(comp)%enc.pad != 0 {
					t.Error(fmt.Errorf("wanted size to be multiple of %d, got size %d with remainder %d", enc.pad, len(comp), len(comp)%enc.pad))
					return
				}
				var got []byte
				if !strings.Contains(name, "-snappy") {
					dec.Reset(&buf)
					got, err = io.ReadAll(dec)
				} else {
					sdec := snapref.NewReader(&buf)
					got, err = io.ReadAll(sdec)
				}
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
				err = enc.Close()
				if err != nil {
					t.Error(err)
					return
				}
				if enc.pad > 0 && buf.Len()%enc.pad != 0 {
					t.Error(fmt.Errorf("wanted size to be multiple of %d, got size %d with remainder %d", enc.pad, buf.Len(), buf.Len()%enc.pad))
					return
				}
				if !strings.Contains(name, "-snappy") {
					dec.Reset(&buf)
					got, err = io.ReadAll(dec)
				} else {
					sdec := snapref.NewReader(&buf)
					got, err = io.ReadAll(sdec)
				}
				if err != nil {
					t.Error(err)
					return
				}
				if !bytes.Equal(data, got) {
					t.Error("frame (reset) decoder mismatch")
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

func TestIndex(t *testing.T) {
	fatalErr := func(t testing.TB, err error) {
		if err != nil {
			t.Fatal(err)
		}
	}

	// Create a test corpus
	var input []byte
	if !testing.Short() {
		input = make([]byte, 10<<20)
	} else {
		input = make([]byte, 500<<10)
	}
	rng := rand.New(rand.NewSource(0xabeefcafe))
	rng.Read(input)
	// Make it compressible...
	for i, v := range input {
		input[i] = '0' + v&3
	}
	// Compress it...
	var buf bytes.Buffer
	// We use smaller blocks just for the example...
	enc := NewWriter(&buf, WriterBlockSize(100<<10), WriterAddIndex(), WriterBetterCompression(), WriterConcurrency(runtime.GOMAXPROCS(0)))
	todo := input
	for len(todo) > 0 {
		// Write random sized inputs..
		x := todo[:rng.Intn(1+len(todo)&65535)]
		if len(x) == 0 {
			x = todo[:1]
		}
		_, err := enc.Write(x)
		fatalErr(t, err)
		// Flush once in a while
		if rng.Intn(8) == 0 {
			err = enc.Flush()
			fatalErr(t, err)
		}
		todo = todo[len(x):]
	}

	// Close and also get index...
	idxBytes, err := enc.CloseIndex()
	fatalErr(t, err)
	if false {
		// Load the index.
		var index Index
		_, err = index.Load(idxBytes)
		fatalErr(t, err)
		t.Log(string(index.JSON()))
	}
	// This is our compressed stream...
	compressed := buf.Bytes()
	for wantOffset := int64(0); wantOffset < int64(len(input)); wantOffset += 65531 {
		t.Run(fmt.Sprintf("offset-%d", wantOffset), func(t *testing.T) {
			// Let's assume we want to read from uncompressed offset 'i'
			// and we cannot seek in input, but we have the index.
			want := input[wantOffset:]

			// Load the index.
			var index Index
			_, err = index.Load(idxBytes)
			fatalErr(t, err)

			// Find offset in file:
			compressedOffset, uncompressedOffset, err := index.Find(wantOffset)
			fatalErr(t, err)

			// Offset the input to the compressed offset.
			// Notice how we do not provide any bytes before the offset.
			in := io.Reader(bytes.NewBuffer(compressed[compressedOffset:]))

			// When creating the decoder we must specify that it should not
			// expect a stream identifier at the beginning og the frame.
			dec := NewReader(in, ReaderIgnoreStreamIdentifier())

			// We now have a reader, but it will start outputting at uncompressedOffset,
			// and not the actual offset we want, so skip forward to that.
			toSkip := wantOffset - uncompressedOffset
			err = dec.Skip(toSkip)
			fatalErr(t, err)

			// Read the rest of the stream...
			got, err := io.ReadAll(dec)
			fatalErr(t, err)
			if !bytes.Equal(got, want) {
				t.Error("Result mismatch", wantOffset)
			}

			// Test with stream index...
			for i := io.SeekStart; i <= io.SeekEnd; i++ {
				t.Run(fmt.Sprintf("seek-%d", i), func(t *testing.T) {
					// Read it from a seekable stream
					dec = NewReader(bytes.NewReader(compressed))

					rs, err := dec.ReadSeeker(true, nil)
					fatalErr(t, err)

					// Read a little...
					var tmp = make([]byte, len(input)/2)
					_, err = io.ReadFull(rs, tmp[:])
					fatalErr(t, err)

					toSkip := wantOffset
					switch i {
					case io.SeekStart:
					case io.SeekCurrent:
						toSkip = wantOffset - int64(len(input)/2)
					case io.SeekEnd:
						toSkip = -(int64(len(input)) - wantOffset)
					}
					gotOffset, err := rs.Seek(toSkip, i)
					if gotOffset != wantOffset {
						t.Errorf("got offset %d, want %d", gotOffset, wantOffset)
					}
					// Read the rest of the stream...
					got, err := io.ReadAll(dec)
					fatalErr(t, err)
					if !bytes.Equal(got, want) {
						t.Error("Result mismatch", wantOffset)
					}
				})
			}
			t.Run("ReadAt", func(t *testing.T) {
				// Read it from a seekable stream
				dec = NewReader(bytes.NewReader(compressed))

				rs, err := dec.ReadSeeker(true, nil)
				fatalErr(t, err)

				// Read a little...
				var tmp = make([]byte, len(input)/2)
				_, err = io.ReadFull(rs, tmp[:])
				fatalErr(t, err)
				wantLen := len(tmp)
				if wantLen+int(wantOffset) > len(input) {
					wantLen = len(input) - int(wantOffset)
				}
				// Read from wantOffset
				n, err := rs.ReadAt(tmp, wantOffset)
				if n != wantLen {
					t.Errorf("got length %d, want %d", n, wantLen)
				}
				if err != io.EOF {
					fatalErr(t, err)
				}
				want := want[:n]
				got := tmp[:n]

				// Read the rest of the stream...
				if !bytes.Equal(got, want) {
					t.Error("Result mismatch", wantOffset)
				}
			})
		})
	}
}

func TestWriterPadding(t *testing.T) {
	n := 100
	if testing.Short() {
		n = 5
	}
	rng := rand.New(rand.NewSource(0x1337))
	d := NewReader(nil)

	for i := 0; i < n; i++ {
		padding := (rng.Int() & 0xffff) + 1
		src := make([]byte, (rng.Int()&0xfffff)+1)
		for i := range src {
			src[i] = uint8(rng.Uint32()) & 3
		}
		var dst bytes.Buffer
		e := NewWriter(&dst, WriterPadding(padding))
		// Test the added padding is invisible.
		_, err := io.Copy(e, bytes.NewBuffer(src))
		if err != nil {
			t.Fatal(err)
		}
		err = e.Close()
		if err != nil {
			t.Fatal(err)
		}
		err = e.Close()
		if err != nil {
			t.Fatal(err)
		}

		if dst.Len()%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, dst.Len(), dst.Len()%padding)
		}
		var got bytes.Buffer
		d.Reset(&dst)
		_, err = io.Copy(&got, d)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(src, got.Bytes()) {
			t.Fatal("output mismatch")
		}

		// Try after reset
		dst.Reset()
		e.Reset(&dst)
		_, err = io.Copy(e, bytes.NewBuffer(src))
		if err != nil {
			t.Fatal(err)
		}
		err = e.Close()
		if err != nil {
			t.Fatal(err)
		}
		if dst.Len()%padding != 0 {
			t.Fatalf("wanted size to be multiple of %d, got size %d with remainder %d", padding, dst.Len(), dst.Len()%padding)
		}

		got.Reset()
		d.Reset(&dst)
		_, err = io.Copy(&got, d)
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(src, got.Bytes()) {
			t.Fatal("output mismatch after reset")
		}
	}
}

func TestBigRegularWrites(t *testing.T) {
	var buf [maxBlockSize * 2]byte
	dst := bytes.NewBuffer(nil)
	enc := NewWriter(dst, WriterBestCompression())
	max := uint8(10)
	if testing.Short() {
		max = 4
	}
	for n := uint8(0); n < max; n++ {
		for i := range buf[:] {
			buf[i] = n
		}
		// Writes may not keep a reference to the data beyond the Write call.
		_, err := enc.Write(buf[:])
		if err != nil {
			t.Fatal(err)
		}
	}
	err := enc.Close()
	if err != nil {
		t.Fatal(err)
	}

	dec := NewReader(dst)
	_, err = io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
}

func TestBigEncodeBuffer(t *testing.T) {
	const blockSize = 1 << 20
	var buf [blockSize * 2]byte
	dst := bytes.NewBuffer(nil)
	enc := NewWriter(dst, WriterBlockSize(blockSize), WriterBestCompression())
	max := uint8(10)
	if testing.Short() {
		max = 4
	}
	for n := uint8(0); n < max; n++ {
		// Change the buffer to a new value.
		for i := range buf[:] {
			buf[i] = n
		}
		err := enc.EncodeBuffer(buf[:])
		if err != nil {
			t.Fatal(err)
		}
		// We can write it again since we aren't changing it.
		err = enc.EncodeBuffer(buf[:])
		if err != nil {
			t.Fatal(err)
		}
		err = enc.Flush()
		if err != nil {
			t.Fatal(err)
		}
	}
	err := enc.Close()
	if err != nil {
		t.Fatal(err)
	}

	dec := NewReader(dst)
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
	t.Log(n)
}

func TestBigEncodeBufferSync(t *testing.T) {
	const blockSize = 1 << 20
	var buf [blockSize * 2]byte
	dst := bytes.NewBuffer(nil)
	enc := NewWriter(dst, WriterBlockSize(blockSize), WriterConcurrency(1), WriterBestCompression())
	max := uint8(10)
	if testing.Short() {
		max = 2
	}
	for n := uint8(0); n < max; n++ {
		// Change the buffer to a new value.
		for i := range buf[:] {
			buf[i] = n
		}
		// When WriterConcurrency == 1 we can encode and reuse the buffer.
		err := enc.EncodeBuffer(buf[:])
		if err != nil {
			t.Fatal(err)
		}
	}
	err := enc.Close()
	if err != nil {
		t.Fatal(err)
	}

	dec := NewReader(dst)
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
	t.Log(n)
}

func TestWriterBufferDone(t *testing.T) {
	const blockSize = 1 << 20
	var buffers [][]byte
	for _, size := range []int{10, 100, 10000, blockSize, blockSize * 8} {
		buffers = append(buffers, make([]byte, size))
	}

	dst := io.Discard
	wantNextBuf := 0
	var cbErr error
	enc := NewWriter(dst, WriterBlockSize(blockSize), WriterConcurrency(4), WriterBufferDone(func(b []byte) {
		if !bytes.Equal(b, buffers[wantNextBuf]) && cbErr == nil {
			cbErr = fmt.Errorf("wrong buffer returned, want %v got %v", buffers[wantNextBuf], b)
		}
		// Detect races.
		for i := range b[:] {
			b[i] = 255
		}
		wantNextBuf++
	}))
	for n, buf := range buffers {
		// Change the buffer to a new value.
		for i := range buf[:] {
			buf[i] = byte(n)
		}
		// Send the buffer
		err := enc.EncodeBuffer(buf)
		if err != nil {
			t.Fatal(err)
		}
	}
	err := enc.Close()
	if err != nil {
		t.Fatal(err)
	}
	if wantNextBuf != len(buffers) {
		t.Fatalf("want %d buffers, got %d ", len(buffers), wantNextBuf)
	}
	if cbErr != nil {
		t.Fatal(cbErr)
	}
}

func BenchmarkWriterRandom(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	// Make max window so we never get matches.
	data := make([]byte, 4<<20)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}

	for name, opts := range testOptions(b) {
		w := NewWriter(io.Discard, opts...)
		b.Run(name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(data)))
			for i := 0; i < b.N; i++ {
				err := w.EncodeBuffer(data)
				if err != nil {
					b.Fatal(err)
				}
			}
			// Flush output
			w.Flush()
		})
		w.Close()
	}
}

func BenchmarkReadFromRandom(b *testing.B) {
	rng := rand.New(rand.NewSource(1))
	// Make max window so we never get matches.
	data := make([]byte, 8<<20)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}

	for name, opts := range testOptions(b) {
		w := NewWriter(io.Discard, opts...)
		in := bytes.NewReader(data)
		w.ReadFrom(in)
		b.Run(name, func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			b.SetBytes(int64(len(data)))
			for i := 0; i < b.N; i++ {
				in.Reset(data)
				_, err := w.ReadFrom(in)
				if err != nil {
					b.Fatal(err)
				}
			}
			// Flush output
			w.Flush()
		})
		w.Close()
	}
}

func BenchmarkIndexFind(b *testing.B) {
	fatalErr := func(t testing.TB, err error) {
		if err != nil {
			t.Fatal(err)
		}
	}
	for blocks := 1; blocks <= 65536; blocks *= 2 {
		if blocks == 65536 {
			blocks = 65535
		}

		var index Index
		index.reset(100)
		index.TotalUncompressed = int64(blocks) * 100
		index.TotalCompressed = int64(blocks) * 100
		for i := 0; i < blocks; i++ {
			err := index.add(int64(i*100), int64(i*100))
			fatalErr(b, err)
		}

		rng := rand.New(rand.NewSource(0xabeefcafe))
		b.Run(fmt.Sprintf("blocks-%d", len(index.info)), func(b *testing.B) {
			b.ResetTimer()
			b.ReportAllocs()
			const prime4bytes = 2654435761
			rng2 := rng.Int63()
			for i := 0; i < b.N; i++ {
				rng2 = ((rng2 + prime4bytes) * prime4bytes) >> 32
				// Find offset:
				_, _, err := index.Find(rng2 % (int64(blocks) * 100))
				fatalErr(b, err)
			}
		})
	}
}
