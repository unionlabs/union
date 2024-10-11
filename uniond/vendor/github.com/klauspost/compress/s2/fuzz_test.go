//go:build go1.18
// +build go1.18

package s2

import (
	"bytes"
	"fmt"
	"io"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
	"github.com/klauspost/compress/internal/snapref"
)

func FuzzEncodingBlocks(f *testing.F) {
	fuzz.AddFromZip(f, "testdata/enc_regressions.zip", fuzz.TypeRaw, false)
	fuzz.AddFromZip(f, "testdata/fuzz/block-corpus-raw.zip", fuzz.TypeRaw, testing.Short())
	fuzz.AddFromZip(f, "testdata/fuzz/block-corpus-enc.zip", fuzz.TypeGoFuzz, testing.Short())

	// Fuzzing tweaks:
	const (
		// Max input size:
		maxSize = 8 << 20
	)

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
		comp := Encode(compDst, data)
		decoded, err := Decode(decDst, comp)
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
		comp = EncodeBetter(compDst, data)
		decoded, err = Decode(decDst, comp)
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

		comp = EncodeBest(compDst, data)
		decoded, err = Decode(decDst, comp)
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

		comp = EncodeSnappy(compDst, data)
		decoded, err = snapref.Decode(decDst, comp)
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
		comp = EncodeSnappyBetter(compDst, data)
		decoded, err = snapref.Decode(decDst, comp)
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

		comp = EncodeSnappyBest(compDst, data)
		decoded, err = snapref.Decode(decDst, comp)
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

		concat, err := ConcatBlocks(nil, data, []byte{0})
		if err != nil || concat == nil {
			return
		}

		EstimateBlockSize(data)
		encoded := make([]byte, MaxEncodedLen(len(data)))
		if len(encoded) < MaxEncodedLen(len(data)) || minNonLiteralBlockSize > len(data) || len(data) > maxBlockSize {
			return
		}

		encodeBlockGo(encoded, data)
		encodeBlockBetterGo(encoded, data)
		encodeBlockSnappyGo(encoded, data)
		encodeBlockBetterSnappyGo(encoded, data)
		dst := encodeGo(encoded, data)
		if dst == nil {
			return
		}
	})
}

func FuzzStreamDecode(f *testing.F) {
	enc := NewWriter(nil, WriterBlockSize(8<<10))
	addCompressed := func(b []byte) {
		var buf bytes.Buffer
		enc.Reset(&buf)
		enc.Write(b)
		enc.Close()
		f.Add(buf.Bytes())
	}
	fuzz.ReturnFromZip(f, "testdata/enc_regressions.zip", fuzz.TypeRaw, addCompressed)
	fuzz.ReturnFromZip(f, "testdata/fuzz/block-corpus-raw.zip", fuzz.TypeRaw, addCompressed)
	fuzz.ReturnFromZip(f, "testdata/fuzz/block-corpus-enc.zip", fuzz.TypeGoFuzz, addCompressed)
	dec := NewReader(nil, ReaderIgnoreCRC())
	f.Fuzz(func(t *testing.T, data []byte) {
		// Using Read
		dec.Reset(bytes.NewReader(data))
		io.Copy(io.Discard, dec)

		// Using DecodeConcurrent
		dec.Reset(bytes.NewReader(data))
		dec.DecodeConcurrent(io.Discard, 2)

		// Use ByteReader.
		dec.Reset(bytes.NewReader(data))
		for {
			_, err := dec.ReadByte()
			if err != nil {
				break
			}
		}
	})
}
