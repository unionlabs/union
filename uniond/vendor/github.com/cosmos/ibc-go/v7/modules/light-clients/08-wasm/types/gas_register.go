package types

import (
	"math"

	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Copied subset of gas features from wasmd
// https://github.com/CosmWasm/wasmd/blob/v0.31.0/x/wasm/keeper/gas_register.go
const (
	// DefaultGasMultiplier is how many CosmWasm gas points = 1 Cosmos SDK gas point.
	//
	// CosmWasm gas strategy is documented in https://github.com/CosmWasm/cosmwasm/blob/v1.0.0-beta/docs/GAS.md.
	// Cosmos SDK reference costs can be found here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.10/store/types/gas.go#L198-L209.
	//
	// The original multiplier of 100 up to CosmWasm 0.16 was based on
	//     "A write at ~3000 gas and ~200us = 10 gas per us (microsecond) cpu/io
	//     Rough timing have 88k gas at 90us, which is equal to 1k sdk gas... (one read)"
	// as well as manual Wasmer benchmarks from 2019. This was then multiplied by 150_000
	// in the 0.16 -> 1.0 upgrade (https://github.com/CosmWasm/cosmwasm/pull/1120).
	//
	// The multiplier deserves more reproducible benchmarking and a strategy that allows easy adjustments.
	// This is tracked in https://github.com/CosmWasm/wasmd/issues/566 and https://github.com/CosmWasm/wasmd/issues/631.
	// Gas adjustments are consensus breaking but may happen in any release marked as consensus breaking.
	// Do not make assumptions on how much gas an operation will consume in places that are hard to adjust,
	// such as hardcoding them in contracts.
	//
	// Please note that all gas prices returned to wasmvm should have this multiplied.
	// Benchmarks and numbers were discussed in: https://github.com/CosmWasm/wasmd/pull/634#issuecomment-938055852
	DefaultGasMultiplier uint64 = 140_000_000
	// DefaultInstanceCost is how much SDK gas we charge each time we load a WASM instance.
	// Creating a new instance is costly, and this helps put a recursion limit to contracts calling contracts.
	// Benchmarks and numbers were discussed in: https://github.com/CosmWasm/wasmd/pull/634#issuecomment-938056803
	DefaultInstanceCost uint64 = 60_000
	// DefaultCompileCost is how much SDK gas is charged *per byte* for compiling WASM code.
	// Benchmarks and numbers were discussed in: https://github.com/CosmWasm/wasmd/pull/634#issuecomment-938056803
	DefaultCompileCost uint64 = 3
	// DefaultContractMessageDataCost is how much SDK gas is charged *per byte* of the message that goes to the contract
	// This is used with len(msg). Note that the message is deserialized in the receiving contract and this is charged
	// with wasm gas already. The derserialization of results is also charged in wasmvm. I am unsure if we need to add
	// additional costs here.
	// Note: also used for error fields on reply, and data on reply. Maybe these should be pulled out to a different (non-zero) field
	DefaultContractMessageDataCost uint64 = 0
	// DefaultDeserializationCostPerByte The formula should be `len(data) * deserializationCostPerByte`
	DefaultDeserializationCostPerByte = 1
)

// default: 0.15 gas.
// see https://github.com/CosmWasm/wasmd/pull/898#discussion_r937727200
var defaultPerByteUncompressCost = wasmvmtypes.UFraction{
	Numerator:   15,
	Denominator: 100,
}

var costJSONDeserialization = wasmvmtypes.UFraction{
	Numerator:   DefaultDeserializationCostPerByte * DefaultGasMultiplier,
	Denominator: 1,
}

// DefaultPerByteUncompressCost is how much SDK gas we charge per source byte to unpack
func DefaultPerByteUncompressCost() wasmvmtypes.UFraction {
	return defaultPerByteUncompressCost
}

// GasRegister abstract source for gas costs
type GasRegister interface {
	// NewContractInstanceCosts costs to create a new contract instance from code
	NewContractInstanceCosts(msgLen int) sdk.Gas
	// CompileCosts costs to persist and "compile" a new wasm contract
	CompileCosts(byteLength int) sdk.Gas
	// InstantiateContractCosts costs when interacting with a wasm contract
	InstantiateContractCosts(msgLen int) sdk.Gas
	// ToWasmVMGas converts from sdk gas to wasmvm gas
	ToWasmVMGas(source sdk.Gas) uint64
	// FromWasmVMGas converts from wasmvm gas to sdk gas
	FromWasmVMGas(source uint64) sdk.Gas
}

// WasmGasRegisterConfig config type
type WasmGasRegisterConfig struct {
	// InstanceCost costs when interacting with a wasm contract
	InstanceCost sdk.Gas
	// CompileCosts costs to persist and "compile" a new wasm contract
	CompileCost sdk.Gas
	// UncompressCost costs per byte to unpack a contract
	UncompressCost wasmvmtypes.UFraction
	// GasMultiplier is how many cosmwasm gas points = 1 sdk gas point
	// SDK reference costs can be found here: https://github.com/cosmos/cosmos-sdk/blob/02c6c9fafd58da88550ab4d7d494724a477c8a68/store/types/gas.go#L153-L164
	GasMultiplier sdk.Gas
	// ContractMessageDataCost SDK gas charged *per byte* of the message that goes to the contract
	// This is used with len(msg)
	ContractMessageDataCost sdk.Gas
}

// DefaultGasRegisterConfig default values
func DefaultGasRegisterConfig() WasmGasRegisterConfig {
	return WasmGasRegisterConfig{
		InstanceCost:            DefaultInstanceCost,
		CompileCost:             DefaultCompileCost,
		GasMultiplier:           DefaultGasMultiplier,
		ContractMessageDataCost: DefaultContractMessageDataCost,
		UncompressCost:          DefaultPerByteUncompressCost(),
	}
}

// WasmGasRegister implements GasRegister interface
type WasmGasRegister struct {
	c WasmGasRegisterConfig
}

// NewDefaultWasmGasRegister creates instance with default values
func NewDefaultWasmGasRegister() WasmGasRegister {
	return NewWasmGasRegister(DefaultGasRegisterConfig())
}

// NewWasmGasRegister constructor
func NewWasmGasRegister(c WasmGasRegisterConfig) WasmGasRegister {
	if c.GasMultiplier == 0 {
		panic(sdkerrors.Wrap(sdkerrors.ErrLogic, "GasMultiplier can not be 0"))
	}
	return WasmGasRegister{
		c: c,
	}
}

// NewContractInstanceCosts costs to create a new contract instance from code
func (g WasmGasRegister) NewContractInstanceCosts(msgLen int) storetypes.Gas {
	return g.InstantiateContractCosts(msgLen)
}

// CompileCosts costs to persist and "compile" a new wasm contract
func (g WasmGasRegister) CompileCosts(byteLength int) storetypes.Gas {
	if byteLength < 0 {
		panic(sdkerrors.Wrap(ErrInvalid, "negative length"))
	}
	return g.c.CompileCost * uint64(byteLength)
}

// UncompressCosts costs to unpack a new wasm contract
func (g WasmGasRegister) UncompressCosts(byteLength int) sdk.Gas {
	if byteLength < 0 {
		panic(sdkerrors.Wrap(ErrInvalid, "negative length"))
	}
	return g.c.UncompressCost.Mul(uint64(byteLength)).Floor()
}

// InstantiateContractCosts costs when interacting with a wasm contract
func (g WasmGasRegister) InstantiateContractCosts(msgLen int) sdk.Gas {
	if msgLen < 0 {
		panic(sdkerrors.Wrap(ErrInvalid, "negative length"))
	}
	dataCosts := sdk.Gas(msgLen) * g.c.ContractMessageDataCost
	return g.c.InstanceCost + dataCosts
}

// ToWasmVMGas convert to wasmVM contract runtime gas unit
func (g WasmGasRegister) ToWasmVMGas(source storetypes.Gas) uint64 {
	x := source * g.c.GasMultiplier
	if x < source {
		panic(sdk.ErrorOutOfGas{Descriptor: "overflow"})
	}
	return x
}

// FromWasmVMGas converts to SDK gas unit
func (g WasmGasRegister) FromWasmVMGas(source uint64) sdk.Gas {
	return source / g.c.GasMultiplier
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
		panic(sdk.ErrorOutOfGas{Descriptor: "Wasmer function execution"})
	}
}

// MultipliedGasMeter wraps the GasMeter from context and multiplies all reads by out defined multiplier
type MultipliedGasMeter struct {
	originalMeter sdk.GasMeter
	GasRegister   GasRegister
}

func NewMultipliedGasMeter(originalMeter sdk.GasMeter, gr GasRegister) MultipliedGasMeter {
	return MultipliedGasMeter{originalMeter: originalMeter, GasRegister: gr}
}

var _ wasmvm.GasMeter = MultipliedGasMeter{}

func (m MultipliedGasMeter) GasConsumed() sdk.Gas {
	return m.GasRegister.ToWasmVMGas(m.originalMeter.GasConsumed())
}
