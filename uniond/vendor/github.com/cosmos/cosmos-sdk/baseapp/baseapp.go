package baseapp

import (
	"errors"
	"fmt"
	"sort"
	"strings"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/gogoproto/proto"
	"golang.org/x/exp/maps"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/snapshots"
	"github.com/cosmos/cosmos-sdk/store"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/mempool"
)

type (
	// Enum mode for app.runTx
	runTxMode uint8

	// StoreLoader defines a customizable function to control how we load the CommitMultiStore
	// from disk. This is useful for state migration, when loading a datastore written with
	// an older version of the software. In particular, if a module changed the substore key name
	// (or removed a substore) between two versions of the software.
	StoreLoader func(ms storetypes.CommitMultiStore) error
)

const (
	runTxModeCheck       runTxMode = iota // Check a transaction
	runTxModeReCheck                      // Recheck a (pending) transaction after a commit
	runTxModeSimulate                     // Simulate a transaction
	runTxModeDeliver                      // Deliver a transaction
	runTxPrepareProposal                  // Prepare a TM block proposal
	runTxProcessProposal                  // Process a TM block proposal
)

var _ abci.Application = (*BaseApp)(nil)

// BaseApp reflects the ABCI application implementation.
type BaseApp struct { //nolint: maligned
	// initialized on creation
	logger            log.Logger
	name              string               // application name from abci.Info
	db                dbm.DB               // common DB backend
	cms               sdk.CommitMultiStore // Main (uncached) state
	qms               sdk.MultiStore       // Optional alternative multistore for querying only.
	storeLoader       StoreLoader          // function to handle store loading, may be overridden with SetStoreLoader()
	grpcQueryRouter   *GRPCQueryRouter     // router for redirecting gRPC query calls
	msgServiceRouter  *MsgServiceRouter    // router for redirecting Msg service messages
	interfaceRegistry codectypes.InterfaceRegistry
	txDecoder         sdk.TxDecoder // unmarshal []byte into sdk.Tx
	txEncoder         sdk.TxEncoder // marshal sdk.Tx into []byte

	mempool         mempool.Mempool            // application side mempool
	anteHandler     sdk.AnteHandler            // ante handler for fee and auth
	postHandler     sdk.PostHandler            // post handler, optional, e.g. for tips
	initChainer     sdk.InitChainer            // initialize state with validators and state blob
	beginBlocker    sdk.BeginBlocker           // logic to run before any txs
	processProposal sdk.ProcessProposalHandler // the handler which runs on ABCI ProcessProposal
	prepareProposal sdk.PrepareProposalHandler // the handler which runs on ABCI PrepareProposal
	endBlocker      sdk.EndBlocker             // logic to run after all txs, and to determine valset changes
	addrPeerFilter  sdk.PeerFilter             // filter peers by address and port
	idPeerFilter    sdk.PeerFilter             // filter peers by node ID
	fauxMerkleMode  bool                       // if true, IAVL MountStores uses MountStoresDB for simulation speed.

	// manages snapshots, i.e. dumps of app state at certain intervals
	snapshotManager *snapshots.Manager

	// volatile states:
	//
	// checkState is set on InitChain and reset on Commit
	// deliverState is set on InitChain and BeginBlock and set to nil on Commit
	checkState           *state // for CheckTx
	deliverState         *state // for DeliverTx
	processProposalState *state // for ProcessProposal
	prepareProposalState *state // for PrepareProposal

	// an inter-block write-through cache provided to the context during deliverState
	interBlockCache sdk.MultiStorePersistentCache

	// absent validators from begin block
	voteInfos []abci.VoteInfo

	// paramStore is used to query for ABCI consensus parameters from an
	// application parameter store.
	paramStore ParamStore

	// The minimum gas prices a validator is willing to accept for processing a
	// transaction. This is mainly used for DoS and spam prevention.
	minGasPrices sdk.DecCoins

	// initialHeight is the initial height at which we start the baseapp
	initialHeight int64

	// flag for sealing options and parameters to a BaseApp
	sealed bool

	// block height at which to halt the chain and gracefully shutdown
	haltHeight uint64

	// minimum block time (in Unix seconds) at which to halt the chain and gracefully shutdown
	haltTime uint64

	// minRetainBlocks defines the minimum block height offset from the current
	// block being committed, such that all blocks past this offset are pruned
	// from Tendermint. It is used as part of the process of determining the
	// ResponseCommit.RetainHeight value during ABCI Commit. A value of 0 indicates
	// that no blocks should be pruned.
	//
	// Note: Tendermint block pruning is dependant on this parameter in conunction
	// with the unbonding (safety threshold) period, state pruning and state sync
	// snapshot parameters to determine the correct minimum value of
	// ResponseCommit.RetainHeight.
	minRetainBlocks uint64

	// application's version string
	version string

	// application's protocol version that increments on every upgrade
	// if BaseApp is passed to the upgrade keeper's NewKeeper method.
	appVersion uint64

	// recovery handler for app.runTx method
	runTxRecoveryMiddleware recoveryMiddleware

	// trace set will return full stack traces for errors in ABCI Log field
	trace bool

	// indexEvents defines the set of events in the form {eventType}.{attributeKey},
	// which informs Tendermint what to index. If empty, all events will be indexed.
	indexEvents map[string]struct{}

	// abciListeners for hooking into the ABCI message processing of the BaseApp
	// and exposing the requests and responses to external consumers
	abciListeners []ABCIListener

	chainID string
}

