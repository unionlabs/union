package types

import (
	"encoding/json"
	"fmt"
)

//------- Results / Msgs -------------

// ContractResult is the raw response from the instantiate/execute/migrate calls.
// This is mirrors Rust's ContractResult<Response>.
type ContractResult struct {
	Ok  *Response `json:"ok,omitempty"`
	Err string    `json:"error,omitempty"`
}

// Response defines the return value on a successful instantiate/execute/migrate.
// This is the counterpart of [Response](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/results/response.rs#L73-L88)
type Response struct {
	// Messages comes directly from the contract and is its request for action.
	// If the ReplyOn value matches the result, the runtime will invoke this
	// contract's `reply` entry point after execution. Otherwise, this is all
	// "fire and forget".
	Messages []SubMsg `json:"messages"`
	// base64-encoded bytes to return as ABCI.Data field
	Data []byte `json:"data"`
	// attributes for a log event to return over abci interface
	Attributes []EventAttribute `json:"attributes"`
	// custom events (separate from the main one that contains the attributes
	// above)
	Events []Event `json:"events"`
}

// Events must encode empty array as []
type Events []Event

// MarshalJSON ensures that we get [] for empty arrays
func (e Events) MarshalJSON() ([]byte, error) {
	if len(e) == 0 {
		return []byte("[]"), nil
	}
	var raw []Event = e
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (e *Events) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []Event
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*e = raw
	return nil
}

type Event struct {
	Type       string          `json:"type"`
	Attributes EventAttributes `json:"attributes"`
}

// EventAttributes must encode empty array as []
type EventAttributes []EventAttribute

// MarshalJSON ensures that we get [] for empty arrays
func (a EventAttributes) MarshalJSON() ([]byte, error) {
	if len(a) == 0 {
		return []byte("[]"), nil
	}
	var raw []EventAttribute = a
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (a *EventAttributes) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []EventAttribute
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*a = raw
	return nil
}

// EventAttribute
type EventAttribute struct {
	Key   string `json:"key"`
	Value string `json:"value"`
}

// CosmosMsg is an rust enum and only (exactly) one of the fields should be set
// Should we do a cleaner approach in Go? (type/data?)
type CosmosMsg struct {
	Bank         *BankMsg         `json:"bank,omitempty"`
	Custom       json.RawMessage  `json:"custom,omitempty"`
	Distribution *DistributionMsg `json:"distribution,omitempty"`
	Gov          *GovMsg          `json:"gov,omitempty"`
	IBC          *IBCMsg          `json:"ibc,omitempty"`
	Staking      *StakingMsg      `json:"staking,omitempty"`
	Stargate     *StargateMsg     `json:"stargate,omitempty"`
	Wasm         *WasmMsg         `json:"wasm,omitempty"`
}

type BankMsg struct {
	Send *SendMsg `json:"send,omitempty"`
	Burn *BurnMsg `json:"burn,omitempty"`
}

// SendMsg contains instructions for a Cosmos-SDK/SendMsg
// It has a fixed interface here and should be converted into the proper SDK format before dispatching
type SendMsg struct {
	ToAddress string `json:"to_address"`
	Amount    Coins  `json:"amount"`
}

// BurnMsg will burn the given coins from the contract's account.
// There is no Cosmos SDK message that performs this, but it can be done by calling the bank keeper.
// Important if a contract controls significant token supply that must be retired.
type BurnMsg struct {
	Amount Coins `json:"amount"`
}

type IBCMsg struct {
	Transfer     *TransferMsg     `json:"transfer,omitempty"`
	SendPacket   *SendPacketMsg   `json:"send_packet,omitempty"`
	CloseChannel *CloseChannelMsg `json:"close_channel,omitempty"`
}

type GovMsg struct {
	// This maps directly to [MsgVote](https://github.com/cosmos/cosmos-sdk/blob/v0.42.5/proto/cosmos/gov/v1beta1/tx.proto#L46-L56) in the Cosmos SDK with voter set to the contract address.
	Vote *VoteMsg `json:"vote,omitempty"`
	/// This maps directly to [MsgVoteWeighted](https://github.com/cosmos/cosmos-sdk/blob/v0.45.8/proto/cosmos/gov/v1beta1/tx.proto#L66-L78) in the Cosmos SDK with voter set to the contract address.
	VoteWeighted *VoteWeightedMsg `json:"vote_weighted,omitempty"`
}

