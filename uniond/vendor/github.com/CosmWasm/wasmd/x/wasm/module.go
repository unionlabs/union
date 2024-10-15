package wasm

import (
	"context"
	"encoding/json"
	"fmt"
	"runtime/debug"
	"strings"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cast"
	"github.com/spf13/cobra"

	"google.golang.org/grpc"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/registry"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"

	"github.com/CosmWasm/wasmd/x/wasm/client/cli"
	"github.com/CosmWasm/wasmd/x/wasm/exported"
	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/simulation"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var (

	_ appmodule.AppModule				= AppModule{}
	_ appmodule.HasMigrations			= AppModule{}
	_ appmodule.HasGenesis				= AppModule{}
	_ appmodule.HasRegisterInterfaces	= AppModule{}

	_ module.HasAminoCodec       = AppModule{}
	_ module.HasGRPCGateway				= AppModule{}
)

// Module init related flags
const (
	flagWasmMemoryCacheSize        = "wasm.memory_cache_size"
	flagWasmQueryGasLimit          = "wasm.query_gas_limit"
	flagWasmSimulationGasLimit     = "wasm.simulation_gas_limit"
	flagWasmSkipWasmVMVersionCheck = "wasm.skip_wasmvm_version_check"
)

// AppModule implements an application module for the wasm module.
type AppModule struct {
	cdc                codec.Codec
	keeper             *keeper.Keeper
	validatorSetSource keeper.ValidatorSetSource
	accountKeeper      types.AccountKeeper // for simulation
	bankKeeper         simulation.BankKeeper
	router             keeper.MessageRouter
	// legacySubspace is used solely for migration of x/params managed parameters
	legacySubspace exported.Subspace
}

// NewAppModule creates a new AppModule object
func NewAppModule(
	cdc codec.Codec,
	keeper *keeper.Keeper,
	validatorSetSource keeper.ValidatorSetSource,
	ak types.AccountKeeper,
	bk simulation.BankKeeper,
	router *baseapp.MsgServiceRouter,
	ss exported.Subspace,
) AppModule {
	return AppModule{
		cdc:                cdc,
		keeper:             keeper,
		validatorSetSource: validatorSetSource,
		accountKeeper:      ak,
		bankKeeper:         bk,
		router:             router,
		legacySubspace:     ss,
	}
}

func (b AppModule) RegisterLegacyAminoCodec(amino registry.AminoRegistrar) {
	types.RegisterLegacyAminoCodec(amino)
}

func (b AppModule) RegisterGRPCGatewayRoutes(clientCtx client.Context, serveMux *runtime.ServeMux) {
	err := types.RegisterQueryHandlerClient(context.Background(), serveMux, types.NewQueryClient(clientCtx))
	if err != nil {
		panic(err)
	}
}

func (AppModule) Name() string {
	return types.ModuleName
}

// DefaultGenesis returns default genesis state as raw bytes for the wasm
// module.
func (am AppModule) DefaultGenesis() json.RawMessage {
	return am.cdc.MustMarshalJSON(&types.GenesisState{
		Params: types.DefaultParams(),
	})
}

// ValidateGenesis performs genesis state validation for the wasm module.
func (am AppModule) ValidateGenesis(bz json.RawMessage) error {
	var gs types.GenesisState
	if err := am.cdc.UnmarshalJSON(bz, &gs); err != nil {
		return fmt.Errorf("failed to unmarshal genesis state: %w", err)
	}
	return types.ValidateGenesis(gs)
}

// GetTxCmd returns the root tx command for the wasm module.
func (b AppModule) GetTxCmd() *cobra.Command {
	return cli.GetTxCmd()
}

// GetQueryCmd returns no root query command for the wasm module.
func (b AppModule) GetQueryCmd() *cobra.Command {
	return cli.GetQueryCmd()
}

// RegisterInterfaces implements InterfaceModule
func (b AppModule) RegisterInterfaces(registry registry.InterfaceRegistrar) {
	types.RegisterInterfaces(registry)
}

