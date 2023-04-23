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

// SupportedNetworks returns an error if there is an invalid
// types.NetworkIdentifier or there is a duplicate.
func SupportedNetworks(supportedNetworks []*types.NetworkIdentifier) error {
	if len(supportedNetworks) == 0 {
		return ErrNoSupportedNetworks
	}

	parsed := make([]*types.NetworkIdentifier, len(supportedNetworks))
	for i, network := range supportedNetworks {
		if err := NetworkIdentifier(network); err != nil {
			return fmt.Errorf(
				"network identifier %s is invalid: %w",
				types.PrintStruct(network),
				err,
			)
		}

		if containsNetworkIdentifier(parsed, network) {
			return fmt.Errorf(
				"network identifier %s is invalid: %w",
				types.PrintStruct(network),
				ErrSupportedNetworksDuplicate,
			)
		}
		parsed[i] = network
	}

	return nil
}

// SupportedNetwork returns a boolean indicating if the requestNetwork
// is allowed. This should be called after the requestNetwork is asserted.
func (a *Asserter) SupportedNetwork(
	requestNetwork *types.NetworkIdentifier,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if !containsNetworkIdentifier(a.supportedNetworks, requestNetwork) {
		return ErrRequestedNetworkNotSupported
	}

	return nil
}

// ValidSupportedNetwork returns an error if a types.NetworkIdentifier
// is not valid or not supported.
func (a *Asserter) ValidSupportedNetwork(
	requestNetwork *types.NetworkIdentifier,
) error {
	if err := NetworkIdentifier(requestNetwork); err != nil {
		return fmt.Errorf(
			"network identifier %s is invalid: %w",
			types.PrintStruct(requestNetwork),
			err,
		)
	}

	if err := a.SupportedNetwork(requestNetwork); err != nil {
		return fmt.Errorf(
			"network identifier %s is not supported: %w",
			types.PrintStruct(requestNetwork),
			err,
		)
	}

	return nil
}

// AccountBalanceRequest ensures that a types.AccountBalanceRequest
// is well-formatted.
func (a *Asserter) AccountBalanceRequest(request *types.AccountBalanceRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrAccountBalanceRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := AccountIdentifier(request.AccountIdentifier); err != nil {
		return fmt.Errorf(
			"account identifier %s is invalid: %w",
			types.PrintStruct(request.AccountIdentifier),
			err,
		)
	}

	if currency := ContainsDuplicateCurrency(request.Currencies); currency != nil {
		return fmt.Errorf(
			"currency %s is invalid: %w",
			types.PrintStruct(currency),
			ErrDuplicateCurrency,
		)
	}

	if request.BlockIdentifier == nil {
		return nil
	}

	if request.BlockIdentifier != nil && !a.historicalBalanceLookup {
		return ErrAccountBalanceRequestHistoricalBalanceLookupNotSupported
	}

	return PartialBlockIdentifier(request.BlockIdentifier)
}

// BlockRequest ensures that a types.BlockRequest
// is well-formatted.
func (a *Asserter) BlockRequest(request *types.BlockRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrBlockRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	return PartialBlockIdentifier(request.BlockIdentifier)
}

// BlockTransactionRequest ensures that a types.BlockTransactionRequest
// is well-formatted.
func (a *Asserter) BlockTransactionRequest(request *types.BlockTransactionRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrBlockTransactionRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := BlockIdentifier(request.BlockIdentifier); err != nil {
		return fmt.Errorf(
			"block identifier %s is invalid: %w",
			types.PrintStruct(request.BlockIdentifier),
			err,
		)
	}

	return TransactionIdentifier(request.TransactionIdentifier)
}

// ConstructionMetadataRequest ensures that a types.ConstructionMetadataRequest
// is well-formatted.
func (a *Asserter) ConstructionMetadataRequest(request *types.ConstructionMetadataRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionMetadataRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	for _, publicKey := range request.PublicKeys {
		if err := PublicKey(publicKey); err != nil {
			return fmt.Errorf("public key %s is invalid: %w", types.PrintStruct(publicKey), err)
		}
	}

	return nil
}

