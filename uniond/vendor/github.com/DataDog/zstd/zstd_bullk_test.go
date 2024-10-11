package zstd

import (
	"bytes"
	"encoding/base64"
	"math/rand"
	"regexp"
	"strings"
	"testing"
)

var dictBase64 string = `
	N6Qw7IsuFDIdENCSQjr//////4+QlekuNkmXbUBIkIDiVRX7H4AzAFCgQCFCO9oHAAAEQEuSikaK
	Dg51OYghBYgBAAAAAAAAAAAAAAAAAAAAANQVpmRQGQAAAAAAAAAAAAAAAAABAAAABAAAAAgAAABo
	ZWxwIEpvaW4gZW5naW5lZXJzIGVuZ2luZWVycyBmdXR1cmUgbG92ZSB0aGF0IGFyZWlsZGluZyB1
	c2UgaGVscCBoZWxwIHVzaGVyIEpvaW4gdXNlIGxvdmUgdXMgSm9pbiB1bmQgaW4gdXNoZXIgdXNo
	ZXIgYSBwbGF0Zm9ybSB1c2UgYW5kIGZ1dHVyZQ==`
var dict []byte
var compressedPayload []byte

func init() {
	var err error
	dict, err = base64.StdEncoding.DecodeString(regexp.MustCompile(`\s+`).ReplaceAllString(dictBase64, ""))
	if err != nil {
		panic("failed to create dictionary")
	}
	p, err := NewBulkProcessor(dict, BestSpeed)
	if err != nil {
		panic("failed to create bulk processor")
	}
	compressedPayload, err = p.Compress(nil, []byte("We're building a platform that engineers love to use. Join us, and help usher in the future."))
	if err != nil {
		panic("failed to compress payload")
	}
}

func newBulkProcessor(t testing.TB, dict []byte, level int) *BulkProcessor {
	p, err := NewBulkProcessor(dict, level)
	if err != nil {
		t.Fatal("failed to create a BulkProcessor")
	}
	return p
}

func getRandomText() string {
	words := []string{"We", "are", "building", "a platform", "that", "engineers", "love", "to", "use", "Join", "us", "and", "help", "usher", "in", "the", "future"}
	wordCount := 10 + rand.Intn(100) // 10 - 109
	result := []string{}
	for i := 0; i < wordCount; i++ {
		result = append(result, words[rand.Intn(len(words))])
	}

	return strings.Join(result, " ")
}

func TestBulkDictionary(t *testing.T) {
	if len(dict) < 1 {
		t.Error("dictionary is empty")
	}
}

func TestBulkCompressAndDecompress(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	for i := 0; i < 100; i++ {
		payload := []byte(getRandomText())

		compressed, err := p.Compress(nil, payload)
		if err != nil {
			t.Error("failed to compress")
		}

		uncompressed, err := p.Decompress(nil, compressed)
		if err != nil {
			t.Error("failed to decompress")
		}

		if bytes.Compare(payload, uncompressed) != 0 {
			t.Error("uncompressed payload didn't match")
		}
	}
}

func TestBulkEmptyOrNilDictionary(t *testing.T) {
	p, err := NewBulkProcessor(nil, BestSpeed)
	if p != nil {
		t.Error("nil is expected")
	}
	if err != ErrEmptyDictionary {
		t.Error("ErrEmptyDictionary is expected")
	}

	p, err = NewBulkProcessor([]byte{}, BestSpeed)
	if p != nil {
		t.Error("nil is expected")
	}
	if err != ErrEmptyDictionary {
		t.Error("ErrEmptyDictionary is expected")
	}
}

func TestBulkCompressDecompressEmptyOrNilContent(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	compressed, err := p.Compress(nil, nil)
	if err != nil {
		t.Error("failed to compress")
	}
	if len(compressed) < 4 {
		t.Error("magic number doesn't exist")
	}

	compressed, err = p.Compress(nil, []byte{})
	if err != nil {
		t.Error("failed to compress")
	}
	if len(compressed) < 4 {
		t.Error("magic number doesn't exist")
	}

	decompressed, err := p.Decompress(nil, compressed)
	if err != nil {
		t.Error("failed to decompress")
	}
	if len(decompressed) != 0 {
		t.Error("content was not decompressed correctly")
	}
}

