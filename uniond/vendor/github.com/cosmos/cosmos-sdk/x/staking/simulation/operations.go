package simulation

import (
	"bytes"
	"fmt"
	"math/rand"

	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"
	"github.com/cosmos/cosmos-sdk/x/staking/keeper"
	"github.com/cosmos/cosmos-sdk/x/staking/types"
)

// Simulation operation weights constants
const (
	DefaultWeightMsgCreateValidator           int = 100
	DefaultWeightMsgEditValidator             int = 5
	DefaultWeightMsgDelegate                  int = 100
	DefaultWeightMsgUndelegate                int = 100
	DefaultWeightMsgBeginRedelegate           int = 100
	DefaultWeightMsgCancelUnbondingDelegation int = 100

	OpWeightMsgCreateValidator           = "op_weight_msg_create_validator"
	OpWeightMsgEditValidator             = "op_weight_msg_edit_validator"
	OpWeightMsgDelegate                  = "op_weight_msg_delegate"
	OpWeightMsgUndelegate                = "op_weight_msg_undelegate"
	OpWeightMsgBeginRedelegate           = "op_weight_msg_begin_redelegate"
	OpWeightMsgCancelUnbondingDelegation = "op_weight_msg_cancel_unbonding_delegation"
)

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(
	appParams simtypes.AppParams,
	cdc codec.JSONCodec,
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simulation.WeightedOperations {
	var (
		weightMsgCreateValidator           int
		weightMsgEditValidator             int
		weightMsgDelegate                  int
		weightMsgUndelegate                int
		weightMsgBeginRedelegate           int
		weightMsgCancelUnbondingDelegation int
	)

	appParams.GetOrGenerate(OpWeightMsgCreateValidator, &weightMsgCreateValidator, nil, func(_ *rand.Rand) {
		weightMsgCreateValidator = DefaultWeightMsgCreateValidator
	})

	appParams.GetOrGenerate(OpWeightMsgEditValidator, &weightMsgEditValidator, nil, func(_ *rand.Rand) {
		weightMsgEditValidator = DefaultWeightMsgEditValidator
	})

	appParams.GetOrGenerate(OpWeightMsgDelegate, &weightMsgDelegate, nil, func(_ *rand.Rand) {
		weightMsgDelegate = DefaultWeightMsgDelegate
	})

	appParams.GetOrGenerate(OpWeightMsgUndelegate, &weightMsgUndelegate, nil, func(_ *rand.Rand) {
		weightMsgUndelegate = DefaultWeightMsgUndelegate
	})

	appParams.GetOrGenerate(OpWeightMsgBeginRedelegate, &weightMsgBeginRedelegate, nil, func(_ *rand.Rand) {
		weightMsgBeginRedelegate = DefaultWeightMsgBeginRedelegate
	})

	appParams.GetOrGenerate(OpWeightMsgCancelUnbondingDelegation, &weightMsgCancelUnbondingDelegation, nil, func(_ *rand.Rand) {
		weightMsgCancelUnbondingDelegation = DefaultWeightMsgCancelUnbondingDelegation
	})

	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgCreateValidator,
			SimulateMsgCreateValidator(txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgEditValidator,
			SimulateMsgEditValidator(txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgDelegate,
			SimulateMsgDelegate(txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgUndelegate,
			SimulateMsgUndelegate(txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgBeginRedelegate,
			SimulateMsgBeginRedelegate(txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgCancelUnbondingDelegation,
			SimulateMsgCancelUnbondingDelegate(txGen, ak, bk, k),
		),
	}
}

// SimulateMsgCreateValidator generates a MsgCreateValidator with random values
func SimulateMsgCreateValidator(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgCreateValidator{})

		simAccount, _ := simtypes.RandomAcc(r, accs)
		address := sdk.ValAddress(simAccount.Address)

		// ensure the validator doesn't exist already
		_, err := k.GetValidator(ctx, address)
		if err == nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator already exists"), nil, nil
		}

		denom, err := k.BondDenom(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "bond denom not found"), nil, err
		}

		balance := bk.GetBalance(ctx, simAccount.Address, denom).Amount
		if !balance.IsPositive() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "balance is negative"), nil, nil
		}

		amount, err := simtypes.RandPositiveInt(r, balance)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate positive amount"), nil, err
		}

		selfDelegation := sdk.NewCoin(denom, amount)

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		var fees sdk.Coins

		coins, hasNeg := spendable.SafeSub(selfDelegation)
		if !hasNeg {
			fees, err = simtypes.RandomFees(r, ctx, coins)
			if err != nil {
				return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate fees"), nil, err
			}
		}

		description := types.NewDescription(
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
		)

		maxCommission := math.LegacyNewDecWithPrec(int64(simtypes.RandIntBetween(r, 0, 100)), 2)
		commission := types.NewCommissionRates(
			simtypes.RandomDecAmount(r, maxCommission),
			maxCommission,
			simtypes.RandomDecAmount(r, maxCommission),
		)

		msg, err := types.NewMsgCreateValidator(address.String(), simAccount.ConsKey.PubKey(), selfDelegation, description, commission, math.OneInt())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(msg), "unable to create CreateValidator message"), nil, err
		}

		txCtx := simulation.OperationInput{
			R:             r,
			App:           app,
			TxGen:         txGen,
			Cdc:           nil,
			Msg:           msg,
			Context:       ctx,
			SimAccount:    simAccount,
			AccountKeeper: ak,
			ModuleName:    types.ModuleName,
		}

		return simulation.GenAndDeliverTx(txCtx, fees)
	}
}

