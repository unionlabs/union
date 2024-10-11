package ante_test

import (
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/core/gas"
	"cosmossdk.io/core/header"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

// Test that simulate transaction accurately estimates gas cost
func TestSimulateGasCost(t *testing.T) {
	// This test has a test case that uses another's output.
	var simulatedGas uint64

	// Same data for every test case
	feeAmount := testdata.NewTestFeeAmount()

	testCases := []TestCase{
		{
			"tx with 150atom fee",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), authtypes.FeeCollectorName, feeAmount).Return(nil)

				msgs := []sdk.Msg{
					testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress()),
					testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress()),
					testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress()),
				}

				return TestCaseArgs{
					feeAmount: feeAmount,
					gasLimit:  testdata.NewTestGasLimit(),
					msgs:      msgs,
				}.WithAccountsInfo(accs)
			},
			true,
			true,
			nil,
		},
		{
			"with previously estimated gas",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), authtypes.FeeCollectorName, feeAmount).Return(nil)

				msgs := []sdk.Msg{
					testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress()),
					testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress()),
					testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress()),
				}

				return TestCaseArgs{
					feeAmount: feeAmount,
					gasLimit:  simulatedGas,
					msgs:      msgs,
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.chainID = suite.ctx.ChainID()
			suite.RunTestCase(t, tc, args)

			// Gather info for the next test case
			simulatedGas = suite.ctx.GasMeter().GasConsumed()
		})
	}
}