// NewBaseApp returns a reference to an initialized BaseApp. It accepts a
// variadic number of option functions, which act on the BaseApp to set
// configuration choices.
//
// NOTE: The db is used to store the version number for now.
func NewBaseApp(
	name string, logger log.Logger, db dbm.DB, txDecoder sdk.TxDecoder, options ...func(*BaseApp),
) *BaseApp {
	app := &BaseApp{
		logger:           logger,
		name:             name,
		db:               db,
		cms:              store.NewCommitMultiStore(db),
		storeLoader:      DefaultStoreLoader,
		grpcQueryRouter:  NewGRPCQueryRouter(),
		msgServiceRouter: NewMsgServiceRouter(),
		txDecoder:        txDecoder,
		fauxMerkleMode:   false,
	}

	for _, option := range options {
		option(app)
	}

	if app.mempool == nil {
		app.SetMempool(mempool.NoOpMempool{})
	}

	abciProposalHandler := NewDefaultProposalHandler(app.mempool, app)

	if app.prepareProposal == nil {
		app.SetPrepareProposal(abciProposalHandler.PrepareProposalHandler())
	}

	if app.processProposal == nil {
		app.SetProcessProposal(abciProposalHandler.ProcessProposalHandler())
	}

	if app.interBlockCache != nil {
		app.cms.SetInterBlockCache(app.interBlockCache)
	}

	app.runTxRecoveryMiddleware = newDefaultRecoveryMiddleware()

	return app
}

// Name returns the name of the BaseApp.
func (app *BaseApp) Name() string {
	return app.name
}

// AppVersion returns the application's protocol version.
func (app *BaseApp) AppVersion() uint64 {
	return app.appVersion
}

// Version returns the application's version string.
func (app *BaseApp) Version() string {
	return app.version
}

// Logger returns the logger of the BaseApp.
func (app *BaseApp) Logger() log.Logger {
	return app.logger
}

// Trace returns the boolean value for logging error stack traces.
func (app *BaseApp) Trace() bool {
	return app.trace
}

// MsgServiceRouter returns the MsgServiceRouter of a BaseApp.
func (app *BaseApp) MsgServiceRouter() *MsgServiceRouter { return app.msgServiceRouter }

// SetMsgServiceRouter sets the MsgServiceRouter of a BaseApp.
func (app *BaseApp) SetMsgServiceRouter(msgServiceRouter *MsgServiceRouter) {
	app.msgServiceRouter = msgServiceRouter
}

// SetCircuitBreaker sets the circuit breaker for the BaseApp.
// The circuit breaker is checked on every message execution to verify if a transaction should be executed or not.
func (app *BaseApp) SetCircuitBreaker(cb CircuitBreaker) {
	if app.msgServiceRouter == nil {
		panic("must be called after message server is set")
	}
	app.msgServiceRouter.SetCircuit(cb)
}

// MountStores mounts all IAVL or DB stores to the provided keys in the BaseApp
// multistore.
func (app *BaseApp) MountStores(keys ...storetypes.StoreKey) {
	for _, key := range keys {
		switch key.(type) {
		case *storetypes.KVStoreKey:
			if !app.fauxMerkleMode {
				app.MountStore(key, storetypes.StoreTypeIAVL)
			} else {
				// StoreTypeDB doesn't do anything upon commit, and it doesn't
				// retain history, but it's useful for faster simulation.
				app.MountStore(key, storetypes.StoreTypeDB)
			}

		case *storetypes.TransientStoreKey:
			app.MountStore(key, storetypes.StoreTypeTransient)

		case *storetypes.MemoryStoreKey:
			app.MountStore(key, storetypes.StoreTypeMemory)

		default:
			panic(fmt.Sprintf("Unrecognized store key type :%T", key))
		}
	}
}

// MountKVStores mounts all IAVL or DB stores to the provided keys in the
// BaseApp multistore.
func (app *BaseApp) MountKVStores(keys map[string]*storetypes.KVStoreKey) {
	for _, key := range keys {
		if !app.fauxMerkleMode {
			app.MountStore(key, storetypes.StoreTypeIAVL)
		} else {
			// StoreTypeDB doesn't do anything upon commit, and it doesn't
			// retain history, but it's useful for faster simulation.
			app.MountStore(key, storetypes.StoreTypeDB)
		}
	}
}

