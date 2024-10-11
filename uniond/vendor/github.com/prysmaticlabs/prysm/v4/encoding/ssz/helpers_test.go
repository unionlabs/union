package ssz_test

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

const merkleizingListLimitError = "merkleizing list that is too large, over limit"

func TestBitlistRoot(t *testing.T) {
	capacity := uint64(10)
	bfield := bitfield.NewBitlist(capacity)
	expected := [32]byte{176, 76, 194, 203, 142, 166, 117, 79, 148, 194, 231, 64, 60, 245, 142, 32, 201, 2, 58, 152, 53, 12, 132, 40, 41, 102, 224, 189, 103, 41, 211, 202}

	result, err := ssz.BitlistRoot(bfield, capacity)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestBitwiseMerkleizeOverLimit(t *testing.T) {
	chunks := [][32]byte{
		{1, 2, 3, 4, 5, 6, 7, 8, 9, 10},
		{11, 12, 13, 14, 15, 16, 17, 18, 19, 20},
	}
	count := uint64(2)
	limit := uint64(1)

	_, err := ssz.BitwiseMerkleize(chunks, count, limit)
	assert.ErrorContains(t, merkleizingListLimitError, err)
}

func TestBitwiseMerkleizeArrays(t *testing.T) {
	chunks := [][32]byte{
		{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32},
		{33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 62, 62, 63, 64},
	}
	count := uint64(2)
	limit := uint64(2)
	expected := [32]byte{138, 81, 210, 194, 151, 231, 249, 241, 64, 118, 209, 58, 145, 109, 225, 89, 118, 110, 159, 220, 193, 183, 203, 124, 166, 24, 65, 26, 160, 215, 233, 219}

	result, err := ssz.BitwiseMerkleize(chunks, count, limit)
	require.NoError(t, err)
	assert.Equal(t, expected, result)

	chunks = [][32]byte{
		{1, 2, 3, 4, 5, 6, 7, 8, 9, 10},
		{11, 12, 13, 14, 15, 16, 17, 18, 19, 20},
	}
	count = uint64(2)
	limit = uint64(2)
	expected = [32]byte{194, 32, 213, 52, 220, 127, 18, 240, 43, 151, 19, 79, 188, 175, 142, 177, 208, 46, 96, 20, 18, 231, 208, 29, 120, 102, 122, 17, 46, 31, 155, 30}

	result, err = ssz.BitwiseMerkleize(chunks, count, limit)
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

func TestBitwiseMerkleizeArraysOverLimit(t *testing.T) {
	chunks := [][32]byte{
		{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32},
		{33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 62, 62, 63, 64},
	}
	count := uint64(2)
	limit := uint64(1)

	_, err := ssz.BitwiseMerkleize(chunks, count, limit)
	assert.ErrorContains(t, merkleizingListLimitError, err)
}

func TestPackByChunk(t *testing.T) {
	byteSlice2D := [][]byte{
		{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 2, 5, 2, 6, 2, 7},
		{1, 1, 2, 3, 5, 8, 13, 21, 34},
	}
	expected := [][32]byte{{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 2, 5, 2, 6, 2, 7, 1, 1},
		{2, 3, 5, 8, 13, 21, 34, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}}

	result, err := ssz.PackByChunk(byteSlice2D)
	require.NoError(t, err)
	assert.Equal(t, len(expected), len(result))
	for i, v := range expected {
		assert.DeepEqual(t, v, result[i])
	}
}

func TestMixInLength(t *testing.T) {
	byteSlice := [32]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32}
	length := []byte{1, 2, 3}
	expected := [32]byte{105, 60, 167, 169, 197, 220, 122, 99, 59, 14, 250, 12, 251, 62, 135, 239, 29, 68, 140, 1, 6, 36, 207, 44, 64, 221, 76, 230, 237, 218, 150, 88}
	result := ssz.MixInLength(byteSlice, length)
	assert.Equal(t, expected, result)
}
