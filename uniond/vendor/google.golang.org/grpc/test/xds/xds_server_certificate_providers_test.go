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

package xds_test

import (
	"context"
	"fmt"
	"net"
	"strconv"
	"testing"
	"time"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	xdscreds "google.golang.org/grpc/credentials/xds"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// Tests the case where the bootstrap configuration contains no certificate
// providers, and xDS credentials with an insecure fallback is specified at
// server creation time. The management server is configured to return a
// server-side xDS Listener resource with no security configuration. The test
// verifies that a gRPC client configured with insecure credentials is able to
// make RPCs to the backend. This ensures that the insecure fallback
// credentials are getting used on the server.
func (s) TestServerSideXDS_WithNoCertificateProvidersInBootstrap_Success(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer, nodeID, bootstrapContents, _ := setup.ManagementServerAndResolver(t)

	// Spin up an xDS-enabled gRPC server that uses xDS credentials with
	// insecure fallback, and the above bootstrap configuration.
	lis, cleanup := setupGRPCServer(t, bootstrapContents)
	defer cleanup()

	// Create an inbound xDS listener resource for the server side that does not
	// contain any security configuration. This should force the server-side
	// xdsCredentials to use fallback.
	host, port, err := hostPortFromListener(lis)
	if err != nil {
		t.Fatalf("Failed to retrieve host and port of server: %v", err)
	}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultServerListener(host, port, e2e.SecurityLevelNone, "routeName")},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a client that uses insecure creds and verify RPCs.
	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
}

