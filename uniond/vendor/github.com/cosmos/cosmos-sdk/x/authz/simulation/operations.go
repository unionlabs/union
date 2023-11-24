package simulation

import (
	"math/rand"
	"time"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cdctypes "github.com/cosmos/cosmos-sdk/codec/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/authz"
	"github.com/cosmos/cosmos-sdk/x/authz/keeper"
	banktype "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/simulation"
)

// authz message types
var (
	TypeMsgGrant  = sdk.MsgTypeURL(&authz.MsgGrant{})
	TypeMsgRevoke = sdk.MsgTypeURL(&authz.MsgRevoke{})
	TypeMsgExec   = sdk.MsgTypeURL(&authz.MsgExec{})
)

// Simulation operation weights constants
const (
	OpWeightMsgGrant = "op_weight_msg_grant"
	OpWeightRevoke   = "op_weight_msg_revoke"
	OpWeightExec     = "op_weight_msg_execute"
)

// authz operations weights
const (
	WeightGrant  = 100
	WeightRevoke = 90
	WeightExec   = 90
)

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(
	registry cdctypes.InterfaceRegistry,
	appParams simtypes.AppParams,
	cdc codec.JSONCodec,
	txGen client.TxConfig,
	ak authz.AccountKeeper,
	bk authz.BankKeeper,
	k keeper.Keeper,
) simulation.WeightedOperations {
	var (
		weightMsgGrant int
		weightExec     int
		weightRevoke   int
	)

	appParams.GetOrGenerate(OpWeightMsgGrant, &weightMsgGrant, nil, func(_ *rand.Rand) {
		weightMsgGrant = WeightGrant
	})

	appParams.GetOrGenerate(OpWeightExec, &weightExec, nil, func(_ *rand.Rand) {
		weightExec = WeightExec
	})

	appParams.GetOrGenerate(OpWeightRevoke, &weightRevoke, nil, func(_ *rand.Rand) {
		weightRevoke = WeightRevoke
	})

	pCdc := codec.NewProtoCodec(registry)

	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgGrant,
			SimulateMsgGrant(pCdc, txGen, ak, bk, k),
		),
		simulation.NewWeightedOperation(
			weightExec,
			SimulateMsgExec(pCdc, txGen, ak, bk, k, registry),
		),
		simulation.NewWeightedOperation(
			weightRevoke,
			SimulateMsgRevoke(pCdc, txGen, ak, bk, k),
		),
	}
}

// SimulateMsgGrant generates a MsgGrant with random values.
func SimulateMsgGrant(
	cdc *codec.ProtoCodec,
	txCfg client.TxConfig,
	ak authz.AccountKeeper,
	bk authz.BankKeeper,
	_ keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		granter, _ := simtypes.RandomAcc(r, accs)
		grantee, _ := simtypes.RandomAcc(r, accs)

		if granter.Address.Equals(grantee.Address) {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgGrant, "granter and grantee are same"), nil, nil
		}

		granterAcc := ak.GetAccount(ctx, granter.Address)
		spendableCoins := bk.SpendableCoins(ctx, granter.Address)
		fees, err := simtypes.RandomFees(r, ctx, spendableCoins)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgGrant, err.Error()), nil, err
		}

		spendLimit := spendableCoins.Sub(fees...)
		if len(spendLimit) == 0 {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgGrant, "spend limit is nil"), nil, nil
		}

		var expiration *time.Time
		t1 := simtypes.RandTimestamp(r)
		if !t1.Before(ctx.BlockTime()) {
			expiration = &t1
		}
		randomAuthz := generateRandomAuthorization(r, spendLimit)

		msg, err := authz.NewMsgGrant(granter.Address, grantee.Address, randomAuthz, expiration)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgGrant, err.Error()), nil, err
		}
		tx, err := simtestutil.GenSignedMockTx(
			r,
			txCfg,
			[]sdk.Msg{msg},
			fees,
			simtestutil.DefaultGenTxGas,
			chainID,
			[]uint64{granterAcc.GetAccountNumber()},
			[]uint64{granterAcc.GetSequence()},
			granter.PrivKey,
		)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgGrant, "unable to generate mock tx"), nil, err
		}

		_, _, err = app.SimTxFinalizeBlock(txCfg.TxEncoder(), tx)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, sdk.MsgTypeURL(msg), "unable to deliver tx"), nil, err
		}
		return simtypes.NewOperationMsg(msg, true, ""), nil, err
	}
}

func generateRandomAuthorization(r *rand.Rand, spendLimit sdk.Coins) authz.Authorization {
	authorizations := make([]authz.Authorization, 2)
	sendAuthz := banktype.NewSendAuthorization(spendLimit, nil)
	authorizations[0] = sendAuthz
	authorizations[1] = authz.NewGenericAuthorization(sdk.MsgTypeURL(&banktype.MsgSend{}))

	return authorizations[r.Intn(len(authorizations))]
}

