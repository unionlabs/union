package abcicli

import (
	"fmt"
	"net"
	"sync"
	"time"

	"golang.org/x/net/context"
	"google.golang.org/grpc"

	"github.com/cometbft/cometbft/abci/types"
	cmtnet "github.com/cometbft/cometbft/libs/net"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

var _ Client = (*grpcClient)(nil)

// A stripped copy of the remoteClient that makes
// synchronous calls using grpc
type grpcClient struct {
	service.BaseService
	mustConnect bool

	client   types.ABCIApplicationClient
	conn     *grpc.ClientConn
	chReqRes chan *ReqRes // dispatches "async" responses to callbacks *in order*, needed by mempool

	mtx   cmtsync.Mutex
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

func dialerFunc(ctx context.Context, addr string) (net.Conn, error) {
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
		//nolint:staticcheck,nolintlint // SA1019 Existing use of deprecated but supported dial option.
		conn, err := grpc.Dial(cli.addr, grpc.WithInsecure(), grpc.WithContextDialer(dialerFunc))
		if err != nil {
			if cli.mustConnect {
				return err
			}
			cli.Logger.Error(fmt.Sprintf("abci.grpcClient failed to connect to %v.  Retrying...\n", cli.addr), "err", err)
			time.Sleep(time.Second * dialRetryIntervalSeconds)
			continue RETRY_LOOP
		}

		cli.Logger.Info("Dialed server. Waiting for echo.", "addr", cli.addr)
		client := types.NewABCIApplicationClient(conn)
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
// GRPC calls are synchronous, but some callbacks expect to be called asynchronously
// (eg. the mempool expects to be able to lock to remove bad txs from cache).
// To accommodate, we finish each call in its own go-routine,
// which is expensive, but easy - if you want something better, use the socket protocol!
// maybe one day, if people really want it, we use grpc streams,
// but hopefully not :D

func (cli *grpcClient) EchoAsync(msg string) *ReqRes {
	req := types.ToRequestEcho(msg)
	res, err := cli.client.Echo(context.Background(), req.GetEcho(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_Echo{Echo: res}})
}

func (cli *grpcClient) FlushAsync() *ReqRes {
	req := types.ToRequestFlush()
	res, err := cli.client.Flush(context.Background(), req.GetFlush(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_Flush{Flush: res}})
}

func (cli *grpcClient) InfoAsync(params types.RequestInfo) *ReqRes {
	req := types.ToRequestInfo(params)
	res, err := cli.client.Info(context.Background(), req.GetInfo(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_Info{Info: res}})
}

func (cli *grpcClient) DeliverTxAsync(params types.RequestDeliverTx) *ReqRes {
	req := types.ToRequestDeliverTx(params)
	res, err := cli.client.DeliverTx(context.Background(), req.GetDeliverTx(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_DeliverTx{DeliverTx: res}})
}

func (cli *grpcClient) CheckTxAsync(params types.RequestCheckTx) *ReqRes {
	req := types.ToRequestCheckTx(params)
	res, err := cli.client.CheckTx(context.Background(), req.GetCheckTx(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_CheckTx{CheckTx: res}})
}

func (cli *grpcClient) QueryAsync(params types.RequestQuery) *ReqRes {
	req := types.ToRequestQuery(params)
	res, err := cli.client.Query(context.Background(), req.GetQuery(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_Query{Query: res}})
}

func (cli *grpcClient) CommitAsync() *ReqRes {
	req := types.ToRequestCommit()
	res, err := cli.client.Commit(context.Background(), req.GetCommit(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_Commit{Commit: res}})
}

func (cli *grpcClient) InitChainAsync(params types.RequestInitChain) *ReqRes {
	req := types.ToRequestInitChain(params)
	res, err := cli.client.InitChain(context.Background(), req.GetInitChain(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_InitChain{InitChain: res}})
}

func (cli *grpcClient) BeginBlockAsync(params types.RequestBeginBlock) *ReqRes {
	req := types.ToRequestBeginBlock(params)
	res, err := cli.client.BeginBlock(context.Background(), req.GetBeginBlock(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_BeginBlock{BeginBlock: res}})
}

func (cli *grpcClient) EndBlockAsync(params types.RequestEndBlock) *ReqRes {
	req := types.ToRequestEndBlock(params)
	res, err := cli.client.EndBlock(context.Background(), req.GetEndBlock(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_EndBlock{EndBlock: res}})
}

func (cli *grpcClient) ListSnapshotsAsync(params types.RequestListSnapshots) *ReqRes {
	req := types.ToRequestListSnapshots(params)
	res, err := cli.client.ListSnapshots(context.Background(), req.GetListSnapshots(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_ListSnapshots{ListSnapshots: res}})
}

func (cli *grpcClient) OfferSnapshotAsync(params types.RequestOfferSnapshot) *ReqRes {
	req := types.ToRequestOfferSnapshot(params)
	res, err := cli.client.OfferSnapshot(context.Background(), req.GetOfferSnapshot(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_OfferSnapshot{OfferSnapshot: res}})
}

func (cli *grpcClient) LoadSnapshotChunkAsync(params types.RequestLoadSnapshotChunk) *ReqRes {
	req := types.ToRequestLoadSnapshotChunk(params)
	res, err := cli.client.LoadSnapshotChunk(context.Background(), req.GetLoadSnapshotChunk(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_LoadSnapshotChunk{LoadSnapshotChunk: res}})
}

func (cli *grpcClient) ApplySnapshotChunkAsync(params types.RequestApplySnapshotChunk) *ReqRes {
	req := types.ToRequestApplySnapshotChunk(params)
	res, err := cli.client.ApplySnapshotChunk(context.Background(), req.GetApplySnapshotChunk(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_ApplySnapshotChunk{ApplySnapshotChunk: res}})
}

func (cli *grpcClient) PrepareProposalAsync(params types.RequestPrepareProposal) *ReqRes {
	req := types.ToRequestPrepareProposal(params)
	res, err := cli.client.PrepareProposal(context.Background(), req.GetPrepareProposal(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}
	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_PrepareProposal{PrepareProposal: res}})
}

func (cli *grpcClient) ProcessProposalAsync(params types.RequestProcessProposal) *ReqRes {
	req := types.ToRequestProcessProposal(params)
	res, err := cli.client.ProcessProposal(context.Background(), req.GetProcessProposal(), grpc.WaitForReady(true))
	if err != nil {
		cli.StopForError(err)
	}

	return cli.finishAsyncCall(req, &types.Response{Value: &types.Response_ProcessProposal{ProcessProposal: res}})
}

// finishAsyncCall creates a ReqRes for an async call, and immediately populates it
// with the response. We don't complete it until it's been ordered via the channel.
func (cli *grpcClient) finishAsyncCall(req *types.Request, res *types.Response) *ReqRes {
	reqres := NewReqRes(req)
	reqres.Response = res
	cli.chReqRes <- reqres // use channel for async responses, since they must be ordered
	return reqres
}

// finishSyncCall waits for an async call to complete. It is necessary to call all
// sync calls asynchronously as well, to maintain call and response ordering via
// the channel, and this method will wait until the async call completes.
func (cli *grpcClient) finishSyncCall(reqres *ReqRes) *types.Response {
	// It's possible that the callback is called twice, since the callback can
	// be called immediately on SetCallback() in addition to after it has been
	// set. This is because completing the ReqRes happens in a separate critical
	// section from the one where the callback is called: there is a race where
	// SetCallback() is called between completing the ReqRes and dispatching the
	// callback.
	//
	// We also buffer the channel with 1 response, since SetCallback() will be
	// called synchronously if the reqres is already completed, in which case
	// it will block on sending to the channel since it hasn't gotten around to
	// receiving from it yet.
	//
	// ReqRes should really handle callback dispatch internally, to guarantee
	// that it's only called once and avoid the above race conditions.
	var once sync.Once
	ch := make(chan *types.Response, 1)
	reqres.SetCallback(func(res *types.Response) {
		once.Do(func() {
			ch <- res
		})
	})
	return <-ch
}

//----------------------------------------

func (cli *grpcClient) FlushSync() error {
	reqres := cli.FlushAsync()
	cli.finishSyncCall(reqres).GetFlush()
	return cli.Error()
}

func (cli *grpcClient) EchoSync(msg string) (*types.ResponseEcho, error) {
	reqres := cli.EchoAsync(msg)
	// StopForError should already have been called if error is set
	return cli.finishSyncCall(reqres).GetEcho(), cli.Error()
}

func (cli *grpcClient) InfoSync(req types.RequestInfo) (*types.ResponseInfo, error) {
	reqres := cli.InfoAsync(req)
	return cli.finishSyncCall(reqres).GetInfo(), cli.Error()
}

func (cli *grpcClient) DeliverTxSync(params types.RequestDeliverTx) (*types.ResponseDeliverTx, error) {
	reqres := cli.DeliverTxAsync(params)
	return cli.finishSyncCall(reqres).GetDeliverTx(), cli.Error()
}

func (cli *grpcClient) CheckTxSync(params types.RequestCheckTx) (*types.ResponseCheckTx, error) {
	reqres := cli.CheckTxAsync(params)
	return cli.finishSyncCall(reqres).GetCheckTx(), cli.Error()
}

func (cli *grpcClient) QuerySync(req types.RequestQuery) (*types.ResponseQuery, error) {
	reqres := cli.QueryAsync(req)
	return cli.finishSyncCall(reqres).GetQuery(), cli.Error()
}

func (cli *grpcClient) CommitSync() (*types.ResponseCommit, error) {
	reqres := cli.CommitAsync()
	return cli.finishSyncCall(reqres).GetCommit(), cli.Error()
}

func (cli *grpcClient) InitChainSync(params types.RequestInitChain) (*types.ResponseInitChain, error) {
	reqres := cli.InitChainAsync(params)
	return cli.finishSyncCall(reqres).GetInitChain(), cli.Error()
}

func (cli *grpcClient) BeginBlockSync(params types.RequestBeginBlock) (*types.ResponseBeginBlock, error) {
	reqres := cli.BeginBlockAsync(params)
	return cli.finishSyncCall(reqres).GetBeginBlock(), cli.Error()
}

func (cli *grpcClient) EndBlockSync(params types.RequestEndBlock) (*types.ResponseEndBlock, error) {
	reqres := cli.EndBlockAsync(params)
	return cli.finishSyncCall(reqres).GetEndBlock(), cli.Error()
}

func (cli *grpcClient) ListSnapshotsSync(params types.RequestListSnapshots) (*types.ResponseListSnapshots, error) {
	reqres := cli.ListSnapshotsAsync(params)
	return cli.finishSyncCall(reqres).GetListSnapshots(), cli.Error()
}

func (cli *grpcClient) OfferSnapshotSync(params types.RequestOfferSnapshot) (*types.ResponseOfferSnapshot, error) {
	reqres := cli.OfferSnapshotAsync(params)
	return cli.finishSyncCall(reqres).GetOfferSnapshot(), cli.Error()
}

func (cli *grpcClient) LoadSnapshotChunkSync(
	params types.RequestLoadSnapshotChunk,
) (*types.ResponseLoadSnapshotChunk, error) {
	reqres := cli.LoadSnapshotChunkAsync(params)
	return cli.finishSyncCall(reqres).GetLoadSnapshotChunk(), cli.Error()
}

func (cli *grpcClient) ApplySnapshotChunkSync(
	params types.RequestApplySnapshotChunk,
) (*types.ResponseApplySnapshotChunk, error) {
	reqres := cli.ApplySnapshotChunkAsync(params)
	return cli.finishSyncCall(reqres).GetApplySnapshotChunk(), cli.Error()
}

func (cli *grpcClient) PrepareProposalSync(
	params types.RequestPrepareProposal,
) (*types.ResponsePrepareProposal, error) {
	reqres := cli.PrepareProposalAsync(params)
	return cli.finishSyncCall(reqres).GetPrepareProposal(), cli.Error()
}

func (cli *grpcClient) ProcessProposalSync(params types.RequestProcessProposal) (*types.ResponseProcessProposal, error) {
	reqres := cli.ProcessProposalAsync(params)
	return cli.finishSyncCall(reqres).GetProcessProposal(), cli.Error()
}
