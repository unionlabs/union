/*
 *
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
 *
 */

package xds

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/credentials/tls/certprovider"
	"google.golang.org/grpc/credentials/xds"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"

	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter
)

const (
	defaultTestTimeout          = 5 * time.Second
	defaultTestShortTimeout     = 10 * time.Millisecond
	nonExistentManagementServer = "non-existent-management-server"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type fakeGRPCServer struct {
	done              chan struct{}
	registerServiceCh *testutils.Channel
	serveCh           *testutils.Channel
}

func (f *fakeGRPCServer) RegisterService(*grpc.ServiceDesc, any) {
	f.registerServiceCh.Send(nil)
}

func (f *fakeGRPCServer) Serve(lis net.Listener) error {
	f.serveCh.Send(nil)
	<-f.done
	lis.Close()
	return nil
}

func (f *fakeGRPCServer) Stop() {
	close(f.done)
}
func (f *fakeGRPCServer) GracefulStop() {
	close(f.done)
}

func (f *fakeGRPCServer) GetServiceInfo() map[string]grpc.ServiceInfo {
	panic("implement me")
}

func newFakeGRPCServer() *fakeGRPCServer {
	return &fakeGRPCServer{
		done:              make(chan struct{}),
		registerServiceCh: testutils.NewChannel(),
		serveCh:           testutils.NewChannel(),
	}
}

func generateBootstrapContents(t *testing.T, nodeID, serverURI string) []byte {
	bs := e2e.DefaultBootstrapContents(t, nodeID, serverURI)
	return bs
}

func (s) TestNewServer_Success(t *testing.T) {
	xdsCreds, err := xds.NewServerCredentials(xds.ServerOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatalf("failed to create xds server credentials: %v", err)
	}

	tests := []struct {
		desc              string
		serverOpts        []grpc.ServerOption
		wantXDSCredsInUse bool
	}{
		{
			desc: "without_xds_creds",
			serverOpts: []grpc.ServerOption{
				grpc.Creds(insecure.NewCredentials()),
				BootstrapContentsForTesting(generateBootstrapContents(t, uuid.NewString(), nonExistentManagementServer)),
			},
		},
		{
			desc: "with_xds_creds",
			serverOpts: []grpc.ServerOption{
				grpc.Creds(xdsCreds),
				BootstrapContentsForTesting(generateBootstrapContents(t, uuid.NewString(), nonExistentManagementServer)),
			},
			wantXDSCredsInUse: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			// The xds package adds a couple of server options (unary and stream
			// interceptors) to the server options passed in by the user.
			wantServerOpts := len(test.serverOpts) + 2

			origNewGRPCServer := newGRPCServer
			newGRPCServer = func(opts ...grpc.ServerOption) grpcServer {
				if got := len(opts); got != wantServerOpts {
					t.Fatalf("%d ServerOptions passed to grpc.Server, want %d", got, wantServerOpts)
				}
				// Verify that the user passed ServerOptions are forwarded as is.
				if !reflect.DeepEqual(opts[2:], test.serverOpts) {
					t.Fatalf("got ServerOptions %v, want %v", opts[2:], test.serverOpts)
				}
				return grpc.NewServer(opts...)
			}
			defer func() {
				newGRPCServer = origNewGRPCServer
			}()

			s, err := NewGRPCServer(test.serverOpts...)
			if err != nil {
				t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
			}
			defer s.Stop()
		})
	}
}