// Test various error cases in the AnteHandler control flow.
func TestAnteHandlerSigErrors(t *testing.T) {
	// This test requires the accounts to not be set, so we create them here
	priv0, _, addr0 := testdata.KeyTestPubAddr()
	priv1, _, addr1 := testdata.KeyTestPubAddr()
	priv2, _, addr2 := testdata.KeyTestPubAddr()
	msgs := []sdk.Msg{
		testdata.NewTestMsg(addr0, addr1),
		testdata.NewTestMsg(addr0, addr2),
	}

	testCases := []TestCase{
		{
			"check no signatures fails",
			func(suite *AnteTestSuite) TestCaseArgs {
				privs, accNums, accSeqs := []cryptotypes.PrivKey{}, []uint64{}, []uint64{}

				// Create tx manually to test the tx's signers
				require.NoError(t, suite.txBuilder.SetMsgs(msgs...))
				tx, err := suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
				require.NoError(t, err)

				// tx.GetSigners returns addresses in correct order: addr1, addr2, addr3
				expectedSigners := [][]byte{addr0, addr1, addr2}
				signers, err := tx.GetSigners()
				require.NoError(t, err)
				require.Equal(t, expectedSigners, signers)

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrNoSignatures,
		},
		{
			"num sigs dont match GetSigners",
			func(suite *AnteTestSuite) TestCaseArgs {
				privs, accNums, accSeqs := []cryptotypes.PrivKey{priv0}, []uint64{0}, []uint64{0}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"unrecognized account",
			func(suite *AnteTestSuite) TestCaseArgs {
				privs, accNums, accSeqs := []cryptotypes.PrivKey{priv0, priv1, priv2}, []uint64{0, 0, 0}, []uint64{0, 0, 0}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
		{
			"unrecognized account2",
			func(suite *AnteTestSuite) TestCaseArgs {
				suite.accountKeeper.SetAccount(suite.ctx, suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr1))
				privs, accNums, accSeqs := []cryptotypes.PrivKey{priv0, priv1, priv2}, []uint64{0, 1, 0}, []uint64{0, 0, 0}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
		{
			"save all the accounts, should pass",
			func(suite *AnteTestSuite) TestCaseArgs {
				suite.accountKeeper.SetAccount(suite.ctx, suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr0))
				suite.accountKeeper.SetAccount(suite.ctx, suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr1))
				suite.accountKeeper.SetAccount(suite.ctx, suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr2))
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				privs, accNums, accSeqs := []cryptotypes.PrivKey{priv0, priv1, priv2}, []uint64{1, 2, 3}, []uint64{0, 0, 0}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.chainID = suite.ctx.ChainID()
			args.feeAmount = testdata.NewTestFeeAmount()
			args.gasLimit = testdata.NewTestGasLimit()

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test logic around account number checking with one signer and many signers.
func TestAnteHandlerAccountNumbers(t *testing.T) {
	testCases := []TestCase{
		{
			"good tx from one signer",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"new tx from wrong account number",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{1},
					accSeqs: []uint64{0},
					msgs:    []sdk.Msg{msg},
					privs:   []cryptotypes.PrivKey{accs[0].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"new tx with another signer and incorrect account numbers",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{2, 0},
					accSeqs: []uint64{0, 0},
					msgs:    []sdk.Msg{msg1, msg2},
					privs:   []cryptotypes.PrivKey{accs[0].priv, accs[1].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"new tx with correct account numbers",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg1, msg2},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			args := tc.malleate(suite)
			args.feeAmount = testdata.NewTestFeeAmount()
			args.gasLimit = testdata.NewTestGasLimit()
			args.chainID = suite.ctx.ChainID()

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test logic around account number checking with many signers when BlockHeight is 0.
func TestAnteHandlerAccountNumbersAtBlockHeightZero(t *testing.T) {
	testCases := []TestCase{
		{
			"good tx from one signer",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{0},
					accSeqs: []uint64{0},
					msgs:    []sdk.Msg{msg},
					privs:   []cryptotypes.PrivKey{accs[0].priv},
				}
			},
			false,
			true,
			nil,
		},
		{
			"new tx from wrong account number",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{1}, // wrong account number
					accSeqs: []uint64{0},
					msgs:    []sdk.Msg{msg},
					privs:   []cryptotypes.PrivKey{accs[0].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"new tx with another signer and incorrect account numbers",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[0].acc.GetAddress())

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{1, 0}, // wrong account numbers
					accSeqs: []uint64{0, 0},
					msgs:    []sdk.Msg{msg1, msg2},
					privs:   []cryptotypes.PrivKey{accs[0].priv, accs[1].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"new tx with another signer and correct account numbers",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[0].acc.GetAddress())

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{0, 0}, // correct account numbers
					accSeqs: []uint64{0, 0},
					msgs:    []sdk.Msg{msg1, msg2},
					privs:   []cryptotypes.PrivKey{accs[0].priv, accs[1].priv},
				}
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.ctx = suite.ctx.WithBlockHeight(0).WithHeaderInfo(header.Info{Height: 0, ChainID: suite.ctx.ChainID()})
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

			args := tc.malleate(suite)
			args.feeAmount = testdata.NewTestFeeAmount()
			args.gasLimit = testdata.NewTestGasLimit()

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test logic around sequence checking with one signer and many signers.
func TestAnteHandlerSequences(t *testing.T) {
	// Same data for every test cases
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []TestCase{
		{
			"good tx from one signer",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"test sending it again fails (replay protection)",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				msgs := []sdk.Msg{msg}
				privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{accs[0].acc.GetAccountNumber()}, []uint64{accs[0].acc.GetSequence()}

				// This will be called only once given that the second tx will fail
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				// Send the same tx before running the test case, to trigger replay protection.
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				return TestCaseArgs{
					msgs: msgs,
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
		{
			"fix sequence, should pass",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
				msgs := []sdk.Msg{msg}

				privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{accs[0].acc.GetAccountNumber()}, []uint64{accs[0].acc.GetSequence()}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				// Send the same tx before running the test case, then change the sequence to a valid one.
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				// +1 the account sequence
				require.NoError(t, accs[0].acc.SetSequence(accs[0].acc.GetSequence()+1))

				return TestCaseArgs{
					msgs: msgs,
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"new tx with another signer and correct sequences",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg1, msg2},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"replay fails",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				// Send the same tx before running the test case, to trigger replay protection.
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg1, msg2},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
		{
			"tx from just second signer with incorrect sequence fails",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				// Send the same tx before running the test case, to trigger replay protection.
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				// Send a message using the second signer, this will fail given that the second signer already sent a TX,
				// thus the sequence (0) is incorrect.
				msg1 = testdata.NewTestMsg(accs[1].acc.GetAddress())
				msgs = []sdk.Msg{msg1}
				privs, accNums, accSeqs = []cryptotypes.PrivKey{accs[1].priv}, []uint64{accs[1].acc.GetAccountNumber()}, []uint64{0}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
		{
			"fix the sequence and it passes",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				// Send the same tx before running the test case, to trigger replay protection.
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				// Send a message using the second signer, this will now pass given that the second signer already sent a TX
				// and the sequence was fixed (1).
				msg1 = testdata.NewTestMsg(accs[1].acc.GetAddress())
				msgs = []sdk.Msg{msg1}
				privs, accNums, accSeqs = []cryptotypes.PrivKey{accs[1].priv}, []uint64{accs[1].acc.GetAccountNumber()}, []uint64{accs[1].acc.GetSequence() + 1}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.feeAmount = feeAmount
			args.gasLimit = gasLimit

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test logic around fee deduction.
func TestAnteHandlerFees(t *testing.T) {
	// Same data for every test cases
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []TestCase{
		{
			"signer has no funds",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), feeAmount).Return(sdkerrors.ErrInsufficientFunds)

				return TestCaseArgs{
					msgs: []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrInsufficientFunds,
		},
		{
			"signer has enough funds, should pass",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), gomock.Any(), feeAmount).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.feeAmount = feeAmount
			args.gasLimit = gasLimit

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test logic around memo gas consumption.
func TestAnteHandlerMemoGas(t *testing.T) {
	testCases := []TestCase{
		{
			"tx does not have enough gas",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				return TestCaseArgs{
					feeAmount: sdk.NewCoins(sdk.NewInt64Coin("atom", 0)),
					gasLimit:  0,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrOutOfGas,
		},
		{
			"tx with memo doesn't have enough gas",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.txBuilder.SetMemo("abcininasidniandsinasindiansdiansdinaisndiasndiadninsd")

				return TestCaseArgs{
					feeAmount: sdk.NewCoins(sdk.NewInt64Coin("atom", 0)),
					gasLimit:  801,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrOutOfGas,
		},
		{
			"memo too large",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.txBuilder.SetMemo(strings.Repeat("01234567890", 500))

				return TestCaseArgs{
					feeAmount: sdk.NewCoins(sdk.NewInt64Coin("atom", 0)),
					gasLimit:  50000,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrMemoTooLarge,
		},
		{
			"tx with memo has enough gas",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.txBuilder.SetMemo(strings.Repeat("0123456789", 10))
				return TestCaseArgs{
					feeAmount: sdk.NewCoins(sdk.NewInt64Coin("atom", 0)),
					gasLimit:  60000,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)

			suite.RunTestCase(t, tc, args)
		})
	}
}

func TestAnteHandlerMultiSigner(t *testing.T) {
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []TestCase{
		{
			"signers in order",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msg3 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress())
				suite.txBuilder.SetMemo("Check signers are in expected order and different account numbers works")
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{msg1, msg2, msg3},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"change sequence numbers (only accounts 0 and 1 sign)",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msg3 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2, msg3}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				msgs = []sdk.Msg{msg1}
				privs, accNums, accSeqs = []cryptotypes.PrivKey{accs[0].priv, accs[1].priv}, []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber()}, []uint64{accs[0].acc.GetSequence() + 1, accs[1].acc.GetSequence() + 1}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
		{
			"change sequence numbers (only accounts 1 and 2 sign)",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msg3 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2, msg3}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				msgs = []sdk.Msg{msg3}
				privs, accNums, accSeqs = []cryptotypes.PrivKey{accs[1].priv, accs[2].priv}, []uint64{accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}, []uint64{accs[1].acc.GetSequence() + 1, accs[2].acc.GetSequence() + 1}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			true,
			nil,
		},
		{
			"everyone signs again",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(3)
				msg1 := testdata.NewTestMsg(accs[0].acc.GetAddress(), accs[1].acc.GetAddress())
				msg2 := testdata.NewTestMsg(accs[2].acc.GetAddress(), accs[0].acc.GetAddress())
				msg3 := testdata.NewTestMsg(accs[1].acc.GetAddress(), accs[2].acc.GetAddress())
				msgs := []sdk.Msg{msg1, msg2, msg3}

				privs := []cryptotypes.PrivKey{accs[0].priv, accs[1].priv, accs[2].priv}
				accNums := []uint64{accs[0].acc.GetAccountNumber(), accs[1].acc.GetAccountNumber(), accs[2].acc.GetAccountNumber()}
				accSeqs := []uint64{accs[0].acc.GetSequence(), accs[1].acc.GetSequence(), accs[2].acc.GetSequence()}

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				for _, acc := range accs {
					require.NoError(t, acc.acc.SetSequence(acc.acc.GetSequence()+1))
				}

				return TestCaseArgs{
					msgs: msgs,
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.feeAmount = feeAmount
			args.gasLimit = gasLimit
			args.chainID = suite.ctx.ChainID()

			suite.RunTestCase(t, tc, args)
		})
	}
}

func TestAnteHandlerBadSignBytes(t *testing.T) {
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []TestCase{
		{
			"test good tx and signBytes",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg0 := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{msg0},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"test wrong chainID",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg0 := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					chainID:   "wrong-chain-id",
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{msg0},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"test wrong accSeqs",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg0 := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)
				require.NoError(t, accs[0].acc.SetSequence(2)) // wrong accSeq

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{msg0},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
		{
			"test wrong accNums",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				msg0 := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)
				require.NoError(t, accs[0].acc.SetAccountNumber(2)) // wrong accNum

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{msg0},
				}.WithAccountsInfo(accs)
			},
			false,
			false,
			sdkerrors.ErrUnauthorized,
		},
		{
			"test wrong msg",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[1].acc.GetAddress())}, // wrong account in the msg
				}.WithAccountsInfo(accs[0:1])
			},
			false,
			false,
			sdkerrors.ErrInvalidPubKey,
		},
		{
			"test wrong signer if public key exist",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				msg0 := testdata.NewTestMsg(accs[0].acc.GetAddress())
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					accNums:   []uint64{accs[0].acc.GetAccountNumber()},
					accSeqs:   []uint64{accs[0].acc.GetSequence()},
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{msg0},
					privs:     []cryptotypes.PrivKey{accs[1].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrInvalidPubKey,
		},
		{
			"test wrong signer if public doesn't exist",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					chainID:   suite.ctx.ChainID(),
					accNums:   []uint64{accs[1].acc.GetAccountNumber()},
					accSeqs:   []uint64{accs[1].acc.GetSequence()},
					feeAmount: feeAmount,
					gasLimit:  gasLimit,
					msgs:      []sdk.Msg{testdata.NewTestMsg(accs[1].acc.GetAddress())},
					privs:     []cryptotypes.PrivKey{accs[0].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrInvalidPubKey,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)

			suite.RunTestCase(t, tc, args)
		})
	}
}

func TestAnteHandlerSetPubKey(t *testing.T) {
	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	testCases := []TestCase{
		{
			"test good tx",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					msgs: []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
				}.WithAccountsInfo(accs)
			},
			false,
			true,
			nil,
		},
		{
			"make sure public key has been set (tx itself should fail because of replay protection)",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(2)

				privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{accs[0].acc.GetAccountNumber()}, []uint64{accs[0].acc.GetSequence()}
				msgs := []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())}
				var err error
				suite.ctx, err = suite.DeliverMsgs(t, privs, msgs, feeAmount, gasLimit, accNums, accSeqs, suite.ctx.ChainID(), false)
				require.NoError(t, err)

				// Make sure public key has been set from previous test.
				acc0 := suite.accountKeeper.GetAccount(suite.ctx, accs[0].acc.GetAddress())
				require.Equal(t, acc0.GetPubKey(), accs[0].priv.PubKey())

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
		{
			"test public key not found",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)
				return TestCaseArgs{
					msgs: []sdk.Msg{testdata.NewTestMsg(accs[1].acc.GetAddress())},
				}.WithAccountsInfo(accs[0:1])
			},
			false,
			false,
			sdkerrors.ErrInvalidPubKey,
		},
		{
			"make sure public key is not set, when tx has no pubkey or signature",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				// Make sure public key has not been set from previous test.
				acc1 := suite.accountKeeper.GetAccount(suite.ctx, accs[1].acc.GetAddress())
				require.Nil(t, acc1.GetPubKey())

				privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[1].priv}, []uint64{accs[1].acc.GetAccountNumber()}, []uint64{accs[1].acc.GetSequence()}
				msgs := []sdk.Msg{testdata.NewTestMsg(accs[1].acc.GetAddress())}
				err := suite.txBuilder.SetMsgs(msgs...)
				require.NoError(t, err)
				suite.txBuilder.SetFeeAmount(feeAmount)
				suite.txBuilder.SetGasLimit(gasLimit)

				// Manually create tx, and remove signature.
				tx, err := suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
				require.NoError(t, err)
				txBuilder, err := suite.clientCtx.TxConfig.WrapTxBuilder(tx)
				require.NoError(t, err)
				require.NoError(t, txBuilder.SetSignatures())

				// Run anteHandler manually, expect ErrNoSignatures.
				_, err = suite.anteHandler(suite.ctx, txBuilder.GetTx(), false)
				require.Error(t, err)
				require.True(t, errors.Is(err, sdkerrors.ErrNoSignatures))

				// Make sure public key has not been set.
				acc1 = suite.accountKeeper.GetAccount(suite.ctx, accs[1].acc.GetAddress())
				require.Nil(t, acc1.GetPubKey())

				// Set incorrect accSeq, to generate incorrect signature.
				privs, accNums, accSeqs = []cryptotypes.PrivKey{accs[1].priv}, []uint64{accs[1].acc.GetAccountNumber()}, []uint64{1}

				return TestCaseArgs{
					accNums: accNums,
					accSeqs: accSeqs,
					msgs:    msgs,
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrWrongSequence,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.chainID = suite.ctx.ChainID()
			args.feeAmount = feeAmount
			args.gasLimit = gasLimit

			suite.RunTestCase(t, tc, args)
		})
	}
}

func generatePubKeysAndSignatures(n int, msg []byte, _ bool) (pubkeys []cryptotypes.PubKey, signatures [][]byte) {
	pubkeys = make([]cryptotypes.PubKey, n)
	signatures = make([][]byte, n)
	for i := 0; i < n; i++ {
		var privkey cryptotypes.PrivKey = secp256k1.GenPrivKey()

		// TODO: also generate ed25519 keys as below when ed25519 keys are
		//  actually supported, https://github.com/cosmos/cosmos-sdk/issues/4789
		// for now this fails:
		// if rand.Int63()%2 == 0 {
		//	privkey = ed25519.GenPrivKey()
		// } else {
		//	privkey = secp256k1.GenPrivKey()
		// }

		pubkeys[i] = privkey.PubKey()
		signatures[i], _ = privkey.Sign(msg)
	}
	return
}

func TestCountSubkeys(t *testing.T) {
	genPubKeys := func(n int) []cryptotypes.PubKey {
		var ret []cryptotypes.PubKey
		for i := 0; i < n; i++ {
			ret = append(ret, secp256k1.GenPrivKey().PubKey())
		}
		return ret
	}
	singleKey := secp256k1.GenPrivKey().PubKey()
	singleLevelMultiKey := kmultisig.NewLegacyAminoPubKey(4, genPubKeys(5))
	multiLevelSubKey1 := kmultisig.NewLegacyAminoPubKey(4, genPubKeys(5))
	multiLevelSubKey2 := kmultisig.NewLegacyAminoPubKey(4, genPubKeys(5))
	multiLevelMultiKey := kmultisig.NewLegacyAminoPubKey(2, []cryptotypes.PubKey{
		multiLevelSubKey1, multiLevelSubKey2, secp256k1.GenPrivKey().PubKey(),
	})
	type args struct {
		pub cryptotypes.PubKey
	}
	testCases := []struct {
		name string
		args args
		want int
	}{
		{"single key", args{singleKey}, 1},
		{"single level multikey", args{singleLevelMultiKey}, 5},
		{"multi level multikey", args{multiLevelMultiKey}, 11},
		{"nil key", args{nil}, 0},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(T *testing.T) {
			require.Equal(t, tc.want, ante.CountSubKeys(tc.args.pub))
		})
	}
}

func TestAnteHandlerSigLimitExceeded(t *testing.T) {
	testCases := []TestCase{
		{
			"test rejection logic",
			func(suite *AnteTestSuite) TestCaseArgs {
				accs := suite.CreateTestAccounts(8)
				var (
					addrs []sdk.AccAddress
					privs []cryptotypes.PrivKey
				)
				for i := 0; i < 8; i++ {
					addrs = append(addrs, accs[i].acc.GetAddress())
					privs = append(privs, accs[i].priv)
				}

				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{0, 1, 2, 3, 4, 5, 6, 7},
					accSeqs: []uint64{0, 0, 0, 0, 0, 0, 0, 0},
					msgs:    []sdk.Msg{testdata.NewTestMsg(addrs...)},
					privs:   privs,
				}
			},
			false,
			false,
			sdkerrors.ErrTooManySignatures,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.chainID = suite.ctx.ChainID()
			args.feeAmount = testdata.NewTestFeeAmount()
			args.gasLimit = testdata.NewTestGasLimit()

			suite.RunTestCase(t, tc, args)
		})
	}
}

