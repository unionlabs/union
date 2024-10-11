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
	"errors"
	"fmt"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/transport"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	statuspb "google.golang.org/genproto/googleapis/rpc/status"
)

var (
	errWantNack = errors.New("unsupported field 'use_original_dst' is present and set to true")

	// A simple update handler for listener resources which validates only the
	// `use_original_dst` field.
	dataModelValidator = func(update transport.ResourceUpdate, onDone func()) error {
		defer onDone()
		for _, r := range update.Resources {
			inner := &v3discoverypb.Resource{}
			if err := proto.Unmarshal(r.GetValue(), inner); err != nil {
				return fmt.Errorf("failed to unmarshal DiscoveryResponse: %v", err)
			}
			lis := &v3listenerpb.Listener{}
			if err := proto.Unmarshal(r.GetValue(), lis); err != nil {
				return fmt.Errorf("failed to unmarshal DiscoveryResponse: %v", err)
			}
			if useOrigDst := lis.GetUseOriginalDst(); useOrigDst != nil && useOrigDst.GetValue() {
				return errWantNack
			}
		}
		return nil
	}
)

// TestSimpleAckAndNack tests simple ACK and NACK scenarios.
//  1. When the data model layer likes a received response, the test verifies
//     that an ACK is sent matching the version and nonce from the response.
//  2. When a subsequent response is disliked by the data model layer, the test
//     verifies that a NACK is sent matching the previously ACKed version and
//     current nonce from the response.
//  3. When a subsequent response is liked by the data model layer, the test
//     verifies that an ACK is sent matching the version and nonce from the
//     current response.
func (s) TestSimpleAckAndNack(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Create an xDS management server listening on a local port. Configure the
	// request and response handlers to push on channels which are inspected by
	// the test goroutine to verify ack version and nonce.
	streamRequestCh := make(chan *v3discoverypb.DiscoveryRequest, 1)
	streamResponseCh := make(chan *v3discoverypb.DiscoveryResponse, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			select {
			case streamRequestCh <- req:
			case <-ctx.Done():
			}
			return nil
		},
		OnStreamResponse: func(_ context.Context, _ int64, _ *v3discoverypb.DiscoveryRequest, resp *v3discoverypb.DiscoveryResponse) {
			select {
			case streamResponseCh <- resp:
			case <-ctx.Done():
			}
		},
	})

	// Configure the management server with appropriate resources.
	apiListener := &v3listenerpb.ApiListener{
		ApiListener: func() *anypb.Any {
			return testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
				RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
					Rds: &v3httppb.Rds{
						ConfigSource: &v3corepb.ConfigSource{
							ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
						},
						RouteConfigName: "route-configuration-name",
					},
				},
			})
		}(),
	}
	const resourceName = "resource name 1"
	listenerResource := &v3listenerpb.Listener{
		Name:        resourceName,
		ApiListener: apiListener,
	}
	nodeID := uuid.New().String()
	mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listenerResource},
		SkipValidation: true,
	})

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: mgmtServer.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	// Create a new transport.
	tr, err := transport.New(transport.Options{
		ServerCfg:      serverCfg,
		OnRecvHandler:  dataModelValidator,
		OnErrorHandler: func(err error) {},
		OnSendHandler:  func(*transport.ResourceSendInfo) {},
		NodeProto:      &v3corepb.Node{Id: nodeID},
	})
	if err != nil {
		t.Fatalf("Failed to create xDS transport: %v", err)
	}
	defer tr.Close()

	// Send a discovery request through the transport.
	tr.SendRequest(version.V3ListenerURL, []string{resourceName})

	// Verify that the initial discovery request matches expectation.
	var gotReq *v3discoverypb.DiscoveryRequest
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery request on the stream")
	}
	wantReq := &v3discoverypb.DiscoveryRequest{
		VersionInfo:   "",
		Node:          &v3corepb.Node{Id: nodeID},
		ResourceNames: []string{resourceName},
		TypeUrl:       "type.googleapis.com/envoy.config.listener.v3.Listener",
		ResponseNonce: "",
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Capture the version and nonce from the response.
	var gotResp *v3discoverypb.DiscoveryResponse
	select {
	case gotResp = <-streamResponseCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery response on the stream")
	}

	// Verify that the ACK contains the appropriate version and nonce.
	wantReq.VersionInfo = gotResp.GetVersionInfo()
	wantReq.ResponseNonce = gotResp.GetNonce()
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for the discovery request ACK on the stream")
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Update the management server's copy of the resource to include a field
	// which will cause the resource to be NACKed.
	badListener := proto.Clone(listenerResource).(*v3listenerpb.Listener)
	badListener.UseOriginalDst = &wrapperspb.BoolValue{Value: true}
	mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{badListener},
		SkipValidation: true,
	})

	select {
	case gotResp = <-streamResponseCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery response on the stream")
	}

	// Verify that the NACK contains the appropriate version, nonce and error.
	// We expect the version to not change as this is a NACK.
	wantReq.ResponseNonce = gotResp.GetNonce()
	wantReq.ErrorDetail = &statuspb.Status{
		Code:    int32(codes.InvalidArgument),
		Message: errWantNack.Error(),
	}
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for the discovery request ACK on the stream")
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Update the management server to send a good resource again.
	mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listenerResource},
		SkipValidation: true,
	})

	// The envoy-go-control-plane management server keeps resending the same
	// resource as long as we keep NACK'ing it. So, we will see the bad resource
	// sent to us a few times here, before receiving the good resource.
	for {
		select {
		case gotResp = <-streamResponseCh:
		case <-ctx.Done():
			t.Fatalf("Timeout waiting for discovery response on the stream")
		}

		// Verify that the ACK contains the appropriate version and nonce.
		wantReq.VersionInfo = gotResp.GetVersionInfo()
		wantReq.ResponseNonce = gotResp.GetNonce()
		wantReq.ErrorDetail = nil
		select {
		case gotReq = <-streamRequestCh:
		case <-ctx.Done():
			t.Fatalf("Timeout waiting for the discovery request ACK on the stream")
		}
		diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort))
		if diff == "" {
			break
		}
		t.Logf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}
}

