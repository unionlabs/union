package server

import (
	"context"
	"net"

	"google.golang.org/grpc"

	"github.com/cometbft/cometbft/abci/types"
	cmtnet "github.com/cometbft/cometbft/libs/net"
	"github.com/cometbft/cometbft/libs/service"
)

type GRPCServer struct {
	service.BaseService

	proto    string
	addr     string
	listener net.Listener
	server   *grpc.Server

	app types.Application
}

// NewGRPCServer returns a new gRPC ABCI server
func NewGRPCServer(protoAddr string, app types.Application) service.Service {
	proto, addr := cmtnet.ProtocolAndAddress(protoAddr)
	s := &GRPCServer{
		proto:    proto,
		addr:     addr,
		listener: nil,
		app:      app,
	}
	s.BaseService = *service.NewBaseService(nil, "ABCIServer", s)
	return s
}

// OnStart starts the gRPC service.
func (s *GRPCServer) OnStart() error {
	ln, err := net.Listen(s.proto, s.addr)
	if err != nil {
		return err
	}

	s.listener = ln
	s.server = grpc.NewServer()
	types.RegisterABCIServer(s.server, &gRPCApplication{s.app})

	s.Logger.Info("Listening", "proto", s.proto, "addr", s.addr)
	go func() {
		if err := s.server.Serve(s.listener); err != nil {
			s.Logger.Error("Error serving gRPC server", "err", err)
		}
	}()
	return nil
}

// OnStop stops the gRPC server.
func (s *GRPCServer) OnStop() {
	s.server.Stop()
}

//-------------------------------------------------------

// gRPCApplication is a gRPC shim for Application
type gRPCApplication struct {
	types.Application
}

func (app *gRPCApplication) Echo(_ context.Context, req *types.RequestEcho) (*types.ResponseEcho, error) {
	return &types.ResponseEcho{Message: req.Message}, nil
}

func (app *gRPCApplication) Flush(context.Context, *types.RequestFlush) (*types.ResponseFlush, error) {
	return &types.ResponseFlush{}, nil
}
