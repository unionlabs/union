package baseapp

import (
	"context"
	"errors"
	"fmt"
	"sort"
	"strconv"
	"strings"
	"time"

	abcitypes "github.com/cometbft/cometbft/abci/types"
	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	grpcstatus "google.golang.org/grpc/status"

	corecomet "cosmossdk.io/core/comet"
	coreheader "cosmossdk.io/core/header"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/rootmulti"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// Supported ABCI Query prefixes and paths
const (
	QueryPathApp    = "app"
	QueryPathCustom = "custom"
	QueryPathP2P    = "p2p"
	QueryPathStore  = "store"

	QueryPathBroadcastTx = "/cosmos.tx.v1beta1.Service/BroadcastTx"
)

// InitChain implements the ABCI interface. It initializes the application's state
// and sets up the initial validator set.
func (app *BaseApp) InitChain(req *abci.InitChainRequest) (*abci.InitChainResponse, error) {
	if req.ChainId != app.chainID {
		return nil, fmt.Errorf("invalid chain-id on InitChain; expected: %s, got: %s", app.chainID, req.ChainId)
	}

	// On a new chain, we consider the init chain block height as 0, even though
	// req.InitialHeight is 1 by default.
	initHeader := cmtproto.Header{ChainID: req.ChainId, Time: req.Time}
	app.logger.Info("InitChain", "initialHeight", req.InitialHeight, "chainID", req.ChainId)

	// Set the initial height, which will be used to determine if we are proposing
	// or processing the first block or not.
	app.initialHeight = req.InitialHeight
	if app.initialHeight == 0 { // If initial height is 0, set it to 1
		app.initialHeight = 1
	}

	// if req.InitialHeight is > 1, then we set the initial version on all stores
	if req.InitialHeight > 1 {
		initHeader.Height = req.InitialHeight
		if err := app.cms.SetInitialVersion(req.InitialHeight); err != nil {
			return nil, err
		}
	}

	// initialize states with a correct header
	app.setState(execModeFinalize, initHeader)
	app.setState(execModeCheck, initHeader)

	// Store the consensus params in the BaseApp's param store. Note, this must be
	// done after the finalizeBlockState and context have been set as it's persisted
	// to state.
	if req.ConsensusParams != nil {
		err := app.StoreConsensusParams(app.finalizeBlockState.Context(), *req.ConsensusParams)
		if err != nil {
			return nil, err
		}
	}

	defer func() {
		// InitChain represents the state of the application BEFORE the first block,
		// i.e. the genesis block. This means that when processing the app's InitChain
		// handler, the block height is zero by default. However, after Commit is called
		// the height needs to reflect the true block height.
		initHeader.Height = req.InitialHeight
		app.checkState.SetContext(app.checkState.Context().WithBlockHeader(initHeader).
			WithHeaderInfo(coreheader.Info{
				ChainID: req.ChainId,
				Height:  req.InitialHeight,
				Time:    req.Time,
			}))
		app.finalizeBlockState.SetContext(app.finalizeBlockState.Context().WithBlockHeader(initHeader).
			WithHeaderInfo(coreheader.Info{
				ChainID: req.ChainId,
				Height:  req.InitialHeight,
				Time:    req.Time,
			}))
	}()

	if app.initChainer == nil {
		return &abci.InitChainResponse{}, nil
	}

	// add block gas meter for any genesis transactions (allow infinite gas)
	app.finalizeBlockState.SetContext(app.finalizeBlockState.Context().WithBlockGasMeter(storetypes.NewInfiniteGasMeter()))

	res, err := app.initChainer(app.finalizeBlockState.Context(), req)
	if err != nil {
		return nil, err
	}

	if len(req.Validators) > 0 {
		if len(req.Validators) != len(res.Validators) {
			return nil, fmt.Errorf(
				"len(RequestInitChain.Validators) != len(GenesisValidators) (%d != %d)",
				len(req.Validators), len(res.Validators),
			)
		}

		sort.Sort(abcitypes.ValidatorUpdates(req.Validators))

		for i := range res.Validators {
			if !proto.Equal(&res.Validators[i], &req.Validators[i]) {
				return nil, fmt.Errorf("genesisValidators[%d] != req.Validators[%d] ", i, i)
			}
		}
	}

	// NOTE: We don't commit, but FinalizeBlock for block InitialHeight starts from
	// this FinalizeBlockState.
	return &abci.InitChainResponse{
		ConsensusParams: res.ConsensusParams,
		Validators:      res.Validators,
		AppHash:         app.LastCommitID().Hash,
	}, nil
}

// Info implements the ABCI interface. It returns information about the application.
func (app *BaseApp) Info(_ *abci.InfoRequest) (*abci.InfoResponse, error) {
	lastCommitID := app.cms.LastCommitID()
	appVersion := InitialAppVersion
	if lastCommitID.Version > 0 {
		ctx, err := app.CreateQueryContextWithCheckHeader(lastCommitID.Version, false, false)
		if err != nil {
			return nil, fmt.Errorf("failed creating query context: %w", err)
		}
		appVersion, err = app.AppVersion(ctx)
		if err != nil {
			return nil, fmt.Errorf("failed getting app version: %w", err)
		}
	}

	return &abci.InfoResponse{
		Data:             app.name,
		Version:          app.version,
		AppVersion:       appVersion,
		LastBlockHeight:  lastCommitID.Version,
		LastBlockAppHash: lastCommitID.Hash,
	}, nil
}

// Query implements the ABCI interface. It delegates to CommitMultiStore if it
// implements Queryable.
func (app *BaseApp) Query(_ context.Context, req *abci.QueryRequest) (resp *abci.QueryResponse, err error) {
	// add panic recovery for all queries
	//
	// Ref: https://github.com/cosmos/cosmos-sdk/pull/8039
	defer func() {
		if r := recover(); r != nil {
			resp = queryResult(errorsmod.Wrapf(sdkerrors.ErrPanic, "%v", r), app.trace)
		}
	}()

	// when a client did not provide a query height, manually inject the latest
	if req.Height == 0 {
		req.Height = app.LastBlockHeight()
	}

	telemetry.IncrCounter(1, "query", "count")
	telemetry.IncrCounter(1, "query", req.Path)
	start := telemetry.Now()
	defer telemetry.MeasureSince(start, req.Path)

	if req.Path == QueryPathBroadcastTx {
		return queryResult(errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "can't route a broadcast tx message"), app.trace), nil
	}

	// handle gRPC routes first rather than calling splitPath because '/' characters
	// are used as part of gRPC paths
	if grpcHandler := app.grpcQueryRouter.Route(req.Path); grpcHandler != nil {
		return app.handleQueryGRPC(grpcHandler, req), nil
	}

	path := SplitABCIQueryPath(req.Path)
	if len(path) == 0 {
		return queryResult(errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "no query path provided"), app.trace), nil
	}

	switch path[0] {
	case QueryPathApp:
		// "/app" prefix for special application queries
		resp = handleQueryApp(app, path, req)

	case QueryPathStore:
		resp = handleQueryStore(app, path, *req)

	case QueryPathP2P:
		resp = handleQueryP2P(app, path)

	default:
		resp = queryResult(errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "unknown query path"), app.trace)
	}

	return resp, nil
}

