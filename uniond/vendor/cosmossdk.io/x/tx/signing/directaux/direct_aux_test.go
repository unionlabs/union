package directaux_test

import (
	"context"
	"encoding/hex"
	"fmt"
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
	"cosmossdk.io/core/address"
	"cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/directaux"
)

func TestDirectAuxHandler(t *testing.T) {
	feePayerAddr := "feePayer"
	chainID := "test-chain"
	memo := "sometestmemo"
	msg, err := anyutil.New(&bankv1beta1.MsgSend{})
	require.NoError(t, err)
	accNum, accSeq := uint64(1), uint64(2) // Arbitrary account number/sequence

	pk := &secp256k1.PubKey{
		Key: make([]byte, 256),
	}
	anyPk, err := anyutil.New(pk)
	require.NoError(t, err)

	signerInfo := []*txv1beta1.SignerInfo{
		{
			PublicKey: anyPk,
			ModeInfo: &txv1beta1.ModeInfo{
				Sum: &txv1beta1.ModeInfo_Single_{
					Single: &txv1beta1.ModeInfo_Single{
						Mode: signingv1beta1.SignMode_SIGN_MODE_DIRECT_AUX,
					},
				},
			},
			Sequence: accSeq,
		},
	}

	fee := &txv1beta1.Fee{
		Amount:   []*basev1beta1.Coin{{Denom: "uatom", Amount: "1000"}},
		GasLimit: 20000,
		Payer:    feePayerAddr,
	}

	txBody := &txv1beta1.TxBody{
		Messages: []*anypb.Any{msg},
		Memo:     memo,
	}

	authInfo := &txv1beta1.AuthInfo{
		Fee:         fee,
		SignerInfos: signerInfo,
	}

	signingData := signing.SignerData{
		ChainID:       chainID,
		AccountNumber: accNum,
		Sequence:      accSeq,
		Address:       "",
		PubKey:        anyPk,
	}

	bodyBz, err := proto.Marshal(txBody)
	require.NoError(t, err)
	authInfoBz, err := proto.Marshal(authInfo)
	require.NoError(t, err)
	txData := signing.TxData{
		Body:          txBody,
		AuthInfo:      authInfo,
		AuthInfoBytes: authInfoBz,
		BodyBytes:     bodyBz,
	}
	signersCtx, err := signing.NewContext(signing.Options{
		AddressCodec:          dummyAddressCodec{},
		ValidatorAddressCodec: dummyAddressCodec{},
	})
	require.NoError(t, err)
	modeHandler, err := directaux.NewSignModeHandler(directaux.SignModeHandlerOptions{
		SignersContext: signersCtx,
	})
	require.NoError(t, err)

	t.Log("verify fee payer cannot use SIGN_MODE_DIRECT_AUX")
	feePayerSigningData := signing.SignerData{
		ChainID:       chainID,
		AccountNumber: accNum,
		Address:       feePayerAddr,
		PubKey:        anyPk,
	}
	_, err = modeHandler.GetSignBytes(context.Background(), feePayerSigningData, txData)
	require.EqualError(t, err, fmt.Sprintf("fee payer %s cannot sign with %s: unauthorized",
		feePayerAddr, signingv1beta1.SignMode_SIGN_MODE_DIRECT_AUX))

	t.Log("verifying fee payer fallback to GetSigners cannot use SIGN_MODE_DIRECT_AUX")
	feeWithNoPayer := &txv1beta1.Fee{
		Amount:   []*basev1beta1.Coin{{Denom: "uatom", Amount: "1000"}},
		GasLimit: 20000,
	}
	authInfoWithNoFeePayer := &txv1beta1.AuthInfo{
		Fee:         feeWithNoPayer,
		SignerInfos: signerInfo,
	}
	authInfoWithNoFeePayerBz, err := proto.Marshal(authInfoWithNoFeePayer)
	require.NoError(t, err)
	txDataWithNoFeePayer := signing.TxData{
		Body:          txBody,
		BodyBytes:     bodyBz,
		AuthInfo:      authInfoWithNoFeePayer,
		AuthInfoBytes: authInfoWithNoFeePayerBz,
	}
	_, err = modeHandler.GetSignBytes(context.Background(), signingData, txDataWithNoFeePayer)
	require.EqualError(t, err, fmt.Sprintf("fee payer %s cannot sign with %s: unauthorized", "",
		signingv1beta1.SignMode_SIGN_MODE_DIRECT_AUX))

	t.Log("verify GetSignBytes with generating sign bytes by marshaling signDocDirectAux")
	signBytes, err := modeHandler.GetSignBytes(context.Background(), signingData, txData)
	require.NoError(t, err)
	require.NotNil(t, signBytes)

	signDocDirectAux := &txv1beta1.SignDocDirectAux{
		BodyBytes:     bodyBz,
		PublicKey:     anyPk,
		ChainId:       chainID,
		AccountNumber: accNum,
		Sequence:      accSeq,
	}
	expectedSignBytes, err := proto.Marshal(signDocDirectAux)
	require.NoError(t, err)
	require.Equal(t, expectedSignBytes, signBytes)

	t.Log("verify GetSignBytes with false txBody data")

	signDocDirectAux.BodyBytes = []byte("dfafdasfds")
	expectedSignBytes, err = proto.Marshal(signDocDirectAux)
	require.NoError(t, err)
	require.NotEqual(t, expectedSignBytes, signBytes)
}

type dummyAddressCodec struct{}

func (d dummyAddressCodec) StringToBytes(text string) ([]byte, error) {
	return hex.DecodeString(text)
}

func (d dummyAddressCodec) BytesToString(bz []byte) (string, error) {
	return hex.EncodeToString(bz), nil
}

var _ address.Codec = dummyAddressCodec{}
