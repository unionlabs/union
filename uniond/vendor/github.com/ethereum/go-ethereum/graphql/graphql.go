// Copyright 2019 The go-ethereum Authors
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

// Package graphql provides a GraphQL interface to Ethereum node data.
package graphql

import (
	"context"
	"errors"
	"fmt"
	"math/big"
	"sort"
	"strconv"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/common/math"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/eth/filters"
	"github.com/ethereum/go-ethereum/internal/ethapi"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/rpc"
)

var (
	errBlockInvariant    = errors.New("block objects must be instantiated with at least one of num or hash")
	errInvalidBlockRange = errors.New("invalid from and to block combination: from > to")
)

type Long int64

// ImplementsGraphQLType returns true if Long implements the provided GraphQL type.
func (b Long) ImplementsGraphQLType(name string) bool { return name == "Long" }

// UnmarshalGraphQL unmarshals the provided GraphQL query data.
func (b *Long) UnmarshalGraphQL(input interface{}) error {
	var err error
	switch input := input.(type) {
	case string:
		// uncomment to support hex values
		if strings.HasPrefix(input, "0x") {
			// apply leniency and support hex representations of longs.
			value, err := hexutil.DecodeUint64(input)
			*b = Long(value)
			return err
		} else {
			value, err := strconv.ParseInt(input, 10, 64)
			*b = Long(value)
			return err
		}
	case int32:
		*b = Long(input)
	case int64:
		*b = Long(input)
	case float64:
		*b = Long(input)
	default:
		err = fmt.Errorf("unexpected type %T for Long", input)
	}
	return err
}

// Account represents an Ethereum account at a particular block.
type Account struct {
	r             *Resolver
	address       common.Address
	blockNrOrHash rpc.BlockNumberOrHash
}

// getState fetches the StateDB object for an account.
func (a *Account) getState(ctx context.Context) (*state.StateDB, error) {
	state, _, err := a.r.backend.StateAndHeaderByNumberOrHash(ctx, a.blockNrOrHash)
	return state, err
}

func (a *Account) Address(ctx context.Context) (common.Address, error) {
	return a.address, nil
}

func (a *Account) Balance(ctx context.Context) (hexutil.Big, error) {
	state, err := a.getState(ctx)
	if err != nil {
		return hexutil.Big{}, err
	}
	balance := state.GetBalance(a.address)
	if balance == nil {
		return hexutil.Big{}, fmt.Errorf("failed to load balance %x", a.address)
	}
	return hexutil.Big(*balance), nil
}

func (a *Account) TransactionCount(ctx context.Context) (hexutil.Uint64, error) {
	// Ask transaction pool for the nonce which includes pending transactions
	if blockNr, ok := a.blockNrOrHash.Number(); ok && blockNr == rpc.PendingBlockNumber {
		nonce, err := a.r.backend.GetPoolNonce(ctx, a.address)
		if err != nil {
			return 0, err
		}
		return hexutil.Uint64(nonce), nil
	}
	state, err := a.getState(ctx)
	if err != nil {
		return 0, err
	}
	return hexutil.Uint64(state.GetNonce(a.address)), nil
}

func (a *Account) Code(ctx context.Context) (hexutil.Bytes, error) {
	state, err := a.getState(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return state.GetCode(a.address), nil
}

func (a *Account) Storage(ctx context.Context, args struct{ Slot common.Hash }) (common.Hash, error) {
	state, err := a.getState(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return state.GetState(a.address, args.Slot), nil
}

// Log represents an individual log message. All arguments are mandatory.
type Log struct {
	r           *Resolver
	transaction *Transaction
	log         *types.Log
}

func (l *Log) Transaction(ctx context.Context) *Transaction {
	return l.transaction
}

func (l *Log) Account(ctx context.Context, args BlockNumberArgs) *Account {
	return &Account{
		r:             l.r,
		address:       l.log.Address,
		blockNrOrHash: args.NumberOrLatest(),
	}
}

func (l *Log) Index(ctx context.Context) hexutil.Uint64 {
	return hexutil.Uint64(l.log.Index)
}

func (l *Log) Topics(ctx context.Context) []common.Hash {
	return l.log.Topics
}

func (l *Log) Data(ctx context.Context) hexutil.Bytes {
	return l.log.Data
}

// AccessTuple represents EIP-2930
type AccessTuple struct {
	address     common.Address
	storageKeys []common.Hash
}

func (at *AccessTuple) Address(ctx context.Context) common.Address {
	return at.address
}

func (at *AccessTuple) StorageKeys(ctx context.Context) []common.Hash {
	return at.storageKeys
}

// Withdrawal represents a withdrawal of value from the beacon chain
// by a validator. For details see EIP-4895.
type Withdrawal struct {
	index     uint64
	validator uint64
	address   common.Address
	amount    uint64
}

func (w *Withdrawal) Index(ctx context.Context) hexutil.Uint64 {
	return hexutil.Uint64(w.index)
}

func (w *Withdrawal) Validator(ctx context.Context) hexutil.Uint64 {
	return hexutil.Uint64(w.validator)
}

func (w *Withdrawal) Address(ctx context.Context) common.Address {
	return w.address
}

func (w *Withdrawal) Amount(ctx context.Context) hexutil.Uint64 {
	return hexutil.Uint64(w.amount)
}

// Transaction represents an Ethereum transaction.
// backend and hash are mandatory; all others will be fetched when required.
type Transaction struct {
	r    *Resolver
	hash common.Hash // Must be present after initialization
	mu   sync.Mutex
	// mu protects following resources
	tx    *types.Transaction
	block *Block
	index uint64
}

// resolve returns the internal transaction object, fetching it if needed.
// It also returns the block the tx belongs to, unless it is a pending tx.
func (t *Transaction) resolve(ctx context.Context) (*types.Transaction, *Block) {
	t.mu.Lock()
	defer t.mu.Unlock()
	if t.tx != nil {
		return t.tx, t.block
	}
	// Try to return an already finalized transaction
	tx, blockHash, _, index, err := t.r.backend.GetTransaction(ctx, t.hash)
	if err == nil && tx != nil {
		t.tx = tx
		blockNrOrHash := rpc.BlockNumberOrHashWithHash(blockHash, false)
		t.block = &Block{
			r:            t.r,
			numberOrHash: &blockNrOrHash,
			hash:         blockHash,
		}
		t.index = index
		return t.tx, t.block
	}
	// No finalized transaction, try to retrieve it from the pool
	t.tx = t.r.backend.GetPoolTransaction(t.hash)
	return t.tx, nil
}

func (t *Transaction) Hash(ctx context.Context) common.Hash {
	return t.hash
}

func (t *Transaction) InputData(ctx context.Context) hexutil.Bytes {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Bytes{}
	}
	return tx.Data()
}

func (t *Transaction) Gas(ctx context.Context) hexutil.Uint64 {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return 0
	}
	return hexutil.Uint64(tx.Gas())
}