func (s) TestNewServer_Failure(t *testing.T) {
	xdsCreds, err := xds.NewServerCredentials(xds.ServerOptions{FallbackCreds: insecure.NewCredentials()})
	if err != nil {
		t.Fatalf("failed to create xds server credentials: %v", err)
	}

	tests := []struct {
		desc       string
		serverOpts []grpc.ServerOption
		wantErr    string
	}{
		{
			desc:       "bootstrap env var not set",
			serverOpts: []grpc.ServerOption{grpc.Creds(xdsCreds)},
			wantErr:    "failed to get xDS bootstrap config",
		},
		{
			desc: "empty bootstrap config",
			serverOpts: []grpc.ServerOption{
				grpc.Creds(xdsCreds),
				BootstrapContentsForTesting([]byte(`{}`)),
			},
			wantErr: "xDS client creation failed",
		},
		{
			desc: "server_listener_resource_name_template is missing",
			serverOpts: []grpc.ServerOption{
				grpc.Creds(xdsCreds),
				func() grpc.ServerOption {
					bs, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
						Servers: []byte(fmt.Sprintf(`[{
							"server_uri": %q,
							"channel_creds": [{"type": "insecure"}]
						}]`, nonExistentManagementServer)),
						Node: []byte(fmt.Sprintf(`{"id": "%s"}`, uuid.New().String())),
						CertificateProviders: map[string]json.RawMessage{
							"cert-provider-instance": json.RawMessage("{}"),
						},
					})
					if err != nil {
						t.Fatalf("Failed to create bootstrap configuration: %v", err)
					}
					return BootstrapContentsForTesting(bs)
				}(),
			},
			wantErr: "missing server_listener_resource_name_template in the bootstrap configuration",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			s, err := NewGRPCServer(test.serverOpts...)
			if err == nil {
				s.Stop()
				t.Fatal("NewGRPCServer() succeeded when expected to fail")
			}
			if !strings.Contains(err.Error(), test.wantErr) {
				t.Fatalf("NewGRPCServer() failed with error: %v, want: %s", err, test.wantErr)
			}
		})
	}
}

func (s) TestRegisterService(t *testing.T) {
	fs := newFakeGRPCServer()

	origNewGRPCServer := newGRPCServer
	newGRPCServer = func(...grpc.ServerOption) grpcServer { return fs }
	defer func() { newGRPCServer = origNewGRPCServer }()

	s, err := NewGRPCServer(BootstrapContentsForTesting(generateBootstrapContents(t, uuid.NewString(), "non-existent-management-server")))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	defer s.Stop()

	s.RegisterService(&grpc.ServiceDesc{}, nil)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := fs.registerServiceCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when expecting RegisterService() to called on grpc.Server: %v", err)
	}
}

const (
	fakeProvider1Name = "fake-certificate-provider-1"
	fakeProvider2Name = "fake-certificate-provider-2"
)

var (
	fpb1, fpb2          *fakeProviderBuilder
	fakeProvider1Config json.RawMessage
	fakeProvider2Config json.RawMessage
)

func init() {
	fpb1 = &fakeProviderBuilder{
		name:    fakeProvider1Name,
		buildCh: testutils.NewChannel(),
	}
	fpb2 = &fakeProviderBuilder{
		name:    fakeProvider2Name,
		buildCh: testutils.NewChannel(),
	}
	certprovider.Register(fpb1)
	certprovider.Register(fpb2)

	fakeProvider1Config = json.RawMessage(fmt.Sprintf(`{
		"plugin_name": "%s",
		"config": "my fake config 1"
	}`, fakeProvider1Name))
	fakeProvider2Config = json.RawMessage(fmt.Sprintf(`{
		"plugin_name": "%s",
		"config": "my fake config 2"
	}`, fakeProvider2Name))
}

// fakeProviderBuilder builds new instances of fakeProvider and interprets the
// config provided to it as a string.
type fakeProviderBuilder struct {
	name    string
	buildCh *testutils.Channel
}

func (b *fakeProviderBuilder) ParseConfig(cfg any) (*certprovider.BuildableConfig, error) {
	var config string
	if err := json.Unmarshal(cfg.(json.RawMessage), &config); err != nil {
		return nil, fmt.Errorf("providerBuilder %s failed to unmarshal config: %v", b.name, cfg)
	}
	return certprovider.NewBuildableConfig(b.name, []byte(config), func(certprovider.BuildOptions) certprovider.Provider {
		b.buildCh.Send(nil)
		return &fakeProvider{
			Distributor: certprovider.NewDistributor(),
			config:      config,
		}
	}), nil
}

func (b *fakeProviderBuilder) Name() string {
	return b.name
}

// fakeProvider is an implementation of the Provider interface which provides a
// method for tests to invoke to push new key materials.
type fakeProvider struct {
	*certprovider.Distributor
	config string
}

// Close helps implement the Provider interface.
func (p *fakeProvider) Close() {
	p.Distributor.Stop()
}

