package solomachine

import (
	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"

	"github.com/cosmos/ibc-go/v8/modules/core/exported"

	"cosmossdk.io/core/registry"
)

// RegisterInterfaces register the ibc channel submodule interfaces to protobuf
// Any.
func RegisterInterfaces(registry registry.InterfaceRegistrar) {
	registry.RegisterImplementations(
		(*exported.ClientState)(nil),
		&ClientState{},
	)
	registry.RegisterImplementations(
		(*exported.ConsensusState)(nil),
		&ConsensusState{},
	)
	registry.RegisterImplementations(
		(*exported.ClientMessage)(nil),
		&Header{},
	)
	registry.RegisterImplementations(
		(*exported.ClientMessage)(nil),
		&Misbehaviour{},
	)
}

func UnmarshalSignatureData(cdc codec.BinaryCodec, data []byte) (signing.SignatureData, error) {
	protoSigData := &signing.SignatureDescriptor_Data{}
	if err := cdc.Unmarshal(data, protoSigData); err != nil {
		return nil, errorsmod.Wrapf(err, "failed to unmarshal proof into type %T", protoSigData)
	}

	sigData := signing.SignatureDataFromProto(protoSigData)

	return sigData, nil
}
