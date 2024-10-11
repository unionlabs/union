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

package transport

import (
	"context"
	"errors"
	"fmt"
	"io"
	"time"

	"google.golang.org/grpc/internal/backoff"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/durationpb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3endpointpb "github.com/envoyproxy/go-control-plane/envoy/config/endpoint/v3"
	v3lrsgrpc "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
	v3lrspb "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
)

type lrsStream = v3lrsgrpc.LoadReportingService_StreamLoadStatsClient

// ReportLoad starts reporting loads to the management server the transport is
// configured to use.
//
// It returns a Store for the user to report loads and a function to cancel the
// load reporting.
func (t *Transport) ReportLoad() (*load.Store, func()) {
	t.lrsStartStream()
	return t.lrsStore, grpcsync.OnceFunc(func() { t.lrsStopStream() })
}

// lrsStartStream starts an LRS stream to the server, if none exists.
func (t *Transport) lrsStartStream() {
	t.lrsMu.Lock()
	defer t.lrsMu.Unlock()

	t.lrsRefCount++
	if t.lrsRefCount != 1 {
		// Return early if the stream has already been started.
		return
	}

	ctx, cancel := context.WithCancel(context.Background())
	t.lrsCancelStream = cancel

	// Create a new done channel everytime a new stream is created. This ensures
	// that we don't close the same channel multiple times (from lrsRunner()
	// goroutine) when multiple streams are created and closed.
	t.lrsRunnerDoneCh = make(chan struct{})
	go t.lrsRunner(ctx)
}

// lrsStopStream closes the LRS stream, if this is the last user of the stream.
func (t *Transport) lrsStopStream() {
	t.lrsMu.Lock()
	defer t.lrsMu.Unlock()

	t.lrsRefCount--
	if t.lrsRefCount != 0 {
		// Return early if the stream has other references.
		return
	}

	t.lrsCancelStream()
	t.logger.Infof("Stopping LRS stream")

	// Wait for the runner goroutine to exit. The done channel will be
	// recreated when a new stream is created.
	<-t.lrsRunnerDoneCh
}

// lrsRunner starts an LRS stream to report load data to the management server.
// It reports load at constant intervals (as configured by the management
// server) until the context is cancelled.
func (t *Transport) lrsRunner(ctx context.Context) {
	defer close(t.lrsRunnerDoneCh)

	// This feature indicates that the client supports the
	// LoadStatsResponse.send_all_clusters field in the LRS response.
	node := proto.Clone(t.nodeProto).(*v3corepb.Node)
	node.ClientFeatures = append(node.ClientFeatures, "envoy.lrs.supports_send_all_clusters")

	runLoadReportStream := func() error {
		// streamCtx is created and canceled in case we terminate the stream
		// early for any reason, to avoid gRPC-Go leaking the RPC's monitoring
		// goroutine.
		streamCtx, cancel := context.WithCancel(ctx)
		defer cancel()
		stream, err := v3lrsgrpc.NewLoadReportingServiceClient(t.cc).StreamLoadStats(streamCtx)
		if err != nil {
			t.logger.Warningf("Creating LRS stream to server %q failed: %v", t.serverURI, err)
			return nil
		}
		t.logger.Infof("Created LRS stream to server %q", t.serverURI)

		if err := t.sendFirstLoadStatsRequest(stream, node); err != nil {
			t.logger.Warningf("Sending first LRS request failed: %v", err)
			return nil
		}

		clusters, interval, err := t.recvFirstLoadStatsResponse(stream)
		if err != nil {
			t.logger.Warningf("Reading from LRS stream failed: %v", err)
			return nil
		}

		// We reset backoff state when we successfully receive at least one
		// message from the server.
		t.sendLoads(streamCtx, stream, clusters, interval)
		return backoff.ErrResetBackoff
	}
	backoff.RunF(ctx, runLoadReportStream, t.backoff)
}

func (t *Transport) sendLoads(ctx context.Context, stream lrsStream, clusterNames []string, interval time.Duration) {
	tick := time.NewTicker(interval)
	defer tick.Stop()
	for {
		select {
		case <-tick.C:
		case <-ctx.Done():
			return
		}
		if err := t.sendLoadStatsRequest(stream, t.lrsStore.Stats(clusterNames)); err != nil {
			t.logger.Warningf("Writing to LRS stream failed: %v", err)
			return
		}
	}
}

