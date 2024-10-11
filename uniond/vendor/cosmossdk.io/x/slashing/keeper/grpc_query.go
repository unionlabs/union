package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/slashing/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var _ types.QueryServer = Querier{}

type Querier struct {
	Keeper
}

func NewQuerier(keeper Keeper) Querier {
	return Querier{Keeper: keeper}
}

// Params returns parameters of x/slashing module
func (k Querier) Params(ctx context.Context, _ *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	params, err := k.Keeper.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	return &types.QueryParamsResponse{Params: params}, nil
}

// SigningInfo returns signing-info of a specific validator.
func (k Querier) SigningInfo(ctx context.Context, req *types.QuerySigningInfoRequest) (*types.QuerySigningInfoResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.ConsAddress == "" {
		return nil, status.Errorf(codes.InvalidArgument, "invalid request")
	}

	consAddr, err := k.sk.ConsensusAddressCodec().StringToBytes(req.ConsAddress)
	if err != nil {
		return nil, err
	}

	signingInfo, err := k.ValidatorSigningInfo.Get(ctx, consAddr)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "SigningInfo not found for validator %s", req.ConsAddress)
	}

	return &types.QuerySigningInfoResponse{ValSigningInfo: signingInfo}, nil
}

// SigningInfos returns signing-infos of all validators.
func (k Querier) SigningInfos(ctx context.Context, req *types.QuerySigningInfosRequest) (*types.QuerySigningInfosResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	store := k.KVStoreService.OpenKVStore(ctx)
	var signInfos []types.ValidatorSigningInfo

	sigInfoStore := prefix.NewStore(runtime.KVStoreAdapter(store), types.ValidatorSigningInfoKeyPrefix)
	pageRes, err := query.Paginate(sigInfoStore, req.Pagination, func(key, value []byte) error {
		var info types.ValidatorSigningInfo
		err := k.cdc.Unmarshal(value, &info)
		if err != nil {
			return err
		}
		signInfos = append(signInfos, info)
		return nil
	})
	if err != nil {
		return nil, err
	}
	return &types.QuerySigningInfosResponse{Info: signInfos, Pagination: pageRes}, nil
}