// MountTransientStores mounts all transient stores to the provided keys in
// the BaseApp multistore.
func (app *BaseApp) MountTransientStores(keys map[string]*storetypes.TransientStoreKey) {
	for _, key := range keys {
		app.MountStore(key, storetypes.StoreTypeTransient)
	}
}

// MountMemoryStores mounts all in-memory KVStores with the BaseApp's internal
// commit multi-store.
func (app *BaseApp) MountMemoryStores(keys map[string]*storetypes.MemoryStoreKey) {
	skeys := maps.Keys(keys)
	sort.Strings(skeys)
	for _, key := range skeys {
		memKey := keys[key]
		app.MountStore(memKey, storetypes.StoreTypeMemory)
	}
}

// MountStore mounts a store to the provided key in the BaseApp multistore,
// using the default DB.
func (app *BaseApp) MountStore(key storetypes.StoreKey, typ storetypes.StoreType) {
	app.cms.MountStoreWithDB(key, typ, nil)
}

// LoadLatestVersion loads the latest application version. It will panic if
// called more than once on a running BaseApp.
func (app *BaseApp) LoadLatestVersion() error {
	err := app.storeLoader(app.cms)
	if err != nil {
		return fmt.Errorf("failed to load latest version: %w", err)
	}

	return app.Init()
}

// DefaultStoreLoader will be used by default and loads the latest version
func DefaultStoreLoader(ms sdk.CommitMultiStore) error {
	return ms.LoadLatestVersion()
}

// CommitMultiStore returns the root multi-store.
// App constructor can use this to access the `cms`.
// UNSAFE: must not be used during the abci life cycle.
func (app *BaseApp) CommitMultiStore() sdk.CommitMultiStore {
	return app.cms
}

// SnapshotManager returns the snapshot manager.
// application use this to register extra extension snapshotters.
func (app *BaseApp) SnapshotManager() *snapshots.Manager {
	return app.snapshotManager
}

// LoadVersion loads the BaseApp application version. It will panic if called
// more than once on a running baseapp.
func (app *BaseApp) LoadVersion(version int64) error {
	app.logger.Info("NOTICE: this could take a long time to migrate IAVL store to fastnode if you enable Fast Node.\n")
	err := app.cms.LoadVersion(version)
	if err != nil {
		return fmt.Errorf("failed to load version %d: %w", version, err)
	}

	return app.Init()
}

// LastCommitID returns the last CommitID of the multistore.
func (app *BaseApp) LastCommitID() storetypes.CommitID {
	return app.cms.LastCommitID()
}

// LastBlockHeight returns the last committed block height.
func (app *BaseApp) LastBlockHeight() int64 {
	return app.cms.LastCommitID().Version
}

// Init initializes the app. It seals the app, preventing any
// further modifications. In addition, it validates the app against
// the earlier provided settings. Returns an error if validation fails.
// nil otherwise. Panics if the app is already sealed.
func (app *BaseApp) Init() error {
	if app.sealed {
		panic("cannot call initFromMainStore: baseapp already sealed")
	}

	emptyHeader := tmproto.Header{ChainID: app.chainID}

	// needed for the export command which inits from store but never calls initchain
	app.setState(runTxModeCheck, emptyHeader)
	app.Seal()

	if app.cms == nil {
		return errors.New("commit multi-store must not be nil")
	}

	return app.cms.GetPruning().Validate()
}

func (app *BaseApp) setMinGasPrices(gasPrices sdk.DecCoins) {
	app.minGasPrices = gasPrices
}

func (app *BaseApp) setHaltHeight(haltHeight uint64) {
	app.haltHeight = haltHeight
}

func (app *BaseApp) setHaltTime(haltTime uint64) {
	app.haltTime = haltTime
}

func (app *BaseApp) setMinRetainBlocks(minRetainBlocks uint64) {
	app.minRetainBlocks = minRetainBlocks
}

func (app *BaseApp) setInterBlockCache(cache sdk.MultiStorePersistentCache) {
	app.interBlockCache = cache
}

func (app *BaseApp) setTrace(trace bool) {
	app.trace = trace
}

func (app *BaseApp) setIndexEvents(ie []string) {
	app.indexEvents = make(map[string]struct{})

	for _, e := range ie {
		app.indexEvents[e] = struct{}{}
	}
}

// Seal seals a BaseApp. It prohibits any further modifications to a BaseApp.
func (app *BaseApp) Seal() { app.sealed = true }

// IsSealed returns true if the BaseApp is sealed and false otherwise.
func (app *BaseApp) IsSealed() bool { return app.sealed }

