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

package beacon

import (
	"errors"
	"fmt"
	"math/big"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/consensus/misc/eip4844"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/ethereum/go-ethereum/trie"
)

// Proof-of-stake protocol constants.
var (
	beaconDifficulty = common.Big0          // The default block difficulty in the beacon consensus
	beaconNonce      = types.EncodeNonce(0) // The default block nonce in the beacon consensus
)

// Various error messages to mark blocks invalid. These should be private to
// prevent engine specific errors from being referenced in the remainder of the
// codebase, inherently breaking if the engine is swapped out. Please put common
// error types into the consensus package.
var (
	errTooManyUncles    = errors.New("too many uncles")
	errInvalidNonce     = errors.New("invalid nonce")
	errInvalidUncleHash = errors.New("invalid uncle hash")
	errInvalidTimestamp = errors.New("invalid timestamp")
)

// Beacon is a consensus engine that combines the eth1 consensus and proof-of-stake
// algorithm. There is a special flag inside to decide whether to use legacy consensus
// rules or new rules. The transition rule is described in the eth1/2 merge spec.
// https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3675.md
//
// The beacon here is a half-functional consensus engine with partial functions which
// is only used for necessary consensus checks. The legacy consensus engine can be any
// engine implements the consensus interface (except the beacon itself).
type Beacon struct {
	ethone consensus.Engine // Original consensus engine used in eth1, e.g. ethash or clique
}

// New creates a consensus engine with the given embedded eth1 engine.
func New(ethone consensus.Engine) *Beacon {
	if _, ok := ethone.(*Beacon); ok {
		panic("nested consensus engine")
	}
	return &Beacon{ethone: ethone}
}

// Author implements consensus.Engine, returning the verified author of the block.
func (beacon *Beacon) Author(header *types.Header) (common.Address, error) {
	if !beacon.IsPoSHeader(header) {
		return beacon.ethone.Author(header)
	}
	return header.Coinbase, nil
}

// VerifyHeader checks whether a header conforms to the consensus rules of the
// stock Ethereum consensus engine.
func (beacon *Beacon) VerifyHeader(chain consensus.ChainHeaderReader, header *types.Header) error {
	reached, err := IsTTDReached(chain, header.ParentHash, header.Number.Uint64()-1)
	if err != nil {
		return err
	}
	if !reached {
		return beacon.ethone.VerifyHeader(chain, header)
	}
	// Short circuit if the parent is not known
	parent := chain.GetHeader(header.ParentHash, header.Number.Uint64()-1)
	if parent == nil {
		return consensus.ErrUnknownAncestor
	}
	// Sanity checks passed, do a proper verification
	return beacon.verifyHeader(chain, header, parent)
}

// errOut constructs an error channel with prefilled errors inside.
func errOut(n int, err error) chan error {
	errs := make(chan error, n)
	for i := 0; i < n; i++ {
		errs <- err
	}
	return errs
}

// splitHeaders splits the provided header batch into two parts according to
// the configured ttd. It requires the parent of header batch along with its
// td are stored correctly in chain. If ttd is not configured yet, all headers
// will be treated legacy PoW headers.
// Note, this function will not verify the header validity but just split them.
func (beacon *Beacon) splitHeaders(chain consensus.ChainHeaderReader, headers []*types.Header) ([]*types.Header, []*types.Header, error) {
	// TTD is not defined yet, all headers should be in legacy format.
	ttd := chain.Config().TerminalTotalDifficulty
	if ttd == nil {
		return headers, nil, nil
	}
	ptd := chain.GetTd(headers[0].ParentHash, headers[0].Number.Uint64()-1)
	if ptd == nil {
		return nil, nil, consensus.ErrUnknownAncestor
	}
	// The entire header batch already crosses the transition.
	if ptd.Cmp(ttd) >= 0 {
		return nil, headers, nil
	}
	var (
		preHeaders  = headers
		postHeaders []*types.Header
		td          = new(big.Int).Set(ptd)
		tdPassed    bool
	)
	for i, header := range headers {
		if tdPassed {
			preHeaders = headers[:i]
			postHeaders = headers[i:]
			break
		}
		td = td.Add(td, header.Difficulty)
		if td.Cmp(ttd) >= 0 {
			// This is the last PoW header, it still belongs to
			// the preHeaders, so we cannot split+break yet.
			tdPassed = true
		}
	}
	return preHeaders, postHeaders, nil
}

