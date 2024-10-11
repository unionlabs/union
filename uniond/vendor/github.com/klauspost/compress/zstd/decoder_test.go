// Copyright 2019+ Klaus Post. All rights reserved.
// License information can be found in the LICENSE file.
// Based on work by Yann Collet, released under BSD License.

package zstd

import (
	"bufio"
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"io"
	"log"
	"math/rand"
	"os"
	"path/filepath"
	"reflect"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	// "github.com/DataDog/zstd"
	// zstd "github.com/valyala/gozstd"

	"github.com/klauspost/compress/zstd/internal/xxhash"
)

func TestNewReaderMismatch(t *testing.T) {
	// To identify a potential decoding error, do the following steps:
	// 1) Place the compressed file in testdata, eg 'testdata/backup.bin.zst'
	// 2) Decompress the file to using zstd, so it will be named 'testdata/backup.bin'
	// 3) Run the test. A hash file will be generated 'testdata/backup.bin.hash'
	// 4) The decoder will also run and decode the file. It will stop as soon as a mismatch is found.
	// The hash file will be reused between runs if present.
	const baseFile = "testdata/backup.bin"
	const blockSize = 1024
	hashes, err := os.ReadFile(baseFile + ".hash")
	if os.IsNotExist(err) {
		// Create the hash file.
		f, err := os.Open(baseFile)
		if os.IsNotExist(err) {
			t.Skip("no decompressed file found")
			return
		}
		defer f.Close()
		br := bufio.NewReader(f)
		var tmp [8]byte
		xx := xxhash.New()
		for {
			xx.Reset()
			buf := make([]byte, blockSize)
			n, err := io.ReadFull(br, buf)
			if err != nil {
				if err != io.EOF && err != io.ErrUnexpectedEOF {
					t.Fatal(err)
				}
			}
			if n > 0 {
				_, _ = xx.Write(buf[:n])
				binary.LittleEndian.PutUint64(tmp[:], xx.Sum64())
				hashes = append(hashes, tmp[4:]...)
			}
			if n != blockSize {
				break
			}
		}
		err = os.WriteFile(baseFile+".hash", hashes, os.ModePerm)
		if err != nil {
			// We can continue for now
			t.Error(err)
		}
		t.Log("Saved", len(hashes)/4, "hashes as", baseFile+".hash")
	}

	f, err := os.Open(baseFile + ".zst")
	if os.IsNotExist(err) {
		t.Skip("no compressed file found")
		return
	}
	defer f.Close()
	dec, err := NewReader(f, WithDecoderConcurrency(1))
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	var tmp [8]byte
	xx := xxhash.New()
	var cHash int
	for {
		xx.Reset()
		buf := make([]byte, blockSize)
		n, err := io.ReadFull(dec, buf)
		if err != nil {
			if err != io.EOF && err != io.ErrUnexpectedEOF {
				t.Fatal("block", cHash, "err:", err)
			}
		}
		if n > 0 {
			if cHash+4 > len(hashes) {
				extra, _ := io.Copy(io.Discard, dec)
				t.Fatal("not enough hashes (length mismatch). Only have", len(hashes)/4, "hashes. Got block of", n, "bytes and", extra, "bytes still on stream.")
			}
			_, _ = xx.Write(buf[:n])
			binary.LittleEndian.PutUint64(tmp[:], xx.Sum64())
			want, got := hashes[cHash:cHash+4], tmp[4:]
			if !bytes.Equal(want, got) {
				org, err := os.Open(baseFile)
				if err == nil {
					const sizeBack = 8 << 20
					defer org.Close()
					start := int64(cHash)/4*blockSize - sizeBack
					if start < 0 {
						start = 0
					}
					_, err = org.Seek(start, io.SeekStart)
					if err != nil {
						t.Fatal(err)
					}
					buf2 := make([]byte, sizeBack+1<<20)
					n, _ := io.ReadFull(org, buf2)
					if n > 0 {
						err = os.WriteFile(baseFile+".section", buf2[:n], os.ModePerm)
						if err == nil {
							t.Log("Wrote problematic section to", baseFile+".section")
						}
					}
				}

				t.Fatal("block", cHash/4, "offset", cHash/4*blockSize, "hash mismatch, want:", hex.EncodeToString(want), "got:", hex.EncodeToString(got))
			}
			cHash += 4
		}
		if n != blockSize {
			break
		}
	}
	t.Log("Output matched")
}

type errorReader struct {
	err error
}

func (r *errorReader) Read(p []byte) (int, error) {
	return 0, r.err
}

func TestErrorReader(t *testing.T) {
	wantErr := fmt.Errorf("i'm a failure")
	zr, err := NewReader(&errorReader{err: wantErr})
	if err != nil {
		t.Fatal(err)
	}
	defer zr.Close()

	_, err = io.ReadAll(zr)
	if !errors.Is(err, wantErr) {
		t.Errorf("want error %v, got %v", wantErr, err)
	}
}

type failingWriter struct {
	err error
}

func (f failingWriter) Write(_ []byte) (n int, err error) {
	return 0, f.err
}

func TestErrorWriter(t *testing.T) {
	input := make([]byte, 100)
	cmp := bytes.Buffer{}
	w, err := NewWriter(&cmp)
	if err != nil {
		t.Fatal(err)
	}
	_, _ = rand.Read(input)
	_, err = w.Write(input)
	if err != nil {
		t.Fatal(err)
	}
	err = w.Close()
	if err != nil {
		t.Fatal(err)
	}
	wantErr := fmt.Errorf("i'm a failure")
	zr, err := NewReader(&cmp)
	if err != nil {
		t.Fatal(err)
	}
	defer zr.Close()
	out := failingWriter{err: wantErr}
	_, err = zr.WriteTo(out)
	if !errors.Is(err, wantErr) {
		t.Errorf("error: wanted: %v, got: %v", wantErr, err)
	}
}

func TestNewDecoder(t *testing.T) {
	for _, n := range []int{1, 4} {
		for _, ignoreCRC := range []bool{false, true} {
			t.Run(fmt.Sprintf("cpu-%d", n), func(t *testing.T) {
				newFn := func() (*Decoder, error) {
					return NewReader(nil, WithDecoderConcurrency(n), IgnoreChecksum(ignoreCRC))
				}
				testDecoderFile(t, "testdata/decoder.zip", newFn)
				dec, err := newFn()
				if err != nil {
					t.Fatal(err)
				}
				testDecoderDecodeAll(t, "testdata/decoder.zip", dec)
			})
		}
	}
}

func TestNewDecoderMemory(t *testing.T) {
	defer timeout(60 * time.Second)()
	var testdata bytes.Buffer
	enc, err := NewWriter(&testdata, WithWindowSize(32<<10), WithSingleSegment(false))
	if err != nil {
		t.Fatal(err)
	}
	// Write 256KB
	for i := 0; i < 256; i++ {
		tmp := strings.Repeat(string([]byte{byte(i)}), 1024)
		_, err := enc.Write([]byte(tmp))
		if err != nil {
			t.Fatal(err)
		}
	}
	err = enc.Close()
	if err != nil {
		t.Fatal(err)
	}

	var n = 5000
	if testing.Short() {
		n = 200
	}

	// 16K buffer
	var tmp [16 << 10]byte

	var before, after runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&before)

	var decs = make([]*Decoder, n)
	for i := range decs {
		// Wrap in NopCloser to avoid shortcut.
		input := io.NopCloser(bytes.NewBuffer(testdata.Bytes()))
		decs[i], err = NewReader(input, WithDecoderConcurrency(1), WithDecoderLowmem(true))
		if err != nil {
			t.Fatal(err)
		}
	}

	for i := range decs {
		_, err := io.ReadFull(decs[i], tmp[:])
		if err != nil {
			t.Fatal(err)
		}
	}

	runtime.GC()
	runtime.ReadMemStats(&after)
	size := (after.HeapInuse - before.HeapInuse) / uint64(n) / 1024

	const expect = 124
	t.Log(size, "KiB per decoder")
	// This is not exact science, but fail if we suddenly get more than 2x what we expect.
	if size > expect*2 && !testing.Short() {
		t.Errorf("expected < %dKB per decoder, got %d", expect, size)
	}

	for _, dec := range decs {
		dec.Close()
	}
}

