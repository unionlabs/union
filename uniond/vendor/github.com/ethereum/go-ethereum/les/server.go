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
	"crypto/ecdsa"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/eth/ethconfig"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/flowcontrol"
	vfs "github.com/ethereum/go-ethereum/les/vflux/server"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/params"
	"github.com/ethereum/go-ethereum/rpc"
)

var (
	defaultPosFactors = vfs.PriceFactors{TimeFactor: 0, CapacityFactor: 1, RequestFactor: 1}
	defaultNegFactors = vfs.PriceFactors{TimeFactor: 0, CapacityFactor: 1, RequestFactor: 1}
)

const defaultConnectedBias = time.Minute * 3

type ethBackend interface {
	ArchiveMode() bool
	BlockChain() *core.BlockChain
	BloomIndexer() *core.ChainIndexer
	ChainDb() ethdb.Database
	Synced() bool
	TxPool() *txpool.TxPool
}

type LesServer struct {
	lesCommons

	archiveMode bool // Flag whether the ethereum node runs in archive mode.
	handler     *serverHandler
	peers       *clientPeerSet
	serverset   *serverSet
	vfluxServer *vfs.Server
	privateKey  *ecdsa.PrivateKey

	// Flow control and capacity management
	fcManager    *flowcontrol.ClientManager
	costTracker  *costTracker
	defParams    flowcontrol.ServerParams
	servingQueue *servingQueue
	clientPool   *vfs.ClientPool

	minCapacity, maxCapacity uint64
	threadsIdle              int // Request serving threads count when system is idle.
	threadsBusy              int // Request serving threads count when system is busy(block insertion).

	p2pSrv *p2p.Server
}

func NewLesServer(node *node.Node, e ethBackend, config *ethconfig.Config) (*LesServer, error) {
	lesDb, err := node.OpenDatabase("les.server", 0, 0, "eth/db/lesserver/", false)
	if err != nil {
		return nil, err
	}
	// Calculate the number of threads used to service the light client
	// requests based on the user-specified value.
	threads := config.LightServ * 4 / 100
	if threads < 4 {
		threads = 4
	}
	srv := &LesServer{
		lesCommons: lesCommons{
			genesis:          e.BlockChain().Genesis().Hash(),
			config:           config,
			chainConfig:      e.BlockChain().Config(),
			iConfig:          light.DefaultServerIndexerConfig,
			chainDb:          e.ChainDb(),
			lesDb:            lesDb,
			chainReader:      e.BlockChain(),
			chtIndexer:       light.NewChtIndexer(e.ChainDb(), nil, params.CHTFrequency, params.HelperTrieProcessConfirmations, true),
			bloomTrieIndexer: light.NewBloomTrieIndexer(e.ChainDb(), nil, params.BloomBitsBlocks, params.BloomTrieFrequency, true),
			closeCh:          make(chan struct{}),
		},
		archiveMode:  e.ArchiveMode(),
		peers:        newClientPeerSet(),
		serverset:    newServerSet(),
		vfluxServer:  vfs.NewServer(time.Millisecond * 10),
		fcManager:    flowcontrol.NewClientManager(nil, &mclock.System{}),
		servingQueue: newServingQueue(int64(time.Millisecond*10), float64(config.LightServ)/100),
		threadsBusy:  config.LightServ/100 + 1,
		threadsIdle:  threads,
		p2pSrv:       node.Server(),
	}
	issync := e.Synced
	if config.LightNoSyncServe {
		issync = func() bool { return true }
	}
	srv.handler = newServerHandler(srv, e.BlockChain(), e.ChainDb(), e.TxPool(), issync)
	srv.costTracker, srv.minCapacity = newCostTracker(e.ChainDb(), config)

	// Initialize the bloom trie indexer.
	e.BloomIndexer().AddChildIndexer(srv.bloomTrieIndexer)

	// Initialize server capacity management fields.
	srv.defParams = flowcontrol.ServerParams{
		BufLimit:    srv.minCapacity * bufLimitRatio,
		MinRecharge: srv.minCapacity,
	}
	// LES flow control tries to more or less guarantee the possibility for the
	// clients to send a certain amount of requests at any time and get a quick
	// response. Most of the clients want this guarantee but don't actually need
	// to send requests most of the time. Our goal is to serve as many clients as
	// possible while the actually used server capacity does not exceed the limits
	totalRecharge := srv.costTracker.totalRecharge()
	srv.maxCapacity = srv.minCapacity * uint64(srv.config.LightPeers)
	if totalRecharge > srv.maxCapacity {
		srv.maxCapacity = totalRecharge
	}
	srv.fcManager.SetCapacityLimits(srv.minCapacity, srv.maxCapacity, srv.minCapacity*2)
	srv.clientPool = vfs.NewClientPool(lesDb, srv.minCapacity, defaultConnectedBias, mclock.System{}, issync)
	srv.clientPool.Start()
	srv.clientPool.SetDefaultFactors(defaultPosFactors, defaultNegFactors)
	srv.vfluxServer.Register(srv.clientPool, "les", "Ethereum light client service")
	srv.chtIndexer.Start(e.BlockChain())

	node.RegisterProtocols(srv.Protocols())
	node.RegisterAPIs(srv.APIs())
	node.RegisterLifecycle(srv)
	return srv, nil
}

