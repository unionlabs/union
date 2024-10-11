package ssz_test

import (
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestGetDepth(t *testing.T) {
	trieSize := uint64(896745231)
	expected := uint8(30)

	result := ssz.Depth(trieSize)
	assert.Equal(t, expected, result)
}

func TestMerkleizeCountGreaterThanLimit(t *testing.T) {
	hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	count := uint64(2)
	limit := uint64(1)
	chunks := [][]byte{{}}
	leafIndexer := func(i uint64) []byte {
		return chunks[i]
	}
	// Error if no panic
	defer func() {
		if r := recover(); r == nil {
			t.Errorf("The code did not panic.")
		}
	}()
	ssz.Merkleize(hashFn, count, limit, leafIndexer)
}

func TestMerkleizeLimitAndCountAreZero(t *testing.T) {
	hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	count := uint64(0)
	limit := uint64(0)
	chunks := [][]byte{{}}
	leafIndexer := func(i uint64) []byte {
		return chunks[i]
	}
	var expected [32]byte
	result := ssz.Merkleize(hashFn, count, limit, leafIndexer)
	assert.Equal(t, expected, result)
}

func TestMerkleizeNormalPath(t *testing.T) {
	hashFn := ssz.NewHasherFunc(hash.CustomSHA256Hasher())
	count := uint64(2)
	limit := uint64(3)
	chunks := [][]byte{{1, 2, 3, 4}, {5, 6, 7, 8}, {9, 10, 11, 12}}
	leafIndexer := func(i uint64) []byte {
		return chunks[i]
	}
	expected := [32]byte{95, 27, 253, 237, 215, 58, 147, 198, 175, 194, 180, 231, 154, 130, 205, 68, 146, 112, 225, 86, 6, 103, 186, 82, 7, 142, 33, 189, 174, 56, 199, 173}
	result := ssz.Merkleize(hashFn, count, limit, leafIndexer)
	assert.Equal(t, expected, result)
}

func TestDepthOfOne(t *testing.T) {
	assert.Equal(t, uint8(0), ssz.Depth(1))
}

func Test_MerkleizeVectorSSZ(t *testing.T) {
	t.Run("empty vector", func(t *testing.T) {
		attList := make([]*ethpb.Attestation, 0)
		expected := [32]byte{83, 109, 152, 131, 127, 45, 209, 101, 165, 93, 94, 234, 233, 20, 133, 149, 68, 114, 213, 111, 36, 109, 242, 86, 191, 60, 174, 25, 53, 42, 18, 60}
		length := uint64(16)
		root, err := ssz.MerkleizeVectorSSZ(attList, length)
		require.NoError(t, err)
		require.Equal(t, expected, root)
	})
	t.Run("non empty vector", func(t *testing.T) {
		sig := make([]byte, 96)
		br := make([]byte, 32)
		attList := make([]*ethpb.Attestation, 1)
		attList[0] = &ethpb.Attestation{
			AggregationBits: bitfield.Bitlist{0x01},
			Data: &ethpb.AttestationData{
				BeaconBlockRoot: br,
				Source: &ethpb.Checkpoint{
					Root: br,
				},
				Target: &ethpb.Checkpoint{
					Root: br,
				},
			},
			Signature: sig,
		}
		expected := [32]byte{199, 186, 55, 142, 200, 75, 219, 191, 66, 153, 100, 181, 200, 15, 143, 160, 25, 133, 105, 26, 183, 107, 10, 198, 232, 231, 107, 162, 243, 243, 56, 20}
		length := uint64(16)
		root, err := ssz.MerkleizeVectorSSZ(attList, length)
		require.NoError(t, err)
		require.Equal(t, expected, root)
	})
}

func Test_MerkleizeListSSZ(t *testing.T) {
	t.Run("empty vector", func(t *testing.T) {
		attList := make([]*ethpb.Attestation, 0)
		expected := [32]byte{121, 41, 48, 187, 213, 186, 172, 67, 188, 199, 152, 238, 73, 170, 129, 133, 239, 118, 187, 59, 68, 186, 98, 185, 29, 134, 174, 86, 158, 75, 181, 53}
		length := uint64(16)
		root, err := ssz.MerkleizeListSSZ(attList, length)
		require.NoError(t, err)
		require.Equal(t, expected, root)
	})
	t.Run("non empty vector", func(t *testing.T) {
		sig := make([]byte, 96)
		br := make([]byte, 32)
		attList := make([]*ethpb.Attestation, 1)
		attList[0] = &ethpb.Attestation{
			AggregationBits: bitfield.Bitlist{0x01},
			Data: &ethpb.AttestationData{
				BeaconBlockRoot: br,
				Source: &ethpb.Checkpoint{
					Root: br,
				},
				Target: &ethpb.Checkpoint{
					Root: br,
				},
			},
			Signature: sig,
		}
		expected := [32]byte{161, 247, 30, 234, 219, 222, 154, 88, 7, 207, 6, 23, 46, 125, 135, 67, 225, 178, 217, 131, 113, 124, 242, 106, 194, 43, 205, 194, 49, 172, 232, 229}
		length := uint64(16)
		root, err := ssz.MerkleizeListSSZ(attList, length)
		require.NoError(t, err)
		require.Equal(t, expected, root)
	})
}
