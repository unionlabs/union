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

// Package e2e provides utilities for end2end testing of xDS functionality.
package e2e

import (
	"context"
	"fmt"
	"net"
	"reflect"
	"strconv"
	"testing"

	"github.com/envoyproxy/go-control-plane/pkg/cache/types"
	"google.golang.org/grpc"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"

	v3clusterpb "github.com/envoyproxy/go-control-plane/envoy/config/cluster/v3"
	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3discoverygrpc "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3lrsgrpc "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
	v3cache "github.com/envoyproxy/go-control-plane/pkg/cache/v3"
	v3resource "github.com/envoyproxy/go-control-plane/pkg/resource/v3"
	v3server "github.com/envoyproxy/go-control-plane/pkg/server/v3"
)

// ManagementServer is a thin wrapper around the xDS control plane
// implementation provided by envoyproxy/go-control-plane.
type ManagementServer struct {
	// Address is the host:port on which the management server is listening for
	// new connections.
	Address string

	// LRSServer points to the fake LRS server implementation. Set only if the
	// SupportLoadReportingService option was set to true when creating this
	// management server.
	LRSServer *fakeserver.Server

	cancel  context.CancelFunc    // To stop the v3 ADS service.
	xs      v3server.Server       // v3 implementation of ADS.
	gs      *grpc.Server          // gRPC server which exports the ADS service.
	cache   v3cache.SnapshotCache // Resource snapshot.
	version int                   // Version of resource snapshot.

	// A logging interface, usually supplied from *testing.T.
	logger interface {
		Logf(format string, args ...any)
	}
}

// ManagementServerOptions contains options to be passed to the management
// server during creation.
type ManagementServerOptions struct {
	// Listener to accept connections on. If nil, a TPC listener on a local port
	// will be created and used.
	Listener net.Listener

	// SupportLoadReportingService, if set, results in the load reporting
	// service being registered on the same port as that of ADS.
	SupportLoadReportingService bool

	// AllowResourceSubSet allows the management server to respond to requests
	// before all configured resources are explicitly named in the request. The
	// default behavior that we want is for the management server to wait for
	// all configured resources to be requested before responding to any of
	// them, since this is how we have run our tests historically, and should be
	// set to true only for tests which explicitly require the other behavior.
	AllowResourceSubset bool

	// ServerFeaturesIgnoreResourceDeletion, if set, results in a bootstrap config
	// where the server features list contains `ignore_resource_deletion`. This
	// results in gRPC ignoring resource deletions from the management server, as
	// per A53.
	ServerFeaturesIgnoreResourceDeletion bool

	// The callbacks defined below correspond to the state of the world (sotw)
	// version of the xDS API on the management server.

	// OnStreamOpen is called when an xDS stream is opened. The callback is
	// invoked with the assigned stream ID and the type URL from the incoming
	// request (or "" for ADS).
	//
	// Returning an error from this callback will end processing and close the
	// stream. OnStreamClosed will still be called.
	OnStreamOpen func(context.Context, int64, string) error

	// OnStreamClosed is called immediately prior to closing an xDS stream. The
	// callback is invoked with the stream ID of the stream being closed.
	OnStreamClosed func(int64, *v3corepb.Node)

	// OnStreamRequest is called when a request is received on the stream. The
	// callback is invoked with the stream ID of the stream on which the request
	// was received and the received request.
	//
	// Returning an error from this callback will end processing and close the
	// stream. OnStreamClosed will still be called.
	OnStreamRequest func(int64, *v3discoverypb.DiscoveryRequest) error

	// OnStreamResponse is called immediately prior to sending a response on the
	// stream. The callback is invoked with the stream ID of the stream on which
	// the response is being sent along with the incoming request and the outgoing
	// response.
	OnStreamResponse func(context.Context, int64, *v3discoverypb.DiscoveryRequest, *v3discoverypb.DiscoveryResponse)
}

