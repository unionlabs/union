/*
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

package orca_test

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/orca"
	"google.golang.org/grpc/orca/internal"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	v3orcapb "github.com/cncf/xds/go/xds/data/orca/v3"
	v3orcaservicegrpc "github.com/cncf/xds/go/xds/service/orca/v3"
	v3orcaservicepb "github.com/cncf/xds/go/xds/service/orca/v3"
)

// customLBB wraps a round robin LB policy but provides a ClientConn wrapper to
// add an ORCA OOB report producer for all created SubConns.
type customLBB struct{}

func (customLBB) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	return balancer.Get(roundrobin.Name).Build(&ccWrapper{ClientConn: cc}, opts)
}

func (customLBB) Name() string { return "customLB" }

func init() {
	balancer.Register(customLBB{})
}

type ccWrapper struct {
	balancer.ClientConn
}

func (w *ccWrapper) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	if len(addrs) != 1 {
		panic(fmt.Sprintf("got addrs=%v; want len(addrs) == 1", addrs))
	}
	sc, err := w.ClientConn.NewSubConn(addrs, opts)
	if err != nil {
		return sc, err
	}
	l := getListenerInfo(addrs[0])
	l.listener.cleanup = orca.RegisterOOBListener(sc, l.listener, l.opts)
	l.sc = sc
	return sc, nil
}

// listenerInfo is stored in an address's attributes to allow ORCA
// listeners to be registered on subconns created for that address.
type listenerInfo struct {
	listener *testOOBListener
	opts     orca.OOBListenerOptions
	sc       balancer.SubConn // Set by the LB policy
}

type listenerInfoKey struct{}

func setListenerInfo(addr resolver.Address, l *listenerInfo) resolver.Address {
	addr.Attributes = addr.Attributes.WithValue(listenerInfoKey{}, l)
	return addr
}

func getListenerInfo(addr resolver.Address) *listenerInfo {
	return addr.Attributes.Value(listenerInfoKey{}).(*listenerInfo)
}

// testOOBListener is a simple listener that pushes load reports to a channel.
type testOOBListener struct {
	cleanup      func()
	loadReportCh chan *v3orcapb.OrcaLoadReport
}

func newTestOOBListener() *testOOBListener {
	return &testOOBListener{cleanup: func() {}, loadReportCh: make(chan *v3orcapb.OrcaLoadReport)}
}

func (t *testOOBListener) Stop() { t.cleanup() }

func (t *testOOBListener) OnLoadReport(r *v3orcapb.OrcaLoadReport) {
	t.loadReportCh <- r
}

// TestProducer is a basic, end-to-end style test of an LB policy with an
// OOBListener communicating with a server with an ORCA service.
func (s) TestProducer(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Use a fixed backoff for stream recreation.
	oldBackoff := internal.DefaultBackoffFunc
	internal.DefaultBackoffFunc = func(int) time.Duration { return 10 * time.Millisecond }
	defer func() { internal.DefaultBackoffFunc = oldBackoff }()

	// Initialize listener for our ORCA server.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatal(err)
	}

	// Register the OpenRCAService with a very short metrics reporting interval.
	const shortReportingInterval = 50 * time.Millisecond
	smr := orca.NewServerMetricsRecorder()
	opts := orca.ServiceOptions{MinReportingInterval: shortReportingInterval, ServerMetricsProvider: smr}
	internal.AllowAnyMinReportingInterval.(func(*orca.ServiceOptions))(&opts)
	s := grpc.NewServer()
	if err := orca.Register(s, opts); err != nil {
		t.Fatalf("orca.Register failed: %v", err)
	}
	go s.Serve(lis)
	defer s.Stop()

	// Create our client with an OOB listener in the LB policy it selects.
	r := manual.NewBuilderWithScheme("whatever")
	oobLis := newTestOOBListener()

	lisOpts := orca.OOBListenerOptions{ReportInterval: 50 * time.Millisecond}
	li := &listenerInfo{listener: oobLis, opts: lisOpts}
	addr := setListenerInfo(resolver.Address{Addr: lis.Addr().String()}, li)
	r.InitialState(resolver.State{Addresses: []resolver.Address{addr}})
	cc, err := grpc.Dial("whatever:///whatever", grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"customLB":{}}]}`), grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial failed: %v", err)
	}
	defer cc.Close()

	// Ensure the OOB listener is stopped before the client is closed to avoid
	// a potential irrelevant error in the logs.
	defer oobLis.Stop()

	// Set a few metrics and wait for them on the client side.
	smr.SetCPUUtilization(10)
	smr.SetMemoryUtilization(0.1)
	smr.SetNamedUtilization("bob", 0.555)
	loadReportWant := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 10,
		MemUtilization: 0.1,
		Utilization:    map[string]float64{"bob": 0.555},
	}

testReport:
	for {
		select {
		case r := <-oobLis.loadReportCh:
			t.Log("Load report received: ", r)
			if proto.Equal(r, loadReportWant) {
				// Success!
				break testReport
			}
		case <-ctx.Done():
			t.Fatalf("timed out waiting for load report: %v", loadReportWant)
		}
	}

	// Change and add metrics and wait for them on the client side.
	smr.SetCPUUtilization(0.5)
	smr.SetMemoryUtilization(0.2)
	smr.SetNamedUtilization("mary", 0.321)
	loadReportWant = &v3orcapb.OrcaLoadReport{
		CpuUtilization: 0.5,
		MemUtilization: 0.2,
		Utilization:    map[string]float64{"bob": 0.555, "mary": 0.321},
	}

	for {
		select {
		case r := <-oobLis.loadReportCh:
			t.Log("Load report received: ", r)
			if proto.Equal(r, loadReportWant) {
				// Success!
				return
			}
		case <-ctx.Done():
			t.Fatalf("timed out waiting for load report: %v", loadReportWant)
		}
	}
}

// fakeORCAService is a simple implementation of an ORCA service that pushes
// requests it receives from clients to a channel and sends responses from a
// channel back.  This allows tests to verify the client is sending requests
// and processing responses properly.
type fakeORCAService struct {
	v3orcaservicegrpc.UnimplementedOpenRcaServiceServer

	reqCh  chan *v3orcaservicepb.OrcaLoadReportRequest
	respCh chan any // either *v3orcapb.OrcaLoadReport or error
}

func newFakeORCAService() *fakeORCAService {
	return &fakeORCAService{
		reqCh:  make(chan *v3orcaservicepb.OrcaLoadReportRequest),
		respCh: make(chan any),
	}
}

func (f *fakeORCAService) close() {
	close(f.respCh)
}

func (f *fakeORCAService) StreamCoreMetrics(req *v3orcaservicepb.OrcaLoadReportRequest, stream v3orcaservicegrpc.OpenRcaService_StreamCoreMetricsServer) error {
	f.reqCh <- req
	for {
		var resp any
		select {
		case resp = <-f.respCh:
		case <-stream.Context().Done():
			return stream.Context().Err()
		}

		if err, ok := resp.(error); ok {
			return err
		}
		if err := stream.Send(resp.(*v3orcapb.OrcaLoadReport)); err != nil {
			// In the event that a stream error occurs, a new stream will have
			// been created that was waiting for this response message.  Push
			// it back onto the channel and return.
			//
			// This happens because we range over respCh.  If we changed to
			// instead select on respCh + stream.Context(), the same situation
			// could still occur due to a race between noticing the two events,
			// so such a workaround would still be needed to prevent flakiness.
			f.respCh <- resp
			return err
		}
	}
}

// TestProducerBackoff verifies that the ORCA producer applies the proper
// backoff after stream failures.
func (s) TestProducerBackoff(t *testing.T) {
	grpctest.TLogger.ExpectErrorN("injected error", 4)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Provide a convenient way to expect backoff calls and return a minimal
	// value.
	const backoffShouldNotBeCalled = 9999 // Use to assert backoff function is not called.
	const backoffAllowAny = -1            // Use to ignore any backoff calls.
	expectedBackoff := backoffAllowAny
	oldBackoff := internal.DefaultBackoffFunc
	internal.DefaultBackoffFunc = func(got int) time.Duration {
		if expectedBackoff == backoffShouldNotBeCalled {
			t.Errorf("Unexpected backoff call; parameter = %v", got)
		} else if expectedBackoff != backoffAllowAny {
			if got != expectedBackoff {
				t.Errorf("Unexpected backoff received; got %v want %v", got, expectedBackoff)
			}
		}
		return time.Millisecond
	}
	defer func() { internal.DefaultBackoffFunc = oldBackoff }()

	// Initialize listener for our ORCA server.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatal(err)
	}

	// Register our fake ORCA service.
	s := grpc.NewServer()
	fake := newFakeORCAService()
	defer fake.close()
	v3orcaservicegrpc.RegisterOpenRcaServiceServer(s, fake)
	go s.Serve(lis)
	defer s.Stop()

	// Define the report interval and a function to wait for it to be sent to
	// the server.
	const reportInterval = 123 * time.Second
	awaitRequest := func(interval time.Duration) {
		select {
		case req := <-fake.reqCh:
			if got := req.GetReportInterval().AsDuration(); got != interval {
				t.Errorf("Unexpected report interval; got %v want %v", got, interval)
			}
		case <-ctx.Done():
			t.Fatalf("Did not receive client request")
		}
	}

	// Create our client with an OOB listener in the LB policy it selects.
	r := manual.NewBuilderWithScheme("whatever")
	oobLis := newTestOOBListener()

	lisOpts := orca.OOBListenerOptions{ReportInterval: reportInterval}
	li := &listenerInfo{listener: oobLis, opts: lisOpts}
	r.InitialState(resolver.State{Addresses: []resolver.Address{setListenerInfo(resolver.Address{Addr: lis.Addr().String()}, li)}})
	cc, err := grpc.Dial("whatever:///whatever", grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"customLB":{}}]}`), grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial failed: %v", err)
	}
	defer cc.Close()

	// Ensure the OOB listener is stopped before the client is closed to avoid
	// a potential irrelevant error in the logs.
	defer oobLis.Stop()

	// Define a load report to send and expect the client to see.
	loadReportWant := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 10,
		MemUtilization: 0.1,
		Utilization:    map[string]float64{"bob": 0.555},
	}

	// Unblock the fake.
	awaitRequest(reportInterval)
	fake.respCh <- loadReportWant
	select {
	case r := <-oobLis.loadReportCh:
		t.Log("Load report received: ", r)
		if proto.Equal(r, loadReportWant) {
			// Success!
			break
		}
	case <-ctx.Done():
		t.Fatalf("timed out waiting for load report: %v", loadReportWant)
	}

	// The next request should be immediate, since there was a message
	// received.
	expectedBackoff = backoffShouldNotBeCalled
	fake.respCh <- status.Errorf(codes.Internal, "injected error")
	awaitRequest(reportInterval)

	// The next requests will need to backoff.
	expectedBackoff = 0
	fake.respCh <- status.Errorf(codes.Internal, "injected error")
	awaitRequest(reportInterval)
	expectedBackoff = 1
	fake.respCh <- status.Errorf(codes.Internal, "injected error")
	awaitRequest(reportInterval)
	expectedBackoff = 2
	fake.respCh <- status.Errorf(codes.Internal, "injected error")
	awaitRequest(reportInterval)
	// The next request should be immediate, since there was a message
	// received.
	expectedBackoff = backoffShouldNotBeCalled

	// Send another valid response and wait for it on the client.
	fake.respCh <- loadReportWant
	select {
	case r := <-oobLis.loadReportCh:
		t.Log("Load report received: ", r)
		if proto.Equal(r, loadReportWant) {
			// Success!
			break
		}
	case <-ctx.Done():
		t.Fatalf("timed out waiting for load report: %v", loadReportWant)
	}
}

// TestProducerMultipleListeners tests that multiple listeners works as
// expected in a producer: requesting the proper interval and delivering the
// update to all listeners.
func (s) TestProducerMultipleListeners(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Provide a convenient way to expect backoff calls and return a minimal
	// value.
	oldBackoff := internal.DefaultBackoffFunc
	internal.DefaultBackoffFunc = func(got int) time.Duration {
		return time.Millisecond
	}
	defer func() { internal.DefaultBackoffFunc = oldBackoff }()

	// Initialize listener for our ORCA server.
	lis, err := testutils.LocalTCPListener()
	if err != nil {
		t.Fatal(err)
	}

	// Register our fake ORCA service.
	s := grpc.NewServer()
	fake := newFakeORCAService()
	defer fake.close()
	v3orcaservicegrpc.RegisterOpenRcaServiceServer(s, fake)
	go s.Serve(lis)
	defer s.Stop()

	// Define the report interval and a function to wait for it to be sent to
	// the server.
	const reportInterval1 = 123 * time.Second
	const reportInterval2 = 234 * time.Second
	const reportInterval3 = 56 * time.Second
	awaitRequest := func(interval time.Duration) {
		select {
		case req := <-fake.reqCh:
			if got := req.GetReportInterval().AsDuration(); got != interval {
				t.Errorf("Unexpected report interval; got %v want %v", got, interval)
			}
		case <-ctx.Done():
			t.Fatalf("Did not receive client request")
		}
	}

	// Create our client with an OOB listener in the LB policy it selects.
	r := manual.NewBuilderWithScheme("whatever")
	oobLis1 := newTestOOBListener()
	lisOpts1 := orca.OOBListenerOptions{ReportInterval: reportInterval1}
	li := &listenerInfo{listener: oobLis1, opts: lisOpts1}
	r.InitialState(resolver.State{Addresses: []resolver.Address{setListenerInfo(resolver.Address{Addr: lis.Addr().String()}, li)}})
	cc, err := grpc.Dial("whatever:///whatever", grpc.WithDefaultServiceConfig(`{"loadBalancingConfig": [{"customLB":{}}]}`), grpc.WithResolvers(r), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial failed: %v", err)
	}
	defer cc.Close()

	// Ensure the OOB listener is stopped before the client is closed to avoid
	// a potential irrelevant error in the logs.
	defer oobLis1.Stop()

	oobLis2 := newTestOOBListener()
	lisOpts2 := orca.OOBListenerOptions{ReportInterval: reportInterval2}

	oobLis3 := newTestOOBListener()
	lisOpts3 := orca.OOBListenerOptions{ReportInterval: reportInterval3}

	// Define a load report to send and expect the client to see.
	loadReportWant := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 10,
		MemUtilization: 0.1,
		Utilization:    map[string]float64{"bob": 0.555},
	}

	// Receive reports and update counts for the three listeners.
	var reportsMu sync.Mutex
	var reportsReceived1, reportsReceived2, reportsReceived3 int
	go func() {
		for {
			select {
			case r := <-oobLis1.loadReportCh:
				t.Log("Load report 1 received: ", r)
				if !proto.Equal(r, loadReportWant) {
					t.Errorf("Unexpected report received: %+v", r)
				}
				reportsMu.Lock()
				reportsReceived1++
				reportsMu.Unlock()
			case r := <-oobLis2.loadReportCh:
				t.Log("Load report 2 received: ", r)
				if !proto.Equal(r, loadReportWant) {
					t.Errorf("Unexpected report received: %+v", r)
				}
				reportsMu.Lock()
				reportsReceived2++
				reportsMu.Unlock()
			case r := <-oobLis3.loadReportCh:
				t.Log("Load report 3 received: ", r)
				if !proto.Equal(r, loadReportWant) {
					t.Errorf("Unexpected report received: %+v", r)
				}
				reportsMu.Lock()
				reportsReceived3++
				reportsMu.Unlock()
			case <-ctx.Done():
				// Test has ended; exit
				return
			}
		}
	}()

	// checkReports is a helper function to check the report counts for the three listeners.
	checkReports := func(r1, r2, r3 int) {
		t.Helper()
		for ctx.Err() == nil {
			reportsMu.Lock()
			if r1 == reportsReceived1 && r2 == reportsReceived2 && r3 == reportsReceived3 {
				// Success!
				reportsMu.Unlock()
				return
			}
			if reportsReceived1 > r1 || reportsReceived2 > r2 || reportsReceived3 > r3 {
				reportsMu.Unlock()
				t.Fatalf("received excess reports. got %v %v %v; want %v %v %v", reportsReceived1, reportsReceived2, reportsReceived3, r1, r2, r3)
				return
			}
			reportsMu.Unlock()
			time.Sleep(10 * time.Millisecond)
		}
		t.Fatalf("timed out waiting for reports received. got %v %v %v; want %v %v %v", reportsReceived1, reportsReceived2, reportsReceived3, r1, r2, r3)
	}

	// Only 1 listener; expect reportInterval1 to be used and expect the report
	// to be sent to the listener.
	awaitRequest(reportInterval1)
	fake.respCh <- loadReportWant
	checkReports(1, 0, 0)

	// Register listener 2 with a less frequent interval; no need to recreate
	// stream.  Report should go to both listeners.
	oobLis2.cleanup = orca.RegisterOOBListener(li.sc, oobLis2, lisOpts2)
	fake.respCh <- loadReportWant
	checkReports(2, 1, 0)

	// Register listener 3 with a more frequent interval; stream is recreated
	// with this interval.  The next report will go to all three listeners.
	oobLis3.cleanup = orca.RegisterOOBListener(li.sc, oobLis3, lisOpts3)
	awaitRequest(reportInterval3)
	fake.respCh <- loadReportWant
	checkReports(3, 2, 1)

	// Another report without a change in listeners should go to all three listeners.
	fake.respCh <- loadReportWant
	checkReports(4, 3, 2)

	// Stop listener 2.  This does not affect the interval as listener 3 is
	// still the shortest.  The next update goes to listeners 1 and 3.
	oobLis2.Stop()
	fake.respCh <- loadReportWant
	checkReports(5, 3, 3)

	// Stop listener 3.  This makes the interval longer.  Reports should only
	// go to listener 1 now.
	oobLis3.Stop()
	awaitRequest(reportInterval1)
	fake.respCh <- loadReportWant
	checkReports(6, 3, 3)
	// Another report without a change in listeners should go to the first listener.
	fake.respCh <- loadReportWant
	checkReports(7, 3, 3)
}
