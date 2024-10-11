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

package core

import (
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/misc"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/consensus/misc/eip4844"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/trie"
)

// BlockGen creates blocks for testing.
// See GenerateChain for a detailed explanation.
type BlockGen struct {
	i       int
	parent  *types.Block
	chain   []*types.Block
	header  *types.Header
	statedb *state.StateDB

	gasPool     *GasPool
	txs         []*types.Transaction
	receipts    []*types.Receipt
	uncles      []*types.Header
	withdrawals []*types.Withdrawal

	config *params.ChainConfig
	engine consensus.Engine
}

// SetCoinbase sets the coinbase of the generated block.
// It can be called at most once.
func (b *BlockGen) SetCoinbase(addr common.Address) {
	if b.gasPool != nil {
		if len(b.txs) > 0 {
			panic("coinbase must be set before adding transactions")
		}
		panic("coinbase can only be set once")
	}
	b.header.Coinbase = addr
	b.gasPool = new(GasPool).AddGas(b.header.GasLimit)
}

// SetExtra sets the extra data field of the generated block.
func (b *BlockGen) SetExtra(data []byte) {
	b.header.Extra = data
}

// SetNonce sets the nonce field of the generated block.
func (b *BlockGen) SetNonce(nonce types.BlockNonce) {
	b.header.Nonce = nonce
}

// SetDifficulty sets the difficulty field of the generated block. This method is
// useful for Clique tests where the difficulty does not depend on time. For the
// ethash tests, please use OffsetTime, which implicitly recalculates the diff.
func (b *BlockGen) SetDifficulty(diff *big.Int) {
	b.header.Difficulty = diff
}

// SetPos makes the header a PoS-header (0 difficulty)
func (b *BlockGen) SetPoS() {
	b.header.Difficulty = new(big.Int)
}

// SetParentBeaconRoot sets the parent beacon root field of the generated
// block.
func (b *BlockGen) SetParentBeaconRoot(root common.Hash) {
	b.header.ParentBeaconRoot = &root
	var (
		blockContext = NewEVMBlockContext(b.header, nil, &b.header.Coinbase)
		vmenv        = vm.NewEVM(blockContext, vm.TxContext{}, b.statedb, b.config, vm.Config{})
	)
	ProcessBeaconBlockRoot(root, vmenv, b.statedb)
}

// addTx adds a transaction to the generated block. If no coinbase has
// been set, the block's coinbase is set to the zero address.
//
// There are a few options can be passed as well in order to run some
// customized rules.
// - bc:       enables the ability to query historical block hashes for BLOCKHASH
// - vmConfig: extends the flexibility for customizing evm rules, e.g. enable extra EIPs
func (b *BlockGen) addTx(bc *BlockChain, vmConfig vm.Config, tx *types.Transaction) {
	if b.gasPool == nil {
		b.SetCoinbase(common.Address{})
	}
	b.statedb.SetTxContext(tx.Hash(), len(b.txs))
	receipt, err := ApplyTransaction(b.config, bc, &b.header.Coinbase, b.gasPool, b.statedb, b.header, tx, &b.header.GasUsed, vmConfig)
	if err != nil {
		panic(err)
	}
	b.txs = append(b.txs, tx)
	b.receipts = append(b.receipts, receipt)
	if b.header.BlobGasUsed != nil {
		*b.header.BlobGasUsed += receipt.BlobGasUsed
	}
}

// AddTx adds a transaction to the generated block. If no coinbase has
// been set, the block's coinbase is set to the zero address.
//
// AddTx panics if the transaction cannot be executed. In addition to
// the protocol-imposed limitations (gas limit, etc.), there are some
// further limitations on the content of transactions that can be
// added. Notably, contract code relying on the BLOCKHASH instruction
// will panic during execution.
func (b *BlockGen) AddTx(tx *types.Transaction) {
	b.addTx(nil, vm.Config{}, tx)
}

// AddTxWithChain adds a transaction to the generated block. If no coinbase has
// been set, the block's coinbase is set to the zero address.
//
// AddTxWithChain panics if the transaction cannot be executed. In addition to
// the protocol-imposed limitations (gas limit, etc.), there are some
// further limitations on the content of transactions that can be
// added. If contract code relies on the BLOCKHASH instruction,
// the block in chain will be returned.
func (b *BlockGen) AddTxWithChain(bc *BlockChain, tx *types.Transaction) {
	b.addTx(bc, vm.Config{}, tx)
}

// AddTxWithVMConfig adds a transaction to the generated block. If no coinbase has
// been set, the block's coinbase is set to the zero address.
// The evm interpreter can be customized with the provided vm config.
func (b *BlockGen) AddTxWithVMConfig(tx *types.Transaction, config vm.Config) {
	b.addTx(nil, config, tx)
}

