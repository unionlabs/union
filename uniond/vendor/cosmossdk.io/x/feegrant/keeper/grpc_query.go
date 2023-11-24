package keeper

import (
	"bytes"
	"context"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/feegrant"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

var _ feegrant.QueryServer = Keeper{}

// Allowance returns granted allowance to the grantee by the granter.
func (q Keeper) Allowance(c context.Context, req *feegrant.QueryAllowanceRequest) (*feegrant.QueryAllowanceResponse, error) {
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

	ctx := sdk.UnwrapSDKContext(c)

	var grants []*feegrant.Grant

	store := q.storeService.OpenKVStore(ctx)
	grantsStore := prefix.NewStore(runtime.KVStoreAdapter(store), feegrant.FeeAllowancePrefixByGrantee(granteeAddr))

	pageRes, err := query.Paginate(grantsStore, req.Pagination, func(key, value []byte) error {
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

	granterAddr, err := q.authKeeper.AddressCodec().StringToBytes(req.Granter)
	if err != nil {
		return nil, err
	}

	ctx := sdk.UnwrapSDKContext(c)

	store := q.storeService.OpenKVStore(ctx)
	prefixStore := prefix.NewStore(runtime.KVStoreAdapter(store), feegrant.FeeAllowanceKeyPrefix)
	grants, pageRes, err := query.GenericFilteredPaginate(q.cdc, prefixStore, req.Pagination, func(key []byte, grant *feegrant.Grant) (*feegrant.Grant, error) {
		// ParseAddressesFromFeeAllowanceKey expects the full key including the prefix.
		granter, _ := feegrant.ParseAddressesFromFeeAllowanceKey(append(feegrant.FeeAllowanceKeyPrefix, key...))
		if !bytes.Equal(granter, granterAddr) {
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
