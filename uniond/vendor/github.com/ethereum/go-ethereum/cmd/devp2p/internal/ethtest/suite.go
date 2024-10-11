// Copyright 2020 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

package ethtest

import (
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/internal/utesting"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

// Suite represents a structure used to test a node's conformance
// to the eth protocol.
type Suite struct {
	Dest *enode.Node

	chain     *Chain
	fullChain *Chain
}

// NewSuite creates and returns a new eth-test suite that can
// be used to test the given node against the given blockchain
// data.
func NewSuite(dest *enode.Node, chainfile string, genesisfile string) (*Suite, error) {
	chain, err := loadChain(chainfile, genesisfile)
	if err != nil {
		return nil, err
	}
	return &Suite{
		Dest:      dest,
		chain:     chain.Shorten(1000),
		fullChain: chain,
	}, nil
}

func (s *Suite) EthTests() []utesting.Test {
	return []utesting.Test{
		// status
		{Name: "TestStatus", Fn: s.TestStatus},
		// get block headers
		{Name: "TestGetBlockHeaders", Fn: s.TestGetBlockHeaders},
		{Name: "TestSimultaneousRequests", Fn: s.TestSimultaneousRequests},
		{Name: "TestSameRequestID", Fn: s.TestSameRequestID},
		{Name: "TestZeroRequestID", Fn: s.TestZeroRequestID},
		// get block bodies
		{Name: "TestGetBlockBodies", Fn: s.TestGetBlockBodies},
		// broadcast
		{Name: "TestBroadcast", Fn: s.TestBroadcast},
		{Name: "TestLargeAnnounce", Fn: s.TestLargeAnnounce},
		{Name: "TestOldAnnounce", Fn: s.TestOldAnnounce},
		{Name: "TestBlockHashAnnounce", Fn: s.TestBlockHashAnnounce},
		// malicious handshakes + status
		{Name: "TestMaliciousHandshake", Fn: s.TestMaliciousHandshake},
		{Name: "TestMaliciousStatus", Fn: s.TestMaliciousStatus},
		// test transactions
		{Name: "TestTransaction", Fn: s.TestTransaction},
		{Name: "TestMaliciousTx", Fn: s.TestMaliciousTx},
		{Name: "TestLargeTxRequest", Fn: s.TestLargeTxRequest},
		{Name: "TestNewPooledTxs", Fn: s.TestNewPooledTxs},
	}
}

func (s *Suite) SnapTests() []utesting.Test {
	return []utesting.Test{
		{Name: "TestSnapStatus", Fn: s.TestSnapStatus},
		{Name: "TestSnapAccountRange", Fn: s.TestSnapGetAccountRange},
		{Name: "TestSnapGetByteCodes", Fn: s.TestSnapGetByteCodes},
		{Name: "TestSnapGetTrieNodes", Fn: s.TestSnapTrieNodes},
		{Name: "TestSnapGetStorageRanges", Fn: s.TestSnapGetStorageRanges},
	}
}

// TestStatus attempts to connect to the given node and exchange
// a status message with it on the eth protocol.
func (s *Suite) TestStatus(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
}

// TestGetBlockHeaders tests whether the given node can respond to
// an eth `GetBlockHeaders` request and that the response is accurate.
func (s *Suite) TestGetBlockHeaders(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// write request
	req := &GetBlockHeaders{
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin:  eth.HashOrNumber{Hash: s.chain.blocks[1].Hash()},
			Amount:  2,
			Skip:    1,
			Reverse: false,
		},
	}
	headers, err := conn.headersRequest(req, s.chain, 33)
	if err != nil {
		t.Fatalf("could not get block headers: %v", err)
	}
	// check for correct headers
	expected, err := s.chain.GetHeaders(req)
	if err != nil {
		t.Fatalf("failed to get headers for given request: %v", err)
	}
	if !headersMatch(expected, headers) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected, headers)
	}
}