// ListSnapshots implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) ListSnapshots(req *abci.ListSnapshotsRequest) (*abci.ListSnapshotsResponse, error) {
	resp := &abci.ListSnapshotsResponse{Snapshots: []*abci.Snapshot{}}
	if app.snapshotManager == nil {
		return resp, nil
	}

	snapshots, err := app.snapshotManager.List()
	if err != nil {
		app.logger.Error("failed to list snapshots", "err", err)
		return nil, err
	}

	for _, snapshot := range snapshots {
		abciSnapshot, err := snapshot.ToABCI()
		if err != nil {
			app.logger.Error("failed to convert ABCI snapshots", "err", err)
			return nil, err
		}

		resp.Snapshots = append(resp.Snapshots, &abciSnapshot)
	}

	return resp, nil
}

// LoadSnapshotChunk implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) LoadSnapshotChunk(req *abci.LoadSnapshotChunkRequest) (*abci.LoadSnapshotChunkResponse, error) {
	if app.snapshotManager == nil {
		return &abci.LoadSnapshotChunkResponse{}, nil
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
		return nil, err
	}

	return &abci.LoadSnapshotChunkResponse{Chunk: chunk}, nil
}

// OfferSnapshot implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) OfferSnapshot(req *abci.OfferSnapshotRequest) (*abci.OfferSnapshotResponse, error) {
	if app.snapshotManager == nil {
		app.logger.Error("snapshot manager not configured")
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ABORT}, nil
	}

	if req.Snapshot == nil {
		app.logger.Error("received nil snapshot")
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil
	}

	snapshot, err := snapshottypes.SnapshotFromABCI(req.Snapshot)
	if err != nil {
		app.logger.Error("failed to decode snapshot metadata", "err", err)
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil
	}

	err = app.snapshotManager.Restore(snapshot)
	switch {
	case err == nil:
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ACCEPT}, nil

	case errors.Is(err, snapshottypes.ErrUnknownFormat):
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT_FORMAT}, nil

	case errors.Is(err, snapshottypes.ErrInvalidMetadata):
		app.logger.Error(
			"rejecting invalid snapshot",
			"height", req.Snapshot.Height,
			"format", req.Snapshot.Format,
			"err", err,
		)
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_REJECT}, nil

	default:
		// CometBFT errors are defined here: https://github.com/cometbft/cometbft/blob/main/statesync/syncer.go
		// It may happen that in case of a CometBFT error, such as a timeout (which occurs after two minutes),
		// the process is aborted. This is done intentionally because deleting the database programmatically
		// can lead to more complicated situations.
		app.logger.Error(
			"failed to restore snapshot",
			"height", req.Snapshot.Height,
			"format", req.Snapshot.Format,
			"err", err,
		)

		// We currently don't support resetting the IAVL stores and retrying a
		// different snapshot, so we ask CometBFT to abort all snapshot restoration.
		return &abci.OfferSnapshotResponse{Result: abci.OFFER_SNAPSHOT_RESULT_ABORT}, nil
	}
}

// ApplySnapshotChunk implements the ABCI interface. It delegates to app.snapshotManager if set.
func (app *BaseApp) ApplySnapshotChunk(req *abci.ApplySnapshotChunkRequest) (*abci.ApplySnapshotChunkResponse, error) {
	if app.snapshotManager == nil {
		app.logger.Error("snapshot manager not configured")
		return &abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT}, nil
	}

	_, err := app.snapshotManager.RestoreChunk(req.Chunk)
	switch {
	case err == nil:
		return &abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil

	case errors.Is(err, snapshottypes.ErrChunkHashMismatch):
		app.logger.Error(
			"chunk checksum mismatch; rejecting sender and requesting refetch",
			"chunk", req.Index,
			"sender", req.Sender,
			"err", err,
		)
		return &abci.ApplySnapshotChunkResponse{
			Result:        abci.APPLY_SNAPSHOT_CHUNK_RESULT_RETRY,
			RefetchChunks: []uint32{req.Index},
			RejectSenders: []string{req.Sender},
		}, nil

	default:
		app.logger.Error("failed to restore snapshot", "err", err)
		return &abci.ApplySnapshotChunkResponse{Result: abci.APPLY_SNAPSHOT_CHUNK_RESULT_ABORT}, nil
	}
}

