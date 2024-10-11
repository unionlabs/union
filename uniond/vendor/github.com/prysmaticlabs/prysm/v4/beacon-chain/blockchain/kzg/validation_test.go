package kzg

import (
	"bytes"
	"crypto/sha256"
	"encoding/binary"
	"testing"

	"github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	GoKZG "github.com/crate-crypto/go-kzg-4844"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
)

func deterministicRandomness(seed int64) [32]byte {
	// Converts an int64 to a byte slice
	buf := new(bytes.Buffer)
	err := binary.Write(buf, binary.BigEndian, seed)
	if err != nil {
		logrus.WithError(err).Error("Failed to write int64 to bytes buffer")
		return [32]byte{}
	}
	bytes := buf.Bytes()

	return sha256.Sum256(bytes)
}

// Returns a serialized random field element in big-endian
func GetRandFieldElement(seed int64) [32]byte {
	bytes := deterministicRandomness(seed)
	var r fr.Element
	r.SetBytes(bytes[:])

	return GoKZG.SerializeScalar(r)
}

// Returns a random blob using the passed seed as entropy
func GetRandBlob(seed int64) GoKZG.Blob {
	var blob GoKZG.Blob
	bytesPerBlob := GoKZG.ScalarsPerBlob * GoKZG.SerializedScalarSize
	for i := 0; i < bytesPerBlob; i += GoKZG.SerializedScalarSize {
		fieldElementBytes := GetRandFieldElement(seed + int64(i))
		copy(blob[i:i+GoKZG.SerializedScalarSize], fieldElementBytes[:])
	}
	return blob
}

func GenerateCommitmentAndProof(blob GoKZG.Blob) (GoKZG.KZGCommitment, GoKZG.KZGProof, error) {
	commitment, err := kzgContext.BlobToKZGCommitment(blob, 0)
	if err != nil {
		return GoKZG.KZGCommitment{}, GoKZG.KZGProof{}, err
	}
	proof, err := kzgContext.ComputeBlobKZGProof(blob, commitment, 0)
	if err != nil {
		return GoKZG.KZGCommitment{}, GoKZG.KZGProof{}, err
	}
	return commitment, proof, err
}

func TestVerify(t *testing.T) {
	sidecars := make([]blocks.ROBlob, 0)
	require.NoError(t, Verify(sidecars...))
}

func TestBytesToAny(t *testing.T) {
	bytes := []byte{0x01, 0x02}
	blob := GoKZG.Blob{0x01, 0x02}
	commitment := GoKZG.KZGCommitment{0x01, 0x02}
	proof := GoKZG.KZGProof{0x01, 0x02}
	require.DeepEqual(t, blob, bytesToBlob(bytes))
	require.DeepEqual(t, commitment, bytesToCommitment(bytes))
	require.DeepEqual(t, proof, bytesToKZGProof(bytes))
}

func TestGenerateCommitmentAndProof(t *testing.T) {
	blob := GetRandBlob(123)
	commitment, proof, err := GenerateCommitmentAndProof(blob)
	require.NoError(t, err)
	expectedCommitment := GoKZG.KZGCommitment{180, 218, 156, 194, 59, 20, 10, 189, 186, 254, 132, 93, 7, 127, 104, 172, 238, 240, 237, 70, 83, 89, 1, 152, 99, 0, 165, 65, 143, 62, 20, 215, 230, 14, 205, 95, 28, 245, 54, 25, 160, 16, 178, 31, 232, 207, 38, 85}
	expectedProof := GoKZG.KZGProof{128, 110, 116, 170, 56, 111, 126, 87, 229, 234, 211, 42, 110, 150, 129, 206, 73, 142, 167, 243, 90, 149, 240, 240, 236, 204, 143, 182, 229, 249, 81, 27, 153, 171, 83, 70, 144, 250, 42, 1, 188, 215, 71, 235, 30, 7, 175, 86}
	require.Equal(t, expectedCommitment, commitment)
	require.Equal(t, expectedProof, proof)
}