func TestNewDecoderMemoryHighMem(t *testing.T) {
	defer timeout(60 * time.Second)()
	var testdata bytes.Buffer
	enc, err := NewWriter(&testdata, WithWindowSize(32<<10), WithSingleSegment(false))
	if err != nil {
		t.Fatal(err)
	}
	// Write 256KB
	for i := 0; i < 256; i++ {
		tmp := strings.Repeat(string([]byte{byte(i)}), 1024)
		_, err := enc.Write([]byte(tmp))
		if err != nil {
			t.Fatal(err)
		}
	}
	err = enc.Close()
	if err != nil {
		t.Fatal(err)
	}

	var n = 50
	if testing.Short() {
		n = 10
	}

	// 16K buffer
	var tmp [16 << 10]byte

	var before, after runtime.MemStats
	runtime.GC()
	runtime.ReadMemStats(&before)

	var decs = make([]*Decoder, n)
	for i := range decs {
		// Wrap in NopCloser to avoid shortcut.
		input := io.NopCloser(bytes.NewBuffer(testdata.Bytes()))
		decs[i], err = NewReader(input, WithDecoderConcurrency(1), WithDecoderLowmem(false))
		if err != nil {
			t.Fatal(err)
		}
	}

	for i := range decs {
		_, err := io.ReadFull(decs[i], tmp[:])
		if err != nil {
			t.Fatal(err)
		}
	}

	runtime.GC()
	runtime.ReadMemStats(&after)
	size := (after.HeapInuse - before.HeapInuse) / uint64(n) / 1024

	const expect = 3915
	t.Log(size, "KiB per decoder")
	// This is not exact science, but fail if we suddenly get more than 2x what we expect.
	if size > expect*2 && !testing.Short() {
		t.Errorf("expected < %dKB per decoder, got %d", expect, size)
	}

	for _, dec := range decs {
		dec.Close()
	}
}

func TestNewDecoderFrameSize(t *testing.T) {
	defer timeout(60 * time.Second)()
	var testdata bytes.Buffer
	enc, err := NewWriter(&testdata, WithWindowSize(64<<10))
	if err != nil {
		t.Fatal(err)
	}
	// Write 256KB
	for i := 0; i < 256; i++ {
		tmp := strings.Repeat(string([]byte{byte(i)}), 1024)
		_, err := enc.Write([]byte(tmp))
		if err != nil {
			t.Fatal(err)
		}
	}
	err = enc.Close()
	if err != nil {
		t.Fatal(err)
	}
	// Must fail
	dec, err := NewReader(bytes.NewReader(testdata.Bytes()), WithDecoderMaxWindow(32<<10))
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.Copy(io.Discard, dec)
	if err == nil {
		dec.Close()
		t.Fatal("Wanted error, got none")
	}
	dec.Close()

	// Must succeed.
	dec, err = NewReader(bytes.NewReader(testdata.Bytes()), WithDecoderMaxWindow(64<<10))
	if err != nil {
		t.Fatal(err)
	}
	_, err = io.Copy(io.Discard, dec)
	if err != nil {
		dec.Close()
		t.Fatalf("Wanted no error, got %+v", err)
	}
	dec.Close()
}

func TestNewDecoderGood(t *testing.T) {
	for _, n := range []int{1, 4} {
		t.Run(fmt.Sprintf("cpu-%d", n), func(t *testing.T) {
			newFn := func() (*Decoder, error) {
				return NewReader(nil, WithDecoderConcurrency(n))
			}
			testDecoderFile(t, "testdata/good.zip", newFn)
			dec, err := newFn()
			if err != nil {
				t.Fatal(err)
			}
			testDecoderDecodeAll(t, "testdata/good.zip", dec)
		})
	}
}

func TestNewDecoderBad(t *testing.T) {
	var errMap = make(map[string]string)
	if true {
		t.Run("Reader-4", func(t *testing.T) {
			newFn := func() (*Decoder, error) {
				return NewReader(nil, WithDecoderConcurrency(4), WithDecoderMaxMemory(1<<30))
			}
			testDecoderFileBad(t, "testdata/bad.zip", newFn, errMap)

		})
		t.Run("Reader-1", func(t *testing.T) {
			newFn := func() (*Decoder, error) {
				return NewReader(nil, WithDecoderConcurrency(1), WithDecoderMaxMemory(1<<30))
			}
			testDecoderFileBad(t, "testdata/bad.zip", newFn, errMap)
		})
		t.Run("Reader-4-bigmem", func(t *testing.T) {
			newFn := func() (*Decoder, error) {
				return NewReader(nil, WithDecoderConcurrency(4), WithDecoderMaxMemory(1<<30), WithDecoderLowmem(false))
			}
			testDecoderFileBad(t, "testdata/bad.zip", newFn, errMap)

		})
		t.Run("Reader-1-bigmem", func(t *testing.T) {
			newFn := func() (*Decoder, error) {
				return NewReader(nil, WithDecoderConcurrency(1), WithDecoderMaxMemory(1<<30), WithDecoderLowmem(false))
			}
			testDecoderFileBad(t, "testdata/bad.zip", newFn, errMap)
		})
	}
	t.Run("DecodeAll", func(t *testing.T) {
		defer timeout(10 * time.Second)()
		dec, err := NewReader(nil, WithDecoderMaxMemory(1<<30))
		if err != nil {
			t.Fatal(err)
		}
		testDecoderDecodeAllError(t, "testdata/bad.zip", dec, errMap)
	})
	t.Run("DecodeAll-bigmem", func(t *testing.T) {
		defer timeout(10 * time.Second)()
		dec, err := NewReader(nil, WithDecoderMaxMemory(1<<30), WithDecoderLowmem(false))
		if err != nil {
			t.Fatal(err)
		}
		testDecoderDecodeAllError(t, "testdata/bad.zip", dec, errMap)
	})
}

func TestNewDecoderLarge(t *testing.T) {
	newFn := func() (*Decoder, error) {
		return NewReader(nil)
	}
	testDecoderFile(t, "testdata/large.zip", newFn)
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	testDecoderDecodeAll(t, "testdata/large.zip", dec)
}

func TestNewReaderRead(t *testing.T) {
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	_, err = dec.Read([]byte{0})
	if err == nil {
		t.Fatal("Wanted error on uninitialized read, got nil")
	}
	t.Log("correctly got error", err)
}

func TestNewDecoderBig(t *testing.T) {
	if testing.Short() || isRaceTest {
		t.SkipNow()
	}
	file := "testdata/zstd-10kfiles.zip"
	if _, err := os.Stat(file); os.IsNotExist(err) {
		t.Skip("To run extended tests, download https://files.klauspost.com/compress/zstd-10kfiles.zip \n" +
			"and place it in " + file + "\n" + "Running it requires about 5GB of RAM")
	}
	newFn := func() (*Decoder, error) {
		return NewReader(nil)
	}
	testDecoderFile(t, file, newFn)
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	testDecoderDecodeAll(t, file, dec)
}

func TestNewDecoderBigFile(t *testing.T) {
	if testing.Short() || isRaceTest {
		t.SkipNow()
	}
	file := "testdata/enwik9.zst"
	const wantSize = 1000000000
	if _, err := os.Stat(file); os.IsNotExist(err) {
		t.Skip("To run extended tests, download http://mattmahoney.net/dc/enwik9.zip unzip it \n" +
			"compress it with 'zstd -15 -T0 enwik9' and place it in " + file)
	}
	f, err := os.Open(file)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	start := time.Now()
	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
	if n != wantSize {
		t.Errorf("want size %d, got size %d", wantSize, n)
	}
	elapsed := time.Since(start)
	mbpersec := (float64(n) / (1024 * 1024)) / (float64(elapsed) / (float64(time.Second)))
	t.Logf("Decoded %d bytes with %f.2 MB/s", n, mbpersec)
}

