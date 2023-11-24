package node

import (
	"bytes"
	"context"
	"fmt"
	"net"
	"net/http"
	"os"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/rs/cors"

	bc "github.com/cometbft/cometbft/blocksync"
	cfg "github.com/cometbft/cometbft/config"
	cs "github.com/cometbft/cometbft/consensus"
	"github.com/cometbft/cometbft/evidence"
	"github.com/cometbft/cometbft/light"

	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/libs/service"
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/p2p/pex"
	"github.com/cometbft/cometbft/proxy"
	rpccore "github.com/cometbft/cometbft/rpc/core"
	grpccore "github.com/cometbft/cometbft/rpc/grpc"
	rpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/state/txindex"
	"github.com/cometbft/cometbft/state/txindex/null"
	"github.com/cometbft/cometbft/statesync"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	"github.com/cometbft/cometbft/version"

	_ "net/http/pprof" //nolint: gosec
)

// Node is the highest level interface to a full CometBFT node.
// It includes all configuration information and running services.
type Node struct {
	service.BaseService

	// config
	config        *cfg.Config
	genesisDoc    *types.GenesisDoc   // initial validator set
	privValidator types.PrivValidator // local node's validator key

	// network
	transport   *p2p.MultiplexTransport
	sw          *p2p.Switch  // p2p connections
	addrBook    pex.AddrBook // known peers
	nodeInfo    p2p.NodeInfo
	nodeKey     *p2p.NodeKey // our node privkey
	isListening bool

	// services
	eventBus          *types.EventBus // pub/sub for services
	stateStore        sm.Store
	blockStore        *store.BlockStore // store the blockchain to disk
	bcReactor         p2p.Reactor       // for block-syncing
	mempoolReactor    p2p.Reactor       // for gossipping transactions
	mempool           mempl.Mempool
	stateSync         bool                    // whether the node should state sync on startup
	stateSyncReactor  *statesync.Reactor      // for hosting and restoring state sync snapshots
	stateSyncProvider statesync.StateProvider // provides state data for bootstrapping a node
	stateSyncGenesis  sm.State                // provides the genesis state for state sync
	consensusState    *cs.State               // latest consensus state
	consensusReactor  *cs.Reactor             // for participating in the consensus
	pexReactor        *pex.Reactor            // for exchanging peer addresses
	evidencePool      *evidence.Pool          // tracking evidence
	proxyApp          proxy.AppConns          // connection to the application
	rpcListeners      []net.Listener          // rpc servers
	txIndexer         txindex.TxIndexer
	blockIndexer      indexer.BlockIndexer
	indexerService    *txindex.IndexerService
	prometheusSrv     *http.Server
	pprofSrv          *http.Server
}

// Option sets a parameter for the node.
type Option func(*Node)

// CustomReactors allows you to add custom reactors (name -> p2p.Reactor) to
// the node's Switch.
//
// WARNING: using any name from the below list of the existing reactors will
// result in replacing it with the custom one.
//
//   - MEMPOOL
//   - BLOCKSYNC
//   - CONSENSUS
//   - EVIDENCE
//   - PEX
//   - STATESYNC
func CustomReactors(reactors map[string]p2p.Reactor) Option {
	return func(n *Node) {
		for name, reactor := range reactors {
			if existingReactor := n.sw.Reactor(name); existingReactor != nil {
				n.sw.Logger.Info("Replacing existing reactor with a custom one",
					"name", name, "existing", existingReactor, "custom", reactor)
				n.sw.RemoveReactor(name, existingReactor)
			}
			n.sw.AddReactor(name, reactor)
			// register the new channels to the nodeInfo
			// NOTE: This is a bit messy now with the type casting but is
			// cleaned up in the following version when NodeInfo is changed from
			// and interface to a concrete type
			if ni, ok := n.nodeInfo.(p2p.DefaultNodeInfo); ok {
				for _, chDesc := range reactor.GetChannels() {
					if !ni.HasChannel(chDesc.ID) {
						ni.Channels = append(ni.Channels, chDesc.ID)
						n.transport.AddChannel(chDesc.ID)
					}
				}
				n.nodeInfo = ni
			} else {
				n.Logger.Error("Node info is not of type DefaultNodeInfo. Custom reactor channels can not be added.")
			}
		}
	}
}

