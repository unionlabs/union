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

package types

import (
	"crypto/sha256"
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"math/big"

	"github.com/mitchellh/mapstructure"
)

// ConstructPartialBlockIdentifier constructs a *PartialBlockIdentifier
// from a *BlockIdentifier.
//
// It is useful to have this helper when making block requests
// with the fetcher.
func ConstructPartialBlockIdentifier(
	blockIdentifier *BlockIdentifier,
) *PartialBlockIdentifier {
	return &PartialBlockIdentifier{
		Hash:  &blockIdentifier.Hash,
		Index: &blockIdentifier.Index,
	}
}

// hashBytes returns a hex-encoded sha256 hash of the provided
// byte slice.
func hashBytes(data []byte) string {
	h := sha256.New()
	_, err := h.Write(data)
	if err != nil {
		log.Fatal(fmt.Errorf("unable to hash data %s: %w", string(data), err))
	}

	return fmt.Sprintf("%x", h.Sum(nil))
}

// Hash returns a deterministic hash for any interface.
// This works because Golang's JSON marshaler sorts all map keys, recursively.
// Source: https://golang.org/pkg/encoding/json/#Marshal
// Inspiration:
// https://github.com/onsi/gomega/blob/c0be49994280db30b6b68390f67126d773bc5558/matchers/match_json_matcher.go#L16
//
// It is important to note that any interface that is a slice
// or contains slices will not be equal if the slice ordering is
// different.
func Hash(i interface{}) string {
	// Convert interface to JSON object (not necessarily ordered if struct
	// contains json.RawMessage)
	a, err := json.Marshal(i)
	if err != nil {
		log.Fatal(fmt.Errorf("unable to marshal %+v: %w", i, err))
	}

	// Convert JSON object to interface (all json.RawMessage converted to go types)
	var b interface{}
	if err := json.Unmarshal(a, &b); err != nil {
		log.Fatal(fmt.Errorf("unable to unmarshal %+v: %w", a, err))
	}

	// Convert interface to JSON object (all map keys ordered)
	c, err := json.Marshal(b)
	if err != nil {
		log.Fatal(fmt.Errorf("unable to marshal %+v: %w", b, err))
	}

	return hashBytes(c)
}

// BigInt returns a *big.Int representation of a value.
func BigInt(value string) (*big.Int, error) {
	parsedVal, ok := new(big.Int).SetString(value, 10)
	if !ok {
		return nil, fmt.Errorf("%s is not an integer", value)
	}

	return parsedVal, nil
}

// AmountValue returns a *big.Int representation of an
// Amount.Value or an error.
func AmountValue(amount *Amount) (*big.Int, error) {
	if amount == nil {
		return nil, errors.New("amount value cannot be nil")
	}

	return BigInt(amount.Value)
}

// AddValues adds string amounts using
// big.Int.
func AddValues(
	a string,
	b string,
) (string, error) {
	aVal, err := BigInt(a)
	if err != nil {
		return "", err
	}

	bVal, err := BigInt(b)
	if err != nil {
		return "", err
	}

	newVal := new(big.Int).Add(aVal, bVal)
	return newVal.String(), nil
}

// SubtractValues subtracts a-b using
// big.Int.
func SubtractValues(
	a string,
	b string,
) (string, error) {
	aVal, err := BigInt(a)
	if err != nil {
		return "", err
	}

	bVal, err := BigInt(b)
	if err != nil {
		return "", err
	}

	newVal := new(big.Int).Sub(aVal, bVal)
	return newVal.String(), nil
}

// MultiplyValues multiplies a*b using
// big.Int.
func MultiplyValues(
	a string,
	b string,
) (string, error) {
	aVal, err := BigInt(a)
	if err != nil {
		return "", err
	}

	bVal, err := BigInt(b)
	if err != nil {
		return "", err
	}

	newVal := new(big.Int).Mul(aVal, bVal)
	return newVal.String(), nil
}

