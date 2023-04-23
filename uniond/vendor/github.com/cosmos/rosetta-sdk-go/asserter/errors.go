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
	"errors"

	utils "github.com/cosmos/rosetta-sdk-go/errors"
)

var (
	// ErrAsserterNotInitialized is returned when some call in the asserter
	// package requires the asserter to be initialized first.
	ErrAsserterNotInitialized = errors.New("asserter not initialized")
)

// Account Balance Errors
var (
	ErrCurrencyUsedMultipleTimes = errors.New("currency is used multiple times")

	ErrReturnedBlockHashMismatch = errors.New(
		"request block hash does not match response block hash",
	)
	ErrReturnedBlockIndexMismatch = errors.New(
		"request block index does not match response block index",
	)

	AccountBalanceErrs = []error{
		ErrCurrencyUsedMultipleTimes,
		ErrReturnedBlockHashMismatch,
		ErrReturnedBlockIndexMismatch,
	}
)

// Block Errors
var (
	ErrAmountValueMissing            = errors.New("Amount.Value is missing")
	ErrAmountIsNotInt                = errors.New("Amount.Value is not an integer")
	ErrAmountCurrencyIsNil           = errors.New("Amount.Currency is nil")
	ErrAmountCurrencySymbolEmpty     = errors.New("Amount.Currency.Symbol is empty")
	ErrAmountCurrencyHasNegDecimals  = errors.New("Amount.Currency.Decimals must be >= 0")
	ErrOperationIdentifierIndexIsNil = errors.New(
		"Operation.OperationIdentifier.Index is invalid",
	)
	ErrOperationIdentifierIndexOutOfOrder = errors.New(
		"Operation.OperationIdentifier.Index is out of order",
	)
	ErrOperationIdentifierNetworkIndexInvalid = errors.New(
		"Operation.OperationIdentifier.NetworkIndex is invalid",
	)
	ErrAccountIsNil                           = errors.New("Account is nil")
	ErrAccountAddrMissing                     = errors.New("Account.Address is missing")
	ErrAccountSubAccountAddrMissing           = errors.New("Account.SubAccount.Address is missing")
	ErrOperationStatusMissing                 = errors.New("Operation.Status is missing")
	ErrOperationStatusInvalid                 = errors.New("Operation.Status is invalid")
	ErrOperationTypeInvalid                   = errors.New("Operation.Type is invalid")
	ErrOperationIsNil                         = errors.New("Operation is nil")
	ErrOperationStatusNotEmptyForConstruction = errors.New(
		"Operation.Status must be empty for construction",
	)
	ErrRelatedOperationIndexOutOfOrder = errors.New(
		"related operation has index greater than operation",
	)
	ErrRelatedOperationIndexDuplicate  = errors.New("found duplicate related operation index")
	ErrRelatedOperationMissing         = errors.New("related operations key is missing")
	ErrRelatedOperationInFeeNotAllowed = errors.New(
		"fee operation shouldn't have related_operations",
	)
	ErrBlockIdentifierIsNil                  = errors.New("BlockIdentifier is nil")
	ErrBlockIdentifierHashMissing            = errors.New("BlockIdentifier.Hash is missing")
	ErrBlockIdentifierIndexIsNeg             = errors.New("BlockIdentifier.Index is negative")
	ErrPartialBlockIdentifierIsNil           = errors.New("PartialBlockIdentifier is nil")
	ErrPartialBlockIdentifierHashIsEmpty     = errors.New("PartialBlockIdentifier hash is empty")
	ErrPartialBlockIdentifierIndexIsNegative = errors.New(
		"PartialBlockIdentifier index is negative",
	)
	ErrTxIdentifierIsNil           = errors.New("TransactionIdentifier is nil")
	ErrTxIdentifierHashMissing     = errors.New("TransactionIdentifier.Hash is missing")
	ErrNoOperationsForConstruction = errors.New(
		"operations cannot be empty for construction",
	)
	ErrTxIsNil                        = errors.New("Transaction is nil")
	ErrTimestampBeforeMin             = errors.New("timestamp is before 01/01/2000")
	ErrTimestampAfterMax              = errors.New("timestamp is after 01/01/2040")
	ErrBlockIsNil                     = errors.New("Block is nil")
	ErrBlockHashEqualsParentBlockHash = errors.New(
		"BlockIdentifier.Hash == ParentBlockIdentifier.Hash",
	)
	ErrBlockIndexPrecedesParentBlockIndex = errors.New(
		"BlockIdentifier.Index <= ParentBlockIdentifier.Index",
	)
	ErrInvalidDirection = errors.New(
		"invalid direction (must be 'forward' or 'backward')",
	)
	ErrDuplicateRelatedTransaction = errors.New("duplicate related transaction")
	ErrPaymentAmountNotBalancing   = errors.New("payment amount doesn't balance")
	ErrFeeAmountNotBalancing       = errors.New("fee amount doesn't balance")
	ErrPaymentCountMismatch        = errors.New("payment count doesn't match")
	ErrFeeCountMismatch            = errors.New("fee count doesn't match")
	ErrFeeAmountNotNegative        = errors.New("fee amount is not negative")

	BlockErrs = []error{
		ErrAmountValueMissing,
		ErrAmountIsNotInt,
		ErrAmountCurrencyIsNil,
		ErrAmountCurrencySymbolEmpty,
		ErrAmountCurrencyHasNegDecimals,
		ErrOperationIdentifierIndexIsNil,
		ErrOperationIdentifierIndexOutOfOrder,
		ErrOperationIdentifierNetworkIndexInvalid,
		ErrAccountIsNil,
		ErrAccountAddrMissing,
		ErrAccountSubAccountAddrMissing,
		ErrOperationStatusMissing,
		ErrOperationStatusInvalid,
		ErrOperationTypeInvalid,
		ErrOperationIsNil,
		ErrOperationStatusNotEmptyForConstruction,
		ErrRelatedOperationIndexOutOfOrder,
		ErrRelatedOperationIndexDuplicate,
		ErrRelatedOperationMissing,
		ErrBlockIdentifierIsNil,
		ErrBlockIdentifierHashMissing,
		ErrBlockIdentifierIndexIsNeg,
		ErrPartialBlockIdentifierIsNil,
		ErrPartialBlockIdentifierHashIsEmpty,
		ErrPartialBlockIdentifierIndexIsNegative,
		ErrTxIdentifierIsNil,
		ErrTxIdentifierHashMissing,
		ErrNoOperationsForConstruction,
		ErrTxIsNil,
		ErrTimestampBeforeMin,
		ErrTimestampAfterMax,
		ErrBlockIsNil,
		ErrBlockHashEqualsParentBlockHash,
		ErrBlockIndexPrecedesParentBlockIndex,
		ErrInvalidDirection,
		ErrDuplicateRelatedTransaction,
		ErrPaymentAmountNotBalancing,
		ErrFeeAmountNotBalancing,
		ErrFeeAmountNotNegative,
	}
)

