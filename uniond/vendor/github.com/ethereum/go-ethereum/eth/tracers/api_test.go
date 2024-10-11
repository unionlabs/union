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

package tracers

import (
	"context"
	"crypto/ecdsa"
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"reflect"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/eth/tracers/logger"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rpc"
	"golang.org/x/exp/slices"
)

var (
	errStateNotFound = errors.New("state not found")
	errBlockNotFound = errors.New("block not found")
)

type testBackend struct {
	chainConfig *params.ChainConfig
	engine      consensus.Engine
	chaindb     ethdb.Database
	chain       *core.BlockChain

	refHook func() // Hook is invoked when the requested state is referenced
	relHook func() // Hook is invoked when the requested state is released
}

// testBackend creates a new test backend. OBS: After test is done, teardown must be
// invoked in order to release associated resources.
func newTestBackend(t *testing.T, n int, gspec *core.Genesis, generator func(i int, b *core.BlockGen)) *testBackend {
	backend := &testBackend{
		chainConfig: gspec.Config,
		engine:      ethash.NewFaker(),
		chaindb:     rawdb.NewMemoryDatabase(),
	}
	// Generate blocks for testing
	_, blocks, _ := core.GenerateChainWithGenesis(gspec, backend.engine, n, generator)

	// Import the canonical chain
	cacheConfig := &core.CacheConfig{
		TrieCleanLimit:    256,
		TrieDirtyLimit:    256,
		TrieTimeLimit:     5 * time.Minute,
		SnapshotLimit:     0,
		TrieDirtyDisabled: true, // Archive mode
	}
	chain, err := core.NewBlockChain(backend.chaindb, cacheConfig, gspec, nil, backend.engine, vm.Config{}, nil, nil)
	if err != nil {
		t.Fatalf("failed to create tester chain: %v", err)
	}
	if n, err := chain.InsertChain(blocks); err != nil {
		t.Fatalf("block %d: failed to insert into chain: %v", n, err)
	}
	backend.chain = chain
	return backend
}

func (b *testBackend) HeaderByHash(ctx context.Context, hash common.Hash) (*types.Header, error) {
	return b.chain.GetHeaderByHash(hash), nil
}

func (b *testBackend) HeaderByNumber(ctx context.Context, number rpc.BlockNumber) (*types.Header, error) {
	if number == rpc.PendingBlockNumber || number == rpc.LatestBlockNumber {
		return b.chain.CurrentHeader(), nil
	}
	return b.chain.GetHeaderByNumber(uint64(number)), nil
}

func (b *testBackend) BlockByHash(ctx context.Context, hash common.Hash) (*types.Block, error) {
	return b.chain.GetBlockByHash(hash), nil
}

func (b *testBackend) BlockByNumber(ctx context.Context, number rpc.BlockNumber) (*types.Block, error) {
	if number == rpc.PendingBlockNumber || number == rpc.LatestBlockNumber {
		return b.chain.GetBlockByNumber(b.chain.CurrentBlock().Number.Uint64()), nil
	}
	return b.chain.GetBlockByNumber(uint64(number)), nil
}

func (b *testBackend) GetTransaction(ctx context.Context, txHash common.Hash) (*types.Transaction, common.Hash, uint64, uint64, error) {
	tx, hash, blockNumber, index := rawdb.ReadTransaction(b.chaindb, txHash)
	return tx, hash, blockNumber, index, nil
}

func (b *testBackend) RPCGasCap() uint64 {
	return 25000000
}

func (b *testBackend) ChainConfig() *params.ChainConfig {
	return b.chainConfig
}

func (b *testBackend) Engine() consensus.Engine {
	return b.engine
}

func (b *testBackend) ChainDb() ethdb.Database {
	return b.chaindb
}

// teardown releases the associated resources.
func (b *testBackend) teardown() {
	b.chain.Stop()
}

