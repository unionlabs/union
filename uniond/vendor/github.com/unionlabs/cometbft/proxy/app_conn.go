package proxy

import (
	"context"
	"time"

	"github.com/go-kit/kit/metrics"

	abcicli "github.com/cometbft/cometbft/abci/client"
	abcitypes "github.com/cometbft/cometbft/abci/types"
)

//go:generate ../scripts/mockery_generate.sh AppConnConsensus|AppConnMempool|AppConnQuery|AppConnSnapshot

// ----------------------------------------------------------------------------------------
// Enforce which abci msgs can be sent on a connection at the type level

type AppConnConsensus interface {
	Error() error
	InitChain(ctx context.Context, req *abcitypes.InitChainRequest) (*abcitypes.InitChainResponse, error)
	PrepareProposal(ctx context.Context, req *abcitypes.PrepareProposalRequest) (*abcitypes.PrepareProposalResponse, error)
	ProcessProposal(ctx context.Context, req *abcitypes.ProcessProposalRequest) (*abcitypes.ProcessProposalResponse, error)
	ExtendVote(ctx context.Context, req *abcitypes.ExtendVoteRequest) (*abcitypes.ExtendVoteResponse, error)
	VerifyVoteExtension(ctx context.Context, req *abcitypes.VerifyVoteExtensionRequest) (*abcitypes.VerifyVoteExtensionResponse, error)
	FinalizeBlock(ctx context.Context, req *abcitypes.FinalizeBlockRequest) (*abcitypes.FinalizeBlockResponse, error)
	Commit(ctx context.Context) (*abcitypes.CommitResponse, error)
}

type AppConnMempool interface {
	SetResponseCallback(cb abcicli.Callback)
	Error() error

	CheckTx(ctx context.Context, req *abcitypes.CheckTxRequest) (*abcitypes.CheckTxResponse, error)
	CheckTxAsync(ctx context.Context, req *abcitypes.CheckTxRequest) (*abcicli.ReqRes, error)
	Flush(ctx context.Context) error
}

type AppConnQuery interface {
	Error() error

	Echo(ctx context.Context, echo string) (*abcitypes.EchoResponse, error)
	Info(ctx context.Context, req *abcitypes.InfoRequest) (*abcitypes.InfoResponse, error)
	Query(ctx context.Context, req *abcitypes.QueryRequest) (*abcitypes.QueryResponse, error)
}

type AppConnSnapshot interface {
	Error() error

	ListSnapshots(ctx context.Context, req *abcitypes.ListSnapshotsRequest) (*abcitypes.ListSnapshotsResponse, error)
	OfferSnapshot(ctx context.Context, req *abcitypes.OfferSnapshotRequest) (*abcitypes.OfferSnapshotResponse, error)
	LoadSnapshotChunk(ctx context.Context, req *abcitypes.LoadSnapshotChunkRequest) (*abcitypes.LoadSnapshotChunkResponse, error)
	ApplySnapshotChunk(ctx context.Context, req *abcitypes.ApplySnapshotChunkRequest) (*abcitypes.ApplySnapshotChunkResponse, error)
}

// -----------------------------------------------------------------------------------------
// Implements AppConnConsensus (subset of abcicli.Client)

type appConnConsensus struct {
	metrics *Metrics
	appConn abcicli.Client
}

var _ AppConnConsensus = (*appConnConsensus)(nil)

func NewAppConnConsensus(appConn abcicli.Client, metrics *Metrics) AppConnConsensus {
	return &appConnConsensus{
		metrics: metrics,
		appConn: appConn,
	}
}

func (app *appConnConsensus) Error() error {
	return app.appConn.Error()
}

func (app *appConnConsensus) InitChain(ctx context.Context, req *abcitypes.InitChainRequest) (*abcitypes.InitChainResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "init_chain", "type", "sync"))()
	return app.appConn.InitChain(ctx, req)
}

func (app *appConnConsensus) PrepareProposal(ctx context.Context,
	req *abcitypes.PrepareProposalRequest,
) (*abcitypes.PrepareProposalResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "prepare_proposal", "type", "sync"))()
	return app.appConn.PrepareProposal(ctx, req)
}

func (app *appConnConsensus) ProcessProposal(ctx context.Context, req *abcitypes.ProcessProposalRequest) (*abcitypes.ProcessProposalResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "process_proposal", "type", "sync"))()
	return app.appConn.ProcessProposal(ctx, req)
}

func (app *appConnConsensus) ExtendVote(ctx context.Context, req *abcitypes.ExtendVoteRequest) (*abcitypes.ExtendVoteResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "extend_vote", "type", "sync"))()
	return app.appConn.ExtendVote(ctx, req)
}

func (app *appConnConsensus) VerifyVoteExtension(ctx context.Context, req *abcitypes.VerifyVoteExtensionRequest) (*abcitypes.VerifyVoteExtensionResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "verify_vote_extension", "type", "sync"))()
	return app.appConn.VerifyVoteExtension(ctx, req)
}