// CheckTx implements the ABCI interface and executes a tx in CheckTx mode. In
// CheckTx mode, messages are not executed. This means messages are only validated
// and only the AnteHandler is executed. State is persisted to the BaseApp's
// internal CheckTx state if the AnteHandler passes. Otherwise, the ResponseCheckTx
// will contain relevant error information. Regardless of tx execution outcome,
// the ResponseCheckTx will contain relevant gas execution context.
func (app *BaseApp) CheckTx(req *abci.CheckTxRequest) (*abci.CheckTxResponse, error) {
	var mode execMode

	switch {
	case req.Type == abci.CHECK_TX_TYPE_CHECK:
		mode = execModeCheck

	case req.Type == abci.CHECK_TX_TYPE_RECHECK:
		mode = execModeReCheck

	default:
		return nil, fmt.Errorf("unknown RequestCheckTx type: %s", req.Type)
	}

	if app.checkTxHandler == nil {
		gInfo, result, anteEvents, err := app.runTx(mode, req.Tx, nil)
		if err != nil {
			return responseCheckTxWithEvents(err, gInfo.GasWanted, gInfo.GasUsed, anteEvents, app.trace), nil
		}

		return &abci.CheckTxResponse{
			GasWanted: int64(gInfo.GasWanted), // TODO: Should type accept unsigned ints?
			GasUsed:   int64(gInfo.GasUsed),   // TODO: Should type accept unsigned ints?
			Log:       result.Log,
			Data:      result.Data,
			Events:    sdk.MarkEventsToIndex(result.Events, app.indexEvents),
		}, nil
	}

	// Create wrapper to avoid users overriding the execution mode
	runTx := func(txBytes []byte, tx sdk.Tx) (gInfo sdk.GasInfo, result *sdk.Result, anteEvents []abci.Event, err error) {
		return app.runTx(mode, txBytes, tx)
	}

	return app.checkTxHandler(runTx, req)
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
// Ref: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
func (app *BaseApp) PrepareProposal(req *abci.PrepareProposalRequest) (resp *abci.PrepareProposalResponse, err error) {
	if app.prepareProposal == nil {
		return nil, errors.New("PrepareProposal handler not set")
	}

	// Always reset state given that PrepareProposal can timeout and be called
	// again in a subsequent round.
	header := cmtproto.Header{
		ChainID:            app.chainID,
		Height:             req.Height,
		Time:               req.Time,
		ProposerAddress:    req.ProposerAddress,
		NextValidatorsHash: req.NextValidatorsHash,
		AppHash:            app.LastCommitID().Hash,
	}
	app.setState(execModePrepareProposal, header)

	// CometBFT must never call PrepareProposal with a height of 0.
	//
	// Ref: https://github.com/cometbft/cometbft/blob/059798a4f5b0c9f52aa8655fa619054a0154088c/spec/core/state.md?plain=1#L37-L38
	if req.Height < 1 {
		return nil, errors.New("PrepareProposal called with invalid height")
	}

	app.prepareProposalState.SetContext(app.getContextForProposal(app.prepareProposalState.Context(), req.Height).
		WithVoteInfos(toVoteInfo(req.LocalLastCommit.Votes)). // this is a set of votes that are not finalized yet, wait for commit
		WithBlockHeight(req.Height).
		WithProposer(req.ProposerAddress).
		WithExecMode(sdk.ExecModePrepareProposal).
		WithCometInfo(corecomet.Info{
			Evidence:        sdk.ToSDKEvidence(req.Misbehavior),
			ValidatorsHash:  req.NextValidatorsHash,
			ProposerAddress: req.ProposerAddress,
			LastCommit:      sdk.ToSDKExtendedCommitInfo(req.LocalLastCommit),
		}).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  req.Height,
			Time:    req.Time,
		}))

	app.prepareProposalState.SetContext(app.prepareProposalState.Context().
		WithConsensusParams(app.GetConsensusParams(app.prepareProposalState.Context())).
		WithBlockGasMeter(app.getBlockGasMeter(app.prepareProposalState.Context())))

	defer func() {
		if err := recover(); err != nil {
			app.logger.Error(
				"panic recovered in PrepareProposal",
				"height", req.Height,
				"time", req.Time,
				"panic", err,
			)

			resp = &abci.PrepareProposalResponse{Txs: req.Txs}
		}
	}()

	resp, err = app.prepareProposal(app.prepareProposalState.Context(), req)
	if err != nil {
		app.logger.Error("failed to prepare proposal", "height", req.Height, "time", req.Time, "err", err)
		return &abci.PrepareProposalResponse{Txs: req.Txs}, nil
	}

	return resp, nil
}