// Coin Errors
var (
	ErrCoinIsNil            = errors.New("coin cannot be nil")
	ErrCoinDuplicate        = errors.New("duplicate coin identifier detected")
	ErrCoinIdentifierIsNil  = errors.New("coin identifier cannot be nil")
	ErrCoinIdentifierNotSet = errors.New("coin identifier cannot be empty")
	ErrCoinChangeIsNil      = errors.New("coin change cannot be nil")
	ErrCoinActionInvalid    = errors.New("not a valid coin action")

	CoinErrs = []error{
		ErrCoinIsNil,
		ErrCoinDuplicate,
		ErrCoinIdentifierIsNil,
		ErrCoinIdentifierNotSet,
		ErrCoinChangeIsNil,
		ErrCoinActionInvalid,
	}
)

// Construction Errors
var (
	ErrConstructionPreprocessResponseIsNil = errors.New(
		"ConstructionPreprocessResponse cannot be nil",
	)
	ErrConstructionMetadataResponseIsNil = errors.New(
		"ConstructionMetadataResponse cannot be nil",
	)
	ErrConstructionMetadataResponseMetadataMissing = errors.New("Metadata is nil")
	ErrTxIdentifierResponseIsNil                   = errors.New(
		"TransactionIdentifierResponse cannot be nil",
	)
	ErrConstructionCombineResponseIsNil = errors.New(
		"construction combine response cannot be nil",
	)
	ErrSignedTxEmpty = errors.New(
		"signed transaction cannot be empty",
	)
	ErrConstructionDeriveResponseIsNil = errors.New(
		"construction derive response cannot be nil",
	)
	ErrConstructionParseResponseIsNil = errors.New(
		"construction parse response cannot be nil",
	)
	ErrConstructionParseResponseOperationsEmpty        = errors.New("operations cannot be empty")
	ErrConstructionParseResponseSignersEmptyOnSignedTx = errors.New(
		"signers cannot be empty on signed transaction",
	)
	ErrConstructionParseResponseSignersNonEmptyOnUnsignedTx = errors.New(
		"signers should be empty for unsigned txs",
	)
	ErrConstructionPayloadsResponseIsNil = errors.New(
		"construction payloads response cannot be nil",
	)
	ErrConstructionPayloadsResponseUnsignedTxEmpty = errors.New(
		"unsigned transaction cannot be empty",
	)
	ErrConstructionPayloadsResponsePayloadsEmpty = errors.New("signing payloads cannot be empty")
	ErrPublicKeyIsNil                            = errors.New("PublicKey cannot be nil")
	ErrPublicKeyBytesEmpty                       = errors.New("public key bytes cannot be empty")
	ErrPublicKeyBytesZero                        = errors.New("public key bytes 0")
	ErrCurveTypeNotSupported                     = errors.New("not a supported CurveType")
	ErrSigningPayloadIsNil                       = errors.New("signing payload cannot be nil")
	ErrSigningPayloadBytesEmpty                  = errors.New(
		"signing payload bytes cannot be empty",
	)
	ErrSigningPayloadBytesZero = errors.New(
		"signing payload bytes cannot be 0",
	)
	ErrSignaturesEmpty               = errors.New("signatures cannot be empty")
	ErrSignaturesReturnedSigMismatch = errors.New(
		"requested signature type does not match returned signature type",
	)
	ErrSignatureBytesEmpty       = errors.New("signature bytes cannot be empty")
	ErrSignatureBytesZero        = errors.New("signature bytes cannot be 0")
	ErrSignatureTypeNotSupported = errors.New("not a supported SignatureType")

	ConstructionErrs = []error{
		ErrConstructionPreprocessResponseIsNil,
		ErrConstructionMetadataResponseIsNil,
		ErrConstructionMetadataResponseMetadataMissing,
		ErrTxIdentifierResponseIsNil,
		ErrConstructionCombineResponseIsNil,
		ErrSignedTxEmpty,
		ErrConstructionDeriveResponseIsNil,
		ErrConstructionParseResponseIsNil,
		ErrConstructionParseResponseOperationsEmpty,
		ErrConstructionParseResponseSignersEmptyOnSignedTx,
		ErrConstructionParseResponseSignersNonEmptyOnUnsignedTx,
		ErrConstructionPayloadsResponseIsNil,
		ErrConstructionPayloadsResponseUnsignedTxEmpty,
		ErrConstructionPayloadsResponsePayloadsEmpty,
		ErrPublicKeyIsNil,
		ErrPublicKeyBytesEmpty,
		ErrPublicKeyBytesZero,
		ErrCurveTypeNotSupported,
		ErrSigningPayloadIsNil,
		ErrSigningPayloadBytesEmpty,
		ErrSigningPayloadBytesZero,
		ErrSignaturesEmpty,
		ErrSignaturesReturnedSigMismatch,
		ErrSignatureBytesEmpty,
		ErrSignatureBytesZero,
		ErrSignatureTypeNotSupported,
	}
)

