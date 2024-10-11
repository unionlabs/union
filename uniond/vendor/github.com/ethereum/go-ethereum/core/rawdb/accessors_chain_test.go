// Copyright 2018 The go-ethereum Authors
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

package rawdb

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"math/big"
	"math/rand"
	"os"
	"reflect"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"golang.org/x/crypto/sha3"
)

// Tests block header storage and retrieval operations.
func TestHeaderStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test header to move around the database and make sure it's really new
	header := &types.Header{Number: big.NewInt(42), Extra: []byte("test header")}
	if entry := ReadHeader(db, header.Hash(), header.Number.Uint64()); entry != nil {
		t.Fatalf("Non existent header returned: %v", entry)
	}
	// Write and verify the header in the database
	WriteHeader(db, header)
	if entry := ReadHeader(db, header.Hash(), header.Number.Uint64()); entry == nil {
		t.Fatalf("Stored header not found")
	} else if entry.Hash() != header.Hash() {
		t.Fatalf("Retrieved header mismatch: have %v, want %v", entry, header)
	}
	if entry := ReadHeaderRLP(db, header.Hash(), header.Number.Uint64()); entry == nil {
		t.Fatalf("Stored header RLP not found")
	} else {
		hasher := sha3.NewLegacyKeccak256()
		hasher.Write(entry)

		if hash := common.BytesToHash(hasher.Sum(nil)); hash != header.Hash() {
			t.Fatalf("Retrieved RLP header mismatch: have %v, want %v", entry, header)
		}
	}
	// Delete the header and verify the execution
	DeleteHeader(db, header.Hash(), header.Number.Uint64())
	if entry := ReadHeader(db, header.Hash(), header.Number.Uint64()); entry != nil {
		t.Fatalf("Deleted header returned: %v", entry)
	}
}

// Tests block body storage and retrieval operations.
func TestBodyStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test body to move around the database and make sure it's really new
	body := &types.Body{Uncles: []*types.Header{{Extra: []byte("test header")}}}

	hasher := sha3.NewLegacyKeccak256()
	rlp.Encode(hasher, body)
	hash := common.BytesToHash(hasher.Sum(nil))

	if entry := ReadBody(db, hash, 0); entry != nil {
		t.Fatalf("Non existent body returned: %v", entry)
	}
	// Write and verify the body in the database
	WriteBody(db, hash, 0, body)
	if entry := ReadBody(db, hash, 0); entry == nil {
		t.Fatalf("Stored body not found")
	} else if types.DeriveSha(types.Transactions(entry.Transactions), newTestHasher()) != types.DeriveSha(types.Transactions(body.Transactions), newTestHasher()) || types.CalcUncleHash(entry.Uncles) != types.CalcUncleHash(body.Uncles) {
		t.Fatalf("Retrieved body mismatch: have %v, want %v", entry, body)
	}
	if entry := ReadBodyRLP(db, hash, 0); entry == nil {
		t.Fatalf("Stored body RLP not found")
	} else {
		hasher := sha3.NewLegacyKeccak256()
		hasher.Write(entry)

		if calc := common.BytesToHash(hasher.Sum(nil)); calc != hash {
			t.Fatalf("Retrieved RLP body mismatch: have %v, want %v", entry, body)
		}
	}
	// Delete the body and verify the execution
	DeleteBody(db, hash, 0)
	if entry := ReadBody(db, hash, 0); entry != nil {
		t.Fatalf("Deleted body returned: %v", entry)
	}
}

// Tests block storage and retrieval operations.
func TestBlockStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test block to move around the database and make sure it's really new
	block := types.NewBlockWithHeader(&types.Header{
		Extra:       []byte("test block"),
		UncleHash:   types.EmptyUncleHash,
		TxHash:      types.EmptyTxsHash,
		ReceiptHash: types.EmptyReceiptsHash,
	})
	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Non existent block returned: %v", entry)
	}
	if entry := ReadHeader(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Non existent header returned: %v", entry)
	}
	if entry := ReadBody(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Non existent body returned: %v", entry)
	}
	// Write and verify the block in the database
	WriteBlock(db, block)
	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry == nil {
		t.Fatalf("Stored block not found")
	} else if entry.Hash() != block.Hash() {
		t.Fatalf("Retrieved block mismatch: have %v, want %v", entry, block)
	}
	if entry := ReadHeader(db, block.Hash(), block.NumberU64()); entry == nil {
		t.Fatalf("Stored header not found")
	} else if entry.Hash() != block.Header().Hash() {
		t.Fatalf("Retrieved header mismatch: have %v, want %v", entry, block.Header())
	}
	if entry := ReadBody(db, block.Hash(), block.NumberU64()); entry == nil {
		t.Fatalf("Stored body not found")
	} else if types.DeriveSha(types.Transactions(entry.Transactions), newTestHasher()) != types.DeriveSha(block.Transactions(), newTestHasher()) || types.CalcUncleHash(entry.Uncles) != types.CalcUncleHash(block.Uncles()) {
		t.Fatalf("Retrieved body mismatch: have %v, want %v", entry, block.Body())
	}
	// Delete the block and verify the execution
	DeleteBlock(db, block.Hash(), block.NumberU64())
	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Deleted block returned: %v", entry)
	}
	if entry := ReadHeader(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Deleted header returned: %v", entry)
	}
	if entry := ReadBody(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Deleted body returned: %v", entry)
	}
}