// VerifyHeaders is similar to VerifyHeader, but verifies a batch of headers
// concurrently. The method returns a quit channel to abort the operations and
// a results channel to retrieve the async verifications.
// VerifyHeaders expect the headers to be ordered and continuous.
func (beacon *Beacon) VerifyHeaders(chain consensus.ChainHeaderReader, headers []*types.Header) (chan<- struct{}, <-chan error) {
	preHeaders, postHeaders, err := beacon.splitHeaders(chain, headers)
	if err != nil {
		return make(chan struct{}), errOut(len(headers), err)
	}
	if len(postHeaders) == 0 {
		return beacon.ethone.VerifyHeaders(chain, headers)
	}
	if len(preHeaders) == 0 {
		return beacon.verifyHeaders(chain, headers, nil)
	}
	// The transition point exists in the middle, separate the headers
	// into two batches and apply different verification rules for them.
	var (
		abort   = make(chan struct{})
		results = make(chan error, len(headers))
	)
	go func() {
		var (
			old, new, out      = 0, len(preHeaders), 0
			errors             = make([]error, len(headers))
			done               = make([]bool, len(headers))
			oldDone, oldResult = beacon.ethone.VerifyHeaders(chain, preHeaders)
			newDone, newResult = beacon.verifyHeaders(chain, postHeaders, preHeaders[len(preHeaders)-1])
		)
		// Collect the results
		for {
			for ; done[out]; out++ {
				results <- errors[out]
				if out == len(headers)-1 {
					return
				}
			}
			select {
			case err := <-oldResult:
				if !done[old] { // skip TTD-verified failures
					errors[old], done[old] = err, true
				}
				old++
			case err := <-newResult:
				errors[new], done[new] = err, true
				new++
			case <-abort:
				close(oldDone)
				close(newDone)
				return
			}
		}
	}()
	return abort, results
}

// VerifyUncles verifies that the given block's uncles conform to the consensus
// rules of the Ethereum consensus engine.
func (beacon *Beacon) VerifyUncles(chain consensus.ChainReader, block *types.Block) error {
	if !beacon.IsPoSHeader(block.Header()) {
		return beacon.ethone.VerifyUncles(chain, block)
	}
	// Verify that there is no uncle block. It's explicitly disabled in the beacon
	if len(block.Uncles()) > 0 {
		return errTooManyUncles
	}
	return nil
}

// verifyHeader checks whether a header conforms to the consensus rules of the
// stock Ethereum consensus engine. The difference between the beacon and classic is
// (a) The following fields are expected to be constants:
//   - difficulty is expected to be 0
//   - nonce is expected to be 0
//   - unclehash is expected to be Hash(emptyHeader)
//     to be the desired constants
//
// (b) we don't verify if a block is in the future anymore
// (c) the extradata is limited to 32 bytes
func (beacon *Beacon) verifyHeader(chain consensus.ChainHeaderReader, header, parent *types.Header) error {
	// Ensure that the header's extra-data section is of a reasonable size
	if len(header.Extra) > 32 {
		return fmt.Errorf("extra-data longer than 32 bytes (%d)", len(header.Extra))
	}
	// Verify the seal parts. Ensure the nonce and uncle hash are the expected value.
	if header.Nonce != beaconNonce {
		return errInvalidNonce
	}
	if header.UncleHash != types.EmptyUncleHash {
		return errInvalidUncleHash
	}
	// Verify the timestamp
	if header.Time <= parent.Time {
		return errInvalidTimestamp
	}
	// Verify the block's difficulty to ensure it's the default constant
	if beaconDifficulty.Cmp(header.Difficulty) != 0 {
		return fmt.Errorf("invalid difficulty: have %v, want %v", header.Difficulty, beaconDifficulty)
	}
	// Verify that the gas limit is <= 2^63-1
	if header.GasLimit > params.MaxGasLimit {
		return fmt.Errorf("invalid gasLimit: have %v, max %v", header.GasLimit, params.MaxGasLimit)
	}
	// Verify that the gasUsed is <= gasLimit
	if header.GasUsed > header.GasLimit {
		return fmt.Errorf("invalid gasUsed: have %d, gasLimit %d", header.GasUsed, header.GasLimit)
	}
	// Verify that the block number is parent's +1
	if diff := new(big.Int).Sub(header.Number, parent.Number); diff.Cmp(common.Big1) != 0 {
		return consensus.ErrInvalidNumber
	}
	// Verify the header's EIP-1559 attributes.
	if err := eip1559.VerifyEIP1559Header(chain.Config(), parent, header); err != nil {
		return err
	}
	// Verify existence / non-existence of withdrawalsHash.
	shanghai := chain.Config().IsShanghai(header.Number, header.Time)
	if shanghai && header.WithdrawalsHash == nil {
		return errors.New("missing withdrawalsHash")
	}
	if !shanghai && header.WithdrawalsHash != nil {
		return fmt.Errorf("invalid withdrawalsHash: have %x, expected nil", header.WithdrawalsHash)
	}
	// Verify the existence / non-existence of cancun-specific header fields
	cancun := chain.Config().IsCancun(header.Number, header.Time)
	if !cancun {
		switch {
		case header.ExcessBlobGas != nil:
			return fmt.Errorf("invalid excessBlobGas: have %d, expected nil", header.ExcessBlobGas)
		case header.BlobGasUsed != nil:
			return fmt.Errorf("invalid blobGasUsed: have %d, expected nil", header.BlobGasUsed)
		case header.ParentBeaconRoot != nil:
			return fmt.Errorf("invalid parentBeaconRoot, have %#x, expected nil", header.ParentBeaconRoot)
		}
	} else {
		if header.ParentBeaconRoot == nil {
			return errors.New("header is missing beaconRoot")
		}
		if err := eip4844.VerifyEIP4844Header(parent, header); err != nil {
			return err
		}
	}
	return nil
}

