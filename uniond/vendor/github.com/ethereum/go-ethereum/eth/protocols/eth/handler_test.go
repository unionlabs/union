// Copyright 2020 The go-ethereum Authors
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

package eth

import (
	"math"
	"math/big"
	"math/rand"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/beacon"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/txpool/legacypool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/core/vm"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/params"
)

var (
	// testKey is a private key to use for funding a tester account.
	testKey, _ = crypto.HexToECDSA("b71c71a67e1177ad4e901695e1b4b9ee17ae16c6668d313eac2f96dbcda3f291")

	// testAddr is the Ethereum address of the tester account.
	testAddr = crypto.PubkeyToAddress(testKey.PublicKey)
)

func u64(val uint64) *uint64 { return &val }

// testBackend is a mock implementation of the live Ethereum message handler. Its
// purpose is to allow testing the request/reply workflows and wire serialization
// in the `eth` protocol without actually doing any data processing.
type testBackend struct {
	db     ethdb.Database
	chain  *core.BlockChain
	txpool *txpool.TxPool
}

// newTestBackend creates an empty chain and wraps it into a mock backend.
func newTestBackend(blocks int) *testBackend {
	return newTestBackendWithGenerator(blocks, false, nil)
}

// newTestBackend creates a chain with a number of explicitly defined blocks and
// wraps it into a mock backend.
func newTestBackendWithGenerator(blocks int, shanghai bool, generator func(int, *core.BlockGen)) *testBackend {
	var (
		// Create a database pre-initialize with a genesis block
		db                      = rawdb.NewMemoryDatabase()
		config                  = params.TestChainConfig
		engine consensus.Engine = ethash.NewFaker()
	)

	if shanghai {
		config = &params.ChainConfig{
			ChainID:                       big.NewInt(1),
			HomesteadBlock:                big.NewInt(0),
			DAOForkBlock:                  nil,
			DAOForkSupport:                true,
			EIP150Block:                   big.NewInt(0),
			EIP155Block:                   big.NewInt(0),
			EIP158Block:                   big.NewInt(0),
			ByzantiumBlock:                big.NewInt(0),
			ConstantinopleBlock:           big.NewInt(0),
			PetersburgBlock:               big.NewInt(0),
			IstanbulBlock:                 big.NewInt(0),
			MuirGlacierBlock:              big.NewInt(0),
			BerlinBlock:                   big.NewInt(0),
			LondonBlock:                   big.NewInt(0),
			ArrowGlacierBlock:             big.NewInt(0),
			GrayGlacierBlock:              big.NewInt(0),
			MergeNetsplitBlock:            big.NewInt(0),
			ShanghaiTime:                  u64(0),
			TerminalTotalDifficulty:       big.NewInt(0),
			TerminalTotalDifficultyPassed: true,
			Ethash:                        new(params.EthashConfig),
		}
		engine = beacon.NewFaker()
	}

	gspec := &core.Genesis{
		Config: config,
		Alloc:  core.GenesisAlloc{testAddr: {Balance: big.NewInt(100_000_000_000_000_000)}},
	}
	chain, _ := core.NewBlockChain(db, nil, gspec, nil, engine, vm.Config{}, nil, nil)

	_, bs, _ := core.GenerateChainWithGenesis(gspec, engine, blocks, generator)
	if _, err := chain.InsertChain(bs); err != nil {
		panic(err)
	}
	for _, block := range bs {
		chain.TrieDB().Commit(block.Root(), false)
	}
	txconfig := legacypool.DefaultConfig
	txconfig.Journal = "" // Don't litter the disk with test journals

	pool := legacypool.New(txconfig, chain)
	txpool, _ := txpool.New(new(big.Int).SetUint64(txconfig.PriceLimit), chain, []txpool.SubPool{pool})

	return &testBackend{
		db:     db,
		chain:  chain,
		txpool: txpool,
	}
}

// close tears down the transaction pool and chain behind the mock backend.
func (b *testBackend) close() {
	b.txpool.Close()
	b.chain.Stop()
}

