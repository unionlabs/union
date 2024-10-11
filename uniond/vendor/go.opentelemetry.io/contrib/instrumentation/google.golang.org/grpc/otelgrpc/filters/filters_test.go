// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package filters // import "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/filters"

import (
	"testing"

	"google.golang.org/grpc/stats"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

type testCase struct {
	name string
	i    *stats.RPCTagInfo
	f    otelgrpc.Filter
	want bool
}

func dummyRPCTagInfo(n string) *stats.RPCTagInfo {
	return &stats.RPCTagInfo{
		FullMethodName: n,
	}
}

func TestMethodName(t *testing.T) {
	const dummyFullMethodName = "/example.HelloService/Hello"
	tcs := []testCase{
		{
			name: "true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    MethodName("Hello"),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    MethodPrefix("Foobar"),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    FullMethodName(dummyFullMethodName),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    ServiceName("example.HelloService"),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    ServicePrefix("example"),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true && true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    Any(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "false && true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    Any(MethodName("Hello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "false && false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true && true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    All(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: true,
		},
		{
			name: "true && false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    All(MethodName("FoobarHello"), MethodPrefix("Barfoo")),
			want: false,
		},
		{
			name: "false && false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true && true",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    None(MethodName("FoobarHello"), MethodPrefix("Foobar")),
			want: false,
		},
		{
			name: "true && false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    None(MethodName("FoobarHello"), MethodPrefix("Barfoo")),
			want: false,
		},
		{
			name: "false && false",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    Not(MethodName("FoobarHello")),
			want: false,
		},
		{
			name: "method prefix not",
			i:    dummyRPCTagInfo(dummyFullMethodName),
			f:    Not(MethodPrefix("FoobarHello")),
			want: false,
		},
		{
			name: "not all(true && true)",
			i:    dummyRPCTagInfo(dummyFullMethodName),
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
			name: "true",
			i:    dummyRPCTagInfo(healthCheck),
			f:    HealthCheck(),
			want: true,
		},
		{
			name: "false",
			i:    dummyRPCTagInfo(dummyFullMethod),
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
