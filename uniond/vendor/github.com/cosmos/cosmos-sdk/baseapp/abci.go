package baseapp

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"os"
	"sort"
	"strings"
	"syscall"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"

	"github.com/cosmos/cosmos-sdk/codec"
	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	"github.com/cosmos/cosmos-sdk/store/rootmulti"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Supported ABCI Query prefixes
const (
	QueryPathApp    = "app"
	QueryPathCustom = "custom"
	QueryPathP2P    = "p2p"
	QueryPathStore  = "store"
)

// InitChain implements the ABCI interface. It runs the initialization logic
// directly on the CommitMultiStore.
func (app *BaseApp) InitChain(req abci.RequestInitChain) (res abci.ResponseInitChain) {
	if req.ChainId != app.chainID {
		panic(fmt.Sprintf("invalid chain-id on InitChain; expected: %s, got: %s", app.chainID, req.ChainId))
	}

	// On a new chain, we consider the init chain block height as 0, even though
	// req.InitialHeight is 1 by default.
	initHeader := tmproto.Header{ChainID: req.ChainId, Time: req.Time}

	app.logger.Info("InitChain", "initialHeight", req.InitialHeight, "chainID", req.ChainId)

	// Set the initial height, which will be used to determine if we are proposing
	// or processing the first block or not.
	app.initialHeight = req.InitialHeight

	// if req.InitialHeight is > 1, then we set the initial version on all stores
	if req.InitialHeight > 1 {
		initHeader.Height = req.InitialHeight
		if err := app.cms.SetInitialVersion(req.InitialHeight); err != nil {
			panic(err)
		}
	}

	// initialize states with a correct header
	app.setState(runTxModeDeliver, initHeader)
	app.setState(runTxModeCheck, initHeader)

	// Store the consensus params in the BaseApp's paramstore. Note, this must be
	// done after the deliver state and context have been set as it's persisted
	// to state.
	if req.ConsensusParams != nil {
		app.StoreConsensusParams(app.deliverState.ctx, req.ConsensusParams)
	}

	if app.initChainer == nil {
		return
	}

	// add block gas meter for any genesis transactions (allow infinite gas)
	app.deliverState.ctx = app.deliverState.ctx.WithBlockGasMeter(sdk.NewInfiniteGasMeter())

	res = app.initChainer(app.deliverState.ctx, req)

	// sanity check
	if len(req.Validators) > 0 {
		if len(req.Validators) != len(res.Validators) {
			panic(
				fmt.Errorf(
					"len(RequestInitChain.Validators) != len(GenesisValidators) (%d != %d)",
					len(req.Validators), len(res.Validators),
				),
			)
		}

		sort.Sort(abci.ValidatorUpdates(req.Validators))
		sort.Sort(abci.ValidatorUpdates(res.Validators))

		for i := range res.Validators {
			if !proto.Equal(&res.Validators[i], &req.Validators[i]) {
				panic(fmt.Errorf("genesisValidators[%d] != req.Validators[%d] ", i, i))
			}
		}
	}

	// In the case of a new chain, AppHash will be the hash of an empty string.
	// During an upgrade, it'll be the hash of the last committed block.
	var appHash []byte
	if !app.LastCommitID().IsZero() {
		appHash = app.LastCommitID().Hash
	} else {
		// $ echo -n '' | sha256sum
		// e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
		emptyHash := sha256.Sum256([]byte{})
		appHash = emptyHash[:]
	}

	// NOTE: We don't commit, but BeginBlock for block `initial_height` starts from this
	// deliverState.
	return abci.ResponseInitChain{
		ConsensusParams: res.ConsensusParams,
		Validators:      res.Validators,
		AppHash:         appHash,
	}
}

// Info implements the ABCI interface.
func (app *BaseApp) Info(req abci.RequestInfo) abci.ResponseInfo {
	lastCommitID := app.cms.LastCommitID()

	return abci.ResponseInfo{
		Data:             app.name,
		Version:          app.version,
		AppVersion:       app.appVersion,
		LastBlockHeight:  lastCommitID.Version,
		LastBlockAppHash: lastCommitID.Hash,
	}
}

// FilterPeerByAddrPort filters peers by address/port.
func (app *BaseApp) FilterPeerByAddrPort(info string) abci.ResponseQuery {
	if app.addrPeerFilter != nil {
		return app.addrPeerFilter(info)
	}

	return abci.ResponseQuery{}
}

