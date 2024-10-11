// Copyright 2023 The go-ethereum Authors
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

package blobpool

import (
	"bytes"
	"crypto/ecdsa"
	"crypto/sha256"
	"errors"
	"math"
	"math/big"
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/consensus/misc/eip4844"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/crypto/kzg4844"
	"github.com/ethereum/go-ethereum/ethdb/memorydb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/holiman/billy"
	"github.com/holiman/uint256"
)

var (
	emptyBlob          = kzg4844.Blob{}
	emptyBlobCommit, _ = kzg4844.BlobToCommitment(emptyBlob)
	emptyBlobProof, _  = kzg4844.ComputeBlobProof(emptyBlob, emptyBlobCommit)
	emptyBlobVHash     = blobHash(emptyBlobCommit)
)

func blobHash(commit kzg4844.Commitment) common.Hash {
	hasher := sha256.New()
	hasher.Write(commit[:])
	hash := hasher.Sum(nil)

	var vhash common.Hash
	vhash[0] = params.BlobTxHashVersion
	copy(vhash[1:], hash[1:])

	return vhash
}

// Chain configuration with Cancun enabled.
//
// TODO(karalabe): replace with params.MainnetChainConfig after Cancun.
var testChainConfig *params.ChainConfig

func init() {
	testChainConfig = new(params.ChainConfig)
	*testChainConfig = *params.MainnetChainConfig

	testChainConfig.CancunTime = new(uint64)
	*testChainConfig.CancunTime = uint64(time.Now().Unix())
}

// testBlockChain is a mock of the live chain for testing the pool.
type testBlockChain struct {
	config  *params.ChainConfig
	basefee *uint256.Int
	blobfee *uint256.Int
	statedb *state.StateDB
}

func (bc *testBlockChain) Config() *params.ChainConfig {
	return bc.config
}

func (bc *testBlockChain) CurrentBlock() *types.Header {
	// Yolo, life is too short to invert mist.CalcBaseFee and misc.CalcBlobFee,
	// just binary search it them.

	// The base fee at 5714 ETH translates into the 21000 base gas higher than
	// mainnet ether existence, use that as a cap for the tests.
	var (
		blockNumber = new(big.Int).Add(bc.config.LondonBlock, big.NewInt(1))
		blockTime   = *bc.config.CancunTime + 1
		gasLimit    = uint64(30_000_000)
	)
	lo := new(big.Int)
	hi := new(big.Int).Mul(big.NewInt(5714), new(big.Int).Exp(big.NewInt(10), big.NewInt(18), nil))

	for new(big.Int).Add(lo, big.NewInt(1)).Cmp(hi) != 0 {
		mid := new(big.Int).Add(lo, hi)
		mid.Div(mid, big.NewInt(2))

		if eip1559.CalcBaseFee(bc.config, &types.Header{
			Number:   blockNumber,
			GasLimit: gasLimit,
			GasUsed:  0,
			BaseFee:  mid,
		}).Cmp(bc.basefee.ToBig()) > 0 {
			hi = mid
		} else {
			lo = mid
		}
	}
	baseFee := lo

	// The excess blob gas at 2^27 translates into a blob fee higher than mainnet
	// ether existence, use that as a cap for the tests.
	lo = new(big.Int)
	hi = new(big.Int).Exp(big.NewInt(2), big.NewInt(27), nil)

	for new(big.Int).Add(lo, big.NewInt(1)).Cmp(hi) != 0 {
		mid := new(big.Int).Add(lo, hi)
		mid.Div(mid, big.NewInt(2))

		if eip4844.CalcBlobFee(mid.Uint64()).Cmp(bc.blobfee.ToBig()) > 0 {
			hi = mid
		} else {
			lo = mid
		}
	}
	excessBlobGas := lo.Uint64()

	return &types.Header{
		Number:        blockNumber,
		Time:          blockTime,
		GasLimit:      gasLimit,
		BaseFee:       baseFee,
		ExcessBlobGas: &excessBlobGas,
	}
}

func (bc *testBlockChain) CurrentFinalBlock() *types.Header {
	return &types.Header{
		Number: big.NewInt(0),
	}
}

func (bt *testBlockChain) GetBlock(hash common.Hash, number uint64) *types.Block {
	return nil
}

func (bc *testBlockChain) StateAt(common.Hash) (*state.StateDB, error) {
	return bc.statedb, nil
}

// makeAddressReserver is a utility method to sanity check that accounts are
// properly reserved by the blobpool (no duplicate reserves or unreserves).
func makeAddressReserver() txpool.AddressReserver {
	var (
		reserved = make(map[common.Address]struct{})
		lock     sync.Mutex
	)
	return func(addr common.Address, reserve bool) error {
		lock.Lock()
		defer lock.Unlock()

		_, exists := reserved[addr]
		if reserve {
			if exists {
				panic("already reserved")
			}
			reserved[addr] = struct{}{}
			return nil
		}
		if !exists {
			panic("not reserved")
		}
		delete(reserved, addr)
		return nil
	}
}

// makeTx is a utility method to construct a random blob transaction and sign it
// with a valid key, only setting the interesting fields from the perspective of
// the blob pool.
func makeTx(nonce uint64, gasTipCap uint64, gasFeeCap uint64, blobFeeCap uint64, key *ecdsa.PrivateKey) *types.Transaction {
	blobtx := makeUnsignedTx(nonce, gasTipCap, gasFeeCap, blobFeeCap)
	return types.MustSignNewTx(key, types.LatestSigner(testChainConfig), blobtx)
}

