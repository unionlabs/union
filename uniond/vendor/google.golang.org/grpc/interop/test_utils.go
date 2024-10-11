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

// Package interop contains functions used by interop client/server.
//
// See interop test case descriptions [here].
//
// [here]: https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md
package interop

import (
	"bytes"
	"context"
	"fmt"
	"io"
	"os"
	"strings"
	"sync"
	"time"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"google.golang.org/grpc"
	"google.golang.org/grpc/benchmark/stats"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/orca"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	v3orcapb "github.com/cncf/xds/go/xds/data/orca/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
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
func DoEmptyUnaryCall(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	reply, err := tc.EmptyCall(ctx, &testpb.Empty{}, args...)
	if err != nil {
		logger.Fatal("/TestService/EmptyCall RPC failed: ", err)
	}
	if !proto.Equal(&testpb.Empty{}, reply) {
		logger.Fatalf("/TestService/EmptyCall receives %v, want %v", reply, testpb.Empty{})
	}
}

// DoLargeUnaryCall performs a unary RPC with large payload in the request and response.
func DoLargeUnaryCall(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
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
func DoClientStreaming(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
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
func DoServerStreaming(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
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
func DoPingPong(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
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
func DoEmptyStream(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
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

// DoTimeoutOnSleepingServer performs an RPC on a sleep server which causes RPC timeout.
func DoTimeoutOnSleepingServer(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	ctx, cancel := context.WithTimeout(ctx, 1*time.Millisecond)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		if status.Code(err) == codes.DeadlineExceeded {
			return
		}
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v", tc, err)
	}
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, 27182)
	req := &testpb.StreamingOutputCallRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		Payload:      pl,
	}
	if err := stream.Send(req); err != nil && err != io.EOF {
		logger.Fatalf("%v.Send(_) = %v", stream, err)
	}
	if _, err := stream.Recv(); status.Code(err) != codes.DeadlineExceeded {
		logger.Fatalf("%v.Recv() = _, %v, want error code %d", stream, err, codes.DeadlineExceeded)
	}
}

// DoComputeEngineCreds performs a unary RPC with compute engine auth.
func DoComputeEngineCreds(ctx context.Context, tc testgrpc.TestServiceClient, serviceAccount, oauthScope string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	user := reply.GetUsername()
	scope := reply.GetOauthScope()
	if user != serviceAccount {
		logger.Fatalf("Got user name %q, want %q.", user, serviceAccount)
	}
	if !strings.Contains(oauthScope, scope) {
		logger.Fatalf("Got OAuth scope %q which is NOT a substring of %q.", scope, oauthScope)
	}
}

func getServiceAccountJSONKey(keyFile string) []byte {
	jsonKey, err := os.ReadFile(keyFile)
	if err != nil {
		logger.Fatalf("Failed to read the service account key file: %v", err)
	}
	return jsonKey
}

// DoServiceAccountCreds performs a unary RPC with service account auth.
func DoServiceAccountCreds(ctx context.Context, tc testgrpc.TestServiceClient, serviceAccountKeyFile, oauthScope string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	jsonKey := getServiceAccountJSONKey(serviceAccountKeyFile)
	user := reply.GetUsername()
	scope := reply.GetOauthScope()
	if !strings.Contains(string(jsonKey), user) {
		logger.Fatalf("Got user name %q which is NOT a substring of %q.", user, jsonKey)
	}
	if !strings.Contains(oauthScope, scope) {
		logger.Fatalf("Got OAuth scope %q which is NOT a substring of %q.", scope, oauthScope)
	}
}

// DoJWTTokenCreds performs a unary RPC with JWT token auth.
func DoJWTTokenCreds(ctx context.Context, tc testgrpc.TestServiceClient, serviceAccountKeyFile string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(largeRespSize),
		Payload:      pl,
		FillUsername: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	jsonKey := getServiceAccountJSONKey(serviceAccountKeyFile)
	user := reply.GetUsername()
	if !strings.Contains(string(jsonKey), user) {
		logger.Fatalf("Got user name %q which is NOT a substring of %q.", user, jsonKey)
	}
}

// GetToken obtains an OAUTH token from the input.
func GetToken(ctx context.Context, serviceAccountKeyFile string, oauthScope string) *oauth2.Token {
	jsonKey := getServiceAccountJSONKey(serviceAccountKeyFile)
	config, err := google.JWTConfigFromJSON(jsonKey, oauthScope)
	if err != nil {
		logger.Fatalf("Failed to get the config: %v", err)
	}
	token, err := config.TokenSource(ctx).Token()
	if err != nil {
		logger.Fatalf("Failed to get the token: %v", err)
	}
	return token
}

// DoOauth2TokenCreds performs a unary RPC with OAUTH2 token auth.
func DoOauth2TokenCreds(ctx context.Context, tc testgrpc.TestServiceClient, serviceAccountKeyFile, oauthScope string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	jsonKey := getServiceAccountJSONKey(serviceAccountKeyFile)
	user := reply.GetUsername()
	scope := reply.GetOauthScope()
	if !strings.Contains(string(jsonKey), user) {
		logger.Fatalf("Got user name %q which is NOT a substring of %q.", user, jsonKey)
	}
	if !strings.Contains(oauthScope, scope) {
		logger.Fatalf("Got OAuth scope %q which is NOT a substring of %q.", scope, oauthScope)
	}
}

// DoPerRPCCreds performs a unary RPC with per RPC OAUTH2 token.
func DoPerRPCCreds(ctx context.Context, tc testgrpc.TestServiceClient, serviceAccountKeyFile, oauthScope string) {
	jsonKey := getServiceAccountJSONKey(serviceAccountKeyFile)
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	token := GetToken(ctx, serviceAccountKeyFile, oauthScope)
	kv := map[string]string{"authorization": token.Type() + " " + token.AccessToken}
	ctx = metadata.NewOutgoingContext(ctx, metadata.MD{"authorization": []string{kv["authorization"]}})
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	user := reply.GetUsername()
	scope := reply.GetOauthScope()
	if !strings.Contains(string(jsonKey), user) {
		logger.Fatalf("Got user name %q which is NOT a substring of %q.", user, jsonKey)
	}
	if !strings.Contains(oauthScope, scope) {
		logger.Fatalf("Got OAuth scope %q which is NOT a substring of %q.", scope, oauthScope)
	}
}

// DoGoogleDefaultCredentials performs an unary RPC with google default credentials
func DoGoogleDefaultCredentials(ctx context.Context, tc testgrpc.TestServiceClient, defaultServiceAccount string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	if reply.GetUsername() != defaultServiceAccount {
		logger.Fatalf("Got user name %q; wanted %q. ", reply.GetUsername(), defaultServiceAccount)
	}
}

// DoComputeEngineChannelCredentials performs an unary RPC with compute engine channel credentials
func DoComputeEngineChannelCredentials(ctx context.Context, tc testgrpc.TestServiceClient, defaultServiceAccount string) {
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, largeReqSize)
	req := &testpb.SimpleRequest{
		ResponseType:   testpb.PayloadType_COMPRESSABLE,
		ResponseSize:   int32(largeRespSize),
		Payload:        pl,
		FillUsername:   true,
		FillOauthScope: true,
	}
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	if reply.GetUsername() != defaultServiceAccount {
		logger.Fatalf("Got user name %q; wanted %q. ", reply.GetUsername(), defaultServiceAccount)
	}
}

var testMetadata = metadata.MD{
	"key1": []string{"value1"},
	"key2": []string{"value2"},
}

// DoCancelAfterBegin cancels the RPC after metadata has been sent but before payloads are sent.
func DoCancelAfterBegin(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	ctx, cancel := context.WithCancel(metadata.NewOutgoingContext(ctx, testMetadata))
	stream, err := tc.StreamingInputCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.StreamingInputCall(_) = _, %v", tc, err)
	}
	cancel()
	_, err = stream.CloseAndRecv()
	if status.Code(err) != codes.Canceled {
		logger.Fatalf("%v.CloseAndRecv() got error code %d, want %d", stream, status.Code(err), codes.Canceled)
	}
}

// DoCancelAfterFirstResponse cancels the RPC after receiving the first message from the server.
func DoCancelAfterFirstResponse(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	ctx, cancel := context.WithCancel(ctx)
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v", tc, err)
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 31415,
		},
	}
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, 27182)
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            pl,
	}
	if err := stream.Send(req); err != nil {
		logger.Fatalf("%v has error %v while sending %v", stream, err, req)
	}
	if _, err := stream.Recv(); err != nil {
		logger.Fatalf("%v.Recv() = %v", stream, err)
	}
	cancel()
	if _, err := stream.Recv(); status.Code(err) != codes.Canceled {
		logger.Fatalf("%v compleled with error code %d, want %d", stream, status.Code(err), codes.Canceled)
	}
}