// StartManagementServer initializes a management server which implements the
// AggregatedDiscoveryService endpoint. The management server is initialized
// with no resources. Tests should call the Update() method to change the
// resource snapshot held by the management server, as per by the test logic.
//
// Registers a cleanup function on t to stop the management server.
func StartManagementServer(t *testing.T, opts ManagementServerOptions) *ManagementServer {
	t.Helper()

	// Create a snapshot cache. The first parameter to NewSnapshotCache()
	// controls whether the server should wait for all resources to be
	// explicitly named in the request before responding to any of them.
	wait := !opts.AllowResourceSubset
	cache := v3cache.NewSnapshotCache(wait, v3cache.IDHash{}, serverLogger{t})
	t.Logf("Created new snapshot cache...")

	lis := opts.Listener
	if lis == nil {
		var err error
		lis, err = net.Listen("tcp", "localhost:0")
		if err != nil {
			t.Fatalf("Failed to listen on localhost:0: %v", err)
		}
	}

	// Cancelling the context passed to the server is the only way of stopping it
	// at the end of the test.
	ctx, cancel := context.WithCancel(context.Background())
	callbacks := v3server.CallbackFuncs{
		StreamOpenFunc:     opts.OnStreamOpen,
		StreamClosedFunc:   opts.OnStreamClosed,
		StreamRequestFunc:  opts.OnStreamRequest,
		StreamResponseFunc: opts.OnStreamResponse,
	}

	// Create an xDS management server and register the ADS implementation
	// provided by it on a gRPC server.
	xs := v3server.NewServer(ctx, cache, callbacks)
	gs := grpc.NewServer()
	v3discoverygrpc.RegisterAggregatedDiscoveryServiceServer(gs, xs)
	t.Logf("Registered Aggregated Discovery Service (ADS)...")

	mgmtServer := &ManagementServer{
		Address: lis.Addr().String(),
		cancel:  cancel,
		version: 0,
		gs:      gs,
		xs:      xs,
		cache:   cache,
		logger:  t,
	}
	if opts.SupportLoadReportingService {
		lrs := fakeserver.NewServer(lis.Addr().String())
		v3lrsgrpc.RegisterLoadReportingServiceServer(gs, lrs)
		mgmtServer.LRSServer = lrs
		t.Logf("Registered Load Reporting Service (LRS)...")
	}

	// Start serving.
	go gs.Serve(lis)
	t.Logf("xDS management server serving at: %v...", lis.Addr().String())
	t.Cleanup(mgmtServer.Stop)
	return mgmtServer
}

// UpdateOptions wraps parameters to be passed to the Update() method.
type UpdateOptions struct {
	// NodeID is the id of the client to which this update is to be pushed.
	NodeID string
	// Endpoints, Clusters, Routes, and Listeners are the updated list of xds
	// resources for the server.  All must be provided with each Update.
	Endpoints []*v3endpointpb.ClusterLoadAssignment
	Clusters  []*v3clusterpb.Cluster
	Routes    []*v3routepb.RouteConfiguration
	Listeners []*v3listenerpb.Listener
	// SkipValidation indicates whether we want to skip validation (by not
	// calling snapshot.Consistent()). It can be useful for negative tests,
	// where we send updates that the client will NACK.
	SkipValidation bool
}

// Update changes the resource snapshot held by the management server, which
// updates connected clients as required.
func (s *ManagementServer) Update(ctx context.Context, opts UpdateOptions) error {
	s.version++

	// Create a snapshot with the passed in resources.
	resources := map[v3resource.Type][]types.Resource{
		v3resource.ListenerType: resourceSlice(opts.Listeners),
		v3resource.RouteType:    resourceSlice(opts.Routes),
		v3resource.ClusterType:  resourceSlice(opts.Clusters),
		v3resource.EndpointType: resourceSlice(opts.Endpoints),
	}
	snapshot, err := v3cache.NewSnapshot(strconv.Itoa(s.version), resources)
	if err != nil {
		return fmt.Errorf("failed to create new snapshot cache: %v", err)

	}
	if !opts.SkipValidation {
		if err := snapshot.Consistent(); err != nil {
			return fmt.Errorf("failed to create new resource snapshot: %v", err)
		}
	}
	s.logger.Logf("Created new resource snapshot...")

	// Update the cache with the new resource snapshot.
	if err := s.cache.SetSnapshot(ctx, opts.NodeID, snapshot); err != nil {
		return fmt.Errorf("failed to update resource snapshot in management server: %v", err)
	}
	s.logger.Logf("Updated snapshot cache with resource snapshot...")
	return nil
}

// Stop stops the management server.
func (s *ManagementServer) Stop() {
	if s.cancel != nil {
		s.cancel()
	}
	s.gs.Stop()
}

// resourceSlice accepts a slice of any type of proto messages and returns a
// slice of types.Resource.  Will panic if there is an input type mismatch.
func resourceSlice(i any) []types.Resource {
	v := reflect.ValueOf(i)
	rs := make([]types.Resource, v.Len())
	for i := 0; i < v.Len(); i++ {
		rs[i] = v.Index(i).Interface().(types.Resource)
	}
	return rs
}
