package tx

import (
	"errors"
	"fmt"
	"time"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/timestamppb"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	multisigv1beta1 "cosmossdk.io/api/cosmos/crypto/multisig/v1beta1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/core/address"
	"cosmossdk.io/x/tx/decode"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	authsign "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

var (
	_ client.TxBuilder          = &builder{}
	_ ExtensionOptionsTxBuilder = &builder{}
)

func newBuilder(addressCodec address.Codec, decoder *decode.Decoder, codec codec.BinaryCodec) *builder {
	return &builder{addressCodec: addressCodec, decoder: decoder, codec: codec}
}

func newBuilderFromDecodedTx(
	addrCodec address.Codec, decoder *decode.Decoder, codec codec.BinaryCodec, decoded *gogoTxWrapper,
) (*builder, error) {
	signatures := make([][]byte, len(decoded.Tx.Signatures))
	copy(signatures, decoded.Tx.Signatures)

	sigInfos := make([]*tx.SignerInfo, len(decoded.Tx.AuthInfo.SignerInfos))
	for i, sigInfo := range decoded.Tx.AuthInfo.SignerInfos {
		modeInfoV1 := new(tx.ModeInfo)
		fromV2ModeInfo(sigInfo.ModeInfo, modeInfoV1)
		sigInfos[i] = &tx.SignerInfo{
			PublicKey: intoAnyV1([]*anypb.Any{sigInfo.PublicKey})[0],
			ModeInfo:  modeInfoV1,
			Sequence:  sigInfo.Sequence,
		}
	}

	var payer []byte
	if decoded.feePayer != nil {
		payer = decoded.feePayer
	}

	return &builder{
		addressCodec:                addrCodec,
		decoder:                     decoder,
		codec:                       codec,
		msgs:                        decoded.Messages,
		timeoutHeight:               decoded.GetTimeoutHeight(),
		granter:                     decoded.FeeGranter(),
		payer:                       payer,
		unordered:                   decoded.GetUnordered(),
		memo:                        decoded.GetMemo(),
		gasLimit:                    decoded.GetGas(),
		fees:                        decoded.GetFee(),
		signerInfos:                 sigInfos,
		signatures:                  signatures,
		extensionOptions:            decoded.GetExtensionOptions(),
		nonCriticalExtensionOptions: decoded.GetNonCriticalExtensionOptions(),
	}, nil
}

type builder struct {
	addressCodec address.Codec
	decoder      *decode.Decoder
	codec        codec.BinaryCodec

	msgs             []sdk.Msg
	timeoutHeight    uint64
	timeoutTimestamp time.Time
	granter          []byte
	payer            []byte
	unordered        bool
	memo             string
	gasLimit         uint64
	fees             sdk.Coins
	signerInfos      []*tx.SignerInfo
	signatures       [][]byte

	extensionOptions            []*codectypes.Any
	nonCriticalExtensionOptions []*codectypes.Any
}

func (w *builder) GetTx() authsign.Tx {
	buildTx, err := w.getTx()
	if err != nil {
		panic(err)
	}
	return buildTx
}

var marshalOption = proto.MarshalOptions{
	Deterministic: true,
}

func (w *builder) getTx() (*gogoTxWrapper, error) {
	anyMsgs, err := msgsV1toAnyV2(w.msgs)
	if err != nil {
		return nil, fmt.Errorf("unable to convert messages: %w", err)
	}
	body := &txv1beta1.TxBody{
		Messages:                    anyMsgs,
		Memo:                        w.memo,
		TimeoutHeight:               w.timeoutHeight,
		TimeoutTimestamp:            timestamppb.New(w.timeoutTimestamp),
		Unordered:                   w.unordered,
		ExtensionOptions:            intoAnyV2(w.extensionOptions),
		NonCriticalExtensionOptions: intoAnyV2(w.nonCriticalExtensionOptions),
	}

	fee, err := w.getFee()
	if err != nil {
		return nil, fmt.Errorf("unable to parse fee: %w", err)
	}
	authInfo := &txv1beta1.AuthInfo{
		SignerInfos: intoV2SignerInfo(w.signerInfos),
		Fee:         fee,
		Tip:         nil, // deprecated
	}

	bodyBytes, err := marshalOption.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("unable to marshal body: %w", err)
	}

	authInfoBytes, err := marshalOption.Marshal(authInfo)
	if err != nil {
		return nil, fmt.Errorf("unable to marshal auth info: %w", err)
	}

	txRawBytes, err := marshalOption.Marshal(&txv1beta1.TxRaw{
		BodyBytes:     bodyBytes,
		AuthInfoBytes: authInfoBytes,
		Signatures:    w.signatures,
	})
	if err != nil {
		return nil, fmt.Errorf("unable to marshal tx raw: %w", err)
	}

	decodedTx, err := w.decoder.Decode(txRawBytes)
	if err != nil {
		return nil, fmt.Errorf("unable to decode tx: %w", err)
	}

	return newWrapperFromDecodedTx(w.addressCodec, w.codec, decodedTx)
}

