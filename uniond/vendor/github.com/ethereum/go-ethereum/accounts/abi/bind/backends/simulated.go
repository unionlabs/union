// Copyright 2015 The go-ethereum Authors
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

package backends

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/accounts/abi"
	"github.com/ethereum/go-ethereum/accounts/abi/bind"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/bloombits"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/eth/filters"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rpc"
)

// This nil assignment ensures at compile time that SimulatedBackend implements bind.ContractBackend.
var _ bind.ContractBackend = (*SimulatedBackend)(nil)

var (
	errBlockNumberUnsupported  = errors.New("simulatedBackend cannot access blocks other than the latest block")
	errBlockHashUnsupported    = errors.New("simulatedBackend cannot access blocks by hash other than the latest block")
	errBlockDoesNotExist       = errors.New("block does not exist in blockchain")
	errTransactionDoesNotExist = errors.New("transaction does not exist")
)

// SimulatedBackend implements bind.ContractBackend, simulating a blockchain in
// the background. Its main purpose is to allow for easy testing of contract bindings.
// Simulated backend implements the following interfaces:
// ChainReader, ChainStateReader, ContractBackend, ContractCaller, ContractFilterer, ContractTransactor,
// DeployBackend, GasEstimator, GasPricer, LogFilterer, PendingContractCaller, TransactionReader, and TransactionSender
type SimulatedBackend struct {
	database   ethdb.Database   // In memory database to store our testing data
	blockchain *core.BlockChain // Ethereum blockchain to handle the consensus

	mu              sync.Mutex
	pendingBlock    *types.Block   // Currently pending block that will be imported on request
	pendingState    *state.StateDB // Currently pending state that will be the active on request
	pendingReceipts types.Receipts // Currently receipts for the pending block

	events       *filters.EventSystem  // for filtering log events live
	filterSystem *filters.FilterSystem // for filtering database logs

	config *params.ChainConfig
}

// NewSimulatedBackendWithDatabase creates a new binding backend based on the given database
// and uses a simulated blockchain for testing purposes.
// A simulated backend always uses chainID 1337.
func NewSimulatedBackendWithDatabase(database ethdb.Database, alloc core.GenesisAlloc, gasLimit uint64) *SimulatedBackend {
	genesis := core.Genesis{
		Config:   params.AllEthashProtocolChanges,
		GasLimit: gasLimit,
		Alloc:    alloc,
	}
	blockchain, _ := core.NewBlockChain(database, nil, &genesis, nil, ethash.NewFaker(), vm.Config{}, nil, nil)

	backend := &SimulatedBackend{
		database:   database,
		blockchain: blockchain,
		config:     genesis.Config,
	}

	filterBackend := &filterBackend{database, blockchain, backend}
	backend.filterSystem = filters.NewFilterSystem(filterBackend, filters.Config{})
	backend.events = filters.NewEventSystem(backend.filterSystem, false)

	header := backend.blockchain.CurrentBlock()
	block := backend.blockchain.GetBlock(header.Hash(), header.Number.Uint64())

	backend.rollback(block)
	return backend
}

// NewSimulatedBackend creates a new binding backend using a simulated blockchain
// for testing purposes.
// A simulated backend always uses chainID 1337.
func NewSimulatedBackend(alloc core.GenesisAlloc, gasLimit uint64) *SimulatedBackend {
	return NewSimulatedBackendWithDatabase(rawdb.NewMemoryDatabase(), alloc, gasLimit)
}

// Close terminates the underlying blockchain's update loop.
func (b *SimulatedBackend) Close() error {
	b.blockchain.Stop()
	return nil
}

// Commit imports all the pending transactions as a single block and starts a
// fresh new state.
func (b *SimulatedBackend) Commit() common.Hash {
	b.mu.Lock()
	defer b.mu.Unlock()

	if _, err := b.blockchain.InsertChain([]*types.Block{b.pendingBlock}); err != nil {
		panic(err) // This cannot happen unless the simulator is wrong, fail in that case
	}
	blockHash := b.pendingBlock.Hash()

	// Using the last inserted block here makes it possible to build on a side
	// chain after a fork.
	b.rollback(b.pendingBlock)

	return blockHash
}

