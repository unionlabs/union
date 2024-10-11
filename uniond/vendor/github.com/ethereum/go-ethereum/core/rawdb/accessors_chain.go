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
	"encoding/binary"
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/misc/eip4844"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"golang.org/x/exp/slices"
)

// ReadCanonicalHash retrieves the hash assigned to a canonical block number.
func ReadCanonicalHash(db ethdb.Reader, number uint64) common.Hash {
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		data, _ = reader.Ancient(ChainFreezerHashTable, number)
		if len(data) == 0 {
			// Get it by hash from leveldb
			data, _ = db.Get(headerHashKey(number))
		}
		return nil
	})
	return common.BytesToHash(data)
}

// WriteCanonicalHash stores the hash assigned to a canonical block number.
func WriteCanonicalHash(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	if err := db.Put(headerHashKey(number), hash.Bytes()); err != nil {
		log.Crit("Failed to store number to hash mapping", "err", err)
	}
}

// DeleteCanonicalHash removes the number to hash canonical mapping.
func DeleteCanonicalHash(db ethdb.KeyValueWriter, number uint64) {
	if err := db.Delete(headerHashKey(number)); err != nil {
		log.Crit("Failed to delete number to hash mapping", "err", err)
	}
}

// ReadAllHashes retrieves all the hashes assigned to blocks at a certain heights,
// both canonical and reorged forks included.
func ReadAllHashes(db ethdb.Iteratee, number uint64) []common.Hash {
	prefix := headerKeyPrefix(number)

	hashes := make([]common.Hash, 0, 1)
	it := db.NewIterator(prefix, nil)
	defer it.Release()

	for it.Next() {
		if key := it.Key(); len(key) == len(prefix)+32 {
			hashes = append(hashes, common.BytesToHash(key[len(key)-32:]))
		}
	}
	return hashes
}

type NumberHash struct {
	Number uint64
	Hash   common.Hash
}

// ReadAllHashesInRange retrieves all the hashes assigned to blocks at certain
// heights, both canonical and reorged forks included.
// This method considers both limits to be _inclusive_.
func ReadAllHashesInRange(db ethdb.Iteratee, first, last uint64) []*NumberHash {
	var (
		start     = encodeBlockNumber(first)
		keyLength = len(headerPrefix) + 8 + 32
		hashes    = make([]*NumberHash, 0, 1+last-first)
		it        = db.NewIterator(headerPrefix, start)
	)
	defer it.Release()
	for it.Next() {
		key := it.Key()
		if len(key) != keyLength {
			continue
		}
		num := binary.BigEndian.Uint64(key[len(headerPrefix) : len(headerPrefix)+8])
		if num > last {
			break
		}
		hash := common.BytesToHash(key[len(key)-32:])
		hashes = append(hashes, &NumberHash{num, hash})
	}
	return hashes
}

// ReadAllCanonicalHashes retrieves all canonical number and hash mappings at the
// certain chain range. If the accumulated entries reaches the given threshold,
// abort the iteration and return the semi-finish result.
func ReadAllCanonicalHashes(db ethdb.Iteratee, from uint64, to uint64, limit int) ([]uint64, []common.Hash) {
	// Short circuit if the limit is 0.
	if limit == 0 {
		return nil, nil
	}
	var (
		numbers []uint64
		hashes  []common.Hash
	)
	// Construct the key prefix of start point.
	start, end := headerHashKey(from), headerHashKey(to)
	it := db.NewIterator(nil, start)
	defer it.Release()

	for it.Next() {
		if bytes.Compare(it.Key(), end) >= 0 {
			break
		}
		if key := it.Key(); len(key) == len(headerPrefix)+8+1 && bytes.Equal(key[len(key)-1:], headerHashSuffix) {
			numbers = append(numbers, binary.BigEndian.Uint64(key[len(headerPrefix):len(headerPrefix)+8]))
			hashes = append(hashes, common.BytesToHash(it.Value()))
			// If the accumulated entries reaches the limit threshold, return.
			if len(numbers) >= limit {
				break
			}
		}
	}
	return numbers, hashes
}

// ReadHeaderNumber returns the header number assigned to a hash.
func ReadHeaderNumber(db ethdb.KeyValueReader, hash common.Hash) *uint64 {
	data, _ := db.Get(headerNumberKey(hash))
	if len(data) != 8 {
		return nil
	}
	number := binary.BigEndian.Uint64(data)
	return &number
}