func msgsV1toAnyV2(msgs []sdk.Msg) ([]*anypb.Any, error) {
	anys := make([]*codectypes.Any, len(msgs))
	for i, msg := range msgs {
		anyMsg, err := codectypes.NewAnyWithValue(msg)
		if err != nil {
			return nil, err
		}
		anys[i] = anyMsg
	}

	return intoAnyV2(anys), nil
}

func intoV2Fees(fees sdk.Coins) []*basev1beta1.Coin {
	coins := make([]*basev1beta1.Coin, len(fees))
	for i, c := range fees {
		coins[i] = &basev1beta1.Coin{
			Denom:  c.Denom,
			Amount: c.Amount.String(),
		}
	}
	return coins
}

func (w *builder) SetMsgs(msgs ...sdk.Msg) error {
	w.msgs = msgs
	return nil
}

// SetTimeoutHeight sets the transaction's height timeout.
func (w *builder) SetTimeoutHeight(height uint64) { w.timeoutHeight = height }

func (w *builder) SetTimeoutTimestamp(timestamp time.Time) { w.timeoutTimestamp = timestamp }

func (w *builder) SetUnordered(v bool) { w.unordered = v }

func (w *builder) SetMemo(memo string) { w.memo = memo }

func (w *builder) SetGasLimit(limit uint64) { w.gasLimit = limit }

func (w *builder) SetFeeAmount(coins sdk.Coins) { w.fees = coins }

func (w *builder) SetFeePayer(feePayer sdk.AccAddress) { w.payer = feePayer }

func (w *builder) SetFeeGranter(feeGranter sdk.AccAddress) { w.granter = feeGranter }