var (
	initialMetadataValue  = "test_initial_metadata_value"
	trailingMetadataValue = "\x0a\x0b\x0a\x0b\x0a\x0b"
	customMetadata        = metadata.Pairs(
		initialMetadataKey, initialMetadataValue,
		trailingMetadataKey, trailingMetadataValue,
	)
)

func validateMetadata(header, trailer metadata.MD) {
	if len(header[initialMetadataKey]) != 1 {
		logger.Fatalf("Expected exactly one header from server. Received %d", len(header[initialMetadataKey]))
	}
	if header[initialMetadataKey][0] != initialMetadataValue {
		logger.Fatalf("Got header %s; want %s", header[initialMetadataKey][0], initialMetadataValue)
	}
	if len(trailer[trailingMetadataKey]) != 1 {
		logger.Fatalf("Expected exactly one trailer from server. Received %d", len(trailer[trailingMetadataKey]))
	}
	if trailer[trailingMetadataKey][0] != trailingMetadataValue {
		logger.Fatalf("Got trailer %s; want %s", trailer[trailingMetadataKey][0], trailingMetadataValue)
	}
}

// DoCustomMetadata checks that metadata is echoed back to the client.
func DoCustomMetadata(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	// Testing with UnaryCall.
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, 1)
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(1),
		Payload:      pl,
	}
	ctx = metadata.NewOutgoingContext(ctx, customMetadata)
	var header, trailer metadata.MD
	args = append(args, grpc.Header(&header), grpc.Trailer(&trailer))
	reply, err := tc.UnaryCall(
		ctx,
		req,
		args...,
	)
	if err != nil {
		logger.Fatal("/TestService/UnaryCall RPC failed: ", err)
	}
	t := reply.GetPayload().GetType()
	s := len(reply.GetPayload().GetBody())
	if t != testpb.PayloadType_COMPRESSABLE || s != 1 {
		logger.Fatalf("Got the reply with type %d len %d; want %d, %d", t, s, testpb.PayloadType_COMPRESSABLE, 1)
	}
	validateMetadata(header, trailer)

	// Testing with FullDuplex.
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}
	streamReq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            pl,
	}
	if err := stream.Send(streamReq); err != nil {
		logger.Fatalf("%v has error %v while sending %v", stream, err, streamReq)
	}
	streamHeader, err := stream.Header()
	if err != nil {
		logger.Fatalf("%v.Header() = %v", stream, err)
	}
	if _, err := stream.Recv(); err != nil {
		logger.Fatalf("%v.Recv() = %v", stream, err)
	}
	if err := stream.CloseSend(); err != nil {
		logger.Fatalf("%v.CloseSend() = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != io.EOF {
		logger.Fatalf("%v failed to complete the custom metadata test: %v", stream, err)
	}
	streamTrailer := stream.Trailer()
	validateMetadata(streamHeader, streamTrailer)
}