// GetBalance returns the balance of the given address at the generated block.
func (b *BlockGen) GetBalance(addr common.Address) *big.Int {
	return b.statedb.GetBalance(addr)
}

// AddUncheckedTx forcefully adds a transaction to the block without any
// validation.
//
// AddUncheckedTx will cause consensus failures when used during real
// chain processing. This is best used in conjunction with raw block insertion.
func (b *BlockGen) AddUncheckedTx(tx *types.Transaction) {
	b.txs = append(b.txs, tx)
}

// Number returns the block number of the block being generated.
func (b *BlockGen) Number() *big.Int {
	return new(big.Int).Set(b.header.Number)
}

// Timestamp returns the timestamp of the block being generated.
func (b *BlockGen) Timestamp() uint64 {
	return b.header.Time
}

// BaseFee returns the EIP-1559 base fee of the block being generated.
func (b *BlockGen) BaseFee() *big.Int {
	return new(big.Int).Set(b.header.BaseFee)
}

// AddUncheckedReceipt forcefully adds a receipts to the block without a
// backing transaction.
//
// AddUncheckedReceipt will cause consensus failures when used during real
// chain processing. This is best used in conjunction with raw block insertion.
func (b *BlockGen) AddUncheckedReceipt(receipt *types.Receipt) {
	b.receipts = append(b.receipts, receipt)
}

// TxNonce returns the next valid transaction nonce for the
// account at addr. It panics if the account does not exist.
func (b *BlockGen) TxNonce(addr common.Address) uint64 {
	if !b.statedb.Exist(addr) {
		panic("account does not exist")
	}
	return b.statedb.GetNonce(addr)
}

// AddUncle adds an uncle header to the generated block.
func (b *BlockGen) AddUncle(h *types.Header) {
	// The uncle will have the same timestamp and auto-generated difficulty
	h.Time = b.header.Time

	var parent *types.Header
	for i := b.i - 1; i >= 0; i-- {
		if b.chain[i].Hash() == h.ParentHash {
			parent = b.chain[i].Header()
			break
		}
	}
	chainreader := &fakeChainReader{config: b.config}
	h.Difficulty = b.engine.CalcDifficulty(chainreader, b.header.Time, parent)

	// The gas limit and price should be derived from the parent
	h.GasLimit = parent.GasLimit
	if b.config.IsLondon(h.Number) {
		h.BaseFee = eip1559.CalcBaseFee(b.config, parent)
		if !b.config.IsLondon(parent.Number) {
			parentGasLimit := parent.GasLimit * b.config.ElasticityMultiplier()
			h.GasLimit = CalcGasLimit(parentGasLimit, parentGasLimit)
		}
	}
	b.uncles = append(b.uncles, h)
}

// AddWithdrawal adds a withdrawal to the generated block.
// It returns the withdrawal index.
func (b *BlockGen) AddWithdrawal(w *types.Withdrawal) uint64 {
	cpy := *w
	cpy.Index = b.nextWithdrawalIndex()
	b.withdrawals = append(b.withdrawals, &cpy)
	return cpy.Index
}

// nextWithdrawalIndex computes the index of the next withdrawal.
func (b *BlockGen) nextWithdrawalIndex() uint64 {
	if len(b.withdrawals) != 0 {
		return b.withdrawals[len(b.withdrawals)-1].Index + 1
	}
	for i := b.i - 1; i >= 0; i-- {
		if wd := b.chain[i].Withdrawals(); len(wd) != 0 {
			return wd[len(wd)-1].Index + 1
		}
		if i == 0 {
			// Correctly set the index if no parent had withdrawals.
			if wd := b.parent.Withdrawals(); len(wd) != 0 {
				return wd[len(wd)-1].Index + 1
			}
		}
	}
	return 0
}

// PrevBlock returns a previously generated block by number. It panics if
// num is greater or equal to the number of the block being generated.
// For index -1, PrevBlock returns the parent block given to GenerateChain.
func (b *BlockGen) PrevBlock(index int) *types.Block {
	if index >= b.i {
		panic(fmt.Errorf("block index %d out of range (%d,%d)", index, -1, b.i))
	}
	if index == -1 {
		return b.parent
	}
	return b.chain[index]
}

