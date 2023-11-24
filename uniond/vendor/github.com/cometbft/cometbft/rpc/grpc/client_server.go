package coregrpc

import (
	"net"

	"golang.org/x/net/context"
	"google.golang.org/grpc"

	cmtnet "github.com/cometbft/cometbft/libs/net"
	"github.com/cometbft/cometbft/rpc/core"
)

// Config is an gRPC server configuration.
//
// Deprecated: A new gRPC API will be introduced after v0.38.
type Config struct {
	MaxOpenConnections int
}

// StartGRPCServer starts a new gRPC BroadcastAPIServer using the given
// net.Listener.
// NOTE: This function blocks - you may want to call it in a go-routine.
//
// Deprecated: A new gRPC API will be introduced after v0.38.
func StartGRPCServer(env *core.Environment, ln net.Listener) error {
	grpcServer := grpc.NewServer()
	RegisterBroadcastAPIServer(grpcServer, &broadcastAPI{env: env})
	return grpcServer.Serve(ln)
}

// StartGRPCClient dials the gRPC server using protoAddr and returns a new
// BroadcastAPIClient.
//
// Deprecated: A new gRPC API will be introduced after v0.38.
func StartGRPCClient(protoAddr string) BroadcastAPIClient {
	conn, err := grpc.Dial(protoAddr, grpc.WithInsecure(), grpc.WithContextDialer(dialerFunc))
	if err != nil {
		panic(err)
	}
	return NewBroadcastAPIClient(conn)
}

func dialerFunc(_ context.Context, addr string) (net.Conn, error) {
	return cmtnet.Connect(addr)
}