// DoStatusCodeAndMessage checks that the status code is propagated back to the client.
func DoStatusCodeAndMessage(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	var code int32 = 2
	msg := "test status message"
	expectedErr := status.Error(codes.Code(code), msg)
	respStatus := &testpb.EchoStatus{
		Code:    code,
		Message: msg,
	}
	// Test UnaryCall.
	req := &testpb.SimpleRequest{
		ResponseStatus: respStatus,
	}
	if _, err := tc.UnaryCall(ctx, req, args...); err == nil || err.Error() != expectedErr.Error() {
		logger.Fatalf("%v.UnaryCall(_, %v) = _, %v, want _, %v", tc, req, err, expectedErr)
	}
	// Test FullDuplexCall.
	stream, err := tc.FullDuplexCall(ctx, args...)
	if err != nil {
		logger.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	streamReq := &testpb.StreamingOutputCallRequest{
		ResponseStatus: respStatus,
	}
	if err := stream.Send(streamReq); err != nil {
		logger.Fatalf("%v has error %v while sending %v, want <nil>", stream, err, streamReq)
	}
	if err := stream.CloseSend(); err != nil {
		logger.Fatalf("%v.CloseSend() = %v, want <nil>", stream, err)
	}
	if _, err = stream.Recv(); err.Error() != expectedErr.Error() {
		logger.Fatalf("%v.Recv() returned error %v, want %v", stream, err, expectedErr)
	}
}

// DoSpecialStatusMessage verifies Unicode and whitespace is correctly processed
// in status message.
func DoSpecialStatusMessage(ctx context.Context, tc testgrpc.TestServiceClient, args ...grpc.CallOption) {
	const (
		code int32  = 2
		msg  string = "\t\ntest with whitespace\r\nand Unicode BMP ☺ and non-BMP 😈\t\n"
	)
	expectedErr := status.Error(codes.Code(code), msg)
	req := &testpb.SimpleRequest{
		ResponseStatus: &testpb.EchoStatus{
			Code:    code,
			Message: msg,
		},
	}
	ctx, cancel := context.WithTimeout(ctx, 10*time.Second)
	defer cancel()
	if _, err := tc.UnaryCall(ctx, req, args...); err == nil || err.Error() != expectedErr.Error() {
		logger.Fatalf("%v.UnaryCall(_, %v) = _, %v, want _, %v", tc, req, err, expectedErr)
	}
}

// DoUnimplementedService attempts to call a method from an unimplemented service.
func DoUnimplementedService(ctx context.Context, tc testgrpc.UnimplementedServiceClient) {
	_, err := tc.UnimplementedCall(ctx, &testpb.Empty{})
	if status.Code(err) != codes.Unimplemented {
		logger.Fatalf("%v.UnimplementedCall() = _, %v, want _, %v", tc, status.Code(err), codes.Unimplemented)
	}
}

// DoUnimplementedMethod attempts to call an unimplemented method.
func DoUnimplementedMethod(ctx context.Context, cc *grpc.ClientConn) {
	var req, reply proto.Message
	if err := cc.Invoke(ctx, "/grpc.testing.TestService/UnimplementedCall", req, reply); err == nil || status.Code(err) != codes.Unimplemented {
		logger.Fatalf("ClientConn.Invoke(_, _, _, _, _) = %v, want error code %s", err, codes.Unimplemented)
	}
}

// DoPickFirstUnary runs multiple RPCs (rpcCount) and checks that all requests
// are sent to the same backend.
func DoPickFirstUnary(ctx context.Context, tc testgrpc.TestServiceClient) {
	const rpcCount = 100

	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, 1)
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(1),
		Payload:      pl,
		FillServerId: true,
	}
	// TODO(mohanli): Revert the timeout back to 10s once TD migrates to xdstp.
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	var serverID string
	for i := 0; i < rpcCount; i++ {
		resp, err := tc.UnaryCall(ctx, req)
		if err != nil {
			logger.Fatalf("iteration %d, failed to do UnaryCall: %v", i, err)
		}
		id := resp.ServerId
		if id == "" {
			logger.Fatalf("iteration %d, got empty server ID", i)
		}
		if i == 0 {
			serverID = id
			continue
		}
		if serverID != id {
			logger.Fatalf("iteration %d, got different server ids: %q vs %q", i, serverID, id)
		}
	}
}

