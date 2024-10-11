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

package light

import (
	"context"
	"errors"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus/misc/eip4844"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
)

// errNonCanonicalHash is returned if the requested chain data doesn't belong
// to the canonical chain. ODR can only retrieve the canonical chain data covered
// by the CHT or Bloom trie for verification.
var errNonCanonicalHash = errors.New("hash is not currently canonical")

// GetHeaderByNumber retrieves the canonical block header corresponding to the
// given number. The returned header is proven by local CHT.
func GetHeaderByNumber(ctx context.Context, odr OdrBackend, number uint64) (*types.Header, error) {
	// Try to find it in the local database first.
	db := odr.Database()
	hash := rawdb.ReadCanonicalHash(db, number)

	// If there is a canonical hash, there should have a header too.
	// But if it's pruned, re-fetch from network again.
	if (hash != common.Hash{}) {
		if header := rawdb.ReadHeader(db, hash, number); header != nil {
			return header, nil
		}
	}
	// Retrieve the header via ODR, ensure the requested header is covered
	// by local trusted CHT.
	chts, _, chtHead := odr.ChtIndexer().Sections()
	if number >= chts*odr.IndexerConfig().ChtSize {
		return nil, errNoTrustedCht
	}
	r := &ChtRequest{
		ChtRoot:  GetChtRoot(db, chts-1, chtHead),
		ChtNum:   chts - 1,
		BlockNum: number,
		Config:   odr.IndexerConfig(),
	}
	if err := odr.Retrieve(ctx, r); err != nil {
		return nil, err
	}
	return r.Header, nil
}

// GetCanonicalHash retrieves the canonical block hash corresponding to the number.
func GetCanonicalHash(ctx context.Context, odr OdrBackend, number uint64) (common.Hash, error) {
	hash := rawdb.ReadCanonicalHash(odr.Database(), number)
	if hash != (common.Hash{}) {
		return hash, nil
	}
	header, err := GetHeaderByNumber(ctx, odr, number)
	if err != nil {
		return common.Hash{}, err
	}
	// number -> canonical mapping already be stored in db, get it.
	return header.Hash(), nil
}

// GetTd retrieves the total difficulty corresponding to the number and hash.
func GetTd(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) (*big.Int, error) {
	td := rawdb.ReadTd(odr.Database(), hash, number)
	if td != nil {
		return td, nil
	}
	header, err := GetHeaderByNumber(ctx, odr, number)
	if err != nil {
		return nil, err
	}
	if header.Hash() != hash {
		return nil, errNonCanonicalHash
	}
	// <hash, number> -> td mapping already be stored in db, get it.
	return rawdb.ReadTd(odr.Database(), hash, number), nil
}

// GetBodyRLP retrieves the block body (transactions and uncles) in RLP encoding.
func GetBodyRLP(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) (rlp.RawValue, error) {
	if data := rawdb.ReadBodyRLP(odr.Database(), hash, number); data != nil {
		return data, nil
	}
	// Retrieve the block header first and pass it for verification.
	header, err := GetHeaderByNumber(ctx, odr, number)
	if err != nil {
		return nil, errNoHeader
	}
	if header.Hash() != hash {
		return nil, errNonCanonicalHash
	}
	r := &BlockRequest{Hash: hash, Number: number, Header: header}
	if err := odr.Retrieve(ctx, r); err != nil {
		return nil, err
	}
	return r.Rlp, nil
}

// GetBody retrieves the block body (transactions, uncles) corresponding to the
// hash.
func GetBody(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) (*types.Body, error) {
	data, err := GetBodyRLP(ctx, odr, hash, number)
	if err != nil {
		return nil, err
	}
	body := new(types.Body)
	if err := rlp.DecodeBytes(data, body); err != nil {
		return nil, err
	}
	return body, nil
}

// GetBlock retrieves an entire block corresponding to the hash, assembling it
// back from the stored header and body.
func GetBlock(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) (*types.Block, error) {
	// Retrieve the block header and body contents
	header, err := GetHeaderByNumber(ctx, odr, number)
	if err != nil {
		return nil, errNoHeader
	}
	body, err := GetBody(ctx, odr, hash, number)
	if err != nil {
		return nil, err
	}
	// Reassemble the block and return
	return types.NewBlockWithHeader(header).WithBody(body.Transactions, body.Uncles), nil
}

