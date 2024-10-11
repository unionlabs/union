package keeper

import (
	"encoding/binary"

	corestoretypes "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// CountTXDecorator ante handler to count the tx position in a block.
type CountTXDecorator struct {
	storeService corestoretypes.KVStoreService
}

// NewCountTXDecorator constructor
func NewCountTXDecorator(s corestoretypes.KVStoreService) *CountTXDecorator {
	return &CountTXDecorator{storeService: s}
}

// AnteHandle handler stores a tx counter with current height encoded in the store to let the app handle
// global rollback behavior instead of keeping state in the handler itself.
// The ante handler passes the counter value via sdk.Context upstream. See `types.TXCounter(ctx)` to read the value.
// Simulations don't get a tx counter value assigned.
func (a CountTXDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	if simulate {
		return next(ctx, tx, simulate)
	}
	store := a.storeService.OpenKVStore(ctx)
	currentHeight := ctx.BlockHeight()

	var txCounter uint32 // start with 0
	// load counter when exists
	bz, err := store.Get(types.TXCounterPrefix)
	if err != nil {
		return ctx, errorsmod.Wrap(err, "read tx counter")
	}
	if bz != nil {
		lastHeight, val := decodeHeightCounter(bz)
		if currentHeight == lastHeight {
			// then use stored counter
			txCounter = val
		} // else use `0` from above to start with
	}
	// store next counter value for current height
	err = store.Set(types.TXCounterPrefix, encodeHeightCounter(currentHeight, txCounter+1))
	if err != nil {
		return ctx, errorsmod.Wrap(err, "store tx counter")
	}
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
	gasLimit *storetypes.Gas
}

// NewLimitSimulationGasDecorator constructor accepts nil value to fallback to block gas limit.
func NewLimitSimulationGasDecorator(gasLimit *storetypes.Gas) *LimitSimulationGasDecorator {
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
		return next(ctx.WithGasMeter(storetypes.NewGasMeter(*d.gasLimit)), tx, simulate)
	}

	// default to max block gas when set, to be on the safe side
	params := ctx.ConsensusParams()
	if maxGas := params.GetBlock().MaxGas; maxGas > 0 {
		return next(ctx.WithGasMeter(storetypes.NewGasMeter(storetypes.Gas(maxGas))), tx, simulate)
	}
	return next(ctx, tx, simulate)
}

// GasRegisterDecorator ante decorator to store gas register in the context
type GasRegisterDecorator struct {
	gasRegister types.GasRegister
}

// NewGasRegisterDecorator constructor.
func NewGasRegisterDecorator(gr types.GasRegister) *GasRegisterDecorator {
	return &GasRegisterDecorator{gasRegister: gr}
}

// AnteHandle adds the gas register to the context.
func (g GasRegisterDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	return next(types.WithGasRegister(ctx, g.gasRegister), tx, simulate)
}