func TestNewDecoderSmallFile(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/z000028.zst"
	const wantSize = 39807
	f, err := os.Open(file)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	start := time.Now()
	dec, err := NewReader(f)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
	if n != wantSize {
		t.Errorf("want size %d, got size %d", wantSize, n)
	}
	mbpersec := (float64(n) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Decoded %d bytes with %f.2 MB/s", n, mbpersec)
}

// cursedReader wraps a reader and returns zero bytes every other read.
// This is used to test the ability of the consumer to handle empty reads without EOF,
// which can happen when reading from a network connection.
type cursedReader struct {
	io.Reader
	numReads int
}

func (r *cursedReader) Read(p []byte) (n int, err error) {
	r.numReads++
	if r.numReads%2 == 0 {
		return 0, nil
	}

	return r.Reader.Read(p)
}

func TestNewDecoderZeroLengthReads(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/z000028.zst"
	const wantSize = 39807
	f, err := os.Open(file)
	if err != nil {
		t.Fatal(err)
	}
	defer f.Close()
	dec, err := NewReader(&cursedReader{Reader: f})
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		t.Fatal(err)
	}
	if n != wantSize {
		t.Errorf("want size %d, got size %d", wantSize, n)
	}
}

type readAndBlock struct {
	buf     []byte
	unblock chan struct{}
}

func (r *readAndBlock) Read(p []byte) (int, error) {
	n := copy(p, r.buf)
	if n == 0 {
		<-r.unblock
		return 0, io.EOF
	}
	r.buf = r.buf[n:]
	return n, nil
}

func TestNewDecoderFlushed(t *testing.T) {
	if testing.Short() {
		t.SkipNow()
	}
	file := "testdata/z000028.zst"
	payload, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}
	payload = append(payload, payload...) //2x
	payload = append(payload, payload...) //4x
	payload = append(payload, payload...) //8x
	rng := rand.New(rand.NewSource(0x1337))
	runs := 100
	if testing.Short() {
		runs = 5
	}
	enc, err := NewWriter(nil, WithWindowSize(128<<10))
	if err != nil {
		t.Fatal(err)
	}
	defer enc.Close()
	for i := 0; i < runs; i++ {
		wantSize := rng.Intn(len(payload)-1) + 1
		t.Run(fmt.Sprint("size-", wantSize), func(t *testing.T) {
			var encoded bytes.Buffer
			enc.Reset(&encoded)
			_, err := enc.Write(payload[:wantSize])
			if err != nil {
				t.Fatal(err)
			}
			err = enc.Flush()
			if err != nil {
				t.Fatal(err)
			}

			// We must be able to read back up until the flush...
			r := readAndBlock{
				buf:     encoded.Bytes(),
				unblock: make(chan struct{}),
			}
			defer timeout(5 * time.Second)()
			dec, err := NewReader(&r)
			if err != nil {
				t.Fatal(err)
			}
			defer dec.Close()
			defer close(r.unblock)
			readBack := 0
			dst := make([]byte, 1024)
			for readBack < wantSize {
				// Read until we have enough.
				n, err := dec.Read(dst)
				if err != nil {
					t.Fatal(err)
				}
				readBack += n
			}
		})
	}
}

func TestDecoderRegression(t *testing.T) {
	defer timeout(160 * time.Second)()

	zr := testCreateZipReader("testdata/regression.zip", t)
	dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderLowmem(true), WithDecoderMaxMemory(1<<20))
	if err != nil {
		t.Error(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if testing.Short() && i > 10 {
			continue
		}
		t.Run("Reader-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			err = dec.Reset(r)
			if err != nil {
				t.Error(err)
				return
			}
			got, gotErr := io.ReadAll(dec)
			t.Log("Received:", len(got), gotErr)

			// Check a fresh instance
			r, err = tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			decL, err := NewReader(r, WithDecoderConcurrency(1), WithDecoderLowmem(true), WithDecoderMaxMemory(1<<20))
			if err != nil {
				t.Error(err)
				return
			}
			defer decL.Close()
			got2, gotErr2 := io.ReadAll(decL)
			t.Log("Fresh Reader received:", len(got2), gotErr2)
			if gotErr != gotErr2 {
				if gotErr != nil && gotErr2 != nil && gotErr.Error() != gotErr2.Error() {
					t.Error(gotErr, "!=", gotErr2)
				}
				if (gotErr == nil) != (gotErr2 == nil) {
					t.Error(gotErr, "!=", gotErr2)
				}
			}
			if !bytes.Equal(got2, got) {
				if gotErr != nil {
					t.Log("Buffer mismatch without Reset")
				} else {
					t.Error("Buffer mismatch without Reset")
				}
			}
		})
		t.Run("DecodeAll-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			in, err := io.ReadAll(r)
			if err != nil {
				t.Error(err)
			}
			got, gotErr := dec.DecodeAll(in, make([]byte, 0, len(in)))
			t.Log("Received:", len(got), gotErr)

			// Check if we got the same:
			decL, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderLowmem(true), WithDecoderMaxMemory(1<<20))
			if err != nil {
				t.Error(err)
				return
			}
			defer decL.Close()
			got2, gotErr2 := decL.DecodeAll(in, make([]byte, 0, len(in)/2))
			t.Log("Fresh Reader received:", len(got2), gotErr2)
			if gotErr != gotErr2 {
				if gotErr != nil && gotErr2 != nil && gotErr.Error() != gotErr2.Error() {
					t.Error(gotErr, "!=", gotErr2)
				}
				if (gotErr == nil) != (gotErr2 == nil) {
					t.Error(gotErr, "!=", gotErr2)
				}
			}
			if !bytes.Equal(got2, got) {
				if gotErr != nil {
					t.Log("Buffer mismatch without Reset")
				} else {
					t.Error("Buffer mismatch without Reset")
				}
			}
		})
		t.Run("Match-"+tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			in, err := io.ReadAll(r)
			if err != nil {
				t.Error(err)
			}
			got, gotErr := dec.DecodeAll(in, make([]byte, 0, len(in)))
			t.Log("Received:", len(got), gotErr)

			// Check a fresh instance
			decL, err := NewReader(bytes.NewBuffer(in), WithDecoderConcurrency(1), WithDecoderLowmem(true), WithDecoderMaxMemory(1<<20))
			if err != nil {
				t.Error(err)
				return
			}
			defer decL.Close()
			got2, gotErr2 := io.ReadAll(decL)
			t.Log("Reader Reader received:", len(got2), gotErr2)
			if gotErr != gotErr2 {
				if gotErr != nil && gotErr2 != nil && gotErr.Error() != gotErr2.Error() {
					t.Error(gotErr, "!=", gotErr2)
				}
				if (gotErr == nil) != (gotErr2 == nil) {
					t.Error(gotErr, "!=", gotErr2)
				}
			}
			if !bytes.Equal(got2, got) {
				if gotErr != nil {
					t.Log("Buffer mismatch")
				} else {
					t.Error("Buffer mismatch")
				}
			}
		})
	}
}

func TestShort(t *testing.T) {
	for _, in := range []string{"f", "fo", "foo"} {
		inb := []byte(in)
		dec, err := NewReader(nil)
		if err != nil {
			t.Fatal(err)
		}
		defer dec.Close()

		t.Run(fmt.Sprintf("DecodeAll-%d", len(in)), func(t *testing.T) {
			_, err := dec.DecodeAll(inb, nil)
			if err == nil {
				t.Error("want error, got nil")
			}
		})
		t.Run(fmt.Sprintf("Reader-%d", len(in)), func(t *testing.T) {
			dec.Reset(bytes.NewReader(inb))
			_, err := io.Copy(io.Discard, dec)
			if err == nil {
				t.Error("want error, got nil")
			}
		})
	}
}