// StateProvider overrides the state provider used by state sync to retrieve trusted app hashes and
// build a State object for bootstrapping the node.
// WARNING: this interface is considered unstable and subject to change.
func StateProvider(stateProvider statesync.StateProvider) Option {
	return func(n *Node) {
		n.stateSyncProvider = stateProvider
	}
}

// BootstrapState synchronizes the stores with the application after state sync
// has been performed offline. It is expected that the block store and state
// store are empty at the time the function is called.
//
// If the block store is not empty, the function returns an error.
func BootstrapState(ctx context.Context, config *cfg.Config, dbProvider cfg.DBProvider, height uint64, appHash []byte) (err error) {
	logger := log.NewTMLogger(log.NewSyncWriter(os.Stdout))
	if ctx == nil {
		ctx = context.Background()
	}

	if config == nil {
		logger.Info("no config provided, using default configuration")
		config = cfg.DefaultConfig()
	}

	if dbProvider == nil {
		dbProvider = cfg.DefaultDBProvider
	}
	blockStore, stateDB, err := initDBs(config, dbProvider)

	defer func() {
		if derr := blockStore.Close(); derr != nil {
			logger.Error("Failed to close blockstore", "err", derr)
			// Set the return value
			err = derr
		}
	}()

	if err != nil {
		return err
	}

	if !blockStore.IsEmpty() {
		return fmt.Errorf("blockstore not empty, trying to initialize non empty state")
	}

	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: config.Storage.DiscardABCIResponses,
	})

	defer func() {
		if derr := stateStore.Close(); derr != nil {
			logger.Error("Failed to close statestore", "err", derr)
			// Set the return value
			err = derr
		}
	}()
	state, err := stateStore.Load()
	if err != nil {
		return err
	}

	if !state.IsEmpty() {
		return fmt.Errorf("state not empty, trying to initialize non empty state")
	}

	genState, _, err := LoadStateFromDBOrGenesisDocProvider(stateDB, DefaultGenesisDocProviderFunc(config))
	if err != nil {
		return err
	}

	stateProvider, err := statesync.NewLightClientStateProvider(
		ctx,
		genState.ChainID, genState.Version, genState.InitialHeight,
		config.StateSync.RPCServers, light.TrustOptions{
			Period: config.StateSync.TrustPeriod,
			Height: config.StateSync.TrustHeight,
			Hash:   config.StateSync.TrustHashBytes(),
		}, logger.With("module", "light"))
	if err != nil {
		return fmt.Errorf("failed to set up light client state provider: %w", err)
	}

	state, err = stateProvider.State(ctx, height)
	if err != nil {
		return err
	}
	if appHash == nil {
		logger.Info("warning: cannot verify appHash. Verification will happen when node boots up!")
	} else {
		if !bytes.Equal(appHash, state.AppHash) {
			if err := blockStore.Close(); err != nil {
				logger.Error("failed to close blockstore: %w", err)
			}
			if err := stateStore.Close(); err != nil {
				logger.Error("failed to close statestore: %w", err)
			}
			return fmt.Errorf("the app hash returned by the light client does not match the provided appHash, expected %X, got %X", state.AppHash, appHash)
		}
	}

	commit, err := stateProvider.Commit(ctx, height)
	if err != nil {
		return err
	}

	if err = stateStore.Bootstrap(state); err != nil {
		return err
	}

	err = blockStore.SaveSeenCommit(state.LastBlockHeight, commit)
	if err != nil {
		return err
	}

	// Once the stores are bootstrapped, we need to set the height at which the node has finished
	// statesyncing. This will allow the blocksync reactor to fetch blocks at a proper height.
	// In case this operation fails, it is equivalent to a failure in  online state sync where the operator
	// needs to manually delete the state and blockstores and rerun the bootstrapping process.
	err = stateStore.SetOfflineStateSyncHeight(state.LastBlockHeight)
	if err != nil {
		return fmt.Errorf("failed to set synced height: %w", err)
	}

	return err
}

//------------------------------------------------------------------------------

