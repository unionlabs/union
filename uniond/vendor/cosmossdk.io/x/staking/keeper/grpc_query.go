package keeper

import (
	"context"
	"errors"
	"strings"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/collections"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/staking/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
)

// Querier is used as Keeper will have duplicate methods if used directly, and gRPC names take precedence over keeper
type Querier struct {
	*Keeper
}

var _ types.QueryServer = Querier{}

func NewQuerier(keeper *Keeper) Querier {
	return Querier{Keeper: keeper}
}

// Validators queries all validators that match the given status
func (k Querier) Validators(ctx context.Context, req *types.QueryValidatorsRequest) (*types.QueryValidatorsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	// validate the provided status, return all the validators if the status is empty
	if req.Status != "" && !(req.Status == types.Bonded.String() || req.Status == types.Unbonded.String() || req.Status == types.Unbonding.String()) {
		return nil, status.Errorf(codes.InvalidArgument, "invalid validator status %s", req.Status)
	}

	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	valStore := prefix.NewStore(store, types.ValidatorsKey)

	validators, pageRes, err := query.GenericFilteredPaginate(k.cdc, valStore, req.Pagination, func(key []byte, val *types.Validator) (*types.Validator, error) {
		if req.Status != "" && !strings.EqualFold(val.GetStatus().String(), req.Status) {
			return nil, nil
		}

		return val, nil
	}, func() *types.Validator {
		return &types.Validator{}
	})
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	vals := types.Validators{}
	var validatorInfoList []types.ValidatorInfo
	for _, val := range validators {
		vals.Validators = append(vals.Validators, *val)
		valInfo := types.ValidatorInfo{}

		cv := val.ConsensusPubkey.GetCachedValue()
		if cv == nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrInvalidType, "public key cached value is nil")
		}

		cpk, ok := cv.(cryptotypes.PubKey)
		if ok {
			consAddr, err := k.consensusAddressCodec.BytesToString(cpk.Address())
			if err == nil {
				valInfo.ConsensusAddress = consAddr
			}
		}

		validatorInfoList = append(validatorInfoList, valInfo)
	}

	return &types.QueryValidatorsResponse{
		Validators:    vals.Validators,
		ValidatorInfo: validatorInfoList,
		Pagination:    pageRes,
	}, nil
}

// Validator queries validator info for given validator address
func (k Querier) Validator(ctx context.Context, req *types.QueryValidatorRequest) (*types.QueryValidatorResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.ValidatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "validator address cannot be empty")
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	validator, err := k.GetValidator(ctx, valAddr)
	if err != nil {
		return nil, status.Errorf(codes.NotFound, "validator %s not found", req.ValidatorAddr)
	}

	return &types.QueryValidatorResponse{Validator: validator}, nil
}

// ValidatorDelegations queries delegate info for given validator
func (k Querier) ValidatorDelegations(ctx context.Context, req *types.QueryValidatorDelegationsRequest) (*types.QueryValidatorDelegationsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.ValidatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "validator address cannot be empty")
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	var (
		dels    types.Delegations
		pageRes *query.PageResponse
	)

	dels, pageRes, err = query.CollectionPaginate(ctx, k.DelegationsByValidator,
		req.Pagination, func(key collections.Pair[sdk.ValAddress, sdk.AccAddress], _ []byte) (types.Delegation, error) {
			valAddr, delAddr := key.K1(), key.K2()
			delegation, err := k.Delegations.Get(ctx, collections.Join(delAddr, valAddr))
			if err != nil {
				return types.Delegation{}, err
			}

			return delegation, nil
		}, query.WithCollectionPaginationPairPrefix[sdk.ValAddress, sdk.AccAddress](valAddr),
	)
	if err != nil {
		delegations, pageResponse, err := k.getValidatorDelegationsLegacy(ctx, req)
		if err != nil {
			return nil, status.Error(codes.Internal, err.Error())
		}

		dels = types.Delegations{}
		for _, d := range delegations {
			dels = append(dels, *d)
		}

		pageRes = pageResponse
	}

	delResponses, err := delegationsToDelegationResponses(ctx, k.Keeper, dels)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryValidatorDelegationsResponse{
		DelegationResponses: delResponses, Pagination: pageRes,
	}, nil
}