// Rollback aborts all pending transactions, reverting to the last committed state.
func (b *SimulatedBackend) Rollback() {
	b.mu.Lock()
	defer b.mu.Unlock()

	header := b.blockchain.CurrentBlock()
	block := b.blockchain.GetBlock(header.Hash(), header.Number.Uint64())

	b.rollback(block)
}

func (b *SimulatedBackend) rollback(parent *types.Block) {
	blocks, _ := core.GenerateChain(b.config, parent, ethash.NewFaker(), b.database, 1, func(int, *core.BlockGen) {})

	b.pendingBlock = blocks[0]
	b.pendingState, _ = state.New(b.pendingBlock.Root(), b.blockchain.StateCache(), nil)
}

// Fork creates a side-chain that can be used to simulate reorgs.
//
// This function should be called with the ancestor block where the new side
// chain should be started. Transactions (old and new) can then be applied on
// top and Commit-ed.
//
// Note, the side-chain will only become canonical (and trigger the events) when
// it becomes longer. Until then CallContract will still operate on the current
// canonical chain.
//
// There is a % chance that the side chain becomes canonical at the same length
// to simulate live network behavior.
func (b *SimulatedBackend) Fork(ctx context.Context, parent common.Hash) error {
	b.mu.Lock()
	defer b.mu.Unlock()

	if len(b.pendingBlock.Transactions()) != 0 {
		return errors.New("pending block dirty")
	}
	block, err := b.blockByHash(ctx, parent)
	if err != nil {
		return err
	}
	b.rollback(block)
	return nil
}

// stateByBlockNumber retrieves a state by a given blocknumber.
func (b *SimulatedBackend) stateByBlockNumber(ctx context.Context, blockNumber *big.Int) (*state.StateDB, error) {
	if blockNumber == nil || blockNumber.Cmp(b.blockchain.CurrentBlock().Number) == 0 {
		return b.blockchain.State()
	}
	block, err := b.blockByNumber(ctx, blockNumber)
	if err != nil {
		return nil, err
	}
	return b.blockchain.StateAt(block.Root())
}

// CodeAt returns the code associated with a certain account in the blockchain.
func (b *SimulatedBackend) CodeAt(ctx context.Context, contract common.Address, blockNumber *big.Int) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	stateDB, err := b.stateByBlockNumber(ctx, blockNumber)
	if err != nil {
		return nil, err
	}
	return stateDB.GetCode(contract), nil
}

// CodeAtHash returns the code associated with a certain account in the blockchain.
func (b *SimulatedBackend) CodeAtHash(ctx context.Context, contract common.Address, blockHash common.Hash) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	header, err := b.headerByHash(blockHash)
	if err != nil {
		return nil, err
	}

	stateDB, err := b.blockchain.StateAt(header.Root)
	if err != nil {
		return nil, err
	}

	return stateDB.GetCode(contract), nil
}

// BalanceAt returns the wei balance of a certain account in the blockchain.
func (b *SimulatedBackend) BalanceAt(ctx context.Context, contract common.Address, blockNumber *big.Int) (*big.Int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	stateDB, err := b.stateByBlockNumber(ctx, blockNumber)
	if err != nil {
		return nil, err
	}
	return stateDB.GetBalance(contract), nil
}

// NonceAt returns the nonce of a certain account in the blockchain.
func (b *SimulatedBackend) NonceAt(ctx context.Context, contract common.Address, blockNumber *big.Int) (uint64, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	stateDB, err := b.stateByBlockNumber(ctx, blockNumber)
	if err != nil {
		return 0, err
	}
	return stateDB.GetNonce(contract), nil
}

// StorageAt returns the value of key in the storage of an account in the blockchain.
func (b *SimulatedBackend) StorageAt(ctx context.Context, contract common.Address, key common.Hash, blockNumber *big.Int) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	stateDB, err := b.stateByBlockNumber(ctx, blockNumber)
	if err != nil {
		return nil, err
	}
	val := stateDB.GetState(contract, key)
	return val[:], nil
}