func (t *Transaction) GasPrice(ctx context.Context) hexutil.Big {
	tx, block := t.resolve(ctx)
	if tx == nil {
		return hexutil.Big{}
	}
	switch tx.Type() {
	case types.DynamicFeeTxType:
		if block != nil {
			if baseFee, _ := block.BaseFeePerGas(ctx); baseFee != nil {
				// price = min(gasTipCap + baseFee, gasFeeCap)
				return (hexutil.Big)(*math.BigMin(new(big.Int).Add(tx.GasTipCap(), baseFee.ToInt()), tx.GasFeeCap()))
			}
		}
		return hexutil.Big(*tx.GasPrice())
	default:
		return hexutil.Big(*tx.GasPrice())
	}
}

func (t *Transaction) EffectiveGasPrice(ctx context.Context) (*hexutil.Big, error) {
	tx, block := t.resolve(ctx)
	if tx == nil {
		return nil, nil
	}
	// Pending tx
	if block == nil {
		return nil, nil
	}
	header, err := block.resolveHeader(ctx)
	if err != nil || header == nil {
		return nil, err
	}
	if header.BaseFee == nil {
		return (*hexutil.Big)(tx.GasPrice()), nil
	}
	return (*hexutil.Big)(math.BigMin(new(big.Int).Add(tx.GasTipCap(), header.BaseFee), tx.GasFeeCap())), nil
}

func (t *Transaction) MaxFeePerGas(ctx context.Context) *hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	switch tx.Type() {
	case types.DynamicFeeTxType, types.BlobTxType:
		return (*hexutil.Big)(tx.GasFeeCap())
	default:
		return nil
	}
}

func (t *Transaction) MaxPriorityFeePerGas(ctx context.Context) *hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	switch tx.Type() {
	case types.DynamicFeeTxType, types.BlobTxType:
		return (*hexutil.Big)(tx.GasTipCap())
	default:
		return nil
	}
}

func (t *Transaction) MaxFeePerBlobGas(ctx context.Context) *hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	return (*hexutil.Big)(tx.BlobGasFeeCap())
}

func (t *Transaction) BlobVersionedHashes(ctx context.Context) *[]common.Hash {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	if tx.Type() != types.BlobTxType {
		return nil
	}
	blobHashes := tx.BlobHashes()
	return &blobHashes
}

func (t *Transaction) EffectiveTip(ctx context.Context) (*hexutil.Big, error) {
	tx, block := t.resolve(ctx)
	if tx == nil {
		return nil, nil
	}
	// Pending tx
	if block == nil {
		return nil, nil
	}
	header, err := block.resolveHeader(ctx)
	if err != nil || header == nil {
		return nil, err
	}
	if header.BaseFee == nil {
		return (*hexutil.Big)(tx.GasPrice()), nil
	}

	tip, err := tx.EffectiveGasTip(header.BaseFee)
	if err != nil {
		return nil, err
	}
	return (*hexutil.Big)(tip), nil
}

func (t *Transaction) Value(ctx context.Context) (hexutil.Big, error) {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Big{}, nil
	}
	if tx.Value() == nil {
		return hexutil.Big{}, fmt.Errorf("invalid transaction value %x", t.hash)
	}
	return hexutil.Big(*tx.Value()), nil
}

func (t *Transaction) Nonce(ctx context.Context) hexutil.Uint64 {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return 0
	}
	return hexutil.Uint64(tx.Nonce())
}

func (t *Transaction) To(ctx context.Context, args BlockNumberArgs) *Account {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	to := tx.To()
	if to == nil {
		return nil
	}
	return &Account{
		r:             t.r,
		address:       *to,
		blockNrOrHash: args.NumberOrLatest(),
	}
}

func (t *Transaction) From(ctx context.Context, args BlockNumberArgs) *Account {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	signer := types.LatestSigner(t.r.backend.ChainConfig())
	from, _ := types.Sender(signer, tx)
	return &Account{
		r:             t.r,
		address:       from,
		blockNrOrHash: args.NumberOrLatest(),
	}
}

