package simulation

import (
	"context"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/baseapp"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	appparams "union/app/params"
	"union/x/deferredack/types"
)

// Simulation operation weights constants
//
//nolint:gosec
const (
	OpWeightMsgWriteDeferredAck = "op_weight_write_deferred_ack"
)

type AccountKeeper interface {
	GetModuleAccount(ctx context.Context, moduleName string) sdk.ModuleAccountI
	GetAccount(ctx context.Context, addr sdk.AccAddress) sdk.AccountI
}

type BankKeeper interface {
	simulation.BankKeeper
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
}

type DeferredAckKeeper interface {
	GetParams(ctx sdk.Context) (params types.Params)
}

func WeightedOperations(
	simstate *module.SimulationState,
	daKeeper DeferredAckKeeper,
	ak AccountKeeper,
	bk BankKeeper,
) simulation.WeightedOperations {
	var (
		weightMsgWriteDeferredAck int
	)

	simstate.AppParams.GetOrGenerate(OpWeightMsgWriteDeferredAck, &weightMsgWriteDeferredAck, nil,
		func(_ *rand.Rand) {
			weightMsgWriteDeferredAck = appparams.DefaultWeightMsgCreateDenom
		},
	)
	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgWriteDeferredAck,
			SimulateMsgWriteDeferredAck(
				daKeeper,
				ak,
				bk,
			),
		),
	}
}

func SimulateMsgWriteDeferredAck(
	keeper DeferredAckKeeper,
	ak AccountKeeper,
	bk BankKeeper,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// TODO: Simulate ack packet
		msg := types.MsgWriteDeferredAck{}

		account, _ := simtypes.RandomAcc(r, accs)

		txCtx := BuildOperationInput(r, app, ctx, &msg, account, ak, bk, nil)

		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// BuildOperationInput helper to build object
func BuildOperationInput(
	r *rand.Rand,
	app *baseapp.BaseApp,
	ctx sdk.Context,
	msg interface {
		sdk.Msg
	},
	simAccount simtypes.Account,
	ak AccountKeeper,
	bk BankKeeper,
	deposit sdk.Coins,
) simulation.OperationInput {
	return simulation.OperationInput{
		R:               r,
		App:             app,
		TxGen:           appparams.MakeEncodingConfig().TxConfig,
		Cdc:             nil,
		Msg:             msg,
		Context:         ctx,
		SimAccount:      simAccount,
		AccountKeeper:   ak,
		Bankkeeper:      bk,
		ModuleName:      types.ModuleName,
		CoinsSpentInMsg: deposit,
	}
}
