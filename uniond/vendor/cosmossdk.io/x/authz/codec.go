package authz

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"
	bank "cosmossdk.io/x/bank/types"
	staking "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

// RegisterLegacyAminoCodec registers the necessary x/authz interfaces and concrete types
// on the provided LegacyAmino codec. These types are used for Amino JSON serialization.
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	legacy.RegisterAminoMsg(registrar, &MsgGrant{}, "cosmos-sdk/MsgGrant")
	legacy.RegisterAminoMsg(registrar, &MsgRevoke{}, "cosmos-sdk/MsgRevoke")
	legacy.RegisterAminoMsg(registrar, &MsgExec{}, "cosmos-sdk/MsgExec")

	registrar.RegisterInterface((*Authorization)(nil), nil)
	registrar.RegisterConcrete(&GenericAuthorization{}, "cosmos-sdk/GenericAuthorization")
}

// RegisterInterfaces registers the interfaces types with the interface registry
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgGrant{},
		&MsgRevoke{},
		&MsgExec{},
	)

	// since bank.SendAuthorization and staking.StakeAuthorization both implement Authorization
	// and authz depends on x/bank and x/staking in other places, these registrations are placed here
	// to prevent a cyclic dependency.
	// see: https://github.com/cosmos/cosmos-sdk/pull/16509
	registrar.RegisterInterface(
		"cosmos.authz.v1beta1.Authorization",
		(*Authorization)(nil),
		&GenericAuthorization{},
		&bank.SendAuthorization{},
		&staking.StakeAuthorization{},
	)
	msgservice.RegisterMsgServiceDesc(registrar, MsgServiceDesc())
}