func doOneSoakIteration(ctx context.Context, tc testgrpc.TestServiceClient, resetChannel bool, serverAddr string, soakRequestSize int, soakResponseSize int, dopts []grpc.DialOption, copts []grpc.CallOption) (latency time.Duration, err error) {
	start := time.Now()
	client := tc
	if resetChannel {
		var conn *grpc.ClientConn
		conn, err = grpc.Dial(serverAddr, dopts...)
		if err != nil {
			return
		}
		defer conn.Close()
		client = testgrpc.NewTestServiceClient(conn)
	}
	// per test spec, don't include channel shutdown in latency measurement
	defer func() { latency = time.Since(start) }()
	// do a large-unary RPC
	pl := ClientNewPayload(testpb.PayloadType_COMPRESSABLE, soakRequestSize)
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(soakResponseSize),
		Payload:      pl,
	}
	var reply *testpb.SimpleResponse
	reply, err = client.UnaryCall(ctx, req, copts...)
	if err != nil {
		err = fmt.Errorf("/TestService/UnaryCall RPC failed: %s", err)
		return
	}
	t := reply.GetPayload().GetType()
	s := len(reply.GetPayload().GetBody())
	if t != testpb.PayloadType_COMPRESSABLE || s != soakResponseSize {
		err = fmt.Errorf("got the reply with type %d len %d; want %d, %d", t, s, testpb.PayloadType_COMPRESSABLE, soakResponseSize)
		return
	}
	return
}