// Tests that partial block contents don't get reassembled into full blocks.
func TestPartialBlockStorage(t *testing.T) {
	db := NewMemoryDatabase()
	block := types.NewBlockWithHeader(&types.Header{
		Extra:       []byte("test block"),
		UncleHash:   types.EmptyUncleHash,
		TxHash:      types.EmptyTxsHash,
		ReceiptHash: types.EmptyReceiptsHash,
	})
	// Store a header and check that it's not recognized as a block
	WriteHeader(db, block.Header())
	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Non existent block returned: %v", entry)
	}
	DeleteHeader(db, block.Hash(), block.NumberU64())

	// Store a body and check that it's not recognized as a block
	WriteBody(db, block.Hash(), block.NumberU64(), block.Body())
	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry != nil {
		t.Fatalf("Non existent block returned: %v", entry)
	}
	DeleteBody(db, block.Hash(), block.NumberU64())

	// Store a header and a body separately and check reassembly
	WriteHeader(db, block.Header())
	WriteBody(db, block.Hash(), block.NumberU64(), block.Body())

	if entry := ReadBlock(db, block.Hash(), block.NumberU64()); entry == nil {
		t.Fatalf("Stored block not found")
	} else if entry.Hash() != block.Hash() {
		t.Fatalf("Retrieved block mismatch: have %v, want %v", entry, block)
	}
}

// Tests block storage and retrieval operations.
func TestBadBlockStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test block to move around the database and make sure it's really new
	block := types.NewBlockWithHeader(&types.Header{
		Number:      big.NewInt(1),
		Extra:       []byte("bad block"),
		UncleHash:   types.EmptyUncleHash,
		TxHash:      types.EmptyTxsHash,
		ReceiptHash: types.EmptyReceiptsHash,
	})
	if entry := ReadBadBlock(db, block.Hash()); entry != nil {
		t.Fatalf("Non existent block returned: %v", entry)
	}
	// Write and verify the block in the database
	WriteBadBlock(db, block)
	if entry := ReadBadBlock(db, block.Hash()); entry == nil {
		t.Fatalf("Stored block not found")
	} else if entry.Hash() != block.Hash() {
		t.Fatalf("Retrieved block mismatch: have %v, want %v", entry, block)
	}
	// Write one more bad block
	blockTwo := types.NewBlockWithHeader(&types.Header{
		Number:      big.NewInt(2),
		Extra:       []byte("bad block two"),
		UncleHash:   types.EmptyUncleHash,
		TxHash:      types.EmptyTxsHash,
		ReceiptHash: types.EmptyReceiptsHash,
	})
	WriteBadBlock(db, blockTwo)

	// Write the block one again, should be filtered out.
	WriteBadBlock(db, block)
	badBlocks := ReadAllBadBlocks(db)
	if len(badBlocks) != 2 {
		t.Fatalf("Failed to load all bad blocks")
	}

	// Write a bunch of bad blocks, all the blocks are should sorted
	// in reverse order. The extra blocks should be truncated.
	for _, n := range rand.Perm(100) {
		block := types.NewBlockWithHeader(&types.Header{
			Number:      big.NewInt(int64(n)),
			Extra:       []byte("bad block"),
			UncleHash:   types.EmptyUncleHash,
			TxHash:      types.EmptyTxsHash,
			ReceiptHash: types.EmptyReceiptsHash,
		})
		WriteBadBlock(db, block)
	}
	badBlocks = ReadAllBadBlocks(db)
	if len(badBlocks) != badBlockToKeep {
		t.Fatalf("The number of persised bad blocks in incorrect %d", len(badBlocks))
	}
	for i := 0; i < len(badBlocks)-1; i++ {
		if badBlocks[i].NumberU64() < badBlocks[i+1].NumberU64() {
			t.Fatalf("The bad blocks are not sorted #[%d](%d) < #[%d](%d)", i, i+1, badBlocks[i].NumberU64(), badBlocks[i+1].NumberU64())
		}
	}

	// Delete all bad blocks
	DeleteBadBlocks(db)
	badBlocks = ReadAllBadBlocks(db)
	if len(badBlocks) != 0 {
		t.Fatalf("Failed to delete bad blocks")
	}
}

