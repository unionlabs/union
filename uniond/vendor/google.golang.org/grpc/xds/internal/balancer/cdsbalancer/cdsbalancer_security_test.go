/*
 * Copyright 2020 gRPC authors.
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

package cdsbalancer

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/json"
	"fmt"
	"os"
	"strings"
	"testing"
	"unsafe"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/credentials/tls/certprovider"
	"google.golang.org/grpc/credentials/xds"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/stub"
	xdscredsinternal "google.golang.org/grpc/internal/credentials/xds"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/testdata"
	"google.golang.org/grpc/xds/internal/xdsclient"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3tlspb "github.com/envoyproxy/go-control-plane/envoy/extensions/transport_sockets/tls/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/credentials/tls/certprovider/pemfile" // Register the file watcher certificate provider plugin.
)

// testCCWrapper wraps a balancer.ClientConn and intercepts NewSubConn and
// returns the xDS handshake info back to the test for inspection.
type testCCWrapper struct {
	balancer.ClientConn
	handshakeInfoCh chan *xdscredsinternal.HandshakeInfo
}

// NewSubConn forwards the call to the underlying balancer.ClientConn, but
// before that, it validates the following:
//   - there is only one address in the addrs slice
//   - the single address contains xDS handshake information, which is then
//     pushed onto the handshakeInfoCh channel
func (tcc *testCCWrapper) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	if len(addrs) != 1 {
		return nil, fmt.Errorf("NewSubConn got %d addresses, want 1", len(addrs))
	}
	getHI := internal.GetXDSHandshakeInfoForTesting.(func(attr *attributes.Attributes) *unsafe.Pointer)
	hi := getHI(addrs[0].Attributes)
	if hi == nil {
		return nil, fmt.Errorf("NewSubConn got address without xDS handshake info")
	}

	sc, err := tcc.ClientConn.NewSubConn(addrs, opts)
	select {
	case tcc.handshakeInfoCh <- (*xdscredsinternal.HandshakeInfo)(*hi):
	default:
	}
	return sc, err
}

// Registers a wrapped cds LB policy for the duration of this test that retains
// all the functionality of the original cds LB policy, but overrides the
// NewSubConn method passed to the policy and makes the xDS handshake
// information passed to NewSubConn available to the test.
//
// Accepts as argument a channel onto which xDS handshake information passed to
// NewSubConn is written to.
func registerWrappedCDSPolicyWithNewSubConnOverride(t *testing.T, ch chan *xdscredsinternal.HandshakeInfo) {
	cdsBuilder := balancer.Get(cdsName)
	internal.BalancerUnregister(cdsBuilder.Name())
	var ccWrapper *testCCWrapper
	stub.Register(cdsBuilder.Name(), stub.BalancerFuncs{
		Init: func(bd *stub.BalancerData) {
			ccWrapper = &testCCWrapper{
				ClientConn:      bd.ClientConn,
				handshakeInfoCh: ch,
			}
			bd.Data = cdsBuilder.Build(ccWrapper, bd.BuildOptions)
		},
		ParseConfig: func(lbCfg json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
			return cdsBuilder.(balancer.ConfigParser).ParseConfig(lbCfg)
		},
		UpdateClientConnState: func(bd *stub.BalancerData, ccs balancer.ClientConnState) error {
			bal := bd.Data.(balancer.Balancer)
			return bal.UpdateClientConnState(ccs)
		},
		Close: func(bd *stub.BalancerData) {
			bal := bd.Data.(balancer.Balancer)
			bal.Close()
		},
	})
	t.Cleanup(func() { balancer.Register(cdsBuilder) })
}

// Common setup for security tests:
//   - creates an xDS client with the specified bootstrap configuration
//   - creates a manual resolver that specifies cds as the top-level LB policy
//   - creates a channel that uses the passed in client creds and the manual
//     resolver
//   - creates a test server that uses the passed in server creds
//
// Returns the following:
// - a client channel to make RPCs
// - address of the test backend server
func setupForSecurityTests(t *testing.T, bootstrapContents []byte, clientCreds, serverCreds credentials.TransportCredentials) (*grpc.ClientConn, string) {
	t.Helper()

	xdsClient, xdsClose, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatalf("Failed to create xDS client: %v", err)
	}
	t.Cleanup(xdsClose)

	// Create a manual resolver that configures the CDS LB policy as the
	// top-level LB policy on the channel.
	r := manual.NewBuilderWithScheme("whatever")
	jsonSC := fmt.Sprintf(`{
			"loadBalancingConfig":[{
				"cds_experimental":{
					"cluster": "%s"
				}
			}]
		}`, clusterName)
	scpr := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(jsonSC)
	state := xdsclient.SetClient(resolver.State{ServiceConfig: scpr}, xdsClient)
	r.InitialState(state)

	// Create a ClientConn with the specified transport credentials.
	cc, err := grpc.Dial(r.Scheme()+":///test.service", grpc.WithTransportCredentials(clientCreds), grpc.WithResolvers(r))
	if err != nil {
		t.Fatalf("Failed to dial local test server: %v", err)
	}
	t.Cleanup(func() { cc.Close() })

	// Start a test service backend with the specified transport credentials.
	sOpts := []grpc.ServerOption{}
	if serverCreds != nil {
		sOpts = append(sOpts, grpc.Creds(serverCreds))
	}
	server := stubserver.StartTestService(t, nil, sOpts...)
	t.Cleanup(server.Stop)

	return cc, server.Address
}

// Creates transport credentials to be used on the client side that rely on xDS
// to provide the security configuration. It falls back to insecure creds if no
// security information is received from the management server.
func xdsClientCredsWithInsecureFallback(t *testing.T) credentials.TransportCredentials {
	t.Helper()

	xdsCreds, err := xds.NewClientCredentials(xds.ClientOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatalf("Failed to create xDS credentials: %v", err)
	}
	return xdsCreds
}

// Creates transport credentials to be used on the server side from certificate
// files in testdata/x509.
//
// The certificate returned by this function has a CommonName of "test-server1".
func tlsServerCreds(t *testing.T) credentials.TransportCredentials {
	t.Helper()

	cert, err := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
	if err != nil {
		t.Fatalf("Failed to load server cert and key: %v", err)

	}
	pemData, err := os.ReadFile(testdata.Path("x509/client_ca_cert.pem"))
	if err != nil {
		t.Fatalf("Failed to read client CA cert: %v", err)
	}
	roots := x509.NewCertPool()
	roots.AppendCertsFromPEM(pemData)
	cfg := &tls.Config{
		Certificates: []tls.Certificate{cert},
		ClientCAs:    roots,
	}
	return credentials.NewTLS(cfg)
}

// Checks the AuthInfo available in the peer if it matches the expected security
// level of the connection.
func verifySecurityInformationFromPeer(t *testing.T, pr *peer.Peer, wantSecLevel e2e.SecurityLevel) {
	// This is not a true helper in the Go sense, because it does not perform
	// setup or cleanup tasks. Marking it a helper is to ensure that when the
	// test fails, the line information of the caller is outputted instead of
	// from here.
	//
	// And this function directly calls t.Fatalf() instead of returning an error
	// and letting the caller decide what to do with it. This is also OK since
	// all callers will simply end up calling t.Fatalf() with the returned
	// error, and can't add any contextual information of value to the error
	// message.
	t.Helper()

	switch wantSecLevel {
	case e2e.SecurityLevelNone:
		if pr.AuthInfo.AuthType() != "insecure" {
			t.Fatalf("AuthType() is %s, want insecure", pr.AuthInfo.AuthType())
		}
	case e2e.SecurityLevelMTLS:
		ai, ok := pr.AuthInfo.(credentials.TLSInfo)
		if !ok {
			t.Fatalf("AuthInfo type is %T, want %T", pr.AuthInfo, credentials.TLSInfo{})
		}
		if len(ai.State.PeerCertificates) != 1 {
			t.Fatalf("Number of peer certificates is %d, want 1", len(ai.State.PeerCertificates))
		}
		cert := ai.State.PeerCertificates[0]
		const wantCommonName = "test-server1"
		if cn := cert.Subject.CommonName; cn != wantCommonName {
			t.Fatalf("Common name in peer certificate is %s, want %s", cn, wantCommonName)
		}
	}
}

// Tests the case where xDS credentials are not in use, but the cds LB policy
// receives a Cluster update with security configuration. Verifies that the
// security configuration is not parsed by the cds LB policy by looking at the
// xDS handshake info passed to NewSubConn.
func (s) TestSecurityConfigWithoutXDSCreds(t *testing.T) {
	// Register a wrapped cds LB policy for the duration of this test that writes
	// the xDS handshake info passed to NewSubConn onto the given channel.
	handshakeInfoCh := make(chan *xdscredsinternal.HandshakeInfo, 1)
	registerWrappedCDSPolicyWithNewSubConnOverride(t, handshakeInfoCh)

	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with insecure creds talking to a test server with
	// insecure credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, insecure.NewCredentials(), nil)

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource is configured to return security configuration.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Ensure that the xDS handshake info passed to NewSubConn is empty.
	var gotHI *xdscredsinternal.HandshakeInfo
	select {
	case gotHI = <-handshakeInfoCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting to read handshake info passed to NewSubConn")
	}
	wantHI := xdscredsinternal.NewHandshakeInfo(nil, nil, nil, false)
	if !cmp.Equal(gotHI, wantHI) {
		t.Fatalf("NewSubConn got handshake info %+v, want %+v", gotHI, wantHI)
	}
}

// Tests the case where xDS credentials are in use, but the cds LB policy
// receives a Cluster update without security configuration. Verifies that the
// xDS handshake info passed to NewSubConn specified the use of fallback
// credentials.
func (s) TestNoSecurityConfigWithXDSCreds(t *testing.T) {
	// Register a wrapped cds LB policy for the duration of this test that writes
	// the xDS handshake info passed to NewSubConn onto the given channel.
	handshakeInfoCh := make(chan *xdscredsinternal.HandshakeInfo, 1)
	registerWrappedCDSPolicyWithNewSubConnOverride(t, handshakeInfoCh)

	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with xDS creds talking to a test server with
	// insecure credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, xdsClientCredsWithInsecureFallback(t), nil)

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource is not configured to return any security configuration.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made.
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}

	// Ensure that the xDS handshake info passed to NewSubConn is empty.
	var gotHI *xdscredsinternal.HandshakeInfo
	select {
	case gotHI = <-handshakeInfoCh:
	case <-ctx.Done():
		t.Fatal("Timeout when waiting to read handshake info passed to NewSubConn")
	}
	wantHI := xdscredsinternal.NewHandshakeInfo(nil, nil, nil, false)
	if !cmp.Equal(gotHI, wantHI) {
		t.Fatalf("NewSubConn got handshake info %+v, want %+v", gotHI, wantHI)
	}
	if !gotHI.UseFallbackCreds() {
		t.Fatal("NewSubConn got handshake info that does not specify the use of fallback creds")
	}
}

// Tests the case where the security config returned by the management server
// cannot be resolved based on the contents of the bootstrap config. Verifies
// that the cds LB policy puts the channel in TRANSIENT_FAILURE.
func (s) TestSecurityConfigNotFoundInBootstrap(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server,
	// and one that does not have certificate providers configuration.
	nodeID := uuid.New().String()
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
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

	// Create a grpc channel with xDS creds.
	cc, _ := setupForSecurityTests(t, bootstrapContents, xdsClientCredsWithInsecureFallback(t), nil)

	// Configure a cluster resource that contains security configuration, in the
	// management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)
}

// A certificate provider builder that returns a nil Provider from the starter
// func passed to certprovider.NewBuildableConfig().
type errCertProviderBuilder struct{}

const errCertProviderName = "err-cert-provider"

func (e errCertProviderBuilder) ParseConfig(any) (*certprovider.BuildableConfig, error) {
	// Returning a nil Provider simulates the case where an error is encountered
	// at the time of building the Provider.
	bc := certprovider.NewBuildableConfig(errCertProviderName, nil, func(certprovider.BuildOptions) certprovider.Provider { return nil })
	return bc, nil
}

func (e errCertProviderBuilder) Name() string {
	return errCertProviderName
}

func init() {
	certprovider.Register(errCertProviderBuilder{})
}

// Tests the case where the certprovider.Store returns an error when the cds LB
// policy attempts to build a certificate provider. Verifies that the cds LB
// policy puts the channel in TRANSIENT_FAILURE.
func (s) TestCertproviderStoreError(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server
	// and one that includes certificate providers configuration for
	// errCertProviderBuilder.
	nodeID := uuid.New().String()
	providerCfg := json.RawMessage(fmt.Sprintf(`{
		"plugin_name": "%s",
		"config": {}
	}`, errCertProviderName))
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node:                               []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		ServerListenerResourceNameTemplate: e2e.ServerListenerResourceNameTemplate,
		CertificateProviders:               map[string]json.RawMessage{e2e.ClientSideCertProviderInstance: providerCfg},
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create a grpc channel with xDS creds.
	cc, _ := setupForSecurityTests(t, bootstrapContents, xdsClientCredsWithInsecureFallback(t), nil)

	// Configure a cluster resource that contains security configuration, in the
	// management server.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)
}

// Tests the case where the cds LB policy receives security configuration as
// part of the Cluster resource that can be successfully resolved using the
// bootstrap file contents. Verifies that the connection between the client and
// the server is secure.
func (s) TestGoodSecurityConfig(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server
	// and one that includes certificate providers configuration.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with xDS creds talking to a test server with TLS
	// credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, xdsClientCredsWithInsecureFallback(t), tlsServerCreds(t))

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource is configured to return security configuration.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made over a secure connection.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelMTLS)
}

// Tests the case where the cds LB policy receives security configuration as
// part of the Cluster resource that contains a certificate provider instance
// that is missing in the bootstrap file. Verifies that the channel moves to
// TRANSIENT_FAILURE. Subsequently, the cds LB policy receives a cluster
// resource that contains a certificate provider that is present in the
// bootstrap file.  Verifies that the connection between the client and the
// server is secure.
func (s) TestSecurityConfigUpdate_BadToGood(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with xDS creds talking to a test server with TLS
	// credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, xdsClientCredsWithInsecureFallback(t), tlsServerCreds(t))

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource contains security configuration with a certificate
	// provider instance that is missing in the bootstrap configuration.
	cluster := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
	cluster.TransportSocket = &v3corepb.TransportSocket{
		Name: "envoy.transport_sockets.tls",
		ConfigType: &v3corepb.TransportSocket_TypedConfig{
			TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
				CommonTlsContext: &v3tlspb.CommonTlsContext{
					ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
						ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
							InstanceName: "unknown-certificate-provider-instance",
						},
					},
				},
			}),
		},
	}
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{cluster},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	testutils.AwaitState(ctx, t, cc, connectivity.TransientFailure)

	// Update the management server with a Cluster resource that contains a
	// certificate provider instance that is present in the bootstrap
	// configuration.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made over a secure connection.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelMTLS)
}

// Tests the case where the cds LB policy receives security configuration as
// part of the Cluster resource that can be successfully resolved using the
// bootstrap file contents. Verifies that the connection between the client and
// the server is secure. Subsequently, the cds LB policy receives a cluster
// resource without security configuration. Verifies that this results in the
// use of fallback credentials, which in this case is insecure creds.
func (s) TestSecurityConfigUpdate_GoodToFallback(t *testing.T) {
	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with xDS creds talking to a test server with TLS
	// credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, xdsClientCredsWithInsecureFallback(t), tlsServerCreds(t))

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource is configured to return security configuration.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made over a secure connection.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelMTLS)

	// Start a test service backend that does not expect a secure connection.
	insecureServer := stubserver.StartTestService(t, nil)
	t.Cleanup(insecureServer.Stop)

	// Update the resources in the management server to contain no security
	// configuration. This should result in the use of fallback credentials,
	// which is insecure in our case.
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, insecureServer.Address)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Wait for the connection to move to the new backend that expects
	// connections without security.
	for ctx.Err() == nil {
		if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
			t.Logf("EmptyCall() failed: %v", err)
		}
		if peer.Addr.String() == insecureServer.Address {
			break
		}
	}
	if ctx.Err() != nil {
		t.Fatal("Timed out when waiting for connection to switch to second backend")
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelNone)
}

// Tests the case where the cds LB policy receives security configuration as
// part of the Cluster resource that can be successfully resolved using the
// bootstrap file contents. Verifies that the connection between the client and
// the server is secure. Subsequently, the cds LB policy receives a cluster
// resource that is NACKed by the xDS client. Test verifies that the cds LB
// policy continues to use the previous good configuration, but the error from
// the xDS client is propagated to the child policy.
func (s) TestSecurityConfigUpdate_GoodToBad(t *testing.T) {
	// Register a wrapped clusterresolver LB policy (child policy of the cds LB
	// policy) for the duration of this test that makes the resolver error
	// pushed to it available to the test.
	_, resolverErrCh, _, _ := registerWrappedClusterResolverPolicy(t)

	// Spin up an xDS management server.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bc := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Create a grpc channel with xDS creds talking to a test server with TLS
	// credentials.
	cc, serverAddress := setupForSecurityTests(t, bc, xdsClientCredsWithInsecureFallback(t), tlsServerCreds(t))

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource is configured to return security configuration.
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelMTLS)},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that a successful RPC can be made over a secure connection.
	client := testgrpc.NewTestServiceClient(cc)
	peer := &peer.Peer{}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(peer)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelMTLS)

	// Configure cluster and endpoints resources in the management server. The
	// cluster resource contains security configuration with a certificate
	// provider instance that is missing in the bootstrap configuration.
	cluster := e2e.DefaultCluster(clusterName, serviceName, e2e.SecurityLevelNone)
	cluster.TransportSocket = &v3corepb.TransportSocket{
		Name: "envoy.transport_sockets.tls",
		ConfigType: &v3corepb.TransportSocket_TypedConfig{
			TypedConfig: testutils.MarshalAny(t, &v3tlspb.UpstreamTlsContext{
				CommonTlsContext: &v3tlspb.CommonTlsContext{
					ValidationContextType: &v3tlspb.CommonTlsContext_ValidationContextCertificateProviderInstance{
						ValidationContextCertificateProviderInstance: &v3tlspb.CommonTlsContext_CertificateProviderInstance{
							InstanceName: "unknown-certificate-provider-instance",
						},
					},
				},
			}),
		},
	}
	resources = e2e.UpdateOptions{
		NodeID:         nodeID,
		Clusters:       []*v3clusterpb.Cluster{cluster},
		Endpoints:      []*v3endpointpb.ClusterLoadAssignment{e2e.DefaultEndpoint(serviceName, "localhost", []uint32{testutils.ParsePort(t, serverAddress)})},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	const wantNACKErr = "instance name \"unknown-certificate-provider-instance\" missing in bootstrap configuration"
	select {
	case err := <-resolverErrCh:
		if !strings.Contains(err.Error(), wantNACKErr) {
			t.Fatalf("Child policy got resolver error: %v, want err: %v", err, wantNACKErr)
		}
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for resolver error to be pushed to the child policy")
	}

	// Verify that a successful RPC can be made over a secure connection.
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("EmptyCall() failed: %v", err)
	}
	verifySecurityInformationFromPeer(t, peer, e2e.SecurityLevelMTLS)
}
