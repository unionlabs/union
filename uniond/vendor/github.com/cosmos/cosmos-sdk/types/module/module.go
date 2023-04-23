/*
Package module contains application module patterns and associated "manager" functionality.
The module pattern has been broken down by:
  - independent module functionality (AppModuleBasic)
  - inter-dependent module genesis functionality (AppModuleGenesis)
  - inter-dependent module simulation functionality (AppModuleSimulation)
  - inter-dependent module full functionality (AppModule)

inter-dependent module functionality is module functionality which somehow
depends on other modules, typically through the module keeper.  Many of the
module keepers are dependent on each other, thus in order to access the full
set of module functionality we need to define all the keepers/params-store/keys
etc. This full set of advanced functionality is defined by the AppModule interface.

Independent module functions are separated to allow for the construction of the
basic application structures required early on in the application definition
and used to enable the definition of full module functionality later in the
process. This separation is necessary, however we still want to allow for a
high level pattern for modules to follow - for instance, such that we don't
have to manually register all of the codecs for all the modules. This basic
procedure as well as other basic patterns are handled through the use of
BasicManager.

Lastly the interface for genesis functionality (AppModuleGenesis) has been
separated out from full module functionality (AppModule) so that modules which
are only used for genesis can take advantage of the Module patterns without
needlessly defining many placeholder functions
*/
package module

