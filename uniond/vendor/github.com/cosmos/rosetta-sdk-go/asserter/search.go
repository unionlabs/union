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

// SearchTransactionsResponse ensures a
// *types.SearchTransactionsResponse is valid.
func (a *Asserter) SearchTransactionsResponse(
	response *types.SearchTransactionsResponse,
) error {
	if a == nil {
		return ErrAsserterNotInitialized
	}

	if response.NextOffset != nil && *response.NextOffset < 0 {
		return ErrNextOffsetInvalid
	}

	if response.TotalCount < 0 {
		return ErrTotalCountInvalid
	}

	for _, blockTransaction := range response.Transactions {
		if err := BlockIdentifier(blockTransaction.BlockIdentifier); err != nil {
			return fmt.Errorf(
				"block identifier %s is invalid: %w",
				types.PrintStruct(blockTransaction.BlockIdentifier),
				err,
			)
		}

		if err := a.Transaction(blockTransaction.Transaction); err != nil {
			return fmt.Errorf(
				"transaction %s is invalid: %w",
				types.PrintStruct(blockTransaction.Transaction),
				err,
			)
		}
	}

	return nil
}
