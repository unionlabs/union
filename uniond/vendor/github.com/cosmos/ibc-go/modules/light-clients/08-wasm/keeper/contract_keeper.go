package keeper

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"

	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"

	internaltypes "github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/types"
	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	host "github.com/cosmos/ibc-go/v8/modules/core/24-host"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var (
	VMGasRegister = types.NewDefaultWasmGasRegister()
	// wasmvmAPI is a wasmvm.GoAPI implementation that is passed to the wasmvm, it
	// doesn't implement any functionality, directly returning an error.
	wasmvmAPI = wasmvm.GoAPI{
		HumanizeAddress:     humanizeAddress,
		CanonicalizeAddress: canonicalizeAddress,
		ValidateAddress:     validateAddress,
	}
)

// instantiateContract calls vm.Instantiate with appropriate arguments.
func (k Keeper) instantiateContract(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, checksum types.Checksum, msg []byte) (*wasmvmtypes.ContractResult, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := types.NewMultipliedGasMeter(sdkGasMeter, types.VMGasRegister)
	gasLimit := VMGasRegister.RuntimeGasForContract(ctx)

	env := getEnv(ctx, clientID)

	msgInfo := wasmvmtypes.MessageInfo{
		Sender: "",
		Funds:  nil,
	}

	ctx.GasMeter().ConsumeGas(types.VMGasRegister.SetupContractCost(true, len(msg)), "Loading CosmWasm module: instantiate")
	resp, gasUsed, err := k.GetVM().Instantiate(checksum, env, msgInfo, msg, internaltypes.NewStoreAdapter(clientStore), wasmvmAPI, k.newQueryHandler(ctx, clientID), multipliedGasMeter, gasLimit, types.CostJSONDeserialization)
	types.VMGasRegister.ConsumeRuntimeGas(ctx, gasUsed)
	return resp, err
}

// callContract calls vm.Sudo with internally constructed gas meter and environment.
func (k Keeper) callContract(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, checksum types.Checksum, msg []byte) (*wasmvmtypes.ContractResult, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := types.NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.RuntimeGasForContract(ctx)

	env := getEnv(ctx, clientID)

	ctx.GasMeter().ConsumeGas(VMGasRegister.SetupContractCost(true, len(msg)), "Loading CosmWasm module: sudo")
	resp, gasUsed, err := k.GetVM().Sudo(checksum, env, msg, internaltypes.NewStoreAdapter(clientStore), wasmvmAPI, k.newQueryHandler(ctx, clientID), multipliedGasMeter, gasLimit, types.CostJSONDeserialization)
	VMGasRegister.ConsumeRuntimeGas(ctx, gasUsed)
	return resp, err
}

// queryContract calls vm.Query.
func (k Keeper) queryContract(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, checksum types.Checksum, msg []byte) (*wasmvmtypes.QueryResult, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := types.NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.RuntimeGasForContract(ctx)

	env := getEnv(ctx, clientID)

	ctx.GasMeter().ConsumeGas(VMGasRegister.SetupContractCost(true, len(msg)), "Loading CosmWasm module: query")
	resp, gasUsed, err := k.GetVM().Query(checksum, env, msg, internaltypes.NewStoreAdapter(clientStore), wasmvmAPI, k.newQueryHandler(ctx, clientID), multipliedGasMeter, gasLimit, types.CostJSONDeserialization)
	VMGasRegister.ConsumeRuntimeGas(ctx, gasUsed)

	return resp, err
}

// migrateContract calls vm.Migrate with internally constructed gas meter and environment.
func (k Keeper) migrateContract(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, checksum types.Checksum, msg []byte) (*wasmvmtypes.ContractResult, error) {
	sdkGasMeter := ctx.GasMeter()
	multipliedGasMeter := types.NewMultipliedGasMeter(sdkGasMeter, VMGasRegister)
	gasLimit := VMGasRegister.RuntimeGasForContract(ctx)

	env := getEnv(ctx, clientID)

	ctx.GasMeter().ConsumeGas(VMGasRegister.SetupContractCost(true, len(msg)), "Loading CosmWasm module: migrate")
	resp, gasUsed, err := k.GetVM().Migrate(checksum, env, msg, internaltypes.NewStoreAdapter(clientStore), wasmvmAPI, k.newQueryHandler(ctx, clientID), multipliedGasMeter, gasLimit, types.CostJSONDeserialization)
	VMGasRegister.ConsumeRuntimeGas(ctx, gasUsed)

	return resp, err
}