// WriteHeaderNumber stores the hash->number mapping.
func WriteHeaderNumber(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	key := headerNumberKey(hash)
	enc := encodeBlockNumber(number)
	if err := db.Put(key, enc); err != nil {
		log.Crit("Failed to store hash to number mapping", "err", err)
	}
}

// DeleteHeaderNumber removes hash->number mapping.
func DeleteHeaderNumber(db ethdb.KeyValueWriter, hash common.Hash) {
	if err := db.Delete(headerNumberKey(hash)); err != nil {
		log.Crit("Failed to delete hash to number mapping", "err", err)
	}
}

// ReadHeadHeaderHash retrieves the hash of the current canonical head header.
func ReadHeadHeaderHash(db ethdb.KeyValueReader) common.Hash {
	data, _ := db.Get(headHeaderKey)
	if len(data) == 0 {
		return common.Hash{}
	}
	return common.BytesToHash(data)
}

// WriteHeadHeaderHash stores the hash of the current canonical head header.
func WriteHeadHeaderHash(db ethdb.KeyValueWriter, hash common.Hash) {
	if err := db.Put(headHeaderKey, hash.Bytes()); err != nil {
		log.Crit("Failed to store last header's hash", "err", err)
	}
}

// ReadHeadBlockHash retrieves the hash of the current canonical head block.
func ReadHeadBlockHash(db ethdb.KeyValueReader) common.Hash {
	data, _ := db.Get(headBlockKey)
	if len(data) == 0 {
		return common.Hash{}
	}
	return common.BytesToHash(data)
}

// WriteHeadBlockHash stores the head block's hash.
func WriteHeadBlockHash(db ethdb.KeyValueWriter, hash common.Hash) {
	if err := db.Put(headBlockKey, hash.Bytes()); err != nil {
		log.Crit("Failed to store last block's hash", "err", err)
	}
}

// ReadHeadFastBlockHash retrieves the hash of the current fast-sync head block.
func ReadHeadFastBlockHash(db ethdb.KeyValueReader) common.Hash {
	data, _ := db.Get(headFastBlockKey)
	if len(data) == 0 {
		return common.Hash{}
	}
	return common.BytesToHash(data)
}

// WriteHeadFastBlockHash stores the hash of the current fast-sync head block.
func WriteHeadFastBlockHash(db ethdb.KeyValueWriter, hash common.Hash) {
	if err := db.Put(headFastBlockKey, hash.Bytes()); err != nil {
		log.Crit("Failed to store last fast block's hash", "err", err)
	}
}

// ReadFinalizedBlockHash retrieves the hash of the finalized block.
func ReadFinalizedBlockHash(db ethdb.KeyValueReader) common.Hash {
	data, _ := db.Get(headFinalizedBlockKey)
	if len(data) == 0 {
		return common.Hash{}
	}
	return common.BytesToHash(data)
}

// WriteFinalizedBlockHash stores the hash of the finalized block.
func WriteFinalizedBlockHash(db ethdb.KeyValueWriter, hash common.Hash) {
	if err := db.Put(headFinalizedBlockKey, hash.Bytes()); err != nil {
		log.Crit("Failed to store last finalized block's hash", "err", err)
	}
}

// ReadLastPivotNumber retrieves the number of the last pivot block. If the node
// full synced, the last pivot will always be nil.
func ReadLastPivotNumber(db ethdb.KeyValueReader) *uint64 {
	data, _ := db.Get(lastPivotKey)
	if len(data) == 0 {
		return nil
	}
	var pivot uint64
	if err := rlp.DecodeBytes(data, &pivot); err != nil {
		log.Error("Invalid pivot block number in database", "err", err)
		return nil
	}
	return &pivot
}

// WriteLastPivotNumber stores the number of the last pivot block.
func WriteLastPivotNumber(db ethdb.KeyValueWriter, pivot uint64) {
	enc, err := rlp.EncodeToBytes(pivot)
	if err != nil {
		log.Crit("Failed to encode pivot block number", "err", err)
	}
	if err := db.Put(lastPivotKey, enc); err != nil {
		log.Crit("Failed to store pivot block number", "err", err)
	}
}

// ReadTxIndexTail retrieves the number of oldest indexed block
// whose transaction indices has been indexed.
func ReadTxIndexTail(db ethdb.KeyValueReader) *uint64 {
	data, _ := db.Get(txIndexTailKey)
	if len(data) != 8 {
		return nil
	}
	number := binary.BigEndian.Uint64(data)
	return &number
}

