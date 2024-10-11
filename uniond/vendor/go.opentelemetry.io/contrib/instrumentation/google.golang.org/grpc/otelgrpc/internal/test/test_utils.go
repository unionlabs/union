// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/*
 *
 * Copyright 2014 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Package test contains functions used by interop client/server.
//
// Copied from https://github.com/grpc/grpc-go/tree/v1.61.0/interop
// That package was not intended to be used by external code.
// See https://github.com/open-telemetry/opentelemetry-go-contrib/issues/4896
package test // import "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/internal/test"

import (
	"context"
	"fmt"
	"io"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var (
	reqSizes            = []int{27182, 8, 1828, 45904}
	respSizes           = []int{31415, 9, 2653, 58979}
	largeReqSize        = 271828
	largeRespSize       = 314159
	initialMetadataKey  = "x-grpc-test-echo-initial"
	trailingMetadataKey = "x-grpc-test-echo-trailing-bin"

	logger = grpclog.Component("interop")
)

// ClientNewPayload returns a payload of the given type and size.
func ClientNewPayload(t testpb.PayloadType, size int) *testpb.Payload {
	if size < 0 {
		logger.Fatalf("Requested a response with invalid length %d", size)
	}
	body := make([]byte, size)
	switch t {
	case testpb.PayloadType_COMPRESSABLE:
	default:
		logger.Fatalf("Unsupported payload type: %d", t)
	}
	return &testpb.Payload{
		Type: t,
		Body: body,
	}
}

// DoEmptyUnaryCall performs a unary RPC with empty request and response messages.
func DoEmptyUnaryCall(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	reply, err := tc.EmptyCall(ctx, &testpb.Empty{}, args...)
	if err != nil {
		logger.Fatal("/TestService/EmptyCall RPC failed: ", err)
	}
	if !proto.Equal(&testpb.Empty{}, reply) {
		logger.Fatalf("/TestService/EmptyCall receives %v, want %v", reply, testpb.Empty{})
	}
}

// DoLargeUnaryCall performs a unary RPC with large payload in the request and response.
func DoLargeUnaryCall(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(largeRespSize),
		Payload:      pl,
	}
	reply, err := tc.UnaryCall(ctx, req, args...)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	t := reply.GetPayload().GetType()
	s := len(reply.GetPayload().GetBody())
	if t != testpb.PayloadType_COMPRESSABLE || s != largeRespSize {
		logger.Fatalf("Got the reply with type %d len %d; want %d, %d", t, s, testpb.PayloadType_COMPRESSABLE, largeRespSize)
	}
}

// DoClientStreaming performs a client streaming RPC.
func DoClientStreaming(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	stream, err := tc.StreamingInputCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.StreamingInputCall(_) = _, %v", tc, err)
	}
	var sum int
	for _, s := range reqSizes {
		pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, s)
		req := &testpb.StreamingInputCallRequest{
			Payload: pl,
		}
		if err := stream.Send(req); err != nil {
			logger.Fatalf("%v has error %v while sending %v", stream, err, req)
		}
		sum += s
	}
	reply, err := stream.CloseAndRecv()
	if err != nil {
		logger.Fatalf("%v.CloseAndRecv() got error %v, want %v", stream, err, nil)
	}
	if reply.GetAggregatedPayloadSize() != int32(sum) {
		logger.Fatalf("%v.CloseAndRecv().GetAggregatePayloadSize() = %v; want %v", stream, reply.GetAggregatedPayloadSize(), sum)
	}
}

// DoServerStreaming performs a server streaming RPC.
func DoServerStreaming(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	respParam := make([]*testpb.ResponseParameters, len(respSizes))
	for i, s := range respSizes {
		respParam[i] = &testpb.ResponseParameters{
			Size: int32(s),
		}
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
	}
	stream, err := tc.StreamingOutputCall(ctx, req, args...)
	if err != nil {
		logger.Fatalf("%v.StreamingOutputCall(_) = _, %v", tc, err)
	}
	var rpcStatus error
	var respCnt int
	var index int
	for {
		reply, err := stream.Recv()
		if err != nil {
			rpcStatus = err
			break
		}
		t := reply.GetPayload().GetType()
		if t != testpb.PayloadType_COMPRESSABLE {
			logger.Fatalf("Got the reply of type %d, want %d", t, testpb.PayloadType_COMPRESSABLE)
		}
		size := len(reply.GetPayload().GetBody())
		if size != respSizes[index] {
			logger.Fatalf("Got reply body of length %d, want %d", size, respSizes[index])
		}
		index++
		respCnt++
	}
	if rpcStatus != io.EOF {
		logger.Fatalf("Failed to finish the server streaming rpc: %v", rpcStatus)
	}
	if respCnt != len(respSizes) {
		logger.Fatalf("Got %d reply, want %d", len(respSizes), respCnt)
	}
}

// DoPingPong performs ping-pong style bi-directional streaming RPC.
func DoPingPong(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v", tc, err)
	}
	var index int
	for index < len(reqSizes) {
		respParam := []*testpb.ResponseParameters{
			{
				Size: int32(respSizes[index]),
			},
		}
		pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, reqSizes[index])
		req := &testpb.StreamingOutputCallRequest{
			ResponseType:       testpb.PayloadType_COMPRESSABLE,
			ResponseParameters: respParam,
			Payload:            pl,
		}
		if err := stream.Send(req); err != nil {
			logger.Fatalf("%v has error %v while sending %v", stream, err, req)
		}
		reply, err := stream.Recv()
		if err != nil {
			logger.Fatalf("%v.Recv() = %v", stream, err)
		}
		t := reply.GetPayload().GetType()
		if t != testpb.PayloadType_COMPRESSABLE {
			logger.Fatalf("Got the reply of type %d, want %d", t, testpb.PayloadType_COMPRESSABLE)
		}
		size := len(reply.GetPayload().GetBody())
		if size != respSizes[index] {
			logger.Fatalf("Got reply body of length %d, want %d", size, respSizes[index])
		}
		index++
	}
	if err := stream.CloseSend(); err != nil {
		logger.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		logger.Fatalf("%v failed to complele the ping pong test: %v", stream, err)
	}
}