// Tests block total difficulty storage and retrieval operations.
func TestTdStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test TD to move around the database and make sure it's really new
	hash, td := common.Hash{}, big.NewInt(314)
	if entry := ReadTd(db, hash, 0); entry != nil {
		t.Fatalf("Non existent TD returned: %v", entry)
	}
	// Write and verify the TD in the database
	WriteTd(db, hash, 0, td)
	if entry := ReadTd(db, hash, 0); entry == nil {
		t.Fatalf("Stored TD not found")
	} else if entry.Cmp(td) != 0 {
		t.Fatalf("Retrieved TD mismatch: have %v, want %v", entry, td)
	}
	// Delete the TD and verify the execution
	DeleteTd(db, hash, 0)
	if entry := ReadTd(db, hash, 0); entry != nil {
		t.Fatalf("Deleted TD returned: %v", entry)
	}
}

// Tests that canonical numbers can be mapped to hashes and retrieved.
func TestCanonicalMappingStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a test canonical number and assigned hash to move around
	hash, number := common.Hash{0: 0xff}, uint64(314)
	if entry := ReadCanonicalHash(db, number); entry != (common.Hash{}) {
		t.Fatalf("Non existent canonical mapping returned: %v", entry)
	}
	// Write and verify the TD in the database
	WriteCanonicalHash(db, hash, number)
	if entry := ReadCanonicalHash(db, number); entry == (common.Hash{}) {
		t.Fatalf("Stored canonical mapping not found")
	} else if entry != hash {
		t.Fatalf("Retrieved canonical mapping mismatch: have %v, want %v", entry, hash)
	}
	// Delete the TD and verify the execution
	DeleteCanonicalHash(db, number)
	if entry := ReadCanonicalHash(db, number); entry != (common.Hash{}) {
		t.Fatalf("Deleted canonical mapping returned: %v", entry)
	}
}

// Tests that head headers and head blocks can be assigned, individually.
func TestHeadStorage(t *testing.T) {
	db := NewMemoryDatabase()

	blockHead := types.NewBlockWithHeader(&types.Header{Extra: []byte("test block header")})
	blockFull := types.NewBlockWithHeader(&types.Header{Extra: []byte("test block full")})
	blockFast := types.NewBlockWithHeader(&types.Header{Extra: []byte("test block fast")})

	// Check that no head entries are in a pristine database
	if entry := ReadHeadHeaderHash(db); entry != (common.Hash{}) {
		t.Fatalf("Non head header entry returned: %v", entry)
	}
	if entry := ReadHeadBlockHash(db); entry != (common.Hash{}) {
		t.Fatalf("Non head block entry returned: %v", entry)
	}
	if entry := ReadHeadFastBlockHash(db); entry != (common.Hash{}) {
		t.Fatalf("Non fast head block entry returned: %v", entry)
	}
	// Assign separate entries for the head header and block
	WriteHeadHeaderHash(db, blockHead.Hash())
	WriteHeadBlockHash(db, blockFull.Hash())
	WriteHeadFastBlockHash(db, blockFast.Hash())

	// Check that both heads are present, and different (i.e. two heads maintained)
	if entry := ReadHeadHeaderHash(db); entry != blockHead.Hash() {
		t.Fatalf("Head header hash mismatch: have %v, want %v", entry, blockHead.Hash())
	}
	if entry := ReadHeadBlockHash(db); entry != blockFull.Hash() {
		t.Fatalf("Head block hash mismatch: have %v, want %v", entry, blockFull.Hash())
	}
	if entry := ReadHeadFastBlockHash(db); entry != blockFast.Hash() {
		t.Fatalf("Fast head block hash mismatch: have %v, want %v", entry, blockFast.Hash())
	}
}

