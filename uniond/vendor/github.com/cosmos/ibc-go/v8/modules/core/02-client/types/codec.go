package types

import (
	proto "github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
	govtypesv1beta1 "github.com/cosmos/cosmos-sdk/x/gov/types/v1beta1"

	ibcerrors "github.com/cosmos/ibc-go/v8/modules/core/errors"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

// RegisterInterfaces registers the client interfaces to protobuf Any.
func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterInterface(
		"ibc.core.client.v1.ClientState",
		(*exported.ClientState)(nil),
	)
	registry.RegisterInterface(
		"ibc.core.client.v1.ConsensusState",
		(*exported.ConsensusState)(nil),
	)
	registry.RegisterInterface(
		"ibc.core.client.v1.Header",
		(*exported.ClientMessage)(nil),
	)
	registry.RegisterInterface(
		"ibc.core.client.v1.Height",
		(*exported.Height)(nil),
		&Height{},
	)
	registry.RegisterInterface(
		"ibc.core.client.v1.Misbehaviour",
		(*exported.ClientMessage)(nil),
	)
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgCreateClient{},
		&MsgUpdateClient{},
		&MsgUpgradeClient{},
		&MsgSubmitMisbehaviour{},
		&MsgRecoverClient{},
		&MsgIBCSoftwareUpgrade{},
		&MsgUpdateParams{},
	)
	registry.RegisterImplementations(
		(*govtypesv1beta1.Content)(nil),
		&ClientUpdateProposal{},
		&UpgradeProposal{},
	)

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}

// PackClientState constructs a new Any packed with the given client state value. It returns
// an error if the client state can't be casted to a protobuf message or if the concrete
// implemention is not registered to the protobuf codec.
func PackClientState(clientState exported.ClientState) (*codectypes.Any, error) {
	msg, ok := clientState.(proto.Message)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrPackAny, "cannot proto marshal %T", clientState)
	}

	anyClientState, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrPackAny, err.Error())
	}

	return anyClientState, nil
}

// UnpackClientState unpacks an Any into a ClientState. It returns an error if the
// client state can't be unpacked into a ClientState.
func UnpackClientState(any *codectypes.Any) (exported.ClientState, error) {
	if any == nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrUnpackAny, "protobuf Any message cannot be nil")
	}

	clientState, ok := any.GetCachedValue().(exported.ClientState)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnpackAny, "cannot unpack Any into ClientState %T", any)
	}

	return clientState, nil
}

// PackConsensusState constructs a new Any packed with the given consensus state value. It returns
// an error if the consensus state can't be casted to a protobuf message or if the concrete
// implemention is not registered to the protobuf codec.
func PackConsensusState(consensusState exported.ConsensusState) (*codectypes.Any, error) {
	msg, ok := consensusState.(proto.Message)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrPackAny, "cannot proto marshal %T", consensusState)
	}

	anyConsensusState, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrPackAny, err.Error())
	}

	return anyConsensusState, nil
}

// MustPackConsensusState calls PackConsensusState and panics on error.
func MustPackConsensusState(consensusState exported.ConsensusState) *codectypes.Any {
	anyConsensusState, err := PackConsensusState(consensusState)
	if err != nil {
		panic(err)
	}

	return anyConsensusState
}

// UnpackConsensusState unpacks an Any into a ConsensusState. It returns an error if the
// consensus state can't be unpacked into a ConsensusState.
func UnpackConsensusState(any *codectypes.Any) (exported.ConsensusState, error) {
	if any == nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrUnpackAny, "protobuf Any message cannot be nil")
	}

	consensusState, ok := any.GetCachedValue().(exported.ConsensusState)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnpackAny, "cannot unpack Any into ConsensusState %T", any)
	}

	return consensusState, nil
}

// PackClientMessage constructs a new Any packed with the given value. It returns
// an error if the value can't be casted to a protobuf message or if the concrete
// implemention is not registered to the protobuf codec.
func PackClientMessage(clientMessage exported.ClientMessage) (*codectypes.Any, error) {
	msg, ok := clientMessage.(proto.Message)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrPackAny, "cannot proto marshal %T", clientMessage)
	}

	protoAny, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrPackAny, err.Error())
	}

	return protoAny, nil
}

// UnpackClientMessage unpacks an Any into a ClientMessage. It returns an error if the
// consensus state can't be unpacked into a ClientMessage.
func UnpackClientMessage(any *codectypes.Any) (exported.ClientMessage, error) {
	if any == nil {
		return nil, errorsmod.Wrap(ibcerrors.ErrUnpackAny, "protobuf Any message cannot be nil")
	}

	clientMessage, ok := any.GetCachedValue().(exported.ClientMessage)
	if !ok {
		return nil, errorsmod.Wrapf(ibcerrors.ErrUnpackAny, "cannot unpack Any into Header %T", any)
	}

	return clientMessage, nil
}
