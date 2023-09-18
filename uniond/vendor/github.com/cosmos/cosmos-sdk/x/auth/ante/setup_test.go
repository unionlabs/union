package ante_test

import (
	"testing"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/stretchr/testify/require"
)

func TestSetupDecorator_BlockMaxGas(t *testing.T) {
	suite := SetupTestSuite(t, true)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg := testdata.NewTestMsg(addr1)
	feeAmount := testdata.NewTestFeeAmount()
	require.NoError(t, suite.txBuilder.SetMsgs(msg))
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(101)

	privs, accNums, accSeqs := []cryptotypes.PrivKey{priv1}, []uint64{0}, []uint64{0}
	tx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	sud := ante.NewSetUpContextDecorator()
	antehandler := sdk.ChainAnteDecorators(sud)

	suite.ctx = suite.ctx.
		WithBlockHeight(1).
		WithGasMeter(storetypes.NewGasMeter(0)).
		WithConsensusParams(&tmproto.ConsensusParams{
			Block: &tmproto.BlockParams{
				MaxGas: 100,
			},
		})

	_, err = antehandler(suite.ctx, tx, false)
	require.Error(t, err)
}

func TestSetup(t *testing.T) {
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
	tx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	sud := ante.NewSetUpContextDecorator()
	antehandler := sdk.ChainAnteDecorators(sud)

	// Set height to non-zero value for GasMeter to be set
	suite.ctx = suite.ctx.WithBlockHeight(1).WithGasMeter(sdk.NewGasMeter(0))

	// Context GasMeter Limit not set
	require.Equal(t, uint64(0), suite.ctx.GasMeter().Limit(), "GasMeter set with limit before setup")

	newCtx, err := antehandler(suite.ctx, tx, false)
	require.Nil(t, err, "SetUpContextDecorator returned error")

	// Context GasMeter Limit should be set after SetUpContextDecorator runs
	require.Equal(t, gasLimit, newCtx.GasMeter().Limit(), "GasMeter not set correctly")
}

func TestRecoverPanic(t *testing.T) {
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
	tx, err := suite.CreateTestTx(privs, accNums, accSeqs, suite.ctx.ChainID())
	require.NoError(t, err)

	sud := ante.NewSetUpContextDecorator()
	antehandler := sdk.ChainAnteDecorators(sud, OutOfGasDecorator{})

	// Set height to non-zero value for GasMeter to be set
	suite.ctx = suite.ctx.WithBlockHeight(1)

	newCtx, err := antehandler(suite.ctx, tx, false)

	require.NotNil(t, err, "Did not return error on OutOfGas panic")

	require.True(t, sdkerrors.ErrOutOfGas.Is(err), "Returned error is not an out of gas error")
	require.Equal(t, gasLimit, newCtx.GasMeter().Limit())

	antehandler = sdk.ChainAnteDecorators(sud, PanicDecorator{})
	require.Panics(t, func() { antehandler(suite.ctx, tx, false) }, "Recovered from non-Out-of-Gas panic") // nolint:errcheck
}

type OutOfGasDecorator struct{}

// AnteDecorator that will throw OutOfGas panic
func (ogd OutOfGasDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	overLimit := ctx.GasMeter().Limit() + 1

	// Should panic with outofgas error
	ctx.GasMeter().ConsumeGas(overLimit, "test panic")

	// not reached
	return next(ctx, tx, simulate)
}

type PanicDecorator struct{}

func (pd PanicDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	panic("random error")
}
