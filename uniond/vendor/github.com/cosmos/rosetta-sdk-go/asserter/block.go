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
	"math/big"

	"github.com/coinbase/rosetta-sdk-go/types"
)

const (
	// MinUnixEpoch is the unix epoch time in milliseconds of
	// 01/01/2000 at 12:00:00 AM.
	MinUnixEpoch = 946713600000

	// MaxUnixEpoch is the unix epoch time in milliseconds of
	// 01/01/2040 at 12:00:00 AM.
	MaxUnixEpoch = 2209017600000
)

// Currency ensures a *types.Currency is valid.
func Currency(currency *types.Currency) error {
	if currency == nil {
		return ErrAmountCurrencyIsNil
	}

	if currency.Symbol == "" {
		return ErrAmountCurrencySymbolEmpty
	}

	if currency.Decimals < 0 {
		return ErrAmountCurrencyHasNegDecimals
	}

	return nil
}

// Amount ensures a types.Amount has an
// integer value, specified precision, and symbol.
func Amount(amount *types.Amount) error {
	if amount == nil || amount.Value == "" {
		return ErrAmountValueMissing
	}

	_, ok := new(big.Int).SetString(amount.Value, 10)
	if !ok {
		return ErrAmountIsNotInt
	}

	return Currency(amount.Currency)
}

// OperationIdentifier returns an error if index of the
// types.Operation is out-of-order or if the NetworkIndex is
// invalid.
func OperationIdentifier(
	identifier *types.OperationIdentifier,
	index int64,
) error {
	if identifier == nil {
		return ErrOperationIdentifierIndexIsNil
	}

	if identifier.Index != index {
		return fmt.Errorf(
			"expected identifier index %d but got %d: %w",
			index,
			identifier.Index,
			ErrOperationIdentifierIndexOutOfOrder,
		)
	}

	if identifier.NetworkIndex != nil && *identifier.NetworkIndex < 0 {
		return ErrOperationIdentifierNetworkIndexInvalid
	}

	return nil
}

// AccountIdentifier returns an error if a types.AccountIdentifier
// is missing an address or a provided SubAccount is missing an identifier.
func AccountIdentifier(account *types.AccountIdentifier) error {
	if account == nil {
		return ErrAccountIsNil
	}

	if account.Address == "" {
		return ErrAccountAddrMissing
	}

	if account.SubAccount == nil {
		return nil
	}

	if account.SubAccount.Address == "" {
		return ErrAccountSubAccountAddrMissing
	}

	return nil
}

// containsString checks if an string is contained in a slice
// of strings.
func containsString(valid []string, value string) bool {
	for _, v := range valid {
		if v == value {
			return true
		}
	}

	return false
}

// containsInt64 checks if an int64 is contained in a slice
// of Int64.
func containsInt64(valid []int64, value int64) bool {
	for _, v := range valid {
		if v == value {
			return true
		}
	}

	return false
}

// OperationStatus returns an error if an operation.Status
// is not valid.
func (a *Asserter) OperationStatus(status *string, construction bool) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	// As of rosetta-specifications@v1.4.7, populating
	// the Operation.Status field is deprecated for construction,
	// however, many implementations may still do this. Therefore,
	// we need to handle a populated but empty Operation.Status
	// field gracefully.
	if status == nil || len(*status) == 0 {
		if construction {
			return nil
		}

		return ErrOperationStatusMissing
	}

	if construction {
		return ErrOperationStatusNotEmptyForConstruction
	}

	if _, ok := a.operationStatusMap[*status]; !ok {
		return fmt.Errorf("operation status %s is invalid: %w", *status, ErrOperationStatusInvalid)
	}

	return nil
}

// OperationType returns an error if an operation.Type
// is not valid.
func (a *Asserter) OperationType(t string) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if t == "" || !containsString(a.operationTypes, t) {
		return fmt.Errorf("operation type %s is invalid: %w", t, ErrOperationTypeInvalid)
	}

	return nil
}