// TestSimultaneousRequests sends two simultaneous `GetBlockHeader` requests from
// the same connection with different request IDs and checks to make sure the node
// responds with the correct headers per request.
func (s *Suite) TestSimultaneousRequests(t *utesting.T) {
	// create a connection
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}

	// create two requests
	req1 := &GetBlockHeaders{
		RequestId: uint64(111),
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin: eth.HashOrNumber{
				Hash: s.chain.blocks[1].Hash(),
			},
			Amount:  2,
			Skip:    1,
			Reverse: false,
		},
	}
	req2 := &GetBlockHeaders{
		RequestId: uint64(222),
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin: eth.HashOrNumber{
				Hash: s.chain.blocks[1].Hash(),
			},
			Amount:  4,
			Skip:    1,
			Reverse: false,
		},
	}

	// write the first request
	if err := conn.Write(req1); err != nil {
		t.Fatalf("failed to write to connection: %v", err)
	}
	// write the second request
	if err := conn.Write(req2); err != nil {
		t.Fatalf("failed to write to connection: %v", err)
	}

	// wait for responses
	msg := conn.waitForResponse(s.chain, timeout, req1.RequestId)
	headers1, ok := msg.(*BlockHeaders)
	if !ok {
		t.Fatalf("unexpected %s", pretty.Sdump(msg))
	}
	msg = conn.waitForResponse(s.chain, timeout, req2.RequestId)
	headers2, ok := msg.(*BlockHeaders)
	if !ok {
		t.Fatalf("unexpected %s", pretty.Sdump(msg))
	}

	// check received headers for accuracy
	expected1, err := s.chain.GetHeaders(req1)
	if err != nil {
		t.Fatalf("failed to get expected headers for request 1: %v", err)
	}
	expected2, err := s.chain.GetHeaders(req2)
	if err != nil {
		t.Fatalf("failed to get expected headers for request 2: %v", err)
	}
	if !headersMatch(expected1, headers1.BlockHeadersRequest) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected1, headers1)
	}
	if !headersMatch(expected2, headers2.BlockHeadersRequest) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected2, headers2)
	}
}

// TestSameRequestID sends two requests with the same request ID to a
// single node.
func (s *Suite) TestSameRequestID(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// create requests
	reqID := uint64(1234)
	request1 := &GetBlockHeaders{
		RequestId: reqID,
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin: eth.HashOrNumber{
				Number: 1,
			},
			Amount: 2,
		},
	}
	request2 := &GetBlockHeaders{
		RequestId: reqID,
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin: eth.HashOrNumber{
				Number: 33,
			},
			Amount: 2,
		},
	}

	// write the requests
	if err = conn.Write(request1); err != nil {
		t.Fatalf("failed to write to connection: %v", err)
	}
	if err = conn.Write(request2); err != nil {
		t.Fatalf("failed to write to connection: %v", err)
	}

	// wait for responses
	msg := conn.waitForResponse(s.chain, timeout, reqID)
	headers1, ok := msg.(*BlockHeaders)
	if !ok {
		t.Fatalf("unexpected %s", pretty.Sdump(msg))
	}
	msg = conn.waitForResponse(s.chain, timeout, reqID)
	headers2, ok := msg.(*BlockHeaders)
	if !ok {
		t.Fatalf("unexpected %s", pretty.Sdump(msg))
	}

	// check if headers match
	expected1, err := s.chain.GetHeaders(request1)
	if err != nil {
		t.Fatalf("failed to get expected block headers: %v", err)
	}
	expected2, err := s.chain.GetHeaders(request2)
	if err != nil {
		t.Fatalf("failed to get expected block headers: %v", err)
	}
	if !headersMatch(expected1, headers1.BlockHeadersRequest) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected1, headers1)
	}
	if !headersMatch(expected2, headers2.BlockHeadersRequest) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected2, headers2)
	}
}

// TestZeroRequestID checks that a message with a request ID of zero is still handled
// by the node.
func (s *Suite) TestZeroRequestID(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	req := &GetBlockHeaders{
		GetBlockHeadersRequest: &eth.GetBlockHeadersRequest{
			Origin: eth.HashOrNumber{Number: 0},
			Amount: 2,
		},
	}
	headers, err := conn.headersRequest(req, s.chain, 0)
	if err != nil {
		t.Fatalf("failed to get block headers: %v", err)
	}
	expected, err := s.chain.GetHeaders(req)
	if err != nil {
		t.Fatalf("failed to get expected block headers: %v", err)
	}
	if !headersMatch(expected, headers) {
		t.Fatalf("header mismatch: \nexpected %v \ngot %v", expected, headers)
	}
}

