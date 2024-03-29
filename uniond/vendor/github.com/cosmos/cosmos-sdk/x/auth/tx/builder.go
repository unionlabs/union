package tx

import (
	"bytes"
	"fmt"

	"github.com/cosmos/gogoproto/proto"
	protov2 "google.golang.org/protobuf/proto"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

// wrapper is a wrapper around the tx.Tx proto.Message which retain the raw
// body and auth_info bytes.
type wrapper struct {
	cdc codec.Codec

	tx *tx.Tx

	// bodyBz represents the protobuf encoding of TxBody. This should be encoding
	// from the client using TxRaw if the tx was decoded from the wire
	bodyBz []byte

	// authInfoBz represents the protobuf encoding of TxBody. This should be encoding
	// from the client using TxRaw if the tx was decoded from the wire
	authInfoBz []byte

	txBodyHasUnknownNonCriticals bool

	signers [][]byte
	msgsV2  []protov2.Message
}

var (
	_ authsigning.Tx             = &wrapper{}
	_ client.TxBuilder           = &wrapper{}
	_ ante.HasExtensionOptionsTx = &wrapper{}
	_ ExtensionOptionsTxBuilder  = &wrapper{}
)

// ExtensionOptionsTxBuilder defines a TxBuilder that can also set extensions.
type ExtensionOptionsTxBuilder interface {
	client.TxBuilder

	SetExtensionOptions(...*codectypes.Any)
	SetNonCriticalExtensionOptions(...*codectypes.Any)
}

func newBuilder(cdc codec.Codec) *wrapper {
	w := &wrapper{
		cdc: cdc,
		tx: &tx.Tx{
			Body: &tx.TxBody{},
			AuthInfo: &tx.AuthInfo{
				Fee: &tx.Fee{},
			},
		},
	}
	return w
}

func (w *wrapper) GetMsgs() []sdk.Msg {
	return w.tx.GetMsgs()
}

func (w *wrapper) GetMsgsV2() ([]protov2.Message, error) {
	if w.msgsV2 == nil {
		err := w.initSignersAndMsgsV2()
		if err != nil {
			return nil, err
		}
	}

	return w.msgsV2, nil
}

func (w *wrapper) ValidateBasic() error {
	if w.tx == nil {
		return fmt.Errorf("bad Tx")
	}

	if err := w.tx.ValidateBasic(); err != nil {
		return err
	}

	sigs := w.tx.Signatures
	signers, err := w.GetSigners()
	if err != nil {
		return err
	}

	if len(sigs) != len(signers) {
		return errorsmod.Wrapf(
			sdkerrors.ErrUnauthorized,
			"wrong number of signers; expected %d, got %d", len(signers), len(sigs),
		)
	}

	return nil
}

func (w *wrapper) getBodyBytes() []byte {
	if len(w.bodyBz) == 0 {
		// if bodyBz is empty, then marshal the body. bodyBz will generally
		// be set to nil whenever SetBody is called so the result of calling
		// this method should always return the correct bytes. Note that after
		// decoding bodyBz is derived from TxRaw so that it matches what was
		// transmitted over the wire
		var err error
		w.bodyBz, err = proto.Marshal(w.tx.Body)
		if err != nil {
			panic(err)
		}
	}
	return w.bodyBz
}

func (w *wrapper) getAuthInfoBytes() []byte {
	if len(w.authInfoBz) == 0 {
		// if authInfoBz is empty, then marshal the body. authInfoBz will generally
		// be set to nil whenever SetAuthInfo is called so the result of calling
		// this method should always return the correct bytes. Note that after
		// decoding authInfoBz is derived from TxRaw so that it matches what was
		// transmitted over the wire
		var err error
		w.authInfoBz, err = proto.Marshal(w.tx.AuthInfo)
		if err != nil {
			panic(err)
		}
	}
	return w.authInfoBz
}

func (w *wrapper) initSignersAndMsgsV2() error {
	var err error
	w.signers, w.msgsV2, err = w.tx.GetSigners(w.cdc)
	return err
}

func (w *wrapper) GetSigners() ([][]byte, error) {
	if w.signers == nil {
		err := w.initSignersAndMsgsV2()
		if err != nil {
			return nil, err
		}
	}
	return w.signers, nil
}

func (w *wrapper) GetPubKeys() ([]cryptotypes.PubKey, error) {
	signerInfos := w.tx.AuthInfo.SignerInfos
	pks := make([]cryptotypes.PubKey, len(signerInfos))

	for i, si := range signerInfos {
		// NOTE: it is okay to leave this nil if there is no PubKey in the SignerInfo.
		// PubKey's can be left unset in SignerInfo.
		if si.PublicKey == nil {
			continue
		}

		pkAny := si.PublicKey.GetCachedValue()
		pk, ok := pkAny.(cryptotypes.PubKey)
		if ok {
			pks[i] = pk
		} else {
			return nil, errorsmod.Wrapf(sdkerrors.ErrLogic, "Expecting PubKey, got: %T", pkAny)
		}
	}

	return pks, nil
}

func (w *wrapper) GetGas() uint64 {
	return w.tx.AuthInfo.Fee.GasLimit
}

func (w *wrapper) GetFee() sdk.Coins {
	return w.tx.AuthInfo.Fee.Amount
}

func (w *wrapper) FeePayer() []byte {
	feePayer := w.tx.AuthInfo.Fee.Payer
	if feePayer != "" {
		feePayerAddr, err := w.cdc.InterfaceRegistry().SigningContext().AddressCodec().StringToBytes(feePayer)
		if err != nil {
			panic(err)
		}
		return feePayerAddr
	}

	// use first signer as default if no payer specified
	signers, err := w.GetSigners()
	if err != nil {
		return nil
	}

	return signers[0]
}

func (w *wrapper) FeeGranter() []byte {
	return w.tx.FeeGranter(w.cdc)
}

func (w *wrapper) GetMemo() string {
	return w.tx.Body.Memo
}

// GetTimeoutHeight returns the transaction's timeout height (if set).
func (w *wrapper) GetTimeoutHeight() uint64 {
	return w.tx.Body.TimeoutHeight
}

func (w *wrapper) GetSignaturesV2() ([]signing.SignatureV2, error) {
	signerInfos := w.tx.AuthInfo.SignerInfos
	sigs := w.tx.Signatures
	pubKeys, err := w.GetPubKeys()
	if err != nil {
		return nil, err
	}
	n := len(signerInfos)
	res := make([]signing.SignatureV2, n)

	for i, si := range signerInfos {
		// handle nil signatures (in case of simulation)
		if si.ModeInfo == nil {
			res[i] = signing.SignatureV2{
				PubKey: pubKeys[i],
			}
		} else {
			var err error
			sigData, err := ModeInfoAndSigToSignatureData(si.ModeInfo, sigs[i])
			if err != nil {
				return nil, err
			}
			// sequence number is functionally a transaction nonce and referred to as such in the SDK
			nonce := si.GetSequence()
			res[i] = signing.SignatureV2{
				PubKey:   pubKeys[i],
				Data:     sigData,
				Sequence: nonce,
			}

		}
	}

	return res, nil
}

func (w *wrapper) SetMsgs(msgs ...sdk.Msg) error {
	anys, err := tx.SetMsgs(msgs)
	if err != nil {
		return err
	}

	w.tx.Body.Messages = anys

	// set bodyBz to nil because the cached bodyBz no longer matches tx.Body
	w.bodyBz = nil

	// reset signers and msgsV2
	w.signers = nil
	w.msgsV2 = nil

	return nil
}

// SetTimeoutHeight sets the transaction's height timeout.
func (w *wrapper) SetTimeoutHeight(height uint64) {
	w.tx.Body.TimeoutHeight = height

	// set bodyBz to nil because the cached bodyBz no longer matches tx.Body
	w.bodyBz = nil
}

func (w *wrapper) SetMemo(memo string) {
	w.tx.Body.Memo = memo

	// set bodyBz to nil because the cached bodyBz no longer matches tx.Body
	w.bodyBz = nil
}

func (w *wrapper) SetGasLimit(limit uint64) {
	if w.tx.AuthInfo.Fee == nil {
		w.tx.AuthInfo.Fee = &tx.Fee{}
	}

	w.tx.AuthInfo.Fee.GasLimit = limit

	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) SetFeeAmount(coins sdk.Coins) {
	if w.tx.AuthInfo.Fee == nil {
		w.tx.AuthInfo.Fee = &tx.Fee{}
	}

	w.tx.AuthInfo.Fee.Amount = coins

	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) SetFeePayer(feePayer sdk.AccAddress) {
	if w.tx.AuthInfo.Fee == nil {
		w.tx.AuthInfo.Fee = &tx.Fee{}
	}

	w.tx.AuthInfo.Fee.Payer = feePayer.String()

	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) SetFeeGranter(feeGranter sdk.AccAddress) {
	if w.tx.AuthInfo.Fee == nil {
		w.tx.AuthInfo.Fee = &tx.Fee{}
	}

	w.tx.AuthInfo.Fee.Granter = feeGranter.String()

	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) SetSignatures(signatures ...signing.SignatureV2) error {
	n := len(signatures)
	signerInfos := make([]*tx.SignerInfo, n)
	rawSigs := make([][]byte, n)

	for i, sig := range signatures {
		var (
			modeInfo *tx.ModeInfo
			pubKey   *codectypes.Any
			err      error
		)
		modeInfo, rawSigs[i] = SignatureDataToModeInfoAndSig(sig.Data)
		if sig.PubKey != nil {
			pubKey, err = codectypes.NewAnyWithValue(sig.PubKey)
			if err != nil {
				return err
			}
		}
		signerInfos[i] = &tx.SignerInfo{
			PublicKey: pubKey,
			ModeInfo:  modeInfo,
			Sequence:  sig.Sequence,
		}
	}

	w.setSignerInfos(signerInfos)
	w.setSignatures(rawSigs)

	return nil
}

func (w *wrapper) setSignerInfos(infos []*tx.SignerInfo) {
	w.tx.AuthInfo.SignerInfos = infos
	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) setSignerInfoAtIndex(index int, info *tx.SignerInfo) {
	signers, err := w.GetSigners()
	if err != nil {
		panic(err)
	}

	if w.tx.AuthInfo.SignerInfos == nil {
		w.tx.AuthInfo.SignerInfos = make([]*tx.SignerInfo, len(signers))
	}

	w.tx.AuthInfo.SignerInfos[index] = info
	// set authInfoBz to nil because the cached authInfoBz no longer matches tx.AuthInfo
	w.authInfoBz = nil
}

func (w *wrapper) setSignatures(sigs [][]byte) {
	w.tx.Signatures = sigs
}

func (w *wrapper) setSignatureAtIndex(index int, sig []byte) {
	signers, err := w.GetSigners()
	if err != nil {
		panic(err)
	}

	if w.tx.Signatures == nil {
		w.tx.Signatures = make([][]byte, len(signers))
	}

	w.tx.Signatures[index] = sig
}

func (w *wrapper) GetTx() authsigning.Tx {
	return w
}

func (w *wrapper) GetProtoTx() *tx.Tx {
	return w.tx
}

// Deprecated: AsAny extracts proto Tx and wraps it into Any.
// NOTE: You should probably use `GetProtoTx` if you want to serialize the transaction.
func (w *wrapper) AsAny() *codectypes.Any {
	return codectypes.UnsafePackAny(w.tx)
}

// WrapTx creates a TxBuilder wrapper around a tx.Tx proto message.
func WrapTx(protoTx *tx.Tx) client.TxBuilder {
	return &wrapper{
		tx: protoTx,
	}
}

func (w *wrapper) GetExtensionOptions() []*codectypes.Any {
	return w.tx.Body.ExtensionOptions
}

func (w *wrapper) GetNonCriticalExtensionOptions() []*codectypes.Any {
	return w.tx.Body.NonCriticalExtensionOptions
}

func (w *wrapper) SetExtensionOptions(extOpts ...*codectypes.Any) {
	w.tx.Body.ExtensionOptions = extOpts
	w.bodyBz = nil
}

func (w *wrapper) SetNonCriticalExtensionOptions(extOpts ...*codectypes.Any) {
	w.tx.Body.NonCriticalExtensionOptions = extOpts
	w.bodyBz = nil
}

func (w *wrapper) AddAuxSignerData(data tx.AuxSignerData) error {
	err := data.ValidateBasic()
	if err != nil {
		return err
	}

	w.bodyBz = data.SignDoc.BodyBytes

	var body tx.TxBody
	err = w.cdc.Unmarshal(w.bodyBz, &body)
	if err != nil {
		return err
	}

	if w.tx.Body.Memo != "" && w.tx.Body.Memo != body.Memo {
		return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has memo %s, got %s in AuxSignerData", w.tx.Body.Memo, body.Memo)
	}
	if w.tx.Body.TimeoutHeight != 0 && w.tx.Body.TimeoutHeight != body.TimeoutHeight {
		return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has timeout height %d, got %d in AuxSignerData", w.tx.Body.TimeoutHeight, body.TimeoutHeight)
	}
	if len(w.tx.Body.ExtensionOptions) != 0 {
		if len(w.tx.Body.ExtensionOptions) != len(body.ExtensionOptions) {
			return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has %d extension options, got %d in AuxSignerData", len(w.tx.Body.ExtensionOptions), len(body.ExtensionOptions))
		}
		for i, o := range w.tx.Body.ExtensionOptions {
			if !o.Equal(body.ExtensionOptions[i]) {
				return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has extension option %+v at index %d, got %+v in AuxSignerData", o, i, body.ExtensionOptions[i])
			}
		}
	}
	if len(w.tx.Body.NonCriticalExtensionOptions) != 0 {
		if len(w.tx.Body.NonCriticalExtensionOptions) != len(body.NonCriticalExtensionOptions) {
			return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has %d non-critical extension options, got %d in AuxSignerData", len(w.tx.Body.NonCriticalExtensionOptions), len(body.NonCriticalExtensionOptions))
		}
		for i, o := range w.tx.Body.NonCriticalExtensionOptions {
			if !o.Equal(body.NonCriticalExtensionOptions[i]) {
				return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has non-critical extension option %+v at index %d, got %+v in AuxSignerData", o, i, body.NonCriticalExtensionOptions[i])
			}
		}
	}
	if len(w.tx.Body.Messages) != 0 {
		if len(w.tx.Body.Messages) != len(body.Messages) {
			return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has %d Msgs, got %d in AuxSignerData", len(w.tx.Body.Messages), len(body.Messages))
		}
		for i, o := range w.tx.Body.Messages {
			if !o.Equal(body.Messages[i]) {
				return sdkerrors.ErrInvalidRequest.Wrapf("TxBuilder has Msg %+v at index %d, got %+v in AuxSignerData", o, i, body.Messages[i])
			}
		}
	}

	w.SetMemo(body.Memo)
	w.SetTimeoutHeight(body.TimeoutHeight)
	w.SetExtensionOptions(body.ExtensionOptions...)
	w.SetNonCriticalExtensionOptions(body.NonCriticalExtensionOptions...)
	msgs := make([]sdk.Msg, len(body.Messages))
	for i, msgAny := range body.Messages {
		msgs[i] = msgAny.GetCachedValue().(sdk.Msg)
	}
	err = w.SetMsgs(msgs...)
	if err != nil {
		return err
	}

	// Get the aux signer's index in GetSigners.
	signerIndex := -1
	signers, err := w.GetSigners()
	if err != nil {
		return err
	}

	for i, signer := range signers {
		addrBz, err := w.cdc.InterfaceRegistry().SigningContext().AddressCodec().StringToBytes(data.Address)
		if err != nil {
			return err
		}
		if bytes.Equal(signer, addrBz) {
			signerIndex = i
		}
	}
	if signerIndex < 0 {
		return sdkerrors.ErrLogic.Wrapf("address %s is not a signer", data.Address)
	}

	w.setSignerInfoAtIndex(signerIndex, &tx.SignerInfo{
		PublicKey: data.SignDoc.PublicKey,
		ModeInfo:  &tx.ModeInfo{Sum: &tx.ModeInfo_Single_{Single: &tx.ModeInfo_Single{Mode: data.Mode}}},
		Sequence:  data.SignDoc.Sequence,
	})
	w.setSignatureAtIndex(signerIndex, data.Sig)

	return nil
}
