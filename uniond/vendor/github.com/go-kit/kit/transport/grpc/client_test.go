package grpc_test

import (
	"context"
	"fmt"
	"net"
	"testing"

	"google.golang.org/grpc"

	test "github.com/go-kit/kit/transport/grpc/_grpc_test"
	"github.com/go-kit/kit/transport/grpc/_grpc_test/pb"
)

const (
	hostPort string = "localhost:8002"
)

func TestGRPCClient(t *testing.T) {
	var (
		server  = grpc.NewServer()
		service = test.NewService()
	)

	sc, err := net.Listen("tcp", hostPort)
	if err != nil {
		t.Fatalf("unable to listen: %+v", err)
	}
	defer server.GracefulStop()

	go func() {
		pb.RegisterTestServer(server, test.NewBinding(service))
		_ = server.Serve(sc)
	}()

	cc, err := grpc.Dial(hostPort, grpc.WithInsecure())
	if err != nil {
		t.Fatalf("unable to Dial: %+v", err)
	}

	client := test.NewClient(cc)

	var (
		a   = "the answer to life the universe and everything"
		b   = int64(42)
		cID = "request-1"
		ctx = test.SetCorrelationID(context.Background(), cID)
	)

	responseCTX, v, err := client.Test(ctx, a, b)
	if err != nil {
		t.Fatalf("unable to Test: %+v", err)
	}
	if want, have := fmt.Sprintf("%s = %d", a, b), v; want != have {
		t.Fatalf("want %q, have %q", want, have)
	}

	if want, have := cID, test.GetConsumedCorrelationID(responseCTX); want != have {
		t.Fatalf("want %q, have %q", want, have)
	}
}
