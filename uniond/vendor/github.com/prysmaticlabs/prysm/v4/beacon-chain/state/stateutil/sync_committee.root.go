package stateutil

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash/htr"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// SyncCommitteeRoot computes the HashTreeRoot Merkleization of a committee root.
// a SyncCommitteeRoot struct according to the eth2
// Simple Serialize specification.
func SyncCommitteeRoot(committee *ethpb.SyncCommittee) ([32]byte, error) {
	var fieldRoots [][32]byte
	if committee == nil {
		return [32]byte{}, nil
	}

	// Field 1:  Vector[BLSPubkey, SYNC_COMMITTEE_SIZE]
	pubKeyRoots := make([][32]byte, 0)
	for _, pubkey := range committee.Pubkeys {
		r, err := merkleizePubkey(pubkey)
		if err != nil {
			return [32]byte{}, err
		}
		pubKeyRoots = append(pubKeyRoots, r)
	}
	pubkeyRoot, err := ssz.BitwiseMerkleize(pubKeyRoots, uint64(len(pubKeyRoots)), uint64(len(pubKeyRoots)))
	if err != nil {
		return [32]byte{}, err
	}

	// Field 2: BLSPubkey
	aggregateKeyRoot, err := merkleizePubkey(committee.AggregatePubkey)
	if err != nil {
		return [32]byte{}, err
	}
	fieldRoots = [][32]byte{pubkeyRoot, aggregateKeyRoot}

	return ssz.BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}

func merkleizePubkey(pubkey []byte) ([32]byte, error) {
	if len(pubkey) == 0 {
		return [32]byte{}, errors.New("zero length pubkey provided")
	}
	chunks, err := ssz.PackByChunk([][]byte{pubkey})
	if err != nil {
		return [32]byte{}, err
	}
	outputChunk := htr.VectorizedSha256(chunks)

	return outputChunk[0], nil
}
