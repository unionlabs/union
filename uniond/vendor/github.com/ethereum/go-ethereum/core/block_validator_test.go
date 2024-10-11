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
	"math/big"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/beacon"
	"github.com/ethereum/go-ethereum/consensus/clique"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/params"
)

// Tests that simple header verification works, for both good and bad blocks.
func TestHeaderVerification(t *testing.T) {
	testHeaderVerification(t, rawdb.HashScheme)
	testHeaderVerification(t, rawdb.PathScheme)
}

func testHeaderVerification(t *testing.T, scheme string) {
	// Create a simple chain to verify
	var (
		gspec        = &Genesis{Config: params.TestChainConfig}
		_, blocks, _ = GenerateChainWithGenesis(gspec, ethash.NewFaker(), 8, nil)
	)
	headers := make([]*types.Header, len(blocks))
	for i, block := range blocks {
		headers[i] = block.Header()
	}
	// Run the header checker for blocks one-by-one, checking for both valid and invalid nonces
	chain, _ := NewBlockChain(rawdb.NewMemoryDatabase(), DefaultCacheConfigWithScheme(scheme), gspec, nil, ethash.NewFaker(), vm.Config{}, nil, nil)
	defer chain.Stop()

	for i := 0; i < len(blocks); i++ {
		for j, valid := range []bool{true, false} {
			var results <-chan error

			if valid {
				engine := ethash.NewFaker()
				_, results = engine.VerifyHeaders(chain, []*types.Header{headers[i]})
			} else {
				engine := ethash.NewFakeFailer(headers[i].Number.Uint64())
				_, results = engine.VerifyHeaders(chain, []*types.Header{headers[i]})
			}
			// Wait for the verification result
			select {
			case result := <-results:
				if (result == nil) != valid {
					t.Errorf("test %d.%d: validity mismatch: have %v, want %v", i, j, result, valid)
				}
			case <-time.After(time.Second):
				t.Fatalf("test %d.%d: verification timeout", i, j)
			}
			// Make sure no more data is returned
			select {
			case result := <-results:
				t.Fatalf("test %d.%d: unexpected result returned: %v", i, j, result)
			case <-time.After(25 * time.Millisecond):
			}
		}
		chain.InsertChain(blocks[i : i+1])
	}
}

func TestHeaderVerificationForMergingClique(t *testing.T) { testHeaderVerificationForMerging(t, true) }
func TestHeaderVerificationForMergingEthash(t *testing.T) { testHeaderVerificationForMerging(t, false) }