// TransactionReceipt returns the receipt of a transaction.
func (b *SimulatedBackend) TransactionReceipt(ctx context.Context, txHash common.Hash) (*types.Receipt, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	receipt, _, _, _ := rawdb.ReadReceipt(b.database, txHash, b.config)
	if receipt == nil {
		return nil, ethereum.NotFound
	}
	return receipt, nil
}

// TransactionByHash checks the pool of pending transactions in addition to the
// blockchain. The isPending return value indicates whether the transaction has been
// mined yet. Note that the transaction may not be part of the canonical chain even if
// it's not pending.
func (b *SimulatedBackend) TransactionByHash(ctx context.Context, txHash common.Hash) (*types.Transaction, bool, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	tx := b.pendingBlock.Transaction(txHash)
	if tx != nil {
		return tx, true, nil
	}
	tx, _, _, _ = rawdb.ReadTransaction(b.database, txHash)
	if tx != nil {
		return tx, false, nil
	}
	return nil, false, ethereum.NotFound
}

// BlockByHash retrieves a block based on the block hash.
func (b *SimulatedBackend) BlockByHash(ctx context.Context, hash common.Hash) (*types.Block, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	return b.blockByHash(ctx, hash)
}

// blockByHash retrieves a block based on the block hash without Locking.
func (b *SimulatedBackend) blockByHash(ctx context.Context, hash common.Hash) (*types.Block, error) {
	if hash == b.pendingBlock.Hash() {
		return b.pendingBlock, nil
	}

	block := b.blockchain.GetBlockByHash(hash)
	if block != nil {
		return block, nil
	}

	return nil, errBlockDoesNotExist
}

// BlockByNumber retrieves a block from the database by number, caching it
// (associated with its hash) if found.
func (b *SimulatedBackend) BlockByNumber(ctx context.Context, number *big.Int) (*types.Block, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	return b.blockByNumber(ctx, number)
}

// blockByNumber retrieves a block from the database by number, caching it
// (associated with its hash) if found without Lock.
func (b *SimulatedBackend) blockByNumber(ctx context.Context, number *big.Int) (*types.Block, error) {
	if number == nil || number.Cmp(b.pendingBlock.Number()) == 0 {
		return b.blockByHash(ctx, b.blockchain.CurrentBlock().Hash())
	}

	block := b.blockchain.GetBlockByNumber(uint64(number.Int64()))
	if block == nil {
		return nil, errBlockDoesNotExist
	}

	return block, nil
}

// HeaderByHash returns a block header from the current canonical chain.
func (b *SimulatedBackend) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.headerByHash(hash)
}

// headerByHash retrieves a header from the database by hash without Lock.
func (b *SimulatedBackend) headerByHash(hash common.Hash) (*types.Header, error) {
	if hash == b.pendingBlock.Hash() {
		return b.pendingBlock.Header(), nil
	}

	header := b.blockchain.GetHeaderByHash(hash)
	if header == nil {
		return nil, errBlockDoesNotExist
	}

	return header, nil
}

// HeaderByNumber returns a block header from the current canonical chain. If number is
// nil, the latest known header is returned.
func (b *SimulatedBackend) HeaderByNumber(ctx context.Context, block *big.Int) (*types.Header, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if block == nil || block.Cmp(b.pendingBlock.Number()) == 0 {
		return b.blockchain.CurrentHeader(), nil
	}

	return b.blockchain.GetHeaderByNumber(uint64(block.Int64())), nil
}

// TransactionCount returns the number of transactions in a given block.
func (b *SimulatedBackend) TransactionCount(ctx context.Context, blockHash common.Hash) (uint, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if blockHash == b.pendingBlock.Hash() {
		return uint(b.pendingBlock.Transactions().Len()), nil
	}

	block := b.blockchain.GetBlockByHash(blockHash)
	if block == nil {
		return uint(0), errBlockDoesNotExist
	}

	return uint(block.Transactions().Len()), nil
}

