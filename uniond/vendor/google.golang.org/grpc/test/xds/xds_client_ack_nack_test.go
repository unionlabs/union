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
 *
 */

package xds_test

import (
	"context"
	"fmt"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/resolver"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

// We are interested in LDS, RDS, CDS and EDS resources as part of the regular
// xDS flow on the client.
const wantResources = 4

// seenAllACKs returns true if the provided ackVersions map contains valid acks
// for all the resources that we are interested in. If `wantNonEmpty` is true,
// only non-empty ack versions are considered valid.
func seenAllACKs(acksVersions map[string]string, wantNonEmpty bool) bool {
	if len(acksVersions) != wantResources {
		return false
	}
	for _, ack := range acksVersions {
		if wantNonEmpty && ack == "" {
			return false
		}
	}
	return true
}

// TestClientResourceVersionAfterStreamRestart tests the scenario where the
// xdsClient's ADS stream to the management server gets broken. This test
// verifies that the version number on the initial request on the new stream
// indicates the most recent version seen by the client on the previous stream.
func (s) TestClientResourceVersionAfterStreamRestart(t *testing.T) {
	// Create a restartable listener which can close existing connections.
	l, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}
	lis := testutils.NewRestartableListener(l)

	// We depend on the fact that the management server assigns monotonically
	// increasing stream IDs starting at 1.
	const (
		idBeforeRestart = 1
		idAfterRestart  = 2
	)

	// Events of importance in the test, in the order in which they are expected
	// to happen.
	acksReceivedBeforeRestart := grpcsync.NewEvent()
	streamRestarted := grpcsync.NewEvent()
	acksReceivedAfterRestart := grpcsync.NewEvent()

	// Map from stream id to a map of resource type to resource version.
	ackVersionsMap := make(map[int64]map[string]string)
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		Listener: lis,
		OnStreamRequest: func(id int64, req *v3discoverypb.DiscoveryRequest) error {
			// Return early under the following circumstances:
			// - Received all the requests we wanted to see. This is to avoid
			//   any stray requests leading to test flakes.
			// - Request contains no resource names. Such requests are usually
			//   seen when the xdsclient is shutting down and is no longer
			//   interested in the resources that it had subscribed to earlier.
			if acksReceivedAfterRestart.HasFired() || len(req.GetResourceNames()) == 0 {
				return nil
			}
			// Create a stream specific map to store ack versions if this is the
			// first time we are seeing this stream id.
			if ackVersionsMap[id] == nil {
				ackVersionsMap[id] = make(map[string]string)
			}
			ackVersionsMap[id][req.GetTypeUrl()] = req.GetVersionInfo()
			// Prior to stream restart, we are interested only in non-empty
			// resource versions. The xdsclient first sends out requests with an
			// empty version string. After receipt of requested resource, it
			// sends out another request for the same resource, but this time
			// with a non-empty version string, to serve as an ACK.
			if seenAllACKs(ackVersionsMap[idBeforeRestart], true) {
				acksReceivedBeforeRestart.Fire()
			}
			// After stream restart, we expect the xdsclient to send out
			// requests with version string set to the previously ACKed
			// versions. If it sends out requests with empty version string, it
			// is a bug and we want this test to catch it.
			if seenAllACKs(ackVersionsMap[idAfterRestart], false) {
				acksReceivedAfterRestart.Fire()
			}
			return nil
		},
		OnStreamClosed: func(int64, *v3corepb.Node) {
			streamRestarted.Fire()
		},
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)

	// Create an xDS resolver with the above bootstrap configuration.
	var xdsResolver resolver.Builder
	if newResolver := internal.NewXDSResolverWithConfigForTesting; newResolver != nil {
		xdsResolver, err = newResolver.(func([]byte) (resolver.Builder, error))(bootstrapContents)
		if err != nil {
			t.Fatalf("Failed to create xDS resolver for testing: %v", err)
		}
	}

	server := stubserver.StartTestService(t, nil)
	defer server.Stop()

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
	if err := managementServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn and make a successful RPC.
	cc, err := grpc.NewClient(fmt.Sprintf("xds:///%s", serviceName), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithResolvers(xdsResolver))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}

	// A successful RPC means that the xdsclient received all requested
	// resources. The ACKs from the xdsclient may get a little delayed. So, we
	// need to wait for all ACKs to be received on the management server before
	// restarting the stream.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for all resources to be ACKed prior to stream restart")
	case <-acksReceivedBeforeRestart.Done():
	}

	// Stop the listener on the management server. This will cause the client to
	// backoff and recreate the stream.
	lis.Stop()

	// Wait for the stream to be closed on the server.
	<-streamRestarted.Done()

	// Restart the listener on the management server to be able to accept
	// reconnect attempts from the client.
	lis.Restart()

	// Wait for all the previously sent resources to be re-requested.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for all resources to be ACKed post stream restart")
	case <-acksReceivedAfterRestart.Done():
	}

	if diff := cmp.Diff(ackVersionsMap[idBeforeRestart], ackVersionsMap[idAfterRestart]); diff != "" {
		t.Fatalf("unexpected diff in ack versions before and after stream restart (-want, +got):\n%s", diff)
	}
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("rpc EmptyCall() failed: %v", err)
	}
}