// DoSoakTest runs large unary RPCs in a loop for a configurable number of times, with configurable failure thresholds.
// If resetChannel is false, then each RPC will be performed on tc. Otherwise, each RPC will be performed on a new
// stub that is created with the provided server address and dial options.
// TODO(mohanli-ml): Create SoakTestOptions as a parameter for this method.
func DoSoakTest(ctx context.Context, tc testgrpc.TestServiceClient, serverAddr string, dopts []grpc.DialOption, resetChannel bool, soakIterations int, maxFailures int, soakRequestSize int, soakResponseSize int, perIterationMaxAcceptableLatency time.Duration, minTimeBetweenRPCs time.Duration) {
	start := time.Now()
	var elapsedTime float64
	iterationsDone := 0
	totalFailures := 0
	hopts := stats.HistogramOptions{
		NumBuckets:     20,
		GrowthFactor:   1,
		BaseBucketSize: 1,
		MinValue:       0,
	}
	h := stats.NewHistogram(hopts)
	for i := 0; i < soakIterations; i++ {
		if ctx.Err() != nil {
			elapsedTime = time.Since(start).Seconds()
			break
		}
		earliestNextStart := time.After(minTimeBetweenRPCs)
		iterationsDone++
		var p peer.Peer
		latency, err := doOneSoakIteration(ctx, tc, resetChannel, serverAddr, soakRequestSize, soakResponseSize, dopts, []grpc.CallOption{grpc.Peer(&p)})
		latencyMs := int64(latency / time.Millisecond)
		h.Add(latencyMs)
		if err != nil {
			totalFailures++
			addrStr := "nil"
			if p.Addr != nil {
				addrStr = p.Addr.String()
			}
			fmt.Fprintf(os.Stderr, "soak iteration: %d elapsed_ms: %d peer: %s server_uri: %s failed: %s\n", i, latencyMs, addrStr, serverAddr, err)
			<-earliestNextStart
			continue
		}
		if latency > perIterationMaxAcceptableLatency {
			totalFailures++
			fmt.Fprintf(os.Stderr, "soak iteration: %d elapsed_ms: %d peer: %s server_uri: %s exceeds max acceptable latency: %d\n", i, latencyMs, p.Addr.String(), serverAddr, perIterationMaxAcceptableLatency.Milliseconds())
			<-earliestNextStart
			continue
		}
		fmt.Fprintf(os.Stderr, "soak iteration: %d elapsed_ms: %d peer: %s server_uri: %s succeeded\n", i, latencyMs, p.Addr.String(), serverAddr)
		<-earliestNextStart
	}
	var b bytes.Buffer
	h.Print(&b)
	fmt.Fprintf(os.Stderr, "(server_uri: %s) histogram of per-iteration latencies in milliseconds: %s\n", serverAddr, b.String())
	fmt.Fprintf(os.Stderr, "(server_uri: %s) soak test ran: %d / %d iterations. total failures: %d. max failures threshold: %d. See breakdown above for which iterations succeeded, failed, and why for more info.\n", serverAddr, iterationsDone, soakIterations, totalFailures, maxFailures)
	if iterationsDone < soakIterations {
		logger.Fatalf("(server_uri: %s) soak test consumed all %f seconds of time and quit early, only having ran %d out of desired %d iterations.", serverAddr, elapsedTime, iterationsDone, soakIterations)
	}
	if totalFailures > maxFailures {
		logger.Fatalf("(server_uri: %s) soak test total failures: %d exceeds max failures threshold: %d.", serverAddr, totalFailures, maxFailures)
	}
}