// verifyHeaders is similar to verifyHeader, but verifies a batch of headers
// concurrently. The method returns a quit channel to abort the operations and
// a results channel to retrieve the async verifications. An additional parent
// header will be passed if the relevant header is not in the database yet.
func (beacon *Beacon) verifyHeaders(chain consensus.ChainHeaderReader, headers []*types.Header, ancestor *types.Header) (chan<- struct{}, <-chan error) {
	var (
		abort   = make(chan struct{})
		results = make(chan error, len(headers))
	)
	go func() {
		for i, header := range headers {
			var parent *types.Header
			if i == 0 {
				if ancestor != nil {
					parent = ancestor
				} else {
					parent = chain.GetHeader(headers[0].ParentHash, headers[0].Number.Uint64()-1)
				}
			} else if headers[i-1].Hash() == headers[i].ParentHash {
				parent = headers[i-1]
			}
			if parent == nil {
				select {
				case <-abort:
					return
				case results <- consensus.ErrUnknownAncestor:
				}
				continue
			}
			err := beacon.verifyHeader(chain, header, parent)
			select {
			case <-abort:
				return
			case results <- err:
			}
		}
	}()
	return abort, results
}

// Prepare implements consensus.Engine, initializing the difficulty field of a
// header to conform to the beacon protocol. The changes are done inline.
func (beacon *Beacon) Prepare(chain consensus.ChainHeaderReader, header *types.Header) error {
	// Transition isn't triggered yet, use the legacy rules for preparation.
	reached, err := IsTTDReached(chain, header.ParentHash, header.Number.Uint64()-1)
	if err != nil {
		return err
	}
	if !reached {
		return beacon.ethone.Prepare(chain, header)
	}
	header.Difficulty = beaconDifficulty
	return nil
}

// Finalize implements consensus.Engine and processes withdrawals on top.
func (beacon *Beacon) Finalize(chain consensus.ChainHeaderReader, header *types.Header, state *state.StateDB, txs []*types.Transaction, uncles []*types.Header, withdrawals []*types.Withdrawal) {
	if !beacon.IsPoSHeader(header) {
		beacon.ethone.Finalize(chain, header, state, txs, uncles, nil)
		return
	}
	// Withdrawals processing.
	for _, w := range withdrawals {
		// Convert amount from gwei to wei.
		amount := new(big.Int).SetUint64(w.Amount)
		amount = amount.Mul(amount, big.NewInt(params.GWei))
		state.AddBalance(w.Address, amount)
	}
	// No block reward which is issued by consensus layer instead.
}