// TestInvalidFirstResponse tests the case where the first response is invalid.
// The test verifies that the NACK contains an empty version string.
func (s) TestInvalidFirstResponse(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Create an xDS management server listening on a local port. Configure the
	// request and response handlers to push on channels which are inspected by
	// the test goroutine to verify ack version and nonce.
	streamRequestCh := make(chan *v3discoverypb.DiscoveryRequest, 1)
	streamResponseCh := make(chan *v3discoverypb.DiscoveryResponse, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			select {
			case streamRequestCh <- req:
			case <-ctx.Done():
			}
			return nil
		},
		OnStreamResponse: func(_ context.Context, _ int64, _ *v3discoverypb.DiscoveryRequest, resp *v3discoverypb.DiscoveryResponse) {
			select {
			case streamResponseCh <- resp:
			case <-ctx.Done():
			}
		},
	})

	// Configure the management server with appropriate resources.
	apiListener := &v3listenerpb.ApiListener{
		ApiListener: func() *anypb.Any {
			return testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
				RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
					Rds: &v3httppb.Rds{
						ConfigSource: &v3corepb.ConfigSource{
							ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
						},
						RouteConfigName: "route-configuration-name",
					},
				},
			})
		}(),
	}
	const resourceName = "resource name 1"
	listenerResource := &v3listenerpb.Listener{
		Name:           resourceName,
		ApiListener:    apiListener,
		UseOriginalDst: &wrapperspb.BoolValue{Value: true}, // This will cause the resource to be NACKed.
	}
	nodeID := uuid.New().String()
	mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listenerResource},
		SkipValidation: true,
	})

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: mgmtServer.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	// Create a new transport.
	tr, err := transport.New(transport.Options{
		ServerCfg:      serverCfg,
		NodeProto:      &v3corepb.Node{Id: nodeID},
		OnRecvHandler:  dataModelValidator,
		OnErrorHandler: func(err error) {},
		OnSendHandler:  func(*transport.ResourceSendInfo) {},
	})
	if err != nil {
		t.Fatalf("Failed to create xDS transport: %v", err)
	}
	defer tr.Close()

	// Send a discovery request through the transport.
	tr.SendRequest(version.V3ListenerURL, []string{resourceName})

	// Verify that the initial discovery request matches expectation.
	var gotReq *v3discoverypb.DiscoveryRequest
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery request on the stream")
	}
	wantReq := &v3discoverypb.DiscoveryRequest{
		Node:          &v3corepb.Node{Id: nodeID},
		ResourceNames: []string{resourceName},
		TypeUrl:       "type.googleapis.com/envoy.config.listener.v3.Listener",
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	var gotResp *v3discoverypb.DiscoveryResponse
	select {
	case gotResp = <-streamResponseCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery response on the stream")
	}

	// NACK should contain the appropriate error, nonce, but empty version.
	wantReq.VersionInfo = ""
	wantReq.ResponseNonce = gotResp.GetNonce()
	wantReq.ErrorDetail = &statuspb.Status{
		Code:    int32(codes.InvalidArgument),
		Message: errWantNack.Error(),
	}
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for the discovery request ACK on the stream")
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}
}