func TestDecoder_Reset(t *testing.T) {
	in, err := os.ReadFile("testdata/z000028")
	if err != nil {
		t.Fatal(err)
	}
	in = append(in, in...)
	var e Encoder
	start := time.Now()
	dst := e.EncodeAll(in, nil)
	t.Log("Simple Encoder len", len(in), "-> zstd len", len(dst))
	mbpersec := (float64(len(in)) / (1024 * 1024)) / (float64(time.Since(start)) / (float64(time.Second)))
	t.Logf("Encoded %d bytes with %.2f MB/s", len(in), mbpersec)

	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()
	decoded, err := dec.DecodeAll(dst, nil)
	if err != nil {
		t.Error(err, len(decoded))
	}
	if !bytes.Equal(decoded, in) {
		t.Logf("size = %d, got = %d", len(decoded), len(in))
		t.Fatal("Decoded does not match")
	}
	t.Log("Encoded content matched")

	// Decode using reset+copy
	for i := 0; i < 3; i++ {
		err = dec.Reset(bytes.NewBuffer(dst))
		if err != nil {
			t.Fatal(err)
		}
		var dBuf bytes.Buffer
		n, err := io.Copy(&dBuf, dec)
		if err != nil {
			t.Fatal(err)
		}
		decoded = dBuf.Bytes()
		if int(n) != len(decoded) {
			t.Fatalf("decoded reported length mismatch %d != %d", n, len(decoded))
		}
		if !bytes.Equal(decoded, in) {
			os.WriteFile("testdata/"+t.Name()+"-z000028.got", decoded, os.ModePerm)
			os.WriteFile("testdata/"+t.Name()+"-z000028.want", in, os.ModePerm)
			t.Fatal("Decoded does not match")
		}
	}
	// Test without WriterTo interface support.
	for i := 0; i < 3; i++ {
		err = dec.Reset(bytes.NewBuffer(dst))
		if err != nil {
			t.Fatal(err)
		}
		decoded, err := io.ReadAll(io.NopCloser(dec))
		if err != nil {
			t.Fatal(err)
		}
		if !bytes.Equal(decoded, in) {
			os.WriteFile("testdata/"+t.Name()+"-z000028.got", decoded, os.ModePerm)
			os.WriteFile("testdata/"+t.Name()+"-z000028.want", in, os.ModePerm)
			t.Fatal("Decoded does not match")
		}
	}
}

func TestDecoderMultiFrame(t *testing.T) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", t)
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
		return
	}
	defer dec.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		t.Run(tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			// 2x
			in = append(in, in...)
			if !testing.Short() {
				// 4x
				in = append(in, in...)
				// 8x
				in = append(in, in...)
			}
			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				t.Fatal(err)
			}
			got, err := io.ReadAll(dec)
			if err != nil {
				t.Fatal(err)
			}
			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				t.Fatal(err)
			}
			got2, err := io.ReadAll(dec)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, got2) {
				t.Error("results mismatch")
			}
		})
	}
}

func TestDecoderMultiFrameReset(t *testing.T) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", t)
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
		return
	}
	rng := rand.New(rand.NewSource(1337))
	defer dec.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		t.Run(tt.Name, func(t *testing.T) {
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			// 2x
			in = append(in, in...)
			if !testing.Short() {
				// 4x
				in = append(in, in...)
				// 8x
				in = append(in, in...)
			}
			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				t.Fatal(err)
			}
			got, err := io.ReadAll(dec)
			if err != nil {
				t.Fatal(err)
			}
			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				t.Fatal(err)
			}
			// Read a random number of bytes
			tmp := make([]byte, rng.Intn(len(got)))
			_, err = io.ReadAtLeast(dec, tmp, len(tmp))
			if err != nil {
				t.Fatal(err)
			}
			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				t.Fatal(err)
			}
			got2, err := io.ReadAll(dec)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, got2) {
				t.Error("results mismatch")
			}
		})
	}
}

func testDecoderFile(t *testing.T, fn string, newDec func() (*Decoder, error)) {
	zr := testCreateZipReader(fn, t)
	var want = make(map[string][]byte)
	for _, tt := range zr.File {
		if strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
			return
		}
		want[tt.Name+".zst"], _ = io.ReadAll(r)
	}

	dec, err := newDec()
	if err != nil {
		t.Error(err)
		return
	}
	defer dec.Close()
	for i, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") || (testing.Short() && i > 20) {
			continue
		}
		t.Run("Reader-"+tt.Name, func(t *testing.T) {
			defer timeout(10 * time.Second)()
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			data, err := io.ReadAll(r)
			r.Close()
			if err != nil {
				t.Error(err)
				return
			}
			err = dec.Reset(io.NopCloser(bytes.NewBuffer(data)))
			if err != nil {
				t.Error(err)
				return
			}
			var got []byte
			var gotError error
			var wg sync.WaitGroup
			wg.Add(1)
			go func() {
				got, gotError = io.ReadAll(dec)
				wg.Done()
			}()

			// This decode should not interfere with the stream...
			gotDecAll, err := dec.DecodeAll(data, nil)
			if err != nil {
				t.Error(err)
				if err != ErrCRCMismatch {
					wg.Wait()
					return
				}
			}
			wg.Wait()
			if gotError != nil {
				t.Error(gotError, err)
				if err != ErrCRCMismatch {
					return
				}
			}

			wantB := want[tt.Name]

			compareWith := func(got []byte, displayName, name string) bool {
				if bytes.Equal(wantB, got) {
					return false
				}

				if len(wantB)+len(got) < 1000 {
					t.Logf(" got: %v\nwant: %v", got, wantB)
				} else {
					fileName, _ := filepath.Abs(filepath.Join("testdata", t.Name()+"-want.bin"))
					_ = os.MkdirAll(filepath.Dir(fileName), os.ModePerm)
					err := os.WriteFile(fileName, wantB, os.ModePerm)
					t.Log("Wrote file", fileName, err)

					fileName, _ = filepath.Abs(filepath.Join("testdata", t.Name()+"-"+name+".bin"))
					_ = os.MkdirAll(filepath.Dir(fileName), os.ModePerm)
					err = os.WriteFile(fileName, got, os.ModePerm)
					t.Log("Wrote file", fileName, err)
				}
				t.Logf("Length, want: %d, got: %d", len(wantB), len(got))
				t.Errorf("%s mismatch", displayName)
				return true
			}

			if compareWith(got, "Output", "got") {
				return
			}

			if compareWith(gotDecAll, "DecodeAll Output", "decoded") {
				return
			}

			t.Log(len(got), "bytes returned, matches input, ok!")
		})
	}
}

func testDecoderFileBad(t *testing.T, fn string, newDec func() (*Decoder, error), errMap map[string]string) {
	zr := testCreateZipReader(fn, t)
	var want = make(map[string][]byte)
	for _, tt := range zr.File {
		if strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
			return
		}
		want[tt.Name+".zst"], _ = io.ReadAll(r)
	}

	dec, err := newDec()
	if err != nil {
		t.Error(err)
		return
	}
	defer dec.Close()
	for _, tt := range zr.File {
		t.Run(tt.Name, func(t *testing.T) {
			defer timeout(10 * time.Second)()
			r, err := tt.Open()
			if err != nil {
				t.Error(err)
				return
			}
			defer r.Close()
			err = dec.Reset(r)
			if err != nil {
				t.Error(err)
				return
			}
			got, err := io.ReadAll(dec)
			if err == ErrCRCMismatch && !strings.Contains(tt.Name, "badsum") {
				t.Error(err)
				return
			}
			if err == nil {
				want := errMap[tt.Name]
				if want == "" {
					want = "<error>"
				}
				t.Error("Did not get expected error", want, "- got", len(got), "bytes")
				return
			}
			if errMap[tt.Name] == "" {
				errMap[tt.Name] = err.Error()
			} else {
				want := errMap[tt.Name]
				if want != err.Error() {
					t.Errorf("error mismatch, prev run got %s, now got %s", want, err.Error())
				}
				return
			}
			t.Log("got error", err)
		})
	}
}

