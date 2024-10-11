// Copyright 2020 The go-ethereum Authors
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

package core

import (
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/signer/core/apitypes"
)

// GnosisSafeTx is a type to parse the safe-tx returned by the relayer,
// it also conforms to the API required by the Gnosis Safe tx relay service.
// See 'SafeMultisigTransaction' on https://safe-transaction.mainnet.gnosis.io/
type GnosisSafeTx struct {
	// These fields are only used on output
	Signature  hexutil.Bytes           `json:"signature"`
	SafeTxHash common.Hash             `json:"contractTransactionHash"`
	Sender     common.MixedcaseAddress `json:"sender"`
	// These fields are used both on input and output
	Safe           common.MixedcaseAddress `json:"safe"`
	To             common.MixedcaseAddress `json:"to"`
	Value          math.Decimal256         `json:"value"`
	GasPrice       math.Decimal256         `json:"gasPrice"`
	Data           *hexutil.Bytes          `json:"data"`
	Operation      uint8                   `json:"operation"`
	GasToken       common.Address          `json:"gasToken"`
	RefundReceiver common.Address          `json:"refundReceiver"`
	BaseGas        big.Int                 `json:"baseGas"`
	SafeTxGas      big.Int                 `json:"safeTxGas"`
	Nonce          big.Int                 `json:"nonce"`
	InputExpHash   common.Hash             `json:"safeTxHash"`
	ChainId        *math.HexOrDecimal256   `json:"chainId,omitempty"`
}

// ToTypedData converts the tx to a EIP-712 Typed Data structure for signing
func (tx *GnosisSafeTx) ToTypedData() apitypes.TypedData {
	var data hexutil.Bytes
	if tx.Data != nil {
		data = *tx.Data
	}
	var domainType = []apitypes.Type{{Name: "verifyingContract", Type: "address"}}
	if tx.ChainId != nil {
		domainType = append([]apitypes.Type{{Name: "chainId", Type: "uint256"}}, domainType[0])
	}

	gnosisTypedData := apitypes.TypedData{
		Types: apitypes.Types{
			"EIP712Domain": domainType,
			"SafeTx": []apitypes.Type{
				{Name: "to", Type: "address"},
				{Name: "value", Type: "uint256"},
				{Name: "data", Type: "bytes"},
				{Name: "operation", Type: "uint8"},
				{Name: "safeTxGas", Type: "uint256"},
				{Name: "baseGas", Type: "uint256"},
				{Name: "gasPrice", Type: "uint256"},
				{Name: "gasToken", Type: "address"},
				{Name: "refundReceiver", Type: "address"},
				{Name: "nonce", Type: "uint256"},
			},
		},
		Domain: apitypes.TypedDataDomain{
			VerifyingContract: tx.Safe.Address().Hex(),
			ChainId:           tx.ChainId,
		},
		PrimaryType: "SafeTx",
		Message: apitypes.TypedDataMessage{
			"to":             tx.To.Address().Hex(),
			"value":          tx.Value.String(),
			"data":           data,
			"operation":      fmt.Sprintf("%d", tx.Operation),
			"safeTxGas":      fmt.Sprintf("%#d", &tx.SafeTxGas),
			"baseGas":        fmt.Sprintf("%#d", &tx.BaseGas),
			"gasPrice":       tx.GasPrice.String(),
			"gasToken":       tx.GasToken.Hex(),
			"refundReceiver": tx.RefundReceiver.Hex(),
			"nonce":          fmt.Sprintf("%d", tx.Nonce.Uint64()),
		},
	}
	return gnosisTypedData
}

// ArgsForValidation returns a SendTxArgs struct, which can be used for the
// common validations, e.g. look up 4byte destinations
func (tx *GnosisSafeTx) ArgsForValidation() *apitypes.SendTxArgs {
	gp := hexutil.Big(tx.GasPrice)
	args := &apitypes.SendTxArgs{
		From:     tx.Safe,
		To:       &tx.To,
		Gas:      hexutil.Uint64(tx.SafeTxGas.Uint64()),
		GasPrice: &gp,
		Value:    hexutil.Big(tx.Value),
		Nonce:    hexutil.Uint64(tx.Nonce.Uint64()),
		Data:     tx.Data,
		Input:    nil,
		ChainID:  (*hexutil.Big)(tx.ChainId),
	}
	return args
}
