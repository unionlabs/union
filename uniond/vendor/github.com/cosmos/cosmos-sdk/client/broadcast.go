package client

import (
	"context"
	"fmt"
	"strings"

	"github.com/cometbft/cometbft/mempool"
	cmttypes "github.com/cometbft/cometbft/types"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"github.com/cosmos/cosmos-sdk/client/flags"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
)

// BroadcastTx broadcasts a transactions either synchronously or asynchronously
// based on the context parameters. The result of the broadcast is parsed into
// an intermediate structure which is logged if the context has a logger
// defined.
func (ctx Context) BroadcastTx(txBytes []byte) (res *sdk.TxResponse, err error) {
	switch ctx.BroadcastMode {
	case flags.BroadcastSync:
		res, err = ctx.BroadcastTxSync(txBytes)

	case flags.BroadcastAsync:
		res, err = ctx.BroadcastTxAsync(txBytes)

	default:
		return nil, fmt.Errorf("unsupported return type %s; supported types: sync, async", ctx.BroadcastMode)
	}

	return res, err
}

// CheckCometError checks if the error returned from BroadcastTx is a
// CometBFT error that is returned before the tx is submitted due to
// precondition checks that failed. If a CometBFT error is detected, this
// function returns the correct code back in TxResponse.
//
// TODO: Avoid brittle string matching in favor of error matching. This requires
// a change to CometBFT's RPCError type to allow retrieval or matching against
// a concrete error type.
func CheckCometError(err error, tx cmttypes.Tx) *sdk.TxResponse {
	if err == nil {
		return nil
	}

	errStr := strings.ToLower(err.Error())
	txHash := fmt.Sprintf("%X", tx.Hash())

	switch {
	case strings.Contains(errStr, strings.ToLower(mempool.ErrTxInCache.Error())):
		return &sdk.TxResponse{
			Code:      sdkerrors.ErrTxInMempoolCache.ABCICode(),
			Codespace: sdkerrors.ErrTxInMempoolCache.Codespace(),
			TxHash:    txHash,
		}

	case strings.Contains(errStr, "mempool is full"):
		return &sdk.TxResponse{
			Code:      sdkerrors.ErrMempoolIsFull.ABCICode(),
			Codespace: sdkerrors.ErrMempoolIsFull.Codespace(),
			TxHash:    txHash,
		}

	case strings.Contains(errStr, "tx too large"):
		return &sdk.TxResponse{
			Code:      sdkerrors.ErrTxTooLarge.ABCICode(),
			Codespace: sdkerrors.ErrTxTooLarge.Codespace(),
			TxHash:    txHash,
		}

	default:
		return nil
	}
}

// BroadcastTxSync broadcasts transaction bytes to a CometBFT node
// synchronously (i.e. returns after CheckTx execution).
func (ctx Context) BroadcastTxSync(txBytes []byte) (*sdk.TxResponse, error) {
	node, err := ctx.GetNode()
	if err != nil {
		return nil, err
	}

	res, err := node.BroadcastTxSync(context.Background(), txBytes)
	if errRes := CheckCometError(err, txBytes); errRes != nil {
		return errRes, nil
	}

	return sdk.NewResponseFormatBroadcastTx(res), err
}

// BroadcastTxAsync broadcasts transaction bytes to a CometBFT node
// asynchronously (i.e. returns immediately).
func (ctx Context) BroadcastTxAsync(txBytes []byte) (*sdk.TxResponse, error) {
	node, err := ctx.GetNode()
	if err != nil {
		return nil, err
	}

	res, err := node.BroadcastTxAsync(context.Background(), txBytes)
	if errRes := CheckCometError(err, txBytes); errRes != nil {
		return errRes, nil
	}

	return sdk.NewResponseFormatBroadcastTx(res), err
}

// TxServiceBroadcast is a helper function to broadcast a Tx with the correct gRPC types
// from the tx service. Calls `clientCtx.BroadcastTx` under the hood.
func TxServiceBroadcast(_ context.Context, clientCtx Context, req *tx.BroadcastTxRequest) (*tx.BroadcastTxResponse, error) {
	if req == nil || req.TxBytes == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid empty tx")
	}

	clientCtx = clientCtx.WithBroadcastMode(normalizeBroadcastMode(req.Mode))
	resp, err := clientCtx.BroadcastTx(req.TxBytes)
	if err != nil {
		return nil, err
	}

	return &tx.BroadcastTxResponse{
		TxResponse: resp,
	}, nil
}

// normalizeBroadcastMode converts a broadcast mode into a normalized string
// to be passed into the clientCtx.
func normalizeBroadcastMode(mode tx.BroadcastMode) string {
	switch mode {
	case tx.BroadcastMode_BROADCAST_MODE_ASYNC:
		return "async"
	case tx.BroadcastMode_BROADCAST_MODE_SYNC:
		return "sync"
	default:
		return "unspecified"
	}
}
