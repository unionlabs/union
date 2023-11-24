package simulation

import (
	"errors"
	"fmt"
	"math/rand"

	"cosmossdk.io/collections"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/distribution/keeper"
	"github.com/cosmos/cosmos-sdk/x/distribution/types"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

// Simulation operation weights constants
const (
	OpWeightMsgSetWithdrawAddress          = "op_weight_msg_set_withdraw_address"
	OpWeightMsgWithdrawDelegationReward    = "op_weight_msg_withdraw_delegation_reward"
	OpWeightMsgWithdrawValidatorCommission = "op_weight_msg_withdraw_validator_commission"
	OpWeightMsgFundCommunityPool           = "op_weight_msg_fund_community_pool"

	DefaultWeightMsgSetWithdrawAddress          int = 50
	DefaultWeightMsgWithdrawDelegationReward    int = 50
	DefaultWeightMsgWithdrawValidatorCommission int = 50
	DefaultWeightMsgFundCommunityPool           int = 50
)

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(
	appParams simtypes.AppParams,
	cdc codec.JSONCodec,
	txConfig client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k keeper.Keeper,
	sk types.StakingKeeper,
) simulation.WeightedOperations {
	var weightMsgSetWithdrawAddress int
	appParams.GetOrGenerate(OpWeightMsgSetWithdrawAddress, &weightMsgSetWithdrawAddress, nil, func(_ *rand.Rand) {
		weightMsgSetWithdrawAddress = DefaultWeightMsgSetWithdrawAddress
	})

	var weightMsgWithdrawDelegationReward int
	appParams.GetOrGenerate(OpWeightMsgWithdrawDelegationReward, &weightMsgWithdrawDelegationReward, nil, func(_ *rand.Rand) {
		weightMsgWithdrawDelegationReward = DefaultWeightMsgWithdrawDelegationReward
	})

	var weightMsgWithdrawValidatorCommission int
	appParams.GetOrGenerate(OpWeightMsgWithdrawValidatorCommission, &weightMsgWithdrawValidatorCommission, nil, func(_ *rand.Rand) {
		weightMsgWithdrawValidatorCommission = DefaultWeightMsgWithdrawValidatorCommission
	})

	var weightMsgFundCommunityPool int
	appParams.GetOrGenerate(OpWeightMsgFundCommunityPool, &weightMsgFundCommunityPool, nil, func(_ *rand.Rand) {
		weightMsgFundCommunityPool = DefaultWeightMsgFundCommunityPool
	})

	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgSetWithdrawAddress,
			SimulateMsgSetWithdrawAddress(txConfig, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgWithdrawDelegationReward,
			SimulateMsgWithdrawDelegatorReward(txConfig, ak, bk, k, sk),
		),
		simulation.NewWeightedOperation(
			weightMsgWithdrawValidatorCommission,
			SimulateMsgWithdrawValidatorCommission(txConfig, ak, bk, k, sk),
		),
		simulation.NewWeightedOperation(
			weightMsgFundCommunityPool,
			SimulateMsgFundCommunityPool(txConfig, ak, bk, k, sk),
		),
	}
}

// SimulateMsgSetWithdrawAddress generates a MsgSetWithdrawAddress with random values.
func SimulateMsgSetWithdrawAddress(txConfig client.TxConfig, ak types.AccountKeeper, bk types.BankKeeper, k keeper.Keeper) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		isWithdrawAddrEnabled, err := k.GetWithdrawAddrEnabled(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgSetWithdrawAddress{}), "error getting params"), nil, err
		}

		if !isWithdrawAddrEnabled {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgSetWithdrawAddress{}), "withdrawal is not enabled"), nil, nil
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)
		simToAccount, _ := simtypes.RandomAcc(r, accs)

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		msg := types.NewMsgSetWithdrawAddress(simAccount.Address, simToAccount.Address)

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txConfig,
			Cdc:             nil,
			Msg:             msg,
			Context:         ctx,
			SimAccount:      simAccount,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      types.ModuleName,
			CoinsSpentInMsg: spendable,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgWithdrawDelegatorReward generates a MsgWithdrawDelegatorReward with random values.