// NewNode returns a new, ready to go, CometBFT Node.
func NewNode(config *cfg.Config,
	privValidator types.PrivValidator,
	nodeKey *p2p.NodeKey,
	clientCreator proxy.ClientCreator,
	genesisDocProvider GenesisDocProvider,
	dbProvider cfg.DBProvider,
	metricsProvider MetricsProvider,
	logger log.Logger,
	options ...Option,
) (*Node, error) {
	return NewNodeWithContext(context.TODO(), config, privValidator,
		nodeKey, clientCreator, genesisDocProvider, dbProvider,
		metricsProvider, logger, options...)
}

// NewNodeWithContext is cancellable version of NewNode.
func NewNodeWithContext(ctx context.Context,
	config *cfg.Config,
	privValidator types.PrivValidator,
	nodeKey *p2p.NodeKey,
	clientCreator proxy.ClientCreator,
	genesisDocProvider GenesisDocProvider,
	dbProvider cfg.DBProvider,
	metricsProvider MetricsProvider,
	logger log.Logger,
	options ...Option,
) (*Node, error) {
	blockStore, stateDB, err := initDBs(config, dbProvider)
	if err != nil {
		return nil, err
	}

	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: config.Storage.DiscardABCIResponses,
	})

	state, genDoc, err := LoadStateFromDBOrGenesisDocProvider(stateDB, genesisDocProvider)
	if err != nil {
		return nil, err
	}

	csMetrics, p2pMetrics, memplMetrics, smMetrics, abciMetrics, bsMetrics, ssMetrics := metricsProvider(genDoc.ChainID)

	// Create the proxyApp and establish connections to the ABCI app (consensus, mempool, query).
	proxyApp, err := createAndStartProxyAppConns(clientCreator, logger, abciMetrics)
	if err != nil {
		return nil, err
	}

	// EventBus and IndexerService must be started before the handshake because
	// we might need to index the txs of the replayed block as this might not have happened
	// when the node stopped last time (i.e. the node stopped after it saved the block
	// but before it indexed the txs)
	eventBus, err := createAndStartEventBus(logger)
	if err != nil {
		return nil, err
	}

	indexerService, txIndexer, blockIndexer, err := createAndStartIndexerService(config,
		genDoc.ChainID, dbProvider, eventBus, logger)
	if err != nil {
		return nil, err
	}

	// If an address is provided, listen on the socket for a connection from an
	// external signing process.
	if config.PrivValidatorListenAddr != "" {
		// FIXME: we should start services inside OnStart
		privValidator, err = createAndStartPrivValidatorSocketClient(config.PrivValidatorListenAddr, genDoc.ChainID, logger)
		if err != nil {
			return nil, fmt.Errorf("error with private validator socket client: %w", err)
		}
	}

	pubKey, err := privValidator.GetPubKey()
	if err != nil {
		return nil, fmt.Errorf("can't get pubkey: %w", err)
	}

	// Determine whether we should attempt state sync.
	stateSync := config.StateSync.Enable && !onlyValidatorIsUs(state, pubKey)
	if stateSync && state.LastBlockHeight > 0 {
		logger.Info("Found local state with non-zero height, skipping state sync")
		stateSync = false
	}

	// Create the handshaker, which calls RequestInfo, sets the AppVersion on the state,
	// and replays any blocks as necessary to sync CometBFT with the app.
	consensusLogger := logger.With("module", "consensus")
	if !stateSync {
		if err := doHandshake(ctx, stateStore, state, blockStore, genDoc, eventBus, proxyApp, consensusLogger); err != nil {
			return nil, err
		}

		// Reload the state. It will have the Version.Consensus.App set by the
		// Handshake, and may have other modifications as well (ie. depending on
		// what happened during block replay).
		state, err = stateStore.Load()
		if err != nil {
			return nil, fmt.Errorf("cannot load state: %w", err)
		}
	}

	// Determine whether we should do block sync. This must happen after the handshake, since the
	// app may modify the validator set, specifying ourself as the only validator.
	blockSync := !onlyValidatorIsUs(state, pubKey)

	logNodeStartupInfo(state, pubKey, logger, consensusLogger)

	// Make MempoolReactor
	mempool, mempoolReactor := createMempoolAndMempoolReactor(config, proxyApp, state, memplMetrics, logger)

	// Make Evidence Reactor
	evidenceReactor, evidencePool, err := createEvidenceReactor(config, dbProvider, stateStore, blockStore, logger)
	if err != nil {
		return nil, err
	}

	// make block executor for consensus and blocksync reactors to execute blocks
	blockExec := sm.NewBlockExecutor(
		stateStore,
		logger.With("module", "state"),
		proxyApp.Consensus(),
		mempool,
		evidencePool,
		blockStore,
		sm.BlockExecutorWithMetrics(smMetrics),
	)

	offlineStateSyncHeight := int64(0)
	if blockStore.Height() == 0 {
		offlineStateSyncHeight, err = blockExec.Store().GetOfflineStateSyncHeight()
		if err != nil && err.Error() != "value empty" {
			panic(fmt.Sprintf("failed to retrieve statesynced height from store %s; expected state store height to be %v", err, state.LastBlockHeight))
		}
	}
	// Make BlocksyncReactor. Don't start block sync if we're doing a state sync first.
	bcReactor, err := createBlocksyncReactor(config, state, blockExec, blockStore, blockSync && !stateSync, logger, bsMetrics, offlineStateSyncHeight)
	if err != nil {
		return nil, fmt.Errorf("could not create blocksync reactor: %w", err)
	}

	// Make ConsensusReactor
	consensusReactor, consensusState := createConsensusReactor(
		config, state, blockExec, blockStore, mempool, evidencePool,
		privValidator, csMetrics, stateSync || blockSync, eventBus, consensusLogger, offlineStateSyncHeight,
	)

	err = stateStore.SetOfflineStateSyncHeight(0)
	if err != nil {
		panic(fmt.Sprintf("failed to reset the offline state sync height %s", err))
	}
	// Set up state sync reactor, and schedule a sync if requested.
	// FIXME The way we do phased startups (e.g. replay -> block sync -> consensus) is very messy,
	// we should clean this whole thing up. See:
	// https://github.com/tendermint/tendermint/issues/4644
	stateSyncReactor := statesync.NewReactor(
		*config.StateSync,
		proxyApp.Snapshot(),
		proxyApp.Query(),
		ssMetrics,
	)
	stateSyncReactor.SetLogger(logger.With("module", "statesync"))

	nodeInfo, err := makeNodeInfo(config, nodeKey, txIndexer, genDoc, state)
	if err != nil {
		return nil, err
	}

	// Setup Transport.
	transport, peerFilters := createTransport(config, nodeInfo, nodeKey, proxyApp)

	// Setup Switch.
	p2pLogger := logger.With("module", "p2p")
	sw := createSwitch(
		config, transport, p2pMetrics, peerFilters, mempoolReactor, bcReactor,
		stateSyncReactor, consensusReactor, evidenceReactor, nodeInfo, nodeKey, p2pLogger,
	)

	err = sw.AddPersistentPeers(splitAndTrimEmpty(config.P2P.PersistentPeers, ",", " "))
	if err != nil {
		return nil, fmt.Errorf("could not add peers from persistent_peers field: %w", err)
	}

	err = sw.AddUnconditionalPeerIDs(splitAndTrimEmpty(config.P2P.UnconditionalPeerIDs, ",", " "))
	if err != nil {
		return nil, fmt.Errorf("could not add peer ids from unconditional_peer_ids field: %w", err)
	}

	addrBook, err := createAddrBookAndSetOnSwitch(config, sw, p2pLogger, nodeKey)
	if err != nil {
		return nil, fmt.Errorf("could not create addrbook: %w", err)
	}

	// Optionally, start the pex reactor
	//
	// TODO:
	//
	// We need to set Seeds and PersistentPeers on the switch,
	// since it needs to be able to use these (and their DNS names)
	// even if the PEX is off. We can include the DNS name in the NetAddress,
	// but it would still be nice to have a clear list of the current "PersistentPeers"
	// somewhere that we can return with net_info.
	//
	// If PEX is on, it should handle dialing the seeds. Otherwise the switch does it.
	// Note we currently use the addrBook regardless at least for AddOurAddress
	var pexReactor *pex.Reactor
	if config.P2P.PexReactor {
		pexReactor = createPEXReactorAndAddToSwitch(addrBook, config, sw, logger)
	}

	// Add private IDs to addrbook to block those peers being added
	addrBook.AddPrivateIDs(splitAndTrimEmpty(config.P2P.PrivatePeerIDs, ",", " "))

	node := &Node{
		config:        config,
		genesisDoc:    genDoc,
		privValidator: privValidator,

		transport: transport,
		sw:        sw,
		addrBook:  addrBook,
		nodeInfo:  nodeInfo,
		nodeKey:   nodeKey,

		stateStore:       stateStore,
		blockStore:       blockStore,
		bcReactor:        bcReactor,
		mempoolReactor:   mempoolReactor,
		mempool:          mempool,
		consensusState:   consensusState,
		consensusReactor: consensusReactor,
		stateSyncReactor: stateSyncReactor,
		stateSync:        stateSync,
		stateSyncGenesis: state, // Shouldn't be necessary, but need a way to pass the genesis state
		pexReactor:       pexReactor,
		evidencePool:     evidencePool,
		proxyApp:         proxyApp,
		txIndexer:        txIndexer,
		indexerService:   indexerService,
		blockIndexer:     blockIndexer,
		eventBus:         eventBus,
	}
	node.BaseService = *service.NewBaseService(logger, "Node", node)

	for _, option := range options {
		option(node)
	}

	return node, nil
}