// makeUnsignedTx is a utility method to construct a random blob tranasaction
// without signing it.
func makeUnsignedTx(nonce uint64, gasTipCap uint64, gasFeeCap uint64, blobFeeCap uint64) *types.BlobTx {
	return &types.BlobTx{
		ChainID:    uint256.MustFromBig(testChainConfig.ChainID),
		Nonce:      nonce,
		GasTipCap:  uint256.NewInt(gasTipCap),
		GasFeeCap:  uint256.NewInt(gasFeeCap),
		Gas:        21000,
		BlobFeeCap: uint256.NewInt(blobFeeCap),
		BlobHashes: []common.Hash{emptyBlobVHash},
		Value:      uint256.NewInt(100),
		Sidecar: &types.BlobTxSidecar{
			Blobs:       []kzg4844.Blob{emptyBlob},
			Commitments: []kzg4844.Commitment{emptyBlobCommit},
			Proofs:      []kzg4844.Proof{emptyBlobProof},
		},
	}
}

// verifyPoolInternals iterates over all the transactions in the pool and checks
// that sort orders, calculated fields, cumulated fields are correct.
func verifyPoolInternals(t *testing.T, pool *BlobPool) {
	// Mark this method as a helper to remove from stack traces
	t.Helper()

	// Verify that all items in the index are present in the lookup and nothing more
	seen := make(map[common.Hash]struct{})
	for addr, txs := range pool.index {
		for _, tx := range txs {
			if _, ok := seen[tx.hash]; ok {
				t.Errorf("duplicate hash #%x in transaction index: address %s, nonce %d", tx.hash, addr, tx.nonce)
			}
			seen[tx.hash] = struct{}{}
		}
	}
	for hash, id := range pool.lookup {
		if _, ok := seen[hash]; !ok {
			t.Errorf("lookup entry missing from transaction index: hash #%x, id %d", hash, id)
		}
		delete(seen, hash)
	}
	for hash := range seen {
		t.Errorf("indexed transaction hash #%x missing from lookup table", hash)
	}
	// Verify that transactions are sorted per account and contain no nonce gaps
	for addr, txs := range pool.index {
		for i := 1; i < len(txs); i++ {
			if txs[i].nonce != txs[i-1].nonce+1 {
				t.Errorf("addr %v, tx %d nonce mismatch: have %d, want %d", addr, i, txs[i].nonce, txs[i-1].nonce+1)
			}
		}
	}
	// Verify that calculated evacuation thresholds are correct
	for addr, txs := range pool.index {
		if !txs[0].evictionExecTip.Eq(txs[0].execTipCap) {
			t.Errorf("addr %v, tx %d eviction execution tip mismatch: have %d, want %d", addr, 0, txs[0].evictionExecTip, txs[0].execTipCap)
		}
		if math.Abs(txs[0].evictionExecFeeJumps-txs[0].basefeeJumps) > 0.001 {
			t.Errorf("addr %v, tx %d eviction execution fee jumps mismatch: have %f, want %f", addr, 0, txs[0].evictionExecFeeJumps, txs[0].basefeeJumps)
		}
		if math.Abs(txs[0].evictionBlobFeeJumps-txs[0].blobfeeJumps) > 0.001 {
			t.Errorf("addr %v, tx %d eviction blob fee jumps mismatch: have %f, want %f", addr, 0, txs[0].evictionBlobFeeJumps, txs[0].blobfeeJumps)
		}
		for i := 1; i < len(txs); i++ {
			wantExecTip := txs[i-1].evictionExecTip
			if wantExecTip.Gt(txs[i].execTipCap) {
				wantExecTip = txs[i].execTipCap
			}
			if !txs[i].evictionExecTip.Eq(wantExecTip) {
				t.Errorf("addr %v, tx %d eviction execution tip mismatch: have %d, want %d", addr, i, txs[i].evictionExecTip, wantExecTip)
			}

			wantExecFeeJumps := txs[i-1].evictionExecFeeJumps
			if wantExecFeeJumps > txs[i].basefeeJumps {
				wantExecFeeJumps = txs[i].basefeeJumps
			}
			if math.Abs(txs[i].evictionExecFeeJumps-wantExecFeeJumps) > 0.001 {
				t.Errorf("addr %v, tx %d eviction execution fee jumps mismatch: have %f, want %f", addr, i, txs[i].evictionExecFeeJumps, wantExecFeeJumps)
			}

			wantBlobFeeJumps := txs[i-1].evictionBlobFeeJumps
			if wantBlobFeeJumps > txs[i].blobfeeJumps {
				wantBlobFeeJumps = txs[i].blobfeeJumps
			}
			if math.Abs(txs[i].evictionBlobFeeJumps-wantBlobFeeJumps) > 0.001 {
				t.Errorf("addr %v, tx %d eviction blob fee jumps mismatch: have %f, want %f", addr, i, txs[i].evictionBlobFeeJumps, wantBlobFeeJumps)
			}
		}
	}
	// Verify that account balance accumulations are correct
	for addr, txs := range pool.index {
		spent := new(uint256.Int)
		for _, tx := range txs {
			spent.Add(spent, tx.costCap)
		}
		if !pool.spent[addr].Eq(spent) {
			t.Errorf("addr %v expenditure mismatch: have %d, want %d", addr, pool.spent[addr], spent)
		}
	}
	// Verify that pool storage size is correct
	var stored uint64
	for _, txs := range pool.index {
		for _, tx := range txs {
			stored += uint64(tx.size)
		}
	}
	if pool.stored != stored {
		t.Errorf("pool storage mismatch: have %d, want %d", pool.stored, stored)
	}
	// Verify the price heap internals
	verifyHeapInternals(t, pool.evict)
}

