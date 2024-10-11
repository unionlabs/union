package tx

import (
	"fmt"
	"reflect"
	"strings"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/anypb"

	"cosmossdk.io/core/address"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math"
	"cosmossdk.io/x/tx/decode"
	txsigning "cosmossdk.io/x/tx/signing"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	txtypes "github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	authsigning "github.com/cosmos/cosmos-sdk/x/auth/signing"
)

func newWrapperFromDecodedTx(
	addrCodec address.Codec, cdc codec.BinaryCodec, decodedTx *decode.DecodedTx,
) (*gogoTxWrapper, error) {
	var (
		fees = make(sdk.Coins, len(decodedTx.Tx.AuthInfo.Fee.Amount))
		err  error
	)
	for i, fee := range decodedTx.Tx.AuthInfo.Fee.Amount {
		amtInt, ok := math.NewIntFromString(fee.Amount)
		if !ok {
			return nil, fmt.Errorf("invalid fee coin amount at index %d: %s", i, fee.Amount)
		}
		if err = sdk.ValidateDenom(fee.Denom); err != nil {
			return nil, fmt.Errorf("invalid fee coin denom at index %d: %w", i, err)
		}
		fees[i] = sdk.Coin{
			Denom:  fee.Denom,
			Amount: amtInt,
		}
	}
	if !fees.IsSorted() {
		return nil, fmt.Errorf("invalid not sorted tx fees: %s", fees.String())
	}
	// set fee payer
	var feePayer []byte
	if len(decodedTx.Signers) != 0 {
		feePayer = decodedTx.Signers[0]
		if decodedTx.Tx.AuthInfo.Fee.Payer != "" {
			feePayer, err = addrCodec.StringToBytes(decodedTx.Tx.AuthInfo.Fee.Payer)
			if err != nil {
				return nil, err
			}
		}
	}

	// fee granter
	var feeGranter []byte
	if decodedTx.Tx.AuthInfo.Fee.Granter != "" {
		feeGranter, err = addrCodec.StringToBytes(decodedTx.Tx.AuthInfo.Fee.Granter)
		if err != nil {
			return nil, err
		}
	}

	// reflectMsgs
	reflectMsgs := make([]protoreflect.Message, len(decodedTx.DynamicMessages))
	for i, msg := range decodedTx.DynamicMessages {
		reflectMsgs[i] = msg.ProtoReflect()
	}

	return &gogoTxWrapper{
		DecodedTx:   decodedTx,
		cdc:         cdc,
		reflectMsgs: reflectMsgs,
		fees:        fees,
		feePayer:    feePayer,
		feeGranter:  feeGranter,
	}, nil
}

// gogoTxWrapper is a gogoTxWrapper around the tx.Tx proto.Message which retain the raw
// body and auth_info bytes.
type gogoTxWrapper struct {
	*decode.DecodedTx

	cdc codec.BinaryCodec

	reflectMsgs []protoreflect.Message
	fees        sdk.Coins
	feePayer    []byte
	feeGranter  []byte
}

func (w *gogoTxWrapper) String() string { return w.Tx.String() }

var (
	_ authsigning.Tx             = &gogoTxWrapper{}
	_ ante.HasExtensionOptionsTx = &gogoTxWrapper{}
)

// ExtensionOptionsTxBuilder defines a TxBuilder that can also set extensions.
type ExtensionOptionsTxBuilder interface {
	client.TxBuilder

	SetExtensionOptions(...*codectypes.Any)
	SetNonCriticalExtensionOptions(...*codectypes.Any)
}

func (w *gogoTxWrapper) GetMsgs() []sdk.Msg {
	return w.Messages
}

func (w *gogoTxWrapper) GetReflectMessages() ([]protoreflect.Message, error) {
	return w.reflectMsgs, nil
}

func (w *gogoTxWrapper) ValidateBasic() error {
	if len(w.Tx.Signatures) == 0 {
		return sdkerrors.ErrNoSignatures.Wrapf("empty signatures")
	}
	if len(w.Signers) != len(w.Tx.Signatures) {
		return sdkerrors.ErrUnauthorized.Wrapf("invalid number of signatures: got %d signatures and %d signers", len(w.Tx.Signatures), len(w.Signers))
	}
	return nil
}

func (w *gogoTxWrapper) GetSigners() ([][]byte, error) {
	return w.Signers, nil
}

func (w *gogoTxWrapper) GetPubKeys() ([]cryptotypes.PubKey, error) {
	signerInfos := w.Tx.AuthInfo.SignerInfos
	pks := make([]cryptotypes.PubKey, len(signerInfos))

	for i, si := range signerInfos {
		// NOTE: it is okay to leave this nil if there is no PubKey in the SignerInfo.
		// PubKey's can be left unset in SignerInfo.
		if si.PublicKey == nil {
			continue
		}
		maybePK, err := decodeFromAny(w.cdc, si.PublicKey)
		if err != nil {
			return nil, err
		}
		pk, ok := maybePK.(cryptotypes.PubKey)
		if ok {
			pks[i] = pk
		} else {
			return nil, errorsmod.Wrapf(sdkerrors.ErrLogic, "expecting pubkey, got: %T", maybePK)
		}
	}

	return pks, nil
}