// Operation ensures a types.Operation has a valid
// type, status, and amount.
func (a *Asserter) Operation(
	operation *types.Operation,
	index int64,
	construction bool,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if operation == nil {
		return ErrOperationIsNil
	}

	if err := OperationIdentifier(operation.OperationIdentifier, index); err != nil {
		return fmt.Errorf(
			"operation identifier %s is invalid in operation %d: %w",
			types.PrintStruct(operation.OperationIdentifier),
			index,
			err,
		)
	}

	if err := a.OperationType(operation.Type); err != nil {
		return fmt.Errorf(
			"operation type %s is invalid in operation %d: %w",
			types.PrintStruct(operation.Type),
			index,
			err,
		)
	}

	if err := a.OperationStatus(operation.Status, construction); err != nil {
		return fmt.Errorf(
			"operation status %s is invalid in operation %d: %w",
			types.PrintStruct(operation.Status),
			index,
			err,
		)
	}

	if operation.Amount == nil {
		return nil
	}

	if err := AccountIdentifier(operation.Account); err != nil {
		return fmt.Errorf(
			"operation account identifier %s is invalid in operation %d: %w",
			types.PrintStruct(operation.Account),
			index,
			err,
		)
	}

	if err := Amount(operation.Amount); err != nil {
		return fmt.Errorf(
			"operation amount %s is invalid in operation %d: %w",
			types.PrintStruct(operation.Amount),
			index,
			err,
		)
	}

	if operation.CoinChange == nil {
		return nil
	}

	if err := CoinChange(operation.CoinChange); err != nil {
		return fmt.Errorf(
			"operation coin change %s is invalid in operation %d: %w",
			types.PrintStruct(operation.CoinChange),
			index,
			err,
		)
	}

	return nil
}

// BlockIdentifier ensures a types.BlockIdentifier
// is well-formatted.
func BlockIdentifier(blockIdentifier *types.BlockIdentifier) error {
	if blockIdentifier == nil {
		return ErrBlockIdentifierIsNil
	}

	if blockIdentifier.Hash == "" {
		return ErrBlockIdentifierHashMissing
	}

	if blockIdentifier.Index < 0 {
		return ErrBlockIdentifierIndexIsNeg
	}

	return nil
}

// PartialBlockIdentifier ensures a types.PartialBlockIdentifier
// is well-formatted.
func PartialBlockIdentifier(blockIdentifier *types.PartialBlockIdentifier) error {
	if blockIdentifier == nil {
		return ErrPartialBlockIdentifierIsNil
	}

	if blockIdentifier.Hash != nil && *blockIdentifier.Hash == "" {
		return ErrPartialBlockIdentifierHashIsEmpty
	}

	if blockIdentifier.Index != nil && *blockIdentifier.Index < 0 {
		return ErrPartialBlockIdentifierIndexIsNegative
	}

	return nil
}

// TransactionIdentifier returns an error if a
// types.TransactionIdentifier has an invalid hash.
func TransactionIdentifier(
	transactionIdentifier *types.TransactionIdentifier,
) error {
	if transactionIdentifier == nil {
		return ErrTxIdentifierIsNil
	}

	if transactionIdentifier.Hash == "" {
		return ErrTxIdentifierHashMissing
	}

	return nil
}

