package keeper

import (
	"context"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/collections"
	"cosmossdk.io/x/feegrant"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var _ feegrant.QueryServer = Keeper{}

// Allowance returns granted allowance to the grantee by the granter.
func (q Keeper) Allowance(ctx context.Context, req *feegrant.QueryAllowanceRequest) (*feegrant.QueryAllowanceResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	granterAddr, err := q.authKeeper.AddressCodec().StringToBytes(req.Granter)
	if err != nil {
		return nil, err
	}

	granteeAddr, err := q.authKeeper.AddressCodec().StringToBytes(req.Grantee)
	if err != nil {
		return nil, err
	}

	feeAllowance, err := q.GetAllowance(ctx, granterAddr, granteeAddr)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	msg, ok := feeAllowance.(proto.Message)
	if !ok {
		return nil, status.Errorf(codes.Internal, "can't proto marshal %T", msg)
	}

	feeAllowanceAny, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &feegrant.QueryAllowanceResponse{
		Allowance: &feegrant.Grant{
			Granter:   req.Granter,
			Grantee:   req.Grantee,
			Allowance: feeAllowanceAny,
		},
	}, nil
}

// Allowances queries all the allowances granted to the given grantee.
func (q Keeper) Allowances(c context.Context, req *feegrant.QueryAllowancesRequest) (*feegrant.QueryAllowancesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	granteeAddr, err := q.authKeeper.AddressCodec().StringToBytes(req.Grantee)
	if err != nil {
		return nil, err
	}

	var grants []*feegrant.Grant

	_, pageRes, err := query.CollectionFilteredPaginate(c, q.FeeAllowance, req.Pagination,
		func(key collections.Pair[sdk.AccAddress, sdk.AccAddress], grant feegrant.Grant) (include bool, err error) {
			grants = append(grants, &grant)
			return true, nil
		}, func(_ collections.Pair[sdk.AccAddress, sdk.AccAddress], value feegrant.Grant) (*feegrant.Grant, error) {
			return &value, nil
		}, query.WithCollectionPaginationPairPrefix[sdk.AccAddress, sdk.AccAddress](granteeAddr),
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &feegrant.QueryAllowancesResponse{Allowances: grants, Pagination: pageRes}, nil
}

// AllowancesByGranter queries all the allowances granted by the given granter
func (q Keeper) AllowancesByGranter(c context.Context, req *feegrant.QueryAllowancesByGranterRequest) (*feegrant.QueryAllowancesByGranterResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	granterAddr, err := q.authKeeper.AddressCodec().StringToBytes(req.Granter)
	if err != nil {
		return nil, err
	}

	var grants []*feegrant.Grant
	_, pageRes, err := query.CollectionFilteredPaginate(c, q.FeeAllowance, req.Pagination,
		func(key collections.Pair[sdk.AccAddress, sdk.AccAddress], grant feegrant.Grant) (include bool, err error) {
			if !sdk.AccAddress(granterAddr).Equals(key.K2()) {
				return false, nil
			}

			grants = append(grants, &grant)
			return true, nil
		},
		func(_ collections.Pair[sdk.AccAddress, sdk.AccAddress], grant feegrant.Grant) (*feegrant.Grant, error) {
			return &grant, nil
		},
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &feegrant.QueryAllowancesByGranterResponse{Allowances: grants, Pagination: pageRes}, nil
}
