// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package otelgrpc_test

import (
	"google.golang.org/grpc"

	"go.opentelemetry.io/contrib/instrumentation/google.golang.org/grpc/otelgrpc"
)

func ExampleNewClientHandler() {
	_, _ = grpc.NewClient("localhost", grpc.WithStatsHandler(otelgrpc.NewClientHandler()))
}

func ExampleNewServerHandler() {
	_ = grpc.NewServer(grpc.StatsHandler(otelgrpc.NewServerHandler()))
}
