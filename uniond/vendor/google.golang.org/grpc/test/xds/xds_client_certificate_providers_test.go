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
	"crypto/tls"
	"fmt"
	"strings"
	"testing"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	xdscreds "google.golang.org/grpc/credentials/xds"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/status"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// Tests the case where the bootstrap configuration contains no certificate
// providers, and xDS credentials with an insecure fallback is specified at dial
// time. The management server is configured to return client side xDS resources
// with no security configuration. The test verifies that the gRPC client is
// able to make RPCs to the backend which is configured to accept plaintext
// connections. This ensures that the insecure fallback credentials are getting
// used on the client.
func (s) TestClientSideXDS_WithNoCertificateProvidersInBootstrap_Success(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer, nodeID, _, resolverBuilder := setup.ManagementServerAndResolver(t)

	// Spin up a test backend.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Configure client side xDS resources on the management server, with no
	// security configuration in the Cluster resource.
	const serviceName = "my-service-client-side-xds"
	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		Port:       testutils.ParsePort(t, server.Address),
		SecLevel:   e2e.SecurityLevelNone,
	})
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create client-side xDS credentials with an insecure fallback.
	creds, err := xdscreds.NewClientCredentials(xdscreds.ClientOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(creds), grpc.WithResolvers(resolverBuilder))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
}

// Tests the case where the bootstrap configuration contains no certificate
// providers, and xDS credentials with an insecure fallback is specified at dial
// time. The management server is configured to return client side xDS resources
// with an mTLS security configuration. The test verifies that the gRPC client
// moves to TRANSIENT_FAILURE and rpcs fail with the expected error code and
// string. This ensures that when the certificate provider instance name
// specified in the security configuration is not present in the bootstrap,
// channel creation does not fail, but it moves to TRANSIENT_FAILURE and
// subsequent rpcs fail.
func (s) TestClientSideXDS_WithNoCertificateProvidersInBootstrap_Failure(t *testing.T) {
	// Start an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server,
	// with no certificate providers.
	nodeID := uuid.New().String()
	bc, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create an xDS resolver with the above bootstrap configuration.
	var resolverBuilder resolver.Builder
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		resolverBuilder, err = newResolver.(func([]byte) (resolver.Builder, error))(bc)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	// Spin up a test backend.
	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

	// Configure client side xDS resources on the management server, with mTLS
	// security configuration in the Cluster resource.
	const serviceName = "my-service-client-side-xds"
	const clusterName = "cluster-" + serviceName
	const endpointsName = "endpoints-" + serviceName
	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: serviceName,
		NodeID:     nodeID,
		Host:       "localhost",
		Port:       testutils.ParsePort(t, server.Address),
		SecLevel:   e2e.SecurityLevelNone,
	})
	resources.Clusters = []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, endpointsName, e2e.SecurityLevelMTLS)}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create client-side xDS credentials with an insecure fallback.
	creds, err := xdscreds.NewClientCredentials(xdscreds.ClientOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and ensure that it moves to TRANSIENT_FAILURE.
	cc, err := grpc.Dial(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(creds), grpc.WithResolvers(resolverBuilder))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()
	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Make an RPC and ensure that expected error is returned.
	wantErr := fmt.Sprintf("identity certificate provider instance name %q missing in bootstrap configuration", e2e.ClientSideCertProviderInstance)
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable || !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("EmptyCall() failed: %v, wantCode: %s, wantErr: %s", err, codes.Unavailable, wantErr)
	}
}

