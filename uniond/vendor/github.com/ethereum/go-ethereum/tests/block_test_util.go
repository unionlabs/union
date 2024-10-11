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

// Package tests implements execution of Ethereum JSON tests.
package tests

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"reflect"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/consensus/beacon"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/triedb/hashdb"
	"github.com/ethereum/go-ethereum/trie/triedb/pathdb"
)

// A BlockTest checks handling of entire blocks.
type BlockTest struct {
	json btJSON
}

// UnmarshalJSON implements json.Unmarshaler interface.
func (t *BlockTest) UnmarshalJSON(in []byte) error {
	return json.Unmarshal(in, &t.json)
}

type btJSON struct {
	Blocks     []btBlock             `json:"blocks"`
	Genesis    btHeader              `json:"genesisBlockHeader"`
	Pre        core.GenesisAlloc     `json:"pre"`
	Post       core.GenesisAlloc     `json:"postState"`
	BestBlock  common.UnprefixedHash `json:"lastblockhash"`
	Network    string                `json:"network"`
	SealEngine string                `json:"sealEngine"`
}

type btBlock struct {
	BlockHeader     *btHeader
	ExpectException string
	Rlp             string
	UncleHeaders    []*btHeader
}

//go:generate go run github.com/fjl/gencodec -type btHeader -field-override btHeaderMarshaling -out gen_btheader.go

type btHeader struct {
	Bloom                 types.Bloom
	Coinbase              common.Address
	MixHash               common.Hash
	Nonce                 types.BlockNonce
	Number                *big.Int
	Hash                  common.Hash
	ParentHash            common.Hash
	ReceiptTrie           common.Hash
	StateRoot             common.Hash
	TransactionsTrie      common.Hash
	UncleHash             common.Hash
	ExtraData             []byte
	Difficulty            *big.Int
	GasLimit              uint64
	GasUsed               uint64
	Timestamp             uint64
	BaseFeePerGas         *big.Int
	WithdrawalsRoot       *common.Hash
	BlobGasUsed           *uint64
	ExcessBlobGas         *uint64
	ParentBeaconBlockRoot *common.Hash
}

type btHeaderMarshaling struct {
	ExtraData     hexutil.Bytes
	Number        *math.HexOrDecimal256
	Difficulty    *math.HexOrDecimal256
	GasLimit      math.HexOrDecimal64
	GasUsed       math.HexOrDecimal64
	Timestamp     math.HexOrDecimal64
	BaseFeePerGas *math.HexOrDecimal256
	BlobGasUsed   *math.HexOrDecimal64
	ExcessBlobGas *math.HexOrDecimal64
}

func (t *BlockTest) Run(snapshotter bool, scheme string, tracer vm.EVMLogger) error {
	config, ok := Forks[t.json.Network]
	if !ok {
		return UnsupportedForkError{t.json.Network}
	}
	// import pre accounts & construct test genesis block & state root
	var (
		db    = rawdb.NewMemoryDatabase()
		tconf = &trie.Config{}
	)
	if scheme == rawdb.PathScheme {
		tconf.PathDB = pathdb.Defaults
	} else {
		tconf.HashDB = hashdb.Defaults
	}
	// Commit genesis state
	gspec := t.genesis(config)
	triedb := trie.NewDatabase(db, tconf)
	gblock, err := gspec.Commit(db, triedb)
	if err != nil {
		return err
	}
	triedb.Close() // close the db to prevent memory leak

	if gblock.Hash() != t.json.Genesis.Hash {
		return fmt.Errorf("genesis block hash doesn't match test: computed=%x, test=%x", gblock.Hash().Bytes()[:6], t.json.Genesis.Hash[:6])
	}
	if gblock.Root() != t.json.Genesis.StateRoot {
		return fmt.Errorf("genesis block state root does not match test: computed=%x, test=%x", gblock.Root().Bytes()[:6], t.json.Genesis.StateRoot[:6])
	}
	// Wrap the original engine within the beacon-engine
	engine := beacon.New(ethash.NewFaker())

	cache := &core.CacheConfig{TrieCleanLimit: 0, StateScheme: scheme}
	if snapshotter {
		cache.SnapshotLimit = 1
		cache.SnapshotWait = true
	}
	chain, err := core.NewBlockChain(db, cache, gspec, nil, engine, vm.Config{
		Tracer: tracer,
	}, nil, nil)
	if err != nil {
		return err
	}
	defer chain.Stop()

	validBlocks, err := t.insertBlocks(chain)
	if err != nil {
		return err
	}
	cmlast := chain.CurrentBlock().Hash()
	if common.Hash(t.json.BestBlock) != cmlast {
		return fmt.Errorf("last block hash validation mismatch: want: %x, have: %x", t.json.BestBlock, cmlast)
	}
	newDB, err := chain.State()
	if err != nil {
		return err
	}
	if err = t.validatePostState(newDB); err != nil {
		return fmt.Errorf("post state validation failed: %v", err)
	}
	// Cross-check the snapshot-to-hash against the trie hash
	if snapshotter {
		if err := chain.Snapshots().Verify(chain.CurrentBlock().Root); err != nil {
			return err
		}
	}
	return t.validateImportedHeaders(chain, validBlocks)
}

