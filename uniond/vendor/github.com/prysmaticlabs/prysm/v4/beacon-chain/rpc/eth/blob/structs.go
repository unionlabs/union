package blob

import "github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"

type SidecarsResponse struct {
	Data []*Sidecar `json:"data"`
}

type Sidecar struct {
	Index                    string                          `json:"index"`
	Blob                     string                          `json:"blob"`
	SignedBeaconBlockHeader  *shared.SignedBeaconBlockHeader `json:"signed_block_header"`
	KzgCommitment            string                          `json:"kzg_commitment"`
	KzgProof                 string                          `json:"kzg_proof"`
	CommitmentInclusionProof []string                        `json:"kzg_commitment_inclusion_proof"`
}