// Test custom SignatureVerificationGasConsumer
func TestCustomSignatureVerificationGasConsumer(t *testing.T) {
	testCases := []TestCase{
		{
			"verify that an secp256k1 account gets rejected",
			func(suite *AnteTestSuite) TestCaseArgs {
				// setup an ante handler that only accepts PubKeyEd25519
				anteHandler, err := ante.NewAnteHandler(
					ante.HandlerOptions{
						AccountKeeper:   suite.accountKeeper,
						BankKeeper:      suite.bankKeeper,
						ConsensusKeeper: suite.consensusKeeper,
						FeegrantKeeper:  suite.feeGrantKeeper,
						SignModeHandler: suite.clientCtx.TxConfig.SignModeHandler(),
						SigGasConsumer: func(meter gas.Meter, sig signing.SignatureV2, params authtypes.Params) error {
							switch pubkey := sig.PubKey.(type) {
							case *ed25519.PubKey:
								return meter.Consume(params.SigVerifyCostED25519, "ante verify: ed25519")
							default:
								return errorsmod.Wrapf(sdkerrors.ErrInvalidPubKey, "unrecognized public key type: %T", pubkey)
							}
						},
						Environment: suite.env,
					},
				)
				require.NoError(t, err)
				suite.anteHandler = anteHandler

				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil)

				return TestCaseArgs{
					accNums: []uint64{0},
					accSeqs: []uint64{0},
					msgs:    []sdk.Msg{testdata.NewTestMsg(accs[0].acc.GetAddress())},
					privs:   []cryptotypes.PrivKey{accs[0].priv},
				}
			},
			false,
			false,
			sdkerrors.ErrInvalidPubKey,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.desc), func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()
			args := tc.malleate(suite)
			args.chainID = suite.ctx.ChainID()
			args.feeAmount = testdata.NewTestFeeAmount()
			args.gasLimit = testdata.NewTestGasLimit()

			suite.RunTestCase(t, tc, args)
		})
	}
}

