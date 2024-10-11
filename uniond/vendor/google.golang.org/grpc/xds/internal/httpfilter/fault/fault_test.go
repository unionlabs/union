//go:build !386
// +build !386

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

// Package xds_test contains e2e tests for xDS use.
package fault

import (
	"context"
	"fmt"
	"io"
	"math/rand"
	"net"
	"reflect"
	"testing"
	"time"

	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	v3listenerpb "github.com/envoyproxy/go-control-plane/envoy/config/listener/v3"
	cpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/common/fault/v3"
	fpb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/http/fault/v3"
	v3httppb "github.com/envoyproxy/go-control-plane/envoy/extensions/filters/network/http_connection_manager/v3"
	tpb "github.com/envoyproxy/go-control-plane/envoy/type/v3"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"

	_ "google.golang.org/grpc/xds/internal/balancer"          // Register the balancers.
	_ "google.golang.org/grpc/xds/internal/httpfilter/router" // Register the router filter.
	_ "google.golang.org/grpc/xds/internal/resolver"          // Register the xds_resolver.
)

const defaultTestTimeout = 10 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type testService struct {
	testgrpc.TestServiceServer
}

func (*testService) EmptyCall(context.Context, *testpb.Empty) (*testpb.Empty, error) {
	return &testpb.Empty{}, nil
}

func (*testService) FullDuplexCall(stream testgrpc.TestService_FullDuplexCallServer) error {
	// End RPC after client does a CloseSend.
	for {
		if _, err := stream.Recv(); err == io.EOF {
			return nil
		} else if err != nil {
			return err
		}
	}
}

// clientSetup performs a bunch of steps common to all xDS server tests here:
// - spin up an xDS management server on a local port
// - spin up a gRPC server and register the test service on it
// - create a local TCP listener and start serving on it
//
// Returns the following:
//   - the management server: tests use this to configure resources
//   - nodeID expected by the management server: this is set in the Node proto
//     sent by the xdsClient for queries.
//   - the port the server is listening on
//   - cleanup function to be invoked by the tests when done
func clientSetup(t *testing.T) (*e2e.ManagementServer, string, uint32, func()) {
	// Spin up a xDS management server on a local port.
	nodeID := uuid.New().String()
	managementServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{})

	// Create a bootstrap file in a temporary directory.
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, managementServer.Address)
	testutils.CreateBootstrapFileForTesting(t, bootstrapContents)

	// Initialize a gRPC server and register the stubServer on it.
	server := grpc.NewServer()
	testgrpc.RegisterTestServiceServer(server, &testService{})

	// Create a local listener and pass it to Serve().
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatalf("testutils.LocalTCPListener() failed: %v", err)
	}

	go func() {
		if err := server.Serve(lis); err != nil {
			t.Errorf("Serve() failed: %v", err)
		}
	}()

	return managementServer, nodeID, uint32(lis.Addr().(*net.TCPAddr).Port), func() {
		server.Stop()
	}
}