// TestResourceIsNotRequestedAnymore tests the scenario where the xDS client is
// no longer interested in a resource. The following sequence of events are
// tested:
//  1. A resource is requested and a good response is received. The test verifies
//     that an ACK is sent for this resource.
//  2. The previously requested resource is no longer requested. The test
//     verifies that a request with no resource names is sent out.
//  3. The same resource is requested again. The test verifies that the request
//     is sent with the previously ACKed version.
func (s) TestResourceIsNotRequestedAnymore(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Create an xDS management server listening on a local port. Configure the
	// request and response handlers to push on channels which are inspected by
	// the test goroutine to verify ack version and nonce.
	streamRequestCh := make(chan *v3discoverypb.DiscoveryRequest, 1)
	streamResponseCh := make(chan *v3discoverypb.DiscoveryResponse, 1)
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			select {
			case streamRequestCh <- req:
			case <-ctx.Done():
			}
			return nil
		},
		OnStreamResponse: func(_ context.Context, _ int64, _ *v3discoverypb.DiscoveryRequest, resp *v3discoverypb.DiscoveryResponse) {
			select {
			case streamResponseCh <- resp:
			case <-ctx.Done():
			}
		},
	})

	// Configure the management server with appropriate resources.
	apiListener := &v3listenerpb.ApiListener{
		ApiListener: func() *anypb.Any {
			return testutils.MarshalAny(t, &v3httppb.HttpConnectionManager{
				RouteSpecifier: &v3httppb.HttpConnectionManager_Rds{
					Rds: &v3httppb.Rds{
						ConfigSource: &v3corepb.ConfigSource{
							ConfigSourceSpecifier: &v3corepb.ConfigSource_Ads{Ads: &v3corepb.AggregatedConfigSource{}},
						},
						RouteConfigName: "route-configuration-name",
					},
				},
			})
		}(),
	}
	const resourceName = "resource name 1"
	listenerResource := &v3listenerpb.Listener{
		Name:        resourceName,
		ApiListener: apiListener,
	}
	nodeID := uuid.New().String()
	mgmtServer.Update(ctx, e2e.UpdateOptions{
		NodeID:         nodeID,
		Listeners:      []*v3listenerpb.Listener{listenerResource},
		SkipValidation: true,
	})

	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: mgmtServer.Address})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	// Create a new transport.
	tr, err := transport.New(transport.Options{
		ServerCfg:      serverCfg,
		NodeProto:      &v3corepb.Node{Id: nodeID},
		OnRecvHandler:  dataModelValidator,
		OnErrorHandler: func(err error) {},
		OnSendHandler:  func(*transport.ResourceSendInfo) {},
	})
	if err != nil {
		t.Fatalf("Failed to create xDS transport: %v", err)
	}
	defer tr.Close()

	// Send a discovery request through the transport.
	tr.SendRequest(version.V3ListenerURL, []string{resourceName})

	// Verify that the initial discovery request matches expectation.
	var gotReq *v3discoverypb.DiscoveryRequest
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery request on the stream")
	}
	wantReq := &v3discoverypb.DiscoveryRequest{
		VersionInfo:   "",
		Node:          &v3corepb.Node{Id: nodeID},
		ResourceNames: []string{resourceName},
		TypeUrl:       "type.googleapis.com/envoy.config.listener.v3.Listener",
		ResponseNonce: "",
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Capture the version and nonce from the response.
	var gotResp *v3discoverypb.DiscoveryResponse
	select {
	case gotResp = <-streamResponseCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery response on the stream")
	}

	// Verify that the ACK contains the appropriate version and nonce.
	wantReq.VersionInfo = gotResp.GetVersionInfo()
	wantReq.ResponseNonce = gotResp.GetNonce()
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for the discovery request ACK on the stream")
	}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Send a discovery request with no resource names.
	tr.SendRequest(version.V3ListenerURL, []string{})

	// Verify that the discovery request matches expectation.
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery request on the stream")
	}
	wantReq.ResourceNames = nil
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}

	// Send a discovery request for the same resource requested earlier.
	tr.SendRequest(version.V3ListenerURL, []string{resourceName})

	// Verify that the discovery request contains the version from the
	// previously received response.
	select {
	case gotReq = <-streamRequestCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for discovery request on the stream")
	}
	wantReq.ResourceNames = []string{resourceName}
	if diff := cmp.Diff(gotReq, wantReq, protocmp.Transform(), cmpopts.SortSlices(strSort)); diff != "" {
		t.Fatalf("Unexpected diff in received discovery request, diff (-got, +want):\n%s", diff)
	}
}
