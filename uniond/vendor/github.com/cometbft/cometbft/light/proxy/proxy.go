package proxy

import (
	"context"
	"fmt"
	"net"
	"net/http"

	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/light"
	lrpc "github.com/cometbft/cometbft/light/rpc"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	rpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
)

// A Proxy defines parameters for running an HTTP server proxy.
type Proxy struct {
	Addr     string // TCP address to listen on, ":http" if empty
	Config   *rpcserver.Config
	Client   *lrpc.Client
	Logger   log.Logger
	Listener net.Listener
}

// NewProxy creates the struct used to run an HTTP server for serving light
// client rpc requests.
func NewProxy(
	lightClient *light.Client,
	listenAddr, providerAddr string,
	config *rpcserver.Config,
	logger log.Logger,
	opts ...lrpc.Option,
) (*Proxy, error) {
	rpcClient, err := rpchttp.NewWithTimeout(providerAddr, "/websocket", uint(config.WriteTimeout.Seconds()))
	if err != nil {
		return nil, fmt.Errorf("failed to create http client for %s: %w", providerAddr, err)
	}

	return &Proxy{
		Addr:   listenAddr,
		Config: config,
		Client: lrpc.NewClient(rpcClient, lightClient, opts...),
		Logger: logger,
	}, nil
}

// ListenAndServe configures the rpcserver.WebsocketManager, sets up the RPC
// routes to proxy via Client, and starts up an HTTP server on the TCP network
// address p.Addr.
// See http#Server#ListenAndServe.
func (p *Proxy) ListenAndServe() error {
	listener, mux, err := p.listen()
	if err != nil {
		return err
	}
	p.Listener = listener

	return rpcserver.Serve(
		listener,
		mux,
		p.Logger,
		p.Config,
	)
}

// ListenAndServeTLS acts identically to ListenAndServe, except that it expects
// HTTPS connections.
// See http#Server#ListenAndServeTLS.
func (p *Proxy) ListenAndServeTLS(certFile, keyFile string) error {
	listener, mux, err := p.listen()
	if err != nil {
		return err
	}
	p.Listener = listener

	return rpcserver.ServeTLS(
		listener,
		mux,
		certFile,
		keyFile,
		p.Logger,
		p.Config,
	)
}

func (p *Proxy) listen() (net.Listener, *http.ServeMux, error) {
	mux := http.NewServeMux()

	// 1) Register regular routes.
	r := RPCRoutes(p.Client)
	rpcserver.RegisterRPCFuncs(mux, r, p.Logger)

	// 2) Allow websocket connections.
	wmLogger := p.Logger.With("protocol", "websocket")
	wm := rpcserver.NewWebsocketManager(r,
		rpcserver.OnDisconnect(func(remoteAddr string) {
			err := p.Client.UnsubscribeAll(context.Background(), remoteAddr)
			if err != nil && err != cmtpubsub.ErrSubscriptionNotFound {
				wmLogger.Error("Failed to unsubscribe addr from events", "addr", remoteAddr, "err", err)
			}
		}),
		rpcserver.ReadLimit(p.Config.MaxBodyBytes),
	)
	wm.SetLogger(wmLogger)
	mux.HandleFunc("/websocket", wm.WebsocketHandler)

	// 3) Start a client.
	if !p.Client.IsRunning() {
		if err := p.Client.Start(); err != nil {
			return nil, mux, fmt.Errorf("can't start client: %w", err)
		}
	}

	// 4) Start listening for new connections.
	listener, err := rpcserver.Listen(p.Addr, p.Config.MaxOpenConnections)
	if err != nil {
		return nil, mux, err
	}

	return listener, mux, nil
}