// FilterPeerByID filters peers by node ID.
func (app *BaseApp) FilterPeerByID(info string) abci.ResponseQuery {
	if app.idPeerFilter != nil {
		return app.idPeerFilter(info)
	}

	return abci.ResponseQuery{}
}

// BeginBlock implements the ABCI application interface.
func (app *BaseApp) BeginBlock(req abci.RequestBeginBlock) (res abci.ResponseBeginBlock) {
	if req.Header.ChainID != app.chainID {
		panic(fmt.Sprintf("invalid chain-id on BeginBlock; expected: %s, got: %s", app.chainID, req.Header.ChainID))
	}

	if app.cms.TracingEnabled() {
		app.cms.SetTracingContext(sdk.TraceContext(
			map[string]interface{}{"blockHeight": req.Header.Height},
		))
	}

	if err := app.validateHeight(req); err != nil {
		panic(err)
	}

	// Initialize the DeliverTx state. If this is the first block, it should
	// already be initialized in InitChain. Otherwise app.deliverState will be
	// nil, since it is reset on Commit.
	if app.deliverState == nil {
		app.setState(runTxModeDeliver, req.Header)
	} else {
		// In the first block, app.deliverState.ctx will already be initialized
		// by InitChain. Context is now updated with Header information.
		app.deliverState.ctx = app.deliverState.ctx.
			WithBlockHeader(req.Header).
			WithBlockHeight(req.Header.Height)
	}

	gasMeter := app.getBlockGasMeter(app.deliverState.ctx)

	app.deliverState.ctx = app.deliverState.ctx.
		WithBlockGasMeter(gasMeter).
		WithHeaderHash(req.Hash).
		WithConsensusParams(app.GetConsensusParams(app.deliverState.ctx))

	if app.checkState != nil {
		app.checkState.ctx = app.checkState.ctx.
			WithBlockGasMeter(gasMeter).
			WithHeaderHash(req.Hash)
	}

	if app.beginBlocker != nil {
		res = app.beginBlocker(app.deliverState.ctx, req)
		res.Events = sdk.MarkEventsToIndex(res.Events, app.indexEvents)
	}
	// set the signed validators for addition to context in deliverTx
	app.voteInfos = req.LastCommitInfo.GetVotes()

	// call the hooks with the BeginBlock messages
	for _, streamingListener := range app.abciListeners {
		if err := streamingListener.ListenBeginBlock(app.deliverState.ctx, req, res); err != nil {
			panic(fmt.Errorf("BeginBlock listening hook failed, height: %d, err: %w", req.Header.Height, err))
		}
	}

	return res
}

// EndBlock implements the ABCI interface.
func (app *BaseApp) EndBlock(req abci.RequestEndBlock) (res abci.ResponseEndBlock) {
	if app.deliverState.ms.TracingEnabled() {
		app.deliverState.ms = app.deliverState.ms.SetTracingContext(nil).(sdk.CacheMultiStore)
	}

	if app.endBlocker != nil {
		res = app.endBlocker(app.deliverState.ctx, req)
		res.Events = sdk.MarkEventsToIndex(res.Events, app.indexEvents)
	}

	if cp := app.GetConsensusParams(app.deliverState.ctx); cp != nil {
		res.ConsensusParamUpdates = cp
	}

	// call the streaming service hooks with the EndBlock messages
	for _, streamingListener := range app.abciListeners {
		if err := streamingListener.ListenEndBlock(app.deliverState.ctx, req, res); err != nil {
			panic(fmt.Errorf("EndBlock listening hook failed, height: %d, err: %w", req.Height, err))
		}
	}

	return res
}