func (w *builder) SetSignatures(signatures ...signing.SignatureV2) error {
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

func (w *builder) setSignerInfos(infos []*tx.SignerInfo) { w.signerInfos = infos }

func (w *builder) setSignatures(sigs [][]byte) { w.signatures = sigs }

func (w *builder) SetExtensionOptions(extOpts ...*codectypes.Any) { w.extensionOptions = extOpts }

func (w *builder) SetNonCriticalExtensionOptions(extOpts ...*codectypes.Any) {
	w.nonCriticalExtensionOptions = extOpts
}

func (w *builder) AddAuxSignerData(data tx.AuxSignerData) error { return errors.New("not supported") }

func (w *builder) getFee() (fee *txv1beta1.Fee, err error) {
	granterStr := ""
	if w.granter != nil {
		granterStr, err = w.addressCodec.BytesToString(w.granter)
		if err != nil {
			return nil, err
		}
	}

	payerStr := ""
	if w.payer != nil {
		payerStr, err = w.addressCodec.BytesToString(w.payer)
		if err != nil {
			return nil, err
		}
	}
	fee = &txv1beta1.Fee{
		Amount:   intoV2Fees(w.fees),
		GasLimit: w.gasLimit,
		Payer:    payerStr,
		Granter:  granterStr,
	}

	return fee, nil
}

func intoAnyV2(v1s []*codectypes.Any) []*anypb.Any {
	v2s := make([]*anypb.Any, len(v1s))
	for i, v1 := range v1s {
		v2s[i] = &anypb.Any{
			TypeUrl: v1.TypeUrl,
			Value:   v1.Value,
		}
	}
	return v2s
}

func intoV2SignerInfo(v1s []*tx.SignerInfo) []*txv1beta1.SignerInfo {
	v2s := make([]*txv1beta1.SignerInfo, len(v1s))
	for i, v1 := range v1s {
		modeInfoV2 := new(txv1beta1.ModeInfo)
		intoV2ModeInfo(v1.ModeInfo, modeInfoV2)
		v2 := &txv1beta1.SignerInfo{
			PublicKey: intoAnyV2([]*codectypes.Any{v1.PublicKey})[0],
			ModeInfo:  modeInfoV2,
			Sequence:  v1.Sequence,
		}
		v2s[i] = v2
	}
	return v2s
}

func intoV2ModeInfo(v1 *tx.ModeInfo, v2 *txv1beta1.ModeInfo) {
	// handle nil modeInfo. this is permissible through the code path:
	// https://github.com/cosmos/cosmos-sdk/blob/4a6a1e3cb8de459891cb0495052589673d14ef51/x/auth/tx/builder.go#L295
	// -> https://github.com/cosmos/cosmos-sdk/blob/b7841e3a76a38d069c1b9cb3d48368f7a67e9c26/x/auth/tx/sigs.go#L15-L17
	// when signature.Data is nil.
	if v1 == nil {
		return
	}

	switch mi := v1.Sum.(type) {
	case *tx.ModeInfo_Single_:
		v2.Sum = &txv1beta1.ModeInfo_Single_{
			Single: &txv1beta1.ModeInfo_Single{
				Mode: signingv1beta1.SignMode(v1.GetSingle().Mode),
			},
		}
	case *tx.ModeInfo_Multi_:
		multiModeInfos := v1.GetMulti().ModeInfos
		modeInfos := make([]*txv1beta1.ModeInfo, len(multiModeInfos))
		for i, modeInfo := range multiModeInfos {
			modeInfos[i] = new(txv1beta1.ModeInfo)
			intoV2ModeInfo(modeInfo, modeInfos[i])
		}
		v2.Sum = &txv1beta1.ModeInfo_Multi_{
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

func fromV2ModeInfo(v2 *txv1beta1.ModeInfo, v1 *tx.ModeInfo) {
	// Check if v2 is nil. If so, return as there's nothing to convert.
	if v2 == nil {
		return
	}

	switch mi := v2.Sum.(type) {
	case *txv1beta1.ModeInfo_Single_:
		// Convert from v2 single mode to v1 single mode
		v1.Sum = &tx.ModeInfo_Single_{
			Single: &tx.ModeInfo_Single{
				Mode: signing.SignMode(mi.Single.Mode),
			},
		}
	case *txv1beta1.ModeInfo_Multi_:
		// Convert from v2 multi mode to v1 multi mode
		multiModeInfos := mi.Multi.ModeInfos
		modeInfos := make([]*tx.ModeInfo, len(multiModeInfos))

		// Recursively convert each modeInfo
		for i, modeInfo := range multiModeInfos {
			modeInfos[i] = &tx.ModeInfo{}
			fromV2ModeInfo(modeInfo, modeInfos[i])
		}
		v1.Sum = &tx.ModeInfo_Multi_{
			Multi: &tx.ModeInfo_Multi{
				Bitarray: &cryptotypes.CompactBitArray{
					Elems:           mi.Multi.Bitarray.Elems,
					ExtraBitsStored: mi.Multi.Bitarray.ExtraBitsStored,
				},
				ModeInfos: modeInfos,
			},
		}
	}
}