// Tests the case where the bootstrap configuration contains one certificate
// provider, and xDS credentials with an insecure fallback is specified at dial
// time. The management server responds with three clusters:
//  1. contains valid security configuration pointing to the certificate provider
//     instance specified in the bootstrap
//  2. contains no security configuration, hence should use insecure fallback
//  3. contains invalid security configuration pointing to a non-existent
//     certificate provider instance
//
// The test verifies that RPCs to the first two clusters succeed, while RPCs to
// the third cluster fails with an appropriate code and error message.
func (s) TestClientSideXDS_WithValidAndInvalidSecurityConfiguration(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{AllowResourceSubset: true})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create an xDS resolver with the above bootstrap configuration.
	var xdsResolver resolver.Builder
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		var err error
		xdsResolver, err = newResolver.(func([]byte) (resolver.Builder, error))(bc)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	// Create test backends for all three clusters
	// backend1 configured with TLS creds, represents cluster1
	// backend2 configured with insecure creds, represents cluster2
	// backend3 configured with insecure creds, represents cluster3
	serverCreds := testutils.CreateServerTLSCredentials(t, tls.RequireAndVerifyClientCert)
	server1 := stubserver.StartTestService(t, nil, grpc.Creds(serverCreds))
	defer server1.Stop()
	server2 := stubserver.StartTestService(t, nil)
	defer server2.Stop()
	server3 := stubserver.StartTestService(t, nil)
	defer server3.Stop()

	// Configure client side xDS resources on the management server.
	const serviceName = "my-service-client-side-xds"
	const routeConfigName = "route-" + serviceName
	const clusterName1 = "cluster1-" + serviceName
	const clusterName2 = "cluster2-" + serviceName
	const clusterName3 = "cluster3-" + serviceName
	const endpointsName1 = "endpoints1-" + serviceName
	const endpointsName2 = "endpoints2-" + serviceName
	const endpointsName3 = "endpoints3-" + serviceName
	listeners := []*v3listenerpb.Listener{e2e.DefaultClientListener(serviceName, routeConfigName)}
	// Route configuration:
	// - "/grpc.testing.TestService/EmptyCall" --> cluster1
	// - "/grpc.testing.TestService/UnaryCall" --> cluster2
	// - "/grpc.testing.TestService/FullDuplexCall" --> cluster3
	routes := []*v3routepb.RouteConfiguration{{
		Name: routeConfigName,
		VirtualHosts: []*v3routepb.VirtualHost{{
			Domains: []string{serviceName},
			Routes: []*v3routepb.Route{
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/grpc.testing.TestService/EmptyCall"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName1},
					}},
				},
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/grpc.testing.TestService/UnaryCall"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName2},
					}},
				},
				{
					Match: &v3routepb.RouteMatch{PathSpecifier: &v3routepb.RouteMatch_Prefix{Prefix: "/grpc.testing.TestService/FullDuplexCall"}},
					Action: &v3routepb.Route_Route{Route: &v3routepb.RouteAction{
						ClusterSpecifier: &v3routepb.RouteAction_Cluster{Cluster: clusterName3},
					}},
				},
			},
		}},
	}}
	// Clusters:
	// - cluster1 with cert provider name e2e.ClientSideCertProviderInstance.
	// - cluster2 with no security configuration.
	// - cluster3 with non-existent cert provider name.
	clusters := []*v3clusterpb.Cluster{
		e2e.DefaultCluster(clusterName1, endpointsName1, e2e.SecurityLevelMTLS),
		e2e.DefaultCluster(clusterName2, endpointsName2, e2e.SecurityLevelNone),
		func() *v3clusterpb.Cluster {
			cluster3 := e2e.DefaultCluster(clusterName3, endpointsName3, e2e.SecurityLevelMTLS)
			cluster3.TransportSocket = &v3corepb.TransportSocket{
				Name: "envoy.transport_sockets.tls",
				ConfigType: &v3corepb.TransportSocket_TypedConfig{
					TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
						CommonTlsContext: &v3tlspb.CommonTlsContext{
							ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
								ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
									InstanceName: "non-existent-certificate-provider-instance-name",
								},
							},
							TlsCertificateCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
								InstanceName: "non-existent-certificate-provider-instance-name",
							},
						},
					}),
				},
			}
			return cluster3
		}(),
	}
	// Endpoints for each of the above clusters with backends created earlier.
	endpoints := []*v3endpointpb.ClusterLoadAssignment{
		e2e.DefaultEndpoint(endpointsName1, "localhost", []uint32{testutils.ParsePort(t, server1.Address)}),
		e2e.DefaultEndpoint(endpointsName2, "localhost", []uint32{testutils.ParsePort(t, server2.Address)}),
	}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      listeners,
		Routes:         routes,
		Clusters:       clusters,
		Endpoints:      endpoints,
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create client-side xDS credentials with an insecure fallback.
	clientCreds, err := xdscreds.NewClientCredentials(xdscreds.ClientOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(clientCreds), grpc.WithResolvers(xdsResolver))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	// Make an RPC to be routed to cluster1 and verify that it succeeds.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	if got, want := peer.Addr.String(), server1.Address; got != want {
		t.Errorf("EmptyCall() routed to %q, want to be routed to: %q", got, want)

	}

	// Make an RPC to be routed to cluster2 and verify that it succeeds.
	if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}, grpc.Peer(peer)); err != nil {
		t.Fatalf("UnaryCall() failed: %v", err)
	}
	if got, want := peer.Addr.String(), server2.Address; got != want {
		t.Errorf("EmptyCall() routed to %q, want to be routed to: %q", got, want)
	}

	// Make an RPC to be routed to cluster3 and verify that it fails.
	const wantErr = `identity certificate provider instance name "non-existent-certificate-provider-instance-name" missing in bootstrap configuration`
	if _, err := client.FullDuplexCall(ctx); status.Code(err) != codes.Unavailable || !strings.Contains(err.Error(), wantErr) {
		t.Fatalf("FullDuplexCall failed: %v, wantCode: %s, wantErr: %s", err, codes.Unavailable, wantErr)
	}
}
