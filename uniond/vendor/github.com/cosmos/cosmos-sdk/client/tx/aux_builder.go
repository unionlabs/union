package tx

import (
	"context"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/timestamppb"

	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	txsigning "cosmossdk.io/x/tx/signing"
	"cosmossdk.io/x/tx/signing/aminojson"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

// AuxTxBuilder is a client-side builder for creating an AuxSignerData.
type AuxTxBuilder struct {
	// msgs is used to store the sdk.Msgs that are added to the
	// TxBuilder. It's also added inside body.Messages, because:
	// - b.msgs is used for constructing the AMINO sign bz,
	// - b.body is used for constructing the DIRECT_AUX sign bz.
	msgs          []sdk.Msg
	body          *txv1beta1.TxBody
	auxSignerData *tx.AuxSignerData
}

// NewAuxTxBuilder creates a new client-side builder for constructing an
// AuxSignerData.
func NewAuxTxBuilder() AuxTxBuilder {
	return AuxTxBuilder{}
}

// SetAddress sets the aux signer's bech32 address.
func (b *AuxTxBuilder) SetAddress(addr string) {
	b.checkEmptyFields()

	b.auxSignerData.Address = addr
}

// SetMemo sets a memo in the tx.
func (b *AuxTxBuilder) SetMemo(memo string) {
	b.checkEmptyFields()

	b.body.Memo = memo
	b.auxSignerData.SignDoc.BodyBytes = nil
}

// SetTimeoutHeight sets a timeout height in the tx.
func (b *AuxTxBuilder) SetTimeoutHeight(height uint64) {
	b.checkEmptyFields()

	b.body.TimeoutHeight = height
	b.auxSignerData.SignDoc.BodyBytes = nil
}

// SetTimeoutTimestamp sets a timeout timestamp in the tx.
func (b *AuxTxBuilder) SetTimeoutTimestamp(timestamp time.Time) {
	b.checkEmptyFields()

	b.body.TimeoutTimestamp = timestamppb.New(timestamp)
	b.auxSignerData.SignDoc.BodyBytes = nil
}

// SetMsgs sets an array of Msgs in the tx.
func (b *AuxTxBuilder) SetMsgs(msgs ...sdk.Msg) error {
	anys := make([]*anypb.Any, len(msgs))
	for i, msg := range msgs {
		legacyAny, err := codectypes.NewAnyWithValue(msg)
		if err != nil {
			return err
		}
		anys[i] = &anypb.Any{
			TypeUrl: legacyAny.TypeUrl,
			Value:   legacyAny.Value,
		}
	}

	b.checkEmptyFields()

	b.msgs = msgs
	b.body.Messages = anys
	b.auxSignerData.SignDoc.BodyBytes = nil

	return nil
}

// SetAccountNumber sets the aux signer's account number in the AuxSignerData.
func (b *AuxTxBuilder) SetAccountNumber(accNum uint64) {
	b.checkEmptyFields()

	b.auxSignerData.SignDoc.AccountNumber = accNum
}

// SetChainID sets the chain id in the AuxSignerData.
func (b *AuxTxBuilder) SetChainID(chainID string) {
	b.checkEmptyFields()

	b.auxSignerData.SignDoc.ChainId = chainID
}

// SetSequence sets the aux signer's sequence in the AuxSignerData.
func (b *AuxTxBuilder) SetSequence(accSeq uint64) {
	b.checkEmptyFields()

	b.auxSignerData.SignDoc.Sequence = accSeq
}

// SetPubKey sets the aux signer's pubkey in the AuxSignerData.
func (b *AuxTxBuilder) SetPubKey(pk cryptotypes.PubKey) error {
	anyPk, err := codectypes.NewAnyWithValue(pk)
	if err != nil {
		return err
	}

	b.checkEmptyFields()
	b.auxSignerData.SignDoc.PublicKey = anyPk

	return nil
}

// SetSignMode sets the aux signer's sign mode. Allowed sign modes are
// DIRECT_AUX and LEGACY_AMINO_JSON.
func (b *AuxTxBuilder) SetSignMode(mode signing.SignMode) error {
	switch mode {
	case signing.SignMode_SIGN_MODE_DIRECT_AUX, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON:
	default:
		return sdkerrors.ErrInvalidRequest.Wrapf("AuxTxBuilder can only sign with %s or %s",
			signing.SignMode_SIGN_MODE_DIRECT_AUX, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	}

	b.auxSignerData.Mode = mode
	return nil
}

// SetSignature sets the aux signer's signature in the AuxSignerData.
func (b *AuxTxBuilder) SetSignature(sig []byte) {
	b.checkEmptyFields()

	b.auxSignerData.Sig = sig
}

// SetExtensionOptions sets the aux signer's extension options.
func (b *AuxTxBuilder) SetExtensionOptions(extOpts ...*codectypes.Any) {
	b.checkEmptyFields()

	anyExtOpts := make([]*anypb.Any, len(extOpts))
	for i, extOpt := range extOpts {
		anyExtOpts[i] = &anypb.Any{
			TypeUrl: extOpt.TypeUrl,
			Value:   extOpt.Value,
		}
	}
	b.body.ExtensionOptions = anyExtOpts
	b.auxSignerData.SignDoc.BodyBytes = nil
}

// SetNonCriticalExtensionOptions sets the aux signer's non-critical extension options.
func (b *AuxTxBuilder) SetNonCriticalExtensionOptions(extOpts ...*codectypes.Any) {
	b.checkEmptyFields()

	anyNonCritExtOpts := make([]*anypb.Any, len(extOpts))
	for i, extOpt := range extOpts {
		anyNonCritExtOpts[i] = &anypb.Any{
			TypeUrl: extOpt.TypeUrl,
			Value:   extOpt.Value,
		}
	}
	b.body.NonCriticalExtensionOptions = anyNonCritExtOpts
	b.auxSignerData.SignDoc.BodyBytes = nil
}

// GetSignBytes returns the builder's sign bytes.
func (b *AuxTxBuilder) GetSignBytes() ([]byte, error) {
	auxTx := b.auxSignerData
	if auxTx == nil {
		return nil, sdkerrors.ErrLogic.Wrap("aux tx is nil, call setters on AuxTxBuilder first")
	}

	body := b.body
	if body == nil {
		return nil, sdkerrors.ErrLogic.Wrap("tx body is nil, call setters on AuxTxBuilder first")
	}

	sd := auxTx.SignDoc
	if sd == nil {
		return nil, sdkerrors.ErrLogic.Wrap("sign doc is nil, call setters on AuxTxBuilder first")
	}

	bodyBz, err := proto.Marshal(body)
	if err != nil {
		return nil, err
	}

	sd.BodyBytes = bodyBz
	if err = sd.ValidateBasic(); err != nil {
		return nil, err
	}

	var signBz []byte
	switch b.auxSignerData.Mode {
	case signing.SignMode_SIGN_MODE_DIRECT_AUX:
		{
			signBz, err = proto.Marshal(b.auxSignerData.SignDoc)
			if err != nil {
				return nil, err
			}
		}
	case signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON:
		{
			handler := aminojson.NewSignModeHandler(aminojson.SignModeHandlerOptions{
				FileResolver: proto.HybridResolver,
			})

			auxBody := &txv1beta1.TxBody{
				Messages:         body.Messages,
				Memo:             body.Memo,
				TimeoutHeight:    body.TimeoutHeight,
				TimeoutTimestamp: body.TimeoutTimestamp,
				// AuxTxBuilder has no concern with extension options, so we set them to nil.
				// This preserves pre-PR#16025 behavior where extension options were ignored, this code path:
				// https://github.com/cosmos/cosmos-sdk/blob/ac3c209326a26b46f65a6cc6f5b5ebf6beb79b38/client/tx/aux_builder.go#L193
				// https://github.com/cosmos/cosmos-sdk/blob/ac3c209326a26b46f65a6cc6f5b5ebf6beb79b38/x/auth/migrations/legacytx/stdsign.go#L49
				ExtensionOptions:            nil,
				NonCriticalExtensionOptions: nil,
			}

			signBz, err = handler.GetSignBytes(
				context.Background(),
				txsigning.SignerData{
					Address:       b.auxSignerData.Address,
					ChainID:       b.auxSignerData.SignDoc.ChainId,
					AccountNumber: b.auxSignerData.SignDoc.AccountNumber,
					Sequence:      b.auxSignerData.SignDoc.Sequence,
					PubKey:        nil,
				},
				txsigning.TxData{
					Body: auxBody,
					AuthInfo: &txv1beta1.AuthInfo{
						SignerInfos: nil,
						// Aux signer never signs over fee.
						// For LEGACY_AMINO_JSON, we use the convention to sign
						// over empty fees.
						// ref: https://github.com/cosmos/cosmos-sdk/pull/10348
						Fee: &txv1beta1.Fee{},
					},
				},
			)
			return signBz, err
		}
	default:
		return nil, sdkerrors.ErrInvalidRequest.Wrapf("got unknown sign mode %s", b.auxSignerData.Mode)
	}

	return signBz, nil
}

// GetAuxSignerData returns the builder's AuxSignerData.
func (b *AuxTxBuilder) GetAuxSignerData() (tx.AuxSignerData, error) {
	if err := b.auxSignerData.ValidateBasic(); err != nil {
		return tx.AuxSignerData{}, err
	}

	return *b.auxSignerData, nil
}

func (b *AuxTxBuilder) checkEmptyFields() {
	if b.body == nil {
		b.body = &txv1beta1.TxBody{}
	}

	if b.auxSignerData == nil {
		b.auxSignerData = &tx.AuxSignerData{}
		if b.auxSignerData.SignDoc == nil {
			b.auxSignerData.SignDoc = &tx.SignDocDirectAux{}
		}
	}
}
