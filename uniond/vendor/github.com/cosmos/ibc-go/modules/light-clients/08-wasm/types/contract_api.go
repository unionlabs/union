package types

import (
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	commitmenttypes "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
)

// InstantiateMessage is the message that is sent to the contract's instantiate entry point.
type InstantiateMessage struct {
	ClientState    []byte `json:"client_state"`
	ConsensusState []byte `json:"consensus_state"`
	Checksum       []byte `json:"checksum"`
}

// QueryMsg is used to encode messages that are sent to the contract's query entry point.
// The json omitempty tag is mandatory since it omits any empty (default initialized) fields from the encoded JSON,
// this is required in order to be compatible with Rust's enum matching as used in the contract.
// Only one field should be set at a time.
type QueryMsg struct {
	Status               *StatusMsg               `json:"status,omitempty"`
	ExportMetadata       *ExportMetadataMsg       `json:"export_metadata,omitempty"`
	TimestampAtHeight    *TimestampAtHeightMsg    `json:"timestamp_at_height,omitempty"`
	VerifyClientMessage  *VerifyClientMessageMsg  `json:"verify_client_message,omitempty"`
	CheckForMisbehaviour *CheckForMisbehaviourMsg `json:"check_for_misbehaviour,omitempty"`
}

// StatusMsg is a queryMsg sent to the contract to query the status of the wasm client.
type StatusMsg struct{}

// ExportMetadataMsg is a queryMsg sent to the contract to query the exported metadata of the wasm client.
type ExportMetadataMsg struct{}

// TimestampAtHeightMsg is a queryMsg sent to the contract to query the timestamp at a given height.
type TimestampAtHeightMsg struct {
	Height clienttypes.Height `json:"height"`
}

// VerifyClientMessageMsg is a queryMsg sent to the contract to verify a client message.
type VerifyClientMessageMsg struct {
	ClientMessage []byte `json:"client_message"`
}

// CheckForMisbehaviourMsg is a queryMsg sent to the contract to check for misbehaviour.
type CheckForMisbehaviourMsg struct {
	ClientMessage []byte `json:"client_message"`
}

// SudoMsg is used to encode messages that are sent to the contract's sudo entry point.
// The json omitempty tag is mandatory since it omits any empty (default initialized) fields from the encoded JSON,
// this is required in order to be compatible with Rust's enum matching as used in the contract.
// Only one field should be set at a time.
type SudoMsg struct {
	UpdateState                 *UpdateStateMsg                 `json:"update_state,omitempty"`
	UpdateStateOnMisbehaviour   *UpdateStateOnMisbehaviourMsg   `json:"update_state_on_misbehaviour,omitempty"`
	VerifyUpgradeAndUpdateState *VerifyUpgradeAndUpdateStateMsg `json:"verify_upgrade_and_update_state,omitempty"`
	VerifyMembership            *VerifyMembershipMsg            `json:"verify_membership,omitempty"`
	VerifyNonMembership         *VerifyNonMembershipMsg         `json:"verify_non_membership,omitempty"`
	MigrateClientStore          *MigrateClientStoreMsg          `json:"migrate_client_store,omitempty"`
}

// UpdateStateMsg is a sudoMsg sent to the contract to update the client state.
type UpdateStateMsg struct {
	ClientMessage []byte `json:"client_message"`
}

// UpdateStateOnMisbehaviourMsg is a sudoMsg sent to the contract to update its state on misbehaviour.
type UpdateStateOnMisbehaviourMsg struct {
	ClientMessage []byte `json:"client_message"`
}

// VerifyMembershipMsg is a sudoMsg sent to the contract to verify a membership proof.
type VerifyMembershipMsg struct {
	Height           clienttypes.Height         `json:"height"`
	DelayTimePeriod  uint64                     `json:"delay_time_period"`
	DelayBlockPeriod uint64                     `json:"delay_block_period"`
	Proof            []byte                     `json:"proof"`
	Path             commitmenttypes.MerklePath `json:"path"`
	Value            []byte                     `json:"value"`
}

// VerifyNonMembershipMsg is a sudoMsg sent to the contract to verify a non-membership proof.
type VerifyNonMembershipMsg struct {
	Height           clienttypes.Height         `json:"height"`
	DelayTimePeriod  uint64                     `json:"delay_time_period"`
	DelayBlockPeriod uint64                     `json:"delay_block_period"`
	Proof            []byte                     `json:"proof"`
	Path             commitmenttypes.MerklePath `json:"path"`
}

// VerifyUpgradeAndUpdateStateMsg is a sudoMsg sent to the contract to verify an upgrade and update its state.
type VerifyUpgradeAndUpdateStateMsg struct {
	UpgradeClientState         []byte `json:"upgrade_client_state"`
	UpgradeConsensusState      []byte `json:"upgrade_consensus_state"`
	ProofUpgradeClient         []byte `json:"proof_upgrade_client"`
	ProofUpgradeConsensusState []byte `json:"proof_upgrade_consensus_state"`
}

// MigrateClientStore is a sudoMsg sent to the contract to verify a given substitute client and update to its state.
type MigrateClientStoreMsg struct{}

// ContractResult is a type constraint that defines the expected results that can be returned by a contract call/query.
type ContractResult interface {
	EmptyResult | StatusResult | ExportMetadataResult | TimestampAtHeightResult | CheckForMisbehaviourResult | UpdateStateResult
}

// EmptyResult is the default return type of any contract call that does not require a custom return type.
type EmptyResult struct{}

// StatusResult is the expected return type of the statusMsg query. It returns the status of the wasm client.
type StatusResult struct {
	Status string `json:"status"`
}

// ExportMetadataResult is the expected return type of the exportMetadataMsg query. It returns the exported metadata of the wasm client.
type ExportMetadataResult struct {
	GenesisMetadata []clienttypes.GenesisMetadata `json:"genesis_metadata"`
}

// TimestampAtHeightResult is the expected return type of the timestampAtHeightMsg query. It returns the timestamp for a light client
// at a given height.
type TimestampAtHeightResult struct {
	Timestamp uint64 `json:"timestamp"`
}

// CheckForMisbehaviourResult is the expected return type of the checkForMisbehaviourMsg query. It returns a boolean indicating
// if misbehaviour was detected.
type CheckForMisbehaviourResult struct {
	FoundMisbehaviour bool `json:"found_misbehaviour"`
}

// UpdateStateResult is the expected return type of the updateStateMsg sudo call. It returns the updated consensus heights.
type UpdateStateResult struct {
	Heights []clienttypes.Height `json:"heights"`
}