func (b *testBackend) StateAtBlock(ctx context.Context, block *types.Block, reexec uint64, base *state.StateDB, readOnly bool, preferDisk bool) (*state.StateDB, StateReleaseFunc, error) {
	statedb, err := b.chain.StateAt(block.Root())
	if err != nil {
		return nil, nil, errStateNotFound
	}
	if b.refHook != nil {
		b.refHook()
	}
	release := func() {
		if b.relHook != nil {
			b.relHook()
		}
	}
	return statedb, release, nil
}

func (b *testBackend) StateAtTransaction(ctx context.Context, block *types.Block, txIndex int, reexec uint64) (*core.Message, vm.BlockContext, *state.StateDB, StateReleaseFunc, error) {
	parent := b.chain.GetBlock(block.ParentHash(), block.NumberU64()-1)
	if parent == nil {
		return nil, vm.BlockContext{}, nil, nil, errBlockNotFound
	}
	statedb, release, err := b.StateAtBlock(ctx, parent, reexec, nil, true, false)
	if err != nil {
		return nil, vm.BlockContext{}, nil, nil, errStateNotFound
	}
	if txIndex == 0 && len(block.Transactions()) == 0 {
		return nil, vm.BlockContext{}, statedb, release, nil
	}
	// Recompute transactions up to the target index.
	signer := types.MakeSigner(b.chainConfig, block.Number(), block.Time())
	for idx, tx := range block.Transactions() {
		msg, _ := core.TransactionToMessage(tx, signer, block.BaseFee())
		txContext := core.NewEVMTxContext(msg)
		context := core.NewEVMBlockContext(block.Header(), b.chain, nil)
		if idx == txIndex {
			return msg, context, statedb, release, nil
		}
		vmenv := vm.NewEVM(context, txContext, statedb, b.chainConfig, vm.Config{})
		if _, err := core.ApplyMessage(vmenv, msg, new(core.GasPool).AddGas(tx.Gas())); err != nil {
			return nil, vm.BlockContext{}, nil, nil, fmt.Errorf("transaction %#x failed: %v", tx.Hash(), err)
		}
		statedb.Finalise(vmenv.ChainConfig().IsEIP158(block.Number()))
	}
	return nil, vm.BlockContext{}, nil, nil, fmt.Errorf("transaction index %d out of range for block %#x", txIndex, block.Hash())
}