// Tests that receipts associated with a single block can be stored and retrieved.
func TestBlockReceiptStorage(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a live block since we need metadata to reconstruct the receipt
	tx1 := types.NewTransaction(1, common.HexToAddress("0x1"), big.NewInt(1), 1, big.NewInt(1), nil)
	tx2 := types.NewTransaction(2, common.HexToAddress("0x2"), big.NewInt(2), 2, big.NewInt(2), nil)

	body := &types.Body{Transactions: types.Transactions{tx1, tx2}}

	// Create the two receipts to manage afterwards
	receipt1 := &types.Receipt{
		Status:            types.ReceiptStatusFailed,
		CumulativeGasUsed: 1,
		Logs: []*types.Log{
			{Address: common.BytesToAddress([]byte{0x11})},
			{Address: common.BytesToAddress([]byte{0x01, 0x11})},
		},
		TxHash:          tx1.Hash(),
		ContractAddress: common.BytesToAddress([]byte{0x01, 0x11, 0x11}),
		GasUsed:         111111,
	}
	receipt1.Bloom = types.CreateBloom(types.Receipts{receipt1})

	receipt2 := &types.Receipt{
		PostState:         common.Hash{2}.Bytes(),
		CumulativeGasUsed: 2,
		Logs: []*types.Log{
			{Address: common.BytesToAddress([]byte{0x22})},
			{Address: common.BytesToAddress([]byte{0x02, 0x22})},
		},
		TxHash:          tx2.Hash(),
		ContractAddress: common.BytesToAddress([]byte{0x02, 0x22, 0x22}),
		GasUsed:         222222,
	}
	receipt2.Bloom = types.CreateBloom(types.Receipts{receipt2})
	receipts := []*types.Receipt{receipt1, receipt2}

	// Check that no receipt entries are in a pristine database
	hash := common.BytesToHash([]byte{0x03, 0x14})
	if rs := ReadReceipts(db, hash, 0, 0, params.TestChainConfig); len(rs) != 0 {
		t.Fatalf("non existent receipts returned: %v", rs)
	}
	// Insert the body that corresponds to the receipts
	WriteBody(db, hash, 0, body)

	// Insert the receipt slice into the database and check presence
	WriteReceipts(db, hash, 0, receipts)
	if rs := ReadReceipts(db, hash, 0, 0, params.TestChainConfig); len(rs) == 0 {
		t.Fatalf("no receipts returned")
	} else {
		if err := checkReceiptsRLP(rs, receipts); err != nil {
			t.Fatalf(err.Error())
		}
	}
	// Delete the body and ensure that the receipts are no longer returned (metadata can't be recomputed)
	DeleteBody(db, hash, 0)
	if rs := ReadReceipts(db, hash, 0, 0, params.TestChainConfig); rs != nil {
		t.Fatalf("receipts returned when body was deleted: %v", rs)
	}
	// Ensure that receipts without metadata can be returned without the block body too
	if err := checkReceiptsRLP(ReadRawReceipts(db, hash, 0), receipts); err != nil {
		t.Fatalf(err.Error())
	}
	// Sanity check that body alone without the receipt is a full purge
	WriteBody(db, hash, 0, body)

	DeleteReceipts(db, hash, 0)
	if rs := ReadReceipts(db, hash, 0, 0, params.TestChainConfig); len(rs) != 0 {
		t.Fatalf("deleted receipts returned: %v", rs)
	}
}

func checkReceiptsRLP(have, want types.Receipts) error {
	if len(have) != len(want) {
		return fmt.Errorf("receipts sizes mismatch: have %d, want %d", len(have), len(want))
	}
	for i := 0; i < len(want); i++ {
		rlpHave, err := rlp.EncodeToBytes(have[i])
		if err != nil {
			return err
		}
		rlpWant, err := rlp.EncodeToBytes(want[i])
		if err != nil {
			return err
		}
		if !bytes.Equal(rlpHave, rlpWant) {
			return fmt.Errorf("receipt #%d: receipt mismatch: have %s, want %s", i, hex.EncodeToString(rlpHave), hex.EncodeToString(rlpWant))
		}
	}
	return nil
}

