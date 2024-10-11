// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelgrpc_test

import (
	"context"
	"net"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/test/bufconn"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/internal/test"
	"go.opentelemetry.io/otel/trace/noop"

	pb "google.golang.org/grpc/interop/grpc_testing"
)

const bufSize = 2048

var tracerProvider = noop.NewTracerProvider()

func benchmark(b *testing.B, cOpt []grpc.DialOption, sOpt []grpc.ServerOption) {
	l := bufconn.Listen(bufSize)
	defer l.Close()

	s := grpc.NewServer(sOpt...)
	pb.RegisterTestServiceServer(s, test.NewTestServer())
	go func() {
		if err := s.Serve(l); err != nil {
			panic(err)
		}
	}()
	defer s.Stop()

	ctx := context.Background()
	dial := func(context.Context, string) (net.Conn, error) { return l.Dial() }
	conn, err := grpc.NewClient(
		"bufnet",
		append([]grpc.DialOption{
			grpc.WithContextDialer(dial),
			grpc.WithTransportCredentials(insecure.NewCredentials()),
		}, cOpt...)...,
	)
	if err != nil {
		b.Fatalf("Failed to dial bufnet: %v", err)
	}
	defer conn.Close()
	client := pb.NewTestServiceClient(conn)

	b.ReportAllocs()
	b.ResetTimer()

	for n := 0; n < b.N; n++ {
		test.DoEmptyUnaryCall(ctx, client)
		test.DoLargeUnaryCall(ctx, client)
		test.DoClientStreaming(ctx, client)
		test.DoServerStreaming(ctx, client)
		test.DoPingPong(ctx, client)
		test.DoEmptyStream(ctx, client)
	}

	b.StopTimer()
}

func BenchmarkNoInstrumentation(b *testing.B) {
	benchmark(b, nil, nil)
}

func BenchmarkUnaryServerInterceptor(b *testing.B) {
	benchmark(b, nil, []grpc.ServerOption{
		grpc.UnaryInterceptor(otelgrpc.UnaryServerInterceptor(
			otelgrpc.WithTracerProvider(tracerProvider),
		)),
	})
}

func BenchmarkStreamServerInterceptor(b *testing.B) {
	benchmark(b, nil, []grpc.ServerOption{
		grpc.StreamInterceptor(otelgrpc.StreamServerInterceptor(
			otelgrpc.WithTracerProvider(tracerProvider),
		)),
	})
}

func BenchmarkUnaryClientInterceptor(b *testing.B) {
	benchmark(b, []grpc.DialOption{
		grpc.WithUnaryInterceptor(otelgrpc.UnaryClientInterceptor(
			otelgrpc.WithTracerProvider(tracerProvider),
		)),
	}, nil)
}

func BenchmarkStreamClientInterceptor(b *testing.B) {
	benchmark(b, []grpc.DialOption{
		grpc.WithStreamInterceptor(otelgrpc.StreamClientInterceptor(
			otelgrpc.WithTracerProvider(tracerProvider),
		)),
	}, nil)
}