// WriteTxIndexTail stores the number of oldest indexed block
// into database.
func WriteTxIndexTail(db ethdb.KeyValueWriter, number uint64) {
	if err := db.Put(txIndexTailKey, encodeBlockNumber(number)); err != nil {
		log.Crit("Failed to store the transaction index tail", "err", err)
	}
}

// ReadFastTxLookupLimit retrieves the tx lookup limit used in fast sync.
func ReadFastTxLookupLimit(db ethdb.KeyValueReader) *uint64 {
	data, _ := db.Get(fastTxLookupLimitKey)
	if len(data) != 8 {
		return nil
	}
	number := binary.BigEndian.Uint64(data)
	return &number
}

// WriteFastTxLookupLimit stores the txlookup limit used in fast sync into database.
func WriteFastTxLookupLimit(db ethdb.KeyValueWriter, number uint64) {
	if err := db.Put(fastTxLookupLimitKey, encodeBlockNumber(number)); err != nil {
		log.Crit("Failed to store transaction lookup limit for fast sync", "err", err)
	}
}

// ReadHeaderRange returns the rlp-encoded headers, starting at 'number', and going
// backwards towards genesis. This method assumes that the caller already has
// placed a cap on count, to prevent DoS issues.
// Since this method operates in head-towards-genesis mode, it will return an empty
// slice in case the head ('number') is missing. Hence, the caller must ensure that
// the head ('number') argument is actually an existing header.
//
// N.B: Since the input is a number, as opposed to a hash, it's implicit that
// this method only operates on canon headers.
func ReadHeaderRange(db ethdb.Reader, number uint64, count uint64) []rlp.RawValue {
	var rlpHeaders []rlp.RawValue
	if count == 0 {
		return rlpHeaders
	}
	i := number
	if count-1 > number {
		// It's ok to request block 0, 1 item
		count = number + 1
	}
	limit, _ := db.Ancients()
	// First read live blocks
	if i >= limit {
		// If we need to read live blocks, we need to figure out the hash first
		hash := ReadCanonicalHash(db, number)
		for ; i >= limit && count > 0; i-- {
			if data, _ := db.Get(headerKey(i, hash)); len(data) > 0 {
				rlpHeaders = append(rlpHeaders, data)
				// Get the parent hash for next query
				hash = types.HeaderParentHashFromRLP(data)
			} else {
				break // Maybe got moved to ancients
			}
			count--
		}
	}
	if count == 0 {
		return rlpHeaders
	}
	// read remaining from ancients
	max := count * 700
	data, err := db.AncientRange(ChainFreezerHeaderTable, i+1-count, count, max)
	if err == nil && uint64(len(data)) == count {
		// the data is on the order [h, h+1, .., n] -- reordering needed
		for i := range data {
			rlpHeaders = append(rlpHeaders, data[len(data)-1-i])
		}
	}
	return rlpHeaders
}

// ReadHeaderRLP retrieves a block header in its raw RLP database encoding.
func ReadHeaderRLP(db ethdb.Reader, hash common.Hash, number uint64) rlp.RawValue {
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		// First try to look up the data in ancient database. Extra hash
		// comparison is necessary since ancient database only maintains
		// the canonical data.
		data, _ = reader.Ancient(ChainFreezerHeaderTable, number)
		if len(data) > 0 && crypto.Keccak256Hash(data) == hash {
			return nil
		}
		// If not, try reading from leveldb
		data, _ = db.Get(headerKey(number, hash))
		return nil
	})
	return data
}

// HasHeader verifies the existence of a block header corresponding to the hash.
func HasHeader(db ethdb.Reader, hash common.Hash, number uint64) bool {
	if isCanon(db, number, hash) {
		return true
	}
	if has, err := db.Has(headerKey(number, hash)); !has || err != nil {
		return false
	}
	return true
}

// ReadHeader retrieves the block header corresponding to the hash.
func ReadHeader(db ethdb.Reader, hash common.Hash, number uint64) *types.Header {
	data := ReadHeaderRLP(db, hash, number)
	if len(data) == 0 {
		return nil
	}
	header := new(types.Header)
	if err := rlp.DecodeBytes(data, header); err != nil {
		log.Error("Invalid block header RLP", "hash", hash, "err", err)
		return nil
	}
	return header
}

