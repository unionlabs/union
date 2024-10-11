package module

import (
	modulev1 "cosmossdk.io/api/cosmos/group/module/v1"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"
	"cosmossdk.io/x/group"
	"cosmossdk.io/x/group/keeper"

	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
)

var _ depinject.OnePerModuleType = AppModule{}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

func init() {
	appconfig.RegisterModule(
		&modulev1.Module{},
		appconfig.Provide(ProvideModule),
	)
}

type GroupInputs struct {
	depinject.In

	Config        *modulev1.Module
	Environment   appmodule.Environment
	Cdc           codec.Codec
	AccountKeeper group.AccountKeeper
	BankKeeper    group.BankKeeper
	Registry      cdctypes.InterfaceRegistry
}

type GroupOutputs struct {
	depinject.Out

	GroupKeeper keeper.Keeper
	Module      appmodule.AppModule
}

func ProvideModule(in GroupInputs) GroupOutputs {
	k := keeper.NewKeeper(in.Environment,
		in.Cdc,
		in.AccountKeeper,
		group.Config{
			MaxExecutionPeriod:    in.Config.MaxExecutionPeriod.AsDuration(),
			MaxMetadataLen:        in.Config.MaxMetadataLen,
			MaxProposalTitleLen:   in.Config.MaxProposalTitleLen,
			MaxProposalSummaryLen: in.Config.MaxProposalSummaryLen,
		},
	)
	m := NewAppModule(in.Cdc, k, in.AccountKeeper, in.BankKeeper, in.Registry)
	return GroupOutputs{GroupKeeper: k, Module: m}
}
