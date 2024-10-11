package stateutil

import (
	"encoding/binary"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// BlockHeaderRoot computes the HashTreeRoot Merkleization of
// a BeaconBlockHeader struct according to the Ethereum
// Simple Serialize specification.
func BlockHeaderRoot(header *ethpb.BeaconBlockHeader) ([32]byte, error) {
	fieldRoots := make([][32]byte, 5)
	if header != nil {
		headerSlotBuf := make([]byte, 8)
		binary.LittleEndian.PutUint64(headerSlotBuf, uint64(header.Slot))
		headerSlotRoot := bytesutil.ToBytes32(headerSlotBuf)
		fieldRoots[0] = headerSlotRoot
		proposerIdxBuf := make([]byte, 8)
		binary.LittleEndian.PutUint64(proposerIdxBuf, uint64(header.ProposerIndex))
		proposerIndexRoot := bytesutil.ToBytes32(proposerIdxBuf)
		fieldRoots[1] = proposerIndexRoot
		parentRoot := bytesutil.ToBytes32(header.ParentRoot)
		fieldRoots[2] = parentRoot
		stateRoot := bytesutil.ToBytes32(header.StateRoot)
		fieldRoots[3] = stateRoot
		bodyRoot := bytesutil.ToBytes32(header.BodyRoot)
		fieldRoots[4] = bodyRoot
	}
	return ssz.BitwiseMerkleize(fieldRoots, uint64(len(fieldRoots)), uint64(len(fieldRoots)))
}