// TestGetBlockBodies tests whether the given node can respond to
// a `GetBlockBodies` request and that the response is accurate.
func (s *Suite) TestGetBlockBodies(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err := conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// create block bodies request
	req := &GetBlockBodies{
		RequestId: uint64(55),
		GetBlockBodiesRequest: eth.GetBlockBodiesRequest{
			s.chain.blocks[54].Hash(),
			s.chain.blocks[75].Hash(),
		},
	}
	if err := conn.Write(req); err != nil {
		t.Fatalf("could not write to connection: %v", err)
	}
	// wait for block bodies response
	msg := conn.waitForResponse(s.chain, timeout, req.RequestId)
	resp, ok := msg.(*BlockBodies)
	if !ok {
		t.Fatalf("unexpected: %s", pretty.Sdump(msg))
	}
	bodies := resp.BlockBodiesResponse
	t.Logf("received %d block bodies", len(bodies))
	if len(bodies) != len(req.GetBlockBodiesRequest) {
		t.Fatalf("wrong bodies in response: expected %d bodies, "+
			"got %d", len(req.GetBlockBodiesRequest), len(bodies))
	}
}

// TestBroadcast tests whether a block announcement is correctly
// propagated to the node's peers.
func (s *Suite) TestBroadcast(t *utesting.T) {
	if err := s.sendNextBlock(); err != nil {
		t.Fatalf("block broadcast failed: %v", err)
	}
}

// TestLargeAnnounce tests the announcement mechanism with a large block.
func (s *Suite) TestLargeAnnounce(t *utesting.T) {
	nextBlock := len(s.chain.blocks)
	blocks := []*NewBlock{
		{
			Block: largeBlock(),
			TD:    s.fullChain.TotalDifficultyAt(nextBlock),
		},
		{
			Block: s.fullChain.blocks[nextBlock],
			TD:    largeNumber(2),
		},
		{
			Block: largeBlock(),
			TD:    largeNumber(2),
		},
	}

	for i, blockAnnouncement := range blocks[0:3] {
		t.Logf("Testing malicious announcement: %v\n", i)
		conn, err := s.dial()
		if err != nil {
			t.Fatalf("dial failed: %v", err)
		}
		if err := conn.peer(s.chain, nil); err != nil {
			t.Fatalf("peering failed: %v", err)
		}
		if err := conn.Write(blockAnnouncement); err != nil {
			t.Fatalf("could not write to connection: %v", err)
		}
		// Invalid announcement, check that peer disconnected
		switch msg := conn.readAndServe(s.chain, 8*time.Second).(type) {
		case *Disconnect:
		case *Error:
			break
		default:
			t.Fatalf("unexpected: %s wanted disconnect", pretty.Sdump(msg))
		}
		conn.Close()
	}
	// Test the last block as a valid block
	if err := s.sendNextBlock(); err != nil {
		t.Fatalf("failed to broadcast next block: %v", err)
	}
}

// TestOldAnnounce tests the announcement mechanism with an old block.
func (s *Suite) TestOldAnnounce(t *utesting.T) {
	if err := s.oldAnnounce(); err != nil {
		t.Fatal(err)
	}
}

// TestBlockHashAnnounce sends a new block hash announcement and expects
// the node to perform a `GetBlockHeaders` request.
func (s *Suite) TestBlockHashAnnounce(t *utesting.T) {
	if err := s.hashAnnounce(); err != nil {
		t.Fatalf("block hash announcement failed: %v", err)
	}
}

// TestMaliciousHandshake tries to send malicious data during the handshake.
func (s *Suite) TestMaliciousHandshake(t *utesting.T) {
	if err := s.maliciousHandshakes(t); err != nil {
		t.Fatal(err)
	}
}

// TestMaliciousStatus sends a status package with a large total difficulty.
func (s *Suite) TestMaliciousStatus(t *utesting.T) {
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()

	if err := s.maliciousStatus(conn); err != nil {
		t.Fatal(err)
	}
}

// TestTransaction sends a valid transaction to the node and
// checks if the transaction gets propagated.
func (s *Suite) TestTransaction(t *utesting.T) {
	if err := s.sendSuccessfulTxs(t); err != nil {
		t.Fatal(err)
	}
}