// OnStart starts the Node. It implements service.Service.
func (n *Node) OnStart() error {
	now := cmttime.Now()
	genTime := n.genesisDoc.GenesisTime
	if genTime.After(now) {
		n.Logger.Info("Genesis time is in the future. Sleeping until then...", "genTime", genTime)
		time.Sleep(genTime.Sub(now))
	}

	// run pprof server if it is enabled
	if n.config.RPC.IsPprofEnabled() {
		n.pprofSrv = n.startPprofServer()
	}

	// begin prometheus metrics gathering if it is enabled
	if n.config.Instrumentation.IsPrometheusEnabled() {
		n.prometheusSrv = n.startPrometheusServer()
	}

	// Start the RPC server before the P2P server
	// so we can eg. receive txs for the first block
	if n.config.RPC.ListenAddress != "" {
		listeners, err := n.startRPC()
		if err != nil {
			return err
		}
		n.rpcListeners = listeners
	}

	// Start the transport.
	addr, err := p2p.NewNetAddressString(p2p.IDAddressString(n.nodeKey.ID(), n.config.P2P.ListenAddress))
	if err != nil {
		return err
	}
	if err := n.transport.Listen(*addr); err != nil {
		return err
	}

	n.isListening = true

	// Start the switch (the P2P server).
	err = n.sw.Start()
	if err != nil {
		return err
	}

	// Always connect to persistent peers
	err = n.sw.DialPeersAsync(splitAndTrimEmpty(n.config.P2P.PersistentPeers, ",", " "))
	if err != nil {
		return fmt.Errorf("could not dial peers from persistent_peers field: %w", err)
	}

	// Run state sync
	if n.stateSync {
		bcR, ok := n.bcReactor.(blockSyncReactor)
		if !ok {
			return fmt.Errorf("this blocksync reactor does not support switching from state sync")
		}
		err := startStateSync(n.stateSyncReactor, bcR, n.stateSyncProvider,
			n.config.StateSync, n.stateStore, n.blockStore, n.stateSyncGenesis)
		if err != nil {
			return fmt.Errorf("failed to start state sync: %w", err)
		}
	}

	return nil
}

