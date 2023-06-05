//go:build !cgo

package keeper

import (
	"github.com/cosmos/cosmos-sdk/codec"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// NewKeeper creates a new contract Keeper instance
// If customEncoders is non-nil, we can use this to override some of the message handler, especially custom
func NewKeeper(
	cdc codec.Codec,
	storeKey storetypes.StoreKey,
	accountKeeper types.AccountKeeper,
	bankKeeper types.BankKeeper,
	stakingKeeper types.StakingKeeper,
	distrKeeper types.DistributionKeeper,
	ics4Wrapper types.ICS4Wrapper,
	channelKeeper types.ChannelKeeper,
	portKeeper types.PortKeeper,
	capabilityKeeper types.CapabilityKeeper,
	portSource types.ICS20TransferPortSource,
	router MessageRouter,
	_ GRPCQueryRouter,
	homeDir string,
	wasmConfig types.WasmConfig,
	availableCapabilities string,
	authority string,
	opts ...Option,
) Keeper {
	panic("not implemented, please build with cgo enabled")
}