func TestTraceCall(t *testing.T) {
	t.Parallel()

	// Initialize test accounts
	accounts := newAccounts(3)
	genesis := &core.Genesis{
		Config: params.TestChainConfig,
		Alloc: core.GenesisAlloc{
			accounts[0].addr: {Balance: big.NewInt(params.Ether)},
			accounts[1].addr: {Balance: big.NewInt(params.Ether)},
			accounts[2].addr: {Balance: big.NewInt(params.Ether)},
		},
	}
	genBlocks := 10
	signer := types.HomesteadSigner{}
	backend := newTestBackend(t, genBlocks, genesis, func(i int, b *core.BlockGen) {
		// Transfer from account[0] to account[1]
		//    value: 1000 wei
		//    fee:   0 wei
		tx, _ := types.SignTx(types.NewTransaction(uint64(i), accounts[1].addr, big.NewInt(1000), params.TxGas, b.BaseFee(), nil), signer, accounts[0].key)
		b.AddTx(tx)
	})
	defer backend.teardown()
	api := NewAPI(backend)
	var testSuite = []struct {
		blockNumber rpc.BlockNumber
		call        ethapi.TransactionArgs
		config      *TraceCallConfig
		expectErr   error
		expect      string
	}{
		// Standard JSON trace upon the genesis, plain transfer.
		{
			blockNumber: rpc.BlockNumber(0),
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				To:    &accounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    nil,
			expectErr: nil,
			expect:    `{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}`,
		},
		// Standard JSON trace upon the head, plain transfer.
		{
			blockNumber: rpc.BlockNumber(genBlocks),
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				To:    &accounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    nil,
			expectErr: nil,
			expect:    `{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}`,
		},
		// Standard JSON trace upon the non-existent block, error expects
		{
			blockNumber: rpc.BlockNumber(genBlocks + 1),
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				To:    &accounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    nil,
			expectErr: fmt.Errorf("block #%d not found", genBlocks+1),
			//expect:    nil,
		},
		// Standard JSON trace upon the latest block
		{
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				To:    &accounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    nil,
			expectErr: nil,
			expect:    `{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}`,
		},
		// Tracing on 'pending' should fail:
		{
			blockNumber: rpc.PendingBlockNumber,
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				To:    &accounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    nil,
			expectErr: errors.New("tracing on top of pending is not supported"),
		},
		{
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From:  &accounts[0].addr,
				Input: &hexutil.Bytes{0x43}, // blocknumber
			},
			config: &TraceCallConfig{
				BlockOverrides: &ethapi.BlockOverrides{Number: (*hexutil.Big)(big.NewInt(0x1337))},
			},
			expectErr: nil,
			expect: ` {"gas":53018,"failed":false,"returnValue":"","structLogs":[
		{"pc":0,"op":"NUMBER","gas":24946984,"gasCost":2,"depth":1,"stack":[]},
		{"pc":1,"op":"STOP","gas":24946982,"gasCost":0,"depth":1,"stack":["0x1337"]}]}`,
		},
	}
	for i, testspec := range testSuite {
		result, err := api.TraceCall(context.Background(), testspec.call, rpc.BlockNumberOrHash{BlockNumber: &testspec.blockNumber}, testspec.config)
		if testspec.expectErr != nil {
			if err == nil {
				t.Errorf("test %d: expect error %v, got nothing", i, testspec.expectErr)
				continue
			}
			if !reflect.DeepEqual(err, testspec.expectErr) {
				t.Errorf("test %d: error mismatch, want %v, git %v", i, testspec.expectErr, err)
			}
		} else {
			if err != nil {
				t.Errorf("test %d: expect no error, got %v", i, err)
				continue
			}
			var have *logger.ExecutionResult
			if err := json.Unmarshal(result.(json.RawMessage), &have); err != nil {
				t.Errorf("test %d: failed to unmarshal result %v", i, err)
			}
			var want *logger.ExecutionResult
			if err := json.Unmarshal([]byte(testspec.expect), &want); err != nil {
				t.Errorf("test %d: failed to unmarshal result %v", i, err)
			}
			if !reflect.DeepEqual(have, want) {
				t.Errorf("test %d: result mismatch, want %v, got %v", i, testspec.expect, string(result.(json.RawMessage)))
			}
		}
	}
}

func TestTraceTransaction(t *testing.T) {
	t.Parallel()

	// Initialize test accounts
	accounts := newAccounts(2)
	genesis := &core.Genesis{
		Config: params.TestChainConfig,
		Alloc: core.GenesisAlloc{
			accounts[0].addr: {Balance: big.NewInt(params.Ether)},
			accounts[1].addr: {Balance: big.NewInt(params.Ether)},
		},
	}
	target := common.Hash{}
	signer := types.HomesteadSigner{}
	backend := newTestBackend(t, 1, genesis, func(i int, b *core.BlockGen) {
		// Transfer from account[0] to account[1]
		//    value: 1000 wei
		//    fee:   0 wei
		tx, _ := types.SignTx(types.NewTransaction(uint64(i), accounts[1].addr, big.NewInt(1000), params.TxGas, b.BaseFee(), nil), signer, accounts[0].key)
		b.AddTx(tx)
		target = tx.Hash()
	})
	defer backend.chain.Stop()
	api := NewAPI(backend)
	result, err := api.TraceTransaction(context.Background(), target, nil)
	if err != nil {
		t.Errorf("Failed to trace transaction %v", err)
	}
	var have *logger.ExecutionResult
	if err := json.Unmarshal(result.(json.RawMessage), &have); err != nil {
		t.Errorf("failed to unmarshal result %v", err)
	}
	if !reflect.DeepEqual(have, &logger.ExecutionResult{
		Gas:         params.TxGas,
		Failed:      false,
		ReturnValue: "",
		StructLogs:  []logger.StructLogRes{},
	}) {
		t.Error("Transaction tracing result is different")
	}

	// Test non-existent transaction
	_, err = api.TraceTransaction(context.Background(), common.Hash{42}, nil)
	if !errors.Is(err, errTxNotFound) {
		t.Fatalf("want %v, have %v", errTxNotFound, err)
	}
}