// SimulateMsgEditValidator generates a MsgEditValidator with random values
func SimulateMsgEditValidator(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgEditValidator{})

		vals, err := k.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validators"), nil, err
		}

		if len(vals) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "number of validators equal zero"), nil, nil
		}

		val, ok := testutil.RandSliceElem(r, vals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to pick a validator"), nil, nil
		}

		address := val.GetOperator()
		newCommissionRate := simtypes.RandomDecAmount(r, val.Commission.MaxRate)

		if err := val.Commission.ValidateNewRate(newCommissionRate, ctx.BlockHeader().Time); err != nil {
			// skip as the commission is invalid
			return simtypes.NoOpMsg(types.ModuleName, msgType, "invalid commission rate"), nil, nil
		}

		bz, err := k.ValidatorAddressCodec().StringToBytes(val.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator address bytes"), nil, err
		}

		simAccount, found := simtypes.FindAccount(accs, sdk.AccAddress(bz))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to find account"), nil, fmt.Errorf("validator %s not found", val.GetOperator())
		}

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		description := types.NewDescription(
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
			simtypes.RandStringOfLength(r, 10),
		)

		msg := types.NewMsgEditValidator(address, description, &newCommissionRate, nil)

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txGen,
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

// SimulateMsgDelegate generates a MsgDelegate with random values
func SimulateMsgDelegate(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgDelegate{})
		denom, err := k.BondDenom(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "bond denom not found"), nil, err
		}

		vals, err := k.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validators"), nil, err
		}

		if len(vals) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "number of validators equal zero"), nil, nil
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)
		val, ok := testutil.RandSliceElem(r, vals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to pick a validator"), nil, nil
		}

		if val.InvalidExRate() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator's invalid echange rate"), nil, nil
		}

		amount := bk.GetBalance(ctx, simAccount.Address, denom).Amount
		if !amount.IsPositive() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "balance is negative"), nil, nil
		}

		amount, err = simtypes.RandPositiveInt(r, amount)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate positive amount"), nil, err
		}

		bondAmt := sdk.NewCoin(denom, amount)

		account := ak.GetAccount(ctx, simAccount.Address)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		var fees sdk.Coins

		coins, hasNeg := spendable.SafeSub(bondAmt)
		if !hasNeg {
			fees, err = simtypes.RandomFees(r, ctx, coins)
			if err != nil {
				return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate fees"), nil, err
			}
		}

		msg := types.NewMsgDelegate(simAccount.Address.String(), val.GetOperator(), bondAmt)

		txCtx := simulation.OperationInput{
			R:             r,
			App:           app,
			TxGen:         txGen,
			Cdc:           nil,
			Msg:           msg,
			Context:       ctx,
			SimAccount:    simAccount,
			AccountKeeper: ak,
			ModuleName:    types.ModuleName,
		}

		return simulation.GenAndDeliverTx(txCtx, fees)
	}
}

