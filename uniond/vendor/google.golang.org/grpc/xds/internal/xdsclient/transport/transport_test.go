/*
 *
 * Copyright 2022 gRPC authors.
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
 */

package transport_test

import (
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/transport"
	"google.golang.org/grpc/xds/internal/xdsclient/transport/internal"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
)

func (s) TestNewWithGRPCDial(t *testing.T) {
	// Override the dialer with a custom one.
	customDialerCalled := false
	customDialer := func(target string, opts ...grpc.DialOption) (*grpc.ClientConn, error) {
		customDialerCalled = true
		return grpc.NewClient(target, opts...)
	}
	oldDial := internal.GRPCNewClient
	internal.GRPCNewClient = customDialer
	defer func() { internal.GRPCNewClient = oldDial }()

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: "server-address"})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}
	// Create a new transport and ensure that the custom dialer was called.
	opts := transport.Options{
		ServerCfg: serverCfg,
		NodeProto: &v3corepb.Node{},
		OnRecvHandler: func(update transport.ResourceUpdate, onDone func()) error {
			onDone()
			return nil
		},
		OnErrorHandler: func(error) {},
		OnSendHandler:  func(*transport.ResourceSendInfo) {},
	}
	c, err := transport.New(opts)
	if err != nil {
		t.Fatalf("transport.New(%v) failed: %v", opts, err)
	}
	defer c.Close()

	if !customDialerCalled {
		t.Fatalf("transport.New(%+v) custom dialer called = false, want true", opts)
	}
	customDialerCalled = false

	// Reset the dialer, create a new transport and ensure that our custom
	// dialer is no longer called.
	internal.GRPCNewClient = grpc.NewClient
	c, err = transport.New(opts)
	defer func() {
		if c != nil {
			c.Close()
		}
	}()
	if err != nil {
		t.Fatalf("transport.New(%v) failed: %v", opts, err)
	}

	if customDialerCalled {
		t.Fatalf("transport.New(%+v) custom dialer called = true, want false", opts)
	}
}