func (w *gogoTxWrapper) GetGas() uint64 {
	return w.Tx.AuthInfo.Fee.GasLimit
}

func (w *gogoTxWrapper) GetFee() sdk.Coins { return w.fees }

func (w *gogoTxWrapper) FeePayer() []byte { return w.feePayer }

func (w *gogoTxWrapper) FeeGranter() []byte { return w.feeGranter }

func (w *gogoTxWrapper) GetMemo() string { return w.Tx.Body.Memo }

// GetTimeoutHeight returns the transaction's timeout height (if set).
func (w *gogoTxWrapper) GetTimeoutHeight() uint64 { return w.Tx.Body.TimeoutHeight }

// GetTimeoutTimeStamp returns the transaction's timeout timestamp (if set).
func (w *gogoTxWrapper) GetTimeoutTimeStamp() time.Time {
	return w.Tx.Body.TimeoutTimestamp.AsTime()
}

// GetUnordered returns the transaction's unordered field (if set).
func (w *gogoTxWrapper) GetUnordered() bool { return w.Tx.Body.Unordered }

// GetSignaturesV2 returns the signatures of the Tx.
func (w *gogoTxWrapper) GetSignaturesV2() ([]signing.SignatureV2, error) {
	signerInfos := w.Tx.AuthInfo.SignerInfos
	sigs := w.Tx.Signatures
	pubKeys, err := w.GetPubKeys()
	if err != nil {
		return nil, err
	}
	n := len(signerInfos)
	res := make([]signing.SignatureV2, n)

	for i, si := range signerInfos {
		// handle nil signatures (in case of simulation)
		if si.ModeInfo == nil || si.ModeInfo.Sum == nil {
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

// GetSigningTxData returns an x/tx/signing.TxData representation of a transaction for use in the signing
// TODO: evaluate if this is even needed considering we have decoded tx.
func (w *gogoTxWrapper) GetSigningTxData() txsigning.TxData {
	return txsigning.TxData{
		Body:                       w.Tx.Body,
		AuthInfo:                   w.Tx.AuthInfo,
		BodyBytes:                  w.TxRaw.BodyBytes,
		AuthInfoBytes:              w.TxRaw.AuthInfoBytes,
		BodyHasUnknownNonCriticals: w.TxBodyHasUnknownNonCriticals,
	}
}

func (w *gogoTxWrapper) GetExtensionOptions() []*codectypes.Any {
	return intoAnyV1(w.Tx.Body.ExtensionOptions)
}

func (w *gogoTxWrapper) GetNonCriticalExtensionOptions() []*codectypes.Any {
	return intoAnyV1(w.Tx.Body.NonCriticalExtensionOptions)
}

func (w *gogoTxWrapper) AsTx() (*txtypes.Tx, error) {
	body := new(txtypes.TxBody)
	authInfo := new(txtypes.AuthInfo)

	err := w.cdc.Unmarshal(w.TxRaw.BodyBytes, body)
	if err != nil {
		return nil, err
	}
	err = w.cdc.Unmarshal(w.TxRaw.AuthInfoBytes, authInfo)
	if err != nil {
		return nil, err
	}
	return &txtypes.Tx{
		Body:       body,
		AuthInfo:   authInfo,
		Signatures: w.TxRaw.Signatures,
	}, nil
}

func (w *gogoTxWrapper) AsTxRaw() (*txtypes.TxRaw, error) {
	return &txtypes.TxRaw{
		BodyBytes:     w.TxRaw.BodyBytes,
		AuthInfoBytes: w.TxRaw.AuthInfoBytes,
		Signatures:    w.TxRaw.Signatures,
	}, nil
}

func intoAnyV1(v2s []*anypb.Any) []*codectypes.Any {
	v1s := make([]*codectypes.Any, len(v2s))
	for i, v2 := range v2s {
		v1s[i] = &codectypes.Any{
			TypeUrl: v2.TypeUrl,
			Value:   v2.Value,
		}
	}
	return v1s
}

func decodeFromAny(cdc codec.BinaryCodec, anyPB *anypb.Any) (proto.Message, error) {
	messageName := anyPB.TypeUrl
	if i := strings.LastIndexByte(anyPB.TypeUrl, '/'); i >= 0 {
		messageName = messageName[i+len("/"):]
	}
	typ := proto.MessageType(messageName)
	if typ == nil {
		return nil, fmt.Errorf("cannot find type: %s", anyPB.TypeUrl)
	}
	v1 := reflect.New(typ.Elem()).Interface().(proto.Message)
	err := cdc.Unmarshal(anyPB.Value, v1)
	if err != nil {
		return nil, err
	}
	return v1, nil
}
