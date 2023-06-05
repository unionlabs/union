package types

import (
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	capabilitytypes "github.com/cosmos/cosmos-sdk/x/capability/types"
	ibcexported "github.com/cosmos/ibc-go/v7/modules/core/exported"
)

// ViewKeeper provides read only operations
type ViewKeeper interface {
	GetContractHistory(ctx sdk.Context, contractAddr sdk.AccAddress) []ContractCodeHistoryEntry
	QuerySmart(ctx sdk.Context, contractAddr sdk.AccAddress, req []byte) ([]byte, error)
	QueryRaw(ctx sdk.Context, contractAddress sdk.AccAddress, key []byte) []byte
	HasContractInfo(ctx sdk.Context, contractAddress sdk.AccAddress) bool
	GetContractInfo(ctx sdk.Context, contractAddress sdk.AccAddress) *ContractInfo
	IterateContractInfo(ctx sdk.Context, cb func(sdk.AccAddress, ContractInfo) bool)
	IterateContractsByCreator(ctx sdk.Context, creator sdk.AccAddress, cb func(address sdk.AccAddress) bool)
	IterateContractsByCode(ctx sdk.Context, codeID uint64, cb func(address sdk.AccAddress) bool)
	IterateContractState(ctx sdk.Context, contractAddress sdk.AccAddress, cb func(key, value []byte) bool)
	GetCodeInfo(ctx sdk.Context, codeID uint64) *CodeInfo
	IterateCodeInfos(ctx sdk.Context, cb func(uint64, CodeInfo) bool)
	GetByteCode(ctx sdk.Context, codeID uint64) ([]byte, error)
	IsPinnedCode(ctx sdk.Context, codeID uint64) bool
	GetParams(ctx sdk.Context) Params
}

// ContractOpsKeeper contains mutable operations on a contract.
type ContractOpsKeeper interface {
	// Create uploads and compiles a WASM contract, returning a short identifier for the contract
	Create(ctx sdk.Context, creator sdk.AccAddress, wasmCode []byte, instantiateAccess *AccessConfig) (codeID uint64, checksum []byte, err error)

	// Instantiate creates an instance of a WASM contract using the classic sequence based address generator
	Instantiate(
		ctx sdk.Context,
		codeID uint64,
		creator, admin sdk.AccAddress,
		initMsg []byte,
		label string,
		deposit sdk.Coins,
	) (sdk.AccAddress, []byte, error)

	// Instantiate2 creates an instance of a WASM contract using the predictable address generator
	Instantiate2(
		ctx sdk.Context,
		codeID uint64,
		creator, admin sdk.AccAddress,
		initMsg []byte,
		label string,
		deposit sdk.Coins,
		salt []byte,
		fixMsg bool,
	) (sdk.AccAddress, []byte, error)

	// Execute executes the contract instance
	Execute(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, msg []byte, coins sdk.Coins) ([]byte, error)

	// Migrate allows to upgrade a contract to a new code with data migration.
	Migrate(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, newCodeID uint64, msg []byte) ([]byte, error)

	// Sudo allows to call privileged entry point of a contract.
	Sudo(ctx sdk.Context, contractAddress sdk.AccAddress, msg []byte) ([]byte, error)

	// UpdateContractAdmin sets the admin value on the ContractInfo. It must be a valid address (use ClearContractAdmin to remove it)
	UpdateContractAdmin(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress, newAdmin sdk.AccAddress) error

	// ClearContractAdmin sets the admin value on the ContractInfo to nil, to disable further migrations/ updates.
	ClearContractAdmin(ctx sdk.Context, contractAddress sdk.AccAddress, caller sdk.AccAddress) error

	// PinCode pins the wasm contract in wasmvm cache
	PinCode(ctx sdk.Context, codeID uint64) error

	// UnpinCode removes the wasm contract from wasmvm cache
	UnpinCode(ctx sdk.Context, codeID uint64) error

	// SetContractInfoExtension updates the extension point data that is stored with the contract info
	SetContractInfoExtension(ctx sdk.Context, contract sdk.AccAddress, extra ContractInfoExtension) error

	// SetAccessConfig updates the access config of a code id.
	SetAccessConfig(ctx sdk.Context, codeID uint64, caller sdk.AccAddress, newConfig AccessConfig) error
}

// IBCContractKeeper IBC lifecycle event handler
type IBCContractKeeper interface {
	OnOpenChannel(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		msg wasmvmtypes.IBCChannelOpenMsg,
	) (string, error)
	OnConnectChannel(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		msg wasmvmtypes.IBCChannelConnectMsg,
	) error
	OnCloseChannel(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		msg wasmvmtypes.IBCChannelCloseMsg,
	) error
	OnRecvPacket(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		msg wasmvmtypes.IBCPacketReceiveMsg,
	) (ibcexported.Acknowledgement, error)
	OnAckPacket(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		acknowledgement wasmvmtypes.IBCPacketAckMsg,
	) error
	OnTimeoutPacket(
		ctx sdk.Context,
		contractAddr sdk.AccAddress,
		msg wasmvmtypes.IBCPacketTimeoutMsg,
	) error
	// ClaimCapability allows the transfer module to claim a capability
	// that IBC module passes to it
	ClaimCapability(ctx sdk.Context, cap *capabilitytypes.Capability, name string) error
	// AuthenticateCapability wraps the scopedKeeper's AuthenticateCapability function
	AuthenticateCapability(ctx sdk.Context, cap *capabilitytypes.Capability, name string) bool
}
