package client

import (
	"context"
	"fmt"
	"strings"

	"github.com/cockroachdb/errors"
	abci "github.com/cometbft/cometbft/abci/types"
	rpcclient "github.com/cometbft/cometbft/rpc/client"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/store/rootmulti"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// GetNode returns an RPC client. If the context's client is not defined, an
// error is returned.
func (ctx Context) GetNode() (CometRPC, error) {
	if ctx.Client == nil {
		return nil, errors.New("no RPC client is defined in offline mode")
	}

	return ctx.Client, nil
}

// Query performs a query to a CometBFT node with the provided path.
// It returns the result and height of the query upon success or an error if
// the query fails.
func (ctx Context) Query(path string) ([]byte, int64, error) {
	return ctx.query(path, nil)
}

// QueryWithData performs a query to a CometBFT node with the provided path
// and a data payload. It returns the result and height of the query upon success
// or an error if the query fails.
func (ctx Context) QueryWithData(path string, data []byte) ([]byte, int64, error) {
	return ctx.query(path, data)
}

// QueryStore performs a query to a CometBFT node with the provided key and
// store name. It returns the result and height of the query upon success
// or an error if the query fails.
func (ctx Context) QueryStore(key []byte, storeName string) ([]byte, int64, error) {
	return ctx.queryStore(key, storeName, "key")
}

// QueryABCI performs a query to a CometBFT node with the provide RequestQuery.
// It returns the ResultQuery obtained from the query. The height used to perform
// the query is the RequestQuery Height if it is non-zero, otherwise the context
// height is used.
func (ctx Context) QueryABCI(req abci.RequestQuery) (abci.ResponseQuery, error) {
	return ctx.queryABCI(req)
}

// GetFromAddress returns the from address from the context's name.
func (ctx Context) GetFromAddress() sdk.AccAddress {
	return ctx.FromAddress
}

// GetFeePayerAddress returns the fee granter address from the context
func (ctx Context) GetFeePayerAddress() sdk.AccAddress {
	return ctx.FeePayer
}

// GetFeeGranterAddress returns the fee granter address from the context
func (ctx Context) GetFeeGranterAddress() sdk.AccAddress {
	return ctx.FeeGranter
}

// GetFromName returns the key name for the current context.
func (ctx Context) GetFromName() string {
	return ctx.FromName
}

func (ctx Context) queryABCI(req abci.RequestQuery) (abci.ResponseQuery, error) {
	node, err := ctx.GetNode()
	if err != nil {
		return abci.ResponseQuery{}, err
	}

	var queryHeight int64
	if req.Height != 0 {
		queryHeight = req.Height
	} else {
		// fallback on the context height
		queryHeight = ctx.Height
	}

	opts := rpcclient.ABCIQueryOptions{
		Height: queryHeight,
		Prove:  req.Prove,
	}

	result, err := node.ABCIQueryWithOptions(context.Background(), req.Path, req.Data, opts)
	if err != nil {
		return abci.ResponseQuery{}, err
	}

	if !result.Response.IsOK() {
		return abci.ResponseQuery{}, sdkErrorToGRPCError(result.Response)
	}

	// data from trusted node or subspace query doesn't need verification
	if !opts.Prove || !isQueryStoreWithProof(req.Path) {
		return result.Response, nil
	}

	return result.Response, nil
}

func sdkErrorToGRPCError(resp abci.ResponseQuery) error {
	switch resp.Code {
	case sdkerrors.ErrInvalidRequest.ABCICode():
		return status.Error(codes.InvalidArgument, resp.Log)
	case sdkerrors.ErrUnauthorized.ABCICode():
		return status.Error(codes.Unauthenticated, resp.Log)
	case sdkerrors.ErrKeyNotFound.ABCICode():
		return status.Error(codes.NotFound, resp.Log)
	default:
		return status.Error(codes.Unknown, resp.Log)
	}
}

// query performs a query to a CometBFT node with the provided store name
// and path. It returns the result and height of the query upon success
// or an error if the query fails.
func (ctx Context) query(path string, key []byte) ([]byte, int64, error) {
	resp, err := ctx.queryABCI(abci.RequestQuery{
		Path:   path,
		Data:   key,
		Height: ctx.Height,
	})
	if err != nil {
		return nil, 0, err
	}

	return resp.Value, resp.Height, nil
}

// queryStore performs a query to a CometBFT node with the provided a store
// name and path. It returns the result and height of the query upon success
// or an error if the query fails.
func (ctx Context) queryStore(key []byte, storeName, endPath string) ([]byte, int64, error) {
	path := fmt.Sprintf("/store/%s/%s", storeName, endPath)
	return ctx.query(path, key)
}

// isQueryStoreWithProof expects a format like /<queryType>/<storeName>/<subpath>
// queryType must be "store" and subpath must be "key" to require a proof.
func isQueryStoreWithProof(path string) bool {
	if !strings.HasPrefix(path, "/") {
		return false
	}

	paths := strings.SplitN(path[1:], "/", 3)

	switch {
	case len(paths) != 3:
		return false
	case paths[0] != "store":
		return false
	case rootmulti.RequireProof("/" + paths[2]):
		return true
	}

	return false
}
