package keeper

import (
	"time"

	channeltypes "github.com/cosmos/ibc-go/v7/modules/core/04-channel/types"
	ibcexported "github.com/cosmos/ibc-go/v7/modules/core/exported"

	errorsmod "cosmossdk.io/errors"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var _ types.IBCContractKeeper = (*Keeper)(nil)

// OnOpenChannel calls the contract to participate in the IBC channel handshake step.
// In the IBC protocol this is either the `Channel Open Init` event on the initiating chain or
// `Channel Open Try` on the counterparty chain.
// Protocol version and channel ordering should be verified for example.
// See https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#channel-lifecycle-management
func (k Keeper) OnOpenChannel(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCChannelOpenMsg,
) (string, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-open-channel")
	_, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return "", err
	}

	env := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCChannelOpen(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return "", errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}
	if res != nil {
		return res.Version, nil
	}
	return "", nil
}

// OnConnectChannel calls the contract to let it know the IBC channel was established.
// In the IBC protocol this is either the `Channel Open Ack` event on the initiating chain or
// `Channel Open Confirm` on the counterparty chain.
//
// There is an open issue with the [cosmos-sdk](https://github.com/cosmos/cosmos-sdk/issues/8334)
// that the counterparty channelID is empty on the initiating chain
// See https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#channel-lifecycle-management
func (k Keeper) OnConnectChannel(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCChannelConnectMsg,
) error {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-connect-channel")
	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return err
	}

	env := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCChannelConnect(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	return k.handleIBCBasicContractResponse(ctx, contractAddr, contractInfo.IBCPortID, res)
}

// OnCloseChannel calls the contract to let it know the IBC channel is closed.
// Calling modules MAY atomically execute appropriate application logic in conjunction with calling chanCloseConfirm.
//
// Once closed, channels cannot be reopened and identifiers cannot be reused. Identifier reuse is prevented because
// we want to prevent potential replay of previously sent packets
// See https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#channel-lifecycle-management
func (k Keeper) OnCloseChannel(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCChannelCloseMsg,
) error {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-close-channel")

	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return err
	}

	params := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCChannelClose(codeInfo.CodeHash, params, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	return k.handleIBCBasicContractResponse(ctx, contractAddr, contractInfo.IBCPortID, res)
}

// OnRecvPacket calls the contract to process the incoming IBC packet. The contract fully owns the data processing and
// returns the acknowledgement data for the chain level. This allows custom applications and protocols on top
// of IBC. Although it is recommended to use the standard acknowledgement envelope defined in
// https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#acknowledgement-envelope
//
// For more information see: https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#packet-flow--handling
func (k Keeper) OnRecvPacket(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCPacketReceiveMsg,
) (ibcexported.Acknowledgement, error) {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-recv-packet")
	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return nil, err
	}

	env := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCPacketReceive(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		panic(execErr) // let the contract fully abort an IBC packet receive.
		// Throwing a panic here instead of an error ack will revert
		// all state downstream and not persist any data in ibc-go.
		// This can be triggered by throwing a panic in the contract
	}
	if res.Err != "" {
		// return error ACK with non-redacted contract message, state will be reverted
		return channeltypes.Acknowledgement{
			Response: &channeltypes.Acknowledgement_Error{Error: res.Err},
		}, nil
	}
	// note submessage reply results can overwrite the `Acknowledgement` data
	data, err := k.handleContractResponse(ctx, contractAddr, contractInfo.IBCPortID, res.Ok.Messages, res.Ok.Attributes, res.Ok.Acknowledgement, res.Ok.Events)
	if err != nil {
		// submessage errors result in error ACK with state reverted. Error message is redacted
		return nil, err
	}
	// success ACK, state will be committed
	return ContractConfirmStateAck(data), nil
}

var _ ibcexported.Acknowledgement = ContractConfirmStateAck{}

type ContractConfirmStateAck []byte

func (w ContractConfirmStateAck) Success() bool {
	return true // always commit state
}

func (w ContractConfirmStateAck) Acknowledgement() []byte {
	return w
}

// OnAckPacket calls the contract to handle the "acknowledgement" data which can contain success or failure of a packet
// acknowledgement written on the receiving chain for example. This is application level data and fully owned by the
// contract. The use of the standard acknowledgement envelope is recommended: https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#acknowledgement-envelope
//
// On application errors the contract can revert an operation like returning tokens as in ibc-transfer.
//
// For more information see: https://github.com/cosmos/ics/tree/master/spec/ics-004-channel-and-packet-semantics#packet-flow--handling
func (k Keeper) OnAckPacket(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCPacketAckMsg,
) error {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-ack-packet")
	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return err
	}

	env := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCPacketAck(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}
	return k.handleIBCBasicContractResponse(ctx, contractAddr, contractInfo.IBCPortID, res)
}

// OnTimeoutPacket calls the contract to let it know the packet was never received on the destination chain within
// the timeout boundaries.
// The contract should handle this on the application level and undo the original operation
func (k Keeper) OnTimeoutPacket(
	ctx sdk.Context,
	contractAddr sdk.AccAddress,
	msg wasmvmtypes.IBCPacketTimeoutMsg,
) error {
	defer telemetry.MeasureSince(time.Now(), "wasm", "contract", "ibc-timeout-packet")

	contractInfo, codeInfo, prefixStore, err := k.contractInstance(ctx, contractAddr)
	if err != nil {
		return err
	}

	env := types.NewEnv(ctx, contractAddr)
	querier := k.newQueryHandler(ctx, contractAddr)

	gas := k.runtimeGasForContract(ctx)
	res, gasUsed, execErr := k.wasmVM.IBCPacketTimeout(codeInfo.CodeHash, env, msg, prefixStore, cosmwasmAPI, querier, ctx.GasMeter(), gas, costJSONDeserialization)
	k.consumeRuntimeGas(ctx, gasUsed)
	if execErr != nil {
		return errorsmod.Wrap(types.ErrExecuteFailed, execErr.Error())
	}

	return k.handleIBCBasicContractResponse(ctx, contractAddr, contractInfo.IBCPortID, res)
}

func (k Keeper) handleIBCBasicContractResponse(ctx sdk.Context, addr sdk.AccAddress, id string, res *wasmvmtypes.IBCBasicResponse) error {
	_, err := k.handleContractResponse(ctx, addr, id, res.Messages, res.Attributes, nil, res.Events)
	return err
}