// Operations returns an error if any *types.Operation
// in a []*types.Operation is invalid.
func (a *Asserter) Operations( // nolint:gocognit
	operations []*types.Operation,
	construction bool,
) error {
	if len(operations) == 0 && construction {
		return ErrNoOperationsForConstruction
	}

	paymentTotal := big.NewInt(0)
	feeTotal := big.NewInt(0)
	paymentCount := 0
	feeCount := 0
	relatedOpsExists := false
	for i, op := range operations {
		// Ensure operations are sorted
		if err := a.Operation(op, int64(i), construction); err != nil {
			return fmt.Errorf("operation %s is invalid: %w", types.PrintStruct(op), err)
		}

		if a.validations.Enabled {
			if op.Type == a.validations.Payment.Name {
				val, _ := new(big.Int).SetString(op.Amount.Value, 10)
				paymentTotal.Add(paymentTotal, val)
				paymentCount++
			}

			if op.Type == a.validations.Fee.Name {
				if op.RelatedOperations != nil {
					return fmt.Errorf(
						"operation %s is invalid with operation index %d: %w",
						types.PrintStruct(op),
						op.OperationIdentifier.Index,
						ErrRelatedOperationInFeeNotAllowed,
					)
				}

				val, err := types.BigInt(op.Amount.Value)
				if err != nil {
					return err
				}

				// Validate that fee operation amount is negative
				if val.Sign() != -1 {
					return fmt.Errorf(
						"operation %s is invalid with operation index %d: %w",
						types.PrintStruct(op),
						op.OperationIdentifier.Index,
						ErrFeeAmountNotNegative,
					)
				}

				feeTotal.Add(feeTotal, val)
				feeCount++
			}
		}

		// Ensure an operation's related_operations are only
		// operations with an index less than the operation
		// and that there are no duplicates.
		relatedIndexes := []int64{}
		for _, relatedOp := range op.RelatedOperations {
			relatedOpsExists = true
			if relatedOp.Index >= op.OperationIdentifier.Index {
				return fmt.Errorf(
					"related operation index %d >= operation index %d: %w",
					relatedOp.Index,
					op.OperationIdentifier.Index,
					ErrRelatedOperationIndexOutOfOrder,
				)
			}

			if containsInt64(relatedIndexes, relatedOp.Index) {
				return fmt.Errorf(
					"related operation index %d found for operation index %d: %w",
					relatedOp.Index,
					op.OperationIdentifier.Index,
					ErrRelatedOperationIndexDuplicate,
				)
			}
			relatedIndexes = append(relatedIndexes, relatedOp.Index)
		}
	}
	// throw an error if relatedOps is not implemented and relatedOps is supported
	if !relatedOpsExists {
		if a.validations.Enabled && a.validations.RelatedOpsExists {
			return ErrRelatedOperationMissing
		}
	}

	// only account based validation
	if a.validations.Enabled && a.validations.ChainType == Account {
		return a.ValidatePaymentAndFee(paymentTotal, paymentCount, feeTotal, feeCount)
	}

	return nil
}

func (a *Asserter) ValidatePaymentAndFee(
	paymentTotal *big.Int,
	paymentCount int,
	feeTotal *big.Int,
	feeCount int,
) error {
	zero := big.NewInt(0)
	if a.validations.Payment.Operation.Count != -1 &&
		a.validations.Payment.Operation.Count != paymentCount {
		return ErrPaymentCountMismatch
	}

	if a.validations.Payment.Operation.ShouldBalance && paymentTotal.Cmp(zero) != 0 {
		return ErrPaymentAmountNotBalancing
	}

	if a.validations.Fee.Operation.Count != -1 && a.validations.Fee.Operation.Count != feeCount {
		return ErrFeeCountMismatch
	}

	if a.validations.Fee.Operation.ShouldBalance && feeTotal.Cmp(zero) != 0 {
		return ErrPaymentAmountNotBalancing
	}

	return nil
}

// Transaction returns an error if the types.TransactionIdentifier
// is invalid, if any types.Operation within the types.Transaction
// is invalid, or if any operation index is reused within a transaction.
func (a *Asserter) Transaction(
	transaction *types.Transaction,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if transaction == nil {
		return ErrTxIsNil
	}

	if err := TransactionIdentifier(transaction.TransactionIdentifier); err != nil {
		return fmt.Errorf(
			"transaction identifier %s is invalid: %w",
			types.PrintStruct(transaction.TransactionIdentifier),
			err,
		)
	}

	if err := a.Operations(transaction.Operations, false); err != nil {
		return fmt.Errorf(
			"invalid operation in transaction operations %s: %w",
			types.PrintStruct(transaction.Operations),
			err,
		)
	}

	if err := a.RelatedTransactions(transaction.RelatedTransactions); err != nil {
		return fmt.Errorf(
			"invalid related transaction in related transactions %s: %w",
			types.PrintStruct(transaction.RelatedTransactions),
			err,
		)
	}

	return nil
}

