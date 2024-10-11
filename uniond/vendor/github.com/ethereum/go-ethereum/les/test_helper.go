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

// This file contains some shares testing functionality, common to multiple
// different files and modules being tested. Client based network and Server
// based network can be created easily with available APIs.

package les

import (
	"context"
	"crypto/rand"
	"fmt"
	"math/big"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/accounts/abi/bind/backends"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/consensus"
	"github.com/ethereum/go-ethereum/consensus/ethash"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/txpool/legacypool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/eth/ethconfig"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/les/flowcontrol"
	vfs "github.com/ethereum/go-ethereum/les/vflux/server"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/trie"
)

var (
	bankKey, _ = crypto.GenerateKey()
	bankAddr   = crypto.PubkeyToAddress(bankKey.PublicKey)
	bankFunds  = big.NewInt(1_000_000_000_000_000_000)

	userKey1, _ = crypto.GenerateKey()
	userKey2, _ = crypto.GenerateKey()
	userAddr1   = crypto.PubkeyToAddress(userKey1.PublicKey)
	userAddr2   = crypto.PubkeyToAddress(userKey2.PublicKey)

	testContractAddr         common.Address
	testContractCode         = common.Hex2Bytes("606060405260cc8060106000396000f360606040526000357c01000000000000000000000000000000000000000000000000000000009004806360cd2685146041578063c16431b914606b57603f565b005b6055600480803590602001909190505060a9565b6040518082815260200191505060405180910390f35b60886004808035906020019091908035906020019091905050608a565b005b80600060005083606481101560025790900160005b50819055505b5050565b6000600060005082606481101560025790900160005b5054905060c7565b91905056")
	testContractCodeDeployed = testContractCode[16:]
	testContractDeployed     = uint64(2)

	testEventEmitterCode = common.Hex2Bytes("60606040523415600e57600080fd5b7f57050ab73f6b9ebdd9f76b8d4997793f48cf956e965ee070551b9ca0bb71584e60405160405180910390a160358060476000396000f3006060604052600080fd00a165627a7a723058203f727efcad8b5811f8cb1fc2620ce5e8c63570d697aef968172de296ea3994140029")

	// Checkpoint oracle relative fields
	signerKey, _ = crypto.GenerateKey()
	signerAddr   = crypto.PubkeyToAddress(signerKey.PublicKey)
)

var (
	// The token bucket buffer limit for testing purpose.
	testBufLimit = uint64(1000000)

	// The buffer recharging speed for testing purpose.
	testBufRecharge = uint64(1000)
)

/*
contract test {

    uint256[100] data;

    function Put(uint256 addr, uint256 value) {
        data[addr] = value;
    }

    function Get(uint256 addr) constant returns (uint256 value) {
        return data[addr];
    }
}
*/