func (s *LesServer) APIs() []rpc.API {
	return []rpc.API{
		{
			Namespace: "les",
			Service:   NewLightServerAPI(s),
		},
		{
			Namespace: "debug",
			Service:   NewDebugAPI(s),
		},
	}
}

func (s *LesServer) Protocols() []p2p.Protocol {
	ps := s.makeProtocols(ServerProtocolVersions, s.handler.runPeer, func(id enode.ID) interface{} {
		if p := s.peers.peer(id); p != nil {
			return p.Info()
		}
		return nil
	}, nil)
	// Add "les" ENR entries.
	for i := range ps {
		ps[i].Attributes = []enr.Entry{&lesEntry{
			VfxVersion: 1,
		}}
	}
	return ps
}

// Start starts the LES server
func (s *LesServer) Start() error {
	s.privateKey = s.p2pSrv.PrivateKey
	s.peers.setSignerKey(s.privateKey)
	s.handler.start()
	s.wg.Add(1)
	go s.capacityManagement()
	if s.p2pSrv.DiscV5 != nil {
		s.p2pSrv.DiscV5.RegisterTalkHandler("vfx", s.vfluxServer.ServeEncoded)
	}
	return nil
}

// Stop stops the LES service
func (s *LesServer) Stop() error {
	close(s.closeCh)

	s.clientPool.Stop()
	if s.serverset != nil {
		s.serverset.close()
	}
	s.peers.close()
	s.fcManager.Stop()
	s.costTracker.stop()
	s.handler.stop()
	s.servingQueue.stop()
	if s.vfluxServer != nil {
		s.vfluxServer.Stop()
	}

	// Note, bloom trie indexer is closed by parent bloombits indexer.
	if s.chtIndexer != nil {
		s.chtIndexer.Close()
	}
	if s.lesDb != nil {
		s.lesDb.Close()
	}
	s.wg.Wait()
	log.Info("Les server stopped")

	return nil
}

// capacityManagement starts an event handler loop that updates the recharge curve of
// the client manager and adjusts the client pool's size according to the total
// capacity updates coming from the client manager
func (s *LesServer) capacityManagement() {
	defer s.wg.Done()

	processCh := make(chan bool, 100)
	sub := s.handler.blockchain.SubscribeBlockProcessingEvent(processCh)
	defer sub.Unsubscribe()

	totalRechargeCh := make(chan uint64, 100)
	totalRecharge := s.costTracker.subscribeTotalRecharge(totalRechargeCh)

	totalCapacityCh := make(chan uint64, 100)
	totalCapacity := s.fcManager.SubscribeTotalCapacity(totalCapacityCh)
	s.clientPool.SetLimits(uint64(s.config.LightPeers), totalCapacity)

	var (
		busy         bool
		freePeers    uint64
		blockProcess mclock.AbsTime
	)
	updateRecharge := func() {
		if busy {
			s.servingQueue.setThreads(s.threadsBusy)
			s.fcManager.SetRechargeCurve(flowcontrol.PieceWiseLinear{{0, 0}, {totalRecharge, totalRecharge}})
		} else {
			s.servingQueue.setThreads(s.threadsIdle)
			s.fcManager.SetRechargeCurve(flowcontrol.PieceWiseLinear{{0, 0}, {totalRecharge / 10, totalRecharge}, {totalRecharge, totalRecharge}})
		}
	}
	updateRecharge()

	for {
		select {
		case busy = <-processCh:
			if busy {
				blockProcess = mclock.Now()
			} else {
				blockProcessingTimer.Update(time.Duration(mclock.Now() - blockProcess))
			}
			updateRecharge()
		case totalRecharge = <-totalRechargeCh:
			totalRechargeGauge.Update(int64(totalRecharge))
			updateRecharge()
		case totalCapacity = <-totalCapacityCh:
			totalCapacityGauge.Update(int64(totalCapacity))
			newFreePeers := totalCapacity / s.minCapacity
			if newFreePeers < freePeers && newFreePeers < uint64(s.config.LightPeers) {
				log.Warn("Reduced free peer connections", "from", freePeers, "to", newFreePeers)
			}
			freePeers = newFreePeers
			s.clientPool.SetLimits(uint64(s.config.LightPeers), totalCapacity)
		case <-s.closeCh:
			return
		}
	}
}
