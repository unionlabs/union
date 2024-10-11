package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	v1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/collections"
	"cosmossdk.io/math"
	"cosmossdk.io/store/prefix"
	"cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
)

type Querier struct {
	BaseKeeper
}

var _ types.QueryServer = BaseKeeper{}

func NewQuerier(keeper *BaseKeeper) Querier {
	return Querier{BaseKeeper: *keeper}
}

// Balance implements the Query/Balance gRPC method
func (k BaseKeeper) Balance(ctx context.Context, req *types.QueryBalanceRequest) (*types.QueryBalanceResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	address, err := k.ak.AddressCodec().StringToBytes(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "invalid address: %s", err.Error())
	}

	balance := k.GetBalance(ctx, address, req.Denom)

	return &types.QueryBalanceResponse{Balance: &balance}, nil
}

// AllBalances implements the Query/AllBalances gRPC method
func (k BaseKeeper) AllBalances(ctx context.Context, req *types.QueryAllBalancesRequest) (*types.QueryAllBalancesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	addr, err := k.ak.AddressCodec().StringToBytes(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "invalid address: %s", err.Error())
	}

	balances, pageRes, err := query.CollectionPaginate(
		ctx,
		k.Balances,
		req.Pagination,
		func(key collections.Pair[sdk.AccAddress, string], value math.Int) (sdk.Coin, error) {
			if req.ResolveDenom {
				if metadata, ok := k.GetDenomMetaData(ctx, key.K2()); ok {
					return sdk.NewCoin(metadata.Display, value), nil
				}
			}
			return sdk.NewCoin(key.K2(), value), nil
		},
		query.WithCollectionPaginationPairPrefix[sdk.AccAddress, string](addr),
	)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "paginate: %v", err)
	}

	return &types.QueryAllBalancesResponse{Balances: balances, Pagination: pageRes}, nil
}

// SpendableBalances implements a gRPC query handler for retrieving an account's
// spendable balances.
func (k BaseKeeper) SpendableBalances(ctx context.Context, req *types.QuerySpendableBalancesRequest) (*types.QuerySpendableBalancesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	addr, err := k.ak.AddressCodec().StringToBytes(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "invalid address: %s", err.Error())
	}

	zeroAmt := math.ZeroInt()
	allLocked := k.LockedCoins(ctx, addr)

	balances, pageRes, err := query.CollectionPaginate(ctx, k.Balances, req.Pagination, func(key collections.Pair[sdk.AccAddress, string], balanceAmt math.Int) (sdk.Coin, error) {
		denom := key.K2()
		coin := sdk.NewCoin(denom, zeroAmt)
		lockedAmt := allLocked.AmountOf(denom)
		switch {
		case !lockedAmt.IsPositive():
			coin.Amount = balanceAmt
		case lockedAmt.LT(balanceAmt):
			coin.Amount = balanceAmt.Sub(lockedAmt)
		}
		return coin, nil
	}, query.WithCollectionPaginationPairPrefix[sdk.AccAddress, string](addr))
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "paginate: %v", err)
	}

	return &types.QuerySpendableBalancesResponse{Balances: balances, Pagination: pageRes}, nil
}

// SpendableBalanceByDenom implements a gRPC query handler for retrieving an account's
// spendable balance for a specific denom.
func (k BaseKeeper) SpendableBalanceByDenom(ctx context.Context, req *types.QuerySpendableBalanceByDenomRequest) (*types.QuerySpendableBalanceByDenomResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	addr, err := k.ak.AddressCodec().StringToBytes(req.Address)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "invalid address: %s", err.Error())
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	spendable := k.SpendableCoin(ctx, addr, req.Denom)

	return &types.QuerySpendableBalanceByDenomResponse{Balance: &spendable}, nil
}

// TotalSupply implements the Query/TotalSupply gRPC method
func (k BaseKeeper) TotalSupply(ctx context.Context, req *types.QueryTotalSupplyRequest) (*types.QueryTotalSupplyResponse, error) {
	totalSupply, pageRes, err := k.GetPaginatedTotalSupply(ctx, req.Pagination)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryTotalSupplyResponse{Supply: totalSupply, Pagination: pageRes}, nil
}

// SupplyOf implements the Query/SupplyOf gRPC method
func (k BaseKeeper) SupplyOf(ctx context.Context, req *types.QuerySupplyOfRequest) (*types.QuerySupplyOfResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	supply := k.GetSupply(ctx, req.Denom)

	return &types.QuerySupplyOfResponse{Amount: sdk.NewCoin(req.Denom, supply.Amount)}, nil
}

// Params implements the gRPC service handler for querying x/bank parameters.
func (k BaseKeeper) Params(ctx context.Context, req *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	params := k.GetParams(ctx)

	return &types.QueryParamsResponse{Params: params}, nil
}

// DenomsMetadata implements Query/DenomsMetadata gRPC method.
func (k BaseKeeper) DenomsMetadata(c context.Context, req *types.QueryDenomsMetadataRequest) (*types.QueryDenomsMetadataResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}
	kvStore := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(c))
	store := prefix.NewStore(kvStore, types.DenomMetadataPrefix)

	metadatas := []types.Metadata{}
	pageRes, err := query.Paginate(store, req.Pagination, func(_, value []byte) error {
		var metadata types.Metadata
		k.cdc.MustUnmarshal(value, &metadata)

		metadatas = append(metadatas, metadata)
		return nil
	})
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDenomsMetadataResponse{
		Metadatas:  metadatas,
		Pagination: pageRes,
	}, nil
}