// TransactionInBlock returns the transaction for a specific block at a specific index.
func (b *SimulatedBackend) TransactionInBlock(ctx context.Context, blockHash common.Hash, index uint) (*types.Transaction, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if blockHash == b.pendingBlock.Hash() {
		transactions := b.pendingBlock.Transactions()
		if uint(len(transactions)) < index+1 {
			return nil, errTransactionDoesNotExist
		}

		return transactions[index], nil
	}

	block := b.blockchain.GetBlockByHash(blockHash)
	if block == nil {
		return nil, errBlockDoesNotExist
	}

	transactions := block.Transactions()
	if uint(len(transactions)) < index+1 {
		return nil, errTransactionDoesNotExist
	}

	return transactions[index], nil
}

// PendingCodeAt returns the code associated with an account in the pending state.
func (b *SimulatedBackend) PendingCodeAt(ctx context.Context, contract common.Address) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	return b.pendingState.GetCode(contract), nil
}

func newRevertError(result *core.ExecutionResult) *revertError {
	reason, errUnpack := abi.UnpackRevert(result.Revert())
	err := errors.New("execution reverted")
	if errUnpack == nil {
		err = fmt.Errorf("execution reverted: %v", reason)
	}
	return &revertError{
		error:  err,
		reason: hexutil.Encode(result.Revert()),
	}
}

// revertError is an API error that encompasses an EVM revert with JSON error
// code and a binary data blob.
type revertError struct {
	error
	reason string // revert reason hex encoded
}

// ErrorCode returns the JSON error code for a revert.
// See: https://github.com/ethereum/wiki/wiki/JSON-RPC-Error-Codes-Improvement-Proposal
func (e *revertError) ErrorCode() int {
	return 3
}

// ErrorData returns the hex encoded revert reason.
func (e *revertError) ErrorData() interface{} {
	return e.reason
}

// CallContract executes a contract call.
func (b *SimulatedBackend) CallContract(ctx context.Context, call ethereum.CallMsg, blockNumber *big.Int) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if blockNumber != nil && blockNumber.Cmp(b.blockchain.CurrentBlock().Number) != 0 {
		return nil, errBlockNumberUnsupported
	}
	return b.callContractAtHead(ctx, call)
}

// CallContractAtHash executes a contract call on a specific block hash.
func (b *SimulatedBackend) CallContractAtHash(ctx context.Context, call ethereum.CallMsg, blockHash common.Hash) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if blockHash != b.blockchain.CurrentBlock().Hash() {
		return nil, errBlockHashUnsupported
	}
	return b.callContractAtHead(ctx, call)
}

// callContractAtHead executes a contract call against the latest block state.
func (b *SimulatedBackend) callContractAtHead(ctx context.Context, call ethereum.CallMsg) ([]byte, error) {
	stateDB, err := b.blockchain.State()
	if err != nil {
		return nil, err
	}
	res, err := b.callContract(ctx, call, b.blockchain.CurrentBlock(), stateDB)
	if err != nil {
		return nil, err
	}
	// If the result contains a revert reason, try to unpack and return it.
	if len(res.Revert()) > 0 {
		return nil, newRevertError(res)
	}
	return res.Return(), res.Err
}

// PendingCallContract executes a contract call on the pending state.
func (b *SimulatedBackend) PendingCallContract(ctx context.Context, call ethereum.CallMsg) ([]byte, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	defer b.pendingState.RevertToSnapshot(b.pendingState.Snapshot())

	res, err := b.callContract(ctx, call, b.pendingBlock.Header(), b.pendingState)
	if err != nil {
		return nil, err
	}
	// If the result contains a revert reason, try to unpack and return it.
	if len(res.Revert()) > 0 {
		return nil, newRevertError(res)
	}
	return res.Return(), res.Err
}

// PendingNonceAt implements PendingStateReader.PendingNonceAt, retrieving
// the nonce currently pending for the account.
func (b *SimulatedBackend) PendingNonceAt(ctx context.Context, account common.Address) (uint64, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	return b.pendingState.GetOrNewStateObject(account).Nonce(), nil
}

