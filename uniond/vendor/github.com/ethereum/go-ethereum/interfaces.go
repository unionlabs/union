// Copyright 2016 The go-ethereum Authors
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

// Package ethereum defines interfaces for interacting with Ethereum.
package ethereum

import (
	"context"
	"errors"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
)

// NotFound is returned by API methods if the requested item does not exist.
var NotFound = errors.New("not found")

// TODO: move subscription to package event

// Subscription represents an event subscription where events are
// delivered on a data channel.
type Subscription interface {
	// Unsubscribe cancels the sending of events to the data channel
	// and closes the error channel.
	Unsubscribe()
	// Err returns the subscription error channel. The error channel receives
	// a value if there is an issue with the subscription (e.g. the network connection
	// delivering the events has been closed). Only one value will ever be sent.
	// The error channel is closed by Unsubscribe.
	Err() <-chan error
}

// ChainReader provides access to the blockchain. The methods in this interface access raw
// data from either the canonical chain (when requesting by block number) or any
// blockchain fork that was previously downloaded and processed by the node. The block
// number argument can be nil to select the latest canonical block. Reading block headers
// should be preferred over full blocks whenever possible.
//
// The returned error is NotFound if the requested item does not exist.
type ChainReader interface {
	BlockByHash(ctx context.Context, hash common.Hash) (*types.Block, error)
	BlockByNumber(ctx context.Context, number *big.Int) (*types.Block, error)
	HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error)
	HeaderByNumber(ctx context.Context, number *big.Int) (*types.Header, error)
	TransactionCount(ctx context.Context, blockHash common.Hash) (uint, error)
	TransactionInBlock(ctx context.Context, blockHash common.Hash, index uint) (*types.Transaction, error)

	// This method subscribes to notifications about changes of the head block of
	// the canonical chain.
	SubscribeNewHead(ctx context.Context, ch chan<- *types.Header) (Subscription, error)
}

// TransactionReader provides access to past transactions and their receipts.
// Implementations may impose arbitrary restrictions on the transactions and receipts that
// can be retrieved. Historic transactions may not be available.
//
// Avoid relying on this interface if possible. Contract logs (through the LogFilterer
// interface) are more reliable and usually safer in the presence of chain
// reorganisations.
//
// The returned error is NotFound if the requested item does not exist.
type TransactionReader interface {
	// TransactionByHash checks the pool of pending transactions in addition to the
	// blockchain. The isPending return value indicates whether the transaction has been
	// mined yet. Note that the transaction may not be part of the canonical chain even if
	// it's not pending.
	TransactionByHash(ctx context.Context, txHash common.Hash) (tx *types.Transaction, isPending bool, err error)
	// TransactionReceipt returns the receipt of a mined transaction. Note that the
	// transaction may not be included in the current canonical chain even if a receipt
	// exists.
	TransactionReceipt(ctx context.Context, txHash common.Hash) (*types.Receipt, error)
}

// ChainStateReader wraps access to the state trie of the canonical blockchain. Note that
// implementations of the interface may be unable to return state values for old blocks.
// In many cases, using CallContract can be preferable to reading raw contract storage.
type ChainStateReader interface {
	BalanceAt(ctx context.Context, account common.Address, blockNumber *big.Int) (*big.Int, error)
	StorageAt(ctx context.Context, account common.Address, key common.Hash, blockNumber *big.Int) ([]byte, error)
	CodeAt(ctx context.Context, account common.Address, blockNumber *big.Int) ([]byte, error)
	NonceAt(ctx context.Context, account common.Address, blockNumber *big.Int) (uint64, error)
}

// SyncProgress gives progress indications when the node is synchronising with
// the Ethereum network.
type SyncProgress struct {
	StartingBlock uint64 // Block number where sync began
	CurrentBlock  uint64 // Current block number where sync is at
	HighestBlock  uint64 // Highest alleged block number in the chain

	// "fast sync" fields. These used to be sent by geth, but are no longer used
	// since version v1.10.
	PulledStates uint64 // Number of state trie entries already downloaded
	KnownStates  uint64 // Total number of state trie entries known about

	// "snap sync" fields.
	SyncedAccounts      uint64 // Number of accounts downloaded
	SyncedAccountBytes  uint64 // Number of account trie bytes persisted to disk
	SyncedBytecodes     uint64 // Number of bytecodes downloaded
	SyncedBytecodeBytes uint64 // Number of bytecode bytes downloaded
	SyncedStorage       uint64 // Number of storage slots downloaded
	SyncedStorageBytes  uint64 // Number of storage trie bytes persisted to disk

	HealedTrienodes     uint64 // Number of state trie nodes downloaded
	HealedTrienodeBytes uint64 // Number of state trie bytes persisted to disk
	HealedBytecodes     uint64 // Number of bytecodes downloaded
	HealedBytecodeBytes uint64 // Number of bytecodes persisted to disk

	HealingTrienodes uint64 // Number of state trie nodes pending
	HealingBytecode  uint64 // Number of bytecodes pending
}

// ChainSyncReader wraps access to the node's current sync status. If there's no
// sync currently running, it returns nil.
type ChainSyncReader interface {
	SyncProgress(ctx context.Context) (*SyncProgress, error)
}

