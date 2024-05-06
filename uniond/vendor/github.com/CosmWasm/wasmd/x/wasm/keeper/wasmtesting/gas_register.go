package wasmtesting

import (
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	storetypes "cosmossdk.io/store/types"
)

// MockGasRegister mock that implements keeper.GasRegister
type MockGasRegister struct {
	SetupContractCostFn func(discount bool, msgLen int) storetypes.Gas
	ReplyCostFn         func(discount bool, reply wasmvmtypes.Reply) storetypes.Gas
	EventCostsFn        func(evts []wasmvmtypes.EventAttribute) storetypes.Gas
	ToWasmVMGasFn       func(source storetypes.Gas) uint64
	FromWasmVMGasFn     func(source uint64) storetypes.Gas
	UncompressCostsFn   func(byteLength int) storetypes.Gas
}

func (m MockGasRegister) UncompressCosts(byteLength int) storetypes.Gas {
	if m.UncompressCostsFn == nil {
		panic("not expected to be called")
	}
	return m.UncompressCostsFn(byteLength)
}

func (m MockGasRegister) SetupContractCost(discount bool, msgLen int) storetypes.Gas {
	if m.SetupContractCostFn == nil {
		panic("not expected to be called")
	}
	return m.SetupContractCostFn(discount, msgLen)
}

func (m MockGasRegister) ReplyCosts(discount bool, reply wasmvmtypes.Reply) storetypes.Gas {
	if m.ReplyCostFn == nil {
		panic("not expected to be called")
	}
	return m.ReplyCostFn(discount, reply)
}

func (m MockGasRegister) EventCosts(evts []wasmvmtypes.EventAttribute, _ wasmvmtypes.Array[wasmvmtypes.Event]) storetypes.Gas {
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