import (
	"encoding/json"
	"fmt"
	"sort"

	"cosmossdk.io/core/appmodule"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/spf13/cobra"
	"golang.org/x/exp/maps"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// AppModuleBasic is the standard form for basic non-dependant elements of an application module.
type AppModuleBasic interface {
	HasName
	RegisterLegacyAminoCodec(*codec.LegacyAmino)
	RegisterInterfaces(codectypes.InterfaceRegistry)

	// client functionality
	RegisterGRPCGatewayRoutes(client.Context, *runtime.ServeMux)
	GetTxCmd() *cobra.Command
	GetQueryCmd() *cobra.Command
}

// HasName allows the module to provide its own name for legacy purposes.
// Newer apps should specify the name for their modules using a map
// using NewManagerFromMap.
type HasName interface {
	Name() string
}

// HasGenesisBasics is the legacy interface for stateless genesis methods.
type HasGenesisBasics interface {
	DefaultGenesis(codec.JSONCodec) json.RawMessage
	ValidateGenesis(codec.JSONCodec, client.TxEncodingConfig, json.RawMessage) error
}

// BasicManager is a collection of AppModuleBasic
type BasicManager map[string]AppModuleBasic

// NewBasicManager creates a new BasicManager object
func NewBasicManager(modules ...AppModuleBasic) BasicManager {
	moduleMap := make(map[string]AppModuleBasic)
	for _, module := range modules {
		moduleMap[module.Name()] = module
	}
	return moduleMap
}

// RegisterLegacyAminoCodec registers all module codecs
func (bm BasicManager) RegisterLegacyAminoCodec(cdc *codec.LegacyAmino) {
	for _, b := range bm {
		b.RegisterLegacyAminoCodec(cdc)
	}
}

// RegisterInterfaces registers all module interface types
func (bm BasicManager) RegisterInterfaces(registry codectypes.InterfaceRegistry) {
	for _, m := range bm {
		m.RegisterInterfaces(registry)
	}
}

// DefaultGenesis provides default genesis information for all modules
func (bm BasicManager) DefaultGenesis(cdc codec.JSONCodec) map[string]json.RawMessage {
	genesis := make(map[string]json.RawMessage)
	for _, b := range bm {
		if mod, ok := b.(HasGenesisBasics); ok {
			genesis[b.Name()] = mod.DefaultGenesis(cdc)
		}
	}

	return genesis
}

// ValidateGenesis performs genesis state validation for all modules
func (bm BasicManager) ValidateGenesis(cdc codec.JSONCodec, txEncCfg client.TxEncodingConfig, genesis map[string]json.RawMessage) error {
	for _, b := range bm {
		if mod, ok := b.(HasGenesisBasics); ok {
			if err := mod.ValidateGenesis(cdc, txEncCfg, genesis[b.Name()]); err != nil {
				return err
			}
		}
	}

	return nil
}

// RegisterGRPCGatewayRoutes registers all module rest routes
func (bm BasicManager) RegisterGRPCGatewayRoutes(clientCtx client.Context, rtr *runtime.ServeMux) {
	for _, b := range bm {
		b.RegisterGRPCGatewayRoutes(clientCtx, rtr)
	}
}

// AddTxCommands adds all tx commands to the rootTxCmd.
//
// TODO: Remove clientCtx argument.
// REF: https://github.com/cosmos/cosmos-sdk/issues/6571
func (bm BasicManager) AddTxCommands(rootTxCmd *cobra.Command) {
	for _, b := range bm {
		if cmd := b.GetTxCmd(); cmd != nil {
			rootTxCmd.AddCommand(cmd)
		}
	}
}

// AddQueryCommands adds all query commands to the rootQueryCmd.
//
// TODO: Remove clientCtx argument.
// REF: https://github.com/cosmos/cosmos-sdk/issues/6571
func (bm BasicManager) AddQueryCommands(rootQueryCmd *cobra.Command) {
	for _, b := range bm {
		if cmd := b.GetQueryCmd(); cmd != nil {
			rootQueryCmd.AddCommand(cmd)
		}
	}
}

// AppModuleGenesis is the standard form for an application module genesis functions
type AppModuleGenesis interface {
	AppModuleBasic
	HasGenesis
}

// HasGenesis is the extension interface for stateful genesis methods.
type HasGenesis interface {
	HasGenesisBasics
	InitGenesis(sdk.Context, codec.JSONCodec, json.RawMessage) []abci.ValidatorUpdate
	ExportGenesis(sdk.Context, codec.JSONCodec) json.RawMessage
}

// AppModule is the form for an application module. Most of
// its functionality has been moved to extension interfaces.
type AppModule interface {
	AppModuleBasic
}

// HasInvariants is the interface for registering invariants.
type HasInvariants interface {
	// RegisterInvariants registers module invariants.
	RegisterInvariants(sdk.InvariantRegistry)
}

// HasServices is the interface for modules to register services.
type HasServices interface {
	// RegisterServices allows a module to register services.
	RegisterServices(Configurator)
}

// HasConsensusVersion is the interface for declaring a module consensus version.
type HasConsensusVersion interface {
	// ConsensusVersion is a sequence number for state-breaking change of the
	// module. It should be incremented on each consensus-breaking change
	// introduced by the module. To avoid wrong/empty versions, the initial version
	// should be set to 1.
	ConsensusVersion() uint64
}

// BeginBlockAppModule is an extension interface that contains information about the AppModule and BeginBlock.
type BeginBlockAppModule interface {
	AppModule
	BeginBlock(sdk.Context, abci.RequestBeginBlock)
}

// EndBlockAppModule is an extension interface that contains information about the AppModule and EndBlock.
type EndBlockAppModule interface {
	AppModule
	EndBlock(sdk.Context, abci.RequestEndBlock) []abci.ValidatorUpdate
}

// GenesisOnlyAppModule is an AppModule that only has import/export functionality
type GenesisOnlyAppModule struct {
	AppModuleGenesis
}

// NewGenesisOnlyAppModule creates a new GenesisOnlyAppModule object
func NewGenesisOnlyAppModule(amg AppModuleGenesis) GenesisOnlyAppModule {
	return GenesisOnlyAppModule{
		AppModuleGenesis: amg,
	}
}

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (GenesisOnlyAppModule) IsOnePerModuleType() {}

// IsAppModule implements the appmodule.AppModule interface.
func (GenesisOnlyAppModule) IsAppModule() {}

// RegisterInvariants is a placeholder function register no invariants
func (GenesisOnlyAppModule) RegisterInvariants(_ sdk.InvariantRegistry) {}

// QuerierRoute returns an empty module querier route
func (GenesisOnlyAppModule) QuerierRoute() string { return "" }

// RegisterServices registers all services.
func (gam GenesisOnlyAppModule) RegisterServices(Configurator) {}

// ConsensusVersion implements AppModule/ConsensusVersion.
func (gam GenesisOnlyAppModule) ConsensusVersion() uint64 { return 1 }

// BeginBlock returns an empty module begin-block
func (gam GenesisOnlyAppModule) BeginBlock(ctx sdk.Context, req abci.RequestBeginBlock) {}

// EndBlock returns an empty module end-block
func (GenesisOnlyAppModule) EndBlock(_ sdk.Context, _ abci.RequestEndBlock) []abci.ValidatorUpdate {
	return []abci.ValidatorUpdate{}
}

// Manager defines a module manager that provides the high level utility for managing and executing
// operations for a group of modules
type Manager struct {
	Modules            map[string]interface{} // interface{} is used now to support the legacy AppModule as well as new core appmodule.AppModule.
	OrderInitGenesis   []string
	OrderExportGenesis []string
	OrderBeginBlockers []string
	OrderEndBlockers   []string
	OrderMigrations    []string
}

// NewManager creates a new Manager object.
func NewManager(modules ...AppModule) *Manager {
	moduleMap := make(map[string]interface{})
	modulesStr := make([]string, 0, len(modules))
	for _, module := range modules {
		moduleMap[module.Name()] = module
		modulesStr = append(modulesStr, module.Name())
	}

	return &Manager{
		Modules:            moduleMap,
		OrderInitGenesis:   modulesStr,
		OrderExportGenesis: modulesStr,
		OrderBeginBlockers: modulesStr,
		OrderEndBlockers:   modulesStr,
	}
}

// NewManagerFromMap creates a new Manager object from a map of module names to module implementations.
// This method should be used for apps and modules which have migrated to the cosmossdk.io/core.appmodule.AppModule API.
func NewManagerFromMap(moduleMap map[string]appmodule.AppModule) *Manager {
	simpleModuleMap := make(map[string]interface{})
	modulesStr := make([]string, 0, len(simpleModuleMap))
	for name, module := range moduleMap {
		simpleModuleMap[name] = module
		modulesStr = append(modulesStr, name)
	}

	return &Manager{
		Modules:            simpleModuleMap,
		OrderInitGenesis:   modulesStr,
		OrderExportGenesis: modulesStr,
		OrderBeginBlockers: modulesStr,
		OrderEndBlockers:   modulesStr,
	}
}

// SetOrderInitGenesis sets the order of init genesis calls
func (m *Manager) SetOrderInitGenesis(moduleNames ...string) {
	m.assertNoForgottenModules("SetOrderInitGenesis", moduleNames)
	m.OrderInitGenesis = moduleNames
}

// SetOrderExportGenesis sets the order of export genesis calls
func (m *Manager) SetOrderExportGenesis(moduleNames ...string) {
	m.assertNoForgottenModules("SetOrderExportGenesis", moduleNames)
	m.OrderExportGenesis = moduleNames
}

// SetOrderBeginBlockers sets the order of set begin-blocker calls
func (m *Manager) SetOrderBeginBlockers(moduleNames ...string) {
	m.assertNoForgottenModules("SetOrderBeginBlockers", moduleNames)
	m.OrderBeginBlockers = moduleNames
}

// SetOrderEndBlockers sets the order of set end-blocker calls
func (m *Manager) SetOrderEndBlockers(moduleNames ...string) {
	m.assertNoForgottenModules("SetOrderEndBlockers", moduleNames)
	m.OrderEndBlockers = moduleNames
}

// SetOrderMigrations sets the order of migrations to be run. If not set
// then migrations will be run with an order defined in `DefaultMigrationsOrder`.
func (m *Manager) SetOrderMigrations(moduleNames ...string) {
	m.assertNoForgottenModules("SetOrderMigrations", moduleNames)
	m.OrderMigrations = moduleNames
}

// RegisterInvariants registers all module invariants
func (m *Manager) RegisterInvariants(ir sdk.InvariantRegistry) {
	for _, module := range m.Modules {
		if module, ok := module.(HasInvariants); ok {
			module.RegisterInvariants(ir)
		}
	}
}

// RegisterServices registers all module services
func (m *Manager) RegisterServices(cfg Configurator) {
	for _, module := range m.Modules {
		if module, ok := module.(HasServices); ok {
			module.RegisterServices(cfg)
		}
	}
}

// InitGenesis performs init genesis functionality for modules. Exactly one
// module must return a non-empty validator set update to correctly initialize
// the chain.
func (m *Manager) InitGenesis(ctx sdk.Context, cdc codec.JSONCodec, genesisData map[string]json.RawMessage) abci.ResponseInitChain {
	var validatorUpdates []abci.ValidatorUpdate
	ctx.Logger().Info("initializing blockchain state from genesis.json")
	for _, moduleName := range m.OrderInitGenesis {
		if genesisData[moduleName] == nil {
			continue
		}

		if module, ok := m.Modules[moduleName].(HasGenesis); ok {
			ctx.Logger().Debug("running initialization for module", "module", moduleName)

			moduleValUpdates := module.InitGenesis(ctx, cdc, genesisData[moduleName])

			// use these validator updates if provided, the module manager assumes
			// only one module will update the validator set
			if len(moduleValUpdates) > 0 {
				if len(validatorUpdates) > 0 {
					panic("validator InitGenesis updates already set by a previous module")
				}
				validatorUpdates = moduleValUpdates
			}
		}
	}

	// a chain must initialize with a non-empty validator set
	if len(validatorUpdates) == 0 {
		panic(fmt.Sprintf("validator set is empty after InitGenesis, please ensure at least one validator is initialized with a delegation greater than or equal to the DefaultPowerReduction (%d)", sdk.DefaultPowerReduction))
	}

	return abci.ResponseInitChain{
		Validators: validatorUpdates,
	}
}

// ExportGenesis performs export genesis functionality for modules
func (m *Manager) ExportGenesis(ctx sdk.Context, cdc codec.JSONCodec) map[string]json.RawMessage {
	return m.ExportGenesisForModules(ctx, cdc, []string{})
}

// ExportGenesisForModules performs export genesis functionality for modules
func (m *Manager) ExportGenesisForModules(ctx sdk.Context, cdc codec.JSONCodec, modulesToExport []string) map[string]json.RawMessage {
	if len(modulesToExport) == 0 {
		modulesToExport = m.OrderExportGenesis
	}

	// verify modules exists in app, so that we don't panic in the middle of an export
	if err := m.checkModulesExists(modulesToExport); err != nil {
		panic(err)
	}

	channels := make(map[string]chan json.RawMessage)
	for _, moduleName := range modulesToExport {
		if module, ok := m.Modules[moduleName].(HasGenesis); ok {
			channels[moduleName] = make(chan json.RawMessage)
			go func(module HasGenesis, ch chan json.RawMessage) {
				ctx := ctx.WithGasMeter(sdk.NewInfiniteGasMeter()) // avoid race conditions
				ch <- module.ExportGenesis(ctx, cdc)
			}(module, channels[moduleName])
		}
	}

	genesisData := make(map[string]json.RawMessage)
	for moduleName := range channels {
		genesisData[moduleName] = <-channels[moduleName]
	}

	return genesisData
}

// checkModulesExists verifies that all modules in the list exist in the app
func (m *Manager) checkModulesExists(moduleName []string) error {
	for _, name := range moduleName {
		if _, ok := m.Modules[name]; !ok {
			return fmt.Errorf("module %s does not exist", name)
		}
	}

	return nil
}

// assertNoForgottenModules checks that we didn't forget any modules in the
// SetOrder* functions.
func (m *Manager) assertNoForgottenModules(setOrderFnName string, moduleNames []string) {
	ms := make(map[string]bool)
	for _, m := range moduleNames {
		ms[m] = true
	}
	var missing []string
	for m := range m.Modules {
		if !ms[m] {
			missing = append(missing, m)
		}
	}
	if len(missing) != 0 {
		sort.Strings(missing)
		panic(fmt.Sprintf(
			"%s: all modules must be defined when setting %s, missing: %v", setOrderFnName, setOrderFnName, missing))
	}
}

// MigrationHandler is the migration function that each module registers.
type MigrationHandler func(sdk.Context) error

// VersionMap is a map of moduleName -> version
type VersionMap map[string]uint64

// RunMigrations performs in-place store migrations for all modules. This
// function MUST be called insde an x/upgrade UpgradeHandler.
//
// Recall that in an upgrade handler, the `fromVM` VersionMap is retrieved from
// x/upgrade's store, and the function needs to return the target VersionMap
// that will in turn be persisted to the x/upgrade's store. In general,
// returning RunMigrations should be enough:
//
// Example:
//
//	cfg := module.NewConfigurator(...)
//	app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
//	    return app.mm.RunMigrations(ctx, cfg, fromVM)
//	})
//
// Internally, RunMigrations will perform the following steps:
// - create an `updatedVM` VersionMap of module with their latest ConsensusVersion
// - make a diff of `fromVM` and `udpatedVM`, and for each module:
//   - if the module's `fromVM` version is less than its `updatedVM` version,
//     then run in-place store migrations for that module between those versions.
//   - if the module does not exist in the `fromVM` (which means that it's a new module,
//     because it was not in the previous x/upgrade's store), then run
//     `InitGenesis` on that module.
//
// - return the `updatedVM` to be persisted in the x/upgrade's store.
//
// Migrations are run in an order defined by `Manager.OrderMigrations` or (if not set) defined by
// `DefaultMigrationsOrder` function.
//
// As an app developer, if you wish to skip running InitGenesis for your new
// module "foo", you need to manually pass a `fromVM` argument to this function
// foo's module version set to its latest ConsensusVersion. That way, the diff
// between the function's `fromVM` and `udpatedVM` will be empty, hence not
// running anything for foo.
//
// Example:
//
//	cfg := module.NewConfigurator(...)
//	app.UpgradeKeeper.SetUpgradeHandler("my-plan", func(ctx sdk.Context, plan upgradetypes.Plan, fromVM module.VersionMap) (module.VersionMap, error) {
//	    // Assume "foo" is a new module.
//	    // `fromVM` is fetched from existing x/upgrade store. Since foo didn't exist
//	    // before this upgrade, `v, exists := fromVM["foo"]; exists == false`, and RunMigration will by default
//	    // run InitGenesis on foo.
//	    // To skip running foo's InitGenesis, you need set `fromVM`'s foo to its latest
//	    // consensus version:
//	    fromVM["foo"] = foo.AppModule{}.ConsensusVersion()
//
//	    return app.mm.RunMigrations(ctx, cfg, fromVM)
//	})
//
// Please also refer to docs/core/upgrade.md for more information.
func (m Manager) RunMigrations(ctx sdk.Context, cfg Configurator, fromVM VersionMap) (VersionMap, error) {
	c, ok := cfg.(configurator)
	if !ok {
		return nil, sdkerrors.Wrapf(sdkerrors.ErrInvalidType, "expected %T, got %T", configurator{}, cfg)
	}
	modules := m.OrderMigrations
	if modules == nil {
		modules = DefaultMigrationsOrder(m.ModuleNames())
	}

	updatedVM := VersionMap{}
	for _, moduleName := range modules {
		module := m.Modules[moduleName]
		fromVersion, exists := fromVM[moduleName]
		toVersion := uint64(0)
		if module, ok := module.(HasConsensusVersion); ok {
			toVersion = module.ConsensusVersion()
		}

		// We run migration if the module is specified in `fromVM`.
		// Otherwise we run InitGenesis.
		//
		// The module won't exist in the fromVM in two cases:
		// 1. A new module is added. In this case we run InitGenesis with an
		// empty genesis state.
		// 2. An existing chain is upgrading from version < 0.43 to v0.43+ for the first time.
		// In this case, all modules have yet to be added to x/upgrade's VersionMap store.
		if exists {
			err := c.runModuleMigrations(ctx, moduleName, fromVersion, toVersion)
			if err != nil {
				return nil, err
			}
		} else {
			ctx.Logger().Info(fmt.Sprintf("adding a new module: %s", moduleName))
			if module, ok := m.Modules[moduleName].(HasGenesis); ok {
				moduleValUpdates := module.InitGenesis(ctx, c.cdc, module.DefaultGenesis(c.cdc))
				// The module manager assumes only one module will update the
				// validator set, and it can't be a new module.
				if len(moduleValUpdates) > 0 {
					return nil, sdkerrors.Wrapf(sdkerrors.ErrLogic, "validator InitGenesis update is already set by another module")
				}
			}
		}

		updatedVM[moduleName] = toVersion
	}

	return updatedVM, nil
}

// BeginBlock performs begin block functionality for all modules. It creates a
// child context with an event manager to aggregate events emitted from all
// modules.
func (m *Manager) BeginBlock(ctx sdk.Context, req abci.RequestBeginBlock) abci.ResponseBeginBlock {
	ctx = ctx.WithEventManager(sdk.NewEventManager())

	for _, moduleName := range m.OrderBeginBlockers {
		module, ok := m.Modules[moduleName].(BeginBlockAppModule)
		if ok {
			module.BeginBlock(ctx, req)
		}
	}

	return abci.ResponseBeginBlock{
		Events: ctx.EventManager().ABCIEvents(),
	}
}

// EndBlock performs end block functionality for all modules. It creates a
// child context with an event manager to aggregate events emitted from all
// modules.
func (m *Manager) EndBlock(ctx sdk.Context, req abci.RequestEndBlock) abci.ResponseEndBlock {
	ctx = ctx.WithEventManager(sdk.NewEventManager())
	validatorUpdates := []abci.ValidatorUpdate{}

	for _, moduleName := range m.OrderEndBlockers {
		module, ok := m.Modules[moduleName].(EndBlockAppModule)
		if !ok {
			continue
		}
		moduleValUpdates := module.EndBlock(ctx, req)

		// use these validator updates if provided, the module manager assumes
		// only one module will update the validator set
		if len(moduleValUpdates) > 0 {
			if len(validatorUpdates) > 0 {
				panic("validator EndBlock updates already set by a previous module")
			}

			validatorUpdates = moduleValUpdates
		}
	}

	return abci.ResponseEndBlock{
		ValidatorUpdates: validatorUpdates,
		Events:           ctx.EventManager().ABCIEvents(),
	}
}

// GetVersionMap gets consensus version from all modules
func (m *Manager) GetVersionMap() VersionMap {
	vermap := make(VersionMap)
	for name, v := range m.Modules {
		version := uint64(0)
		if v, ok := v.(HasConsensusVersion); ok {
			version = v.ConsensusVersion()
		}
		name := name
		vermap[name] = version
	}

	return vermap
}

// ModuleNames returns list of all module names, without any particular order.
func (m *Manager) ModuleNames() []string {
	return maps.Keys(m.Modules)
}

// DefaultMigrationsOrder returns a default migrations order: ascending alphabetical by module name,
// except x/auth which will run last, see:
// https://github.com/cosmos/cosmos-sdk/issues/10591
func DefaultMigrationsOrder(modules []string) []string {
	const authName = "auth"
	out := make([]string, 0, len(modules))
	hasAuth := false
	for _, m := range modules {
		if m == authName {
			hasAuth = true
		} else {
			out = append(out, m)
		}
	}
	sort.Strings(out)
	if hasAuth {
		out = append(out, authName)
	}
	return out
}