func (t *Transaction) Block(ctx context.Context) *Block {
	_, block := t.resolve(ctx)
	return block
}

func (t *Transaction) Index(ctx context.Context) *hexutil.Uint64 {
	_, block := t.resolve(ctx)
	// Pending tx
	if block == nil {
		return nil
	}
	index := hexutil.Uint64(t.index)
	return &index
}

// getReceipt returns the receipt associated with this transaction, if any.
func (t *Transaction) getReceipt(ctx context.Context) (*types.Receipt, error) {
	_, block := t.resolve(ctx)
	// Pending tx
	if block == nil {
		return nil, nil
	}
	receipts, err := block.resolveReceipts(ctx)
	if err != nil {
		return nil, err
	}
	return receipts[t.index], nil
}

func (t *Transaction) Status(ctx context.Context) (*hexutil.Uint64, error) {
	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return nil, err
	}
	if len(receipt.PostState) != 0 {
		return nil, nil
	}
	ret := hexutil.Uint64(receipt.Status)
	return &ret, nil
}

func (t *Transaction) GasUsed(ctx context.Context) (*hexutil.Uint64, error) {
	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return nil, err
	}
	ret := hexutil.Uint64(receipt.GasUsed)
	return &ret, nil
}

func (t *Transaction) CumulativeGasUsed(ctx context.Context) (*hexutil.Uint64, error) {
	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return nil, err
	}
	ret := hexutil.Uint64(receipt.CumulativeGasUsed)
	return &ret, nil
}

func (t *Transaction) BlobGasUsed(ctx context.Context) (*hexutil.Uint64, error) {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil, nil
	}
	if tx.Type() != types.BlobTxType {
		return nil, nil
	}

	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return nil, err
	}
	ret := hexutil.Uint64(receipt.BlobGasUsed)
	return &ret, nil
}

func (t *Transaction) BlobGasPrice(ctx context.Context) (*hexutil.Big, error) {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil, nil
	}
	if tx.Type() != types.BlobTxType {
		return nil, nil
	}

	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return nil, err
	}
	ret := (*hexutil.Big)(receipt.BlobGasPrice)
	return ret, nil
}

func (t *Transaction) CreatedContract(ctx context.Context, args BlockNumberArgs) (*Account, error) {
	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil || receipt.ContractAddress == (common.Address{}) {
		return nil, err
	}
	return &Account{
		r:             t.r,
		address:       receipt.ContractAddress,
		blockNrOrHash: args.NumberOrLatest(),
	}, nil
}

func (t *Transaction) Logs(ctx context.Context) (*[]*Log, error) {
	_, block := t.resolve(ctx)
	// Pending tx
	if block == nil {
		return nil, nil
	}
	h, err := block.Hash(ctx)
	if err != nil {
		return nil, err
	}
	return t.getLogs(ctx, h)
}

// getLogs returns log objects for the given tx.
// Assumes block hash is resolved.
func (t *Transaction) getLogs(ctx context.Context, hash common.Hash) (*[]*Log, error) {
	var (
		filter    = t.r.filterSystem.NewBlockFilter(hash, nil, nil)
		logs, err = filter.Logs(ctx)
	)
	if err != nil {
		return nil, err
	}
	var ret []*Log
	// Select tx logs from all block logs
	ix := sort.Search(len(logs), func(i int) bool { return uint64(logs[i].TxIndex) >= t.index })
	for ix < len(logs) && uint64(logs[ix].TxIndex) == t.index {
		ret = append(ret, &Log{
			r:           t.r,
			transaction: t,
			log:         logs[ix],
		})
		ix++
	}
	return &ret, nil
}

func (t *Transaction) Type(ctx context.Context) *hexutil.Uint64 {
	tx, _ := t.resolve(ctx)
	txType := hexutil.Uint64(tx.Type())
	return &txType
}

func (t *Transaction) AccessList(ctx context.Context) *[]*AccessTuple {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return nil
	}
	accessList := tx.AccessList()
	ret := make([]*AccessTuple, 0, len(accessList))
	for _, al := range accessList {
		ret = append(ret, &AccessTuple{
			address:     al.Address,
			storageKeys: al.StorageKeys,
		})
	}
	return &ret
}

func (t *Transaction) R(ctx context.Context) hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Big{}
	}
	_, r, _ := tx.RawSignatureValues()
	return hexutil.Big(*r)
}

func (t *Transaction) S(ctx context.Context) hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Big{}
	}
	_, _, s := tx.RawSignatureValues()
	return hexutil.Big(*s)
}

func (t *Transaction) V(ctx context.Context) hexutil.Big {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Big{}
	}
	v, _, _ := tx.RawSignatureValues()
	return hexutil.Big(*v)
}

func (t *Transaction) YParity(ctx context.Context) (*hexutil.Uint64, error) {
	tx, _ := t.resolve(ctx)
	if tx == nil || tx.Type() == types.LegacyTxType {
		return nil, nil
	}
	v, _, _ := tx.RawSignatureValues()
	ret := hexutil.Uint64(v.Int64())
	return &ret, nil
}

func (t *Transaction) Raw(ctx context.Context) (hexutil.Bytes, error) {
	tx, _ := t.resolve(ctx)
	if tx == nil {
		return hexutil.Bytes{}, nil
	}
	return tx.MarshalBinary()
}

