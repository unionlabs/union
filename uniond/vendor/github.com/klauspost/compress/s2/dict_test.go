// Copyright (c) 2023+ Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package s2

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"encoding/base64"
	"fmt"
	"io"
	"math/rand"
	"os"
	"strings"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
	"github.com/klauspost/compress/zstd"
)

func TestDict(t *testing.T) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 128<<10)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}

	// Should match the first 64K
	d := NewDict(append([]byte{0}, data[:65536]...))
	encoded := make([]byte, MaxEncodedLen(len(data)))
	res := encodeBlockDictGo(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded := make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// Add dict that will produce a full match 5000 chars into the input.
	d = NewDict(append([]byte{0}, data[5000:65536+5000]...))
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockDictGo(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// generate copies
	for i := 1; i < len(data); {
		n := rng.Intn(32) + 4
		off := rng.Intn(len(data) - n)
		copy(data[i:], data[off:off+n])
		i += n
	}

	dict := make([]byte, 65536)
	for i := 1; i < len(dict); {
		n := rng.Intn(32) + 4
		off := rng.Intn(65536 - n)
		copy(dict[i:], data[off:off+n])
		i += n
	}
	d = NewDict(dict)
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockDictGo(encoded, data, d)
	if res == 0 || res > len(data)-20000 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		os.WriteFile("decoded.bin", decoded, os.ModePerm)
		os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}
}

func TestDictBetter(t *testing.T) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 128<<10)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}

	// Should match the first 64K
	d := NewDict(append([]byte{0}, data[:65536]...))
	encoded := make([]byte, MaxEncodedLen(len(data)))
	res := encodeBlockBetterDict(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded := make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// Add dict that will produce a full match 5000 chars into the input.
	d = NewDict(append([]byte{0}, data[5000:65536+5000]...))
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockBetterDict(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// generate copies
	for i := 1; i < len(data); {
		n := rng.Intn(32) + 4
		off := rng.Intn(len(data) - n)
		copy(data[i:], data[off:off+n])
		i += n
	}

	dict := make([]byte, 65536)
	for i := 1; i < len(dict); {
		n := rng.Intn(32) + 4
		off := rng.Intn(65536 - n)
		copy(dict[i:], data[off:off+n])
		i += n
	}
	d = NewDict(dict)
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockBetterDict(encoded, data, d)
	if res == 0 || res > len(data)-20000 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		os.WriteFile("decoded.bin", decoded, os.ModePerm)
		os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}
}

func TestDictBest(t *testing.T) {
	rng := rand.New(rand.NewSource(1))
	data := make([]byte, 128<<10)
	for i := range data {
		data[i] = uint8(rng.Intn(256))
	}

	// Should match the first 64K
	d := NewDict(append([]byte{0}, data[:65536]...))
	encoded := make([]byte, MaxEncodedLen(len(data)))
	res := encodeBlockBest(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded := make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// Add dict that will produce a full match 5000 chars into the input.
	d = NewDict(append([]byte{0}, data[5000:65536+5000]...))
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockBest(encoded, data, d)
	if res == 0 || res > len(data)-65500 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}

	// generate copies
	for i := 1; i < len(data); {
		n := rng.Intn(32) + 4
		off := rng.Intn(len(data) - n)
		copy(data[i:], data[off:off+n])
		i += n
	}

	dict := make([]byte, 65536)
	for i := 1; i < len(dict); {
		n := rng.Intn(32) + 4
		off := rng.Intn(65536 - n)
		copy(dict[i:], data[off:off+n])
		i += n
	}
	d = NewDict(dict)
	encoded = make([]byte, MaxEncodedLen(len(data)))
	res = encodeBlockBest(encoded, data, d)
	if res == 0 || res > len(data)-20000 {
		t.Errorf("did no get expected dict saving. Saved %d bytes", len(data)-res)
	}
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	decoded = make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		os.WriteFile("decoded.bin", decoded, os.ModePerm)
		os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}
}

func TestDictBetter2(t *testing.T) {
	// Should match the first 64K
	data := []byte("10 bananas which were brown were added")
	d := NewDict(append([]byte{6}, []byte("Yesterday 25 bananas were added to Benjamins brown bag")...))
	encoded := make([]byte, MaxEncodedLen(len(data)))
	res := encodeBlockBetterDict(encoded, data, d)
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	t.Log(string(encoded))
	decoded := make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}
}

func TestDictBest2(t *testing.T) {
	// Should match the first 64K
	data := []byte("10 bananas which were brown were added")
	d := NewDict(append([]byte{6}, []byte("Yesterday 25 bananas were added to Benjamins brown bag")...))
	encoded := make([]byte, MaxEncodedLen(len(data)))
	res := encodeBlockBest(encoded, data, d)
	encoded = encoded[:res]
	t.Log("saved", len(data)-res, "bytes")
	t.Log(string(encoded))
	decoded := make([]byte, len(data))
	res = s2DecodeDict(decoded, encoded, d)
	if res != 0 {
		t.Fatalf("got result: %d", res)
	}
	if !bytes.Equal(decoded, data) {
		//os.WriteFile("decoded.bin", decoded, os.ModePerm)
		//os.WriteFile("original.bin", data, os.ModePerm)
		t.Fatal("decoded mismatch")
	}
}

