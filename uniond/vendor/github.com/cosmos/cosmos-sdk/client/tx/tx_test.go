package tx_test

import (
	gocontext "context"
	"fmt"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc"

	"cosmossdk.io/depinject"

	"github.com/cosmos/cosmos-sdk/client"
	clienttestutil "github.com/cosmos/cosmos-sdk/client/testutil"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	signingtypes "github.com/cosmos/cosmos-sdk/types/tx/signing"
	ante "github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func newTestTxConfig(t *testing.T) (client.TxConfig, codec.Codec) {
	var (
		pcdc codec.ProtoCodecMarshaler
		cdc  codec.Codec
	)
	err := depinject.Inject(clienttestutil.TestConfig, &pcdc, &cdc)
	require.NoError(t, err)
	return authtx.NewTxConfig(pcdc, authtx.DefaultSignModes), cdc
}

// mockContext is a mock client.Context to return abitrary simulation response, used to
// unit test CalculateGas.
type mockContext struct {
	gasUsed uint64
	wantErr bool
}

func (m mockContext) Invoke(grpcCtx gocontext.Context, method string, req, reply interface{}, opts ...grpc.CallOption) (err error) {
	if m.wantErr {
		return fmt.Errorf("mock err")
	}

	*(reply.(*txtypes.SimulateResponse)) = txtypes.SimulateResponse{
		GasInfo: &sdk.GasInfo{GasUsed: m.gasUsed, GasWanted: m.gasUsed},
		Result:  &sdk.Result{Data: []byte("tx data"), Log: "log"},
	}

	return nil
}

func (mockContext) NewStream(gocontext.Context, *grpc.StreamDesc, string, ...grpc.CallOption) (grpc.ClientStream, error) {
	panic("not implemented")
}

func TestCalculateGas(t *testing.T) {
	type args struct {
		mockGasUsed uint64
		mockWantErr bool
		adjustment  float64
	}

	testCases := []struct {
		name         string
		args         args
		wantEstimate uint64
		wantAdjusted uint64
		expPass      bool
	}{
		{"error", args{0, true, 1.2}, 0, 0, false},
		{"adjusted gas", args{10, false, 1.2}, 10, 12, true},
	}

	for _, tc := range testCases {
		stc := tc
		txCfg, _ := newTestTxConfig(t)

		txf := tx.Factory{}.
			WithChainID("test-chain").
			WithTxConfig(txCfg).WithSignMode(txCfg.SignModeHandler().DefaultMode())

		t.Run(stc.name, func(t *testing.T) {
			mockClientCtx := mockContext{
				gasUsed: tc.args.mockGasUsed,
				wantErr: tc.args.mockWantErr,
			}
			simRes, gotAdjusted, err := tx.CalculateGas(mockClientCtx, txf.WithGasAdjustment(stc.args.adjustment))
			if stc.expPass {
				require.NoError(t, err)
				require.Equal(t, simRes.GasInfo.GasUsed, stc.wantEstimate)
				require.Equal(t, gotAdjusted, stc.wantAdjusted)
				require.NotNil(t, simRes.Result)
			} else {
				require.Error(t, err)
				require.Nil(t, simRes)
			}
		})
	}
}

func mockTxFactory(txCfg client.TxConfig) tx.Factory {
	return tx.Factory{}.
		WithTxConfig(txCfg).
		WithAccountNumber(50).
		WithSequence(23).
		WithFees("50stake").
		WithMemo("memo").
		WithChainID("test-chain")
}

func TestBuildSimTx(t *testing.T) {
	txCfg, cdc := newTestTxConfig(t)

	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, cdc)
	require.NoError(t, err)

	path := hd.CreateHDPath(118, 0, 0).String()
	_, _, err = kb.NewMnemonic("test_key1", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)
	txf := mockTxFactory(txCfg).WithSignMode(txCfg.SignModeHandler().DefaultMode()).WithKeybase(kb)
	msg := banktypes.NewMsgSend(sdk.AccAddress("from"), sdk.AccAddress("to"), nil)
	bz, err := txf.BuildSimTx(msg)
	require.NoError(t, err)
	require.NotNil(t, bz)
}

