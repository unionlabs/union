//go:build cgo

package keeper

import (
	"path/filepath"

	storetypes "github.com/cosmos/cosmos-sdk/store/types"

	wasmvm "github.com/CosmWasm/wasmvm"
	"github.com/cosmos/cosmos-sdk/codec"

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
	wasmer, err := wasmvm.NewVM(filepath.Join(homeDir, "wasm"), availableCapabilities, contractMemoryLimit, wasmConfig.ContractDebugMode, wasmConfig.MemoryCacheSize)
	if err != nil {
		panic(err)
	}

	keeper := &Keeper{
		storeKey:             storeKey,
		cdc:                  cdc,
		wasmVM:               wasmer,
		accountKeeper:        accountKeeper,
		bank:                 NewBankCoinTransferrer(bankKeeper),
		accountPruner:        NewVestingCoinBurner(bankKeeper),
		portKeeper:           portKeeper,
		capabilityKeeper:     capabilityKeeper,
		messenger:            NewDefaultMessageHandler(router, ics4Wrapper, channelKeeper, capabilityKeeper, bankKeeper, cdc, portSource),
		queryGasLimit:        wasmConfig.SmartQueryGasLimit,
		gasRegister:          NewDefaultWasmGasRegister(),
		maxQueryStackSize:    types.DefaultMaxQueryStackSize,
		acceptedAccountTypes: defaultAcceptedAccountTypes,
		authority:            authority,
	}
	keeper.wasmVMQueryHandler = DefaultQueryPlugins(bankKeeper, stakingKeeper, distrKeeper, channelKeeper, keeper)
	for _, o := range opts {
		o.apply(keeper)
	}
	// not updateable, yet
	keeper.wasmVMResponseHandler = NewDefaultWasmVMContractResponseHandler(NewMessageDispatcher(keeper.messenger, keeper))
	return *keeper
}