// SuggestGasPrice implements ContractTransactor.SuggestGasPrice. Since the simulated
// chain doesn't have miners, we just return a gas price of 1 for any call.
func (b *SimulatedBackend) SuggestGasPrice(ctx context.Context) (*big.Int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	if b.pendingBlock.Header().BaseFee != nil {
		return b.pendingBlock.Header().BaseFee, nil
	}
	return big.NewInt(1), nil
}

// SuggestGasTipCap implements ContractTransactor.SuggestGasTipCap. Since the simulated
// chain doesn't have miners, we just return a gas tip of 1 for any call.
func (b *SimulatedBackend) SuggestGasTipCap(ctx context.Context) (*big.Int, error) {
	return big.NewInt(1), nil
}

// EstimateGas executes the requested code against the currently pending block/state and
// returns the used amount of gas.
func (b *SimulatedBackend) EstimateGas(ctx context.Context, call ethereum.CallMsg) (uint64, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	// Determine the lowest and highest possible gas limits to binary search in between
	var (
		lo  uint64 = params.TxGas - 1
		hi  uint64
		cap uint64
	)
	if call.Gas >= params.TxGas {
		hi = call.Gas
	} else {
		hi = b.pendingBlock.GasLimit()
	}
	// Normalize the max fee per gas the call is willing to spend.
	var feeCap *big.Int
	if call.GasPrice != nil && (call.GasFeeCap != nil || call.GasTipCap != nil) {
		return 0, errors.New("both gasPrice and (maxFeePerGas or maxPriorityFeePerGas) specified")
	} else if call.GasPrice != nil {
		feeCap = call.GasPrice
	} else if call.GasFeeCap != nil {
		feeCap = call.GasFeeCap
	} else {
		feeCap = common.Big0
	}
	// Recap the highest gas allowance with account's balance.
	if feeCap.BitLen() != 0 {
		balance := b.pendingState.GetBalance(call.From) // from can't be nil
		available := new(big.Int).Set(balance)
		if call.Value != nil {
			if call.Value.Cmp(available) >= 0 {
				return 0, core.ErrInsufficientFundsForTransfer
			}
			available.Sub(available, call.Value)
		}
		allowance := new(big.Int).Div(available, feeCap)
		if allowance.IsUint64() && hi > allowance.Uint64() {
			transfer := call.Value
			if transfer == nil {
				transfer = new(big.Int)
			}
			log.Warn("Gas estimation capped by limited funds", "original", hi, "balance", balance,
				"sent", transfer, "feecap", feeCap, "fundable", allowance)
			hi = allowance.Uint64()
		}
	}
	cap = hi

	// Create a helper to check if a gas allowance results in an executable transaction
	executable := func(gas uint64) (bool, *core.ExecutionResult, error) {
		call.Gas = gas

		snapshot := b.pendingState.Snapshot()
		res, err := b.callContract(ctx, call, b.pendingBlock.Header(), b.pendingState)
		b.pendingState.RevertToSnapshot(snapshot)

		if err != nil {
			if errors.Is(err, core.ErrIntrinsicGas) {
				return true, nil, nil // Special case, raise gas limit
			}
			return true, nil, err // Bail out
		}
		return res.Failed(), res, nil
	}
	// Execute the binary search and hone in on an executable gas limit
	for lo+1 < hi {
		mid := (hi + lo) / 2
		failed, _, err := executable(mid)

		// If the error is not nil(consensus error), it means the provided message
		// call or transaction will never be accepted no matter how much gas it is
		// assigned. Return the error directly, don't struggle any more
		if err != nil {
			return 0, err
		}
		if failed {
			lo = mid
		} else {
			hi = mid
		}
	}
	// Reject the transaction as invalid if it still fails at the highest allowance
	if hi == cap {
		failed, result, err := executable(hi)
		if err != nil {
			return 0, err
		}
		if failed {
			if result != nil && !errors.Is(result.Err, vm.ErrOutOfGas) {
				if len(result.Revert()) > 0 {
					return 0, newRevertError(result)
				}
				return 0, result.Err
			}
			// Otherwise, the specified gas cap is too low
			return 0, fmt.Errorf("gas required exceeds allowance (%d)", cap)
		}
	}
	return hi, nil
}