func SimulateMsgWithdrawDelegatorReward(txConfig client.TxConfig, ak types.AccountKeeper, bk types.BankKeeper, k keeper.Keeper, sk types.StakingKeeper) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		simAccount, _ := simtypes.RandomAcc(r, accs)
		delegations, err := sk.GetAllDelegatorDelegations(ctx, simAccount.Address)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgWithdrawDelegatorReward{}), "error getting delegations"), nil, err
		}
		if len(delegations) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgWithdrawDelegatorReward{}), "number of delegators equal 0"), nil, nil
		}

		delegation := delegations[r.Intn(len(delegations))]

		delAddr, err := sk.ValidatorAddressCodec().StringToBytes(delegation.GetValidatorAddr())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgWithdrawDelegatorReward{}), "error converting validator address"), nil, err
		}
		validator, err := sk.Validator(ctx, delAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgWithdrawDelegatorReward{}), "error getting validator"), nil, err
		}
		if validator == nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgWithdrawDelegatorReward{}), "validator is nil"), nil, fmt.Errorf("validator %s not found", delegation.GetValidatorAddr())
		}

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		msg := types.NewMsgWithdrawDelegatorReward(simAccount.Address.String(), validator.GetOperator())

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txConfig,
			Cdc:             nil,
			Msg:             msg,
			Context:         ctx,
			SimAccount:      simAccount,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      types.ModuleName,
			CoinsSpentInMsg: spendable,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgWithdrawValidatorCommission generates a MsgWithdrawValidatorCommission with random values.
func SimulateMsgWithdrawValidatorCommission(txConfig client.TxConfig, ak types.AccountKeeper, bk types.BankKeeper, k keeper.Keeper, sk types.StakingKeeper) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgWithdrawValidatorCommission{})

		allVals, err := sk.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting all validators"), nil, err
		}

		validator, ok := testutil.RandSliceElem(r, allVals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "random validator is not ok"), nil, nil
		}

		valBz, err := sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error converting validator address"), nil, err
		}

		commission, err := k.GetValidatorAccumulatedCommission(ctx, valBz)

		if err != nil && !errors.Is(err, collections.ErrNotFound) {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator commission"), nil, err
		}

		if commission.Commission.IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator commission is zero"), nil, nil
		}

		simAccount, found := simtypes.FindAccount(accs, sdk.AccAddress(valBz))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "could not find account"), nil, fmt.Errorf("validator %s not found", validator.GetOperator())
		}

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		msg := types.NewMsgWithdrawValidatorCommission(validator.GetOperator())

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txConfig,
			Cdc:             nil,
			Msg:             msg,
			Context:         ctx,
			SimAccount:      simAccount,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      types.ModuleName,
			CoinsSpentInMsg: spendable,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgFundCommunityPool simulates MsgFundCommunityPool execution where
// a random account sends a random amount of its funds to the community pool.
func SimulateMsgFundCommunityPool(txConfig client.TxConfig, ak types.AccountKeeper, bk types.BankKeeper, k keeper.Keeper, sk types.StakingKeeper) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		funder, _ := simtypes.RandomAcc(r, accs)

		account := ak.GetAccount(ctx, funder.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		fundAmount := simtypes.RandSubsetCoins(r, spendable)
		if fundAmount.Empty() {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgFundCommunityPool{}), "fund amount is empty"), nil, nil
		}

		var (
			fees sdk.Coins
			err  error
		)

		coins, hasNeg := spendable.SafeSub(fundAmount...)
		if !hasNeg {
			fees, err = simtypes.RandomFees(r, ctx, coins)
			if err != nil {
				return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(&types.MsgFundCommunityPool{}), "unable to generate fees"), nil, err
			}
		}

		msg := types.NewMsgFundCommunityPool(fundAmount, funder.Address.String())

		txCtx := simulation.OperationInput{
			R:             r,
			App:           app,
			TxGen:         txConfig,
			Cdc:           nil,
			Msg:           msg,
			Context:       ctx,
			SimAccount:    funder,
			AccountKeeper: ak,
			ModuleName:    types.ModuleName,
		}

		return simulation.GenAndDeliverTx(txCtx, fees)
	}
}