type voteOption int

type VoteMsg struct {
	ProposalId uint64 `json:"proposal_id"`
	// Vote is the vote option.
	//
	// This should be called "option" for consistency with Cosmos SDK. Sorry for that.
	// See <https://github.com/CosmWasm/cosmwasm/issues/1571>.
	Vote voteOption `json:"vote"`
}

type VoteWeightedMsg struct {
	ProposalId uint64               `json:"proposal_id"`
	Options    []WeightedVoteOption `json:"options"`
}

type WeightedVoteOption struct {
	Option voteOption `json:"option"`
	// Weight is a Decimal string, e.g. "0.25" for 25%
	Weight string `json:"weight"`
}

const (
	UnsetVoteOption voteOption = iota // The default value. We never return this in any valid instance (see toVoteOption).
	Yes
	No
	Abstain
	NoWithVeto
)

var fromVoteOption = map[voteOption]string{
	Yes:        "yes",
	No:         "no",
	Abstain:    "abstain",
	NoWithVeto: "no_with_veto",
}

var toVoteOption = map[string]voteOption{
	"yes":          Yes,
	"no":           No,
	"abstain":      Abstain,
	"no_with_veto": NoWithVeto,
}

func (v voteOption) String() string {
	return fromVoteOption[v]
}

func (v voteOption) MarshalJSON() ([]byte, error) {
	return json.Marshal(v.String())
}

func (s *voteOption) UnmarshalJSON(b []byte) error {
	var j string
	err := json.Unmarshal(b, &j)
	if err != nil {
		return err
	}

	voteOption, ok := toVoteOption[j]
	if !ok {
		return fmt.Errorf("invalid vote option '%v'", j)
	}
	*s = voteOption
	return nil
}

type TransferMsg struct {
	ChannelID string     `json:"channel_id"`
	ToAddress string     `json:"to_address"`
	Amount    Coin       `json:"amount"`
	Timeout   IBCTimeout `json:"timeout"`
}

type SendPacketMsg struct {
	ChannelID string     `json:"channel_id"`
	Data      []byte     `json:"data"`
	Timeout   IBCTimeout `json:"timeout"`
}

type CloseChannelMsg struct {
	ChannelID string `json:"channel_id"`
}

type StakingMsg struct {
	Delegate   *DelegateMsg   `json:"delegate,omitempty"`
	Undelegate *UndelegateMsg `json:"undelegate,omitempty"`
	Redelegate *RedelegateMsg `json:"redelegate,omitempty"`
}

type DelegateMsg struct {
	Validator string `json:"validator"`
	Amount    Coin   `json:"amount"`
}

type UndelegateMsg struct {
	Validator string `json:"validator"`
	Amount    Coin   `json:"amount"`
}

type RedelegateMsg struct {
	SrcValidator string `json:"src_validator"`
	DstValidator string `json:"dst_validator"`
	Amount       Coin   `json:"amount"`
}

type DistributionMsg struct {
	SetWithdrawAddress      *SetWithdrawAddressMsg      `json:"set_withdraw_address,omitempty"`
	WithdrawDelegatorReward *WithdrawDelegatorRewardMsg `json:"withdraw_delegator_reward,omitempty"`
	FundCommunityPool       *FundCommunityPoolMsg       `json:"fund_community_pool,omitempty"`
}

// SetWithdrawAddressMsg is translated to a [MsgSetWithdrawAddress](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L29-L37).
// `delegator_address` is automatically filled with the current contract's address.
type SetWithdrawAddressMsg struct {
	// Address contains the `delegator_address` of a MsgSetWithdrawAddress
	Address string `json:"address"`
}

// WithdrawDelegatorRewardMsg is translated to a [MsgWithdrawDelegatorReward](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#L42-L50).
// `delegator_address` is automatically filled with the current contract's address.
type WithdrawDelegatorRewardMsg struct {
	// Validator contains `validator_address` of a MsgWithdrawDelegatorReward
	Validator string `json:"validator"`
}

// FundCommunityPoolMsg is translated to a [MsgFundCommunityPool](https://github.com/cosmos/cosmos-sdk/blob/v0.42.4/proto/cosmos/distribution/v1beta1/tx.proto#LL69C1-L76C2).
// `depositor` is automatically filled with the current contract's address
type FundCommunityPoolMsg struct {
	// Amount is the list of coins to be send to the community pool
	Amount Coins `json:"amount"`
}