// WriteHeader stores a block header into the database and also stores the hash-
// to-number mapping.
func WriteHeader(db ethdb.KeyValueWriter, header *types.Header) {
	var (
		hash   = header.Hash()
		number = header.Number.Uint64()
	)
	// Write the hash -> number mapping
	WriteHeaderNumber(db, hash, number)

	// Write the encoded header
	data, err := rlp.EncodeToBytes(header)
	if err != nil {
		log.Crit("Failed to RLP encode header", "err", err)
	}
	key := headerKey(number, hash)
	if err := db.Put(key, data); err != nil {
		log.Crit("Failed to store header", "err", err)
	}
}

// DeleteHeader removes all block header data associated with a hash.
func DeleteHeader(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	deleteHeaderWithoutNumber(db, hash, number)
	if err := db.Delete(headerNumberKey(hash)); err != nil {
		log.Crit("Failed to delete hash to number mapping", "err", err)
	}
}

// deleteHeaderWithoutNumber removes only the block header but does not remove
// the hash to number mapping.
func deleteHeaderWithoutNumber(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	if err := db.Delete(headerKey(number, hash)); err != nil {
		log.Crit("Failed to delete header", "err", err)
	}
}

// isCanon is an internal utility method, to check whether the given number/hash
// is part of the ancient (canon) set.
func isCanon(reader ethdb.AncientReaderOp, number uint64, hash common.Hash) bool {
	h, err := reader.Ancient(ChainFreezerHashTable, number)
	if err != nil {
		return false
	}
	return bytes.Equal(h, hash[:])
}

// ReadBodyRLP retrieves the block body (transactions and uncles) in RLP encoding.
func ReadBodyRLP(db ethdb.Reader, hash common.Hash, number uint64) rlp.RawValue {
	// First try to look up the data in ancient database. Extra hash
	// comparison is necessary since ancient database only maintains
	// the canonical data.
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		// Check if the data is in ancients
		if isCanon(reader, number, hash) {
			data, _ = reader.Ancient(ChainFreezerBodiesTable, number)
			return nil
		}
		// If not, try reading from leveldb
		data, _ = db.Get(blockBodyKey(number, hash))
		return nil
	})
	return data
}

// ReadCanonicalBodyRLP retrieves the block body (transactions and uncles) for the canonical
// block at number, in RLP encoding.
func ReadCanonicalBodyRLP(db ethdb.Reader, number uint64) rlp.RawValue {
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		data, _ = reader.Ancient(ChainFreezerBodiesTable, number)
		if len(data) > 0 {
			return nil
		}
		// Block is not in ancients, read from leveldb by hash and number.
		// Note: ReadCanonicalHash cannot be used here because it also
		// calls ReadAncients internally.
		hash, _ := db.Get(headerHashKey(number))
		data, _ = db.Get(blockBodyKey(number, common.BytesToHash(hash)))
		return nil
	})
	return data
}

// WriteBodyRLP stores an RLP encoded block body into the database.
func WriteBodyRLP(db ethdb.KeyValueWriter, hash common.Hash, number uint64, rlp rlp.RawValue) {
	if err := db.Put(blockBodyKey(number, hash), rlp); err != nil {
		log.Crit("Failed to store block body", "err", err)
	}
}

// HasBody verifies the existence of a block body corresponding to the hash.
func HasBody(db ethdb.Reader, hash common.Hash, number uint64) bool {
	if isCanon(db, number, hash) {
		return true
	}
	if has, err := db.Has(blockBodyKey(number, hash)); !has || err != nil {
		return false
	}
	return true
}

// ReadBody retrieves the block body corresponding to the hash.
func ReadBody(db ethdb.Reader, hash common.Hash, number uint64) *types.Body {
	data := ReadBodyRLP(db, hash, number)
	if len(data) == 0 {
		return nil
	}
	body := new(types.Body)
	if err := rlp.DecodeBytes(data, body); err != nil {
		log.Error("Invalid block body RLP", "hash", hash, "err", err)
		return nil
	}
	return body
}

// WriteBody stores a block body into the database.
func WriteBody(db ethdb.KeyValueWriter, hash common.Hash, number uint64, body *types.Body) {
	data, err := rlp.EncodeToBytes(body)
	if err != nil {
		log.Crit("Failed to RLP encode body", "err", err)
	}
	WriteBodyRLP(db, hash, number, data)
}

