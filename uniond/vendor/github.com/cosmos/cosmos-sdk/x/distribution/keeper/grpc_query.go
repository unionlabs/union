package keeper

import (
	"context"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"

	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/distribution/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var _ types.QueryServer = Querier{}

type Querier struct {
	Keeper
}

func NewQuerier(keeper Keeper) Querier {
	return Querier{Keeper: keeper}
}

// Params queries params of distribution module
func (k Querier) Params(ctx context.Context, req *types.QueryParamsRequest) (*types.QueryParamsResponse, error) {
	params, err := k.Keeper.Params.Get(ctx)
	if err != nil {
		return nil, err
	}

	return &types.QueryParamsResponse{Params: params}, nil
}

// ValidatorDistributionInfo query validator's commission and self-delegation rewards
func (k Querier) ValidatorDistributionInfo(ctx context.Context, req *types.QueryValidatorDistributionInfoRequest) (*types.QueryValidatorDistributionInfoResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.ValidatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty validator address")
	}

	valAdr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(req.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	// self-delegation rewards
	val, err := k.stakingKeeper.Validator(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	if val == nil {
		return nil, errors.Wrap(types.ErrNoValidatorExists, req.ValidatorAddress)
	}

	delAdr := sdk.AccAddress(valAdr)

	del, err := k.stakingKeeper.Delegation(ctx, delAdr, valAdr)
	if err != nil {
		return nil, err
	}

	if del == nil {
		return nil, types.ErrNoDelegationExists
	}

	endingPeriod, err := k.IncrementValidatorPeriod(ctx, val)
	if err != nil {
		return nil, err
	}

	rewards, err := k.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	if err != nil {
		return nil, err
	}

	// validator's commission
	validatorCommission, err := k.GetValidatorAccumulatedCommission(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	return &types.QueryValidatorDistributionInfoResponse{
		Commission:      validatorCommission.Commission,
		OperatorAddress: delAdr.String(),
		SelfBondRewards: rewards,
	}, nil
}

// ValidatorOutstandingRewards queries rewards of a validator address
func (k Querier) ValidatorOutstandingRewards(ctx context.Context, req *types.QueryValidatorOutstandingRewardsRequest) (*types.QueryValidatorOutstandingRewardsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.ValidatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty validator address")
	}

	valAdr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(req.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	validator, err := k.stakingKeeper.Validator(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	if validator == nil {
		return nil, errors.Wrapf(types.ErrNoValidatorExists, req.ValidatorAddress)
	}

	rewards, err := k.GetValidatorOutstandingRewards(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	return &types.QueryValidatorOutstandingRewardsResponse{Rewards: rewards}, nil
}

// ValidatorCommission queries accumulated commission for a validator
func (k Querier) ValidatorCommission(ctx context.Context, req *types.QueryValidatorCommissionRequest) (*types.QueryValidatorCommissionResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.ValidatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty validator address")
	}

	valAdr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(req.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	validator, err := k.stakingKeeper.Validator(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	if validator == nil {
		return nil, errors.Wrapf(types.ErrNoValidatorExists, req.ValidatorAddress)
	}
	commission, err := k.GetValidatorAccumulatedCommission(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	return &types.QueryValidatorCommissionResponse{Commission: commission}, nil
}

// ValidatorSlashes queries slash events of a validator
func (k Querier) ValidatorSlashes(ctx context.Context, req *types.QueryValidatorSlashesRequest) (*types.QueryValidatorSlashesResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.ValidatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty validator address")
	}

	if req.EndingHeight < req.StartingHeight {
		return nil, status.Errorf(codes.InvalidArgument, "starting height greater than ending height (%d > %d)", req.StartingHeight, req.EndingHeight)
	}

	valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(req.ValidatorAddress)
	if err != nil {
		return nil, status.Errorf(codes.InvalidArgument, "invalid validator address")
	}

	store := runtime.KVStoreAdapter(k.storeService.OpenKVStore(ctx))
	slashesStore := prefix.NewStore(store, types.GetValidatorSlashEventPrefix(valAddr))

	events, pageRes, err := query.GenericFilteredPaginate(k.cdc, slashesStore, req.Pagination, func(key []byte, result *types.ValidatorSlashEvent) (*types.ValidatorSlashEvent, error) {
		if result.ValidatorPeriod < req.StartingHeight || result.ValidatorPeriod > req.EndingHeight {
			return nil, nil
		}

		return result, nil
	}, func() *types.ValidatorSlashEvent {
		return &types.ValidatorSlashEvent{}
	})
	if err != nil {
		return nil, err
	}

	slashes := []types.ValidatorSlashEvent{}
	for _, event := range events {
		slashes = append(slashes, *event)
	}

	return &types.QueryValidatorSlashesResponse{Slashes: slashes, Pagination: pageRes}, nil
}

// DelegationRewards the total rewards accrued by a delegation
func (k Querier) DelegationRewards(ctx context.Context, req *types.QueryDelegationRewardsRequest) (*types.QueryDelegationRewardsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.DelegatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty delegator address")
	}

	if req.ValidatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty validator address")
	}

	valAdr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(req.ValidatorAddress)
	if err != nil {
		return nil, err
	}

	val, err := k.stakingKeeper.Validator(ctx, valAdr)
	if err != nil {
		return nil, err
	}

	if val == nil {
		return nil, errors.Wrap(types.ErrNoValidatorExists, req.ValidatorAddress)
	}

	delAdr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddress)
	if err != nil {
		return nil, err
	}
	del, err := k.stakingKeeper.Delegation(ctx, delAdr, valAdr)
	if err != nil {
		return nil, err
	}

	if del == nil {
		return nil, types.ErrNoDelegationExists
	}

	endingPeriod, err := k.IncrementValidatorPeriod(ctx, val)
	if err != nil {
		return nil, err
	}

	rewards, err := k.CalculateDelegationRewards(ctx, val, del, endingPeriod)
	if err != nil {
		return nil, err
	}

	return &types.QueryDelegationRewardsResponse{Rewards: rewards}, nil
}

// DelegationTotalRewards the total rewards accrued by a each validator
func (k Querier) DelegationTotalRewards(ctx context.Context, req *types.QueryDelegationTotalRewardsRequest) (*types.QueryDelegationTotalRewardsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.DelegatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty delegator address")
	}

	total := sdk.DecCoins{}
	var delRewards []types.DelegationDelegatorReward

	delAdr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddress)
	if err != nil {
		return nil, err
	}

	err = k.stakingKeeper.IterateDelegations(
		ctx, delAdr,
		func(_ int64, del stakingtypes.DelegationI) (stop bool) {
			valAddr, err := k.stakingKeeper.ValidatorAddressCodec().StringToBytes(del.GetValidatorAddr())
			if err != nil {
				panic(err)
			}

			val, err := k.stakingKeeper.Validator(ctx, valAddr)
			if err != nil {
				panic(err)
			}

			endingPeriod, err := k.IncrementValidatorPeriod(ctx, val)
			if err != nil {
				panic(err)
			}

			delReward, err := k.CalculateDelegationRewards(ctx, val, del, endingPeriod)
			if err != nil {
				panic(err)
			}

			delRewards = append(delRewards, types.NewDelegationDelegatorReward(del.GetValidatorAddr(), delReward))
			total = total.Add(delReward...)
			return false
		},
	)
	if err != nil {
		return nil, err
	}

	return &types.QueryDelegationTotalRewardsResponse{Rewards: delRewards, Total: total}, nil
}

// DelegatorValidators queries the validators list of a delegator
func (k Querier) DelegatorValidators(ctx context.Context, req *types.QueryDelegatorValidatorsRequest) (*types.QueryDelegatorValidatorsResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.DelegatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty delegator address")
	}

	delAdr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddress)
	if err != nil {
		return nil, err
	}
	var validators []string

	err = k.stakingKeeper.IterateDelegations(
		ctx, delAdr,
		func(_ int64, del stakingtypes.DelegationI) (stop bool) {
			validators = append(validators, del.GetValidatorAddr())
			return false
		},
	)

	if err != nil {
		return nil, err
	}

	return &types.QueryDelegatorValidatorsResponse{Validators: validators}, nil
}

