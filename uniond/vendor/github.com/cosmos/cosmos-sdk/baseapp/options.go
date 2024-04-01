package baseapp

import (
	"fmt"
	"io"
	"math"

	dbm "github.com/cosmos/cosmos-db"

	"cosmossdk.io/store/metrics"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/snapshots"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp/oe"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/mempool"
)

// File for storing in-package BaseApp optional functions,
// for options that need access to non-exported fields of the BaseApp

// SetPruning sets a pruning option on the multistore associated with the app
func SetPruning(opts pruningtypes.PruningOptions) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.cms.SetPruning(opts) }
}

// SetMinGasPrices returns an option that sets the minimum gas prices on the app.
func SetMinGasPrices(gasPricesStr string) func(*BaseApp) {
	gasPrices, err := sdk.ParseDecCoins(gasPricesStr)
	if err != nil {
		panic(fmt.Sprintf("invalid minimum gas prices: %v", err))
	}

	return func(bapp *BaseApp) { bapp.setMinGasPrices(gasPrices) }
}

// SetQueryGasLimit returns an option that sets a gas limit for queries.
func SetQueryGasLimit(queryGasLimit uint64) func(*BaseApp) {
	if queryGasLimit == 0 {
		queryGasLimit = math.MaxUint64
	}

	return func(bapp *BaseApp) { bapp.queryGasLimit = queryGasLimit }
}

// SetHaltHeight returns a BaseApp option function that sets the halt block height.
func SetHaltHeight(blockHeight uint64) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.setHaltHeight(blockHeight) }
}

// SetHaltTime returns a BaseApp option function that sets the halt block time.
func SetHaltTime(haltTime uint64) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.setHaltTime(haltTime) }
}

// SetMinRetainBlocks returns a BaseApp option function that sets the minimum
// block retention height value when determining which heights to prune during
// ABCI Commit.
func SetMinRetainBlocks(minRetainBlocks uint64) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.setMinRetainBlocks(minRetainBlocks) }
}

// SetTrace will turn on or off trace flag
func SetTrace(trace bool) func(*BaseApp) {
	return func(app *BaseApp) { app.setTrace(trace) }
}

// SetIndexEvents provides a BaseApp option function that sets the events to index.
func SetIndexEvents(ie []string) func(*BaseApp) {
	return func(app *BaseApp) { app.setIndexEvents(ie) }
}

// SetIAVLCacheSize provides a BaseApp option function that sets the size of IAVL cache.
func SetIAVLCacheSize(size int) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.cms.SetIAVLCacheSize(size) }
}

// SetIAVLDisableFastNode enables(false)/disables(true) fast node usage from the IAVL store.
func SetIAVLDisableFastNode(disable bool) func(*BaseApp) {
	return func(bapp *BaseApp) { bapp.cms.SetIAVLDisableFastNode(disable) }
}

// SetInterBlockCache provides a BaseApp option function that sets the
// inter-block cache.
func SetInterBlockCache(cache storetypes.MultiStorePersistentCache) func(*BaseApp) {
	return func(app *BaseApp) { app.setInterBlockCache(cache) }
}

// SetSnapshot sets the snapshot store.
func SetSnapshot(snapshotStore *snapshots.Store, opts snapshottypes.SnapshotOptions) func(*BaseApp) {
	return func(app *BaseApp) { app.SetSnapshot(snapshotStore, opts) }
}

// SetMempool sets the mempool on BaseApp.
func SetMempool(mempool mempool.Mempool) func(*BaseApp) {
	return func(app *BaseApp) { app.SetMempool(mempool) }
}

// SetChainID sets the chain ID in BaseApp.
func SetChainID(chainID string) func(*BaseApp) {
	return func(app *BaseApp) { app.chainID = chainID }
}

// SetStoreLoader allows customization of the rootMultiStore initialization.
func SetStoreLoader(loader StoreLoader) func(*BaseApp) {
	return func(app *BaseApp) { app.SetStoreLoader(loader) }
}

// SetOptimisticExecution enables optimistic execution.
func SetOptimisticExecution(opts ...func(*oe.OptimisticExecution)) func(*BaseApp) {
	return func(app *BaseApp) {
		app.optimisticExec = oe.NewOptimisticExecution(app.logger, app.internalFinalizeBlock, opts...)
	}
}