// DeleteBody removes all block body data associated with a hash.
func DeleteBody(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	if err := db.Delete(blockBodyKey(number, hash)); err != nil {
		log.Crit("Failed to delete block body", "err", err)
	}
}

// ReadTdRLP retrieves a block's total difficulty corresponding to the hash in RLP encoding.
func ReadTdRLP(db ethdb.Reader, hash common.Hash, number uint64) rlp.RawValue {
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		// Check if the data is in ancients
		if isCanon(reader, number, hash) {
			data, _ = reader.Ancient(ChainFreezerDifficultyTable, number)
			return nil
		}
		// If not, try reading from leveldb
		data, _ = db.Get(headerTDKey(number, hash))
		return nil
	})
	return data
}

// ReadTd retrieves a block's total difficulty corresponding to the hash.
func ReadTd(db ethdb.Reader, hash common.Hash, number uint64) *big.Int {
	data := ReadTdRLP(db, hash, number)
	if len(data) == 0 {
		return nil
	}
	td := new(big.Int)
	if err := rlp.DecodeBytes(data, td); err != nil {
		log.Error("Invalid block total difficulty RLP", "hash", hash, "err", err)
		return nil
	}
	return td
}

// WriteTd stores the total difficulty of a block into the database.
func WriteTd(db ethdb.KeyValueWriter, hash common.Hash, number uint64, td *big.Int) {
	data, err := rlp.EncodeToBytes(td)
	if err != nil {
		log.Crit("Failed to RLP encode block total difficulty", "err", err)
	}
	if err := db.Put(headerTDKey(number, hash), data); err != nil {
		log.Crit("Failed to store block total difficulty", "err", err)
	}
}

// DeleteTd removes all block total difficulty data associated with a hash.
func DeleteTd(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	if err := db.Delete(headerTDKey(number, hash)); err != nil {
		log.Crit("Failed to delete block total difficulty", "err", err)
	}
}

// HasReceipts verifies the existence of all the transaction receipts belonging
// to a block.
func HasReceipts(db ethdb.Reader, hash common.Hash, number uint64) bool {
	if isCanon(db, number, hash) {
		return true
	}
	if has, err := db.Has(blockReceiptsKey(number, hash)); !has || err != nil {
		return false
	}
	return true
}

// ReadReceiptsRLP retrieves all the transaction receipts belonging to a block in RLP encoding.
func ReadReceiptsRLP(db ethdb.Reader, hash common.Hash, number uint64) rlp.RawValue {
	var data []byte
	db.ReadAncients(func(reader ethdb.AncientReaderOp) error {
		// Check if the data is in ancients
		if isCanon(reader, number, hash) {
			data, _ = reader.Ancient(ChainFreezerReceiptTable, number)
			return nil
		}
		// If not, try reading from leveldb
		data, _ = db.Get(blockReceiptsKey(number, hash))
		return nil
	})
	return data
}

// ReadRawReceipts retrieves all the transaction receipts belonging to a block.
// The receipt metadata fields are not guaranteed to be populated, so they
// should not be used. Use ReadReceipts instead if the metadata is needed.
func ReadRawReceipts(db ethdb.Reader, hash common.Hash, number uint64) types.Receipts {
	// Retrieve the flattened receipt slice
	data := ReadReceiptsRLP(db, hash, number)
	if len(data) == 0 {
		return nil
	}
	// Convert the receipts from their storage form to their internal representation
	storageReceipts := []*types.ReceiptForStorage{}
	if err := rlp.DecodeBytes(data, &storageReceipts); err != nil {
		log.Error("Invalid receipt array RLP", "hash", hash, "err", err)
		return nil
	}
	receipts := make(types.Receipts, len(storageReceipts))
	for i, storageReceipt := range storageReceipts {
		receipts[i] = (*types.Receipt)(storageReceipt)
	}
	return receipts
}