// OffsetTime modifies the time instance of a block, implicitly changing its
// associated difficulty. It's useful to test scenarios where forking is not
// tied to chain length directly.
func (b *BlockGen) OffsetTime(seconds int64) {
	b.header.Time += uint64(seconds)
	if b.header.Time <= b.parent.Header().Time {
		panic("block time out of range")
	}
	chainreader := &fakeChainReader{config: b.config}
	b.header.Difficulty = b.engine.CalcDifficulty(chainreader, b.header.Time, b.parent.Header())
}

// GenerateChain creates a chain of n blocks. The first block's
// parent will be the provided parent. db is used to store
// intermediate states and should contain the parent's state trie.
//
// The generator function is called with a new block generator for
// every block. Any transactions and uncles added to the generator
// become part of the block. If gen is nil, the blocks will be empty
// and their coinbase will be the zero address.
//
// Blocks created by GenerateChain do not contain valid proof of work
// values. Inserting them into BlockChain requires use of FakePow or
// a similar non-validating proof of work implementation.
func GenerateChain(config *params.ChainConfig, parent *types.Block, engine consensus.Engine, db ethdb.Database, n int, gen func(int, *BlockGen)) ([]*types.Block, []types.Receipts) {
	if config == nil {
		config = params.TestChainConfig
	}
	blocks, receipts := make(types.Blocks, n), make([]types.Receipts, n)
	chainreader := &fakeChainReader{config: config}
	genblock := func(i int, parent *types.Block, triedb *trie.Database, statedb *state.StateDB) (*types.Block, types.Receipts) {
		b := &BlockGen{i: i, chain: blocks, parent: parent, statedb: statedb, config: config, engine: engine}
		b.header = makeHeader(chainreader, parent, statedb, b.engine)

		// Set the difficulty for clique block. The chain maker doesn't have access
		// to a chain, so the difficulty will be left unset (nil). Set it here to the
		// correct value.
		if b.header.Difficulty == nil {
			if config.TerminalTotalDifficulty == nil {
				// Clique chain
				b.header.Difficulty = big.NewInt(2)
			} else {
				// Post-merge chain
				b.header.Difficulty = big.NewInt(0)
			}
		}
		// Mutate the state and block according to any hard-fork specs
		if daoBlock := config.DAOForkBlock; daoBlock != nil {
			limit := new(big.Int).Add(daoBlock, params.DAOForkExtraRange)
			if b.header.Number.Cmp(daoBlock) >= 0 && b.header.Number.Cmp(limit) < 0 {
				if config.DAOForkSupport {
					b.header.Extra = common.CopyBytes(params.DAOForkBlockExtra)
				}
			}
		}
		if config.DAOForkSupport && config.DAOForkBlock != nil && config.DAOForkBlock.Cmp(b.header.Number) == 0 {
			misc.ApplyDAOHardFork(statedb)
		}
		// Execute any user modifications to the block
		if gen != nil {
			gen(i, b)
		}
		if b.engine != nil {
			block, err := b.engine.FinalizeAndAssemble(chainreader, b.header, statedb, b.txs, b.uncles, b.receipts, b.withdrawals)
			if err != nil {
				panic(err)
			}

			// Write state changes to db
			root, err := statedb.Commit(b.header.Number.Uint64(), config.IsEIP158(b.header.Number))
			if err != nil {
				panic(fmt.Sprintf("state write error: %v", err))
			}
			if err = triedb.Commit(root, false); err != nil {
				panic(fmt.Sprintf("trie write error: %v", err))
			}
			return block, b.receipts
		}
		return nil, nil
	}
	// Forcibly use hash-based state scheme for retaining all nodes in disk.
	triedb := trie.NewDatabase(db, trie.HashDefaults)
	defer triedb.Close()

	for i := 0; i < n; i++ {
		statedb, err := state.New(parent.Root(), state.NewDatabaseWithNodeDB(db, triedb), nil)
		if err != nil {
			panic(err)
		}
		block, receipt := genblock(i, parent, triedb, statedb)
		blocks[i] = block
		receipts[i] = receipt
		parent = block
	}
	return blocks, receipts
}

// GenerateChainWithGenesis is a wrapper of GenerateChain which will initialize
// genesis block to database first according to the provided genesis specification
// then generate chain on top.
func GenerateChainWithGenesis(genesis *Genesis, engine consensus.Engine, n int, gen func(int, *BlockGen)) (ethdb.Database, []*types.Block, []types.Receipts) {
	db := rawdb.NewMemoryDatabase()
	triedb := trie.NewDatabase(db, trie.HashDefaults)
	defer triedb.Close()
	_, err := genesis.Commit(db, triedb)
	if err != nil {
		panic(err)
	}
	blocks, receipts := GenerateChain(genesis.Config, genesis.ToBlock(), engine, db, n, gen)
	return db, blocks, receipts
}