// prepare pre-commits specified number customized blocks into chain.
func prepare(n int, backend *backends.SimulatedBackend) {
	var (
		ctx    = context.Background()
		signer = types.HomesteadSigner{}
	)
	for i := 0; i < n; i++ {
		switch i {
		case 0:
			// Builtin-block
			//    number: 1
			//    txs:    1

			// bankUser transfers some ether to user1
			nonce, _ := backend.PendingNonceAt(ctx, bankAddr)
			tx, _ := types.SignTx(types.NewTransaction(nonce, userAddr1, big.NewInt(10_000_000_000_000_000), params.TxGas, big.NewInt(params.InitialBaseFee), nil), signer, bankKey)
			backend.SendTransaction(ctx, tx)
		case 1:
			// Builtin-block
			//    number: 2
			//    txs:    4

			bankNonce, _ := backend.PendingNonceAt(ctx, bankAddr)
			userNonce1, _ := backend.PendingNonceAt(ctx, userAddr1)

			// bankUser transfers more ether to user1
			tx1, _ := types.SignTx(types.NewTransaction(bankNonce, userAddr1, big.NewInt(1_000_000_000_000_000), params.TxGas, big.NewInt(params.InitialBaseFee), nil), signer, bankKey)
			backend.SendTransaction(ctx, tx1)

			// user1 relays ether to user2
			tx2, _ := types.SignTx(types.NewTransaction(userNonce1, userAddr2, big.NewInt(1_000_000_000_000_000), params.TxGas, big.NewInt(params.InitialBaseFee), nil), signer, userKey1)
			backend.SendTransaction(ctx, tx2)

			// user1 deploys a test contract
			tx3, _ := types.SignTx(types.NewContractCreation(userNonce1+1, big.NewInt(0), 200000, big.NewInt(params.InitialBaseFee), testContractCode), signer, userKey1)
			backend.SendTransaction(ctx, tx3)
			testContractAddr = crypto.CreateAddress(userAddr1, userNonce1+1)

			// user1 deploys a event contract
			tx4, _ := types.SignTx(types.NewContractCreation(userNonce1+2, big.NewInt(0), 200000, big.NewInt(params.InitialBaseFee), testEventEmitterCode), signer, userKey1)
			backend.SendTransaction(ctx, tx4)
		case 2:
			// Builtin-block
			//    number: 3
			//    txs:    2

			// bankUser transfer some ether to signer
			bankNonce, _ := backend.PendingNonceAt(ctx, bankAddr)
			tx1, _ := types.SignTx(types.NewTransaction(bankNonce, signerAddr, big.NewInt(1000000000), params.TxGas, big.NewInt(params.InitialBaseFee), nil), signer, bankKey)
			backend.SendTransaction(ctx, tx1)

			// invoke test contract
			data := common.Hex2Bytes("C16431B900000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001")
			tx2, _ := types.SignTx(types.NewTransaction(bankNonce+1, testContractAddr, big.NewInt(0), 100000, big.NewInt(params.InitialBaseFee), data), signer, bankKey)
			backend.SendTransaction(ctx, tx2)
		case 3:
			// Builtin-block
			//    number: 4
			//    txs:    1

			// invoke test contract
			bankNonce, _ := backend.PendingNonceAt(ctx, bankAddr)
			data := common.Hex2Bytes("C16431B900000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002")
			tx, _ := types.SignTx(types.NewTransaction(bankNonce, testContractAddr, big.NewInt(0), 100000, big.NewInt(params.InitialBaseFee), data), signer, bankKey)
			backend.SendTransaction(ctx, tx)
		}
		backend.Commit()
	}
}

// testIndexers creates a set of indexers with specified params for testing purpose.
func testIndexers(db ethdb.Database, odr light.OdrBackend, config *light.IndexerConfig, disablePruning bool) []*core.ChainIndexer {
	var indexers [3]*core.ChainIndexer
	indexers[0] = light.NewChtIndexer(db, odr, config.ChtSize, config.ChtConfirms, disablePruning)
	indexers[1] = core.NewBloomIndexer(db, config.BloomSize, config.BloomConfirms)
	indexers[2] = light.NewBloomTrieIndexer(db, odr, config.BloomSize, config.BloomTrieSize, disablePruning)
	// make bloomTrieIndexer as a child indexer of bloom indexer.
	indexers[1].AddChildIndexer(indexers[2])
	return indexers[:]
}

func newTestClientHandler(backend *backends.SimulatedBackend, odr *LesOdr, indexers []*core.ChainIndexer, db ethdb.Database, peers *serverPeerSet) (*clientHandler, func()) {
	var (
		evmux  = new(event.TypeMux)
		engine = ethash.NewFaker()
		gspec  = core.Genesis{
			Config:   params.AllEthashProtocolChanges,
			Alloc:    core.GenesisAlloc{bankAddr: {Balance: bankFunds}},
			GasLimit: 100000000,
			BaseFee:  big.NewInt(params.InitialBaseFee),
		}
	)
	genesis := gspec.MustCommit(db, trie.NewDatabase(db, trie.HashDefaults))
	chain, _ := light.NewLightChain(odr, gspec.Config, engine)

	client := &LightEthereum{
		lesCommons: lesCommons{
			genesis:     genesis.Hash(),
			config:      &ethconfig.Config{LightPeers: 100, NetworkId: NetworkId},
			chainConfig: params.AllEthashProtocolChanges,
			iConfig:     light.TestClientIndexerConfig,
			chainDb:     db,
			chainReader: chain,
			closeCh:     make(chan struct{}),
		},
		peers:      peers,
		reqDist:    odr.retriever.dist,
		retriever:  odr.retriever,
		odr:        odr,
		engine:     engine,
		blockchain: chain,
		eventMux:   evmux,
		merger:     consensus.NewMerger(rawdb.NewMemoryDatabase()),
	}
	client.handler = newClientHandler(client)

	return client.handler, func() {
		client.handler.stop()
	}
}