// PrepareProposal implements the PrepareProposal ABCI method and returns a
// ResponsePrepareProposal object to the client. The PrepareProposal method is
// responsible for allowing the block proposer to perform application-dependent
// work in a block before proposing it.
//
// Transactions can be modified, removed, or added by the application. Since the
// application maintains its own local mempool, it will ignore the transactions
// provided to it in RequestPrepareProposal. Instead, it will determine which
// transactions to return based on the mempool's semantics and the MaxTxBytes
// provided by the client's request.
//
// Ref: https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-060-abci-1.0.md
// Ref: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
func (app *BaseApp) PrepareProposal(req abci.RequestPrepareProposal) (resp abci.ResponsePrepareProposal) {
	if app.prepareProposal == nil {
		panic("PrepareProposal method not set")
	}

	// always reset state given that PrepareProposal can timeout and be called again
	emptyHeader := tmproto.Header{ChainID: app.chainID}
	app.setState(runTxPrepareProposal, emptyHeader)

	// CometBFT must never call PrepareProposal with a height of 0.
	// Ref: https://github.com/cometbft/cometbft/blob/059798a4f5b0c9f52aa8655fa619054a0154088c/spec/core/state.md?plain=1#L37-L38
	if req.Height < 1 {
		panic("PrepareProposal called with invalid height")
	}

	app.prepareProposalState.ctx = app.getContextForProposal(app.prepareProposalState.ctx, req.Height).
		WithVoteInfos(app.voteInfos).
		WithBlockHeight(req.Height).
		WithBlockTime(req.Time).
		WithProposer(req.ProposerAddress)

	app.prepareProposalState.ctx = app.prepareProposalState.ctx.
		WithConsensusParams(app.GetConsensusParams(app.prepareProposalState.ctx)).
		WithBlockGasMeter(app.getBlockGasMeter(app.prepareProposalState.ctx))

	defer func() {
		if err := recover(); err != nil {
			app.logger.Error(
				"panic recovered in PrepareProposal",
				"height", req.Height,
				"time", req.Time,
				"panic", err,
			)

			resp = abci.ResponsePrepareProposal{Txs: req.Txs}
		}
	}()

	resp = app.prepareProposal(app.prepareProposalState.ctx, req)
	return resp
}

// ProcessProposal implements the ProcessProposal ABCI method and returns a
// ResponseProcessProposal object to the client. The ProcessProposal method is
// responsible for allowing execution of application-dependent work in a proposed
// block. Note, the application defines the exact implementation details of
// ProcessProposal. In general, the application must at the very least ensure
// that all transactions are valid. If all transactions are valid, then we inform
// Tendermint that the Status is ACCEPT. However, the application is also able
// to implement optimizations such as executing the entire proposed block
// immediately.
//
// If a panic is detected during execution of an application's ProcessProposal
// handler, it will be recovered and we will reject the proposal.
//
// Ref: https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-060-abci-1.0.md
// Ref: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
func (app *BaseApp) ProcessProposal(req abci.RequestProcessProposal) (resp abci.ResponseProcessProposal) {
	if app.processProposal == nil {
		panic("app.ProcessProposal is not set")
	}

	// CometBFT must never call ProcessProposal with a height of 0.
	// Ref: https://github.com/cometbft/cometbft/blob/059798a4f5b0c9f52aa8655fa619054a0154088c/spec/core/state.md?plain=1#L37-L38
	if req.Height < 1 {
		panic("ProcessProposal called with invalid height")
	}

	// always reset state given that ProcessProposal can timeout and be called again
	emptyHeader := tmproto.Header{ChainID: app.chainID}
	app.setState(runTxProcessProposal, emptyHeader)

	app.processProposalState.ctx = app.getContextForProposal(app.processProposalState.ctx, req.Height).
		WithVoteInfos(app.voteInfos).
		WithBlockHeight(req.Height).
		WithBlockTime(req.Time).
		WithHeaderHash(req.Hash).
		WithProposer(req.ProposerAddress)

	app.processProposalState.ctx = app.processProposalState.ctx.
		WithConsensusParams(app.GetConsensusParams(app.processProposalState.ctx)).
		WithBlockGasMeter(app.getBlockGasMeter(app.processProposalState.ctx))

	defer func() {
		if err := recover(); err != nil {
			app.logger.Error(
				"panic recovered in ProcessProposal",
				"height", req.Height,
				"time", req.Time,
				"hash", fmt.Sprintf("%X", req.Hash),
				"panic", err,
			)
			resp = abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}
		}
	}()

	resp = app.processProposal(app.processProposalState.ctx, req)
	return resp
}

// CheckTx implements the ABCI interface and executes a tx in CheckTx mode. In
// CheckTx mode, messages are not executed. This means messages are only validated
// and only the AnteHandler is executed. State is persisted to the BaseApp's
// internal CheckTx state if the AnteHandler passes. Otherwise, the ResponseCheckTx
// will contain relevant error information. Regardless of tx execution outcome,
// the ResponseCheckTx will contain relevant gas execution context.
func (app *BaseApp) CheckTx(req abci.RequestCheckTx) abci.ResponseCheckTx {
	var mode runTxMode

	switch {
	case req.Type == abci.CheckTxType_New:
		mode = runTxModeCheck

	case req.Type == abci.CheckTxType_Recheck:
		mode = runTxModeReCheck

	default:
		panic(fmt.Sprintf("unknown RequestCheckTx type: %s", req.Type))
	}

	gInfo, result, anteEvents, priority, err := app.runTx(mode, req.Tx)
	if err != nil {
		return sdkerrors.ResponseCheckTxWithEvents(err, gInfo.GasWanted, gInfo.GasUsed, anteEvents, app.trace)
	}

	return abci.ResponseCheckTx{
		GasWanted: int64(gInfo.GasWanted), // TODO: Should type accept unsigned ints?
		GasUsed:   int64(gInfo.GasUsed),   // TODO: Should type accept unsigned ints?
		Log:       result.Log,
		Data:      result.Data,
		Events:    sdk.MarkEventsToIndex(result.Events, app.indexEvents),
		Priority:  priority,
	}
}

