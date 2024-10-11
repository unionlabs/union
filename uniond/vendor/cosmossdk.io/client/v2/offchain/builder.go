package offchain

// TODO: remove custom off-chain builder once v2 tx builder is developed.

import (
	"errors"
	"fmt"

	"github.com/cosmos/cosmos-proto/anyutil"
	"github.com/cosmos/gogoproto/proto"
	protov2 "google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	apitx "cosmossdk.io/api/cosmos/tx/v1beta1"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type builder struct {
	cdc codec.Codec
	tx  *apitx.Tx
}

func newBuilder(cdc codec.Codec) *builder {
	return &builder{
		cdc: cdc,
		tx: &apitx.Tx{
			Body: &apitx.TxBody{},
			AuthInfo: &apitx.AuthInfo{
				Fee: &apitx.Fee{
					Amount:   nil,
					GasLimit: 0,
					Payer:    "",
					Granter:  "",
				},
			},
			Signatures: nil,
		},
	}
}

// GetTx returns the tx.
func (b *builder) GetTx() *apitx.Tx {
	return b.tx
}

// GetSigningTxData returns the necessary data to generate sign bytes.
func (b *builder) GetSigningTxData() (txsigning.TxData, error) {
	body := b.tx.Body
	authInfo := b.tx.AuthInfo

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
			Amount: coin.Amount,
		}
	}

	txSignerInfos := make([]*apitx.SignerInfo, len(authInfo.SignerInfos))
	for i, signerInfo := range authInfo.SignerInfos {
		txSignerInfo := &apitx.SignerInfo{
			PublicKey: &anypb.Any{
				TypeUrl: signerInfo.PublicKey.TypeUrl,
				Value:   signerInfo.PublicKey.Value,
			},
			Sequence: signerInfo.Sequence,
			ModeInfo: signerInfo.ModeInfo,
		}
		txSignerInfos[i] = txSignerInfo
	}

	txAuthInfo := &apitx.AuthInfo{
		SignerInfos: txSignerInfos,
		Fee: &apitx.Fee{
			Amount:   feeAmount,
			GasLimit: authInfo.Fee.GasLimit,
			Payer:    authInfo.Fee.Payer,
			Granter:  authInfo.Fee.Granter,
		},
	}

	txBody := &apitx.TxBody{
		Messages:                    msgs,
		Memo:                        body.Memo,
		TimeoutHeight:               body.TimeoutHeight,
		TimeoutTimestamp:            body.TimeoutTimestamp,
		ExtensionOptions:            extOptions,
		NonCriticalExtensionOptions: nonCriticalExtOptions,
	}
	authInfoBz, err := protov2.Marshal(b.tx.AuthInfo)
	if err != nil {
		return txsigning.TxData{}, err
	}
	bodyBz, err := protov2.Marshal(b.tx.Body)
	if err != nil {
		return txsigning.TxData{}, err
	}
	txData := txsigning.TxData{
		AuthInfo:      txAuthInfo,
		AuthInfoBytes: authInfoBz,
		Body:          txBody,
		BodyBytes:     bodyBz,
	}
	return txData, nil
}

// GetPubKeys returns the pubKeys of the tx.
func (b *builder) GetPubKeys() ([]cryptotypes.PubKey, error) { // If signer already has pubkey in context, this list will have nil in its place
	signerInfos := b.tx.AuthInfo.SignerInfos
	pks := make([]cryptotypes.PubKey, len(signerInfos))

	for i, si := range signerInfos {
		// NOTE: it is okay to leave this nil if there is no PubKey in the SignerInfo.
		// PubKey's can be left unset in SignerInfo.
		if si.PublicKey == nil {
			continue
		}
		var pk cryptotypes.PubKey
		anyPk := &codectypes.Any{
			TypeUrl: si.PublicKey.TypeUrl,
			Value:   si.PublicKey.Value,
		}
		err := b.cdc.UnpackAny(anyPk, &pk)
		if err != nil {
			return nil, err
		}
		pks[i] = pk
	}

	return pks, nil
}

