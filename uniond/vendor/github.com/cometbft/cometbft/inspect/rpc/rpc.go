package rpc

import (
	"context"
	"net/http"
	"time"

	"github.com/rs/cors"

	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/core"
	"github.com/cometbft/cometbft/rpc/jsonrpc/server"
	"github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/state/txindex"
)

// Server defines parameters for running an Inspector rpc server.
type Server struct {
	Addr    string // TCP address to listen on, ":http" if empty
	Handler http.Handler
	Logger  log.Logger
	Config  *config.RPCConfig
}

// Routes returns the set of routes used by the Inspector server.
func Routes(cfg config.RPCConfig, s state.Store, bs state.BlockStore, txidx txindex.TxIndexer, blkidx indexer.BlockIndexer, logger log.Logger) core.RoutesMap { //nolint: lll
	env := &core.Environment{
		Config:           cfg,
		BlockIndexer:     blkidx,
		TxIndexer:        txidx,
		StateStore:       s,
		BlockStore:       bs,
		ConsensusReactor: waitSyncCheckerImpl{},
		Logger:           logger,
	}
	return core.RoutesMap{
		"blockchain":       server.NewRPCFunc(env.BlockchainInfo, "minHeight,maxHeight"),
		"consensus_params": server.NewRPCFunc(env.ConsensusParams, "height"),
		"block":            server.NewRPCFunc(env.Block, "height"),
		"block_by_hash":    server.NewRPCFunc(env.BlockByHash, "hash"),
		"block_results":    server.NewRPCFunc(env.BlockResults, "height"),
		"commit":           server.NewRPCFunc(env.Commit, "height"),
		"header":           server.NewRPCFunc(env.Header, "height"),
		"header_by_hash":   server.NewRPCFunc(env.HeaderByHash, "hash"),
		"validators":       server.NewRPCFunc(env.Validators, "height,page,per_page"),
		"tx":               server.NewRPCFunc(env.Tx, "hash,prove"),
		"tx_search":        server.NewRPCFunc(env.TxSearch, "query,prove,page,per_page,order_by"),
		"block_search":     server.NewRPCFunc(env.BlockSearch, "query,page,per_page,order_by"),
	}
}

// Handler returns the http.Handler configured for use with an Inspector server. Handler
// registers the routes on the http.Handler and also registers the websocket handler
// and the CORS handler if specified by the configuration options.
func Handler(rpcConfig *config.RPCConfig, routes core.RoutesMap, logger log.Logger) http.Handler {
	mux := http.NewServeMux()
	wmLogger := logger.With("protocol", "websocket")
	wm := server.NewWebsocketManager(routes,
		server.ReadLimit(rpcConfig.MaxBodyBytes))
	wm.SetLogger(wmLogger)
	mux.HandleFunc("/websocket", wm.WebsocketHandler)

	server.RegisterRPCFuncs(mux, routes, logger)
	var rootHandler http.Handler = mux
	if rpcConfig.IsCorsEnabled() {
		rootHandler = addCORSHandler(rpcConfig, mux)
	}
	return rootHandler
}

func addCORSHandler(rpcConfig *config.RPCConfig, h http.Handler) http.Handler {
	corsMiddleware := cors.New(cors.Options{
		AllowedOrigins: rpcConfig.CORSAllowedOrigins,
		AllowedMethods: rpcConfig.CORSAllowedMethods,
		AllowedHeaders: rpcConfig.CORSAllowedHeaders,
	})
	h = corsMiddleware.Handler(h)
	return h
}

type waitSyncCheckerImpl struct{}

func (waitSyncCheckerImpl) WaitSync() bool {
	return false
}

// ListenAndServe listens on the address specified in srv.Addr and handles any
// incoming requests over HTTP using the Inspector rpc handler specified on the server.
func (srv *Server) ListenAndServe(ctx context.Context) error {
	listener, err := server.Listen(srv.Addr, srv.Config.MaxOpenConnections)
	if err != nil {
		return err
	}
	go func() {
		<-ctx.Done()
		listener.Close()
	}()
	return server.Serve(listener, srv.Handler, srv.Logger, serverRPCConfig(srv.Config))
}

// ListenAndServeTLS listens on the address specified in srv.Addr. ListenAndServeTLS handles
// incoming requests over HTTPS using the Inspector rpc handler specified on the server.
func (srv *Server) ListenAndServeTLS(ctx context.Context, certFile, keyFile string) error {
	listener, err := server.Listen(srv.Addr, srv.Config.MaxOpenConnections)
	if err != nil {
		return err
	}
	go func() {
		<-ctx.Done()
		listener.Close()
	}()
	return server.ServeTLS(listener, srv.Handler, certFile, keyFile, srv.Logger, serverRPCConfig(srv.Config))
}

func serverRPCConfig(r *config.RPCConfig) *server.Config {
	cfg := server.DefaultConfig()
	cfg.MaxBodyBytes = r.MaxBodyBytes
	cfg.MaxHeaderBytes = r.MaxHeaderBytes
	// If necessary adjust global WriteTimeout to ensure it's greater than
	// TimeoutBroadcastTxCommit.
	// See https://github.com/tendermint/tendermint/issues/3435
	if cfg.WriteTimeout <= r.TimeoutBroadcastTxCommit {
		cfg.WriteTimeout = r.TimeoutBroadcastTxCommit + 1*time.Second
	}
	return cfg
}
