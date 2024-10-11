/*
 *
 * Copyright 2023 gRPC authors.
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

package test

import (
	"bytes"
	"compress/gzip"
	"context"
	"io"
	"reflect"
	"strings"
	"sync/atomic"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/encoding"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// TestUnsupportedEncodingResponse validates gRPC status codes
// for different client-server compression setups
// ensuring the correct behavior when compression is enabled or disabled on either side.
func (s) TestUnsupportedEncodingResponse(t *testing.T) {
	tests := []struct {
		name           string
		clientCompress bool
		serverCompress bool
		wantStatus     codes.Code
	}{
		{
			name:           "client_server_compression",
			clientCompress: true,
			serverCompress: true,
			wantStatus:     codes.OK,
		},
		{
			name:           "client_compression",
			clientCompress: true,
			serverCompress: false,
			wantStatus:     codes.Unimplemented,
		},
		{
			name:           "server_compression",
			clientCompress: false,
			serverCompress: true,
			wantStatus:     codes.Internal,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			ss := &stubserver.StubServer{
				UnaryCallF: func(_ context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
					return &testpb.SimpleResponse{Payload: in.Payload}, nil
				},
			}
			sopts := []grpc.ServerOption{}
			if test.serverCompress {
				// Using deprecated methods to selectively apply compression
				// only on the server side. With encoding.registerCompressor(),
				// the compressor is applied globally, affecting client and server
				sopts = append(sopts, grpc.RPCCompressor(newNopCompressor()), grpc.RPCDecompressor(newNopDecompressor()))
			}
			if err := ss.StartServer(sopts...); err != nil {
				t.Fatalf("Error starting server: %v", err)
			}
			defer ss.Stop()

			dopts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials())}
			if test.clientCompress {
				// UseCompressor() requires the compressor to be registered
				// using encoding.RegisterCompressor() which applies compressor globally,
				// Hence, using deprecated WithCompressor() and WithDecompressor()
				// to apply compression only on client.
				dopts = append(dopts, grpc.WithCompressor(newNopCompressor()), grpc.WithDecompressor(newNopDecompressor()))
			}
			if err := ss.StartClient(dopts...); err != nil {
				t.Fatalf("Error starting client: %v", err)
			}

			payload := &testpb.SimpleRequest{
				Payload: &testpb.Payload{
					Body: []byte("test message"),
				},
			}
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			_, err := ss.Client.UnaryCall(ctx, payload)
			if got, want := status.Code(err), test.wantStatus; got != want {
				t.Errorf("Client.UnaryCall() = %v, want %v", got, want)
			}
		})
	}
}

func (s) TestCompressServerHasNoSupport(t *testing.T) {
	for _, e := range listTestEnv() {
		testCompressServerHasNoSupport(t, e)
	}
}

func testCompressServerHasNoSupport(t *testing.T, e env) {
	te := newTest(t, e)
	te.serverCompression = false
	te.clientCompression = false
	te.clientNopCompression = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const argSize = 271828
	const respSize = 314159
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.Unimplemented {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code %s", err, codes.Unimplemented)
	}
	// Streaming RPC
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.Unimplemented {
		t.Fatalf("%v.Recv() = %v, want error code %s", stream, err, codes.Unimplemented)
	}
}

func (s) TestCompressOK(t *testing.T) {
	for _, e := range listTestEnv() {
		testCompressOK(t, e)
	}
}

func testCompressOK(t *testing.T, e env) {
	te := newTest(t, e)
	te.serverCompression = true
	te.clientCompression = true
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	// Unary call
	const argSize = 271828
	const respSize = 314159
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("something", "something"))
	if _, err := tc.UnaryCall(ctx, req); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, <nil>", err)
	}
	// Streaming RPC
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 31415,
		},
	}
	payload, err = newPayload(testpb.PayloadType_COMPRESSABLE, int32(31415))
	if err != nil {
		t.Fatal(err)
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	stream.CloseSend()
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v.Recv() = %v, want io.EOF", stream, err)
	}
}

func (s) TestIdentityEncoding(t *testing.T) {
	for _, e := range listTestEnv() {
		testIdentityEncoding(t, e)
	}
}

func testIdentityEncoding(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	// Unary call
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 5)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: 10,
		Payload:      payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("something", "something"))
	if _, err := tc.UnaryCall(ctx, req); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, <nil>", err)
	}
	// Streaming RPC
	stream, err := tc.FullDuplexCall(ctx, grpc.UseCompressor("identity"))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	payload, err = newPayload(testpb.PayloadType_COMPRESSABLE, int32(31415))
	if err != nil {
		t.Fatal(err)
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{{Size: 10}},
		Payload:            payload,
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	stream.CloseSend()
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v.Recv() = %v, want io.EOF", stream, err)
	}
}

// renameCompressor is a grpc.Compressor wrapper that allows customizing the
// Type() of another compressor.
type renameCompressor struct {
	grpc.Compressor
	name string
}

func (r *renameCompressor) Type() string { return r.name }

// renameDecompressor is a grpc.Decompressor wrapper that allows customizing the
// Type() of another Decompressor.
type renameDecompressor struct {
	grpc.Decompressor
	name string
}

func (r *renameDecompressor) Type() string { return r.name }

func (s) TestClientForwardsGrpcAcceptEncodingHeader(t *testing.T) {
	wantGrpcAcceptEncodingCh := make(chan []string, 1)
	defer close(wantGrpcAcceptEncodingCh)

	compressor := renameCompressor{Compressor: grpc.NewGZIPCompressor(), name: "testgzip"}
	decompressor := renameDecompressor{Decompressor: grpc.NewGZIPDecompressor(), name: "testgzip"}

	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			md, ok := metadata.FromIncomingContext(ctx)
			if !ok {
				return nil, status.Errorf(codes.Internal, "no metadata in context")
			}
			if got, want := md["grpc-accept-encoding"], <-wantGrpcAcceptEncodingCh; !reflect.DeepEqual(got, want) {
				return nil, status.Errorf(codes.Internal, "got grpc-accept-encoding=%q; want [%q]", got, want)
			}
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{grpc.RPCDecompressor(&decompressor)}); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	wantGrpcAcceptEncodingCh <- []string{"gzip"}
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}

	wantGrpcAcceptEncodingCh <- []string{"gzip"}
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}, grpc.UseCompressor("gzip")); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}

	// Use compressor directly which is not registered via
	// encoding.RegisterCompressor.
	if err := ss.StartClient(grpc.WithCompressor(&compressor)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	wantGrpcAcceptEncodingCh <- []string{"gzip,testgzip"}
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}
}

// wrapCompressor is a wrapper of encoding.Compressor which maintains count of
// Compressor method invokes.
type wrapCompressor struct {
	encoding.Compressor
	compressInvokes int32
}

func (wc *wrapCompressor) Compress(w io.Writer) (io.WriteCloser, error) {
	atomic.AddInt32(&wc.compressInvokes, 1)
	return wc.Compressor.Compress(w)
}

func setupGzipWrapCompressor(t *testing.T) *wrapCompressor {
	oldC := encoding.GetCompressor("gzip")
	c := &wrapCompressor{Compressor: oldC}
	encoding.RegisterCompressor(c)
	t.Cleanup(func() {
		encoding.RegisterCompressor(oldC)
	})
	return c
}

func (s) TestSetSendCompressorSuccess(t *testing.T) {
	for _, tt := range []struct {
		name                string
		desc                string
		payload             *testpb.Payload
		dialOpts            []grpc.DialOption
		resCompressor       string
		wantCompressInvokes int32
	}{
		{
			name:                "identity_request_and_gzip_response",
			desc:                "request is uncompressed and response is gzip compressed",
			payload:             &testpb.Payload{Body: []byte("payload")},
			resCompressor:       "gzip",
			wantCompressInvokes: 1,
		},
		{
			name:                "identity_request_and_empty_response",
			desc:                "request is uncompressed and response is gzip compressed",
			payload:             nil,
			resCompressor:       "gzip",
			wantCompressInvokes: 0,
		},
		{
			name:          "gzip_request_and_identity_response",
			desc:          "request is gzip compressed and response is uncompressed with identity",
			payload:       &testpb.Payload{Body: []byte("payload")},
			resCompressor: "identity",
			dialOpts: []grpc.DialOption{
				// Use WithCompressor instead of UseCompressor to avoid counting
				// the client's compressor usage.
				grpc.WithCompressor(grpc.NewGZIPCompressor()),
			},
			wantCompressInvokes: 0,
		},
	} {
		t.Run(tt.name, func(t *testing.T) {
			t.Run("unary", func(t *testing.T) {
				testUnarySetSendCompressorSuccess(t, tt.payload, tt.resCompressor, tt.wantCompressInvokes, tt.dialOpts)
			})

			t.Run("stream", func(t *testing.T) {
				testStreamSetSendCompressorSuccess(t, tt.payload, tt.resCompressor, tt.wantCompressInvokes, tt.dialOpts)
			})
		})
	}
}

func testUnarySetSendCompressorSuccess(t *testing.T, payload *testpb.Payload, resCompressor string, wantCompressInvokes int32, dialOpts []grpc.DialOption) {
	wc := setupGzipWrapCompressor(t)
	ss := &stubserver.StubServer{
		UnaryCallF: func(ctx context.Context, _ *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			if err := grpc.SetSendCompressor(ctx, resCompressor); err != nil {
				return nil, err
			}
			return &testpb.SimpleResponse{
				Payload: payload,
			}, nil
		},
	}
	if err := ss.Start(nil, dialOpts...); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("Unexpected unary call error, got: %v, want: nil", err)
	}

	compressInvokes := atomic.LoadInt32(&wc.compressInvokes)
	if compressInvokes != wantCompressInvokes {
		t.Fatalf("Unexpected compress invokes, got:%d, want: %d", compressInvokes, wantCompressInvokes)
	}
}

func testStreamSetSendCompressorSuccess(t *testing.T, payload *testpb.Payload, resCompressor string, wantCompressInvokes int32, dialOpts []grpc.DialOption) {
	wc := setupGzipWrapCompressor(t)
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if _, err := stream.Recv(); err != nil {
				return err
			}

			if err := grpc.SetSendCompressor(stream.Context(), resCompressor); err != nil {
				return err
			}

			return stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: payload,
			})
		},
	}
	if err := ss.Start(nil, dialOpts...); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	s, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("Unexpected full duplex call error, got: %v, want: nil", err)
	}

	if err := s.Send(&testpb.StreamingOutputCallRequest{}); err != nil {
		t.Fatalf("Unexpected full duplex call send error, got: %v, want: nil", err)
	}

	if _, err := s.Recv(); err != nil {
		t.Fatalf("Unexpected full duplex recv error, got: %v, want: nil", err)
	}

	compressInvokes := atomic.LoadInt32(&wc.compressInvokes)
	if compressInvokes != wantCompressInvokes {
		t.Fatalf("Unexpected compress invokes, got:%d, want: %d", compressInvokes, wantCompressInvokes)
	}
}

func (s) TestUnregisteredSetSendCompressorFailure(t *testing.T) {
	resCompressor := "snappy2"
	wantErr := status.Error(codes.Unknown, "unable to set send compressor: compressor not registered \"snappy2\"")

	t.Run("unary", func(t *testing.T) {
		testUnarySetSendCompressorFailure(t, resCompressor, wantErr)
	})

	t.Run("stream", func(t *testing.T) {
		testStreamSetSendCompressorFailure(t, resCompressor, wantErr)
	})
}

func testUnarySetSendCompressorFailure(t *testing.T, resCompressor string, wantErr error) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			if err := grpc.SetSendCompressor(ctx, resCompressor); err != nil {
				return nil, err
			}
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); !equalError(err, wantErr) {
		t.Fatalf("Unexpected unary call error, got: %v, want: %v", err, wantErr)
	}
}

func testStreamSetSendCompressorFailure(t *testing.T, resCompressor string, wantErr error) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			if _, err := stream.Recv(); err != nil {
				return err
			}

			if err := grpc.SetSendCompressor(stream.Context(), resCompressor); err != nil {
				return err
			}

			return stream.Send(&testpb.StreamingOutputCallResponse{})
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v, want: nil", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	s, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("Unexpected full duplex call error, got: %v, want: nil", err)
	}

	if err := s.Send(&testpb.StreamingOutputCallRequest{}); err != nil {
		t.Fatalf("Unexpected full duplex call send error, got: %v, want: nil", err)
	}

	if _, err := s.Recv(); !equalError(err, wantErr) {
		t.Fatalf("Unexpected full duplex recv error, got: %v, want: nil", err)
	}
}

func (s) TestUnarySetSendCompressorAfterHeaderSendFailure(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			// Send headers early and then set send compressor.
			grpc.SendHeader(ctx, metadata.MD{})
			err := grpc.SetSendCompressor(ctx, "gzip")
			if err == nil {
				t.Error("Wanted set send compressor error")
				return &testpb.Empty{}, nil
			}
			return nil, err
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	wantErr := status.Error(codes.Unknown, "transport: set send compressor called after headers sent or stream done")
	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); !equalError(err, wantErr) {
		t.Fatalf("Unexpected unary call error, got: %v, want: %v", err, wantErr)
	}
}

func (s) TestStreamSetSendCompressorAfterHeaderSendFailure(t *testing.T) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			// Send headers early and then set send compressor.
			grpc.SendHeader(stream.Context(), metadata.MD{})
			err := grpc.SetSendCompressor(stream.Context(), "gzip")
			if err == nil {
				t.Error("Wanted set send compressor error")
			}
			return err
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	wantErr := status.Error(codes.Unknown, "transport: set send compressor called after headers sent or stream done")
	s, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("Unexpected full duplex call error, got: %v, want: nil", err)
	}

	if _, err := s.Recv(); !equalError(err, wantErr) {
		t.Fatalf("Unexpected full duplex recv error, got: %v, want: %v", err, wantErr)
	}
}

func (s) TestClientSupportedCompressors(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	for _, tt := range []struct {
		desc string
		ctx  context.Context
		want []string
	}{
		{
			desc: "No additional grpc-accept-encoding header",
			ctx:  ctx,
			want: []string{"gzip"},
		},
		{
			desc: "With additional grpc-accept-encoding header",
			ctx: metadata.AppendToOutgoingContext(ctx,
				"grpc-accept-encoding", "test-compressor-1",
				"grpc-accept-encoding", "test-compressor-2",
			),
			want: []string{"gzip", "test-compressor-1", "test-compressor-2"},
		},
		{
			desc: "With additional empty grpc-accept-encoding header",
			ctx: metadata.AppendToOutgoingContext(ctx,
				"grpc-accept-encoding", "",
			),
			want: []string{"gzip"},
		},
		{
			desc: "With additional grpc-accept-encoding header with spaces between values",
			ctx: metadata.AppendToOutgoingContext(ctx,
				"grpc-accept-encoding", "identity, deflate",
			),
			want: []string{"gzip", "identity", "deflate"},
		},
	} {
		t.Run(tt.desc, func(t *testing.T) {
			ss := &stubserver.StubServer{
				EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
					got, err := grpc.ClientSupportedCompressors(ctx)
					if err != nil {
						return nil, err
					}

					if !reflect.DeepEqual(got, tt.want) {
						t.Errorf("unexpected client compressors got: %v, want: %v", got, tt.want)
					}

					return &testpb.Empty{}, nil
				},
			}
			if err := ss.Start(nil); err != nil {
				t.Fatalf("Error starting endpoint server: %v, want: nil", err)
			}
			defer ss.Stop()

			_, err := ss.Client.EmptyCall(tt.ctx, &testpb.Empty{})
			if err != nil {
				t.Fatalf("Unexpected unary call error, got: %v, want: nil", err)
			}
		})
	}
}

func (s) TestCompressorRegister(t *testing.T) {
	for _, e := range listTestEnv() {
		testCompressorRegister(t, e)
	}
}

func testCompressorRegister(t *testing.T, e env) {
	te := newTest(t, e)
	te.clientCompression = false
	te.serverCompression = false
	te.clientUseCompression = true

	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	// Unary call
	const argSize = 271828
	const respSize = 314159
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, metadata.Pairs("something", "something"))
	if _, err := tc.UnaryCall(ctx, req); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, <nil>", err)
	}
	// Streaming RPC
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 31415,
		},
	}
	payload, err = newPayload(testpb.PayloadType_COMPRESSABLE, int32(31415))
	if err != nil {
		t.Fatal(err)
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}
}

type badGzipCompressor struct{}

func (badGzipCompressor) Do(w io.Writer, p []byte) error {
	buf := &bytes.Buffer{}
	gzw := gzip.NewWriter(buf)
	if _, err := gzw.Write(p); err != nil {
		return err
	}
	err := gzw.Close()
	bs := buf.Bytes()
	if len(bs) >= 6 {
		bs[len(bs)-6] ^= 1 // modify checksum at end by 1 byte
	}
	w.Write(bs)
	return err
}

func (badGzipCompressor) Type() string {
	return "gzip"
}

func (s) TestGzipBadChecksum(t *testing.T) {
	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
	}
	if err := ss.Start(nil, grpc.WithCompressor(badGzipCompressor{})); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	p, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(1024))
	if err != nil {
		t.Fatalf("Unexpected error from newPayload: %v", err)
	}
	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{Payload: p}); err == nil ||
		status.Code(err) != codes.Internal ||
		!strings.Contains(status.Convert(err).Message(), gzip.ErrChecksum.Error()) {
		t.Errorf("ss.Client.UnaryCall(_) = _, %v\n\twant: _, status(codes.Internal, contains %q)", err, gzip.ErrChecksum)
	}
}