// TestMaliciousTx sends several invalid transactions and tests whether
// the node will propagate them.
func (s *Suite) TestMaliciousTx(t *utesting.T) {
	if err := s.sendMaliciousTxs(t); err != nil {
		t.Fatal(err)
	}
}

// TestLargeTxRequest tests whether a node can fulfill a large GetPooledTransactions
// request.
func (s *Suite) TestLargeTxRequest(t *utesting.T) {
	// send the next block to ensure the node is no longer syncing and
	// is able to accept txs
	if err := s.sendNextBlock(); err != nil {
		t.Fatalf("failed to send next block: %v", err)
	}
	// send 2000 transactions to the node
	hashMap, txs, err := generateTxs(s, 2000)
	if err != nil {
		t.Fatalf("failed to generate transactions: %v", err)
	}
	if err = sendMultipleSuccessfulTxs(t, s, txs); err != nil {
		t.Fatalf("failed to send multiple txs: %v", err)
	}
	// set up connection to receive to ensure node is peered with the receiving connection
	// before tx request is sent
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}
	// create and send pooled tx request
	hashes := make([]common.Hash, 0)
	for _, hash := range hashMap {
		hashes = append(hashes, hash)
	}
	getTxReq := &GetPooledTransactions{
		RequestId:                    1234,
		GetPooledTransactionsRequest: hashes,
	}
	if err = conn.Write(getTxReq); err != nil {
		t.Fatalf("could not write to conn: %v", err)
	}
	// check that all received transactions match those that were sent to node
	switch msg := conn.waitForResponse(s.chain, timeout, getTxReq.RequestId).(type) {
	case *PooledTransactions:
		for _, gotTx := range msg.PooledTransactionsResponse {
			if _, exists := hashMap[gotTx.Hash()]; !exists {
				t.Fatalf("unexpected tx received: %v", gotTx.Hash())
			}
		}
	default:
		t.Fatalf("unexpected %s", pretty.Sdump(msg))
	}
}

// TestNewPooledTxs tests whether a node will do a GetPooledTransactions
// request upon receiving a NewPooledTransactionHashes announcement.
func (s *Suite) TestNewPooledTxs(t *utesting.T) {
	// send the next block to ensure the node is no longer syncing and
	// is able to accept txs
	if err := s.sendNextBlock(); err != nil {
		t.Fatalf("failed to send next block: %v", err)
	}

	// generate 50 txs
	_, txs, err := generateTxs(s, 50)
	if err != nil {
		t.Fatalf("failed to generate transactions: %v", err)
	}
	hashes := make([]common.Hash, len(txs))
	types := make([]byte, len(txs))
	sizes := make([]uint32, len(txs))
	for i, tx := range txs {
		hashes[i] = tx.Hash()
		types[i] = tx.Type()
		sizes[i] = uint32(tx.Size())
	}

	// send announcement
	conn, err := s.dial()
	if err != nil {
		t.Fatalf("dial failed: %v", err)
	}
	defer conn.Close()
	if err = conn.peer(s.chain, nil); err != nil {
		t.Fatalf("peering failed: %v", err)
	}

	var ann Message = NewPooledTransactionHashes{Types: types, Sizes: sizes, Hashes: hashes}
	if conn.negotiatedProtoVersion < eth.ETH68 {
		ann = NewPooledTransactionHashes66(hashes)
	}
	err = conn.Write(ann)
	if err != nil {
		t.Fatalf("failed to write to connection: %v", err)
	}

	// wait for GetPooledTxs request
	for {
		msg := conn.readAndServe(s.chain, timeout)
		switch msg := msg.(type) {
		case *GetPooledTransactions:
			if len(msg.GetPooledTransactionsRequest) != len(hashes) {
				t.Fatalf("unexpected number of txs requested: wanted %d, got %d", len(hashes), len(msg.GetPooledTransactionsRequest))
			}
			return

		// ignore propagated txs from previous tests
		case *NewPooledTransactionHashes66:
			continue
		case *NewPooledTransactionHashes:
			continue
		case *Transactions:
			continue

		// ignore block announcements from previous tests
		case *NewBlockHashes:
			continue
		case *NewBlock:
			continue
		default:
			t.Fatalf("unexpected %s", pretty.Sdump(msg))
		}
	}
}
