package types

import (
	"encoding/json"
)

//-------- Queries --------

// QueryResponse is the Go counterpart of `ContractResult<Binary>`.
// The JSON annotations are used for deserializing directly. There is a custom serializer below.
type QueryResponse queryResponseImpl

type queryResponseImpl struct {
	Ok  []byte `json:"ok,omitempty"`
	Err string `json:"error,omitempty"`
}

// A custom serializer that allows us to map QueryResponse instances to the Rust
// enum `ContractResult<Binary>`
func (q QueryResponse) MarshalJSON() ([]byte, error) {
	// In case both Ok and Err are empty, this is interpreted and seralized
	// as an Ok case with no data because errors must not be empty.
	if len(q.Ok) == 0 && len(q.Err) == 0 {
		return []byte(`{"ok":""}`), nil
	}
	return json.Marshal(queryResponseImpl(q))
}

//-------- Querier -----------

type Querier interface {
	Query(request QueryRequest, gasLimit uint64) ([]byte, error)
	GasConsumed() uint64
}

// this is a thin wrapper around the desired Go API to give us types closer to Rust FFI
func RustQuery(querier Querier, binRequest []byte, gasLimit uint64) QuerierResult {
	var request QueryRequest
	err := json.Unmarshal(binRequest, &request)
	if err != nil {
		return QuerierResult{
			Err: &SystemError{
				InvalidRequest: &InvalidRequest{
					Err:     err.Error(),
					Request: binRequest,
				},
			},
		}
	}
	bz, err := querier.Query(request, gasLimit)
	return ToQuerierResult(bz, err)
}

// This is a 2-level result
type QuerierResult struct {
	Ok  *QueryResponse `json:"ok,omitempty"`
	Err *SystemError   `json:"error,omitempty"`
}

func ToQuerierResult(response []byte, err error) QuerierResult {
	if err == nil {
		return QuerierResult{
			Ok: &QueryResponse{
				Ok: response,
			},
		}
	}
	syserr := ToSystemError(err)
	if syserr != nil {
		return QuerierResult{
			Err: syserr,
		}
	}
	return QuerierResult{
		Ok: &QueryResponse{
			Err: err.Error(),
		},
	}
}

// QueryRequest is an rust enum and only (exactly) one of the fields should be set
// Should we do a cleaner approach in Go? (type/data?)
type QueryRequest struct {
	Bank     *BankQuery      `json:"bank,omitempty"`
	Custom   json.RawMessage `json:"custom,omitempty"`
	IBC      *IBCQuery       `json:"ibc,omitempty"`
	Staking  *StakingQuery   `json:"staking,omitempty"`
	Stargate *StargateQuery  `json:"stargate,omitempty"`
	Wasm     *WasmQuery      `json:"wasm,omitempty"`
}

type BankQuery struct {
	Supply      *SupplyQuery      `json:"supply,omitempty"`
	Balance     *BalanceQuery     `json:"balance,omitempty"`
	AllBalances *AllBalancesQuery `json:"all_balances,omitempty"`
}

type SupplyQuery struct {
	Denom string `json:"denom"`
}

// SupplyResponse is the expected response to SupplyQuery
type SupplyResponse struct {
	Amount Coin `json:"amount"`
}

type BalanceQuery struct {
	Address string `json:"address"`
	Denom   string `json:"denom"`
}

// BalanceResponse is the expected response to BalanceQuery
type BalanceResponse struct {
	Amount Coin `json:"amount"`
}

type AllBalancesQuery struct {
	Address string `json:"address"`
}

// AllBalancesResponse is the expected response to AllBalancesQuery
type AllBalancesResponse struct {
	Amount Coins `json:"amount"`
}

// IBCQuery defines a query request from the contract into the chain.
// This is the counterpart of [IbcQuery](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L61-L83).
type IBCQuery struct {
	PortID       *PortIDQuery       `json:"port_id,omitempty"`
	ListChannels *ListChannelsQuery `json:"list_channels,omitempty"`
	Channel      *ChannelQuery      `json:"channel,omitempty"`
}

type PortIDQuery struct{}

type PortIDResponse struct {
	PortID string `json:"port_id"`
}

// ListChannelsQuery is an IBCQuery that lists all channels that are bound to a given port.
// If `PortID` is unset, this list all channels bound to the contract's port.
// Returns a `ListChannelsResponse`.
// This is the counterpart of [IbcQuery::ListChannels](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/packages/std/src/ibc.rs#L70-L73).
type ListChannelsQuery struct {
	// optional argument
	PortID string `json:"port_id,omitempty"`
}

type ListChannelsResponse struct {
	Channels IBCChannels `json:"channels"`
}

// IBCChannels must JSON encode empty array as [] (not null) for consistency with Rust parser
type IBCChannels []IBCChannel

// MarshalJSON ensures that we get [] for empty arrays
func (e IBCChannels) MarshalJSON() ([]byte, error) {
	if len(e) == 0 {
		return []byte("[]"), nil
	}
	var raw []IBCChannel = e
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (e *IBCChannels) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []IBCChannel
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*e = raw
	return nil
}

// IBCEndpoints must JSON encode empty array as [] (not null) for consistency with Rust parser
type IBCEndpoints []IBCEndpoint

// MarshalJSON ensures that we get [] for empty arrays
func (e IBCEndpoints) MarshalJSON() ([]byte, error) {
	if len(e) == 0 {
		return []byte("[]"), nil
	}
	var raw []IBCEndpoint = e
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (e *IBCEndpoints) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []IBCEndpoint
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*e = raw
	return nil
}