func TestTraceBlock(t *testing.T) {
	t.Parallel()

	// Initialize test accounts
	accounts := newAccounts(3)
	genesis := &core.Genesis{
		Config: params.TestChainConfig,
		Alloc: core.GenesisAlloc{
			accounts[0].addr: {Balance: big.NewInt(params.Ether)},
			accounts[1].addr: {Balance: big.NewInt(params.Ether)},
			accounts[2].addr: {Balance: big.NewInt(params.Ether)},
		},
	}
	genBlocks := 10
	signer := types.HomesteadSigner{}
	var txHash common.Hash
	backend := newTestBackend(t, genBlocks, genesis, func(i int, b *core.BlockGen) {
		// Transfer from account[0] to account[1]
		//    value: 1000 wei
		//    fee:   0 wei
		tx, _ := types.SignTx(types.NewTransaction(uint64(i), accounts[1].addr, big.NewInt(1000), params.TxGas, b.BaseFee(), nil), signer, accounts[0].key)
		b.AddTx(tx)
		txHash = tx.Hash()
	})
	defer backend.chain.Stop()
	api := NewAPI(backend)

	var testSuite = []struct {
		blockNumber rpc.BlockNumber
		config      *TraceConfig
		want        string
		expectErr   error
	}{
		// Trace genesis block, expect error
		{
			blockNumber: rpc.BlockNumber(0),
			expectErr:   errors.New("genesis is not traceable"),
		},
		// Trace head block
		{
			blockNumber: rpc.BlockNumber(genBlocks),
			want:        fmt.Sprintf(`[{"txHash":"%v","result":{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}}]`, txHash),
		},
		// Trace non-existent block
		{
			blockNumber: rpc.BlockNumber(genBlocks + 1),
			expectErr:   fmt.Errorf("block #%d not found", genBlocks+1),
		},
		// Trace latest block
		{
			blockNumber: rpc.LatestBlockNumber,
			want:        fmt.Sprintf(`[{"txHash":"%v","result":{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}}]`, txHash),
		},
		// Trace pending block
		{
			blockNumber: rpc.PendingBlockNumber,
			want:        fmt.Sprintf(`[{"txHash":"%v","result":{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}}]`, txHash),
		},
	}
	for i, tc := range testSuite {
		result, err := api.TraceBlockByNumber(context.Background(), tc.blockNumber, tc.config)
		if tc.expectErr != nil {
			if err == nil {
				t.Errorf("test %d, want error %v", i, tc.expectErr)
				continue
			}
			if !reflect.DeepEqual(err, tc.expectErr) {
				t.Errorf("test %d: error mismatch, want %v, get %v", i, tc.expectErr, err)
			}
			continue
		}
		if err != nil {
			t.Errorf("test %d, want no error, have %v", i, err)
			continue
		}
		have, _ := json.Marshal(result)
		want := tc.want
		if string(have) != want {
			t.Errorf("test %d, result mismatch, have\n%v\n, want\n%v\n", i, string(have), want)
		}
	}
}