func (k Querier) getValidatorDelegationsLegacy(ctx context.Context, req *types.QueryValidatorDelegationsRequest) ([]*types.Delegation, *query.PageResponse, error) {
	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))

	valStore := prefix.NewStore(store, types.DelegationKey)
	return query.GenericFilteredPaginate(k.cdc, valStore, req.Pagination, func(key []byte, delegation *types.Delegation) (*types.Delegation, error) {
		_, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
		if err != nil {
			return nil, err
		}

		if !strings.EqualFold(delegation.GetValidatorAddr(), req.ValidatorAddr) {
			return nil, nil
		}

		return delegation, nil
	}, func() *types.Delegation {
		return &types.Delegation{}
	})
}

// ValidatorUnbondingDelegations queries unbonding delegations of a validator
func (k Querier) ValidatorUnbondingDelegations(ctx context.Context, req *types.QueryValidatorUnbondingDelegationsRequest) (*types.QueryValidatorUnbondingDelegationsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.ValidatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "validator address cannot be empty")
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	keys, pageRes, err := query.CollectionPaginate(
		ctx,
		k.UnbondingDelegationByValIndex,
		req.Pagination,
		func(key collections.Pair[[]byte, []byte], value []byte) (collections.Pair[[]byte, []byte], error) {
			return key, nil
		},
		query.WithCollectionPaginationPairPrefix[[]byte, []byte](valAddr),
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	// loop over the collected keys and fetch unbonding delegations
	var ubds []types.UnbondingDelegation
	for _, key := range keys {
		valAddr := key.K1()
		delAddr := key.K2()
		ubdKey := types.GetUBDKey(delAddr, valAddr)
		storeValue := store.Get(ubdKey)

		ubd, err := types.UnmarshalUBD(k.cdc, storeValue)
		if err != nil {
			return nil, err
		}
		ubds = append(ubds, ubd)
	}

	return &types.QueryValidatorUnbondingDelegationsResponse{
		UnbondingResponses: ubds,
		Pagination:         pageRes,
	}, nil
}

// Delegation queries delegate info for given validator delegator pair
func (k Querier) Delegation(ctx context.Context, req *types.QueryDelegationRequest) (*types.QueryDelegationResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "delegator address cannot be empty")
	}
	if req.ValidatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "validator address cannot be empty")
	}

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	delegation, err := k.Delegations.Get(ctx, collections.Join(sdk.AccAddress(delAddr), sdk.ValAddress(valAddr)))
	if err != nil {
		if errors.Is(err, collections.ErrNotFound) {
			return nil, status.Errorf(
				codes.NotFound,
				"delegation with delegator %s not found for validator %s",
				req.DelegatorAddr, req.ValidatorAddr)
		}
		return nil, status.Error(codes.Internal, err.Error())
	}

	delResponse, err := delegationToDelegationResponse(ctx, k.Keeper, delegation)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDelegationResponse{DelegationResponse: &delResponse}, nil
}

// UnbondingDelegation queries unbonding info for given validator delegator pair
func (k Querier) UnbondingDelegation(ctx context.Context, req *types.QueryUnbondingDelegationRequest) (*types.QueryUnbondingDelegationResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Errorf(codes.InvalidArgument, "delegator address cannot be empty")
	}
	if req.ValidatorAddr == "" {
		return nil, status.Errorf(codes.InvalidArgument, "validator address cannot be empty")
	}

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	unbond, err := k.GetUnbondingDelegation(ctx, delAddr, valAddr)
	if err != nil {
		return nil, status.Errorf(
			codes.NotFound,
			"unbonding delegation with delegator %s not found for validator %s",
			req.DelegatorAddr, req.ValidatorAddr)
	}

	return &types.QueryUnbondingDelegationResponse{Unbond: unbond}, nil
}