// ConstructionSubmitRequest ensures that a types.ConstructionSubmitRequest
// is well-formatted.
func (a *Asserter) ConstructionSubmitRequest(request *types.ConstructionSubmitRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionSubmitRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if request.SignedTransaction == "" {
		return ErrConstructionSubmitRequestSignedTxEmpty
	}

	return nil
}

// MempoolTransactionRequest ensures that a types.MempoolTransactionRequest
// is well-formatted.
func (a *Asserter) MempoolTransactionRequest(request *types.MempoolTransactionRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrMempoolTransactionRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	return TransactionIdentifier(request.TransactionIdentifier)
}

// MetadataRequest ensures that a types.MetadataRequest
// is well-formatted.
func (a *Asserter) MetadataRequest(request *types.MetadataRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrMetadataRequestIsNil
	}

	return nil
}

// NetworkRequest ensures that a types.NetworkRequest
// is well-formatted.
func (a *Asserter) NetworkRequest(request *types.NetworkRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrNetworkRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	return nil
}

// ConstructionDeriveRequest ensures that a types.ConstructionDeriveRequest
// is well-formatted.
func (a *Asserter) ConstructionDeriveRequest(request *types.ConstructionDeriveRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionDeriveRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := PublicKey(request.PublicKey); err != nil {
		return fmt.Errorf("public key %s is invalid: %w", types.PrintStruct(request.PublicKey), err)
	}

	return nil
}

// ConstructionPreprocessRequest ensures that a types.ConstructionPreprocessRequest
// is well-formatted.
func (a *Asserter) ConstructionPreprocessRequest(
	request *types.ConstructionPreprocessRequest,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionPreprocessRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := a.Operations(request.Operations, true); err != nil {
		return fmt.Errorf(
			"operations %s are invalid: %w",
			types.PrintStruct(request.Operations),
			err,
		)
	}

	return nil
}

// ConstructionPayloadsRequest ensures that a types.ConstructionPayloadsRequest
// is well-formatted.
func (a *Asserter) ConstructionPayloadsRequest(request *types.ConstructionPayloadsRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionPayloadsRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := a.Operations(request.Operations, true); err != nil {
		return fmt.Errorf(
			"operations %s are invalid: %w",
			types.PrintStruct(request.Operations),
			err,
		)
	}

	for _, publicKey := range request.PublicKeys {
		if err := PublicKey(publicKey); err != nil {
			return fmt.Errorf("public key %s is invalid: %w", types.PrintStruct(publicKey), err)
		}
	}

	return nil
}

// ConstructionCombineRequest ensures that a types.ConstructionCombineRequest
// is well-formatted.
func (a *Asserter) ConstructionCombineRequest(request *types.ConstructionCombineRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionCombineRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if len(request.UnsignedTransaction) == 0 {
		return ErrConstructionCombineRequestUnsignedTxEmpty
	}

	if err := Signatures(request.Signatures); err != nil {
		return fmt.Errorf(
			"signatures %s are invalid: %w",
			types.PrintStruct(request.Signatures),
			err,
		)
	}

	return nil
}

// ConstructionHashRequest ensures that a types.ConstructionHashRequest
// is well-formatted.
func (a *Asserter) ConstructionHashRequest(request *types.ConstructionHashRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionHashRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if len(request.SignedTransaction) == 0 {
		return ErrConstructionHashRequestSignedTxEmpty
	}

	return nil
}

// ConstructionParseRequest ensures that a types.ConstructionParseRequest
// is well-formatted.
func (a *Asserter) ConstructionParseRequest(request *types.ConstructionParseRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrConstructionParseRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if len(request.Transaction) == 0 {
		return ErrConstructionParseRequestEmpty
	}

	return nil
}

// ValidCallMethod returns an error if a CallRequest.Method
// is not valid.
func (a *Asserter) ValidCallMethod(method string) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if len(method) == 0 {
		return ErrCallMethodEmpty
	}

	if _, ok := a.callMethods[method]; !ok {
		return ErrCallMethodUnsupported
	}

	return nil
}