// ProcessProposal implements the ProcessProposal ABCI method and returns a
// ResponseProcessProposal object to the client. The ProcessProposal method is
// responsible for allowing execution of application-dependent work in a proposed
// block. Note, the application defines the exact implementation details of
// ProcessProposal. In general, the application must at the very least ensure
// that all transactions are valid. If all transactions are valid, then we inform
// CometBFT that the Status is ACCEPT. However, the application is also able
// to implement optimizations such as executing the entire proposed block
// immediately.
//
// If a panic is detected during execution of an application's ProcessProposal
// handler, it will be recovered and we will reject the proposal.
//
// Ref: https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-060-abci-1.0.md
// Ref: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_basic_concepts.md
func (app *BaseApp) ProcessProposal(req *abci.ProcessProposalRequest) (resp *abci.ProcessProposalResponse, err error) {
	if app.processProposal == nil {
		return nil, errors.New("ProcessProposal handler not set")
	}

	// CometBFT must never call ProcessProposal with a height of 0.
	// Ref: https://github.com/cometbft/cometbft/blob/059798a4f5b0c9f52aa8655fa619054a0154088c/spec/core/state.md?plain=1#L37-L38
	if req.Height < 1 {
		return nil, errors.New("ProcessProposal called with invalid height")
	}

	// Always reset state given that ProcessProposal can timeout and be called
	// again in a subsequent round.
	header := cmtproto.Header{
		ChainID:            app.chainID,
		Height:             req.Height,
		Time:               req.Time,
		ProposerAddress:    req.ProposerAddress,
		NextValidatorsHash: req.NextValidatorsHash,
		AppHash:            app.LastCommitID().Hash,
	}
	app.setState(execModeProcessProposal, header)

	// Since the application can get access to FinalizeBlock state and write to it,
	// we must be sure to reset it in case ProcessProposal timeouts and is called
	// again in a subsequent round. However, we only want to do this after we've
	// processed the first block, as we want to avoid overwriting the finalizeState
	// after state changes during InitChain.
	if req.Height > app.initialHeight {
		// abort any running OE
		app.optimisticExec.Abort()
		app.setState(execModeFinalize, header)
	}

	app.processProposalState.SetContext(app.getContextForProposal(app.processProposalState.Context(), req.Height).
		WithVoteInfos(req.ProposedLastCommit.Votes). // this is a set of votes that are not finalized yet, wait for commit
		WithBlockHeight(req.Height).
		WithHeaderHash(req.Hash).
		WithProposer(req.ProposerAddress).
		WithCometInfo(corecomet.Info{
			ProposerAddress: req.ProposerAddress,
			ValidatorsHash:  req.NextValidatorsHash,
			Evidence:        sdk.ToSDKEvidence(req.Misbehavior),
			LastCommit:      sdk.ToSDKCommitInfo(req.ProposedLastCommit),
		},
		).
		WithExecMode(sdk.ExecModeProcessProposal).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  req.Height,
			Time:    req.Time,
		}))

	app.processProposalState.SetContext(app.processProposalState.Context().
		WithConsensusParams(app.GetConsensusParams(app.processProposalState.Context())).
		WithBlockGasMeter(app.getBlockGasMeter(app.processProposalState.Context())))

	defer func() {
		if err := recover(); err != nil {
			app.logger.Error(
				"panic recovered in ProcessProposal",
				"height", req.Height,
				"time", req.Time,
				"hash", fmt.Sprintf("%X", req.Hash),
				"panic", err,
			)
			resp = &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_REJECT}
		}
	}()

	resp, err = app.processProposal(app.processProposalState.Context(), req)
	if err != nil {
		app.logger.Error("failed to process proposal", "height", req.Height, "time", req.Time, "hash", fmt.Sprintf("%X", req.Hash), "err", err)
		return &abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_REJECT}, nil
	}

	// Only execute optimistic execution if the proposal is accepted, OE is
	// enabled and the block height is greater than the initial height. During
	// the first block we'll be carrying state from InitChain, so it would be
	// impossible for us to easily revert.
	// After the first block has been processed, the next blocks will get executed
	// optimistically, so that when the ABCI client calls `FinalizeBlock` the app
	// can have a response ready.
	if resp.Status == abci.PROCESS_PROPOSAL_STATUS_ACCEPT &&
		app.optimisticExec.Enabled() &&
		req.Height > app.initialHeight {
		app.optimisticExec.Execute(req)
	}

	return resp, nil
}

// ExtendVote implements the ExtendVote ABCI method and returns a ResponseExtendVote.
// It calls the application's ExtendVote handler which is responsible for performing
// application-specific business logic when sending a pre-commit for the NEXT
// block height. The extensions response may be non-deterministic but must always
// be returned, even if empty.
//
// Agreed upon vote extensions are made available to the proposer of the next
// height and are committed in the subsequent height, i.e. H+2. An error is
// returned if vote extensions are not enabled or if extendVote fails or panics.
func (app *BaseApp) ExtendVote(_ context.Context, req *abci.ExtendVoteRequest) (resp *abci.ExtendVoteResponse, err error) {
	// Always reset state given that ExtendVote and VerifyVoteExtension can timeout
	// and be called again in a subsequent round.
	var ctx sdk.Context

	// If we're extending the vote for the initial height, we need to use the
	// finalizeBlockState context, otherwise we don't get the uncommitted data
	// from InitChain.
	if req.Height == app.initialHeight {
		ctx, _ = app.finalizeBlockState.Context().CacheContext()
	} else {
		ms := app.cms.CacheMultiStore()
		ctx = sdk.NewContext(ms, false, app.logger).WithStreamingManager(app.streamingManager).WithChainID(app.chainID).WithBlockHeight(req.Height)
	}

	if app.extendVote == nil {
		return nil, errors.New("application ExtendVote handler not set")
	}

	// If vote extensions are not enabled, as a safety precaution, we return an
	// error.
	cp := app.GetConsensusParams(ctx)

	// Note: In this case, we do want to extend vote if the height is equal or
	// greater than VoteExtensionsEnableHeight. This defers from the check done
	// in ValidateVoteExtensions and PrepareProposal in which we'll check for
	// vote extensions on VoteExtensionsEnableHeight+1.
	extsEnabled := cp.Feature != nil && req.Height >= cp.Feature.VoteExtensionsEnableHeight.Value && cp.Feature.VoteExtensionsEnableHeight.Value != 0
	if !extsEnabled {
		// check abci params
		extsEnabled = cp.Abci != nil && req.Height >= cp.Abci.VoteExtensionsEnableHeight && cp.Abci.VoteExtensionsEnableHeight != 0
		if !extsEnabled {
			return nil, fmt.Errorf("vote extensions are not enabled; unexpected call to ExtendVote at height %d", req.Height)
		}
	}

	ctx = ctx.
		WithConsensusParams(cp).
		WithBlockGasMeter(storetypes.NewInfiniteGasMeter()).
		WithBlockHeight(req.Height).
		WithHeaderHash(req.Hash).
		WithExecMode(sdk.ExecModeVoteExtension).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  req.Height,
			Hash:    req.Hash,
		})

	// add a deferred recover handler in case extendVote panics
	defer func() {
		if r := recover(); r != nil {
			app.logger.Error(
				"panic recovered in ExtendVote",
				"height", req.Height,
				"hash", fmt.Sprintf("%X", req.Hash),
				"panic", err,
			)
			err = fmt.Errorf("recovered application panic in ExtendVote: %v", r)
		}
	}()

	resp, err = app.extendVote(ctx, req)
	if err != nil {
		app.logger.Error("failed to extend vote", "height", req.Height, "hash", fmt.Sprintf("%X", req.Hash), "err", err)
		return &abci.ExtendVoteResponse{VoteExtension: []byte{}}, nil
	}

	return resp, err
}