func (t *Transaction) RawReceipt(ctx context.Context) (hexutil.Bytes, error) {
	receipt, err := t.getReceipt(ctx)
	if err != nil || receipt == nil {
		return hexutil.Bytes{}, err
	}
	return receipt.MarshalBinary()
}

type BlockType int

// Block represents an Ethereum block.
// backend, and numberOrHash are mandatory. All other fields are lazily fetched
// when required.
type Block struct {
	r            *Resolver
	numberOrHash *rpc.BlockNumberOrHash // Field resolvers assume numberOrHash is always present
	mu           sync.Mutex
	// mu protects following resources
	hash     common.Hash // Must be resolved during initialization
	header   *types.Header
	block    *types.Block
	receipts []*types.Receipt
}

// resolve returns the internal Block object representing this block, fetching
// it if necessary.
func (b *Block) resolve(ctx context.Context) (*types.Block, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.block != nil {
		return b.block, nil
	}
	if b.numberOrHash == nil {
		latest := rpc.BlockNumberOrHashWithNumber(rpc.LatestBlockNumber)
		b.numberOrHash = &latest
	}
	var err error
	b.block, err = b.r.backend.BlockByNumberOrHash(ctx, *b.numberOrHash)
	if b.block != nil {
		b.hash = b.block.Hash()
		if b.header == nil {
			b.header = b.block.Header()
		}
	}
	return b.block, err
}

// resolveHeader returns the internal Header object for this block, fetching it
// if necessary. Call this function instead of `resolve` unless you need the
// additional data (transactions and uncles).
func (b *Block) resolveHeader(ctx context.Context) (*types.Header, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.header != nil {
		return b.header, nil
	}
	if b.numberOrHash == nil && b.hash == (common.Hash{}) {
		return nil, errBlockInvariant
	}
	var err error
	b.header, err = b.r.backend.HeaderByNumberOrHash(ctx, *b.numberOrHash)
	if err != nil {
		return nil, err
	}
	if b.hash == (common.Hash{}) {
		b.hash = b.header.Hash()
	}
	return b.header, nil
}

// resolveReceipts returns the list of receipts for this block, fetching them
// if necessary.
func (b *Block) resolveReceipts(ctx context.Context) ([]*types.Receipt, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.receipts != nil {
		return b.receipts, nil
	}
	receipts, err := b.r.backend.GetReceipts(ctx, b.hash)
	if err != nil {
		return nil, err
	}
	b.receipts = receipts
	return receipts, nil
}

func (b *Block) Number(ctx context.Context) (hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return 0, err
	}

	return hexutil.Uint64(header.Number.Uint64()), nil
}

func (b *Block) Hash(ctx context.Context) (common.Hash, error) {
	b.mu.Lock()
	defer b.mu.Unlock()
	return b.hash, nil
}

func (b *Block) GasLimit(ctx context.Context) (hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return 0, err
	}
	return hexutil.Uint64(header.GasLimit), nil
}

func (b *Block) GasUsed(ctx context.Context) (hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return 0, err
	}
	return hexutil.Uint64(header.GasUsed), nil
}

func (b *Block) BaseFeePerGas(ctx context.Context) (*hexutil.Big, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	if header.BaseFee == nil {
		return nil, nil
	}
	return (*hexutil.Big)(header.BaseFee), nil
}

func (b *Block) NextBaseFeePerGas(ctx context.Context) (*hexutil.Big, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	chaincfg := b.r.backend.ChainConfig()
	if header.BaseFee == nil {
		// Make sure next block doesn't enable EIP-1559
		if !chaincfg.IsLondon(new(big.Int).Add(header.Number, common.Big1)) {
			return nil, nil
		}
	}
	nextBaseFee := eip1559.CalcBaseFee(chaincfg, header)
	return (*hexutil.Big)(nextBaseFee), nil
}

func (b *Block) Parent(ctx context.Context) (*Block, error) {
	if _, err := b.resolveHeader(ctx); err != nil {
		return nil, err
	}
	if b.header == nil || b.header.Number.Uint64() < 1 {
		return nil, nil
	}
	var (
		num       = rpc.BlockNumber(b.header.Number.Uint64() - 1)
		hash      = b.header.ParentHash
		numOrHash = rpc.BlockNumberOrHash{
			BlockNumber: &num,
			BlockHash:   &hash,
		}
	)
	return &Block{
		r:            b.r,
		numberOrHash: &numOrHash,
		hash:         hash,
	}, nil
}

func (b *Block) Difficulty(ctx context.Context) (hexutil.Big, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return hexutil.Big{}, err
	}
	return hexutil.Big(*header.Difficulty), nil
}

func (b *Block) Timestamp(ctx context.Context) (hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return 0, err
	}
	return hexutil.Uint64(header.Time), nil
}

func (b *Block) Nonce(ctx context.Context) (hexutil.Bytes, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return header.Nonce[:], nil
}

func (b *Block) MixHash(ctx context.Context) (common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return header.MixDigest, nil
}

func (b *Block) TransactionsRoot(ctx context.Context) (common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return header.TxHash, nil
}

func (b *Block) StateRoot(ctx context.Context) (common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return header.Root, nil
}

func (b *Block) ReceiptsRoot(ctx context.Context) (common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return header.ReceiptHash, nil
}

func (b *Block) OmmerHash(ctx context.Context) (common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return common.Hash{}, err
	}
	return header.UncleHash, nil
}