func TestAncientStorage(t *testing.T) {
	// Freezer style fast import the chain.
	frdir := t.TempDir()
	db, err := NewDatabaseWithFreezer(NewMemoryDatabase(), frdir, "", false)
	if err != nil {
		t.Fatalf("failed to create database with ancient backend")
	}
	defer db.Close()

	// Create a test block
	block := types.NewBlockWithHeader(&types.Header{
		Number:      big.NewInt(0),
		Extra:       []byte("test block"),
		UncleHash:   types.EmptyUncleHash,
		TxHash:      types.EmptyTxsHash,
		ReceiptHash: types.EmptyReceiptsHash,
	})
	// Ensure nothing non-existent will be read
	hash, number := block.Hash(), block.NumberU64()
	if blob := ReadHeaderRLP(db, hash, number); len(blob) > 0 {
		t.Fatalf("non existent header returned")
	}
	if blob := ReadBodyRLP(db, hash, number); len(blob) > 0 {
		t.Fatalf("non existent body returned")
	}
	if blob := ReadReceiptsRLP(db, hash, number); len(blob) > 0 {
		t.Fatalf("non existent receipts returned")
	}
	if blob := ReadTdRLP(db, hash, number); len(blob) > 0 {
		t.Fatalf("non existent td returned")
	}

	// Write and verify the header in the database
	WriteAncientBlocks(db, []*types.Block{block}, []types.Receipts{nil}, big.NewInt(100))

	if blob := ReadHeaderRLP(db, hash, number); len(blob) == 0 {
		t.Fatalf("no header returned")
	}
	if blob := ReadBodyRLP(db, hash, number); len(blob) == 0 {
		t.Fatalf("no body returned")
	}
	if blob := ReadReceiptsRLP(db, hash, number); len(blob) == 0 {
		t.Fatalf("no receipts returned")
	}
	if blob := ReadTdRLP(db, hash, number); len(blob) == 0 {
		t.Fatalf("no td returned")
	}

	// Use a fake hash for data retrieval, nothing should be returned.
	fakeHash := common.BytesToHash([]byte{0x01, 0x02, 0x03})
	if blob := ReadHeaderRLP(db, fakeHash, number); len(blob) != 0 {
		t.Fatalf("invalid header returned")
	}
	if blob := ReadBodyRLP(db, fakeHash, number); len(blob) != 0 {
		t.Fatalf("invalid body returned")
	}
	if blob := ReadReceiptsRLP(db, fakeHash, number); len(blob) != 0 {
		t.Fatalf("invalid receipts returned")
	}
	if blob := ReadTdRLP(db, fakeHash, number); len(blob) != 0 {
		t.Fatalf("invalid td returned")
	}
}

func TestCanonicalHashIteration(t *testing.T) {
	var cases = []struct {
		from, to uint64
		limit    int
		expect   []uint64
	}{
		{1, 8, 0, nil},
		{1, 8, 1, []uint64{1}},
		{1, 8, 10, []uint64{1, 2, 3, 4, 5, 6, 7}},
		{1, 9, 10, []uint64{1, 2, 3, 4, 5, 6, 7, 8}},
		{2, 9, 10, []uint64{2, 3, 4, 5, 6, 7, 8}},
		{9, 10, 10, nil},
	}
	// Test empty db iteration
	db := NewMemoryDatabase()
	numbers, _ := ReadAllCanonicalHashes(db, 0, 10, 10)
	if len(numbers) != 0 {
		t.Fatalf("No entry should be returned to iterate an empty db")
	}
	// Fill database with testing data.
	for i := uint64(1); i <= 8; i++ {
		WriteCanonicalHash(db, common.Hash{}, i)
		WriteTd(db, common.Hash{}, i, big.NewInt(10)) // Write some interferential data
	}
	for i, c := range cases {
		numbers, _ := ReadAllCanonicalHashes(db, c.from, c.to, c.limit)
		if !reflect.DeepEqual(numbers, c.expect) {
			t.Fatalf("Case %d failed, want %v, got %v", i, c.expect, numbers)
		}
	}
}

func TestHashesInRange(t *testing.T) {
	mkHeader := func(number, seq int) *types.Header {
		h := types.Header{
			Difficulty: new(big.Int),
			Number:     big.NewInt(int64(number)),
			GasLimit:   uint64(seq),
		}
		return &h
	}
	db := NewMemoryDatabase()
	// For each number, write N versions of that particular number
	total := 0
	for i := 0; i < 15; i++ {
		for ii := 0; ii < i; ii++ {
			WriteHeader(db, mkHeader(i, ii))
			total++
		}
	}
	if have, want := len(ReadAllHashesInRange(db, 10, 10)), 10; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashesInRange(db, 10, 9)), 0; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashesInRange(db, 0, 100)), total; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashesInRange(db, 9, 10)), 9+10; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashes(db, 10)), 10; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashes(db, 16)), 0; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
	if have, want := len(ReadAllHashes(db, 1)), 1; have != want {
		t.Fatalf("Wrong number of hashes read, want %d, got %d", want, have)
	}
}

