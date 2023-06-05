package keeper

import (
	"context"
	"encoding/hex"
	"fmt"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	sdkquery "github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/ibc-go/v7/modules/light-clients/08-wasm/types"
)

var _ types.QueryServer = (*Keeper)(nil)

// Code implements the Query/CodeId gRPC method
func (k Keeper) Code(c context.Context, req *types.QueryCodeRequest) (*types.QueryCodeResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	ctx := sdk.UnwrapSDKContext(c)
	store := ctx.KVStore(k.storeKey)

	codeID, err := hex.DecodeString(req.CodeId)
	if err != nil {
		return nil, status.Error(codes.InvalidArgument, "invalid code ID")
	}

	codeKey := types.CodeIDKey(codeID)
	code := store.Get(codeKey)
	if code == nil {
		return nil, status.Error(codes.NotFound, sdkerrors.Wrap(types.ErrWasmCodeIDNotFound, req.CodeId).Error())
	}

	return &types.QueryCodeResponse{
		Code: code,
	}, nil
}

// CodeIds implements the Query/CodeIds gRPC method
func (k Keeper) CodeIds(c context.Context, req *types.QueryCodeIdsRequest) (*types.QueryCodeIdsResponse, error) {
	var codeIDs []string

	ctx := sdk.UnwrapSDKContext(c)
	store := ctx.KVStore(k.storeKey)
	prefixStore := prefix.NewStore(store, []byte(fmt.Sprintf("%s/", types.KeyCodeIDPrefix)))

	pageRes, err := sdkquery.FilteredPaginate(prefixStore, req.Pagination, func(key []byte, _ []byte, accumulate bool) (bool, error) {
		if accumulate {
			codeIDs = append(codeIDs, string(key))
		}
		return true, nil
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryCodeIdsResponse{
		CodeIds:    codeIDs,
		Pagination: pageRes,
	}, nil
}