// DeliverTx implements the ABCI interface and executes a tx in DeliverTx mode.
// State only gets persisted if all messages are valid and get executed successfully.
// Otherwise, the ResponseDeliverTx will contain relevant error information.
// Regardless of tx execution outcome, the ResponseDeliverTx will contain relevant
// gas execution context.
func (app *BaseApp) DeliverTx(req abci.RequestDeliverTx) (res abci.ResponseDeliverTx) {
	gInfo := sdk.GasInfo{}
	resultStr := "successful"

	defer func() {
		for _, streamingListener := range app.abciListeners {
			if err := streamingListener.ListenDeliverTx(app.deliverState.ctx, req, res); err != nil {
				panic(fmt.Errorf("DeliverTx listening hook failed: %w", err))
			}
		}
	}()

	defer func() {
		telemetry.IncrCounter(1, "tx", "count")
		telemetry.IncrCounter(1, "tx", resultStr)
		telemetry.SetGauge(float32(gInfo.GasUsed), "tx", "gas", "used")
		telemetry.SetGauge(float32(gInfo.GasWanted), "tx", "gas", "wanted")
	}()

	gInfo, result, anteEvents, _, err := app.runTx(runTxModeDeliver, req.Tx)
	if err != nil {
		resultStr = "failed"
		return sdkerrors.ResponseDeliverTxWithEvents(err, gInfo.GasWanted, gInfo.GasUsed, sdk.MarkEventsToIndex(anteEvents, app.indexEvents), app.trace)
	}

	return abci.ResponseDeliverTx{
		GasWanted: int64(gInfo.GasWanted), // TODO: Should type accept unsigned ints?
		GasUsed:   int64(gInfo.GasUsed),   // TODO: Should type accept unsigned ints?
		Log:       result.Log,
		Data:      result.Data,
		Events:    sdk.MarkEventsToIndex(result.Events, app.indexEvents),
	}
}

// Commit implements the ABCI interface. It will commit all state that exists in
// the deliver state's multi-store and includes the resulting commit ID in the
// returned abci.ResponseCommit. Commit will set the check state based on the
// latest header and reset the deliver state. Also, if a non-zero halt height is
// defined in config, Commit will execute a deferred function call to check
// against that height and gracefully halt if it matches the latest committed
// height.
func (app *BaseApp) Commit() abci.ResponseCommit {
	header := app.deliverState.ctx.BlockHeader()
	retainHeight := app.GetBlockRetentionHeight(header.Height)

	rms, ok := app.cms.(*rootmulti.Store)
	if ok {
		rms.SetCommitHeader(header)
	}

	// Write the DeliverTx state into branched storage and commit the MultiStore.
	// The write to the DeliverTx state writes all state transitions to the root
	// MultiStore (app.cms) so when Commit() is called is persists those values.
	app.deliverState.ms.Write()
	commitID := app.cms.Commit()

	res := abci.ResponseCommit{
		Data:         commitID.Hash,
		RetainHeight: retainHeight,
	}

	// call the hooks with the Commit message
	for _, streamingListener := range app.abciListeners {
		if err := streamingListener.ListenCommit(app.deliverState.ctx, res); err != nil {
			panic(fmt.Errorf("Commit listening hook failed, height: %d, err: %w", header.Height, err))
		}
	}

	app.logger.Info("commit synced", "commit", fmt.Sprintf("%X", commitID))

	// Reset the Check state to the latest committed.
	//
	// NOTE: This is safe because Tendermint holds a lock on the mempool for
	// Commit. Use the header from this latest block.
	app.setState(runTxModeCheck, header)

	// empty/reset the deliver state
	app.deliverState = nil

	var halt bool

	switch {
	case app.haltHeight > 0 && uint64(header.Height) >= app.haltHeight:
		halt = true

	case app.haltTime > 0 && header.Time.Unix() >= int64(app.haltTime):
		halt = true
	}

	if halt {
		// Halt the binary and allow Tendermint to receive the ResponseCommit
		// response with the commit ID hash. This will allow the node to successfully
		// restart and process blocks assuming the halt configuration has been
		// reset or moved to a more distant value.
		app.halt()
	}

	go app.snapshotManager.SnapshotIfApplicable(header.Height)

	return res
}

