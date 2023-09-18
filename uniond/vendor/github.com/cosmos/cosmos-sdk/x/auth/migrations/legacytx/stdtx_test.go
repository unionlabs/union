package legacytx

import (
	"fmt"
	"testing"

	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

var (
	priv = ed25519.GenPrivKey()
	addr = sdk.AccAddress(priv.PubKey().Address())
)

func init() {
	amino := codec.NewLegacyAmino()
	RegisterLegacyAminoCodec(amino)
}

// Deprecated, use fee amount and gas limit separately on TxBuilder.
func NewTestStdFee() StdFee {
	return NewStdFee(100000,
		sdk.NewCoins(sdk.NewInt64Coin("atom", 150)),
	)
}

// Deprecated, use TxBuilder.
func NewTestTx(ctx sdk.Context, msgs []sdk.Msg, privs []cryptotypes.PrivKey, accNums []uint64, seqs []uint64, timeout uint64, fee StdFee) sdk.Tx {
	sigs := make([]StdSignature, len(privs))
	for i, priv := range privs {
		signBytes := StdSignBytes(ctx.ChainID(), accNums[i], seqs[i], timeout, fee, msgs, "", nil)

		sig, err := priv.Sign(signBytes)
		if err != nil {
			panic(err)
		}

		sigs[i] = StdSignature{PubKey: priv.PubKey(), Signature: sig}
	}

	tx := NewStdTx(msgs, fee, sigs, "")
	return tx
}

func TestStdTx(t *testing.T) {
	msgs := []sdk.Msg{testdata.NewTestMsg(addr)}
	fee := NewTestStdFee()
	sigs := []StdSignature{}

	tx := NewStdTx(msgs, fee, sigs, "")
	require.Equal(t, msgs, tx.GetMsgs())
	require.Equal(t, sigs, tx.Signatures)

	feePayer := tx.GetSigners()[0]
	require.Equal(t, addr, feePayer)

	feeGranter := tx.FeeGranter()
	require.Empty(t, feeGranter)
}

func TestStdSignBytes(t *testing.T) {
	type args struct {
		chainID       string
		accnum        uint64
		sequence      uint64
		timeoutHeight uint64
		fee           StdFee
		msgs          []sdk.Msg
		memo          string
		tip           *tx.Tip
	}
	defaultFee := NewTestStdFee()
	defaultTip := &tx.Tip{Tipper: addr.String(), Amount: sdk.NewCoins(sdk.NewInt64Coin("tiptoken", 150))}
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			"with timeout height",
			args{"1234", 3, 6, 10, defaultFee, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[["%s"]],"sequence":"6","timeout_height":"10"}`, addr),
		},
		{
			"no timeout height (omitempty)",
			args{"1234", 3, 6, 0, defaultFee, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr),
		},
		{
			"empty fee",
			args{"1234", 3, 6, 0, StdFee{}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr),
		},
		{
			"no fee payer and fee granter (both omitempty)",
			args{"1234", 3, 6, 0, StdFee{Amount: defaultFee.Amount, Gas: defaultFee.Gas}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr),
		},
		{
			"with fee granter, no fee payer (omitempty)",
			args{"1234", 3, 6, 0, StdFee{Amount: defaultFee.Amount, Gas: defaultFee.Gas, Granter: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","granter":"%s"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr, addr),
		},
		{
			"with fee payer, no fee granter (omitempty)",
			args{"1234", 3, 6, 0, StdFee{Amount: defaultFee.Amount, Gas: defaultFee.Gas, Payer: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","payer":"%s"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr, addr),
		},
		{
			"with fee payer and fee granter",
			args{"1234", 3, 6, 0, StdFee{Amount: defaultFee.Amount, Gas: defaultFee.Gas, Payer: addr.String(), Granter: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", nil},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","granter":"%s","payer":"%s"},"memo":"memo","msgs":[["%s"]],"sequence":"6"}`, addr, addr, addr),
		},
		{
			"no fee, with tip",
			args{"1234", 3, 6, 0, StdFee{}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", defaultTip},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[["%s"]],"sequence":"6","tip":{"amount":[{"amount":"150","denom":"tiptoken"}],"tipper":"%s"}}`, addr, addr),
		},
		{
			"with fee and with tip",
			args{"1234", 3, 6, 0, StdFee{Amount: defaultFee.Amount, Gas: defaultFee.Gas, Payer: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", defaultTip},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","payer":"%s"},"memo":"memo","msgs":[["%s"]],"sequence":"6","tip":{"amount":[{"amount":"150","denom":"tiptoken"}],"tipper":"%s"}}`, addr, addr, addr),
		},
		{
			"with empty tip (but not nil), tipper cannot be empty",
			args{"1234", 3, 6, 0, defaultFee, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo", &tx.Tip{Tipper: addr.String()}},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[["%s"]],"sequence":"6","tip":{"amount":[],"tipper":"%s"}}`, addr, addr),
		},
	}
	for i, tc := range tests {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			got := string(StdSignBytes(tc.args.chainID, tc.args.accnum, tc.args.sequence, tc.args.timeoutHeight, tc.args.fee, tc.args.msgs, tc.args.memo, tc.args.tip))
			require.Equal(t, tc.want, got, "Got unexpected result on test case i: %d", i)
		})
	}
}