// setState sets the BaseApp's state for the corresponding mode with a branched
// multi-store (i.e. a CacheMultiStore) and a new Context with the same
// multi-store branch, and provided header.
func (app *BaseApp) setState(mode runTxMode, header tmproto.Header) {
	ms := app.cms.CacheMultiStore()
	baseState := &state{
		ms:  ms,
		ctx: sdk.NewContext(ms, header, false, app.logger),
	}

	switch mode {
	case runTxModeCheck:
		// Minimum gas prices are also set. It is set on InitChain and reset on Commit.
		baseState.ctx = baseState.ctx.WithIsCheckTx(true).WithMinGasPrices(app.minGasPrices)
		app.checkState = baseState
	case runTxModeDeliver:
		// It is set on InitChain and BeginBlock and set to nil on Commit.
		app.deliverState = baseState
	case runTxPrepareProposal:
		// It is set on InitChain and Commit.
		app.prepareProposalState = baseState
	case runTxProcessProposal:
		// It is set on InitChain and Commit.
		app.processProposalState = baseState
	default:
		panic(fmt.Sprintf("invalid runTxMode for setState: %d", mode))
	}
}

// GetConsensusParams returns the current consensus parameters from the BaseApp's
// ParamStore. If the BaseApp has no ParamStore defined, nil is returned.
func (app *BaseApp) GetConsensusParams(ctx sdk.Context) *tmproto.ConsensusParams {
	if app.paramStore == nil {
		return nil
	}

	cp, err := app.paramStore.Get(ctx)
	if err != nil {
		panic(err)
	}

	return cp
}

// StoreConsensusParams sets the consensus parameters to the baseapp's param store.
func (app *BaseApp) StoreConsensusParams(ctx sdk.Context, cp *tmproto.ConsensusParams) {
	if app.paramStore == nil {
		panic("cannot store consensus params with no params store set")
	}

	if cp == nil {
		return
	}

	app.paramStore.Set(ctx, cp)
	// We're explicitly not storing the Tendermint app_version in the param store. It's
	// stored instead in the x/upgrade store, with its own bump logic.
}

// AddRunTxRecoveryHandler adds custom app.runTx method panic handlers.
func (app *BaseApp) AddRunTxRecoveryHandler(handlers ...RecoveryHandler) {
	for _, h := range handlers {
		app.runTxRecoveryMiddleware = newRecoveryMiddleware(h, app.runTxRecoveryMiddleware)
	}
}

// GetMaximumBlockGas gets the maximum gas from the consensus params. It panics
// if maximum block gas is less than negative one and returns zero if negative
// one.
func (app *BaseApp) GetMaximumBlockGas(ctx sdk.Context) uint64 {
	cp := app.GetConsensusParams(ctx)
	if cp == nil || cp.Block == nil {
		return 0
	}

	maxGas := cp.Block.MaxGas

	switch {
	case maxGas < -1:
		panic(fmt.Sprintf("invalid maximum block gas: %d", maxGas))

	case maxGas == -1:
		return 0

	default:
		return uint64(maxGas)
	}
}

func (app *BaseApp) validateHeight(req abci.RequestBeginBlock) error {
	if req.Header.Height < 1 {
		return fmt.Errorf("invalid height: %d", req.Header.Height)
	}

	lastBlockHeight := app.LastBlockHeight()

	// expectedHeight holds the expected height to validate
	var expectedHeight int64
	if lastBlockHeight == 0 && app.initialHeight > 1 {
		// In this case, we're validating the first block of the chain, i.e no
		// previous commit. The height we're expecting is the initial height.
		expectedHeight = app.initialHeight
	} else {
		// This case can mean two things:
		//
		// - Either there was already a previous commit in the store, in which
		// case we increment the version from there.
		// - Or there was no previous commit, in which case we start at version 1.
		expectedHeight = lastBlockHeight + 1
	}

	if req.Header.Height != expectedHeight {
		return fmt.Errorf("invalid height: %d; expected: %d", req.Header.Height, expectedHeight)
	}

	return nil
}

// validateBasicTxMsgs executes basic validator calls for messages.
func validateBasicTxMsgs(msgs []sdk.Msg) error {
	if len(msgs) == 0 {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "must contain at least one message")
	}

	for _, msg := range msgs {
		err := msg.ValidateBasic()
		if err != nil {
			return err
		}
	}

	return nil
}

// Returns the application's deliverState if app is in runTxModeDeliver,
// prepareProposalState if app is in runTxPrepareProposal, processProposalState
// if app is in runTxProcessProposal, and checkState otherwise.
func (app *BaseApp) getState(mode runTxMode) *state {
	switch mode {
	case runTxModeDeliver:
		return app.deliverState

	case runTxPrepareProposal:
		return app.prepareProposalState

	case runTxProcessProposal:
		return app.processProposalState

	default:
		return app.checkState
	}
}

func (app *BaseApp) getBlockGasMeter(ctx sdk.Context) storetypes.GasMeter {
	if maxGas := app.GetMaximumBlockGas(ctx); maxGas > 0 {
		return storetypes.NewGasMeter(maxGas)
	}

	return storetypes.NewInfiniteGasMeter()
}