func newTestServerHandler(blocks int, indexers []*core.ChainIndexer, db ethdb.Database, clock mclock.Clock) (*serverHandler, *backends.SimulatedBackend, func()) {
	var (
		gspec = core.Genesis{
			Config:   params.AllEthashProtocolChanges,
			Alloc:    core.GenesisAlloc{bankAddr: {Balance: bankFunds}},
			GasLimit: 100000000,
			BaseFee:  big.NewInt(params.InitialBaseFee),
		}
	)
	genesis := gspec.MustCommit(db, trie.NewDatabase(db, trie.HashDefaults))

	// create a simulation backend and pre-commit several customized block to the database.
	simulation := backends.NewSimulatedBackendWithDatabase(db, gspec.Alloc, 100000000)
	prepare(blocks, simulation)

	txpoolConfig := legacypool.DefaultConfig
	txpoolConfig.Journal = ""

	pool := legacypool.New(txpoolConfig, simulation.Blockchain())
	txpool, _ := txpool.New(new(big.Int).SetUint64(txpoolConfig.PriceLimit), simulation.Blockchain(), []txpool.SubPool{pool})

	server := &LesServer{
		lesCommons: lesCommons{
			genesis:     genesis.Hash(),
			config:      &ethconfig.Config{LightPeers: 100, NetworkId: NetworkId},
			chainConfig: params.AllEthashProtocolChanges,
			iConfig:     light.TestServerIndexerConfig,
			chainDb:     db,
			chainReader: simulation.Blockchain(),
			closeCh:     make(chan struct{}),
		},
		peers:        newClientPeerSet(),
		servingQueue: newServingQueue(int64(time.Millisecond*10), 1),
		defParams: flowcontrol.ServerParams{
			BufLimit:    testBufLimit,
			MinRecharge: testBufRecharge,
		},
		fcManager: flowcontrol.NewClientManager(nil, clock),
	}
	server.costTracker, server.minCapacity = newCostTracker(db, server.config)
	server.costTracker.testCostList = testCostList(0) // Disable flow control mechanism.
	server.clientPool = vfs.NewClientPool(db, testBufRecharge, defaultConnectedBias, clock, alwaysTrueFn)
	server.clientPool.Start()
	server.clientPool.SetLimits(10000, 10000) // Assign enough capacity for clientpool
	server.handler = newServerHandler(server, simulation.Blockchain(), db, txpool, func() bool { return true })
	server.servingQueue.setThreads(4)
	server.handler.start()
	closer := func() { server.Stop() }
	return server.handler, simulation, closer
}

func alwaysTrueFn() bool {
	return true
}

// testPeer is a simulated peer to allow testing direct network calls.
type testPeer struct {
	cpeer *clientPeer
	speer *serverPeer

	net p2p.MsgReadWriter // Network layer reader/writer to simulate remote messaging
	app *p2p.MsgPipeRW    // Application layer reader/writer to simulate the local side
}

