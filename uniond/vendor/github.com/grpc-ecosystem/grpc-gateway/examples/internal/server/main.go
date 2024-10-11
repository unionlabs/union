package server

import (
	"context"
	"net"
	"net/http"

	"github.com/golang/glog"
	examples "github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/examplepb"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc"
)

// Run starts the example gRPC service.
// "network" and "address" are passed to net.Listen.
func Run(ctx context.Context, network, address string) error {
	l, err := net.Listen(network, address)
	if err != nil {
		return err
	}
	defer func() {
		if err := l.Close(); err != nil {
			glog.Errorf("Failed to close %s %s: %v", network, address, err)
		}
	}()

	s := grpc.NewServer()
	examples.RegisterEchoServiceServer(s, newEchoServer())
	examples.RegisterFlowCombinationServer(s, newFlowCombinationServer())
	examples.RegisterNonStandardServiceServer(s, newNonStandardServer())

	abe := newABitOfEverythingServer()
	examples.RegisterABitOfEverythingServiceServer(s, abe)
	examples.RegisterStreamServiceServer(s, abe)
	examples.RegisterResponseBodyServiceServer(s, newResponseBodyServer())

	go func() {
		defer s.GracefulStop()
		<-ctx.Done()
	}()
	return s.Serve(l)
}

// RunInProcessGateway starts the invoke in process http gateway.
func RunInProcessGateway(ctx context.Context, addr string, opts ...runtime.ServeMuxOption) error {
	mux := runtime.NewServeMux(opts...)

	examples.RegisterEchoServiceHandlerServer(ctx, mux, newEchoServer())
	examples.RegisterFlowCombinationHandlerServer(ctx, mux, newFlowCombinationServer())
	examples.RegisterNonStandardServiceHandlerServer(ctx, mux, newNonStandardServer())

	abe := newABitOfEverythingServer()
	examples.RegisterABitOfEverythingServiceHandlerServer(ctx, mux, abe)
	examples.RegisterStreamServiceHandlerServer(ctx, mux, abe)
	examples.RegisterResponseBodyServiceHandlerServer(ctx, mux, newResponseBodyServer())

	s := &http.Server{
		Addr:    addr,
		Handler: mux,
	}

	go func() {
		<-ctx.Done()
		glog.Infof("Shutting down the http gateway server")
		if err := s.Shutdown(context.Background()); err != nil {
			glog.Errorf("Failed to shutdown http gateway server: %v", err)
		}
	}()

	if err := s.ListenAndServe(); err != http.ErrServerClosed {
		glog.Errorf("Failed to listen and serve: %v", err)
		return err
	}
	return nil

}
