package runtime

import (
	"encoding/json"
	"fmt"
	"io"
	"path/filepath"

	"github.com/spf13/cast"

	corestore "cosmossdk.io/core/store"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client/flags"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
)

// AppBuilder is a type that is injected into a container by the runtime module
// (as *AppBuilder) which can be used to create an app which is compatible with
// the existing app.go initialization conventions.
type AppBuilder struct {
	app *App

	appOptions servertypes.AppOptions
}

// DefaultGenesis returns a default genesis from the registered modules.
func (a *AppBuilder) DefaultGenesis() map[string]json.RawMessage {
	return a.app.DefaultGenesis()
}

// Build builds an *App instance.
func (a *AppBuilder) Build(db corestore.KVStoreWithBatch, traceStore io.Writer, baseAppOptions ...func(*baseapp.BaseApp)) *App {
	for _, option := range a.app.baseAppOptions {
		baseAppOptions = append(baseAppOptions, option)
	}

	// set routers first in case they get modified by other options
	baseAppOptions = append(
		[]func(*baseapp.BaseApp){
			func(bApp *baseapp.BaseApp) {
				bApp.SetMsgServiceRouter(a.app.msgServiceRouter)
				bApp.SetGRPCQueryRouter(a.app.grpcQueryRouter)
			},
		},
		baseAppOptions...,
	)

	bApp := baseapp.NewBaseApp(a.app.config.AppName, a.app.logger, db, nil, baseAppOptions...)
	bApp.SetCommitMultiStoreTracer(traceStore)
	bApp.SetVersion(version.Version)
	bApp.SetInterfaceRegistry(a.app.interfaceRegistry)
	bApp.MountStores(a.app.storeKeys...)

	a.app.BaseApp = bApp
	a.app.configurator = module.NewConfigurator(a.app.cdc, a.app.MsgServiceRouter(), a.app.GRPCQueryRouter())

	if a.appOptions != nil {
		// register unordered tx manager
		if err := a.registerUnorderedTxManager(); err != nil {
			panic(err)
		}

		// register indexer if enabled
		if err := a.registerIndexer(); err != nil {
			panic(err)
		}
	}

	// register services
	if err := a.app.ModuleManager.RegisterServices(a.app.configurator); err != nil {
		panic(err)
	}

	return a.app
}

// register unordered tx manager
func (a *AppBuilder) registerUnorderedTxManager() error {
	// create, start, and load the unordered tx manager
	utxDataDir := filepath.Join(cast.ToString(a.appOptions.Get(flags.FlagHome)), "data")
	a.app.UnorderedTxManager = unorderedtx.NewManager(utxDataDir)
	a.app.UnorderedTxManager.Start()

	if err := a.app.UnorderedTxManager.OnInit(); err != nil {
		return fmt.Errorf("failed to initialize unordered tx manager: %w", err)
	}

	return nil
}

// register indexer
func (a *AppBuilder) registerIndexer() error {
	// if we have indexer options in app.toml, then enable the built-in indexer framework
	if indexerOpts := a.appOptions.Get("indexer"); indexerOpts != nil {
		moduleSet := map[string]any{}
		for modName, mod := range a.app.ModuleManager.Modules {
			moduleSet[modName] = mod
		}

		return a.app.EnableIndexer(indexerOpts, a.kvStoreKeys(), moduleSet)
	}

	// register legacy streaming services if we don't have the built-in indexer enabled
	return a.app.RegisterStreamingServices(a.appOptions, a.kvStoreKeys())
}

func (a *AppBuilder) kvStoreKeys() map[string]*storetypes.KVStoreKey {
	keys := make(map[string]*storetypes.KVStoreKey)
	for _, k := range a.app.GetStoreKeys() {
		if kv, ok := k.(*storetypes.KVStoreKey); ok {
			keys[kv.Name()] = kv
		}
	}

	return keys
}