// GetSignatures returns the signatures of the tx.
func (b *builder) GetSignatures() ([]OffchainSignature, error) {
	signerInfos := b.tx.AuthInfo.SignerInfos
	sigs := b.tx.Signatures
	pubKeys, err := b.GetPubKeys()
	if err != nil {
		return nil, err
	}
	n := len(signerInfos)
	res := make([]OffchainSignature, n)

	for i, si := range signerInfos {
		// handle nil signatures (in case of simulation)
		if si.ModeInfo == nil {
			res[i] = OffchainSignature{
				PubKey: pubKeys[i],
			}
		} else {
			var err error
			sigData, err := modeInfoAndSigToSignatureData(si.ModeInfo, sigs[i])
			if err != nil {
				return nil, err
			}
			// sequence number is functionally a transaction nonce and referred to as such in the SDK
			nonce := si.GetSequence()
			res[i] = OffchainSignature{
				PubKey:   pubKeys[i],
				Data:     sigData,
				Sequence: nonce,
			}
		}
	}

	return res, nil
}

// GetSigners returns the signers of the tx.
func (b *builder) GetSigners() ([][]byte, error) {
	signers, _, err := b.getSigners()
	return signers, err
}

func (b *builder) getSigners() ([][]byte, []protov2.Message, error) {
	var signers [][]byte
	seen := map[string]bool{}

	var msgsv2 []protov2.Message
	for _, msg := range b.tx.Body.Messages {
		msgv2, err := anyutil.Unpack(msg, b.cdc.InterfaceRegistry(), nil)
		if err != nil {
			return nil, nil, err
		}
		xs, err := b.cdc.InterfaceRegistry().SigningContext().GetSigners(msgv2)
		if err != nil {
			return nil, nil, err
		}

		msgsv2 = append(msgsv2, msg)

		for _, signer := range xs {
			if !seen[string(signer)] {
				signers = append(signers, signer)
				seen[string(signer)] = true
			}
		}
	}

	return signers, msgsv2, nil
}

func (b *builder) setMsgs(msgs ...proto.Message) error {
	anys := make([]*anypb.Any, len(msgs))
	for i, msg := range msgs {
		protoMsg, ok := msg.(protov2.Message)
		if !ok {
			return errors.New("message is not a proto.Message")
		}
		protov2MarshalOpts := protov2.MarshalOptions{Deterministic: true}
		bz, err := protov2MarshalOpts.Marshal(protoMsg)
		if err != nil {
			return err
		}
		anys[i] = &anypb.Any{
			TypeUrl: codectypes.MsgTypeURL(msg),
			Value:   bz,
		}
	}
	b.tx.Body.Messages = anys
	return nil
}

// SetSignatures set the signatures of the tx.
func (b *builder) SetSignatures(signatures ...OffchainSignature) error {
	n := len(signatures)
	signerInfos := make([]*apitx.SignerInfo, n)
	rawSigs := make([][]byte, n)
	var err error
	for i, sig := range signatures {
		var mi *apitx.ModeInfo
		mi, rawSigs[i], err = b.signatureDataToModeInfoAndSig(sig.Data)
		if err != nil {
			return err
		}

		pubKey, err := codectypes.NewAnyWithValue(sig.PubKey)
		if err != nil {
			return err
		}

		signerInfos[i] = &apitx.SignerInfo{
			PublicKey: &anypb.Any{
				TypeUrl: pubKey.TypeUrl,
				Value:   pubKey.Value,
			},
			ModeInfo: mi,
			Sequence: sig.Sequence,
		}
	}

	b.tx.AuthInfo.SignerInfos = signerInfos
	b.tx.Signatures = rawSigs

	return nil
}

// signatureDataToModeInfoAndSig converts a SignatureData to a ModeInfo and raw bytes signature.
func (b *builder) signatureDataToModeInfoAndSig(data SignatureData) (*apitx.ModeInfo, []byte, error) {
	if data == nil {
		return nil, nil, errors.New("empty SignatureData")
	}

	switch data := data.(type) {
	case *SingleSignatureData:
		return &apitx.ModeInfo{
			Sum: &apitx.ModeInfo_Single_{
				Single: &apitx.ModeInfo_Single{Mode: data.SignMode},
			},
		}, data.Signature, nil
	default:
		return nil, nil, fmt.Errorf("unexpected signature data type %T", data)
	}
}

// modeInfoAndSigToSignatureData converts a ModeInfo and raw bytes signature to a SignatureData.
func modeInfoAndSigToSignatureData(modeInfo *apitx.ModeInfo, sig []byte) (SignatureData, error) {
	switch modeInfoType := modeInfo.Sum.(type) {
	case *apitx.ModeInfo_Single_:
		return &SingleSignatureData{
			SignMode:  modeInfoType.Single.Mode,
			Signature: sig,
		}, nil

	default:
		return nil, fmt.Errorf("unexpected ModeInfo data type %T", modeInfo)
	}
}
