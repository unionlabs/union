package runtime

import (
	"encoding/json"
	"io"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/version"
)

// AppBuilder is a type that is injected into a container by the runtime module
// (as *AppBuilder) which can be used to create an app which is compatible with
// the existing app.go initialization conventions.
type AppBuilder struct {
	app *App
}

// DefaultGenesis returns a default genesis from the registered AppModuleBasic's.
func (a *AppBuilder) DefaultGenesis() map[string]json.RawMessage {
	return a.app.DefaultGenesis()
}

// Build builds an *App instance.
func (a *AppBuilder) Build(
	logger log.Logger,
	db dbm.DB,
	traceStore io.Writer,
	baseAppOptions ...func(*baseapp.BaseApp),
) *App {
	for _, option := range a.app.baseAppOptions {
		baseAppOptions = append(baseAppOptions, option)
	}

	bApp := baseapp.NewBaseApp(a.app.config.AppName, logger, db, nil, baseAppOptions...)
	bApp.SetMsgServiceRouter(a.app.msgServiceRouter)
	bApp.SetCommitMultiStoreTracer(traceStore)
	bApp.SetVersion(version.Version)
	bApp.SetInterfaceRegistry(a.app.interfaceRegistry)
	bApp.MountStores(a.app.storeKeys...)

	a.app.BaseApp = bApp
	a.app.configurator = module.NewConfigurator(a.app.cdc, a.app.MsgServiceRouter(), a.app.GRPCQueryRouter())
	a.app.ModuleManager.RegisterServices(a.app.configurator)

	return a.app
}
