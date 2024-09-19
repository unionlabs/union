package simulation

import (
	"context"
	"math/rand"

	"cosmossdk.io/math"
	"cosmossdk.io/store"

	"github.com/cosmos/cosmos-sdk/baseapp"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	banktypes "cosmossdk.io/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/simulation"

	appparams "union/app/params"
	"union/x/tokenfactory/types"
)

// Simulation operation weights constants
//
//nolint:gosec
const (
	OpWeightMsgCreateDenom      = "op_weight_msg_create_denom"
	OpWeightMsgMint             = "op_weight_msg_mint"
	OpWeightMsgBurn             = "op_weight_msg_burn"
	OpWeightMsgChangeAdmin      = "op_weight_msg_change_admin"
	OpWeightMsgSetDenomMetadata = "op_weight_msg_set_denom_metadata"
)

type TokenfactoryKeeper interface {
	GetParams(ctx sdk.Context) (params types.Params)
	GetAuthorityMetadata(ctx sdk.Context, denom string) (types.DenomAuthorityMetadata, error)
	GetAllDenomsIterator(ctx sdk.Context) store.Iterator
	GetDenomsFromCreator(ctx sdk.Context, creator string) []string
}

type BankKeeper interface {
	simulation.BankKeeper
	GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
	GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
}

