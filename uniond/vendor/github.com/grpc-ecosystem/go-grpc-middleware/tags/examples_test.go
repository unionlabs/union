package grpc_ctxtags_test

import (
	"github.com/grpc-ecosystem/go-grpc-middleware/tags"
	"google.golang.org/grpc"
)

// Simple example of server initialization code, with data automatically populated from `log_fields` Golang tags.
func Example_initialization() {
	opts := []grpc_ctxtags.Option{
		grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.TagBasedRequestFieldExtractor("log_fields")),
	}
	_ = grpc.NewServer(
		grpc.StreamInterceptor(grpc_ctxtags.StreamServerInterceptor(opts...)),
		grpc.UnaryInterceptor(grpc_ctxtags.UnaryServerInterceptor(opts...)),
	)
}

// Example using WithFieldExtractorForInitialReq
func Example_initialisationWithOptions() {
	opts := []grpc_ctxtags.Option{
		grpc_ctxtags.WithFieldExtractorForInitialReq(grpc_ctxtags.TagBasedRequestFieldExtractor("log_fields")),
	}
	_ = grpc.NewServer(
		grpc.StreamInterceptor(grpc_ctxtags.StreamServerInterceptor(opts...)),
		grpc.UnaryInterceptor(grpc_ctxtags.UnaryServerInterceptor(opts...)),
	)
}
