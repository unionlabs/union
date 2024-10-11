package events

import (
	"encoding/json"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
)

type HeadEvent struct {
	Slot                      string `json:"slot"`
	Block                     string `json:"block"`
	State                     string `json:"state"`
	EpochTransition           bool   `json:"epoch_transition"`
	ExecutionOptimistic       bool   `json:"execution_optimistic"`
	PreviousDutyDependentRoot string `json:"previous_duty_dependent_root"`
	CurrentDutyDependentRoot  string `json:"current_duty_dependent_root"`
}

type BlockEvent struct {
	Slot                string `json:"slot"`
	Block               string `json:"block"`
	ExecutionOptimistic bool   `json:"execution_optimistic"`
}

type AggregatedAttEventSource struct {
	Aggregate *shared.Attestation `json:"aggregate"`
}

type UnaggregatedAttEventSource struct {
	AggregationBits string                  `json:"aggregation_bits"`
	Data            *shared.AttestationData `json:"data"`
	Signature       string                  `json:"signature"`
}

type FinalizedCheckpointEvent struct {
	Block               string `json:"block"`
	State               string `json:"state"`
	Epoch               string `json:"epoch"`
	ExecutionOptimistic bool   `json:"execution_optimistic"`
}

type ChainReorgEvent struct {
	Slot                string `json:"slot"`
	Depth               string `json:"depth"`
	OldHeadBlock        string `json:"old_head_block"`
	NewHeadBlock        string `json:"old_head_state"`
	OldHeadState        string `json:"new_head_block"`
	NewHeadState        string `json:"new_head_state"`
	Epoch               string `json:"epoch"`
	ExecutionOptimistic bool   `json:"execution_optimistic"`
}

type PayloadAttributesEvent struct {
	Version string          `json:"version"`
	Data    json.RawMessage `json:"data"`
}

type PayloadAttributesEventData struct {
	ProposerIndex     string          `json:"proposer_index"`
	ProposalSlot      string          `json:"proposal_slot"`
	ParentBlockNumber string          `json:"parent_block_number"`
	ParentBlockRoot   string          `json:"parent_block_root"`
	ParentBlockHash   string          `json:"parent_block_hash"`
	PayloadAttributes json.RawMessage `json:"payload_attributes"`
}

type PayloadAttributesV1 struct {
	Timestamp             string `json:"timestamp"`
	PrevRandao            string `json:"prev_randao"`
	SuggestedFeeRecipient string `json:"suggested_fee_recipient"`
}

type PayloadAttributesV2 struct {
	Timestamp             string               `json:"timestamp"`
	PrevRandao            string               `json:"prev_randao"`
	SuggestedFeeRecipient string               `json:"suggested_fee_recipient"`
	Withdrawals           []*shared.Withdrawal `json:"withdrawals"`
}

type PayloadAttributesV3 struct {
	Timestamp             string               `json:"timestamp"`
	PrevRandao            string               `json:"prev_randao"`
	SuggestedFeeRecipient string               `json:"suggested_fee_recipient"`
	Withdrawals           []*shared.Withdrawal `json:"withdrawals"`
	ParentBeaconBlockRoot string               `json:"parent_beacon_block_root"`
}

type BlobSidecarEvent struct {
	BlockRoot     string `json:"block_root"`
	Index         string `json:"index"`
	Slot          string `json:"slot"`
	KzgCommitment string `json:"kzg_commitment"`
	VersionedHash string `json:"versioned_hash"`
}

type LightClientFinalityUpdateEvent struct {
	Version string                     `json:"version"`
	Data    *LightClientFinalityUpdate `json:"data"`
}

type LightClientFinalityUpdate struct {
	AttestedHeader  *shared.BeaconBlockHeader `json:"attested_header"`
	FinalizedHeader *shared.BeaconBlockHeader `json:"finalized_header"`
	FinalityBranch  []string                  `json:"finality_branch"`
	SyncAggregate   *shared.SyncAggregate     `json:"sync_aggregate"`
	SignatureSlot   string                    `json:"signature_slot"`
}

type LightClientOptimisticUpdateEvent struct {
	Version string                       `json:"version"`
	Data    *LightClientOptimisticUpdate `json:"data"`
}

type LightClientOptimisticUpdate struct {
	AttestedHeader *shared.BeaconBlockHeader `json:"attested_header"`
	SyncAggregate  *shared.SyncAggregate     `json:"sync_aggregate"`
	SignatureSlot  string                    `json:"signature_slot"`
}