func (b *testBackend) Chain() *core.BlockChain { return b.chain }
func (b *testBackend) TxPool() TxPool          { return b.txpool }

func (b *testBackend) RunPeer(peer *Peer, handler Handler) error {
	// Normally the backend would do peer maintenance and handshakes. All that
	// is omitted and we will just give control back to the handler.
	return handler(peer)
}
func (b *testBackend) PeerInfo(enode.ID) interface{} { panic("not implemented") }

func (b *testBackend) AcceptTxs() bool {
	panic("data processing tests should be done in the handler package")
}
func (b *testBackend) Handle(*Peer, Packet) error {
	panic("data processing tests should be done in the handler package")
}

// Tests that block headers can be retrieved from a remote chain based on user queries.
func TestGetBlockHeaders67(t *testing.T) { testGetBlockHeaders(t, ETH67) }
func TestGetBlockHeaders68(t *testing.T) { testGetBlockHeaders(t, ETH68) }

func testGetBlockHeaders(t *testing.T, protocol uint) {
	t.Parallel()

	backend := newTestBackend(maxHeadersServe + 15)
	defer backend.close()

	peer, _ := newTestPeer("peer", protocol, backend)
	defer peer.close()

	// Create a "random" unknown hash for testing
	var unknown common.Hash
	for i := range unknown {
		unknown[i] = byte(i)
	}
	getHashes := func(from, limit uint64) (hashes []common.Hash) {
		for i := uint64(0); i < limit; i++ {
			hashes = append(hashes, backend.chain.GetCanonicalHash(from-1-i))
		}
		return hashes
	}
	// Create a batch of tests for various scenarios
	limit := uint64(maxHeadersServe)
	tests := []struct {
		query  *GetBlockHeadersRequest // The query to execute for header retrieval
		expect []common.Hash           // The hashes of the block whose headers are expected
	}{
		// A single random block should be retrievable by hash
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Hash: backend.chain.GetBlockByNumber(limit / 2).Hash()}, Amount: 1},
			[]common.Hash{backend.chain.GetBlockByNumber(limit / 2).Hash()},
		},
		// A single random block should be retrievable by number
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: limit / 2}, Amount: 1},
			[]common.Hash{backend.chain.GetBlockByNumber(limit / 2).Hash()},
		},
		// Multiple headers should be retrievable in both directions
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: limit / 2}, Amount: 3},
			[]common.Hash{
				backend.chain.GetBlockByNumber(limit / 2).Hash(),
				backend.chain.GetBlockByNumber(limit/2 + 1).Hash(),
				backend.chain.GetBlockByNumber(limit/2 + 2).Hash(),
			},
		}, {
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: limit / 2}, Amount: 3, Reverse: true},
			[]common.Hash{
				backend.chain.GetBlockByNumber(limit / 2).Hash(),
				backend.chain.GetBlockByNumber(limit/2 - 1).Hash(),
				backend.chain.GetBlockByNumber(limit/2 - 2).Hash(),
			},
		},
		// Multiple headers with skip lists should be retrievable
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: limit / 2}, Skip: 3, Amount: 3},
			[]common.Hash{
				backend.chain.GetBlockByNumber(limit / 2).Hash(),
				backend.chain.GetBlockByNumber(limit/2 + 4).Hash(),
				backend.chain.GetBlockByNumber(limit/2 + 8).Hash(),
			},
		}, {
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: limit / 2}, Skip: 3, Amount: 3, Reverse: true},
			[]common.Hash{
				backend.chain.GetBlockByNumber(limit / 2).Hash(),
				backend.chain.GetBlockByNumber(limit/2 - 4).Hash(),
				backend.chain.GetBlockByNumber(limit/2 - 8).Hash(),
			},
		},
		// The chain endpoints should be retrievable
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: 0}, Amount: 1},
			[]common.Hash{backend.chain.GetBlockByNumber(0).Hash()},
		},
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64()}, Amount: 1},
			[]common.Hash{backend.chain.CurrentBlock().Hash()},
		},
		{ // If the peer requests a bit into the future, we deliver what we have
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64()}, Amount: 10},
			[]common.Hash{backend.chain.CurrentBlock().Hash()},
		},
		// Ensure protocol limits are honored
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64() - 1}, Amount: limit + 10, Reverse: true},
			getHashes(backend.chain.CurrentBlock().Number.Uint64(), limit),
		},
		// Check that requesting more than available is handled gracefully
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64() - 4}, Skip: 3, Amount: 3},
			[]common.Hash{
				backend.chain.GetBlockByNumber(backend.chain.CurrentBlock().Number.Uint64() - 4).Hash(),
				backend.chain.GetBlockByNumber(backend.chain.CurrentBlock().Number.Uint64()).Hash(),
			},
		}, {
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: 4}, Skip: 3, Amount: 3, Reverse: true},
			[]common.Hash{
				backend.chain.GetBlockByNumber(4).Hash(),
				backend.chain.GetBlockByNumber(0).Hash(),
			},
		},
		// Check that requesting more than available is handled gracefully, even if mid skip
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64() - 4}, Skip: 2, Amount: 3},
			[]common.Hash{
				backend.chain.GetBlockByNumber(backend.chain.CurrentBlock().Number.Uint64() - 4).Hash(),
				backend.chain.GetBlockByNumber(backend.chain.CurrentBlock().Number.Uint64() - 1).Hash(),
			},
		}, {
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: 4}, Skip: 2, Amount: 3, Reverse: true},
			[]common.Hash{
				backend.chain.GetBlockByNumber(4).Hash(),
				backend.chain.GetBlockByNumber(1).Hash(),
			},
		},
		// Check a corner case where requesting more can iterate past the endpoints
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: 2}, Amount: 5, Reverse: true},
			[]common.Hash{
				backend.chain.GetBlockByNumber(2).Hash(),
				backend.chain.GetBlockByNumber(1).Hash(),
				backend.chain.GetBlockByNumber(0).Hash(),
			},
		},
		// Check a corner case where skipping overflow loops back into the chain start
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Hash: backend.chain.GetBlockByNumber(3).Hash()}, Amount: 2, Reverse: false, Skip: math.MaxUint64 - 1},
			[]common.Hash{
				backend.chain.GetBlockByNumber(3).Hash(),
			},
		},
		// Check a corner case where skipping overflow loops back to the same header
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Hash: backend.chain.GetBlockByNumber(1).Hash()}, Amount: 2, Reverse: false, Skip: math.MaxUint64},
			[]common.Hash{
				backend.chain.GetBlockByNumber(1).Hash(),
			},
		},
		// Check that non existing headers aren't returned
		{
			&GetBlockHeadersRequest{Origin: HashOrNumber{Hash: unknown}, Amount: 1},
			[]common.Hash{},
		}, {
			&GetBlockHeadersRequest{Origin: HashOrNumber{Number: backend.chain.CurrentBlock().Number.Uint64() + 1}, Amount: 1},
			[]common.Hash{},
		},
	}
	// Run each of the tests and verify the results against the chain
	for i, tt := range tests {
		// Collect the headers to expect in the response
		var headers []*types.Header
		for _, hash := range tt.expect {
			headers = append(headers, backend.chain.GetBlockByHash(hash).Header())
		}
		// Send the hash request and verify the response
		p2p.Send(peer.app, GetBlockHeadersMsg, &GetBlockHeadersPacket{
			RequestId:              123,
			GetBlockHeadersRequest: tt.query,
		})
		if err := p2p.ExpectMsg(peer.app, BlockHeadersMsg, &BlockHeadersPacket{
			RequestId:           123,
			BlockHeadersRequest: headers,
		}); err != nil {
			t.Errorf("test %d: headers mismatch: %v", i, err)
		}
		// If the test used number origins, repeat with hashes as the too
		if tt.query.Origin.Hash == (common.Hash{}) {
			if origin := backend.chain.GetBlockByNumber(tt.query.Origin.Number); origin != nil {
				tt.query.Origin.Hash, tt.query.Origin.Number = origin.Hash(), 0

				p2p.Send(peer.app, GetBlockHeadersMsg, &GetBlockHeadersPacket{
					RequestId:              456,
					GetBlockHeadersRequest: tt.query,
				})
				expected := &BlockHeadersPacket{RequestId: 456, BlockHeadersRequest: headers}
				if err := p2p.ExpectMsg(peer.app, BlockHeadersMsg, expected); err != nil {
					t.Errorf("test %d by hash: headers mismatch: %v", i, err)
				}
			}
		}
	}
}