// halt attempts to gracefully shutdown the node via SIGINT and SIGTERM falling
// back on os.Exit if both fail.
func (app *BaseApp) halt() {
	app.logger.Info("halting node per configuration", "height", app.haltHeight, "time", app.haltTime)

	p, err := os.FindProcess(os.Getpid())
	if err == nil {
		// attempt cascading signals in case SIGINT fails (os dependent)
		sigIntErr := p.Signal(syscall.SIGINT)
		sigTermErr := p.Signal(syscall.SIGTERM)

		if sigIntErr == nil || sigTermErr == nil {
			return
		}
	}

	// Resort to exiting immediately if the process could not be found or killed
	// via SIGINT/SIGTERM signals.
	app.logger.Info("failed to send SIGINT/SIGTERM; exiting...")
	os.Exit(0)
}

// Query implements the ABCI interface. It delegates to CommitMultiStore if it
// implements Queryable.
func (app *BaseApp) Query(req abci.RequestQuery) (res abci.ResponseQuery) {
	// Add panic recovery for all queries.
	// ref: https://github.com/cosmos/cosmos-sdk/pull/8039
	defer func() {
		if r := recover(); r != nil {
			res = sdkerrors.QueryResult(sdkerrors.Wrapf(sdkerrors.ErrPanic, "%v", r), app.trace)
		}
	}()

	// when a client did not provide a query height, manually inject the latest
	if req.Height == 0 {
		req.Height = app.LastBlockHeight()
	}

	telemetry.IncrCounter(1, "query", "count")
	telemetry.IncrCounter(1, "query", req.Path)
	defer telemetry.MeasureSince(time.Now(), req.Path)

	if req.Path == "/cosmos.tx.v1beta1.Service/BroadcastTx" {
		return sdkerrors.QueryResult(sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, "can't route a broadcast tx message"), app.trace)
	}

	// handle gRPC routes first rather than calling splitPath because '/' characters
	// are used as part of gRPC paths
	if grpcHandler := app.grpcQueryRouter.Route(req.Path); grpcHandler != nil {
		return app.handleQueryGRPC(grpcHandler, req)
	}

	path := SplitABCIQueryPath(req.Path)
	if len(path) == 0 {
		return sdkerrors.QueryResult(sdkerrors.Wrap(sdkerrors.ErrUnknownRequest, "no query path provided"), app.trace)
	}

	switch path[0] {
	case QueryPathApp:
		// "/app" prefix for special application queries
		return handleQueryApp(app, path, req)

	case QueryPathStore:
		return handleQueryStore(app, path, req)

	case QueryPathP2P:
		return handleQueryP2P(app, path)
	}

	return sdkerrors.QueryResult(sdkerrors.Wrap(sdkerrors.ErrUnknownRequest, "unknown query path"), app.trace)
}

// ListSnapshots implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) ListSnapshots(req abci.RequestListSnapshots) abci.ResponseListSnapshots {
	resp := abci.ResponseListSnapshots{Snapshots: []*abci.Snapshot{}}
	if app.snapshotManager == nil {
		return resp
	}

	snapshots, err := app.snapshotManager.List()
	if err != nil {
		app.logger.Error("failed to list snapshots", "err", err)
		return resp
	}

	for _, snapshot := range snapshots {
		abciSnapshot, err := snapshot.ToABCI()
		if err != nil {
			app.logger.Error("failed to list snapshots", "err", err)
			return resp
		}
		resp.Snapshots = append(resp.Snapshots, &abciSnapshot)
	}

	return resp
}

// LoadSnapshotChunk implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) LoadSnapshotChunk(req abci.RequestLoadSnapshotChunk) abci.ResponseLoadSnapshotChunk {
	if app.snapshotManager == nil {
		return abci.ResponseLoadSnapshotChunk{}
	}
	chunk, err := app.snapshotManager.LoadChunk(req.Height, req.Format, req.Chunk)
	if err != nil {
		app.logger.Error(
			"failed to load snapshot chunk",
			"height", req.Height,
			"format", req.Format,
			"chunk", req.Chunk,
			"err", err,
		)
		return abci.ResponseLoadSnapshotChunk{}
	}
	return abci.ResponseLoadSnapshotChunk{Chunk: chunk}
}