// CallMsg contains parameters for contract calls.
type CallMsg struct {
	From      common.Address  // the sender of the 'transaction'
	To        *common.Address // the destination contract (nil for contract creation)
	Gas       uint64          // if 0, the call executes with near-infinite gas
	GasPrice  *big.Int        // wei <-> gas exchange ratio
	GasFeeCap *big.Int        // EIP-1559 fee cap per gas.
	GasTipCap *big.Int        // EIP-1559 tip per gas.
	Value     *big.Int        // amount of wei sent along with the call
	Data      []byte          // input data, usually an ABI-encoded contract method invocation

	AccessList types.AccessList // EIP-2930 access list.
}

// A ContractCaller provides contract calls, essentially transactions that are executed by
// the EVM but not mined into the blockchain. ContractCall is a low-level method to
// execute such calls. For applications which are structured around specific contracts,
// the abigen tool provides a nicer, properly typed way to perform calls.
type ContractCaller interface {
	CallContract(ctx context.Context, call CallMsg, blockNumber *big.Int) ([]byte, error)
}

// FilterQuery contains options for contract log filtering.
type FilterQuery struct {
	BlockHash *common.Hash     // used by eth_getLogs, return logs only from block with this hash
	FromBlock *big.Int         // beginning of the queried range, nil means genesis block
	ToBlock   *big.Int         // end of the range, nil means latest block
	Addresses []common.Address // restricts matches to events created by specific contracts

	// The Topic list restricts matches to particular event topics. Each event has a list
	// of topics. Topics matches a prefix of that list. An empty element slice matches any
	// topic. Non-empty elements represent an alternative that matches any of the
	// contained topics.
	//
	// Examples:
	// {} or nil          matches any topic list
	// {{A}}              matches topic A in first position
	// {{}, {B}}          matches any topic in first position AND B in second position
	// {{A}, {B}}         matches topic A in first position AND B in second position
	// {{A, B}, {C, D}}   matches topic (A OR B) in first position AND (C OR D) in second position
	Topics [][]common.Hash
}

// LogFilterer provides access to contract log events using a one-off query or continuous
// event subscription.
//
// Logs received through a streaming query subscription may have Removed set to true,
// indicating that the log was reverted due to a chain reorganisation.
type LogFilterer interface {
	FilterLogs(ctx context.Context, q FilterQuery) ([]types.Log, error)
	SubscribeFilterLogs(ctx context.Context, q FilterQuery, ch chan<- types.Log) (Subscription, error)
}

// TransactionSender wraps transaction sending. The SendTransaction method injects a
// signed transaction into the pending transaction pool for execution. If the transaction
// was a contract creation, the TransactionReceipt method can be used to retrieve the
// contract address after the transaction has been mined.
//
// The transaction must be signed and have a valid nonce to be included. Consumers of the
// API can use package accounts to maintain local private keys and need can retrieve the
// next available nonce using PendingNonceAt.
type TransactionSender interface {
	SendTransaction(ctx context.Context, tx *types.Transaction) error
}

// GasPricer wraps the gas price oracle, which monitors the blockchain to determine the
// optimal gas price given current fee market conditions.
type GasPricer interface {
	SuggestGasPrice(ctx context.Context) (*big.Int, error)
}

// FeeHistory provides recent fee market data that consumers can use to determine
// a reasonable maxPriorityFeePerGas value.
type FeeHistory struct {
	OldestBlock  *big.Int     // block corresponding to first response value
	Reward       [][]*big.Int // list every txs priority fee per block
	BaseFee      []*big.Int   // list of each block's base fee
	GasUsedRatio []float64    // ratio of gas used out of the total available limit
}

// A PendingStateReader provides access to the pending state, which is the result of all
// known executable transactions which have not yet been included in the blockchain. It is
// commonly used to display the result of ’unconfirmed’ actions (e.g. wallet value
// transfers) initiated by the user. The PendingNonceAt operation is a good way to
// retrieve the next available transaction nonce for a specific account.
type PendingStateReader interface {
	PendingBalanceAt(ctx context.Context, account common.Address) (*big.Int, error)
	PendingStorageAt(ctx context.Context, account common.Address, key common.Hash) ([]byte, error)
	PendingCodeAt(ctx context.Context, account common.Address) ([]byte, error)
	PendingNonceAt(ctx context.Context, account common.Address) (uint64, error)
	PendingTransactionCount(ctx context.Context) (uint, error)
}

// PendingContractCaller can be used to perform calls against the pending state.
type PendingContractCaller interface {
	PendingCallContract(ctx context.Context, call CallMsg) ([]byte, error)
}

// GasEstimator wraps EstimateGas, which tries to estimate the gas needed to execute a
// specific transaction based on the pending state. There is no guarantee that this is the
// true gas limit requirement as other transactions may be added or removed by miners, but
// it should provide a basis for setting a reasonable default.
type GasEstimator interface {
	EstimateGas(ctx context.Context, call CallMsg) (uint64, error)
}

// A PendingStateEventer provides access to real time notifications about changes to the
// pending state.
type PendingStateEventer interface {
	SubscribePendingTransactions(ctx context.Context, ch chan<- *types.Transaction) (Subscription, error)
}
