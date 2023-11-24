package keeper

import (
	"encoding/hex"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
)

// emitStoreWasmCodeEvent emits a store wasm code event
func emitStoreWasmCodeEvent(ctx sdk.Context, checksum types.Checksum) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeStoreWasmCode,
			sdk.NewAttribute(types.AttributeKeyWasmChecksum, hex.EncodeToString(checksum)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}

// emitMigrateContractEvent emits a migrate contract event
func emitMigrateContractEvent(ctx sdk.Context, clientID string, checksum, newChecksum types.Checksum) {
	ctx.EventManager().EmitEvents(sdk.Events{
		sdk.NewEvent(
			types.EventTypeMigrateContract,
			sdk.NewAttribute(types.AttributeKeyClientID, clientID),
			sdk.NewAttribute(types.AttributeKeyWasmChecksum, hex.EncodeToString(checksum)),
			sdk.NewAttribute(types.AttributeKeyNewChecksum, hex.EncodeToString(newChecksum)),
		),
		sdk.NewEvent(
			sdk.EventTypeMessage,
			sdk.NewAttribute(sdk.AttributeKeyModule, types.AttributeValueCategory),
		),
	})
}