func (b *Block) OmmerCount(ctx context.Context) (*hexutil.Uint64, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	count := hexutil.Uint64(len(block.Uncles()))
	return &count, err
}

func (b *Block) Ommers(ctx context.Context) (*[]*Block, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	ret := make([]*Block, 0, len(block.Uncles()))
	for _, uncle := range block.Uncles() {
		blockNumberOrHash := rpc.BlockNumberOrHashWithHash(uncle.Hash(), false)
		ret = append(ret, &Block{
			r:            b.r,
			numberOrHash: &blockNumberOrHash,
			header:       uncle,
			hash:         uncle.Hash(),
		})
	}
	return &ret, nil
}

func (b *Block) ExtraData(ctx context.Context) (hexutil.Bytes, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return header.Extra, nil
}

func (b *Block) LogsBloom(ctx context.Context) (hexutil.Bytes, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return header.Bloom.Bytes(), nil
}

func (b *Block) TotalDifficulty(ctx context.Context) (hexutil.Big, error) {
	hash, err := b.Hash(ctx)
	if err != nil {
		return hexutil.Big{}, err
	}
	td := b.r.backend.GetTd(ctx, hash)
	if td == nil {
		return hexutil.Big{}, fmt.Errorf("total difficulty not found %x", hash)
	}
	return hexutil.Big(*td), nil
}

func (b *Block) RawHeader(ctx context.Context) (hexutil.Bytes, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return rlp.EncodeToBytes(header)
}

func (b *Block) Raw(ctx context.Context) (hexutil.Bytes, error) {
	block, err := b.resolve(ctx)
	if err != nil {
		return hexutil.Bytes{}, err
	}
	return rlp.EncodeToBytes(block)
}

// BlockNumberArgs encapsulates arguments to accessors that specify a block number.
type BlockNumberArgs struct {
	// TODO: Ideally we could use input unions to allow the query to specify the
	// block parameter by hash, block number, or tag but input unions aren't part of the
	// standard GraphQL schema SDL yet, see: https://github.com/graphql/graphql-spec/issues/488
	Block *Long
}

// NumberOr returns the provided block number argument, or the "current" block number or hash if none
// was provided.
func (a BlockNumberArgs) NumberOr(current rpc.BlockNumberOrHash) rpc.BlockNumberOrHash {
	if a.Block != nil {
		blockNr := rpc.BlockNumber(*a.Block)
		return rpc.BlockNumberOrHashWithNumber(blockNr)
	}
	return current
}

// NumberOrLatest returns the provided block number argument, or the "latest" block number if none
// was provided.
func (a BlockNumberArgs) NumberOrLatest() rpc.BlockNumberOrHash {
	return a.NumberOr(rpc.BlockNumberOrHashWithNumber(rpc.LatestBlockNumber))
}

func (b *Block) Miner(ctx context.Context, args BlockNumberArgs) (*Account, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	return &Account{
		r:             b.r,
		address:       header.Coinbase,
		blockNrOrHash: args.NumberOrLatest(),
	}, nil
}

func (b *Block) TransactionCount(ctx context.Context) (*hexutil.Uint64, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	count := hexutil.Uint64(len(block.Transactions()))
	return &count, err
}

func (b *Block) Transactions(ctx context.Context) (*[]*Transaction, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	ret := make([]*Transaction, 0, len(block.Transactions()))
	for i, tx := range block.Transactions() {
		ret = append(ret, &Transaction{
			r:     b.r,
			hash:  tx.Hash(),
			tx:    tx,
			block: b,
			index: uint64(i),
		})
	}
	return &ret, nil
}

func (b *Block) TransactionAt(ctx context.Context, args struct{ Index Long }) (*Transaction, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	txs := block.Transactions()
	if args.Index < 0 || int(args.Index) >= len(txs) {
		return nil, nil
	}
	tx := txs[args.Index]
	return &Transaction{
		r:     b.r,
		hash:  tx.Hash(),
		tx:    tx,
		block: b,
		index: uint64(args.Index),
	}, nil
}

func (b *Block) OmmerAt(ctx context.Context, args struct{ Index Long }) (*Block, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	uncles := block.Uncles()
	if args.Index < 0 || int(args.Index) >= len(uncles) {
		return nil, nil
	}
	uncle := uncles[args.Index]
	blockNumberOrHash := rpc.BlockNumberOrHashWithHash(uncle.Hash(), false)
	return &Block{
		r:            b.r,
		numberOrHash: &blockNumberOrHash,
		header:       uncle,
		hash:         uncle.Hash(),
	}, nil
}

func (b *Block) WithdrawalsRoot(ctx context.Context) (*common.Hash, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	// Pre-shanghai blocks
	if header.WithdrawalsHash == nil {
		return nil, nil
	}
	return header.WithdrawalsHash, nil
}

func (b *Block) Withdrawals(ctx context.Context) (*[]*Withdrawal, error) {
	block, err := b.resolve(ctx)
	if err != nil || block == nil {
		return nil, err
	}
	// Pre-shanghai blocks
	if block.Header().WithdrawalsHash == nil {
		return nil, nil
	}
	ret := make([]*Withdrawal, 0, len(block.Withdrawals()))
	for _, w := range block.Withdrawals() {
		ret = append(ret, &Withdrawal{
			index:     w.Index,
			validator: w.Validator,
			address:   w.Address,
			amount:    w.Amount,
		})
	}
	return &ret, nil
}

