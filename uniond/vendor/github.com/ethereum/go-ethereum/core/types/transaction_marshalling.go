// Copyright 2021 The go-ethereum Authors
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

package types

import (
	"encoding/json"
	"errors"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/holiman/uint256"
)

// txJSON is the JSON representation of transactions.
type txJSON struct {
	Type hexutil.Uint64 `json:"type"`

	ChainID              *hexutil.Big    `json:"chainId,omitempty"`
	Nonce                *hexutil.Uint64 `json:"nonce"`
	To                   *common.Address `json:"to"`
	Gas                  *hexutil.Uint64 `json:"gas"`
	GasPrice             *hexutil.Big    `json:"gasPrice"`
	MaxPriorityFeePerGas *hexutil.Big    `json:"maxPriorityFeePerGas"`
	MaxFeePerGas         *hexutil.Big    `json:"maxFeePerGas"`
	MaxFeePerBlobGas     *hexutil.Big    `json:"maxFeePerBlobGas,omitempty"`
	Value                *hexutil.Big    `json:"value"`
	Input                *hexutil.Bytes  `json:"input"`
	AccessList           *AccessList     `json:"accessList,omitempty"`
	BlobVersionedHashes  []common.Hash   `json:"blobVersionedHashes,omitempty"`
	V                    *hexutil.Big    `json:"v"`
	R                    *hexutil.Big    `json:"r"`
	S                    *hexutil.Big    `json:"s"`
	YParity              *hexutil.Uint64 `json:"yParity,omitempty"`

	// Only used for encoding:
	Hash common.Hash `json:"hash"`
}

// yParityValue returns the YParity value from JSON. For backwards-compatibility reasons,
// this can be given in the 'v' field or the 'yParity' field. If both exist, they must match.
func (tx *txJSON) yParityValue() (*big.Int, error) {
	if tx.YParity != nil {
		val := uint64(*tx.YParity)
		if val != 0 && val != 1 {
			return nil, errors.New("'yParity' field must be 0 or 1")
		}
		bigval := new(big.Int).SetUint64(val)
		if tx.V != nil && tx.V.ToInt().Cmp(bigval) != 0 {
			return nil, errors.New("'v' and 'yParity' fields do not match")
		}
		return bigval, nil
	}
	if tx.V != nil {
		return tx.V.ToInt(), nil
	}
	return nil, errors.New("missing 'yParity' or 'v' field in transaction")
}

