package validator

import (
	"bytes"

	"github.com/pkg/errors"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

func unblindBlobsSidecars(block interfaces.SignedBeaconBlock, bundle *enginev1.BlobsBundle) ([]*ethpb.BlobSidecar, error) {
	if block.Version() < version.Deneb || bundle == nil {
		return nil, nil
	}
	header, err := block.Header()
	if err != nil {
		return nil, err
	}
	body := block.Block().Body()
	blockCommitments, err := body.BlobKzgCommitments()
	if err != nil {
		return nil, err
	}

	// Ensure there are equal counts of blobs/commitments/proofs.
	if len(bundle.KzgCommitments) != len(bundle.Blobs) {
		return nil, errors.New("mismatch commitments count")
	}
	if len(bundle.Proofs) != len(bundle.Blobs) {
		return nil, errors.New("mismatch proofs count")
	}

	// Verify that commitments in the bundle match the block.
	if len(bundle.KzgCommitments) != len(blockCommitments) {
		return nil, errors.New("commitment count doesn't match block")
	}
	for i, commitment := range blockCommitments {
		if !bytes.Equal(bundle.KzgCommitments[i], commitment) {
			return nil, errors.New("commitment value doesn't match block")
		}
	}

	sidecars := make([]*ethpb.BlobSidecar, len(bundle.Blobs))
	for i, b := range bundle.Blobs {
		proof, err := consensusblocks.MerkleProofKZGCommitment(body, i)
		if err != nil {
			return nil, err
		}
		sidecars[i] = &ethpb.BlobSidecar{
			Index:                    uint64(i),
			Blob:                     bytesutil.SafeCopyBytes(b),
			KzgCommitment:            bytesutil.SafeCopyBytes(bundle.KzgCommitments[i]),
			KzgProof:                 bytesutil.SafeCopyBytes(bundle.Proofs[i]),
			SignedBlockHeader:        header,
			CommitmentInclusionProof: proof,
		}
	}
	return sidecars, nil
}
