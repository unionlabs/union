package simulation

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"testing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
)

func getTestingMode(tb testing.TB) (testingMode bool, t *testing.T, b *testing.B) {
	tb.Helper()
	testingMode = false

	if _t, ok := tb.(*testing.T); ok {
		t = _t
		testingMode = true
	} else {
		b = tb.(*testing.B)
	}

	return testingMode, t, b
}

// getBlockSize returns a block size as determined from the transition matrix.
// It targets making average block size the provided parameter. The three
// states it moves between are:
//   - "over stuffed" blocks with average size of 2 * avgblocksize,
//   - normal sized blocks, hitting avgBlocksize on average,
//   - and empty blocks, with no txs / only txs scheduled from the past.
func getBlockSize(r *rand.Rand, params Params, lastBlockSizeState, avgBlockSize int) (state, blockSize int) {
	// TODO: Make default blocksize transition matrix actually make the average
	// blocksize equal to avgBlockSize.
	state = params.BlockSizeTransitionMatrix().NextState(r, lastBlockSizeState)

	switch state {
	case 0:
		blockSize = r.Intn(avgBlockSize * 4)

	case 1:
		blockSize = r.Intn(avgBlockSize * 2)

	default:
		blockSize = 0
	}

	return state, blockSize
}

func mustMarshalJSONIndent(o interface{}) []byte {
	bz, err := json.MarshalIndent(o, "", "  ")
	if err != nil {
		panic(fmt.Sprintf("failed to JSON encode: %s", err))
	}

	return bz
}

// OperationInput is a struct that holds all the needed values to generate a tx and deliver it
type OperationInput struct {
	R               *rand.Rand
	App             simtypes.AppEntrypoint
	TxGen           client.TxConfig
	Cdc             *codec.ProtoCodec
	Msg             sdk.Msg
	CoinsSpentInMsg sdk.Coins
	Context         sdk.Context
	SimAccount      simtypes.Account
	AccountKeeper   AccountKeeper
	Bankkeeper      BankKeeper
	ModuleName      string
}

// GenAndDeliverTxWithRandFees generates a transaction with a random fee and delivers it.
func GenAndDeliverTxWithRandFees(txCtx OperationInput) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
	account := txCtx.AccountKeeper.GetAccount(txCtx.Context, txCtx.SimAccount.Address)
	spendable := txCtx.Bankkeeper.SpendableCoins(txCtx.Context, account.GetAddress())

	var fees sdk.Coins
	var err error

	coins, hasNeg := spendable.SafeSub(txCtx.CoinsSpentInMsg...)
	if hasNeg {
		return simtypes.NoOpMsg(txCtx.ModuleName, sdk.MsgTypeURL(txCtx.Msg), "message doesn't leave room for fees"), nil, err
	}

	fees, err = simtypes.RandomFees(txCtx.R, coins)
	if err != nil {
		return simtypes.NoOpMsg(txCtx.ModuleName, sdk.MsgTypeURL(txCtx.Msg), "unable to generate fees"), nil, err
	}
	return GenAndDeliverTx(txCtx, fees)
}

// GenAndDeliverTx generates a transactions and delivers it.
func GenAndDeliverTx(txCtx OperationInput, fees sdk.Coins) (simtypes.OperationMsg, []simtypes.FutureOperation, error) {
	account := txCtx.AccountKeeper.GetAccount(txCtx.Context, txCtx.SimAccount.Address)
	tx, err := simtestutil.GenSignedMockTx(
		txCtx.R,
		txCtx.TxGen,
		[]sdk.Msg{txCtx.Msg},
		fees,
		simtestutil.DefaultGenTxGas,
		txCtx.Context.ChainID(),
		[]uint64{account.GetAccountNumber()},
		[]uint64{account.GetSequence()},
		txCtx.SimAccount.PrivKey,
	)
	if err != nil {
		return simtypes.NoOpMsg(txCtx.ModuleName, sdk.MsgTypeURL(txCtx.Msg), "unable to generate mock tx"), nil, err
	}

	_, _, err = txCtx.App.SimDeliver(txCtx.TxGen.TxEncoder(), tx)
	if err != nil {
		return simtypes.NoOpMsg(txCtx.ModuleName, sdk.MsgTypeURL(txCtx.Msg), "unable to deliver tx"), nil, err
	}

	return simtypes.NewOperationMsg(txCtx.Msg, true, ""), nil, nil
}