type testServer struct {
	testgrpc.UnimplementedTestServiceServer

	orcaMu          sync.Mutex
	metricsRecorder orca.ServerMetricsRecorder
}

// NewTestServerOptions contains options that control the behavior of the test
// server returned by NewTestServer.
type NewTestServerOptions struct {
	MetricsRecorder orca.ServerMetricsRecorder
}

// NewTestServer creates a test server for test service.  opts carries optional
// settings and does not need to be provided.  If multiple opts are provided,
// only the first one is used.
func NewTestServer(opts ...NewTestServerOptions) testgrpc.TestServiceServer {
	if len(opts) > 0 {
		return &testServer{metricsRecorder: opts[0].MetricsRecorder}
	}
	return &testServer{}
}

func (s *testServer) EmptyCall(context.Context, *testpb.Empty) (*testpb.Empty, error) {
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
			grpc.SendHeader(ctx, header)
		}
		if trailingMetadata, ok := md[trailingMetadataKey]; ok {
			trailer := metadata.Pairs(trailingMetadataKey, trailingMetadata[0])
			grpc.SetTrailer(ctx, trailer)
		}
	}
	if st != nil && st.Code != 0 {
		return nil, status.Error(codes.Code(st.Code), st.Message)
	}
	pl, err := serverNewPayload(in.GetResponseType(), in.GetResponseSize())
	if err != nil {
		return nil, err
	}
	if r, orcaData := orca.CallMetricsRecorderFromContext(ctx), in.GetOrcaPerQueryReport(); r != nil && orcaData != nil {
		// Transfer the request's per-Call ORCA data to the call metrics
		// recorder in the context, if present.
		setORCAMetrics(r, orcaData)
	}
	return &testpb.SimpleResponse{
		Payload: pl,
	}, nil
}

func setORCAMetrics(r orca.ServerMetricsRecorder, orcaData *testpb.TestOrcaReport) {
	r.SetCPUUtilization(orcaData.CpuUtilization)
	r.SetMemoryUtilization(orcaData.MemoryUtilization)
	if rq, ok := r.(orca.CallMetricsRecorder); ok {
		for k, v := range orcaData.RequestCost {
			rq.SetRequestCost(k, v)
		}
	}
	for k, v := range orcaData.Utilization {
		r.SetNamedUtilization(k, v)
	}
}

