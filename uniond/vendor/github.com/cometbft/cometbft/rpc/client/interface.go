package client

/*
The client package provides a general purpose interface (Client) for connecting
to a CometBFT node, as well as higher-level functionality.

The main implementation for production code is client.HTTP, which
connects via http to the jsonrpc interface of the CometBFT node.

For connecting to a node running in the same process (eg. when
compiling the abci app in the same process), you can use the client.Local
implementation.

For mocking out server responses during testing to see behavior for
arbitrary return values, use the mock package.

In addition to the Client interface, which should be used externally
for maximum flexibility and testability, and two implementations,
this package also provides helper functions that work on any Client
implementation.
*/

import (
	"context"

	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/service"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	"github.com/cometbft/cometbft/types"
)

// Client wraps most important rpc calls a client would make if you want to
// listen for events, test if it also implements events.EventSwitch.
type Client interface {
	service.Service
	ABCIClient
	EventsClient
	HistoryClient
	NetworkClient
	SignClient
	StatusClient
	EvidenceClient
	MempoolClient
}

// ABCIClient groups together the functionality that principally affects the
// ABCI app.
//
// In many cases this will be all we want, so we can accept an interface which
// is easier to mock.
type ABCIClient interface {
	// Reading from abci app
	ABCIInfo(context.Context) (*ctypes.ResultABCIInfo, error)
	ABCIQuery(ctx context.Context, path string, data bytes.HexBytes) (*ctypes.ResultABCIQuery, error)
	ABCIQueryWithOptions(ctx context.Context, path string, data bytes.HexBytes,
		opts ABCIQueryOptions) (*ctypes.ResultABCIQuery, error)

	// Writing to abci app
	BroadcastTxCommit(context.Context, types.Tx) (*ctypes.ResultBroadcastTxCommit, error)
	BroadcastTxAsync(context.Context, types.Tx) (*ctypes.ResultBroadcastTx, error)
	BroadcastTxSync(context.Context, types.Tx) (*ctypes.ResultBroadcastTx, error)
}

// SignClient groups together the functionality needed to get valid signatures
// and prove anything about the chain.
type SignClient interface {
	Block(ctx context.Context, height *int64) (*ctypes.ResultBlock, error)
	BlockByHash(ctx context.Context, hash []byte) (*ctypes.ResultBlock, error)
	BlockResults(ctx context.Context, height *int64) (*ctypes.ResultBlockResults, error)
	Header(ctx context.Context, height *int64) (*ctypes.ResultHeader, error)
	HeaderByHash(ctx context.Context, hash bytes.HexBytes) (*ctypes.ResultHeader, error)
	Commit(ctx context.Context, height *int64) (*ctypes.ResultCommit, error)
	Validators(ctx context.Context, height *int64, page, perPage *int) (*ctypes.ResultValidators, error)
	Tx(ctx context.Context, hash []byte, prove bool) (*ctypes.ResultTx, error)

	// TxSearch defines a method to search for a paginated set of transactions by
	// transaction event search criteria.
	TxSearch(
		ctx context.Context,
		query string,
		prove bool,
		page, perPage *int,
		orderBy string,
	) (*ctypes.ResultTxSearch, error)

	// BlockSearch defines a method to search for a paginated set of blocks based
	// from FinalizeBlock event search criteria.
	BlockSearch(
		ctx context.Context,
		query string,
		page, perPage *int,
		orderBy string,
	) (*ctypes.ResultBlockSearch, error)
}

// HistoryClient provides access to data from genesis to now in large chunks.
type HistoryClient interface {
	Genesis(context.Context) (*ctypes.ResultGenesis, error)
	GenesisChunked(context.Context, uint) (*ctypes.ResultGenesisChunk, error)
	BlockchainInfo(ctx context.Context, minHeight, maxHeight int64) (*ctypes.ResultBlockchainInfo, error)
}

// StatusClient provides access to general chain info.
type StatusClient interface {
	Status(context.Context) (*ctypes.ResultStatus, error)
}

// NetworkClient is general info about the network state. May not be needed
// usually.
type NetworkClient interface {
	NetInfo(context.Context) (*ctypes.ResultNetInfo, error)
	DumpConsensusState(context.Context) (*ctypes.ResultDumpConsensusState, error)
	ConsensusState(context.Context) (*ctypes.ResultConsensusState, error)
	ConsensusParams(ctx context.Context, height *int64) (*ctypes.ResultConsensusParams, error)
	Health(context.Context) (*ctypes.ResultHealth, error)
}

// EventsClient is reactive, you can subscribe to any message, given the proper
// string. see cometbft/types/events.go
type EventsClient interface {
	// Subscribe subscribes given subscriber to query. Returns a channel with
	// cap=1 onto which events are published. An error is returned if it fails to
	// subscribe. outCapacity can be used optionally to set capacity for the
	// channel. Channel is never closed to prevent accidental reads.
	//
	// ctx cannot be used to unsubscribe. To unsubscribe, use either Unsubscribe
	// or UnsubscribeAll.
	Subscribe(ctx context.Context, subscriber, query string, outCapacity ...int) (out <-chan ctypes.ResultEvent, err error)
	// Unsubscribe unsubscribes given subscriber from query.
	Unsubscribe(ctx context.Context, subscriber, query string) error
	// UnsubscribeAll unsubscribes given subscriber from all the queries.
	UnsubscribeAll(ctx context.Context, subscriber string) error
}

// MempoolClient shows us data about current mempool state.
type MempoolClient interface {
	UnconfirmedTxs(ctx context.Context, limit *int) (*ctypes.ResultUnconfirmedTxs, error)
	NumUnconfirmedTxs(context.Context) (*ctypes.ResultUnconfirmedTxs, error)
	CheckTx(context.Context, types.Tx) (*ctypes.ResultCheckTx, error)
}

// EvidenceClient is used for submitting an evidence of the malicious
// behavior.
type EvidenceClient interface {
	BroadcastEvidence(context.Context, types.Evidence) (*ctypes.ResultBroadcastEvidence, error)
}

// RemoteClient is a Client, which can also return the remote network address.
type RemoteClient interface {
	Client

	// Remote returns the remote network address in a string form.
	Remote() string
}