// Network Errors
var (
	ErrSubNetworkIdentifierInvalid = errors.New(
		"NetworkIdentifier.SubNetworkIdentifier.Network is missing",
	)
	ErrNetworkIdentifierIsNil               = errors.New("NetworkIdentifier is nil")
	ErrNetworkIdentifierBlockchainMissing   = errors.New("NetworkIdentifier.Blockchain is missing")
	ErrNetworkIdentifierNetworkMissing      = errors.New("NetworkIdentifier.Network is missing")
	ErrPeerIDMissing                        = errors.New("Peer.PeerID is missing")
	ErrVersionIsNil                         = errors.New("version is nil")
	ErrVersionNodeVersionMissing            = errors.New("Version.NodeVersion is missing")
	ErrVersionMiddlewareVersionMissing      = errors.New("Version.MiddlewareVersion is missing")
	ErrNetworkStatusResponseIsNil           = errors.New("network status response is nil")
	ErrNoAllowedOperationStatuses           = errors.New("no Allow.OperationStatuses found")
	ErrNoSuccessfulAllowedOperationStatuses = errors.New(
		"no successful Allow.OperationStatuses found",
	)
	ErrErrorCodeUsedMultipleTimes = errors.New("error code used multiple times")
	ErrErrorDetailsPopulated      = errors.New(
		"error details populated in /network/options",
	)
	ErrAllowIsNil                                    = errors.New("Allow is nil")
	ErrNetworkOptionsResponseIsNil                   = errors.New("options is nil")
	ErrNetworkListResponseIsNil                      = errors.New("NetworkListResponse is nil")
	ErrNetworkListResponseNetworksContainsDuplicates = errors.New(
		"NetworkListResponse.Networks contains duplicates",
	)
	ErrBalanceExemptionIsNil                  = errors.New("BalanceExemption is nil")
	ErrBalanceExemptionTypeInvalid            = errors.New("BalanceExemption.Type is invalid")
	ErrBalanceExemptionMissingSubject         = errors.New("BalanceExemption missing subject")
	ErrBalanceExemptionSubAccountAddressEmpty = errors.New(
		"BalanceExemption.SubAccountAddress is empty",
	)
	ErrBalanceExemptionNoHistoricalLookup = errors.New(
		"BalanceExemptions only supported when HistoricalBalanceLookup supported",
	)
	ErrTimestampStartIndexInvalid = errors.New(
		"TimestampStartIndex is invalid",
	)
	ErrSyncStatusCurrentIndexNegative = errors.New(
		"SyncStatus.CurrentIndex is negative",
	)
	ErrSyncStatusTargetIndexNegative = errors.New(
		"SyncStatus.TargetIndex is negative",
	)
	ErrSyncStatusStageInvalid = errors.New(
		"SyncStatus.Stage is invalid",
	)

	NetworkErrs = []error{
		ErrSubNetworkIdentifierInvalid,
		ErrNetworkIdentifierIsNil,
		ErrNetworkIdentifierBlockchainMissing,
		ErrNetworkIdentifierNetworkMissing,
		ErrPeerIDMissing,
		ErrVersionIsNil,
		ErrVersionNodeVersionMissing,
		ErrVersionMiddlewareVersionMissing,
		ErrNetworkStatusResponseIsNil,
		ErrNoAllowedOperationStatuses,
		ErrNoSuccessfulAllowedOperationStatuses,
		ErrErrorCodeUsedMultipleTimes,
		ErrErrorDetailsPopulated,
		ErrAllowIsNil,
		ErrNetworkOptionsResponseIsNil,
		ErrNetworkListResponseIsNil,
		ErrNetworkListResponseNetworksContainsDuplicates,
		ErrBalanceExemptionIsNil,
		ErrBalanceExemptionTypeInvalid,
		ErrBalanceExemptionMissingSubject,
		ErrBalanceExemptionSubAccountAddressEmpty,
		ErrBalanceExemptionNoHistoricalLookup,
		ErrTimestampStartIndexInvalid,
		ErrSyncStatusCurrentIndexNegative,
		ErrSyncStatusTargetIndexNegative,
		ErrSyncStatusStageInvalid,
	}
)