func BenchmarkDecoder_DecoderSmall(b *testing.B) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", b)
	dec, err := NewReader(nil, WithDecodeBuffersBelow(1<<30))
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec.Close()
	dec2, err := NewReader(nil, WithDecodeBuffersBelow(0))
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec2.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		b.Run(tt.Name, func(b *testing.B) {
			r, err := tt.Open()
			if err != nil {
				b.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				b.Fatal(err)
			}
			// 2x
			in = append(in, in...)
			// 4x
			in = append(in, in...)
			// 8x
			in = append(in, in...)

			err = dec.Reset(bytes.NewBuffer(in))
			if err != nil {
				b.Fatal(err)
			}
			got, err := io.ReadAll(dec)
			if err != nil {
				b.Fatal(err)
			}
			b.Run("buffered", func(b *testing.B) {
				b.SetBytes(int64(len(got)))
				b.ReportAllocs()
				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					err = dec.Reset(bytes.NewBuffer(in))
					if err != nil {
						b.Fatal(err)
					}
					n, err := io.Copy(io.Discard, dec)
					if err != nil {
						b.Fatal(err)
					}
					if int(n) != len(got) {
						b.Fatalf("want %d, got %d", len(got), n)
					}

				}
			})
			b.Run("unbuffered", func(b *testing.B) {
				b.SetBytes(int64(len(got)))
				b.ReportAllocs()
				b.ResetTimer()
				for i := 0; i < b.N; i++ {
					err = dec2.Reset(bytes.NewBuffer(in))
					if err != nil {
						b.Fatal(err)
					}
					n, err := io.Copy(io.Discard, dec2)
					if err != nil {
						b.Fatal(err)
					}
					if int(n) != len(got) {
						b.Fatalf("want %d, got %d", len(got), n)
					}
				}
			})
		})
	}
}

func BenchmarkDecoder_DecoderReset(b *testing.B) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", b)
	dec, err := NewReader(nil, WithDecodeBuffersBelow(0))
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec.Close()
	bench := func(name string, b *testing.B, opts []DOption, in, want []byte) {
		b.Helper()
		buf := newBytesReader(in)
		dec, err := NewReader(nil, opts...)
		if err != nil {
			b.Fatal(err)
			return
		}
		defer dec.Close()
		b.Run(name, func(b *testing.B) {
			b.SetBytes(1)
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				buf.Reset(in)
				err = dec.Reset(buf)
				if err != nil {
					b.Fatal(err)
				}
			}
		})
	}
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		b.Run(tt.Name, func(b *testing.B) {
			r, err := tt.Open()
			if err != nil {
				b.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				b.Fatal(err)
			}

			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				b.Fatal(err)
			}
			// Disable buffers:
			bench("stream", b, []DOption{WithDecodeBuffersBelow(0)}, in, got)
			bench("stream-single", b, []DOption{WithDecodeBuffersBelow(0), WithDecoderConcurrency(1)}, in, got)
			// Force buffers:
			bench("buffer", b, []DOption{WithDecodeBuffersBelow(1 << 30)}, in, got)
			bench("buffer-single", b, []DOption{WithDecodeBuffersBelow(1 << 30), WithDecoderConcurrency(1)}, in, got)
		})
	}
}

// newBytesReader returns a *bytes.Reader that also supports Bytes() []byte
func newBytesReader(b []byte) *bytesReader {
	return &bytesReader{Reader: bytes.NewReader(b), buf: b}
}

type bytesReader struct {
	*bytes.Reader
	buf []byte
}

func (b *bytesReader) Bytes() []byte {
	n := b.Reader.Len()
	if n > len(b.buf) {
		panic("buffer mismatch")
	}
	return b.buf[len(b.buf)-n:]
}

func (b *bytesReader) Reset(data []byte) {
	b.buf = data
	b.Reader.Reset(data)
}

func BenchmarkDecoder_DecoderNewNoRead(b *testing.B) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", b)
	dec, err := NewReader(nil)
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec.Close()

	bench := func(name string, b *testing.B, opts []DOption, in, want []byte) {
		b.Helper()
		b.Run(name, func(b *testing.B) {
			buf := newBytesReader(in)
			b.SetBytes(1)
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				buf.Reset(in)
				dec, err := NewReader(buf, opts...)
				if err != nil {
					b.Fatal(err)
					return
				}
				dec.Close()
			}
		})
	}
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		b.Run(tt.Name, func(b *testing.B) {
			r, err := tt.Open()
			if err != nil {
				b.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				b.Fatal(err)
			}

			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				b.Fatal(err)
			}
			// Disable buffers:
			bench("stream", b, []DOption{WithDecodeBuffersBelow(0)}, in, got)
			bench("stream-single", b, []DOption{WithDecodeBuffersBelow(0), WithDecoderConcurrency(1)}, in, got)
			// Force buffers:
			bench("buffer", b, []DOption{WithDecodeBuffersBelow(1 << 30)}, in, got)
			bench("buffer-single", b, []DOption{WithDecodeBuffersBelow(1 << 30), WithDecoderConcurrency(1)}, in, got)
		})
	}
}

func BenchmarkDecoder_DecoderNewSomeRead(b *testing.B) {
	var buf [1 << 20]byte
	bench := func(name string, b *testing.B, opts []DOption, in *os.File) {
		b.Helper()
		b.Run(name, func(b *testing.B) {
			//b.ReportAllocs()
			b.ResetTimer()
			var heapTotal int64
			var m runtime.MemStats
			for i := 0; i < b.N; i++ {
				runtime.GC()
				runtime.ReadMemStats(&m)
				heapTotal -= int64(m.HeapInuse)
				_, err := in.Seek(io.SeekStart, 0)
				if err != nil {
					b.Fatal(err)
				}

				dec, err := NewReader(in, opts...)
				if err != nil {
					b.Fatal(err)
				}
				// Read 16 MB
				_, err = io.CopyBuffer(io.Discard, io.LimitReader(dec, 16<<20), buf[:])
				if err != nil {
					b.Fatal(err)
				}
				runtime.GC()
				runtime.ReadMemStats(&m)
				heapTotal += int64(m.HeapInuse)

				dec.Close()
			}
			b.ReportMetric(float64(heapTotal)/float64(b.N), "b/op")
		})
	}
	files := []string{"testdata/000002.map.win32K.zst", "testdata/000002.map.win1MB.zst", "testdata/000002.map.win8MB.zst"}
	for _, file := range files {
		if !strings.HasSuffix(file, ".zst") {
			continue
		}
		r, err := os.Open(file)
		if err != nil {
			b.Fatal(err)
		}
		defer r.Close()

		b.Run(file, func(b *testing.B) {
			bench("stream-single", b, []DOption{WithDecodeBuffersBelow(0), WithDecoderConcurrency(1)}, r)
			bench("stream-single-himem", b, []DOption{WithDecodeBuffersBelow(0), WithDecoderConcurrency(1), WithDecoderLowmem(false)}, r)
		})
	}
}

func BenchmarkDecoder_DecodeAll(b *testing.B) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", b)
	dec, err := NewReader(nil, WithDecoderConcurrency(1))
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		b.Run(tt.Name, func(b *testing.B) {
			r, err := tt.Open()
			if err != nil {
				b.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				b.Fatal(err)
			}
			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				b.Fatal(err)
			}
			b.SetBytes(int64(len(got)))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				_, err = dec.DecodeAll(in, got[:0])
				if err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}

func BenchmarkDecoder_DecodeAllFiles(b *testing.B) {
	filepath.Walk("../testdata/", func(path string, info os.FileInfo, err error) error {
		if info.IsDir() || info.Size() < 100 {
			return nil
		}
		b.Run(filepath.Base(path), func(b *testing.B) {
			raw, err := os.ReadFile(path)
			if err != nil {
				b.Error(err)
			}
			for i := SpeedFastest; i <= SpeedBestCompression; i++ {
				if testing.Short() && i > SpeedFastest {
					break
				}
				b.Run(i.String(), func(b *testing.B) {
					enc, err := NewWriter(nil, WithEncoderLevel(i), WithSingleSegment(true))
					if err != nil {
						b.Error(err)
					}
					encoded := enc.EncodeAll(raw, nil)
					if err != nil {
						b.Error(err)
					}
					dec, err := NewReader(nil, WithDecoderConcurrency(1))
					if err != nil {
						b.Error(err)
					}
					decoded, err := dec.DecodeAll(encoded, nil)
					if err != nil {
						b.Error(err)
					}
					b.SetBytes(int64(len(raw)))
					b.ReportAllocs()
					b.ResetTimer()
					for i := 0; i < b.N; i++ {
						decoded, err = dec.DecodeAll(encoded, decoded[:0])
						if err != nil {
							b.Error(err)
						}
					}
					b.ReportMetric(100*float64(len(encoded))/float64(len(raw)), "pct")
				})
			}
		})
		return nil
	})
}

