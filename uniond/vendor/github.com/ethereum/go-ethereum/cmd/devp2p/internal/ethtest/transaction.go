// Copyright 2020 The go-ethereum Authors
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

package ethtest

import (
	"errors"
	"fmt"
	"math/big"
	"strings"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/internal/utesting"
	"github.com/ethereum/go-ethereum/params"
)

// var faucetAddr = common.HexToAddress("0x71562b71999873DB5b286dF957af199Ec94617F7")
var faucetKey, _ = crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")

func (s *Suite) sendSuccessfulTxs(t *utesting.T) error {
	tests := []*types.Transaction{
		getNextTxFromChain(s),
		unknownTx(s),
	}
	for i, tx := range tests {
		if tx == nil {
			return errors.New("could not find tx to send")
		}
		t.Logf("Testing tx propagation %d: sending tx %v %v %v\n", i, tx.Hash().String(), tx.GasPrice(), tx.Gas())
		// get previous tx if exists for reference in case of old tx propagation
		var prevTx *types.Transaction
		if i != 0 {
			prevTx = tests[i-1]
		}
		// write tx to connection
		if err := sendSuccessfulTx(s, tx, prevTx); err != nil {
			return fmt.Errorf("send successful tx test failed: %v", err)
		}
	}
	return nil
}

func sendSuccessfulTx(s *Suite, tx *types.Transaction, prevTx *types.Transaction) error {
	sendConn, recvConn, err := s.createSendAndRecvConns()
	if err != nil {
		return err
	}
	defer sendConn.Close()
	defer recvConn.Close()
	if err = sendConn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}
	// Send the transaction
	if err = sendConn.Write(&Transactions{tx}); err != nil {
		return fmt.Errorf("failed to write to connection: %v", err)
	}
	// peer receiving connection to node
	if err = recvConn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}

	// update last nonce seen
	nonce = tx.Nonce()

	// Wait for the transaction announcement
	for {
		switch msg := recvConn.readAndServe(s.chain, timeout).(type) {
		case *Transactions:
			recTxs := *msg
			// if you receive an old tx propagation, read from connection again
			if len(recTxs) == 1 && prevTx != nil {
				if recTxs[0] == prevTx {
					continue
				}
			}
			for _, gotTx := range recTxs {
				if gotTx.Hash() == tx.Hash() {
					// Ok
					return nil
				}
			}
			return fmt.Errorf("missing transaction: got %v missing %v", recTxs, tx.Hash())
		case *NewPooledTransactionHashes66:
			txHashes := *msg
			// if you receive an old tx propagation, read from connection again
			if len(txHashes) == 1 && prevTx != nil {
				if txHashes[0] == prevTx.Hash() {
					continue
				}
			}
			for _, gotHash := range txHashes {
				if gotHash == tx.Hash() {
					// Ok
					return nil
				}
			}
			return fmt.Errorf("missing transaction announcement: got %v missing %v", txHashes, tx.Hash())
		case *NewPooledTransactionHashes:
			txHashes := msg.Hashes
			if len(txHashes) != len(msg.Sizes) {
				return fmt.Errorf("invalid msg size lengths: hashes: %v sizes: %v", len(txHashes), len(msg.Sizes))
			}
			if len(txHashes) != len(msg.Types) {
				return fmt.Errorf("invalid msg type lengths: hashes: %v types: %v", len(txHashes), len(msg.Types))
			}
			// if you receive an old tx propagation, read from connection again
			if len(txHashes) == 1 && prevTx != nil {
				if txHashes[0] == prevTx.Hash() {
					continue
				}
			}
			for index, gotHash := range txHashes {
				if gotHash == tx.Hash() {
					if msg.Sizes[index] != uint32(tx.Size()) {
						return fmt.Errorf("invalid tx size: got %v want %v", msg.Sizes[index], tx.Size())
					}
					if msg.Types[index] != tx.Type() {
						return fmt.Errorf("invalid tx type: got %v want %v", msg.Types[index], tx.Type())
					}
					// Ok
					return nil
				}
			}
			return fmt.Errorf("missing transaction announcement: got %v missing %v", txHashes, tx.Hash())

		default:
			return fmt.Errorf("unexpected message in sendSuccessfulTx: %s", pretty.Sdump(msg))
		}
	}
}

func (s *Suite) sendMaliciousTxs(t *utesting.T) error {
	badTxs := []*types.Transaction{
		getOldTxFromChain(s),
		invalidNonceTx(s),
		hugeAmount(s),
		hugeGasPrice(s),
		hugeData(s),
	}

	// setup receiving connection before sending malicious txs
	recvConn, err := s.dial()
	if err != nil {
		return fmt.Errorf("dial failed: %v", err)
	}
	defer recvConn.Close()
	if err = recvConn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}

	for i, tx := range badTxs {
		t.Logf("Testing malicious tx propagation: %v\n", i)
		if err = sendMaliciousTx(s, tx); err != nil {
			return fmt.Errorf("malicious tx test failed:\ntx: %v\nerror: %v", tx, err)
		}
	}
	// check to make sure bad txs aren't propagated
	return checkMaliciousTxPropagation(s, badTxs, recvConn)
}

