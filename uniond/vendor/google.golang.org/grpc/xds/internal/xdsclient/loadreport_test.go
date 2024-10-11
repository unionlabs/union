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

package xdsclient

import (
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/uuid"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/testing/protocmp"

	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3lrspb "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
	"google.golang.org/protobuf/types/known/durationpb"
)

func (s) TestLRSClient(t *testing.T) {
	fs1, sCleanup, err := fakeserver.StartServer(nil)
	if err != nil {
		t.Fatalf("failed to start fake xDS server: %v", err)
	}
	defer sCleanup()

	nodeID := uuid.New().String()
	serverCfg1, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: fs1.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}
	bc := e2e.DefaultBootstrapContents(t, nodeID, fs1.Address)
	xdsC, closeFn, err := NewForTesting(OptionsForTesting{
		Name:               t.Name(),
		Contents:           bc,
		WatchExpiryTimeout: defaultTestWatchExpiryTimeout,
	})
	if err != nil {
		t.Fatalf("Failed to create an xDS client: %v", err)
	}
	defer closeFn()

	// Report to the same address should not create new ClientConn.
	store1, lrsCancel1 := xdsC.ReportLoad(serverCfg1)
	defer lrsCancel1()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if u, err := fs1.NewConnChan.Receive(ctx); err != nil {
		t.Errorf("unexpected timeout: %v, %v, want NewConn", u, err)
	}

	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if u, err := fs1.NewConnChan.Receive(sCtx); err != context.DeadlineExceeded {
		t.Errorf("unexpected NewConn: %v, %v, want channel recv timeout", u, err)
	}

	fs2, sCleanup2, err := fakeserver.StartServer(nil)
	if err != nil {
		t.Fatalf("failed to start fake xDS server: %v", err)
	}
	defer sCleanup2()

	// Report to a different address should create new ClientConn.
	serverCfg2, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: fs2.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}
	store2, lrsCancel2 := xdsC.ReportLoad(serverCfg2)
	defer lrsCancel2()
	if u, err := fs2.NewConnChan.Receive(ctx); err != nil {
		t.Errorf("unexpected timeout: %v, %v, want NewConn", u, err)
	}

	if store1 == store2 {
		t.Fatalf("got same store for different servers, want different")
	}

	if u, err := fs2.LRSRequestChan.Receive(ctx); err != nil {
		t.Errorf("unexpected timeout: %v, %v, want NewConn", u, err)
	}
	store2.PerCluster("cluster", "eds").CallDropped("test")

	// Send one resp to the client.
	fs2.LRSResponseChan <- &fakeserver.Response{
		Resp: &v3lrspb.LoadStatsResponse{
			SendAllClusters:       true,
			LoadReportingInterval: &durationpb.Duration{Nanos: 50000000},
		},
	}

	// Server should receive a req with the loads.
	u, err := fs2.LRSRequestChan.Receive(ctx)
	if err != nil {
		t.Fatalf("unexpected LRS request: %v, %v, want error canceled", u, err)
	}
	receivedLoad := u.(*fakeserver.Request).Req.(*v3lrspb.LoadStatsRequest).ClusterStats
	if len(receivedLoad) <= 0 {
		t.Fatalf("unexpected load received, want load for cluster, eds, dropped for test")
	}
	receivedLoad[0].LoadReportInterval = nil
	want := &v3endpointpb.ClusterStats{
		ClusterName:          "cluster",
		ClusterServiceName:   "eds",
		TotalDroppedRequests: 1,
		DroppedRequests:      []*v3endpointpb.ClusterStats_DroppedRequests{{Category: "test", DroppedCount: 1}},
	}
	if d := cmp.Diff(want, receivedLoad[0], protocmp.Transform()); d != "" {
		t.Fatalf("unexpected load received, want load for cluster, eds, dropped for test, diff (-want +got):\n%s", d)
	}

	// Cancel this load reporting stream, server should see error canceled.
	lrsCancel2()

	// Server should receive a stream canceled error. There may be additional
	// load reports from the client in the channel.
	for {
		u, err := fs2.LRSRequestChan.Receive(ctx)
		if err != nil {
			t.Fatalf("unexpected error while reading LRS request: %v", err)
		}
		// Ignore load reports sent before the stream was cancelled.
		if u.(*fakeserver.Request).Err == nil {
			continue
		}
		if status.Code(u.(*fakeserver.Request).Err) != codes.Canceled {
			t.Errorf("unexpected LRS request: %v, want error canceled", u)
		}
		break
	}
}