func (t *BlockTest) genesis(config *params.ChainConfig) *core.Genesis {
	return &core.Genesis{
		Config:        config,
		Nonce:         t.json.Genesis.Nonce.Uint64(),
		Timestamp:     t.json.Genesis.Timestamp,
		ParentHash:    t.json.Genesis.ParentHash,
		ExtraData:     t.json.Genesis.ExtraData,
		GasLimit:      t.json.Genesis.GasLimit,
		GasUsed:       t.json.Genesis.GasUsed,
		Difficulty:    t.json.Genesis.Difficulty,
		Mixhash:       t.json.Genesis.MixHash,
		Coinbase:      t.json.Genesis.Coinbase,
		Alloc:         t.json.Pre,
		BaseFee:       t.json.Genesis.BaseFeePerGas,
		BlobGasUsed:   t.json.Genesis.BlobGasUsed,
		ExcessBlobGas: t.json.Genesis.ExcessBlobGas,
	}
}

/*
See https://github.com/ethereum/tests/wiki/Blockchain-Tests-II

	Whether a block is valid or not is a bit subtle, it's defined by presence of
	blockHeader, transactions and uncleHeaders fields. If they are missing, the block is
	invalid and we must verify that we do not accept it.

	Since some tests mix valid and invalid blocks we need to check this for every block.

	If a block is invalid it does not necessarily fail the test, if it's invalidness is
	expected we are expected to ignore it and continue processing and then validate the
	post state.
*/
func (t *BlockTest) insertBlocks(blockchain *core.BlockChain) ([]btBlock, error) {
	validBlocks := make([]btBlock, 0)
	// insert the test blocks, which will execute all transactions
	for bi, b := range t.json.Blocks {
		cb, err := b.decode()
		if err != nil {
			if b.BlockHeader == nil {
				continue // OK - block is supposed to be invalid, continue with next block
			} else {
				return nil, fmt.Errorf("block RLP decoding failed when expected to succeed: %v", err)
			}
		}
		// RLP decoding worked, try to insert into chain:
		blocks := types.Blocks{cb}
		i, err := blockchain.InsertChain(blocks)
		if err != nil {
			if b.BlockHeader == nil {
				continue // OK - block is supposed to be invalid, continue with next block
			} else {
				return nil, fmt.Errorf("block #%v insertion into chain failed: %v", blocks[i].Number(), err)
			}
		}
		if b.BlockHeader == nil {
			if data, err := json.MarshalIndent(cb.Header(), "", "  "); err == nil {
				fmt.Fprintf(os.Stderr, "block (index %d) insertion should have failed due to: %v:\n%v\n",
					bi, b.ExpectException, string(data))
			}
			return nil, fmt.Errorf("block (index %d) insertion should have failed due to: %v",
				bi, b.ExpectException)
		}

		// validate RLP decoding by checking all values against test file JSON
		if err = validateHeader(b.BlockHeader, cb.Header()); err != nil {
			return nil, fmt.Errorf("deserialised block header validation failed: %v", err)
		}
		validBlocks = append(validBlocks, b)
	}
	return validBlocks, nil
}