// Tests that transactions can be loaded from disk on startup and that they are
// correctly discarded if invalid.
//
//   - 1. A transaction that cannot be decoded must be dropped
//   - 2. A transaction that cannot be recovered (bad signature) must be dropped
//   - 3. All transactions after a nonce gap must be dropped
//   - 4. All transactions after an underpriced one (including it) must be dropped
func TestOpenDrops(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Create a temporary folder for the persistent backend
	storage, _ := os.MkdirTemp("", "blobpool-")
	defer os.RemoveAll(storage)

	os.MkdirAll(filepath.Join(storage, pendingTransactionStore), 0700)
	store, _ := billy.Open(billy.Options{Path: filepath.Join(storage, pendingTransactionStore)}, newSlotter(), nil)

	// Insert a malformed transaction to verify that decoding errors (or format
	// changes) are handled gracefully (case 1)
	malformed, _ := store.Put([]byte("this is a badly encoded transaction"))

	// Insert a transaction with a bad signature to verify that stale junk after
	// potential hard-forks can get evicted (case 2)
	tx := types.NewTx(&types.BlobTx{
		ChainID:    uint256.MustFromBig(testChainConfig.ChainID),
		GasTipCap:  new(uint256.Int),
		GasFeeCap:  new(uint256.Int),
		Gas:        0,
		Value:      new(uint256.Int),
		Data:       nil,
		BlobFeeCap: new(uint256.Int),
		V:          new(uint256.Int),
		R:          new(uint256.Int),
		S:          new(uint256.Int),
	})
	blob, _ := rlp.EncodeToBytes(tx)
	badsig, _ := store.Put(blob)

	// Insert a sequence of transactions with a nonce gap in between to verify
	// that anything gapped will get evicted (case 3)
	var (
		gapper, _ = crypto.GenerateKey()

		valids = make(map[uint64]struct{})
		gapped = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{0, 1, 3, 4, 6, 7} { // first gap at #2, another at #5
		tx := makeTx(nonce, 1, 1, 1, gapper)
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		if nonce < 2 {
			valids[id] = struct{}{}
		} else {
			gapped[id] = struct{}{}
		}
	}
	// Insert a sequence of transactions with a gapped starting nonce to verify
	// that the entire set will get dropped.
	var (
		dangler, _ = crypto.GenerateKey()
		dangling   = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{1, 2, 3} { // first gap at #0, all set dangling
		tx := makeTx(nonce, 1, 1, 1, dangler)
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		dangling[id] = struct{}{}
	}
	// Insert a sequence of transactions with already passed nonces to veirfy
	// that the entire set will get dropped.
	var (
		filler, _ = crypto.GenerateKey()
		filled    = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{0, 1, 2} { // account nonce at 3, all set filled
		tx := makeTx(nonce, 1, 1, 1, filler)
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		filled[id] = struct{}{}
	}
	// Insert a sequence of transactions with partially passed nonces to veirfy
	// that the included part of the set will get dropped
	var (
		overlapper, _ = crypto.GenerateKey()
		overlapped    = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{0, 1, 2, 3} { // account nonce at 2, half filled
		tx := makeTx(nonce, 1, 1, 1, overlapper)
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		if nonce >= 2 {
			valids[id] = struct{}{}
		} else {
			overlapped[id] = struct{}{}
		}
	}
	// Insert a sequence of transactions with an underpriced first to verify that
	// the entire set will get dropped (case 4).
	var (
		underpayer, _ = crypto.GenerateKey()
		underpaid     = make(map[uint64]struct{})
	)
	for i := 0; i < 5; i++ { // make #0 underpriced
		var tx *types.Transaction
		if i == 0 {
			tx = makeTx(uint64(i), 0, 0, 0, underpayer)
		} else {
			tx = makeTx(uint64(i), 1, 1, 1, underpayer)
		}
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		underpaid[id] = struct{}{}
	}

	// Insert a sequence of transactions with an underpriced in between to verify
	// that it and anything newly gapped will get evicted (case 4).
	var (
		outpricer, _ = crypto.GenerateKey()
		outpriced    = make(map[uint64]struct{})
	)
	for i := 0; i < 5; i++ { // make #2 underpriced
		var tx *types.Transaction
		if i == 2 {
			tx = makeTx(uint64(i), 0, 0, 0, outpricer)
		} else {
			tx = makeTx(uint64(i), 1, 1, 1, outpricer)
		}
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		if i < 2 {
			valids[id] = struct{}{}
		} else {
			outpriced[id] = struct{}{}
		}
	}
	// Insert a sequence of transactions fully overdrafted to verify that the
	// entire set will get invalidated.
	var (
		exceeder, _ = crypto.GenerateKey()
		exceeded    = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{0, 1, 2} { // nonce 0 overdrafts the account
		var tx *types.Transaction
		if nonce == 0 {
			tx = makeTx(nonce, 1, 100, 1, exceeder)
		} else {
			tx = makeTx(nonce, 1, 1, 1, exceeder)
		}
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		exceeded[id] = struct{}{}
	}
	// Insert a sequence of transactions partially overdrafted to verify that part
	// of the set will get invalidated.
	var (
		overdrafter, _ = crypto.GenerateKey()
		overdrafted    = make(map[uint64]struct{})
	)
	for _, nonce := range []uint64{0, 1, 2} { // nonce 1 overdrafts the account
		var tx *types.Transaction
		if nonce == 1 {
			tx = makeTx(nonce, 1, 100, 1, overdrafter)
		} else {
			tx = makeTx(nonce, 1, 1, 1, overdrafter)
		}
		blob, _ := rlp.EncodeToBytes(tx)

		id, _ := store.Put(blob)
		if nonce < 1 {
			valids[id] = struct{}{}
		} else {
			overdrafted[id] = struct{}{}
		}
	}
	// Insert a sequence of transactions overflowing the account cap to verify
	// that part of the set will get invalidated.
	var (
		overcapper, _ = crypto.GenerateKey()
		overcapped    = make(map[uint64]struct{})
	)
	for nonce := uint64(0); nonce < maxTxsPerAccount+3; nonce++ {
		blob, _ := rlp.EncodeToBytes(makeTx(nonce, 1, 1, 1, overcapper))

		id, _ := store.Put(blob)
		if nonce < maxTxsPerAccount {
			valids[id] = struct{}{}
		} else {
			overcapped[id] = struct{}{}
		}
	}
	store.Close()

	// Create a blob pool out of the pre-seeded data
	statedb, _ := state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewDatabase(memorydb.New())), nil)
	statedb.AddBalance(crypto.PubkeyToAddress(gapper.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(dangler.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(filler.PublicKey), big.NewInt(1000000))
	statedb.SetNonce(crypto.PubkeyToAddress(filler.PublicKey), 3)
	statedb.AddBalance(crypto.PubkeyToAddress(overlapper.PublicKey), big.NewInt(1000000))
	statedb.SetNonce(crypto.PubkeyToAddress(overlapper.PublicKey), 2)
	statedb.AddBalance(crypto.PubkeyToAddress(underpayer.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(outpricer.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(exceeder.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(overdrafter.PublicKey), big.NewInt(1000000))
	statedb.AddBalance(crypto.PubkeyToAddress(overcapper.PublicKey), big.NewInt(10000000))
	statedb.Commit(0, true)

	chain := &testBlockChain{
		config:  testChainConfig,
		basefee: uint256.NewInt(params.InitialBaseFee),
		blobfee: uint256.NewInt(params.BlobTxMinBlobGasprice),
		statedb: statedb,
	}
	pool := New(Config{Datadir: storage}, chain)
	if err := pool.Init(big.NewInt(1), chain.CurrentBlock(), makeAddressReserver()); err != nil {
		t.Fatalf("failed to create blob pool: %v", err)
	}
	defer pool.Close()

	// Verify that the malformed (case 1), badly signed (case 2) and gapped (case
	// 3) txs have been deleted from the pool
	alive := make(map[uint64]struct{})
	for _, txs := range pool.index {
		for _, tx := range txs {
			switch tx.id {
			case malformed:
				t.Errorf("malformed RLP transaction remained in storage")
			case badsig:
				t.Errorf("invalidly signed transaction remained in storage")
			default:
				if _, ok := dangling[tx.id]; ok {
					t.Errorf("dangling transaction remained in storage: %d", tx.id)
				} else if _, ok := filled[tx.id]; ok {
					t.Errorf("filled transaction remained in storage: %d", tx.id)
				} else if _, ok := overlapped[tx.id]; ok {
					t.Errorf("overlapped transaction remained in storage: %d", tx.id)
				} else if _, ok := gapped[tx.id]; ok {
					t.Errorf("gapped transaction remained in storage: %d", tx.id)
				} else if _, ok := underpaid[tx.id]; ok {
					t.Errorf("underpaid transaction remained in storage: %d", tx.id)
				} else if _, ok := outpriced[tx.id]; ok {
					t.Errorf("outpriced transaction remained in storage: %d", tx.id)
				} else if _, ok := exceeded[tx.id]; ok {
					t.Errorf("fully overdrafted transaction remained in storage: %d", tx.id)
				} else if _, ok := overdrafted[tx.id]; ok {
					t.Errorf("partially overdrafted transaction remained in storage: %d", tx.id)
				} else if _, ok := overcapped[tx.id]; ok {
					t.Errorf("overcapped transaction remained in storage: %d", tx.id)
				} else {
					alive[tx.id] = struct{}{}
				}
			}
		}
	}
	// Verify that the rest of the transactions remained alive
	if len(alive) != len(valids) {
		t.Errorf("valid transaction count mismatch: have %d, want %d", len(alive), len(valids))
	}
	for id := range alive {
		if _, ok := valids[id]; !ok {
			t.Errorf("extra transaction %d", id)
		}
	}
	for id := range valids {
		if _, ok := alive[id]; !ok {
			t.Errorf("missing transaction %d", id)
		}
	}
	// Verify all the calculated pool internals. Interestingly, this is **not**
	// a duplication of the above checks, this actually validates the verifier
	// using the above already hard coded checks.
	//
	// Do not remove this, nor alter the above to be generic.
	verifyPoolInternals(t, pool)
}

// Tests that transactions loaded from disk are indexed corrently.
//
//   - 1. Transactions must be groupped by sender, sorted by nonce
//   - 2. Eviction thresholds are calculated correctly for the sequences
//   - 3. Balance usage of an account is totals across all transactions
func TestOpenIndex(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Create a temporary folder for the persistent backend
	storage, _ := os.MkdirTemp("", "blobpool-")
	defer os.RemoveAll(storage)

	os.MkdirAll(filepath.Join(storage, pendingTransactionStore), 0700)
	store, _ := billy.Open(billy.Options{Path: filepath.Join(storage, pendingTransactionStore)}, newSlotter(), nil)

	// Insert a sequence of transactions with varying price points to check that
	// the cumulative minimumw will be maintained.
	var (
		key, _ = crypto.GenerateKey()
		addr   = crypto.PubkeyToAddress(key.PublicKey)

		txExecTipCaps = []uint64{10, 25, 5, 7, 1, 100}
		txExecFeeCaps = []uint64{100, 90, 200, 10, 80, 300}
		txBlobFeeCaps = []uint64{55, 66, 77, 33, 22, 11}

		//basefeeJumps = []float64{39.098, 38.204, 44.983, 19.549, 37.204, 48.426} // log 1.125 (exec fee cap)
		//blobfeeJumps = []float64{34.023, 35.570, 36.879, 29.686, 26.243, 20.358} // log 1.125 (blob fee cap)

		evictExecTipCaps  = []uint64{10, 10, 5, 5, 1, 1}
		evictExecFeeJumps = []float64{39.098, 38.204, 38.204, 19.549, 19.549, 19.549} //  min(log 1.125 (exec fee cap))
		evictBlobFeeJumps = []float64{34.023, 34.023, 34.023, 29.686, 26.243, 20.358} // min(log 1.125 (blob fee cap))

		totalSpent = uint256.NewInt(21000*(100+90+200+10+80+300) + blobSize*(55+66+77+33+22+11) + 100*6) // 21000 gas x price + 128KB x blobprice + value
	)
	for _, i := range []int{5, 3, 4, 2, 0, 1} { // Randomize the tx insertion order to force sorting on load
		tx := makeTx(uint64(i), txExecTipCaps[i], txExecFeeCaps[i], txBlobFeeCaps[i], key)
		blob, _ := rlp.EncodeToBytes(tx)
		store.Put(blob)
	}
	store.Close()

	// Create a blob pool out of the pre-seeded data
	statedb, _ := state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewDatabase(memorydb.New())), nil)
	statedb.AddBalance(addr, big.NewInt(1_000_000_000))
	statedb.Commit(0, true)

	chain := &testBlockChain{
		config:  testChainConfig,
		basefee: uint256.NewInt(params.InitialBaseFee),
		blobfee: uint256.NewInt(params.BlobTxMinBlobGasprice),
		statedb: statedb,
	}
	pool := New(Config{Datadir: storage}, chain)
	if err := pool.Init(big.NewInt(1), chain.CurrentBlock(), makeAddressReserver()); err != nil {
		t.Fatalf("failed to create blob pool: %v", err)
	}
	defer pool.Close()

	// Verify that the transactions have been sorted by nonce (case 1)
	for i := 0; i < len(pool.index[addr]); i++ {
		if pool.index[addr][i].nonce != uint64(i) {
			t.Errorf("tx %d nonce mismatch: have %d, want %d", i, pool.index[addr][i].nonce, uint64(i))
		}
	}
	// Verify that the cumulative fee minimums have been correctly calculated (case 2)
	for i, cap := range evictExecTipCaps {
		if !pool.index[addr][i].evictionExecTip.Eq(uint256.NewInt(cap)) {
			t.Errorf("eviction tip cap %d mismatch: have %d, want %d", i, pool.index[addr][i].evictionExecTip, cap)
		}
	}
	for i, jumps := range evictExecFeeJumps {
		if math.Abs(pool.index[addr][i].evictionExecFeeJumps-jumps) > 0.001 {
			t.Errorf("eviction fee cap jumps %d mismatch: have %f, want %f", i, pool.index[addr][i].evictionExecFeeJumps, jumps)
		}
	}
	for i, jumps := range evictBlobFeeJumps {
		if math.Abs(pool.index[addr][i].evictionBlobFeeJumps-jumps) > 0.001 {
			t.Errorf("eviction blob fee cap jumps %d mismatch: have %f, want %f", i, pool.index[addr][i].evictionBlobFeeJumps, jumps)
		}
	}
	// Verify that the balance usage has been correctly calculated (case 3)
	if !pool.spent[addr].Eq(totalSpent) {
		t.Errorf("expenditure mismatch: have %d, want %d", pool.spent[addr], totalSpent)
	}
	// Verify all the calculated pool internals. Interestingly, this is **not**
	// a duplication of the above checks, this actually validates the verifier
	// using the above already hard coded checks.
	//
	// Do not remove this, nor alter the above to be generic.
	verifyPoolInternals(t, pool)
}

// Tests that after indexing all the loaded transactions from disk, a price heap
// is correctly constructed based on the head basefee and blobfee.
func TestOpenHeap(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Create a temporary folder for the persistent backend
	storage, _ := os.MkdirTemp("", "blobpool-")
	defer os.RemoveAll(storage)

	os.MkdirAll(filepath.Join(storage, pendingTransactionStore), 0700)
	store, _ := billy.Open(billy.Options{Path: filepath.Join(storage, pendingTransactionStore)}, newSlotter(), nil)

	// Insert a few transactions from a few accounts. To remove randomness from
	// the heap initialization, use a deterministic account/tx/priority ordering.
	var (
		key1, _ = crypto.GenerateKey()
		key2, _ = crypto.GenerateKey()
		key3, _ = crypto.GenerateKey()

		addr1 = crypto.PubkeyToAddress(key1.PublicKey)
		addr2 = crypto.PubkeyToAddress(key2.PublicKey)
		addr3 = crypto.PubkeyToAddress(key3.PublicKey)
	)
	if bytes.Compare(addr1[:], addr2[:]) > 0 {
		key1, addr1, key2, addr2 = key2, addr2, key1, addr1
	}
	if bytes.Compare(addr1[:], addr3[:]) > 0 {
		key1, addr1, key3, addr3 = key3, addr3, key1, addr1
	}
	if bytes.Compare(addr2[:], addr3[:]) > 0 {
		key2, addr2, key3, addr3 = key3, addr3, key2, addr2
	}
	var (
		tx1 = makeTx(0, 1, 1000, 90, key1)
		tx2 = makeTx(0, 1, 800, 70, key2)
		tx3 = makeTx(0, 1, 1500, 110, key3)

		blob1, _ = rlp.EncodeToBytes(tx1)
		blob2, _ = rlp.EncodeToBytes(tx2)
		blob3, _ = rlp.EncodeToBytes(tx3)

		heapOrder = []common.Address{addr2, addr1, addr3}
		heapIndex = map[common.Address]int{addr2: 0, addr1: 1, addr3: 2}
	)
	store.Put(blob1)
	store.Put(blob2)
	store.Put(blob3)
	store.Close()

	// Create a blob pool out of the pre-seeded data
	statedb, _ := state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewDatabase(memorydb.New())), nil)
	statedb.AddBalance(addr1, big.NewInt(1_000_000_000))
	statedb.AddBalance(addr2, big.NewInt(1_000_000_000))
	statedb.AddBalance(addr3, big.NewInt(1_000_000_000))
	statedb.Commit(0, true)

	chain := &testBlockChain{
		config:  testChainConfig,
		basefee: uint256.NewInt(1050),
		blobfee: uint256.NewInt(105),
		statedb: statedb,
	}
	pool := New(Config{Datadir: storage}, chain)
	if err := pool.Init(big.NewInt(1), chain.CurrentBlock(), makeAddressReserver()); err != nil {
		t.Fatalf("failed to create blob pool: %v", err)
	}
	defer pool.Close()

	// Verify that the heap's internal state matches the expectations
	for i, addr := range pool.evict.addrs {
		if addr != heapOrder[i] {
			t.Errorf("slot %d mismatch: have %v, want %v", i, addr, heapOrder[i])
		}
	}
	for addr, i := range pool.evict.index {
		if i != heapIndex[addr] {
			t.Errorf("index for %v mismatch: have %d, want %d", addr, i, heapIndex[addr])
		}
	}
	// Verify all the calculated pool internals. Interestingly, this is **not**
	// a duplication of the above checks, this actually validates the verifier
	// using the above already hard coded checks.
	//
	// Do not remove this, nor alter the above to be generic.
	verifyPoolInternals(t, pool)
}

// Tests that after the pool's previous state is loaded back, any transactions
// over the new storage cap will get dropped.
func TestOpenCap(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Create a temporary folder for the persistent backend
	storage, _ := os.MkdirTemp("", "blobpool-")
	defer os.RemoveAll(storage)

	os.MkdirAll(filepath.Join(storage, pendingTransactionStore), 0700)
	store, _ := billy.Open(billy.Options{Path: filepath.Join(storage, pendingTransactionStore)}, newSlotter(), nil)

	// Insert a few transactions from a few accounts
	var (
		key1, _ = crypto.GenerateKey()
		key2, _ = crypto.GenerateKey()
		key3, _ = crypto.GenerateKey()

		addr1 = crypto.PubkeyToAddress(key1.PublicKey)
		addr2 = crypto.PubkeyToAddress(key2.PublicKey)
		addr3 = crypto.PubkeyToAddress(key3.PublicKey)

		tx1 = makeTx(0, 1, 1000, 100, key1)
		tx2 = makeTx(0, 1, 800, 70, key2)
		tx3 = makeTx(0, 1, 1500, 110, key3)

		blob1, _ = rlp.EncodeToBytes(tx1)
		blob2, _ = rlp.EncodeToBytes(tx2)
		blob3, _ = rlp.EncodeToBytes(tx3)

		keep = []common.Address{addr1, addr3}
		drop = []common.Address{addr2}
		size = uint64(2 * (txAvgSize + blobSize))
	)
	store.Put(blob1)
	store.Put(blob2)
	store.Put(blob3)
	store.Close()

	// Verify pool capping twice: first by reducing the data cap, then restarting
	// with a high cap to ensure everything was persisted previously
	for _, datacap := range []uint64{2 * (txAvgSize + blobSize), 100 * (txAvgSize + blobSize)} {
		// Create a blob pool out of the pre-seeded data, but cap it to 2 blob transaction
		statedb, _ := state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewDatabase(memorydb.New())), nil)
		statedb.AddBalance(addr1, big.NewInt(1_000_000_000))
		statedb.AddBalance(addr2, big.NewInt(1_000_000_000))
		statedb.AddBalance(addr3, big.NewInt(1_000_000_000))
		statedb.Commit(0, true)

		chain := &testBlockChain{
			config:  testChainConfig,
			basefee: uint256.NewInt(1050),
			blobfee: uint256.NewInt(105),
			statedb: statedb,
		}
		pool := New(Config{Datadir: storage, Datacap: datacap}, chain)
		if err := pool.Init(big.NewInt(1), chain.CurrentBlock(), makeAddressReserver()); err != nil {
			t.Fatalf("failed to create blob pool: %v", err)
		}
		// Verify that enough transactions have been dropped to get the pool's size
		// under the requested limit
		if len(pool.index) != len(keep) {
			t.Errorf("tracked account count mismatch: have %d, want %d", len(pool.index), len(keep))
		}
		for _, addr := range keep {
			if _, ok := pool.index[addr]; !ok {
				t.Errorf("expected account %v missing from pool", addr)
			}
		}
		for _, addr := range drop {
			if _, ok := pool.index[addr]; ok {
				t.Errorf("unexpected account %v present in pool", addr)
			}
		}
		if pool.stored != size {
			t.Errorf("pool stored size mismatch: have %v, want %v", pool.stored, size)
		}
		// Verify all the calculated pool internals. Interestingly, this is **not**
		// a duplication of the above checks, this actually validates the verifier
		// using the above already hard coded checks.
		//
		// Do not remove this, nor alter the above to be generic.
		verifyPoolInternals(t, pool)

		pool.Close()
	}
}

// Tests that adding transaction will correctly store it in the persistent store
// and update all the indices.
//
// Note, this tests mostly checks the pool transaction shuffling logic or things
// specific to the blob pool. It does not do an exhaustive transaction validity
// check.
func TestAdd(t *testing.T) {
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// seed is a helper tumpe to seed an initial state db and pool
	type seed struct {
		balance uint64
		nonce   uint64
		txs     []*types.BlobTx
	}

	// addtx is a helper sender/tx tuple to represent a new tx addition
	type addtx struct {
		from string
		tx   *types.BlobTx
		err  error
	}

	tests := []struct {
		seeds map[string]seed
		adds  []addtx
	}{
		// Transactions from new accounts should be accepted if their initial
		// nonce matches the expected one from the statedb. Higher or lower must
		// be rejected.
		{
			seeds: map[string]seed{
				"alice":  {balance: 21100 + blobSize},
				"bob":    {balance: 21100 + blobSize, nonce: 1},
				"claire": {balance: 21100 + blobSize},
				"dave":   {balance: 21100 + blobSize, nonce: 1},
			},
			adds: []addtx{
				{ // New account, no previous txs: accept nonce 0
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 1, 1),
					err:  nil,
				},
				{ // Old account, 1 tx in chain, 0 pending: accept nonce 1
					from: "bob",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  nil,
				},
				{ // New account, no previous txs: reject nonce 1
					from: "claire",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  core.ErrNonceTooHigh,
				},
				{ // Old account, 1 tx in chain, 0 pending: reject nonce 0
					from: "dave",
					tx:   makeUnsignedTx(0, 1, 1, 1),
					err:  core.ErrNonceTooLow,
				},
				{ // Old account, 1 tx in chain, 0 pending: reject nonce 2
					from: "dave",
					tx:   makeUnsignedTx(2, 1, 1, 1),
					err:  core.ErrNonceTooHigh,
				},
			},
		},
		// Transactions from already pooled accounts should only be accepted if
		// the nonces are contiguous (ignore prices for now, will check later)
		{
			seeds: map[string]seed{
				"alice": {
					balance: 1000000,
					txs: []*types.BlobTx{
						makeUnsignedTx(0, 1, 1, 1),
					},
				},
				"bob": {
					balance: 1000000,
					nonce:   1,
					txs: []*types.BlobTx{
						makeUnsignedTx(1, 1, 1, 1),
					},
				},
			},
			adds: []addtx{
				{ // New account, 1 tx pending: reject replacement nonce 0 (ignore price for now)
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 1, 1),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 1 tx pending: accept nonce 1
					from: "alice",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 2 txs pending: reject nonce 3
					from: "alice",
					tx:   makeUnsignedTx(3, 1, 1, 1),
					err:  core.ErrNonceTooHigh,
				},
				{ // New account, 2 txs pending: accept nonce 2
					from: "alice",
					tx:   makeUnsignedTx(2, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 3 txs pending: accept nonce 3 now
					from: "alice",
					tx:   makeUnsignedTx(3, 1, 1, 1),
					err:  nil,
				},
				{ // Old account, 1 tx in chain, 1 tx pending: reject replacement nonce 1 (ignore price for now)
					from: "bob",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // Old account, 1 tx in chain, 1 tx pending: accept nonce 2 (ignore price for now)
					from: "bob",
					tx:   makeUnsignedTx(2, 1, 1, 1),
					err:  nil,
				},
			},
		},
		// Transactions should only be accepted into the pool if the cumulative
		// expenditure doesn't overflow the account balance
		{
			seeds: map[string]seed{
				"alice": {balance: 63299 + 3*blobSize}, // 3 tx - 1 wei
			},
			adds: []addtx{
				{ // New account, no previous txs: accept nonce 0 with 21100 wei spend
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 1 pooled tx with 21100 wei spent: accept nonce 1 with 21100 wei spend
					from: "alice",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 2 pooled tx with 42200 wei spent: reject nonce 2 with 21100 wei spend (1 wei overflow)
					from: "alice",
					tx:   makeUnsignedTx(2, 1, 1, 1),
					err:  core.ErrInsufficientFunds,
				},
			},
		},
		// Transactions should only be accepted into the pool if the total count
		// from the same account doesn't overflow the pool limits
		{
			seeds: map[string]seed{
				"alice": {balance: 10000000},
			},
			adds: []addtx{
				{ // New account, no previous txs, 16 slots left: accept nonce 0
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 1 pooled tx, 15 slots left: accept nonce 1
					from: "alice",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 2 pooled tx, 14 slots left: accept nonce 2
					from: "alice",
					tx:   makeUnsignedTx(2, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 3 pooled tx, 13 slots left: accept nonce 3
					from: "alice",
					tx:   makeUnsignedTx(3, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 4 pooled tx, 12 slots left: accept nonce 4
					from: "alice",
					tx:   makeUnsignedTx(4, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 5 pooled tx, 11 slots left: accept nonce 5
					from: "alice",
					tx:   makeUnsignedTx(5, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 6 pooled tx, 10 slots left: accept nonce 6
					from: "alice",
					tx:   makeUnsignedTx(6, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 7 pooled tx, 9 slots left: accept nonce 7
					from: "alice",
					tx:   makeUnsignedTx(7, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 8 pooled tx, 8 slots left: accept nonce 8
					from: "alice",
					tx:   makeUnsignedTx(8, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 9 pooled tx, 7 slots left: accept nonce 9
					from: "alice",
					tx:   makeUnsignedTx(9, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 10 pooled tx, 6 slots left: accept nonce 10
					from: "alice",
					tx:   makeUnsignedTx(10, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 11 pooled tx, 5 slots left: accept nonce 11
					from: "alice",
					tx:   makeUnsignedTx(11, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 12 pooled tx, 4 slots left: accept nonce 12
					from: "alice",
					tx:   makeUnsignedTx(12, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 13 pooled tx, 3 slots left: accept nonce 13
					from: "alice",
					tx:   makeUnsignedTx(13, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 14 pooled tx, 2 slots left: accept nonce 14
					from: "alice",
					tx:   makeUnsignedTx(14, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 15 pooled tx, 1 slots left: accept nonce 15
					from: "alice",
					tx:   makeUnsignedTx(15, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 16 pooled tx, 0 slots left: accept nonce 15 replacement
					from: "alice",
					tx:   makeUnsignedTx(15, 10, 10, 10),
					err:  nil,
				},
				{ // New account, 16 pooled tx, 0 slots left: reject nonce 16 with overcap
					from: "alice",
					tx:   makeUnsignedTx(16, 1, 1, 1),
					err:  txpool.ErrAccountLimitExceeded,
				},
			},
		},
		// Previously existing transactions should be allowed to be replaced iff
		// the new cumulative expenditure can be covered by the account and the
		// prices are bumped all around (no percentage check here).
		{
			seeds: map[string]seed{
				"alice": {balance: 2*100 + 5*21000 + 3*blobSize},
			},
			adds: []addtx{
				{ // New account, no previous txs: reject nonce 0 with 341172 wei spend
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 20, 1),
					err:  core.ErrInsufficientFunds,
				},
				{ // New account, no previous txs: accept nonce 0 with 173172 wei spend
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 2, 1),
					err:  nil,
				},
				{ // New account, 1 pooled tx with 173172 wei spent: accept nonce 1 with 152172 wei spend
					from: "alice",
					tx:   makeUnsignedTx(1, 1, 1, 1),
					err:  nil,
				},
				{ // New account, 2 pooled tx with 325344 wei spent: reject nonce 0 with 599684 wei spend (173072 extra) (would overflow balance at nonce 1)
					from: "alice",
					tx:   makeUnsignedTx(0, 2, 5, 2),
					err:  core.ErrInsufficientFunds,
				},
				{ // New account, 2 pooled tx with 325344 wei spent: reject nonce 0 with no-gastip-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 1, 3, 2),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 2 pooled tx with 325344 wei spent: reject nonce 0 with no-gascap-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 2, 2, 2),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 2 pooled tx with 325344 wei spent: reject nonce 0 with no-blobcap-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 2, 4, 1),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 2 pooled tx with 325344 wei spent: accept nonce 0 with 84100 wei spend (42000 extra)
					from: "alice",
					tx:   makeUnsignedTx(0, 2, 4, 2),
					err:  nil,
				},
			},
		},
		// Previously existing transactions should be allowed to be replaced iff
		// the new prices are bumped by a sufficient amount.
		{
			seeds: map[string]seed{
				"alice": {balance: 100 + 8*21000 + 4*blobSize},
			},
			adds: []addtx{
				{ // New account, no previous txs: accept nonce 0
					from: "alice",
					tx:   makeUnsignedTx(0, 2, 4, 2),
					err:  nil,
				},
				{ // New account, 1 pooled tx: reject nonce 0 with low-gastip-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 3, 8, 4),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 1 pooled tx: reject nonce 0 with low-gascap-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 4, 6, 4),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 1 pooled tx: reject nonce 0 with low-blobcap-bump
					from: "alice",
					tx:   makeUnsignedTx(0, 4, 8, 3),
					err:  txpool.ErrReplaceUnderpriced,
				},
				{ // New account, 1 pooled tx: accept nonce 0 with all-bumps
					from: "alice",
					tx:   makeUnsignedTx(0, 4, 8, 4),
					err:  nil,
				},
			},
		},
	}
	for i, tt := range tests {
		// Create a temporary folder for the persistent backend
		storage, _ := os.MkdirTemp("", "blobpool-")
		defer os.RemoveAll(storage) // late defer, still ok

		os.MkdirAll(filepath.Join(storage, pendingTransactionStore), 0700)
		store, _ := billy.Open(billy.Options{Path: filepath.Join(storage, pendingTransactionStore)}, newSlotter(), nil)

		// Insert the seed transactions for the pool startup
		var (
			keys  = make(map[string]*ecdsa.PrivateKey)
			addrs = make(map[string]common.Address)
		)
		statedb, _ := state.New(types.EmptyRootHash, state.NewDatabase(rawdb.NewDatabase(memorydb.New())), nil)
		for acc, seed := range tt.seeds {
			// Generate a new random key/address for the seed account
			keys[acc], _ = crypto.GenerateKey()
			addrs[acc] = crypto.PubkeyToAddress(keys[acc].PublicKey)

			// Seed the state database with this acocunt
			statedb.AddBalance(addrs[acc], new(big.Int).SetUint64(seed.balance))
			statedb.SetNonce(addrs[acc], seed.nonce)

			// Sign the seed transactions and store them in the data store
			for _, tx := range seed.txs {
				signed := types.MustSignNewTx(keys[acc], types.LatestSigner(testChainConfig), tx)
				blob, _ := rlp.EncodeToBytes(signed)
				store.Put(blob)
			}
		}
		statedb.Commit(0, true)
		store.Close()

		// Create a blob pool out of the pre-seeded dats
		chain := &testBlockChain{
			config:  testChainConfig,
			basefee: uint256.NewInt(1050),
			blobfee: uint256.NewInt(105),
			statedb: statedb,
		}
		pool := New(Config{Datadir: storage}, chain)
		if err := pool.Init(big.NewInt(1), chain.CurrentBlock(), makeAddressReserver()); err != nil {
			t.Fatalf("test %d: failed to create blob pool: %v", i, err)
		}
		verifyPoolInternals(t, pool)

		// Add each transaction one by one, verifying the pool internals in between
		for j, add := range tt.adds {
			signed, _ := types.SignNewTx(keys[add.from], types.LatestSigner(testChainConfig), add.tx)
			if err := pool.add(signed); !errors.Is(err, add.err) {
				t.Errorf("test %d, tx %d: adding transaction error mismatch: have %v, want %v", i, j, err, add.err)
			}
			verifyPoolInternals(t, pool)
		}
		// Verify the pool internals and close down the test
		verifyPoolInternals(t, pool)
		pool.Close()
	}
}