func BenchmarkDecoder_DecodeAllFilesP(b *testing.B) {
	filepath.Walk("../testdata/", func(path string, info os.FileInfo, err error) error {
		if info.IsDir() || info.Size() < 100 {
			return nil
		}
		b.Run(filepath.Base(path), func(b *testing.B) {
			raw, err := os.ReadFile(path)
			if err != nil {
				b.Error(err)
			}
			for i := SpeedFastest; i <= SpeedBestCompression; i++ {
				if testing.Short() && i > SpeedFastest {
					break
				}
				b.Run(i.String(), func(b *testing.B) {
					enc, err := NewWriter(nil, WithEncoderLevel(i), WithSingleSegment(true))
					if err != nil {
						b.Error(err)
					}
					encoded := enc.EncodeAll(raw, nil)
					if err != nil {
						b.Error(err)
					}
					dec, err := NewReader(nil, WithDecoderConcurrency(0))
					if err != nil {
						b.Error(err)
					}
					raw, err := dec.DecodeAll(encoded, nil)
					if err != nil {
						b.Error(err)
					}

					b.SetBytes(int64(len(raw)))
					b.ReportAllocs()
					b.ResetTimer()
					b.RunParallel(func(pb *testing.PB) {
						buf := make([]byte, cap(raw))
						var err error
						for pb.Next() {
							buf, err = dec.DecodeAll(encoded, buf[:0])
							if err != nil {
								b.Error(err)
							}
						}
					})
					b.ReportMetric(100*float64(len(encoded))/float64(len(raw)), "pct")
				})
			}
		})
		return nil
	})
}

func BenchmarkDecoder_DecodeAllParallel(b *testing.B) {
	zr := testCreateZipReader("testdata/benchdecoder.zip", b)
	dec, err := NewReader(nil, WithDecoderConcurrency(runtime.GOMAXPROCS(0)))
	if err != nil {
		b.Fatal(err)
		return
	}
	defer dec.Close()
	for _, tt := range zr.File {
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		b.Run(tt.Name, func(b *testing.B) {
			r, err := tt.Open()
			if err != nil {
				b.Fatal(err)
			}
			defer r.Close()
			in, err := io.ReadAll(r)
			if err != nil {
				b.Fatal(err)
			}
			got, err := dec.DecodeAll(in, nil)
			if err != nil {
				b.Fatal(err)
			}
			b.SetBytes(int64(len(got)))
			b.ReportAllocs()
			b.ResetTimer()
			b.RunParallel(func(pb *testing.PB) {
				got := make([]byte, cap(got))
				for pb.Next() {
					_, err = dec.DecodeAll(in, got[:0])
					if err != nil {
						b.Fatal(err)
					}
				}
			})
			b.ReportMetric(100*float64(len(in))/float64(len(got)), "pct")
		})
	}
}

func benchmarkDecoderWithFile(path string, b *testing.B) {
	_, err := os.Stat(path)
	if err != nil {
		if os.IsNotExist(err) {
			b.Skipf("Missing %s", path)
			return
		}
		b.Fatal(err)
	}

	data, err := os.ReadFile(path)
	if err != nil {
		b.Fatal(err)
	}
	dec, err := NewReader(bytes.NewBuffer(data), WithDecoderLowmem(false), WithDecoderConcurrency(1))
	if err != nil {
		b.Fatal(err)
	}
	n, err := io.Copy(io.Discard, dec)
	if err != nil {
		b.Fatal(err)
	}

	b.Run("multithreaded-writer", func(b *testing.B) {
		dec, err := NewReader(nil, WithDecoderLowmem(true))
		if err != nil {
			b.Fatal(err)
		}
		b.SetBytes(n)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			err = dec.Reset(bytes.NewBuffer(data))
			if err != nil {
				b.Fatal(err)
			}
			_, err := io.CopyN(io.Discard, dec, n)
			if err != nil {
				b.Fatal(err)
			}
		}
	})

	b.Run("multithreaded-writer-himem", func(b *testing.B) {
		dec, err := NewReader(nil, WithDecoderLowmem(false))
		if err != nil {
			b.Fatal(err)
		}

		b.SetBytes(n)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			err = dec.Reset(bytes.NewBuffer(data))
			if err != nil {
				b.Fatal(err)
			}
			_, err := io.CopyN(io.Discard, dec, n)
			if err != nil {
				b.Fatal(err)
			}
		}
	})

	b.Run("singlethreaded-writer", func(b *testing.B) {
		dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderLowmem(true))
		if err != nil {
			b.Fatal(err)
		}

		b.SetBytes(n)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			err = dec.Reset(bytes.NewBuffer(data))
			if err != nil {
				b.Fatal(err)
			}
			_, err := io.CopyN(io.Discard, dec, n)
			if err != nil {
				b.Fatal(err)
			}
		}
	})

	b.Run("singlethreaded-writerto", func(b *testing.B) {
		dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderLowmem(true))
		if err != nil {
			b.Fatal(err)
		}

		b.SetBytes(n)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			err = dec.Reset(bytes.NewBuffer(data))
			if err != nil {
				b.Fatal(err)
			}
			// io.Copy will use io.WriterTo
			_, err := io.Copy(io.Discard, dec)
			if err != nil {
				b.Fatal(err)
			}
		}
	})
	b.Run("singlethreaded-himem", func(b *testing.B) {
		dec, err := NewReader(nil, WithDecoderConcurrency(1), WithDecoderLowmem(false))
		if err != nil {
			b.Fatal(err)
		}

		b.SetBytes(n)
		b.ReportAllocs()
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			err = dec.Reset(bytes.NewBuffer(data))
			if err != nil {
				b.Fatal(err)
			}
			// io.Copy will use io.WriterTo
			_, err := io.Copy(io.Discard, dec)
			if err != nil {
				b.Fatal(err)
			}
		}
	})
}

func BenchmarkDecoderSilesia(b *testing.B) {
	benchmarkDecoderWithFile("testdata/silesia.tar.zst", b)
}

func BenchmarkDecoderEnwik9(b *testing.B) {
	benchmarkDecoderWithFile("testdata/enwik9.zst", b)
}

func BenchmarkDecoderWithCustomFiles(b *testing.B) {
	const info = "To run benchmark on custom .zst files, please place your files in subdirectory 'testdata/benchmark-custom'.\nEach file is tested in a separate benchmark, thus it is possible to select files with the standard command 'go test -bench BenchmarkDecoderWithCustomFiles/<pattern>."

	const subdir = "testdata/benchmark-custom"

	if _, err := os.Stat(subdir); os.IsNotExist(err) {
		b.Skip(info)
	}

	files, err := filepath.Glob(filepath.Join(subdir, "*.zst"))
	if err != nil {
		b.Error(err)
		return
	}

	if len(files) == 0 {
		b.Skip(info)
	}

	for _, path := range files {
		name := filepath.Base(path)
		b.Run(name, func(b *testing.B) { benchmarkDecoderWithFile(path, b) })
	}
}