// DelegatorDelegations queries all delegations of a given delegator address
func (k Querier) DelegatorDelegations(ctx context.Context, req *types.QueryDelegatorDelegationsRequest) (*types.QueryDelegatorDelegationsResponse, error) {
	if req == nil {
		return nil, status.Errorf(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "delegator address cannot be empty")
	}

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	delegations, pageRes, err := query.CollectionPaginate(ctx, k.Delegations, req.Pagination,
		func(_ collections.Pair[sdk.AccAddress, sdk.ValAddress], del types.Delegation) (types.Delegation, error) {
			return del, nil
		}, query.WithCollectionPaginationPairPrefix[sdk.AccAddress, sdk.ValAddress](delAddr),
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	delegationResps, err := delegationsToDelegationResponses(ctx, k.Keeper, delegations)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDelegatorDelegationsResponse{DelegationResponses: delegationResps, Pagination: pageRes}, nil
}

// DelegatorValidator queries validator info for given delegator validator pair
func (k Querier) DelegatorValidator(ctx context.Context, req *types.QueryDelegatorValidatorRequest) (*types.QueryDelegatorValidatorResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "delegator address cannot be empty")
	}
	if req.ValidatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "validator address cannot be empty")
	}

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	valAddr, err := k.validatorAddressCodec.StringToBytes(req.ValidatorAddr)
	if err != nil {
		return nil, err
	}

	validator, err := k.GetDelegatorValidator(ctx, delAddr, valAddr)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDelegatorValidatorResponse{Validator: validator}, nil
}

// DelegatorUnbondingDelegations queries all unbonding delegations of a given delegator address
func (k Querier) DelegatorUnbondingDelegations(ctx context.Context, req *types.QueryDelegatorUnbondingDelegationsRequest) (*types.QueryDelegatorUnbondingDelegationsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "delegator address cannot be empty")
	}
	var unbondingDelegations types.UnbondingDelegations

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	_, pageRes, err := query.CollectionPaginate(
		ctx,
		k.UnbondingDelegations,
		req.Pagination,
		func(key collections.Pair[[]byte, []byte], value types.UnbondingDelegation) (types.UnbondingDelegation, error) {
			unbondingDelegations = append(unbondingDelegations, value)
			return value, nil
		},
		query.WithCollectionPaginationPairPrefix[[]byte, []byte](delAddr),
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDelegatorUnbondingDelegationsResponse{
		UnbondingResponses: unbondingDelegations, Pagination: pageRes,
	}, nil
}

// HistoricalInfo queries the historical info for given height
func (k Querier) HistoricalInfo(ctx context.Context, req *types.QueryHistoricalInfoRequest) (*types.QueryHistoricalInfoResponse, error) { //nolint:staticcheck // SA1019: deprecated endpoint
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	return nil, status.Error(codes.Internal, "this endpoint has been deprecated and removed in 0.52")
}

// Redelegations queries redelegations of given address
func (k Querier) Redelegations(ctx context.Context, req *types.QueryRedelegationsRequest) (*types.QueryRedelegationsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	var redels types.Redelegations
	var pageRes *query.PageResponse
	var err error

	store := runtime.KVStoreAdapter(k.KVStoreService.OpenKVStore(ctx))
	switch {
	case req.DelegatorAddr != "" && req.SrcValidatorAddr != "" && req.DstValidatorAddr != "":
		redels, err = queryRedelegation(ctx, k, req)
	case req.DelegatorAddr == "" && req.SrcValidatorAddr != "" && req.DstValidatorAddr == "":
		redels, pageRes, err = queryRedelegationsFromSrcValidator(ctx, k, req)
	default:
		redels, pageRes, err = queryAllRedelegations(ctx, store, k, req)
	}
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}
	redelResponses, err := redelegationsToRedelegationResponses(ctx, k.Keeper, redels)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryRedelegationsResponse{RedelegationResponses: redelResponses, Pagination: pageRes}, nil
}

