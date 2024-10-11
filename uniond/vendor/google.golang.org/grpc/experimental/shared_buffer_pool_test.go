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

package experimental_test

import (
	"bytes"
	"context"
	"io"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/encoding/gzip"
	"google.golang.org/grpc/experimental"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const defaultTestTimeout = 10 * time.Second

func (s) TestRecvBufferPoolStream(t *testing.T) {
	// TODO: How much of this test can be preserved now that buffer reuse happens at
	// the codec and HTTP/2 level?
	t.SkipNow()
	tcs := []struct {
		name     string
		callOpts []grpc.CallOption
	}{
		{
			name: "default",
		},
		{
			name: "useCompressor",
			callOpts: []grpc.CallOption{
				grpc.UseCompressor(gzip.Name),
			},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			const reqCount = 10

			ss := &stubserver.StubServer{
				FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
					for i := 0; i < reqCount; i++ {
						preparedMsg := &grpc.PreparedMsg{}
						if err := preparedMsg.Encode(stream, &testpb.StreamingOutputCallResponse{
							Payload: &testpb.Payload{
								Body: []byte{'0' + uint8(i)},
							},
						}); err != nil {
							return err
						}
						stream.SendMsg(preparedMsg)
					}
					return nil
				},
			}

			pool := &checkBufferPool{}
			sopts := []grpc.ServerOption{experimental.BufferPool(pool)}
			dopts := []grpc.DialOption{experimental.WithBufferPool(pool)}
			if err := ss.Start(sopts, dopts...); err != nil {
				t.Fatalf("Error starting endpoint server: %v", err)
			}
			defer ss.Stop()

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()

			stream, err := ss.Client.FullDuplexCall(ctx, tc.callOpts...)
			if err != nil {
				t.Fatalf("ss.Client.FullDuplexCall failed: %v", err)
			}

			var ngot int
			var buf bytes.Buffer
			for {
				reply, err := stream.Recv()
				if err == io.EOF {
					break
				}
				if err != nil {
					t.Fatal(err)
				}
				ngot++
				if buf.Len() > 0 {
					buf.WriteByte(',')
				}
				buf.Write(reply.GetPayload().GetBody())
			}
			if want := 10; ngot != want {
				t.Fatalf("Got %d replies, want %d", ngot, want)
			}
			if got, want := buf.String(), "0,1,2,3,4,5,6,7,8,9"; got != want {
				t.Fatalf("Got replies %q; want %q", got, want)
			}

			if len(pool.puts) != reqCount {
				t.Fatalf("Expected 10 buffers to be returned to the pool, got %d", len(pool.puts))
			}
		})
	}
}

func (s) TestRecvBufferPoolUnary(t *testing.T) {
	// TODO: See above
	t.SkipNow()
	tcs := []struct {
		name     string
		callOpts []grpc.CallOption
	}{
		{
			name: "default",
		},
		{
			name: "useCompressor",
			callOpts: []grpc.CallOption{
				grpc.UseCompressor(gzip.Name),
			},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			const largeSize = 1024

			ss := &stubserver.StubServer{
				UnaryCallF: func(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
					return &testpb.SimpleResponse{
						Payload: &testpb.Payload{
							Body: make([]byte, largeSize),
						},
					}, nil
				},
			}

			pool := &checkBufferPool{}
			sopts := []grpc.ServerOption{experimental.BufferPool(pool)}
			dopts := []grpc.DialOption{experimental.WithBufferPool(pool)}
			if err := ss.Start(sopts, dopts...); err != nil {
				t.Fatalf("Error starting endpoint server: %v", err)
			}
			defer ss.Stop()

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()

			const reqCount = 10
			for i := 0; i < reqCount; i++ {
				if _, err := ss.Client.UnaryCall(
					ctx,
					&testpb.SimpleRequest{
						Payload: &testpb.Payload{
							Body: make([]byte, largeSize),
						},
					},
					tc.callOpts...,
				); err != nil {
					t.Fatalf("ss.Client.UnaryCall failed: %v", err)
				}
			}

			const bufferCount = reqCount * 2 // req + resp
			if len(pool.puts) != bufferCount {
				t.Fatalf("Expected %d buffers to be returned to the pool, got %d", bufferCount, len(pool.puts))
			}
		})
	}
}

type checkBufferPool struct {
	puts [][]byte
}

func (p *checkBufferPool) Get(size int) *[]byte {
	b := make([]byte, size)
	return &b
}

func (p *checkBufferPool) Put(bs *[]byte) {
	p.puts = append(p.puts, *bs)
}
