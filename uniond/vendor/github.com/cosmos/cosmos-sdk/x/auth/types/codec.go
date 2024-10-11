package types

import (
	"cosmossdk.io/core/registry"
	coretransaction "cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/codec/legacy"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
)

// RegisterLegacyAminoCodec registers the account interfaces and concrete types on the
// provided LegacyAmino codec. These types are used for Amino JSON serialization
func RegisterLegacyAminoCodec(registrar registry.AminoRegistrar) {
	registrar.RegisterInterface((*sdk.ModuleAccountI)(nil), nil)
	registrar.RegisterInterface((*GenesisAccount)(nil), nil)
	registrar.RegisterInterface((*sdk.AccountI)(nil), nil)
	registrar.RegisterConcrete(&BaseAccount{}, "cosmos-sdk/BaseAccount")
	registrar.RegisterConcrete(&ModuleAccount{}, "cosmos-sdk/ModuleAccount")
	registrar.RegisterConcrete(Params{}, "cosmos-sdk/x/auth/Params")
	registrar.RegisterConcrete(&ModuleCredential{}, "cosmos-sdk/GroupAccountCredential")

	legacy.RegisterAminoMsg(registrar, &MsgUpdateParams{}, "cosmos-sdk/x/auth/MsgUpdateParams")

	legacytx.RegisterLegacyAminoCodec(registrar)
}

// RegisterInterfaces associates protoName with AccountI interface
// and creates a registry of it's concrete implementations
func RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	registrar.RegisterInterface(
		"cosmos.auth.v1beta1.AccountI",
		(*AccountI)(nil),
		&BaseAccount{},
		&ModuleAccount{},
	)

	registrar.RegisterInterface(
		"cosmos.auth.v1beta1.AccountI",
		(*sdk.AccountI)(nil),
		&BaseAccount{},
		&ModuleAccount{},
	)

	registrar.RegisterInterface(
		"cosmos.auth.v1beta1.GenesisAccount",
		(*GenesisAccount)(nil),
		&BaseAccount{},
		&ModuleAccount{},
	)

	registrar.RegisterInterface(
		"cosmos.auth.v1.ModuleCredential",
		(*cryptotypes.PubKey)(nil),
		&ModuleCredential{},
	)

	registrar.RegisterImplementations((*coretransaction.Msg)(nil),
		&MsgUpdateParams{},
		&MsgNonAtomicExec{},
		&MsgMigrateAccount{},
	)
}