// WasmInstantiate accepts a message to instantiate a wasm contract, JSON encodes it and calls instantiateContract.
func (k Keeper) WasmInstantiate(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, cs *types.ClientState, payload types.InstantiateMessage) error {
	encodedData, err := json.Marshal(payload)
	if err != nil {
		return errorsmod.Wrap(err, "failed to marshal payload for wasm contract instantiation")
	}

	checksum := cs.Checksum
	res, err := k.instantiateContract(ctx, clientID, clientStore, checksum, encodedData)
	if err != nil {
		return errorsmod.Wrap(types.ErrVMError, err.Error())
	}
	if res.Err != "" {
		return errorsmod.Wrap(types.ErrWasmContractCallFailed, res.Err)
	}

	if err = checkResponse(res.Ok); err != nil {
		return errorsmod.Wrapf(err, "checksum (%s)", hex.EncodeToString(cs.Checksum))
	}

	newClientState, err := validatePostExecutionClientState(clientStore, k.Codec())
	if err != nil {
		return err
	}

	// Checksum should only be able to be modified during migration.
	if !bytes.Equal(checksum, newClientState.Checksum) {
		return errorsmod.Wrapf(types.ErrWasmInvalidContractModification, "expected checksum %s, got %s", hex.EncodeToString(checksum), hex.EncodeToString(newClientState.Checksum))
	}

	return nil
}

// WasmSudo calls the contract with the given payload and returns the result.
// WasmSudo returns an error if:
// - the contract call returns an error
// - the response of the contract call contains non-empty messages
// - the response of the contract call contains non-empty events
// - the response of the contract call contains non-empty attributes
// - the data bytes of the response cannot be unmarshaled into the result type
func (k Keeper) WasmSudo(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, cs *types.ClientState, payload types.SudoMsg) ([]byte, error) {
	encodedData, err := json.Marshal(payload)
	if err != nil {
		return nil, errorsmod.Wrap(err, "failed to marshal payload for wasm execution")
	}

	checksum := cs.Checksum
	res, err := k.callContract(ctx, clientID, clientStore, checksum, encodedData)
	if err != nil {
		return nil, errorsmod.Wrap(types.ErrVMError, err.Error())
	}
	if res.Err != "" {
		return nil, errorsmod.Wrap(types.ErrWasmContractCallFailed, res.Err)
	}

	if err = checkResponse(res.Ok); err != nil {
		return nil, errorsmod.Wrapf(err, "checksum (%s)", hex.EncodeToString(cs.Checksum))
	}

	newClientState, err := validatePostExecutionClientState(clientStore, k.Codec())
	if err != nil {
		return nil, err
	}

	// Checksum should only be able to be modified during migration.
	if !bytes.Equal(checksum, newClientState.Checksum) {
		return nil, errorsmod.Wrapf(types.ErrWasmInvalidContractModification, "expected checksum %s, got %s", hex.EncodeToString(checksum), hex.EncodeToString(newClientState.Checksum))
	}

	return res.Ok.Data, nil
}

// WasmMigrate migrate calls the migrate entry point of the contract with the given payload and returns the result.
// WasmMigrate returns an error if:
// - the contract migration returns an error
func (k Keeper) WasmMigrate(ctx sdk.Context, clientStore storetypes.KVStore, cs *types.ClientState, clientID string, payload []byte) error {
	res, err := k.migrateContract(ctx, clientID, clientStore, cs.Checksum, payload)
	if err != nil {
		return errorsmod.Wrap(types.ErrVMError, err.Error())
	}
	if res.Err != "" {
		return errorsmod.Wrap(types.ErrWasmContractCallFailed, res.Err)
	}

	if err = checkResponse(res.Ok); err != nil {
		return errorsmod.Wrapf(err, "checksum (%s)", hex.EncodeToString(cs.Checksum))
	}

	_, err = validatePostExecutionClientState(clientStore, k.cdc)
	return err
}

