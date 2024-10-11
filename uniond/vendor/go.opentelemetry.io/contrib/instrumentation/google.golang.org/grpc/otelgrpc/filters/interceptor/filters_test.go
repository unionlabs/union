// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package interceptor // import "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/filters/interceptor"

import (
	"testing"

	"google.golang.org/grpc"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

type testCase struct {
	name string
	i    *otelgrpc.InterceptorInfo
	//nolint:staticcheck // Interceptors are deprecated and will be removed in the next release.
	f    otelgrpc.InterceptorFilter
	want bool
}

func dummyUnaryServerInfo(n string) *grpc.UnaryServerInfo {
	return &grpc.UnaryServerInfo{
		FullMethod: n,
	}
}

func dummyStreamServerInfo(n string) *grpc.StreamServerInfo {
	return &grpc.StreamServerInfo{
		FullMethod: n,
	}
}

func TestMethodName(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/Hello"
	tcs := []testCase{
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    MethodName("Hello"),
			want: true,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.StreamClient},
			f:    MethodName("Hello"),
			want: true,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    MethodName("Hello"),
			want: true,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethodName), Type: otelgrpc.StreamServer},
			f:    MethodName("Hello"),
			want: true,
		},
		{
			name: "unary client interceptor fail",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    MethodName("Goodbye"),
			want: false,
		},
	}

	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestMethodPrefix(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    MethodPrefix("Foobar"),
			want: true,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.StreamClient},
			f:    MethodPrefix("Foobar"),
			want: true,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    MethodPrefix("Foobar"),
			want: true,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethodName), Type: otelgrpc.StreamServer},
			f:    MethodPrefix("Foobar"),
			want: true,
		},
		{
			name: "unary client interceptor fail",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    MethodPrefix("Barfoo"),
			want: false,
		},
	}
	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestFullMethodName(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/Hello"
	tcs := []testCase{
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    FullMethodName(dummyFullMethodName),
			want: true,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.StreamClient},
			f:    FullMethodName(dummyFullMethodName),
			want: true,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    FullMethodName(dummyFullMethodName),
			want: true,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethodName), Type: otelgrpc.StreamServer},
			f:    FullMethodName(dummyFullMethodName),
			want: true,
		},
		{
			name: "unary client interceptor fail",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    FullMethodName("/example.HelloService/Goodbye"),
			want: false,
		},
	}

	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestServiceName(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/Hello"

	tcs := []testCase{
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    ServiceName("example.HelloService"),
			want: true,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.StreamClient},
			f:    ServiceName("example.HelloService"),
			want: true,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    ServiceName("example.HelloService"),
			want: true,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethodName), Type: otelgrpc.StreamServer},
			f:    ServiceName("example.HelloService"),
			want: true,
		},
		{
			name: "unary client interceptor fail",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    ServiceName("opentelemetry.HelloService"),
			want: false,
		},
	}

	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestServicePrefix(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    ServicePrefix("example"),
			want: true,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.StreamClient},
			f:    ServicePrefix("example"),
			want: true,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    ServicePrefix("example"),
			want: true,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethodName), Type: otelgrpc.StreamServer},
			f:    ServicePrefix("example"),
			want: true,
		},
		{
			name: "unary client interceptor fail",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    ServicePrefix("opentelemetry"),
			want: false,
		},
	}
	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestAny(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "unary client interceptor true && true",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    Any(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "unary client interceptor false && true",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    Any(MethodName("Hello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "unary client interceptor false && false",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    Any(MethodName("Goodbye"), MethodPrefix("Barfoo")),
			want: false,
		},
	}
	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestAll(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "unary client interceptor true && true",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    All(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "unary client interceptor true && false",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    All(MethodName("FoobarHello"), MethodPrefix("Barfoo")),
			want: false,
		},
		{
			name: "unary client interceptor false && false",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    All(MethodName("FoobarGoodbye"), MethodPrefix("Barfoo")),
			want: false,
		},
	}
	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestNone(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "unary client interceptor true && true",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    None(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: false,
		},
		{
			name: "unary client interceptor true && false",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    None(MethodName("FoobarHello"), MethodPrefix("Barfoo")),
			want: false,
		},
		{
			name: "unary client interceptor false && false",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    None(MethodName("FoobarGoodbye"), MethodPrefix("Barfoo")),
			want: true,
		},
	}
	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestNot(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/FoobarHello"
	tcs := []testCase{
		{
			name: "methodname not",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    Not(MethodName("FoobarHello")),
			want: false,
		},
		{
			name: "method prefix not",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethodName), Type: otelgrpc.UnaryServer},
			f:    Not(MethodPrefix("FoobarHello")),
			want: false,
		},
		{
			name: "unary client interceptor not all(true && true)",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethodName, Type: otelgrpc.UnaryClient},
			f:    Not(All(MethodName("FoobarHello"), MethodPrefix("Foobar"))),
			want: false,
		},
	}

	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}

func TestHealthCheck(t *testing.T) {
	const (
		healthCheck     = "/grpc.health.v1.Health/Check"
		dummyFullMethod = "/example.HelloService/FoobarHello"
	)
	tcs := []testCase{
		{
			name: "unary client interceptor healthcheck",
			i:    &otelgrpc.InterceptorInfo{Method: healthCheck, Type: otelgrpc.UnaryClient},
			f:    HealthCheck(),
			want: true,
		},
		{
			name: "stream client interceptor healthcheck",
			i:    &otelgrpc.InterceptorInfo{Method: healthCheck, Type: otelgrpc.StreamClient},
			f:    HealthCheck(),
			want: true,
		},
		{
			name: "unary server interceptor healthcheck",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(healthCheck), Type: otelgrpc.UnaryServer},
			f:    HealthCheck(),
			want: true,
		},
		{
			name: "stream server interceptor healthcheck",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(healthCheck), Type: otelgrpc.StreamServer},
			f:    HealthCheck(),
			want: true,
		},
		{
			name: "unary client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethod, Type: otelgrpc.UnaryClient},
			f:    HealthCheck(),
			want: false,
		},
		{
			name: "stream client interceptor",
			i:    &otelgrpc.InterceptorInfo{Method: dummyFullMethod, Type: otelgrpc.StreamClient},
			f:    HealthCheck(),
			want: false,
		},
		{
			name: "unary server interceptor",
			i:    &otelgrpc.InterceptorInfo{UnaryServerInfo: dummyUnaryServerInfo(dummyFullMethod), Type: otelgrpc.UnaryServer},
			f:    HealthCheck(),
			want: false,
		},
		{
			name: "stream server interceptor",
			i:    &otelgrpc.InterceptorInfo{StreamServerInfo: dummyStreamServerInfo(dummyFullMethod), Type: otelgrpc.StreamServer},
			f:    HealthCheck(),
			want: false,
		},
	}

	for _, tc := range tcs {
		out := tc.f(tc.i)
		if tc.want != out {
			t.Errorf("test case '%v' failed, wanted %v but obtained %v", tc.name, tc.want, out)
		}
	}
}
