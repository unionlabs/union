package direct_test

import (
	"context"
	"testing"

	"github.com/cosmos/cosmos-proto/anyutil"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/api/cosmos/crypto/secp256k1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/direct"
)

func TestDirectModeHandler(t *testing.T) {
	memo := "sometestmemo"

	msg, err := anyutil.New(&bankv1beta1.MsgSend{})
	require.NoError(t, err)

	pk, err := anyutil.New(&secp256k1.PubKey{
		Key: make([]byte, 256),
	})
	require.NoError(t, err)

	accSeq := uint64(2) // Arbitrary account sequence

	signerInfo := []*txv1beta1.SignerInfo{
		{
			PublicKey: pk,
			ModeInfo: &txv1beta1.ModeInfo{
				Sum: &txv1beta1.ModeInfo_Single_{
					Single: &txv1beta1.ModeInfo_Single{
						Mode: signingv1beta1.SignMode_SIGN_MODE_DIRECT,
					},
				},
			},
			Sequence: accSeq,
		},
	}

	fee := &txv1beta1.Fee{Amount: []*basev1beta1.Coin{{Denom: "uatom", Amount: "1000"}}, GasLimit: 20000}
	txBody := &txv1beta1.TxBody{
		Messages: []*anypb.Any{msg},
		Memo:     memo,
	}

	authInfo := &txv1beta1.AuthInfo{
		Fee:         fee,
		SignerInfos: signerInfo,
	}

	directHandler := direct.SignModeHandler{}

	chainID := "test-chain"
	accNum := uint64(1)

	signingData := signing.SignerData{
		Address:       "",
		ChainID:       chainID,
		AccountNumber: accNum,
		PubKey:        pk,
	}

	bodyBz, err := proto.Marshal(txBody)
	require.NoError(t, err)

	authInfoBz, err := proto.Marshal(authInfo)
	require.NoError(t, err)

	txData := signing.TxData{
		Body:                       txBody,
		AuthInfo:                   authInfo,
		BodyBytes:                  bodyBz,
		AuthInfoBytes:              authInfoBz,
		BodyHasUnknownNonCriticals: false,
	}

	signBytes, err := directHandler.GetSignBytes(context.Background(), signingData, txData)
	require.NoError(t, err)
	require.NotNil(t, signBytes)

	signBytes2, err := proto.Marshal(&txv1beta1.SignDoc{
		BodyBytes:     txData.BodyBytes,
		AuthInfoBytes: txData.AuthInfoBytes,
		ChainId:       chainID,
		AccountNumber: accNum,
	})
	require.NoError(t, err)
	require.NotNil(t, signBytes2)

	require.Equal(t, signBytes2, signBytes)
}
