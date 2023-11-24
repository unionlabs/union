package upgrade

import (
	"context"
	"encoding/json"
	"fmt"

	gwruntime "github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cast"
	"github.com/spf13/cobra"

	modulev1 "cosmossdk.io/api/cosmos/upgrade/module/v1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/store"
	"cosmossdk.io/depinject"
	"cosmossdk.io/x/upgrade/client/cli"
	"cosmossdk.io/x/upgrade/keeper"
	"cosmossdk.io/x/upgrade/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
)

func init() {
	types.RegisterLegacyAminoCodec(codec.NewLegacyAmino())
}

// ConsensusVersion defines the current x/upgrade module consensus version.
const ConsensusVersion uint64 = 2

var (
	_ module.AppModuleBasic = AppModule{}
	_ module.HasGenesis     = AppModule{}
	_ module.HasServices    = AppModule{}

	_ appmodule.AppModule     = AppModule{}
	_ appmodule.HasPreBlocker = AppModule{}
)

// AppModuleBasic implements the sdk.AppModuleBasic interface
type AppModuleBasic struct {
	ac address.Codec
}

// Name returns the ModuleName
func (AppModuleBasic) Name() string {
	return types.ModuleName
}

// RegisterLegacyAminoCodec registers the upgrade types on the LegacyAmino codec
func (AppModuleBasic) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	types.RegisterLegacyAminoCodec(cdc)
}

// RegisterGRPCGatewayRoutes registers the gRPC Gateway routes for the upgrade module.
func (AppModuleBasic) RegisterGRPCGatewayRoutes(clientCtx client.Context, mux *gwruntime.ServeMux) {
	if err := types.RegisterQueryHandlerClient(context.Background(), mux, types.NewQueryClient(clientCtx)); err != nil {
		panic(err)
	}
}

// GetTxCmd returns the CLI transaction commands for this module
func (ab AppModuleBasic) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd(ab.ac)
}

// RegisterInterfaces registers interfaces and implementations of the upgrade module.
func (AppModuleBasic) RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	types.RegisterInterfaces(registry)
}

// DefaultGenesis is an empty object
func (AppModuleBasic) DefaultGenesis(_ codec.JSONCodec) json.RawMessage {
	return []byte("{}")
}

// ValidateGenesis is always successful, as we ignore the value
func (AppModuleBasic) ValidateGenesis(_ codec.JSONCodec, config client.TxEncodingConfig, _ json.RawMessage) error {
	return nil
}

// AppModule implements the sdk.AppModule interface
type AppModule struct {
	AppModuleBasic
	keeper *keeper.Keeper
}

// NewAppModule creates a new AppModule object
func NewAppModule(keeper *keeper.Keeper, ac address.Codec) AppModule {
	return AppModule{
		AppModuleBasic: AppModuleBasic{ac: ac},
		keeper:         keeper,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() {}

// RegisterServices registers module services.
func (am AppModule) RegisterServices(cfg module.Configurator) {
	types.RegisterMsgServer(cfg.MsgServer(), keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(cfg.QueryServer(), am.keeper)

	m := keeper.NewMigrator(am.keeper)
	err := cfg.RegisterMigration(types.ModuleName, 1, m.Migrate1to2)
	if err != nil {
		panic(fmt.Sprintf("failed to migrate x/%s from version 1 to 2: %v", types.ModuleName, err))
	}
}

// InitGenesis is ignored, no sense in serializing future upgrades
func (am AppModule) InitGenesis(ctx sdk.Context, _ codec.JSONCodec, _ json.RawMessage) {
	// set version map automatically if available
	if versionMap := am.keeper.GetInitVersionMap(); versionMap != nil {
		// chains can still use a custom init chainer for setting the version map
		// this means that we need to combine the manually wired modules version map with app wiring enabled modules version map
		moduleVM, err := am.keeper.GetModuleVersionMap(ctx)
		if err != nil {
			panic(err)
		}

		for name, version := range moduleVM {
			if _, ok := versionMap[name]; !ok {
				versionMap[name] = version
			}
		}

		err = am.keeper.SetModuleVersionMap(ctx, versionMap)
		if err != nil {
			panic(err)
		}
	}
}

// ExportGenesis is always empty, as InitGenesis does nothing either
func (am AppModule) ExportGenesis(_ sdk.Context, cdc codec.JSONCodec) json.RawMessage {
	return am.DefaultGenesis(cdc)
}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (AppModule) ConsensusVersion() uint64 { return ConsensusVersion }

// PreBlock calls the upgrade module hooks
//
// CONTRACT: this is called *before* all other modules' BeginBlock functions
func (am AppModule) PreBlock(ctx context.Context) (appmodule.ResponsePreBlock, error) {
	return PreBlocker(ctx, am.keeper)
}

//
// App Wiring Setup
//

func init() {
	appmodule.Register(&modulev1.Module{},
		appmodule.Provide(ProvideModule),
		appmodule.Invoke(PopulateVersionMap),
	)
}

type ModuleInputs struct {
	depinject.In

	Config       *modulev1.Module
	StoreService store.KVStoreService
	Cdc          codec.Codec
	AddressCodec address.Codec

	AppOpts servertypes.AppOptions `optional:"true"`
}

type ModuleOutputs struct {
	depinject.Out

	UpgradeKeeper *keeper.Keeper
	Module        appmodule.AppModule
	BaseAppOption runtime.BaseAppOption
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	var (
		homePath           string
		skipUpgradeHeights = make(map[int64]bool)
	)

	if in.AppOpts != nil {
		for _, h := range cast.ToIntSlice(in.AppOpts.Get(server.FlagUnsafeSkipUpgrades)) {
			skipUpgradeHeights[int64(h)] = true
		}

		homePath = cast.ToString(in.AppOpts.Get(flags.FlagHome))
	}

	// default to governance authority if not provided
	authority := authtypes.NewModuleAddress(govtypes.ModuleName)
	if in.Config.Authority != "" {
		authority = authtypes.NewModuleAddressOrBech32Address(in.Config.Authority)
	}

	// set the governance module account as the authority for conducting upgrades
	k := keeper.NewKeeper(skipUpgradeHeights, in.StoreService, in.Cdc, homePath, nil, authority.String())
	baseappOpt := func(app *baseapp.BaseApp) {
		k.SetVersionSetter(app)
	}
	m := NewAppModule(k, in.AddressCodec)

	return ModuleOutputs{UpgradeKeeper: k, Module: m, BaseAppOption: baseappOpt}
}

func PopulateVersionMap(upgradeKeeper *keeper.Keeper, modules map[string]appmodule.AppModule) {
	if upgradeKeeper == nil {
		return
	}

	upgradeKeeper.SetInitVersionMap(module.NewManagerFromMap(modules).GetVersionMap())
}
