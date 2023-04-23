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

// ContainsDuplicateCurrency retruns a boolean indicating
// if an array of *types.Currency contains any duplicate currencies.
func ContainsDuplicateCurrency(currencies []*types.Currency) *types.Currency {
	seen := map[string]struct{}{}
	for _, curr := range currencies {
		key := types.Hash(curr)
		if _, ok := seen[key]; ok {
			return curr
		}

		seen[key] = struct{}{}
	}

	return nil
}

// ContainsCurrency returns a boolean indicating if a
// *types.Currency is contained within a slice of
// *types.Currency. The check for equality takes
// into account everything within the types.Currency
// struct (including currency.Metadata).
func ContainsCurrency(currencies []*types.Currency, currency *types.Currency) bool {
	for _, curr := range currencies {
		if types.Hash(curr) == types.Hash(currency) {
			return true
		}
	}

	return false
}

// AssertUniqueAmounts returns an error if a slice
// of types.Amount is invalid. It is considered invalid if the same
// currency is returned multiple times (these should be
// consolidated) or if a types.Amount is considered invalid.
func AssertUniqueAmounts(amounts []*types.Amount) error {
	seen := map[string]struct{}{}
	for _, amount := range amounts {
		// Ensure a currency is used at most once
		key := types.Hash(amount.Currency)
		if _, ok := seen[key]; ok {
			return fmt.Errorf(
				"amount currency %s of amount %s is invalid: %w",
				types.PrintStruct(amount.Currency),
				types.PrintStruct(amount),
				ErrCurrencyUsedMultipleTimes,
			)
		}
		seen[key] = struct{}{}

		// Check amount for validity
		if err := Amount(amount); err != nil {
			return fmt.Errorf("amount %s is invalid: %w", types.PrintStruct(amount), err)
		}
	}

	return nil
}

// AccountBalanceResponse returns an error if the provided
// types.BlockIdentifier is invalid, if the requestBlock
// is not nil and not equal to the response block, or
// if the same currency is present in multiple amounts.
func AccountBalanceResponse(
	requestBlock *types.PartialBlockIdentifier,
	response *types.AccountBalanceResponse,
) error {
	if err := BlockIdentifier(response.BlockIdentifier); err != nil {
		return fmt.Errorf(
			"block identifier %s is invalid: %w",
			types.PrintStruct(response.BlockIdentifier),
			err,
		)
	}

	if err := AssertUniqueAmounts(response.Balances); err != nil {
		return fmt.Errorf(
			"balance amounts %s are invalid: %w",
			types.PrintStruct(response.Balances),
			err,
		)
	}

	if requestBlock == nil {
		return nil
	}

	if requestBlock.Hash != nil && *requestBlock.Hash != response.BlockIdentifier.Hash {
		return fmt.Errorf(
			"requested block hash %s, but got %s: %w",
			*requestBlock.Hash,
			response.BlockIdentifier.Hash,
			ErrReturnedBlockHashMismatch,
		)
	}

	if requestBlock.Index != nil && *requestBlock.Index != response.BlockIdentifier.Index {
		return fmt.Errorf(
			"requested block index %d, but got %d: %w",
			*requestBlock.Index,
			response.BlockIdentifier.Index,
			ErrReturnedBlockIndexMismatch,
		)
	}

	return nil
}

// AccountCoinsResponse returns an error if the provided
// *types.AccountCoinsResponse is invalid.
func AccountCoinsResponse(
	response *types.AccountCoinsResponse,
) error {
	if err := BlockIdentifier(response.BlockIdentifier); err != nil {
		return fmt.Errorf(
			"block identifier %s is invalid: %w",
			types.PrintStruct(response.BlockIdentifier),
			err,
		)
	}

	if err := Coins(response.Coins); err != nil {
		return fmt.Errorf("coins %s are invalid: %w", types.PrintStruct(response.Coins), err)
	}

	return nil
}
