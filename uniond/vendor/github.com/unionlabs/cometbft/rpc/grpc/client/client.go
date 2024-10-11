// Note that no auto-generated gRPC code is directly exposed via the client
// interface. This is on purpose to minimize the potential impact on users of
// switching to Google's Go code generator in future. See
// https://github.com/cometbft/cometbft/issues/731 for more details.
package client

import (
	"context"
	"fmt"
	"net"

	ggrpc "google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	cmtnet "github.com/cometbft/cometbft/internal/net"
)

type Option func(*clientBuilder)

// Client defines the full client interface for interacting with a CometBFT
// node via gRPC.
type Client interface {
	VersionServiceClient
	BlockServiceClient
	BlockResultsServiceClient

	// Close the connection to the server. Any subsequent requests will fail.
	Close() error
}

type clientBuilder struct {
	dialerFunc func(context.Context, string) (net.Conn, error)
	grpcOpts   []ggrpc.DialOption

	versionServiceEnabled      bool
	blockServiceEnabled        bool
	blockResultsServiceEnabled bool
}

func newClientBuilder() *clientBuilder {
	return &clientBuilder{
		dialerFunc:                 defaultDialerFunc,
		grpcOpts:                   make([]ggrpc.DialOption, 0),
		versionServiceEnabled:      true,
		blockServiceEnabled:        true,
		blockResultsServiceEnabled: true,
	}
}

func defaultDialerFunc(ctx context.Context, addr string) (net.Conn, error) {
	return cmtnet.ConnectContext(ctx, addr)
}

type client struct {
	conn *ggrpc.ClientConn

	VersionServiceClient
	BlockServiceClient
	BlockResultsServiceClient
}

// Close implements Client.
func (c *client) Close() error {
	return c.conn.Close()
}

// WithInsecure disables transport security for the underlying client
// connection.
//
// A shortcut for using grpc.WithTransportCredentials and
// insecure.NewCredentials from google.golang.org/grpc.
func WithInsecure() Option {
	return WithGRPCDialOption(ggrpc.WithTransportCredentials(insecure.NewCredentials()))
}

// WithVersionServiceEnabled allows control of whether or not to create a
// client for interacting with the version service of a CometBFT node.
//
// If disabled and the client attempts to access the version service API, the
// client will panic.
func WithVersionServiceEnabled(enabled bool) Option {
	return func(b *clientBuilder) {
		b.versionServiceEnabled = enabled
	}
}

// WithBlockServiceEnabled allows control of whether or not to create a
// client for interacting with the block service of a CometBFT node.
//
// If disabled and the client attempts to access the block service API, the
// client will panic.
func WithBlockServiceEnabled(enabled bool) Option {
	return func(b *clientBuilder) {
		b.blockServiceEnabled = enabled
	}
}

// WithBlockResultsServiceEnabled allows control of whether or not to create a
// client for interacting with the block results service of a CometBFT node.
//
// If disabled and the client attempts to access the block results service API, the
// client will panic.
func WithBlockResultsServiceEnabled(enabled bool) Option {
	return func(b *clientBuilder) {
		b.blockResultsServiceEnabled = enabled
	}
}

// WithGRPCDialOption allows passing lower-level gRPC dial options through to
// the gRPC dialer when creating the client.
func WithGRPCDialOption(opt ggrpc.DialOption) Option {
	return func(b *clientBuilder) {
		b.grpcOpts = append(b.grpcOpts, opt)
	}
}

// New constructs a client for interacting with a CometBFT node via gRPC.
//
// Makes no assumptions about whether or not to use TLS to connect to the given
// address. To connect to a gRPC server without using TLS, use the WithInsecure
// option.
//
// To connect to a gRPC server with TLS, use the WithGRPCDialOption option with
// the appropriate gRPC credentials configuration. See
// https://pkg.go.dev/google.golang.org/grpc#WithTransportCredentials
func New(_ context.Context, addr string, opts ...Option) (Client, error) {
	builder := newClientBuilder()
	for _, opt := range opts {
		opt(builder)
	}
	conn, err := ggrpc.NewClient(addr, builder.grpcOpts...)
	if err != nil {
		return nil, fmt.Errorf("failed to dial %s: %w", addr, err)
	}
	versionServiceClient := newDisabledVersionServiceClient()
	if builder.versionServiceEnabled {
		versionServiceClient = newVersionServiceClient(conn)
	}
	blockServiceClient := newDisabledBlockServiceClient()
	if builder.blockServiceEnabled {
		blockServiceClient = newBlockServiceClient(conn)
	}
	blockResultServiceClient := newDisabledBlockResultsServiceClient()
	if builder.blockResultsServiceEnabled {
		blockResultServiceClient = newBlockResultsServiceClient(conn)
	}
	return &client{
		conn:                      conn,
		VersionServiceClient:      versionServiceClient,
		BlockServiceClient:        blockServiceClient,
		BlockResultsServiceClient: blockResultServiceClient,
	}, nil
}
