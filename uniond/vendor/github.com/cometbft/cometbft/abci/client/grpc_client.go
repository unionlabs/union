package abcicli

import (
	"context"
	"fmt"
	"net"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"github.com/cometbft/cometbft/abci/types"
	cmtnet "github.com/cometbft/cometbft/libs/net"
	"github.com/cometbft/cometbft/libs/service"
)

var _ Client = (*grpcClient)(nil)

// A stripped copy of the remoteClient that makes
// synchronous calls using grpc
type grpcClient struct {
	service.BaseService
	mustConnect bool

	client   types.ABCIClient
	conn     *grpc.ClientConn
	chReqRes chan *ReqRes // dispatches "async" responses to callbacks *in order*, needed by mempool

	mtx   sync.Mutex
	addr  string
	err   error
	resCb func(*types.Request, *types.Response) // listens to all callbacks
}

func NewGRPCClient(addr string, mustConnect bool) Client {
	cli := &grpcClient{
		addr:        addr,
		mustConnect: mustConnect,
		// Buffering the channel is needed to make calls appear asynchronous,
		// which is required when the caller makes multiple async calls before
		// processing callbacks (e.g. due to holding locks). 64 means that a
		// caller can make up to 64 async calls before a callback must be
		// processed (otherwise it deadlocks). It also means that we can make 64
		// gRPC calls while processing a slow callback at the channel head.
		chReqRes: make(chan *ReqRes, 64),
	}
	cli.BaseService = *service.NewBaseService(nil, "grpcClient", cli)
	return cli
}

func dialerFunc(_ context.Context, addr string) (net.Conn, error) {
	return cmtnet.Connect(addr)
}

func (cli *grpcClient) OnStart() error {
	if err := cli.BaseService.OnStart(); err != nil {
		return err
	}

	// This processes asynchronous request/response messages and dispatches
	// them to callbacks.
	go func() {
		// Use a separate function to use defer for mutex unlocks (this handles panics)
		callCb := func(reqres *ReqRes) {
			cli.mtx.Lock()
			defer cli.mtx.Unlock()

			reqres.Done()

			// Notify client listener if set
			if cli.resCb != nil {
				cli.resCb(reqres.Request, reqres.Response)
			}

			// Notify reqRes listener if set
			reqres.InvokeCallback()
		}
		for reqres := range cli.chReqRes {
			if reqres != nil {
				callCb(reqres)
			} else {
				cli.Logger.Error("Received nil reqres")
			}
		}
	}()

RETRY_LOOP:
	for {
		conn, err := grpc.Dial(cli.addr,
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithContextDialer(dialerFunc),
		)
		if err != nil {
			if cli.mustConnect {
				return err
			}
			cli.Logger.Error(fmt.Sprintf("abci.grpcClient failed to connect to %v.  Retrying...\n", cli.addr), "err", err)
			time.Sleep(time.Second * dialRetryIntervalSeconds)
			continue RETRY_LOOP
		}

		cli.Logger.Info("Dialed server. Waiting for echo.", "addr", cli.addr)
		client := types.NewABCIClient(conn)
		cli.conn = conn

	ENSURE_CONNECTED:
		for {
			_, err := client.Echo(context.Background(), &types.RequestEcho{Message: "hello"}, grpc.WaitForReady(true))
			if err == nil {
				break ENSURE_CONNECTED
			}
			cli.Logger.Error("Echo failed", "err", err)
			time.Sleep(time.Second * echoRetryIntervalSeconds)
		}

		cli.client = client
		return nil
	}
}

func (cli *grpcClient) OnStop() {
	cli.BaseService.OnStop()

	if cli.conn != nil {
		cli.conn.Close()
	}
	close(cli.chReqRes)
}

func (cli *grpcClient) StopForError(err error) {
	if !cli.IsRunning() {
		return
	}

	cli.mtx.Lock()
	if cli.err == nil {
		cli.err = err
	}
	cli.mtx.Unlock()

	cli.Logger.Error(fmt.Sprintf("Stopping abci.grpcClient for error: %v", err.Error()))
	if err := cli.Stop(); err != nil {
		cli.Logger.Error("Error stopping abci.grpcClient", "err", err)
	}
}

func (cli *grpcClient) Error() error {
	cli.mtx.Lock()
	defer cli.mtx.Unlock()
	return cli.err
}

// Set listener for all responses
// NOTE: callback may get internally generated flush responses.
func (cli *grpcClient) SetResponseCallback(resCb Callback) {
	cli.mtx.Lock()
	cli.resCb = resCb
	cli.mtx.Unlock()
}

//----------------------------------------

func (cli *grpcClient) CheckTxAsync(ctx context.Context, req *types.RequestCheckTx) (*ReqRes, error) {
	res, err := cli.client.CheckTx(ctx, req, grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
		return nil, err
	}
	return cli.finishAsyncCall(types.ToRequestCheckTx(req), &types.Response{Value: &types.Response_CheckTx{CheckTx: res}}), nil
}

