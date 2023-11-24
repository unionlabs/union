// Package v2 creates in-place store migrations for fixing tracking
// delegations with vesting accounts.
// ref: https://github.com/cosmos/cosmos-sdk/issues/8601
// ref: https://github.com/cosmos/cosmos-sdk/issues/8812
//
// The migration script modifies x/auth state, hence lives in the `x/auth/legacy`
// folder. However, it needs access to staking and bank state. To avoid
// cyclic dependencies, we cannot import those 2 keepers in this file. To solve
// this, we use the baseapp router to do inter-module querying, by importing
// the `baseapp.QueryRouter grpc.Server`. This is really hacky.
//
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
//
// Proposals to refactor this file have been made in:
// https://github.com/cosmos/cosmos-sdk/issues/9070
// The preferred solution is to use inter-module communication (ADR-033), and
// this file will be refactored to use ADR-033 once it's ready.
package v2

import (
	"errors"
	"fmt"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cosmos/gogoproto/grpc"
	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"github.com/cosmos/cosmos-sdk/baseapp"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting/exported"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

const (
	delegatorDelegationPath           = "/cosmos.staking.v1beta1.Query/DelegatorDelegations"
	stakingParamsPath                 = "/cosmos.staking.v1beta1.Query/Params"
	delegatorUnbondingDelegationsPath = "/cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations"
	balancesPath                      = "/cosmos.bank.v1beta1.Query/AllBalances"
)

// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func migrateVestingAccounts(ctx sdk.Context, account sdk.AccountI, queryServer grpc.Server) (sdk.AccountI, error) {
	bondDenom, err := getBondDenom(ctx, queryServer)
	if err != nil {
		return nil, err
	}

	asVesting, ok := account.(exported.VestingAccount)
	if !ok {
		return nil, nil
	}

	addr := account.GetAddress().String()
	balance, err := getBalance(
		ctx,
		addr,
		queryServer,
	)
	if err != nil {
		return nil, err
	}

	delegations, err := getDelegatorDelegationsSum(
		ctx,
		addr,
		queryServer,
	)
	if err != nil {
		return nil, err
	}

	unbondingDelegations, err := getDelegatorUnbondingDelegationsSum(
		ctx,
		addr,
		bondDenom,
		queryServer,
	)
	if err != nil {
		return nil, err
	}

	delegations = delegations.Add(unbondingDelegations...)

	asVesting, ok = resetVestingDelegatedBalances(asVesting)
	if !ok {
		return nil, nil
	}

	// balance before any delegation includes balance of delegation
	for _, coin := range delegations {
		balance = balance.Add(coin)
	}

	asVesting.TrackDelegation(ctx.BlockTime(), balance, delegations)

	return asVesting.(sdk.AccountI), nil
}

func resetVestingDelegatedBalances(evacct exported.VestingAccount) (exported.VestingAccount, bool) {
	// reset `DelegatedVesting` and `DelegatedFree` to zero
	df := sdk.NewCoins()
	dv := sdk.NewCoins()

	switch vacct := evacct.(type) {
	case *vestingtypes.ContinuousVestingAccount:
		vacct.DelegatedVesting = dv
		vacct.DelegatedFree = df
		return vacct, true
	case *vestingtypes.DelayedVestingAccount:
		vacct.DelegatedVesting = dv
		vacct.DelegatedFree = df
		return vacct, true
	case *vestingtypes.PeriodicVestingAccount:
		vacct.DelegatedVesting = dv
		vacct.DelegatedFree = df
		return vacct, true
	default:
		return nil, false
	}
}

// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func getDelegatorDelegationsSum(ctx sdk.Context, address string, queryServer grpc.Server) (sdk.Coins, error) {
	querier, ok := queryServer.(*baseapp.GRPCQueryRouter)
	if !ok {
		return nil, fmt.Errorf("unexpected type: %T wanted *baseapp.GRPCQueryRouter", queryServer)
	}

	queryFn := querier.Route(delegatorDelegationPath)

	q := &stakingtypes.QueryDelegatorDelegationsRequest{
		DelegatorAddr: address,
	}

	b, err := proto.Marshal(q)
	if err != nil {
		return nil, fmt.Errorf("cannot marshal staking type query request, %w", err)
	}
	req := abci.RequestQuery{
		Data: b,
		Path: delegatorDelegationPath,
	}
	resp, err := queryFn(ctx, &req)
	if err != nil {
		e, ok := status.FromError(err)
		if ok && e.Code() == codes.NotFound {
			return nil, nil
		}
		return nil, fmt.Errorf("staking query error, %w", err)
	}

	balance := new(stakingtypes.QueryDelegatorDelegationsResponse)
	if err := proto.Unmarshal(resp.Value, balance); err != nil {
		return nil, fmt.Errorf("unable to unmarshal delegator query delegations: %w", err)
	}

	res := sdk.NewCoins()
	for _, i := range balance.DelegationResponses {
		res = res.Add(i.Balance)
	}

	return res, nil
}

// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func getDelegatorUnbondingDelegationsSum(ctx sdk.Context, address, bondDenom string, queryServer grpc.Server) (sdk.Coins, error) {
	querier, ok := queryServer.(*baseapp.GRPCQueryRouter)
	if !ok {
		return nil, fmt.Errorf("unexpected type: %T wanted *baseapp.GRPCQueryRouter", queryServer)
	}

	queryFn := querier.Route(delegatorUnbondingDelegationsPath)

	q := &stakingtypes.QueryDelegatorUnbondingDelegationsRequest{
		DelegatorAddr: address,
	}

	b, err := proto.Marshal(q)
	if err != nil {
		return nil, fmt.Errorf("cannot marshal staking type query request, %w", err)
	}
	req := abci.RequestQuery{
		Data: b,
		Path: delegatorUnbondingDelegationsPath,
	}
	resp, err := queryFn(ctx, &req)
	if err != nil && !errors.Is(err, sdkerrors.ErrNotFound) {
		e, ok := status.FromError(err)
		if ok && e.Code() == codes.NotFound {
			return nil, nil
		}
		return nil, fmt.Errorf("staking query error, %w", err)
	}

	balance := new(stakingtypes.QueryDelegatorUnbondingDelegationsResponse)
	if err := proto.Unmarshal(resp.Value, balance); err != nil {
		return nil, fmt.Errorf("unable to unmarshal delegator query delegations: %w", err)
	}

	res := sdk.NewCoins()
	for _, i := range balance.UnbondingResponses {
		for _, r := range i.Entries {
			res = res.Add(sdk.NewCoin(bondDenom, r.Balance))
		}
	}

	return res, nil
}

// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func getBalance(ctx sdk.Context, address string, queryServer grpc.Server) (sdk.Coins, error) {
	querier, ok := queryServer.(*baseapp.GRPCQueryRouter)
	if !ok {
		return nil, fmt.Errorf("unexpected type: %T wanted *baseapp.GRPCQueryRouter", queryServer)
	}

	queryFn := querier.Route(balancesPath)

	q := &banktypes.QueryAllBalancesRequest{
		Address:    address,
		Pagination: nil,
	}
	b, err := proto.Marshal(q)
	if err != nil {
		return nil, fmt.Errorf("cannot marshal bank type query request, %w", err)
	}

	req := abci.RequestQuery{
		Data: b,
		Path: balancesPath,
	}
	resp, err := queryFn(ctx, &req)
	if err != nil {
		return nil, fmt.Errorf("bank query error, %w", err)
	}
	balance := new(banktypes.QueryAllBalancesResponse)
	if err := proto.Unmarshal(resp.Value, balance); err != nil {
		return nil, fmt.Errorf("unable to unmarshal bank balance response: %w", err)
	}
	return balance.Balances, nil
}

// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func getBondDenom(ctx sdk.Context, queryServer grpc.Server) (string, error) {
	querier, ok := queryServer.(*baseapp.GRPCQueryRouter)
	if !ok {
		return "", fmt.Errorf("unexpected type: %T wanted *baseapp.GRPCQueryRouter", queryServer)
	}

	queryFn := querier.Route(stakingParamsPath)

	q := &stakingtypes.QueryParamsRequest{}

	b, err := proto.Marshal(q)
	if err != nil {
		return "", fmt.Errorf("cannot marshal staking params query request, %w", err)
	}
	req := abci.RequestQuery{
		Data: b,
		Path: stakingParamsPath,
	}

	resp, err := queryFn(ctx, &req)
	if err != nil {
		return "", fmt.Errorf("staking query error, %w", err)
	}

	params := new(stakingtypes.QueryParamsResponse)
	if err := proto.Unmarshal(resp.Value, params); err != nil {
		return "", fmt.Errorf("unable to unmarshal delegator query delegations: %w", err)
	}

	return params.Params.BondDenom, nil
}

// MigrateAccount migrates vesting account to make the DelegatedVesting and DelegatedFree fields correctly
// track delegations.
// References: https://github.com/cosmos/cosmos-sdk/issues/8601, https://github.com/cosmos/cosmos-sdk/issues/8812
//
// We use the baseapp.QueryRouter here to do inter-module state querying.
// PLEASE DO NOT REPLICATE THIS PATTERN IN YOUR OWN APP.
func MigrateAccount(ctx sdk.Context, account sdk.AccountI, queryServer grpc.Server) (sdk.AccountI, error) {
	return migrateVestingAccounts(ctx, account, queryServer)
}
