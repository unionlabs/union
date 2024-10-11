// Copyright (c) HashiCorp, Inc.
// SPDX-License-Identifier: MPL-2.0

package plugin

import (
	"context"
	"reflect"
	"testing"

	grpctest "github.com/hashicorp/go-plugin/test/grpc"
	"github.com/jhump/protoreflect/grpcreflect"
	"google.golang.org/grpc"
	reflectpb "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"
)

func TestGRPC_App(t *testing.T) {
	t.Run("default", func(t *testing.T) {
		testGRPCClientApp(t, false)
	})
	t.Run("mux", func(t *testing.T) {
		testGRPCClientApp(t, true)
	})
}

func testGRPCClientApp(t *testing.T, multiplex bool) {
	client, server := TestPluginGRPCConn(t, multiplex, map[string]Plugin{
		"test": new(testGRPCInterfacePlugin),
	})
	defer client.Close()
	defer server.Stop()

	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	impl, ok := raw.(testInterface)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	result := impl.Double(21)
	if result != 42 {
		t.Fatalf("bad: %#v", result)
	}

	err = impl.Bidirectional()
	if err != nil {
		t.Fatal(err)
	}
}

func TestGRPCConn_BidirectionalPing(t *testing.T) {
	conn, _ := TestGRPCConn(t, func(s *grpc.Server) {
		grpctest.RegisterPingPongServer(s, &pingPongServer{})
	})
	defer conn.Close()
	pingPongClient := grpctest.NewPingPongClient(conn)

	pResp, err := pingPongClient.Ping(context.Background(), &grpctest.PingRequest{})
	if err != nil {
		t.Fatal(err)
	}
	if pResp.Msg != "pong" {
		t.Fatal("Bad PingPong")
	}
}

func TestGRPCC_Stream(t *testing.T) {
	t.Run("default", func(t *testing.T) {
		testGRPCStream(t, false)
	})
	t.Run("mux", func(t *testing.T) {
		testGRPCStream(t, true)
	})
}

func testGRPCStream(t *testing.T, multiplex bool) {
	client, server := TestPluginGRPCConn(t, multiplex, map[string]Plugin{
		"test": new(testGRPCInterfacePlugin),
	})
	defer client.Close()
	defer server.Stop()

	raw, err := client.Dispense("test")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	impl, ok := raw.(testStreamer)
	if !ok {
		t.Fatalf("bad: %#v", raw)
	}

	expected := []int32{21, 22, 23, 24, 25, 26}
	result, err := impl.Stream(21, 27)
	if err != nil {
		t.Fatal(err)
	}

	if !reflect.DeepEqual(result, expected) {
		t.Fatalf("expected: %v\ngot: %v", expected, result)
	}
}

func TestGRPC_Ping(t *testing.T) {
	t.Run("default", func(t *testing.T) {
		testGRPCClientPing(t, false)
	})
	t.Run("mux", func(t *testing.T) {
		testGRPCClientPing(t, true)
	})
}

func testGRPCClientPing(t *testing.T, multiplex bool) {
	client, server := TestPluginGRPCConn(t, multiplex, map[string]Plugin{
		"test": new(testGRPCInterfacePlugin),
	})
	defer client.Close()
	defer server.Stop()

	// Run a couple pings
	if err := client.Ping(); err != nil {
		t.Fatalf("err: %s", err)
	}
	if err := client.Ping(); err != nil {
		t.Fatalf("err: %s", err)
	}

	// Close the remote end
	server.server.Stop()

	// Test ping fails
	if err := client.Ping(); err == nil {
		t.Fatal("should error")
	}
}

func TestGRPC_Reflection(t *testing.T) {
	t.Run("default", func(t *testing.T) {
		testGRPCClientReflection(t, false)
	})
	t.Run("mux", func(t *testing.T) {
		testGRPCClientReflection(t, true)
	})
}

func testGRPCClientReflection(t *testing.T, multiplex bool) {
	ctx := context.Background()

	client, server := TestPluginGRPCConn(t, multiplex, map[string]Plugin{
		"test": new(testGRPCInterfacePlugin),
	})
	defer client.Close()
	defer server.Stop()

	refClient := grpcreflect.NewClient(ctx, reflectpb.NewServerReflectionClient(client.Conn))

	svcs, err := refClient.ListServices()
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	// TODO: maybe only assert some specific services here to make test more resilient
	expectedSvcs := []string{"grpc.health.v1.Health", "grpc.reflection.v1alpha.ServerReflection", "grpctest.Test", "plugin.GRPCBroker", "plugin.GRPCController", "plugin.GRPCStdio"}

	if !reflect.DeepEqual(svcs, expectedSvcs) {
		t.Fatalf("expected: %v\ngot: %v", expectedSvcs, svcs)
	}

	healthDesc, err := refClient.ResolveService("grpc.health.v1.Health")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	methods := healthDesc.GetMethods()
	var methodNames []string
	for _, m := range methods {
		methodNames = append(methodNames, m.GetName())
	}

	expectedMethodNames := []string{"Check", "Watch"}

	if !reflect.DeepEqual(methodNames, expectedMethodNames) {
		t.Fatalf("expected: %v\ngot: %v", expectedMethodNames, methodNames)
	}
}