// DenomMetadata implements Query/DenomMetadata gRPC method.
func (k BaseKeeper) DenomMetadata(ctx context.Context, req *types.QueryDenomMetadataRequest) (*types.QueryDenomMetadataResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	metadata, found := k.GetDenomMetaData(ctx, req.Denom)
	if !found {
		return nil, status.Errorf(codes.NotFound, "client metadata for denom %s", req.Denom)
	}

	return &types.QueryDenomMetadataResponse{
		Metadata: metadata,
	}, nil
}

// DenomMetadataByQueryString is identical to DenomMetadata query, but receives request via query string.
func (k BaseKeeper) DenomMetadataByQueryString(ctx context.Context, req *types.QueryDenomMetadataByQueryStringRequest) (*types.QueryDenomMetadataByQueryStringResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	res, err := k.DenomMetadata(ctx, &types.QueryDenomMetadataRequest{
		Denom: req.Denom,
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryDenomMetadataByQueryStringResponse{Metadata: res.Metadata}, nil
}

// DenomMetadataV2 is identical to DenomMetadata but receives protoreflect types instead of gogo types.  It exists to
// resolve a cyclic dependency existent between x/auth and x/bank, so that x/auth may call this keeper without
// depending on x/bank.
func (k BaseKeeper) DenomMetadataV2(ctx context.Context, req *v1beta1.QueryDenomMetadataRequest) (*v1beta1.QueryDenomMetadataResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	metadata, found := k.GetDenomMetaData(ctx, req.Denom)
	if !found {
		return nil, status.Errorf(codes.NotFound, "client metadata for denom %s", req.Denom)
	}

	denomUnits := make([]*v1beta1.DenomUnit, len(metadata.DenomUnits))
	for i, unit := range metadata.DenomUnits {
		denomUnits[i] = &v1beta1.DenomUnit{
			Denom:    unit.Denom,
			Exponent: unit.Exponent,
			Aliases:  unit.Aliases,
		}
	}
	metadataV2 := &v1beta1.Metadata{
		Description: metadata.Description,
		DenomUnits:  denomUnits,
		Base:        metadata.Base,
		Display:     metadata.Display,
		Name:        metadata.Name,
		Symbol:      metadata.Symbol,
		Uri:         metadata.URI,
		UriHash:     metadata.URIHash,
	}

	return &v1beta1.QueryDenomMetadataResponse{
		Metadata: metadataV2,
	}, nil
}

// DenomOwners returns all the account address that own a requested token denom.
func (k BaseKeeper) DenomOwners(
	ctx context.Context,
	req *types.QueryDenomOwnersRequest,
) (*types.QueryDenomOwnersResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if err := sdk.ValidateDenom(req.Denom); err != nil {
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}

	denomOwners, pageRes, err := query.CollectionPaginate(
		ctx,
		k.Balances.Indexes.Denom,
		req.Pagination,
		func(key collections.Pair[string, sdk.AccAddress], value collections.NoValue) (*types.DenomOwner, error) {
			amt, err := k.Balances.Get(ctx, collections.Join(key.K2(), req.Denom))
			if err != nil {
				return nil, err
			}
			addr, err := k.ak.AddressCodec().BytesToString(key.K2())
			if err != nil {
				return nil, err
			}
			return &types.DenomOwner{Address: addr, Balance: sdk.NewCoin(req.Denom, amt)}, nil
		},
		query.WithCollectionPaginationPairPrefix[string, sdk.AccAddress](req.Denom),
	)
	if err != nil {
		return nil, err
	}

	return &types.QueryDenomOwnersResponse{DenomOwners: denomOwners, Pagination: pageRes}, nil
}

func (k BaseKeeper) SendEnabled(ctx context.Context, req *types.QuerySendEnabledRequest) (*types.QuerySendEnabledResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}
	resp := &types.QuerySendEnabledResponse{}
	if len(req.Denoms) > 0 {
		for _, denom := range req.Denoms {
			if se, ok := k.getSendEnabled(ctx, denom); ok {
				resp.SendEnabled = append(resp.SendEnabled, types.NewSendEnabled(denom, se))
			}
		}
	} else {
		results, pageResp, err := query.CollectionPaginate(
			ctx,
			k.BaseViewKeeper.SendEnabled,
			req.Pagination, func(key string, value bool) (*types.SendEnabled, error) {
				return types.NewSendEnabled(key, value), nil
			},
		)
		if err != nil {
			return nil, status.Error(codes.Internal, err.Error())
		}
		resp.SendEnabled = results
		resp.Pagination = pageResp
	}

	return resp, nil
}

// DenomOwnersByQuery is identical to DenomOwner query, but receives denom values via query string.
func (k BaseKeeper) DenomOwnersByQuery(ctx context.Context, req *types.QueryDenomOwnersByQueryRequest) (*types.QueryDenomOwnersByQueryResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}
	resp, err := k.DenomOwners(ctx, &types.QueryDenomOwnersRequest{
		Denom:      req.Denom,
		Pagination: req.Pagination,
	})
	if err != nil {
		return nil, err
	}

	return &types.QueryDenomOwnersByQueryResponse{DenomOwners: resp.DenomOwners, Pagination: resp.Pagination}, nil
}
