package beacon

import (
	"encoding/json"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
)

type BlockRootResponse struct {
	Data                *BlockRoot `json:"data"`
	ExecutionOptimistic bool       `json:"execution_optimistic"`
	Finalized           bool       `json:"finalized"`
}

type BlockRoot struct {
	Root string `json:"root"`
}

type GetCommitteesResponse struct {
	Data                []*shared.Committee `json:"data"`
	ExecutionOptimistic bool                `json:"execution_optimistic"`
	Finalized           bool                `json:"finalized"`
}

type ListAttestationsResponse struct {
	Data []*shared.Attestation `json:"data"`
}

type SubmitAttestationsRequest struct {
	Data []*shared.Attestation `json:"data"`
}

type ListVoluntaryExitsResponse struct {
	Data []*shared.SignedVoluntaryExit `json:"data"`
}

type SubmitSyncCommitteeSignaturesRequest struct {
	Data []*shared.SyncCommitteeMessage `json:"data"`
}

type GetStateForkResponse struct {
	Data                *shared.Fork `json:"data"`
	ExecutionOptimistic bool         `json:"execution_optimistic"`
	Finalized           bool         `json:"finalized"`
}

type GetFinalityCheckpointsResponse struct {
	ExecutionOptimistic bool                 `json:"execution_optimistic"`
	Finalized           bool                 `json:"finalized"`
	Data                *FinalityCheckpoints `json:"data"`
}

type FinalityCheckpoints struct {
	PreviousJustified *shared.Checkpoint `json:"previous_justified"`
	CurrentJustified  *shared.Checkpoint `json:"current_justified"`
	Finalized         *shared.Checkpoint `json:"finalized"`
}

type GetGenesisResponse struct {
	Data *Genesis `json:"data"`
}

type Genesis struct {
	GenesisTime           string `json:"genesis_time"`
	GenesisValidatorsRoot string `json:"genesis_validators_root"`
	GenesisForkVersion    string `json:"genesis_fork_version"`
}

type GetBlockHeadersResponse struct {
	Data                []*shared.SignedBeaconBlockHeaderContainer `json:"data"`
	ExecutionOptimistic bool                                       `json:"execution_optimistic"`
	Finalized           bool                                       `json:"finalized"`
}

type GetBlockHeaderResponse struct {
	ExecutionOptimistic bool                                     `json:"execution_optimistic"`
	Finalized           bool                                     `json:"finalized"`
	Data                *shared.SignedBeaconBlockHeaderContainer `json:"data"`
}

type GetValidatorsRequest struct {
	Ids      []string `json:"ids"`
	Statuses []string `json:"statuses"`
}

type GetValidatorsResponse struct {
	ExecutionOptimistic bool                  `json:"execution_optimistic"`
	Finalized           bool                  `json:"finalized"`
	Data                []*ValidatorContainer `json:"data"`
}

type GetValidatorResponse struct {
	ExecutionOptimistic bool                `json:"execution_optimistic"`
	Finalized           bool                `json:"finalized"`
	Data                *ValidatorContainer `json:"data"`
}

type GetValidatorBalancesResponse struct {
	ExecutionOptimistic bool                `json:"execution_optimistic"`
	Finalized           bool                `json:"finalized"`
	Data                []*ValidatorBalance `json:"data"`
}

type ValidatorContainer struct {
	Index     string     `json:"index"`
	Balance   string     `json:"balance"`
	Status    string     `json:"status"`
	Validator *Validator `json:"validator"`
}

type Validator struct {
	Pubkey                     string `json:"pubkey"`
	WithdrawalCredentials      string `json:"withdrawal_credentials"`
	EffectiveBalance           string `json:"effective_balance"`
	Slashed                    bool   `json:"slashed"`
	ActivationEligibilityEpoch string `json:"activation_eligibility_epoch"`
	ActivationEpoch            string `json:"activation_epoch"`
	ExitEpoch                  string `json:"exit_epoch"`
	WithdrawableEpoch          string `json:"withdrawable_epoch"`
}

type ValidatorBalance struct {
	Index   string `json:"index"`
	Balance string `json:"balance"`
}

type GetBlockResponse struct {
	Data *SignedBlock `json:"data"`
}

type GetBlockV2Response struct {
	Version             string       `json:"version"`
	ExecutionOptimistic bool         `json:"execution_optimistic"`
	Finalized           bool         `json:"finalized"`
	Data                *SignedBlock `json:"data"`
}

type SignedBlock struct {
	Message   json.RawMessage `json:"message"` // represents the block values based on the version
	Signature string          `json:"signature"`
}

type GetBlockAttestationsResponse struct {
	ExecutionOptimistic bool                  `json:"execution_optimistic"`
	Finalized           bool                  `json:"finalized"`
	Data                []*shared.Attestation `json:"data"`
}

type GetStateRootResponse struct {
	ExecutionOptimistic bool       `json:"execution_optimistic"`
	Finalized           bool       `json:"finalized"`
	Data                *StateRoot `json:"data"`
}

type StateRoot struct {
	Root string `json:"root"`
}

type GetRandaoResponse struct {
	ExecutionOptimistic bool    `json:"execution_optimistic"`
	Finalized           bool    `json:"finalized"`
	Data                *Randao `json:"data"`
}

type Randao struct {
	Randao string `json:"randao"`
}

type GetSyncCommitteeResponse struct {
	ExecutionOptimistic bool                     `json:"execution_optimistic"`
	Finalized           bool                     `json:"finalized"`
	Data                *SyncCommitteeValidators `json:"data"`
}

type SyncCommitteeValidators struct {
	Validators          []string   `json:"validators"`
	ValidatorAggregates [][]string `json:"validator_aggregates"`
}

type BLSToExecutionChangesPoolResponse struct {
	Data []*shared.SignedBLSToExecutionChange `json:"data"`
}

type GetAttesterSlashingsResponse struct {
	Data []*shared.AttesterSlashing `json:"data"`
}

type GetProposerSlashingsResponse struct {
	Data []*shared.ProposerSlashing `json:"data"`
}