// OnStop stops the Node. It implements service.Service.
func (n *Node) OnStop() {
	n.BaseService.OnStop()

	n.Logger.Info("Stopping Node")

	// first stop the non-reactor services
	if err := n.eventBus.Stop(); err != nil {
		n.Logger.Error("Error closing eventBus", "err", err)
	}
	if err := n.indexerService.Stop(); err != nil {
		n.Logger.Error("Error closing indexerService", "err", err)
	}

	// now stop the reactors
	if err := n.sw.Stop(); err != nil {
		n.Logger.Error("Error closing switch", "err", err)
	}

	if err := n.transport.Close(); err != nil {
		n.Logger.Error("Error closing transport", "err", err)
	}

	n.isListening = false

	// finally stop the listeners / external services
	for _, l := range n.rpcListeners {
		n.Logger.Info("Closing rpc listener", "listener", l)
		if err := l.Close(); err != nil {
			n.Logger.Error("Error closing listener", "listener", l, "err", err)
		}
	}

	if pvsc, ok := n.privValidator.(service.Service); ok {
		if err := pvsc.Stop(); err != nil {
			n.Logger.Error("Error closing private validator", "err", err)
		}
	}

	if n.prometheusSrv != nil {
		if err := n.prometheusSrv.Shutdown(context.Background()); err != nil {
			// Error from closing listeners, or context timeout:
			n.Logger.Error("Prometheus HTTP server Shutdown", "err", err)
		}
	}
	if n.pprofSrv != nil {
		if err := n.pprofSrv.Shutdown(context.Background()); err != nil {
			n.Logger.Error("Pprof HTTP server Shutdown", "err", err)
		}
	}
	if n.blockStore != nil {
		n.Logger.Info("Closing blockstore")
		if err := n.blockStore.Close(); err != nil {
			n.Logger.Error("problem closing blockstore", "err", err)
		}
	}
	if n.stateStore != nil {
		n.Logger.Info("Closing statestore")
		if err := n.stateStore.Close(); err != nil {
			n.Logger.Error("problem closing statestore", "err", err)
		}
	}
	if n.evidencePool != nil {
		n.Logger.Info("Closing evidencestore")
		if err := n.EvidencePool().Close(); err != nil {
			n.Logger.Error("problem closing evidencestore", "err", err)
		}
	}
}

// ConfigureRPC makes sure RPC has all the objects it needs to operate.
func (n *Node) ConfigureRPC() (*rpccore.Environment, error) {
	pubKey, err := n.privValidator.GetPubKey()
	if pubKey == nil || err != nil {
		return nil, fmt.Errorf("can't get pubkey: %w", err)
	}
	rpcCoreEnv := rpccore.Environment{
		ProxyAppQuery:   n.proxyApp.Query(),
		ProxyAppMempool: n.proxyApp.Mempool(),

		StateStore:     n.stateStore,
		BlockStore:     n.blockStore,
		EvidencePool:   n.evidencePool,
		ConsensusState: n.consensusState,
		P2PPeers:       n.sw,
		P2PTransport:   n,
		PubKey:         pubKey,

		GenDoc:           n.genesisDoc,
		TxIndexer:        n.txIndexer,
		BlockIndexer:     n.blockIndexer,
		ConsensusReactor: n.consensusReactor,
		EventBus:         n.eventBus,
		Mempool:          n.mempool,

		Logger: n.Logger.With("module", "rpc"),

		Config: *n.config.RPC,
	}
	if err := rpcCoreEnv.InitGenesisChunks(); err != nil {
		return nil, err
	}
	return &rpcCoreEnv, nil
}

func (n *Node) startRPC() ([]net.Listener, error) {
	env, err := n.ConfigureRPC()
	if err != nil {
		return nil, err
	}

	listenAddrs := splitAndTrimEmpty(n.config.RPC.ListenAddress, ",", " ")
	routes := env.GetRoutes()

	if n.config.RPC.Unsafe {
		env.AddUnsafeRoutes(routes)
	}

	config := rpcserver.DefaultConfig()
	config.MaxBodyBytes = n.config.RPC.MaxBodyBytes
	config.MaxHeaderBytes = n.config.RPC.MaxHeaderBytes
	config.MaxOpenConnections = n.config.RPC.MaxOpenConnections
	// If necessary adjust global WriteTimeout to ensure it's greater than
	// TimeoutBroadcastTxCommit.
	// See https://github.com/tendermint/tendermint/issues/3435
	if config.WriteTimeout <= n.config.RPC.TimeoutBroadcastTxCommit {
		config.WriteTimeout = n.config.RPC.TimeoutBroadcastTxCommit + 1*time.Second
	}

	// we may expose the rpc over both a unix and tcp socket
	listeners := make([]net.Listener, len(listenAddrs))
	for i, listenAddr := range listenAddrs {
		mux := http.NewServeMux()
		rpcLogger := n.Logger.With("module", "rpc-server")
		wmLogger := rpcLogger.With("protocol", "websocket")
		wm := rpcserver.NewWebsocketManager(routes,
			rpcserver.OnDisconnect(func(remoteAddr string) {
				err := n.eventBus.UnsubscribeAll(context.Background(), remoteAddr)
				if err != nil && err != cmtpubsub.ErrSubscriptionNotFound {
					wmLogger.Error("Failed to unsubscribe addr from events", "addr", remoteAddr, "err", err)
				}
			}),
			rpcserver.ReadLimit(config.MaxBodyBytes),
			rpcserver.WriteChanCapacity(n.config.RPC.WebSocketWriteBufferSize),
		)
		wm.SetLogger(wmLogger)
		mux.HandleFunc("/websocket", wm.WebsocketHandler)
		rpcserver.RegisterRPCFuncs(mux, routes, rpcLogger)
		listener, err := rpcserver.Listen(
			listenAddr,
			config.MaxOpenConnections,
		)
		if err != nil {
			return nil, err
		}

		var rootHandler http.Handler = mux
		if n.config.RPC.IsCorsEnabled() {
			corsMiddleware := cors.New(cors.Options{
				AllowedOrigins: n.config.RPC.CORSAllowedOrigins,
				AllowedMethods: n.config.RPC.CORSAllowedMethods,
				AllowedHeaders: n.config.RPC.CORSAllowedHeaders,
			})
			rootHandler = corsMiddleware.Handler(mux)
		}
		if n.config.RPC.IsTLSEnabled() {
			go func() {
				if err := rpcserver.ServeTLS(
					listener,
					rootHandler,
					n.config.RPC.CertFile(),
					n.config.RPC.KeyFile(),
					rpcLogger,
					config,
				); err != nil {
					n.Logger.Error("Error serving server with TLS", "err", err)
				}
			}()
		} else {
			go func() {
				if err := rpcserver.Serve(
					listener,
					rootHandler,
					rpcLogger,
					config,
				); err != nil {
					n.Logger.Error("Error serving server", "err", err)
				}
			}()
		}

		listeners[i] = listener
	}

	// we expose a simplified api over grpc for convenience to app devs
	grpcListenAddr := n.config.RPC.GRPCListenAddress
	if grpcListenAddr != "" {
		config := rpcserver.DefaultConfig()
		config.MaxBodyBytes = n.config.RPC.MaxBodyBytes
		config.MaxHeaderBytes = n.config.RPC.MaxHeaderBytes
		// NOTE: GRPCMaxOpenConnections is used, not MaxOpenConnections
		config.MaxOpenConnections = n.config.RPC.GRPCMaxOpenConnections
		// If necessary adjust global WriteTimeout to ensure it's greater than
		// TimeoutBroadcastTxCommit.
		// See https://github.com/tendermint/tendermint/issues/3435
		if config.WriteTimeout <= n.config.RPC.TimeoutBroadcastTxCommit {
			config.WriteTimeout = n.config.RPC.TimeoutBroadcastTxCommit + 1*time.Second
		}
		listener, err := rpcserver.Listen(grpcListenAddr, config.MaxOpenConnections)
		if err != nil {
			return nil, err
		}
		go func() {
			//nolint:staticcheck // SA1019: core_grpc.StartGRPCClient is deprecated: A new gRPC API will be introduced after v0.38.
			if err := grpccore.StartGRPCServer(env, listener); err != nil {
				n.Logger.Error("Error starting gRPC server", "err", err)
			}
		}()
		listeners = append(listeners, listener)

	}

	return listeners, nil
}

