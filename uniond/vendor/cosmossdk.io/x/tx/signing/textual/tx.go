package textual

import (
	"bytes"
	"context"
	"crypto/sha256"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"regexp"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/anypb"

	msg "cosmossdk.io/api/cosmos/msg/v1"
	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing/textual/internal/textualpb"
)

var (
	// msgRe is a regex matching the beginning of the TxBody msgs in the envelope.
	msgRe = regexp.MustCompile("([0-9]+) Any")
	// inverseMsgRe is a regex matching the textual output of the TxBody msgs
	// header.
	inverseMsgRe = regexp.MustCompile("This transaction has ([0-9]+) Messages?")
)

type txValueRenderer struct {
	tr *SignModeHandler
}

// NewTxValueRenderer returns a ValueRenderer for the protobuf
// TextualData type. It follows the specification defined in ADR-050.
// The reason we create a renderer for TextualData (and not directly Tx)
// is that TextualData is a single place that contains all data needed
// to create the `[]Screen` SignDoc.
func NewTxValueRenderer(tr *SignModeHandler) ValueRenderer {
	return txValueRenderer{
		tr: tr,
	}
}

// Format implements the ValueRenderer interface.
func (vr txValueRenderer) Format(ctx context.Context, v protoreflect.Value) ([]Screen, error) {
	// Reify the reflected message as a proto Tx
	msg := v.Message().Interface()
	textualData, ok := msg.(*textualpb.TextualData)
	if !ok {
		return nil, fmt.Errorf("expected Tx, got %T", msg)
	}

	txBody := &txv1beta1.TxBody{}
	txAuthInfo := &txv1beta1.AuthInfo{}
	err := proto.Unmarshal(textualData.BodyBytes, txBody)
	if err != nil {
		return nil, err
	}
	err = proto.Unmarshal(textualData.AuthInfoBytes, txAuthInfo)
	if err != nil {
		return nil, err
	}

	// Create envelope here. We really need to make sure that all the non-Msg
	// fields inside both TxBody and AuthInfo are flattened here. For example,
	// if we decide to add new fields in either of those 2 structs, then we
	// should add a new field here in Envelope.
	envelope := &textualpb.Envelope{
		ChainId:                     textualData.SignerData.ChainId,
		AccountNumber:               textualData.SignerData.AccountNumber,
		Sequence:                    textualData.SignerData.Sequence,
		Address:                     textualData.SignerData.Address,
		PublicKey:                   textualData.SignerData.PubKey,
		Message:                     txBody.Messages,
		Memo:                        txBody.Memo,
		Fees:                        txAuthInfo.Fee.Amount,
		FeePayer:                    txAuthInfo.Fee.Payer,
		FeeGranter:                  txAuthInfo.Fee.Granter,
		GasLimit:                    txAuthInfo.Fee.GasLimit,
		TimeoutHeight:               txBody.TimeoutHeight,
		ExtensionOptions:            txBody.ExtensionOptions,
		NonCriticalExtensionOptions: txBody.NonCriticalExtensionOptions,
		HashOfRawBytes:              getHash(textualData.BodyBytes, textualData.AuthInfoBytes),
	}

	// Find all other tx signers than the current signer. In the case where our
	// Textual signer is one key of a multisig, then otherSigners will include
	// the multisig pubkey.
	otherSigners := []*txv1beta1.SignerInfo{}
	for _, si := range txAuthInfo.SignerInfos {
		if bytes.Equal(si.PublicKey.Value, textualData.SignerData.PubKey.Value) {
			continue
		}

		otherSigners = append(otherSigners, si)
	}
	envelope.OtherSigner = otherSigners

	mvr, err := vr.tr.GetMessageValueRenderer(envelope.ProtoReflect().Descriptor())
	if err != nil {
		return nil, err
	}

	screens, err := mvr.Format(ctx, protoreflect.ValueOf(envelope.ProtoReflect()))
	if err != nil {
		return nil, err
	}

	// Since we're value-rendering the (internal) envelope message, we do some
	// postprocessing. First, we remove first envelope header screen, and
	// unindent 1 level.

	// Remove 1st screen
	screens = screens[1:]
	for i := range screens {
		screens[i].Indent--
	}

	// Expert fields.
	expert := map[string]struct{}{
		"Address":                        {},
		"Public key":                     {},
		"Fee payer":                      {},
		"Fee granter":                    {},
		"Gas limit":                      {},
		"Timeout height":                 {},
		"Other signer":                   {},
		"Extension options":              {},
		"Non critical extension options": {},
		"Hash of raw bytes":              {},
	}

	for i := range screens {
		if screens[i].Indent == 0 {
			// Do expert fields.
			if _, ok := expert[screens[i].Title]; ok {
				expertify(screens, i, screens[i].Title)
			}

			// Replace:
			// "Message: <N> Any"
			// with:
			// "This transaction has <N> Message"
			if screens[i].Title == "Message" {
				matches := msgRe.FindStringSubmatch(screens[i].Content)
				if len(matches) > 0 {
					screens[i].Title = ""
					screens[i].Content = fmt.Sprintf("This transaction has %s Message", matches[1])
					if matches[1] != "1" {
						screens[i].Content += "s"
					}
				}
			}

		}
	}

	return screens, nil
}

