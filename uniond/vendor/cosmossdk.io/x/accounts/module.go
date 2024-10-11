package accounts

import (
	"context"
	"encoding/json"

	"github.com/spf13/cobra"
	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"
	"cosmossdk.io/x/accounts/cli"
	v1 "cosmossdk.io/x/accounts/v1"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/msgservice"
)

const (
	ModuleName = "accounts"
	StoreKey   = "_" + ModuleName // unfortunately accounts collides with auth store key

	ConsensusVersion = 1
)

// ModuleAccountAddress defines the x/accounts module address.
var ModuleAccountAddress = address.Module(ModuleName)

var (
	_ appmodule.AppModule           = AppModule{}
	_ appmodule.HasGenesis          = AppModule{}
	_ appmodule.HasConsensusVersion = AppModule{}
)

func NewAppModule(cdc codec.Codec, k Keeper) AppModule {
	return AppModule{k: k, cdc: cdc}
}

type AppModule struct {
	cdc codec.Codec
	k   Keeper
}

func (m AppModule) IsAppModule() {}

// Name returns the module's name.
// Deprecated: kept for legacy reasons.
func (AppModule) Name() string { return ModuleName }

func (m AppModule) RegisterInterfaces(registrar registry.InterfaceRegistrar) {
	msgservice.RegisterMsgServiceDesc(registrar, v1.MsgServiceDesc())
}

// App module services

func (m AppModule) RegisterServices(registar grpc.ServiceRegistrar) error {
	v1.RegisterQueryServer(registar, NewQueryServer(m.k))
	v1.RegisterMsgServer(registar, NewMsgServer(m.k))

	return nil
}

// App module genesis

func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(&v1.GenesisState{})
}

func (am AppModule) ValidateGenesis(message json.RawMessage) error {
	gs := &v1.GenesisState{}
	if err := am.cdc.UnmarshalJSON(message, gs); err != nil {
		return err
	}
	// Add validation logic for gs here
	return nil
}

func (am AppModule) InitGenesis(ctx context.Context, message json.RawMessage) error {
	gs := &v1.GenesisState{}
	if err := am.cdc.UnmarshalJSON(message, gs); err != nil {
		return err
	}
	err := am.k.ImportState(ctx, gs)
	if err != nil {
		return err
	}
	return nil
}

func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := am.k.ExportState(ctx)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

func (AppModule) GetTxCmd() *cobra.Command {
	return cli.TxCmd(ModuleName)
}

func (AppModule) GetQueryCmd() *cobra.Command {
	return cli.QueryCmd(ModuleName)
}

func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }
