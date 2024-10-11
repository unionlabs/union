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

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/internal/testutils/xds/e2e/setup"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestClientSideRetry(t *testing.T) {
	ctr := 0
	errs := []codes.Code{codes.ResourceExhausted}

	managementServer, nodeID, _, xdsResolver := setup.ManagementServerAndResolver(t)

	server := stubserver.StartTestService(t, &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			defer func() { ctr++ }()
			if ctr < len(errs) {
				return nil, status.Errorf(errs[ctr], "this should be retried")
			}
			return &testpb.Empty{}, nil
		},
	})
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
	defer cancel()
	if _, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("rpc EmptyCall() = _, %v; want _, ResourceExhausted", err)
	}

	testCases := []struct {
		name        string
		vhPolicy    *v3routepb.RetryPolicy
		routePolicy *v3routepb.RetryPolicy
		errs        []codes.Code // the errors returned by the server for each RPC
		tryAgainErr codes.Code   // the error that would be returned if we are still using the old retry policies.
		errWant     codes.Code
	}{{
		name: "virtualHost only, fail",
		vhPolicy: &v3routepb.RetryPolicy{
			RetryOn:    "resource-exhausted,unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 1},
		},
		errs:        []codes.Code{codes.ResourceExhausted, codes.Unavailable},
		routePolicy: nil,
		tryAgainErr: codes.ResourceExhausted,
		errWant:     codes.Unavailable,
	}, {
		name: "virtualHost only",
		vhPolicy: &v3routepb.RetryPolicy{
			RetryOn:    "resource-exhausted, unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		errs:        []codes.Code{codes.ResourceExhausted, codes.Unavailable},
		routePolicy: nil,
		tryAgainErr: codes.Unavailable,
		errWant:     codes.OK,
	}, {
		name: "virtualHost+route, fail",
		vhPolicy: &v3routepb.RetryPolicy{
			RetryOn:    "resource-exhausted,unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		routePolicy: &v3routepb.RetryPolicy{
			RetryOn:    "resource-exhausted",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		errs:        []codes.Code{codes.ResourceExhausted, codes.Unavailable},
		tryAgainErr: codes.OK,
		errWant:     codes.Unavailable,
	}, {
		name: "virtualHost+route",
		vhPolicy: &v3routepb.RetryPolicy{
			RetryOn:    "resource-exhausted",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		routePolicy: &v3routepb.RetryPolicy{
			RetryOn:    "unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		errs:        []codes.Code{codes.Unavailable},
		tryAgainErr: codes.Unavailable,
		errWant:     codes.OK,
	}, {
		name: "virtualHost+route, not enough attempts",
		vhPolicy: &v3routepb.RetryPolicy{
			RetryOn:    "unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 2},
		},
		routePolicy: &v3routepb.RetryPolicy{
			RetryOn:    "unavailable",
			NumRetries: &wrapperspb.UInt32Value{Value: 1},
		},
		errs:        []codes.Code{codes.Unavailable, codes.Unavailable},
		tryAgainErr: codes.OK,
		errWant:     codes.Unavailable,
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			errs = tc.errs

			// Confirm tryAgainErr is correct before updating resources.
			ctr = 0
			_, err := client.EmptyCall(ctx, &testpb.Empty{})
			if code := status.Code(err); code != tc.tryAgainErr {
				t.Fatalf("with old retry policy: EmptyCall() = _, %v; want _, %v", err, tc.tryAgainErr)
			}

			resources.Routes[0].VirtualHosts[0].RetryPolicy = tc.vhPolicy
			resources.Routes[0].VirtualHosts[0].Routes[0].GetRoute().RetryPolicy = tc.routePolicy
			if err := managementServer.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}

			for {
				ctr = 0
				_, err := client.EmptyCall(ctx, &testpb.Empty{})
				if code := status.Code(err); code == tc.tryAgainErr {
					continue
				} else if code != tc.errWant {
					t.Fatalf("rpc EmptyCall() = _, %v; want _, %v", err, tc.errWant)
				}
				break
			}
		})
	}
}