// expertify marks all screens starting from `fromIdx` as expert, and stops
// just before it finds the next screen with Indent==0 (unless it's a "End of"
// termination screen). It modifies screens in-place.
func expertify(screens []Screen, fromIdx int, fieldName string) {
	for i := fromIdx; i < len(screens); i++ {
		if i > fromIdx &&
			screens[i].Indent == 0 &&
			screens[i].Content != fmt.Sprintf("End of %s", fieldName) {
			break
		}

		screens[i].Expert = true
	}
}

// getHash gets the hash of raw bytes to be signed over:
// HEX(sha256(len(body_bytes) ++ body_bytes ++ len(auth_info_bytes) ++ auth_info_bytes))
func getHash(bodyBz, authInfoBz []byte) string {
	bodyLen, authInfoLen := make([]byte, 8), make([]byte, 8)
	binary.BigEndian.PutUint64(bodyLen, uint64(len(bodyBz)))
	binary.BigEndian.PutUint64(authInfoLen, uint64(len(authInfoBz)))

	b := make([]byte, 16+len(bodyBz)+len(authInfoBz))
	copy(b[:8], bodyLen)
	copy(b[8:8+len(bodyBz)], bodyBz)
	copy(b[8+len(bodyBz):16+len(bodyBz)], authInfoLen)
	copy(b[16+len(bodyBz):], authInfoBz)

	h := sha256.Sum256(b)

	return hex.EncodeToString(h[:])
}

