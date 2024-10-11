package test

import (
	"context"

	"github.com/go-kit/kit/transport/grpc/_grpc_test/pb"
)

func encodeRequest(ctx context.Context, req interface{}) (interface{}, error) {
	r := req.(TestRequest)
	return &pb.TestRequest{A: r.A, B: r.B}, nil
}

func decodeRequest(ctx context.Context, req interface{}) (interface{}, error) {
	r := req.(*pb.TestRequest)
	return TestRequest{A: r.A, B: r.B}, nil
}

func encodeResponse(ctx context.Context, resp interface{}) (interface{}, error) {
	r := resp.(*TestResponse)
	return &pb.TestResponse{V: r.V}, nil
}

func decodeResponse(ctx context.Context, resp interface{}) (interface{}, error) {
	r := resp.(*pb.TestResponse)
	return &TestResponse{V: r.V, Ctx: ctx}, nil
}
