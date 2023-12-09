package simulation

import (
	"errors"
	"math/rand"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/x/simulation"
	"github.com/cosmos/cosmos-sdk/x/slashing/keeper"
	"github.com/cosmos/cosmos-sdk/x/slashing/types"
)

// Simulation operation weights constants
const (
	OpWeightMsgUnjail = "op_weight_msg_unjail"

	DefaultWeightMsgUnjail = 100
)

// WeightedOperations returns all the operations from the module with their respective weights
func WeightedOperations(
	registry codectypes.InterfaceRegistry,
	appParams simtypes.AppParams,
	cdc codec.JSONCodec,
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k keeper.Keeper,
	sk types.StakingKeeper,
) simulation.WeightedOperations {
	var weightMsgUnjail int
	appParams.GetOrGenerate(OpWeightMsgUnjail, &weightMsgUnjail, nil, func(_ *rand.Rand) {
		weightMsgUnjail = DefaultWeightMsgUnjail
	})

	return simulation.WeightedOperations{
		simulation.NewWeightedOperation(
			weightMsgUnjail,
			SimulateMsgUnjail(codec.NewProtoCodec(registry), txGen, ak, bk, k, sk),
		),
	}
}

// SimulateMsgUnjail generates a MsgUnjail with random values
func SimulateMsgUnjail(
	cdc *codec.ProtoCodec,
	txGen client.TxConfig,
	ak types.AccountKeeper,
	bk types.BankKeeper,
	k keeper.Keeper,
	sk types.StakingKeeper,
) simtypes.Operation {
	return func(
		r *rand.Rand, app *baseapp.BaseApp, ctx sdk.Context,
		accs []simtypes.Account, chainID string,
	) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
		msgType := sdk.MsgTypeURL(&types.MsgUnjail{})

		allVals, err := sk.GetAllValidators(ctx)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get all validators"), nil, err
		}

		validator, ok := testutil.RandSliceElem(r, allVals)
		if !ok {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator is not ok"), nil, nil // skip
		}

		bz, err := sk.ValidatorAddressCodec().StringToBytes(validator.GetOperator())
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to convert validator address to bytes"), nil, err
		}

		simAccount, found := simtypes.FindAccount(accs, sdk.AccAddress(bz))
		if !found {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to find account"), nil, nil // skip
		}

		if !validator.IsJailed() {
			// TODO: due to this condition this message is almost, if not always, skipped !
			return simtypes.NoOpMsg(types.ModuleName, msgType, "validator is not jailed"), nil, nil
		}

		consAddr, err := validator.GetConsAddr()
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get validator consensus key"), nil, err
		}
		info, err := k.GetValidatorSigningInfo(ctx, consAddr)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to find validator signing info"), nil, err // skip
		}

		selfDel, err := sk.Delegation(ctx, simAccount.Address, bz)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to get self delegation"), nil, err
		}

		if selfDel == nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "self delegation is nil"), nil, nil // skip
		}

		account := ak.GetAccount(ctx, sdk.AccAddress(bz))
		spendable := bk.SpendableCoins(ctx, account.GetAddress())

		fees, err := simtypes.RandomFees(r, ctx, spendable)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, msgType, "unable to generate fees"), nil, err
		}

		msg := types.NewMsgUnjail(validator.GetOperator())

		tx, err := simtestutil.GenSignedMockTx(
			r,
			txGen,
			[]sdk.Msg{msg},
			fees,
			simtestutil.DefaultGenTxGas,
			chainID,
			[]uint64{account.GetAccountNumber()},
			[]uint64{account.GetSequence()},
			simAccount.PrivKey,
		)
		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(msg), "unable to generate mock tx"), nil, err
		}

		_, res, err := app.SimDeliver(txGen.TxEncoder(), tx)

		// result should fail if:
		// - validator cannot be unjailed due to tombstone
		// - validator is still in jailed period
		// - self delegation too low
		if info.Tombstoned ||
			ctx.BlockHeader().Time.Before(info.JailedUntil) ||
			validator.TokensFromShares(selfDel.GetShares()).TruncateInt().LT(validator.GetMinSelfDelegation()) {
			if res != nil && err == nil {
				if info.Tombstoned {
					return simtypes.NewOperationMsg(msg, true, ""), nil, errors.New("validator should not have been unjailed if validator tombstoned")
				}
				if ctx.BlockHeader().Time.Before(info.JailedUntil) {
					return simtypes.NewOperationMsg(msg, true, ""), nil, errors.New("validator unjailed while validator still in jail period")
				}
				if validator.TokensFromShares(selfDel.GetShares()).TruncateInt().LT(validator.GetMinSelfDelegation()) {
					return simtypes.NewOperationMsg(msg, true, ""), nil, errors.New("validator unjailed even though self-delegation too low")
				}
			}
			// msg failed as expected
			return simtypes.NewOperationMsg(msg, false, ""), nil, nil
		}

		if err != nil {
			return simtypes.NoOpMsg(types.ModuleName, sdk.MsgTypeURL(msg), "unable to deliver tx"), nil, errors.New(res.Log)
		}

		return simtypes.NewOperationMsg(msg, true, ""), nil, nil
	}
}
