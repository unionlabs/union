//go:build !noasm && !appengine && gc
// +build !noasm,!appengine,gc

package sha256

import (
	"crypto/sha256"
	"encoding/binary"
	"testing"
)

func sha256hash(m []byte) (r [32]byte) {
	var h [8]uint32

	h[0] = 0x6a09e667
	h[1] = 0xbb67ae85
	h[2] = 0x3c6ef372
	h[3] = 0xa54ff53a
	h[4] = 0x510e527f
	h[5] = 0x9b05688c
	h[6] = 0x1f83d9ab
	h[7] = 0x5be0cd19

	blockIntelSha(&h, m)
	l0 := len(m)
	l := l0 & (BlockSize - 1)
	m = m[l0-l:]

	var k [64]byte
	copy(k[:], m)

	k[l] = 0x80

	if l >= 56 {
		blockIntelSha(&h, k[:])
		binary.LittleEndian.PutUint64(k[0:8], 0)
		binary.LittleEndian.PutUint64(k[8:16], 0)
		binary.LittleEndian.PutUint64(k[16:24], 0)
		binary.LittleEndian.PutUint64(k[24:32], 0)
		binary.LittleEndian.PutUint64(k[32:40], 0)
		binary.LittleEndian.PutUint64(k[40:48], 0)
		binary.LittleEndian.PutUint64(k[48:56], 0)
	}
	binary.BigEndian.PutUint64(k[56:64], uint64(l0)<<3)
	blockIntelSha(&h, k[:])

	binary.BigEndian.PutUint32(r[0:4], h[0])
	binary.BigEndian.PutUint32(r[4:8], h[1])
	binary.BigEndian.PutUint32(r[8:12], h[2])
	binary.BigEndian.PutUint32(r[12:16], h[3])
	binary.BigEndian.PutUint32(r[16:20], h[4])
	binary.BigEndian.PutUint32(r[20:24], h[5])
	binary.BigEndian.PutUint32(r[24:28], h[6])
	binary.BigEndian.PutUint32(r[28:32], h[7])

	return
}

func runTestSha(hashfunc func([]byte) [32]byte) bool {
	var m = []byte("This is a message. This is a message. This is a message. This is a message.")

	ar := hashfunc(m)
	br := sha256.Sum256(m)

	return ar == br
}

func TestSha0(t *testing.T) {
	if !runTestSha(Sum256) {
		t.Errorf("FAILED")
	}
}

func TestSha1(t *testing.T) {
	if hasIntelSha && !runTestSha(sha256hash) {
		t.Errorf("FAILED")
	}
}