// ____________________________________________________________________________

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (am AppModule) IsOnePerModuleType() { // marker
}

// IsAppModule implements the appmodule.AppModule interface.
func (am AppModule) IsAppModule() { // marker
}

// ConsensusVersion is a sequence number for state-breaking change of the
// module. It should be incremented on each consensus-breaking change
// introduced by the module. To avoid wrong/empty versions, the initial version
// should be set to 1.
func (AppModule) ConsensusVersion() uint64 { return 4 }

// RegisterServices registers module services.
func (am AppModule) RegisterServices(registrar grpc.ServiceRegistrar) error {
	types.RegisterMsgServer(registrar, keeper.NewMsgServerImpl(am.keeper))
	types.RegisterQueryServer(registrar, keeper.Querier(am.keeper))
	return nil
}

func (am AppModule) RegisterMigrations(mr appmodule.MigrationRegistrar) error {
	m := keeper.NewMigrator(*am.keeper, am.legacySubspace)
	err := mr.Register(types.ModuleName, 1, m.Migrate1to2)
	if err != nil {
		return err
	}
	err = mr.Register(types.ModuleName, 2, m.Migrate2to3)
	if err != nil {
		return err
	}
	err = mr.Register(types.ModuleName, 3, m.Migrate3to4)
	if err != nil {
		return err
	}
	return nil
}

// RegisterInvariants registers the wasm module invariants.
func (am AppModule) RegisterInvariants(_ sdk.InvariantRegistry) {}

// QuerierRoute returns the wasm module's querier route name.
func (AppModule) QuerierRoute() string {
	return types.QuerierRoute
}

// InitGenesis performs genesis initialization for the wasm module. It returns
// no validator updates.
func (am AppModule) InitGenesis(ctx context.Context, bz json.RawMessage) error {
	var gs types.GenesisState
	err := am.cdc.UnmarshalJSON(bz, &gs)
	if err != nil {
		return fmt.Errorf("failed to unmarshal genesis state: %s", err)
	}
	return InitGenesis(ctx, am.keeper, &gs)
}

// ExportGenesis returns the exported genesis state as raw bytes for the wasm
// module.
func (am AppModule) ExportGenesis(ctx context.Context) (json.RawMessage, error) {
	gs, err := ExportGenesis(ctx, am.keeper)
	if err != nil {
		return nil, err
	}
	return am.cdc.MarshalJSON(gs)
}

// ____________________________________________________________________________

// AppModuleSimulation functions

// GenerateGenesisState creates a randomized GenState of the bank module.
func (AppModule) GenerateGenesisState(simState *module.SimulationState) {
	simulation.RandomizedGenState(simState)
}

// ProposalMsgs returns msgs used for governance proposals for simulations.
func (am AppModule) ProposalMsgs(simState module.SimulationState) []simtypes.WeightedProposalMsg {
	return simulation.ProposalMsgs(am.bankKeeper, am.keeper)
}

// RegisterStoreDecoder registers a decoder for supply module's types
func (am AppModule) RegisterStoreDecoder(_ simtypes.StoreDecoderRegistry) {
}

// WeightedOperations returns the all the gov module operations with their respective weights.
func (am AppModule) WeightedOperations(simState module.SimulationState) []simtypes.WeightedOperation {
	return simulation.WeightedOperations(simState.AppParams, am.accountKeeper, am.bankKeeper, am.keeper)
}

// ____________________________________________________________________________