// handshakeWithServer executes the handshake with the remote server peer.
func (p *testPeer) handshakeWithServer(t *testing.T, td *big.Int, head common.Hash, headNum uint64, genesis common.Hash, forkID forkid.ID) {
	// It only works for the simulated client peer
	if p.cpeer == nil {
		t.Fatal("handshake for client peer only")
	}
	var sendList keyValueList
	sendList = sendList.add("protocolVersion", uint64(p.cpeer.version))
	sendList = sendList.add("networkId", uint64(NetworkId))
	sendList = sendList.add("headTd", td)
	sendList = sendList.add("headHash", head)
	sendList = sendList.add("headNum", headNum)
	sendList = sendList.add("genesisHash", genesis)
	if p.cpeer.version >= lpv4 {
		sendList = sendList.add("forkID", &forkID)
	}
	if err := p2p.ExpectMsg(p.app, StatusMsg, nil); err != nil {
		t.Fatalf("status recv: %v", err)
	}
	if err := p2p.Send(p.app, StatusMsg, &sendList); err != nil {
		t.Fatalf("status send: %v", err)
	}
}

// handshakeWithClient executes the handshake with the remote client peer.
// (used by temporarily disabled tests)
/*func (p *testPeer) handshakeWithClient(t *testing.T, td *big.Int, head common.Hash, headNum uint64, genesis common.Hash, forkID forkid.ID, costList RequestCostList, recentTxLookup uint64) {
	// It only works for the simulated client peer
	if p.speer == nil {
		t.Fatal("handshake for server peer only")
	}
	var sendList keyValueList
	sendList = sendList.add("protocolVersion", uint64(p.speer.version))
	sendList = sendList.add("networkId", uint64(NetworkId))
	sendList = sendList.add("headTd", td)
	sendList = sendList.add("headHash", head)
	sendList = sendList.add("headNum", headNum)
	sendList = sendList.add("genesisHash", genesis)
	sendList = sendList.add("serveHeaders", nil)
	sendList = sendList.add("serveChainSince", uint64(0))
	sendList = sendList.add("serveStateSince", uint64(0))
	sendList = sendList.add("serveRecentState", uint64(core.TriesInMemory-4))
	sendList = sendList.add("txRelay", nil)
	sendList = sendList.add("flowControl/BL", testBufLimit)
	sendList = sendList.add("flowControl/MRR", testBufRecharge)
	sendList = sendList.add("flowControl/MRC", costList)
	if p.speer.version >= lpv4 {
		sendList = sendList.add("forkID", &forkID)
		sendList = sendList.add("recentTxLookup", recentTxLookup)
	}
	if err := p2p.ExpectMsg(p.app, StatusMsg, nil); err != nil {
		t.Fatalf("status recv: %v", err)
	}
	if err := p2p.Send(p.app, StatusMsg, &sendList); err != nil {
		t.Fatalf("status send: %v", err)
	}
}*/

// close terminates the local side of the peer, notifying the remote protocol
// manager of termination.
func (p *testPeer) close() {
	p.app.Close()
}