func makeHeader(chain consensus.ChainReader, parent *types.Block, state *state.StateDB, engine consensus.Engine) *types.Header {
	var time uint64
	if parent.Time() == 0 {
		time = 10
	} else {
		time = parent.Time() + 10 // block time is fixed at 10 seconds
	}
	header := &types.Header{
		Root:       state.IntermediateRoot(chain.Config().IsEIP158(parent.Number())),
		ParentHash: parent.Hash(),
		Coinbase:   parent.Coinbase(),
		Difficulty: engine.CalcDifficulty(chain, time, &types.Header{
			Number:     parent.Number(),
			Time:       time - 10,
			Difficulty: parent.Difficulty(),
			UncleHash:  parent.UncleHash(),
		}),
		GasLimit: parent.GasLimit(),
		Number:   new(big.Int).Add(parent.Number(), common.Big1),
		Time:     time,
	}
	if chain.Config().IsLondon(header.Number) {
		header.BaseFee = eip1559.CalcBaseFee(chain.Config(), parent.Header())
		if !chain.Config().IsLondon(parent.Number()) {
			parentGasLimit := parent.GasLimit() * chain.Config().ElasticityMultiplier()
			header.GasLimit = CalcGasLimit(parentGasLimit, parentGasLimit)
		}
	}
	if chain.Config().IsCancun(header.Number, header.Time) {
		var (
			parentExcessBlobGas uint64
			parentBlobGasUsed   uint64
		)
		if parent.ExcessBlobGas() != nil {
			parentExcessBlobGas = *parent.ExcessBlobGas()
			parentBlobGasUsed = *parent.BlobGasUsed()
		}
		excessBlobGas := eip4844.CalcExcessBlobGas(parentExcessBlobGas, parentBlobGasUsed)
		header.ExcessBlobGas = &excessBlobGas
		header.BlobGasUsed = new(uint64)
		header.ParentBeaconRoot = new(common.Hash)
	}
	return header
}

// makeHeaderChain creates a deterministic chain of headers rooted at parent.
func makeHeaderChain(chainConfig *params.ChainConfig, parent *types.Header, n int, engine consensus.Engine, db ethdb.Database, seed int) []*types.Header {
	blocks := makeBlockChain(chainConfig, types.NewBlockWithHeader(parent), n, engine, db, seed)
	headers := make([]*types.Header, len(blocks))
	for i, block := range blocks {
		headers[i] = block.Header()
	}
	return headers
}

// makeHeaderChainWithGenesis creates a deterministic chain of headers from genesis.
func makeHeaderChainWithGenesis(genesis *Genesis, n int, engine consensus.Engine, seed int) (ethdb.Database, []*types.Header) {
	db, blocks := makeBlockChainWithGenesis(genesis, n, engine, seed)
	headers := make([]*types.Header, len(blocks))
	for i, block := range blocks {
		headers[i] = block.Header()
	}
	return db, headers
}

// makeBlockChain creates a deterministic chain of blocks rooted at parent.
func makeBlockChain(chainConfig *params.ChainConfig, parent *types.Block, n int, engine consensus.Engine, db ethdb.Database, seed int) []*types.Block {
	blocks, _ := GenerateChain(chainConfig, parent, engine, db, n, func(i int, b *BlockGen) {
		b.SetCoinbase(common.Address{0: byte(seed), 19: byte(i)})
	})
	return blocks
}

// makeBlockChain creates a deterministic chain of blocks from genesis
func makeBlockChainWithGenesis(genesis *Genesis, n int, engine consensus.Engine, seed int) (ethdb.Database, []*types.Block) {
	db, blocks, _ := GenerateChainWithGenesis(genesis, engine, n, func(i int, b *BlockGen) {
		b.SetCoinbase(common.Address{0: byte(seed), 19: byte(i)})
	})
	return db, blocks
}

type fakeChainReader struct {
	config *params.ChainConfig
}

// Config returns the chain configuration.
func (cr *fakeChainReader) Config() *params.ChainConfig {
	return cr.config
}

func (cr *fakeChainReader) CurrentHeader() *types.Header                            { return nil }
func (cr *fakeChainReader) GetHeaderByNumber(number uint64) *types.Header           { return nil }
func (cr *fakeChainReader) GetHeaderByHash(hash common.Hash) *types.Header          { return nil }
func (cr *fakeChainReader) GetHeader(hash common.Hash, number uint64) *types.Header { return nil }
func (cr *fakeChainReader) GetBlock(hash common.Hash, number uint64) *types.Block   { return nil }
func (cr *fakeChainReader) GetTd(hash common.Hash, number uint64) *big.Int          { return nil }