// This measures the write speed of the WriteAncientBlocks operation.
func BenchmarkWriteAncientBlocks(b *testing.B) {
	// Open freezer database.
	frdir := b.TempDir()
	db, err := NewDatabaseWithFreezer(NewMemoryDatabase(), frdir, "", false)
	if err != nil {
		b.Fatalf("failed to create database with ancient backend")
	}
	defer db.Close()

	// Create the data to insert. The blocks must have consecutive numbers, so we create
	// all of them ahead of time. However, there is no need to create receipts
	// individually for each block, just make one batch here and reuse it for all writes.
	const batchSize = 128
	const blockTxs = 20
	allBlocks := makeTestBlocks(b.N, blockTxs)
	batchReceipts := makeTestReceipts(batchSize, blockTxs)
	b.ResetTimer()

	// The benchmark loop writes batches of blocks, but note that the total block count is
	// b.N. This means the resulting ns/op measurement is the time it takes to write a
	// single block and its associated data.
	var td = big.NewInt(55)
	var totalSize int64
	for i := 0; i < b.N; i += batchSize {
		length := batchSize
		if i+batchSize > b.N {
			length = b.N - i
		}

		blocks := allBlocks[i : i+length]
		receipts := batchReceipts[:length]
		writeSize, err := WriteAncientBlocks(db, blocks, receipts, td)
		if err != nil {
			b.Fatal(err)
		}
		totalSize += writeSize
	}

	// Enable MB/s reporting.
	b.SetBytes(totalSize / int64(b.N))
}

// makeTestBlocks creates fake blocks for the ancient write benchmark.
func makeTestBlocks(nblock int, txsPerBlock int) []*types.Block {
	key, _ := crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
	signer := types.LatestSignerForChainID(big.NewInt(8))

	// Create transactions.
	txs := make([]*types.Transaction, txsPerBlock)
	for i := 0; i < len(txs); i++ {
		var err error
		to := common.Address{1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}
		txs[i], err = types.SignNewTx(key, signer, &types.LegacyTx{
			Nonce:    2,
			GasPrice: big.NewInt(30000),
			Gas:      0x45454545,
			To:       &to,
		})
		if err != nil {
			panic(err)
		}
	}

	// Create the blocks.
	blocks := make([]*types.Block, nblock)
	for i := 0; i < nblock; i++ {
		header := &types.Header{
			Number: big.NewInt(int64(i)),
			Extra:  []byte("test block"),
		}
		blocks[i] = types.NewBlockWithHeader(header).WithBody(txs, nil)
		blocks[i].Hash() // pre-cache the block hash
	}
	return blocks
}

// makeTestReceipts creates fake receipts for the ancient write benchmark.
func makeTestReceipts(n int, nPerBlock int) []types.Receipts {
	receipts := make([]*types.Receipt, nPerBlock)
	for i := 0; i < len(receipts); i++ {
		receipts[i] = &types.Receipt{
			Status:            types.ReceiptStatusSuccessful,
			CumulativeGasUsed: 0x888888888,
			Logs:              make([]*types.Log, 5),
		}
	}
	allReceipts := make([]types.Receipts, n)
	for i := 0; i < n; i++ {
		allReceipts[i] = receipts
	}
	return allReceipts
}

type fullLogRLP struct {
	Address     common.Address
	Topics      []common.Hash
	Data        []byte
	BlockNumber uint64
	TxHash      common.Hash
	TxIndex     uint
	BlockHash   common.Hash
	Index       uint
}

func newFullLogRLP(l *types.Log) *fullLogRLP {
	return &fullLogRLP{
		Address:     l.Address,
		Topics:      l.Topics,
		Data:        l.Data,
		BlockNumber: l.BlockNumber,
		TxHash:      l.TxHash,
		TxIndex:     l.TxIndex,
		BlockHash:   l.BlockHash,
		Index:       l.Index,
	}
}

