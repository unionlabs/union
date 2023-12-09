package http

import (
	"context"
	"errors"
	"net/http"
	"strings"
	"time"

	"github.com/cometbft/cometbft/libs/bytes"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	jsonrpcclient "github.com/cometbft/cometbft/rpc/jsonrpc/client"
	"github.com/cometbft/cometbft/types"
)

/*
HTTP is a Client implementation that communicates with a CometBFT node over
JSON RPC and WebSockets.

This is the main implementation you probably want to use in production code.
There are other implementations when calling the CometBFT node in-process
(Local), or when you want to mock out the server for test code (mock).

You can subscribe for any event published by CometBFT using Subscribe method.
Note delivery is best-effort. If you don't read events fast enough or network is
slow, CometBFT might cancel the subscription. The client will attempt to
resubscribe (you don't need to do anything). It will keep trying every second
indefinitely until successful.

Request batching is available for JSON RPC requests over HTTP, which conforms to
the JSON RPC specification (https://www.jsonrpc.org/specification#batch). See
the example for more details.

Example:

	c, err := New("http://192.168.1.10:26657", "/websocket")
	if err != nil {
		// handle error
	}

	// call Start/Stop if you're subscribing to events
	err = c.Start()
	if err != nil {
		// handle error
	}
	defer c.Stop()

	res, err := c.Status()
	if err != nil {
		// handle error
	}

	// handle result
*/
type HTTP struct {
	remote string
	rpc    *jsonrpcclient.Client

	*baseRPCClient
	*WSEvents
}

// BatchHTTP provides the same interface as `HTTP`, but allows for batching of
// requests (as per https://www.jsonrpc.org/specification#batch). Do not
// instantiate directly - rather use the HTTP.NewBatch() method to create an
// instance of this struct.
//
// Batching of HTTP requests is thread-safe in the sense that multiple
// goroutines can each create their own batches and send them using the same
// HTTP client. Multiple goroutines could also enqueue transactions in a single
// batch, but ordering of transactions in the batch cannot be guaranteed in such
// an example.
type BatchHTTP struct {
	rpcBatch *jsonrpcclient.RequestBatch
	*baseRPCClient
}

// rpcClient is an internal interface to which our RPC clients (batch and
// non-batch) must conform. Acts as an additional code-level sanity check to
// make sure the implementations stay coherent.
type rpcClient interface {
	rpcclient.ABCIClient
	rpcclient.HistoryClient
	rpcclient.NetworkClient
	rpcclient.SignClient
	rpcclient.StatusClient
}

// baseRPCClient implements the basic RPC method logic without the actual
// underlying RPC call functionality, which is provided by `caller`.
type baseRPCClient struct {
	caller jsonrpcclient.Caller
}

var (
	_ rpcClient = (*HTTP)(nil)
	_ rpcClient = (*BatchHTTP)(nil)
	_ rpcClient = (*baseRPCClient)(nil)
)

//-----------------------------------------------------------------------------
// HTTP

// New takes a remote endpoint in the form <protocol>://<host>:<port> and
// the websocket path (which always seems to be "/websocket")
// An error is returned on invalid remote. The function panics when remote is nil.
func New(remote, wsEndpoint string) (*HTTP, error) {
	httpClient, err := jsonrpcclient.DefaultHTTPClient(remote)
	if err != nil {
		return nil, err
	}
	return NewWithClient(remote, wsEndpoint, httpClient)
}

// Create timeout enabled http client
func NewWithTimeout(remote, wsEndpoint string, timeout uint) (*HTTP, error) {
	httpClient, err := jsonrpcclient.DefaultHTTPClient(remote)
	if err != nil {
		return nil, err
	}
	httpClient.Timeout = time.Duration(timeout) * time.Second
	return NewWithClient(remote, wsEndpoint, httpClient)
}