// Parse implements the ValueRenderer interface.
func (vr txValueRenderer) Parse(ctx context.Context, screens []Screen) (protoreflect.Value, error) {
	// Process the screens to be parsable by a envelope message value renderer
	parsable := make([]Screen, len(screens)+1)
	parsable[0] = Screen{Content: "Envelope object"}
	for i := range screens {
		parsable[i+1].Indent = screens[i].Indent + 1

		// Take same text, except that we weplace:
		// "This transaction has <N> Message"
		// with:
		// "Message: <N> Any"
		matches := inverseMsgRe.FindStringSubmatch(screens[i].Content)
		if len(matches) > 0 {
			parsable[i+1].Title = "Message"
			parsable[i+1].Content = fmt.Sprintf("%s Any", matches[1])
		} else {
			parsable[i+1].Title = screens[i].Title
			parsable[i+1].Content = screens[i].Content
		}
	}

	mvr, err := vr.tr.GetMessageValueRenderer((&textualpb.Envelope{}).ProtoReflect().Descriptor())
	if err != nil {
		return nilValue, err
	}

	envelopeV, err := mvr.Parse(ctx, parsable)
	if err != nil {
		return nilValue, err
	}
	envelope := envelopeV.Message().Interface().(*textualpb.Envelope)

	txBody := &txv1beta1.TxBody{
		Messages:                    envelope.Message,
		Memo:                        envelope.Memo,
		TimeoutHeight:               envelope.TimeoutHeight,
		ExtensionOptions:            envelope.ExtensionOptions,
		NonCriticalExtensionOptions: envelope.NonCriticalExtensionOptions,
	}
	authInfo := &txv1beta1.AuthInfo{
		Fee: &txv1beta1.Fee{
			Amount:   envelope.Fees,
			GasLimit: envelope.GasLimit,
			Payer:    envelope.FeePayer,
			Granter:  envelope.FeeGranter,
		},
	}

	// Figure out the signers in the correct order.
	signers, err := getSigners(txBody, authInfo)
	if err != nil {
		return nilValue, err
	}

	signerInfos := make([]*txv1beta1.SignerInfo, len(signers))
	for i, s := range signers {
		if s == envelope.Address {
			signerInfos[i] = &txv1beta1.SignerInfo{
				PublicKey: envelope.PublicKey,
				ModeInfo: &txv1beta1.ModeInfo{
					Sum: &txv1beta1.ModeInfo_Single_{
						Single: &txv1beta1.ModeInfo_Single{
							Mode: signingv1beta1.SignMode_SIGN_MODE_TEXTUAL,
						},
					},
				},
				Sequence: envelope.Sequence,
			}
		} else {
			// We know that signerInfos is well ordered, so just pop from it.
			signerInfos[i] = envelope.OtherSigner[0]
			envelope.OtherSigner = envelope.OtherSigner[1:]
		}
	}
	authInfo.SignerInfos = signerInfos

	// Note that we might not always get back the exact bodyBz and authInfoBz
	// that was passed into, because protobuf is not deterministic.
	// In tests, we don't check bytes equality, but protobuf object equality.
	protov2MarshalOpts := proto.MarshalOptions{Deterministic: true}
	bodyBz, err := protov2MarshalOpts.Marshal(txBody)
	if err != nil {
		return nilValue, err
	}
	authInfoBz, err := protov2MarshalOpts.Marshal(authInfo)
	if err != nil {
		return nilValue, err
	}

	tx := &textualpb.TextualData{
		BodyBytes:     bodyBz,
		AuthInfoBytes: authInfoBz,
		SignerData: &textualpb.SignerData{
			Address:       envelope.Address,
			AccountNumber: envelope.AccountNumber,
			ChainId:       envelope.ChainId,
			Sequence:      envelope.Sequence,
			PubKey:        envelope.PublicKey,
		},
	}

	return protoreflect.ValueOf(tx.ProtoReflect()), nil
}

// getSigners gets the ordered signers of a transaction. It's mostly a
// copy-paste of `types/tx/types.go` GetSigners method, but uses the proto
// annotation `cosmos.msg.v1.signer`, instead of the sdk.Msg#GetSigners method.
func getSigners(body *txv1beta1.TxBody, authInfo *txv1beta1.AuthInfo) ([]string, error) {
	var signers []string
	seen := map[string]bool{}

	for _, msgAny := range body.Messages {
		m, err := anypb.UnmarshalNew(msgAny, proto.UnmarshalOptions{})
		if err != nil {
			return nil, err
		}

		ext := proto.GetExtension(m.ProtoReflect().Descriptor().Options(), msg.E_Signer)
		signerFields, ok := ext.([]string)
		if !ok {
			return nil, fmt.Errorf("expected []string, got %T", ext)
		}

		for _, fieldName := range signerFields {
			fd := m.ProtoReflect().Descriptor().Fields().ByName(protoreflect.Name(fieldName))
			addr := m.ProtoReflect().Get(fd).String()
			if !seen[addr] {
				signers = append(signers, addr)
				seen[addr] = true
			}
		}
	}

	// ensure any specified fee payer is included in the required signers (at the end)
	feePayer := authInfo.Fee.Payer
	if feePayer != "" && !seen[feePayer] {
		signers = append(signers, feePayer)
		seen[feePayer] = true
	}

	return signers, nil
}