// DivideValues divides a/b using
// big.Int.
func DivideValues(
	a string,
	b string,
) (string, error) {
	aVal, err := BigInt(a)
	if err != nil {
		return "", err
	}

	bVal, err := BigInt(b)
	if err != nil {
		return "", err
	}

	newVal := new(big.Int).Div(aVal, bVal)
	return newVal.String(), nil
}

// NegateValue flips the sign of a value.
func NegateValue(
	val string,
) (string, error) {
	existing, err := BigInt(val)
	if err != nil {
		return "", err
	}

	return new(big.Int).Neg(existing).String(), nil
}

// AccountString returns a human-readable representation of a
// *AccountIdentifier.
func AccountString(account *AccountIdentifier) string {
	if account.SubAccount == nil {
		return account.Address
	}

	if account.SubAccount.Metadata == nil {
		return fmt.Sprintf(
			"%s:%s",
			account.Address,
			account.SubAccount.Address,
		)
	}

	return fmt.Sprintf(
		"%s:%s:%+v",
		account.Address,
		account.SubAccount.Address,
		account.SubAccount.Metadata,
	)
}

// CurrencyString returns a human-readable representation
// of a *Currency.
func CurrencyString(currency *Currency) string {
	if currency.Metadata == nil {
		return fmt.Sprintf("%s:%d", currency.Symbol, currency.Decimals)
	}

	return fmt.Sprintf(
		"%s:%d:%+v",
		currency.Symbol,
		currency.Decimals,
		currency.Metadata,
	)
}

// PrettyPrintStruct marshals a struct to JSON and returns
// it as a string.
func PrettyPrintStruct(val interface{}) string {
	prettyStruct, err := json.MarshalIndent(
		val,
		"",
		" ",
	)
	if err != nil {
		log.Fatal(err)
	}

	return string(prettyStruct)
}

// PrintStruct marshals a struct to JSON and returns
// it as a string without newlines.
func PrintStruct(val interface{}) string {
	str, err := json.Marshal(
		val,
	)
	if err != nil {
		log.Fatal(err)
	}

	return string(str)
}

// MarshalMap attempts to marshal an interface into a map[string]interface{}.
// This function is used similarly to json.Marshal.
func MarshalMap(input interface{}) (map[string]interface{}, error) {
	if input == nil {
		return nil, nil
	}

	// Only create output if input is not nil, otherwise we will
	// return a map for a nil input.
	output := map[string]interface{}{}
	decoder, err := mapstructure.NewDecoder(&mapstructure.DecoderConfig{
		TagName: "json",
		Result:  &output,
	})
	if err != nil {
		return nil, err
	}

	if err := decoder.Decode(input); err != nil {
		return nil, err
	}

	return output, nil
}

// UnmarshalMap attempts to unmarshal a map[string]interface{} into an
// interface. This function is used similarly to json.Unmarshal.
func UnmarshalMap(metadata map[string]interface{}, output interface{}) error {
	decoder, err := mapstructure.NewDecoder(&mapstructure.DecoderConfig{
		TagName: "json",
		Result:  output,
	})
	if err != nil {
		return err
	}

	return decoder.Decode(metadata)
}

// ExtractAmount returns the Amount from a slice of Balance
// pertaining to an AccountAndCurrency.
func ExtractAmount(
	balances []*Amount,
	currency *Currency,
) *Amount {
	for _, b := range balances {
		if Hash(b.Currency) != Hash(currency) {
			continue
		}

		return b
	}

	// return a 0 amount if currency isn't found in balances
	return &Amount{Value: "0", Currency: currency}
}

// String returns a pointer to the
// string passed as an argument.
func String(s string) *string {
	return &s
}

// Int64 returns a pointer to the
// int64 passed as an argument.
func Int64(i int64) *int64 {
	return &i
}

// Bool returns a pointer to the
// bool passed as an argument.
func Bool(b bool) *bool {
	return &b
}

// OperatorP returns a pointer to the
// Operator passed as an argument.
//
// We can't just use Operator because
// the types package already declares
// the Operator type.
func OperatorP(o Operator) *Operator {
	return &o
}