// DisableBlockGasMeter disables the block gas meter.
func DisableBlockGasMeter() func(*BaseApp) {
	return func(app *BaseApp) { app.SetDisableBlockGasMeter(true) }
}

func (app *BaseApp) SetName(name string) {
	if app.sealed {
		panic("SetName() on sealed BaseApp")
	}

	app.name = name
}

// SetParamStore sets a parameter store on the BaseApp.
func (app *BaseApp) SetParamStore(ps ParamStore) {
	if app.sealed {
		panic("SetParamStore() on sealed BaseApp")
	}

	app.paramStore = ps
}

// SetVersion sets the application's version string.
func (app *BaseApp) SetVersion(v string) {
	if app.sealed {
		panic("SetVersion() on sealed BaseApp")
	}
	app.version = v
}

// SetProtocolVersion sets the application's protocol version
func (app *BaseApp) SetProtocolVersion(v uint64) {
	app.appVersion = v
}

func (app *BaseApp) SetDB(db dbm.DB) {
	if app.sealed {
		panic("SetDB() on sealed BaseApp")
	}

	app.db = db
}

func (app *BaseApp) SetCMS(cms storetypes.CommitMultiStore) {
	if app.sealed {
		panic("SetEndBlocker() on sealed BaseApp")
	}

	app.cms = cms
}

func (app *BaseApp) SetInitChainer(initChainer sdk.InitChainer) {
	if app.sealed {
		panic("SetInitChainer() on sealed BaseApp")
	}

	app.initChainer = initChainer
}

func (app *BaseApp) PreBlocker() sdk.PreBlocker {
	return app.preBlocker
}

func (app *BaseApp) SetPreBlocker(preBlocker sdk.PreBlocker) {
	if app.sealed {
		panic("SetPreBlocker() on sealed BaseApp")
	}

	app.preBlocker = preBlocker
}

func (app *BaseApp) SetBeginBlocker(beginBlocker sdk.BeginBlocker) {
	if app.sealed {
		panic("SetBeginBlocker() on sealed BaseApp")
	}

	app.beginBlocker = beginBlocker
}

func (app *BaseApp) SetEndBlocker(endBlocker sdk.EndBlocker) {
	if app.sealed {
		panic("SetEndBlocker() on sealed BaseApp")
	}

	app.endBlocker = endBlocker
}

func (app *BaseApp) SetPrepareCheckStater(prepareCheckStater sdk.PrepareCheckStater) {
	if app.sealed {
		panic("SetPrepareCheckStater() on sealed BaseApp")
	}

	app.prepareCheckStater = prepareCheckStater
}

func (app *BaseApp) SetPrecommiter(precommiter sdk.Precommiter) {
	if app.sealed {
		panic("SetPrecommiter() on sealed BaseApp")
	}

	app.precommiter = precommiter
}

func (app *BaseApp) SetAnteHandler(ah sdk.AnteHandler) {
	if app.sealed {
		panic("SetAnteHandler() on sealed BaseApp")
	}

	app.anteHandler = ah
}

func (app *BaseApp) SetPostHandler(ph sdk.PostHandler) {
	if app.sealed {
		panic("SetPostHandler() on sealed BaseApp")
	}

	app.postHandler = ph
}

func (app *BaseApp) SetAddrPeerFilter(pf sdk.PeerFilter) {
	if app.sealed {
		panic("SetAddrPeerFilter() on sealed BaseApp")
	}

	app.addrPeerFilter = pf
}

func (app *BaseApp) SetIDPeerFilter(pf sdk.PeerFilter) {
	if app.sealed {
		panic("SetIDPeerFilter() on sealed BaseApp")
	}

	app.idPeerFilter = pf
}

func (app *BaseApp) SetFauxMerkleMode() {
	if app.sealed {
		panic("SetFauxMerkleMode() on sealed BaseApp")
	}

	app.fauxMerkleMode = true
}

// SetCommitMultiStoreTracer sets the store tracer on the BaseApp's underlying
// CommitMultiStore.
func (app *BaseApp) SetCommitMultiStoreTracer(w io.Writer) {
	app.cms.SetTracer(w)
}