func WeightedOperations(
	simstate *module.SimulationState,
	tfKeeper TokenfactoryKeeper,
	ak types.AccountKeeper,
	bk BankKeeper,
) simulation.WeightedOperations {
	var (
		weightMsgCreateDenom      int
		weightMsgMint             int
		weightMsgBurn             int
		weightMsgChangeAdmin      int
		weightMsgSetDenomMetadata int
	)

	simstate.AppParams.GetOrGenerate(OpWeightMsgCreateDenom, &weightMsgCreateDenom, nil,
		func(_ *rand.Rand) {
			weightMsgCreateDenom = appparams.DefaultWeightMsgCreateDenom
		},
	)
	simstate.AppParams.GetOrGenerate(OpWeightMsgMint, &weightMsgMint, nil,
		func(_ *rand.Rand) {
			weightMsgMint = appparams.DefaultWeightMsgMint
		},
	)
	simstate.AppParams.GetOrGenerate(OpWeightMsgBurn, &weightMsgBurn, nil,
		func(_ *rand.Rand) {
			weightMsgBurn = appparams.DefaultWeightMsgBurn
		},
	)
	simstate.AppParams.GetOrGenerate(OpWeightMsgChangeAdmin, &weightMsgChangeAdmin, nil,
		func(_ *rand.Rand) {
			weightMsgChangeAdmin = appparams.DefaultWeightMsgChangeAdmin
		},
	)
	simstate.AppParams.GetOrGenerate(OpWeightMsgSetDenomMetadata, &weightMsgSetDenomMetadata, nil,
		func(_ *rand.Rand) {
			weightMsgSetDenomMetadata = appparams.DefaultWeightMsgSetDenomMetadata
		},
	)
	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgCreateDenom,
			SimulateMsgCreateDenom(
				tfKeeper,
				ak,
				bk,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgMint,
			SimulateMsgMint(
				tfKeeper,
				ak,
				bk,
				DefaultSimulationDenomSelector,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgBurn,
			SimulateMsgBurn(
				tfKeeper,
				ak,
				bk,
				DefaultSimulationDenomSelector,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgChangeAdmin,
			SimulateMsgChangeAdmin(
				tfKeeper,
				ak,
				bk,
				DefaultSimulationDenomSelector,
			),
		),
		simulation.NewWeightedOperation(
			weightMsgSetDenomMetadata,
			SimulateMsgSetDenomMetadata(
				tfKeeper,
				ak,
				bk,
				DefaultSimulationDenomSelector,
			),
		),
	}
}

type DenomSelector = func(*rand.Rand, sdk.Context, TokenfactoryKeeper, string) (string, bool)

func DefaultSimulationDenomSelector(r *rand.Rand, ctx sdk.Context, tfKeeper TokenfactoryKeeper, creator string) (string, bool) {
	denoms := tfKeeper.GetDenomsFromCreator(ctx, creator)
	if len(denoms) == 0 {
		return "", false
	}
	randPos := r.Intn(len(denoms))

	return denoms[randPos], true
}

func SimulateMsgSetDenomMetadata(
	tfKeeper TokenfactoryKeeper,
	ak types.AccountKeeper,
	bk BankKeeper,
	denomSelector DenomSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// Get create denom account
		createdDenomAccount, _ := simtypes.RandomAcc(r, accs)

		// Get demon
		denom, hasDenom := denomSelector(r, ctx, tfKeeper, createdDenomAccount.Address.String())
		if !hasDenom {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgSetDenomMetadata{}.Type(), "sim account have no denom created"), nil, nil
		}

		// Get admin of the denom
		authData, err := tfKeeper.GetAuthorityMetadata(ctx, denom)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgSetDenomMetadata{}.Type(), "err authority metadata"), nil, err
		}
		adminAccount, found := simtypes.FindAccount(accs, sdk.MustAccAddressFromBech32(authData.Admin))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgSetDenomMetadata{}.Type(), "admin account not found"), nil, nil
		}

		metadata := banktypes.Metadata{
			Description: simtypes.RandStringOfLength(r, 10),
			DenomUnits: []*banktypes.DenomUnit{{
				Denom:    denom,
				Exponent: 0,
			}},
			Base:    denom,
			Display: denom,
			Name:    simtypes.RandStringOfLength(r, 10),
			Symbol:  simtypes.RandStringOfLength(r, 10),
		}

		msg := types.MsgSetDenomMetadata{
			Sender:   adminAccount.Address.String(),
			Metadata: metadata,
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, adminAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

func SimulateMsgChangeAdmin(
	tfKeeper TokenfactoryKeeper,
	ak types.AccountKeeper,
	bk BankKeeper,
	denomSelector DenomSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// Get create denom account
		createdDenomAccount, _ := simtypes.RandomAcc(r, accs)

		// Get demon
		denom, hasDenom := denomSelector(r, ctx, tfKeeper, createdDenomAccount.Address.String())
		if !hasDenom {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgChangeAdmin{}.Type(), "sim account have no denom created"), nil, nil
		}

		// Get admin of the denom
		authData, err := tfKeeper.GetAuthorityMetadata(ctx, denom)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgChangeAdmin{}.Type(), "err authority metadata"), nil, err
		}
		curAdminAccount, found := simtypes.FindAccount(accs, sdk.MustAccAddressFromBech32(authData.Admin))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgChangeAdmin{}.Type(), "admin account not found"), nil, nil
		}

		// Rand new admin account
		newAdmin, _ := simtypes.RandomAcc(r, accs)
		if newAdmin.Address.String() == curAdminAccount.Address.String() {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgChangeAdmin{}.Type(), "new admin cannot be the same as current admin"), nil, nil
		}

		// Create msg
		msg := types.MsgChangeAdmin{
			Sender:   curAdminAccount.Address.String(),
			Denom:    denom,
			NewAdmin: newAdmin.Address.String(),
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, curAdminAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

func SimulateMsgBurn(
	tfKeeper TokenfactoryKeeper,
	ak types.AccountKeeper,
	bk BankKeeper,
	denomSelector DenomSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// Get create denom account
		createdDenomAccount, _ := simtypes.RandomAcc(r, accs)

		// Get demon
		denom, hasDenom := denomSelector(r, ctx, tfKeeper, createdDenomAccount.Address.String())
		if !hasDenom {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgBurn{}.Type(), "sim account have no denom created"), nil, nil
		}

		// Get admin of the denom
		authData, err := tfKeeper.GetAuthorityMetadata(ctx, denom)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgBurn{}.Type(), "err authority metadata"), nil, err
		}
		adminAccount, found := simtypes.FindAccount(accs, sdk.MustAccAddressFromBech32(authData.Admin))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgBurn{}.Type(), "admin account not found"), nil, nil
		}

		// Check if admin account balance = 0
		accountBalance := bk.GetBalance(ctx, adminAccount.Address, denom)
		if accountBalance.Amount.LTE(math.ZeroInt()) {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgBurn{}.Type(), "sim account have no balance"), nil, nil
		}

		// Rand burn amount
		amount, _ := simtypes.RandPositiveInt(r, accountBalance.Amount)
		burnAmount := sdk.NewCoin(denom, amount)

		// Create msg
		msg := types.MsgBurn{
			Sender: adminAccount.Address.String(),
			Amount: burnAmount,
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, adminAccount, ak, bk, sdk.NewCoins(burnAmount))
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// Simulate msg mint denom
func SimulateMsgMint(
	tfKeeper TokenfactoryKeeper,
	ak types.AccountKeeper,
	bk BankKeeper,
	denomSelector DenomSelector,
) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// Get create denom account
		createdDenomAccount, _ := simtypes.RandomAcc(r, accs)

		// Get demon
		denom, hasDenom := denomSelector(r, ctx, tfKeeper, createdDenomAccount.Address.String())
		if !hasDenom {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgMint{}.Type(), "sim account have no denom created"), nil, nil
		}

		// Get admin of the denom
		authData, err := tfKeeper.GetAuthorityMetadata(ctx, denom)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgMint{}.Type(), "err authority metadata"), nil, err
		}
		adminAccount, found := simtypes.FindAccount(accs, sdk.MustAccAddressFromBech32(authData.Admin))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgMint{}.Type(), "admin account not found"), nil, nil
		}

		// Rand mint amount
		mintAmount, _ := simtypes.RandPositiveInt(r, math.NewIntFromUint64(100_000_000))

		// Create msg mint
		msg := types.MsgMint{
			Sender: adminAccount.Address.String(),
			Amount: sdk.NewCoin(denom, mintAmount),
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, adminAccount, ak, bk, nil)
		return simulation.GenAndDeliverTxWithRandFees(txCtx)
	}
}

// Simulate msg create denom
func SimulateMsgCreateDenom(tfKeeper TokenfactoryKeeper, ak types.AccountKeeper, bk BankKeeper) simtypes.Operation {
	return func(
		r *rand.Rand,
		app *baseapp.BaseApp,
		ctx sdk.Context,
		accs []simtypes.Account,
		chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		// Get sims account
		simAccount, _ := simtypes.RandomAcc(r, accs)

		// Check if sims account enough create fee
		createFee := tfKeeper.GetParams(ctx).DenomCreationFee
		balances := bk.GetAllBalances(ctx, simAccount.Address)
		_, hasNeg := balances.SafeSub(createFee[0])
		if hasNeg {
			return simtypes.NoOpMsg(types.ModuleName, types.MsgCreateDenom{}.Type(), "Creator not enough creation fee"), nil, nil
		}

		// Create msg create denom
		msg := types.MsgCreateDenom{
			Sender:   simAccount.Address.String(),
			Subdenom: simtypes.RandStringOfLength(r, 10),
		}

		txCtx := BuildOperationInput(r, app, ctx, &msg, simAccount, ak, bk, createFee)
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
		Type() string
	},
	simAccount simtypes.Account,
	ak types.AccountKeeper,
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
