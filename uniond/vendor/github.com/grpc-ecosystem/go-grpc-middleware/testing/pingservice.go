// Copyright 2016 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

/*
Package `grpc_testing` provides helper functions for testing validators in this package.
*/

package grpc_testing

import (
	"context"
	"io"
	"testing"

	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

const (
	// DefaultPongValue is the default value used.
	DefaultResponseValue = "default_response_value"
	// ListResponseCount is the expected number of responses to PingList
	ListResponseCount = 100
)

type TestPingService struct {
	T *testing.T
}

func (s *TestPingService) PingEmpty(ctx context.Context, _ *pb_testproto.Empty) (*pb_testproto.PingResponse, error) {
	return &pb_testproto.PingResponse{Value: DefaultResponseValue, Counter: 42}, nil
}

func (s *TestPingService) Ping(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.PingResponse, error) {
	// Send user trailers and headers.
	return &pb_testproto.PingResponse{Value: ping.Value, Counter: 42}, nil
}

func (s *TestPingService) PingError(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.Empty, error) {
	code := codes.Code(ping.ErrorCodeReturned)
	return nil, status.Errorf(code, "Userspace error.")
}

func (s *TestPingService) PingList(ping *pb_testproto.PingRequest, stream pb_testproto.TestService_PingListServer) error {
	if ping.ErrorCodeReturned != 0 {
		return status.Errorf(codes.Code(ping.ErrorCodeReturned), "foobar")
	}
	// Send user trailers and headers.
	for i := 0; i < ListResponseCount; i++ {
		stream.Send(&pb_testproto.PingResponse{Value: ping.Value, Counter: int32(i)})
	}
	return nil
}

func (s *TestPingService) PingStream(stream pb_testproto.TestService_PingStreamServer) error {
	count := 0
	for true {
		ping, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}
		stream.Send(&pb_testproto.PingResponse{Value: ping.Value, Counter: int32(count)})
		count += 1
	}
	return nil
}
