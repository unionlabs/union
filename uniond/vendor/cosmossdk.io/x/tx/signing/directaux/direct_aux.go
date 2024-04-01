package directaux

import (
	"context"
	"errors"
	"fmt"

	"github.com/cosmos/cosmos-proto/anyutil"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoregistry"

	signingv1beta1 "cosmossdk.io/api/cosmos/tx/signing/v1beta1"
	txv1beta1 "cosmossdk.io/api/cosmos/tx/v1beta1"
	"cosmossdk.io/x/tx/signing"
)

// SignModeHandler is the SIGN_MODE_DIRECT_AUX implementation of signing.SignModeHandler.
type SignModeHandler struct {
	signersContext *signing.Context
	fileResolver   signing.ProtoFileResolver
	typeResolver   protoregistry.MessageTypeResolver
}

// SignModeHandlerOptions are the options for the SignModeHandler.
type SignModeHandlerOptions struct {
	// TypeResolver is the protoregistry.MessageTypeResolver to use for resolving protobuf types when unpacking any messages.
	TypeResolver protoregistry.MessageTypeResolver

	// SignersContext is the signing.Context to use for getting signers.
	SignersContext *signing.Context
}

// NewSignModeHandler returns a new SignModeHandler.
func NewSignModeHandler(options SignModeHandlerOptions) (SignModeHandler, error) {
	h := SignModeHandler{}

	if options.SignersContext == nil {
		return h, errors.New("signers context is required")
	}
	h.signersContext = options.SignersContext

	h.fileResolver = h.signersContext.FileResolver()

	if options.TypeResolver == nil {
		h.typeResolver = protoregistry.GlobalTypes
	} else {
		h.typeResolver = options.TypeResolver
	}

	return h, nil
}

var _ signing.SignModeHandler = SignModeHandler{}

// Mode implements signing.SignModeHandler.Mode.
func (h SignModeHandler) Mode() signingv1beta1.SignMode {
	return signingv1beta1.SignMode_SIGN_MODE_DIRECT_AUX
}

// getFirstSigner returns the first signer from the first message in the tx. It replicates behavior in
// https://github.com/cosmos/cosmos-sdk/blob/4a6a1e3cb8de459891cb0495052589673d14ef51/x/auth/tx/builder.go#L142
func (h SignModeHandler) getFirstSigner(txData signing.TxData) ([]byte, error) {
	if len(txData.Body.Messages) == 0 {
		return nil, errors.New("no signer found")
	}

	msg, err := anyutil.Unpack(txData.Body.Messages[0], h.fileResolver, h.typeResolver)
	if err != nil {
		return nil, err
	}
	signer, err := h.signersContext.GetSigners(msg)
	if err != nil {
		return nil, err
	}
	return signer[0], nil
}

// GetSignBytes implements signing.SignModeHandler.GetSignBytes.
func (h SignModeHandler) GetSignBytes(
	_ context.Context, signerData signing.SignerData, txData signing.TxData,
) ([]byte, error) {
	feePayer := txData.AuthInfo.Fee.Payer
	if feePayer == "" {
		fp, err := h.getFirstSigner(txData)
		if err != nil {
			return nil, err
		}
		feePayer, err = h.signersContext.AddressCodec().BytesToString(fp)
		if err != nil {
			return nil, err
		}
	}
	if feePayer == signerData.Address {
		return nil, fmt.Errorf("fee payer %s cannot sign with %s: unauthorized",
			feePayer, signingv1beta1.SignMode_SIGN_MODE_DIRECT_AUX)
	}

	signDocDirectAux := &txv1beta1.SignDocDirectAux{
		BodyBytes:     txData.BodyBytes,
		PublicKey:     signerData.PubKey,
		ChainId:       signerData.ChainID,
		AccountNumber: signerData.AccountNumber,
		Sequence:      signerData.Sequence,
	}

	protov2MarshalOpts := proto.MarshalOptions{Deterministic: true}
	return protov2MarshalOpts.Marshal(signDocDirectAux)
}