func TestBuildUnsignedTx(t *testing.T) {
	txConfig, cdc := newTestTxConfig(t)
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, cdc)
	require.NoError(t, err)

	path := hd.CreateHDPath(118, 0, 0).String()

	_, _, err = kb.NewMnemonic("test_key1", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)
	txf := mockTxFactory(txConfig).WithKeybase(kb)
	msg := banktypes.NewMsgSend(sdk.AccAddress("from"), sdk.AccAddress("to"), nil)
	tx, err := txf.BuildUnsignedTx(msg)
	require.NoError(t, err)
	require.NotNil(t, tx)

	sigs, err := tx.GetTx().(signing.SigVerifiableTx).GetSignaturesV2()
	require.NoError(t, err)
	require.Empty(t, sigs)
}

func TestBuildUnsignedTxWithWithExtensionOptions(t *testing.T) {
	txCfg := moduletestutil.MakeBuilderTestTxConfig()
	extOpts := []*codectypes.Any{
		{
			TypeUrl: "/test",
			Value:   []byte("test"),
		},
	}
	txf := mockTxFactory(txCfg).WithExtensionOptions(extOpts...)
	msg := banktypes.NewMsgSend(sdk.AccAddress("from"), sdk.AccAddress("to"), nil)
	tx, err := txf.BuildUnsignedTx(msg)
	require.NoError(t, err)
	require.NotNil(t, tx)
	txb := tx.(*moduletestutil.TestTxBuilder)
	require.Equal(t, extOpts, txb.ExtOptions)
}

func TestMnemonicInMemo(t *testing.T) {
	txConfig, cdc := newTestTxConfig(t)
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, cdc)
	require.NoError(t, err)

	path := hd.CreateHDPath(118, 0, 0).String()

	_, seed, err := kb.NewMnemonic("test_key1", keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)

	testCases := []struct {
		name  string
		memo  string
		error bool
	}{
		{name: "bare seed", memo: seed, error: true},
		{name: "padding bare seed", memo: fmt.Sprintf("   %s", seed), error: true},
		{name: "prefixed", memo: fmt.Sprintf("%s: %s", "prefixed: ", seed), error: false},
		{name: "normal memo", memo: "this is a memo", error: false},
		{name: "empty memo", memo: "", error: false},
		{name: "invalid mnemonic", memo: strings.Repeat("egg", 24), error: false},
		{name: "caps", memo: strings.ToUpper(seed), error: true},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			txf := tx.Factory{}.
				WithTxConfig(txConfig).
				WithAccountNumber(50).
				WithSequence(23).
				WithFees("50stake").
				WithMemo(tc.memo).
				WithChainID("test-chain").
				WithKeybase(kb)

			msg := banktypes.NewMsgSend(sdk.AccAddress("from"), sdk.AccAddress("to"), nil)
			tx, err := txf.BuildUnsignedTx(msg)
			if tc.error {
				require.Error(t, err)
				require.ErrorContains(t, err, "mnemonic")
				require.Nil(t, tx)
			} else {
				require.NoError(t, err)
				require.NotNil(t, tx)
			}
		})
	}
}