func testDecoderDecodeAll(t *testing.T, fn string, dec *Decoder) {
	zr := testCreateZipReader(fn, t)
	var want = make(map[string][]byte)
	for _, tt := range zr.File {
		if strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		r, err := tt.Open()
		if err != nil {
			t.Fatal(err)
			return
		}
		want[tt.Name+".zst"], _ = io.ReadAll(r)
	}
	var wg sync.WaitGroup
	for i, tt := range zr.File {
		tt := tt
		if !strings.HasSuffix(tt.Name, ".zst") || (testing.Short() && i > 20) {
			continue
		}
		wg.Add(1)
		t.Run("DecodeAll-"+tt.Name, func(t *testing.T) {
			defer wg.Done()
			t.Parallel()
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			wantB := want[tt.Name]
			// make a buffer that is too small.
			got, err := dec.DecodeAll(in, make([]byte, 10, 200))
			if err != nil {
				t.Error(err)
			}
			if len(got) < 10 {
				t.Fatal("didn't get input back")
			}
			got = got[10:]
			if !bytes.Equal(wantB, got) {
				if len(wantB)+len(got) < 1000 {
					t.Logf(" got: %v\nwant: %v", got, wantB)
				} else {
					fileName, _ := filepath.Abs(filepath.Join("testdata", t.Name()+"-want.bin"))
					_ = os.MkdirAll(filepath.Dir(fileName), os.ModePerm)
					err := os.WriteFile(fileName, wantB, os.ModePerm)
					t.Log("Wrote file", fileName, err)

					fileName, _ = filepath.Abs(filepath.Join("testdata", t.Name()+"-got.bin"))
					_ = os.MkdirAll(filepath.Dir(fileName), os.ModePerm)
					err = os.WriteFile(fileName, got, os.ModePerm)
					t.Log("Wrote file", fileName, err)
				}
				t.Logf("Length, want: %d, got: %d", len(wantB), len(got))
				t.Error("Output mismatch")
				return
			}
			t.Log(len(got), "bytes returned, matches input, ok!")
		})
	}
	go func() {
		wg.Wait()
		dec.Close()
	}()
}

func testDecoderDecodeAllError(t *testing.T, fn string, dec *Decoder, errMap map[string]string) {
	zr := testCreateZipReader(fn, t)

	var wg sync.WaitGroup
	for _, tt := range zr.File {
		tt := tt
		if !strings.HasSuffix(tt.Name, ".zst") {
			continue
		}
		wg.Add(1)
		t.Run(tt.Name, func(t *testing.T) {
			defer wg.Done()
			r, err := tt.Open()
			if err != nil {
				t.Fatal(err)
			}
			in, err := io.ReadAll(r)
			if err != nil {
				t.Fatal(err)
			}
			// make a buffer that is small.
			got, err := dec.DecodeAll(in, make([]byte, 0, 20))
			if err == nil {
				t.Error("Did not get expected error, got", len(got), "bytes")
				return
			}
			t.Log(err)
			if errMap[tt.Name] == "" {
				t.Error("cannot check error")
			} else {
				want := errMap[tt.Name]
				if want != err.Error() {
					if want == ErrFrameSizeMismatch.Error() && err == ErrDecoderSizeExceeded {
						return
					}
					if want == ErrWindowSizeExceeded.Error() && err == ErrDecoderSizeExceeded {
						return
					}
					t.Errorf("error mismatch, prev run got %s, now got %s", want, err.Error())
				}
				return
			}
		})
	}
	go func() {
		wg.Wait()
		dec.Close()
	}()
}

// Test our predefined tables are correct.
// We don't predefine them, since this also tests our transformations.
// Reference from here: https://github.com/facebook/zstd/blob/ededcfca57366461021c922720878c81a5854a0a/lib/decompress/zstd_decompress_block.c#L234
func TestPredefTables(t *testing.T) {
	initPredefined()
	x := func(nextState uint16, nbAddBits, nbBits uint8, baseVal uint32) decSymbol {
		return newDecSymbol(nbBits, nbAddBits, nextState, baseVal)
	}
	for i := range fsePredef[:] {
		var want []decSymbol
		switch tableIndex(i) {
		case tableLiteralLengths:
			want = []decSymbol{
				/* nextState, nbAddBits, nbBits, baseVal */
				x(0, 0, 4, 0), x(16, 0, 4, 0),
				x(32, 0, 5, 1), x(0, 0, 5, 3),
				x(0, 0, 5, 4), x(0, 0, 5, 6),
				x(0, 0, 5, 7), x(0, 0, 5, 9),
				x(0, 0, 5, 10), x(0, 0, 5, 12),
				x(0, 0, 6, 14), x(0, 1, 5, 16),
				x(0, 1, 5, 20), x(0, 1, 5, 22),
				x(0, 2, 5, 28), x(0, 3, 5, 32),
				x(0, 4, 5, 48), x(32, 6, 5, 64),
				x(0, 7, 5, 128), x(0, 8, 6, 256),
				x(0, 10, 6, 1024), x(0, 12, 6, 4096),
				x(32, 0, 4, 0), x(0, 0, 4, 1),
				x(0, 0, 5, 2), x(32, 0, 5, 4),
				x(0, 0, 5, 5), x(32, 0, 5, 7),
				x(0, 0, 5, 8), x(32, 0, 5, 10),
				x(0, 0, 5, 11), x(0, 0, 6, 13),
				x(32, 1, 5, 16), x(0, 1, 5, 18),
				x(32, 1, 5, 22), x(0, 2, 5, 24),
				x(32, 3, 5, 32), x(0, 3, 5, 40),
				x(0, 6, 4, 64), x(16, 6, 4, 64),
				x(32, 7, 5, 128), x(0, 9, 6, 512),
				x(0, 11, 6, 2048), x(48, 0, 4, 0),
				x(16, 0, 4, 1), x(32, 0, 5, 2),
				x(32, 0, 5, 3), x(32, 0, 5, 5),
				x(32, 0, 5, 6), x(32, 0, 5, 8),
				x(32, 0, 5, 9), x(32, 0, 5, 11),
				x(32, 0, 5, 12), x(0, 0, 6, 15),
				x(32, 1, 5, 18), x(32, 1, 5, 20),
				x(32, 2, 5, 24), x(32, 2, 5, 28),
				x(32, 3, 5, 40), x(32, 4, 5, 48),
				x(0, 16, 6, 65536), x(0, 15, 6, 32768),
				x(0, 14, 6, 16384), x(0, 13, 6, 8192),
			}
		case tableOffsets:
			want = []decSymbol{
				/* nextState, nbAddBits, nbBits, baseVal */
				x(0, 0, 5, 0), x(0, 6, 4, 61),
				x(0, 9, 5, 509), x(0, 15, 5, 32765),
				x(0, 21, 5, 2097149), x(0, 3, 5, 5),
				x(0, 7, 4, 125), x(0, 12, 5, 4093),
				x(0, 18, 5, 262141), x(0, 23, 5, 8388605),
				x(0, 5, 5, 29), x(0, 8, 4, 253),
				x(0, 14, 5, 16381), x(0, 20, 5, 1048573),
				x(0, 2, 5, 1), x(16, 7, 4, 125),
				x(0, 11, 5, 2045), x(0, 17, 5, 131069),
				x(0, 22, 5, 4194301), x(0, 4, 5, 13),
				x(16, 8, 4, 253), x(0, 13, 5, 8189),
				x(0, 19, 5, 524285), x(0, 1, 5, 1),
				x(16, 6, 4, 61), x(0, 10, 5, 1021),
				x(0, 16, 5, 65533), x(0, 28, 5, 268435453),
				x(0, 27, 5, 134217725), x(0, 26, 5, 67108861),
				x(0, 25, 5, 33554429), x(0, 24, 5, 16777213),
			}
		case tableMatchLengths:
			want = []decSymbol{
				/* nextState, nbAddBits, nbBits, baseVal */
				x(0, 0, 6, 3), x(0, 0, 4, 4),
				x(32, 0, 5, 5), x(0, 0, 5, 6),
				x(0, 0, 5, 8), x(0, 0, 5, 9),
				x(0, 0, 5, 11), x(0, 0, 6, 13),
				x(0, 0, 6, 16), x(0, 0, 6, 19),
				x(0, 0, 6, 22), x(0, 0, 6, 25),
				x(0, 0, 6, 28), x(0, 0, 6, 31),
				x(0, 0, 6, 34), x(0, 1, 6, 37),
				x(0, 1, 6, 41), x(0, 2, 6, 47),
				x(0, 3, 6, 59), x(0, 4, 6, 83),
				x(0, 7, 6, 131), x(0, 9, 6, 515),
				x(16, 0, 4, 4), x(0, 0, 4, 5),
				x(32, 0, 5, 6), x(0, 0, 5, 7),
				x(32, 0, 5, 9), x(0, 0, 5, 10),
				x(0, 0, 6, 12), x(0, 0, 6, 15),
				x(0, 0, 6, 18), x(0, 0, 6, 21),
				x(0, 0, 6, 24), x(0, 0, 6, 27),
				x(0, 0, 6, 30), x(0, 0, 6, 33),
				x(0, 1, 6, 35), x(0, 1, 6, 39),
				x(0, 2, 6, 43), x(0, 3, 6, 51),
				x(0, 4, 6, 67), x(0, 5, 6, 99),
				x(0, 8, 6, 259), x(32, 0, 4, 4),
				x(48, 0, 4, 4), x(16, 0, 4, 5),
				x(32, 0, 5, 7), x(32, 0, 5, 8),
				x(32, 0, 5, 10), x(32, 0, 5, 11),
				x(0, 0, 6, 14), x(0, 0, 6, 17),
				x(0, 0, 6, 20), x(0, 0, 6, 23),
				x(0, 0, 6, 26), x(0, 0, 6, 29),
				x(0, 0, 6, 32), x(0, 16, 6, 65539),
				x(0, 15, 6, 32771), x(0, 14, 6, 16387),
				x(0, 13, 6, 8195), x(0, 12, 6, 4099),
				x(0, 11, 6, 2051), x(0, 10, 6, 1027),
			}
		}
		pre := fsePredef[i]
		got := pre.dt[:1<<pre.actualTableLog]
		if !reflect.DeepEqual(got, want) {
			t.Logf("want: %v", want)
			t.Logf("got : %v", got)
			t.Errorf("Predefined table %d incorrect, len(got) = %d, len(want) = %d", i, len(got), len(want))
		}
	}
}

