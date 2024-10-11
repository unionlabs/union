// Copyright 2023 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package t8ntool

import (
	"bytes"
	"crypto/ecdsa"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"strings"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
)

// txWithKey is a helper-struct, to allow us to use the types.Transaction along with
// a `secretKey`-field, for input
type txWithKey struct {
	key       *ecdsa.PrivateKey
	tx        *types.Transaction
	protected bool
}

func (t *txWithKey) UnmarshalJSON(input []byte) error {
	// Read the metadata, if present
	type txMetadata struct {
		Key       *common.Hash `json:"secretKey"`
		Protected *bool        `json:"protected"`
	}
	var data txMetadata
	if err := json.Unmarshal(input, &data); err != nil {
		return err
	}
	if data.Key != nil {
		k := data.Key.Hex()[2:]
		if ecdsaKey, err := crypto.HexToECDSA(k); err != nil {
			return err
		} else {
			t.key = ecdsaKey
		}
	}
	if data.Protected != nil {
		t.protected = *data.Protected
	} else {
		t.protected = true
	}
	// Now, read the transaction itself
	var tx types.Transaction
	if err := json.Unmarshal(input, &tx); err != nil {
		return err
	}
	t.tx = &tx
	return nil
}

// signUnsignedTransactions converts the input txs to canonical transactions.
//
// The transactions can have two forms, either
//  1. unsigned or
//  2. signed
//
// For (1), r, s, v, need so be zero, and the `secretKey` needs to be set.
// If so, we sign it here and now, with the given `secretKey`
// If the condition above is not met, then it's considered a signed transaction.
//
// To manage this, we read the transactions twice, first trying to read the secretKeys,
// and secondly to read them with the standard tx json format
func signUnsignedTransactions(txs []*txWithKey, signer types.Signer) (types.Transactions, error) {
	var signedTxs []*types.Transaction
	for i, tx := range txs {
		var (
			v, r, s = tx.tx.RawSignatureValues()
			signed  *types.Transaction
			err     error
		)
		if tx.key == nil || v.BitLen()+r.BitLen()+s.BitLen() != 0 {
			// Already signed
			signedTxs = append(signedTxs, tx.tx)
			continue
		}
		// This transaction needs to be signed
		if tx.protected {
			signed, err = types.SignTx(tx.tx, signer, tx.key)
		} else {
			signed, err = types.SignTx(tx.tx, types.FrontierSigner{}, tx.key)
		}
		if err != nil {
			return nil, NewError(ErrorJson, fmt.Errorf("tx %d: failed to sign tx: %v", i, err))
		}
		signedTxs = append(signedTxs, signed)
	}
	return signedTxs, nil
}

func loadTransactions(txStr string, inputData *input, env stEnv, chainConfig *params.ChainConfig) (txIterator, error) {
	var txsWithKeys []*txWithKey
	if txStr != stdinSelector {
		data, err := os.ReadFile(txStr)
		if err != nil {
			return nil, NewError(ErrorIO, fmt.Errorf("failed reading txs file: %v", err))
		}
		if strings.HasSuffix(txStr, ".rlp") { // A file containing an rlp list
			var body hexutil.Bytes
			if err := json.Unmarshal(data, &body); err != nil {
				return nil, err
			}
			return newRlpTxIterator(body), nil
		}
		if err := json.Unmarshal(data, &txsWithKeys); err != nil {
			return nil, NewError(ErrorJson, fmt.Errorf("failed unmarshaling txs-file: %v", err))
		}
	} else {
		if len(inputData.TxRlp) > 0 {
			// Decode the body of already signed transactions
			return newRlpTxIterator(common.FromHex(inputData.TxRlp)), nil
		}
		// JSON encoded transactions
		txsWithKeys = inputData.Txs
	}
	// We may have to sign the transactions.
	signer := types.LatestSignerForChainID(chainConfig.ChainID)
	txs, err := signUnsignedTransactions(txsWithKeys, signer)
	return newSliceTxIterator(txs), err
}

type txIterator interface {
	// Next returns true until EOF
	Next() bool
	// Tx returns the next transaction, OR an error.
	Tx() (*types.Transaction, error)
}

type sliceTxIterator struct {
	idx int
	txs []*types.Transaction
}

func newSliceTxIterator(transactions types.Transactions) txIterator {
	return &sliceTxIterator{0, transactions}
}

func (ait *sliceTxIterator) Next() bool {
	return ait.idx < len(ait.txs)
}

func (ait *sliceTxIterator) Tx() (*types.Transaction, error) {
	if ait.idx < len(ait.txs) {
		ait.idx++
		return ait.txs[ait.idx-1], nil
	}
	return nil, io.EOF
}

type rlpTxIterator struct {
	in *rlp.Stream
}

func newRlpTxIterator(rlpData []byte) txIterator {
	in := rlp.NewStream(bytes.NewBuffer(rlpData), 1024*1024)
	in.List()
	return &rlpTxIterator{in}
}

func (it *rlpTxIterator) Next() bool {
	return it.in.MoreDataInList()
}

func (it *rlpTxIterator) Tx() (*types.Transaction, error) {
	var a types.Transaction
	if err := it.in.Decode(&a); err != nil {
		return nil, err
	}
	return &a, nil
}