// callContract implements common code between normal and pending contract calls.
// state is modified during execution, make sure to copy it if necessary.
func (b *SimulatedBackend) callContract(ctx context.Context, call ethereum.CallMsg, header *types.Header, stateDB *state.StateDB) (*core.ExecutionResult, error) {
	// Gas prices post 1559 need to be initialized
	if call.GasPrice != nil && (call.GasFeeCap != nil || call.GasTipCap != nil) {
		return nil, errors.New("both gasPrice and (maxFeePerGas or maxPriorityFeePerGas) specified")
	}
	if !b.blockchain.Config().IsLondon(header.Number) {
		// If there's no basefee, then it must be a non-1559 execution
		if call.GasPrice == nil {
			call.GasPrice = new(big.Int)
		}
		call.GasFeeCap, call.GasTipCap = call.GasPrice, call.GasPrice
	} else {
		// A basefee is provided, necessitating 1559-type execution
		if call.GasPrice != nil {
			// User specified the legacy gas field, convert to 1559 gas typing
			call.GasFeeCap, call.GasTipCap = call.GasPrice, call.GasPrice
		} else {
			// User specified 1559 gas fields (or none), use those
			if call.GasFeeCap == nil {
				call.GasFeeCap = new(big.Int)
			}
			if call.GasTipCap == nil {
				call.GasTipCap = new(big.Int)
			}
			// Backfill the legacy gasPrice for EVM execution, unless we're all zeroes
			call.GasPrice = new(big.Int)
			if call.GasFeeCap.BitLen() > 0 || call.GasTipCap.BitLen() > 0 {
				call.GasPrice = math.BigMin(new(big.Int).Add(call.GasTipCap, header.BaseFee), call.GasFeeCap)
			}
		}
	}
	// Ensure message is initialized properly.
	if call.Gas == 0 {
		call.Gas = 10 * header.GasLimit
	}
	if call.Value == nil {
		call.Value = new(big.Int)
	}

	// Set infinite balance to the fake caller account.
	from := stateDB.GetOrNewStateObject(call.From)
	from.SetBalance(math.MaxBig256)

	// Execute the call.
	msg := &core.Message{
		From:              call.From,
		To:                call.To,
		Value:             call.Value,
		GasLimit:          call.Gas,
		GasPrice:          call.GasPrice,
		GasFeeCap:         call.GasFeeCap,
		GasTipCap:         call.GasTipCap,
		Data:              call.Data,
		AccessList:        call.AccessList,
		SkipAccountChecks: true,
	}

	// Create a new environment which holds all relevant information
	// about the transaction and calling mechanisms.
	txContext := core.NewEVMTxContext(msg)
	evmContext := core.NewEVMBlockContext(header, b.blockchain, nil)
	vmEnv := vm.NewEVM(evmContext, txContext, stateDB, b.config, vm.Config{NoBaseFee: true})
	gasPool := new(core.GasPool).AddGas(math.MaxUint64)

	return core.ApplyMessage(vmEnv, msg, gasPool)
}

// SendTransaction updates the pending block to include the given transaction.
func (b *SimulatedBackend) SendTransaction(ctx context.Context, tx *types.Transaction) error {
	b.mu.Lock()
	defer b.mu.Unlock()

	// Get the last block
	block, err := b.blockByHash(ctx, b.pendingBlock.ParentHash())
	if err != nil {
		return errors.New("could not fetch parent")
	}
	// Check transaction validity
	signer := types.MakeSigner(b.blockchain.Config(), block.Number(), block.Time())
	sender, err := types.Sender(signer, tx)
	if err != nil {
		return fmt.Errorf("invalid transaction: %v", err)
	}
	nonce := b.pendingState.GetNonce(sender)
	if tx.Nonce() != nonce {
		return fmt.Errorf("invalid transaction nonce: got %d, want %d", tx.Nonce(), nonce)
	}
	// Include tx in chain
	blocks, receipts := core.GenerateChain(b.config, block, ethash.NewFaker(), b.database, 1, func(number int, block *core.BlockGen) {
		for _, tx := range b.pendingBlock.Transactions() {
			block.AddTxWithChain(b.blockchain, tx)
		}
		block.AddTxWithChain(b.blockchain, tx)
	})
	stateDB, err := b.blockchain.State()
	if err != nil {
		return err
	}
	b.pendingBlock = blocks[0]
	b.pendingState, _ = state.New(b.pendingBlock.Root(), stateDB.Database(), nil)
	b.pendingReceipts = receipts[0]
	return nil
}