// Tests the verification for eth1/2 merging, including pre-merge and post-merge
func testHeaderVerificationForMerging(t *testing.T, isClique bool) {
	var (
		gspec      *Genesis
		preBlocks  []*types.Block
		postBlocks []*types.Block
		engine     consensus.Engine
		merger     = consensus.NewMerger(rawdb.NewMemoryDatabase())
	)
	if isClique {
		var (
			key, _ = crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")
			addr   = crypto.PubkeyToAddress(key.PublicKey)
			config = *params.AllCliqueProtocolChanges
		)
		engine = beacon.New(clique.New(params.AllCliqueProtocolChanges.Clique, rawdb.NewMemoryDatabase()))
		gspec = &Genesis{
			Config:    &config,
			ExtraData: make([]byte, 32+common.AddressLength+crypto.SignatureLength),
			Alloc: map[common.Address]GenesisAccount{
				addr: {Balance: big.NewInt(1)},
			},
			BaseFee:    big.NewInt(params.InitialBaseFee),
			Difficulty: new(big.Int),
		}
		copy(gspec.ExtraData[32:], addr[:])

		td := 0
		genDb, blocks, _ := GenerateChainWithGenesis(gspec, engine, 8, nil)
		for i, block := range blocks {
			header := block.Header()
			if i > 0 {
				header.ParentHash = blocks[i-1].Hash()
			}
			header.Extra = make([]byte, 32+crypto.SignatureLength)
			header.Difficulty = big.NewInt(2)

			sig, _ := crypto.Sign(engine.SealHash(header).Bytes(), key)
			copy(header.Extra[len(header.Extra)-crypto.SignatureLength:], sig)
			blocks[i] = block.WithSeal(header)

			// calculate td
			td += int(block.Difficulty().Uint64())
		}
		preBlocks = blocks
		gspec.Config.TerminalTotalDifficulty = big.NewInt(int64(td))
		postBlocks, _ = GenerateChain(gspec.Config, preBlocks[len(preBlocks)-1], engine, genDb, 8, nil)
	} else {
		config := *params.TestChainConfig
		gspec = &Genesis{Config: &config}
		engine = beacon.New(ethash.NewFaker())
		td := int(params.GenesisDifficulty.Uint64())
		genDb, blocks, _ := GenerateChainWithGenesis(gspec, engine, 8, nil)
		for _, block := range blocks {
			// calculate td
			td += int(block.Difficulty().Uint64())
		}
		preBlocks = blocks
		gspec.Config.TerminalTotalDifficulty = big.NewInt(int64(td))
		t.Logf("Set ttd to %v\n", gspec.Config.TerminalTotalDifficulty)
		postBlocks, _ = GenerateChain(gspec.Config, preBlocks[len(preBlocks)-1], engine, genDb, 8, func(i int, gen *BlockGen) {
			gen.SetPoS()
		})
	}
	// Assemble header batch
	preHeaders := make([]*types.Header, len(preBlocks))
	for i, block := range preBlocks {
		preHeaders[i] = block.Header()
		t.Logf("Pre-merge header: %d", block.NumberU64())
	}
	postHeaders := make([]*types.Header, len(postBlocks))
	for i, block := range postBlocks {
		postHeaders[i] = block.Header()
		t.Logf("Post-merge header: %d", block.NumberU64())
	}
	// Run the header checker for blocks one-by-one, checking for both valid and invalid nonces
	chain, _ := NewBlockChain(rawdb.NewMemoryDatabase(), nil, gspec, nil, engine, vm.Config{}, nil, nil)
	defer chain.Stop()

	// Verify the blocks before the merging
	for i := 0; i < len(preBlocks); i++ {
		_, results := engine.VerifyHeaders(chain, []*types.Header{preHeaders[i]})
		// Wait for the verification result
		select {
		case result := <-results:
			if result != nil {
				t.Errorf("pre-block %d: verification failed %v", i, result)
			}
		case <-time.After(time.Second):
			t.Fatalf("pre-block %d: verification timeout", i)
		}
		// Make sure no more data is returned
		select {
		case result := <-results:
			t.Fatalf("pre-block %d: unexpected result returned: %v", i, result)
		case <-time.After(25 * time.Millisecond):
		}
		chain.InsertChain(preBlocks[i : i+1])
	}

	// Make the transition
	merger.ReachTTD()
	merger.FinalizePoS()

	// Verify the blocks after the merging
	for i := 0; i < len(postBlocks); i++ {
		_, results := engine.VerifyHeaders(chain, []*types.Header{postHeaders[i]})
		// Wait for the verification result
		select {
		case result := <-results:
			if result != nil {
				t.Errorf("post-block %d: verification failed %v", i, result)
			}
		case <-time.After(time.Second):
			t.Fatalf("test %d: verification timeout", i)
		}
		// Make sure no more data is returned
		select {
		case result := <-results:
			t.Fatalf("post-block %d: unexpected result returned: %v", i, result)
		case <-time.After(25 * time.Millisecond):
		}
		chain.InsertBlockWithoutSetHead(postBlocks[i])
	}

	// Verify the blocks with pre-merge blocks and post-merge blocks
	var headers []*types.Header
	for _, block := range preBlocks {
		headers = append(headers, block.Header())
	}
	for _, block := range postBlocks {
		headers = append(headers, block.Header())
	}
	_, results := engine.VerifyHeaders(chain, headers)
	for i := 0; i < len(headers); i++ {
		select {
		case result := <-results:
			if result != nil {
				t.Errorf("test %d: verification failed %v", i, result)
			}
		case <-time.After(time.Second):
			t.Fatalf("test %d: verification timeout", i)
		}
	}
	// Make sure no more data is returned
	select {
	case result := <-results:
		t.Fatalf("unexpected result returned: %v", result)
	case <-time.After(25 * time.Millisecond):
	}
}

func TestCalcGasLimit(t *testing.T) {
	for i, tc := range []struct {
		pGasLimit uint64
		max       uint64
		min       uint64
	}{
		{20000000, 20019530, 19980470},
		{40000000, 40039061, 39960939},
	} {
		// Increase
		if have, want := CalcGasLimit(tc.pGasLimit, 2*tc.pGasLimit), tc.max; have != want {
			t.Errorf("test %d: have %d want <%d", i, have, want)
		}
		// Decrease
		if have, want := CalcGasLimit(tc.pGasLimit, 0), tc.min; have != want {
			t.Errorf("test %d: have %d want >%d", i, have, want)
		}
		// Small decrease
		if have, want := CalcGasLimit(tc.pGasLimit, tc.pGasLimit-1), tc.pGasLimit-1; have != want {
			t.Errorf("test %d: have %d want %d", i, have, want)
		}
		// Small increase
		if have, want := CalcGasLimit(tc.pGasLimit, tc.pGasLimit+1), tc.pGasLimit+1; have != want {
			t.Errorf("test %d: have %d want %d", i, have, want)
		}
		// No change
		if have, want := CalcGasLimit(tc.pGasLimit, tc.pGasLimit), tc.pGasLimit; have != want {
			t.Errorf("test %d: have %d want %d", i, have, want)
		}
	}
}
