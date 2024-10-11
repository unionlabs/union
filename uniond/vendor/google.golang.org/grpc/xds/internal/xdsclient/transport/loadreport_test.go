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
	"context"
	"testing"
	"time"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/testutils/xds/fakeserver"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/transport"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/durationpb"

	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3lrspb "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
)

const (
	testLocality1 = `{"region":"test-region1"}`
	testLocality2 = `{"region":"test-region2"}`
	testKey1      = "test-key1"
	testKey2      = "test-key2"
)

var (
	toleranceCmpOpt   = cmpopts.EquateApprox(0, 1e-5)
	ignoreOrderCmpOpt = protocmp.FilterField(&v3endpointpb.ClusterStats{}, "upstream_locality_stats",
		cmpopts.SortSlices(func(a, b protocmp.Message) bool {
			return a.String() < b.String()
		}),
	)
)

func (s) TestReportLoad(t *testing.T) {
	// Create a fake xDS management server listening on a local port.
	mgmtServer, cleanup := startFakeManagementServer(t)
	defer cleanup()
	t.Logf("Started xDS management server on %s", mgmtServer.Address)

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: mgmtServer.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	// Create a transport to the fake management server.
	nodeProto := &v3corepb.Node{Id: uuid.New().String()}
	tr, err := transport.New(transport.Options{
		ServerCfg:      serverCfg,
		NodeProto:      nodeProto,
		OnRecvHandler:  noopRecvHandler,                                     // No ADS validation.
		OnErrorHandler: func(error) {},                                      // No ADS stream error handling.
		OnSendHandler:  func(*transport.ResourceSendInfo) {},                // No ADS stream update handling.
		Backoff:        func(int) time.Duration { return time.Duration(0) }, // No backoff.
	})
	if err != nil {
		t.Fatalf("Failed to create xDS transport: %v", err)
	}
	defer tr.Close()

	// Ensure that a new connection is made to the management server.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := mgmtServer.NewConnChan.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for a new connection to the management server: %v", err)
	}

	// Call the load reporting API, and ensure that an LRS stream is created.
	store1, cancelLRS1 := tr.ReportLoad()
	if err != nil {
		t.Fatalf("Failed to start LRS load reporting: %v", err)
	}
	if _, err := mgmtServer.LRSStreamOpenChan.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for LRS stream to be created: %v", err)
	}

	// Push some loads on the received store.
	store1.PerCluster("cluster1", "eds1").CallDropped("test")
	store1.PerCluster("cluster1", "eds1").CallStarted(testLocality1)
	store1.PerCluster("cluster1", "eds1").CallServerLoad(testLocality1, testKey1, 3.14)
	store1.PerCluster("cluster1", "eds1").CallServerLoad(testLocality1, testKey1, 2.718)
	store1.PerCluster("cluster1", "eds1").CallFinished(testLocality1, nil)
	store1.PerCluster("cluster1", "eds1").CallStarted(testLocality2)
	store1.PerCluster("cluster1", "eds1").CallServerLoad(testLocality2, testKey2, 1.618)
	store1.PerCluster("cluster1", "eds1").CallFinished(testLocality2, nil)

	// Ensure the initial request is received.
	req, err := mgmtServer.LRSRequestChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for initial LRS request: %v", err)
	}
	gotInitialReq := req.(*fakeserver.Request).Req.(*v3lrspb.LoadStatsRequest)
	nodeProto.ClientFeatures = []string{"envoy.lrs.supports_send_all_clusters"}
	wantInitialReq := &v3lrspb.LoadStatsRequest{Node: nodeProto}
	if diff := cmp.Diff(gotInitialReq, wantInitialReq, protocmp.Transform()); diff != "" {
		t.Fatalf("Unexpected diff in initial LRS request (-got, +want):\n%s", diff)
	}

	// Send a response from the server with a small deadline.
	mgmtServer.LRSResponseChan <- &fakeserver.Response{
		Resp: &v3lrspb.LoadStatsResponse{
			SendAllClusters:       true,
			LoadReportingInterval: &durationpb.Duration{Nanos: 50000000}, // 50ms
		},
	}

	// Ensure that loads are seen on the server.
	req, err = mgmtServer.LRSRequestChan.Receive(ctx)
	if err != nil {
		t.Fatalf("Timeout when waiting for LRS request with loads: %v", err)
	}
	gotLoad := req.(*fakeserver.Request).Req.(*v3lrspb.LoadStatsRequest).ClusterStats
	if l := len(gotLoad); l != 1 {
		t.Fatalf("Received load for %d clusters, want 1", l)
	}
	// This field is set by the client to indicate the actual time elapsed since
	// the last report was sent. We cannot deterministically compare this, and
	// we cannot use the cmpopts.IgnoreFields() option on proto structs, since
	// we already use the protocmp.Transform() which marshals the struct into
	// another message. Hence setting this field to nil is the best option here.
	gotLoad[0].LoadReportInterval = nil
	wantLoad := &v3endpointpb.ClusterStats{
		ClusterName:          "cluster1",
		ClusterServiceName:   "eds1",
		TotalDroppedRequests: 1,
		DroppedRequests:      []*v3endpointpb.ClusterStats_DroppedRequests{{Category: "test", DroppedCount: 1}},
		UpstreamLocalityStats: []*v3endpointpb.UpstreamLocalityStats{
			{
				Locality: &v3corepb.Locality{Region: "test-region1"},
				LoadMetricStats: []*v3endpointpb.EndpointLoadMetricStats{
					// TotalMetricValue is the aggregation of 3.14 + 2.718 = 5.858
					{MetricName: testKey1, NumRequestsFinishedWithMetric: 2, TotalMetricValue: 5.858}},
				TotalSuccessfulRequests: 1,
				TotalIssuedRequests:     1,
			},
			{
				Locality: &v3corepb.Locality{Region: "test-region2"},
				LoadMetricStats: []*v3endpointpb.EndpointLoadMetricStats{
					{MetricName: testKey2, NumRequestsFinishedWithMetric: 1, TotalMetricValue: 1.618}},
				TotalSuccessfulRequests: 1,
				TotalIssuedRequests:     1,
			},
		},
	}
	if diff := cmp.Diff(wantLoad, gotLoad[0], protocmp.Transform(), toleranceCmpOpt, ignoreOrderCmpOpt); diff != "" {
		t.Fatalf("Unexpected diff in LRS request (-got, +want):\n%s", diff)
	}

	// Make another call to the load reporting API, and ensure that a new LRS
	// stream is not created.
	store2, cancelLRS2 := tr.ReportLoad()
	if err != nil {
		t.Fatalf("Failed to start LRS load reporting: %v", err)
	}
	sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := mgmtServer.LRSStreamOpenChan.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatal("New LRS stream created when expected to use an existing one")
	}

	// Push more loads.
	store2.PerCluster("cluster2", "eds2").CallDropped("test")

	// Ensure that loads are seen on the server. We need a loop here because
	// there could have been some requests from the client in the time between
	// us reading the first request and now. Those would have been queued in the
	// request channel that we read out of.
	for {
		if ctx.Err() != nil {
			t.Fatalf("Timeout when waiting for new loads to be seen on the server")
		}

		req, err = mgmtServer.LRSRequestChan.Receive(ctx)
		if err != nil {
			continue
		}
		gotLoad = req.(*fakeserver.Request).Req.(*v3lrspb.LoadStatsRequest).ClusterStats
		if l := len(gotLoad); l != 1 {
			continue
		}
		gotLoad[0].LoadReportInterval = nil
		wantLoad := &v3endpointpb.ClusterStats{
			ClusterName:          "cluster2",
			ClusterServiceName:   "eds2",
			TotalDroppedRequests: 1,
			DroppedRequests:      []*v3endpointpb.ClusterStats_DroppedRequests{{Category: "test", DroppedCount: 1}},
		}
		if diff := cmp.Diff(wantLoad, gotLoad[0], protocmp.Transform()); diff != "" {
			t.Logf("Unexpected diff in LRS request (-got, +want):\n%s", diff)
			continue
		}
		break
	}

	// Cancel the first load reporting call, and ensure that the stream does not
	// close (because we have another call open).
	cancelLRS1()
	sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer sCancel()
	if _, err := mgmtServer.LRSStreamCloseChan.Receive(sCtx); err != context.DeadlineExceeded {
		t.Fatal("LRS stream closed when expected to stay open")
	}

	// Cancel the second load reporting call, and ensure the stream is closed.
	cancelLRS2()
	if _, err := mgmtServer.LRSStreamCloseChan.Receive(ctx); err != nil {
		t.Fatal("Timeout waiting for LRS stream to close")
	}

	// Calling the load reporting API again should result in the creation of a
	// new LRS stream. This ensures that creating and closing multiple streams
	// works smoothly.
	_, cancelLRS3 := tr.ReportLoad()
	if err != nil {
		t.Fatalf("Failed to start LRS load reporting: %v", err)
	}
	if _, err := mgmtServer.LRSStreamOpenChan.Receive(ctx); err != nil {
		t.Fatalf("Timeout when waiting for LRS stream to be created: %v", err)
	}
	cancelLRS3()
}
