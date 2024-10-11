//go:build go1.18
// +build go1.18

package gohashtree_test

import (
	"testing"

	"github.com/prysmaticlabs/gohashtree"
)

func convertRawChunks(raw []byte) [][32]byte {
	var chunks [][32]byte
	for i := 32; i <= len(raw); i += 32 {
		var c [32]byte
		copy(c[:], raw[i-32:i])
		chunks = append(chunks, c)
	}
	return chunks
}

func FuzzHash(f *testing.F) {
	for i := 1; i <= 10; i++ {
		f.Add(make([]byte, 64*i))
	}
	f.Fuzz(func(t *testing.T, chunksRaw []byte) {
		if len(chunksRaw) < 64 || len(chunksRaw)%64 != 0 {
			return // No chunks and odd number of chunks are invalid
		}
		chunks := convertRawChunks(chunksRaw)
		digests := make([][32]byte, len(chunks)/2)
		if err := gohashtree.Hash(digests, chunks); err != nil {
			t.Fatal(err)
		}
	})
}

func FuzzHash_Differential_Minio(f *testing.F) {
	for i := uint(0); i < 128; i++ {
		d := make([]byte, 64)
		for j := 0; j < 64; j++ {
			d[j] = byte(i)
		}
		f.Add(d)
	}
	f.Fuzz(func(t *testing.T, chunksRaw []byte) {
		if len(chunksRaw) < 64 || len(chunksRaw)%64 != 0 {
			return // No chunks and odd number of chunks are invalid
		}
		chunks := convertRawChunks(chunksRaw)
		digests := make([][32]byte, len(chunks)/2)
		if err := gohashtree.Hash(digests, chunks); err != nil {
			t.Fatal(err)
		}
		for i := 64; i <= len(chunksRaw); i += 64 {
			a := OldHash(chunksRaw[i-64 : i])
			b := digests[(i/64)-1]
			if a != b {
				t.Error("minio.Hash() != gohashtree.Hash()")
			}
		}
	})
}