func sendMaliciousTx(s *Suite, tx *types.Transaction) error {
	conn, err := s.dial()
	if err != nil {
		return fmt.Errorf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}

	// write malicious tx
	if err = conn.Write(&Transactions{tx}); err != nil {
		return fmt.Errorf("failed to write to connection: %v", err)
	}
	return nil
}

var nonce = uint64(99)

// sendMultipleSuccessfulTxs sends the given transactions to the node and
// expects the node to accept and propagate them.
func sendMultipleSuccessfulTxs(t *utesting.T, s *Suite, txs []*types.Transaction) error {
	txMsg := Transactions(txs)
	t.Logf("sending %d txs\n", len(txs))

	sendConn, recvConn, err := s.createSendAndRecvConns()
	if err != nil {
		return err
	}
	defer sendConn.Close()
	defer recvConn.Close()
	if err = sendConn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}
	if err = recvConn.peer(s.chain, nil); err != nil {
		return fmt.Errorf("peering failed: %v", err)
	}

	// Send the transactions
	if err = sendConn.Write(&txMsg); err != nil {
		return fmt.Errorf("failed to write message to connection: %v", err)
	}

	// update nonce
	nonce = txs[len(txs)-1].Nonce()

	// Wait for the transaction announcement(s) and make sure all sent txs are being propagated.
	// all txs should be announced within a couple announcements.
	recvHashes := make([]common.Hash, 0)

	for i := 0; i < 20; i++ {
		switch msg := recvConn.readAndServe(s.chain, timeout).(type) {
		case *Transactions:
			for _, tx := range *msg {
				recvHashes = append(recvHashes, tx.Hash())
			}
		case *NewPooledTransactionHashes66:
			recvHashes = append(recvHashes, *msg...)
		case *NewPooledTransactionHashes:
			recvHashes = append(recvHashes, msg.Hashes...)
		default:
			if !strings.Contains(pretty.Sdump(msg), "i/o timeout") {
				return fmt.Errorf("unexpected message while waiting to receive txs: %s", pretty.Sdump(msg))
			}
		}
		// break once all 2000 txs have been received
		if len(recvHashes) == 2000 {
			break
		}
		if len(recvHashes) > 0 {
			_, missingTxs := compareReceivedTxs(recvHashes, txs)
			if len(missingTxs) > 0 {
				continue
			} else {
				t.Logf("successfully received all %d txs", len(txs))
				return nil
			}
		}
	}
	_, missingTxs := compareReceivedTxs(recvHashes, txs)
	if len(missingTxs) > 0 {
		for _, missing := range missingTxs {
			t.Logf("missing tx: %v", missing.Hash())
		}
		return fmt.Errorf("missing %d txs", len(missingTxs))
	}
	return nil
}

// checkMaliciousTxPropagation checks whether the given malicious transactions were
// propagated by the node.
func checkMaliciousTxPropagation(s *Suite, txs []*types.Transaction, conn *Conn) error {
	switch msg := conn.readAndServe(s.chain, time.Second*8).(type) {
	case *Transactions:
		// check to see if any of the failing txs were in the announcement
		recvTxs := make([]common.Hash, len(*msg))
		for i, recvTx := range *msg {
			recvTxs[i] = recvTx.Hash()
		}
		badTxs, _ := compareReceivedTxs(recvTxs, txs)
		if len(badTxs) > 0 {
			return fmt.Errorf("received %d bad txs: \n%v", len(badTxs), badTxs)
		}
	case *NewPooledTransactionHashes66:
		badTxs, _ := compareReceivedTxs(*msg, txs)
		if len(badTxs) > 0 {
			return fmt.Errorf("received %d bad txs: \n%v", len(badTxs), badTxs)
		}
	case *NewPooledTransactionHashes:
		badTxs, _ := compareReceivedTxs(msg.Hashes, txs)
		if len(badTxs) > 0 {
			return fmt.Errorf("received %d bad txs: \n%v", len(badTxs), badTxs)
		}
	case *Error:
		// Transaction should not be announced -> wait for timeout
		return nil
	default:
		return fmt.Errorf("unexpected message in sendFailingTx: %s", pretty.Sdump(msg))
	}
	return nil
}

