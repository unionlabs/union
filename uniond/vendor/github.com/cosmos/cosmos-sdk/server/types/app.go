package types

import (
	"encoding/json"
	"io"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cosmos/gogoproto/grpc"

	"cosmossdk.io/core/server"
	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/log"
	"cosmossdk.io/store/snapshots"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/server/api"
	"github.com/cosmos/cosmos-sdk/server/config"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type (
	// AppOptions defines an interface that is passed into an application
	// constructor, typically used to set BaseApp options that are either supplied
	// via config file or through CLI arguments/flags. The underlying implementation
	// is defined by the server package and is typically implemented via a Viper
	// literal defined on the server Context. Note, casting Get calls may not yield
	// the expected types and could result in type assertion errors. It is recommend
	// to either use the cast package or perform manual conversion for safety.
	AppOptions = server.DynamicConfig

	// Application defines an application interface that wraps abci.Application.
	// The interface defines the necessary contracts to be implemented in order
	// to fully bootstrap and start an application.
	Application interface {
		ABCI

		RegisterAPIRoutes(*api.Server, config.APIConfig)

		// RegisterGRPCServer registers gRPC services directly with the gRPC
		// server.
		RegisterGRPCServer(grpc.Server)

		// RegisterTxService registers the gRPC Query service for tx (such as tx
		// simulation, fetching txs by hash...).
		RegisterTxService(client.Context)

		// RegisterTendermintService registers the gRPC Query service for CometBFT queries.
		RegisterTendermintService(client.Context)

		// RegisterNodeService registers the node gRPC Query service.
		RegisterNodeService(client.Context, config.Config)

		// CommitMultiStore return the multistore instance
		CommitMultiStore() storetypes.CommitMultiStore

		// SnapshotManager return the snapshot manager
		SnapshotManager() *snapshots.Manager

		// ValidatorKeyProvider returns a function that generates a validator key
		ValidatorKeyProvider() func() (cmtcrypto.PrivKey, error)

		// Close is called in start cmd to gracefully cleanup resources.
		// Must be safe to be called multiple times.
		Close() error
	}

	// AppCreator is a function that allows us to lazily initialize an
	// application using various configurations.
	AppCreator[T Application] func(log.Logger, corestore.KVStoreWithBatch, io.Writer, AppOptions) T

	// ExportedApp represents an exported app state, along with
	// validators, consensus params and latest app height.
	ExportedApp struct {
		// AppState is the application state as JSON.
		AppState json.RawMessage
		// Validators is the exported validator set.
		Validators []sdk.GenesisValidator
		// Height is the app's latest block height.
		Height int64
		// ConsensusParams are the exported consensus params for ABCI.
		ConsensusParams cmtproto.ConsensusParams
	}

	// AppExporter is a function that dumps all app state to
	// JSON-serializable structure and returns the current validator set.
	AppExporter func(
		logger log.Logger,
		db corestore.KVStoreWithBatch,
		traceWriter io.Writer,
		height int64,
		forZeroHeight bool,
		jailAllowedAddrs []string,
		opts AppOptions,
		modulesToExport []string,
	) (ExportedApp, error)
)
