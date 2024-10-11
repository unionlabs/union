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

package les

import (
	"encoding/binary"
	"math/big"
	"math/rand"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/eth/downloader"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

func expectResponse(r p2p.MsgReader, msgcode, reqID, bv uint64, data interface{}) error {
	type resp struct {
		ReqID, BV uint64
		Data      interface{}
	}
	return p2p.ExpectMsg(r, msgcode, resp{reqID, bv, data})
}

// Tests that block headers can be retrieved from a remote chain based on user queries.
func TestGetBlockHeadersLes2(t *testing.T) { testGetBlockHeaders(t, 2) }
func TestGetBlockHeadersLes3(t *testing.T) { testGetBlockHeaders(t, 3) }
func TestGetBlockHeadersLes4(t *testing.T) { testGetBlockHeaders(t, 4) }

func testGetBlockHeaders(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		blocks:    downloader.MaxHeaderFetch + 15,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()
	bc := server.handler.blockchain

	// Create a "random" unknown hash for testing
	var unknown common.Hash
	for i := range unknown {
		unknown[i] = byte(i)
	}
	// Create a batch of tests for various scenarios
	limit := uint64(MaxHeaderFetch)
	tests := []struct {
		query  *GetBlockHeadersData // The query to execute for header retrieval
		expect []common.Hash        // The hashes of the block whose headers are expected
	}{
		// A single random block should be retrievable by hash and number too
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Hash: bc.GetBlockByNumber(limit / 2).Hash()}, Amount: 1},
			[]common.Hash{bc.GetBlockByNumber(limit / 2).Hash()},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: limit / 2}, Amount: 1},
			[]common.Hash{bc.GetBlockByNumber(limit / 2).Hash()},
		},
		// Multiple headers should be retrievable in both directions
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Number: limit / 2}, Amount: 3},
			[]common.Hash{
				bc.GetBlockByNumber(limit / 2).Hash(),
				bc.GetBlockByNumber(limit/2 + 1).Hash(),
				bc.GetBlockByNumber(limit/2 + 2).Hash(),
			},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: limit / 2}, Amount: 3, Reverse: true},
			[]common.Hash{
				bc.GetBlockByNumber(limit / 2).Hash(),
				bc.GetBlockByNumber(limit/2 - 1).Hash(),
				bc.GetBlockByNumber(limit/2 - 2).Hash(),
			},
		},
		// Multiple headers with skip lists should be retrievable
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Number: limit / 2}, Skip: 3, Amount: 3},
			[]common.Hash{
				bc.GetBlockByNumber(limit / 2).Hash(),
				bc.GetBlockByNumber(limit/2 + 4).Hash(),
				bc.GetBlockByNumber(limit/2 + 8).Hash(),
			},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: limit / 2}, Skip: 3, Amount: 3, Reverse: true},
			[]common.Hash{
				bc.GetBlockByNumber(limit / 2).Hash(),
				bc.GetBlockByNumber(limit/2 - 4).Hash(),
				bc.GetBlockByNumber(limit/2 - 8).Hash(),
			},
		},
		// The chain endpoints should be retrievable
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Number: 0}, Amount: 1},
			[]common.Hash{bc.GetBlockByNumber(0).Hash()},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: bc.CurrentBlock().Number.Uint64()}, Amount: 1},
			[]common.Hash{bc.CurrentBlock().Hash()},
		},
		// Ensure protocol limits are honored
		//{
		//	&GetBlockHeadersData{Origin: hashOrNumber{Number: bc.CurrentBlock().Number.Uint64()() - 1}, Amount: limit + 10, Reverse: true},
		//	[]common.Hash{},
		//},
		// Check that requesting more than available is handled gracefully
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Number: bc.CurrentBlock().Number.Uint64() - 4}, Skip: 3, Amount: 3},
			[]common.Hash{
				bc.GetBlockByNumber(bc.CurrentBlock().Number.Uint64() - 4).Hash(),
				bc.GetBlockByNumber(bc.CurrentBlock().Number.Uint64()).Hash(),
			},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: 4}, Skip: 3, Amount: 3, Reverse: true},
			[]common.Hash{
				bc.GetBlockByNumber(4).Hash(),
				bc.GetBlockByNumber(0).Hash(),
			},
		},
		// Check that requesting more than available is handled gracefully, even if mid skip
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Number: bc.CurrentBlock().Number.Uint64() - 4}, Skip: 2, Amount: 3},
			[]common.Hash{
				bc.GetBlockByNumber(bc.CurrentBlock().Number.Uint64() - 4).Hash(),
				bc.GetBlockByNumber(bc.CurrentBlock().Number.Uint64() - 1).Hash(),
			},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: 4}, Skip: 2, Amount: 3, Reverse: true},
			[]common.Hash{
				bc.GetBlockByNumber(4).Hash(),
				bc.GetBlockByNumber(1).Hash(),
			},
		},
		// Check that non existing headers aren't returned
		{
			&GetBlockHeadersData{Origin: hashOrNumber{Hash: unknown}, Amount: 1},
			[]common.Hash{},
		}, {
			&GetBlockHeadersData{Origin: hashOrNumber{Number: bc.CurrentBlock().Number.Uint64() + 1}, Amount: 1},
			[]common.Hash{},
		},
	}
	// Run each of the tests and verify the results against the chain
	var reqID uint64
	for i, tt := range tests {
		// Collect the headers to expect in the response
		var headers []*types.Header
		for _, hash := range tt.expect {
			headers = append(headers, bc.GetHeaderByHash(hash))
		}
		// Send the hash request and verify the response
		reqID++

		sendRequest(rawPeer.app, GetBlockHeadersMsg, reqID, tt.query)
		if err := expectResponse(rawPeer.app, BlockHeadersMsg, reqID, testBufLimit, headers); err != nil {
			t.Errorf("test %d: headers mismatch: %v", i, err)
		}
	}
}

