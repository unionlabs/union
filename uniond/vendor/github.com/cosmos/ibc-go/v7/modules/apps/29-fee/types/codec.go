package types

import (
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/ibc 29-fee interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	cdc.RegisterConcrete(&MsgPayPacketFee{}, "cosmos-sdk/MsgPayPacketFee", nil)
	cdc.RegisterConcrete(&MsgPayPacketFeeAsync{}, "cosmos-sdk/MsgPayPacketFeeAsync", nil)
	cdc.RegisterConcrete(&MsgRegisterPayee{}, "cosmos-sdk/MsgRegisterPayee", nil)
	cdc.RegisterConcrete(&MsgRegisterCounterpartyPayee{}, "cosmos-sdk/MsgRegisterCounterpartyPayee", nil)
}

// RegisterInterfaces register the 29-fee module interfaces to protobuf
// Any.
func RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	registry.RegisterImplementations(
		(*sdk.Msg)(nil),
		&MsgPayPacketFee{},
		&MsgPayPacketFeeAsync{},
		&MsgRegisterPayee{},
		&MsgRegisterCounterpartyPayee{},
	)

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}

var (
	amino = codec.NewLegacyAmino()

	// ModuleCdc references the global x/ibc-transfer module codec. Note, the codec
	// should ONLY be used in certain instances of tests and for JSON encoding.
	//
	// The actual codec used for serialization should be provided to x/ibc transfer and
	// defined at the application level.
	ModuleCdc = codec.NewProtoCodec(codectypes.NewInterfaceRegistry())

	// AminoCdc is a amino codec created to support amino json compatible msgs.
	AminoCdc = codec.NewAminoCodec(amino)
)

func init() {
	RegisterLegacyAminoCodec(amino)
	amino.Seal()
}
