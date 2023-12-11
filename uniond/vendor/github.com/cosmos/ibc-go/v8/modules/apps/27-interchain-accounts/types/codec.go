package types

import (
	"github.com/cosmos/gogoproto/proto"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

// ModuleCdc references the global interchain accounts module codec. Note, the codec
// should ONLY be used in certain instances of tests and for JSON encoding.
//
// The actual codec used for serialization should be provided to interchain accounts and
// defined at the application level.
var ModuleCdc = codec.NewProtoCodec(codectypes.NewInterfaceRegistry())

// RegisterInterfaces registers the interchain accounts controller types and the concrete InterchainAccount implementation
// against the associated x/auth AccountI and GenesisAccount interfaces.
func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.AccountI)(nil), &InterchainAccount{})
	registry.RegisterImplementations((*authtypes.GenesisAccount)(nil), &InterchainAccount{})
}

// SerializeCosmosTx serializes a slice of sdk.Msg's using the CosmosTx type. The sdk.Msg's are
// packed into Any's and inserted into the Messages field of a CosmosTx. The CosmosTx is marshaled
// depending on the encoding type passed in. The marshaled bytes are returned. Only the ProtoCodec
// is supported for serializing messages. Both protobuf and proto3 JSON are supported.
func SerializeCosmosTx(cdc codec.Codec, msgs []proto.Message, encoding string) ([]byte, error) {
	// this is a defensive check to ensure only the ProtoCodec is used for message serialization
	if _, ok := cdc.(*codec.ProtoCodec); !ok {
		return nil, errorsmod.Wrap(ErrInvalidCodec, "only the ProtoCodec may be used for receiving messages on the host chain")
	}

	var bz []byte
	var err error

	msgAnys := make([]*codectypes.Any, len(msgs))

	for i, msg := range msgs {
		msgAnys[i], err = codectypes.NewAnyWithValue(msg)
		if err != nil {
			return nil, err
		}
	}

	cosmosTx := &CosmosTx{
		Messages: msgAnys,
	}

	switch encoding {
	case EncodingProtobuf:
		bz, err = cdc.Marshal(cosmosTx)
		if err != nil {
			return nil, errorsmod.Wrapf(err, "cannot marshal CosmosTx with protobuf")
		}
	case EncodingProto3JSON:
		bz, err = cdc.MarshalJSON(cosmosTx)
		if err != nil {
			return nil, errorsmod.Wrapf(ErrUnknownDataType, "cannot marshal CosmosTx with proto3 json")
		}
	default:
		return nil, errorsmod.Wrapf(ErrInvalidCodec, "unsupported encoding format %s", encoding)
	}

	return bz, nil
}

// DeserializeCosmosTx unmarshals and unpacks a slice of transaction bytes into a slice of sdk.Msg's.
// The transaction bytes are unmarshaled depending on the encoding type passed in. The sdk.Msg's are
// unpacked from Any's and returned. Only the ProtoCodec is supported for serializing messages. Both
// protobuf and proto3 JSON are supported.
func DeserializeCosmosTx(cdc codec.Codec, data []byte, encoding string) ([]sdk.Msg, error) {
	// this is a defensive check to ensure only the ProtoCodec is used for message deserialization
	if _, ok := cdc.(*codec.ProtoCodec); !ok {
		return nil, errorsmod.Wrap(ErrInvalidCodec, "only the ProtoCodec may be used for receiving messages on the host chain")
	}

	var cosmosTx CosmosTx

	switch encoding {
	case EncodingProtobuf:
		if err := cdc.Unmarshal(data, &cosmosTx); err != nil {
			return nil, errorsmod.Wrapf(err, "cannot unmarshal CosmosTx with protobuf")
		}
	case EncodingProto3JSON:
		if err := cdc.UnmarshalJSON(data, &cosmosTx); err != nil {
			return nil, errorsmod.Wrapf(ErrUnknownDataType, "cannot unmarshal CosmosTx with proto3 json")
		}
	default:
		return nil, errorsmod.Wrapf(ErrInvalidCodec, "unsupported encoding format %s", encoding)
	}

	msgs := make([]sdk.Msg, len(cosmosTx.Messages))

	for i, protoAny := range cosmosTx.Messages {
		var msg sdk.Msg
		err := cdc.UnpackAny(protoAny, &msg)
		if err != nil {
			return nil, err
		}
		msgs[i] = msg
	}

	return msgs, nil
}