// WasmQuery queries the contract with the given payload and returns the result.
// WasmQuery returns an error if:
// - the contract query returns an error
// - the data bytes of the response cannot be unmarshal into the result type
func (k Keeper) WasmQuery(ctx sdk.Context, clientID string, clientStore storetypes.KVStore, cs *types.ClientState, payload types.QueryMsg) ([]byte, error) {
	encodedData, err := json.Marshal(payload)
	if err != nil {
		return nil, errorsmod.Wrap(err, "failed to marshal payload for wasm query")
	}

	res, err := k.queryContract(ctx, clientID, clientStore, cs.Checksum, encodedData)
	if err != nil {
		return nil, errorsmod.Wrap(types.ErrVMError, err.Error())
	}
	if res.Err != "" {
		return nil, errorsmod.Wrap(types.ErrWasmContractCallFailed, res.Err)
	}

	return res.Ok, nil
}

// validatePostExecutionClientState validates that the contract has not many any invalid modifications
// to the client state during execution. It ensures that
// - the client state is still present
// - the client state can be unmarshaled successfully.
// - the client state is of type *ClientState
func validatePostExecutionClientState(clientStore storetypes.KVStore, cdc codec.BinaryCodec) (*types.ClientState, error) {
	key := host.ClientStateKey()
	_, ok := clientStore.(internaltypes.ClientRecoveryStore)
	if ok {
		key = append(internaltypes.SubjectPrefix, key...)
	}

	bz := clientStore.Get(key)
	if len(bz) == 0 {
		return nil, errorsmod.Wrap(types.ErrWasmInvalidContractModification, clienttypes.ErrClientNotFound.Error())
	}

	clientState, err := unmarshalClientState(cdc, bz)
	if err != nil {
		return nil, errorsmod.Wrap(types.ErrWasmInvalidContractModification, err.Error())
	}

	cs, ok := clientState.(*types.ClientState)
	if !ok {
		return nil, errorsmod.Wrapf(types.ErrWasmInvalidContractModification, "expected client state type %T, got %T", (*types.ClientState)(nil), clientState)
	}

	return cs, nil
}

// unmarshalClientState unmarshals the client state from the given bytes.
func unmarshalClientState(cdc codec.BinaryCodec, bz []byte) (exported.ClientState, error) {
	var clientState exported.ClientState
	if err := cdc.UnmarshalInterface(bz, &clientState); err != nil {
		return nil, err
	}

	return clientState, nil
}

// getEnv returns the state of the blockchain environment the contract is running on
func getEnv(ctx sdk.Context, contractAddr string) wasmvmtypes.Env {
	chainID := ctx.BlockHeader().ChainID
	height := ctx.BlockHeader().Height

	// safety checks before casting below
	if height < 0 {
		panic(errors.New("block height must never be negative"))
	}
	nsec := ctx.BlockTime().UnixNano()
	if nsec < 0 {
		panic(errors.New("block (unix) time must never be negative "))
	}

	env := wasmvmtypes.Env{
		Block: wasmvmtypes.BlockInfo{
			Height:  uint64(height),
			Time:    wasmvmtypes.Uint64(nsec),
			ChainID: chainID,
		},
		Contract: wasmvmtypes.ContractInfo{
			Address: contractAddr,
		},
	}

	return env
}

func humanizeAddress(canon []byte) (string, uint64, error) {
	return "", 0, errors.New("humanizeAddress not implemented")
}

func canonicalizeAddress(human string) ([]byte, uint64, error) {
	return nil, 0, errors.New("canonicalizeAddress not implemented")
}

func validateAddress(human string) (uint64, error) {
	return 0, errors.New("validateAddress not implemented")
}

// checkResponse returns an error if the response from a sudo, instantiate or migrate call
// to the Wasm VM contains messages, events or attributes.
func checkResponse(response *wasmvmtypes.Response) error {
	// Only allow Data to flow back to us. SubMessages, Events and Attributes are not allowed.
	if len(response.Messages) > 0 {
		return types.ErrWasmSubMessagesNotAllowed
	}
	if len(response.Events) > 0 {
		return types.ErrWasmEventsNotAllowed
	}
	if len(response.Attributes) > 0 {
		return types.ErrWasmAttributesNotAllowed
	}

	return nil
}