func TestSign(t *testing.T) {
	txConfig, cdc := newTestTxConfig(t)
	requireT := require.New(t)
	path := hd.CreateHDPath(118, 0, 0).String()
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, cdc)
	requireT.NoError(err)

	from1 := "test_key1"
	from2 := "test_key2"

	// create a new key using a mnemonic generator and test if we can reuse seed to recreate that account
	_, seed, err := kb.NewMnemonic(from1, keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	requireT.NoError(err)
	requireT.NoError(kb.Delete(from1))
	k1, _, err := kb.NewMnemonic(from1, keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	requireT.NoError(err)

	k2, err := kb.NewAccount(from2, seed, "", path, hd.Secp256k1)
	requireT.NoError(err)

	pubKey1, err := k1.GetPubKey()
	requireT.NoError(err)
	pubKey2, err := k2.GetPubKey()
	requireT.NoError(err)
	requireT.NotEqual(pubKey1.Bytes(), pubKey2.Bytes())
	t.Log("Pub keys:", pubKey1, pubKey2)

	txfNoKeybase := mockTxFactory(txConfig)
	txfDirect := txfNoKeybase.
		WithKeybase(kb).
		WithSignMode(signingtypes.SignMode_SIGN_MODE_DIRECT)
	txfAmino := txfDirect.
		WithSignMode(signingtypes.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	addr1, err := k1.GetAddress()
	requireT.NoError(err)
	addr2, err := k2.GetAddress()
	requireT.NoError(err)
	msg1 := banktypes.NewMsgSend(addr1, sdk.AccAddress("to"), nil)
	msg2 := banktypes.NewMsgSend(addr2, sdk.AccAddress("to"), nil)
	txb, err := txfNoKeybase.BuildUnsignedTx(msg1, msg2)
	requireT.NoError(err)
	txb2, err := txfNoKeybase.BuildUnsignedTx(msg1, msg2)
	requireT.NoError(err)
	txbSimple, err := txfNoKeybase.BuildUnsignedTx(msg2)
	requireT.NoError(err)

	testCases := []struct {
		name         string
		txf          tx.Factory
		txb          client.TxBuilder
		from         string
		overwrite    bool
		expectedPKs  []cryptotypes.PubKey
		matchingSigs []int // if not nil, check matching signature against old ones.
	}{
		{
			"should fail if txf without keyring",
			txfNoKeybase, txb, from1, true, nil, nil,
		},
		{
			"should fail for non existing key",
			txfAmino, txb, "unknown", true, nil, nil,
		},
		{
			"amino: should succeed with keyring",
			txfAmino, txbSimple, from1, true,
			[]cryptotypes.PubKey{pubKey1},
			nil,
		},
		{
			"direct: should succeed with keyring",
			txfDirect, txbSimple, from1, true,
			[]cryptotypes.PubKey{pubKey1},
			nil,
		},

		/**** test double sign Amino mode ****/
		{
			"amino: should sign multi-signers tx",
			txfAmino, txb, from1, true,
			[]cryptotypes.PubKey{pubKey1},
			nil,
		},
		{
			"amino: should append a second signature and not overwrite",
			txfAmino, txb, from2, false,
			[]cryptotypes.PubKey{pubKey1, pubKey2},
			[]int{0, 0},
		},
		{
			"amino: should overwrite a signature",
			txfAmino, txb, from2, true,
			[]cryptotypes.PubKey{pubKey2},
			[]int{1, 0},
		},

		/**** test double sign Direct mode
		  signing transaction with 2 or more DIRECT signers should fail in DIRECT mode ****/
		{
			"direct: should  append a DIRECT signature with existing AMINO",
			// txb already has 1 AMINO signature
			txfDirect, txb, from1, false,
			[]cryptotypes.PubKey{pubKey2, pubKey1},
			nil,
		},
		{
			"direct: should add single DIRECT sig in multi-signers tx",
			txfDirect, txb2, from1, false,
			[]cryptotypes.PubKey{pubKey1},
			nil,
		},
		{
			"direct: should fail to append 2nd DIRECT sig in multi-signers tx",
			txfDirect, txb2, from2, false,
			[]cryptotypes.PubKey{},
			nil,
		},
		{
			"amino: should append 2nd AMINO sig in multi-signers tx with 1 DIRECT sig",
			// txb2 already has 1 DIRECT signature
			txfAmino, txb2, from2, false,
			[]cryptotypes.PubKey{},
			nil,
		},
		{
			"direct: should overwrite multi-signers tx with DIRECT sig",
			txfDirect, txb2, from1, true,
			[]cryptotypes.PubKey{pubKey1},
			nil,
		},
	}

	var prevSigs []signingtypes.SignatureV2
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			err = tx.Sign(tc.txf, tc.from, tc.txb, tc.overwrite)
			if len(tc.expectedPKs) == 0 {
				requireT.Error(err)
			} else {
				requireT.NoError(err)
				sigs := testSigners(requireT, tc.txb.GetTx(), tc.expectedPKs...)
				if tc.matchingSigs != nil {
					requireT.Equal(prevSigs[tc.matchingSigs[0]], sigs[tc.matchingSigs[1]])
				}
				prevSigs = sigs
			}
		})
	}
}