// retrieve the context for the tx w/ txBytes and other memoized values.
func (app *BaseApp) getContextForTx(mode runTxMode, txBytes []byte) sdk.Context {
	modeState := app.getState(mode)
	if modeState == nil {
		panic(fmt.Sprintf("state is nil for mode %v", mode))
	}
	ctx := modeState.ctx.
		WithTxBytes(txBytes).
		WithVoteInfos(app.voteInfos)

	ctx = ctx.WithConsensusParams(app.GetConsensusParams(ctx))

	if mode == runTxModeReCheck {
		ctx = ctx.WithIsReCheckTx(true)
	}

	if mode == runTxModeSimulate {
		ctx, _ = ctx.CacheContext()
	}

	return ctx
}

// cacheTxContext returns a new context based off of the provided context with
// a branched multi-store.
func (app *BaseApp) cacheTxContext(ctx sdk.Context, txBytes []byte) (sdk.Context, sdk.CacheMultiStore) {
	ms := ctx.MultiStore()
	// TODO: https://github.com/cosmos/cosmos-sdk/issues/2824
	msCache := ms.CacheMultiStore()
	if msCache.TracingEnabled() {
		msCache = msCache.SetTracingContext(
			sdk.TraceContext(
				map[string]interface{}{
					"txHash": fmt.Sprintf("%X", tmhash.Sum(txBytes)),
				},
			),
		).(sdk.CacheMultiStore)
	}

	return ctx.WithMultiStore(msCache), msCache
}

// runTx processes a transaction within a given execution mode, encoded transaction
// bytes, and the decoded transaction itself. All state transitions occur through
// a cached Context depending on the mode provided. State only gets persisted
// if all messages get executed successfully and the execution mode is DeliverTx.
// Note, gas execution info is always returned. A reference to a Result is
// returned if the tx does not run out of gas and if all the messages are valid
// and execute successfully. An error is returned otherwise.
func (app *BaseApp) runTx(mode runTxMode, txBytes []byte) (gInfo sdk.GasInfo, result *sdk.Result, anteEvents []abci.Event, priority int64, err error) {
	// NOTE: GasWanted should be returned by the AnteHandler. GasUsed is
	// determined by the GasMeter. We need access to the context to get the gas
	// meter, so we initialize upfront.
	var gasWanted uint64

	ctx := app.getContextForTx(mode, txBytes)
	ms := ctx.MultiStore()

	// only run the tx if there is block gas remaining
	if mode == runTxModeDeliver && ctx.BlockGasMeter().IsOutOfGas() {
		return gInfo, nil, nil, 0, sdkerrors.Wrap(sdkerrors.ErrOutOfGas, "no block gas left to run tx")
	}

	defer func() {
		if r := recover(); r != nil {
			recoveryMW := newOutOfGasRecoveryMiddleware(gasWanted, ctx, app.runTxRecoveryMiddleware)
			err, result = processRecovery(r, recoveryMW), nil
		}

		gInfo = sdk.GasInfo{GasWanted: gasWanted, GasUsed: ctx.GasMeter().GasConsumed()}
	}()

	blockGasConsumed := false

	// consumeBlockGas makes sure block gas is consumed at most once. It must
	// happen after tx processing, and must be executed even if tx processing
	// fails. Hence, it's execution is deferred.
	consumeBlockGas := func() {
		if !blockGasConsumed {
			blockGasConsumed = true
			ctx.BlockGasMeter().ConsumeGas(
				ctx.GasMeter().GasConsumedToLimit(), "block gas meter",
			)
		}
	}

	// If BlockGasMeter() panics it will be caught by the above recover and will
	// return an error - in any case BlockGasMeter will consume gas past the limit.
	//
	// NOTE: consumeBlockGas must exist in a separate defer function from the
	// general deferred recovery function to recover from consumeBlockGas as it'll
	// be executed first (deferred statements are executed as stack).
	if mode == runTxModeDeliver {
		defer consumeBlockGas()
	}

	tx, err := app.txDecoder(txBytes)
	if err != nil {
		return sdk.GasInfo{}, nil, nil, 0, err
	}

	msgs := tx.GetMsgs()
	if err := validateBasicTxMsgs(msgs); err != nil {
		return sdk.GasInfo{}, nil, nil, 0, err
	}

	if app.anteHandler != nil {
		var (
			anteCtx sdk.Context
			msCache sdk.CacheMultiStore
		)

		// Branch context before AnteHandler call in case it aborts.
		// This is required for both CheckTx and DeliverTx.
		// Ref: https://github.com/cosmos/cosmos-sdk/issues/2772
		//
		// NOTE: Alternatively, we could require that AnteHandler ensures that
		// writes do not happen if aborted/failed.  This may have some
		// performance benefits, but it'll be more difficult to get right.
		anteCtx, msCache = app.cacheTxContext(ctx, txBytes)
		anteCtx = anteCtx.WithEventManager(sdk.NewEventManager())
		newCtx, err := app.anteHandler(anteCtx, tx, mode == runTxModeSimulate)

		if !newCtx.IsZero() {
			// At this point, newCtx.MultiStore() is a store branch, or something else
			// replaced by the AnteHandler. We want the original multistore.
			//
			// Also, in the case of the tx aborting, we need to track gas consumed via
			// the instantiated gas meter in the AnteHandler, so we update the context
			// prior to returning.
			ctx = newCtx.WithMultiStore(ms)
		}

		events := ctx.EventManager().Events()

		// GasMeter expected to be set in AnteHandler
		gasWanted = ctx.GasMeter().Limit()

		if err != nil {
			return gInfo, nil, nil, 0, err
		}

		priority = ctx.Priority()
		msCache.Write()
		anteEvents = events.ToABCIEvents()
	}

	if mode == runTxModeCheck {
		err = app.mempool.Insert(ctx, tx)
		if err != nil {
			return gInfo, nil, anteEvents, priority, err
		}
	} else if mode == runTxModeDeliver {
		err = app.mempool.Remove(tx)
		if err != nil && !errors.Is(err, mempool.ErrTxNotFound) {
			return gInfo, nil, anteEvents, priority,
				fmt.Errorf("failed to remove tx from mempool: %w", err)
		}
	}

	// Create a new Context based off of the existing Context with a MultiStore branch
	// in case message processing fails. At this point, the MultiStore
	// is a branch of a branch.
	runMsgCtx, msCache := app.cacheTxContext(ctx, txBytes)

	// Attempt to execute all messages and only update state if all messages pass
	// and we're in DeliverTx. Note, runMsgs will never return a reference to a
	// Result if any single message fails or does not have a registered Handler.
	result, err = app.runMsgs(runMsgCtx, msgs, mode)
	if err == nil {
		// Run optional postHandlers.
		//
		// Note: If the postHandler fails, we also revert the runMsgs state.
		if app.postHandler != nil {
			// The runMsgCtx context currently contains events emitted by the ante handler.
			// We clear this to correctly order events without duplicates.
			// Note that the state is still preserved.
			postCtx := runMsgCtx.WithEventManager(sdk.NewEventManager())

			newCtx, err := app.postHandler(postCtx, tx, mode == runTxModeSimulate, err == nil)
			if err != nil {
				return gInfo, nil, anteEvents, priority, err
			}

			result.Events = append(result.Events, newCtx.EventManager().ABCIEvents()...)
		}

		if mode == runTxModeDeliver {
			// When block gas exceeds, it'll panic and won't commit the cached store.
			consumeBlockGas()

			msCache.Write()
		}

		if len(anteEvents) > 0 && (mode == runTxModeDeliver || mode == runTxModeSimulate) {
			// append the events in the order of occurrence
			result.Events = append(anteEvents, result.Events...)
		}
	}

	return gInfo, result, anteEvents, priority, err
}

