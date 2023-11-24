package feegrant

import (
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/feegrant interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	legacy.RegisterAminoMsg(cdc, &MsgGrantAllowance{}, "cosmos-sdk/MsgGrantAllowance")
	legacy.RegisterAminoMsg(cdc, &MsgRevokeAllowance{}, "cosmos-sdk/MsgRevokeAllowance")

	cdc.RegisterInterface((*FeeAllowanceI)(nil), nil)
	cdc.RegisterConcrete(&BasicAllowance{}, "cosmos-sdk/BasicAllowance", nil)
	cdc.RegisterConcrete(&PeriodicAllowance{}, "cosmos-sdk/PeriodicAllowance", nil)
	cdc.RegisterConcrete(&AllowedMsgAllowance{}, "cosmos-sdk/AllowedMsgAllowance", nil)
}

// RegisterInterfaces registers the interfaces types with the interface registry
func RegisterInterfaces(registry types.InterfaceRegistry) {
	registry.RegisterImplementations((*sdk.Msg)(nil),
		&MsgGrantAllowance{},
		&MsgRevokeAllowance{},
	)

	registry.RegisterInterface(
		"cosmos.feegrant.v1beta1.FeeAllowanceI",
		(*FeeAllowanceI)(nil),
		&BasicAllowance{},
		&PeriodicAllowance{},
		&AllowedMsgAllowance{},
	)

	msgservice.RegisterMsgServiceDesc(registry, &_Msg_serviceDesc)
}
