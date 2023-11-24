package wasmtesting

import (
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	storetypes "cosmossdk.io/store/types"
)

// MockGasRegister mock that implements keeper.GasRegister
type MockGasRegister struct {
	CompileCostFn             func(byteLength int) storetypes.Gas
	NewContractInstanceCostFn func(pinned bool, msgLen int) storetypes.Gas
	InstantiateContractCostFn func(pinned bool, msgLen int) storetypes.Gas
	ReplyCostFn               func(pinned bool, reply wasmvmtypes.Reply) storetypes.Gas
	EventCostsFn              func(evts []wasmvmtypes.EventAttribute) storetypes.Gas
	ToWasmVMGasFn             func(source storetypes.Gas) uint64
	FromWasmVMGasFn           func(source uint64) storetypes.Gas
	UncompressCostsFn         func(byteLength int) storetypes.Gas
}

func (m MockGasRegister) NewContractInstanceCosts(pinned bool, msgLen int) storetypes.Gas {
	if m.NewContractInstanceCostFn == nil {
		panic("not expected to be called")
	}
	return m.NewContractInstanceCostFn(pinned, msgLen)
}

func (m MockGasRegister) CompileCosts(byteLength int) storetypes.Gas {
	if m.CompileCostFn == nil {
		panic("not expected to be called")
	}
	return m.CompileCostFn(byteLength)
}

func (m MockGasRegister) UncompressCosts(byteLength int) storetypes.Gas {
	if m.UncompressCostsFn == nil {
		panic("not expected to be called")
	}
	return m.UncompressCostsFn(byteLength)
}

func (m MockGasRegister) InstantiateContractCosts(pinned bool, msgLen int) storetypes.Gas {
	if m.InstantiateContractCostFn == nil {
		panic("not expected to be called")
	}
	return m.InstantiateContractCostFn(pinned, msgLen)
}

func (m MockGasRegister) ReplyCosts(pinned bool, reply wasmvmtypes.Reply) storetypes.Gas {
	if m.ReplyCostFn == nil {
		panic("not expected to be called")
	}
	return m.ReplyCostFn(pinned, reply)
}

func (m MockGasRegister) EventCosts(evts []wasmvmtypes.EventAttribute, _ wasmvmtypes.Events) storetypes.Gas {
	if m.EventCostsFn == nil {
		panic("not expected to be called")
	}
	return m.EventCostsFn(evts)
}

func (m MockGasRegister) ToWasmVMGas(source storetypes.Gas) uint64 {
	if m.ToWasmVMGasFn == nil {
		panic("not expected to be called")
	}
	return m.ToWasmVMGasFn(source)
}

func (m MockGasRegister) FromWasmVMGas(source uint64) storetypes.Gas {
	if m.FromWasmVMGasFn == nil {
		panic("not expected to be called")
	}
	return m.FromWasmVMGasFn(source)
}
