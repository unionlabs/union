package signing_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/crypto/types/multisig"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"

	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	"github.com/cosmos/cosmos-sdk/x/auth/migrations/legacytx"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestVerifySignature(t *testing.T) {
	priv, pubKey, addr := testdata.KeyTestPubAddr()
	priv1, pubKey1, addr1 := testdata.KeyTestPubAddr()

	const (
		memo    = "testmemo"
		chainId = "test-chain"
	)

	encCfg := moduletestutil.MakeTestEncodingConfig(auth.AppModuleBasic{})
	key := sdk.NewKVStoreKey(types.StoreKey)

	maccPerms := map[string][]string{
		"fee_collector":          nil,
		"mint":                   {"minter"},
		"bonded_tokens_pool":     {"burner", "staking"},
		"not_bonded_tokens_pool": {"burner", "staking"},
		"multiPerm":              {"burner", "minter", "staking"},
		"random":                 {"random"},
	}

	accountKeeper := keeper.NewAccountKeeper(
		encCfg.Codec,
		key,
		types.ProtoBaseAccount,
		maccPerms,
		"cosmos",
		types.NewModuleAddress("gov").String(),
	)

	testCtx := testutil.DefaultContextWithDB(t, key, sdk.NewTransientStoreKey("transient_test"))
	ctx := testCtx.Ctx.WithBlockHeader(tmproto.Header{})
	encCfg.Amino.RegisterConcrete(testdata.TestMsg{}, "cosmos-sdk/Test", nil)

	acc1 := accountKeeper.NewAccountWithAddress(ctx, addr)
	_ = accountKeeper.NewAccountWithAddress(ctx, addr1)
	accountKeeper.SetAccount(ctx, acc1)
	acc, err := ante.GetSignerAcc(ctx, accountKeeper, addr)
	require.NoError(t, err)

	msgs := []sdk.Msg{testdata.NewTestMsg(addr)}
	fee := legacytx.NewStdFee(50000, sdk.Coins{sdk.NewInt64Coin("atom", 150)})
	signerData := signing.SignerData{
		Address:       addr.String(),
		ChainID:       chainId,
		AccountNumber: acc.GetAccountNumber(),
		Sequence:      acc.GetSequence(),
		PubKey:        pubKey,
	}
	signBytes := legacytx.StdSignBytes(signerData.ChainID, signerData.AccountNumber, signerData.Sequence, 10, fee, msgs, memo, nil)
	signature, err := priv.Sign(signBytes)
	require.NoError(t, err)

	stdSig := legacytx.StdSignature{PubKey: pubKey, Signature: signature}
	sigV2, err := legacytx.StdSignatureToSignatureV2(encCfg.Amino, stdSig)
	require.NoError(t, err)

	handler := MakeTestHandlerMap()
	stdTx := legacytx.NewStdTx(msgs, fee, []legacytx.StdSignature{stdSig}, memo)
	stdTx.TimeoutHeight = 10
	err = signing.VerifySignature(pubKey, signerData, sigV2.Data, handler, stdTx)
	require.NoError(t, err)

	pkSet := []cryptotypes.PubKey{pubKey, pubKey1}
	multisigKey := kmultisig.NewLegacyAminoPubKey(2, pkSet)
	multisignature := multisig.NewMultisig(2)
	msgs = []sdk.Msg{testdata.NewTestMsg(addr, addr1)}
	multiSignBytes := legacytx.StdSignBytes(signerData.ChainID, signerData.AccountNumber, signerData.Sequence, 10, fee, msgs, memo, nil)

	sig1, err := priv.Sign(multiSignBytes)
	require.NoError(t, err)
	stdSig1 := legacytx.StdSignature{PubKey: pubKey, Signature: sig1}
	sig1V2, err := legacytx.StdSignatureToSignatureV2(encCfg.Amino, stdSig1)
	require.NoError(t, err)

	sig2, err := priv1.Sign(multiSignBytes)
	require.NoError(t, err)
	stdSig2 := legacytx.StdSignature{PubKey: pubKey, Signature: sig2}
	sig2V2, err := legacytx.StdSignatureToSignatureV2(encCfg.Amino, stdSig2)
	require.NoError(t, err)

	err = multisig.AddSignatureFromPubKey(multisignature, sig1V2.Data, pkSet[0], pkSet)
	require.NoError(t, err)
	err = multisig.AddSignatureFromPubKey(multisignature, sig2V2.Data, pkSet[1], pkSet)
	require.NoError(t, err)

	stdTx = legacytx.NewStdTx(msgs, fee, []legacytx.StdSignature{stdSig1, stdSig2}, memo)
	stdTx.TimeoutHeight = 10

	err = signing.VerifySignature(multisigKey, signerData, multisignature, handler, stdTx)
	require.NoError(t, err)
}