// DelegatorWithdrawAddress queries Query/delegatorWithdrawAddress
func (k Querier) DelegatorWithdrawAddress(ctx context.Context, req *types.QueryDelegatorWithdrawAddressRequest) (*types.QueryDelegatorWithdrawAddressResponse, error) {
	if req == nil {
		return nil, status.Error(codes.InvalidArgument, "invalid request")
	}

	if req.DelegatorAddress == "" {
		return nil, status.Error(codes.InvalidArgument, "empty delegator address")
	}
	delAdr, err := k.authKeeper.AddressCodec().StringToBytes(req.DelegatorAddress)
	if err != nil {
		return nil, err
	}

	withdrawAddr, err := k.GetDelegatorWithdrawAddr(ctx, delAdr)
	if err != nil {
		return nil, err
	}

	return &types.QueryDelegatorWithdrawAddressResponse{WithdrawAddress: withdrawAddr.String()}, nil
}

// CommunityPool queries the community pool coins
func (k Querier) CommunityPool(ctx context.Context, req *types.QueryCommunityPoolRequest) (*types.QueryCommunityPoolResponse, error) {
	pool, err := k.FeePool.Get(ctx)
	if err != nil {
		return nil, err
	}

	return &types.QueryCommunityPoolResponse{Pool: pool.CommunityPool}, nil
}
