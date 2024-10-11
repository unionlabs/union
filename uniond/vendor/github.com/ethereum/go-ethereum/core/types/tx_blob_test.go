package types

import (
	"crypto/ecdsa"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/crypto/kzg4844"
	"github.com/holiman/uint256"
)

// This test verifies that tx.Hash() is not affected by presence of a BlobTxSidecar.
func TestBlobTxHashing(t *testing.T) {
	key, _ := crypto.GenerateKey()
	withBlobs := createEmptyBlobTx(key, true)
	withBlobsStripped := withBlobs.WithoutBlobTxSidecar()
	withoutBlobs := createEmptyBlobTx(key, false)

	hash := withBlobs.Hash()
	t.Log("tx hash:", hash)

	if h := withBlobsStripped.Hash(); h != hash {
		t.Fatal("wrong tx hash after WithoutBlobTxSidecar:", h)
	}
	if h := withoutBlobs.Hash(); h != hash {
		t.Fatal("wrong tx hash on tx created without sidecar:", h)
	}
}

// This test verifies that tx.Size() takes BlobTxSidecar into account.
func TestBlobTxSize(t *testing.T) {
	key, _ := crypto.GenerateKey()
	withBlobs := createEmptyBlobTx(key, true)
	withBlobsStripped := withBlobs.WithoutBlobTxSidecar()
	withoutBlobs := createEmptyBlobTx(key, false)

	withBlobsEnc, _ := withBlobs.MarshalBinary()
	withoutBlobsEnc, _ := withoutBlobs.MarshalBinary()

	size := withBlobs.Size()
	t.Log("size with blobs:", size)

	sizeNoBlobs := withoutBlobs.Size()
	t.Log("size without blobs:", sizeNoBlobs)

	if size != uint64(len(withBlobsEnc)) {
		t.Error("wrong size with blobs:", size, "encoded length:", len(withBlobsEnc))
	}
	if sizeNoBlobs != uint64(len(withoutBlobsEnc)) {
		t.Error("wrong size without blobs:", sizeNoBlobs, "encoded length:", len(withoutBlobsEnc))
	}
	if sizeNoBlobs >= size {
		t.Error("size without blobs >= size with blobs")
	}
	if sz := withBlobsStripped.Size(); sz != sizeNoBlobs {
		t.Fatal("wrong size on tx after WithoutBlobTxSidecar:", sz)
	}
}

var (
	emptyBlob          = kzg4844.Blob{}
	emptyBlobCommit, _ = kzg4844.BlobToCommitment(emptyBlob)
	emptyBlobProof, _  = kzg4844.ComputeBlobProof(emptyBlob, emptyBlobCommit)
)

func createEmptyBlobTx(key *ecdsa.PrivateKey, withSidecar bool) *Transaction {
	sidecar := &BlobTxSidecar{
		Blobs:       []kzg4844.Blob{emptyBlob},
		Commitments: []kzg4844.Commitment{emptyBlobCommit},
		Proofs:      []kzg4844.Proof{emptyBlobProof},
	}
	blobtx := &BlobTx{
		ChainID:    uint256.NewInt(1),
		Nonce:      5,
		GasTipCap:  uint256.NewInt(22),
		GasFeeCap:  uint256.NewInt(5),
		Gas:        25000,
		To:         common.Address{0x03, 0x04, 0x05},
		Value:      uint256.NewInt(99),
		Data:       make([]byte, 50),
		BlobFeeCap: uint256.NewInt(15),
		BlobHashes: sidecar.BlobHashes(),
	}
	if withSidecar {
		blobtx.Sidecar = sidecar
	}
	signer := NewCancunSigner(blobtx.ChainID.ToBig())
	return MustSignNewTx(key, signer, blobtx)
}