// VerifyVoteExtension implements the VerifyVoteExtension ABCI method and returns
// a ResponseVerifyVoteExtension. It calls the applications' VerifyVoteExtension
// handler which is responsible for performing application-specific business
// logic in verifying a vote extension from another validator during the pre-commit
// phase. The response MUST be deterministic. An error is returned if vote
// extensions are not enabled or if verifyVoteExt fails or panics.
// We highly recommend a size validation due to performance degradation,
// see more here https://docs.cometbft.com/v1.0/references/qa/cometbft-qa-38#vote-extensions-testbed
func (app *BaseApp) VerifyVoteExtension(req *abci.VerifyVoteExtensionRequest) (resp *abci.VerifyVoteExtensionResponse, err error) {
	if app.verifyVoteExt == nil {
		return nil, errors.New("application VerifyVoteExtension handler not set")
	}

	var ctx sdk.Context

	// If we're verifying the vote for the initial height, we need to use the
	// finalizeBlockState context, otherwise we don't get the uncommitted data
	// from InitChain.
	if req.Height == app.initialHeight {
		ctx, _ = app.finalizeBlockState.Context().CacheContext()
	} else {
		ms := app.cms.CacheMultiStore()
		ctx = sdk.NewContext(ms, false, app.logger).WithStreamingManager(app.streamingManager).WithChainID(app.chainID).WithBlockHeight(req.Height)
	}

	// If vote extensions are not enabled, as a safety precaution, we return an
	// error.
	cp := app.GetConsensusParams(ctx)

	// Note: we verify votes extensions on VoteExtensionsEnableHeight+1. Check
	// comment in ExtendVote and ValidateVoteExtensions for more details.
	extsEnabled := cp.Feature.VoteExtensionsEnableHeight != nil && req.Height >= cp.Feature.VoteExtensionsEnableHeight.Value && cp.Feature.VoteExtensionsEnableHeight.Value != 0
	if !extsEnabled {
		// check abci params
		extsEnabled = cp.Abci != nil && req.Height >= cp.Abci.VoteExtensionsEnableHeight && cp.Abci.VoteExtensionsEnableHeight != 0
		if !extsEnabled {
			return nil, fmt.Errorf("vote extensions are not enabled; unexpected call to VerifyVoteExtension at height %d", req.Height)
		}
	}

	// add a deferred recover handler in case verifyVoteExt panics
	defer func() {
		if r := recover(); r != nil {
			app.logger.Error(
				"panic recovered in VerifyVoteExtension",
				"height", req.Height,
				"hash", fmt.Sprintf("%X", req.Hash),
				"validator", fmt.Sprintf("%X", req.ValidatorAddress),
				"panic", r,
			)
			err = fmt.Errorf("recovered application panic in VerifyVoteExtension: %v", r)
		}
	}()

	ctx = ctx.
		WithConsensusParams(cp).
		WithBlockGasMeter(storetypes.NewInfiniteGasMeter()).
		WithBlockHeight(req.Height).
		WithHeaderHash(req.Hash).
		WithExecMode(sdk.ExecModeVerifyVoteExtension).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  req.Height,
			Hash:    req.Hash,
		})

	resp, err = app.verifyVoteExt(ctx, req)
	if err != nil {
		app.logger.Error("failed to verify vote extension", "height", req.Height, "err", err)
		return &abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_REJECT}, nil
	}

	return resp, err
}