// SetStoreLoader allows us to customize the rootMultiStore initialization.
func (app *BaseApp) SetStoreLoader(loader StoreLoader) {
	if app.sealed {
		panic("SetStoreLoader() on sealed BaseApp")
	}

	app.storeLoader = loader
}

// SetSnapshot sets the snapshot store and options.
func (app *BaseApp) SetSnapshot(snapshotStore *snapshots.Store, opts snapshottypes.SnapshotOptions) {
	if app.sealed {
		panic("SetSnapshot() on sealed BaseApp")
	}
	if snapshotStore == nil {
		app.snapshotManager = nil
		return
	}
	app.cms.SetSnapshotInterval(opts.Interval)
	app.snapshotManager = snapshots.NewManager(snapshotStore, opts, app.cms, nil, app.logger)
}

// SetInterfaceRegistry sets the InterfaceRegistry.
func (app *BaseApp) SetInterfaceRegistry(registry types.InterfaceRegistry) {
	app.interfaceRegistry = registry
	app.grpcQueryRouter.SetInterfaceRegistry(registry)
	app.msgServiceRouter.SetInterfaceRegistry(registry)
	app.cdc = codec.NewProtoCodec(registry)
}

// SetTxDecoder sets the TxDecoder if it wasn't provided in the BaseApp constructor.
func (app *BaseApp) SetTxDecoder(txDecoder sdk.TxDecoder) {
	app.txDecoder = txDecoder
}

// SetTxEncoder sets the TxEncoder if it wasn't provided in the BaseApp constructor.
func (app *BaseApp) SetTxEncoder(txEncoder sdk.TxEncoder) {
	app.txEncoder = txEncoder
}

// SetQueryMultiStore set a alternative MultiStore implementation to support grpc query service.
//
// Ref: https://github.com/cosmos/cosmos-sdk/issues/13317
func (app *BaseApp) SetQueryMultiStore(ms storetypes.MultiStore) {
	app.qms = ms
}

// SetMempool sets the mempool for the BaseApp and is required for the app to start up.
func (app *BaseApp) SetMempool(mempool mempool.Mempool) {
	if app.sealed {
		panic("SetMempool() on sealed BaseApp")
	}
	app.mempool = mempool
}

// SetProcessProposal sets the process proposal function for the BaseApp.
func (app *BaseApp) SetProcessProposal(handler sdk.ProcessProposalHandler) {
	if app.sealed {
		panic("SetProcessProposal() on sealed BaseApp")
	}
	app.processProposal = handler
}

// SetPrepareProposal sets the prepare proposal function for the BaseApp.
func (app *BaseApp) SetPrepareProposal(handler sdk.PrepareProposalHandler) {
	if app.sealed {
		panic("SetPrepareProposal() on sealed BaseApp")
	}

	app.prepareProposal = handler
}

func (app *BaseApp) SetExtendVoteHandler(handler sdk.ExtendVoteHandler) {
	if app.sealed {
		panic("SetExtendVoteHandler() on sealed BaseApp")
	}

	app.extendVote = handler
}

func (app *BaseApp) SetVerifyVoteExtensionHandler(handler sdk.VerifyVoteExtensionHandler) {
	if app.sealed {
		panic("SetVerifyVoteExtensionHandler() on sealed BaseApp")
	}

	app.verifyVoteExt = handler
}

// SetStoreMetrics sets the prepare proposal function for the BaseApp.
func (app *BaseApp) SetStoreMetrics(gatherer metrics.StoreMetrics) {
	if app.sealed {
		panic("SetStoreMetrics() on sealed BaseApp")
	}

	app.cms.SetMetrics(gatherer)
}

// SetStreamingManager sets the streaming manager for the BaseApp.
func (app *BaseApp) SetStreamingManager(manager storetypes.StreamingManager) {
	app.streamingManager = manager
}

// SetDisableBlockGasMeter sets the disableBlockGasMeter flag for the BaseApp.
func (app *BaseApp) SetDisableBlockGasMeter(disableBlockGasMeter bool) {
	app.disableBlockGasMeter = disableBlockGasMeter
}