func TestTxValidateBasic(t *testing.T) {
	ctx := sdk.NewContext(nil, tmproto.Header{ChainID: "mychainid"}, false, log.NewNopLogger())

	// keys and addresses
	priv1, _, addr1 := testdata.KeyTestPubAddr()
	priv2, _, addr2 := testdata.KeyTestPubAddr()

	// msg and signatures
	msg1 := testdata.NewTestMsg(addr1, addr2)
	fee := NewTestStdFee()

	msgs := []sdk.Msg{msg1}

	// require to fail validation upon invalid fee
	badFee := NewTestStdFee()
	badFee.Amount[0].Amount = sdk.NewInt(-5)
	tx := NewTestTx(ctx, nil, nil, nil, nil, 0, badFee)

	err := tx.ValidateBasic()
	require.Error(t, err)
	_, code, _ := sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrInsufficientFee.ABCICode(), code)

	// require to fail validation when no signatures exist
	privs, accNums, seqs := []cryptotypes.PrivKey{}, []uint64{}, []uint64{}
	tx = NewTestTx(ctx, msgs, privs, accNums, seqs, 0, fee)

	err = tx.ValidateBasic()
	require.Error(t, err)
	_, code, _ = sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrNoSignatures.ABCICode(), code)

	// require to fail validation when signatures do not match expected signers
	privs, accNums, seqs = []cryptotypes.PrivKey{priv1}, []uint64{0, 1}, []uint64{0, 0}
	tx = NewTestTx(ctx, msgs, privs, accNums, seqs, 0, fee)

	err = tx.ValidateBasic()
	require.Error(t, err)
	_, code, _ = sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrUnauthorized.ABCICode(), code)

	// require to fail with invalid gas supplied
	badFee = NewTestStdFee()
	badFee.Gas = 9223372036854775808
	tx = NewTestTx(ctx, nil, nil, nil, nil, 0, badFee)

	err = tx.ValidateBasic()
	require.Error(t, err)
	_, code, _ = sdkerrors.ABCIInfo(err, false)
	require.Equal(t, sdkerrors.ErrInvalidRequest.ABCICode(), code)

	// require to pass when above criteria are matched
	privs, accNums, seqs = []cryptotypes.PrivKey{priv1, priv2}, []uint64{0, 1}, []uint64{0, 0}
	tx = NewTestTx(ctx, msgs, privs, accNums, seqs, 0, fee)

	err = tx.ValidateBasic()
	require.NoError(t, err)
}

func TestDefaultTxEncoder(t *testing.T) {
	cdc := codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)
	cdc.RegisterConcrete(testdata.TestMsg{}, "cosmos-sdk/Test", nil)
	encoder := DefaultTxEncoder(cdc)

	msgs := []sdk.Msg{testdata.NewTestMsg(addr)}
	fee := NewTestStdFee()
	sigs := []StdSignature{}

	tx := NewStdTx(msgs, fee, sigs, "")

	cdcBytes, err := cdc.Marshal(tx)

	require.NoError(t, err)
	encoderBytes, err := encoder(tx)

	require.NoError(t, err)
	require.Equal(t, cdcBytes, encoderBytes)
}

func TestSignatureV2Conversions(t *testing.T) {
	_, pubKey, _ := testdata.KeyTestPubAddr()
	cdc := codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)
	dummy := []byte("dummySig")
	sig := StdSignature{PubKey: pubKey, Signature: dummy}

	sigV2, err := StdSignatureToSignatureV2(cdc, sig)
	require.NoError(t, err)
	require.Equal(t, pubKey, sigV2.PubKey)
	require.Equal(t, &signing.SingleSignatureData{
		SignMode:  signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		Signature: dummy,
	}, sigV2.Data)

	sigBz, err := SignatureDataToAminoSignature(cdc, sigV2.Data)
	require.NoError(t, err)
	require.Equal(t, dummy, sigBz)

	// multisigs
	_, pubKey2, _ := testdata.KeyTestPubAddr()
	multiPK := kmultisig.NewLegacyAminoPubKey(1, []cryptotypes.PubKey{
		pubKey, pubKey2,
	})
	dummy2 := []byte("dummySig2")
	bitArray := cryptotypes.NewCompactBitArray(2)
	bitArray.SetIndex(0, true)
	bitArray.SetIndex(1, true)
	msigData := &signing.MultiSignatureData{
		BitArray: bitArray,
		Signatures: []signing.SignatureData{
			&signing.SingleSignatureData{
				SignMode:  signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
				Signature: dummy,
			},
			&signing.SingleSignatureData{
				SignMode:  signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
				Signature: dummy2,
			},
		},
	}

	msig, err := SignatureDataToAminoSignature(cdc, msigData)
	require.NoError(t, err)

	sigV2, err = StdSignatureToSignatureV2(cdc, StdSignature{
		PubKey:    multiPK,
		Signature: msig,
	})
	require.NoError(t, err)
	require.Equal(t, multiPK, sigV2.PubKey)
	require.Equal(t, msigData, sigV2.Data)
}

func TestGetSignaturesV2(t *testing.T) {
	_, pubKey, _ := testdata.KeyTestPubAddr()
	dummy := []byte("dummySig")

	cdc := codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)
	cryptocodec.RegisterCrypto(cdc)

	fee := NewStdFee(50000, sdk.Coins{sdk.NewInt64Coin("atom", 150)})
	sig := StdSignature{PubKey: pubKey, Signature: dummy}
	stdTx := NewStdTx([]sdk.Msg{testdata.NewTestMsg()}, fee, []StdSignature{sig}, "testsigs")

	sigs, err := stdTx.GetSignaturesV2()
	require.Nil(t, err)
	require.Equal(t, len(sigs), 1)

	require.Equal(t, cdc.MustMarshal(sigs[0].PubKey), cdc.MustMarshal(sig.GetPubKey()))
	require.Equal(t, sigs[0].Data, &signing.SingleSignatureData{
		SignMode:  signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON,
		Signature: sig.GetSignature(),
	})
}
