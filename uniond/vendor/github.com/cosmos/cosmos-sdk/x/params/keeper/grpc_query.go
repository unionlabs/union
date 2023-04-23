package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/params/types/proposal"
)

var _ proposal.QueryServer = Keeper{}

// Params returns subspace params
func (k Keeper) Params(c context.Context, req *proposal.QueryParamsRequest) (*proposal.QueryParamsResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.Subspace == "" || req.Key == "" {
		return nil, status.Errorf(codes.InvalidArgument, "invalid request")
	}

	ss, ok := k.GetSubspace(req.Subspace)
	if !ok {
		return nil, sdkerrors.Wrap(proposal.ErrUnknownSubspace, req.Subspace)
	}

	ctx := sdk.UnwrapSDKContext(c)
	rawValue := ss.GetRaw(ctx, []byte(req.Key))
	param := proposal.NewParamChange(req.Subspace, req.Key, string(rawValue))

	return &proposal.QueryParamsResponse{Param: param}, nil
}

// Subspaces implements the gRPC query handler for fetching all registered
// subspaces and all the keys for each subspace.
func (k Keeper) Subspaces(
	goCtx context.Context,
	req *proposal.QuerySubspacesRequest,
) (*proposal.QuerySubspacesResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	spaces := k.GetSubspaces()
	resp := &proposal.QuerySubspacesResponse{
		Subspaces: make([]*proposal.Subspace, len(spaces)),
	}

	ctx := sdk.UnwrapSDKContext(goCtx)
	for i, ss := range spaces {
		var keys []string
		ss.IterateKeys(ctx, func(key []byte) bool {
			keys = append(keys, string(key))
			return false
		})

		resp.Subspaces[i] = &proposal.Subspace{
			Subspace: ss.Name(),
			Keys:     keys,
		}
	}

	return resp, nil
}
