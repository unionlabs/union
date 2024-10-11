package mock

/*
package mock returns a Client implementation that
accepts various (mock) implementations of the various methods.

This implementation is useful for using in tests, when you don't
need a real server, but want a high-level of control about
the server response you want to mock (eg. error handling),
or if you just want to record the calls to verify in your tests.

For real clients, you probably want the "http" package.  If you
want to directly call a CometBFT node in process, you can use the
"local" package.
*/

import (
	"context"
	"reflect"

	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/service"
	"github.com/cometbft/cometbft/rpc/client"
	"github.com/cometbft/cometbft/rpc/core"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	rpctypes "github.com/cometbft/cometbft/rpc/jsonrpc/types"
	"github.com/cometbft/cometbft/types"
)

// Client wraps arbitrary implementations of the various interfaces.
type Client struct {
	client.ABCIClient
	client.SignClient
	client.HistoryClient
	client.StatusClient
	client.EventsClient
	client.EvidenceClient
	client.MempoolClient
	service.Service

	env *core.Environment
}

func New() Client {
	return Client{
		env: &core.Environment{},
	}
}

var _ client.Client = Client{}

// Call is used by recorders to save a call and response.
// It can also be used to configure mock responses.
type Call struct {
	Name     string
	Args     any
	Response any
	Error    error
}

// GetResponse will generate the appropriate response for us, when
// using the Call struct to configure a Mock handler.
//
// When configuring a response, if only one of Response or Error is
// set then that will always be returned. If both are set, then
// we return Response if the Args match the set args, Error otherwise.
func (c Call) GetResponse(args any) (any, error) {
	// handle the case with no response
	if c.Response == nil {
		if c.Error == nil {
			panic("Misconfigured call, you must set either Response or Error")
		}
		return nil, c.Error
	}
	// response without error
	if c.Error == nil {
		return c.Response, nil
	}
	// have both, we must check args....
	if reflect.DeepEqual(args, c.Args) {
		return c.Response, nil
	}
	return nil, c.Error
}

func (c Client) Status(context.Context) (*ctypes.ResultStatus, error) {
	return c.env.Status(&rpctypes.Context{})
}

func (c Client) ABCIInfo(context.Context) (*ctypes.ResultABCIInfo, error) {
	return c.env.ABCIInfo(&rpctypes.Context{})
}

func (c Client) ABCIQuery(ctx context.Context, path string, data bytes.HexBytes) (*ctypes.ResultABCIQuery, error) {
	return c.ABCIQueryWithOptions(ctx, path, data, client.DefaultABCIQueryOptions)
}

func (c Client) ABCIQueryWithOptions(
	_ context.Context,
	path string,
	data bytes.HexBytes,
	opts client.ABCIQueryOptions,
) (*ctypes.ResultABCIQuery, error) {
	return c.env.ABCIQuery(&rpctypes.Context{}, path, data, opts.Height, opts.Prove)
}

func (c Client) BroadcastTxCommit(_ context.Context, tx types.Tx) (*ctypes.ResultBroadcastTxCommit, error) {
	return c.env.BroadcastTxCommit(&rpctypes.Context{}, tx)
}

func (c Client) BroadcastTxAsync(_ context.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	return c.env.BroadcastTxAsync(&rpctypes.Context{}, tx)
}

func (c Client) BroadcastTxSync(_ context.Context, tx types.Tx) (*ctypes.ResultBroadcastTx, error) {
	return c.env.BroadcastTxSync(&rpctypes.Context{}, tx)
}

func (c Client) CheckTx(_ context.Context, tx types.Tx) (*ctypes.ResultCheckTx, error) {
	return c.env.CheckTx(&rpctypes.Context{}, tx)
}

func (c Client) NetInfo(_ context.Context) (*ctypes.ResultNetInfo, error) {
	return c.env.NetInfo(&rpctypes.Context{})
}

func (c Client) ConsensusState(_ context.Context) (*ctypes.ResultConsensusState, error) {
	return c.env.GetConsensusState(&rpctypes.Context{})
}

func (c Client) DumpConsensusState(_ context.Context) (*ctypes.ResultDumpConsensusState, error) {
	return c.env.DumpConsensusState(&rpctypes.Context{})
}

func (c Client) ConsensusParams(_ context.Context, height *int64) (*ctypes.ResultConsensusParams, error) {
	return c.env.ConsensusParams(&rpctypes.Context{}, height)
}

func (c Client) Health(_ context.Context) (*ctypes.ResultHealth, error) {
	return c.env.Health(&rpctypes.Context{})
}

func (c Client) DialSeeds(_ context.Context, seeds []string) (*ctypes.ResultDialSeeds, error) {
	return c.env.UnsafeDialSeeds(&rpctypes.Context{}, seeds)
}

func (c Client) DialPeers(
	_ context.Context,
	peers []string,
	persistent,
	unconditional,
	private bool,
) (*ctypes.ResultDialPeers, error) {
	return c.env.UnsafeDialPeers(&rpctypes.Context{}, peers, persistent, unconditional, private)
}

func (c Client) BlockchainInfo(_ context.Context, minHeight, maxHeight int64) (*ctypes.ResultBlockchainInfo, error) {
	return c.env.BlockchainInfo(&rpctypes.Context{}, minHeight, maxHeight)
}

func (c Client) Genesis(context.Context) (*ctypes.ResultGenesis, error) {
	return c.env.Genesis(&rpctypes.Context{})
}

func (c Client) Block(_ context.Context, height *int64) (*ctypes.ResultBlock, error) {
	return c.env.Block(&rpctypes.Context{}, height)
}

func (c Client) BlockByHash(_ context.Context, hash []byte) (*ctypes.ResultBlock, error) {
	return c.env.BlockByHash(&rpctypes.Context{}, hash)
}

func (c Client) Commit(_ context.Context, height *int64) (*ctypes.ResultCommit, error) {
	return c.env.Commit(&rpctypes.Context{}, height)
}

func (c Client) Validators(_ context.Context, height *int64, page, perPage *int) (*ctypes.ResultValidators, error) {
	return c.env.Validators(&rpctypes.Context{}, height, page, perPage)
}

func (c Client) BroadcastEvidence(_ context.Context, ev types.Evidence) (*ctypes.ResultBroadcastEvidence, error) {
	return c.env.BroadcastEvidence(&rpctypes.Context{}, ev)
}