// ReadReceipts retrieves all the transaction receipts belonging to a block, including
// its corresponding metadata fields. If it is unable to populate these metadata
// fields then nil is returned.
//
// The current implementation populates these metadata fields by reading the receipts'
// corresponding block body, so if the block body is not found it will return nil even
// if the receipt itself is stored.
func ReadReceipts(db ethdb.Reader, hash common.Hash, number uint64, time uint64, config *params.ChainConfig) types.Receipts {
	// We're deriving many fields from the block body, retrieve beside the receipt
	receipts := ReadRawReceipts(db, hash, number)
	if receipts == nil {
		return nil
	}
	body := ReadBody(db, hash, number)
	if body == nil {
		log.Error("Missing body but have receipt", "hash", hash, "number", number)
		return nil
	}
	header := ReadHeader(db, hash, number)

	var baseFee *big.Int
	if header == nil {
		baseFee = big.NewInt(0)
	} else {
		baseFee = header.BaseFee
	}
	// Compute effective blob gas price.
	var blobGasPrice *big.Int
	if header != nil && header.ExcessBlobGas != nil {
		blobGasPrice = eip4844.CalcBlobFee(*header.ExcessBlobGas)
	}
	if err := receipts.DeriveFields(config, hash, number, time, baseFee, blobGasPrice, body.Transactions); err != nil {
		log.Error("Failed to derive block receipts fields", "hash", hash, "number", number, "err", err)
		return nil
	}
	return receipts
}

// WriteReceipts stores all the transaction receipts belonging to a block.
func WriteReceipts(db ethdb.KeyValueWriter, hash common.Hash, number uint64, receipts types.Receipts) {
	// Convert the receipts into their storage form and serialize them
	storageReceipts := make([]*types.ReceiptForStorage, len(receipts))
	for i, receipt := range receipts {
		storageReceipts[i] = (*types.ReceiptForStorage)(receipt)
	}
	bytes, err := rlp.EncodeToBytes(storageReceipts)
	if err != nil {
		log.Crit("Failed to encode block receipts", "err", err)
	}
	// Store the flattened receipt slice
	if err := db.Put(blockReceiptsKey(number, hash), bytes); err != nil {
		log.Crit("Failed to store block receipts", "err", err)
	}
}

// DeleteReceipts removes all receipt data associated with a block hash.
func DeleteReceipts(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	if err := db.Delete(blockReceiptsKey(number, hash)); err != nil {
		log.Crit("Failed to delete block receipts", "err", err)
	}
}

// storedReceiptRLP is the storage encoding of a receipt.
// Re-definition in core/types/receipt.go.
// TODO: Re-use the existing definition.
type storedReceiptRLP struct {
	PostStateOrStatus []byte
	CumulativeGasUsed uint64
	Logs              []*types.Log
}

// ReceiptLogs is a barebone version of ReceiptForStorage which only keeps
// the list of logs. When decoding a stored receipt into this object we
// avoid creating the bloom filter.
type receiptLogs struct {
	Logs []*types.Log
}

// DecodeRLP implements rlp.Decoder.
func (r *receiptLogs) DecodeRLP(s *rlp.Stream) error {
	var stored storedReceiptRLP
	if err := s.Decode(&stored); err != nil {
		return err
	}
	r.Logs = stored.Logs
	return nil
}

// DeriveLogFields fills the logs in receiptLogs with information such as block number, txhash, etc.
func deriveLogFields(receipts []*receiptLogs, hash common.Hash, number uint64, txs types.Transactions) error {
	logIndex := uint(0)
	if len(txs) != len(receipts) {
		return errors.New("transaction and receipt count mismatch")
	}
	for i := 0; i < len(receipts); i++ {
		txHash := txs[i].Hash()
		// The derived log fields can simply be set from the block and transaction
		for j := 0; j < len(receipts[i].Logs); j++ {
			receipts[i].Logs[j].BlockNumber = number
			receipts[i].Logs[j].BlockHash = hash
			receipts[i].Logs[j].TxHash = txHash
			receipts[i].Logs[j].TxIndex = uint(i)
			receipts[i].Logs[j].Index = logIndex
			logIndex++
		}
	}
	return nil
}

// ReadLogs retrieves the logs for all transactions in a block. In case
// receipts is not found, a nil is returned.
// Note: ReadLogs does not derive unstored log fields.
func ReadLogs(db ethdb.Reader, hash common.Hash, number uint64) [][]*types.Log {
	// Retrieve the flattened receipt slice
	data := ReadReceiptsRLP(db, hash, number)
	if len(data) == 0 {
		return nil
	}
	receipts := []*receiptLogs{}
	if err := rlp.DecodeBytes(data, &receipts); err != nil {
		log.Error("Invalid receipt array RLP", "hash", hash, "err", err)
		return nil
	}

	logs := make([][]*types.Log, len(receipts))
	for i, receipt := range receipts {
		logs[i] = receipt.Logs
	}
	return logs
}