// compareReceivedTxs compares the received set of txs against the given set of txs,
// returning both the set received txs that were present within the given txs, and
// the set of txs that were missing from the set of received txs
func compareReceivedTxs(recvTxs []common.Hash, txs []*types.Transaction) (present []*types.Transaction, missing []*types.Transaction) {
	// create a map of the hashes received from node
	recvHashes := make(map[common.Hash]common.Hash)
	for _, hash := range recvTxs {
		recvHashes[hash] = hash
	}

	// collect present txs and missing txs separately
	present = make([]*types.Transaction, 0)
	missing = make([]*types.Transaction, 0)
	for _, tx := range txs {
		if _, exists := recvHashes[tx.Hash()]; exists {
			present = append(present, tx)
		} else {
			missing = append(missing, tx)
		}
	}
	return present, missing
}

func unknownTx(s *Suite) *types.Transaction {
	tx := getNextTxFromChain(s)
	if tx == nil {
		return nil
	}
	var to common.Address
	if tx.To() != nil {
		to = *tx.To()
	}
	txNew := types.NewTransaction(tx.Nonce()+1, to, tx.Value(), tx.Gas(), tx.GasPrice(), tx.Data())
	return signWithFaucet(s.chain.chainConfig, txNew)
}

func getNextTxFromChain(s *Suite) *types.Transaction {
	// Get a new transaction
	for _, blocks := range s.fullChain.blocks[s.chain.Len():] {
		txs := blocks.Transactions()
		if txs.Len() != 0 {
			return txs[0]
		}
	}
	return nil
}

func generateTxs(s *Suite, numTxs int) (map[common.Hash]common.Hash, []*types.Transaction, error) {
	txHashMap := make(map[common.Hash]common.Hash, numTxs)
	txs := make([]*types.Transaction, numTxs)

	nextTx := getNextTxFromChain(s)
	if nextTx == nil {
		return nil, nil, errors.New("failed to get the next transaction")
	}
	gas := nextTx.Gas()

	nonce = nonce + 1
	// generate txs
	for i := 0; i < numTxs; i++ {
		tx := generateTx(s.chain.chainConfig, nonce, gas)
		if tx == nil {
			return nil, nil, errors.New("failed to get the next transaction")
		}
		txHashMap[tx.Hash()] = tx.Hash()
		txs[i] = tx
		nonce = nonce + 1
	}
	return txHashMap, txs, nil
}

func generateTx(chainConfig *params.ChainConfig, nonce uint64, gas uint64) *types.Transaction {
	var to common.Address
	tx := types.NewTransaction(nonce, to, big.NewInt(1), gas, big.NewInt(1), []byte{})
	return signWithFaucet(chainConfig, tx)
}

func getOldTxFromChain(s *Suite) *types.Transaction {
	for _, blocks := range s.fullChain.blocks[:s.chain.Len()-1] {
		txs := blocks.Transactions()
		if txs.Len() != 0 {
			return txs[0]
		}
	}
	return nil
}

func invalidNonceTx(s *Suite) *types.Transaction {
	tx := getNextTxFromChain(s)
	if tx == nil {
		return nil
	}
	var to common.Address
	if tx.To() != nil {
		to = *tx.To()
	}
	txNew := types.NewTransaction(tx.Nonce()-2, to, tx.Value(), tx.Gas(), tx.GasPrice(), tx.Data())
	return signWithFaucet(s.chain.chainConfig, txNew)
}

func hugeAmount(s *Suite) *types.Transaction {
	tx := getNextTxFromChain(s)
	if tx == nil {
		return nil
	}
	amount := largeNumber(2)
	var to common.Address
	if tx.To() != nil {
		to = *tx.To()
	}
	txNew := types.NewTransaction(tx.Nonce(), to, amount, tx.Gas(), tx.GasPrice(), tx.Data())
	return signWithFaucet(s.chain.chainConfig, txNew)
}

func hugeGasPrice(s *Suite) *types.Transaction {
	tx := getNextTxFromChain(s)
	if tx == nil {
		return nil
	}
	gasPrice := largeNumber(2)
	var to common.Address
	if tx.To() != nil {
		to = *tx.To()
	}
	txNew := types.NewTransaction(tx.Nonce(), to, tx.Value(), tx.Gas(), gasPrice, tx.Data())
	return signWithFaucet(s.chain.chainConfig, txNew)
}

func hugeData(s *Suite) *types.Transaction {
	tx := getNextTxFromChain(s)
	if tx == nil {
		return nil
	}
	var to common.Address
	if tx.To() != nil {
		to = *tx.To()
	}
	txNew := types.NewTransaction(tx.Nonce(), to, tx.Value(), tx.Gas(), tx.GasPrice(), largeBuffer(2))
	return signWithFaucet(s.chain.chainConfig, txNew)
}

func signWithFaucet(chainConfig *params.ChainConfig, tx *types.Transaction) *types.Transaction {
	signer := types.LatestSigner(chainConfig)
	signedTx, err := types.SignTx(tx, signer, faucetKey)
	if err != nil {
		return nil
	}
	return signedTx
}