func verifyCertProviderNotCreated() error {
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := fpb1.buildCh.Receive(sCtx); err != context.DeadlineExceeded {
		return errors.New("certificate provider created when no xDS creds were specified")
	}
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := fpb2.buildCh.Receive(sCtx); err != context.DeadlineExceeded {
		return errors.New("certificate provider created when no xDS creds were specified")
	}
	return nil
}

func hostPortFromListener(t *testing.T, lis net.Listener) (string, uint32) {
	t.Helper()

	host, p, err := net.SplitHostPort(lis.Addr().String())
	if err != nil {
		t.Fatalf("net.SplitHostPort(%s) failed: %v", lis.Addr().String(), err)
	}
	port, err := strconv.ParseInt(p, 10, 32)
	if err != nil {
		t.Fatalf("strconv.ParseInt(%s, 10, 32) failed: %v", p, err)
	}
	return host, uint32(port)
}

// TestServeSuccess tests the successful case of creating an xDS enabled gRPC
// server and calling Serve() on it. The test verifies that an LDS request is
// sent out for the expected name, and also verifies that the serving mode
// changes appropriately.
func (s) TestServeSuccess(t *testing.T) {
	// Setup an xDS management server that pushes on a channel when an LDS
	// request is received by it.
	ldsRequestCh := make(chan []string, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() == version.V3ListenerURL {
				select {
				case ldsRequestCh <- req.GetResourceNames():
				default:
				}
			}
			return nil
		},
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	// Override the function to create the underlying grpc.Server to allow the
	// test to verify that Serve() is called on the underlying server.
	fs := newFakeGRPCServer()
	origNewGRPCServer := newGRPCServer
	newGRPCServer = func(...grpc.ServerOption) grpcServer { return fs }
	defer func() { newGRPCServer = origNewGRPCServer }()

	// Create a new xDS enabled gRPC server and pass it a server option to get
	// notified about serving mode changes.
	modeChangeCh := testutils.NewChannel()
	modeChangeOption := ServingModeCallback(func(addr net.Addr, args ServingModeChangeArgs) {
		t.Logf("Server mode change callback invoked for listener %q with mode %q and error %v", addr.String(), args.Mode, args.Err)
		modeChangeCh.Send(args.Mode)
	})
	server, err := NewGRPCServer(modeChangeOption, BootstrapContentsForTesting(bootstrapContents))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	defer server.Stop()

	// Call Serve() in a goroutine.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	go func() {
		if err := server.Serve(lis); err != nil {
			t.Error(err)
		}
	}()

	// Ensure that the LDS request is sent out for the expected name.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	var gotNames []string
	select {
	case gotNames = <-ldsRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout when waiting for an LDS request to be sent out")
	}
	wantNames := []string{strings.ReplaceAll(e2e.ServerListenerResourceNameTemplate, "%s", lis.Addr().String())}
	if !cmp.Equal(gotNames, wantNames) {
		t.Fatalf("LDS watch registered for names %v, want %v", gotNames, wantNames)
	}

	// Update the management server with a good listener resource.
	host, port := hostPortFromListener(t, lis)
	resources := e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*v3listenerpb.Listener{e2e.DefaultServerListener(host, port, e2e.SecurityLevelNone, "routeName")},
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify the serving mode reports SERVING.
	v, err := modeChangeCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for serving mode to change: %v", err)
	}
	if mode := v.(connectivity.ServingMode); mode != connectivity.ServingModeServing {
		t.Fatalf("Serving mode is %q, want %q", mode, connectivity.ServingModeServing)
	}

	// Verify that Serve() is called on the underlying gRPC server.
	if _, err := fs.serveCh.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for Serve() to be invoked on the grpc.Server")
	}

	// Update the listener resource on the management server in such a way that
	// it will be NACKed by our xDS client. The listener_filters field is
	// unsupported and will be NACKed.
	resources.Listeners[0].ListenerFilters = []*v3listenerpb.ListenerFilter{{Name: "foo"}}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify that there is no change in the serving mode. The server should
	// continue using the previously received good configuration.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if v, err := modeChangeCh.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatalf("Unexpected change in serving mode. New mode is %v", v.(connectivity.ServingMode))
	}

	// Remove the listener resource from the management server. This should
	// result in a resource-not-found error from the xDS client and should
	// result in the server moving to NOT_SERVING mode.
	resources.Listeners = nil
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	v, err = modeChangeCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for serving mode to change: %v", err)
	}
	if mode := v.(connectivity.ServingMode); mode != connectivity.ServingModeNotServing {
		t.Fatalf("Serving mode is %q, want %q", mode, connectivity.ServingModeNotServing)
	}
}

