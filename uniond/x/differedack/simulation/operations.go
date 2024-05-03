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
	"union/x/differedack/types"
)

// Simulation operation weights constants
//
//nolint:gosec
const (
	OpWeightMsgWriteDifferedAck = "op_weight_write_differed_ack"
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

type DifferedAckKeeper interface {
	GetParams(ctx sdk.Context) (params types.Params)
}

func WeightedOperations(
	simstate *module.SimulationState,
	daKeeper DifferedAckKeeper,
	ak AccountKeeper,
	bk BankKeeper,
) simulation.WeightedOperations {
	var (
		weightMsgWriteDifferedAck int
	)

	simstate.AppParams.GetOrGenerate(OpWeightMsgWriteDifferedAck, &weightMsgWriteDifferedAck, nil,
		func(_ *rand.Rand) {
			weightMsgWriteDifferedAck = appparams.DefaultWeightMsgCreateDenom
		},
	)
	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgWriteDifferedAck,
			SimulateMsgWriteDifferedAck(
				daKeeper,
				ak,
				bk,
			),
		),
	}
}

func SimulateMsgWriteDifferedAck(
	keeper DifferedAckKeeper,
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
		msg := types.MsgWriteDifferedAck{}

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
