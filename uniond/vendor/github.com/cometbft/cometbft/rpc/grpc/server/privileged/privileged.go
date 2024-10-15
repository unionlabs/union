package privileged

import (
	"fmt"
	"net"

	"google.golang.org/grpc"

	pbpruningsvc "github.com/cometbft/cometbft/api/cometbft/services/pruning/v1"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/rpc/grpc/server/services/pruningservice"
	sm "github.com/cometbft/cometbft/state"
)

// Option is any function that allows for configuration of the gRPC server
// during its creation.
type Option func(*serverBuilder)

type serverBuilder struct {
	listener       net.Listener
	pruningService pbpruningsvc.PruningServiceServer
	logger         log.Logger
	grpcOpts       []grpc.ServerOption
}

func newServerBuilder(listener net.Listener) *serverBuilder {
	return &serverBuilder{
		listener: listener,
		logger:   log.NewNopLogger(),
		grpcOpts: make([]grpc.ServerOption, 0),
	}
}

// WithVersionService enables the version service on the CometBFT server.
func WithPruningService(pruner *sm.Pruner, logger log.Logger) Option {
	return func(b *serverBuilder) {
		b.pruningService = pruningservice.New(pruner, logger)
	}
}

// WithLogger enables logging using the given logger. If not specified, the
// gRPC server does not log anything.
func WithLogger(logger log.Logger) Option {
	return func(b *serverBuilder) {
		b.logger = logger.With("module", "privileged-grpc-server")
	}
}

// WithGRPCOption allows one to specify Google gRPC server options during the
// construction of the CometBFT gRPC server.
func WithGRPCOption(opt grpc.ServerOption) Option {
	return func(b *serverBuilder) {
		b.grpcOpts = append(b.grpcOpts, opt)
	}
}

// Serve constructs and runs a CometBFT privileged gRPC server using the given
// listener and options.
//
// This function only returns upon error, otherwise it blocks the current
// goroutine.
func Serve(listener net.Listener, opts ...Option) error {
	b := newServerBuilder(listener)
	for _, opt := range opts {
		opt(b)
	}
	server := grpc.NewServer(b.grpcOpts...)
	if b.pruningService != nil {
		pbpruningsvc.RegisterPruningServiceServer(server, b.pruningService)
		b.logger.Debug("Registered pruning service")
	}
	b.logger.Info("serve", "msg", fmt.Sprintf("Starting privileged gRPC server on %s", listener.Addr()))
	return server.Serve(b.listener)
}
