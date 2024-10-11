package testutil

import (
	"github.com/cosmos/cosmos-proto/anyutil"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	"cosmossdk.io/api/cosmos/crypto/secp256k1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing"
)

type HandlerArgumentOptions struct {
	ChainID       string
	Memo          string
	Msg           proto.Message
	AccNum        uint64
	AccSeq        uint64
	Fee           *txv1beta1.Fee
	SignerAddress string
}

func MakeHandlerArguments(options HandlerArgumentOptions) (signing.SignerData, signing.TxData, error) {
	pk := &secp256k1.PubKey{
		Key: make([]byte, 256),
	}
	anyPk, err := anyutil.New(pk)
	if err != nil {
		return signing.SignerData{}, signing.TxData{}, err
	}

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
			Sequence: options.AccSeq,
		},
	}

	anyMsg, err := anyutil.New(options.Msg)
	if err != nil {
		return signing.SignerData{}, signing.TxData{}, err
	}

	txBody := &txv1beta1.TxBody{
		Messages: []*anypb.Any{anyMsg},
		Memo:     options.Memo,
	}

	authInfo := &txv1beta1.AuthInfo{
		Fee:         options.Fee,
		SignerInfos: signerInfo,
	}

	protov2MarshalOpts := proto.MarshalOptions{Deterministic: true}
	bodyBz, err := protov2MarshalOpts.Marshal(txBody)
	if err != nil {
		return signing.SignerData{}, signing.TxData{}, err
	}

	authInfoBz, err := protov2MarshalOpts.Marshal(authInfo)
	if err != nil {
		return signing.SignerData{}, signing.TxData{}, err
	}

	txData := signing.TxData{
		Body:          txBody,
		AuthInfo:      authInfo,
		AuthInfoBytes: authInfoBz,
		BodyBytes:     bodyBz,
	}

	signerAddress := options.SignerAddress
	signerData := signing.SignerData{
		ChainID:       options.ChainID,
		AccountNumber: options.AccNum,
		Sequence:      options.AccSeq,
		Address:       signerAddress,
		PubKey:        anyPk,
	}

	return signerData, txData, nil
}