// TestNewServer_ClientCreationFailure tests the case where the xDS client
// creation fails and verifies that the call to NewGRPCServer() fails.
func (s) TestNewServer_ClientCreationFailure(t *testing.T) {
	origNewXDSClient := newXDSClient
	newXDSClient = func(string) (xdsclient.XDSClient, func(), error) {
		return nil, nil, errors.New("xdsClient creation failed")
	}
	defer func() { newXDSClient = origNewXDSClient }()

	if _, err := NewGRPCServer(); err == nil {
		t.Fatal("NewGRPCServer() succeeded when expected to fail")
	}
}

// TestHandleListenerUpdate_NoXDSCreds tests the case where an xds-enabled gRPC
// server is not configured with xDS credentials. Verifies that the security
// config received as part of a Listener update is not acted upon.
func (s) TestHandleListenerUpdate_NoXDSCreds(t *testing.T) {
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Generate bootstrap configuration pointing to the above management server
	// with certificate provider configuration pointing to fake certificate
	// providers.
	nodeID := uuid.NewString()
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		CertificateProviders: map[string]json.RawMessage{
			e2e.ServerSideCertProviderInstance: fakeProvider1Config,
			e2e.ClientSideCertProviderInstance: fakeProvider2Config,
		},
		ServerListenerResourceNameTemplate: e2e.ServerListenerResourceNameTemplate,
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create a new xDS enabled gRPC server and pass it a server option to get
	// notified about serving mode changes. Also pass the above bootstrap
	// configuration to be used during xDS client creation.
	modeChangeCh := testutils.NewChannel()
	modeChangeOption := ServingModeCallback(func(addr net.Addr, args ServingModeChangeArgs) {
		t.Logf("Server mode change callback invoked for listener %q with mode %q and error %v", addr.String(), args.Mode, args.Err)
		modeChangeCh.Send(args.Mode)
	})
	server, err := NewGRPCServer(modeChangeOption, BootstrapContentsForTesting(bootstrapContents))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	defer server.Stop()

	// Call Serve() in a goroutine.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	go func() {
		if err := server.Serve(lis); err != nil {
			t.Error(err)
		}
	}()

	// Update the management server with a good listener resource that contains
	// security configuration.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	host, port := hostPortFromListener(t, lis)
	resources := e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*v3listenerpb.Listener{e2e.DefaultServerListener(host, port, e2e.SecurityLevelMTLS, "routeName")},
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Verify the serving mode reports SERVING.
	v, err := modeChangeCh.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for serving mode to change: %v", err)
	}
	if mode := v.(connectivity.ServingMode); mode != connectivity.ServingModeServing {
		t.Fatalf("Serving mode is %q, want %q", mode, connectivity.ServingModeServing)
	}

	// Make sure the security configuration is not acted upon.
	if err := verifyCertProviderNotCreated(); err != nil {
		t.Fatal(err)
	}
}