func validateHeader(h *btHeader, h2 *types.Header) error {
	if h.Bloom != h2.Bloom {
		return fmt.Errorf("bloom: want: %x have: %x", h.Bloom, h2.Bloom)
	}
	if h.Coinbase != h2.Coinbase {
		return fmt.Errorf("coinbase: want: %x have: %x", h.Coinbase, h2.Coinbase)
	}
	if h.MixHash != h2.MixDigest {
		return fmt.Errorf("MixHash: want: %x have: %x", h.MixHash, h2.MixDigest)
	}
	if h.Nonce != h2.Nonce {
		return fmt.Errorf("nonce: want: %x have: %x", h.Nonce, h2.Nonce)
	}
	if h.Number.Cmp(h2.Number) != 0 {
		return fmt.Errorf("number: want: %v have: %v", h.Number, h2.Number)
	}
	if h.ParentHash != h2.ParentHash {
		return fmt.Errorf("parent hash: want: %x have: %x", h.ParentHash, h2.ParentHash)
	}
	if h.ReceiptTrie != h2.ReceiptHash {
		return fmt.Errorf("receipt hash: want: %x have: %x", h.ReceiptTrie, h2.ReceiptHash)
	}
	if h.TransactionsTrie != h2.TxHash {
		return fmt.Errorf("tx hash: want: %x have: %x", h.TransactionsTrie, h2.TxHash)
	}
	if h.StateRoot != h2.Root {
		return fmt.Errorf("state hash: want: %x have: %x", h.StateRoot, h2.Root)
	}
	if h.UncleHash != h2.UncleHash {
		return fmt.Errorf("uncle hash: want: %x have: %x", h.UncleHash, h2.UncleHash)
	}
	if !bytes.Equal(h.ExtraData, h2.Extra) {
		return fmt.Errorf("extra data: want: %x have: %x", h.ExtraData, h2.Extra)
	}
	if h.Difficulty.Cmp(h2.Difficulty) != 0 {
		return fmt.Errorf("difficulty: want: %v have: %v", h.Difficulty, h2.Difficulty)
	}
	if h.GasLimit != h2.GasLimit {
		return fmt.Errorf("gasLimit: want: %d have: %d", h.GasLimit, h2.GasLimit)
	}
	if h.GasUsed != h2.GasUsed {
		return fmt.Errorf("gasUsed: want: %d have: %d", h.GasUsed, h2.GasUsed)
	}
	if h.Timestamp != h2.Time {
		return fmt.Errorf("timestamp: want: %v have: %v", h.Timestamp, h2.Time)
	}
	if !reflect.DeepEqual(h.BaseFeePerGas, h2.BaseFee) {
		return fmt.Errorf("baseFeePerGas: want: %v have: %v", h.BaseFeePerGas, h2.BaseFee)
	}
	if !reflect.DeepEqual(h.WithdrawalsRoot, h2.WithdrawalsHash) {
		return fmt.Errorf("withdrawalsRoot: want: %v have: %v", h.WithdrawalsRoot, h2.WithdrawalsHash)
	}
	if !reflect.DeepEqual(h.BlobGasUsed, h2.BlobGasUsed) {
		return fmt.Errorf("blobGasUsed: want: %v have: %v", h.BlobGasUsed, h2.BlobGasUsed)
	}
	if !reflect.DeepEqual(h.ExcessBlobGas, h2.ExcessBlobGas) {
		return fmt.Errorf("excessBlobGas: want: %v have: %v", h.ExcessBlobGas, h2.ExcessBlobGas)
	}
	if !reflect.DeepEqual(h.ParentBeaconBlockRoot, h2.ParentBeaconRoot) {
		return fmt.Errorf("parentBeaconBlockRoot: want: %v have: %v", h.ParentBeaconBlockRoot, h2.ParentBeaconRoot)
	}
	return nil
}

func (t *BlockTest) validatePostState(statedb *state.StateDB) error {
	// validate post state accounts in test file against what we have in state db
	for addr, acct := range t.json.Post {
		// address is indirectly verified by the other fields, as it's the db key
		code2 := statedb.GetCode(addr)
		balance2 := statedb.GetBalance(addr)
		nonce2 := statedb.GetNonce(addr)
		if !bytes.Equal(code2, acct.Code) {
			return fmt.Errorf("account code mismatch for addr: %s want: %v have: %s", addr, acct.Code, hex.EncodeToString(code2))
		}
		if balance2.Cmp(acct.Balance) != 0 {
			return fmt.Errorf("account balance mismatch for addr: %s, want: %d, have: %d", addr, acct.Balance, balance2)
		}
		if nonce2 != acct.Nonce {
			return fmt.Errorf("account nonce mismatch for addr: %s want: %d have: %d", addr, acct.Nonce, nonce2)
		}
	}
	return nil
}

func (t *BlockTest) validateImportedHeaders(cm *core.BlockChain, validBlocks []btBlock) error {
	// to get constant lookup when verifying block headers by hash (some tests have many blocks)
	bmap := make(map[common.Hash]btBlock, len(t.json.Blocks))
	for _, b := range validBlocks {
		bmap[b.BlockHeader.Hash] = b
	}
	// iterate over blocks backwards from HEAD and validate imported
	// headers vs test file. some tests have reorgs, and we import
	// block-by-block, so we can only validate imported headers after
	// all blocks have been processed by BlockChain, as they may not
	// be part of the longest chain until last block is imported.
	for b := cm.CurrentBlock(); b != nil && b.Number.Uint64() != 0; b = cm.GetBlockByHash(b.ParentHash).Header() {
		if err := validateHeader(bmap[b.Hash()].BlockHeader, b); err != nil {
			return fmt.Errorf("imported block header validation failed: %v", err)
		}
	}
	return nil
}

func (bb *btBlock) decode() (*types.Block, error) {
	data, err := hexutil.Decode(bb.Rlp)
	if err != nil {
		return nil, err
	}
	var b types.Block
	err = rlp.DecodeBytes(data, &b)
	return &b, err
}
