package bank_test

import (
	"math/rand"
	"testing"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

var moduleAccAddr = authtypes.NewModuleAddress(stakingtypes.BondedPoolName)

// GenSequenceOfTxs generates a set of signed transactions of messages, such
// that they differ only by having the sequence numbers incremented between
// every transaction.
func genSequenceOfTxs(txGen client.TxConfig,
	msgs []sdk.Msg,
	accNums []uint64,
	initSeqNums []uint64,
	numToGenerate int,
	priv ...cryptotypes.PrivKey,
) ([]sdk.Tx, error) {
	txs := make([]sdk.Tx, numToGenerate)
	var err error
	for i := 0; i < numToGenerate; i++ {
		txs[i], err = simtestutil.GenSignedMockTx(
			rand.New(rand.NewSource(time.Now().UnixNano())),
			txGen,
			msgs,
			sdk.Coins{sdk.NewInt64Coin(sdk.DefaultBondDenom, 0)},
			simtestutil.DefaultGenTxGas,
			"",
			accNums,
			initSeqNums,
			priv...,
		)
		if err != nil {
			break
		}

		for i := 0; i < len(initSeqNums); i++ {
			initSeqNums[i]++
		}
	}

	return txs, err
}

func BenchmarkOneBankSendTxPerBlock(b *testing.B) {
	// b.Skip("Skipping benchmark with buggy code reported at https://github.com/cosmos/cosmos-sdk/issues/10023")

	b.ReportAllocs()
	// Add an account at genesis
	acc := authtypes.BaseAccount{
		Address: addr1.String(),
	}

	// construct genesis state
	genAccs := []types.GenesisAccount{&acc}
	s := createTestSuite(&testing.T{}, genAccs)
	baseApp := s.App.BaseApp
	ctx := baseApp.NewContext(false, tmproto.Header{})

	// some value conceivably higher than the benchmarks would ever go
	require.NoError(b, testutil.FundAccount(s.BankKeeper, ctx, addr1, sdk.NewCoins(sdk.NewInt64Coin("foocoin", 100000000000))))

	baseApp.Commit()
	txGen := moduletestutil.MakeTestEncodingConfig().TxConfig

	// Precompute all txs
	txs, err := genSequenceOfTxs(txGen, []sdk.Msg{sendMsg1}, []uint64{0}, []uint64{uint64(0)}, b.N, priv1)
	require.NoError(b, err)
	b.ResetTimer()

	height := int64(3)

	// Run this with a profiler, so its easy to distinguish what time comes from
	// Committing, and what time comes from Check/Deliver Tx.
	for i := 0; i < b.N; i++ {
		baseApp.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: height}})
		_, _, err := baseApp.SimCheck(txGen.TxEncoder(), txs[i])
		if err != nil {
			panic("something is broken in checking transaction")
		}

		_, _, err = baseApp.SimDeliver(txGen.TxEncoder(), txs[i])
		require.NoError(b, err)
		baseApp.EndBlock(abci.RequestEndBlock{Height: height})
		baseApp.Commit()
		height++
	}
}

func BenchmarkOneBankMultiSendTxPerBlock(b *testing.B) {
	// b.Skip("Skipping benchmark with buggy code reported at https://github.com/cosmos/cosmos-sdk/issues/10023")

	b.ReportAllocs()
	// Add an account at genesis
	acc := authtypes.BaseAccount{
		Address: addr1.String(),
	}

	// Construct genesis state
	genAccs := []authtypes.GenesisAccount{&acc}
	s := createTestSuite(&testing.T{}, genAccs)
	baseApp := s.App.BaseApp
	ctx := baseApp.NewContext(false, tmproto.Header{})

	// some value conceivably higher than the benchmarks would ever go
	require.NoError(b, testutil.FundAccount(s.BankKeeper, ctx, addr1, sdk.NewCoins(sdk.NewInt64Coin("foocoin", 100000000000))))

	baseApp.Commit()
	txGen := moduletestutil.MakeTestEncodingConfig().TxConfig

	// Precompute all txs
	txs, err := genSequenceOfTxs(txGen, []sdk.Msg{multiSendMsg1}, []uint64{0}, []uint64{uint64(0)}, b.N, priv1)
	require.NoError(b, err)
	b.ResetTimer()

	height := int64(3)

	// Run this with a profiler, so its easy to distinguish what time comes from
	// Committing, and what time comes from Check/Deliver Tx.
	for i := 0; i < b.N; i++ {
		baseApp.BeginBlock(abci.RequestBeginBlock{Header: tmproto.Header{Height: height}})
		_, _, err := baseApp.SimCheck(txGen.TxEncoder(), txs[i])
		if err != nil {
			panic("something is broken in checking transaction")
		}

		_, _, err = baseApp.SimDeliver(txGen.TxEncoder(), txs[i])
		require.NoError(b, err)
		baseApp.EndBlock(abci.RequestEndBlock{Height: height})
		baseApp.Commit()
		height++
	}
}
