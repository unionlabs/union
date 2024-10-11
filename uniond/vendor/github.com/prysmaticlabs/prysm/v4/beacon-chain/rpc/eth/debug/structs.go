package debug

import (
	"encoding/json"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
)

type GetBeaconStateV2Response struct {
	Version             string          `json:"version"`
	ExecutionOptimistic bool            `json:"execution_optimistic"`
	Finalized           bool            `json:"finalized"`
	Data                json.RawMessage `json:"data"` // represents the state values based on the version
}

type GetForkChoiceHeadsV2Response struct {
	Data []*ForkChoiceHead `json:"data"`
}

type ForkChoiceHead struct {
	Root                string `json:"root"`
	Slot                string `json:"slot"`
	ExecutionOptimistic bool   `json:"execution_optimistic"`
}

type GetForkChoiceDumpResponse struct {
	JustifiedCheckpoint *shared.Checkpoint       `json:"justified_checkpoint"`
	FinalizedCheckpoint *shared.Checkpoint       `json:"finalized_checkpoint"`
	ForkChoiceNodes     []*ForkChoiceNode        `json:"fork_choice_nodes"`
	ExtraData           *ForkChoiceDumpExtraData `json:"extra_data"`
}

type ForkChoiceDumpExtraData struct {
	UnrealizedJustifiedCheckpoint *shared.Checkpoint `json:"unrealized_justified_checkpoint"`
	UnrealizedFinalizedCheckpoint *shared.Checkpoint `json:"unrealized_finalized_checkpoint"`
	ProposerBoostRoot             string             `json:"proposer_boost_root"`
	PreviousProposerBoostRoot     string             `json:"previous_proposer_boost_root"`
	HeadRoot                      string             `json:"head_root"`
}

type ForkChoiceNode struct {
	Slot               string                   `json:"slot"`
	BlockRoot          string                   `json:"block_root"`
	ParentRoot         string                   `json:"parent_root"`
	JustifiedEpoch     string                   `json:"justified_epoch"`
	FinalizedEpoch     string                   `json:"finalized_epoch"`
	Weight             string                   `json:"weight"`
	Validity           string                   `json:"validity"`
	ExecutionBlockHash string                   `json:"execution_block_hash"`
	ExtraData          *ForkChoiceNodeExtraData `json:"extra_data"`
}

type ForkChoiceNodeExtraData struct {
	UnrealizedJustifiedEpoch string `json:"unrealized_justified_epoch"`
	UnrealizedFinalizedEpoch string `json:"unrealized_finalized_epoch"`
	Balance                  string `json:"balance"`
	ExecutionOptimistic      bool   `json:"execution_optimistic"`
	TimeStamp                string `json:"timestamp"`
}