// Server Errors
var (
	ErrNoSupportedNetworks = errors.New(
		"no supported networks",
	)
	ErrSupportedNetworksDuplicate = errors.New(
		"supported network duplicate",
	)
	ErrRequestedNetworkNotSupported = errors.New(
		"requestNetwork is not supported",
	)
	ErrAccountBalanceRequestIsNil = errors.New(
		"AccountBalanceRequest is nil",
	)
	ErrAccountBalanceRequestHistoricalBalanceLookupNotSupported = errors.New(
		"historical balance lookup is not supported",
	)
	ErrBlockRequestIsNil                      = errors.New("BlockRequest is nil")
	ErrBlockTransactionRequestIsNil           = errors.New("BlockTransactionRequest is nil")
	ErrConstructionMetadataRequestIsNil       = errors.New("ConstructionMetadataRequest is nil")
	ErrConstructionSubmitRequestIsNil         = errors.New("ConstructionSubmitRequest is nil")
	ErrConstructionSubmitRequestSignedTxEmpty = errors.New(
		"ConstructionSubmitRequest.SignedTransaction is empty",
	)
	ErrMempoolTransactionRequestIsNil = errors.New(
		"MempoolTransactionRequest is nil",
	)
	ErrMetadataRequestIsNil = errors.New(
		"MetadataRequest is nil",
	)
	ErrNetworkRequestIsNil = errors.New(
		"NetworkRequest is nil",
	)
	ErrConstructionDeriveRequestIsNil = errors.New(
		"ConstructionDeriveRequest is nil",
	)
	ErrConstructionPreprocessRequestIsNil = errors.New(
		"ConstructionPreprocessRequest is nil",
	)
	ErrConstructionPreprocessRequestSuggestedFeeMultiplierIsNeg = errors.New(
		"suggested fee multiplier cannot be less than 0",
	)
	ErrConstructionPayloadsRequestIsNil          = errors.New("ConstructionPayloadsRequest is nil")
	ErrConstructionCombineRequestIsNil           = errors.New("ConstructionCombineRequest is nil")
	ErrConstructionCombineRequestUnsignedTxEmpty = errors.New("UnsignedTransaction cannot be empty")
	ErrConstructionHashRequestIsNil              = errors.New("ConstructionHashRequest is nil")
	ErrConstructionHashRequestSignedTxEmpty      = errors.New("SignedTransaction cannot be empty")
	ErrConstructionParseRequestIsNil             = errors.New("ConstructionParseRequest is nil")
	ErrConstructionParseRequestEmpty             = errors.New("Transaction cannot be empty")
	ErrCallRequestIsNil                          = errors.New("CallRequest is nil")
	ErrCallMethodEmpty                           = errors.New("call method cannot be empty")
	ErrCallMethodUnsupported                     = errors.New("call method is not supported")
	ErrCallMethodDuplicate                       = errors.New("duplicate call method detected")
	ErrAccountCoinsRequestIsNil                  = errors.New("AccountCoinsRequest is nil")
	ErrMempoolCoinsNotSupported                  = errors.New("mempool coins not supported")
	ErrEventsBlocksRequestIsNil                  = errors.New("EventsBlocksRequest is nil")
	ErrOffsetIsNegative                          = errors.New("offset is negative")
	ErrLimitIsNegative                           = errors.New("limit is negative")
	ErrSearchTransactionsRequestIsNil            = errors.New("SearchTransactionsRequest is nil")
	ErrOperatorInvalid                           = errors.New("operator is invalid")
	ErrMaxBlockInvalid                           = errors.New("max block invalid")
	ErrDuplicateCurrency                         = errors.New("duplicate currency")

	ServerErrs = []error{
		ErrNoSupportedNetworks,
		ErrSupportedNetworksDuplicate,
		ErrRequestedNetworkNotSupported,
		ErrAccountBalanceRequestIsNil,
		ErrAccountBalanceRequestHistoricalBalanceLookupNotSupported,
		ErrBlockRequestIsNil,
		ErrBlockTransactionRequestIsNil,
		ErrConstructionMetadataRequestIsNil,
		ErrConstructionSubmitRequestIsNil,
		ErrConstructionSubmitRequestSignedTxEmpty,
		ErrMempoolTransactionRequestIsNil,
		ErrMetadataRequestIsNil,
		ErrNetworkRequestIsNil,
		ErrConstructionDeriveRequestIsNil,
		ErrConstructionPreprocessRequestIsNil,
		ErrConstructionPreprocessRequestSuggestedFeeMultiplierIsNeg,
		ErrConstructionPayloadsRequestIsNil,
		ErrConstructionCombineRequestIsNil,
		ErrConstructionCombineRequestUnsignedTxEmpty,
		ErrConstructionHashRequestIsNil,
		ErrConstructionHashRequestSignedTxEmpty,
		ErrConstructionParseRequestIsNil,
		ErrConstructionParseRequestEmpty,
		ErrCallRequestIsNil,
		ErrCallMethodEmpty,
		ErrCallMethodUnsupported,
		ErrCallMethodDuplicate,
		ErrAccountCoinsRequestIsNil,
		ErrMempoolCoinsNotSupported,
		ErrEventsBlocksRequestIsNil,
		ErrOffsetIsNegative,
		ErrLimitIsNegative,
		ErrSearchTransactionsRequestIsNil,
		ErrOperatorInvalid,
		ErrMaxBlockInvalid,
		ErrDuplicateCurrency,
	}
)