// NewWithClient allows for setting a custom http client (See New).
// An error is returned on invalid remote. The function panics when remote is nil.
func NewWithClient(remote, wsEndpoint string, client *http.Client) (*HTTP, error) {
	if client == nil {
		panic("nil http.Client provided")
	}

	rc, err := jsonrpcclient.NewWithHTTPClient(remote, client)
	if err != nil {
		return nil, err
	}

	wsEvents, err := newWSEvents(remote, wsEndpoint)
	if err != nil {
		return nil, err
	}

	httpClient := &HTTP{
		rpc:           rc,
		remote:        remote,
		baseRPCClient: &baseRPCClient{caller: rc},
		WSEvents:      wsEvents,
	}

	return httpClient, nil
}

var _ rpcclient.Client = (*HTTP)(nil)

// SetLogger sets a logger.
func (c *HTTP) SetLogger(l log.Logger) {
	c.WSEvents.SetLogger(l)
}

// Remote returns the remote network address in a string form.
func (c *HTTP) Remote() string {
	return c.remote
}

// NewBatch creates a new batch client for this HTTP client.
func (c *HTTP) NewBatch() *BatchHTTP {
	rpcBatch := c.rpc.NewRequestBatch()
	return &BatchHTTP{
		rpcBatch: rpcBatch,
		baseRPCClient: &baseRPCClient{
			caller: rpcBatch,
		},
	}
}

//-----------------------------------------------------------------------------
// BatchHTTP

// Send is a convenience function for an HTTP batch that will trigger the
// compilation of the batched requests and send them off using the client as a
// single request. On success, this returns a list of the deserialized results
// from each request in the sent batch.
func (b *BatchHTTP) Send(ctx context.Context) ([]interface{}, error) {
	return b.rpcBatch.Send(ctx)
}

// Clear will empty out this batch of requests and return the number of requests
// that were cleared out.
func (b *BatchHTTP) Clear() int {
	return b.rpcBatch.Clear()
}

// Count returns the number of enqueued requests waiting to be sent.
func (b *BatchHTTP) Count() int {
	return b.rpcBatch.Count()
}

//-----------------------------------------------------------------------------
// baseRPCClient