// TestHandleListenerUpdate_ErrorUpdate tests the case where an xds-enabled gRPC
// server is configured with xDS credentials, but receives a Listener update
// with an error. Verifies that no certificate providers are created.
func (s) TestHandleListenerUpdate_ErrorUpdate(t *testing.T) {
	// Setup an xDS management server that pushes on a channel when an LDS
	// request is received by it.
	ldsRequestCh := make(chan []string, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			if req.GetTypeUrl() == version.V3ListenerURL {
				select {
				case ldsRequestCh <- req.GetResourceNames():
				default:
				}
			}
			return nil
		},
	})

	// Generate bootstrap configuration pointing to the above management server
	// with certificate provider configuration pointing to fake certificate
	// providers.
	nodeID := uuid.New().String()
	bootstrapContents, err := bootstrap.NewContentsForTesting(bootstrap.ConfigOptionsForTesting{
		Servers: []byte(fmt.Sprintf(`[{
			"server_uri": %q,
			"channel_creds": [{"type": "insecure"}]
		}]`, mgmtServer.Address)),
		Node: []byte(fmt.Sprintf(`{"id": "%s"}`, nodeID)),
		CertificateProviders: map[string]json.RawMessage{
			e2e.ServerSideCertProviderInstance: fakeProvider1Config,
			e2e.ClientSideCertProviderInstance: fakeProvider2Config,
		},
		ServerListenerResourceNameTemplate: e2e.ServerListenerResourceNameTemplate,
	})
	if err != nil {
		t.Fatalf("Failed to create bootstrap configuration: %v", err)
	}

	// Create a new xDS enabled gRPC server and pass it a server option to get
	// notified about serving mode changes. Also pass the above bootstrap
	// configuration to be used during xDS client creation.
	modeChangeCh := testutils.NewChannel()
	modeChangeOption := ServingModeCallback(func(addr net.Addr, args ServingModeChangeArgs) {
		t.Logf("Server mode change callback invoked for listener %q with mode %q and error %v", addr.String(), args.Mode, args.Err)
		modeChangeCh.Send(args.Mode)
	})
	server, err := NewGRPCServer(modeChangeOption, BootstrapContentsForTesting(bootstrapContents))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}
	defer server.Stop()

	// Call Serve() in a goroutine.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	go server.Serve(lis)

	// Update the listener resource on the management server in such a way that
	// it will be NACKed by our xDS client. The listener_filters field is
	// unsupported and will be NACKed.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	host, port := hostPortFromListener(t, lis)
	listener := e2e.DefaultServerListener(host, port, e2e.SecurityLevelMTLS, "routeName")
	listener.ListenerFilters = []*v3listenerpb.ListenerFilter{{Name: "foo"}}
	resources := e2e.UpdateOptions{
		NodeID:    nodeID,
		Listeners: []*v3listenerpb.Listener{listener},
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Ensure that the LDS request is sent out for the expected name.
	var gotNames []string
	select {
	case gotNames = <-ldsRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout when waiting for an LDS request to be sent out")
	}
	wantNames := []string{strings.ReplaceAll(e2e.ServerListenerResourceNameTemplate, "%s", lis.Addr().String())}
	if !cmp.Equal(gotNames, wantNames) {
		t.Fatalf("LDS watch registered for names %v, want %v", gotNames, wantNames)
	}

	// Make sure that no certificate providers are created.
	if err := verifyCertProviderNotCreated(); err != nil {
		t.Fatal(err)
	}

	// Also make sure that no serving mode updates are received. The serving
	// mode does not change until the server comes to the conclusion that the
	// requested resource is not present in the management server. This happens
	// when the watch timer expires or when the resource is explicitly deleted
	// by the management server.
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := modeChangeCh.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatal("Serving mode changed received when none expected")
	}
}

// TestServeReturnsErrorAfterClose tests that the xds Server returns
// grpc.ErrServerStopped if Serve is called after Close on the server.
func (s) TestServeReturnsErrorAfterClose(t *testing.T) {
	server, err := NewGRPCServer(BootstrapContentsForTesting(generateBootstrapContents(t, uuid.NewString(), nonExistentManagementServer)))
	if err != nil {
		t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
	}

	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	server.Stop()
	err = server.Serve(lis)
	if err == nil || !strings.Contains(err.Error(), grpc.ErrServerStopped.Error()) {
		t.Fatalf("server erorred with wrong error, want: %v, got :%v", grpc.ErrServerStopped, err)
	}
}

// TestServeAndCloseDoNotRace tests that Serve and Close on the xDS Server do
// not race and leak the xDS Client. A leak would be found by the leak checker.
func (s) TestServeAndCloseDoNotRace(t *testing.T) {
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}

	wg := sync.WaitGroup{}
	for i := 0; i < 100; i++ {
		server, err := NewGRPCServer(BootstrapContentsForTesting(generateBootstrapContents(t, uuid.NewString(), nonExistentManagementServer)))
		if err != nil {
			t.Fatalf("Failed to create an xDS enabled gRPC server: %v", err)
		}
		wg.Add(1)
		go func() {
			server.Serve(lis)
			wg.Done()
		}()
		wg.Add(1)
		go func() {
			server.Stop()
			wg.Done()
		}()
	}
	wg.Wait()
}
