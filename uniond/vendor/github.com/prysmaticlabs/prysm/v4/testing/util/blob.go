package util

import (
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// HydrateBlobSidecar hydrates a blob sidecar with correct field length sizes
// to comply with SSZ marshalling and unmarshalling rules.
func HydrateBlobSidecar(b *ethpb.BlobSidecar) *ethpb.BlobSidecar {
	if b.SignedBlockHeader == nil {
		b.SignedBlockHeader = HydrateSignedBeaconHeader(&ethpb.SignedBeaconBlockHeader{
			Header: &ethpb.BeaconBlockHeader{},
		})
	}
	if b.Blob == nil {
		b.Blob = make([]byte, fieldparams.BlobLength)
	}
	if b.KzgCommitment == nil {
		b.KzgCommitment = make([]byte, fieldparams.BLSPubkeyLength)
	}
	if b.KzgProof == nil {
		b.KzgProof = make([]byte, fieldparams.BLSPubkeyLength)
	}

	if b.CommitmentInclusionProof == nil {
		b.CommitmentInclusionProof = HydrateCommitmentInclusionProofs()
	}
	return b
}

// HydrateCommitmentInclusionProofs returns 2d byte slice of Commitment Inclusion Proofs
func HydrateCommitmentInclusionProofs() [][]byte {
	r := make([][]byte, fieldparams.KzgCommitmentInclusionProofDepth)
	for i := range r {
		r[i] = make([]byte, fieldparams.RootLength)
	}
	return r
}