func (b *Block) BlobGasUsed(ctx context.Context) (*hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	if header.BlobGasUsed == nil {
		return nil, nil
	}
	ret := hexutil.Uint64(*header.BlobGasUsed)
	return &ret, nil
}

func (b *Block) ExcessBlobGas(ctx context.Context) (*hexutil.Uint64, error) {
	header, err := b.resolveHeader(ctx)
	if err != nil {
		return nil, err
	}
	if header.ExcessBlobGas == nil {
		return nil, nil
	}
	ret := hexutil.Uint64(*header.ExcessBlobGas)
	return &ret, nil
}

// BlockFilterCriteria encapsulates criteria passed to a `logs` accessor inside
// a block.
type BlockFilterCriteria struct {
	Addresses *[]common.Address // restricts matches to events created by specific contracts

	// The Topic list restricts matches to particular event topics. Each event has a list
	// of topics. Topics matches a prefix of that list. An empty element slice matches any
	// topic. Non-empty elements represent an alternative that matches any of the
	// contained topics.
	//
	// Examples:
	// {} or nil          matches any topic list
	// {{A}}              matches topic A in first position
	// {{}, {B}}          matches any topic in first position, B in second position
	// {{A}, {B}}         matches topic A in first position, B in second position
	// {{A, B}}, {C, D}}  matches topic (A OR B) in first position, (C OR D) in second position
	Topics *[][]common.Hash
}

// runFilter accepts a filter and executes it, returning all its results as
// `Log` objects.
func runFilter(ctx context.Context, r *Resolver, filter *filters.Filter) ([]*Log, error) {
	logs, err := filter.Logs(ctx)
	if err != nil || logs == nil {
		return nil, err
	}
	ret := make([]*Log, 0, len(logs))
	for _, log := range logs {
		ret = append(ret, &Log{
			r:           r,
			transaction: &Transaction{r: r, hash: log.TxHash},
			log:         log,
		})
	}
	return ret, nil
}

func (b *Block) Logs(ctx context.Context, args struct{ Filter BlockFilterCriteria }) ([]*Log, error) {
	var addresses []common.Address
	if args.Filter.Addresses != nil {
		addresses = *args.Filter.Addresses
	}
	var topics [][]common.Hash
	if args.Filter.Topics != nil {
		topics = *args.Filter.Topics
	}
	// Construct the range filter
	hash, err := b.Hash(ctx)
	if err != nil {
		return nil, err
	}
	filter := b.r.filterSystem.NewBlockFilter(hash, addresses, topics)

	// Run the filter and return all the logs
	return runFilter(ctx, b.r, filter)
}

func (b *Block) Account(ctx context.Context, args struct {
	Address common.Address
}) (*Account, error) {
	return &Account{
		r:             b.r,
		address:       args.Address,
		blockNrOrHash: *b.numberOrHash,
	}, nil
}

// CallData encapsulates arguments to `call` or `estimateGas`.
// All arguments are optional.
type CallData struct {
	From                 *common.Address // The Ethereum address the call is from.
	To                   *common.Address // The Ethereum address the call is to.
	Gas                  *Long           // The amount of gas provided for the call.
	GasPrice             *hexutil.Big    // The price of each unit of gas, in wei.
	MaxFeePerGas         *hexutil.Big    // The max price of each unit of gas, in wei (1559).
	MaxPriorityFeePerGas *hexutil.Big    // The max tip of each unit of gas, in wei (1559).
	Value                *hexutil.Big    // The value sent along with the call.
	Data                 *hexutil.Bytes  // Any data sent with the call.
}

// CallResult encapsulates the result of an invocation of the `call` accessor.
type CallResult struct {
	data    hexutil.Bytes  // The return data from the call
	gasUsed hexutil.Uint64 // The amount of gas used
	status  hexutil.Uint64 // The return status of the call - 0 for failure or 1 for success.
}

func (c *CallResult) Data() hexutil.Bytes {
	return c.data
}

func (c *CallResult) GasUsed() hexutil.Uint64 {
	return c.gasUsed
}

func (c *CallResult) Status() hexutil.Uint64 {
	return c.status
}

func (b *Block) Call(ctx context.Context, args struct {
	Data ethapi.TransactionArgs
}) (*CallResult, error) {
	result, err := ethapi.DoCall(ctx, b.r.backend, args.Data, *b.numberOrHash, nil, nil, b.r.backend.RPCEVMTimeout(), b.r.backend.RPCGasCap())
	if err != nil {
		return nil, err
	}
	status := hexutil.Uint64(1)
	if result.Failed() {
		status = 0
	}

	return &CallResult{
		data:    result.ReturnData,
		gasUsed: hexutil.Uint64(result.UsedGas),
		status:  status,
	}, nil
}

func (b *Block) EstimateGas(ctx context.Context, args struct {
	Data ethapi.TransactionArgs
}) (hexutil.Uint64, error) {
	return ethapi.DoEstimateGas(ctx, b.r.backend, args.Data, *b.numberOrHash, nil, b.r.backend.RPCGasCap())
}

type Pending struct {
	r *Resolver
}

func (p *Pending) TransactionCount(ctx context.Context) (hexutil.Uint64, error) {
	txs, err := p.r.backend.GetPoolTransactions()
	return hexutil.Uint64(len(txs)), err
}

