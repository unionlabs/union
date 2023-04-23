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
	"bytes"
	"fmt"

	"github.com/coinbase/rosetta-sdk-go/types"
)

// StringArray ensures all strings in an array
// are non-empty strings and not duplicates.
func StringArray(arrName string, arr []string) error {
	if len(arr) == 0 {
		return fmt.Errorf("string array %s is empty: %w", arrName, ErrStringArrayEmpty)
	}

	parsed := map[string]struct{}{}
	for _, s := range arr {
		if s == "" {
			return fmt.Errorf(
				"string array %s has an empty string: %w",
				arrName,
				ErrStringArrayEmptyString,
			)
		}

		if _, ok := parsed[s]; ok {
			return fmt.Errorf(
				"string array %s contains a duplicate %s: %w",
				arrName,
				s,
				ErrStringArrayDuplicateString,
			)
		}

		parsed[s] = struct{}{}
	}

	return nil
}

// AccountArray ensures all *types.AccountIdentifier in an array
// are valid and not duplicates.
func AccountArray(arrName string, arr []*types.AccountIdentifier) error {
	if len(arr) == 0 {
		return fmt.Errorf("account array %s is empty: %w", arrName, ErrAccountArrayEmpty)
	}

	parsed := map[string]struct{}{}
	for _, s := range arr {
		if err := AccountIdentifier(s); err != nil {
			return fmt.Errorf(
				"account array %s has an invalid account identifier: %w",
				arrName,
				ErrAccountArrayInvalidAccount,
			)
		}

		if _, ok := parsed[types.Hash(s)]; ok {
			return fmt.Errorf(
				"account array %s contains a duplicate account identifier %s: %w",
				arrName,
				types.PrintStruct(s),
				ErrAccountArrayDuplicateAccount,
			)
		}

		parsed[types.Hash(s)] = struct{}{}
	}

	return nil
}

// BytesArrayZero returns a boolean indicating if
// all elements in an array are 0.
func BytesArrayZero(arr []byte) bool {
	return bytes.Equal(arr, make([]byte, len(arr)))
}