// SimulateMsgUndelegate generates a MsgUndelegate with random values
func SimulateMsgUndelegate(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgUndelegate{})

		vals, err := k.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validators"), nil, err
		}

		if len(vals) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "number of validators equal zero"), nil, nil
		}

		val, ok := testutil.RandSliceElem(r, vals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator is not ok"), nil, nil
		}

		valAddr, err := k.ValidatorAddressCodec().StringToBytes(val.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator address bytes"), nil, err
		}
		delegations, err := k.GetValidatorDelegations(ctx, valAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator delegations"), nil, nil
		}

		if delegations == nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "keeper does have any delegation entries"), nil, nil
		}

		// get random delegator from validator
		delegation := delegations[r.Intn(len(delegations))]
		delAddr := delegation.GetDelegatorAddr()

		delAddrBz, err := ak.AddressCodec().StringToBytes(delAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting delegator address bytes"), nil, err
		}

		hasMaxUD, err := k.HasMaxUnbondingDelegationEntries(ctx, delAddrBz, valAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting max unbonding delegation entries"), nil, err
		}

		if hasMaxUD {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "keeper does have a max unbonding delegation entries"), nil, nil
		}

		totalBond := val.TokensFromShares(delegation.GetShares()).TruncateInt()
		if !totalBond.IsPositive() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "total bond is negative"), nil, nil
		}

		unbondAmt, err := simtypes.RandPositiveInt(r, totalBond)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "invalid unbond amount"), nil, err
		}

		if unbondAmt.IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unbond amount is zero"), nil, nil
		}

		bondDenom, err := k.BondDenom(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "bond denom not found"), nil, err
		}

		msg := types.NewMsgUndelegate(
			delAddr, val.GetOperator(), sdk.NewCoin(bondDenom, unbondAmt),
		)

		// need to retrieve the simulation account associated with delegation to retrieve PrivKey
		var simAccount simtypes.Account

		for _, simAcc := range accs {
			if simAcc.Address.Equals(sdk.AccAddress(delAddrBz)) {
				simAccount = simAcc
				break
			}
		}
		// if simaccount.PrivKey == nil, delegation address does not exist in accs. However, since smart contracts and module accounts can stake, we can ignore the error
		if simAccount.PrivKey == nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(msg), "account private key is nil"), nil, nil
		}

		account := ak.GetAccount(ctx, delAddrBz)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txGen,
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

// SimulateMsgCancelUnbondingDelegate generates a MsgCancelUnbondingDelegate with random values
func SimulateMsgCancelUnbondingDelegate(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgCancelUnbondingDelegation{})

		vals, err := k.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validators"), nil, err
		}

		if len(vals) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "number of validators equal zero"), nil, nil
		}

		simAccount, _ := simtypes.RandomAcc(r, accs)
		val, ok := testutil.RandSliceElem(r, vals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator is not ok"), nil, nil
		}

		if val.IsJailed() || val.InvalidExRate() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator is jailed"), nil, nil
		}

		valAddr, err := k.ValidatorAddressCodec().StringToBytes(val.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator address bytes"), nil, err
		}
		unbondingDelegation, err := k.GetUnbondingDelegation(ctx, simAccount.Address, valAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "account does have any unbonding delegation"), nil, nil
		}

		// This is a temporary fix to make staking simulation pass. We should fetch
		// the first unbondingDelegationEntry that matches the creationHeight, because
		// currently the staking msgServer chooses the first unbondingDelegationEntry
		// with the matching creationHeight.
		//
		// ref: https://github.com/cosmos/cosmos-sdk/issues/12932
		creationHeight := unbondingDelegation.Entries[r.Intn(len(unbondingDelegation.Entries))].CreationHeight

		var unbondingDelegationEntry types.UnbondingDelegationEntry

		for _, entry := range unbondingDelegation.Entries {
			if entry.CreationHeight == creationHeight {
				unbondingDelegationEntry = entry
				break
			}
		}

		if unbondingDelegationEntry.CompletionTime.Before(ctx.BlockTime()) {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unbonding delegation is already processed"), nil, nil
		}

		if !unbondingDelegationEntry.Balance.IsPositive() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "delegator receiving balance is negative"), nil, nil
		}

		cancelBondAmt := simtypes.RandomAmount(r, unbondingDelegationEntry.Balance)

		if cancelBondAmt.IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "cancelBondAmt amount is zero"), nil, nil
		}

		bondDenom, err := k.BondDenom(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "bond denom not found"), nil, err
		}

		msg := types.NewMsgCancelUnbondingDelegation(
			simAccount.Address.String(), val.GetOperator(), unbondingDelegationEntry.CreationHeight, sdk.NewCoin(bondDenom, cancelBondAmt),
		)

		spendable := bk.SpendableCoins(ctx, simAccount.Address)

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txGen,
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