func (p *Pending) Transactions(ctx context.Context) (*[]*Transaction, error) {
	txs, err := p.r.backend.GetPoolTransactions()
	if err != nil {
		return nil, err
	}
	ret := make([]*Transaction, 0, len(txs))
	for i, tx := range txs {
		ret = append(ret, &Transaction{
			r:     p.r,
			hash:  tx.Hash(),
			tx:    tx,
			index: uint64(i),
		})
	}
	return &ret, nil
}

func (p *Pending) Account(ctx context.Context, args struct {
	Address common.Address
}) *Account {
	pendingBlockNr := rpc.BlockNumberOrHashWithNumber(rpc.PendingBlockNumber)
	return &Account{
		r:             p.r,
		address:       args.Address,
		blockNrOrHash: pendingBlockNr,
	}
}

func (p *Pending) Call(ctx context.Context, args struct {
	Data ethapi.TransactionArgs
}) (*CallResult, error) {
	pendingBlockNr := rpc.BlockNumberOrHashWithNumber(rpc.PendingBlockNumber)
	result, err := ethapi.DoCall(ctx, p.r.backend, args.Data, pendingBlockNr, nil, nil, p.r.backend.RPCEVMTimeout(), p.r.backend.RPCGasCap())
	if err != nil {
		return nil, err
	}
	status := hexutil.Uint64(1)
	if result.Failed() {
		status = 0
	}

	return &CallResult{
		data:    result.ReturnData,
		gasUsed: hexutil.Uint64(result.UsedGas),
		status:  status,
	}, nil
}

func (p *Pending) EstimateGas(ctx context.Context, args struct {
	Data ethapi.TransactionArgs
}) (hexutil.Uint64, error) {
	latestBlockNr := rpc.BlockNumberOrHashWithNumber(rpc.LatestBlockNumber)
	return ethapi.DoEstimateGas(ctx, p.r.backend, args.Data, latestBlockNr, nil, p.r.backend.RPCGasCap())
}

// Resolver is the top-level object in the GraphQL hierarchy.
type Resolver struct {
	backend      ethapi.Backend
	filterSystem *filters.FilterSystem
}

func (r *Resolver) Block(ctx context.Context, args struct {
	Number *Long
	Hash   *common.Hash
}) (*Block, error) {
	if args.Number != nil && args.Hash != nil {
		return nil, errors.New("only one of number or hash must be specified")
	}
	var numberOrHash rpc.BlockNumberOrHash
	if args.Number != nil {
		if *args.Number < 0 {
			return nil, nil
		}
		number := rpc.BlockNumber(*args.Number)
		numberOrHash = rpc.BlockNumberOrHashWithNumber(number)
	} else if args.Hash != nil {
		numberOrHash = rpc.BlockNumberOrHashWithHash(*args.Hash, false)
	} else {
		numberOrHash = rpc.BlockNumberOrHashWithNumber(rpc.LatestBlockNumber)
	}
	block := &Block{
		r:            r,
		numberOrHash: &numberOrHash,
	}
	// Resolve the header, return nil if it doesn't exist.
	// Note we don't resolve block directly here since it will require an
	// additional network request for light client.
	h, err := block.resolveHeader(ctx)
	if err != nil {
		return nil, err
	} else if h == nil {
		return nil, nil
	}
	return block, nil
}

func (r *Resolver) Blocks(ctx context.Context, args struct {
	From *Long
	To   *Long
}) ([]*Block, error) {
	if args.From == nil {
		return nil, errors.New("from block number must be specified")
	}
	from := rpc.BlockNumber(*args.From)

	var to rpc.BlockNumber
	if args.To != nil {
		to = rpc.BlockNumber(*args.To)
	} else {
		to = rpc.BlockNumber(r.backend.CurrentBlock().Number.Int64())
	}
	if to < from {
		return nil, errInvalidBlockRange
	}
	var ret []*Block
	for i := from; i <= to; i++ {
		numberOrHash := rpc.BlockNumberOrHashWithNumber(i)
		block := &Block{
			r:            r,
			numberOrHash: &numberOrHash,
		}
		// Resolve the header to check for existence.
		// Note we don't resolve block directly here since it will require an
		// additional network request for light client.
		h, err := block.resolveHeader(ctx)
		if err != nil {
			return nil, err
		} else if h == nil {
			// Blocks after must be non-existent too, break.
			break
		}
		ret = append(ret, block)
		if err := ctx.Err(); err != nil {
			return nil, err
		}
	}
	return ret, nil
}

func (r *Resolver) Pending(ctx context.Context) *Pending {
	return &Pending{r}
}

func (r *Resolver) Transaction(ctx context.Context, args struct{ Hash common.Hash }) *Transaction {
	tx := &Transaction{
		r:    r,
		hash: args.Hash,
	}
	// Resolve the transaction; if it doesn't exist, return nil.
	t, _ := tx.resolve(ctx)
	if t == nil {
		return nil
	}
	return tx
}

func (r *Resolver) SendRawTransaction(ctx context.Context, args struct{ Data hexutil.Bytes }) (common.Hash, error) {
	tx := new(types.Transaction)
	if err := tx.UnmarshalBinary(args.Data); err != nil {
		return common.Hash{}, err
	}
	hash, err := ethapi.SubmitTransaction(ctx, r.backend, tx)
	return hash, err
}