// Events Errors
var (
	ErrMaxSequenceInvalid    = errors.New("max sequence invalid")
	ErrSequenceInvalid       = errors.New("sequence invalid")
	ErrBlockEventTypeInvalid = errors.New("block event type invalid")
	ErrSequenceOutOfOrder    = errors.New("sequence out of order")

	EventsErrs = []error{
		ErrMaxSequenceInvalid,
		ErrSequenceInvalid,
		ErrBlockEventTypeInvalid,
		ErrSequenceOutOfOrder,
	}
)

// Search Errors
var (
	ErrNextOffsetInvalid = errors.New("next offset invalid")
	ErrTotalCountInvalid = errors.New("total count invalid")

	SearchErrs = []error{
		ErrNextOffsetInvalid,
		ErrTotalCountInvalid,
	}
)

// Error Errors
var (
	ErrErrorIsNil           = errors.New("Error is nil")
	ErrErrorCodeIsNeg       = errors.New("Error.Code is negative")
	ErrErrorMessageMissing  = errors.New("Error.Message is missing")
	ErrErrorUnexpectedCode  = errors.New("Error.Code unexpected")
	ErrErrorMessageMismatch = errors.New(
		"Error.Message does not match message from /network/options",
	)
	ErrErrorRetriableMismatch = errors.New("Error.Retriable mismatch")
	ErrErrorDescriptionEmpty  = errors.New(
		"Error.Description is provided but is empty",
	)

	ErrorErrs = []error{
		ErrErrorIsNil,
		ErrErrorCodeIsNeg,
		ErrErrorMessageMissing,
		ErrErrorUnexpectedCode,
		ErrErrorMessageMismatch,
		ErrErrorRetriableMismatch,
		ErrErrorDescriptionEmpty,
	}
)

// Util Errors
var (
	ErrStringArrayEmpty             = errors.New("string array is empty")
	ErrStringArrayEmptyString       = errors.New("empty string exists in string array")
	ErrStringArrayDuplicateString   = errors.New("duplicate string exists in string array")
	ErrAccountArrayEmpty            = errors.New("account array is empty")
	ErrAccountArrayInvalidAccount   = errors.New("invalid account exists in account array")
	ErrAccountArrayDuplicateAccount = errors.New("duplicate account exists in account array")
)

// Err takes an error as an argument and returns
// whether or not the error is one thrown by the asserter
// along with the specific source of the error
func Err(err error) (bool, string) {
	assertErrs := map[string][]error{
		"account balance error": AccountBalanceErrs,
		"block error":           BlockErrs,
		"coin error":            CoinErrs,
		"construction error":    ConstructionErrs,
		"network error":         NetworkErrs,
		"server error":          ServerErrs,
		"events error":          EventsErrs,
		"search error":          SearchErrs,
		"error error":           ErrorErrs,
	}

	for key, val := range assertErrs {
		if utils.FindError(val, err) {
			return true, key
		}
	}
	return false, ""
}
