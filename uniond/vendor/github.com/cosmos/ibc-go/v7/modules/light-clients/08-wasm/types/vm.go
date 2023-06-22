package types

import (
	cosmwasm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	clienttypes "github.com/cosmos/ibc-go/v7/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v7/modules/core/exported"
)

var (
	Querier       cosmwasm.Querier
	WasmVM        *cosmwasm.VM
	VMGasRegister = NewDefaultWasmGasRegister()
)

type queryResponse struct {
	Status          exported.Status               `json:"status,omitempty"`
	GenesisMetadata []clienttypes.GenesisMetadata `json:"genesis_metadata,omitempty"`
}

type ContractResult interface {
	Validate() bool
	Error() string
}

type contractResult struct {
	IsValid           bool   `json:"is_valid,omitempty"`
	ErrorMsg          string `json:"error_msg,omitempty"`
	Data              []byte `json:"data,omitempty"`
	FoundMisbehaviour bool   `json:"found_misbehaviour"`
}

func (r contractResult) Validate() bool {
	return r.IsValid
}

func (r contractResult) Error() string {
	return r.ErrorMsg
}

// initContract calls vm.Init with appropriate arguments.
func initContract(codeID []byte, ctx sdk.Context, clientStore sdk.KVStore) (*wasmvmtypes.Response, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.runtimeGasForContract(ctx)
	chainID := ctx.BlockHeader().ChainID
	height := ctx.BlockHeader().Height
	// safety checks before casting below
	if height < 0 {
		panic("Block height must never be negative")
	}
	nsec := ctx.BlockTime().UnixNano()
	if nsec < 0 {
		panic("Block (unix) time must never be negative ")
	}
	env := wasmvmtypes.Env{
		Block: wasmvmtypes.BlockInfo{
			Height:  uint64(height),
			Time:    uint64(nsec),
			ChainID: chainID,
		},
		Contract: wasmvmtypes.ContractInfo{
			Address: "",
		},
	}

	msgInfo := wasmvmtypes.MessageInfo{
		Sender: "",
		Funds:  nil,
	}

	initMsg := []byte("{}")
	ctx.GasMeter().ConsumeGas(VMGasRegister.NewContractInstanceCosts(len(initMsg)), "Loading CosmWasm module: instantiate")
	response, gasUsed, err := WasmVM.Instantiate(codeID, env, msgInfo, initMsg, newStoreAdapter(clientStore), cosmwasm.GoAPI{}, Querier, multipliedGasMeter, gasLimit, costJSONDeserialization)
	VMGasRegister.consumeRuntimeGas(ctx, gasUsed)
	return response, err
}

// Calls vm.Execute with internally constructed Gas meter and environment.
func callContract(codeID []byte, ctx sdk.Context, clientStore sdk.KVStore, msg []byte) (*wasmvmtypes.Response, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.runtimeGasForContract(ctx)
	chainID := ctx.BlockHeader().ChainID
	height := ctx.BlockHeader().Height
	// safety checks before casting below
	if height < 0 {
		panic("Block height must never be negative")
	}
	nsec := ctx.BlockTime().UnixNano()
	if nsec < 0 {
		panic("Block (unix) time must never be negative ")
	}
	env := wasmvmtypes.Env{
		Block: wasmvmtypes.BlockInfo{
			Height:  uint64(height),
			Time:    uint64(nsec),
			ChainID: chainID,
		},
		Contract: wasmvmtypes.ContractInfo{
			Address: "",
		},
	}

	msgInfo := wasmvmtypes.MessageInfo{
		Sender: "",
		Funds:  nil,
	}
	ctx.GasMeter().ConsumeGas(VMGasRegister.InstantiateContractCosts(len(msg)), "Loading CosmWasm module: execute")
	resp, gasUsed, err := WasmVM.Execute(codeID, env, msgInfo, msg, newStoreAdapter(clientStore), cosmwasm.GoAPI{}, Querier, multipliedGasMeter, gasLimit, costJSONDeserialization)
	VMGasRegister.consumeRuntimeGas(ctx, gasUsed)
	return resp, err
}

// queryContractWithStore calls vm.Query.
func queryContractWithStore(ctx sdk.Context, clientStore sdk.KVStore, codeID cosmwasm.Checksum, msg []byte) ([]byte, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.runtimeGasForContract(ctx)
	chainID := ctx.BlockHeader().ChainID
	height := ctx.BlockHeader().Height
	// safety checks before casting below
	if height < 0 {
		panic("Block height must never be negative")
	}
	nsec := ctx.BlockTime().UnixNano()
	if nsec < 0 {
		panic("Block (unix) time must never be negative ")
	}
	env := wasmvmtypes.Env{
		Block: wasmvmtypes.BlockInfo{
			Height:  uint64(height),
			Time:    uint64(nsec),
			ChainID: chainID,
		},
		Contract: wasmvmtypes.ContractInfo{
			Address: "",
		},
	}
	ctx.GasMeter().ConsumeGas(VMGasRegister.InstantiateContractCosts(len(msg)), "Loading CosmWasm module: query")
	resp, gasUsed, err := WasmVM.Query(codeID, env, msg, newStoreAdapter(clientStore), cosmwasm.GoAPI{}, Querier, multipliedGasMeter, gasLimit, costJSONDeserialization)
	VMGasRegister.consumeRuntimeGas(ctx, gasUsed)
	return resp, err
}
