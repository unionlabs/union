// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Package filters provides a set of filters useful with the
// [otelgrpc.WithFilter] option to control which inbound requests are instrumented.
package filters // import "go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc/filters"

import (
	"path"
	"strings"

	"google.golang.org/grpc/stats"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

type gRPCPath struct {
	service string
	method  string
}

// splitFullMethod splits path defined in gRPC protocol
// and returns as gRPCPath object that has divided service and method names
// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
// If name is not FullMethod, returned gRPCPath has empty service field.
func splitFullMethod(i *stats.RPCTagInfo) gRPCPath {
	s, m := path.Split(i.FullMethodName)
	if s != "" {
		s = path.Clean(s)
		s = strings.TrimLeft(s, "/")
	}

	return gRPCPath{
		service: s,
		method:  m,
	}
}

// Any takes a list of Filters and returns a Filter that
// returns true if any Filter in the list returns true.
func Any(fs ...otelgrpc.Filter) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		for _, f := range fs {
			if f(i) {
				return true
			}
		}
		return false
	}
}

// All takes a list of Filters and returns a Filter that
// returns true only if all Filters in the list return true.
func All(fs ...otelgrpc.Filter) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		for _, f := range fs {
			if !f(i) {
				return false
			}
		}
		return true
	}
}

// None takes a list of Filters and returns a Filter that returns
// true only if none of the Filters in the list return true.
func None(fs ...otelgrpc.Filter) otelgrpc.Filter {
	return Not(Any(fs...))
}

// Not provides a convenience mechanism for inverting a Filter.
func Not(f otelgrpc.Filter) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		return !f(i)
	}
}

// MethodName returns a Filter that returns true if the request's
// method name matches the provided string n.
func MethodName(n string) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		p := splitFullMethod(i)
		return p.method == n
	}
}

// MethodPrefix returns a Filter that returns true if the request's
// method starts with the provided string pre.
func MethodPrefix(pre string) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		p := splitFullMethod(i)
		return strings.HasPrefix(p.method, pre)
	}
}

// FullMethodName returns a Filter that returns true if the request's
// full RPC method string, i.e. /package.service/method, starts with
// the provided string n.
func FullMethodName(n string) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		return i.FullMethodName == n
	}
}

// ServiceName returns a Filter that returns true if the request's
// service name, i.e. package.service, matches s.
func ServiceName(s string) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		p := splitFullMethod(i)
		return p.service == s
	}
}

// ServicePrefix returns a Filter that returns true if the request's
// service name, i.e. package.service, starts with the provided string pre.
func ServicePrefix(pre string) otelgrpc.Filter {
	return func(i *stats.RPCTagInfo) bool {
		p := splitFullMethod(i)
		return strings.HasPrefix(p.service, pre)
	}
}

// HealthCheck returns a Filter that returns true if the request's
// service name is health check defined by gRPC Health Checking Protocol.
// https://github.com/grpc/grpc/blob/master/doc/health-checking.md
func HealthCheck() otelgrpc.Filter {
	return ServicePrefix("grpc.health.v1.Health")
}
