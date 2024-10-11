/*
 *
 * Copyright 2016 gRPC authors.
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

package grpc

import (
	"context"
	"net"
	"reflect"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/transport"
	"google.golang.org/grpc/status"
)

type emptyServiceServer any

type testServer struct{}

func errorDesc(err error) string {
	if s, ok := status.FromError(err); ok {
		return s.Message()
	}
	return err.Error()
}

func (s) TestStopBeforeServe(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to create listener: %v", err)
	}

	server := NewServer()
	server.Stop()
	err = server.Serve(lis)
	if err != ErrServerStopped {
		t.Fatalf("server.Serve() error = %v, want %v", err, ErrServerStopped)
	}

	// server.Serve is responsible for closing the listener, even if the
	// server was already stopped.
	err = lis.Close()
	if got, want := errorDesc(err), "use of closed"; !strings.Contains(got, want) {
		t.Errorf("Close() error = %q, want %q", got, want)
	}
}

func (s) TestGracefulStop(t *testing.T) {

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to create listener: %v", err)
	}

	server := NewServer()
	go func() {
		// make sure Serve() is called
		time.Sleep(time.Millisecond * 500)
		server.GracefulStop()
	}()

	err = server.Serve(lis)
	if err != nil {
		t.Fatalf("Serve() returned non-nil error on GracefulStop: %v", err)
	}
}

func (s) TestGetServiceInfo(t *testing.T) {
	testSd := ServiceDesc{
		ServiceName: "grpc.testing.EmptyService",
		HandlerType: (*emptyServiceServer)(nil),
		Methods: []MethodDesc{
			{
				MethodName: "EmptyCall",
				Handler:    nil,
			},
		},
		Streams: []StreamDesc{
			{
				StreamName:    "EmptyStream",
				Handler:       nil,
				ServerStreams: false,
				ClientStreams: true,
			},
		},
		Metadata: []int{0, 2, 1, 3},
	}

	server := NewServer()
	server.RegisterService(&testSd, &testServer{})

	info := server.GetServiceInfo()
	want := map[string]ServiceInfo{
		"grpc.testing.EmptyService": {
			Methods: []MethodInfo{
				{
					Name:           "EmptyCall",
					IsClientStream: false,
					IsServerStream: false,
				},
				{
					Name:           "EmptyStream",
					IsClientStream: true,
					IsServerStream: false,
				}},
			Metadata: []int{0, 2, 1, 3},
		},
	}

	if !reflect.DeepEqual(info, want) {
		t.Errorf("GetServiceInfo() = %+v, want %+v", info, want)
	}
}

func (s) TestRetryChainedInterceptor(t *testing.T) {
	var records []int
	i1 := func(ctx context.Context, req any, _ *UnaryServerInfo, handler UnaryHandler) (resp any, err error) {
		records = append(records, 1)
		// call handler twice to simulate a retry here.
		handler(ctx, req)
		return handler(ctx, req)
	}
	i2 := func(ctx context.Context, req any, _ *UnaryServerInfo, handler UnaryHandler) (resp any, err error) {
		records = append(records, 2)
		return handler(ctx, req)
	}
	i3 := func(ctx context.Context, req any, _ *UnaryServerInfo, handler UnaryHandler) (resp any, err error) {
		records = append(records, 3)
		return handler(ctx, req)
	}

	ii := chainUnaryInterceptors([]UnaryServerInterceptor{i1, i2, i3})

	handler := func(context.Context, any) (any, error) {
		return nil, nil
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	ii(ctx, nil, nil, handler)
	if !cmp.Equal(records, []int{1, 2, 3, 2, 3}) {
		t.Fatalf("retry failed on chained interceptors: %v", records)
	}
}

func (s) TestStreamContext(t *testing.T) {
	expectedStream := &transport.Stream{}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = NewContextWithServerTransportStream(ctx, expectedStream)

	s := ServerTransportStreamFromContext(ctx)
	stream, ok := s.(*transport.Stream)
	if !ok || expectedStream != stream {
		t.Fatalf("GetStreamFromContext(%v) = %v, %t, want: %v, true", ctx, stream, ok, expectedStream)
	}
}

func BenchmarkChainUnaryInterceptor(b *testing.B) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for _, n := range []int{1, 3, 5, 10} {
		n := n
		b.Run(strconv.Itoa(n), func(b *testing.B) {
			interceptors := make([]UnaryServerInterceptor, 0, n)
			for i := 0; i < n; i++ {
				interceptors = append(interceptors, func(
					ctx context.Context, req any, _ *UnaryServerInfo, handler UnaryHandler,
				) (any, error) {
					return handler(ctx, req)
				})
			}

			s := NewServer(ChainUnaryInterceptor(interceptors...))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				if _, err := s.opts.unaryInt(ctx, nil, nil,
					func(context.Context, any) (any, error) {
						return nil, nil
					},
				); err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}

func BenchmarkChainStreamInterceptor(b *testing.B) {
	for _, n := range []int{1, 3, 5, 10} {
		n := n
		b.Run(strconv.Itoa(n), func(b *testing.B) {
			interceptors := make([]StreamServerInterceptor, 0, n)
			for i := 0; i < n; i++ {
				interceptors = append(interceptors, func(
					srv any, ss ServerStream, _ *StreamServerInfo, handler StreamHandler,
				) error {
					return handler(srv, ss)
				})
			}

			s := NewServer(ChainStreamInterceptor(interceptors...))
			b.ReportAllocs()
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				if err := s.opts.streamInt(nil, nil, nil, func(any, ServerStream) error {
					return nil
				}); err != nil {
					b.Fatal(err)
				}
			}
		})
	}
}
