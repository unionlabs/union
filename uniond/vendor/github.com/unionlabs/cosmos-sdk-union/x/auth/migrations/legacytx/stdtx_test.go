package legacytx

import (
	"context"
	"fmt"
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/types/known/anypb"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	txsigning "cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/aminojson"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

var (
	priv = ed25519.GenPrivKey()
	addr = sdk.AccAddress(priv.PubKey().Address())
)

func TestStdSignBytes(t *testing.T) {
	type args struct {
		chainID       string
		accnum        uint64
		sequence      uint64
		timeoutHeight uint64
		fee           *txv1beta1.Fee
		msgs          []sdk.Msg
		memo          string
	}
	defaultFee := &txv1beta1.Fee{
		Amount:   []*basev1beta1.Coin{{Denom: "atom", Amount: "150"}},
		GasLimit: 100000,
	}
	msgStr := fmt.Sprintf(`{"type":"testpb/TestMsg","value":{"decField":"0.000000000000000000","signers":["%s"]}}`, addr)
	tests := []struct {
		name string
		args args
		want string
	}{
		{
			"with timeout height",
			args{"1234", 3, 6, 10, defaultFee, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[%s],"sequence":"6","timeout_height":"10"}`, msgStr),
		},
		{
			"no timeout height (omitempty)",
			args{"1234", 3, 6, 0, defaultFee, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[%s],"sequence":"6"}`, msgStr),
		},
		{
			"empty fee",
			args{"1234", 3, 6, 0, &txv1beta1.Fee{}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[],"gas":"0"},"memo":"memo","msgs":[%s],"sequence":"6"}`, msgStr),
		},
		{
			"no fee payer and fee granter (both omitempty)",
			args{"1234", 3, 6, 0, &txv1beta1.Fee{Amount: defaultFee.Amount, GasLimit: defaultFee.GasLimit}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000"},"memo":"memo","msgs":[%s],"sequence":"6"}`, msgStr),
		},
		{
			"with fee granter, no fee payer (omitempty)",
			args{"1234", 3, 6, 0, &txv1beta1.Fee{Amount: defaultFee.Amount, GasLimit: defaultFee.GasLimit, Granter: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","granter":"%s"},"memo":"memo","msgs":[%s],"sequence":"6"}`, addr, msgStr),
		},
		{
			"with fee payer, no fee granter (omitempty)",
			args{"1234", 3, 6, 0, &txv1beta1.Fee{Amount: defaultFee.Amount, GasLimit: defaultFee.GasLimit, Payer: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","payer":"%s"},"memo":"memo","msgs":[%s],"sequence":"6"}`, addr, msgStr),
		},
		{
			"with fee payer and fee granter",
			args{"1234", 3, 6, 0, &txv1beta1.Fee{Amount: defaultFee.Amount, GasLimit: defaultFee.GasLimit, Payer: addr.String(), Granter: addr.String()}, []sdk.Msg{testdata.NewTestMsg(addr)}, "memo"},
			fmt.Sprintf(`{"account_number":"3","chain_id":"1234","fee":{"amount":[{"amount":"150","denom":"atom"}],"gas":"100000","granter":"%s","payer":"%s"},"memo":"memo","msgs":[%s],"sequence":"6"}`, addr, addr, msgStr),
		},
	}
	handler := aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{
		FileResolver: proto.HybridResolver,
	})
	for i, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			anyMsgs := make([]*anypb.Any, len(tc.args.msgs))
			for j, msg := range tc.args.msgs {
				legacyAny, err := codectypes.NewAnyWithValue(msg)
				require.NoError(t, err)
				anyMsgs[j] = &anypb.Any{
					TypeUrl: legacyAny.TypeUrl,
					Value:   legacyAny.Value,
				}
			}
			got, err := handler.GetSignBytes(
				context.TODO(),
				txsigning.SignerData{
					Address:       "foo",
					ChainID:       tc.args.chainID,
					AccountNumber: tc.args.accnum,
					Sequence:      tc.args.sequence,
				},
				txsigning.TxData{
					Body: &txv1beta1.TxBody{
						Memo:          tc.args.memo,
						Messages:      anyMsgs,
						TimeoutHeight: tc.args.timeoutHeight,
					},
					AuthInfo: &txv1beta1.AuthInfo{
						Fee: tc.args.fee,
					},
				},
			)
			require.NoError(t, err)
			require.Equal(t, tc.want, string(got), "Got unexpected result on test case i: %d", i)
		})
	}
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