func newTestPeerPair(name string, version int, server *serverHandler, client *clientHandler, noInitAnnounce bool) (*testPeer, *testPeer, error) {
	// Create a message pipe to communicate through
	app, net := p2p.MsgPipe()

	// Generate a random id and create the peer
	var id enode.ID
	rand.Read(id[:])

	peer1 := newClientPeer(version, NetworkId, p2p.NewPeer(id, name, nil), net)
	peer2 := newServerPeer(version, NetworkId, false, p2p.NewPeer(id, name, nil), app)

	// Start the peer on a new thread
	errc1 := make(chan error, 1)
	errc2 := make(chan error, 1)
	go func() {
		select {
		case <-server.closeCh:
			errc1 <- p2p.DiscQuitting
		case errc1 <- server.handle(peer1):
		}
	}()
	go func() {
		select {
		case <-client.closeCh:
			errc2 <- p2p.DiscQuitting
		case errc2 <- client.handle(peer2, noInitAnnounce):
		}
	}()
	// Ensure the connection is established or exits when any error occurs
	for {
		select {
		case err := <-errc1:
			return nil, nil, fmt.Errorf("failed to establish protocol connection %v", err)
		case err := <-errc2:
			return nil, nil, fmt.Errorf("failed to establish protocol connection %v", err)
		default:
		}
		if peer1.serving.Load() && peer2.serving.Load() {
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	return &testPeer{cpeer: peer1, net: net, app: app}, &testPeer{speer: peer2, net: app, app: net}, nil
}

type indexerCallback func(*core.ChainIndexer, *core.ChainIndexer, *core.ChainIndexer)

// testClient represents a client object for testing with necessary auxiliary fields.
type testClient struct {
	clock   mclock.Clock
	db      ethdb.Database
	peer    *testPeer
	handler *clientHandler

	chtIndexer       *core.ChainIndexer
	bloomIndexer     *core.ChainIndexer
	bloomTrieIndexer *core.ChainIndexer
}

// newRawPeer creates a new server peer connects to the server and do the handshake.
// (used by temporarily disabled tests)
/*func (client *testClient) newRawPeer(t *testing.T, name string, version int, recentTxLookup uint64) (*testPeer, func(), <-chan error) {
	// Create a message pipe to communicate through
	app, net := p2p.MsgPipe()

	// Generate a random id and create the peer
	var id enode.ID
	rand.Read(id[:])
	peer := newServerPeer(version, NetworkId, false, p2p.NewPeer(id, name, nil), net)

	// Start the peer on a new thread
	errCh := make(chan error, 1)
	go func() {
		select {
		case <-client.handler.closeCh:
			errCh <- p2p.DiscQuitting
		case errCh <- client.handler.handle(peer, false):
		}
	}()
	tp := &testPeer{
		app:   app,
		net:   net,
		speer: peer,
	}
	var (
		genesis = client.handler.backend.blockchain.Genesis()
		head    = client.handler.backend.blockchain.CurrentHeader()
		td      = client.handler.backend.blockchain.GetTd(head.Hash(), head.Number.Uint64())
	)
	forkID := forkid.NewID(client.handler.backend.blockchain.Config(), genesis.Hash(), head.Number.Uint64(), head.Time)
	tp.handshakeWithClient(t, td, head.Hash(), head.Number.Uint64(), genesis.Hash(), forkID, testCostList(0), recentTxLookup) // disable flow control by default

	// Ensure the connection is established or exits when any error occurs
	for {
		select {
		case <-errCh:
			return nil, nil, nil
		default:
		}
		if peer.serving.Load() {
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	closePeer := func() {
		tp.speer.close()
		tp.close()
	}
	return tp, closePeer, errCh
}*/

// testServer represents a server object for testing with necessary auxiliary fields.
type testServer struct {
	clock   mclock.Clock
	backend *backends.SimulatedBackend
	db      ethdb.Database
	peer    *testPeer
	handler *serverHandler

	chtIndexer       *core.ChainIndexer
	bloomIndexer     *core.ChainIndexer
	bloomTrieIndexer *core.ChainIndexer
}

// newRawPeer creates a new client peer connects to the server and do the handshake.
func (server *testServer) newRawPeer(t *testing.T, name string, version int) (*testPeer, func(), <-chan error) {
	// Create a message pipe to communicate through
	app, net := p2p.MsgPipe()

	// Generate a random id and create the peer
	var id enode.ID
	rand.Read(id[:])
	peer := newClientPeer(version, NetworkId, p2p.NewPeer(id, name, nil), net)

	// Start the peer on a new thread
	errCh := make(chan error, 1)
	go func() {
		select {
		case <-server.handler.closeCh:
			errCh <- p2p.DiscQuitting
		case errCh <- server.handler.handle(peer):
		}
	}()
	tp := &testPeer{
		app:   app,
		net:   net,
		cpeer: peer,
	}
	var (
		genesis = server.handler.blockchain.Genesis()
		head    = server.handler.blockchain.CurrentHeader()
		td      = server.handler.blockchain.GetTd(head.Hash(), head.Number.Uint64())
	)
	forkID := forkid.NewID(server.handler.blockchain.Config(), genesis, head.Number.Uint64(), head.Time)
	tp.handshakeWithServer(t, td, head.Hash(), head.Number.Uint64(), genesis.Hash(), forkID)

	// Ensure the connection is established or exits when any error occurs
	for {
		select {
		case <-errCh:
			return nil, nil, nil
		default:
		}
		if peer.serving.Load() {
			break
		}
		time.Sleep(50 * time.Millisecond)
	}
	closePeer := func() {
		tp.cpeer.close()
		tp.close()
	}
	return tp, closePeer, errCh
}

// testnetConfig wraps all the configurations for testing network.
type testnetConfig struct {
	blocks    int
	protocol  int
	indexFn   indexerCallback
	simClock  bool
	connect   bool
	nopruning bool
}

func newClientServerEnv(t *testing.T, config testnetConfig) (*testServer, *testClient, func()) {
	var (
		sdb    = rawdb.NewMemoryDatabase()
		cdb    = rawdb.NewMemoryDatabase()
		speers = newServerPeerSet()
	)
	var clock mclock.Clock = &mclock.System{}
	if config.simClock {
		clock = &mclock.Simulated{}
	}
	dist := newRequestDistributor(speers, clock)
	rm := newRetrieveManager(speers, dist, func() time.Duration { return time.Millisecond * 500 })
	odr := NewLesOdr(cdb, light.TestClientIndexerConfig, speers, rm)

	sindexers := testIndexers(sdb, nil, light.TestServerIndexerConfig, true)
	cIndexers := testIndexers(cdb, odr, light.TestClientIndexerConfig, config.nopruning)

	scIndexer, sbIndexer, sbtIndexer := sindexers[0], sindexers[1], sindexers[2]
	ccIndexer, cbIndexer, cbtIndexer := cIndexers[0], cIndexers[1], cIndexers[2]
	odr.SetIndexers(ccIndexer, cbIndexer, cbtIndexer)

	server, b, serverClose := newTestServerHandler(config.blocks, sindexers, sdb, clock)
	client, clientClose := newTestClientHandler(b, odr, cIndexers, cdb, speers)

	scIndexer.Start(server.blockchain)
	sbIndexer.Start(server.blockchain)
	ccIndexer.Start(client.backend.blockchain)
	cbIndexer.Start(client.backend.blockchain)

	if config.indexFn != nil {
		config.indexFn(scIndexer, sbIndexer, sbtIndexer)
	}
	var (
		err          error
		speer, cpeer *testPeer
	)
	if config.connect {
		done := make(chan struct{})
		cpeer, speer, err = newTestPeerPair("peer", config.protocol, server, client, false)
		if err != nil {
			t.Fatalf("Failed to connect testing peers %v", err)
		}
		select {
		case <-done:
		case <-time.After(10 * time.Second):
			t.Fatal("test peer did not connect and sync within 3s")
		}
	}
	s := &testServer{
		clock:            clock,
		backend:          b,
		db:               sdb,
		peer:             cpeer,
		handler:          server,
		chtIndexer:       scIndexer,
		bloomIndexer:     sbIndexer,
		bloomTrieIndexer: sbtIndexer,
	}
	c := &testClient{
		clock:            clock,
		db:               cdb,
		peer:             speer,
		handler:          client,
		chtIndexer:       ccIndexer,
		bloomIndexer:     cbIndexer,
		bloomTrieIndexer: cbtIndexer,
	}
	teardown := func() {
		if config.connect {
			speer.close()
			cpeer.close()
			cpeer.cpeer.close()
			speer.speer.close()
		}
		ccIndexer.Close()
		cbIndexer.Close()
		scIndexer.Close()
		sbIndexer.Close()
		dist.close()
		serverClose()
		b.Close()
		clientClose()
	}
	return s, c, teardown
}

// NewFuzzerPeer creates a client peer for test purposes, and also returns
// a function to close the peer: this is needed to avoid goroutine leaks in the
// exec queue.
func NewFuzzerPeer(version int) (p *clientPeer, closer func()) {
	p = newClientPeer(version, 0, p2p.NewPeer(enode.ID{}, "", nil), nil)
	return p, func() { p.peerCommons.close() }
}