// MarshalJSON marshals as JSON with a hash.
func (tx *Transaction) MarshalJSON() ([]byte, error) {
	var enc txJSON
	// These are set for all tx types.
	enc.Hash = tx.Hash()
	enc.Type = hexutil.Uint64(tx.Type())

	// Other fields are set conditionally depending on tx type.
	switch itx := tx.inner.(type) {
	case *LegacyTx:
		enc.Nonce = (*hexutil.Uint64)(&itx.Nonce)
		enc.To = tx.To()
		enc.Gas = (*hexutil.Uint64)(&itx.Gas)
		enc.GasPrice = (*hexutil.Big)(itx.GasPrice)
		enc.Value = (*hexutil.Big)(itx.Value)
		enc.Input = (*hexutil.Bytes)(&itx.Data)
		enc.V = (*hexutil.Big)(itx.V)
		enc.R = (*hexutil.Big)(itx.R)
		enc.S = (*hexutil.Big)(itx.S)
		if tx.Protected() {
			enc.ChainID = (*hexutil.Big)(tx.ChainId())
		}

	case *AccessListTx:
		enc.ChainID = (*hexutil.Big)(itx.ChainID)
		enc.Nonce = (*hexutil.Uint64)(&itx.Nonce)
		enc.To = tx.To()
		enc.Gas = (*hexutil.Uint64)(&itx.Gas)
		enc.GasPrice = (*hexutil.Big)(itx.GasPrice)
		enc.Value = (*hexutil.Big)(itx.Value)
		enc.Input = (*hexutil.Bytes)(&itx.Data)
		enc.AccessList = &itx.AccessList
		enc.V = (*hexutil.Big)(itx.V)
		enc.R = (*hexutil.Big)(itx.R)
		enc.S = (*hexutil.Big)(itx.S)
		yparity := itx.V.Uint64()
		enc.YParity = (*hexutil.Uint64)(&yparity)

	case *DynamicFeeTx:
		enc.ChainID = (*hexutil.Big)(itx.ChainID)
		enc.Nonce = (*hexutil.Uint64)(&itx.Nonce)
		enc.To = tx.To()
		enc.Gas = (*hexutil.Uint64)(&itx.Gas)
		enc.MaxFeePerGas = (*hexutil.Big)(itx.GasFeeCap)
		enc.MaxPriorityFeePerGas = (*hexutil.Big)(itx.GasTipCap)
		enc.Value = (*hexutil.Big)(itx.Value)
		enc.Input = (*hexutil.Bytes)(&itx.Data)
		enc.AccessList = &itx.AccessList
		enc.V = (*hexutil.Big)(itx.V)
		enc.R = (*hexutil.Big)(itx.R)
		enc.S = (*hexutil.Big)(itx.S)
		yparity := itx.V.Uint64()
		enc.YParity = (*hexutil.Uint64)(&yparity)

	case *BlobTx:
		enc.ChainID = (*hexutil.Big)(itx.ChainID.ToBig())
		enc.Nonce = (*hexutil.Uint64)(&itx.Nonce)
		enc.Gas = (*hexutil.Uint64)(&itx.Gas)
		enc.MaxFeePerGas = (*hexutil.Big)(itx.GasFeeCap.ToBig())
		enc.MaxPriorityFeePerGas = (*hexutil.Big)(itx.GasTipCap.ToBig())
		enc.MaxFeePerBlobGas = (*hexutil.Big)(itx.BlobFeeCap.ToBig())
		enc.Value = (*hexutil.Big)(itx.Value.ToBig())
		enc.Input = (*hexutil.Bytes)(&itx.Data)
		enc.AccessList = &itx.AccessList
		enc.BlobVersionedHashes = itx.BlobHashes
		enc.To = tx.To()
		enc.V = (*hexutil.Big)(itx.V.ToBig())
		enc.R = (*hexutil.Big)(itx.R.ToBig())
		enc.S = (*hexutil.Big)(itx.S.ToBig())
		yparity := itx.V.Uint64()
		enc.YParity = (*hexutil.Uint64)(&yparity)
	}
	return json.Marshal(&enc)
}