// FilterCriteria encapsulates the arguments to `logs` on the root resolver object.
type FilterCriteria struct {
	FromBlock *Long             // beginning of the queried range, nil means genesis block
	ToBlock   *Long             // end of the range, nil means latest block
	Addresses *[]common.Address // restricts matches to events created by specific contracts

	// The Topic list restricts matches to particular event topics. Each event has a list
	// of topics. Topics matches a prefix of that list. An empty element slice matches any
	// topic. Non-empty elements represent an alternative that matches any of the
	// contained topics.
	//
	// Examples:
	// {} or nil          matches any topic list
	// {{A}}              matches topic A in first position
	// {{}, {B}}          matches any topic in first position, B in second position
	// {{A}, {B}}         matches topic A in first position, B in second position
	// {{A, B}}, {C, D}}  matches topic (A OR B) in first position, (C OR D) in second position
	Topics *[][]common.Hash
}

func (r *Resolver) Logs(ctx context.Context, args struct{ Filter FilterCriteria }) ([]*Log, error) {
	// Convert the RPC block numbers into internal representations
	begin := rpc.LatestBlockNumber.Int64()
	if args.Filter.FromBlock != nil {
		begin = int64(*args.Filter.FromBlock)
	}
	end := rpc.LatestBlockNumber.Int64()
	if args.Filter.ToBlock != nil {
		end = int64(*args.Filter.ToBlock)
	}
	if begin > 0 && end > 0 && begin > end {
		return nil, errInvalidBlockRange
	}
	var addresses []common.Address
	if args.Filter.Addresses != nil {
		addresses = *args.Filter.Addresses
	}
	var topics [][]common.Hash
	if args.Filter.Topics != nil {
		topics = *args.Filter.Topics
	}
	// Construct the range filter
	filter := r.filterSystem.NewRangeFilter(begin, end, addresses, topics)
	return runFilter(ctx, r, filter)
}

func (r *Resolver) GasPrice(ctx context.Context) (hexutil.Big, error) {
	tipcap, err := r.backend.SuggestGasTipCap(ctx)
	if err != nil {
		return hexutil.Big{}, err
	}
	if head := r.backend.CurrentHeader(); head.BaseFee != nil {
		tipcap.Add(tipcap, head.BaseFee)
	}
	return (hexutil.Big)(*tipcap), nil
}

func (r *Resolver) MaxPriorityFeePerGas(ctx context.Context) (hexutil.Big, error) {
	tipcap, err := r.backend.SuggestGasTipCap(ctx)
	if err != nil {
		return hexutil.Big{}, err
	}
	return (hexutil.Big)(*tipcap), nil
}

func (r *Resolver) ChainID(ctx context.Context) (hexutil.Big, error) {
	return hexutil.Big(*r.backend.ChainConfig().ChainID), nil
}

// SyncState represents the synchronisation status returned from the `syncing` accessor.
type SyncState struct {
	progress ethereum.SyncProgress
}

func (s *SyncState) StartingBlock() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.StartingBlock)
}
func (s *SyncState) CurrentBlock() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.CurrentBlock)
}
func (s *SyncState) HighestBlock() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HighestBlock)
}
func (s *SyncState) SyncedAccounts() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedAccounts)
}
func (s *SyncState) SyncedAccountBytes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedAccountBytes)
}
func (s *SyncState) SyncedBytecodes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedBytecodes)
}
func (s *SyncState) SyncedBytecodeBytes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedBytecodeBytes)
}
func (s *SyncState) SyncedStorage() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedStorage)
}
func (s *SyncState) SyncedStorageBytes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.SyncedStorageBytes)
}
func (s *SyncState) HealedTrienodes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealedTrienodes)
}
func (s *SyncState) HealedTrienodeBytes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealedTrienodeBytes)
}
func (s *SyncState) HealedBytecodes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealedBytecodes)
}
func (s *SyncState) HealedBytecodeBytes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealedBytecodeBytes)
}
func (s *SyncState) HealingTrienodes() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealingTrienodes)
}
func (s *SyncState) HealingBytecode() hexutil.Uint64 {
	return hexutil.Uint64(s.progress.HealingBytecode)
}

// Syncing returns false in case the node is currently not syncing with the network. It can be up-to-date or has not
// yet received the latest block headers from its pears. In case it is synchronizing:
// - startingBlock:       block number this node started to synchronize from
// - currentBlock:        block number this node is currently importing
// - highestBlock:        block number of the highest block header this node has received from peers
// - syncedAccounts:      number of accounts downloaded
// - syncedAccountBytes:  number of account trie bytes persisted to disk
// - syncedBytecodes:     number of bytecodes downloaded
// - syncedBytecodeBytes: number of bytecode bytes downloaded
// - syncedStorage:       number of storage slots downloaded
// - syncedStorageBytes:  number of storage trie bytes persisted to disk
// - healedTrienodes:     number of state trie nodes downloaded
// - healedTrienodeBytes: number of state trie bytes persisted to disk
// - healedBytecodes:     number of bytecodes downloaded
// - healedBytecodeBytes: number of bytecodes persisted to disk
// - healingTrienodes:    number of state trie nodes pending
// - healingBytecode:     number of bytecodes pending
func (r *Resolver) Syncing() (*SyncState, error) {
	progress := r.backend.SyncProgress()

	// Return not syncing if the synchronisation already completed
	if progress.CurrentBlock >= progress.HighestBlock {
		return nil, nil
	}
	// Otherwise gather the block sync stats
	return &SyncState{progress}, nil
}