func (app *appConnConsensus) FinalizeBlock(ctx context.Context, req *abcitypes.FinalizeBlockRequest) (*abcitypes.FinalizeBlockResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "finalize_block", "type", "sync"))()
	return app.appConn.FinalizeBlock(ctx, req)
}

func (app *appConnConsensus) Commit(ctx context.Context) (*abcitypes.CommitResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "commit", "type", "sync"))()
	return app.appConn.Commit(ctx, &abcitypes.CommitRequest{})
}

// ------------------------------------------------
// Implements AppConnMempool (subset of abcicli.Client)

type appConnMempool struct {
	metrics *Metrics
	appConn abcicli.Client
}

func NewAppConnMempool(appConn abcicli.Client, metrics *Metrics) AppConnMempool {
	return &appConnMempool{
		metrics: metrics,
		appConn: appConn,
	}
}

// Deprecated: Do not use.
func (app *appConnMempool) SetResponseCallback(cb abcicli.Callback) {
	app.appConn.SetResponseCallback(cb)
}

func (app *appConnMempool) Error() error {
	return app.appConn.Error()
}

func (app *appConnMempool) Flush(ctx context.Context) error {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "flush", "type", "sync"))()
	return app.appConn.Flush(ctx)
}

func (app *appConnMempool) CheckTx(ctx context.Context, req *abcitypes.CheckTxRequest) (*abcitypes.CheckTxResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "check_tx", "type", "sync"))()
	return app.appConn.CheckTx(ctx, req)
}

func (app *appConnMempool) CheckTxAsync(ctx context.Context, req *abcitypes.CheckTxRequest) (*abcicli.ReqRes, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "check_tx", "type", "async"))()
	return app.appConn.CheckTxAsync(ctx, req)
}

// ------------------------------------------------
// Implements AppConnQuery (subset of abcicli.Client)

type appConnQuery struct {
	metrics *Metrics
	appConn abcicli.Client
}

func NewAppConnQuery(appConn abcicli.Client, metrics *Metrics) AppConnQuery {
	return &appConnQuery{
		metrics: metrics,
		appConn: appConn,
	}
}

func (app *appConnQuery) Error() error {
	return app.appConn.Error()
}

func (app *appConnQuery) Echo(ctx context.Context, msg string) (*abcitypes.EchoResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "echo", "type", "sync"))()
	return app.appConn.Echo(ctx, msg)
}

func (app *appConnQuery) Info(ctx context.Context, req *abcitypes.InfoRequest) (*abcitypes.InfoResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "info", "type", "sync"))()
	return app.appConn.Info(ctx, req)
}

func (app *appConnQuery) Query(ctx context.Context, req *abcitypes.QueryRequest) (*abcitypes.QueryResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "query", "type", "sync"))()
	return app.appConn.Query(ctx, req)
}

// ------------------------------------------------
// Implements AppConnSnapshot (subset of abcicli.Client)

type appConnSnapshot struct {
	metrics *Metrics
	appConn abcicli.Client
}

func NewAppConnSnapshot(appConn abcicli.Client, metrics *Metrics) AppConnSnapshot {
	return &appConnSnapshot{
		metrics: metrics,
		appConn: appConn,
	}
}

func (app *appConnSnapshot) Error() error {
	return app.appConn.Error()
}

func (app *appConnSnapshot) ListSnapshots(ctx context.Context, req *abcitypes.ListSnapshotsRequest) (*abcitypes.ListSnapshotsResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "list_snapshots", "type", "sync"))()
	return app.appConn.ListSnapshots(ctx, req)
}

func (app *appConnSnapshot) OfferSnapshot(ctx context.Context, req *abcitypes.OfferSnapshotRequest) (*abcitypes.OfferSnapshotResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "offer_snapshot", "type", "sync"))()
	return app.appConn.OfferSnapshot(ctx, req)
}

func (app *appConnSnapshot) LoadSnapshotChunk(ctx context.Context, req *abcitypes.LoadSnapshotChunkRequest) (*abcitypes.LoadSnapshotChunkResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "load_snapshot_chunk", "type", "sync"))()
	return app.appConn.LoadSnapshotChunk(ctx, req)
}

func (app *appConnSnapshot) ApplySnapshotChunk(ctx context.Context, req *abcitypes.ApplySnapshotChunkRequest) (*abcitypes.ApplySnapshotChunkResponse, error) {
	defer addTimeSample(app.metrics.MethodTimingSeconds.With("method", "apply_snapshot_chunk", "type", "sync"))()
	return app.appConn.ApplySnapshotChunk(ctx, req)
}

// addTimeSample returns a function that, when called, adds an observation to m.
// The observation added to m is the number of seconds elapsed since addTimeSample
// was initially called. addTimeSample is meant to be called in a defer to calculate
// the amount of time a function takes to complete.
func addTimeSample(m metrics.Histogram) func() {
	start := time.Now()
	return func() { m.Observe(time.Since(start).Seconds()) }
}
