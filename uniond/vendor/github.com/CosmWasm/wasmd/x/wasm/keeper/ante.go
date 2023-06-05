package keeper

import (
	"encoding/binary"

	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// CountTXDecorator ante handler to count the tx position in a block.
type CountTXDecorator struct {
	storeKey storetypes.StoreKey
}

// NewCountTXDecorator constructor
func NewCountTXDecorator(storeKey storetypes.StoreKey) *CountTXDecorator {
	return &CountTXDecorator{storeKey: storeKey}
}

// AnteHandle handler stores a tx counter with current height encoded in the store to let the app handle
// global rollback behavior instead of keeping state in the handler itself.
// The ante handler passes the counter value via sdk.Context upstream. See `types.TXCounter(ctx)` to read the value.
// Simulations don't get a tx counter value assigned.
func (a CountTXDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	if simulate {
		return next(ctx, tx, simulate)
	}
	store := ctx.KVStore(a.storeKey)
	currentHeight := ctx.BlockHeight()

	var txCounter uint32 // start with 0
	// load counter when exists
	if bz := store.Get(types.TXCounterPrefix); bz != nil {
		lastHeight, val := decodeHeightCounter(bz)
		if currentHeight == lastHeight {
			// then use stored counter
			txCounter = val
		} // else use `0` from above to start with
	}
	// store next counter value for current height
	store.Set(types.TXCounterPrefix, encodeHeightCounter(currentHeight, txCounter+1))

	return next(types.WithTXCounter(ctx, txCounter), tx, simulate)
}

func encodeHeightCounter(height int64, counter uint32) []byte {
	b := make([]byte, 4)
	binary.BigEndian.PutUint32(b, counter)
	return append(sdk.Uint64ToBigEndian(uint64(height)), b...)
}

func decodeHeightCounter(bz []byte) (int64, uint32) {
	return int64(sdk.BigEndianToUint64(bz[0:8])), binary.BigEndian.Uint32(bz[8:])
}

// LimitSimulationGasDecorator ante decorator to limit gas in simulation calls
type LimitSimulationGasDecorator struct {
	gasLimit *sdk.Gas
}

// NewLimitSimulationGasDecorator constructor accepts nil value to fallback to block gas limit.
func NewLimitSimulationGasDecorator(gasLimit *sdk.Gas) *LimitSimulationGasDecorator {
	if gasLimit != nil && *gasLimit == 0 {
		panic("gas limit must not be zero")
	}
	return &LimitSimulationGasDecorator{gasLimit: gasLimit}
}

// AnteHandle that limits the maximum gas available in simulations only.
// A custom max value can be configured and will be applied when set. The value should not
// exceed the max block gas limit.
// Different values on nodes are not consensus breaking as they affect only
// simulations but may have effect on client user experience.
//
// When no custom value is set then the max block gas is used as default limit.
func (d LimitSimulationGasDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	if !simulate {
		// Wasm code is not executed in checkTX so that we don't need to limit it further.
		// Tendermint rejects the TX afterwards when the tx.gas > max block gas.
		// On deliverTX we rely on the tendermint/sdk mechanics that ensure
		// tx has gas set and gas < max block gas
		return next(ctx, tx, simulate)
	}

	// apply custom node gas limit
	if d.gasLimit != nil {
		return next(ctx.WithGasMeter(sdk.NewGasMeter(*d.gasLimit)), tx, simulate)
	}

	// default to max block gas when set, to be on the safe side
	if maxGas := ctx.ConsensusParams().GetBlock().MaxGas; maxGas > 0 {
		return next(ctx.WithGasMeter(sdk.NewGasMeter(sdk.Gas(maxGas))), tx, simulate)
	}
	return next(ctx, tx, simulate)
}
