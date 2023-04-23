// Copyright 2020 Coinbase, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package asserter

import (
	"fmt"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// ConstructionPreprocessResponse returns an error if
// the request public keys are not valid AccountIdentifiers.
func ConstructionPreprocessResponse(
	response *types.ConstructionPreprocessResponse,
) error {
	if response == nil {
		return ErrConstructionPreprocessResponseIsNil
	}

	for _, accountIdentifier := range response.RequiredPublicKeys {
		if err := AccountIdentifier(accountIdentifier); err != nil {
			return fmt.Errorf(
				"account identifier %s is invalid: %w",
				types.PrintStruct(accountIdentifier),
				err,
			)
		}
	}

	return nil
}

// ConstructionMetadataResponse returns an error if
// the metadata is not a JSON object.
func ConstructionMetadataResponse(
	response *types.ConstructionMetadataResponse,
) error {
	if response == nil {
		return ErrConstructionMetadataResponseIsNil
	}

	if response.Metadata == nil {
		return ErrConstructionMetadataResponseMetadataMissing
	}

	if err := AssertUniqueAmounts(response.SuggestedFee); err != nil {
		return fmt.Errorf(
			"suggested fee %s is invalid: %w",
			types.PrintStruct(response.SuggestedFee),
			err,
		)
	}

	return nil
}

// TransactionIdentifierResponse returns an error if
// the types.TransactionIdentifier in the response is not
// valid.
func TransactionIdentifierResponse(
	response *types.TransactionIdentifierResponse,
) error {
	if response == nil {
		return ErrTxIdentifierResponseIsNil
	}

	if err := TransactionIdentifier(response.TransactionIdentifier); err != nil {
		return fmt.Errorf(
			"transaction identifier %s is invalid: %w",
			types.PrintStruct(response.TransactionIdentifier),
			err,
		)
	}

	return nil
}

// ConstructionCombineResponse returns an error if
// a *types.ConstructionCombineResponse does
// not have a populated SignedTransaction.
func ConstructionCombineResponse(
	response *types.ConstructionCombineResponse,
) error {
	if response == nil {
		return ErrConstructionCombineResponseIsNil
	}

	if len(response.SignedTransaction) == 0 {
		return ErrSignedTxEmpty
	}

	return nil
}

// ConstructionDeriveResponse returns an error if
// a *types.ConstructionDeriveResponse does
// not have a populated Address.
func ConstructionDeriveResponse(
	response *types.ConstructionDeriveResponse,
) error {
	if response == nil {
		return ErrConstructionDeriveResponseIsNil
	}

	if err := AccountIdentifier(response.AccountIdentifier); err != nil {
		return fmt.Errorf(
			"account identifier %s is invalid: %w",
			types.PrintStruct(response.AccountIdentifier),
			err,
		)
	}

	return nil
}

// ConstructionParseResponse returns an error if
// a *types.ConstructionParseResponse does
// not have a valid set of operations or
// if the signers is empty.
func (a *Asserter) ConstructionParseResponse(
	response *types.ConstructionParseResponse,
	signed bool,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if response == nil {
		return ErrConstructionParseResponseIsNil
	}

	if len(response.Operations) == 0 {
		return ErrConstructionParseResponseOperationsEmpty
	}

	if err := a.Operations(response.Operations, true); err != nil {
		return fmt.Errorf(
			"operations %s are invalid: %w",
			types.PrintStruct(response.Operations),
			err,
		)
	}

	if signed && len(response.AccountIdentifierSigners) == 0 {
		return ErrConstructionParseResponseSignersEmptyOnSignedTx
	}

	if !signed && len(response.AccountIdentifierSigners) > 0 {
		return ErrConstructionParseResponseSignersNonEmptyOnUnsignedTx
	}

	for _, signer := range response.AccountIdentifierSigners {
		if err := AccountIdentifier(signer); err != nil {
			return fmt.Errorf(
				"account identifier of signer %s is invalid: %w",
				types.PrintStruct(signer),
				err,
			)
		}
	}

	if len(response.AccountIdentifierSigners) > 0 {
		if err := AccountArray("signers", response.AccountIdentifierSigners); err != nil {
			return fmt.Errorf(
				"account identifiers of signers %s are invalid: %w",
				types.PrintStruct(response.AccountIdentifierSigners),
				err,
			)
		}
	}

	return nil
}

// ConstructionPayloadsResponse returns an error if
// a *types.ConstructionPayloadsResponse does
// not have an UnsignedTransaction or has no
// valid *SigningPaylod.
func ConstructionPayloadsResponse(
	response *types.ConstructionPayloadsResponse,
) error {
	if response == nil {
		return ErrConstructionPayloadsResponseIsNil
	}

	if len(response.UnsignedTransaction) == 0 {
		return ErrConstructionPayloadsResponseUnsignedTxEmpty
	}

	if len(response.Payloads) == 0 {
		return ErrConstructionPayloadsResponsePayloadsEmpty
	}

	for _, payload := range response.Payloads {
		if err := SigningPayload(payload); err != nil {
			return fmt.Errorf("signing payload %s is invalid: %w", types.PrintStruct(payload), err)
		}
	}

	return nil
}

// PublicKey returns an error if
// the *types.PublicKey is nil, is not
// valid hex, or has an undefined CurveType.
func PublicKey(
	publicKey *types.PublicKey,
) error {
	if publicKey == nil {
		return ErrPublicKeyIsNil
	}

	if len(publicKey.Bytes) == 0 {
		return ErrPublicKeyBytesEmpty
	}

	if BytesArrayZero(publicKey.Bytes) {
		return ErrPublicKeyBytesZero
	}

	if err := CurveType(publicKey.CurveType); err != nil {
		return fmt.Errorf(
			"public key curve type %s is invalid: %w",
			types.PrintStruct(publicKey.CurveType),
			err,
		)
	}

	return nil
}

// CurveType returns an error if
// the curve is not a valid types.CurveType.
func CurveType(
	curve types.CurveType,
) error {
	switch curve {
	case types.Secp256k1, types.Secp256r1, types.Edwards25519, types.Tweedle, types.Pallas:
		return nil
	default:
		return ErrCurveTypeNotSupported
	}
}

// SigningPayload returns an error
// if a *types.SigningPayload is nil,
// has an empty address, has invlaid hex,
// or has an invalid SignatureType (if populated).
func SigningPayload(
	signingPayload *types.SigningPayload,
) error {
	if signingPayload == nil {
		return ErrSigningPayloadIsNil
	}

	if err := AccountIdentifier(signingPayload.AccountIdentifier); err != nil {
		return fmt.Errorf(
			"account identifier %s is invalid: %w",
			types.PrintStruct(signingPayload.AccountIdentifier),
			err,
		)
	}

	if len(signingPayload.Bytes) == 0 {
		return ErrSigningPayloadBytesEmpty
	}

	if BytesArrayZero(signingPayload.Bytes) {
		return ErrSigningPayloadBytesZero
	}

	// SignatureType can be optionally populated
	if len(signingPayload.SignatureType) == 0 {
		return nil
	}

	if err := SignatureType(signingPayload.SignatureType); err != nil {
		return fmt.Errorf(
			"signature type %s is invalid: %w",
			types.PrintStruct(signingPayload.SignatureType),
			err,
		)
	}

	return nil
}

// Signatures returns an error if any
// *types.Signature is invalid.
func Signatures(
	signatures []*types.Signature,
) error {
	if len(signatures) == 0 {
		return ErrSignaturesEmpty
	}

	for _, signature := range signatures {
		if err := SigningPayload(signature.SigningPayload); err != nil {
			return fmt.Errorf(
				"signing payload %s is invalid: %w",
				types.PrintStruct(signature.SigningPayload),
				err,
			)
		}

		if err := PublicKey(signature.PublicKey); err != nil {
			return fmt.Errorf(
				"public key %s is invalid: %w",
				types.PrintStruct(signature.PublicKey),
				err,
			)
		}

		if err := SignatureType(signature.SignatureType); err != nil {
			return fmt.Errorf(
				"signature type %s is invalid: %w",
				types.PrintStruct(signature.SignatureType),
				err,
			)
		}

		// Return an error if the requested signature type does not match the
		// signature type in the returned signature.
		if len(signature.SigningPayload.SignatureType) > 0 &&
			signature.SigningPayload.SignatureType != signature.SignatureType {
			return ErrSignaturesReturnedSigMismatch
		}

		if len(signature.Bytes) == 0 {
			return ErrSignatureBytesEmpty
		}

		if BytesArrayZero(signature.Bytes) {
			return ErrSignatureBytesZero
		}
	}

	return nil
}

// SignatureType returns an error if
// signature is not a valid types.SignatureType.
func SignatureType(
	signature types.SignatureType,
) error {
	switch signature {
	case types.Ecdsa, types.EcdsaRecovery, types.Ed25519, types.Schnorr1, types.SchnorrPoseidon:
		return nil
	default:
		return ErrSignatureTypeNotSupported
	}
}