// AddModuleInitFlags implements servertypes.ModuleInitFlags interface.
func AddModuleInitFlags(startCmd *cobra.Command) {
	defaults := types.DefaultWasmConfig()
	startCmd.Flags().Uint32(flagWasmMemoryCacheSize, defaults.MemoryCacheSize, "Sets the size in MiB (NOT bytes) of an in-memory cache for Wasm modules. Set to 0 to disable.")
	startCmd.Flags().Uint64(flagWasmQueryGasLimit, defaults.SmartQueryGasLimit, "Set the max gas that can be spent on executing a query with a Wasm contract")
	startCmd.Flags().String(flagWasmSimulationGasLimit, "", "Set the max gas that can be spent when executing a simulation TX")
	startCmd.Flags().Bool(flagWasmSkipWasmVMVersionCheck, false, "Skip check that ensures that libwasmvm version (the Rust project) and wasmvm version (the Go project) match")

	preCheck := func(cmd *cobra.Command, _ []string) error {
		skip, err := cmd.Flags().GetBool(flagWasmSkipWasmVMVersionCheck)
		if err != nil {
			return fmt.Errorf("unable to read skip flag value: %w", err)
		}
		if skip {
			cmd.Println("libwasmvm version check skipped")
			return nil
		}
		return CheckLibwasmVersion(getExpectedLibwasmVersion())
	}
	startCmd.PreRunE = chainPreRuns(preCheck, startCmd.PreRunE)
}

// ReadWasmConfig reads the wasm specifig configuration
func ReadWasmConfig(opts servertypes.AppOptions) (types.WasmConfig, error) {
	cfg := types.DefaultWasmConfig()
	var err error
	if v := opts.Get(flagWasmMemoryCacheSize); v != nil {
		if cfg.MemoryCacheSize, err = cast.ToUint32E(v); err != nil {
			return cfg, err
		}
	}
	if v := opts.Get(flagWasmQueryGasLimit); v != nil {
		if cfg.SmartQueryGasLimit, err = cast.ToUint64E(v); err != nil {
			return cfg, err
		}
	}
	if v := opts.Get(flagWasmSimulationGasLimit); v != nil {
		if raw, ok := v.(string); !ok || raw != "" {
			limit, err := cast.ToUint64E(v) // non empty string set
			if err != nil {
				return cfg, err
			}
			cfg.SimulationGasLimit = &limit
		}
	}
	// attach contract debugging to global "trace" flag
	if v := opts.Get(server.FlagTrace); v != nil {
		if cfg.ContractDebugMode, err = cast.ToBoolE(v); err != nil {
			return cfg, err
		}
	}
	return cfg, nil
}

func getExpectedLibwasmVersion() string {
	buildInfo, ok := debug.ReadBuildInfo()
	if !ok {
		panic("can't read build info")
	}
	for _, d := range buildInfo.Deps {
		if d.Path != "github.com/CosmWasm/wasmvm/v2" {
			continue
		}
		if d.Replace != nil {
			return d.Replace.Version
		}
		return d.Version
	}
	return ""
}

// CheckLibwasmVersion ensures that the libwasmvm version loaded at runtime matches the version
// of the github.com/CosmWasm/wasmvm dependency in go.mod. This us useful when dealing with
// shared libraries that are copied or moved from their default location, e.g. when building the node
// on one machine and deploying it to other machines.
//
// Usually the libwasmvm version (the Rust project) and wasmvm version (the Go project) match. However,
// there are situations in which this is not the case. This can be during development or if one of the
// two is patched. In such cases it is advised to not execute the check.
//
// An alternative method to obtain the libwasmvm version loaded at runtime is executing
// `wasmd query wasm libwasmvm-version`.
func CheckLibwasmVersion(wasmExpectedVersion string) error {
	if wasmExpectedVersion == "" {
		return fmt.Errorf("wasmvm module not exist")
	}
	wasmVersion, err := wasmvm.LibwasmvmVersion()
	if err != nil {
		return fmt.Errorf("unable to retrieve libwasmversion %w", err)
	}
	if !strings.Contains(wasmExpectedVersion, wasmVersion) {
		return fmt.Errorf("libwasmversion mismatch. got: %s; expected: %s", wasmVersion, wasmExpectedVersion)
	}
	return nil
}

type preRunFn func(cmd *cobra.Command, args []string) error

func chainPreRuns(pfns ...preRunFn) preRunFn {
	return func(cmd *cobra.Command, args []string) error {
		for _, pfn := range pfns {
			if pfn != nil {
				if err := pfn(cmd, args); err != nil {
					return err
				}
			}
		}
		return nil
	}
}