func (s *testServer) StreamingOutputCall(args *testpb.StreamingOutputCallRequest, stream testgrpc.TestService_StreamingOutputCallServer) error {
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

func (s *testServer) StreamingInputCall(stream testgrpc.TestService_StreamingInputCallServer) error {
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

func (s *testServer) FullDuplexCall(stream testgrpc.TestService_FullDuplexCallServer) error {
	if md, ok := metadata.FromIncomingContext(stream.Context()); ok {
		if initialMetadata, ok := md[initialMetadataKey]; ok {
			header := metadata.Pairs(initialMetadataKey, initialMetadata[0])
			stream.SendHeader(header)
		}
		if trailingMetadata, ok := md[trailingMetadataKey]; ok {
			trailer := metadata.Pairs(trailingMetadataKey, trailingMetadata[0])
			stream.SetTrailer(trailer)
		}
	}
	hasORCALock := false
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

		if r, orcaData := s.metricsRecorder, in.GetOrcaOobReport(); r != nil && orcaData != nil {
			// Transfer the request's OOB ORCA data to the server metrics recorder
			// in the server, if present.
			if !hasORCALock {
				s.orcaMu.Lock()
				defer s.orcaMu.Unlock()
				hasORCALock = true
			}
			setORCAMetrics(r, orcaData)
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

func (s *testServer) HalfDuplexCall(stream testgrpc.TestService_HalfDuplexCallServer) error {
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

// DoORCAPerRPCTest performs a unary RPC that enables ORCA per-call reporting
// and verifies the load report sent back to the LB policy's Done callback.
func DoORCAPerRPCTest(ctx context.Context, tc testgrpc.TestServiceClient) {
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	orcaRes := &v3orcapb.OrcaLoadReport{}
	_, err := tc.UnaryCall(contextWithORCAResult(ctx, &orcaRes), &testpb.SimpleRequest{
		OrcaPerQueryReport: &testpb.TestOrcaReport{
			CpuUtilization:    0.8210,
			MemoryUtilization: 0.5847,
			RequestCost:       map[string]float64{"cost": 3456.32},
			Utilization:       map[string]float64{"util": 0.30499},
		},
	})
	if err != nil {
		logger.Fatalf("/TestService/UnaryCall RPC failed: ", err)
	}
	want := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 0.8210,
		MemUtilization: 0.5847,
		RequestCost:    map[string]float64{"cost": 3456.32},
		Utilization:    map[string]float64{"util": 0.30499},
	}
	if !proto.Equal(orcaRes, want) {
		logger.Fatalf("/TestService/UnaryCall RPC received ORCA load report %+v; want %+v", orcaRes, want)
	}
}

// DoORCAOOBTest performs a streaming RPC that enables ORCA OOB reporting and
// verifies the load report sent to the LB policy's OOB listener.
func DoORCAOOBTest(ctx context.Context, tc testgrpc.TestServiceClient) {
	ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		logger.Fatalf("/TestService/FullDuplexCall received error starting stream: %v", err)
	}
	err = stream.Send(&testpb.StreamingOutputCallRequest{
		OrcaOobReport: &testpb.TestOrcaReport{
			CpuUtilization:    0.8210,
			MemoryUtilization: 0.5847,
			Utilization:       map[string]float64{"util": 0.30499},
		},
		ResponseParameters: []*testpb.ResponseParameters{{Size: 1}},
	})
	if err != nil {
		logger.Fatalf("/TestService/FullDuplexCall received error sending: %v", err)
	}
	_, err = stream.Recv()
	if err != nil {
		logger.Fatalf("/TestService/FullDuplexCall received error receiving: %v", err)
	}

	want := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 0.8210,
		MemUtilization: 0.5847,
		Utilization:    map[string]float64{"util": 0.30499},
	}
	checkORCAMetrics(ctx, tc, want)

	err = stream.Send(&testpb.StreamingOutputCallRequest{
		OrcaOobReport: &testpb.TestOrcaReport{
			CpuUtilization:    0.29309,
			MemoryUtilization: 0.2,
			Utilization:       map[string]float64{"util": 0.2039},
		},
		ResponseParameters: []*testpb.ResponseParameters{{Size: 1}},
	})
	if err != nil {
		logger.Fatalf("/TestService/FullDuplexCall received error sending: %v", err)
	}
	_, err = stream.Recv()
	if err != nil {
		logger.Fatalf("/TestService/FullDuplexCall received error receiving: %v", err)
	}

	want = &v3orcapb.OrcaLoadReport{
		CpuUtilization: 0.29309,
		MemUtilization: 0.2,
		Utilization:    map[string]float64{"util": 0.2039},
	}
	checkORCAMetrics(ctx, tc, want)
}

func checkORCAMetrics(ctx context.Context, tc testgrpc.TestServiceClient, want *v3orcapb.OrcaLoadReport) {
	for ctx.Err() == nil {
		orcaRes := &v3orcapb.OrcaLoadReport{}
		if _, err := tc.UnaryCall(contextWithORCAResult(ctx, &orcaRes), &testpb.SimpleRequest{}); err != nil {
			logger.Fatalf("/TestService/UnaryCall RPC failed: ", err)
		}
		if proto.Equal(orcaRes, want) {
			return
		}
		logger.Infof("/TestService/UnaryCall RPC received ORCA load report %+v; want %+v", orcaRes, want)
		time.Sleep(time.Second)
	}
	logger.Fatalf("timed out waiting for expected ORCA load report")
}