// GetBlockReceipts retrieves the receipts generated by the transactions included
// in a block given by its hash. Receipts will be filled in with context data.
func GetBlockReceipts(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) (types.Receipts, error) {
	// Assume receipts are already stored locally and attempt to retrieve.
	receipts := rawdb.ReadRawReceipts(odr.Database(), hash, number)
	if receipts == nil {
		header, err := GetHeaderByNumber(ctx, odr, number)
		if err != nil {
			return nil, errNoHeader
		}
		if header.Hash() != hash {
			return nil, errNonCanonicalHash
		}
		r := &ReceiptsRequest{Hash: hash, Number: number, Header: header}
		if err := odr.Retrieve(ctx, r); err != nil {
			return nil, err
		}
		receipts = r.Receipts
	}
	// If the receipts are incomplete, fill the derived fields
	if len(receipts) > 0 && receipts[0].TxHash == (common.Hash{}) {
		block, err := GetBlock(ctx, odr, hash, number)
		if err != nil {
			return nil, err
		}
		genesis := rawdb.ReadCanonicalHash(odr.Database(), 0)
		config := rawdb.ReadChainConfig(odr.Database(), genesis)

		var blobGasPrice *big.Int
		excessBlobGas := block.ExcessBlobGas()
		if excessBlobGas != nil {
			blobGasPrice = eip4844.CalcBlobFee(*excessBlobGas)
		}

		if err := receipts.DeriveFields(config, block.Hash(), block.NumberU64(), block.Time(), block.BaseFee(), blobGasPrice, block.Transactions()); err != nil {
			return nil, err
		}
		rawdb.WriteReceipts(odr.Database(), hash, number, receipts)
	}
	return receipts, nil
}

// GetBlockLogs retrieves the logs generated by the transactions included in a
// block given by its hash. Logs will be filled in with context data.
func GetBlockLogs(ctx context.Context, odr OdrBackend, hash common.Hash, number uint64) ([][]*types.Log, error) {
	receipts, err := GetBlockReceipts(ctx, odr, hash, number)
	if err != nil {
		return nil, err
	}
	logs := make([][]*types.Log, len(receipts))
	for i, receipt := range receipts {
		logs[i] = receipt.Logs
	}
	return logs, nil
}

// GetBloomBits retrieves a batch of compressed bloomBits vectors belonging to
// the given bit index and section indexes.
func GetBloomBits(ctx context.Context, odr OdrBackend, bit uint, sections []uint64) ([][]byte, error) {
	var (
		reqIndex    []int
		reqSections []uint64
		db          = odr.Database()
		result      = make([][]byte, len(sections))
	)
	blooms, _, sectionHead := odr.BloomTrieIndexer().Sections()
	for i, section := range sections {
		sectionHead := rawdb.ReadCanonicalHash(db, (section+1)*odr.IndexerConfig().BloomSize-1)
		// If we don't have the canonical hash stored for this section head number,
		// we'll still look for an entry with a zero sectionHead (we store it with
		// zero section head too if we don't know it at the time of the retrieval)
		if bloomBits, _ := rawdb.ReadBloomBits(db, bit, section, sectionHead); len(bloomBits) != 0 {
			result[i] = bloomBits
			continue
		}
		// TODO(rjl493456442) Convert sectionIndex to BloomTrie relative index
		if section >= blooms {
			return nil, errNoTrustedBloomTrie
		}
		reqSections = append(reqSections, section)
		reqIndex = append(reqIndex, i)
	}
	// Find all bloombits in database, nothing to query via odr, return.
	if reqSections == nil {
		return result, nil
	}
	// Send odr request to retrieve missing bloombits.
	r := &BloomRequest{
		BloomTrieRoot:    GetBloomTrieRoot(db, blooms-1, sectionHead),
		BloomTrieNum:     blooms - 1,
		BitIdx:           bit,
		SectionIndexList: reqSections,
		Config:           odr.IndexerConfig(),
	}
	if err := odr.Retrieve(ctx, r); err != nil {
		return nil, err
	}
	for i, idx := range reqIndex {
		result[idx] = r.BloomBits[i]
	}
	return result, nil
}

// GetTransaction retrieves a canonical transaction by hash and also returns
// its position in the chain. There is no guarantee in the LES protocol that
// the mined transaction will be retrieved back for sure because of different
// reasons(the transaction is unindexed, the malicious server doesn't reply it
// deliberately, etc). Therefore, unretrieved transactions will receive a certain
// number of retries, thus giving a weak guarantee.
func GetTransaction(ctx context.Context, odr OdrBackend, txHash common.Hash) (*types.Transaction, common.Hash, uint64, uint64, error) {
	r := &TxStatusRequest{Hashes: []common.Hash{txHash}}
	if err := odr.RetrieveTxStatus(ctx, r); err != nil || r.Status[0].Status != txpool.TxStatusIncluded {
		return nil, common.Hash{}, 0, 0, err
	}
	pos := r.Status[0].Lookup
	// first ensure that we have the header, otherwise block body retrieval will fail
	// also verify if this is a canonical block by getting the header by number and checking its hash
	if header, err := GetHeaderByNumber(ctx, odr, pos.BlockIndex); err != nil || header.Hash() != pos.BlockHash {
		return nil, common.Hash{}, 0, 0, err
	}
	body, err := GetBody(ctx, odr, pos.BlockHash, pos.BlockIndex)
	if err != nil || uint64(len(body.Transactions)) <= pos.Index || body.Transactions[pos.Index].Hash() != txHash {
		return nil, common.Hash{}, 0, 0, err
	}
	return body.Transactions[pos.Index], pos.BlockHash, pos.BlockIndex, pos.Index, nil
}
