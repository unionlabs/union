//go:build !cgo || nolink_libwasmvm

package keeper

import (
	storetypes "cosmossdk.io/core/store"

	"github.com/cosmos/cosmos-sdk/codec"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// NewKeeperWithVM creates a new Keeper instance with the provided Wasm VM.
// This constructor function is used when binaries are compiled with cgo disabled or the
// custom build directive: nolink_libwasmvm.
// This function is intended to panic and notify users that 08-wasm keeper functionality is not available.
func NewKeeperWithVM(
	_ codec.BinaryCodec,
	_ storetypes.KVStoreService,
	_ types.ClientKeeper,
	_ string,
	_ ibcwasm.WasmEngine,
	_ ibcwasm.QueryRouter,
	_ ...Option,
) Keeper {
	panic("not implemented, please build with cgo enabled or nolink_libwasmvm disabled")
}

// NewKeeperWithConfig creates a new Keeper instance with the provided Wasm configuration.
// This constructor function is used when binaries are compiled with cgo disabled or the
// custom build directive: nolink_libwasmvm.
// This function is intended to panic and notify users that 08-wasm keeper functionality is not available.
func NewKeeperWithConfig(
	_ codec.BinaryCodec,
	_ storetypes.KVStoreService,
	_ types.ClientKeeper,
	_ string,
	_ types.WasmConfig,
	_ ibcwasm.QueryRouter,
	_ ...Option,
) Keeper {
	panic("not implemented, please build with cgo enabled or nolink_libwasmvm disabled")
}
