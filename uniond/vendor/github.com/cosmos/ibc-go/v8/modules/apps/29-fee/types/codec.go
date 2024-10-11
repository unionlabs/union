package types

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/ibc 29-fee interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	legacy.RegisterAminoMsg(cdc, &MsgPayPacketFee{}, "cosmos-sdk/MsgPayPacketFee")
	legacy.RegisterAminoMsg(cdc, &MsgPayPacketFeeAsync{}, "cosmos-sdk/MsgPayPacketFeeAsync")
	legacy.RegisterAminoMsg(cdc, &MsgRegisterPayee{}, "cosmos-sdk/MsgRegisterPayee")
	legacy.RegisterAminoMsg(cdc, &MsgRegisterCounterpartyPayee{}, "cosmos-sdk/MsgRegisterCounterpartyPayee")
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

// ModuleCdc references the global x/ibc 29-fee module codec. Note, the codec
// should ONLY be used in certain instances of tests and for JSON encoding.
//
// The actual codec used for serialization should be provided to x/ibc 29-fee and
// defined at the application level.
var ModuleCdc = codec.NewProtoCodec(codectypes.NewInterfaceRegistry())