func TestPreprocessHook(t *testing.T) {
	txConfig, cdc := newTestTxConfig(t)
	requireT := require.New(t)
	path := hd.CreateHDPath(118, 0, 0).String()
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, cdc)
	requireT.NoError(err)

	from := "test_key"
	kr, _, err := kb.NewMnemonic(from, keyring.English, path, keyring.DefaultBIP39Passphrase, hd.Secp256k1)
	requireT.NoError(err)

	extVal := &testdata.Cat{
		Moniker: "einstein",
		Lives:   9,
	}
	extAny, err := codectypes.NewAnyWithValue(extVal)
	requireT.NoError(err)

	coin := sdk.Coin{
		Denom:  "atom",
		Amount: sdk.NewInt(20),
	}
	newTip := &txtypes.Tip{
		Amount: sdk.Coins{coin},
		Tipper: "galaxy",
	}

	preprocessHook := client.PreprocessTxFn(func(chainID string, key keyring.KeyType, tx client.TxBuilder) error {
		extensionBuilder, ok := tx.(authtx.ExtensionOptionsTxBuilder)
		requireT.True(ok)

		// Set new extension and tip
		extensionBuilder.SetExtensionOptions(extAny)
		tx.SetTip(newTip)

		return nil
	})

	txfDirect := mockTxFactory(txConfig).
		WithKeybase(kb).
		WithSignMode(signingtypes.SignMode_SIGN_MODE_DIRECT).
		WithPreprocessTxHook(preprocessHook)

	addr1, err := kr.GetAddress()
	requireT.NoError(err)
	msg1 := banktypes.NewMsgSend(addr1, sdk.AccAddress("to"), nil)
	msg2 := banktypes.NewMsgSend(addr2, sdk.AccAddress("to"), nil)
	txb, err := txfDirect.BuildUnsignedTx(msg1, msg2)

	err = tx.Sign(txfDirect, from, txb, false)
	requireT.NoError(err)

	// Run preprocessing
	err = txfDirect.PreprocessTx(from, txb)
	requireT.NoError(err)

	hasExtOptsTx, ok := txb.(ante.HasExtensionOptionsTx)
	requireT.True(ok)

	hasOneExt := len(hasExtOptsTx.GetExtensionOptions()) == 1
	requireT.True(hasOneExt)

	opt := hasExtOptsTx.GetExtensionOptions()[0]
	requireT.Equal(opt, extAny)

	tip := txb.GetTx().GetTip()
	requireT.Equal(tip, newTip)
}

func testSigners(require *require.Assertions, tr signing.Tx, pks ...cryptotypes.PubKey) []signingtypes.SignatureV2 {
	sigs, err := tr.GetSignaturesV2()
	require.Len(sigs, len(pks))
	require.NoError(err)
	require.Len(sigs, len(pks))
	for i := range pks {
		require.True(sigs[i].PubKey.Equals(pks[i]), "Signature is signed with a wrong pubkey. Got: %s, expected: %s", sigs[i].PubKey, pks[i])
	}
	return sigs
}