// ReadBlock retrieves an entire block corresponding to the hash, assembling it
// back from the stored header and body. If either the header or body could not
// be retrieved nil is returned.
//
// Note, due to concurrent download of header and block body the header and thus
// canonical hash can be stored in the database but the body data not (yet).
func ReadBlock(db ethdb.Reader, hash common.Hash, number uint64) *types.Block {
	header := ReadHeader(db, hash, number)
	if header == nil {
		return nil
	}
	body := ReadBody(db, hash, number)
	if body == nil {
		return nil
	}
	return types.NewBlockWithHeader(header).WithBody(body.Transactions, body.Uncles).WithWithdrawals(body.Withdrawals)
}

// WriteBlock serializes a block into the database, header and body separately.
func WriteBlock(db ethdb.KeyValueWriter, block *types.Block) {
	WriteBody(db, block.Hash(), block.NumberU64(), block.Body())
	WriteHeader(db, block.Header())
}

// WriteAncientBlocks writes entire block data into ancient store and returns the total written size.
func WriteAncientBlocks(db ethdb.AncientWriter, blocks []*types.Block, receipts []types.Receipts, td *big.Int) (int64, error) {
	var (
		tdSum      = new(big.Int).Set(td)
		stReceipts []*types.ReceiptForStorage
	)
	return db.ModifyAncients(func(op ethdb.AncientWriteOp) error {
		for i, block := range blocks {
			// Convert receipts to storage format and sum up total difficulty.
			stReceipts = stReceipts[:0]
			for _, receipt := range receipts[i] {
				stReceipts = append(stReceipts, (*types.ReceiptForStorage)(receipt))
			}
			header := block.Header()
			if i > 0 {
				tdSum.Add(tdSum, header.Difficulty)
			}
			if err := writeAncientBlock(op, block, header, stReceipts, tdSum); err != nil {
				return err
			}
		}
		return nil
	})
}

func writeAncientBlock(op ethdb.AncientWriteOp, block *types.Block, header *types.Header, receipts []*types.ReceiptForStorage, td *big.Int) error {
	num := block.NumberU64()
	if err := op.AppendRaw(ChainFreezerHashTable, num, block.Hash().Bytes()); err != nil {
		return fmt.Errorf("can't add block %d hash: %v", num, err)
	}
	if err := op.Append(ChainFreezerHeaderTable, num, header); err != nil {
		return fmt.Errorf("can't append block header %d: %v", num, err)
	}
	if err := op.Append(ChainFreezerBodiesTable, num, block.Body()); err != nil {
		return fmt.Errorf("can't append block body %d: %v", num, err)
	}
	if err := op.Append(ChainFreezerReceiptTable, num, receipts); err != nil {
		return fmt.Errorf("can't append block %d receipts: %v", num, err)
	}
	if err := op.Append(ChainFreezerDifficultyTable, num, td); err != nil {
		return fmt.Errorf("can't append block %d total difficulty: %v", num, err)
	}
	return nil
}

// DeleteBlock removes all block data associated with a hash.
func DeleteBlock(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	DeleteReceipts(db, hash, number)
	DeleteHeader(db, hash, number)
	DeleteBody(db, hash, number)
	DeleteTd(db, hash, number)
}

// DeleteBlockWithoutNumber removes all block data associated with a hash, except
// the hash to number mapping.
func DeleteBlockWithoutNumber(db ethdb.KeyValueWriter, hash common.Hash, number uint64) {
	DeleteReceipts(db, hash, number)
	deleteHeaderWithoutNumber(db, hash, number)
	DeleteBody(db, hash, number)
	DeleteTd(db, hash, number)
}

const badBlockToKeep = 10

type badBlock struct {
	Header *types.Header
	Body   *types.Body
}

// ReadBadBlock retrieves the bad block with the corresponding block hash.
func ReadBadBlock(db ethdb.Reader, hash common.Hash) *types.Block {
	blob, err := db.Get(badBlockKey)
	if err != nil {
		return nil
	}
	var badBlocks []*badBlock
	if err := rlp.DecodeBytes(blob, &badBlocks); err != nil {
		return nil
	}
	for _, bad := range badBlocks {
		if bad.Header.Hash() == hash {
			return types.NewBlockWithHeader(bad.Header).WithBody(bad.Body.Transactions, bad.Body.Uncles).WithWithdrawals(bad.Body.Withdrawals)
		}
	}
	return nil
}