// FilterLogs executes a log filter operation, blocking during execution and
// returning all the results in one batch.
//
// TODO(karalabe): Deprecate when the subscription one can return past data too.
func (b *SimulatedBackend) FilterLogs(ctx context.Context, query ethereum.FilterQuery) ([]types.Log, error) {
	var filter *filters.Filter
	if query.BlockHash != nil {
		// Block filter requested, construct a single-shot filter
		filter = b.filterSystem.NewBlockFilter(*query.BlockHash, query.Addresses, query.Topics)
	} else {
		// Initialize unset filter boundaries to run from genesis to chain head
		from := int64(0)
		if query.FromBlock != nil {
			from = query.FromBlock.Int64()
		}
		to := int64(-1)
		if query.ToBlock != nil {
			to = query.ToBlock.Int64()
		}
		// Construct the range filter
		filter = b.filterSystem.NewRangeFilter(from, to, query.Addresses, query.Topics)
	}
	// Run the filter and return all the logs
	logs, err := filter.Logs(ctx)
	if err != nil {
		return nil, err
	}
	res := make([]types.Log, len(logs))
	for i, nLog := range logs {
		res[i] = *nLog
	}
	return res, nil
}

// SubscribeFilterLogs creates a background log filtering operation, returning a
// subscription immediately, which can be used to stream the found events.
func (b *SimulatedBackend) SubscribeFilterLogs(ctx context.Context, query ethereum.FilterQuery, ch chan<- types.Log) (ethereum.Subscription, error) {
	// Subscribe to contract events
	sink := make(chan []*types.Log)

	sub, err := b.events.SubscribeLogs(query, sink)
	if err != nil {
		return nil, err
	}
	// Since we're getting logs in batches, we need to flatten them into a plain stream
	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case logs := <-sink:
				for _, nlog := range logs {
					select {
					case ch <- *nlog:
					case err := <-sub.Err():
						return err
					case <-quit:
						return nil
					}
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// SubscribeNewHead returns an event subscription for a new header.
func (b *SimulatedBackend) SubscribeNewHead(ctx context.Context, ch chan<- *types.Header) (ethereum.Subscription, error) {
	// subscribe to a new head
	sink := make(chan *types.Header)
	sub := b.events.SubscribeNewHeads(sink)

	return event.NewSubscription(func(quit <-chan struct{}) error {
		defer sub.Unsubscribe()
		for {
			select {
			case head := <-sink:
				select {
				case ch <- head:
				case err := <-sub.Err():
					return err
				case <-quit:
					return nil
				}
			case err := <-sub.Err():
				return err
			case <-quit:
				return nil
			}
		}
	}), nil
}

// AdjustTime adds a time shift to the simulated clock.
// It can only be called on empty blocks.
func (b *SimulatedBackend) AdjustTime(adjustment time.Duration) error {
	b.mu.Lock()
	defer b.mu.Unlock()

	if len(b.pendingBlock.Transactions()) != 0 {
		return errors.New("Could not adjust time on non-empty block")
	}
	// Get the last block
	block := b.blockchain.GetBlockByHash(b.pendingBlock.ParentHash())
	if block == nil {
		return errors.New("could not find parent")
	}

	blocks, _ := core.GenerateChain(b.config, block, ethash.NewFaker(), b.database, 1, func(number int, block *core.BlockGen) {
		block.OffsetTime(int64(adjustment.Seconds()))
	})
	stateDB, err := b.blockchain.State()
	if err != nil {
		return err
	}
	b.pendingBlock = blocks[0]
	b.pendingState, _ = state.New(b.pendingBlock.Root(), stateDB.Database(), nil)
	return nil
}

// Blockchain returns the underlying blockchain.
func (b *SimulatedBackend) Blockchain() *core.BlockChain {
	return b.blockchain
}

// filterBackend implements filters.Backend to support filtering for logs without
// taking bloom-bits acceleration structures into account.
type filterBackend struct {
	db      ethdb.Database
	bc      *core.BlockChain
	backend *SimulatedBackend
}

func (fb *filterBackend) ChainDb() ethdb.Database { return fb.db }

func (fb *filterBackend) EventMux() *event.TypeMux { panic("not supported") }

func (fb *filterBackend) HeaderByNumber(ctx context.Context, number rpc.BlockNumber) (*types.Header, error) {
	switch number {
	case rpc.PendingBlockNumber:
		if block := fb.backend.pendingBlock; block != nil {
			return block.Header(), nil
		}
		return nil, nil
	case rpc.LatestBlockNumber:
		return fb.bc.CurrentHeader(), nil
	case rpc.FinalizedBlockNumber:
		return fb.bc.CurrentFinalBlock(), nil
	case rpc.SafeBlockNumber:
		return fb.bc.CurrentSafeBlock(), nil
	default:
		return fb.bc.GetHeaderByNumber(uint64(number.Int64())), nil
	}
}

func (fb *filterBackend) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	return fb.bc.GetHeaderByHash(hash), nil
}

func (fb *filterBackend) GetBody(ctx context.Context, hash common.Hash, number rpc.BlockNumber) (*types.Body, error) {
	if body := fb.bc.GetBody(hash); body != nil {
		return body, nil
	}
	return nil, errors.New("block body not found")
}

func (fb *filterBackend) PendingBlockAndReceipts() (*types.Block, types.Receipts) {
	return fb.backend.pendingBlock, fb.backend.pendingReceipts
}

func (fb *filterBackend) GetReceipts(ctx context.Context, hash common.Hash) (types.Receipts, error) {
	number := rawdb.ReadHeaderNumber(fb.db, hash)
	if number == nil {
		return nil, nil
	}
	header := rawdb.ReadHeader(fb.db, hash, *number)
	if header == nil {
		return nil, nil
	}
	return rawdb.ReadReceipts(fb.db, hash, *number, header.Time, fb.bc.Config()), nil
}

func (fb *filterBackend) GetLogs(ctx context.Context, hash common.Hash, number uint64) ([][]*types.Log, error) {
	logs := rawdb.ReadLogs(fb.db, hash, number)
	return logs, nil
}

func (fb *filterBackend) SubscribeNewTxsEvent(ch chan<- core.NewTxsEvent) event.Subscription {
	return nullSubscription()
}

func (fb *filterBackend) SubscribeChainEvent(ch chan<- core.ChainEvent) event.Subscription {
	return fb.bc.SubscribeChainEvent(ch)
}

func (fb *filterBackend) SubscribeRemovedLogsEvent(ch chan<- core.RemovedLogsEvent) event.Subscription {
	return fb.bc.SubscribeRemovedLogsEvent(ch)
}

func (fb *filterBackend) SubscribeLogsEvent(ch chan<- []*types.Log) event.Subscription {
	return fb.bc.SubscribeLogsEvent(ch)
}

func (fb *filterBackend) SubscribePendingLogsEvent(ch chan<- []*types.Log) event.Subscription {
	return nullSubscription()
}

func (fb *filterBackend) BloomStatus() (uint64, uint64) { return 4096, 0 }

func (fb *filterBackend) ServiceFilter(ctx context.Context, ms *bloombits.MatcherSession) {
	panic("not supported")
}

func (fb *filterBackend) ChainConfig() *params.ChainConfig {
	panic("not supported")
}

func (fb *filterBackend) CurrentHeader() *types.Header {
	panic("not supported")
}

func nullSubscription() event.Subscription {
	return event.NewSubscription(func(quit <-chan struct{}) error {
		<-quit
		return nil
	})
}
