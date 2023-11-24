package keeper

import (
	"context"
	"fmt"
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
)

var _ types.QueryServer = (*Keeper)(nil)

// DenomTrace implements the Query/DenomTrace gRPC method
func (k Keeper) DenomTrace(c context.Context, req *types.QueryDenomTraceRequest) (*types.QueryDenomTraceResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	hash, err := types.ParseHexHash(strings.TrimPrefix(req.Hash, "ibc/"))
	if err != nil {
		return nil, status.Error(codes.InvalidArgument, fmt.Sprintf("invalid denom trace hash: %s, error: %s", hash.String(), err))
	}

	ctx := sdk.UnwrapSDKContext(c)
	denomTrace, found := k.GetDenomTrace(ctx, hash)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrTraceNotFound, req.Hash).Error(),
		)
	}

	return &types.QueryDenomTraceResponse{
		DenomTrace: &denomTrace,
	}, nil
}

// DenomTraces implements the Query/DenomTraces gRPC method
func (k Keeper) DenomTraces(c context.Context, req *types.QueryDenomTracesRequest) (*types.QueryDenomTracesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	var traces types.Traces
	store := prefix.NewStore(ctx.KVStore(k.storeKey), types.DenomTraceKey)

	pageRes, err := query.Paginate(store, req.Pagination, func(_, value []byte) error {
		result, err := k.UnmarshalDenomTrace(value)
		if err != nil {
			return err
		}

		traces = append(traces, result)
		return nil
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryDenomTracesResponse{
		DenomTraces: traces.Sort(),
		Pagination:  pageRes,
	}, nil
}

// Params implements the Query/Params gRPC method
func (k Keeper) Params(c context.Context, _ *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	ctx := sdk.UnwrapSDKContext(c)
	params := k.GetParams(ctx)

	return &types.QueryParamsResponse{
		Params: &params,
	}, nil
}

// DenomHash implements the Query/DenomHash gRPC method
func (k Keeper) DenomHash(c context.Context, req *types.QueryDenomHashRequest) (*types.QueryDenomHashResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	// Convert given request trace path to DenomTrace struct to confirm the path in a valid denom trace format
	denomTrace := types.ParseDenomTrace(req.Trace)
	if err := denomTrace.Validate(); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	ctx := sdk.UnwrapSDKContext(c)
	denomHash := denomTrace.Hash()
	found := k.HasDenomTrace(ctx, denomHash)
	if !found {
		return nil, status.Error(
			codes.NotFound,
			errorsmod.Wrap(types.ErrTraceNotFound, req.Trace).Error(),
		)
	}

	return &types.QueryDenomHashResponse{
		Hash: denomHash.String(),
	}, nil
}

// EscrowAddress implements the EscrowAddress gRPC method
func (Keeper) EscrowAddress(c context.Context, req *types.QueryEscrowAddressRequest) (*types.QueryEscrowAddressResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	addr := types.GetEscrowAddress(req.PortId, req.ChannelId)

	return &types.QueryEscrowAddressResponse{
		EscrowAddress: addr.String(),
	}, nil
}

// TotalEscrowForDenom implements the TotalEscrowForDenom gRPC method.
func (k Keeper) TotalEscrowForDenom(c context.Context, req *types.QueryTotalEscrowForDenomRequest) (*types.QueryTotalEscrowForDenomResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	amount := k.GetTotalEscrowForDenom(ctx, req.Denom)

	return &types.QueryTotalEscrowForDenomResponse{
		Amount: amount,
	}, nil
}