// finishAsyncCall creates a ReqRes for an async call, and immediately populates it
// with the response. We don't complete it until it's been ordered via the channel.
func (cli *grpcClient) finishAsyncCall(req *types.Request, res *types.Response) *ReqRes {
	reqres := NewReqRes(req)
	reqres.Response = res
	cli.chReqRes <- reqres // use channel for async responses, since they must be ordered
	return reqres
}

//----------------------------------------

func (cli *grpcClient) Flush(ctx context.Context) error {
	_, err := cli.client.Flush(ctx, types.ToRequestFlush().GetFlush(), grpc.WaitForReady(true))
	return err
}

func (cli *grpcClient) Echo(ctx context.Context, msg string) (*types.ResponseEcho, error) {
	return cli.client.Echo(ctx, types.ToRequestEcho(msg).GetEcho(), grpc.WaitForReady(true))
}

func (cli *grpcClient) Info(ctx context.Context, req *types.RequestInfo) (*types.ResponseInfo, error) {
	return cli.client.Info(ctx, req, grpc.WaitForReady(true))
}

func (cli *grpcClient) CheckTx(ctx context.Context, req *types.RequestCheckTx) (*types.ResponseCheckTx, error) {
	return cli.client.CheckTx(ctx, req, grpc.WaitForReady(true))
}

func (cli *grpcClient) Query(ctx context.Context, req *types.RequestQuery) (*types.ResponseQuery, error) {
	return cli.client.Query(ctx, types.ToRequestQuery(req).GetQuery(), grpc.WaitForReady(true))
}

func (cli *grpcClient) Commit(ctx context.Context, _ *types.RequestCommit) (*types.ResponseCommit, error) {
	return cli.client.Commit(ctx, types.ToRequestCommit().GetCommit(), grpc.WaitForReady(true))
}

func (cli *grpcClient) InitChain(ctx context.Context, req *types.RequestInitChain) (*types.ResponseInitChain, error) {
	return cli.client.InitChain(ctx, types.ToRequestInitChain(req).GetInitChain(), grpc.WaitForReady(true))
}

func (cli *grpcClient) ListSnapshots(ctx context.Context, req *types.RequestListSnapshots) (*types.ResponseListSnapshots, error) {
	return cli.client.ListSnapshots(ctx, types.ToRequestListSnapshots(req).GetListSnapshots(), grpc.WaitForReady(true))
}

func (cli *grpcClient) OfferSnapshot(ctx context.Context, req *types.RequestOfferSnapshot) (*types.ResponseOfferSnapshot, error) {
	return cli.client.OfferSnapshot(ctx, types.ToRequestOfferSnapshot(req).GetOfferSnapshot(), grpc.WaitForReady(true))
}

func (cli *grpcClient) LoadSnapshotChunk(ctx context.Context, req *types.RequestLoadSnapshotChunk) (*types.ResponseLoadSnapshotChunk, error) {
	return cli.client.LoadSnapshotChunk(ctx, types.ToRequestLoadSnapshotChunk(req).GetLoadSnapshotChunk(), grpc.WaitForReady(true))
}

func (cli *grpcClient) ApplySnapshotChunk(ctx context.Context, req *types.RequestApplySnapshotChunk) (*types.ResponseApplySnapshotChunk, error) {
	return cli.client.ApplySnapshotChunk(ctx, types.ToRequestApplySnapshotChunk(req).GetApplySnapshotChunk(), grpc.WaitForReady(true))
}

func (cli *grpcClient) PrepareProposal(ctx context.Context, req *types.RequestPrepareProposal) (*types.ResponsePrepareProposal, error) {
	return cli.client.PrepareProposal(ctx, types.ToRequestPrepareProposal(req).GetPrepareProposal(), grpc.WaitForReady(true))
}

func (cli *grpcClient) ProcessProposal(ctx context.Context, req *types.RequestProcessProposal) (*types.ResponseProcessProposal, error) {
	return cli.client.ProcessProposal(ctx, types.ToRequestProcessProposal(req).GetProcessProposal(), grpc.WaitForReady(true))
}

func (cli *grpcClient) ExtendVote(ctx context.Context, req *types.RequestExtendVote) (*types.ResponseExtendVote, error) {
	return cli.client.ExtendVote(ctx, types.ToRequestExtendVote(req).GetExtendVote(), grpc.WaitForReady(true))
}

func (cli *grpcClient) VerifyVoteExtension(ctx context.Context, req *types.RequestVerifyVoteExtension) (*types.ResponseVerifyVoteExtension, error) {
	return cli.client.VerifyVoteExtension(ctx, types.ToRequestVerifyVoteExtension(req).GetVerifyVoteExtension(), grpc.WaitForReady(true))
}

func (cli *grpcClient) FinalizeBlock(ctx context.Context, req *types.RequestFinalizeBlock) (*types.ResponseFinalizeBlock, error) {
	return cli.client.FinalizeBlock(ctx, types.ToRequestFinalizeBlock(req).GetFinalizeBlock(), grpc.WaitForReady(true))
}