func (s) TestFaultInjection_Unary(t *testing.T) {
	type subcase struct {
		name   string
		code   codes.Code
		repeat int
		randIn []int           // Intn calls per-repeat (not per-subcase)
		delays []time.Duration // NewTimer calls per-repeat (not per-subcase)
		md     metadata.MD
	}
	testCases := []struct {
		name       string
		cfgs       []*fpb.HTTPFault
		randOutInc int
		want       []subcase
	}{{
		name: "max faults zero",
		cfgs: []*fpb.HTTPFault{{
			MaxActiveFaults: wrapperspb.UInt32(0),
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 100, Denominator: tpb.FractionalPercent_HUNDRED},
				ErrorType:  &fpb.FaultAbort_GrpcStatus{GrpcStatus: uint32(codes.Aborted)},
			},
		}},
		randOutInc: 5,
		want: []subcase{{
			code:   codes.OK,
			repeat: 25,
		}},
	}, {
		name:       "no abort or delay",
		cfgs:       []*fpb.HTTPFault{{}},
		randOutInc: 5,
		want: []subcase{{
			code:   codes.OK,
			repeat: 25,
		}},
	}, {
		name: "abort always",
		cfgs: []*fpb.HTTPFault{{
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 100, Denominator: tpb.FractionalPercent_HUNDRED},
				ErrorType:  &fpb.FaultAbort_GrpcStatus{GrpcStatus: uint32(codes.Aborted)},
			},
		}},
		randOutInc: 5,
		want: []subcase{{
			code:   codes.Aborted,
			randIn: []int{100},
			repeat: 25,
		}},
	}, {
		name: "abort 10%",
		cfgs: []*fpb.HTTPFault{{
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 100000, Denominator: tpb.FractionalPercent_MILLION},
				ErrorType:  &fpb.FaultAbort_GrpcStatus{GrpcStatus: uint32(codes.Aborted)},
			},
		}},
		randOutInc: 50000,
		want: []subcase{{
			name:   "[0,10]%",
			code:   codes.Aborted,
			randIn: []int{1000000},
			repeat: 2,
		}, {
			name:   "(10,100]%",
			code:   codes.OK,
			randIn: []int{1000000},
			repeat: 18,
		}, {
			name:   "[0,10]% again",
			code:   codes.Aborted,
			randIn: []int{1000000},
			repeat: 2,
		}},
	}, {
		name: "delay always",
		cfgs: []*fpb.HTTPFault{{
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 100, Denominator: tpb.FractionalPercent_HUNDRED},
				FaultDelaySecifier: &cpb.FaultDelay_FixedDelay{FixedDelay: durationpb.New(time.Second)},
			},
		}},
		randOutInc: 5,
		want: []subcase{{
			randIn: []int{100},
			repeat: 25,
			delays: []time.Duration{time.Second},
		}},
	}, {
		name: "delay 10%",
		cfgs: []*fpb.HTTPFault{{
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 1000, Denominator: tpb.FractionalPercent_TEN_THOUSAND},
				FaultDelaySecifier: &cpb.FaultDelay_FixedDelay{FixedDelay: durationpb.New(time.Second)},
			},
		}},
		randOutInc: 500,
		want: []subcase{{
			name:   "[0,10]%",
			randIn: []int{10000},
			repeat: 2,
			delays: []time.Duration{time.Second},
		}, {
			name:   "(10,100]%",
			randIn: []int{10000},
			repeat: 18,
		}, {
			name:   "[0,10]% again",
			randIn: []int{10000},
			repeat: 2,
			delays: []time.Duration{time.Second},
		}},
	}, {
		name: "delay 80%, abort 50%",
		cfgs: []*fpb.HTTPFault{{
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 80, Denominator: tpb.FractionalPercent_HUNDRED},
				FaultDelaySecifier: &cpb.FaultDelay_FixedDelay{FixedDelay: durationpb.New(3 * time.Second)},
			},
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 50, Denominator: tpb.FractionalPercent_HUNDRED},
				ErrorType:  &fpb.FaultAbort_GrpcStatus{GrpcStatus: uint32(codes.Unimplemented)},
			},
		}},
		randOutInc: 5,
		want: []subcase{{
			name:   "50% delay and abort",
			code:   codes.Unimplemented,
			randIn: []int{100, 100},
			repeat: 10,
			delays: []time.Duration{3 * time.Second},
		}, {
			name:   "30% delay, no abort",
			randIn: []int{100, 100},
			repeat: 6,
			delays: []time.Duration{3 * time.Second},
		}, {
			name:   "20% success",
			randIn: []int{100, 100},
			repeat: 4,
		}, {
			name:   "50% delay and abort again",
			code:   codes.Unimplemented,
			randIn: []int{100, 100},
			repeat: 10,
			delays: []time.Duration{3 * time.Second},
		}},
	}, {
		name: "header abort",
		cfgs: []*fpb.HTTPFault{{
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 80, Denominator: tpb.FractionalPercent_HUNDRED},
				ErrorType:  &fpb.FaultAbort_HeaderAbort_{},
			},
		}},
		randOutInc: 10,
		want: []subcase{{
			name: "30% abort; [0,30]%",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.DataLoss)},
				headerAbortPercentage: []string{"30"},
			},
			code:   codes.DataLoss,
			randIn: []int{100},
			repeat: 3,
		}, {
			name: "30% abort; (30,60]%",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.DataLoss)},
				headerAbortPercentage: []string{"30"},
			},
			randIn: []int{100},
			repeat: 3,
		}, {
			name: "80% abort; (60,80]%",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.DataLoss)},
				headerAbortPercentage: []string{"80"},
			},
			code:   codes.DataLoss,
			randIn: []int{100},
			repeat: 2,
		}, {
			name: "cannot exceed percentage in filter",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.DataLoss)},
				headerAbortPercentage: []string{"100"},
			},
			randIn: []int{100},
			repeat: 2,
		}, {
			name: "HTTP Status 404",
			md: metadata.MD{
				headerAbortHTTPStatus: []string{"404"},
				headerAbortPercentage: []string{"100"},
			},
			code:   codes.Unimplemented,
			randIn: []int{100},
			repeat: 1,
		}, {
			name: "HTTP Status 429",
			md: metadata.MD{
				headerAbortHTTPStatus: []string{"429"},
				headerAbortPercentage: []string{"100"},
			},
			code:   codes.Unavailable,
			randIn: []int{100},
			repeat: 1,
		}, {
			name: "HTTP Status 200",
			md: metadata.MD{
				headerAbortHTTPStatus: []string{"200"},
				headerAbortPercentage: []string{"100"},
			},
			// No GRPC status, but HTTP Status of 200 translates to Unknown,
			// per spec in statuscodes.md.
			code:   codes.Unknown,
			randIn: []int{100},
			repeat: 1,
		}, {
			name: "gRPC Status OK",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.OK)},
				headerAbortPercentage: []string{"100"},
			},
			// This should be Unimplemented (mismatched request/response
			// count), per spec in statuscodes.md, but grpc-go currently
			// returns io.EOF which status.Code() converts to Unknown
			code:   codes.Unknown,
			randIn: []int{100},
			repeat: 1,
		}, {
			name: "invalid header results in no abort",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{"error"},
				headerAbortPercentage: []string{"100"},
			},
			repeat: 1,
		}, {
			name: "invalid header results in default percentage",
			md: metadata.MD{
				headerAbortGRPCStatus: []string{fmt.Sprintf("%d", codes.DataLoss)},
				headerAbortPercentage: []string{"error"},
			},
			code:   codes.DataLoss,
			randIn: []int{100},
			repeat: 1,
		}},
	}, {
		name: "header delay",
		cfgs: []*fpb.HTTPFault{{
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 80, Denominator: tpb.FractionalPercent_HUNDRED},
				FaultDelaySecifier: &cpb.FaultDelay_HeaderDelay_{},
			},
		}},
		randOutInc: 10,
		want: []subcase{{
			name: "30% delay; [0,30]%",
			md: metadata.MD{
				headerDelayDuration:   []string{"2"},
				headerDelayPercentage: []string{"30"},
			},
			randIn: []int{100},
			delays: []time.Duration{2 * time.Millisecond},
			repeat: 3,
		}, {
			name: "30% delay; (30, 60]%",
			md: metadata.MD{
				headerDelayDuration:   []string{"2"},
				headerDelayPercentage: []string{"30"},
			},
			randIn: []int{100},
			repeat: 3,
		}, {
			name: "invalid header results in no delay",
			md: metadata.MD{
				headerDelayDuration:   []string{"error"},
				headerDelayPercentage: []string{"80"},
			},
			repeat: 1,
		}, {
			name: "invalid header results in default percentage",
			md: metadata.MD{
				headerDelayDuration:   []string{"2"},
				headerDelayPercentage: []string{"error"},
			},
			randIn: []int{100},
			delays: []time.Duration{2 * time.Millisecond},
			repeat: 1,
		}, {
			name: "invalid header results in default percentage",
			md: metadata.MD{
				headerDelayDuration:   []string{"2"},
				headerDelayPercentage: []string{"error"},
			},
			randIn: []int{100},
			repeat: 1,
		}, {
			name: "cannot exceed percentage in filter",
			md: metadata.MD{
				headerDelayDuration:   []string{"2"},
				headerDelayPercentage: []string{"100"},
			},
			randIn: []int{100},
			repeat: 1,
		}},
	}, {
		name: "abort then delay filters",
		cfgs: []*fpb.HTTPFault{{
			Abort: &fpb.FaultAbort{
				Percentage: &tpb.FractionalPercent{Numerator: 50, Denominator: tpb.FractionalPercent_HUNDRED},
				ErrorType:  &fpb.FaultAbort_GrpcStatus{GrpcStatus: uint32(codes.Unimplemented)},
			},
		}, {
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 80, Denominator: tpb.FractionalPercent_HUNDRED},
				FaultDelaySecifier: &cpb.FaultDelay_FixedDelay{FixedDelay: durationpb.New(time.Second)},
			},
		}},
		randOutInc: 10,
		want: []subcase{{
			name:   "50% delay and abort (abort skips delay)",
			code:   codes.Unimplemented,
			randIn: []int{100},
			repeat: 5,
		}, {
			name:   "30% delay, no abort",
			randIn: []int{100, 100},
			repeat: 3,
			delays: []time.Duration{time.Second},
		}, {
			name:   "20% success",
			randIn: []int{100, 100},
			repeat: 2,
		}},
	}}

	fs, nodeID, port, cleanup := clientSetup(t)
	defer cleanup()

	for tcNum, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			defer func() { randIntn = rand.Intn; newTimer = time.NewTimer }()
			var intnCalls []int
			var newTimerCalls []time.Duration
			randOut := 0
			randIntn = func(n int) int {
				intnCalls = append(intnCalls, n)
				return randOut % n
			}

			newTimer = func(d time.Duration) *time.Timer {
				newTimerCalls = append(newTimerCalls, d)
				return time.NewTimer(0)
			}

			serviceName := fmt.Sprintf("myservice%d", tcNum)
			resources := e2e.DefaultClientResources(e2e.ResourceParams{
				DialTarget: serviceName,
				NodeID:     nodeID,
				Host:       "localhost",
				Port:       port,
				SecLevel:   e2e.SecurityLevelNone,
			})
			hcm := new(v3httppb.HttpConnectionManager)
			lis := resources.Listeners[0].GetApiListener().GetApiListener()
			err := lis.UnmarshalTo(hcm)
			if err != nil {
				t.Fatal(err)
			}
			routerFilter := hcm.HttpFilters[len(hcm.HttpFilters)-1]

			hcm.HttpFilters = nil
			for i, cfg := range tc.cfgs {
				hcm.HttpFilters = append(hcm.HttpFilters, e2e.HTTPFilter(fmt.Sprintf("fault%d", i), cfg))
			}
			hcm.HttpFilters = append(hcm.HttpFilters, routerFilter)
			hcmAny := testutils.MarshalAny(t, hcm)
			resources.Listeners[0].ApiListener.ApiListener = hcmAny
			resources.Listeners[0].FilterChains[0].Filters[0].ConfigType = &v3listenerpb.Filter_TypedConfig{TypedConfig: hcmAny}

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			if err := fs.Update(ctx, resources); err != nil {
				t.Fatal(err)
			}

			// Create a ClientConn and run the test case.
			cc, err := grpc.NewClient("xds:///"+serviceName, grpc.WithTransportCredentials(insecure.NewCredentials()))
			if err != nil {
				t.Fatalf("failed to dial local test server: %v", err)
			}
			defer cc.Close()

			client := testgrpc.NewTestServiceClient(cc)
			count := 0
			for _, want := range tc.want {
				ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
				defer cancel()
				if want.repeat == 0 {
					t.Fatalf("invalid repeat count")
				}
				for n := 0; n < want.repeat; n++ {
					intnCalls = nil
					newTimerCalls = nil
					ctx = metadata.NewOutgoingContext(ctx, want.md)
					_, err := client.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true))
					t.Logf("%v: RPC %d: err: %v, intnCalls: %v, newTimerCalls: %v", want.name, count, err, intnCalls, newTimerCalls)
					if status.Code(err) != want.code || !reflect.DeepEqual(intnCalls, want.randIn) || !reflect.DeepEqual(newTimerCalls, want.delays) {
						t.Fatalf("WANTED code: %v, intnCalls: %v, newTimerCalls: %v", want.code, want.randIn, want.delays)
					}
					randOut += tc.randOutInc
					count++
				}
			}
		})
	}
}