// Tests that block contents can be retrieved from a remote chain based on their hashes.
func TestGetBlockBodiesLes2(t *testing.T) { testGetBlockBodies(t, 2) }
func TestGetBlockBodiesLes3(t *testing.T) { testGetBlockBodies(t, 3) }
func TestGetBlockBodiesLes4(t *testing.T) { testGetBlockBodies(t, 4) }

func testGetBlockBodies(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		blocks:    downloader.MaxHeaderFetch + 15,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	// Create a batch of tests for various scenarios
	limit := MaxBodyFetch
	tests := []struct {
		random    int           // Number of blocks to fetch randomly from the chain
		explicit  []common.Hash // Explicitly requested blocks
		available []bool        // Availability of explicitly requested blocks
		expected  int           // Total number of existing blocks to expect
	}{
		{1, nil, nil, 1},         // A single random block should be retrievable
		{10, nil, nil, 10},       // Multiple random blocks should be retrievable
		{limit, nil, nil, limit}, // The maximum possible blocks should be retrievable
		//{limit + 1, nil, nil, limit},                                  // No more than the possible block count should be returned
		{0, []common.Hash{bc.Genesis().Hash()}, []bool{true}, 1},      // The genesis block should be retrievable
		{0, []common.Hash{bc.CurrentBlock().Hash()}, []bool{true}, 1}, // The chains head block should be retrievable
		{0, []common.Hash{{}}, []bool{false}, 0},                      // A non existent block should not be returned

		// Existing and non-existing blocks interleaved should not cause problems
		{0, []common.Hash{
			{},
			bc.GetBlockByNumber(1).Hash(),
			{},
			bc.GetBlockByNumber(10).Hash(),
			{},
			bc.GetBlockByNumber(100).Hash(),
			{},
		}, []bool{false, true, false, true, false, true, false}, 3},
	}
	// Run each of the tests and verify the results against the chain
	var reqID uint64
	for i, tt := range tests {
		// Collect the hashes to request, and the response to expect
		var hashes []common.Hash
		seen := make(map[int64]bool)
		var bodies []*types.Body

		for j := 0; j < tt.random; j++ {
			for {
				num := rand.Int63n(int64(bc.CurrentBlock().Number.Uint64()))
				if !seen[num] {
					seen[num] = true

					block := bc.GetBlockByNumber(uint64(num))
					hashes = append(hashes, block.Hash())
					if len(bodies) < tt.expected {
						bodies = append(bodies, &types.Body{Transactions: block.Transactions(), Uncles: block.Uncles()})
					}
					break
				}
			}
		}
		for j, hash := range tt.explicit {
			hashes = append(hashes, hash)
			if tt.available[j] && len(bodies) < tt.expected {
				block := bc.GetBlockByHash(hash)
				bodies = append(bodies, &types.Body{Transactions: block.Transactions(), Uncles: block.Uncles()})
			}
		}
		reqID++

		// Send the hash request and verify the response
		sendRequest(rawPeer.app, GetBlockBodiesMsg, reqID, hashes)
		if err := expectResponse(rawPeer.app, BlockBodiesMsg, reqID, testBufLimit, bodies); err != nil {
			t.Errorf("test %d: bodies mismatch: %v", i, err)
		}
	}
}

// Tests that the contract codes can be retrieved based on account addresses.
func TestGetCodeLes2(t *testing.T) { testGetCode(t, 2) }
func TestGetCodeLes3(t *testing.T) { testGetCode(t, 3) }
func TestGetCodeLes4(t *testing.T) { testGetCode(t, 4) }

func testGetCode(t *testing.T, protocol int) {
	// Assemble the test environment
	netconfig := testnetConfig{
		blocks:    4,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	var codereqs []*CodeReq
	var codes [][]byte
	for i := uint64(0); i <= bc.CurrentBlock().Number.Uint64(); i++ {
		header := bc.GetHeaderByNumber(i)
		req := &CodeReq{
			BHash:          header.Hash(),
			AccountAddress: testContractAddr[:],
		}
		codereqs = append(codereqs, req)
		if i >= testContractDeployed {
			codes = append(codes, testContractCodeDeployed)
		}
	}

	sendRequest(rawPeer.app, GetCodeMsg, 42, codereqs)
	if err := expectResponse(rawPeer.app, CodeMsg, 42, testBufLimit, codes); err != nil {
		t.Errorf("codes mismatch: %v", err)
	}
}

// Tests that the stale contract codes can't be retrieved based on account addresses.
func TestGetStaleCodeLes2(t *testing.T) { testGetStaleCode(t, 2) }
func TestGetStaleCodeLes3(t *testing.T) { testGetStaleCode(t, 3) }
func TestGetStaleCodeLes4(t *testing.T) { testGetStaleCode(t, 4) }

func testGetStaleCode(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		blocks:    core.TriesInMemory + 4,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	check := func(number uint64, expected [][]byte) {
		req := &CodeReq{
			BHash:          bc.GetHeaderByNumber(number).Hash(),
			AccountAddress: testContractAddr[:],
		}
		sendRequest(rawPeer.app, GetCodeMsg, 42, []*CodeReq{req})
		if err := expectResponse(rawPeer.app, CodeMsg, 42, testBufLimit, expected); err != nil {
			t.Errorf("codes mismatch: %v", err)
		}
	}
	check(0, [][]byte{})                                                          // Non-exist contract
	check(testContractDeployed, [][]byte{})                                       // Stale contract
	check(bc.CurrentHeader().Number.Uint64(), [][]byte{testContractCodeDeployed}) // Fresh contract
}

// Tests that the transaction receipts can be retrieved based on hashes.
func TestGetReceiptLes2(t *testing.T) { testGetReceipt(t, 2) }
func TestGetReceiptLes3(t *testing.T) { testGetReceipt(t, 3) }
func TestGetReceiptLes4(t *testing.T) { testGetReceipt(t, 4) }

func testGetReceipt(t *testing.T, protocol int) {
	// Assemble the test environment
	netconfig := testnetConfig{
		blocks:    4,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	// Collect the hashes to request, and the response to expect
	var receipts []types.Receipts
	var hashes []common.Hash
	for i := uint64(0); i <= bc.CurrentBlock().Number.Uint64(); i++ {
		block := bc.GetBlockByNumber(i)

		hashes = append(hashes, block.Hash())
		receipts = append(receipts, rawdb.ReadReceipts(server.db, block.Hash(), block.NumberU64(), block.Time(), bc.Config()))
	}
	// Send the hash request and verify the response
	sendRequest(rawPeer.app, GetReceiptsMsg, 42, hashes)
	if err := expectResponse(rawPeer.app, ReceiptsMsg, 42, testBufLimit, receipts); err != nil {
		t.Errorf("receipts mismatch: %v", err)
	}
}

// Tests that trie merkle proofs can be retrieved
func TestGetProofsLes2(t *testing.T) { testGetProofs(t, 2) }
func TestGetProofsLes3(t *testing.T) { testGetProofs(t, 3) }
func TestGetProofsLes4(t *testing.T) { testGetProofs(t, 4) }

func testGetProofs(t *testing.T, protocol int) {
	// Assemble the test environment
	netconfig := testnetConfig{
		blocks:    4,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	var proofreqs []ProofReq
	proofsV2 := trienode.NewProofSet()

	accounts := []common.Address{bankAddr, userAddr1, userAddr2, signerAddr, {}}
	for i := uint64(0); i <= bc.CurrentBlock().Number.Uint64(); i++ {
		header := bc.GetHeaderByNumber(i)
		trie, _ := trie.New(trie.StateTrieID(header.Root), server.backend.Blockchain().TrieDB())

		for _, acc := range accounts {
			req := ProofReq{
				BHash: header.Hash(),
				Key:   crypto.Keccak256(acc[:]),
			}
			proofreqs = append(proofreqs, req)
			trie.Prove(crypto.Keccak256(acc[:]), proofsV2)
		}
	}
	// Send the proof request and verify the response
	sendRequest(rawPeer.app, GetProofsV2Msg, 42, proofreqs)
	if err := expectResponse(rawPeer.app, ProofsV2Msg, 42, testBufLimit, proofsV2.List()); err != nil {
		t.Errorf("proofs mismatch: %v", err)
	}
}

// Tests that the stale contract codes can't be retrieved based on account addresses.
func TestGetStaleProofLes2(t *testing.T) { testGetStaleProof(t, 2) }
func TestGetStaleProofLes3(t *testing.T) { testGetStaleProof(t, 3) }
func TestGetStaleProofLes4(t *testing.T) { testGetStaleProof(t, 4) }

func testGetStaleProof(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		blocks:    core.TriesInMemory + 4,
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	check := func(number uint64, wantOK bool) {
		var (
			header  = bc.GetHeaderByNumber(number)
			account = crypto.Keccak256(userAddr1.Bytes())
		)
		req := &ProofReq{
			BHash: header.Hash(),
			Key:   account,
		}
		sendRequest(rawPeer.app, GetProofsV2Msg, 42, []*ProofReq{req})

		var expected []rlp.RawValue
		if wantOK {
			proofsV2 := trienode.NewProofSet()
			t, _ := trie.New(trie.StateTrieID(header.Root), server.backend.Blockchain().TrieDB())
			t.Prove(account, proofsV2)
			expected = proofsV2.List()
		}
		if err := expectResponse(rawPeer.app, ProofsV2Msg, 42, testBufLimit, expected); err != nil {
			t.Errorf("codes mismatch: %v", err)
		}
	}
	check(0, false)                                 // Non-exist proof
	check(2, false)                                 // Stale proof
	check(bc.CurrentHeader().Number.Uint64(), true) // Fresh proof
}

// Tests that CHT proofs can be correctly retrieved.
func TestGetCHTProofsLes2(t *testing.T) { testGetCHTProofs(t, 2) }
func TestGetCHTProofsLes3(t *testing.T) { testGetCHTProofs(t, 3) }
func TestGetCHTProofsLes4(t *testing.T) { testGetCHTProofs(t, 4) }

func testGetCHTProofs(t *testing.T, protocol int) {
	var (
		config       = light.TestServerIndexerConfig
		waitIndexers = func(cIndexer, bIndexer, btIndexer *core.ChainIndexer) {
			for {
				cs, _, _ := cIndexer.Sections()
				if cs >= 1 {
					break
				}
				time.Sleep(10 * time.Millisecond)
			}
		}
		netconfig = testnetConfig{
			blocks:    int(config.ChtSize + config.ChtConfirms),
			protocol:  protocol,
			indexFn:   waitIndexers,
			nopruning: true,
		}
	)
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	// Assemble the proofs from the different protocols
	header := bc.GetHeaderByNumber(config.ChtSize - 1)
	rlp, _ := rlp.EncodeToBytes(header)

	key := make([]byte, 8)
	binary.BigEndian.PutUint64(key, config.ChtSize-1)

	proofsV2 := HelperTrieResps{
		AuxData: [][]byte{rlp},
	}
	root := light.GetChtRoot(server.db, 0, bc.GetHeaderByNumber(config.ChtSize-1).Hash())
	trie, _ := trie.New(trie.TrieID(root), trie.NewDatabase(rawdb.NewTable(server.db, string(rawdb.ChtTablePrefix)), trie.HashDefaults))
	trie.Prove(key, &proofsV2.Proofs)
	// Assemble the requests for the different protocols
	requestsV2 := []HelperTrieReq{{
		Type:    htCanonical,
		TrieIdx: 0,
		Key:     key,
		AuxReq:  htAuxHeader,
	}}
	// Send the proof request and verify the response
	sendRequest(rawPeer.app, GetHelperTrieProofsMsg, 42, requestsV2)
	if err := expectResponse(rawPeer.app, HelperTrieProofsMsg, 42, testBufLimit, proofsV2); err != nil {
		t.Errorf("proofs mismatch: %v", err)
	}
}

func TestGetBloombitsProofsLes2(t *testing.T) { testGetBloombitsProofs(t, 2) }
func TestGetBloombitsProofsLes3(t *testing.T) { testGetBloombitsProofs(t, 3) }
func TestGetBloombitsProofsLes4(t *testing.T) { testGetBloombitsProofs(t, 4) }

// Tests that bloombits proofs can be correctly retrieved.
func testGetBloombitsProofs(t *testing.T, protocol int) {
	var (
		config       = light.TestServerIndexerConfig
		waitIndexers = func(cIndexer, bIndexer, btIndexer *core.ChainIndexer) {
			for {
				bts, _, _ := btIndexer.Sections()
				if bts >= 1 {
					break
				}
				time.Sleep(10 * time.Millisecond)
			}
		}
		netconfig = testnetConfig{
			blocks:    int(config.BloomTrieSize + config.BloomTrieConfirms),
			protocol:  protocol,
			indexFn:   waitIndexers,
			nopruning: true,
		}
	)
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	bc := server.handler.blockchain

	// Request and verify each bit of the bloom bits proofs
	for bit := 0; bit < 2048; bit++ {
		// Assemble the request and proofs for the bloombits
		key := make([]byte, 10)

		binary.BigEndian.PutUint16(key[:2], uint16(bit))
		// Only the first bloom section has data.
		binary.BigEndian.PutUint64(key[2:], 0)

		requests := []HelperTrieReq{{
			Type:    htBloomBits,
			TrieIdx: 0,
			Key:     key,
		}}
		var proofs HelperTrieResps

		root := light.GetBloomTrieRoot(server.db, 0, bc.GetHeaderByNumber(config.BloomTrieSize-1).Hash())
		trie, _ := trie.New(trie.TrieID(root), trie.NewDatabase(rawdb.NewTable(server.db, string(rawdb.BloomTrieTablePrefix)), trie.HashDefaults))
		trie.Prove(key, &proofs.Proofs)

		// Send the proof request and verify the response
		sendRequest(rawPeer.app, GetHelperTrieProofsMsg, 42, requests)
		if err := expectResponse(rawPeer.app, HelperTrieProofsMsg, 42, testBufLimit, proofs); err != nil {
			t.Errorf("bit %d: proofs mismatch: %v", bit, err)
		}
	}
}

func TestTransactionStatusLes2(t *testing.T) { testTransactionStatus(t, lpv2) }
func TestTransactionStatusLes3(t *testing.T) { testTransactionStatus(t, lpv3) }
func TestTransactionStatusLes4(t *testing.T) { testTransactionStatus(t, lpv4) }

func testTransactionStatus(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		protocol:  protocol,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	server.handler.addTxsSync = true

	chain := server.handler.blockchain

	var reqID uint64

	test := func(tx *types.Transaction, send bool, expStatus light.TxStatus) {
		t.Helper()

		reqID++
		if send {
			sendRequest(rawPeer.app, SendTxV2Msg, reqID, types.Transactions{tx})
		} else {
			sendRequest(rawPeer.app, GetTxStatusMsg, reqID, []common.Hash{tx.Hash()})
		}
		if err := expectResponse(rawPeer.app, TxStatusMsg, reqID, testBufLimit, []light.TxStatus{expStatus}); err != nil {
			t.Errorf("transaction status mismatch: %v", err)
		}
	}
	signer := types.HomesteadSigner{}

	// test error status by sending an underpriced transaction
	tx0, _ := types.SignTx(types.NewTransaction(0, userAddr1, big.NewInt(10000), params.TxGas, nil, nil), signer, bankKey)
	test(tx0, true, light.TxStatus{Status: txpool.TxStatusUnknown, Error: "transaction underpriced: tip needed 1, tip permitted 0"})

	tx1, _ := types.SignTx(types.NewTransaction(0, userAddr1, big.NewInt(10000), params.TxGas, big.NewInt(100000000000), nil), signer, bankKey)
	test(tx1, false, light.TxStatus{Status: txpool.TxStatusUnknown}) // query before sending, should be unknown
	test(tx1, true, light.TxStatus{Status: txpool.TxStatusPending})  // send valid processable tx, should return pending
	test(tx1, true, light.TxStatus{Status: txpool.TxStatusPending})  // adding it again should not return an error

	tx2, _ := types.SignTx(types.NewTransaction(1, userAddr1, big.NewInt(10000), params.TxGas, big.NewInt(100000000000), nil), signer, bankKey)
	tx3, _ := types.SignTx(types.NewTransaction(2, userAddr1, big.NewInt(10000), params.TxGas, big.NewInt(100000000000), nil), signer, bankKey)
	// send transactions in the wrong order, tx3 should be queued
	test(tx3, true, light.TxStatus{Status: txpool.TxStatusQueued})
	test(tx2, true, light.TxStatus{Status: txpool.TxStatusPending})
	// query again, now tx3 should be pending too
	test(tx3, false, light.TxStatus{Status: txpool.TxStatusPending})

	// generate and add a block with tx1 and tx2 included
	gchain, _ := core.GenerateChain(params.TestChainConfig, chain.GetBlockByNumber(0), ethash.NewFaker(), server.db, 1, func(i int, block *core.BlockGen) {
		block.AddTx(tx1)
		block.AddTx(tx2)
	})
	if _, err := chain.InsertChain(gchain); err != nil {
		panic(err)
	}
	// wait until TxPool processes the inserted block
	for i := 0; i < 10; i++ {
		if pending, _ := server.handler.txpool.Stats(); pending == 1 {
			break
		}
		time.Sleep(100 * time.Millisecond)
	}
	if pending, _ := server.handler.txpool.Stats(); pending != 1 {
		t.Fatalf("pending count mismatch: have %d, want 1", pending)
	}
	// Discard new block announcement
	msg, _ := rawPeer.app.ReadMsg()
	msg.Discard()

	// check if their status is included now
	block1hash := rawdb.ReadCanonicalHash(server.db, 1)
	test(tx1, false, light.TxStatus{Status: txpool.TxStatusIncluded, Lookup: &rawdb.LegacyTxLookupEntry{BlockHash: block1hash, BlockIndex: 1, Index: 0}})

	test(tx2, false, light.TxStatus{Status: txpool.TxStatusIncluded, Lookup: &rawdb.LegacyTxLookupEntry{BlockHash: block1hash, BlockIndex: 1, Index: 1}})

	// create a reorg that rolls them back
	gchain, _ = core.GenerateChain(params.TestChainConfig, chain.GetBlockByNumber(0), ethash.NewFaker(), server.db, 2, func(i int, block *core.BlockGen) {})
	if _, err := chain.InsertChain(gchain); err != nil {
		panic(err)
	}
	// wait until TxPool processes the reorg
	for i := 0; i < 10; i++ {
		if pending, _ := server.handler.txpool.Stats(); pending == 3 {
			break
		}
		time.Sleep(100 * time.Millisecond)
	}
	if pending, _ := server.handler.txpool.Stats(); pending != 3 {
		t.Fatalf("pending count mismatch: have %d, want 3", pending)
	}
	// Discard new block announcement
	msg, _ = rawPeer.app.ReadMsg()
	msg.Discard()

	// check if their status is pending again
	test(tx1, false, light.TxStatus{Status: txpool.TxStatusPending})
	test(tx2, false, light.TxStatus{Status: txpool.TxStatusPending})
}

func TestStopResumeLES3(t *testing.T) { testStopResume(t, lpv3) }
func TestStopResumeLES4(t *testing.T) { testStopResume(t, lpv4) }

func testStopResume(t *testing.T, protocol int) {
	netconfig := testnetConfig{
		protocol:  protocol,
		simClock:  true,
		nopruning: true,
	}
	server, _, tearDown := newClientServerEnv(t, netconfig)
	defer tearDown()

	server.handler.server.costTracker.testing = true
	server.handler.server.costTracker.testCostList = testCostList(testBufLimit / 10)

	rawPeer, closePeer, _ := server.newRawPeer(t, "peer", protocol)
	defer closePeer()

	var (
		reqID    uint64
		expBuf   = testBufLimit
		testCost = testBufLimit / 10
	)
	header := server.handler.blockchain.CurrentHeader()
	req := func() {
		reqID++
		sendRequest(rawPeer.app, GetBlockHeadersMsg, reqID, &GetBlockHeadersData{Origin: hashOrNumber{Hash: header.Hash()}, Amount: 1})
	}
	for i := 1; i <= 5; i++ {
		// send requests while we still have enough buffer and expect a response
		for expBuf >= testCost {
			req()
			expBuf -= testCost
			if err := expectResponse(rawPeer.app, BlockHeadersMsg, reqID, expBuf, []*types.Header{header}); err != nil {
				t.Errorf("expected response and failed: %v", err)
			}
		}
		// send some more requests in excess and expect a single StopMsg
		c := i
		for c > 0 {
			req()
			c--
		}
		if err := p2p.ExpectMsg(rawPeer.app, StopMsg, nil); err != nil {
			t.Errorf("expected StopMsg and failed: %v", err)
		}
		// wait until the buffer is recharged by half of the limit
		wait := testBufLimit / testBufRecharge / 2
		server.clock.(*mclock.Simulated).Run(time.Millisecond * time.Duration(wait))

		// expect a ResumeMsg with the partially recharged buffer value
		expBuf += testBufRecharge * wait
		if err := p2p.ExpectMsg(rawPeer.app, ResumeMsg, expBuf); err != nil {
			t.Errorf("expected ResumeMsg and failed: %v", err)
		}
	}
}