// ReadAllBadBlocks retrieves all the bad blocks in the database.
// All returned blocks are sorted in reverse order by number.
func ReadAllBadBlocks(db ethdb.Reader) []*types.Block {
	blob, err := db.Get(badBlockKey)
	if err != nil {
		return nil
	}
	var badBlocks []*badBlock
	if err := rlp.DecodeBytes(blob, &badBlocks); err != nil {
		return nil
	}
	var blocks []*types.Block
	for _, bad := range badBlocks {
		blocks = append(blocks, types.NewBlockWithHeader(bad.Header).WithBody(bad.Body.Transactions, bad.Body.Uncles).WithWithdrawals(bad.Body.Withdrawals))
	}
	return blocks
}

// WriteBadBlock serializes the bad block into the database. If the cumulated
// bad blocks exceeds the limitation, the oldest will be dropped.
func WriteBadBlock(db ethdb.KeyValueStore, block *types.Block) {
	blob, err := db.Get(badBlockKey)
	if err != nil {
		log.Warn("Failed to load old bad blocks", "error", err)
	}
	var badBlocks []*badBlock
	if len(blob) > 0 {
		if err := rlp.DecodeBytes(blob, &badBlocks); err != nil {
			log.Crit("Failed to decode old bad blocks", "error", err)
		}
	}
	for _, b := range badBlocks {
		if b.Header.Number.Uint64() == block.NumberU64() && b.Header.Hash() == block.Hash() {
			log.Info("Skip duplicated bad block", "number", block.NumberU64(), "hash", block.Hash())
			return
		}
	}
	badBlocks = append(badBlocks, &badBlock{
		Header: block.Header(),
		Body:   block.Body(),
	})
	slices.SortFunc(badBlocks, func(a, b *badBlock) int {
		// Note: sorting in descending number order.
		return -a.Header.Number.Cmp(b.Header.Number)
	})
	if len(badBlocks) > badBlockToKeep {
		badBlocks = badBlocks[:badBlockToKeep]
	}
	data, err := rlp.EncodeToBytes(badBlocks)
	if err != nil {
		log.Crit("Failed to encode bad blocks", "err", err)
	}
	if err := db.Put(badBlockKey, data); err != nil {
		log.Crit("Failed to write bad blocks", "err", err)
	}
}

// DeleteBadBlocks deletes all the bad blocks from the database
func DeleteBadBlocks(db ethdb.KeyValueWriter) {
	if err := db.Delete(badBlockKey); err != nil {
		log.Crit("Failed to delete bad blocks", "err", err)
	}
}

// FindCommonAncestor returns the last common ancestor of two block headers
func FindCommonAncestor(db ethdb.Reader, a, b *types.Header) *types.Header {
	for bn := b.Number.Uint64(); a.Number.Uint64() > bn; {
		a = ReadHeader(db, a.ParentHash, a.Number.Uint64()-1)
		if a == nil {
			return nil
		}
	}
	for an := a.Number.Uint64(); an < b.Number.Uint64(); {
		b = ReadHeader(db, b.ParentHash, b.Number.Uint64()-1)
		if b == nil {
			return nil
		}
	}
	for a.Hash() != b.Hash() {
		a = ReadHeader(db, a.ParentHash, a.Number.Uint64()-1)
		if a == nil {
			return nil
		}
		b = ReadHeader(db, b.ParentHash, b.Number.Uint64()-1)
		if b == nil {
			return nil
		}
	}
	return a
}

// ReadHeadHeader returns the current canonical head header.
func ReadHeadHeader(db ethdb.Reader) *types.Header {
	headHeaderHash := ReadHeadHeaderHash(db)
	if headHeaderHash == (common.Hash{}) {
		return nil
	}
	headHeaderNumber := ReadHeaderNumber(db, headHeaderHash)
	if headHeaderNumber == nil {
		return nil
	}
	return ReadHeader(db, headHeaderHash, *headHeaderNumber)
}

// ReadHeadBlock returns the current canonical head block.
func ReadHeadBlock(db ethdb.Reader) *types.Block {
	headBlockHash := ReadHeadBlockHash(db)
	if headBlockHash == (common.Hash{}) {
		return nil
	}
	headBlockNumber := ReadHeaderNumber(db, headBlockHash)
	if headBlockNumber == nil {
		return nil
	}
	return ReadBlock(db, headBlockHash, *headBlockNumber)
}