// DelegatorValidators queries all validators info for given delegator address
func (k Querier) DelegatorValidators(ctx context.Context, req *types.QueryDelegatorValidatorsRequest) (*types.QueryDelegatorValidatorsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "empty request")
	}

	if req.DelegatorAddr == "" {
		return nil, status.Error(codes.InvalidArgument, "delegator address cannot be empty")
	}
	var validators types.Validators

	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	_, pageRes, err := query.CollectionPaginate(ctx, k.Delegations, req.Pagination,
		func(_ collections.Pair[sdk.AccAddress, sdk.ValAddress], delegation types.Delegation) (types.Delegation, error) {
			valAddr, err := k.validatorAddressCodec.StringToBytes(delegation.GetValidatorAddr())
			if err != nil {
				return types.Delegation{}, err
			}
			validator, err := k.GetValidator(ctx, valAddr)
			if err != nil {
				return types.Delegation{}, err
			}

			validators.Validators = append(validators.Validators, validator)
			return types.Delegation{}, nil
		}, query.WithCollectionPaginationPairPrefix[sdk.AccAddress, sdk.ValAddress](delAddr),
	)
	if err != nil {
		return nil, status.Error(codes.Internal, err.Error())
	}

	return &types.QueryDelegatorValidatorsResponse{Validators: validators.Validators, Pagination: pageRes}, nil
}

// Pool queries the pool info
func (k Querier) Pool(ctx context.Context, _ *types.QueryPoolRequest) (*types.QueryPoolResponse, error) {
	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return nil, err
	}
	bondedPool := k.GetBondedPool(ctx)
	notBondedPool := k.GetNotBondedPool(ctx)

	pool := types.NewPool(
		k.bankKeeper.GetBalance(ctx, notBondedPool.GetAddress(), bondDenom).Amount,
		k.bankKeeper.GetBalance(ctx, bondedPool.GetAddress(), bondDenom).Amount,
	)

	return &types.QueryPoolResponse{Pool: pool}, nil
}

// Params queries the staking parameters
func (k Querier) Params(ctx context.Context, _ *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	params, err := k.Keeper.Params.Get(ctx)
	if err != nil {
		return nil, err
	}
	return &types.QueryParamsResponse{Params: params}, nil
}

func queryRedelegation(ctx context.Context, k Querier, req *types.QueryRedelegationsRequest) (redels types.Redelegations, err error) {
	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, err
	}

	srcValAddr, err := k.validatorAddressCodec.StringToBytes(req.SrcValidatorAddr)
	if err != nil {
		return nil, err
	}

	dstValAddr, err := k.validatorAddressCodec.StringToBytes(req.DstValidatorAddr)
	if err != nil {
		return nil, err
	}

	redel, err := k.Keeper.Redelegations.Get(ctx, collections.Join3(delAddr, srcValAddr, dstValAddr))
	if err != nil {
		return nil, status.Errorf(
			codes.NotFound,
			"redelegation not found for delegator address %s from validator address %s",
			req.DelegatorAddr, req.SrcValidatorAddr)
	}
	redels = []types.Redelegation{redel}

	return redels, nil
}

func queryRedelegationsFromSrcValidator(ctx context.Context, k Querier, req *types.QueryRedelegationsRequest) (types.Redelegations, *query.PageResponse, error) {
	valAddr, err := k.validatorAddressCodec.StringToBytes(req.SrcValidatorAddr)
	if err != nil {
		return nil, nil, err
	}

	return query.CollectionPaginate(ctx, k.RedelegationsByValSrc, req.Pagination, func(key collections.Triple[[]byte, []byte, []byte], val []byte) (types.Redelegation, error) {
		valSrcAddr, delAddr, valDstAddr := key.K1(), key.K2(), key.K3()
		red, err := k.Keeper.Redelegations.Get(ctx, collections.Join3(delAddr, valSrcAddr, valDstAddr))
		if err != nil {
			return types.Redelegation{}, err
		}
		return red, nil
	}, query.WithCollectionPaginationTriplePrefix[[]byte, []byte, []byte](valAddr))
}