// Tests that block contents can be retrieved from a remote chain based on their hashes.
func TestGetBlockBodies67(t *testing.T) { testGetBlockBodies(t, ETH67) }
func TestGetBlockBodies68(t *testing.T) { testGetBlockBodies(t, ETH68) }

func testGetBlockBodies(t *testing.T, protocol uint) {
	t.Parallel()

	gen := func(n int, g *core.BlockGen) {
		if n%2 == 0 {
			w := &types.Withdrawal{
				Address: common.Address{0xaa},
				Amount:  42,
			}
			g.AddWithdrawal(w)
		}
	}

	backend := newTestBackendWithGenerator(maxBodiesServe+15, true, gen)
	defer backend.close()

	peer, _ := newTestPeer("peer", protocol, backend)
	defer peer.close()

	// Create a batch of tests for various scenarios
	limit := maxBodiesServe
	tests := []struct {
		random    int           // Number of blocks to fetch randomly from the chain
		explicit  []common.Hash // Explicitly requested blocks
		available []bool        // Availability of explicitly requested blocks
		expected  int           // Total number of existing blocks to expect
	}{
		{1, nil, nil, 1},             // A single random block should be retrievable
		{10, nil, nil, 10},           // Multiple random blocks should be retrievable
		{limit, nil, nil, limit},     // The maximum possible blocks should be retrievable
		{limit + 1, nil, nil, limit}, // No more than the possible block count should be returned
		{0, []common.Hash{backend.chain.Genesis().Hash()}, []bool{true}, 1},      // The genesis block should be retrievable
		{0, []common.Hash{backend.chain.CurrentBlock().Hash()}, []bool{true}, 1}, // The chains head block should be retrievable
		{0, []common.Hash{{}}, []bool{false}, 0},                                 // A non existent block should not be returned

		// Existing and non-existing blocks interleaved should not cause problems
		{0, []common.Hash{
			{},
			backend.chain.GetBlockByNumber(1).Hash(),
			{},
			backend.chain.GetBlockByNumber(10).Hash(),
			{},
			backend.chain.GetBlockByNumber(100).Hash(),
			{},
		}, []bool{false, true, false, true, false, true, false}, 3},
	}
	// Run each of the tests and verify the results against the chain
	for i, tt := range tests {
		// Collect the hashes to request, and the response to expect
		var (
			hashes []common.Hash
			bodies []*BlockBody
			seen   = make(map[int64]bool)
		)
		for j := 0; j < tt.random; j++ {
			for {
				num := rand.Int63n(int64(backend.chain.CurrentBlock().Number.Uint64()))
				if !seen[num] {
					seen[num] = true

					block := backend.chain.GetBlockByNumber(uint64(num))
					hashes = append(hashes, block.Hash())
					if len(bodies) < tt.expected {
						bodies = append(bodies, &BlockBody{Transactions: block.Transactions(), Uncles: block.Uncles(), Withdrawals: block.Withdrawals()})
					}
					break
				}
			}
		}
		for j, hash := range tt.explicit {
			hashes = append(hashes, hash)
			if tt.available[j] && len(bodies) < tt.expected {
				block := backend.chain.GetBlockByHash(hash)
				bodies = append(bodies, &BlockBody{Transactions: block.Transactions(), Uncles: block.Uncles(), Withdrawals: block.Withdrawals()})
			}
		}

		// Send the hash request and verify the response
		p2p.Send(peer.app, GetBlockBodiesMsg, &GetBlockBodiesPacket{
			RequestId:             123,
			GetBlockBodiesRequest: hashes,
		})
		if err := p2p.ExpectMsg(peer.app, BlockBodiesMsg, &BlockBodiesPacket{
			RequestId:           123,
			BlockBodiesResponse: bodies,
		}); err != nil {
			t.Fatalf("test %d: bodies mismatch: %v", i, err)
		}
	}
}