func TestBulkCompressIntoGivenDestination(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	dst := make([]byte, 100000)
	compressed, err := p.Compress(dst, []byte(getRandomText()))
	if err != nil {
		t.Error("failed to compress")
	}
	if len(compressed) < 4 {
		t.Error("magic number doesn't exist")
	}
	if &dst[0] != &compressed[0] {
		t.Error("'dst' and 'compressed' are not the same object")
	}
}

func TestBulkCompressNotEnoughDestination(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	dst := make([]byte, 1)
	compressed, err := p.Compress(dst, []byte(getRandomText()))
	if err != nil {
		t.Error("failed to compress")
	}
	if len(compressed) < 4 {
		t.Error("magic number doesn't exist")
	}
	if &dst[0] == &compressed[0] {
		t.Error("'dst' and 'compressed' are the same object")
	}
}

func TestBulkDecompressIntoGivenDestination(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	dst := make([]byte, 100000)
	decompressed, err := p.Decompress(dst, compressedPayload)
	if err != nil {
		t.Error("failed to decompress")
	}
	if &dst[0] != &decompressed[0] {
		t.Error("'dst' and 'decompressed' are not the same object")
	}
}

func TestBulkDecompressNotEnoughDestination(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	dst := make([]byte, 1)
	decompressed, err := p.Decompress(dst, compressedPayload)
	if err != nil {
		t.Error("failed to decompress")
	}
	if &dst[0] == &decompressed[0] {
		t.Error("'dst' and 'decompressed' are the same object")
	}
}

func TestBulkDecompressEmptyOrNilContent(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	decompressed, err := p.Decompress(nil, nil)
	if err != ErrEmptySlice {
		t.Error("ErrEmptySlice is expected")
	}
	if decompressed != nil {
		t.Error("nil is expected")
	}

	decompressed, err = p.Decompress(nil, []byte{})
	if err != ErrEmptySlice {
		t.Error("ErrEmptySlice is expected")
	}
	if decompressed != nil {
		t.Error("nil is expected")
	}
}

func TestBulkCompressAndDecompressInReverseOrder(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)
	payloads := [][]byte{}
	compressedPayloads := [][]byte{}
	for i := 0; i < 100; i++ {
		payloads = append(payloads, []byte(getRandomText()))

		compressed, err := p.Compress(nil, payloads[i])
		if err != nil {
			t.Error("failed to compress")
		}
		compressedPayloads = append(compressedPayloads, compressed)
	}

	for i := 99; i >= 0; i-- {
		uncompressed, err := p.Decompress(nil, compressedPayloads[i])
		if err != nil {
			t.Error("failed to decompress")
		}

		if bytes.Compare(payloads[i], uncompressed) != 0 {
			t.Error("uncompressed payload didn't match")
		}
	}
}

func TestBulkDecompressHighlyCompressable(t *testing.T) {
	p := newBulkProcessor(t, dict, BestSpeed)

	// Generate a big payload
	msgSize := 10 * 1000 * 1000 // 10 MiB
	msg := make([]byte, msgSize)
	compressed, err := Compress(nil, msg)
	if err != nil {
		t.Error("failed to compress")
	}

	// Regular decompression would trigger zipbomb prevention
	_, err = p.Decompress(nil, compressed)
	if !IsDstSizeTooSmallError(err) {
		t.Error("expected too small error")
	}

	// Passing an output should suceed the decompression
	dst := make([]byte, 10*msgSize)
	_, err = p.Decompress(dst, compressed)
	if err != nil {
		t.Errorf("failed to decompress: %s", err)
	}
}

// BenchmarkBulkCompress-8   	  780148	      1505 ns/op	  61.14 MB/s	     208 B/op	       5 allocs/op
func BenchmarkBulkCompress(b *testing.B) {
	p := newBulkProcessor(b, dict, BestSpeed)

	payload := []byte("We're building a platform that engineers love to use. Join us, and help usher in the future.")
	b.SetBytes(int64(len(payload)))
	for n := 0; n < b.N; n++ {
		_, err := p.Compress(nil, payload)
		if err != nil {
			b.Error("failed to compress")
		}
	}
}

// BenchmarkBulkDecompress-8   	  817425	      1412 ns/op	  40.37 MB/s	     192 B/op	       7 allocs/op
func BenchmarkBulkDecompress(b *testing.B) {
	p := newBulkProcessor(b, dict, BestSpeed)

	b.SetBytes(int64(len(compressedPayload)))
	for n := 0; n < b.N; n++ {
		_, err := p.Decompress(nil, compressedPayload)
		if err != nil {
			b.Error("failed to decompress")
		}
	}
}
