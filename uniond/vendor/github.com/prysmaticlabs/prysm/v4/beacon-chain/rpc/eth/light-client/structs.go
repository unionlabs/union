package lightclient

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
)

type LightClientBootstrapResponse struct {
	Version string                `json:"version"`
	Data    *LightClientBootstrap `json:"data"`
}

type LightClientBootstrap struct {
	Header                     *shared.BeaconBlockHeader `json:"header"`
	CurrentSyncCommittee       *shared.SyncCommittee     `json:"current_sync_committee"`
	CurrentSyncCommitteeBranch []string                  `json:"current_sync_committee_branch"`
}

type LightClientUpdate struct {
	AttestedHeader          *shared.BeaconBlockHeader `json:"attested_header"`
	NextSyncCommittee       *shared.SyncCommittee     `json:"next_sync_committee,omitempty"`
	FinalizedHeader         *shared.BeaconBlockHeader `json:"finalized_header,omitempty"`
	SyncAggregate           *shared.SyncAggregate     `json:"sync_aggregate"`
	NextSyncCommitteeBranch []string                  `json:"next_sync_committee_branch,omitempty"`
	FinalityBranch          []string                  `json:"finality_branch,omitempty"`
	SignatureSlot           string                    `json:"signature_slot"`
}

type LightClientUpdateWithVersion struct {
	Version string             `json:"version"`
	Data    *LightClientUpdate `json:"data"`
}

type LightClientUpdatesByRangeResponse struct {
	Updates []*LightClientUpdateWithVersion `json:"updates"`
}