func (t *Transport) sendFirstLoadStatsRequest(stream lrsStream, node *v3corepb.Node) error {
	req := &v3lrspb.LoadStatsRequest{Node: node}
	if t.logger.V(perRPCVerbosityLevel) {
		t.logger.Infof("Sending initial LoadStatsRequest: %s", pretty.ToJSON(req))
	}
	err := stream.Send(req)
	if err == io.EOF {
		return getStreamError(stream)
	}
	return err
}

func (t *Transport) recvFirstLoadStatsResponse(stream lrsStream) ([]string, time.Duration, error) {
	resp, err := stream.Recv()
	if err != nil {
		return nil, 0, fmt.Errorf("failed to receive first LoadStatsResponse: %v", err)
	}
	if t.logger.V(perRPCVerbosityLevel) {
		t.logger.Infof("Received first LoadStatsResponse: %s", pretty.ToJSON(resp))
	}

	rInterval := resp.GetLoadReportingInterval()
	if rInterval.CheckValid() != nil {
		return nil, 0, fmt.Errorf("invalid load_reporting_interval: %v", err)
	}
	interval := rInterval.AsDuration()

	if resp.ReportEndpointGranularity {
		// TODO(easwars): Support per endpoint loads.
		return nil, 0, errors.New("lrs: endpoint loads requested, but not supported by current implementation")
	}

	clusters := resp.Clusters
	if resp.SendAllClusters {
		// Return nil to send stats for all clusters.
		clusters = nil
	}

	return clusters, interval, nil
}

func (t *Transport) sendLoadStatsRequest(stream lrsStream, loads []*load.Data) error {
	clusterStats := make([]*v3endpointpb.ClusterStats, 0, len(loads))
	for _, sd := range loads {
		droppedReqs := make([]*v3endpointpb.ClusterStats_DroppedRequests, 0, len(sd.Drops))
		for category, count := range sd.Drops {
			droppedReqs = append(droppedReqs, &v3endpointpb.ClusterStats_DroppedRequests{
				Category:     category,
				DroppedCount: count,
			})
		}
		localityStats := make([]*v3endpointpb.UpstreamLocalityStats, 0, len(sd.LocalityStats))
		for l, localityData := range sd.LocalityStats {
			lid, err := internal.LocalityIDFromString(l)
			if err != nil {
				return err
			}
			loadMetricStats := make([]*v3endpointpb.EndpointLoadMetricStats, 0, len(localityData.LoadStats))
			for name, loadData := range localityData.LoadStats {
				loadMetricStats = append(loadMetricStats, &v3endpointpb.EndpointLoadMetricStats{
					MetricName:                    name,
					NumRequestsFinishedWithMetric: loadData.Count,
					TotalMetricValue:              loadData.Sum,
				})
			}
			localityStats = append(localityStats, &v3endpointpb.UpstreamLocalityStats{
				Locality: &v3corepb.Locality{
					Region:  lid.Region,
					Zone:    lid.Zone,
					SubZone: lid.SubZone,
				},
				TotalSuccessfulRequests: localityData.RequestStats.Succeeded,
				TotalRequestsInProgress: localityData.RequestStats.InProgress,
				TotalErrorRequests:      localityData.RequestStats.Errored,
				TotalIssuedRequests:     localityData.RequestStats.Issued,
				LoadMetricStats:         loadMetricStats,
				UpstreamEndpointStats:   nil, // TODO: populate for per endpoint loads.
			})
		}

		clusterStats = append(clusterStats, &v3endpointpb.ClusterStats{
			ClusterName:           sd.Cluster,
			ClusterServiceName:    sd.Service,
			UpstreamLocalityStats: localityStats,
			TotalDroppedRequests:  sd.TotalDrops,
			DroppedRequests:       droppedReqs,
			LoadReportInterval:    durationpb.New(sd.ReportInterval),
		})
	}

	req := &v3lrspb.LoadStatsRequest{ClusterStats: clusterStats}
	if t.logger.V(perRPCVerbosityLevel) {
		t.logger.Infof("Sending LRS loads: %s", pretty.ToJSON(req))
	}
	err := stream.Send(req)
	if err == io.EOF {
		return getStreamError(stream)
	}
	return err
}

func getStreamError(stream lrsStream) error {
	for {
		if _, err := stream.Recv(); err != nil {
			return err
		}
	}
}
