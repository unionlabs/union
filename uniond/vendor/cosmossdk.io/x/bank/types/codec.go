package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/bank interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	legacy.RegisterAminoMsg(registrar, &MsgSend{}, "cosmos-sdk/MsgSend")
	legacy.RegisterAminoMsg(registrar, &MsgMultiSend{}, "cosmos-sdk/MsgMultiSend")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateParams{}, "cosmos-sdk/x/bank/MsgUpdateParams")
	legacy.RegisterAminoMsg(registrar, &MsgSetSendEnabled{}, "cosmos-sdk/MsgSetSendEnabled")

	registrar.RegisterConcrete(&SendAuthorization{}, "cosmos-sdk/SendAuthorization")
	registrar.RegisterConcrete(&Params{}, "cosmos-sdk/x/bank/Params")
}

func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgSend{},
		&MsgMultiSend{},
		&MsgUpdateParams{},
		&MsgBurn{},
		&MsgSetSendEnabled{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
