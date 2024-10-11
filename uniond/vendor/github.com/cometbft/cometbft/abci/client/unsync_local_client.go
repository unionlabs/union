package abcicli

import (
	"context"
	"sync"

	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/service"
)

type unsyncLocalClient struct {
	service.BaseService

	types.Application

	mtx sync.Mutex
	Callback
}

var _ Client = (*unsyncLocalClient)(nil)

// NewUnsyncLocalClient creates a local client, which wraps the application
// interface that Comet as the client will call to the application as the
// server.
//
// This differs from [NewLocalClient] in that it returns a client that only
// maintains a mutex over the callback used by CheckTxAsync and not over the
// application, leaving it up to the proxy to handle all concurrency. If the
// proxy does not impose any concurrency restrictions, it is then left up to
// the application to implement its own concurrency for the relevant group of
// calls.
func NewUnsyncLocalClient(app types.Application) Client {
	cli := &unsyncLocalClient{
		Application: app,
	}
	cli.BaseService = *service.NewBaseService(nil, "unsyncLocalClient", cli)
	return cli
}

func (app *unsyncLocalClient) SetResponseCallback(cb Callback) {
	app.mtx.Lock()
	app.Callback = cb
	app.mtx.Unlock()
}

func (app *unsyncLocalClient) CheckTxAsync(ctx context.Context, req *types.CheckTxRequest) (*ReqRes, error) {
	res, err := app.Application.CheckTx(ctx, req)
	if err != nil {
		return nil, err
	}
	return app.callback(
		types.ToCheckTxRequest(req),
		types.ToCheckTxResponse(res),
	), nil
}

func (app *unsyncLocalClient) callback(req *types.Request, res *types.Response) *ReqRes {
	if app.Callback != nil {
		app.Callback(req, res)
	}
	rr := newLocalReqRes(req, res)
	rr.callbackInvoked = true
	return rr
}

// -------------------------------------------------------

func (*unsyncLocalClient) Error() error {
	return nil
}

func (*unsyncLocalClient) Flush(context.Context) error {
	return nil
}

func (*unsyncLocalClient) Echo(_ context.Context, msg string) (*types.EchoResponse, error) {
	return &types.EchoResponse{Message: msg}, nil
}

func (app *unsyncLocalClient) Info(ctx context.Context, req *types.InfoRequest) (*types.InfoResponse, error) {
	return app.Application.Info(ctx, req)
}

func (app *unsyncLocalClient) CheckTx(ctx context.Context, req *types.CheckTxRequest) (*types.CheckTxResponse, error) {
	return app.Application.CheckTx(ctx, req)
}

func (app *unsyncLocalClient) Query(ctx context.Context, req *types.QueryRequest) (*types.QueryResponse, error) {
	return app.Application.Query(ctx, req)
}

func (app *unsyncLocalClient) Commit(ctx context.Context, req *types.CommitRequest) (*types.CommitResponse, error) {
	return app.Application.Commit(ctx, req)
}

func (app *unsyncLocalClient) InitChain(ctx context.Context, req *types.InitChainRequest) (*types.InitChainResponse, error) {
	return app.Application.InitChain(ctx, req)
}

func (app *unsyncLocalClient) ListSnapshots(ctx context.Context, req *types.ListSnapshotsRequest) (*types.ListSnapshotsResponse, error) {
	return app.Application.ListSnapshots(ctx, req)
}

func (app *unsyncLocalClient) OfferSnapshot(ctx context.Context, req *types.OfferSnapshotRequest) (*types.OfferSnapshotResponse, error) {
	return app.Application.OfferSnapshot(ctx, req)
}

func (app *unsyncLocalClient) LoadSnapshotChunk(ctx context.Context,
	req *types.LoadSnapshotChunkRequest,
) (*types.LoadSnapshotChunkResponse, error) {
	return app.Application.LoadSnapshotChunk(ctx, req)
}

func (app *unsyncLocalClient) ApplySnapshotChunk(ctx context.Context,
	req *types.ApplySnapshotChunkRequest,
) (*types.ApplySnapshotChunkResponse, error) {
	return app.Application.ApplySnapshotChunk(ctx, req)
}

func (app *unsyncLocalClient) PrepareProposal(ctx context.Context, req *types.PrepareProposalRequest) (*types.PrepareProposalResponse, error) {
	return app.Application.PrepareProposal(ctx, req)
}

func (app *unsyncLocalClient) ProcessProposal(ctx context.Context, req *types.ProcessProposalRequest) (*types.ProcessProposalResponse, error) {
	return app.Application.ProcessProposal(ctx, req)
}

func (app *unsyncLocalClient) ExtendVote(ctx context.Context, req *types.ExtendVoteRequest) (*types.ExtendVoteResponse, error) {
	return app.Application.ExtendVote(ctx, req)
}

func (app *unsyncLocalClient) VerifyVoteExtension(ctx context.Context, req *types.VerifyVoteExtensionRequest) (*types.VerifyVoteExtensionResponse, error) {
	return app.Application.VerifyVoteExtension(ctx, req)
}

func (app *unsyncLocalClient) FinalizeBlock(ctx context.Context, req *types.FinalizeBlockRequest) (*types.FinalizeBlockResponse, error) {
	return app.Application.FinalizeBlock(ctx, req)
}
