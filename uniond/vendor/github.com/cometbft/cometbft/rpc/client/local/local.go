package local

import (
	"context"
	"fmt"
	"time"

	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	cmtquery "github.com/cometbft/cometbft/libs/pubsub/query"
	nm "github.com/cometbft/cometbft/node"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	"github.com/cometbft/cometbft/rpc/core"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
	"github.com/cometbft/cometbft/types"
)

/*
Local is a Client implementation that directly executes the rpc
functions on a given node, without going through HTTP or GRPC.

This implementation is useful for:

* Running tests against a node in-process without the overhead
of going through an http server
* Communication between an ABCI app and CometBFT when they
are compiled in process.

For real clients, you probably want to use client.HTTP.  For more
powerful control during testing, you probably want the "client/mock" package.

You can subscribe for any event published by CometBFT using Subscribe method.
Note delivery is best-effort. If you don't read events fast enough, CometBFT
might cancel the subscription. The client will attempt to resubscribe (you
don't need to do anything). It will keep trying indefinitely with exponential
backoff (10ms -> 20ms -> 40ms) until successful.
*/
type Local struct {
	*types.EventBus
	Logger log.Logger
	ctx    *rpctypes.Context
}

// NewLocal configures a client that calls the Node directly.
//
// Note that given how rpc/core works with package singletons, that
// you can only have one node per process.  So make sure test cases
// don't run in parallel, or try to simulate an entire network in
// one process...
func New(node *nm.Node) *Local {
	if err := node.ConfigureRPC(); err != nil {
		node.Logger.Error("Error configuring RPC", "err", err)
	}
	return &Local{
		EventBus: node.EventBus(),
		Logger:   log.NewNopLogger(),
		ctx:      &rpctypes.Context{},
	}
}

var _ rpcclient.Client = (*Local)(nil)

// SetLogger allows to set a logger on the client.
func (c *Local) SetLogger(l log.Logger) {
	c.Logger = l
}

func (c *Local) Status(ctx context.Context) (*ctypes.ResultStatus, error) {
	return core.Status(c.ctx)
}

func (c *Local) ABCIInfo(ctx context.Context) (*ctypes.ResultABCIInfo, error) {
	return core.ABCIInfo(c.ctx)
}

func (c *Local) ABCIQuery(ctx context.Context, path string, data bytes.HexBytes) (*ctypes.ResultABCIQuery, error) {
	return c.ABCIQueryWithOptions(ctx, path, data, rpcclient.DefaultABCIQueryOptions)
}

func (c *Local) ABCIQueryWithOptions(
	ctx context.Context,
	path string,
	data bytes.HexBytes,
	opts rpcclient.ABCIQueryOptions) (*ctypes.ResultABCIQuery, error) {
	return core.ABCIQuery(c.ctx, path, data, opts.Height, opts.Prove)
}

func (c *Local) BroadcastTxCommit(ctx context.Context, tx types.Tx) (*ctypes.ResultBroadcastTxCommit, error) {
	return core.BroadcastTxCommit(c.ctx, tx)
}

func (c *Local) BroadcastTxAsync(ctx context.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	return core.BroadcastTxAsync(c.ctx, tx)
}

func (c *Local) BroadcastTxSync(ctx context.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	return core.BroadcastTxSync(c.ctx, tx)
}

func (c *Local) UnconfirmedTxs(ctx context.Context, limit *int) (*ctypes.ResultUnconfirmedTxs, error) {
	return core.UnconfirmedTxs(c.ctx, limit)
}

func (c *Local) NumUnconfirmedTxs(ctx context.Context) (*ctypes.ResultUnconfirmedTxs, error) {
	return core.NumUnconfirmedTxs(c.ctx)
}

func (c *Local) CheckTx(ctx context.Context, tx types.Tx) (*ctypes.ResultCheckTx, error) {
	return core.CheckTx(c.ctx, tx)
}

func (c *Local) NetInfo(ctx context.Context) (*ctypes.ResultNetInfo, error) {
	return core.NetInfo(c.ctx)
}

func (c *Local) DumpConsensusState(ctx context.Context) (*ctypes.ResultDumpConsensusState, error) {
	return core.DumpConsensusState(c.ctx)
}

func (c *Local) ConsensusState(ctx context.Context) (*ctypes.ResultConsensusState, error) {
	return core.ConsensusState(c.ctx)
}

func (c *Local) ConsensusParams(ctx context.Context, height *int64) (*ctypes.ResultConsensusParams, error) {
	return core.ConsensusParams(c.ctx, height)
}

func (c *Local) Health(ctx context.Context) (*ctypes.ResultHealth, error) {
	return core.Health(c.ctx)
}

func (c *Local) DialSeeds(ctx context.Context, seeds []string) (*ctypes.ResultDialSeeds, error) {
	return core.UnsafeDialSeeds(c.ctx, seeds)
}

func (c *Local) DialPeers(
	ctx context.Context,
	peers []string,
	persistent,
	unconditional,
	private bool,
) (*ctypes.ResultDialPeers, error) {
	return core.UnsafeDialPeers(c.ctx, peers, persistent, unconditional, private)
}

func (c *Local) BlockchainInfo(ctx context.Context, minHeight, maxHeight int64) (*ctypes.ResultBlockchainInfo, error) {
	return core.BlockchainInfo(c.ctx, minHeight, maxHeight)
}