// internalFinalizeBlock executes the block, called by the Optimistic
// Execution flow or by the FinalizeBlock ABCI method. The context received is
// only used to handle early cancellation, for anything related to state app.finalizeBlockState.Context()
// must be used.
func (app *BaseApp) internalFinalizeBlock(ctx context.Context, req *abci.FinalizeBlockRequest) (*abci.FinalizeBlockResponse, error) {
	var events []abci.Event

	if err := app.checkHalt(req.Height, req.Time); err != nil {
		return nil, err
	}

	if err := app.validateFinalizeBlockHeight(req); err != nil {
		return nil, err
	}

	if app.cms.TracingEnabled() {
		app.cms.SetTracingContext(storetypes.TraceContext(
			map[string]any{"blockHeight": req.Height},
		))
	}

	header := cmtproto.Header{
		ChainID:            app.chainID,
		Height:             req.Height,
		Time:               req.Time,
		ProposerAddress:    req.ProposerAddress,
		NextValidatorsHash: req.NextValidatorsHash,
		AppHash:            app.LastCommitID().Hash,
	}

	// finalizeBlockState should be set on InitChain or ProcessProposal. If it is
	// nil, it means we are replaying this block and we need to set the state here
	// given that during block replay ProcessProposal is not executed by CometBFT.
	if app.finalizeBlockState == nil {
		app.setState(execModeFinalize, header)
	}

	// Context is now updated with Header information.
	app.finalizeBlockState.SetContext(app.finalizeBlockState.Context().
		WithBlockHeader(header).
		WithHeaderHash(req.Hash).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  req.Height,
			Time:    req.Time,
			Hash:    req.Hash,
			AppHash: app.LastCommitID().Hash,
		}).
		WithConsensusParams(app.GetConsensusParams(app.finalizeBlockState.Context())).
		WithVoteInfos(req.DecidedLastCommit.Votes).
		WithExecMode(sdk.ExecModeFinalize).
		WithCometInfo(corecomet.Info{
			Evidence:        sdk.ToSDKEvidence(req.Misbehavior),
			ValidatorsHash:  req.NextValidatorsHash,
			ProposerAddress: req.ProposerAddress,
			LastCommit:      sdk.ToSDKCommitInfo(req.DecidedLastCommit),
		}))

	// GasMeter must be set after we get a context with updated consensus params.
	gasMeter := app.getBlockGasMeter(app.finalizeBlockState.Context())
	app.finalizeBlockState.SetContext(app.finalizeBlockState.Context().WithBlockGasMeter(gasMeter))

	if app.checkState != nil {
		app.checkState.SetContext(app.checkState.Context().
			WithBlockGasMeter(gasMeter).
			WithHeaderHash(req.Hash))
	}

	preblockEvents, err := app.preBlock(req)
	if err != nil {
		return nil, err
	}
	events = append(events, preblockEvents...)

	beginBlock, err := app.beginBlock(req)
	if err != nil {
		return nil, err
	}

	// First check for an abort signal after beginBlock, as it's the first place
	// we spend any significant amount of time.
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
		// continue
	}

	events = append(events, beginBlock.Events...)

	// Reset the gas meter so that the AnteHandlers aren't required to
	gasMeter = app.getBlockGasMeter(app.finalizeBlockState.Context())
	app.finalizeBlockState.SetContext(app.finalizeBlockState.Context().WithBlockGasMeter(gasMeter))

	// Iterate over all raw transactions in the proposal and attempt to execute
	// them, gathering the execution results.
	//
	// NOTE: Not all raw transactions may adhere to the sdk.Tx interface, e.g.
	// vote extensions, so skip those.
	txResults := make([]*abci.ExecTxResult, 0, len(req.Txs))
	for txIndex, rawTx := range req.Txs {

		response := app.deliverTx(rawTx)

		// check after every tx if we should abort
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		default:
			// continue
		}

		// append the tx index to the response.Events
		for i, event := range response.Events {
			response.Events[i].Attributes = append(event.Attributes,
				abci.EventAttribute{Key: "tx_index", Value: strconv.Itoa(txIndex)})
		}

		txResults = append(txResults, response)
	}

	if app.finalizeBlockState.ms.TracingEnabled() {
		app.finalizeBlockState.ms = app.finalizeBlockState.ms.SetTracingContext(nil).(storetypes.CacheMultiStore)
	}

	endBlock, err := app.endBlock(app.finalizeBlockState.Context())
	if err != nil {
		return nil, err
	}

	// check after endBlock if we should abort, to avoid propagating the result
	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	default:
		// continue
	}

	events = append(events, endBlock.Events...)
	cp := app.GetConsensusParams(app.finalizeBlockState.Context())

	return &abci.FinalizeBlockResponse{
		Events:                events,
		TxResults:             txResults,
		ValidatorUpdates:      endBlock.ValidatorUpdates,
		ConsensusParamUpdates: &cp,
	}, nil
}

// FinalizeBlock will execute the block proposal provided by RequestFinalizeBlock.
// Specifically, it will execute an application's BeginBlock (if defined), followed
// by the transactions in the proposal, finally followed by the application's
// EndBlock (if defined).
//
// For each raw transaction, i.e. a byte slice, BaseApp will only execute it if
// it adheres to the sdk.Tx interface. Otherwise, the raw transaction will be
// skipped. This is to support compatibility with proposers injecting vote
// extensions into the proposal, which should not themselves be executed in cases
// where they adhere to the sdk.Tx interface.
func (app *BaseApp) FinalizeBlock(req *abci.FinalizeBlockRequest) (res *abci.FinalizeBlockResponse, err error) {
	defer func() {
		// call the streaming service hooks with the FinalizeBlock messages
		for _, streamingListener := range app.streamingManager.ABCIListeners {
			if err := streamingListener.ListenFinalizeBlock(app.finalizeBlockState.Context(), *req, *res); err != nil {
				app.logger.Error("ListenFinalizeBlock listening hook failed", "height", req.Height, "err", err)
			}
		}
	}()

	if app.optimisticExec.Initialized() {
		// check if the hash we got is the same as the one we are executing
		aborted := app.optimisticExec.AbortIfNeeded(req.Hash)
		// Wait for the OE to finish, regardless of whether it was aborted or not
		res, err = app.optimisticExec.WaitResult()

		// only return if we are not aborting
		if !aborted {
			if res != nil {
				res.AppHash = app.workingHash()
			}

			return res, err
		}

		// if it was aborted, we need to reset the state
		app.finalizeBlockState = nil
		app.optimisticExec.Reset()
	}

	// if no OE is running, just run the block (this is either a block replay or a OE that got aborted)
	res, err = app.internalFinalizeBlock(context.Background(), req)
	if res != nil {
		res.AppHash = app.workingHash()
	}

	return res, err
}

// checkHalt checks if height or time exceeds halt-height or halt-time respectively.
func (app *BaseApp) checkHalt(height int64, time time.Time) error {
	var halt bool
	switch {
	case app.haltHeight > 0 && uint64(height) >= app.haltHeight:
		halt = true

	case app.haltTime > 0 && time.Unix() >= int64(app.haltTime):
		halt = true
	}

	if halt {
		return fmt.Errorf("halt per configuration height %d time %d", app.haltHeight, app.haltTime)
	}

	return nil
}

