package types

import (
	"math"

	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// While gas_register.go is a direct copy of https://github.com/CosmWasm/wasmd/blob/main/x/wasm/types/gas_register.go
// This file contains additional constructs that can be maintained separately.
// Most of these functions are slight modifications of keeper function from wasmd, which act on `WasmGasRegister` instead of `Keeper`.
const (
	// DefaultDeserializationCostPerByte The formula should be `len(data) * deserializationCostPerByte`
	DefaultDeserializationCostPerByte = 1
)

var costJSONDeserialization = wasmvmtypes.UFraction{
	Numerator:   DefaultDeserializationCostPerByte * DefaultGasMultiplier,
	Denominator: 1,
}

func (g WasmGasRegister) runtimeGasForContract(ctx sdk.Context) uint64 {
	meter := ctx.GasMeter()
	if meter.IsOutOfGas() {
		return 0
	}
	// infinite gas meter with limit=0 or MaxUint64
	if meter.Limit() == 0 || meter.Limit() == math.MaxUint64 {
		return math.MaxUint64
	}
	return g.ToWasmVMGas(meter.Limit() - meter.GasConsumedToLimit())
}

func (g WasmGasRegister) consumeRuntimeGas(ctx sdk.Context, gas uint64) {
	consumed := g.FromWasmVMGas(gas)
	ctx.GasMeter().ConsumeGas(consumed, "wasm contract")
	// throw OutOfGas error if we ran out (got exactly to zero due to better limit enforcing)
	if ctx.GasMeter().IsOutOfGas() {
		panic(storetypes.ErrorOutOfGas{Descriptor: "Wasmer function execution"})
	}
}

// MultipliedGasMeter wraps the GasMeter from context and multiplies all reads by out defined multiplier
type MultipliedGasMeter struct {
	originalMeter storetypes.GasMeter
	GasRegister   GasRegister
}

func NewMultipliedGasMeter(originalMeter storetypes.GasMeter, gr GasRegister) MultipliedGasMeter {
	return MultipliedGasMeter{originalMeter: originalMeter, GasRegister: gr}
}

var _ wasmvm.GasMeter = MultipliedGasMeter{}

func (m MultipliedGasMeter) GasConsumed() storetypes.Gas {
	return m.GasRegister.ToWasmVMGas(m.originalMeter.GasConsumed())
}