func queryAllRedelegations(ctx context.Context, store storetypes.KVStore, k Querier, req *types.QueryRedelegationsRequest) (redels types.Redelegations, res *query.PageResponse, err error) {
	delAddr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddr)
	if err != nil {
		return nil, nil, err
	}

	redels, res, err = query.CollectionPaginate(ctx, k.Keeper.Redelegations, req.Pagination, func(_ collections.Triple[[]byte, []byte, []byte], red types.Redelegation) (types.Redelegation, error) {
		return red, nil
	}, query.WithCollectionPaginationTriplePrefix[[]byte, []byte, []byte](delAddr))
	if err != nil {
		return nil, nil, err
	}

	return redels, res, err
}

// util

func delegationToDelegationResponse(ctx context.Context, k *Keeper, del types.Delegation) (types.DelegationResponse, error) {
	valAddr, err := k.validatorAddressCodec.StringToBytes(del.GetValidatorAddr())
	if err != nil {
		return types.DelegationResponse{}, err
	}

	val, err := k.GetValidator(ctx, valAddr)
	if err != nil {
		return types.DelegationResponse{}, err
	}

	_, err = k.authKeeper.AddressCodec().StringToBytes(del.DelegatorAddress)
	if err != nil {
		return types.DelegationResponse{}, err
	}

	bondDenom, err := k.BondDenom(ctx)
	if err != nil {
		return types.DelegationResponse{}, err
	}

	return types.NewDelegationResp(
		del.DelegatorAddress,
		del.GetValidatorAddr(),
		del.Shares,
		sdk.NewCoin(bondDenom, val.TokensFromShares(del.Shares).TruncateInt()),
	), nil
}

func delegationsToDelegationResponses(ctx context.Context, k *Keeper, delegations types.Delegations) (types.DelegationResponses, error) {
	resp := make(types.DelegationResponses, len(delegations))

	for i, del := range delegations {
		delResp, err := delegationToDelegationResponse(ctx, k, del)
		if err != nil {
			return nil, err
		}

		resp[i] = delResp
	}

	return resp, nil
}

func redelegationsToRedelegationResponses(ctx context.Context, k *Keeper, redels types.Redelegations) (types.RedelegationResponses, error) {
	resp := make(types.RedelegationResponses, len(redels))

	for i, redel := range redels {
		_, err := k.validatorAddressCodec.StringToBytes(redel.ValidatorSrcAddress)
		if err != nil {
			return nil, err
		}
		valDstAddr, err := k.validatorAddressCodec.StringToBytes(redel.ValidatorDstAddress)
		if err != nil {
			return nil, err
		}

		_, err = k.authKeeper.AddressCodec().StringToBytes(redel.DelegatorAddress)
		if err != nil {
			return nil, err
		}

		val, err := k.GetValidator(ctx, valDstAddr)
		if err != nil {
			return nil, err
		}

		entryResponses := make([]types.RedelegationEntryResponse, len(redel.Entries))
		for j, entry := range redel.Entries {
			entryResponses[j] = types.NewRedelegationEntryResponse(
				entry.CreationHeight,
				entry.CompletionTime,
				entry.SharesDst,
				entry.InitialBalance,
				val.TokensFromShares(entry.SharesDst).TruncateInt(),
				entry.UnbondingId,
			)
		}

		resp[i] = types.NewRedelegationResponse(
			redel.DelegatorAddress,
			redel.ValidatorSrcAddress,
			redel.ValidatorDstAddress,
			entryResponses,
		)
	}

	return resp, nil
}