// OfferSnapshot implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) OfferSnapshot(req abci.RequestOfferSnapshot) abci.ResponseOfferSnapshot {
	if app.snapshotManager == nil {
		app.logger.Error("snapshot manager not configured")
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ABORT}
	}

	if req.Snapshot == nil {
		app.logger.Error("received nil snapshot")
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_REJECT}
	}

	snapshot, err := snapshottypes.SnapshotFromABCI(req.Snapshot)
	if err != nil {
		app.logger.Error("failed to decode snapshot metadata", "err", err)
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_REJECT}
	}

	err = app.snapshotManager.Restore(snapshot)
	switch {
	case err == nil:
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ACCEPT}

	case errors.Is(err, snapshottypes.ErrUnknownFormat):
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_REJECT_FORMAT}

	case errors.Is(err, snapshottypes.ErrInvalidMetadata):
		app.logger.Error(
			"rejecting invalid snapshot",
			"height", req.Snapshot.Height,
			"format", req.Snapshot.Format,
			"err", err,
		)
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_REJECT}

	default:
		app.logger.Error(
			"failed to restore snapshot",
			"height", req.Snapshot.Height,
			"format", req.Snapshot.Format,
			"err", err,
		)

		// We currently don't support resetting the IAVL stores and retrying a different snapshot,
		// so we ask Tendermint to abort all snapshot restoration.
		return abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ABORT}
	}
}

// ApplySnapshotChunk implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) ApplySnapshotChunk(req abci.RequestApplySnapshotChunk) abci.ResponseApplySnapshotChunk {
	if app.snapshotManager == nil {
		app.logger.Error("snapshot manager not configured")
		return abci.ResponseApplySnapshotChunk{Result: abci.ResponseApplySnapshotChunk_ABORT}
	}

	_, err := app.snapshotManager.RestoreChunk(req.Chunk)
	switch {
	case err == nil:
		return abci.ResponseApplySnapshotChunk{Result: abci.ResponseApplySnapshotChunk_ACCEPT}

	case errors.Is(err, snapshottypes.ErrChunkHashMismatch):
		app.logger.Error(
			"chunk checksum mismatch; rejecting sender and requesting refetch",
			"chunk", req.Index,
			"sender", req.Sender,
			"err", err,
		)
		return abci.ResponseApplySnapshotChunk{
			Result:        abci.ResponseApplySnapshotChunk_RETRY,
			RefetchChunks: []uint32{req.Index},
			RejectSenders: []string{req.Sender},
		}

	default:
		app.logger.Error("failed to restore snapshot", "err", err)
		return abci.ResponseApplySnapshotChunk{Result: abci.ResponseApplySnapshotChunk_ABORT}
	}
}

func (app *BaseApp) handleQueryGRPC(handler GRPCQueryHandler, req abci.RequestQuery) abci.ResponseQuery {
	ctx, err := app.CreateQueryContext(req.Height, req.Prove)
	if err != nil {
		return sdkerrors.QueryResult(err, app.trace)
	}

	res, err := handler(ctx, req)
	if err != nil {
		res = sdkerrors.QueryResult(gRPCErrorToSDKError(err), app.trace)
		res.Height = req.Height
		return res
	}

	return res
}

func gRPCErrorToSDKError(err error) error {
	status, ok := grpcstatus.FromError(err)
	if !ok {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, err.Error())
	}

	switch status.Code() {
	case codes.NotFound:
		return sdkerrors.Wrap(sdkerrors.ErrKeyNotFound, err.Error())
	case codes.InvalidArgument:
		return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, err.Error())
	case codes.FailedPrecondition:
		return sdkerrors.Wrap(sdkerrors.ErrInvalidRequest, err.Error())
	case codes.Unauthenticated:
		return sdkerrors.Wrap(sdkerrors.ErrUnauthorized, err.Error())
	default:
		return sdkerrors.Wrap(sdkerrors.ErrUnknownRequest, err.Error())
	}
}

func checkNegativeHeight(height int64) error {
	if height < 0 {
		// Reject invalid heights.
		return sdkerrors.Wrap(
			sdkerrors.ErrInvalidRequest,
			"cannot query with height < 0; please provide a valid height",
		)
	}
	return nil
}