// StargateMsg is encoded the same way as a protobof [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
type StargateMsg struct {
	TypeURL string `json:"type_url"`
	Value   []byte `json:"value"`
}

type WasmMsg struct {
	Execute      *ExecuteMsg      `json:"execute,omitempty"`
	Instantiate  *InstantiateMsg  `json:"instantiate,omitempty"`
	Instantiate2 *Instantiate2Msg `json:"instantiate2,omitempty"`
	Migrate      *MigrateMsg      `json:"migrate,omitempty"`
	UpdateAdmin  *UpdateAdminMsg  `json:"update_admin,omitempty"`
	ClearAdmin   *ClearAdminMsg   `json:"clear_admin,omitempty"`
}

// ExecuteMsg is used to call another defined contract on this chain.
// The calling contract requires the callee to be defined beforehand,
// and the address should have been defined in initialization.
// And we assume the developer tested the ABIs and coded them together.
//
// Since a contract is immutable once it is deployed, we don't need to transform this.
// If it was properly coded and worked once, it will continue to work throughout upgrades.
type ExecuteMsg struct {
	// ContractAddr is the sdk.AccAddress of the contract, which uniquely defines
	// the contract ID and instance ID. The sdk module should maintain a reverse lookup table.
	ContractAddr string `json:"contract_addr"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Handle` on the above-defined contract
	Msg []byte `json:"msg"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds Coins `json:"funds"`
}

// InstantiateMsg will create a new contract instance from a previously uploaded CodeID.
// This allows one contract to spawn "sub-contracts".
type InstantiateMsg struct {
	// CodeID is the reference to the wasm byte code as used by the Cosmos-SDK
	CodeID uint64 `json:"code_id"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Instantiate` on a new contract with the above-defined CodeID
	Msg []byte `json:"msg"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds Coins `json:"funds"`
	// Label is optional metadata to be stored with a contract instance.
	Label string `json:"label"`
	// Admin (optional) may be set here to allow future migrations from this address
	Admin string `json:"admin,omitempty"`
}

// Instantiate2Msg will create a new contract instance from a previously uploaded CodeID
// using the predictable address derivation.
type Instantiate2Msg struct {
	// CodeID is the reference to the wasm byte code as used by the Cosmos-SDK
	CodeID uint64 `json:"code_id"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Instantiate` on a new contract with the above-defined CodeID
	Msg []byte `json:"msg"`
	// Send is an optional amount of coins this contract sends to the called contract
	Funds Coins `json:"funds"`
	// Label is optional metadata to be stored with a contract instance.
	Label string `json:"label"`
	// Admin (optional) may be set here to allow future migrations from this address
	Admin string `json:"admin,omitempty"`
	Salt  []byte `json:"salt"`
}

// MigrateMsg will migrate an existing contract from it's current wasm code (logic)
// to another previously uploaded wasm code. It requires the calling contract to be
// listed as "admin" of the contract to be migrated.
type MigrateMsg struct {
	// ContractAddr is the sdk.AccAddress of the target contract, to migrate.
	ContractAddr string `json:"contract_addr"`
	// NewCodeID is the reference to the wasm byte code for the new logic to migrate to
	NewCodeID uint64 `json:"new_code_id"`
	// Msg is assumed to be a json-encoded message, which will be passed directly
	// as `userMsg` when calling `Migrate` on the above-defined contract
	Msg []byte `json:"msg"`
}

// UpdateAdminMsg is the Go counterpart of WasmMsg::UpdateAdmin
// (https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta5/packages/std/src/results/cosmos_msg.rs#L158-L160).
type UpdateAdminMsg struct {
	// ContractAddr is the sdk.AccAddress of the target contract.
	ContractAddr string `json:"contract_addr"`
	// Admin is the sdk.AccAddress of the new admin.
	Admin string `json:"admin"`
}

// ClearAdminMsg is the Go counterpart of WasmMsg::ClearAdmin
// (https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta5/packages/std/src/results/cosmos_msg.rs#L158-L160).
type ClearAdminMsg struct {
	// ContractAddr is the sdk.AccAddress of the target contract.
	ContractAddr string `json:"contract_addr"`
}