// Tests that logs associated with a single block can be retrieved.
func TestReadLogs(t *testing.T) {
	db := NewMemoryDatabase()

	// Create a live block since we need metadata to reconstruct the receipt
	tx1 := types.NewTransaction(1, common.HexToAddress("0x1"), big.NewInt(1), 1, big.NewInt(1), nil)
	tx2 := types.NewTransaction(2, common.HexToAddress("0x2"), big.NewInt(2), 2, big.NewInt(2), nil)

	body := &types.Body{Transactions: types.Transactions{tx1, tx2}}

	// Create the two receipts to manage afterwards
	receipt1 := &types.Receipt{
		Status:            types.ReceiptStatusFailed,
		CumulativeGasUsed: 1,
		Logs: []*types.Log{
			{Address: common.BytesToAddress([]byte{0x11})},
			{Address: common.BytesToAddress([]byte{0x01, 0x11})},
		},
		TxHash:          tx1.Hash(),
		ContractAddress: common.BytesToAddress([]byte{0x01, 0x11, 0x11}),
		GasUsed:         111111,
	}
	receipt1.Bloom = types.CreateBloom(types.Receipts{receipt1})

	receipt2 := &types.Receipt{
		PostState:         common.Hash{2}.Bytes(),
		CumulativeGasUsed: 2,
		Logs: []*types.Log{
			{Address: common.BytesToAddress([]byte{0x22})},
			{Address: common.BytesToAddress([]byte{0x02, 0x22})},
		},
		TxHash:          tx2.Hash(),
		ContractAddress: common.BytesToAddress([]byte{0x02, 0x22, 0x22}),
		GasUsed:         222222,
	}
	receipt2.Bloom = types.CreateBloom(types.Receipts{receipt2})
	receipts := []*types.Receipt{receipt1, receipt2}

	hash := common.BytesToHash([]byte{0x03, 0x14})
	// Check that no receipt entries are in a pristine database
	if rs := ReadReceipts(db, hash, 0, 0, params.TestChainConfig); len(rs) != 0 {
		t.Fatalf("non existent receipts returned: %v", rs)
	}
	// Insert the body that corresponds to the receipts
	WriteBody(db, hash, 0, body)

	// Insert the receipt slice into the database and check presence
	WriteReceipts(db, hash, 0, receipts)

	logs := ReadLogs(db, hash, 0)
	if len(logs) == 0 {
		t.Fatalf("no logs returned")
	}
	if have, want := len(logs), 2; have != want {
		t.Fatalf("unexpected number of logs returned, have %d want %d", have, want)
	}
	if have, want := len(logs[0]), 2; have != want {
		t.Fatalf("unexpected number of logs[0] returned, have %d want %d", have, want)
	}
	if have, want := len(logs[1]), 2; have != want {
		t.Fatalf("unexpected number of logs[1] returned, have %d want %d", have, want)
	}

	for i, pr := range receipts {
		for j, pl := range pr.Logs {
			rlpHave, err := rlp.EncodeToBytes(newFullLogRLP(logs[i][j]))
			if err != nil {
				t.Fatal(err)
			}
			rlpWant, err := rlp.EncodeToBytes(newFullLogRLP(pl))
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(rlpHave, rlpWant) {
				t.Fatalf("receipt #%d: receipt mismatch: have %s, want %s", i, hex.EncodeToString(rlpHave), hex.EncodeToString(rlpWant))
			}
		}
	}
}

func TestDeriveLogFields(t *testing.T) {
	// Create a few transactions to have receipts for
	to2 := common.HexToAddress("0x2")
	to3 := common.HexToAddress("0x3")
	txs := types.Transactions{
		types.NewTx(&types.LegacyTx{
			Nonce:    1,
			Value:    big.NewInt(1),
			Gas:      1,
			GasPrice: big.NewInt(1),
		}),
		types.NewTx(&types.LegacyTx{
			To:       &to2,
			Nonce:    2,
			Value:    big.NewInt(2),
			Gas:      2,
			GasPrice: big.NewInt(2),
		}),
		types.NewTx(&types.AccessListTx{
			To:       &to3,
			Nonce:    3,
			Value:    big.NewInt(3),
			Gas:      3,
			GasPrice: big.NewInt(3),
		}),
	}
	// Create the corresponding receipts
	receipts := []*receiptLogs{
		{
			Logs: []*types.Log{
				{Address: common.BytesToAddress([]byte{0x11})},
				{Address: common.BytesToAddress([]byte{0x01, 0x11})},
			},
		},
		{
			Logs: []*types.Log{
				{Address: common.BytesToAddress([]byte{0x22})},
				{Address: common.BytesToAddress([]byte{0x02, 0x22})},
			},
		},
		{
			Logs: []*types.Log{
				{Address: common.BytesToAddress([]byte{0x33})},
				{Address: common.BytesToAddress([]byte{0x03, 0x33})},
			},
		},
	}

	// Derive log metadata fields
	number := big.NewInt(1)
	hash := common.BytesToHash([]byte{0x03, 0x14})
	if err := deriveLogFields(receipts, hash, number.Uint64(), txs); err != nil {
		t.Fatal(err)
	}

	// Iterate over all the computed fields and check that they're correct
	logIndex := uint(0)
	for i := range receipts {
		for j := range receipts[i].Logs {
			if receipts[i].Logs[j].BlockNumber != number.Uint64() {
				t.Errorf("receipts[%d].Logs[%d].BlockNumber = %d, want %d", i, j, receipts[i].Logs[j].BlockNumber, number.Uint64())
			}
			if receipts[i].Logs[j].BlockHash != hash {
				t.Errorf("receipts[%d].Logs[%d].BlockHash = %s, want %s", i, j, receipts[i].Logs[j].BlockHash.String(), hash.String())
			}
			if receipts[i].Logs[j].TxHash != txs[i].Hash() {
				t.Errorf("receipts[%d].Logs[%d].TxHash = %s, want %s", i, j, receipts[i].Logs[j].TxHash.String(), txs[i].Hash().String())
			}
			if receipts[i].Logs[j].TxIndex != uint(i) {
				t.Errorf("receipts[%d].Logs[%d].TransactionIndex = %d, want %d", i, j, receipts[i].Logs[j].TxIndex, i)
			}
			if receipts[i].Logs[j].Index != logIndex {
				t.Errorf("receipts[%d].Logs[%d].Index = %d, want %d", i, j, receipts[i].Logs[j].Index, logIndex)
			}
			logIndex++
		}
	}
}

