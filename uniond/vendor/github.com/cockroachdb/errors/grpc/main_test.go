package grpc

import (
	"net"
	"os"
	"testing"
	"time"

	"google.golang.org/grpc"

	"github.com/cockroachdb/errors/grpc/middleware"
	"github.com/hydrogen18/memlistener"
)

var (
	Client EchoerClient
)

func TestMain(m *testing.M) {

	srv := &EchoServer{}

	lis := memlistener.NewMemoryListener()

	grpcServer := grpc.NewServer(grpc.UnaryInterceptor(middleware.UnaryServerInterceptor))
	RegisterEchoerServer(grpcServer, srv)

	go grpcServer.Serve(lis)

	dialOpts := []grpc.DialOption{
		grpc.WithDialer(func(target string, d time.Duration) (net.Conn, error) {
			return lis.Dial("", "")
		}),
		grpc.WithInsecure(),
		grpc.WithUnaryInterceptor(middleware.UnaryClientInterceptor),
	}

	clientConn, err := grpc.Dial("", dialOpts...)
	if err != nil {
		panic(err)
	}

	Client = NewEchoerClient(clientConn)

	code := m.Run()

	grpcServer.Stop()
	clientConn.Close()

	os.Exit(code)
}