// Commit implements the ABCI interface. It will commit all state that exists in
// the deliver state's multi-store and includes the resulting commit ID in the
// returned abci.ResponseCommit. Commit will set the check state based on the
// latest header and reset the deliver state. Also, if a non-zero halt height is
// defined in config, Commit will execute a deferred function call to check
// against that height and gracefully halt if it matches the latest committed
// height.
func (app *BaseApp) Commit() (*abci.CommitResponse, error) {
	header := app.finalizeBlockState.Context().BlockHeader()
	retainHeight := app.GetBlockRetentionHeight(header.Height)

	if app.precommiter != nil {
		app.precommiter(app.finalizeBlockState.Context())
	}

	rms, ok := app.cms.(*rootmulti.Store)
	if ok {
		rms.SetCommitHeader(header)
	}

	app.cms.Commit()

	resp := &abci.CommitResponse{
		RetainHeight: retainHeight,
	}

	abciListeners := app.streamingManager.ABCIListeners
	if len(abciListeners) > 0 {
		ctx := app.finalizeBlockState.Context()
		blockHeight := ctx.BlockHeight()
		changeSet := app.cms.PopStateCache()

		for _, abciListener := range abciListeners {
			if err := abciListener.ListenCommit(ctx, *resp, changeSet); err != nil {
				app.logger.Error("Commit listening hook failed", "height", blockHeight, "err", err)
			}
		}
	}

	// Reset the CheckTx state to the latest committed.
	//
	// NOTE: This is safe because CometBFT holds a lock on the mempool for
	// Commit. Use the header from this latest block.
	app.setState(execModeCheck, header)

	app.finalizeBlockState = nil

	if app.prepareCheckStater != nil {
		app.prepareCheckStater(app.checkState.Context())
	}

	// The SnapshotIfApplicable method will create the snapshot by starting the goroutine
	app.snapshotManager.SnapshotIfApplicable(header.Height)

	return resp, nil
}

// workingHash gets the apphash that will be finalized in commit.
// These writes will be persisted to the root multi-store (app.cms) and flushed to
// disk in the Commit phase. This means when the ABCI client requests Commit(), the application
// state transitions will be flushed to disk and as a result, but we already have
// an application Merkle root.
func (app *BaseApp) workingHash() []byte {
	// Write the FinalizeBlock state into branched storage and commit the MultiStore.
	// The write to the FinalizeBlock state writes all state transitions to the root
	// MultiStore (app.cms) so when Commit() is called it persists those values.
	app.finalizeBlockState.ms.Write()

	// Get the hash of all writes in order to return the apphash to the comet in finalizeBlock.
	commitHash := app.cms.WorkingHash()
	app.logger.Debug("hash of all writes", "workingHash", fmt.Sprintf("%X", commitHash))

	return commitHash
}

func handleQueryApp(app *BaseApp, path []string, req *abci.QueryRequest) *abci.QueryResponse {
	if len(path) >= 2 {
		switch path[1] {
		case "simulate":
			txBytes := req.Data

			gInfo, res, err := app.Simulate(txBytes)
			if err != nil {
				return queryResult(errorsmod.Wrap(err, "failed to simulate tx"), app.trace)
			}

			simRes := &sdk.SimulationResponse{
				GasInfo: gInfo,
				Result:  res,
			}

			bz, err := codec.ProtoMarshalJSON(simRes, app.interfaceRegistry)
			if err != nil {
				return queryResult(errorsmod.Wrap(err, "failed to JSON encode simulation response"), app.trace)
			}

			return &abci.QueryResponse{
				Codespace: sdkerrors.RootCodespace,
				Height:    req.Height,
				Value:     bz,
			}

		case "version":
			return &abci.QueryResponse{
				Codespace: sdkerrors.RootCodespace,
				Height:    req.Height,
				Value:     []byte(app.version),
			}

		default:
			return queryResult(errorsmod.Wrapf(sdkerrors.ErrUnknownRequest, "unknown query: %s", path), app.trace)
		}
	}

	return queryResult(
		errorsmod.Wrap(
			sdkerrors.ErrUnknownRequest,
			"expected second parameter to be either 'simulate' or 'version', neither was present",
		), app.trace)
}

func handleQueryStore(app *BaseApp, path []string, req abci.QueryRequest) *abci.QueryResponse {
	// "/store" prefix for store queries
	queryable, ok := app.cms.(storetypes.Queryable)
	if !ok {
		return queryResult(errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "multi-store does not support queries"), app.trace)
	}

	req.Path = "/" + strings.Join(path[1:], "/")

	if req.Height <= 1 && req.Prove {
		return queryResult(
			errorsmod.Wrap(
				sdkerrors.ErrInvalidRequest,
				"cannot query with proof when height <= 1; please provide a valid height",
			), app.trace)
	}

	sdkReq := storetypes.RequestQuery(req)
	resp, err := queryable.Query(&sdkReq)
	if err != nil {
		return queryResult(err, app.trace)
	}
	resp.Height = req.Height

	abciResp := abci.QueryResponse(*resp)

	return &abciResp
}

func handleQueryP2P(app *BaseApp, path []string) *abci.QueryResponse {
	// "/p2p" prefix for p2p queries
	if len(path) < 4 {
		return queryResult(errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "path should be p2p filter <addr|id> <parameter>"), app.trace)
	}

	var resp *abci.QueryResponse

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
		resp = queryResult(errorsmod.Wrap(sdkerrors.ErrUnknownRequest, "expected second parameter to be 'filter'"), app.trace)
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

// FilterPeerByAddrPort filters peers by address/port.
func (app *BaseApp) FilterPeerByAddrPort(info string) *abci.QueryResponse {
	if app.addrPeerFilter != nil {
		return app.addrPeerFilter(info)
	}

	return &abci.QueryResponse{}
}

// FilterPeerByID filters peers by node ID.
func (app *BaseApp) FilterPeerByID(info string) *abci.QueryResponse {
	if app.idPeerFilter != nil {
		return app.idPeerFilter(info)
	}

	return &abci.QueryResponse{}
}

// getContextForProposal returns the correct Context for PrepareProposal and
// ProcessProposal. We use finalizeBlockState on the first block to be able to
// access any state changes made in InitChain.
func (app *BaseApp) getContextForProposal(ctx sdk.Context, height int64) sdk.Context {
	if height == app.initialHeight {
		ctx, _ = app.finalizeBlockState.Context().CacheContext()

		// clear all context data set during InitChain to avoid inconsistent behavior
		ctx = ctx.WithHeaderInfo(coreheader.Info{}).WithBlockHeader(cmtproto.Header{})
		return ctx
	}

	return ctx
}