func TestTracingWithOverrides(t *testing.T) {
	t.Parallel()
	// Initialize test accounts
	accounts := newAccounts(3)
	storageAccount := common.Address{0x13, 37}
	genesis := &core.Genesis{
		Config: params.TestChainConfig,
		Alloc: core.GenesisAlloc{
			accounts[0].addr: {Balance: big.NewInt(params.Ether)},
			accounts[1].addr: {Balance: big.NewInt(params.Ether)},
			accounts[2].addr: {Balance: big.NewInt(params.Ether)},
			// An account with existing storage
			storageAccount: {
				Balance: new(big.Int),
				Storage: map[common.Hash]common.Hash{
					common.HexToHash("0x03"): common.HexToHash("0x33"),
					common.HexToHash("0x04"): common.HexToHash("0x44"),
				},
			},
		},
	}
	genBlocks := 10
	signer := types.HomesteadSigner{}
	backend := newTestBackend(t, genBlocks, genesis, func(i int, b *core.BlockGen) {
		// Transfer from account[0] to account[1]
		//    value: 1000 wei
		//    fee:   0 wei
		tx, _ := types.SignTx(types.NewTransaction(uint64(i), accounts[1].addr, big.NewInt(1000), params.TxGas, b.BaseFee(), nil), signer, accounts[0].key)
		b.AddTx(tx)
	})
	defer backend.chain.Stop()
	api := NewAPI(backend)
	randomAccounts := newAccounts(3)
	type res struct {
		Gas         int
		Failed      bool
		ReturnValue string
	}
	var testSuite = []struct {
		blockNumber rpc.BlockNumber
		call        ethapi.TransactionArgs
		config      *TraceCallConfig
		expectErr   error
		want        string
	}{
		// Call which can only succeed if state is state overridden
		{
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From:  &randomAccounts[0].addr,
				To:    &randomAccounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					randomAccounts[0].addr: ethapi.OverrideAccount{Balance: newRPCBalance(new(big.Int).Mul(big.NewInt(1), big.NewInt(params.Ether)))},
				},
			},
			want: `{"gas":21000,"failed":false,"returnValue":""}`,
		},
		// Invalid call without state overriding
		{
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From:  &randomAccounts[0].addr,
				To:    &randomAccounts[1].addr,
				Value: (*hexutil.Big)(big.NewInt(1000)),
			},
			config:    &TraceCallConfig{},
			expectErr: core.ErrInsufficientFunds,
		},
		// Successful simple contract call
		//
		// // SPDX-License-Identifier: GPL-3.0
		//
		//  pragma solidity >=0.7.0 <0.8.0;
		//
		//  /**
		//   * @title Storage
		//   * @dev Store & retrieve value in a variable
		//   */
		//  contract Storage {
		//      uint256 public number;
		//      constructor() {
		//          number = block.number;
		//      }
		//  }
		{
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &randomAccounts[2].addr,
				Data: newRPCBytes(common.Hex2Bytes("8381f58a")), // call number()
			},
			config: &TraceCallConfig{
				//Tracer: &tracer,
				StateOverrides: &ethapi.StateOverride{
					randomAccounts[2].addr: ethapi.OverrideAccount{
						Code:      newRPCBytes(common.Hex2Bytes("6080604052348015600f57600080fd5b506004361060285760003560e01c80638381f58a14602d575b600080fd5b60336049565b6040518082815260200191505060405180910390f35b6000548156fea2646970667358221220eab35ffa6ab2adfe380772a48b8ba78e82a1b820a18fcb6f59aa4efb20a5f60064736f6c63430007040033")),
						StateDiff: newStates([]common.Hash{{}}, []common.Hash{common.BigToHash(big.NewInt(123))}),
					},
				},
			},
			want: `{"gas":23347,"failed":false,"returnValue":"000000000000000000000000000000000000000000000000000000000000007b"}`,
		},
		{ // Override blocknumber
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &accounts[0].addr,
				// BLOCKNUMBER PUSH1 MSTORE
				Input: newRPCBytes(common.Hex2Bytes("4360005260206000f3")),
				//&hexutil.Bytes{0x43}, // blocknumber
			},
			config: &TraceCallConfig{
				BlockOverrides: &ethapi.BlockOverrides{Number: (*hexutil.Big)(big.NewInt(0x1337))},
			},
			want: `{"gas":59537,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000001337"}`,
		},
		{ // Override blocknumber, and query a blockhash
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &accounts[0].addr,
				Input: &hexutil.Bytes{
					0x60, 0x00, 0x40, // BLOCKHASH(0)
					0x60, 0x00, 0x52, // STORE memory offset 0
					0x61, 0x13, 0x36, 0x40, // BLOCKHASH(0x1336)
					0x60, 0x20, 0x52, // STORE memory offset 32
					0x61, 0x13, 0x37, 0x40, // BLOCKHASH(0x1337)
					0x60, 0x40, 0x52, // STORE memory offset 64
					0x60, 0x60, 0x60, 0x00, 0xf3, // RETURN (0-96)

				}, // blocknumber
			},
			config: &TraceCallConfig{
				BlockOverrides: &ethapi.BlockOverrides{Number: (*hexutil.Big)(big.NewInt(0x1337))},
			},
			want: `{"gas":72666,"failed":false,"returnValue":"000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"}`,
		},
		/*
			pragma solidity =0.8.12;

			contract Test {
			    uint private x;

			    function test2() external {
			        x = 1337;
			        revert();
			    }

			    function test() external returns (uint) {
			        x = 1;
			        try this.test2() {} catch (bytes memory) {}
			        return x;
			    }
			}
		*/
		{ // First with only code override, not storage override
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &randomAccounts[2].addr,
				Data: newRPCBytes(common.Hex2Bytes("f8a8fd6d")), //
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					randomAccounts[2].addr: ethapi.OverrideAccount{
						Code: newRPCBytes(common.Hex2Bytes("6080604052348015600f57600080fd5b506004361060325760003560e01c806366e41cb7146037578063f8a8fd6d14603f575b600080fd5b603d6057565b005b60456062565b60405190815260200160405180910390f35b610539600090815580fd5b60006001600081905550306001600160a01b03166366e41cb76040518163ffffffff1660e01b8152600401600060405180830381600087803b15801560a657600080fd5b505af192505050801560b6575060015b60e9573d80801560e1576040519150601f19603f3d011682016040523d82523d6000602084013e60e6565b606091505b50505b506000549056fea26469706673582212205ce45de745a5308f713cb2f448589177ba5a442d1a2eff945afaa8915961b4d064736f6c634300080c0033")),
					},
				},
			},
			want: `{"gas":44100,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000001"}`,
		},
		{ // Same again, this time with storage override
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &randomAccounts[2].addr,
				Data: newRPCBytes(common.Hex2Bytes("f8a8fd6d")), //
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					randomAccounts[2].addr: ethapi.OverrideAccount{
						Code:  newRPCBytes(common.Hex2Bytes("6080604052348015600f57600080fd5b506004361060325760003560e01c806366e41cb7146037578063f8a8fd6d14603f575b600080fd5b603d6057565b005b60456062565b60405190815260200160405180910390f35b610539600090815580fd5b60006001600081905550306001600160a01b03166366e41cb76040518163ffffffff1660e01b8152600401600060405180830381600087803b15801560a657600080fd5b505af192505050801560b6575060015b60e9573d80801560e1576040519150601f19603f3d011682016040523d82523d6000602084013e60e6565b606091505b50505b506000549056fea26469706673582212205ce45de745a5308f713cb2f448589177ba5a442d1a2eff945afaa8915961b4d064736f6c634300080c0033")),
						State: newStates([]common.Hash{{}}, []common.Hash{{}}),
					},
				},
			},
			//want: `{"gas":46900,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000539"}`,
			want: `{"gas":44100,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000001"}`,
		},
		{ // No state override
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &storageAccount,
				Data: newRPCBytes(common.Hex2Bytes("f8a8fd6d")), //
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					storageAccount: ethapi.OverrideAccount{
						Code: newRPCBytes([]byte{
							// SLOAD(3) + SLOAD(4) (which is 0x77)
							byte(vm.PUSH1), 0x04,
							byte(vm.SLOAD),
							byte(vm.PUSH1), 0x03,
							byte(vm.SLOAD),
							byte(vm.ADD),
							// 0x77 -> MSTORE(0)
							byte(vm.PUSH1), 0x00,
							byte(vm.MSTORE),
							// RETURN (0, 32)
							byte(vm.PUSH1), 32,
							byte(vm.PUSH1), 00,
							byte(vm.RETURN),
						}),
					},
				},
			},
			want: `{"gas":25288,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000077"}`,
		},
		{ // Full state override
			// The original storage is
			// 3: 0x33
			// 4: 0x44
			// With a full override, where we set 3:0x11, the slot 4 should be
			// removed. So SLOT(3)+SLOT(4) should be 0x11.
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &storageAccount,
				Data: newRPCBytes(common.Hex2Bytes("f8a8fd6d")), //
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					storageAccount: ethapi.OverrideAccount{
						Code: newRPCBytes([]byte{
							// SLOAD(3) + SLOAD(4) (which is now 0x11 + 0x00)
							byte(vm.PUSH1), 0x04,
							byte(vm.SLOAD),
							byte(vm.PUSH1), 0x03,
							byte(vm.SLOAD),
							byte(vm.ADD),
							// 0x11 -> MSTORE(0)
							byte(vm.PUSH1), 0x00,
							byte(vm.MSTORE),
							// RETURN (0, 32)
							byte(vm.PUSH1), 32,
							byte(vm.PUSH1), 00,
							byte(vm.RETURN),
						}),
						State: newStates(
							[]common.Hash{common.HexToHash("0x03")},
							[]common.Hash{common.HexToHash("0x11")}),
					},
				},
			},
			want: `{"gas":25288,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000011"}`,
		},
		{ // Partial state override
			// The original storage is
			// 3: 0x33
			// 4: 0x44
			// With a partial override, where we set 3:0x11, the slot 4 as before.
			// So SLOT(3)+SLOT(4) should be 0x55.
			blockNumber: rpc.LatestBlockNumber,
			call: ethapi.TransactionArgs{
				From: &randomAccounts[0].addr,
				To:   &storageAccount,
				Data: newRPCBytes(common.Hex2Bytes("f8a8fd6d")), //
			},
			config: &TraceCallConfig{
				StateOverrides: &ethapi.StateOverride{
					storageAccount: ethapi.OverrideAccount{
						Code: newRPCBytes([]byte{
							// SLOAD(3) + SLOAD(4) (which is now 0x11 + 0x44)
							byte(vm.PUSH1), 0x04,
							byte(vm.SLOAD),
							byte(vm.PUSH1), 0x03,
							byte(vm.SLOAD),
							byte(vm.ADD),
							// 0x55 -> MSTORE(0)
							byte(vm.PUSH1), 0x00,
							byte(vm.MSTORE),
							// RETURN (0, 32)
							byte(vm.PUSH1), 32,
							byte(vm.PUSH1), 00,
							byte(vm.RETURN),
						}),
						StateDiff: &map[common.Hash]common.Hash{
							common.HexToHash("0x03"): common.HexToHash("0x11"),
						},
					},
				},
			},
			want: `{"gas":25288,"failed":false,"returnValue":"0000000000000000000000000000000000000000000000000000000000000055"}`,
		},
	}
	for i, tc := range testSuite {
		result, err := api.TraceCall(context.Background(), tc.call, rpc.BlockNumberOrHash{BlockNumber: &tc.blockNumber}, tc.config)
		if tc.expectErr != nil {
			if err == nil {
				t.Errorf("test %d: want error %v, have nothing", i, tc.expectErr)
				continue
			}
			if !errors.Is(err, tc.expectErr) {
				t.Errorf("test %d: error mismatch, want %v, have %v", i, tc.expectErr, err)
			}
			continue
		}
		if err != nil {
			t.Errorf("test %d: want no error, have %v", i, err)
			continue
		}
		// Turn result into res-struct
		var (
			have res
			want res
		)
		resBytes, _ := json.Marshal(result)
		json.Unmarshal(resBytes, &have)
		json.Unmarshal([]byte(tc.want), &want)
		if !reflect.DeepEqual(have, want) {
			t.Logf("result: %v\n", string(resBytes))
			t.Errorf("test %d, result mismatch, have\n%v\n, want\n%v\n", i, have, want)
		}
	}
}

