/*
 *
 * Copyright 2023 gRPC authors.
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

package weightedroundrobin_test

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/internal/testutils/stats"
	"google.golang.org/grpc/orca"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"

	wrr "google.golang.org/grpc/balancer/weightedroundrobin"
	iwrr "google.golang.org/grpc/balancer/weightedroundrobin/internal"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const defaultTestTimeout = 10 * time.Second
const weightUpdatePeriod = 50 * time.Millisecond
const weightExpirationPeriod = time.Minute
const oobReportingInterval = 10 * time.Millisecond

func init() {
	iwrr.AllowAnyWeightUpdatePeriod = true
}

func boolp(b bool) *bool          { return &b }
func float64p(f float64) *float64 { return &f }
func stringp(s string) *string    { return &s }

var (
	perCallConfig = iwrr.LBConfig{
		EnableOOBLoadReport:     boolp(false),
		OOBReportingPeriod:      stringp("0.005s"),
		BlackoutPeriod:          stringp("0s"),
		WeightExpirationPeriod:  stringp("60s"),
		WeightUpdatePeriod:      stringp(".050s"),
		ErrorUtilizationPenalty: float64p(0),
	}
	oobConfig = iwrr.LBConfig{
		EnableOOBLoadReport:     boolp(true),
		OOBReportingPeriod:      stringp("0.005s"),
		BlackoutPeriod:          stringp("0s"),
		WeightExpirationPeriod:  stringp("60s"),
		WeightUpdatePeriod:      stringp(".050s"),
		ErrorUtilizationPenalty: float64p(0),
	}
	testMetricsConfig = iwrr.LBConfig{
		EnableOOBLoadReport:     boolp(false),
		OOBReportingPeriod:      stringp("0.005s"),
		BlackoutPeriod:          stringp("0s"),
		WeightExpirationPeriod:  stringp("60s"),
		WeightUpdatePeriod:      stringp(".050s"),
		ErrorUtilizationPenalty: float64p(0),
	}
)

type testServer struct {
	*stubserver.StubServer

	oobMetrics  orca.ServerMetricsRecorder // Attached to the OOB stream.
	callMetrics orca.CallMetricsRecorder   // Attached to per-call metrics.
}

type reportType int

const (
	reportNone reportType = iota
	reportOOB
	reportCall
	reportBoth
)

func startServer(t *testing.T, r reportType) *testServer {
	t.Helper()

	smr := orca.NewServerMetricsRecorder()
	cmr := orca.NewServerMetricsRecorder().(orca.CallMetricsRecorder)

	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			if r := orca.CallMetricsRecorderFromContext(ctx); r != nil {
				// Copy metrics from what the test set in cmr into r.
				sm := cmr.(orca.ServerMetricsProvider).ServerMetrics()
				r.SetApplicationUtilization(sm.AppUtilization)
				r.SetQPS(sm.QPS)
				r.SetEPS(sm.EPS)
			}
			return &testpb.Empty{}, nil
		},
	}

	var sopts []grpc.ServerOption
	if r == reportCall || r == reportBoth {
		sopts = append(sopts, orca.CallMetricsServerOption(nil))
	}

	if r == reportOOB || r == reportBoth {
		oso := orca.ServiceOptions{
			ServerMetricsProvider: smr,
			MinReportingInterval:  10 * time.Millisecond,
		}
		internal.ORCAAllowAnyMinReportingInterval.(func(so *orca.ServiceOptions))(&oso)
		sopts = append(sopts, stubserver.RegisterServiceServerOption(func(s *grpc.Server) {
			if err := orca.Register(s, oso); err != nil {
				t.Fatalf("Failed to register orca service: %v", err)
			}
		}))
	}

	if err := ss.StartServer(sopts...); err != nil {
		t.Fatalf("Error starting server: %v", err)
	}
	t.Cleanup(ss.Stop)

	return &testServer{
		StubServer:  ss,
		oobMetrics:  smr,
		callMetrics: cmr,
	}
}

func svcConfig(t *testing.T, wrrCfg iwrr.LBConfig) string {
	t.Helper()
	m, err := json.Marshal(wrrCfg)
	if err != nil {
		t.Fatalf("Error marshaling JSON %v: %v", wrrCfg, err)
	}
	sc := fmt.Sprintf(`{"loadBalancingConfig": [ {%q:%v} ] }`, wrr.Name, string(m))
	t.Logf("Marshaled service config: %v", sc)
	return sc
}

// Tests basic functionality with one address.  With only one address, load
// reporting doesn't affect routing at all.
func (s) TestBalancer_OneAddress(t *testing.T) {
	testCases := []struct {
		rt  reportType
		cfg iwrr.LBConfig
	}{
		{rt: reportNone, cfg: perCallConfig},
		{rt: reportCall, cfg: perCallConfig},
		{rt: reportOOB, cfg: oobConfig},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("reportType:%v", tc.rt), func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()

			srv := startServer(t, tc.rt)

			sc := svcConfig(t, tc.cfg)
			if err := srv.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
				t.Fatalf("Error starting client: %v", err)
			}

			// Perform many RPCs to ensure the LB policy works with 1 address.
			for i := 0; i < 100; i++ {
				srv.callMetrics.SetQPS(float64(i))
				srv.oobMetrics.SetQPS(float64(i))
				if _, err := srv.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
					t.Fatalf("Error from EmptyCall: %v", err)
				}
				time.Sleep(time.Millisecond) // Delay; test will run 100ms and should perform ~10 weight updates
			}
		})
	}
}

// TestWRRMetricsBasic tests metrics emitted from the WRR balancer. It
// configures a weighted round robin balancer as the top level balancer of a
// ClientConn, and configures a fake stats handler on the ClientConn to receive
// metrics. It verifies stats emitted from the Weighted Round Robin Balancer on
// balancer startup case which triggers the first picker and scheduler update
// before any load reports are received.
//
// Note that this test and others, metrics emission asssertions are a snapshot
// of the most recently emitted metrics. This is due to the nondeterminism of
// scheduler updates with respect to test bodies, so the assertions made are
// from the most recently synced state of the system (picker/scheduler) from the
// test body.
func (s) TestWRRMetricsBasic(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv := startServer(t, reportCall)
	sc := svcConfig(t, testMetricsConfig)

	mr := stats.NewTestMetricsRecorder(t)
	if err := srv.StartClient(grpc.WithDefaultServiceConfig(sc), grpc.WithStatsHandler(mr)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	srv.callMetrics.SetQPS(float64(1))

	if _, err := srv.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Error from EmptyCall: %v", err)
	}

	mr.AssertDataForMetric("grpc.lb.wrr.rr_fallback", 1)           // Falls back because only one SubConn.
	mr.AssertDataForMetric("grpc.lb.wrr.endpoint_weight_stale", 0) // The endpoint weight has not expired so this is 0 (never emitted).
	mr.AssertDataForMetric("grpc.lb.wrr.endpoint_weight_not_yet_usable", 1)
	// Unusable, so no endpoint weight. Due to only one SubConn, this will never
	// update the weight. Thus, this will stay 0.
	mr.AssertDataForMetric("grpc.lb.wrr.endpoint_weights", 0)
}

// Tests two addresses with ORCA reporting disabled (should fall back to pure
// RR).
func (s) TestBalancer_TwoAddresses_ReportingDisabled(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportNone)
	srv2 := startServer(t, reportNone)

	sc := svcConfig(t, perCallConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Perform many RPCs to ensure the LB policy works with 2 addresses.
	for i := 0; i < 20; i++ {
		roundrobin.CheckRoundRobinRPCs(ctx, srv1.Client, addrs)
	}
}

// Tests two addresses with per-call ORCA reporting enabled.  Checks the
// backends are called in the appropriate ratios.
func (s) TestBalancer_TwoAddresses_ReportingEnabledPerCall(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportCall)
	srv2 := startServer(t, reportCall)

	// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
	// disproportionately to srv2 (10:1).
	srv1.callMetrics.SetQPS(10.0)
	srv1.callMetrics.SetApplicationUtilization(1.0)

	srv2.callMetrics.SetQPS(10.0)
	srv2.callMetrics.SetApplicationUtilization(.1)

	sc := svcConfig(t, perCallConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 2)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})
}

// Tests two addresses with OOB ORCA reporting enabled.  Checks the backends
// are called in the appropriate ratios.
func (s) TestBalancer_TwoAddresses_ReportingEnabledOOB(t *testing.T) {
	testCases := []struct {
		name       string
		utilSetter func(orca.ServerMetricsRecorder, float64)
	}{{
		name: "application_utilization",
		utilSetter: func(smr orca.ServerMetricsRecorder, val float64) {
			smr.SetApplicationUtilization(val)
		},
	}, {
		name: "cpu_utilization",
		utilSetter: func(smr orca.ServerMetricsRecorder, val float64) {
			smr.SetCPUUtilization(val)
		},
	}, {
		name: "application over cpu",
		utilSetter: func(smr orca.ServerMetricsRecorder, val float64) {
			smr.SetApplicationUtilization(val)
			smr.SetCPUUtilization(2.0) // ignored because ApplicationUtilization is set
		},
	}}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()

			srv1 := startServer(t, reportOOB)
			srv2 := startServer(t, reportOOB)

			// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
			// disproportionately to srv2 (10:1).
			srv1.oobMetrics.SetQPS(10.0)
			tc.utilSetter(srv1.oobMetrics, 1.0)

			srv2.oobMetrics.SetQPS(10.0)
			tc.utilSetter(srv2.oobMetrics, 0.1)

			sc := svcConfig(t, oobConfig)
			if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
				t.Fatalf("Error starting client: %v", err)
			}
			addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
			srv1.R.UpdateState(resolver.State{Addresses: addrs})

			// Call each backend once to ensure the weights have been received.
			ensureReached(ctx, t, srv1.Client, 2)

			// Wait for the weight update period to allow the new weights to be processed.
			time.Sleep(weightUpdatePeriod)
			checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})
		})
	}
}

// Tests two addresses with OOB ORCA reporting enabled, where the reports
// change over time.  Checks the backends are called in the appropriate ratios
// before and after modifying the reports.
func (s) TestBalancer_TwoAddresses_UpdateLoads(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportOOB)
	srv2 := startServer(t, reportOOB)

	// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
	// disproportionately to srv2 (10:1).
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(1.0)

	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(.1)

	sc := svcConfig(t, oobConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 2)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})

	// Update the loads so srv2 is loaded and srv1 is not; ensure RPCs are
	// routed disproportionately to srv1.
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(.1)

	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(1.0)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod + oobReportingInterval)
	checkWeights(ctx, t, srvWeight{srv1, 10}, srvWeight{srv2, 1})
}

// Tests two addresses with OOB ORCA reporting enabled, then with switching to
// per-call reporting.  Checks the backends are called in the appropriate
// ratios before and after the change.
func (s) TestBalancer_TwoAddresses_OOBThenPerCall(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportBoth)
	srv2 := startServer(t, reportBoth)

	// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
	// disproportionately to srv2 (10:1).
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(1.0)

	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(.1)

	// For per-call metrics (not used initially), srv2 reports that it is
	// loaded and srv1 reports low load.  After confirming OOB works, switch to
	// per-call and confirm the new routing weights are applied.
	srv1.callMetrics.SetQPS(10.0)
	srv1.callMetrics.SetApplicationUtilization(.1)

	srv2.callMetrics.SetQPS(10.0)
	srv2.callMetrics.SetApplicationUtilization(1.0)

	sc := svcConfig(t, oobConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 2)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})

	// Update to per-call weights.
	c := svcConfig(t, perCallConfig)
	parsedCfg := srv1.R.CC.ParseServiceConfig(c)
	if parsedCfg.Err != nil {
		panic(fmt.Sprintf("Error parsing config %q: %v", c, parsedCfg.Err))
	}
	srv1.R.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: parsedCfg})

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 10}, srvWeight{srv2, 1})
}

// Tests two addresses with OOB ORCA reporting enabled and a non-zero error
// penalty applied.
func (s) TestBalancer_TwoAddresses_ErrorPenalty(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportOOB)
	srv2 := startServer(t, reportOOB)

	// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
	// disproportionately to srv2 (10:1).  EPS values are set (but ignored
	// initially due to ErrorUtilizationPenalty=0).  Later EUP will be updated
	// to 0.9 which will cause the weights to be equal and RPCs to be routed
	// 50/50.
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(1.0)
	srv1.oobMetrics.SetEPS(0)
	// srv1 weight before: 10.0 / 1.0 = 10.0
	// srv1 weight after:  10.0 / 1.0 = 10.0

	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(.1)
	srv2.oobMetrics.SetEPS(10.0)
	// srv2 weight before: 10.0 / 0.1 = 100.0
	// srv2 weight after:  10.0 / 1.0 = 10.0

	sc := svcConfig(t, oobConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 2)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})

	// Update to include an error penalty in the weights.
	newCfg := oobConfig
	newCfg.ErrorUtilizationPenalty = float64p(0.9)
	c := svcConfig(t, newCfg)
	parsedCfg := srv1.R.CC.ParseServiceConfig(c)
	if parsedCfg.Err != nil {
		panic(fmt.Sprintf("Error parsing config %q: %v", c, parsedCfg.Err))
	}
	srv1.R.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: parsedCfg})

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod + oobReportingInterval)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 1})
}

// Tests that the blackout period causes backends to use 0 as their weight
// (meaning to use the average weight) until the blackout period elapses.
func (s) TestBalancer_TwoAddresses_BlackoutPeriod(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	var mu sync.Mutex
	start := time.Now()
	now := start
	setNow := func(t time.Time) {
		mu.Lock()
		defer mu.Unlock()
		now = t
	}

	setTimeNow(func() time.Time {
		mu.Lock()
		defer mu.Unlock()
		return now
	})
	t.Cleanup(func() { setTimeNow(time.Now) })

	testCases := []struct {
		blackoutPeriodCfg *string
		blackoutPeriod    time.Duration
	}{{
		blackoutPeriodCfg: stringp("1s"),
		blackoutPeriod:    time.Second,
	}, {
		blackoutPeriodCfg: nil,
		blackoutPeriod:    10 * time.Second, // the default
	}}
	for _, tc := range testCases {
		setNow(start)
		srv1 := startServer(t, reportOOB)
		srv2 := startServer(t, reportOOB)

		// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
		// disproportionately to srv2 (10:1).
		srv1.oobMetrics.SetQPS(10.0)
		srv1.oobMetrics.SetApplicationUtilization(1.0)

		srv2.oobMetrics.SetQPS(10.0)
		srv2.oobMetrics.SetApplicationUtilization(.1)

		cfg := oobConfig
		cfg.BlackoutPeriod = tc.blackoutPeriodCfg
		sc := svcConfig(t, cfg)
		if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
			t.Fatalf("Error starting client: %v", err)
		}
		addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
		srv1.R.UpdateState(resolver.State{Addresses: addrs})

		// Call each backend once to ensure the weights have been received.
		ensureReached(ctx, t, srv1.Client, 2)

		// Wait for the weight update period to allow the new weights to be processed.
		time.Sleep(weightUpdatePeriod)
		// During the blackout period (1s) we should route roughly 50/50.
		checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 1})

		// Advance time to right before the blackout period ends and the weights
		// should still be zero.
		setNow(start.Add(tc.blackoutPeriod - time.Nanosecond))
		// Wait for the weight update period to allow the new weights to be processed.
		time.Sleep(weightUpdatePeriod)
		checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 1})

		// Advance time to right after the blackout period ends and the weights
		// should now activate.
		setNow(start.Add(tc.blackoutPeriod))
		// Wait for the weight update period to allow the new weights to be processed.
		time.Sleep(weightUpdatePeriod)
		checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})
	}
}

// Tests that the weight expiration period causes backends to use 0 as their
// weight (meaning to use the average weight) once the expiration period
// elapses.
func (s) TestBalancer_TwoAddresses_WeightExpiration(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	var mu sync.Mutex
	start := time.Now()
	now := start
	setNow := func(t time.Time) {
		mu.Lock()
		defer mu.Unlock()
		now = t
	}
	setTimeNow(func() time.Time {
		mu.Lock()
		defer mu.Unlock()
		return now
	})
	t.Cleanup(func() { setTimeNow(time.Now) })

	srv1 := startServer(t, reportBoth)
	srv2 := startServer(t, reportBoth)

	// srv1 starts loaded and srv2 starts without load; ensure RPCs are routed
	// disproportionately to srv2 (10:1).  Because the OOB reporting interval
	// is 1 minute but the weights expire in 1 second, routing will go to 50/50
	// after the weights expire.
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(1.0)

	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(.1)

	cfg := oobConfig
	cfg.OOBReportingPeriod = stringp("60s")
	sc := svcConfig(t, cfg)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 2)

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})

	// Advance what time.Now returns to the weight expiration time minus 1s to
	// ensure all weights are still honored.
	setNow(start.Add(weightExpirationPeriod - time.Second))

	// Wait for the weight update period to allow the new weights to be processed.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10})

	// Advance what time.Now returns to the weight expiration time plus 1s to
	// ensure all weights expired and addresses are routed evenly.
	setNow(start.Add(weightExpirationPeriod + time.Second))

	// Wait for the weight expiration period so the weights have expired.
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 1})
}

// Tests logic surrounding subchannel management.
func (s) TestBalancer_AddressesChanging(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	srv1 := startServer(t, reportBoth)
	srv2 := startServer(t, reportBoth)
	srv3 := startServer(t, reportBoth)
	srv4 := startServer(t, reportBoth)

	// srv1: weight 10
	srv1.oobMetrics.SetQPS(10.0)
	srv1.oobMetrics.SetApplicationUtilization(1.0)
	// srv2: weight 100
	srv2.oobMetrics.SetQPS(10.0)
	srv2.oobMetrics.SetApplicationUtilization(.1)
	// srv3: weight 20
	srv3.oobMetrics.SetQPS(20.0)
	srv3.oobMetrics.SetApplicationUtilization(1.0)
	// srv4: weight 200
	srv4.oobMetrics.SetQPS(20.0)
	srv4.oobMetrics.SetApplicationUtilization(.1)

	sc := svcConfig(t, oobConfig)
	if err := srv1.StartClient(grpc.WithDefaultServiceConfig(sc)); err != nil {
		t.Fatalf("Error starting client: %v", err)
	}
	srv2.Client = srv1.Client
	addrs := []resolver.Address{{Addr: srv1.Address}, {Addr: srv2.Address}, {Addr: srv3.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})

	// Call each backend once to ensure the weights have been received.
	ensureReached(ctx, t, srv1.Client, 3)
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10}, srvWeight{srv3, 2})

	// Add backend 4
	addrs = append(addrs, resolver.Address{Addr: srv4.Address})
	srv1.R.UpdateState(resolver.State{Addresses: addrs})
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10}, srvWeight{srv3, 2}, srvWeight{srv4, 20})

	// Shutdown backend 3.  RPCs will no longer be routed to it.
	srv3.Stop()
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv2, 10}, srvWeight{srv4, 20})

	// Remove addresses 2 and 3.  RPCs will no longer be routed to 2 either.
	addrs = []resolver.Address{{Addr: srv1.Address}, {Addr: srv4.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv1, 1}, srvWeight{srv4, 20})

	// Re-add 2 and remove the rest.
	addrs = []resolver.Address{{Addr: srv2.Address}}
	srv1.R.UpdateState(resolver.State{Addresses: addrs})
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv2, 10})

	// Re-add 4.
	addrs = append(addrs, resolver.Address{Addr: srv4.Address})
	srv1.R.UpdateState(resolver.State{Addresses: addrs})
	time.Sleep(weightUpdatePeriod)
	checkWeights(ctx, t, srvWeight{srv2, 10}, srvWeight{srv4, 20})
}

func ensureReached(ctx context.Context, t *testing.T, c testgrpc.TestServiceClient, n int) {
	t.Helper()
	reached := make(map[string]struct{})
	for len(reached) != n {
		var peer peer.Peer
		if _, err := c.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
			t.Fatalf("Error from EmptyCall: %v", err)
		}
		reached[peer.Addr.String()] = struct{}{}
	}
}

type srvWeight struct {
	srv *testServer
	w   int
}

const rrIterations = 100

// checkWeights does rrIterations RPCs and expects the different backends to be
// routed in a ratio as determined by the srvWeights passed in.  Allows for
// some variance (+/- 2 RPCs per backend).
func checkWeights(ctx context.Context, t *testing.T, sws ...srvWeight) {
	t.Helper()

	c := sws[0].srv.Client

	// Replace the weights with approximate counts of RPCs wanted given the
	// iterations performed.
	weightSum := 0
	for _, sw := range sws {
		weightSum += sw.w
	}
	for i := range sws {
		sws[i].w = rrIterations * sws[i].w / weightSum
	}

	for attempts := 0; attempts < 10; attempts++ {
		serverCounts := make(map[string]int)
		for i := 0; i < rrIterations; i++ {
			var peer peer.Peer
			if _, err := c.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(&peer)); err != nil {
				t.Fatalf("Error from EmptyCall: %v; timed out waiting for weighted RR behavior?", err)
			}
			serverCounts[peer.Addr.String()]++
		}
		if len(serverCounts) != len(sws) {
			continue
		}
		success := true
		for _, sw := range sws {
			c := serverCounts[sw.srv.Address]
			if c < sw.w-2 || c > sw.w+2 {
				success = false
				break
			}
		}
		if success {
			t.Logf("Passed iteration %v; counts: %v", attempts, serverCounts)
			return
		}
		t.Logf("Failed iteration %v; counts: %v; want %+v", attempts, serverCounts, sws)
		time.Sleep(5 * time.Millisecond)
	}
	t.Fatalf("Failed to route RPCs with proper ratio")
}

func init() {
	setTimeNow(time.Now)
	iwrr.TimeNow = timeNow
}

var timeNowFunc atomic.Value // func() time.Time

func timeNow() time.Time {
	return timeNowFunc.Load().(func() time.Time)()
}

func setTimeNow(f func() time.Time) {
	timeNowFunc.Store(f)
}