// DoEmptyStream sets up a bi-directional streaming with zero message.
func DoEmptyStream(ctx context.Context, tc testpb.TestServiceClient, args ...grpc.CallOption) {
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v", tc, err)
	}
	if err := stream.CloseSend(); err != nil {
		logger.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		logger.Fatalf("%v failed to complete the empty stream test: %v", stream, err)
	}
}

type testServer struct {
	testpb.UnimplementedTestServiceServer
}

// NewTestServer creates a test server for test service.  opts carries optional
// settings and does not need to be provided.  If multiple opts are provided,
// only the first one is used.
func NewTestServer() testpb.TestServiceServer {
	return &testServer{}
}

func (s *testServer) EmptyCall(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
	return new(testpb.Empty), nil
}

func serverNewPayload(t testpb.PayloadType, size int32) (*testpb.Payload, error) {
	if size < 0 {
		return nil, fmt.Errorf("requested a response with invalid length %d", size)
	}
	body := make([]byte, size)
	switch t {
	case testpb.PayloadType_COMPRESSABLE:
	default:
		return nil, fmt.Errorf("unsupported payload type: %d", t)
	}
	return &testpb.Payload{
		Type: t,
		Body: body,
	}, nil
}

func (s *testServer) UnaryCall(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	st := in.GetResponseStatus()
	if md, ok := metadata.FromIncomingContext(ctx); ok {
		if initialMetadata, ok := md[initialMetadataKey]; ok {
			header := metadata.Pairs(initialMetadataKey, initialMetadata[0])
			_ = grpc.SendHeader(ctx, header)
		}
		if trailingMetadata, ok := md[trailingMetadataKey]; ok {
			trailer := metadata.Pairs(trailingMetadataKey, trailingMetadata[0])
			_ = grpc.SetTrailer(ctx, trailer)
		}
	}
	if st != nil && st.Code != 0 {
		return nil, status.Error(codes.Code(st.Code), st.Message)
	}
	pl, err := serverNewPayload(in.GetResponseType(), in.GetResponseSize())
	if err != nil {
		return nil, err
	}
	return &testpb.SimpleResponse{
		Payload: pl,
	}, nil
}

func (s *testServer) StreamingOutputCall(args *testpb.StreamingOutputCallRequest, stream testpb.TestService_StreamingOutputCallServer) error {
	cs := args.GetResponseParameters()
	for _, c := range cs {
		if us := c.GetIntervalUs(); us > 0 {
			time.Sleep(time.Duration(us) * time.Microsecond)
		}
		pl, err := serverNewPayload(args.GetResponseType(), c.GetSize())
		if err != nil {
			return err
		}
		if err := stream.Send(&testpb.StreamingOutputCallResponse{
			Payload: pl,
		}); err != nil {
			return err
		}
	}
	return nil
}

func (s *testServer) StreamingInputCall(stream testpb.TestService_StreamingInputCallServer) error {
	var sum int
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			return stream.SendAndClose(&testpb.StreamingInputCallResponse{
				AggregatedPayloadSize: int32(sum),
			})
		}
		if err != nil {
			return err
		}
		p := in.GetPayload().GetBody()
		sum += len(p)
	}
}

func (s *testServer) FullDuplexCall(stream testpb.TestService_FullDuplexCallServer) error {
	if md, ok := metadata.FromIncomingContext(stream.Context()); ok {
		if initialMetadata, ok := md[initialMetadataKey]; ok {
			header := metadata.Pairs(initialMetadataKey, initialMetadata[0])
			_ = stream.SendHeader(header)
		}
		if trailingMetadata, ok := md[trailingMetadataKey]; ok {
			trailer := metadata.Pairs(trailingMetadataKey, trailingMetadata[0])
			stream.SetTrailer(trailer)
		}
	}
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			return nil
		}
		if err != nil {
			return err
		}
		st := in.GetResponseStatus()
		if st != nil && st.Code != 0 {
			return status.Error(codes.Code(st.Code), st.Message)
		}

		cs := in.GetResponseParameters()
		for _, c := range cs {
			if us := c.GetIntervalUs(); us > 0 {
				time.Sleep(time.Duration(us) * time.Microsecond)
			}
			pl, err := serverNewPayload(in.GetResponseType(), c.GetSize())
			if err != nil {
				return err
			}
			if err := stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: pl,
			}); err != nil {
				return err
			}
		}
	}
}

func (s *testServer) HalfDuplexCall(stream testpb.TestService_HalfDuplexCallServer) error {
	var msgBuf []*testpb.StreamingOutputCallRequest
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			break
		}
		if err != nil {
			return err
		}
		msgBuf = append(msgBuf, in)
	}
	for _, m := range msgBuf {
		cs := m.GetResponseParameters()
		for _, c := range cs {
			if us := c.GetIntervalUs(); us > 0 {
				time.Sleep(time.Duration(us) * time.Microsecond)
			}
			pl, err := serverNewPayload(m.GetResponseType(), c.GetSize())
			if err != nil {
				return err
			}
			if err := stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: pl,
			}); err != nil {
				return err
			}
		}
	}
	return nil
}
