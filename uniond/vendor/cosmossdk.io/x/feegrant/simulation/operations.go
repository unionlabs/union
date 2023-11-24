package simulation

import (
	"math/rand"

	"cosmossdk.io/core/address"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/keeper"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

// Simulation operation weights constants
const (
	OpWeightMsgGrantAllowance        = "op_weight_msg_grant_fee_allowance"
	OpWeightMsgRevokeAllowance       = "op_weight_msg_grant_revoke_allowance"
	DefaultWeightGrantAllowance  int = 100
	DefaultWeightRevokeAllowance int = 100
)

var (
	TypeMsgGrantAllowance  = sdk.MsgTypeURL(&feegrant.MsgGrantAllowance{})
	TypeMsgRevokeAllowance = sdk.MsgTypeURL(&feegrant.MsgRevokeAllowance{})
)

func WeightedOperations(
	registry codectypes.InterfaceRegistry,
	appParams simtypes.AppParams,
	cdc codec.JSONCodec,
	txConfig client.TxConfig,
	ak feegrant.AccountKeeper,
	bk feegrant.BankKeeper,
	k keeper.Keeper,
	ac address.Codec,
) simulation.WeightedOperations {
	var (
		weightMsgGrantAllowance  int
		weightMsgRevokeAllowance int
	)

	appParams.GetOrGenerate(OpWeightMsgGrantAllowance, &weightMsgGrantAllowance, nil,
		func(_ *rand.Rand) {
			weightMsgGrantAllowance = DefaultWeightGrantAllowance
		},
	)

	appParams.GetOrGenerate(OpWeightMsgRevokeAllowance, &weightMsgRevokeAllowance, nil,
		func(_ *rand.Rand) {
			weightMsgRevokeAllowance = DefaultWeightRevokeAllowance
		},
	)

	pCdc := codec.NewProtoCodec(registry)

	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgGrantAllowance,
			SimulateMsgGrantAllowance(pCdc, txConfig, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightMsgRevokeAllowance,
			SimulateMsgRevokeAllowance(pCdc, txConfig, ak, bk, k, ac),
		),
	}
}

// SimulateMsgGrantAllowance generates MsgGrantAllowance with random values.
func SimulateMsgGrantAllowance(
	cdc *codec.ProtoCodec,
	txConfig client.TxConfig,
	ak feegrant.AccountKeeper,
	bk feegrant.BankKeeper,
	k keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		granter, _ := simtypes.RandomAcc(r, accs)
		grantee, _ := simtypes.RandomAcc(r, accs)
		if grantee.Address.String() == granter.Address.String() {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgGrantAllowance, "grantee and granter cannot be same"), nil, nil
		}

		if f, _ := k.GetAllowance(ctx, granter.Address, grantee.Address); f != nil {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgGrantAllowance, "fee allowance exists"), nil, nil
		}

		account := ak.GetAccount(ctx, granter.Address)

		spendableCoins := bk.SpendableCoins(ctx, account.GetAddress())
		if spendableCoins.Empty() {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgGrantAllowance, "unable to grant empty coins as SpendLimit"), nil, nil
		}

		oneYear := ctx.BlockTime().AddDate(1, 0, 0)
		msg, err := feegrant.NewMsgGrantAllowance(&feegrant.BasicAllowance{
			SpendLimit: spendableCoins,
			Expiration: &oneYear,
		}, granter.Address, grantee.Address)
		if err != nil {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgGrantAllowance, err.Error()), nil, err
		}

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txConfig,
			Cdc:             nil,
			Msg:             msg,
			Context:         ctx,
			SimAccount:      granter,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      feegrant.ModuleName,
			CoinsSpentInMsg: spendableCoins,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// SimulateMsgRevokeAllowance generates a MsgRevokeAllowance with random values.
func SimulateMsgRevokeAllowance(
	cdc *codec.ProtoCodec,
	txConfig client.TxConfig,
	ak feegrant.AccountKeeper,
	bk feegrant.BankKeeper,
	k keeper.Keeper,
	ac address.Codec,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		hasGrant := false
		var granterAddr sdk.AccAddress
		var granteeAddr sdk.AccAddress
		k.IterateAllFeeAllowances(ctx, func(grant feegrant.Grant) bool {
			granter, err := ac.StringToBytes(grant.Granter)
			if err != nil {
				panic(err)
			}
			grantee, err := ac.StringToBytes(grant.Grantee)
			if err != nil {
				panic(err)
			}
			granterAddr = granter
			granteeAddr = grantee
			hasGrant = true
			return true
		})

		if !hasGrant {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgRevokeAllowance, "no grants"), nil, nil
		}
		granter, ok := simtypes.FindAccount(accs, granterAddr)

		if !ok {
			return simtypes.NoOpMsg(feegrant.ModuleName, TypeMsgRevokeAllowance, "Account not found"), nil, nil
		}

		account := ak.GetAccount(ctx, granter.Address)
		spendableCoins := bk.SpendableCoins(ctx, account.GetAddress())

		msg := feegrant.NewMsgRevokeAllowance(granterAddr, granteeAddr)

		txCtx := simulation.OperationInput{
			R:               r,
			App:             app,
			TxGen:           txConfig,
			Cdc:             nil,
			Msg:             &msg,
			Context:         ctx,
			SimAccount:      granter,
			AccountKeeper:   ak,
			Bankkeeper:      bk,
			ModuleName:      feegrant.ModuleName,
			CoinsSpentInMsg: spendableCoins,
		}

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}
