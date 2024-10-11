package server

import (
	"context"

	"github.com/golang/glog"
	examples "github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/examplepb"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"
)

// Implements of UnannotatedEchoServiceServer

type unannotatedEchoServer struct{}

func newUnannotatedEchoServer() examples.UnannotatedEchoServiceServer {
	return new(unannotatedEchoServer)
}

func (s *unannotatedEchoServer) Echo(ctx context.Context, msg *examples.UnannotatedSimpleMessage) (*examples.UnannotatedSimpleMessage, error) {
	glog.Info(msg)
	return msg, nil
}

func (s *unannotatedEchoServer) EchoBody(ctx context.Context, msg *examples.UnannotatedSimpleMessage) (*examples.UnannotatedSimpleMessage, error) {
	glog.Info(msg)
	grpc.SendHeader(ctx, metadata.New(map[string]string{
		"foo": "foo1",
		"bar": "bar1",
	}))
	grpc.SetTrailer(ctx, metadata.New(map[string]string{
		"foo": "foo2",
		"bar": "bar2",
	}))
	return msg, nil
}

func (s *unannotatedEchoServer) EchoDelete(ctx context.Context, msg *examples.UnannotatedSimpleMessage) (*examples.UnannotatedSimpleMessage, error) {
	glog.Info(msg)
	return msg, nil
}