func TestDictSize(t *testing.T) {
	//f, err := os.Open("testdata/xlmeta.tar.s2")
	//f, err := os.Open("testdata/broken.tar.s2")
	f, err := os.Open("testdata/github_users_sample_set.tar.s2")
	//f, err := os.Open("testdata/gofiles2.tar.s2")
	//f, err := os.Open("testdata/gosrc.tar.s2")
	if err != nil {
		t.Skip(err)
	}
	stream := NewReader(f)
	in := tar.NewReader(stream)
	//rawDict, err := os.ReadFile("testdata/godict.dictator")
	rawDict, err := os.ReadFile("testdata/gofiles.dict")
	//rawDict, err := os.ReadFile("testdata/gosrc2.dict")
	//rawDict, err := os.ReadFile("testdata/td.dict")
	//rawDict, err := os.ReadFile("testdata/users.dict")
	//rawDict, err := os.ReadFile("testdata/xlmeta.dict")
	if err != nil {
		t.Fatal(err)
	}

	lidx := -1
	if di, err := zstd.InspectDictionary(rawDict); err == nil {
		rawDict = di.Content()
		lidx = len(rawDict) - di.Offsets()[0]
	} else {
		t.Errorf("Loading dict: %v", err)
		return
	}

	searchFor := ""
	if false {
		searchFor = "// Copyright 2022"
	}
	d := MakeDict(rawDict, []byte(searchFor))
	if d == nil {
		t.Fatal("no dict", lidx)
	}

	var totalIn int
	var totalOut int
	var totalCount int
	for {
		h, err := in.Next()
		if err != nil {
			break
		}
		if h.Size == 0 {
			continue
		}
		data := make([]byte, 65536)
		t.Run(h.Name, func(t *testing.T) {
			if int(h.Size) < 65536 {
				data = data[:h.Size]
			} else {
				data = data[:65536]
			}
			_, err := io.ReadFull(in, data)
			if err != nil {
				t.Skip()
			}
			if d == nil {
				// Use first file as dict
				d = MakeDict(data, nil)
			}
			// encode
			encoded := make([]byte, MaxEncodedLen(len(data)))
			totalIn += len(data)
			totalCount++
			//res := encodeBlockBest(encoded, data, nil)
			res := encodeBlockBest(encoded, data, d)
			//res := encodeBlockBetterDict(encoded, data, d)
			//res := encodeBlockBetterGo(encoded, data)
			//res := encodeBlockDictGo(encoded, data, d)
			//			res := encodeBlockGo(encoded, data)
			if res == 0 {
				totalOut += len(data)
				return
			}
			totalOut += res
			encoded = encoded[:res]
			//t.Log("encoded", len(data), "->", res, "saved", len(data)-res, "bytes")
			decoded := make([]byte, len(data))
			res = s2DecodeDict(decoded, encoded, d)
			if res != 0 {
				t.Fatalf("got result: %d", res)
			}
			if !bytes.Equal(decoded, data) {
				os.WriteFile("decoded.bin", decoded, os.ModePerm)
				os.WriteFile("original.bin", data, os.ModePerm)
				t.Fatal("decoded mismatch")
			}
		})
	}
	t.Logf("%d files, %d -> %d (%.2f%%) - %.02f bytes saved/file\n", totalCount, totalIn, totalOut, float64(totalOut*100)/float64(totalIn), float64(totalIn-totalOut)/float64(totalCount))
}

func FuzzDictBlocks(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/enc_regressions.zip", fuzz.TypeRaw, false)
	fuzz.AddFromZip(f, "testdata/fuzz/block-corpus-raw.zip", fuzz.TypeRaw, testing.Short())
	fuzz.AddFromZip(f, "testdata/fuzz/block-corpus-enc.zip", fuzz.TypeGoFuzz, testing.Short())
	fuzz.AddFromZip(f, "testdata/fuzz/dict-corpus-oss.zip", fuzz.TypeOSSFuzz, testing.Short())

	// Fuzzing tweaks:
	const (
		// Max input size:
		maxSize = 8 << 20
	)
	//It does not work with oss-fuzz setup, thus we embed the content of testdata/s2-dict.bin.gz file
	//file, err := os.Open("testdata/s2-dict.bin.gz")

	base64GzipArr := "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"
	gzipArr, err := base64.StdEncoding.DecodeString(base64GzipArr)
	if err != nil {
		f.Fatal(err)
	}
	gzr, err := gzip.NewReader(strings.NewReader(string(gzipArr)))

	if err != nil {
		f.Fatal(err)
	}
	dictBytes, err := io.ReadAll(gzr)
	if err != nil {
		f.Fatal(err)
	}
	dict := NewDict(dictBytes)
	if dict == nil {
		f.Fatal("invalid dict")
	}

	f.Fuzz(func(t *testing.T, data []byte) {
		if len(data) > maxSize {
			return
		}

		writeDst := make([]byte, MaxEncodedLen(len(data)), MaxEncodedLen(len(data))+4)
		writeDst = append(writeDst, 1, 2, 3, 4)
		defer func() {
			got := writeDst[MaxEncodedLen(len(data)):]
			want := []byte{1, 2, 3, 4}
			if !bytes.Equal(got, want) {
				t.Fatalf("want %v, got %v - dest modified outside cap", want, got)
			}
		}()
		compDst := writeDst[:MaxEncodedLen(len(data)):MaxEncodedLen(len(data))] // Hard cap
		decDst := make([]byte, len(data))
		comp := dict.Encode(compDst, data)
		decoded, err := dict.Decode(decDst, comp)
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
		comp = dict.EncodeBetter(compDst, data)
		decoded, err = dict.Decode(decDst, comp)
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

		comp = dict.EncodeBest(compDst, data)
		decoded, err = dict.Decode(decDst, comp)
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
	})
}
