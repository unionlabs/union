package grpc

import (
	"context"
	grpc "google.golang.org/grpc"
)

type Server interface {
	RegisterService(sd *grpc.ServiceDesc, ss interface{})
}

type ClientConn interface {
	Invoke(ctx context.Context, method string, args, reply interface{}, opts ...grpc.CallOption) error
	NewStream(ctx context.Context, desc *grpc.StreamDesc, method string, opts ...grpc.CallOption) (grpc.ClientStream, error)
}