// createQueryContext creates a new sdk.Context for a query, taking as args
// the block height and whether the query needs a proof or not.
func (app *BaseApp) CreateQueryContext(height int64, prove bool) (sdk.Context, error) {
	if err := checkNegativeHeight(height); err != nil {
		return sdk.Context{}, err
	}

	// use custom query multistore if provided
	qms := app.qms
	if qms == nil {
		qms = app.cms.(sdk.MultiStore)
	}

	lastBlockHeight := qms.LatestVersion()
	if lastBlockHeight == 0 {
		return sdk.Context{}, sdkerrors.Wrapf(sdkerrors.ErrInvalidHeight, "%s is not ready; please wait for first block", app.Name())
	}

	if height > lastBlockHeight {
		return sdk.Context{},
			sdkerrors.Wrap(
				sdkerrors.ErrInvalidHeight,
				"cannot query with height in the future; please provide a valid height",
			)
	}

	// when a client did not provide a query height, manually inject the latest
	if height == 0 {
		height = lastBlockHeight
	}

	if height <= 1 && prove {
		return sdk.Context{},
			sdkerrors.Wrap(
				sdkerrors.ErrInvalidRequest,
				"cannot query with proof when height <= 1; please provide a valid height",
			)
	}

	cacheMS, err := qms.CacheMultiStoreWithVersion(height)
	if err != nil {
		return sdk.Context{},
			sdkerrors.Wrapf(
				sdkerrors.ErrInvalidRequest,
				"failed to load state at height %d; %s (latest height: %d)", height, err, lastBlockHeight,
			)
	}

	// branch the commit-multistore for safety
	ctx := sdk.NewContext(cacheMS, app.checkState.ctx.BlockHeader(), true, app.logger).
		WithMinGasPrices(app.minGasPrices).
		WithBlockHeight(height)

	if height != lastBlockHeight {
		rms, ok := app.cms.(*rootmulti.Store)
		if ok {
			cInfo, err := rms.GetCommitInfo(height)
			if cInfo != nil && err == nil {
				ctx = ctx.WithBlockTime(cInfo.Timestamp)
			}
		}
	}

	return ctx, nil
}

// GetBlockRetentionHeight returns the height for which all blocks below this height
// are pruned from Tendermint. Given a commitment height and a non-zero local
// minRetainBlocks configuration, the retentionHeight is the smallest height that
// satisfies:
//
// - Unbonding (safety threshold) time: The block interval in which validators
// can be economically punished for misbehavior. Blocks in this interval must be
// auditable e.g. by the light client.
//
// - Logical store snapshot interval: The block interval at which the underlying
// logical store database is persisted to disk, e.g. every 10000 heights. Blocks
// since the last IAVL snapshot must be available for replay on application restart.
//
// - State sync snapshots: Blocks since the oldest available snapshot must be
// available for state sync nodes to catch up (oldest because a node may be
// restoring an old snapshot while a new snapshot was taken).
//
// - Local (minRetainBlocks) config: Archive nodes may want to retain more or
// all blocks, e.g. via a local config option min-retain-blocks. There may also
// be a need to vary retention for other nodes, e.g. sentry nodes which do not
// need historical blocks.
func (app *BaseApp) GetBlockRetentionHeight(commitHeight int64) int64 {
	// pruning is disabled if minRetainBlocks is zero
	if app.minRetainBlocks == 0 {
		return 0
	}

	minNonZero := func(x, y int64) int64 {
		switch {
		case x == 0:
			return y
		case y == 0:
			return x
		case x < y:
			return x
		default:
			return y
		}
	}

	// Define retentionHeight as the minimum value that satisfies all non-zero
	// constraints. All blocks below (commitHeight-retentionHeight) are pruned
	// from Tendermint.
	var retentionHeight int64

	// Define the number of blocks needed to protect against misbehaving validators
	// which allows light clients to operate safely. Note, we piggy back of the
	// evidence parameters instead of computing an estimated nubmer of blocks based
	// on the unbonding period and block commitment time as the two should be
	// equivalent.
	cp := app.GetConsensusParams(app.deliverState.ctx)
	if cp != nil && cp.Evidence != nil && cp.Evidence.MaxAgeNumBlocks > 0 {
		retentionHeight = commitHeight - cp.Evidence.MaxAgeNumBlocks
	}

	if app.snapshotManager != nil {
		snapshotRetentionHeights := app.snapshotManager.GetSnapshotBlockRetentionHeights()
		if snapshotRetentionHeights > 0 {
			retentionHeight = minNonZero(retentionHeight, commitHeight-snapshotRetentionHeights)
		}
	}

	v := commitHeight - int64(app.minRetainBlocks)
	retentionHeight = minNonZero(retentionHeight, v)

	if retentionHeight <= 0 {
		// prune nothing in the case of a non-positive height
		return 0
	}

	return retentionHeight
}

