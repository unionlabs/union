package validator

import (
	"encoding/json"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
)

type AggregateAttestationResponse struct {
	Data *shared.Attestation `json:"data"`
}

type SubmitContributionAndProofsRequest struct {
	Data []*shared.SignedContributionAndProof `json:"data"`
}

type SubmitAggregateAndProofsRequest struct {
	Data []*shared.SignedAggregateAttestationAndProof `json:"data"`
}

type SubmitSyncCommitteeSubscriptionsRequest struct {
	Data []*shared.SyncCommitteeSubscription `json:"data"`
}

type SubmitBeaconCommitteeSubscriptionsRequest struct {
	Data []*shared.BeaconCommitteeSubscription `json:"data"`
}

type GetAttestationDataResponse struct {
	Data *shared.AttestationData `json:"data"`
}

type ProduceSyncCommitteeContributionResponse struct {
	Data *shared.SyncCommitteeContribution `json:"data"`
}

type GetAttesterDutiesResponse struct {
	DependentRoot       string          `json:"dependent_root"`
	ExecutionOptimistic bool            `json:"execution_optimistic"`
	Data                []*AttesterDuty `json:"data"`
}

type AttesterDuty struct {
	Pubkey                  string `json:"pubkey"`
	ValidatorIndex          string `json:"validator_index"`
	CommitteeIndex          string `json:"committee_index"`
	CommitteeLength         string `json:"committee_length"`
	CommitteesAtSlot        string `json:"committees_at_slot"`
	ValidatorCommitteeIndex string `json:"validator_committee_index"`
	Slot                    string `json:"slot"`
}

type GetProposerDutiesResponse struct {
	DependentRoot       string          `json:"dependent_root"`
	ExecutionOptimistic bool            `json:"execution_optimistic"`
	Data                []*ProposerDuty `json:"data"`
}

type ProposerDuty struct {
	Pubkey         string `json:"pubkey"`
	ValidatorIndex string `json:"validator_index"`
	Slot           string `json:"slot"`
}

type GetSyncCommitteeDutiesResponse struct {
	ExecutionOptimistic bool                 `json:"execution_optimistic"`
	Data                []*SyncCommitteeDuty `json:"data"`
}

type SyncCommitteeDuty struct {
	Pubkey                        string   `json:"pubkey"`
	ValidatorIndex                string   `json:"validator_index"`
	ValidatorSyncCommitteeIndices []string `json:"validator_sync_committee_indices"`
}

// ProduceBlockV3Response is a wrapper json object for the returned block from the ProduceBlockV3 endpoint
type ProduceBlockV3Response struct {
	Version                 string          `json:"version"`
	ExecutionPayloadBlinded bool            `json:"execution_payload_blinded"`
	ExecutionPayloadValue   string          `json:"execution_payload_value"`
	ConsensusBlockValue     string          `json:"consensus_block_value"`
	Data                    json.RawMessage `json:"data"` // represents the block values based on the version
}

type GetLivenessResponse struct {
	Data []*Liveness `json:"data"`
}

type Liveness struct {
	Index  string `json:"index"`
	IsLive bool   `json:"is_live"`
}