// FinalizeAndAssemble implements consensus.Engine, setting the final state and
// assembling the block.
func (beacon *Beacon) FinalizeAndAssemble(chain consensus.ChainHeaderReader, header *types.Header, state *state.StateDB, txs []*types.Transaction, uncles []*types.Header, receipts []*types.Receipt, withdrawals []*types.Withdrawal) (*types.Block, error) {
	if !beacon.IsPoSHeader(header) {
		return beacon.ethone.FinalizeAndAssemble(chain, header, state, txs, uncles, receipts, nil)
	}
	shanghai := chain.Config().IsShanghai(header.Number, header.Time)
	if shanghai {
		// All blocks after Shanghai must include a withdrawals root.
		if withdrawals == nil {
			withdrawals = make([]*types.Withdrawal, 0)
		}
	} else {
		if len(withdrawals) > 0 {
			return nil, errors.New("withdrawals set before Shanghai activation")
		}
	}
	// Finalize and assemble the block.
	beacon.Finalize(chain, header, state, txs, uncles, withdrawals)

	// Assign the final state root to header.
	header.Root = state.IntermediateRoot(true)

	// Assemble and return the final block.
	return types.NewBlockWithWithdrawals(header, txs, uncles, receipts, withdrawals, trie.NewStackTrie(nil)), nil
}

// Seal generates a new sealing request for the given input block and pushes
// the result into the given channel.
//
// Note, the method returns immediately and will send the result async. More
// than one result may also be returned depending on the consensus algorithm.
func (beacon *Beacon) Seal(chain consensus.ChainHeaderReader, block *types.Block, results chan<- *types.Block, stop <-chan struct{}) error {
	if !beacon.IsPoSHeader(block.Header()) {
		return beacon.ethone.Seal(chain, block, results, stop)
	}
	// The seal verification is done by the external consensus engine,
	// return directly without pushing any block back. In another word
	// beacon won't return any result by `results` channel which may
	// blocks the receiver logic forever.
	return nil
}

// SealHash returns the hash of a block prior to it being sealed.
func (beacon *Beacon) SealHash(header *types.Header) common.Hash {
	return beacon.ethone.SealHash(header)
}

// CalcDifficulty is the difficulty adjustment algorithm. It returns
// the difficulty that a new block should have when created at time
// given the parent block's time and difficulty.
func (beacon *Beacon) CalcDifficulty(chain consensus.ChainHeaderReader, time uint64, parent *types.Header) *big.Int {
	// Transition isn't triggered yet, use the legacy rules for calculation
	if reached, _ := IsTTDReached(chain, parent.Hash(), parent.Number.Uint64()); !reached {
		return beacon.ethone.CalcDifficulty(chain, time, parent)
	}
	return beaconDifficulty
}

// APIs implements consensus.Engine, returning the user facing RPC APIs.
func (beacon *Beacon) APIs(chain consensus.ChainHeaderReader) []rpc.API {
	return beacon.ethone.APIs(chain)
}

// Close shutdowns the consensus engine
func (beacon *Beacon) Close() error {
	return beacon.ethone.Close()
}

// IsPoSHeader reports the header belongs to the PoS-stage with some special fields.
// This function is not suitable for a part of APIs like Prepare or CalcDifficulty
// because the header difficulty is not set yet.
func (beacon *Beacon) IsPoSHeader(header *types.Header) bool {
	if header.Difficulty == nil {
		panic("IsPoSHeader called with invalid difficulty")
	}
	return header.Difficulty.Cmp(beaconDifficulty) == 0
}

// InnerEngine returns the embedded eth1 consensus engine.
func (beacon *Beacon) InnerEngine() consensus.Engine {
	return beacon.ethone
}

// SetThreads updates the mining threads. Delegate the call
// to the eth1 engine if it's threaded.
func (beacon *Beacon) SetThreads(threads int) {
	type threaded interface {
		SetThreads(threads int)
	}
	if th, ok := beacon.ethone.(threaded); ok {
		th.SetThreads(threads)
	}
}

// IsTTDReached checks if the TotalTerminalDifficulty has been surpassed on the `parentHash` block.
// It depends on the parentHash already being stored in the database.
// If the parentHash is not stored in the database a UnknownAncestor error is returned.
func IsTTDReached(chain consensus.ChainHeaderReader, parentHash common.Hash, parentNumber uint64) (bool, error) {
	if chain.Config().TerminalTotalDifficulty == nil {
		return false, nil
	}
	td := chain.GetTd(parentHash, parentNumber)
	if td == nil {
		return false, consensus.ErrUnknownAncestor
	}
	return td.Cmp(chain.Config().TerminalTotalDifficulty) >= 0, nil
}
