package ante_test

import (
	"strings"
	"testing"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/stretchr/testify/require"
)

func TestValidateBasic(t *testing.T) {
	suite := SetupTestSuite(t, true)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg := testdata.NewTestMsg(addr1)
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	require.NoError(t, suite.txBuilder.SetMsgs(msg))
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(gasLimit)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{}, []uint64{}, []uint64{}
	invalidTx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	vbd := ante.NewValidateBasicDecorator()
	antehandler := sdk.ChainAnteDecorators(vbd)
	_, err = antehandler(suite.ctx, invalidTx, false)

	require.ErrorIs(t, err, sdkerrors.ErrNoSignatures, "Did not error on invalid tx")

	privs, accNums, accSeqs = []cryptotypes.PrivKey{priv1}, []uint64{0}, []uint64{0}
	validTx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	_, err = antehandler(suite.ctx, validTx, false)
	require.Nil(t, err, "ValidateBasicDecorator returned error on valid tx. err: %v", err)

	// test decorator skips on recheck
	suite.ctx = suite.ctx.WithIsReCheckTx(true)

	// decorator should skip processing invalidTx on recheck and thus return nil-error
	_, err = antehandler(suite.ctx, invalidTx, false)

	require.Nil(t, err, "ValidateBasicDecorator ran on ReCheck")
}

func TestValidateMemo(t *testing.T) {
	suite := SetupTestSuite(t, true)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg := testdata.NewTestMsg(addr1)
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	require.NoError(t, suite.txBuilder.SetMsgs(msg))
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(gasLimit)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{priv1}, []uint64{0}, []uint64{0}
	suite.txBuilder.SetMemo(strings.Repeat("01234567890", 500))
	invalidTx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	// require that long memos get rejected
	vmd := ante.NewValidateMemoDecorator(suite.accountKeeper)
	antehandler := sdk.ChainAnteDecorators(vmd)
	_, err = antehandler(suite.ctx, invalidTx, false)

	require.ErrorIs(t, err, sdkerrors.ErrMemoTooLarge, "Did not error on tx with high memo")

	suite.txBuilder.SetMemo(strings.Repeat("01234567890", 10))
	validTx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	// require small memos pass ValidateMemo Decorator
	_, err = antehandler(suite.ctx, validTx, false)
	require.Nil(t, err, "ValidateBasicDecorator returned error on valid tx. err: %v", err)
}

func TestConsumeGasForTxSize(t *testing.T) {
	suite := SetupTestSuite(t, true)

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg := testdata.NewTestMsg(addr1)
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	cgtsd := ante.NewConsumeGasForTxSizeDecorator(suite.accountKeeper)
	antehandler := sdk.ChainAnteDecorators(cgtsd)

	testCases := []struct {
		name  string
		sigV2 signing.SignatureV2
	}{
		{"SingleSignatureData", signing.SignatureV2{PubKey: priv1.PubKey()}},
		{"MultiSignatureData", signing.SignatureV2{PubKey: priv1.PubKey(), Data: multisig.NewMultisig(2)}},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			require.NoError(t, suite.txBuilder.SetMsgs(msg))
			suite.txBuilder.SetFeeAmount(feeAmount)
			suite.txBuilder.SetGasLimit(gasLimit)
			suite.txBuilder.SetMemo(strings.Repeat("01234567890", 10))

			privs, accNums, accSeqs := []cryptotypes.PrivKey{priv1}, []uint64{0}, []uint64{0}
			tx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
			require.NoError(t, err)

			txBytes, err := suite.clientCtx.TxConfig.TxJSONEncoder()(tx)
			require.Nil(t, err, "Cannot marshal tx: %v", err)

			params := suite.accountKeeper.GetParams(suite.ctx)
			expectedGas := sdk.Gas(len(txBytes)) * params.TxSizeCostPerByte

			// Set suite.ctx with TxBytes manually
			suite.ctx = suite.ctx.WithTxBytes(txBytes)

			// track how much gas is necessary to retrieve parameters
			beforeGas := suite.ctx.GasMeter().GasConsumed()
			suite.accountKeeper.GetParams(suite.ctx)
			afterGas := suite.ctx.GasMeter().GasConsumed()
			expectedGas += afterGas - beforeGas

			beforeGas = suite.ctx.GasMeter().GasConsumed()
			suite.ctx, err = antehandler(suite.ctx, tx, false)
			require.Nil(t, err, "ConsumeTxSizeGasDecorator returned error: %v", err)

			// require that decorator consumes expected amount of gas
			consumedGas := suite.ctx.GasMeter().GasConsumed() - beforeGas
			require.Equal(t, expectedGas, consumedGas, "Decorator did not consume the correct amount of gas")

			// simulation must not underestimate gas of this decorator even with nil signatures
			txBuilder, err := suite.clientCtx.TxConfig.WrapTxBuilder(tx)
			require.NoError(t, err)
			require.NoError(t, txBuilder.SetSignatures(tc.sigV2))
			tx = txBuilder.GetTx()

			simTxBytes, err := suite.clientCtx.TxConfig.TxJSONEncoder()(tx)
			require.Nil(t, err, "Cannot marshal tx: %v", err)
			// require that simulated tx is smaller than tx with signatures
			require.True(t, len(simTxBytes) < len(txBytes), "simulated tx still has signatures")

			// Set suite.ctx with smaller simulated TxBytes manually
			suite.ctx = suite.ctx.WithTxBytes(simTxBytes)

			beforeSimGas := suite.ctx.GasMeter().GasConsumed()

			// run antehandler with simulate=true
			suite.ctx, err = antehandler(suite.ctx, tx, true)
			consumedSimGas := suite.ctx.GasMeter().GasConsumed() - beforeSimGas

			// require that antehandler passes and does not underestimate decorator cost
			require.Nil(t, err, "ConsumeTxSizeGasDecorator returned error: %v", err)
			require.True(t, consumedSimGas >= expectedGas, "Simulate mode underestimates gas on AnteDecorator. Simulated cost: %d, expected cost: %d", consumedSimGas, expectedGas)
		})
	}
}

func TestTxHeightTimeoutDecorator(t *testing.T) {
	suite := SetupTestSuite(t, true)

	antehandler := sdk.ChainAnteDecorators(ante.NewTxTimeoutHeightDecorator())

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg := testdata.NewTestMsg(addr1)
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []struct {
		name        string
		timeout     uint64
		height      int64
		expectedErr error
	}{
		{"default value", 0, 10, nil},
		{"no timeout (greater height)", 15, 10, nil},
		{"no timeout (same height)", 10, 10, nil},
		{"timeout (smaller height)", 9, 10, sdkerrors.ErrTxTimeoutHeight},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

			require.NoError(t, suite.txBuilder.SetMsgs(msg))

			suite.txBuilder.SetFeeAmount(feeAmount)
			suite.txBuilder.SetGasLimit(gasLimit)
			suite.txBuilder.SetMemo(strings.Repeat("01234567890", 10))
			suite.txBuilder.SetTimeoutHeight(tc.timeout)

			privs, accNums, accSeqs := []cryptotypes.PrivKey{priv1}, []uint64{0}, []uint64{0}
			tx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
			require.NoError(t, err)

			ctx := suite.ctx.WithBlockHeight(tc.height)
			_, err = antehandler(ctx, tx, true)
			require.ErrorIs(t, err, tc.expectedErr)
		})
	}
}
