package ante_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/math"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestDeductFeeDecorator_ZeroGas(t *testing.T) {
	s := SetupTestSuite(t, true)
	s.txBuilder = s.clientCtx.TxConfig.NewTxBuilder()

	mfd := ante.NewDeductFeeDecorator(s.accountKeeper, s.bankKeeper, s.feeGrantKeeper, nil)
	antehandler := sdk.ChainAnteDecorators(mfd)

	// keys and addresses
	accs := s.CreateTestAccounts(1)

	// msg and signatures
	msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
	require.NoError(t, s.txBuilder.SetMsgs(msg))

	// set zero gas
	s.txBuilder.SetGasLimit(0)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{0}, []uint64{0}
	tx, err := s.CreateTestTx(s.ctx, privs, accNums, accSeqs, s.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)

	// Set IsCheckTx to true
	s.ctx = s.ctx.WithIsCheckTx(true)

	// Set current block height in headerInfo
	headerInfo := s.ctx.HeaderInfo()
	headerInfo.Height = s.ctx.BlockHeight()
	s.ctx = s.ctx.WithHeaderInfo(headerInfo)

	_, err = antehandler(s.ctx, tx, false)
	require.Error(t, err)

	// zero gas is accepted in simulation mode
	s.ctx = s.ctx.WithExecMode(sdk.ExecModeSimulate)
	_, err = antehandler(s.ctx, tx, true)
	require.NoError(t, err)
}

func TestEnsureMempoolFees(t *testing.T) {
	s := SetupTestSuite(t, true) // setup
	s.txBuilder = s.clientCtx.TxConfig.NewTxBuilder()

	mfd := ante.NewDeductFeeDecorator(s.accountKeeper, s.bankKeeper, s.feeGrantKeeper, nil)
	antehandler := sdk.ChainAnteDecorators(mfd)

	// keys and addresses
	accs := s.CreateTestAccounts(1)

	// msg and signatures
	msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := uint64(15)
	require.NoError(t, s.txBuilder.SetMsgs(msg))
	s.txBuilder.SetFeeAmount(feeAmount)
	s.txBuilder.SetGasLimit(gasLimit)

	s.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), authtypes.FeeCollectorName, feeAmount).Return(nil).Times(3)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{0}, []uint64{0}
	tx, err := s.CreateTestTx(s.ctx, privs, accNums, accSeqs, s.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)

	// Set high gas price so standard test fee fails
	atomPrice := sdk.NewDecCoinFromDec("atom", math.LegacyNewDec(20))
	highGasPrice := []sdk.DecCoin{atomPrice}
	s.ctx = s.ctx.WithMinGasPrices(highGasPrice)

	// antehandler errors with insufficient fees
	_, err = antehandler(s.ctx, tx, false)
	require.NotNil(t, err, "Decorator should have errored on too low fee for local gasPrice")

	// antehandler should not error since we do not check minGasPrice in simulation mode
	cacheCtx, _ := s.ctx.CacheContext()
	cacheCtx = cacheCtx.WithExecMode(sdk.ExecModeSimulate)
	_, err = antehandler(cacheCtx, tx, true)
	require.Nil(t, err, "Decorator should not have errored in simulation mode")

	// antehandler should not error since we do not check minGasPrice in DeliverTx
	s.ctx = s.ctx.WithExecMode(sdk.ExecModeFinalize)
	_, err = antehandler(s.ctx, tx, false)
	require.Nil(t, err, "MempoolFeeDecorator returned error in DeliverTx")

	atomPrice = sdk.NewDecCoinFromDec("atom", math.LegacyNewDec(0).Quo(math.LegacyNewDec(100000)))
	lowGasPrice := []sdk.DecCoin{atomPrice}
	s.ctx = s.ctx.WithMinGasPrices(lowGasPrice)

	newCtx, err := antehandler(s.ctx, tx, false)
	require.Nil(t, err, "Decorator should not have errored on fee higher than local gasPrice")
	// Priority is the smallest gas price amount in any denom. Since we have only 1 gas price
	// of 10atom, the priority here is 10.
	require.Equal(t, int64(10), newCtx.Priority())
}

func TestDeductFees(t *testing.T) {
	s := SetupTestSuite(t, false)
	s.txBuilder = s.clientCtx.TxConfig.NewTxBuilder()

	// keys and addresses
	accs := s.CreateTestAccounts(1)

	// msg and signatures
	msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	require.NoError(t, s.txBuilder.SetMsgs(msg))
	s.txBuilder.SetFeeAmount(feeAmount)
	s.txBuilder.SetGasLimit(gasLimit)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{0}, []uint64{0}
	tx, err := s.CreateTestTx(s.ctx, privs, accNums, accSeqs, s.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)

	dfd := ante.NewDeductFeeDecorator(s.accountKeeper, s.bankKeeper, nil, nil)
	antehandler := sdk.ChainAnteDecorators(dfd)
	s.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(sdkerrors.ErrInsufficientFunds)

	_, err = antehandler(s.ctx, tx, false)

	require.NotNil(t, err, "Tx did not error when fee payer had insufficient funds")

	s.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)
	_, err = antehandler(s.ctx, tx, false)

	require.Nil(t, err, "Tx errored after account has been set with sufficient funds")
}