func TestResetNil(t *testing.T) {
	dec, err := NewReader(nil)
	if err != nil {
		t.Fatal(err)
	}
	defer dec.Close()

	_, err = io.ReadAll(dec)
	if err != ErrDecoderNilInput {
		t.Fatalf("Expected ErrDecoderNilInput when decoding from a nil reader, got %v", err)
	}

	emptyZstdBlob := []byte{40, 181, 47, 253, 32, 0, 1, 0, 0}

	dec.Reset(bytes.NewBuffer(emptyZstdBlob))

	result, err := io.ReadAll(dec)
	if err != nil && err != io.EOF {
		t.Fatal(err)
	}
	if len(result) != 0 {
		t.Fatalf("Expected to read 0 bytes, actually read %d", len(result))
	}

	dec.Reset(nil)

	_, err = io.ReadAll(dec)
	if err != ErrDecoderNilInput {
		t.Fatalf("Expected ErrDecoderNilInput when decoding from a nil reader, got %v", err)
	}

	dec.Reset(bytes.NewBuffer(emptyZstdBlob))

	result, err = io.ReadAll(dec)
	if err != nil && err != io.EOF {
		t.Fatal(err)
	}
	if len(result) != 0 {
		t.Fatalf("Expected to read 0 bytes, actually read %d", len(result))
	}
}

func TestIgnoreChecksum(t *testing.T) {
	// zstd file containing text "compress\n" and has an xxhash checksum
	zstdBlob := []byte{0x28, 0xb5, 0x2f, 0xfd, 0x24, 0x09, 0x49, 0x00, 0x00, 'C', 'o', 'm', 'p', 'r', 'e', 's', 's', '\n', 0x79, 0x6e, 0xe0, 0xd2}

	// replace letter 'c' with 'C', so decoding should fail.
	zstdBlob[9] = 'C'

	{
		// Check if the file is indeed incorrect
		dec, err := NewReader(nil)
		if err != nil {
			t.Fatal(err)
		}
		defer dec.Close()

		dec.Reset(bytes.NewBuffer(zstdBlob))

		_, err = io.ReadAll(dec)
		if err == nil {
			t.Fatal("Expected decoding error")
		}

		if !errors.Is(err, ErrCRCMismatch) {
			t.Fatalf("Expected checksum error, got '%s'", err)
		}
	}

	{
		// Ignore CRC error and decompress the content
		dec, err := NewReader(nil, IgnoreChecksum(true))
		if err != nil {
			t.Fatal(err)
		}
		defer dec.Close()

		dec.Reset(bytes.NewBuffer(zstdBlob))

		res, err := io.ReadAll(dec)
		if err != nil {
			t.Fatalf("Unexpected error: '%s'", err)
		}

		want := []byte{'C', 'o', 'm', 'p', 'r', 'e', 's', 's', '\n'}
		if !bytes.Equal(res, want) {
			t.Logf("want: %s", want)
			t.Logf("got:  %s", res)
			t.Fatalf("Wrong output")
		}
	}
}

func timeout(after time.Duration) (cancel func()) {
	if isRaceTest {
		return func() {}
	}
	c := time.After(after)
	cc := make(chan struct{})
	go func() {
		select {
		case <-cc:
			return
		case <-c:
			buf := make([]byte, 1<<20)
			stacklen := runtime.Stack(buf, true)
			log.Printf("=== Timeout, assuming deadlock ===\n*** goroutine dump...\n%s\n*** end\n", string(buf[:stacklen]))
			os.Exit(2)
		}
	}()
	return func() {
		close(cc)
	}
}

func TestWithDecodeAllCapLimit(t *testing.T) {
	var encs []*Encoder
	var decs []*Decoder
	addEnc := func(e *Encoder, _ error) {
		encs = append(encs, e)
	}
	addDec := func(d *Decoder, _ error) {
		decs = append(decs, d)
	}
	addEnc(NewWriter(nil, WithZeroFrames(true), WithWindowSize(4<<10)))
	addEnc(NewWriter(nil, WithEncoderConcurrency(1), WithWindowSize(4<<10)))
	addEnc(NewWriter(nil, WithZeroFrames(false), WithWindowSize(4<<10)))
	addEnc(NewWriter(nil, WithWindowSize(128<<10)))
	addDec(NewReader(nil, WithDecodeAllCapLimit(true)))
	addDec(NewReader(nil, WithDecodeAllCapLimit(true), WithDecoderConcurrency(1)))
	addDec(NewReader(nil, WithDecodeAllCapLimit(true), WithDecoderLowmem(true)))
	addDec(NewReader(nil, WithDecodeAllCapLimit(true), WithDecoderMaxWindow(128<<10)))
	addDec(NewReader(nil, WithDecodeAllCapLimit(true), WithDecoderMaxMemory(1<<20)))
	for sz := 0; sz < 1<<20; sz = (sz + 1) * 2 {
		sz := sz
		t.Run(strconv.Itoa(sz), func(t *testing.T) {
			t.Parallel()
			for ei, enc := range encs {
				for di, dec := range decs {
					t.Run(fmt.Sprintf("e%d:d%d", ei, di), func(t *testing.T) {
						encoded := enc.EncodeAll(make([]byte, sz), nil)
						for i := sz - 1; i < sz+1; i++ {
							if i < 0 {
								continue
							}
							const existinglen = 5
							got, err := dec.DecodeAll(encoded, make([]byte, existinglen, i+existinglen))
							if i < sz {
								if err != ErrDecoderSizeExceeded {
									t.Errorf("cap: %d, want %v, got %v", i, ErrDecoderSizeExceeded, err)
								}
							} else {
								if err != nil {
									t.Errorf("cap: %d, want %v, got %v", i, nil, err)
									continue
								}
								if len(got) != existinglen+i {
									t.Errorf("cap: %d, want output size %d, got %d", i, existinglen+i, len(got))
								}
							}
						}
					})
				}
			}
		})
	}
}