// startPrometheusServer starts a Prometheus HTTP server, listening for metrics
// collectors on addr.
func (n *Node) startPrometheusServer() *http.Server {
	srv := &http.Server{
		Addr: n.config.Instrumentation.PrometheusListenAddr,
		Handler: promhttp.InstrumentMetricHandler(
			prometheus.DefaultRegisterer, promhttp.HandlerFor(
				prometheus.DefaultGatherer,
				promhttp.HandlerOpts{MaxRequestsInFlight: n.config.Instrumentation.MaxOpenConnections},
			),
		),
		ReadHeaderTimeout: readHeaderTimeout,
	}
	go func() {
		if err := srv.ListenAndServe(); err != http.ErrServerClosed {
			// Error starting or closing listener:
			n.Logger.Error("Prometheus HTTP server ListenAndServe", "err", err)
		}
	}()
	return srv
}

// starts a ppro
func (n *Node) startPprofServer() *http.Server {
	srv := &http.Server{
		Addr:              n.config.RPC.PprofListenAddress,
		Handler:           nil,
		ReadHeaderTimeout: readHeaderTimeout,
	}
	go func() {
		if err := srv.ListenAndServe(); err != http.ErrServerClosed {
			// Error starting or closing listener:
			n.Logger.Error("pprof HTTP server ListenAndServe", "err", err)
		}
	}()
	return srv
}