// RelatedTransactions returns an error if the related transactions array is non-null and non-empty
// and
// any of the related transactions contain invalid types, invalid network identifiers,
// invalid transaction identifiers, or a direction not defined by the enum.
func (a *Asserter) RelatedTransactions(relatedTransactions []*types.RelatedTransaction) error {
	if dup := DuplicateRelatedTransaction(relatedTransactions); dup != nil {
		return fmt.Errorf(
			"related transaction %s is invalid: %w",
			types.PrintStruct(dup),
			ErrDuplicateRelatedTransaction,
		)
	}

	for i, relatedTransaction := range relatedTransactions {
		if relatedTransaction.NetworkIdentifier != nil {
			if err := NetworkIdentifier(relatedTransaction.NetworkIdentifier); err != nil {
				return fmt.Errorf(
					"network identifier %s is invalid in related transaction at index %d: %w",
					types.PrintStruct(relatedTransaction.NetworkIdentifier),
					i,
					err,
				)
			}
		}

		if err := TransactionIdentifier(relatedTransaction.TransactionIdentifier); err != nil {
			return fmt.Errorf(
				"invalid transaction identifier %s in related transaction at index %d: %w",
				types.PrintStruct(relatedTransaction.TransactionIdentifier),
				i,
				err,
			)
		}

		if err := a.Direction(relatedTransaction.Direction); err != nil {
			return fmt.Errorf(
				"invalid direction %s in related transaction at index %d: %w",
				types.PrintStruct(relatedTransaction.Direction),
				i,
				err,
			)
		}
	}

	return nil
}

// DuplicateRelatedTransaction returns nil if no duplicates are found in the array and
// returns the first duplicated item found otherwise.
func DuplicateRelatedTransaction(
	items []*types.RelatedTransaction,
) *types.RelatedTransaction {
	seen := map[string]struct{}{}
	for _, item := range items {
		key := types.Hash(item)
		if _, ok := seen[key]; ok {
			return item
		}

		seen[key] = struct{}{}
	}

	return nil
}

// Direction returns an error if the value passed is not types.Forward or types.Backward
func (a *Asserter) Direction(direction types.Direction) error {
	if direction != types.Forward &&
		direction != types.Backward {
		return ErrInvalidDirection
	}

	return nil
}

// Timestamp returns an error if the timestamp
// on a block is less than or equal to 0.
func Timestamp(timestamp int64) error {
	switch {
	case timestamp < MinUnixEpoch:
		return ErrTimestampBeforeMin
	case timestamp > MaxUnixEpoch:
		return ErrTimestampAfterMax
	default:
		return nil
	}
}

// Block runs a basic set of assertions for each returned block.
func (a *Asserter) Block(
	block *types.Block,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if block == nil {
		return ErrBlockIsNil
	}

	if err := BlockIdentifier(block.BlockIdentifier); err != nil {
		return fmt.Errorf(
			"block identifier %s is invalid: %w",
			types.PrintStruct(block.BlockIdentifier),
			err,
		)
	}

	if err := BlockIdentifier(block.ParentBlockIdentifier); err != nil {
		return fmt.Errorf(
			"parent block identifier %s is invalid: %w",
			types.PrintStruct(block.ParentBlockIdentifier),
			err,
		)
	}

	// Only apply duplicate hash and index checks if the block index is not the
	// genesis index.
	if a.genesisBlock.Index != block.BlockIdentifier.Index {
		if block.BlockIdentifier.Hash == block.ParentBlockIdentifier.Hash {
			return ErrBlockHashEqualsParentBlockHash
		}

		if block.BlockIdentifier.Index <= block.ParentBlockIdentifier.Index {
			return ErrBlockIndexPrecedesParentBlockIndex
		}
	}

	// Only check for timestamp validity if timestamp start index is <=
	// the current block index.
	if a.timestampStartIndex <= block.BlockIdentifier.Index {
		if err := Timestamp(block.Timestamp); err != nil {
			return fmt.Errorf("timestamp %d is invalid: %w", block.Timestamp, err)
		}
	}

	for _, transaction := range block.Transactions {
		if err := a.Transaction(transaction); err != nil {
			return fmt.Errorf("transaction %s is invalid: %w", types.PrintStruct(transaction), err)
		}
	}

	return nil
}