type Account struct {
	key  *ecdsa.PrivateKey
	addr common.Address
}

func newAccounts(n int) (accounts []Account) {
	for i := 0; i < n; i++ {
		key, _ := crypto.GenerateKey()
		addr := crypto.PubkeyToAddress(key.PublicKey)
		accounts = append(accounts, Account{key: key, addr: addr})
	}
	slices.SortFunc(accounts, func(a, b Account) int { return a.addr.Cmp(b.addr) })
	return accounts
}

func newRPCBalance(balance *big.Int) **hexutil.Big {
	rpcBalance := (*hexutil.Big)(balance)
	return &rpcBalance
}

func newRPCBytes(bytes []byte) *hexutil.Bytes {
	rpcBytes := hexutil.Bytes(bytes)
	return &rpcBytes
}

func newStates(keys []common.Hash, vals []common.Hash) *map[common.Hash]common.Hash {
	if len(keys) != len(vals) {
		panic("invalid input")
	}
	m := make(map[common.Hash]common.Hash)
	for i := 0; i < len(keys); i++ {
		m[keys[i]] = vals[i]
	}
	return &m
}

func TestTraceChain(t *testing.T) {
	// Initialize test accounts
	accounts := newAccounts(3)
	genesis := &core.Genesis{
		Config: params.TestChainConfig,
		Alloc: core.GenesisAlloc{
			accounts[0].addr: {Balance: big.NewInt(params.Ether)},
			accounts[1].addr: {Balance: big.NewInt(params.Ether)},
			accounts[2].addr: {Balance: big.NewInt(params.Ether)},
		},
	}
	genBlocks := 50
	signer := types.HomesteadSigner{}

	var (
		ref   atomic.Uint32 // total refs has made
		rel   atomic.Uint32 // total rels has made
		nonce uint64
	)
	backend := newTestBackend(t, genBlocks, genesis, func(i int, b *core.BlockGen) {
		// Transfer from account[0] to account[1]
		//    value: 1000 wei
		//    fee:   0 wei
		for j := 0; j < i+1; j++ {
			tx, _ := types.SignTx(types.NewTransaction(nonce, accounts[1].addr, big.NewInt(1000), params.TxGas, b.BaseFee(), nil), signer, accounts[0].key)
			b.AddTx(tx)
			nonce += 1
		}
	})
	backend.refHook = func() { ref.Add(1) }
	backend.relHook = func() { rel.Add(1) }
	api := NewAPI(backend)

	single := `{"txHash":"0x0000000000000000000000000000000000000000000000000000000000000000","result":{"gas":21000,"failed":false,"returnValue":"","structLogs":[]}}`
	var cases = []struct {
		start  uint64
		end    uint64
		config *TraceConfig
	}{
		{0, 50, nil},  // the entire chain range, blocks [1, 50]
		{10, 20, nil}, // the middle chain range, blocks [11, 20]
	}
	for _, c := range cases {
		ref.Store(0)
		rel.Store(0)

		from, _ := api.blockByNumber(context.Background(), rpc.BlockNumber(c.start))
		to, _ := api.blockByNumber(context.Background(), rpc.BlockNumber(c.end))
		resCh := api.traceChain(from, to, c.config, nil)

		next := c.start + 1
		for result := range resCh {
			if have, want := uint64(result.Block), next; have != want {
				t.Fatalf("unexpected tracing block, have %d want %d", have, want)
			}
			if have, want := len(result.Traces), int(next); have != want {
				t.Fatalf("unexpected result length, have %d want %d", have, want)
			}
			for _, trace := range result.Traces {
				trace.TxHash = common.Hash{}
				blob, _ := json.Marshal(trace)
				if have, want := string(blob), single; have != want {
					t.Fatalf("unexpected tracing result, have\n%v\nwant:\n%v", have, want)
				}
			}
			next += 1
		}
		if next != c.end+1 {
			t.Error("Missing tracing block")
		}

		if nref, nrel := ref.Load(), rel.Load(); nref != nrel {
			t.Errorf("Ref and deref actions are not equal, ref %d rel %d", nref, nrel)
		}
	}
}
