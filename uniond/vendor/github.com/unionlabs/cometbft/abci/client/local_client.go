package abcicli

import (
	"context"

	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

// NOTE: use defer to unlock mutex because Application might panic (e.g., in
// case of malicious tx or query). It only makes sense for publicly exposed
// methods like CheckTx (/broadcast_tx_* RPC endpoint) or Query (/abci_query
// RPC endpoint), but defers are used everywhere for the sake of consistency.
type localClient struct {
	service.BaseService

	mtx *cmtsync.Mutex
	types.Application
	Callback
}

var _ Client = (*localClient)(nil)

// NewLocalClient creates a local client, which wraps the application interface
// that Comet as the client will call to the application as the server.
//
// Concurrency control in each client instance is enforced by way of a single
// mutex. If a mutex is not supplied (i.e. if mtx is nil), then one will be
// created.
func NewLocalClient(mtx *cmtsync.Mutex, app types.Application) Client {
	if mtx == nil {
		mtx = new(cmtsync.Mutex)
	}
	cli := &localClient{
		mtx:         mtx,
		Application: app,
	}
	cli.BaseService = *service.NewBaseService(nil, "localClient", cli)
	return cli
}

func (app *localClient) SetResponseCallback(cb Callback) {
	app.mtx.Lock()
	app.Callback = cb
	app.mtx.Unlock()
}

func (app *localClient) CheckTxAsync(ctx context.Context, req *types.CheckTxRequest) (*ReqRes, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	res, err := app.Application.CheckTx(ctx, req)
	if err != nil {
		return nil, err
	}
	return app.callback(
		types.ToCheckTxRequest(req),
		types.ToCheckTxResponse(res),
	), nil
}

func (app *localClient) callback(req *types.Request, res *types.Response) *ReqRes {
	if app.Callback != nil {
		app.Callback(req, res)
	}
	rr := newLocalReqRes(req, res)
	rr.callbackInvoked = true
	return rr
}

func newLocalReqRes(req *types.Request, res *types.Response) *ReqRes {
	reqRes := NewReqRes(req)
	reqRes.Response = res
	reqRes.Done() // release waiters on response
	return reqRes
}

// -------------------------------------------------------

func (*localClient) Error() error {
	return nil
}

func (*localClient) Flush(context.Context) error {
	return nil
}

func (*localClient) Echo(_ context.Context, msg string) (*types.EchoResponse, error) {
	return &types.EchoResponse{Message: msg}, nil
}

func (app *localClient) Info(ctx context.Context, req *types.InfoRequest) (*types.InfoResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.Info(ctx, req)
}

func (app *localClient) CheckTx(ctx context.Context, req *types.CheckTxRequest) (*types.CheckTxResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.CheckTx(ctx, req)
}

func (app *localClient) Query(ctx context.Context, req *types.QueryRequest) (*types.QueryResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.Query(ctx, req)
}

func (app *localClient) Commit(ctx context.Context, req *types.CommitRequest) (*types.CommitResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.Commit(ctx, req)
}

func (app *localClient) InitChain(ctx context.Context, req *types.InitChainRequest) (*types.InitChainResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.InitChain(ctx, req)
}

func (app *localClient) ListSnapshots(ctx context.Context, req *types.ListSnapshotsRequest) (*types.ListSnapshotsResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.ListSnapshots(ctx, req)
}

func (app *localClient) OfferSnapshot(ctx context.Context, req *types.OfferSnapshotRequest) (*types.OfferSnapshotResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.OfferSnapshot(ctx, req)
}

func (app *localClient) LoadSnapshotChunk(ctx context.Context,
	req *types.LoadSnapshotChunkRequest,
) (*types.LoadSnapshotChunkResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.LoadSnapshotChunk(ctx, req)
}

func (app *localClient) ApplySnapshotChunk(ctx context.Context,
	req *types.ApplySnapshotChunkRequest,
) (*types.ApplySnapshotChunkResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.ApplySnapshotChunk(ctx, req)
}

func (app *localClient) PrepareProposal(ctx context.Context, req *types.PrepareProposalRequest) (*types.PrepareProposalResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.PrepareProposal(ctx, req)
}

func (app *localClient) ProcessProposal(ctx context.Context, req *types.ProcessProposalRequest) (*types.ProcessProposalResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.ProcessProposal(ctx, req)
}

func (app *localClient) ExtendVote(ctx context.Context, req *types.ExtendVoteRequest) (*types.ExtendVoteResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.ExtendVote(ctx, req)
}

func (app *localClient) VerifyVoteExtension(ctx context.Context, req *types.VerifyVoteExtensionRequest) (*types.VerifyVoteExtensionResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.VerifyVoteExtension(ctx, req)
}

func (app *localClient) FinalizeBlock(ctx context.Context, req *types.FinalizeBlockRequest) (*types.FinalizeBlockResponse, error) {
	app.mtx.Lock()
	defer app.mtx.Unlock()

	return app.Application.FinalizeBlock(ctx, req)
}
