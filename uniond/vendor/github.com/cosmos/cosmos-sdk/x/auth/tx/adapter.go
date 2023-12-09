package tx

import (
	"google.golang.org/protobuf/types/known/anypb"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	multisigv1beta1 "cosmossdk.io/api/cosmos/crypto/multisig/v1beta1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/types/tx"
)

// GetSigningTxData returns an x/tx/signing.TxData representation of a transaction for use in the signing
// API defined in x/tx.  The reason for all of this conversion is that x/tx depends on the protoreflect API
// defined in google.golang.org/protobuf while x/auth/tx depends on the legacy proto API defined in
// github.com/gogo/protobuf and the downstream SDK fork of that library, github.com/cosmos/gogoproto.
// Therefore we need to convert between the two APIs.
func (w *wrapper) GetSigningTxData() txsigning.TxData {
	body := w.tx.Body
	authInfo := w.tx.AuthInfo

	msgs := make([]*anypb.Any, len(body.Messages))
	for i, msg := range body.Messages {
		msgs[i] = &anypb.Any{
			TypeUrl: msg.TypeUrl,
			Value:   msg.Value,
		}
	}

	extOptions := make([]*anypb.Any, len(body.ExtensionOptions))
	for i, extOption := range body.ExtensionOptions {
		extOptions[i] = &anypb.Any{
			TypeUrl: extOption.TypeUrl,
			Value:   extOption.Value,
		}
	}

	nonCriticalExtOptions := make([]*anypb.Any, len(body.NonCriticalExtensionOptions))
	for i, extOption := range body.NonCriticalExtensionOptions {
		nonCriticalExtOptions[i] = &anypb.Any{
			TypeUrl: extOption.TypeUrl,
			Value:   extOption.Value,
		}
	}

	feeCoins := authInfo.Fee.Amount
	feeAmount := make([]*basev1beta1.Coin, len(feeCoins))
	for i, coin := range feeCoins {
		feeAmount[i] = &basev1beta1.Coin{
			Denom:  coin.Denom,
			Amount: coin.Amount.String(),
		}
	}

	txSignerInfos := make([]*txv1beta1.SignerInfo, len(authInfo.SignerInfos))
	for i, signerInfo := range authInfo.SignerInfos {
		modeInfo := &txv1beta1.ModeInfo{}
		adaptModeInfo(signerInfo.ModeInfo, modeInfo)
		txSignerInfo := &txv1beta1.SignerInfo{
			PublicKey: &anypb.Any{
				TypeUrl: signerInfo.PublicKey.TypeUrl,
				Value:   signerInfo.PublicKey.Value,
			},
			Sequence: signerInfo.Sequence,
			ModeInfo: modeInfo,
		}
		txSignerInfos[i] = txSignerInfo
	}

	txAuthInfo := &txv1beta1.AuthInfo{
		SignerInfos: txSignerInfos,
		Fee: &txv1beta1.Fee{
			Amount:   feeAmount,
			GasLimit: authInfo.Fee.GasLimit,
			Payer:    authInfo.Fee.Payer,
			Granter:  authInfo.Fee.Granter,
		},
	}

	txBody := &txv1beta1.TxBody{
		Messages:                    msgs,
		Memo:                        body.Memo,
		TimeoutHeight:               body.TimeoutHeight,
		ExtensionOptions:            extOptions,
		NonCriticalExtensionOptions: nonCriticalExtOptions,
	}
	txData := txsigning.TxData{
		AuthInfo:      txAuthInfo,
		AuthInfoBytes: w.getAuthInfoBytes(),
		Body:          txBody,
		BodyBytes:     w.getBodyBytes(),
	}
	return txData
}

func adaptModeInfo(legacy *tx.ModeInfo, res *txv1beta1.ModeInfo) {
	// handle nil modeInfo. this is permissible through the code path:
	// https://github.com/cosmos/cosmos-sdk/blob/4a6a1e3cb8de459891cb0495052589673d14ef51/x/auth/tx/builder.go#L295
	// -> https://github.com/cosmos/cosmos-sdk/blob/b7841e3a76a38d069c1b9cb3d48368f7a67e9c26/x/auth/tx/sigs.go#L15-L17
	// when signature.Data is nil.
	if legacy == nil {
		return
	}

	switch mi := legacy.Sum.(type) {
	case *tx.ModeInfo_Single_:
		res.Sum = &txv1beta1.ModeInfo_Single_{
			Single: &txv1beta1.ModeInfo_Single{
				Mode: signingv1beta1.SignMode(legacy.GetSingle().Mode),
			},
		}
	case *tx.ModeInfo_Multi_:
		multiModeInfos := legacy.GetMulti().ModeInfos
		modeInfos := make([]*txv1beta1.ModeInfo, len(multiModeInfos))
		for _, modeInfo := range multiModeInfos {
			adaptModeInfo(modeInfo, &txv1beta1.ModeInfo{})
		}
		res.Sum = &txv1beta1.ModeInfo_Multi_{
			Multi: &txv1beta1.ModeInfo_Multi{
				Bitarray: &multisigv1beta1.CompactBitArray{
					Elems:           mi.Multi.Bitarray.Elems,
					ExtraBitsStored: mi.Multi.Bitarray.ExtraBitsStored,
				},
				ModeInfos: modeInfos,
			},
		}
	}
}
