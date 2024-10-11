package ante_test

import (
	"context"
	"errors"
	"math/rand"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/simulation"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authsign "github.com/cosmos/cosmos-sdk/x/auth/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/tx"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestDeductFeesNoDelegation(t *testing.T) {
	cases := map[string]struct {
		fee      int64
		valid    bool
		err      error
		errMsg   string
		malleate func(*AnteTestSuite) (signer TestAccount, feeAcc sdk.AccAddress)
	}{
		"paying with low funds": {
			fee:   50,
			valid: false,
			err:   sdkerrors.ErrInsufficientFunds,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(1)
				// 2 calls are needed because we run the ante twice
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), authtypes.FeeCollectorName, gomock.Any()).Return(sdkerrors.ErrInsufficientFunds).Times(2)
				return accs[0], nil
			},
		},
		"paying with good funds": {
			fee:   50,
			valid: true,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(1)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[0].acc.GetAddress(), authtypes.FeeCollectorName, gomock.Any()).Return(nil).Times(2)
				return accs[0], nil
			},
		},
		"paying with no account": {
			fee:   1,
			valid: true,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				// Do not register the account
				priv, _, addr := testdata.KeyTestPubAddr()
				acc := authtypes.NewBaseAccountWithAddress(addr)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), acc.GetAddress(), authtypes.FeeCollectorName, gomock.Any()).Return(nil).Times(2)
				return TestAccount{
					acc:  acc,
					priv: priv,
				}, nil
			},
		},
		"no fee with real account": {
			fee:   0,
			valid: true,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(1)
				return accs[0], nil
			},
		},
		"no fee with no account": {
			fee:   0,
			valid: true,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				// Do not register the account
				priv, _, addr := testdata.KeyTestPubAddr()
				return TestAccount{
					acc:  authtypes.NewBaseAccountWithAddress(addr),
					priv: priv,
				}, nil
			},
		},
		"valid fee grant": {
			// note: the original test said "valid fee grant with no account".
			// this is impossible given that feegrant.GrantAllowance calls
			// SetAccount for the grantee.
			fee:   50,
			valid: true,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(2)

				suite.feeGrantKeeper.EXPECT().UseGrantedFees(gomock.Any(), accs[1].acc.GetAddress(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil).Times(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[1].acc.GetAddress(), authtypes.FeeCollectorName, gomock.Any()).Return(nil).Times(2)
				return accs[0], accs[1].acc.GetAddress()
			},
		},
		"no fee grant": {
			fee:   2,
			valid: false,
			err:   sdkerrors.ErrNotFound,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(2)
				suite.feeGrantKeeper.EXPECT().
					UseGrantedFees(gomock.Any(), accs[1].acc.GetAddress(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).
					Return(sdkerrors.ErrNotFound.Wrap("fee-grant not found")).
					Times(2)
				return accs[0], accs[1].acc.GetAddress()
			},
		},
		"allowance smaller than requested fee": {
			fee:    50,
			valid:  false,
			errMsg: "fee limit exceeded",
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(2)
				suite.feeGrantKeeper.EXPECT().
					UseGrantedFees(gomock.Any(), accs[1].acc.GetAddress(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).
					Return(errors.New("fee limit exceeded")).
					Times(2)
				return accs[0], accs[1].acc.GetAddress()
			},
		},
		"granter cannot cover allowed fee grant": {
			fee:   50,
			valid: false,
			err:   sdkerrors.ErrInsufficientFunds,
			malleate: func(suite *AnteTestSuite) (TestAccount, sdk.AccAddress) {
				accs := suite.CreateTestAccounts(2)
				suite.feeGrantKeeper.EXPECT().UseGrantedFees(gomock.Any(), accs[1].acc.GetAddress(), accs[0].acc.GetAddress(), gomock.Any(), gomock.Any()).Return(nil).Times(2)
				suite.bankKeeper.EXPECT().SendCoinsFromAccountToModule(gomock.Any(), accs[1].acc.GetAddress(), authtypes.FeeCollectorName, gomock.Any()).Return(sdkerrors.ErrInsufficientFunds).Times(2)
				return accs[0], accs[1].acc.GetAddress()
			},
		},
	}

	for name, stc := range cases {
		tc := stc // to make scopelint happy
		t.Run(name, func(t *testing.T) {
			suite := SetupTestSuite(t, false)
			cdc := codec.NewProtoCodec(suite.encCfg.InterfaceRegistry)
			signingCtx := suite.encCfg.InterfaceRegistry.SigningContext()
			protoTxCfg := tx.NewTxConfig(cdc, signingCtx.AddressCodec(), signingCtx.ValidatorAddressCodec(), tx.DefaultSignModes)
			// this just tests our handler
			dfd := ante.NewDeductFeeDecorator(suite.accountKeeper, suite.bankKeeper, suite.feeGrantKeeper, nil)
			feeAnteHandler := sdk.ChainAnteDecorators(dfd)

			// this tests the whole stack
			anteHandlerStack := suite.anteHandler

			signer, feeAcc := stc.malleate(suite)

			fee := sdk.NewCoins(sdk.NewInt64Coin("atom", tc.fee))
			msgs := []sdk.Msg{testdata.NewTestMsg(signer.acc.GetAddress())}

			acc := suite.accountKeeper.GetAccount(suite.ctx, signer.acc.GetAddress())
			privs, accNums, seqs := []cryptotypes.PrivKey{signer.priv}, []uint64{0}, []uint64{0}
			if acc != nil {
				accNums, seqs = []uint64{acc.GetAccountNumber()}, []uint64{acc.GetSequence()}
			}

			var defaultGenTxGas uint64 = 10000000
			tx, err := genTxWithFeeGranter(protoTxCfg, msgs, fee, defaultGenTxGas, suite.ctx.ChainID(), accNums, seqs, feeAcc, privs...)
			require.NoError(t, err)
			txBytes, err := protoTxCfg.TxEncoder()(tx)
			require.NoError(t, err)
			bytesCtx := suite.ctx.WithTxBytes(txBytes)
			require.NoError(t, err)
			_, err = feeAnteHandler(bytesCtx, tx, false) // tests only feegrant ante
			if tc.valid {
				require.NoError(t, err)
			} else {
				testutil.AssertError(t, err, tc.err, tc.errMsg)
			}

			_, err = anteHandlerStack(bytesCtx, tx, false) // tests whole stack
			if tc.valid {
				require.NoError(t, err)
			} else {
				testutil.AssertError(t, err, tc.err, tc.errMsg)
			}
		})
	}
}