func (s) TestFaultInjection_MaxActiveFaults(t *testing.T) {
	fs, nodeID, port, cleanup := clientSetup(t)
	defer cleanup()
	resources := e2e.DefaultClientResources(e2e.ResourceParams{
		DialTarget: "myservice",
		NodeID:     nodeID,
		Host:       "localhost",
		Port:       port,
		SecLevel:   e2e.SecurityLevelNone,
	})
	hcm := new(v3httppb.HttpConnectionManager)
	lis := resources.Listeners[0].GetApiListener().GetApiListener()
	err := lis.UnmarshalTo(hcm)
	if err != nil {
		t.Fatal(err)
	}

	defer func() { newTimer = time.NewTimer }()
	timers := make(chan *time.Timer, 2)
	newTimer = func(time.Duration) *time.Timer {
		t := time.NewTimer(24 * time.Hour) // Will reset to fire.
		timers <- t
		return t
	}

	hcm.HttpFilters = append([]*v3httppb.HttpFilter{
		e2e.HTTPFilter("fault", &fpb.HTTPFault{
			MaxActiveFaults: wrapperspb.UInt32(2),
			Delay: &cpb.FaultDelay{
				Percentage:         &tpb.FractionalPercent{Numerator: 100, Denominator: tpb.FractionalPercent_HUNDRED},
				FaultDelaySecifier: &cpb.FaultDelay_FixedDelay{FixedDelay: durationpb.New(time.Second)},
			},
		})},
		hcm.HttpFilters...)
	hcmAny := testutils.MarshalAny(t, hcm)
	resources.Listeners[0].ApiListener.ApiListener = hcmAny
	resources.Listeners[0].FilterChains[0].Filters[0].ConfigType = &v3listenerpb.Filter_TypedConfig{TypedConfig: hcmAny}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := fs.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	// Create a ClientConn
	cc, err := grpc.NewClient("xds:///myservice", grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("failed to dial local test server: %v", err)
	}
	defer cc.Close()

	client := testgrpc.NewTestServiceClient(cc)

	streams := make(chan testgrpc.TestService_FullDuplexCallClient, 5) // startStream() is called 5 times
	startStream := func() {
		str, err := client.FullDuplexCall(ctx)
		if err != nil {
			t.Error("RPC error:", err)
		}
		streams <- str
	}
	endStream := func() {
		str := <-streams
		str.CloseSend()
		if _, err := str.Recv(); err != io.EOF {
			t.Error("stream error:", err)
		}
	}
	releaseStream := func() {
		timer := <-timers
		timer.Reset(0)
	}

	// Start three streams; two should delay.
	go startStream()
	go startStream()
	go startStream()

	// End one of the streams.  Ensure the others are blocked on creation.
	endStream()

	select {
	case <-streams:
		t.Errorf("unexpected second stream created before delay expires")
	case <-time.After(50 * time.Millisecond):
		// Wait a short time to ensure no other streams were started yet.
	}

	// Start one more; it should not be blocked.
	go startStream()
	endStream()

	// Expire one stream's delay; it should be created.
	releaseStream()
	endStream()

	// Another new stream should delay.
	go startStream()
	select {
	case <-streams:
		t.Errorf("unexpected second stream created before delay expires")
	case <-time.After(50 * time.Millisecond):
		// Wait a short time to ensure no other streams were started yet.
	}

	// Expire both pending timers and end the two streams.
	releaseStream()
	releaseStream()
	endStream()
	endStream()
}
