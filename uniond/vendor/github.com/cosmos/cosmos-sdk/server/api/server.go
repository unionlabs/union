package api

import (
	"fmt"
	"net"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	tmrpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
	gateway "github.com/cosmos/gogogateway"
	"github.com/gorilla/handlers"
	"github.com/gorilla/mux"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/codec/legacy"
	"github.com/cosmos/cosmos-sdk/server/config"
	"github.com/cosmos/cosmos-sdk/telemetry"
	grpctypes "github.com/cosmos/cosmos-sdk/types/grpc"
)

// Server defines the server's API interface.
type Server struct {
	Router            *mux.Router
	GRPCGatewayRouter *runtime.ServeMux
	ClientCtx         client.Context

	logger  log.Logger
	metrics *telemetry.Metrics
	// Start() is blocking and generally called from a separate goroutine.
	// Close() can be called asynchronously and access shared memory
	// via the listener. Therefore, we sync access to Start and Close with
	// this mutex to avoid data races.
	mtx      sync.Mutex
	listener net.Listener
}

// CustomGRPCHeaderMatcher for mapping request headers to
// GRPC metadata.
// HTTP headers that start with 'Grpc-Metadata-' are automatically mapped to
// gRPC metadata after removing prefix 'Grpc-Metadata-'. We can use this
// CustomGRPCHeaderMatcher if headers don't start with `Grpc-Metadata-`
func CustomGRPCHeaderMatcher(key string) (string, bool) {
	switch strings.ToLower(key) {
	case grpctypes.GRPCBlockHeightHeader:
		return grpctypes.GRPCBlockHeightHeader, true
	default:
		return runtime.DefaultHeaderMatcher(key)
	}
}

func New(clientCtx client.Context, logger log.Logger) *Server {
	// The default JSON marshaller used by the gRPC-Gateway is unable to marshal non-nullable non-scalar fields.
	// Using the gogo/gateway package with the gRPC-Gateway WithMarshaler option fixes the scalar field marshalling issue.
	marshalerOption := &gateway.JSONPb{
		EmitDefaults: true,
		Indent:       "",
		OrigName:     true,
		AnyResolver:  clientCtx.InterfaceRegistry,
	}

	return &Server{
		Router:    mux.NewRouter(),
		ClientCtx: clientCtx,
		logger:    logger,
		GRPCGatewayRouter: runtime.NewServeMux(
			// Custom marshaler option is required for gogo proto
			runtime.WithMarshalerOption(runtime.MIMEWildcard, marshalerOption),

			// This is necessary to get error details properly
			// marshalled in unary requests.
			runtime.WithProtoErrorHandler(runtime.DefaultHTTPProtoErrorHandler),

			// Custom header matcher for mapping request headers to
			// GRPC metadata
			runtime.WithIncomingHeaderMatcher(CustomGRPCHeaderMatcher),
		),
	}
}

// Start starts the API server. Internally, the API server leverages Tendermint's
// JSON RPC server. Configuration options are provided via config.APIConfig
// and are delegated to the Tendermint JSON RPC server. The process is
// non-blocking, so an external signal handler must be used.
func (s *Server) Start(cfg config.Config) error {
	s.mtx.Lock()

	tmCfg := tmrpcserver.DefaultConfig()
	tmCfg.MaxOpenConnections = int(cfg.API.MaxOpenConnections)
	tmCfg.ReadTimeout = time.Duration(cfg.API.RPCReadTimeout) * time.Second
	tmCfg.WriteTimeout = time.Duration(cfg.API.RPCWriteTimeout) * time.Second
	tmCfg.MaxBodyBytes = int64(cfg.API.RPCMaxBodyBytes)

	listener, err := tmrpcserver.Listen(cfg.API.Address, tmCfg)
	if err != nil {
		s.mtx.Unlock()
		return err
	}

	s.registerGRPCGatewayRoutes()
	s.listener = listener
	var h http.Handler = s.Router

	s.mtx.Unlock()

	if cfg.API.EnableUnsafeCORS {
		allowAllCORS := handlers.CORS(handlers.AllowedHeaders([]string{"Content-Type"}))
		return tmrpcserver.Serve(s.listener, allowAllCORS(h), s.logger, tmCfg)
	}

	s.logger.Info("starting API server...")
	return tmrpcserver.Serve(s.listener, s.Router, s.logger, tmCfg)
}

// Close closes the API server.
func (s *Server) Close() error {
	s.mtx.Lock()
	defer s.mtx.Unlock()
	return s.listener.Close()
}

func (s *Server) registerGRPCGatewayRoutes() {
	s.Router.PathPrefix("/").Handler(s.GRPCGatewayRouter)
}

func (s *Server) SetTelemetry(m *telemetry.Metrics) {
	s.mtx.Lock()
	s.metrics = m
	s.registerMetrics()
	s.mtx.Unlock()
}

func (s *Server) registerMetrics() {
	metricsHandler := func(w http.ResponseWriter, r *http.Request) {
		format := strings.TrimSpace(r.FormValue("format"))

		gr, err := s.metrics.Gather(format)
		if err != nil {
			writeErrorResponse(w, http.StatusBadRequest, fmt.Sprintf("failed to gather metrics: %s", err))
			return
		}

		w.Header().Set("Content-Type", gr.ContentType)
		_, _ = w.Write(gr.Metrics)
	}

	s.Router.HandleFunc("/metrics", metricsHandler).Methods("GET")
}

// errorResponse defines the attributes of a JSON error response.
type errorResponse struct {
	Code  int    `json:"code,omitempty"`
	Error string `json:"error"`
}

// newErrorResponse creates a new errorResponse instance.
func newErrorResponse(code int, err string) errorResponse {
	return errorResponse{Code: code, Error: err}
}

// writeErrorResponse prepares and writes a HTTP error
// given a status code and an error message.
func writeErrorResponse(w http.ResponseWriter, status int, err string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_, _ = w.Write(legacy.Cdc.MustMarshalJSON(newErrorResponse(0, err)))
}