// runMsgs iterates through a list of messages and executes them with the provided
// Context and execution mode. Messages will only be executed during simulation
// and DeliverTx. An error is returned if any single message fails or if a
// Handler does not exist for a given message route. Otherwise, a reference to a
// Result is returned. The caller must not commit state if an error is returned.
func (app *BaseApp) runMsgs(ctx sdk.Context, msgs []sdk.Msg, mode runTxMode) (*sdk.Result, error) {
	msgLogs := make(sdk.ABCIMessageLogs, 0, len(msgs))
	events := sdk.EmptyEvents()
	var msgResponses []*codectypes.Any

	// NOTE: GasWanted is determined by the AnteHandler and GasUsed by the GasMeter.
	for i, msg := range msgs {
		if mode != runTxModeDeliver && mode != runTxModeSimulate {
			break
		}

		handler := app.msgServiceRouter.Handler(msg)
		if handler == nil {
			return nil, sdkerrors.Wrapf(sdkerrors.ErrUnknownRequest, "can't route message %+v", msg)
		}

		// ADR 031 request type routing
		msgResult, err := handler(ctx, msg)
		if err != nil {
			return nil, sdkerrors.Wrapf(err, "failed to execute message; message index: %d", i)
		}

		// create message events
		msgEvents := createEvents(msgResult.GetEvents(), msg)

		// append message events, data and logs
		//
		// Note: Each message result's data must be length-prefixed in order to
		// separate each result.
		events = events.AppendEvents(msgEvents)

		// Each individual sdk.Result that went through the MsgServiceRouter
		// (which should represent 99% of the Msgs now, since everyone should
		// be using protobuf Msgs) has exactly one Msg response, set inside
		// `WrapServiceResult`. We take that Msg response, and aggregate it
		// into an array.
		if len(msgResult.MsgResponses) > 0 {
			msgResponse := msgResult.MsgResponses[0]
			if msgResponse == nil {
				return nil, sdkerrors.ErrLogic.Wrapf("got nil Msg response at index %d for msg %s", i, sdk.MsgTypeURL(msg))
			}
			msgResponses = append(msgResponses, msgResponse)
		}

		msgLogs = append(msgLogs, sdk.NewABCIMessageLog(uint32(i), msgResult.Log, msgEvents))
	}

	data, err := makeABCIData(msgResponses)
	if err != nil {
		return nil, sdkerrors.Wrap(err, "failed to marshal tx data")
	}

	return &sdk.Result{
		Data:         data,
		Log:          strings.TrimSpace(msgLogs.String()),
		Events:       events.ToABCIEvents(),
		MsgResponses: msgResponses,
	}, nil
}

