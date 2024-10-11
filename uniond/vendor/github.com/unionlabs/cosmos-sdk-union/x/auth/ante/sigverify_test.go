package ante_test

import (
	"fmt"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/require"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	"cosmossdk.io/core/gas"
	"cosmossdk.io/core/header"
	gastestutil "cosmossdk.io/core/testing/gas"
	storetypes "cosmossdk.io/store/types"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256r1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	authsign "github.com/cosmos/cosmos-sdk/x/auth/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	txmodule "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestConsumeSignatureVerificationGas(t *testing.T) {
	suite := SetupTestSuite(t, true)
	params := types.DefaultParams()
	msg := []byte{1, 2, 3, 4}

	p := types.DefaultParams()
	skR1, _ := secp256r1.GenPrivKey()
	pkSet1, sigSet1 := generatePubKeysAndSignatures(5, msg, false)
	multisigKey1 := kmultisig.NewLegacyAminoPubKey(2, pkSet1)
	multisignature1 := multisig.NewMultisig(len(pkSet1))
	for i := 0; i < len(pkSet1); i++ {
		stdSig := legacytx.StdSignature{PubKey: pkSet1[i], Signature: sigSet1[i]} //nolint:staticcheck // SA1019: legacytx.StdSignature is deprecated
		sigV2, err := legacytx.StdSignatureToSignatureV2(suite.clientCtx.LegacyAmino, stdSig)
		require.NoError(t, err)
		err = multisig.AddSignatureV2(multisignature1, sigV2, pkSet1)
		require.NoError(t, err)
	}

	simulationMultiSignatureData := make([]signing.SignatureData, 0, multisigKey1.Threshold)
	for i := uint32(0); i < multisigKey1.Threshold; i++ {
		simulationMultiSignatureData = append(simulationMultiSignatureData, &signing.SingleSignatureData{})
	}
	multisigSimulationSignature := &signing.MultiSignatureData{
		Signatures: simulationMultiSignatureData,
	}

	type args struct {
		sig      signing.SignatureData
		pubkey   cryptotypes.PubKey
		params   types.Params
		malleate func(*gastestutil.MockMeter)
	}
	tests := []struct {
		name      string
		args      args
		shouldErr bool
	}{
		{
			"PubKeyEd25519",
			args{nil, ed25519.GenPrivKey().PubKey(), params, func(mm *gastestutil.MockMeter) {
				mm.EXPECT().Consume(p.SigVerifyCostED25519, "ante verify: ed25519").Times(1)
			}},
			true,
		},
		{
			"PubKeySecp256k1",
			args{nil, secp256k1.GenPrivKey().PubKey(), params, func(mm *gastestutil.MockMeter) {
				mm.EXPECT().Consume(p.SigVerifyCostSecp256k1, "ante verify: secp256k1").Times(1)
			}},
			false,
		},
		{
			"PubKeySecp256r1",
			args{nil, skR1.PubKey(), params, func(mm *gastestutil.MockMeter) {
				mm.EXPECT().Consume(p.SigVerifyCostSecp256r1(), "ante verify: secp256r1").Times(1)
			}},
			false,
		},
		{
			"Multisig",
			args{multisignature1, multisigKey1, params, func(mm *gastestutil.MockMeter) {
				// 5 signatures
				mm.EXPECT().Consume(p.SigVerifyCostSecp256k1, "ante verify: secp256k1").Times(5)
			}},
			false,
		},
		{
			"Multisig simulation",
			args{multisigSimulationSignature, multisigKey1, params, func(mm *gastestutil.MockMeter) {
				mm.EXPECT().Consume(p.SigVerifyCostSecp256k1, "ante verify: secp256k1").Times(int(multisigKey1.Threshold))
			}},
			false,
		},
		{
			"unknown key",
			args{nil, nil, params, func(mm *gastestutil.MockMeter) {}},
			true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			sigV2 := signing.SignatureV2{
				PubKey:   tt.args.pubkey,
				Data:     tt.args.sig,
				Sequence: 0, // Arbitrary account sequence
			}

			ctrl := gomock.NewController(t)
			mockMeter := gastestutil.NewMockMeter(ctrl)
			tt.args.malleate(mockMeter)
			err := ante.DefaultSigVerificationGasConsumer(mockMeter, sigV2, tt.args.params)

			if tt.shouldErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestSigVerification(t *testing.T) {
	suite := SetupTestSuite(t, true)
	suite.txBankKeeper.EXPECT().DenomMetadataV2(gomock.Any(), gomock.Any()).Return(&bankv1beta1.QueryDenomMetadataResponse{}, nil).AnyTimes()

	enabledSignModes := []signing.SignMode{signing.SignMode_SIGN_MODE_DIRECT, signing.SignMode_SIGN_MODE_TEXTUAL, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON}
	// Since TEXTUAL is not enabled by default, we create a custom TxConfig
	// here which includes it.
	cdc := codec.NewProtoCodec(suite.encCfg.InterfaceRegistry)
	txConfigOpts := authtx.ConfigOptions{
		TextualCoinMetadataQueryFn: txmodule.NewGRPCCoinMetadataQueryFn(suite.clientCtx),
		EnabledSignModes:           enabledSignModes,
		SigningOptions: &txsigning.Options{
			AddressCodec:          cdc.InterfaceRegistry().SigningContext().AddressCodec(),
			ValidatorAddressCodec: cdc.InterfaceRegistry().SigningContext().ValidatorAddressCodec(),
		},
	}
	var err error
	suite.clientCtx.TxConfig, err = authtx.NewTxConfigWithOptions(
		cdc,
		txConfigOpts,
	)
	require.NoError(t, err)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// make block height non-zero to ensure account numbers part of signBytes
	suite.ctx = suite.ctx.WithBlockHeight(1).WithHeaderInfo(header.Info{Height: 1, ChainID: suite.ctx.ChainID()})

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()
	priv2, _, addr2 := testdata.KeyTestPubAddr()
	priv3, _, addr3 := testdata.KeyTestPubAddr()

	addrs := []sdk.AccAddress{addr1, addr2, addr3}

	msgs := make([]sdk.Msg, len(addrs))
	accs := make([]sdk.AccountI, len(addrs))
	// set accounts and create msg for each address
	for i, addr := range addrs {
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr)
		require.NoError(t, acc.SetAccountNumber(uint64(i)+1000))
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		msgs[i] = testdata.NewTestMsg(addr)
		accs[i] = acc
	}

	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()

	txConfigOpts = authtx.ConfigOptions{
		TextualCoinMetadataQueryFn: txmodule.NewBankKeeperCoinMetadataQueryFn(suite.txBankKeeper),
		EnabledSignModes:           enabledSignModes,
		SigningOptions: &txsigning.Options{
			AddressCodec:          cdc.InterfaceRegistry().SigningContext().AddressCodec(),
			ValidatorAddressCodec: cdc.InterfaceRegistry().SigningContext().ValidatorAddressCodec(),
		},
	}
	anteTxConfig, err := authtx.NewTxConfigWithOptions(
		codec.NewProtoCodec(suite.encCfg.InterfaceRegistry),
		txConfigOpts,
	)
	require.NoError(t, err)
	noOpGasConsume := func(_ gas.Meter, _ signing.SignatureV2, _ types.Params) error { return nil }
	svd := ante.NewSigVerificationDecorator(suite.accountKeeper, anteTxConfig.SignModeHandler(), noOpGasConsume, nil)
	antehandler := sdk.ChainAnteDecorators(svd)
	defaultSignMode, err := authsign.APISignModeToInternal(anteTxConfig.SignModeHandler().DefaultMode())
	require.NoError(t, err)

	type testCase struct {
		name        string
		privs       []cryptotypes.PrivKey
		accNums     []uint64
		accSeqs     []uint64
		invalidSigs bool // used for testing sigverify on RecheckTx
		recheck     bool
		sigverify   bool
		shouldErr   bool
	}
	validSigs := false
	testCases := []testCase{
		{"no signers", []cryptotypes.PrivKey{}, []uint64{}, []uint64{}, validSigs, false, true, true},
		{"not enough signers", []cryptotypes.PrivKey{priv1, priv2}, []uint64{accs[0].GetAccountNumber(), accs[1].GetAccountNumber()}, []uint64{0, 0}, validSigs, false, true, true},
		{"wrong order signers", []cryptotypes.PrivKey{priv3, priv2, priv1}, []uint64{accs[2].GetAccountNumber(), accs[1].GetAccountNumber(), accs[0].GetAccountNumber()}, []uint64{0, 0, 0}, validSigs, false, true, true},
		{"wrong accnums", []cryptotypes.PrivKey{priv1, priv2, priv3}, []uint64{7, 8, 9}, []uint64{0, 0, 0}, validSigs, false, true, true},
		{"wrong sequences", []cryptotypes.PrivKey{priv1, priv2, priv3}, []uint64{accs[0].GetAccountNumber(), accs[1].GetAccountNumber(), accs[2].GetAccountNumber()}, []uint64{3, 4, 5}, validSigs, false, true, true},
		{"valid tx", []cryptotypes.PrivKey{priv1, priv2, priv3}, []uint64{accs[0].GetAccountNumber(), accs[1].GetAccountNumber(), accs[2].GetAccountNumber()}, []uint64{0, 0, 0}, validSigs, false, true, false},
		{"sigverify tx with wrong order signers", []cryptotypes.PrivKey{priv3, priv2, priv1}, []uint64{accs[0].GetAccountNumber(), accs[1].GetAccountNumber(), accs[2].GetAccountNumber()}, []uint64{0, 0, 0}, validSigs, false, true, true},
		{"skip sigverify tx with wrong order signers", []cryptotypes.PrivKey{priv3, priv2, priv1}, []uint64{accs[0].GetAccountNumber(), accs[1].GetAccountNumber(), accs[2].GetAccountNumber()}, []uint64{0, 0, 0}, validSigs, false, false, false},
		{"no err on recheck", []cryptotypes.PrivKey{priv1, priv2, priv3}, []uint64{0, 0, 0}, []uint64{0, 0, 0}, !validSigs, true, true, false},
	}

	for i, tc := range testCases {
		for _, signMode := range enabledSignModes {
			t.Run(fmt.Sprintf("%s with %s", tc.name, signMode), func(t *testing.T) {
				ctx, _ := suite.ctx.CacheContext()
				ctx = ctx.WithIsReCheckTx(tc.recheck).WithIsSigverifyTx(tc.sigverify)
				if tc.recheck {
					ctx = ctx.WithExecMode(sdk.ExecModeReCheck)
				} else {
					ctx = ctx.WithExecMode(sdk.ExecModeCheck)
				}

				suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder() // Create new txBuilder for each test

				require.NoError(t, suite.txBuilder.SetMsgs(msgs...))
				suite.txBuilder.SetFeeAmount(feeAmount)
				suite.txBuilder.SetGasLimit(gasLimit)

				tx, err := suite.CreateTestTx(ctx, tc.privs, tc.accNums, tc.accSeqs, ctx.ChainID(), signMode)
				require.NoError(t, err)
				if tc.invalidSigs {
					txSigs, _ := tx.GetSignaturesV2()
					badSig, _ := tc.privs[0].Sign([]byte("unrelated message"))
					txSigs[0] = signing.SignatureV2{
						PubKey: tc.privs[0].PubKey(),
						Data: &signing.SingleSignatureData{
							SignMode:  defaultSignMode,
							Signature: badSig,
						},
						Sequence: tc.accSeqs[0],
					}
					err := suite.txBuilder.SetSignatures(txSigs...)
					require.NoError(t, err)

					tx = suite.txBuilder.GetTx()
				}

				txBytes, err := suite.clientCtx.TxConfig.TxEncoder()(tx)
				require.NoError(t, err)
				byteCtx := ctx.WithTxBytes(txBytes)
				_, err = antehandler(byteCtx, tx, false)
				if tc.shouldErr {
					require.NotNil(t, err, "TestCase %d: %s did not error as expected", i, tc.name)
				} else {
					require.Nil(t, err, "TestCase %d: %s errored unexpectedly. Err: %v", i, tc.name, err)
					// check account sequence
					signers, err := tx.GetSigners()
					require.NoError(t, err)
					for i, signer := range signers {
						wantSeq := tc.accSeqs[i] + 1
						acc, err := suite.accountKeeper.Accounts.Get(ctx, signer)
						require.NoError(t, err)
						require.Equal(t, int(wantSeq), int(acc.GetSequence()))
					}
				}
			})
		}
	}
}

func TestSigIntegration(t *testing.T) {
	// generate private keys
	privs := []cryptotypes.PrivKey{
		secp256k1.GenPrivKey(),
		secp256k1.GenPrivKey(),
		secp256k1.GenPrivKey(),
	}

	params := types.DefaultParams()
	initialSigCost := params.SigVerifyCostSecp256k1
	initialCost, err := runSigDecorators(t, params, privs...)
	require.Nil(t, err)

	params.SigVerifyCostSecp256k1 *= 2
	doubleCost, err := runSigDecorators(t, params, privs...)
	require.Nil(t, err)

	require.Equal(t, initialSigCost*uint64(len(privs)), doubleCost-initialCost)
}

func runSigDecorators(t *testing.T, params types.Params, privs ...cryptotypes.PrivKey) (storetypes.Gas, error) {
	t.Helper()
	suite := SetupTestSuite(t, true)
	suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder()

	// Make block-height non-zero to include accNum in SignBytes
	suite.ctx = suite.ctx.WithBlockHeight(1).WithHeaderInfo(header.Info{Height: 1})
	err := suite.accountKeeper.Params.Set(suite.ctx, params)
	require.NoError(t, err)

	msgs := make([]sdk.Msg, len(privs))
	accNums := make([]uint64, len(privs))
	accSeqs := make([]uint64, len(privs))
	// set accounts and create msg for each address
	for i, priv := range privs {
		addr := sdk.AccAddress(priv.PubKey().Address())
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr)
		require.NoError(t, acc.SetAccountNumber(uint64(i)+1000))
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		msgs[i] = testdata.NewTestMsg(addr)
		accNums[i] = acc.GetAccountNumber()
		accSeqs[i] = uint64(0)
	}
	require.NoError(t, suite.txBuilder.SetMsgs(msgs...))

	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	suite.txBuilder.SetFeeAmount(feeAmount)
	suite.txBuilder.SetGasLimit(gasLimit)

	tx, err := suite.CreateTestTx(suite.ctx, privs, accNums, accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
	require.NoError(t, err)

	svd := ante.NewSigVerificationDecorator(suite.accountKeeper, suite.clientCtx.TxConfig.SignModeHandler(), ante.DefaultSigVerificationGasConsumer, nil)
	antehandler := sdk.ChainAnteDecorators(svd)

	txBytes, err := suite.clientCtx.TxConfig.TxEncoder()(tx)
	require.NoError(t, err)
	suite.ctx = suite.ctx.WithTxBytes(txBytes)

	// Determine gas consumption of antehandler with default params
	before := suite.ctx.GasMeter().GasConsumed()
	ctx, err := antehandler(suite.ctx, tx, false)
	after := ctx.GasMeter().GasConsumed()

	return after - before, err
}

func TestAnteHandlerChecks(t *testing.T) {
	suite := SetupTestSuite(t, true)
	suite.txBankKeeper.EXPECT().DenomMetadataV2(gomock.Any(), gomock.Any()).Return(&bankv1beta1.QueryDenomMetadataResponse{}, nil).AnyTimes()

	feeAmount := testdata.NewTestFeeAmount()
	gasLimit := testdata.NewTestGasLimit()
	enabledSignModes := []signing.SignMode{signing.SignMode_SIGN_MODE_DIRECT, signing.SignMode_SIGN_MODE_TEXTUAL, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON}
	// Since TEXTUAL is not enabled by default, we create a custom TxConfig
	// here which includes it.
	cdc := codec.NewProtoCodec(suite.encCfg.InterfaceRegistry)
	txConfigOpts := authtx.ConfigOptions{
		TextualCoinMetadataQueryFn: txmodule.NewGRPCCoinMetadataQueryFn(suite.clientCtx),
		EnabledSignModes:           enabledSignModes,
		SigningOptions: &txsigning.Options{
			AddressCodec:          cdc.InterfaceRegistry().SigningContext().AddressCodec(),
			ValidatorAddressCodec: cdc.InterfaceRegistry().SigningContext().ValidatorAddressCodec(),
		},
	}

	anteTxConfig, err := authtx.NewTxConfigWithOptions(
		cdc,
		txConfigOpts,
	)
	require.NoError(t, err)

	// make block height non-zero to ensure account numbers part of signBytes
	suite.ctx = suite.ctx.WithBlockHeight(1)

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()
	priv2, _, addr2 := testdata.KeyTestPubAddrSecp256R1(t)
	priv3, _, addr3 := testdata.KeyTestPubAddrED25519()

	addrs := []sdk.AccAddress{addr1, addr2, addr3}

	msgs := make([]sdk.Msg, len(addrs))
	accs := make([]sdk.AccountI, len(addrs))
	// set accounts and create msg for each address
	for i, addr := range addrs {
		acc := suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr)
		require.NoError(t, acc.SetAccountNumber(uint64(i)+1000))
		suite.accountKeeper.SetAccount(suite.ctx, acc)
		msgs[i] = testdata.NewTestMsg(addr)
		accs[i] = acc
	}

	sigVerificationDecorator := ante.NewSigVerificationDecorator(suite.accountKeeper, anteTxConfig.SignModeHandler(), ante.DefaultSigVerificationGasConsumer, nil)

	anteHandler := sdk.ChainAnteDecorators(sigVerificationDecorator)

	type testCase struct {
		name      string
		privs     []cryptotypes.PrivKey
		msg       sdk.Msg
		accNums   []uint64
		accSeqs   []uint64
		shouldErr bool
		supported bool
	}

	// Secp256r1 keys that are not on curve will fail before even doing any operation i.e when trying to get the pubkey
	testCases := []testCase{
		{"secp256k1_onCurve", []cryptotypes.PrivKey{priv1}, msgs[0], []uint64{accs[0].GetAccountNumber()}, []uint64{0}, false, true},
		{"secp256r1_onCurve", []cryptotypes.PrivKey{priv2}, msgs[1], []uint64{accs[1].GetAccountNumber()}, []uint64{0}, false, true},
		{"ed255619", []cryptotypes.PrivKey{priv3}, msgs[2], []uint64{accs[2].GetAccountNumber()}, []uint64{2}, true, false},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("%s key", tc.name), func(t *testing.T) {
			suite.txBuilder = suite.clientCtx.TxConfig.NewTxBuilder() // Create new txBuilder for each test

			require.NoError(t, suite.txBuilder.SetMsgs(tc.msg))

			suite.txBuilder.SetFeeAmount(feeAmount)
			suite.txBuilder.SetGasLimit(gasLimit)

			tx, err := suite.CreateTestTx(suite.ctx, tc.privs, tc.accNums, tc.accSeqs, suite.ctx.ChainID(), signing.SignMode_SIGN_MODE_DIRECT)
			require.NoError(t, err)

			txBytes, err := suite.clientCtx.TxConfig.TxEncoder()(tx)
			require.NoError(t, err)

			byteCtx := suite.ctx.WithTxBytes(txBytes)
			_, err = anteHandler(byteCtx, tx, true)
			if tc.shouldErr {
				require.NotNil(t, err, "TestCase %d: %s did not error as expected", i, tc.name)
				if tc.supported {
					require.ErrorContains(t, err, "not on curve")
				} else {
					require.ErrorContains(t, err, "unsupported key type")
				}
			} else {
				require.Nil(t, err, "TestCase %d: %s errored unexpectedly. Err: %v", i, tc.name, err)
			}
		})
	}
}