// Tests the case where the bootstrap configuration contains no certificate
// providers, and xDS credentials with an insecure fallback is specified at
// server creation time. The management server is configured to return a
// server-side xDS Listener resource with mTLS security configuration. The xDS
// client is expected to NACK this resource because the certificate provider
// instance name specified in the Listener resource will not be present in the
// bootstrap file. The test verifies that server creation does not fail and that
// the xDS-enabled gRPC server does not enter "serving" mode.
func (s) TestServerSideXDS_WithNoCertificateProvidersInBootstrap_Failure(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Spin up an xDS management server that pushes on a channel when it
	// receives a NACK for an LDS response.
	nackCh := make(chan struct{}, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() != "type.googleapis.com/envoy.config.listener.v3.Listener" {
				return nil
			}
			if req.GetErrorDetail() == nil {
				return nil
			}
			select {
			case nackCh <- struct{}{}:
			default:
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration with no certificate providers.
	nodeID := uuid.New().String()
	bs, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node:                               []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		ServerListenerResourceNameTemplate: e2e.ServerListenerResourceNameTemplate,
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Configure xDS credentials with an insecure fallback to be used on the
	// server-side.
	creds, err := xdscreds.NewServerCredentials(xdscreds.ServerOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatal(err)
	}

	// Initialize an xDS-enabled gRPC server and register the stubServer on it.
	// Pass it a mode change server option that pushes on a channel the mode
	// changes to "not serving".
	servingModeCh := make(chan struct{})
	modeChangeOpt := xds.ServingModeCallback(func(addr net.Addr, args xds.ServingModeChangeArgs) {
		if args.Mode == connectivity.ServingModeServing {
			close(servingModeCh)
		}
	})
	server, err := xds.NewGRPCServer(grpc.Creds(creds), modeChangeOpt, xds.BootstrapContentsForTesting(bs))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	testgrpc.RegisterTestServiceServer(server, &testService{})
	defer server.Stop()

	// Create a local listener and pass it to Serve().
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}

	go func() {
		if err := server.Serve(lis); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()

	// Create an inbound xDS listener resource for the server side that contains
	// mTLS security configuration. Since the received certificate provider
	// instance name would be missing in the bootstrap configuration, this
	// resource is expected to NACKed by the xDS client.
	host, port, err := hostPortFromListener(lis)
	if err != nil {
		t.Fatalf("Failed to retrieve host and port of server: %v", err)
	}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{e2e.DefaultServerListener(host, port, e2e.SecurityLevelMTLS, "routeName")},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for the NACK from the xDS client.
	select {
	case <-nackCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for an NACK from the xDS client for the LDS response")
	}

	// Wait a short duration and ensure that the server does not enter "serving"
	// mode.
	select {
	case <-time.After(2 * defaultTestShortTimeout):
	case <-servingModeCh:
		t.Fatal("Server changed to serving mode when not expected to")
	}

	// Create a client that uses insecure creds and verify that RPCs don't
	// succeed.
	cc, err := grpc.NewClient(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc.Close()

	waitForFailedRPCWithStatus(ctx, t, cc, errAcceptAndClose)
}

// Tests the case where the bootstrap configuration contains one certificate
// provider, and xDS credentials with an insecure fallback is specified at
// server creation time. Two listeners are configured on the xDS-enabled gRPC
// server. The management server responds with two listener resources:
//  1. contains valid security configuration pointing to the certificate provider
//     instance specified in the bootstrap
//  2. contains invalid security configuration pointing to a non-existent
//     certificate provider instance
//
// The test verifies that an RPC to the first listener succeeds, while the
// second listener never moves to "serving" mode and RPCs to it fail.
func (s) TestServerSideXDS_WithValidAndInvalidSecurityConfiguration(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Spin up an xDS management server that pushes on a channel when it
	// receives a NACK for an LDS response.
	nackCh := make(chan struct{}, 1)
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() != "type.googleapis.com/envoy.config.listener.v3.Listener" {
				return nil
			}
			if req.GetErrorDetail() == nil {
				return nil
			}
			select {
			case nackCh <- struct{}{}:
			default:
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Create two local listeners.
	lis1, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	lis2, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}

	// Create an xDS-enabled grpc server that is configured to use xDS
	// credentials, and register the test service on it. Configure a mode change
	// option that closes a channel when listener2 enter serving mode.
	creds, err := xdscreds.NewServerCredentials(xdscreds.ServerOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatal(err)
	}
	servingModeCh := make(chan struct{})
	modeChangeOpt := xds.ServingModeCallback(func(addr net.Addr, args xds.ServingModeChangeArgs) {
		if addr.String() == lis2.Addr().String() {
			if args.Mode == connectivity.ServingModeServing {
				close(servingModeCh)
			}
		}
	})
	server, err := xds.NewGRPCServer(grpc.Creds(creds), modeChangeOpt, xds.BootstrapContentsForTesting(bootstrapContents))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	testgrpc.RegisterTestServiceServer(server, &testService{})
	defer server.Stop()

	go func() {
		if err := server.Serve(lis1); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()
	go func() {
		if err := server.Serve(lis2); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()

	// Create inbound xDS listener resources for the server side that contains
	// mTLS security configuration.
	// lis1 --> security configuration pointing to a valid cert provider
	// lis2 --> security configuration pointing to a non-existent cert provider
	host1, port1, err := hostPortFromListener(lis1)
	if err != nil {
		t.Fatalf("Failed to retrieve host and port of server: %v", err)
	}
	resource1 := e2e.DefaultServerListener(host1, port1, e2e.SecurityLevelMTLS, "routeName")
	host2, port2, err := hostPortFromListener(lis2)
	if err != nil {
		t.Fatalf("Failed to retrieve host and port of server: %v", err)
	}
	hcm := &v3httppb.HttpConnectionManager{
		RouteSpecifier: &v3httppb.HttpConnectionManager_RouteConfig{
			RouteConfig: &v3routepb.RouteConfiguration{
				Name: "routeName",
				VirtualHosts: []*v3routepb.VirtualHost{{
					Domains: []string{"*"},
					Routes: []*v3routepb.Route{{
						Match: &v3routepb.RouteMatch{
							PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/"},
						},
						Action: &v3routepb.Route_NonForwardingAction{},
					}}}}},
		},
		HttpFilters: []*v3httppb.HttpFilter{e2e.RouterHTTPFilter},
	}
	ts := &v3corepb.TransportSocket{
		Name: "envoy.transport_sockets.tls",
		ConfigType: &v3corepb.TransportSocket_TypedConfig{
			TypedConfig: testutils.MarshalAny(t, &v3tlspb.DownstreamTlsContext{
				RequireClientCertificate: &wrapperspb.BoolValue{Value: true},
				CommonTlsContext: &v3tlspb.CommonTlsContext{
					TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
						InstanceName: "non-existent-certificate-provider",
					},
					ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
						ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
							InstanceName: "non-existent-certificate-provider",
						},
					},
				},
			}),
		},
	}
	resource2 := &v3listenerpb.Listener{
		Name: fmt.Sprintf(e2e.ServerListenerResourceNameTemplate, net.JoinHostPort(host2, strconv.Itoa(int(port2)))),
		Address: &v3corepb.Address{
			Address: &v3corepb.Address_SocketAddress{
				SocketAddress: &v3corepb.SocketAddress{
					Address: host2,
					PortSpecifier: &v3corepb.SocketAddress_PortValue{
						PortValue: port2,
					},
				},
			},
		},
		FilterChains: []*v3listenerpb.FilterChain{
			{
				Name: "v4-wildcard",
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "0.0.0.0",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
					SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
					SourcePrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "0.0.0.0",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
				},
				Filters: []*v3listenerpb.Filter{
					{
						Name:       "filter-1",
						ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: testutils.MarshalAny(t, hcm)},
					},
				},
				TransportSocket: ts,
			},
			{
				Name: "v6-wildcard",
				FilterChainMatch: &v3listenerpb.FilterChainMatch{
					PrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "::",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
					SourceType: v3listenerpb.FilterChainMatch_SAME_IP_OR_LOOPBACK,
					SourcePrefixRanges: []*v3corepb.CidrRange{
						{
							AddressPrefix: "::",
							PrefixLen: &wrapperspb.UInt32Value{
								Value: uint32(0),
							},
						},
					},
				},
				Filters: []*v3listenerpb.Filter{
					{
						Name:       "filter-1",
						ConfigType: &v3listenerpb.Filter_TypedConfig{TypedConfig: testutils.MarshalAny(t, hcm)},
					},
				},
				TransportSocket: ts,
			},
		},
	}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{resource1, resource2},
		SkipValidation: true,
	}
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a client that uses TLS creds and verify RPCs to listener1.
	clientCreds := testutils.CreateClientTLSCredentials(t)
	cc1, err := grpc.NewClient(lis1.Addr().String(), grpc.WithTransportCredentials(clientCreds))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc1.Close()

	client1 := testgrpc.NewTestServiceClient(cc1)
	if _, err := client1.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Wait for the NACK from the xDS client.
	select {
	case <-nackCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for an NACK from the xDS client for the LDS response")
	}

	// Wait a short duration and ensure that the server does not enter "serving"
	// mode.
	select {
	case <-time.After(2 * defaultTestShortTimeout):
	case <-servingModeCh:
		t.Fatal("Server changed to serving mode when not expected to")
	}

	// Create a client that uses insecure creds and verify that RPCs don't
	// succeed to listener2.
	cc2, err := grpc.NewClient(lis2.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	defer cc2.Close()

	waitForFailedRPCWithStatus(ctx, t, cc2, errAcceptAndClose)
}