func (c *Local) Genesis(ctx context.Context) (*ctypes.ResultGenesis, error) {
	return core.Genesis(c.ctx)
}

func (c *Local) GenesisChunked(ctx context.Context, id uint) (*ctypes.ResultGenesisChunk, error) {
	return core.GenesisChunked(c.ctx, id)
}

func (c *Local) Block(ctx context.Context, height *int64) (*ctypes.ResultBlock, error) {
	return core.Block(c.ctx, height)
}

func (c *Local) BlockByHash(ctx context.Context, hash []byte) (*ctypes.ResultBlock, error) {
	return core.BlockByHash(c.ctx, hash)
}

func (c *Local) BlockResults(ctx context.Context, height *int64) (*ctypes.ResultBlockResults, error) {
	return core.BlockResults(c.ctx, height)
}

func (c *Local) Header(ctx context.Context, height *int64) (*ctypes.ResultHeader, error) {
	return core.Header(c.ctx, height)
}

func (c *Local) HeaderByHash(ctx context.Context, hash bytes.HexBytes) (*ctypes.ResultHeader, error) {
	return core.HeaderByHash(c.ctx, hash)
}

func (c *Local) Commit(ctx context.Context, height *int64) (*ctypes.ResultCommit, error) {
	return core.Commit(c.ctx, height)
}

func (c *Local) Validators(ctx context.Context, height *int64, page, perPage *int) (*ctypes.ResultValidators, error) {
	return core.Validators(c.ctx, height, page, perPage)
}

func (c *Local) Tx(ctx context.Context, hash []byte, prove bool) (*ctypes.ResultTx, error) {
	return core.Tx(c.ctx, hash, prove)
}

func (c *Local) TxSearch(
	_ context.Context,
	query string,
	prove bool,
	page,
	perPage *int,
	orderBy string,
) (*ctypes.ResultTxSearch, error) {
	return core.TxSearch(c.ctx, query, prove, page, perPage, orderBy)
}

func (c *Local) BlockSearch(
	_ context.Context,
	query string,
	page, perPage *int,
	orderBy string,
) (*ctypes.ResultBlockSearch, error) {
	return core.BlockSearch(c.ctx, query, page, perPage, orderBy)
}

func (c *Local) BroadcastEvidence(ctx context.Context, ev types.Evidence) (*ctypes.ResultBroadcastEvidence, error) {
	return core.BroadcastEvidence(c.ctx, ev)
}

func (c *Local) Subscribe(
	ctx context.Context,
	subscriber,
	query string,
	outCapacity ...int) (out <-chan ctypes.ResultEvent, err error) {
	q, err := cmtquery.New(query)
	if err != nil {
		return nil, fmt.Errorf("failed to parse query: %w", err)
	}

	outCap := 1
	if len(outCapacity) > 0 {
		outCap = outCapacity[0]
	}

	var sub types.Subscription
	if outCap > 0 {
		sub, err = c.EventBus.Subscribe(ctx, subscriber, q, outCap)
	} else {
		sub, err = c.EventBus.SubscribeUnbuffered(ctx, subscriber, q)
	}
	if err != nil {
		return nil, fmt.Errorf("failed to subscribe: %w", err)
	}

	outc := make(chan ctypes.ResultEvent, outCap)
	go c.eventsRoutine(sub, subscriber, q, outc)

	return outc, nil
}

func (c *Local) eventsRoutine(
	sub types.Subscription,
	subscriber string,
	q cmtpubsub.Query,
	outc chan<- ctypes.ResultEvent) {
	for {
		select {
		case msg := <-sub.Out():
			result := ctypes.ResultEvent{Query: q.String(), Data: msg.Data(), Events: msg.Events()}
			if cap(outc) == 0 {
				outc <- result
			} else {
				select {
				case outc <- result:
				default:
					c.Logger.Error("wanted to publish ResultEvent, but out channel is full", "result", result, "query", result.Query)
				}
			}
		case <-sub.Cancelled():
			if sub.Err() == cmtpubsub.ErrUnsubscribed {
				return
			}

			c.Logger.Error("subscription was canceled, resubscribing...", "err", sub.Err(), "query", q.String())
			sub = c.resubscribe(subscriber, q)
			if sub == nil { // client was stopped
				return
			}
		case <-c.Quit():
			return
		}
	}
}

// Try to resubscribe with exponential backoff.
func (c *Local) resubscribe(subscriber string, q cmtpubsub.Query) types.Subscription {
	attempts := 0
	for {
		if !c.IsRunning() {
			return nil
		}

		sub, err := c.EventBus.Subscribe(context.Background(), subscriber, q)
		if err == nil {
			return sub
		}

		attempts++
		time.Sleep((10 << uint(attempts)) * time.Millisecond) // 10ms -> 20ms -> 40ms
	}
}

func (c *Local) Unsubscribe(ctx context.Context, subscriber, query string) error {
	q, err := cmtquery.New(query)
	if err != nil {
		return fmt.Errorf("failed to parse query: %w", err)
	}
	return c.EventBus.Unsubscribe(ctx, subscriber, q)
}

func (c *Local) UnsubscribeAll(ctx context.Context, subscriber string) error {
	return c.EventBus.UnsubscribeAll(ctx, subscriber)
}