// SimulateMsgBeginRedelegate generates a MsgBeginRedelegate with random values
func SimulateMsgBeginRedelegate(
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k *keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgBeginRedelegate{})

		allVals, err := k.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validators"), nil, err
		}

		if len(allVals) == 0 {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "number of validators equal zero"), nil, nil
		}

		srcVal, ok := testutil.RandSliceElem(r, allVals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to pick validator"), nil, nil
		}

		srcAddr, err := k.ValidatorAddressCodec().StringToBytes(srcVal.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator address bytes"), nil, err
		}
		delegations, err := k.GetValidatorDelegations(ctx, srcAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator delegations"), nil, nil
		}

		if delegations == nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "keeper does have any delegation entries"), nil, nil
		}

		// get random delegator from src validator
		delegation := delegations[r.Intn(len(delegations))]
		delAddr := delegation.GetDelegatorAddr()

		delAddrBz, err := ak.AddressCodec().StringToBytes(delAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting delegator address bytes"), nil, err
		}

		hasRecRedel, err := k.HasReceivingRedelegation(ctx, delAddrBz, srcAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting receiving redelegation"), nil, err
		}

		if hasRecRedel {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "receveing redelegation is not allowed"), nil, nil // skip
		}

		// get random destination validator
		destVal, ok := testutil.RandSliceElem(r, allVals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to pick validator"), nil, nil
		}

		destAddr, err := k.ValidatorAddressCodec().StringToBytes(destVal.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting validator address bytes"), nil, err
		}
		hasMaxRedel, err := k.HasMaxRedelegationEntries(ctx, delAddrBz, srcAddr, destAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "error getting max redelegation entries"), nil, err
		}

		if bytes.Equal(srcAddr, destAddr) || destVal.InvalidExRate() || hasMaxRedel {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "checks failed"), nil, nil
		}

		totalBond := srcVal.TokensFromShares(delegation.GetShares()).TruncateInt()
		if !totalBond.IsPositive() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "total bond is negative"), nil, nil
		}

		redAmt, err := simtypes.RandPositiveInt(r, totalBond)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate positive amount"), nil, err
		}

		if redAmt.IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "amount is zero"), nil, nil
		}

		// check if the shares truncate to zero
		shares, err := srcVal.SharesFromTokens(redAmt)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "invalid shares"), nil, err
		}

		if srcVal.TokensFromShares(shares).TruncateInt().IsZero() {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "shares truncate to zero"), nil, nil // skip
		}

		// need to retrieve the simulation account associated with delegation to retrieve PrivKey
		var simAccount simtypes.Account

		for _, simAcc := range accs {
			if simAcc.Address.Equals(sdk.AccAddress(delAddrBz)) {
				simAccount = simAcc
				break
			}
		}

		// if simaccount.PrivKey == nil, delegation address does not exist in accs. However, since smart contracts and module accounts can stake, we can ignore the error
		if simAccount.PrivKey == nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "account private key is nil"), nil, nil
		}

		account := ak.GetAccount(ctx, delAddrBz)
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		bondDenom, err := k.BondDenom(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "bond denom not found"), nil, err
		}

		msg := types.NewMsgBeginRedelegate(
			delAddr, srcVal.GetOperator(), destVal.GetOperator(),
			sdk.NewCoin(bondDenom, redAmt),
		)

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txGen,
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