// makeABCIData generates the Data field to be sent to ABCI Check/DeliverTx.
func makeABCIData(msgResponses []*codectypes.Any) ([]byte, error) {
	return proto.Marshal(&sdk.TxMsgData{MsgResponses: msgResponses})
}

func createEvents(events sdk.Events, msg sdk.Msg) sdk.Events {
	eventMsgName := sdk.MsgTypeURL(msg)
	msgEvent := sdk.NewEvent(sdk.EventTypeMessage, sdk.NewAttribute(sdk.AttributeKeyAction, eventMsgName))

	// we set the signer attribute as the sender
	if len(msg.GetSigners()) > 0 && !msg.GetSigners()[0].Empty() {
		msgEvent = msgEvent.AppendAttributes(sdk.NewAttribute(sdk.AttributeKeySender, msg.GetSigners()[0].String()))
	}

	// verify that events have no module attribute set
	if _, found := events.GetAttributes(sdk.AttributeKeyModule); !found {
		// here we assume that routes module name is the second element of the route
		// e.g. "cosmos.bank.v1beta1.MsgSend" => "bank"
		moduleName := strings.Split(eventMsgName, ".")
		if len(moduleName) > 1 {
			msgEvent = msgEvent.AppendAttributes(sdk.NewAttribute(sdk.AttributeKeyModule, moduleName[1]))
		}
	}

	return sdk.Events{msgEvent}.AppendEvents(events)
}

// PrepareProposalVerifyTx performs transaction verification when a proposer is
// creating a block proposal during PrepareProposal. Any state committed to the
// PrepareProposal state internally will be discarded. <nil, err> will be
// returned if the transaction cannot be encoded. <bz, nil> will be returned if
// the transaction is valid, otherwise <bz, err> will be returned.
func (app *BaseApp) PrepareProposalVerifyTx(tx sdk.Tx) ([]byte, error) {
	bz, err := app.txEncoder(tx)
	if err != nil {
		return nil, err
	}

	_, _, _, _, err = app.runTx(runTxPrepareProposal, bz) //nolint:dogsled
	if err != nil {
		return nil, err
	}

	return bz, nil
}

// ProcessProposalVerifyTx performs transaction verification when receiving a
// block proposal during ProcessProposal. Any state committed to the
// ProcessProposal state internally will be discarded. <nil, err> will be
// returned if the transaction cannot be decoded. <Tx, nil> will be returned if
// the transaction is valid, otherwise <Tx, err> will be returned.
func (app *BaseApp) ProcessProposalVerifyTx(txBz []byte) (sdk.Tx, error) {
	tx, err := app.txDecoder(txBz)
	if err != nil {
		return nil, err
	}

	_, _, _, _, err = app.runTx(runTxProcessProposal, txBz) //nolint:dogsled
	if err != nil {
		return nil, err
	}

	return tx, nil
}

type (
	// ProposalTxVerifier defines the interface that is implemented by BaseApp,
	// that any custom ABCI PrepareProposal and ProcessProposal handler can use
	// to verify a transaction.
	ProposalTxVerifier interface {
		PrepareProposalVerifyTx(tx sdk.Tx) ([]byte, error)
		ProcessProposalVerifyTx(txBz []byte) (sdk.Tx, error)
	}

	// DefaultProposalHandler defines the default ABCI PrepareProposal and
	// ProcessProposal handlers.
	DefaultProposalHandler struct {
		mempool    mempool.Mempool
		txVerifier ProposalTxVerifier
	}
)

func NewDefaultProposalHandler(mp mempool.Mempool, txVerifier ProposalTxVerifier) DefaultProposalHandler {
	return DefaultProposalHandler{
		mempool:    mp,
		txVerifier: txVerifier,
	}
}

