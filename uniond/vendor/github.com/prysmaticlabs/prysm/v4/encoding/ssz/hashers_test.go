package ssz_test

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestHash(t *testing.T) {
	byteSlice := []byte{1, 2, 3, 4, 5, 6, 7, 8, 9}
	hasher := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	expected := [32]byte{71, 228, 238, 127, 33, 31, 115, 38, 93, 209, 118, 88, 246, 226, 28, 19, 24, 189, 108, 129, 243, 117, 152, 226, 10, 39, 86, 41, 149, 66, 239, 207}
	result := hasher.Hash(byteSlice)
	assert.Equal(t, expected, result)
}

func TestCombi(t *testing.T) {
	byteSlice1 := [32]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32}
	byteSlice2 := [32]byte{32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1}
	hasher := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	expected := [32]byte{203, 73, 0, 148, 142, 9, 145, 147, 186, 232, 143, 117, 95, 44, 38, 46, 102, 69, 101, 74, 50, 37, 87, 189, 40, 196, 203, 140, 19, 233, 161, 225}
	result := hasher.Combi(byteSlice1, byteSlice2)
	assert.Equal(t, expected, result)
}

func TestMixIn(t *testing.T) {
	byteSlice := [32]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32}
	intToAdd := uint64(33)
	hasher := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	expected := [32]byte{170, 90, 0, 249, 34, 60, 140, 68, 77, 51, 218, 139, 54, 119, 179, 238, 80, 72, 13, 20, 212, 218, 124, 215, 68, 122, 214, 157, 178, 85, 225, 213}
	result := hasher.MixIn(byteSlice, intToAdd)
	assert.Equal(t, expected, result)
}
