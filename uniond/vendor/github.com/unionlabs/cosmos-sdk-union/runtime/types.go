package runtime

import (
	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"

	"github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
)

const ModuleName = "runtime"

// AppI implements the common methods for a Cosmos SDK-based application
// specific blockchain.
type AppI interface {
	// Name the assigned name of the app.
	Name() string

	// BeginBlocker updates every begin block.
	BeginBlocker(ctx sdk.Context) (sdk.BeginBlock, error)

	// EndBlocker updates every end block.
	EndBlocker(ctx sdk.Context) (sdk.EndBlock, error)

	// InitChainer update at chain (i.e app) initialization.
	InitChainer(ctx sdk.Context, req *abci.InitChainRequest) (*abci.InitChainResponse, error)

	// LoadHeight load the app at a given height.
	LoadHeight(height int64) error

	// ExportAppStateAndValidators exports the state of the application for a genesis file.
	ExportAppStateAndValidators(forZeroHeight bool, jailAllowedAddrs, modulesToExport []string) (types.ExportedApp, error)
}

// AppSimI implements the common methods for a Cosmos SDK-based application
// specific blockchain that chooses to utilize the sdk simulation framework.
type AppSimI interface {
	AppI
	// SimulationManager helper for the simulation framework.
	SimulationManager() *module.SimulationManager
}