// UnmarshalJSON unmarshals from JSON.
func (tx *Transaction) UnmarshalJSON(input []byte) error {
	var dec txJSON
	err := json.Unmarshal(input, &dec)
	if err != nil {
		return err
	}

	// Decode / verify fields according to transaction type.
	var inner TxData
	switch dec.Type {
	case LegacyTxType:
		var itx LegacyTx
		inner = &itx
		if dec.Nonce == nil {
			return errors.New("missing required field 'nonce' in transaction")
		}
		itx.Nonce = uint64(*dec.Nonce)
		if dec.To != nil {
			itx.To = dec.To
		}
		if dec.Gas == nil {
			return errors.New("missing required field 'gas' in transaction")
		}
		itx.Gas = uint64(*dec.Gas)
		if dec.GasPrice == nil {
			return errors.New("missing required field 'gasPrice' in transaction")
		}
		itx.GasPrice = (*big.Int)(dec.GasPrice)
		if dec.Value == nil {
			return errors.New("missing required field 'value' in transaction")
		}
		itx.Value = (*big.Int)(dec.Value)
		if dec.Input == nil {
			return errors.New("missing required field 'input' in transaction")
		}
		itx.Data = *dec.Input

		// signature R
		if dec.R == nil {
			return errors.New("missing required field 'r' in transaction")
		}
		itx.R = (*big.Int)(dec.R)
		// signature S
		if dec.S == nil {
			return errors.New("missing required field 's' in transaction")
		}
		itx.S = (*big.Int)(dec.S)
		// signature V
		if dec.V == nil {
			return errors.New("missing required field 'v' in transaction")
		}
		itx.V = (*big.Int)(dec.V)
		if itx.V.Sign() != 0 || itx.R.Sign() != 0 || itx.S.Sign() != 0 {
			if err := sanityCheckSignature(itx.V, itx.R, itx.S, true); err != nil {
				return err
			}
		}

	case AccessListTxType:
		var itx AccessListTx
		inner = &itx
		if dec.ChainID == nil {
			return errors.New("missing required field 'chainId' in transaction")
		}
		itx.ChainID = (*big.Int)(dec.ChainID)
		if dec.Nonce == nil {
			return errors.New("missing required field 'nonce' in transaction")
		}
		itx.Nonce = uint64(*dec.Nonce)
		if dec.To != nil {
			itx.To = dec.To
		}
		if dec.Gas == nil {
			return errors.New("missing required field 'gas' in transaction")
		}
		itx.Gas = uint64(*dec.Gas)
		if dec.GasPrice == nil {
			return errors.New("missing required field 'gasPrice' in transaction")
		}
		itx.GasPrice = (*big.Int)(dec.GasPrice)
		if dec.Value == nil {
			return errors.New("missing required field 'value' in transaction")
		}
		itx.Value = (*big.Int)(dec.Value)
		if dec.Input == nil {
			return errors.New("missing required field 'input' in transaction")
		}
		itx.Data = *dec.Input
		if dec.AccessList != nil {
			itx.AccessList = *dec.AccessList
		}

		// signature R
		if dec.R == nil {
			return errors.New("missing required field 'r' in transaction")
		}
		itx.R = (*big.Int)(dec.R)
		// signature S
		if dec.S == nil {
			return errors.New("missing required field 's' in transaction")
		}
		itx.S = (*big.Int)(dec.S)
		// signature V
		itx.V, err = dec.yParityValue()
		if err != nil {
			return err
		}
		if itx.V.Sign() != 0 || itx.R.Sign() != 0 || itx.S.Sign() != 0 {
			if err := sanityCheckSignature(itx.V, itx.R, itx.S, false); err != nil {
				return err
			}
		}

	case DynamicFeeTxType:
		var itx DynamicFeeTx
		inner = &itx
		if dec.ChainID == nil {
			return errors.New("missing required field 'chainId' in transaction")
		}
		itx.ChainID = (*big.Int)(dec.ChainID)
		if dec.Nonce == nil {
			return errors.New("missing required field 'nonce' in transaction")
		}
		itx.Nonce = uint64(*dec.Nonce)
		if dec.To != nil {
			itx.To = dec.To
		}
		if dec.Gas == nil {
			return errors.New("missing required field 'gas' for txdata")
		}
		itx.Gas = uint64(*dec.Gas)
		if dec.MaxPriorityFeePerGas == nil {
			return errors.New("missing required field 'maxPriorityFeePerGas' for txdata")
		}
		itx.GasTipCap = (*big.Int)(dec.MaxPriorityFeePerGas)
		if dec.MaxFeePerGas == nil {
			return errors.New("missing required field 'maxFeePerGas' for txdata")
		}
		itx.GasFeeCap = (*big.Int)(dec.MaxFeePerGas)
		if dec.Value == nil {
			return errors.New("missing required field 'value' in transaction")
		}
		itx.Value = (*big.Int)(dec.Value)
		if dec.Input == nil {
			return errors.New("missing required field 'input' in transaction")
		}
		itx.Data = *dec.Input
		if dec.V == nil {
			return errors.New("missing required field 'v' in transaction")
		}
		if dec.AccessList != nil {
			itx.AccessList = *dec.AccessList
		}

		// signature R
		if dec.R == nil {
			return errors.New("missing required field 'r' in transaction")
		}
		itx.R = (*big.Int)(dec.R)
		// signature S
		if dec.S == nil {
			return errors.New("missing required field 's' in transaction")
		}
		itx.S = (*big.Int)(dec.S)
		// signature V
		itx.V, err = dec.yParityValue()
		if err != nil {
			return err
		}
		if itx.V.Sign() != 0 || itx.R.Sign() != 0 || itx.S.Sign() != 0 {
			if err := sanityCheckSignature(itx.V, itx.R, itx.S, false); err != nil {
				return err
			}
		}

	case BlobTxType:
		var itx BlobTx
		inner = &itx
		if dec.ChainID == nil {
			return errors.New("missing required field 'chainId' in transaction")
		}
		itx.ChainID = uint256.MustFromBig((*big.Int)(dec.ChainID))
		if dec.Nonce == nil {
			return errors.New("missing required field 'nonce' in transaction")
		}
		itx.Nonce = uint64(*dec.Nonce)
		if dec.To == nil {
			return errors.New("missing required field 'to' in transaction")
		}
		itx.To = *dec.To
		if dec.Gas == nil {
			return errors.New("missing required field 'gas' for txdata")
		}
		itx.Gas = uint64(*dec.Gas)
		if dec.MaxPriorityFeePerGas == nil {
			return errors.New("missing required field 'maxPriorityFeePerGas' for txdata")
		}
		itx.GasTipCap = uint256.MustFromBig((*big.Int)(dec.MaxPriorityFeePerGas))
		if dec.MaxFeePerGas == nil {
			return errors.New("missing required field 'maxFeePerGas' for txdata")
		}
		itx.GasFeeCap = uint256.MustFromBig((*big.Int)(dec.MaxFeePerGas))
		if dec.MaxFeePerBlobGas == nil {
			return errors.New("missing required field 'maxFeePerBlobGas' for txdata")
		}
		itx.BlobFeeCap = uint256.MustFromBig((*big.Int)(dec.MaxFeePerBlobGas))
		if dec.Value == nil {
			return errors.New("missing required field 'value' in transaction")
		}
		itx.Value = uint256.MustFromBig((*big.Int)(dec.Value))
		if dec.Input == nil {
			return errors.New("missing required field 'input' in transaction")
		}
		itx.Data = *dec.Input
		if dec.V == nil {
			return errors.New("missing required field 'v' in transaction")
		}
		if dec.AccessList != nil {
			itx.AccessList = *dec.AccessList
		}
		if dec.BlobVersionedHashes == nil {
			return errors.New("missing required field 'blobVersionedHashes' in transaction")
		}
		itx.BlobHashes = dec.BlobVersionedHashes

		// signature R
		var overflow bool
		if dec.R == nil {
			return errors.New("missing required field 'r' in transaction")
		}
		itx.R, overflow = uint256.FromBig((*big.Int)(dec.R))
		if overflow {
			return errors.New("'r' value overflows uint256")
		}
		// signature S
		if dec.S == nil {
			return errors.New("missing required field 's' in transaction")
		}
		itx.S, overflow = uint256.FromBig((*big.Int)(dec.S))
		if overflow {
			return errors.New("'s' value overflows uint256")
		}
		// signature V
		vbig, err := dec.yParityValue()
		if err != nil {
			return err
		}
		itx.V, overflow = uint256.FromBig(vbig)
		if overflow {
			return errors.New("'v' value overflows uint256")
		}
		if itx.V.Sign() != 0 || itx.R.Sign() != 0 || itx.S.Sign() != 0 {
			if err := sanityCheckSignature(vbig, itx.R.ToBig(), itx.S.ToBig(), false); err != nil {
				return err
			}
		}

	default:
		return ErrTxTypeNotSupported
	}

	// Now set the inner transaction.
	tx.setDecoded(inner, 0)

	// TODO: check hash here?
	return nil
}
