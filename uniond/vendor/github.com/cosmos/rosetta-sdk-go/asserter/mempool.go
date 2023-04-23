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

// MempoolTransactions returns an error if any
// types.TransactionIdentifier returns is missing a hash.
// The correctness of each populated MempoolTransaction is
// asserted by Transaction.
func MempoolTransactions(
	transactions []*types.TransactionIdentifier,
) error {
	for _, t := range transactions {
		if err := TransactionIdentifier(t); err != nil {
			return fmt.Errorf("transaction identifier %s is invalid: %w", types.PrintStruct(t), err)
		}
	}

	return nil
}
