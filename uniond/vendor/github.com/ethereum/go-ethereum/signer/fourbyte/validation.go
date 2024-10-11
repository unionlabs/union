// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package fourbyte

import (
	"bytes"
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/signer/core/apitypes"
)

// ValidateTransaction does a number of checks on the supplied transaction, and
// returns either a list of warnings, or an error (indicating that the transaction
// should be immediately rejected).
func (db *Database) ValidateTransaction(selector *string, tx *apitypes.SendTxArgs) (*apitypes.ValidationMessages, error) {
	messages := new(apitypes.ValidationMessages)

	// Prevent accidental erroneous usage of both 'input' and 'data' (show stopper)
	if tx.Data != nil && tx.Input != nil && !bytes.Equal(*tx.Data, *tx.Input) {
		return nil, errors.New(`ambiguous request: both "data" and "input" are set and are not identical`)
	}
	// Place data on 'data', and nil 'input'
	var data []byte
	if tx.Input != nil {
		tx.Data = tx.Input
		tx.Input = nil
	}
	if tx.Data != nil {
		data = *tx.Data
	}
	// Contract creation doesn't validate call data, handle first
	if tx.To == nil {
		// Contract creation should contain sufficient data to deploy a contract. A
		// typical error is omitting sender due to some quirk in the javascript call
		// e.g. https://github.com/ethereum/go-ethereum/issues/16106.
		if len(data) == 0 {
			// Prevent sending ether into black hole (show stopper)
			if tx.Value.ToInt().Cmp(big.NewInt(0)) > 0 {
				return nil, errors.New("transaction will create a contract with value but empty code")
			}
			// No value submitted at least, critically Warn, but don't blow up
			messages.Crit("Transaction will create a contract with empty code")
		} else if len(data) < 40 { // arbitrary heuristic limit
			messages.Warn(fmt.Sprintf("Transaction will create a contract, but the payload is suspiciously small (%d bytes)", len(data)))
		}
		// Method selector should be nil for contract creation
		if selector != nil {
			messages.Warn("Transaction will create a contract, but method selector supplied, indicating an intent to call a method")
		}
		return messages, nil
	}
	// Not a contract creation, validate as a plain transaction
	if !tx.To.ValidChecksum() {
		messages.Warn("Invalid checksum on recipient address")
	}
	if bytes.Equal(tx.To.Address().Bytes(), common.Address{}.Bytes()) {
		messages.Crit("Transaction recipient is the zero address")
	}
	switch {
	case tx.GasPrice == nil && tx.MaxFeePerGas == nil:
		messages.Crit("Neither 'gasPrice' nor 'maxFeePerGas' specified.")
	case tx.GasPrice == nil && tx.MaxPriorityFeePerGas == nil:
		messages.Crit("Neither 'gasPrice' nor 'maxPriorityFeePerGas' specified.")
	case tx.GasPrice != nil && tx.MaxFeePerGas != nil:
		messages.Crit("Both 'gasPrice' and 'maxFeePerGas' specified.")
	case tx.GasPrice != nil && tx.MaxPriorityFeePerGas != nil:
		messages.Crit("Both 'gasPrice' and 'maxPriorityFeePerGas' specified.")
	}
	// Semantic fields validated, try to make heads or tails of the call data
	db.ValidateCallData(selector, data, messages)
	return messages, nil
}

// ValidateCallData checks if the ABI call-data + method selector (if given) can
// be parsed and seems to match.
func (db *Database) ValidateCallData(selector *string, data []byte, messages *apitypes.ValidationMessages) {
	// If the data is empty, we have a plain value transfer, nothing more to do
	if len(data) == 0 {
		return
	}
	// Validate the call data that it has the 4byte prefix and the rest divisible by 32 bytes
	if len(data) < 4 {
		messages.Warn("Transaction data is not valid ABI (missing the 4 byte call prefix)")
		return
	}
	if n := len(data) - 4; n%32 != 0 {
		messages.Warn(fmt.Sprintf("Transaction data is not valid ABI (length should be a multiple of 32 (was %d))", n))
	}
	// If a custom method selector was provided, validate with that
	if selector != nil {
		if info, err := verifySelector(*selector, data); err != nil {
			messages.Warn(fmt.Sprintf("Transaction contains data, but provided ABI signature could not be matched: %v", err))
		} else {
			messages.Info(fmt.Sprintf("Transaction invokes the following method: %q", info.String()))
			db.AddSelector(*selector, data[:4])
		}
		return
	}
	// No method selector was provided, check the database for embedded ones
	embedded, err := db.Selector(data[:4])
	if err != nil {
		messages.Warn(fmt.Sprintf("Transaction contains data, but the ABI signature could not be found: %v", err))
		return
	}
	if info, err := verifySelector(embedded, data); err != nil {
		messages.Warn(fmt.Sprintf("Transaction contains data, but provided ABI signature could not be verified: %v", err))
	} else {
		messages.Info(fmt.Sprintf("Transaction invokes the following method: %q", info.String()))
	}
}