func handleQueryApp(app *BaseApp, path []string, req abci.RequestQuery) abci.ResponseQuery {
	if len(path) >= 2 {
		switch path[1] {
		case "simulate":
			txBytes := req.Data

			gInfo, res, err := app.Simulate(txBytes)
			if err != nil {
				return sdkerrors.QueryResult(sdkerrors.Wrap(err, "failed to simulate tx"), app.trace)
			}

			simRes := &sdk.SimulationResponse{
				GasInfo: gInfo,
				Result:  res,
			}

			bz, err := codec.ProtoMarshalJSON(simRes, app.interfaceRegistry)
			if err != nil {
				return sdkerrors.QueryResult(sdkerrors.Wrap(err, "failed to JSON encode simulation response"), app.trace)
			}

			return abci.ResponseQuery{
				Codespace: sdkerrors.RootCodespace,
				Height:    req.Height,
				Value:     bz,
			}

		case "version":
			return abci.ResponseQuery{
				Codespace: sdkerrors.RootCodespace,
				Height:    req.Height,
				Value:     []byte(app.version),
			}

		default:
			return sdkerrors.QueryResult(sdkerrors.Wrapf(sdkerrors.ErrUnknownRequest, "unknown query: %s", path), app.trace)
		}
	}

	return sdkerrors.QueryResult(
		sdkerrors.Wrap(
			sdkerrors.ErrUnknownRequest,
			"expected second parameter to be either 'simulate' or 'version', neither was present",
		), app.trace)
}

func handleQueryStore(app *BaseApp, path []string, req abci.RequestQuery) abci.ResponseQuery {
	// "/store" prefix for store queries
	queryable, ok := app.cms.(sdk.Queryable)
	if !ok {
		return sdkerrors.QueryResult(sdkerrors.Wrap(sdkerrors.ErrUnknownRequest, "multistore doesn't support queries"), app.trace)
	}

	req.Path = "/" + strings.Join(path[1:], "/")

	if req.Height <= 1 && req.Prove {
		return sdkerrors.QueryResult(
			sdkerrors.Wrap(
				sdkerrors.ErrInvalidRequest,
				"cannot query with proof when height <= 1; please provide a valid height",
			), app.trace)
	}

	resp := queryable.Query(req)
	resp.Height = req.Height

	return resp
}

func handleQueryP2P(app *BaseApp, path []string) abci.ResponseQuery {
	// "/p2p" prefix for p2p queries
	if len(path) < 4 {
		return sdkerrors.QueryResult(
			sdkerrors.Wrap(
				sdkerrors.ErrUnknownRequest, "path should be p2p filter <addr|id> <parameter>",
			), app.trace)
	}

	var resp abci.ResponseQuery

	cmd, typ, arg := path[1], path[2], path[3]
	switch cmd {
	case "filter":
		switch typ {
		case "addr":
			resp = app.FilterPeerByAddrPort(arg)

		case "id":
			resp = app.FilterPeerByID(arg)
		}

	default:
		resp = sdkerrors.QueryResult(sdkerrors.Wrap(sdkerrors.ErrUnknownRequest, "expected second parameter to be 'filter'"), app.trace)
	}

	return resp
}

// SplitABCIQueryPath splits a string path using the delimiter '/'.
//
// e.g. "this/is/funny" becomes []string{"this", "is", "funny"}
func SplitABCIQueryPath(requestPath string) (path []string) {
	path = strings.Split(requestPath, "/")

	// first element is empty string
	if len(path) > 0 && path[0] == "" {
		path = path[1:]
	}

	return path
}

// getContextForProposal returns the right context for PrepareProposal and
// ProcessProposal. We use deliverState on the first block to be able to access
// any state changes made in InitChain.
func (app *BaseApp) getContextForProposal(ctx sdk.Context, height int64) sdk.Context {
	if height == app.initialHeight {
		ctx, _ = app.deliverState.ctx.CacheContext()

		// clear all context data set during InitChain to avoid inconsistent behavior
		ctx = ctx.WithBlockHeader(tmproto.Header{})
		return ctx
	}

	return ctx
}