type ChannelQuery struct {
	// optional argument
	PortID    string `json:"port_id,omitempty"`
	ChannelID string `json:"channel_id"`
}

type ChannelResponse struct {
	// may be empty if there is no matching channel
	Channel *IBCChannel `json:"channel,omitempty"`
}

type StakingQuery struct {
	AllValidators  *AllValidatorsQuery  `json:"all_validators,omitempty"`
	Validator      *ValidatorQuery      `json:"validator,omitempty"`
	AllDelegations *AllDelegationsQuery `json:"all_delegations,omitempty"`
	Delegation     *DelegationQuery     `json:"delegation,omitempty"`
	BondedDenom    *struct{}            `json:"bonded_denom,omitempty"`
}

type AllValidatorsQuery struct{}

// AllValidatorsResponse is the expected response to AllValidatorsQuery
type AllValidatorsResponse struct {
	Validators Validators `json:"validators"`
}

// Validators must JSON encode empty array as []
type Validators []Validator

// MarshalJSON ensures that we get [] for empty arrays
func (v Validators) MarshalJSON() ([]byte, error) {
	if len(v) == 0 {
		return []byte("[]"), nil
	}
	var raw []Validator = v
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (v *Validators) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []Validator
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*v = raw
	return nil
}

type ValidatorQuery struct {
	/// Address is the validator's address (e.g. cosmosvaloper1...)
	Address string `json:"address"`
}

// ValidatorResponse is the expected response to ValidatorQuery
type ValidatorResponse struct {
	Validator *Validator `json:"validator"` // serializes to `null` when unset which matches Rust's Option::None serialization
}

type Validator struct {
	Address string `json:"address"`
	// decimal string, eg "0.02"
	Commission string `json:"commission"`
	// decimal string, eg "0.02"
	MaxCommission string `json:"max_commission"`
	// decimal string, eg "0.02"
	MaxChangeRate string `json:"max_change_rate"`
}

type AllDelegationsQuery struct {
	Delegator string `json:"delegator"`
}

type DelegationQuery struct {
	Delegator string `json:"delegator"`
	Validator string `json:"validator"`
}

// AllDelegationsResponse is the expected response to AllDelegationsQuery
type AllDelegationsResponse struct {
	Delegations Delegations `json:"delegations"`
}

type Delegations []Delegation

// MarshalJSON ensures that we get [] for empty arrays
func (d Delegations) MarshalJSON() ([]byte, error) {
	if len(d) == 0 {
		return []byte("[]"), nil
	}
	var raw []Delegation = d
	return json.Marshal(raw)
}

// UnmarshalJSON ensures that we get [] for empty arrays
func (d *Delegations) UnmarshalJSON(data []byte) error {
	// make sure we deserialize [] back to null
	if string(data) == "[]" || string(data) == "null" {
		return nil
	}
	var raw []Delegation
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	*d = raw
	return nil
}

type Delegation struct {
	Delegator string `json:"delegator"`
	Validator string `json:"validator"`
	Amount    Coin   `json:"amount"`
}

// DelegationResponse is the expected response to DelegationsQuery
type DelegationResponse struct {
	Delegation *FullDelegation `json:"delegation,omitempty"`
}

type FullDelegation struct {
	Delegator          string `json:"delegator"`
	Validator          string `json:"validator"`
	Amount             Coin   `json:"amount"`
	AccumulatedRewards Coins  `json:"accumulated_rewards"`
	CanRedelegate      Coin   `json:"can_redelegate"`
}

type BondedDenomResponse struct {
	Denom string `json:"denom"`
}

// StargateQuery is encoded the same way as abci_query, with path and protobuf encoded request data.
// The format is defined in [ADR-21](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-021-protobuf-query-encoding.md).
// The response is protobuf encoded data directly without a JSON response wrapper.
// The caller is responsible for compiling the proper protobuf definitions for both requests and responses.
type StargateQuery struct {
	// this is the fully qualified service path used for routing,
	// eg. custom/cosmos_sdk.x.bank.v1.Query/QueryBalance
	Path string `json:"path"`
	// this is the expected protobuf message type (not any), binary encoded
	Data []byte `json:"data"`
}

type WasmQuery struct {
	Smart        *SmartQuery        `json:"smart,omitempty"`
	Raw          *RawQuery          `json:"raw,omitempty"`
	ContractInfo *ContractInfoQuery `json:"contract_info,omitempty"`
	CodeInfo     *CodeInfoQuery     `json:"code_info,omitempty"`
}

// SmartQuery response is raw bytes ([]byte)
type SmartQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
	Msg          []byte `json:"msg"`
}

// RawQuery response is raw bytes ([]byte)
type RawQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
	Key          []byte `json:"key"`
}

type ContractInfoQuery struct {
	// Bech32 encoded sdk.AccAddress of the contract
	ContractAddr string `json:"contract_addr"`
}

type ContractInfoResponse struct {
	CodeID  uint64 `json:"code_id"`
	Creator string `json:"creator"`
	// Set to the admin who can migrate contract, if any
	Admin  string `json:"admin,omitempty"`
	Pinned bool   `json:"pinned"`
	// Set if the contract is IBC enabled
	IBCPort string `json:"ibc_port,omitempty"`
}

type CodeInfoQuery struct {
	CodeID uint64 `json:"code_id"`
}

type CodeInfoResponse struct {
	CodeID   uint64   `json:"code_id"`
	Creator  string   `json:"creator"`
	Checksum Checksum `json:"checksum,omitempty"`
}
