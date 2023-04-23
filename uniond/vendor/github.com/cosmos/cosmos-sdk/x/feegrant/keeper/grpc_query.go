package keeper

import (
	"context"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/feegrant"
)

var _ feegrant.QueryServer = Keeper{}

// Allowance returns fee granted to the grantee by the granter.
func (q Keeper) Allowance(c context.Context, req *feegrant.QueryAllowanceRequest) (*feegrant.QueryAllowanceResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	granterAddr, err := sdk.AccAddressFromBech32(req.Granter)
	if err != nil {
		return nil, err
	}

	granteeAddr, err := sdk.AccAddressFromBech32(req.Grantee)
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	feeAllowance, err := q.GetAllowance(ctx, granterAddr, granteeAddr)
	if err != nil {
		return nil, status.Errorf(codes.Internal, err.Error())
	}

	msg, ok := feeAllowance.(proto.Message)
	if !ok {
		return nil, status.Errorf(codes.Internal, "can't proto marshal %T", msg)
	}

	feeAllowanceAny, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return nil, status.Errorf(codes.Internal, err.Error())
	}

	return &feegrant.QueryAllowanceResponse{
		Allowance: &feegrant.Grant{
			Granter:   granterAddr.String(),
			Grantee:   granteeAddr.String(),
			Allowance: feeAllowanceAny,
		},
	}, nil
}

// Allowances queries all the allowances granted to the given grantee.
func (q Keeper) Allowances(c context.Context, req *feegrant.QueryAllowancesRequest) (*feegrant.QueryAllowancesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	granteeAddr, err := sdk.AccAddressFromBech32(req.Grantee)
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	var grants []*feegrant.Grant

	store := ctx.KVStore(q.storeKey)
	grantsStore := prefix.NewStore(store, feegrant.FeeAllowancePrefixByGrantee(granteeAddr))

	pageRes, err := query.Paginate(grantsStore, req.Pagination, func(key []byte, value []byte) error {
		var grant feegrant.Grant

		if err := q.cdc.Unmarshal(value, &grant); err != nil {
			return err
		}

		grants = append(grants, &grant)
		return nil
	})
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

	granterAddr, err := sdk.AccAddressFromBech32(req.Granter)
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	store := ctx.KVStore(q.storeKey)
	prefixStore := prefix.NewStore(store, feegrant.FeeAllowanceKeyPrefix)
	grants, pageRes, err := query.GenericFilteredPaginate(q.cdc, prefixStore, req.Pagination, func(key []byte, grant *feegrant.Grant) (*feegrant.Grant, error) {
		// ParseAddressesFromFeeAllowanceKey expects the full key including the prefix.
		granter, _ := feegrant.ParseAddressesFromFeeAllowanceKey(append(feegrant.FeeAllowanceKeyPrefix, key...))
		if !granter.Equals(granterAddr) {
			return nil, nil
		}

		return grant, nil
	}, func() *feegrant.Grant {
		return &feegrant.Grant{}
	})
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &feegrant.QueryAllowancesByGranterResponse{Allowances: grants, Pagination: pageRes}, nil
}