// Switch returns the Node's Switch.
func (n *Node) Switch() *p2p.Switch {
	return n.sw
}

// BlockStore returns the Node's BlockStore.
func (n *Node) BlockStore() *store.BlockStore {
	return n.blockStore
}

// ConsensusReactor returns the Node's ConsensusReactor.
func (n *Node) ConsensusReactor() *cs.Reactor {
	return n.consensusReactor
}

// MempoolReactor returns the Node's mempool reactor.
func (n *Node) MempoolReactor() p2p.Reactor {
	return n.mempoolReactor
}

// Mempool returns the Node's mempool.
func (n *Node) Mempool() mempl.Mempool {
	return n.mempool
}

// PEXReactor returns the Node's PEXReactor. It returns nil if PEX is disabled.
func (n *Node) PEXReactor() *pex.Reactor {
	return n.pexReactor
}

// EvidencePool returns the Node's EvidencePool.
func (n *Node) EvidencePool() *evidence.Pool {
	return n.evidencePool
}

// EventBus returns the Node's EventBus.
func (n *Node) EventBus() *types.EventBus {
	return n.eventBus
}

// PrivValidator returns the Node's PrivValidator.
// XXX: for convenience only!
func (n *Node) PrivValidator() types.PrivValidator {
	return n.privValidator
}

// GenesisDoc returns the Node's GenesisDoc.
func (n *Node) GenesisDoc() *types.GenesisDoc {
	return n.genesisDoc
}

// ProxyApp returns the Node's AppConns, representing its connections to the ABCI application.
func (n *Node) ProxyApp() proxy.AppConns {
	return n.proxyApp
}

// Config returns the Node's config.
func (n *Node) Config() *cfg.Config {
	return n.config
}

//------------------------------------------------------------------------------

func (n *Node) Listeners() []string {
	return []string{
		fmt.Sprintf("Listener(@%v)", n.config.P2P.ExternalAddress),
	}
}

func (n *Node) IsListening() bool {
	return n.isListening
}

// NodeInfo returns the Node's Info from the Switch.
func (n *Node) NodeInfo() p2p.NodeInfo {
	return n.nodeInfo
}

func makeNodeInfo(
	config *cfg.Config,
	nodeKey *p2p.NodeKey,
	txIndexer txindex.TxIndexer,
	genDoc *types.GenesisDoc,
	state sm.State,
) (p2p.DefaultNodeInfo, error) {
	txIndexerStatus := "on"
	if _, ok := txIndexer.(*null.TxIndex); ok {
		txIndexerStatus = "off"
	}

	nodeInfo := p2p.DefaultNodeInfo{
		ProtocolVersion: p2p.NewProtocolVersion(
			version.P2PProtocol, // global
			state.Version.Consensus.Block,
			state.Version.Consensus.App,
		),
		DefaultNodeID: nodeKey.ID(),
		Network:       genDoc.ChainID,
		Version:       version.TMCoreSemVer,
		Channels: []byte{
			bc.BlocksyncChannel,
			cs.StateChannel, cs.DataChannel, cs.VoteChannel, cs.VoteSetBitsChannel,
			mempl.MempoolChannel,
			evidence.EvidenceChannel,
			statesync.SnapshotChannel, statesync.ChunkChannel,
		},
		Moniker: config.Moniker,
		Other: p2p.DefaultNodeInfoOther{
			TxIndex:    txIndexerStatus,
			RPCAddress: config.RPC.ListenAddress,
		},
	}

	if config.P2P.PexReactor {
		nodeInfo.Channels = append(nodeInfo.Channels, pex.PexChannel)
	}

	lAddr := config.P2P.ExternalAddress

	if lAddr == "" {
		lAddr = config.P2P.ListenAddress
	}

	nodeInfo.ListenAddr = lAddr

	err := nodeInfo.Validate()
	return nodeInfo, err
}