func genTxWithFeeGranter(gen client.TxConfig, msgs []sdk.Msg, feeAmt sdk.Coins, gas uint64, chainID string, accNums,
	accSeqs []uint64, feeGranter sdk.AccAddress, priv ...cryptotypes.PrivKey,
) (sdk.Tx, error) {
	sigs := make([]signing.SignatureV2, len(priv))

	// create a random length memo
	r := rand.New(rand.NewSource(time.Now().UnixNano()))

	memo := simulation.RandStringOfLength(r, simulation.RandIntBetween(r, 0, 100))

	signMode := signing.SignMode_SIGN_MODE_DIRECT

	// 1st round: set SignatureV2 with empty signatures, to set correct
	// signer infos.
	for i, p := range priv {
		sigs[i] = signing.SignatureV2{
			PubKey: p.PubKey(),
			Data: &signing.SingleSignatureData{
				SignMode: signMode,
			},
			Sequence: accSeqs[i],
		}
	}

	tx := gen.NewTxBuilder()
	err := tx.SetMsgs(msgs...)
	if err != nil {
		return nil, err
	}
	err = tx.SetSignatures(sigs...)
	if err != nil {
		return nil, err
	}
	tx.SetMemo(memo)
	tx.SetFeeAmount(feeAmt)
	tx.SetGasLimit(gas)
	tx.SetFeeGranter(feeGranter)

	// 2nd round: once all signer infos are set, every signer can sign.
	for i, p := range priv {
		signerData := authsign.SignerData{
			ChainID:       chainID,
			AccountNumber: accNums[i],
			Sequence:      accSeqs[i],
			PubKey:        p.PubKey(),
		}
		signBytes, err := authsign.GetSignBytesAdapter(
			context.Background(), gen.SignModeHandler(), signMode, signerData, tx.GetTx())
		if err != nil {
			panic(err)
		}
		sig, err := p.Sign(signBytes)
		if err != nil {
			panic(err)
		}
		sigs[i].Data.(*signing.SingleSignatureData).Signature = sig
		err = tx.SetSignatures(sigs...)
		if err != nil {
			panic(err)
		}
	}

	return tx.GetTx(), nil
}