// PrepareProposalHandler returns the default implementation for processing an
// ABCI proposal. The application's mempool is enumerated and all valid
// transactions are added to the proposal. Transactions are valid if they:
//
// 1) Successfully encode to bytes.
// 2) Are valid (i.e. pass runTx, AnteHandler only).
//
// Enumeration is halted once RequestPrepareProposal.MaxBytes of transactions is
// reached or the mempool is exhausted.
//
// Note:
//
// - Step (2) is identical to the validation step performed in
// DefaultProcessProposal. It is very important that the same validation logic
// is used in both steps, and applications must ensure that this is the case in
// non-default handlers.
//
// - If no mempool is set or if the mempool is a no-op mempool, the transactions
// requested from Tendermint will simply be returned, which, by default, are in
// FIFO order.
func (h DefaultProposalHandler) PrepareProposalHandler() sdk.PrepareProposalHandler {
	return func(ctx sdk.Context, req abci.RequestPrepareProposal) abci.ResponsePrepareProposal {
		// If the mempool is nil or NoOp we simply return the transactions
		// requested from CometBFT, which, by default, should be in FIFO order.
		_, isNoOp := h.mempool.(mempool.NoOpMempool)
		if h.mempool == nil || isNoOp {
			return abci.ResponsePrepareProposal{Txs: req.Txs}
		}

		var maxBlockGas int64
		if b := ctx.ConsensusParams().Block; b != nil {
			maxBlockGas = b.MaxGas
		}

		var (
			selectedTxs  [][]byte
			totalTxBytes int64
			totalTxGas   uint64
		)

		iterator := h.mempool.Select(ctx, req.Txs)

		for iterator != nil {
			memTx := iterator.Tx()

			// NOTE: Since transaction verification was already executed in CheckTx,
			// which calls mempool.Insert, in theory everything in the pool should be
			// valid. But some mempool implementations may insert invalid txs, so we
			// check again.
			bz, err := h.txVerifier.PrepareProposalVerifyTx(memTx)
			if err != nil {
				err := h.mempool.Remove(memTx)
				if err != nil && !errors.Is(err, mempool.ErrTxNotFound) {
					panic(err)
				}
			} else {
				var txGasLimit uint64
				txSize := int64(len(bz))

				gasTx, ok := memTx.(interface{ GetGas() uint64 })
				if ok {
					txGasLimit = gasTx.GetGas()
				}

				// only add the transaction to the proposal if we have enough capacity
				if (txSize + totalTxBytes) < req.MaxTxBytes {
					// If there is a max block gas limit, add the tx only if the limit has
					// not been met.
					if maxBlockGas > 0 {
						if (txGasLimit + totalTxGas) <= uint64(maxBlockGas) {
							totalTxGas += txGasLimit
							totalTxBytes += txSize
							selectedTxs = append(selectedTxs, bz)
						}
					} else {
						totalTxBytes += txSize
						selectedTxs = append(selectedTxs, bz)
					}
				}

				// Check if we've reached capacity. If so, we cannot select any more
				// transactions.
				if totalTxBytes >= req.MaxTxBytes || (maxBlockGas > 0 && (totalTxGas >= uint64(maxBlockGas))) {
					break
				}
			}

			iterator = iterator.Next()
		}

		return abci.ResponsePrepareProposal{Txs: selectedTxs}
	}
}

// ProcessProposalHandler returns the default implementation for processing an
// ABCI proposal. Every transaction in the proposal must pass 2 conditions:
//
// 1. The transaction bytes must decode to a valid transaction.
// 2. The transaction must be valid (i.e. pass runTx, AnteHandler only)
//
// If any transaction fails to pass either condition, the proposal is rejected.
// Note that step (2) is identical to the validation step performed in
// DefaultPrepareProposal. It is very important that the same validation logic
// is used in both steps, and applications must ensure that this is the case in
// non-default handlers.
func (h DefaultProposalHandler) ProcessProposalHandler() sdk.ProcessProposalHandler {
	// If the mempool is nil or NoOp we simply return ACCEPT,
	// because PrepareProposal may have included txs that could fail verification.
	_, isNoOp := h.mempool.(mempool.NoOpMempool)
	if h.mempool == nil || isNoOp {
		return NoOpProcessProposal()
	}

	return func(ctx sdk.Context, req abci.RequestProcessProposal) abci.ResponseProcessProposal {
		for _, txBytes := range req.Txs {
			_, err := h.txVerifier.ProcessProposalVerifyTx(txBytes)
			if err != nil {
				return abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}
			}
		}

		return abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}
	}
}

// NoOpPrepareProposal defines a no-op PrepareProposal handler. It will always
// return the transactions sent by the client's request.
func NoOpPrepareProposal() sdk.PrepareProposalHandler {
	return func(_ sdk.Context, req abci.RequestPrepareProposal) abci.ResponsePrepareProposal {
		return abci.ResponsePrepareProposal{Txs: req.Txs}
	}
}

// NoOpProcessProposal defines a no-op ProcessProposal Handler. It will always
// return ACCEPT.
func NoOpProcessProposal() sdk.ProcessProposalHandler {
	return func(_ sdk.Context, _ abci.RequestProcessProposal) abci.ResponseProcessProposal {
		return abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}
	}
}

// Close is called in start cmd to gracefully cleanup resources.
func (app *BaseApp) Close() error {
	return nil
}
