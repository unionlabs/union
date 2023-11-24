package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/store/prefix"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

var _ types.QueryServer = Querier{}

type Querier struct {
	Keeper
}

func NewQuerier(keeper Keeper) Querier {
	return Querier{Keeper: keeper}
}

// Params returns parameters of x/slashing module
func (k Keeper) Params(ctx context.Context, req *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	params, err := k.GetParams(ctx)

	return &types.QueryParamsResponse{Params: params}, err
}

// SigningInfo returns signing-info of a specific validator.
func (k Keeper) SigningInfo(ctx context.Context, req *types.QuerySigningInfoRequest) (*types.QuerySigningInfoResponse, error) {
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

	signingInfo, err := k.GetValidatorSigningInfo(ctx, consAddr)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "SigningInfo not found for validator %s", req.ConsAddress)
	}

	return &types.QuerySigningInfoResponse{ValSigningInfo: signingInfo}, nil
}

// SigningInfos returns signing-infos of all validators.
func (k Keeper) SigningInfos(ctx context.Context, req *types.QuerySigningInfosRequest) (*types.QuerySigningInfosResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	store := k.storeService.OpenKVStore(ctx)
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