func BenchmarkDecodeRLPLogs(b *testing.B) {
	// Encoded receipts from block 0x14ee094309fbe8f70b65f45ebcc08fb33f126942d97464aad5eb91cfd1e2d269
	buf, err := os.ReadFile("testdata/stored_receipts.bin")
	if err != nil {
		b.Fatal(err)
	}
	b.Run("ReceiptForStorage", func(b *testing.B) {
		b.ReportAllocs()
		var r []*types.ReceiptForStorage
		for i := 0; i < b.N; i++ {
			if err := rlp.DecodeBytes(buf, &r); err != nil {
				b.Fatal(err)
			}
		}
	})
	b.Run("rlpLogs", func(b *testing.B) {
		b.ReportAllocs()
		var r []*receiptLogs
		for i := 0; i < b.N; i++ {
			if err := rlp.DecodeBytes(buf, &r); err != nil {
				b.Fatal(err)
			}
		}
	})
}

func TestHeadersRLPStorage(t *testing.T) {
	// Have N headers in the freezer
	frdir := t.TempDir()

	db, err := NewDatabaseWithFreezer(NewMemoryDatabase(), frdir, "", false)
	if err != nil {
		t.Fatalf("failed to create database with ancient backend")
	}
	defer db.Close()
	// Create blocks
	var chain []*types.Block
	var pHash common.Hash
	for i := 0; i < 100; i++ {
		block := types.NewBlockWithHeader(&types.Header{
			Number:      big.NewInt(int64(i)),
			Extra:       []byte("test block"),
			UncleHash:   types.EmptyUncleHash,
			TxHash:      types.EmptyTxsHash,
			ReceiptHash: types.EmptyReceiptsHash,
			ParentHash:  pHash,
		})
		chain = append(chain, block)
		pHash = block.Hash()
	}
	var receipts []types.Receipts = make([]types.Receipts, 100)
	// Write first half to ancients
	WriteAncientBlocks(db, chain[:50], receipts[:50], big.NewInt(100))
	// Write second half to db
	for i := 50; i < 100; i++ {
		WriteCanonicalHash(db, chain[i].Hash(), chain[i].NumberU64())
		WriteBlock(db, chain[i])
	}
	checkSequence := func(from, amount int) {
		headersRlp := ReadHeaderRange(db, uint64(from), uint64(amount))
		if have, want := len(headersRlp), amount; have != want {
			t.Fatalf("have %d headers, want %d", have, want)
		}
		for i, headerRlp := range headersRlp {
			var header types.Header
			if err := rlp.DecodeBytes(headerRlp, &header); err != nil {
				t.Fatal(err)
			}
			if have, want := header.Number.Uint64(), uint64(from-i); have != want {
				t.Fatalf("wrong number, have %d want %d", have, want)
			}
		}
	}
	checkSequence(99, 20)  // Latest block and 19 parents
	checkSequence(99, 50)  // Latest block -> all db blocks
	checkSequence(99, 51)  // Latest block -> one from ancients
	checkSequence(99, 52)  // Latest blocks -> two from ancients
	checkSequence(50, 2)   // One from db, one from ancients
	checkSequence(49, 1)   // One from ancients
	checkSequence(49, 50)  // All ancient ones
	checkSequence(99, 100) // All blocks
	checkSequence(0, 1)    // Only genesis
	checkSequence(1, 1)    // Only block 1
	checkSequence(1, 2)    // Genesis + block 1
}