func (app *BaseApp) handleQueryGRPC(handler GRPCQueryHandler, req *abci.QueryRequest) *abci.QueryResponse {
	ctx, err := app.CreateQueryContext(req.Height, req.Prove)
	if err != nil {
		return queryResult(err, app.trace)
	}

	resp, err := handler(ctx, req)
	if err != nil {
		resp = queryResult(gRPCErrorToSDKError(err), app.trace)
		resp.Height = req.Height
		return resp
	}

	return resp
}

func gRPCErrorToSDKError(err error) error {
	status, ok := grpcstatus.FromError(err)
	if !ok {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, err.Error())
	}

	switch status.Code() {
	case codes.NotFound:
		return errorsmod.Wrap(sdkerrors.ErrKeyNotFound, err.Error())

	case codes.InvalidArgument:
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, err.Error())

	case codes.FailedPrecondition:
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, err.Error())

	case codes.Unauthenticated:
		return errorsmod.Wrap(sdkerrors.ErrUnauthorized, err.Error())

	default:
		return errorsmod.Wrap(sdkerrors.ErrUnknownRequest, err.Error())
	}
}

func checkNegativeHeight(height int64) error {
	if height < 0 {
		return errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "cannot query with height < 0; please provide a valid height")
	}

	return nil
}

// CreateQueryContext creates a new sdk.Context for a query, taking as args
// the block height and whether the query needs a proof or not.
func (app *BaseApp) CreateQueryContext(height int64, prove bool) (sdk.Context, error) {
	return app.CreateQueryContextWithCheckHeader(height, prove, true)
}

// CreateQueryContextWithCheckHeader creates a new sdk.Context for a query, taking as args
// the block height, whether the query needs a proof or not, and whether to check the header or not.
func (app *BaseApp) CreateQueryContextWithCheckHeader(height int64, prove, checkHeader bool) (sdk.Context, error) {
	if err := checkNegativeHeight(height); err != nil {
		return sdk.Context{}, err
	}

	// use custom query multi-store if provided
	qms := app.qms
	if qms == nil {
		qms = app.cms.(storetypes.MultiStore)
	}

	lastBlockHeight := qms.LatestVersion()
	if lastBlockHeight == 0 {
		return sdk.Context{}, errorsmod.Wrapf(sdkerrors.ErrInvalidHeight, "%s is not ready; please wait for first block", app.Name())
	}

	if height > lastBlockHeight {
		return sdk.Context{},
			errorsmod.Wrap(
				sdkerrors.ErrInvalidHeight,
				"cannot query with height in the future; please provide a valid height",
			)
	}

	if height > 0 && height <= 1 && prove {
		return sdk.Context{},
			errorsmod.Wrap(
				sdkerrors.ErrInvalidRequest,
				"cannot query with proof when height <= 1; please provide a valid height",
			)
	}

	var header *cmtproto.Header
	isLatest := height == 0
	for _, state := range []*state{
		app.checkState,
		app.finalizeBlockState,
	} {
		if state != nil {
			// branch the commit multi-store for safety
			h := state.Context().BlockHeader()
			if isLatest {
				lastBlockHeight = qms.LatestVersion()
			}
			if !checkHeader || !isLatest || isLatest && h.Height == lastBlockHeight {
				header = &h
				break
			}
		}
	}

	if header == nil {
		return sdk.Context{},
			errorsmod.Wrapf(
				sdkerrors.ErrInvalidHeight,
				"header height in all state context is not latest height (%d)", lastBlockHeight,
			)
	}

	// when a client did not provide a query height, manually inject the latest
	if isLatest {
		height = lastBlockHeight
	}

	cacheMS, err := qms.CacheMultiStoreWithVersion(height)
	if err != nil {
		return sdk.Context{},
			errorsmod.Wrapf(
				sdkerrors.ErrNotFound,
				"failed to load state at height %d; %s (latest height: %d)", height, err, lastBlockHeight,
			)
	}

	// branch the commit multi-store for safety
	ctx := sdk.NewContext(cacheMS, true, app.logger).
		WithMinGasPrices(app.minGasPrices).
		WithGasMeter(storetypes.NewGasMeter(app.queryGasLimit)).
		WithHeaderInfo(coreheader.Info{
			ChainID: app.chainID,
			Height:  height,
		}).
		WithBlockHeader(*header).
		WithBlockHeight(height)

	if !isLatest {
		rms, ok := app.cms.(*rootmulti.Store)
		if ok {
			cInfo, err := rms.GetCommitInfo(height)
			if cInfo != nil && err == nil {
				ctx = ctx.WithHeaderInfo(coreheader.Info{Height: height, Time: cInfo.Timestamp})
			}
		}
	}
	return ctx, nil
}

// GetBlockRetentionHeight returns the height for which all blocks below this height
// are pruned from CometBFT. Given a commitment height and a non-zero local
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
	// from CometBFT.
	var retentionHeight int64

	// Define the number of blocks needed to protect against misbehaving validators
	// which allows light clients to operate safely. Note, we piggy back of the
	// evidence parameters instead of computing an estimated number of blocks based
	// on the unbonding period and block commitment time as the two should be
	// equivalent.
	cp := app.GetConsensusParams(app.finalizeBlockState.Context())
	if cp.Evidence != nil && cp.Evidence.MaxAgeNumBlocks > 0 {
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

// toVoteInfo converts the new ExtendedVoteInfo to VoteInfo.
func toVoteInfo(votes []abci.ExtendedVoteInfo) []abci.VoteInfo {
	legacyVotes := make([]abci.VoteInfo, len(votes))
	for i, vote := range votes {
		legacyVotes[i] = abci.VoteInfo{
			Validator: abci.Validator{
				Address: vote.Validator.Address,
				Power:   vote.Validator.Power,
			},
			BlockIdFlag: vote.BlockIdFlag,
		}
	}

	return legacyVotes
}