// CallRequest ensures that a types.CallRequest
// is well-formatted.
func (a *Asserter) CallRequest(request *types.CallRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrCallRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := a.ValidCallMethod(request.Method); err != nil {
		return fmt.Errorf("method %s is invalid: %w", request.Method, err)
	}

	return nil
}

// AccountCoinsRequest ensures that a types.AccountCoinsRequest
// is well-formatted.
func (a *Asserter) AccountCoinsRequest(request *types.AccountCoinsRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrAccountCoinsRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if err := AccountIdentifier(request.AccountIdentifier); err != nil {
		return fmt.Errorf(
			"account identifier %s is invalid: %w",
			types.PrintStruct(request.AccountIdentifier),
			err,
		)
	}

	if request.IncludeMempool && !a.mempoolCoins {
		return ErrMempoolCoinsNotSupported
	}

	if currency := ContainsDuplicateCurrency(request.Currencies); currency != nil {
		return fmt.Errorf(
			"currency %s is invalid: %w",
			types.PrintStruct(currency),
			ErrDuplicateCurrency,
		)
	}

	return nil
}

// EventsBlocksRequest ensures that a types.EventsBlocksRequest
// is well-formatted.
func (a *Asserter) EventsBlocksRequest(request *types.EventsBlocksRequest) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrEventsBlocksRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if request.Offset != nil && *request.Offset < 0 {
		return ErrOffsetIsNegative
	}

	if request.Limit != nil && *request.Limit < 0 {
		return ErrLimitIsNegative
	}

	return nil
}

// SearchTransactionsRequest ensures that a types.SearchTransactionsRequest
// is well-formatted.
func (a *Asserter) SearchTransactionsRequest( // nolint:gocognit
	request *types.SearchTransactionsRequest,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if request == nil {
		return ErrSearchTransactionsRequestIsNil
	}

	if err := a.ValidSupportedNetwork(request.NetworkIdentifier); err != nil {
		return err
	}

	if request.Operator != nil {
		switch *request.Operator {
		case types.AND, types.OR:
		default:
			return ErrOperatorInvalid
		}
	}

	if request.MaxBlock != nil && *request.MaxBlock < 0 {
		return ErrMaxBlockInvalid
	}

	if request.Offset != nil && *request.Offset < 0 {
		return ErrOffsetIsNegative
	}

	if request.Limit != nil && *request.Limit < 0 {
		return ErrLimitIsNegative
	}

	if request.TransactionIdentifier != nil {
		if err := TransactionIdentifier(request.TransactionIdentifier); err != nil {
			return fmt.Errorf(
				"transaction identifier %s is invalid: %w",
				types.PrintStruct(request.TransactionIdentifier),
				err,
			)
		}
	}

	if request.AccountIdentifier != nil {
		if err := AccountIdentifier(request.AccountIdentifier); err != nil {
			return fmt.Errorf(
				"account identifier %s is invalid: %w",
				types.PrintStruct(request.AccountIdentifier),
				err,
			)
		}
	}

	if request.CoinIdentifier != nil {
		if err := CoinIdentifier(request.CoinIdentifier); err != nil {
			return fmt.Errorf(
				"coin identifier %s is invalid: %w",
				types.PrintStruct(request.CoinIdentifier),
				err,
			)
		}
	}

	if request.Currency != nil {
		if err := Currency(request.Currency); err != nil {
			return fmt.Errorf(
				"currency %s is invalid: %w",
				types.PrintStruct(request.Currency),
				err,
			)
		}
	}

	if request.Status != nil {
		if err := a.OperationStatus(request.Status, false); err != nil {
			return fmt.Errorf("operation status %s is invalid: %w", *request.Status, err)
		}
	}

	if request.Type != nil {
		if err := a.OperationType(*request.Type); err != nil {
			return fmt.Errorf("operation type %s is invalid: %w", *request.Type, err)
		}
	}

	if request.Address != nil && len(*request.Address) == 0 {
		return ErrAccountAddrMissing
	}

	return nil
}