// Tests that the transaction receipts can be retrieved based on hashes.
func TestGetBlockReceipts67(t *testing.T) { testGetBlockReceipts(t, ETH67) }
func TestGetBlockReceipts68(t *testing.T) { testGetBlockReceipts(t, ETH68) }

func testGetBlockReceipts(t *testing.T, protocol uint) {
	t.Parallel()

	// Define three accounts to simulate transactions with
	acc1Key, _ := crypto.HexToECDSA("8a1f9a8f95be41cd7ccb6168179afb4504aefe388d1e14474d32c45c72ce7b7a")
	acc2Key, _ := crypto.HexToECDSA("49a7b37aa6f6645917e7b807e9d1c00d4fa71f18343b0d4122a4d2df64dd6fee")
	acc1Addr := crypto.PubkeyToAddress(acc1Key.PublicKey)
	acc2Addr := crypto.PubkeyToAddress(acc2Key.PublicKey)

	signer := types.HomesteadSigner{}
	// Create a chain generator with some simple transactions (blatantly stolen from @fjl/chain_markets_test)
	generator := func(i int, block *core.BlockGen) {
		switch i {
		case 0:
			// In block 1, the test bank sends account #1 some ether.
			tx, _ := types.SignTx(types.NewTransaction(block.TxNonce(testAddr), acc1Addr, big.NewInt(10_000_000_000_000_000), params.TxGas, block.BaseFee(), nil), signer, testKey)
			block.AddTx(tx)
		case 1:
			// In block 2, the test bank sends some more ether to account #1.
			// acc1Addr passes it on to account #2.
			tx1, _ := types.SignTx(types.NewTransaction(block.TxNonce(testAddr), acc1Addr, big.NewInt(1_000_000_000_000_000), params.TxGas, block.BaseFee(), nil), signer, testKey)
			tx2, _ := types.SignTx(types.NewTransaction(block.TxNonce(acc1Addr), acc2Addr, big.NewInt(1_000_000_000_000_000), params.TxGas, block.BaseFee(), nil), signer, acc1Key)
			block.AddTx(tx1)
			block.AddTx(tx2)
		case 2:
			// Block 3 is empty but was mined by account #2.
			block.SetCoinbase(acc2Addr)
			block.SetExtra([]byte("yeehaw"))
		case 3:
			// Block 4 includes blocks 2 and 3 as uncle headers (with modified extra data).
			b2 := block.PrevBlock(1).Header()
			b2.Extra = []byte("foo")
			block.AddUncle(b2)
			b3 := block.PrevBlock(2).Header()
			b3.Extra = []byte("foo")
			block.AddUncle(b3)
		}
	}
	// Assemble the test environment
	backend := newTestBackendWithGenerator(4, false, generator)
	defer backend.close()

	peer, _ := newTestPeer("peer", protocol, backend)
	defer peer.close()

	// Collect the hashes to request, and the response to expect
	var (
		hashes   []common.Hash
		receipts [][]*types.Receipt
	)
	for i := uint64(0); i <= backend.chain.CurrentBlock().Number.Uint64(); i++ {
		block := backend.chain.GetBlockByNumber(i)

		hashes = append(hashes, block.Hash())
		receipts = append(receipts, backend.chain.GetReceiptsByHash(block.Hash()))
	}
	// Send the hash request and verify the response
	p2p.Send(peer.app, GetReceiptsMsg, &GetReceiptsPacket{
		RequestId:          123,
		GetReceiptsRequest: hashes,
	})
	if err := p2p.ExpectMsg(peer.app, ReceiptsMsg, &ReceiptsPacket{
		RequestId:        123,
		ReceiptsResponse: receipts,
	}); err != nil {
		t.Errorf("receipts mismatch: %v", err)
	}
}