func TestAnteHandlerReCheck(t *testing.T) {
	suite := SetupTestSuite(t, false)
	// Set recheck=true
	suite.ctx = suite.ctx.WithIsReCheckTx(true)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// Same data for every test case
	accs := suite.CreateTestAccounts(1)

	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(gasLimit)

	msg := testdata.NewTestMsg(accs[0].acc.GetAddress())
	msgs := []sdk.Msg{msg}
	require.NoError(t, suite.txBuilder.SetMsgs(msgs...))

	suite.txBuilder.SetMemo("thisisatestmemo")

	// test that operations skipped on recheck do not run
	privs, accNums, accSeqs := []cryptotypes.PrivKey{accs[0].priv}, []uint64{0}, []uint64{0}
	tx, err := suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)

	// make signature array empty which would normally cause ValidateBasicDecorator and SigVerificationDecorator fail
	// since these decorators don't run on recheck, the tx should pass the antehandler
	txBuilder, err := suite.clientCtx.TxConfig.WrapTxBuilder(tx)
	require.NoError(t, err)

	suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(nil).Times(3)
	_, err = suite.anteHandler(suite.ctx, txBuilder.GetTx(), false)
	require.Nil(t, err, "AnteHandler errored on recheck unexpectedly: %v", err)

	tx, err = suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)
	txBytes, err := json.Marshal(tx)
	require.Nil(t, err, "Error marshaling tx: %v", err)
	suite.ctx = suite.ctx.WithTxBytes(txBytes)

	// require that state machine param-dependent checking is still run on recheck since parameters can change between check and recheck
	testCases := []struct {
		name   string
		params authtypes.Params
	}{
		{"memo size check", authtypes.NewParams(1, authtypes.DefaultTxSigLimit, authtypes.DefaultTxSizeCostPerByte, authtypes.DefaultSigVerifyCostED25519, authtypes.DefaultSigVerifyCostSecp256k1)},
		{"txsize check", authtypes.NewParams(authtypes.DefaultMaxMemoCharacters, authtypes.DefaultTxSigLimit, 10000000, authtypes.DefaultSigVerifyCostED25519, authtypes.DefaultSigVerifyCostSecp256k1)},
		{"sig verify cost check", authtypes.NewParams(authtypes.DefaultMaxMemoCharacters, authtypes.DefaultTxSigLimit, authtypes.DefaultTxSizeCostPerByte, authtypes.DefaultSigVerifyCostED25519, 100000000)},
	}

	for _, tc := range testCases {

		// set testcase parameters
		err := suite.accountKeeper.Params.Set(suite.ctx, tc.params)
		require.NoError(t, err)

		_, err = suite.anteHandler(suite.ctx, tx, false)

		require.NotNil(t, err, "tx does not fail on recheck with updated params in test case: %s", tc.name)

		// reset parameters to default values
		err = suite.accountKeeper.Params.Set(suite.ctx, authtypes.DefaultParams())
		require.NoError(t, err)
	}

	suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), gomock.Any(), gomock.Any(), gomock.Any()).Return(sdkerrors.ErrInsufficientFee)
	// require that local mempool fee check is still run on recheck since validator may change minFee between check and recheck
	// create new minimum gas price so antehandler fails on recheck
	suite.ctx = suite.ctx.WithMinGasPrices([]sdk.DecCoin{{
		Denom:  "dnecoin", // fee does not have this denom
		Amount: math.LegacyNewDec(5),
	}})
	_, err = suite.anteHandler(suite.ctx, tx, false)
	require.NotNil(t, err, "antehandler on recheck did not fail when mingasPrice was changed")
	// reset min gasprice
	suite.ctx = suite.ctx.WithMinGasPrices(sdk.DecCoins{})

	// remove funds for account so antehandler fails on recheck
	suite.accountKeeper.SetAccount(suite.ctx, accs[0].acc)

	_, err = suite.anteHandler(suite.ctx, tx, false)
	require.NotNil(t, err, "antehandler on recheck did not fail once feePayer no longer has sufficient funds")
}