// SimulateMsgRevoke generates a MsgRevoke with random values.
func SimulateMsgRevoke(
	cdc *codec.ProtoCodec,
	txCfg client.TxConfig,
	ak authz.AccountKeeper,
	bk authz.BankKeeper,
	k keeper.Keeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		var granterAddr, granteeAddr sdk.AccAddress
		var grant authz.Grant
		hasGrant := false

		k.IterateGrants(ctx, func(granter, grantee sdk.AccAddress, g authz.Grant) bool {
			grant = g
			granterAddr = granter
			granteeAddr = grantee
			hasGrant = true
			return true
		})

		if !hasGrant {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "no grants"), nil, nil
		}

		granterAcc, ok := simtypes.FindAccount(accs, granterAddr)
		if !ok {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "account not found"), nil, sdkerrors.ErrNotFound.Wrapf("account not found")
		}

		spendableCoins := bk.SpendableCoins(ctx, granterAddr)
		fees, err := simtypes.RandomFees(r, ctx, spendableCoins)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "fee error"), nil, err
		}

		a, err := grant.GetAuthorization()
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "authorization error"), nil, err
		}

		msg := authz.NewMsgRevoke(granterAddr, granteeAddr, a.MsgTypeURL())
		account := ak.GetAccount(ctx, granterAddr)
		tx, err := simtestutil.GenSignedMockTx(
			r,
			txCfg,
			[]sdk.Msg{&msg},
			fees,
			simtestutil.DefaultGenTxGas,
			chainID,
			[]uint64{account.GetAccountNumber()},
			[]uint64{account.GetSequence()},
			granterAcc.PrivKey,
		)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, err.Error()), nil, err
		}

		_, _, err = app.SimTxFinalizeBlock(txCfg.TxEncoder(), tx)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "unable to execute tx: "+err.Error()), nil, err
		}

		return simtypes.NewOperationMsg(&msg, true, ""), nil, nil
	}
}

// SimulateMsgExec generates a MsgExec with random values.
func SimulateMsgExec(
	cdc *codec.ProtoCodec,
	txCfg client.TxConfig,
	ak authz.AccountKeeper,
	bk authz.BankKeeper,
	k keeper.Keeper,
	unpacker cdctypes.AnyUnpacker,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context, accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		var granterAddr sdk.AccAddress
		var granteeAddr sdk.AccAddress
		var sendAuth *banktype.SendAuthorization
		var err error
		k.IterateGrants(ctx, func(granter, grantee sdk.AccAddress, grant authz.Grant) bool {
			granterAddr = granter
			granteeAddr = grantee
			var a authz.Authorization
			a, err = grant.GetAuthorization()
			if err != nil {
				return true
			}
			var ok bool
			sendAuth, ok = a.(*banktype.SendAuthorization)
			return ok
		})

		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, err
		}
		if sendAuth == nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, "no grant found"), nil, nil
		}

		grantee, ok := simtypes.FindAccount(accs, granteeAddr)
		if !ok {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "Account not found"), nil, sdkerrors.ErrNotFound.Wrapf("grantee account not found")
		}

		if _, ok := simtypes.FindAccount(accs, granterAddr); !ok {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgRevoke, "Account not found"), nil, sdkerrors.ErrNotFound.Wrapf("granter account not found")
		}

		granterspendableCoins := bk.SpendableCoins(ctx, granterAddr)
		coins := simtypes.RandSubsetCoins(r, granterspendableCoins)
		// if coins slice is empty, we can not create valid banktype.MsgSend
		if len(coins) == 0 {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, "empty coins slice"), nil, nil
		}

		// Check send_enabled status of each sent coin denom
		if err := bk.IsSendEnabledCoins(ctx, coins...); err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, nil
		}

		msg := []sdk.Msg{banktype.NewMsgSend(granterAddr, granteeAddr, coins)}

		_, err = sendAuth.Accept(ctx, msg[0])
		if err != nil {
			if sdkerrors.ErrInsufficientFunds.Is(err) {
				return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, nil
			}
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, err

		}

		msgExec := authz.NewMsgExec(granteeAddr, msg)
		granteeSpendableCoins := bk.SpendableCoins(ctx, granteeAddr)
		fees, err := simtypes.RandomFees(r, ctx, granteeSpendableCoins)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, "fee error"), nil, err
		}

		granteeAcc := ak.GetAccount(ctx, granteeAddr)
		tx, err := simtestutil.GenSignedMockTx(
			r,
			txCfg,
			[]sdk.Msg{&msgExec},
			fees,
			simtestutil.DefaultGenTxGas,
			chainID,
			[]uint64{granteeAcc.GetAccountNumber()},
			[]uint64{granteeAcc.GetSequence()},
			grantee.PrivKey,
		)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, err
		}

		_, _, err = app.SimTxFinalizeBlock(txCfg.TxEncoder(), tx)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, err.Error()), nil, err
		}

		err = msgExec.UnpackInterfaces(unpacker)
		if err != nil {
			return simtypes.NoOpMsg(authz.ModuleName, TypeMsgExec, "unmarshal error"), nil, err
		}
		return simtypes.NewOperationMsg(&msgExec, true, "success"), nil, nil
	}
}
