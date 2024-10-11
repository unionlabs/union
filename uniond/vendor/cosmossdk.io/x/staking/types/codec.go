package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/staking interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	legacy.RegisterAminoMsg(registrar, &MsgCreateValidator{}, "cosmos-sdk/MsgCreateValidator")
	legacy.RegisterAminoMsg(registrar, &MsgEditValidator{}, "cosmos-sdk/MsgEditValidator")
	legacy.RegisterAminoMsg(registrar, &MsgDelegate{}, "cosmos-sdk/MsgDelegate")
	legacy.RegisterAminoMsg(registrar, &MsgUndelegate{}, "cosmos-sdk/MsgUndelegate")
	legacy.RegisterAminoMsg(registrar, &MsgBeginRedelegate{}, "cosmos-sdk/MsgBeginRedelegate")
	legacy.RegisterAminoMsg(registrar, &MsgCancelUnbondingDelegation{}, "cosmos-sdk/MsgCancelUnbondingDelegation")
	legacy.RegisterAminoMsg(registrar, &MsgUpdateParams{}, "cosmos-sdk/x/staking/MsgUpdateParams")
	legacy.RegisterAminoMsg(registrar, &MsgRotateConsPubKey{}, "cosmos-sdk/MsgRotateConsPubKey")

	registrar.RegisterInterface((*isStakeAuthorization_Validators)(nil), nil)
	registrar.RegisterConcrete(&StakeAuthorization_AllowList{}, "cosmos-sdk/StakeAuthorization/AllowList")
	registrar.RegisterConcrete(&StakeAuthorization_DenyList{}, "cosmos-sdk/StakeAuthorization/DenyList")
	registrar.RegisterConcrete(&StakeAuthorization{}, "cosmos-sdk/StakeAuthorization")
	registrar.RegisterConcrete(Params{}, "cosmos-sdk/x/staking/Params")
}

// RegisterInterfaces registers the x/staking interfaces types with the interface registry
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgCreateValidator{},
		&MsgEditValidator{},
		&MsgDelegate{},
		&MsgUndelegate{},
		&MsgBeginRedelegate{},
		&MsgCancelUnbondingDelegation{},
		&MsgUpdateParams{},
	)

	msgservice.RegisterMsgServiceDesc(registrar, &_Msg_serviceDesc)
}
