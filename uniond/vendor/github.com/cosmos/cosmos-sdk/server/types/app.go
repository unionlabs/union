package types

import (
	"encoding/json"
	"io"
	"time"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	tmtypes "github.com/cometbft/cometbft/types"
	"github.com/cosmos/gogoproto/grpc"
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	"github.com/cosmos/cosmos-sdk/snapshots"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
)

// ServerStartTime defines the time duration that the server need to stay running after startup
// for the startup be considered successful
const ServerStartTime = 5 * time.Second

type (
	// AppOptions defines an interface that is passed into an application
	// constructor, typically used to set BaseApp options that are either supplied
	// via config file or through CLI arguments/flags. The underlying implementation
	// is defined by the server package and is typically implemented via a Viper
	// literal defined on the server Context. Note, casting Get calls may not yield
	// the expected types and could result in type assertion errors. It is recommend
	// to either use the cast package or perform manual conversion for safety.
	AppOptions interface {
		Get(string) interface{}
	}

	// Application defines an application interface that wraps abci.Application.
	// The interface defines the necessary contracts to be implemented in order
	// to fully bootstrap and start an application.
	Application interface {
		abci.Application

		RegisterAPIRoutes(*api.Server, config.APIConfig)

		// RegisterGRPCServer registers gRPC services directly with the gRPC
		// server.
		RegisterGRPCServer(grpc.Server)

		// RegisterTxService registers the gRPC Query service for tx (such as tx
		// simulation, fetching txs by hash...).
		RegisterTxService(client.Context)

		// RegisterTendermintService registers the gRPC Query service for tendermint queries.
		RegisterTendermintService(client.Context)

		// RegisterNodeService registers the node gRPC Query service.
		RegisterNodeService(client.Context)

		// CommitMultiStore return the multistore instance
		CommitMultiStore() storetypes.CommitMultiStore

		// Return the snapshot manager
		SnapshotManager() *snapshots.Manager

		// Close is called in start cmd to gracefully cleanup resources.
		Close() error
	}

	// AppCreator is a function that allows us to lazily initialize an
	// application using various configurations.
	AppCreator func(log.Logger, dbm.DB, io.Writer, AppOptions) Application

	// ModuleInitFlags takes a start command and adds modules specific init flags.
	ModuleInitFlags func(startCmd *cobra.Command)

	// ExportedApp represents an exported app state, along with
	// validators, consensus params and latest app height.
	ExportedApp struct {
		// AppState is the application state as JSON.
		AppState json.RawMessage
		// Validators is the exported validator set.
		Validators []tmtypes.GenesisValidator
		// Height is the app's latest block height.
		Height int64
		// ConsensusParams are the exported consensus params for ABCI.
		ConsensusParams *tmproto.ConsensusParams
	}

	// AppExporter is a function that dumps all app state to
	// JSON-serializable structure and returns the current validator set.
	AppExporter func(log.Logger, dbm.DB, io.Writer, int64, bool, []string, AppOptions, []string) (ExportedApp, error)
)