func (c *baseRPCClient) Status(ctx context.Context) (*ctypes.ResultStatus, error) {
	result := new(ctypes.ResultStatus)
	_, err := c.caller.Call(ctx, "status", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (c *baseRPCClient) ABCIInfo(ctx context.Context) (*ctypes.ResultABCIInfo, error) {
	result := new(ctypes.ResultABCIInfo)
	_, err := c.caller.Call(ctx, "abci_info", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (c *baseRPCClient) ABCIQuery(
	ctx context.Context,
	path string,
	data bytes.HexBytes,
) (*ctypes.ResultABCIQuery, error) {
	return c.ABCIQueryWithOptions(ctx, path, data, rpcclient.DefaultABCIQueryOptions)
}

func (c *baseRPCClient) ABCIQueryWithOptions(
	ctx context.Context,
	path string,
	data bytes.HexBytes,
	opts rpcclient.ABCIQueryOptions,
) (*ctypes.ResultABCIQuery, error) {
	result := new(ctypes.ResultABCIQuery)
	_, err := c.caller.Call(ctx, "abci_query",
		map[string]interface{}{"path": path, "data": data, "height": opts.Height, "prove": opts.Prove},
		result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (c *baseRPCClient) BroadcastTxCommit(
	ctx context.Context,
	tx types.Tx,
) (*ctypes.ResultBroadcastTxCommit, error) {
	result := new(ctypes.ResultBroadcastTxCommit)
	_, err := c.caller.Call(ctx, "broadcast_tx_commit", map[string]interface{}{"tx": tx}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) BroadcastTxAsync(
	ctx context.Context,
	tx types.Tx,
) (*ctypes.ResultBroadcastTx, error) {
	return c.broadcastTX(ctx, "broadcast_tx_async", tx)
}

func (c *baseRPCClient) BroadcastTxSync(
	ctx context.Context,
	tx types.Tx,
) (*ctypes.ResultBroadcastTx, error) {
	return c.broadcastTX(ctx, "broadcast_tx_sync", tx)
}

func (c *baseRPCClient) broadcastTX(
	ctx context.Context,
	route string,
	tx types.Tx,
) (*ctypes.ResultBroadcastTx, error) {
	result := new(ctypes.ResultBroadcastTx)
	_, err := c.caller.Call(ctx, route, map[string]interface{}{"tx": tx}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) UnconfirmedTxs(
	ctx context.Context,
	limit *int,
) (*ctypes.ResultUnconfirmedTxs, error) {
	result := new(ctypes.ResultUnconfirmedTxs)
	params := make(map[string]interface{})
	if limit != nil {
		params["limit"] = limit
	}
	_, err := c.caller.Call(ctx, "unconfirmed_txs", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) NumUnconfirmedTxs(ctx context.Context) (*ctypes.ResultUnconfirmedTxs, error) {
	result := new(ctypes.ResultUnconfirmedTxs)
	_, err := c.caller.Call(ctx, "num_unconfirmed_txs", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) CheckTx(ctx context.Context, tx types.Tx) (*ctypes.ResultCheckTx, error) {
	result := new(ctypes.ResultCheckTx)
	_, err := c.caller.Call(ctx, "check_tx", map[string]interface{}{"tx": tx}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) NetInfo(ctx context.Context) (*ctypes.ResultNetInfo, error) {
	result := new(ctypes.ResultNetInfo)
	_, err := c.caller.Call(ctx, "net_info", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) DumpConsensusState(ctx context.Context) (*ctypes.ResultDumpConsensusState, error) {
	result := new(ctypes.ResultDumpConsensusState)
	_, err := c.caller.Call(ctx, "dump_consensus_state", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) ConsensusState(ctx context.Context) (*ctypes.ResultConsensusState, error) {
	result := new(ctypes.ResultConsensusState)
	_, err := c.caller.Call(ctx, "consensus_state", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) ConsensusParams(
	ctx context.Context,
	height *int64,
) (*ctypes.ResultConsensusParams, error) {
	result := new(ctypes.ResultConsensusParams)
	params := make(map[string]interface{})
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "consensus_params", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Health(ctx context.Context) (*ctypes.ResultHealth, error) {
	result := new(ctypes.ResultHealth)
	_, err := c.caller.Call(ctx, "health", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) BlockchainInfo(
	ctx context.Context,
	minHeight,
	maxHeight int64,
) (*ctypes.ResultBlockchainInfo, error) {
	result := new(ctypes.ResultBlockchainInfo)
	_, err := c.caller.Call(ctx, "blockchain",
		map[string]interface{}{"minHeight": minHeight, "maxHeight": maxHeight},
		result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Genesis(ctx context.Context) (*ctypes.ResultGenesis, error) {
	result := new(ctypes.ResultGenesis)
	_, err := c.caller.Call(ctx, "genesis", map[string]interface{}{}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) GenesisChunked(ctx context.Context, id uint) (*ctypes.ResultGenesisChunk, error) {
	result := new(ctypes.ResultGenesisChunk)
	_, err := c.caller.Call(ctx, "genesis_chunked", map[string]interface{}{"chunk": id}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Block(ctx context.Context, height *int64) (*ctypes.ResultBlock, error) {
	result := new(ctypes.ResultBlock)
	params := make(map[string]interface{})
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "block", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) BlockByHash(ctx context.Context, hash []byte) (*ctypes.ResultBlock, error) {
	result := new(ctypes.ResultBlock)
	params := map[string]interface{}{
		"hash": hash,
	}
	_, err := c.caller.Call(ctx, "block_by_hash", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) BlockResults(
	ctx context.Context,
	height *int64,
) (*ctypes.ResultBlockResults, error) {
	result := new(ctypes.ResultBlockResults)
	params := make(map[string]interface{})
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "block_results", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Header(ctx context.Context, height *int64) (*ctypes.ResultHeader, error) {
	result := new(ctypes.ResultHeader)
	params := make(map[string]interface{})
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "header", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) HeaderByHash(ctx context.Context, hash bytes.HexBytes) (*ctypes.ResultHeader, error) {
	result := new(ctypes.ResultHeader)
	params := map[string]interface{}{
		"hash": hash,
	}
	_, err := c.caller.Call(ctx, "header_by_hash", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Commit(ctx context.Context, height *int64) (*ctypes.ResultCommit, error) {
	result := new(ctypes.ResultCommit)
	params := make(map[string]interface{})
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "commit", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) Tx(ctx context.Context, hash []byte, prove bool) (*ctypes.ResultTx, error) {
	result := new(ctypes.ResultTx)
	params := map[string]interface{}{
		"hash":  hash,
		"prove": prove,
	}
	_, err := c.caller.Call(ctx, "tx", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) TxSearch(
	ctx context.Context,
	query string,
	prove bool,
	page,
	perPage *int,
	orderBy string,
) (*ctypes.ResultTxSearch, error) {
	result := new(ctypes.ResultTxSearch)
	params := map[string]interface{}{
		"query":    query,
		"prove":    prove,
		"order_by": orderBy,
	}

	if page != nil {
		params["page"] = page
	}
	if perPage != nil {
		params["per_page"] = perPage
	}

	_, err := c.caller.Call(ctx, "tx_search", params, result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (c *baseRPCClient) BlockSearch(
	ctx context.Context,
	query string,
	page, perPage *int,
	orderBy string,
) (*ctypes.ResultBlockSearch, error) {
	result := new(ctypes.ResultBlockSearch)
	params := map[string]interface{}{
		"query":    query,
		"order_by": orderBy,
	}

	if page != nil {
		params["page"] = page
	}
	if perPage != nil {
		params["per_page"] = perPage
	}

	_, err := c.caller.Call(ctx, "block_search", params, result)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (c *baseRPCClient) Validators(
	ctx context.Context,
	height *int64,
	page,
	perPage *int,
) (*ctypes.ResultValidators, error) {
	result := new(ctypes.ResultValidators)
	params := make(map[string]interface{})
	if page != nil {
		params["page"] = page
	}
	if perPage != nil {
		params["per_page"] = perPage
	}
	if height != nil {
		params["height"] = height
	}
	_, err := c.caller.Call(ctx, "validators", params, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

func (c *baseRPCClient) BroadcastEvidence(
	ctx context.Context,
	ev types.Evidence,
) (*ctypes.ResultBroadcastEvidence, error) {
	result := new(ctypes.ResultBroadcastEvidence)
	_, err := c.caller.Call(ctx, "broadcast_evidence", map[string]interface{}{"evidence": ev}, result)
	if err != nil {
		return nil, err
	}
	return result, nil
}

//-----------------------------------------------------------------------------
// WSEvents

var errNotRunning = errors.New("client is not running. Use .Start() method to start")

// WSEvents is a wrapper around WSClient, which implements EventsClient.
type WSEvents struct {
	service.BaseService
	remote   string
	endpoint string
	ws       *jsonrpcclient.WSClient

	mtx           cmtsync.RWMutex
	subscriptions map[string]chan ctypes.ResultEvent // query -> chan
}

func newWSEvents(remote, endpoint string) (*WSEvents, error) {
	w := &WSEvents{
		endpoint:      endpoint,
		remote:        remote,
		subscriptions: make(map[string]chan ctypes.ResultEvent),
	}
	w.BaseService = *service.NewBaseService(nil, "WSEvents", w)

	var err error
	w.ws, err = jsonrpcclient.NewWS(w.remote, w.endpoint, jsonrpcclient.OnReconnect(func() {
		// resubscribe immediately
		w.redoSubscriptionsAfter(0 * time.Second)
	}))
	if err != nil {
		return nil, err
	}
	w.ws.SetLogger(w.Logger)

	return w, nil
}

// OnStart implements service.Service by starting WSClient and event loop.
func (w *WSEvents) OnStart() error {
	if err := w.ws.Start(); err != nil {
		return err
	}

	go w.eventListener()

	return nil
}

// OnStop implements service.Service by stopping WSClient.
func (w *WSEvents) OnStop() {
	if err := w.ws.Stop(); err != nil {
		w.Logger.Error("Can't stop ws client", "err", err)
	}
}

// Subscribe implements EventsClient by using WSClient to subscribe given
// subscriber to query. By default, returns a channel with cap=1. Error is
// returned if it fails to subscribe.
//
// Channel is never closed to prevent clients from seeing an erroneous event.
//
// It returns an error if WSEvents is not running.
func (w *WSEvents) Subscribe(ctx context.Context, _, query string,
	outCapacity ...int,
) (out <-chan ctypes.ResultEvent, err error) {
	if !w.IsRunning() {
		return nil, errNotRunning
	}

	if err := w.ws.Subscribe(ctx, query); err != nil {
		return nil, err
	}

	outCap := 1
	if len(outCapacity) > 0 {
		outCap = outCapacity[0]
	}

	outc := make(chan ctypes.ResultEvent, outCap)
	w.mtx.Lock()
	// subscriber param is ignored because CometBFT will override it with
	// remote IP anyway.
	w.subscriptions[query] = outc
	w.mtx.Unlock()

	return outc, nil
}

// Unsubscribe implements EventsClient by using WSClient to unsubscribe given
// subscriber from query.
//
// It returns an error if WSEvents is not running.
func (w *WSEvents) Unsubscribe(ctx context.Context, _, query string) error {
	if !w.IsRunning() {
		return errNotRunning
	}

	if err := w.ws.Unsubscribe(ctx, query); err != nil {
		return err
	}

	w.mtx.Lock()
	_, ok := w.subscriptions[query]
	if ok {
		delete(w.subscriptions, query)
	}
	w.mtx.Unlock()

	return nil
}

// UnsubscribeAll implements EventsClient by using WSClient to unsubscribe
// given subscriber from all the queries.
//
// It returns an error if WSEvents is not running.
func (w *WSEvents) UnsubscribeAll(ctx context.Context, _ string) error {
	if !w.IsRunning() {
		return errNotRunning
	}

	if err := w.ws.UnsubscribeAll(ctx); err != nil {
		return err
	}

	w.mtx.Lock()
	w.subscriptions = make(map[string]chan ctypes.ResultEvent)
	w.mtx.Unlock()

	return nil
}

// After being reconnected, it is necessary to redo subscription to server
// otherwise no data will be automatically received.
func (w *WSEvents) redoSubscriptionsAfter(d time.Duration) {
	time.Sleep(d)

	w.mtx.RLock()
	defer w.mtx.RUnlock()
	for q := range w.subscriptions {
		err := w.ws.Subscribe(context.Background(), q)
		if err != nil {
			w.Logger.Error("Failed to resubscribe", "err", err)
		}
	}
}

func isErrAlreadySubscribed(err error) bool {
	return strings.Contains(err.Error(), cmtpubsub.ErrAlreadySubscribed.Error())
}

func (w *WSEvents) eventListener() {
	for {
		select {
		case resp, ok := <-w.ws.ResponsesCh:
			if !ok {
				return
			}

			if resp.Error != nil {
				w.Logger.Error("WS error", "err", resp.Error.Error())
				// Error can be ErrAlreadySubscribed or max client (subscriptions per
				// client) reached or CometBFT exited.
				// We can ignore ErrAlreadySubscribed, but need to retry in other
				// cases.
				if !isErrAlreadySubscribed(resp.Error) {
					// Resubscribe after 1 second to give CometBFT time to restart (if
					// crashed).
					w.redoSubscriptionsAfter(1 * time.Second)
				}
				continue
			}

			result := new(ctypes.ResultEvent)
			err := cmtjson.Unmarshal(resp.Result, result)
			if err != nil {
				w.Logger.Error("failed to unmarshal response", "err", err)
				continue
			}

			w.mtx.RLock()
			if out, ok := w.subscriptions[result.Query]; ok {
				if cap(out) == 0 {
					out <- *result
				} else {
					select {
					case out <- *result:
					default:
						w.Logger.Error("wanted to publish ResultEvent, but out channel is full", "result", result, "query", result.Query)
					}
				}
			}
			w.mtx.RUnlock()
		case <-w.Quit():
			return
		}
	}
}
