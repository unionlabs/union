/*
 *
 * Copyright 2016 gRPC authors.
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

package grpclb

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"

	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	grpclbstate "google.golang.org/grpc/balancer/grpclb/state"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/testutils/pickfirst"
	"google.golang.org/grpc/internal/testutils/roundrobin"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/durationpb"

	lbgrpc "google.golang.org/grpc/balancer/grpclb/grpc_lb_v1"
	lbpb "google.golang.org/grpc/balancer/grpclb/grpc_lb_v1"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var (
	lbServerName = "lb.server.com"
	beServerName = "backends.com"
	lbToken      = "iamatoken"

	// Resolver replaces localhost with fakeName in Next().
	// Dialer replaces fakeName with localhost when dialing.
	// This will test that custom dialer is passed from Dial to grpclb.
	fakeName = "fake.Name"
)

const (
	defaultTestTimeout      = 10 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond
	testUserAgent           = "test-user-agent"
	grpclbConfig            = `{"loadBalancingConfig": [{"grpclb": {}}]}`
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

type serverNameCheckCreds struct {
	mu sync.Mutex
	sn string
}

func (c *serverNameCheckCreds) ServerHandshake(rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	if _, err := io.WriteString(rawConn, c.sn); err != nil {
		fmt.Printf("Failed to write the server name %s to the client %v", c.sn, err)
		return nil, nil, err
	}
	return rawConn, nil, nil
}
func (c *serverNameCheckCreds) ClientHandshake(ctx context.Context, authority string, rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	c.mu.Lock()
	defer c.mu.Unlock()
	b := make([]byte, len(authority))
	errCh := make(chan error, 1)
	go func() {
		_, err := rawConn.Read(b)
		errCh <- err
	}()
	select {
	case err := <-errCh:
		if err != nil {
			fmt.Printf("test-creds: failed to read expected authority name from the server: %v\n", err)
			return nil, nil, err
		}
	case <-ctx.Done():
		return nil, nil, ctx.Err()
	}
	if authority != string(b) {
		fmt.Printf("test-creds: got authority from ClientConn %q, expected by server %q\n", authority, string(b))
		return nil, nil, errors.New("received unexpected server name")
	}
	return rawConn, nil, nil
}
func (c *serverNameCheckCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c *serverNameCheckCreds) Clone() credentials.TransportCredentials {
	return &serverNameCheckCreds{}
}
func (c *serverNameCheckCreds) OverrideServerName(string) error {
	return nil
}

// fakeNameDialer replaces fakeName with localhost when dialing.
// This will test that custom dialer is passed from Dial to grpclb.
func fakeNameDialer(ctx context.Context, addr string) (net.Conn, error) {
	addr = strings.Replace(addr, fakeName, "localhost", 1)
	return (&net.Dialer{}).DialContext(ctx, "tcp", addr)
}

// merge merges the new client stats into current stats.
//
// It's a test-only method. rpcStats is defined in grpclb_picker.
func (s *rpcStats) merge(cs *lbpb.ClientStats) {
	atomic.AddInt64(&s.numCallsStarted, cs.NumCallsStarted)
	atomic.AddInt64(&s.numCallsFinished, cs.NumCallsFinished)
	atomic.AddInt64(&s.numCallsFinishedWithClientFailedToSend, cs.NumCallsFinishedWithClientFailedToSend)
	atomic.AddInt64(&s.numCallsFinishedKnownReceived, cs.NumCallsFinishedKnownReceived)
	s.mu.Lock()
	for _, perToken := range cs.CallsFinishedWithDrop {
		s.numCallsDropped[perToken.LoadBalanceToken] += perToken.NumCalls
	}
	s.mu.Unlock()
}

func atomicEqual(a, b *int64) bool {
	return atomic.LoadInt64(a) == atomic.LoadInt64(b)
}

// equal compares two rpcStats.
//
// It's a test-only method. rpcStats is defined in grpclb_picker.
func (s *rpcStats) equal(o *rpcStats) bool {
	if !atomicEqual(&s.numCallsStarted, &o.numCallsStarted) {
		return false
	}
	if !atomicEqual(&s.numCallsFinished, &o.numCallsFinished) {
		return false
	}
	if !atomicEqual(&s.numCallsFinishedWithClientFailedToSend, &o.numCallsFinishedWithClientFailedToSend) {
		return false
	}
	if !atomicEqual(&s.numCallsFinishedKnownReceived, &o.numCallsFinishedKnownReceived) {
		return false
	}
	s.mu.Lock()
	defer s.mu.Unlock()
	o.mu.Lock()
	defer o.mu.Unlock()
	return cmp.Equal(s.numCallsDropped, o.numCallsDropped, cmpopts.EquateEmpty())
}

func (s *rpcStats) String() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	return fmt.Sprintf("Started: %v, Finished: %v, FinishedWithClientFailedToSend: %v, FinishedKnownReceived: %v, Dropped: %v",
		atomic.LoadInt64(&s.numCallsStarted),
		atomic.LoadInt64(&s.numCallsFinished),
		atomic.LoadInt64(&s.numCallsFinishedWithClientFailedToSend),
		atomic.LoadInt64(&s.numCallsFinishedKnownReceived),
		s.numCallsDropped)
}

type remoteBalancer struct {
	lbgrpc.UnimplementedLoadBalancerServer
	sls           chan *lbpb.ServerList
	statsDura     time.Duration
	done          chan struct{}
	stats         *rpcStats
	statsChan     chan *lbpb.ClientStats
	fbChan        chan struct{}
	balanceLoadCh chan struct{} // notify successful invocation of BalanceLoad

	wantUserAgent  string // expected user-agent in metadata of BalancerLoad
	wantServerName string // expected server name in InitialLoadBalanceRequest
}

func newRemoteBalancer(wantUserAgent, wantServerName string, statsChan chan *lbpb.ClientStats) *remoteBalancer {
	return &remoteBalancer{
		sls:            make(chan *lbpb.ServerList, 1),
		done:           make(chan struct{}),
		stats:          newRPCStats(),
		statsChan:      statsChan,
		fbChan:         make(chan struct{}),
		balanceLoadCh:  make(chan struct{}, 1),
		wantUserAgent:  wantUserAgent,
		wantServerName: wantServerName,
	}
}

func (b *remoteBalancer) stop() {
	close(b.sls)
	close(b.done)
}

func (b *remoteBalancer) fallbackNow() {
	b.fbChan <- struct{}{}
}

func (b *remoteBalancer) updateServerName(name string) {
	b.wantServerName = name
}

func (b *remoteBalancer) BalanceLoad(stream lbgrpc.LoadBalancer_BalanceLoadServer) error {
	md, ok := metadata.FromIncomingContext(stream.Context())
	if !ok {
		return status.Error(codes.Internal, "failed to receive metadata")
	}
	if b.wantUserAgent != "" {
		if ua := md["user-agent"]; len(ua) == 0 || !strings.HasPrefix(ua[0], b.wantUserAgent) {
			return status.Errorf(codes.InvalidArgument, "received unexpected user-agent: %v, want prefix %q", ua, b.wantUserAgent)
		}
	}

	req, err := stream.Recv()
	if err != nil {
		return err
	}
	initReq := req.GetInitialRequest()
	if initReq.Name != b.wantServerName {
		return status.Errorf(codes.InvalidArgument, "invalid service name: %q, want: %q", initReq.Name, b.wantServerName)
	}
	b.balanceLoadCh <- struct{}{}
	resp := &lbpb.LoadBalanceResponse{
		LoadBalanceResponseType: &lbpb.LoadBalanceResponse_InitialResponse{
			InitialResponse: &lbpb.InitialLoadBalanceResponse{
				ClientStatsReportInterval: &durationpb.Duration{
					Seconds: int64(b.statsDura.Seconds()),
					Nanos:   int32(b.statsDura.Nanoseconds() - int64(b.statsDura.Seconds())*1e9),
				},
			},
		},
	}
	if err := stream.Send(resp); err != nil {
		return err
	}
	go func() {
		for {
			req, err := stream.Recv()
			if err != nil {
				return
			}
			b.stats.merge(req.GetClientStats())
			if b.statsChan != nil && req.GetClientStats() != nil {
				b.statsChan <- req.GetClientStats()
			}
		}
	}()
	for {
		select {
		case v := <-b.sls:
			resp = &lbpb.LoadBalanceResponse{
				LoadBalanceResponseType: &lbpb.LoadBalanceResponse_ServerList{
					ServerList: v,
				},
			}
		case <-b.fbChan:
			resp = &lbpb.LoadBalanceResponse{
				LoadBalanceResponseType: &lbpb.LoadBalanceResponse_FallbackResponse{
					FallbackResponse: &lbpb.FallbackResponse{},
				},
			}
		case <-stream.Context().Done():
			return stream.Context().Err()
		}
		if err := stream.Send(resp); err != nil {
			return err
		}
	}
}

type testServer struct {
	testgrpc.UnimplementedTestServiceServer

	addr     string
	fallback bool
}

const testmdkey = "testmd"

func (s *testServer) EmptyCall(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return nil, status.Error(codes.Internal, "failed to receive metadata")
	}
	if !s.fallback && (md == nil || len(md["lb-token"]) == 0 || md["lb-token"][0] != lbToken) {
		return nil, status.Errorf(codes.Internal, "received unexpected metadata: %v", md)
	}
	grpc.SetTrailer(ctx, metadata.Pairs(testmdkey, s.addr))
	return &testpb.Empty{}, nil
}

func (s *testServer) FullDuplexCall(testgrpc.TestService_FullDuplexCallServer) error {
	return nil
}

func startBackends(t *testing.T, sn string, fallback bool, lis ...net.Listener) (servers []*grpc.Server) {
	for _, l := range lis {
		creds := &serverNameCheckCreds{
			sn: sn,
		}
		s := grpc.NewServer(grpc.Creds(creds))
		testgrpc.RegisterTestServiceServer(s, &testServer{addr: l.Addr().String(), fallback: fallback})
		servers = append(servers, s)
		go func(s *grpc.Server, l net.Listener) {
			s.Serve(l)
		}(s, l)
		t.Logf("Started backend server listening on %s", l.Addr().String())
	}
	return
}

func stopBackends(servers []*grpc.Server) {
	for _, s := range servers {
		s.Stop()
	}
}

type testServers struct {
	lbAddr   string
	ls       *remoteBalancer
	lb       *grpc.Server
	backends []*grpc.Server
	beIPs    []net.IP
	bePorts  []int

	lbListener  net.Listener
	beListeners []net.Listener
}

func startBackendsAndRemoteLoadBalancer(t *testing.T, numberOfBackends int, customUserAgent string, statsChan chan *lbpb.ClientStats) (tss *testServers, cleanup func(), err error) {
	var (
		beListeners []net.Listener
		ls          *remoteBalancer
		lb          *grpc.Server
		beIPs       []net.IP
		bePorts     []int
	)
	for i := 0; i < numberOfBackends; i++ {
		beLis, e := net.Listen("tcp", "localhost:0")
		if e != nil {
			err = fmt.Errorf("failed to listen %v", err)
			return
		}
		beIPs = append(beIPs, beLis.Addr().(*net.TCPAddr).IP)
		bePorts = append(bePorts, beLis.Addr().(*net.TCPAddr).Port)

		beListeners = append(beListeners, testutils.NewRestartableListener(beLis))
	}
	backends := startBackends(t, beServerName, false, beListeners...)

	lbLis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		err = fmt.Errorf("failed to create the listener for the load balancer %v", err)
		return
	}
	lbLis = testutils.NewRestartableListener(lbLis)
	lbCreds := &serverNameCheckCreds{
		sn: lbServerName,
	}
	lb = grpc.NewServer(grpc.Creds(lbCreds))
	ls = newRemoteBalancer(customUserAgent, beServerName, statsChan)
	lbgrpc.RegisterLoadBalancerServer(lb, ls)
	go func() {
		lb.Serve(lbLis)
	}()
	t.Logf("Started remote load balancer server listening on %s", lbLis.Addr().String())

	tss = &testServers{
		lbAddr:   net.JoinHostPort(fakeName, strconv.Itoa(lbLis.Addr().(*net.TCPAddr).Port)),
		ls:       ls,
		lb:       lb,
		backends: backends,
		beIPs:    beIPs,
		bePorts:  bePorts,

		lbListener:  lbLis,
		beListeners: beListeners,
	}
	cleanup = func() {
		defer stopBackends(backends)
		defer func() {
			ls.stop()
			lb.Stop()
		}()
	}
	return
}

// TestGRPCLB_Basic tests the basic case of a channel being configured with
// grpclb as the load balancing policy.
func (s) TestGRPCLB_Basic(t *testing.T) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, testUserAgent, nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()

	// Push the test backend address to the remote balancer.
	tss.ls.sls <- &lbpb.ServerList{
		Servers: []*lbpb.Server{
			{
				IpAddress:        tss.beIPs[0],
				Port:             int32(tss.bePorts[0]),
				LoadBalanceToken: lbToken,
			},
		},
	}

	// Configure the manual resolver with an initial state containing a service
	// config with grpclb as the load balancing policy and the remote balancer
	// address specified via attributes.
	r := manual.NewBuilderWithScheme("whatever")
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs := grpclbstate.Set(resolver.State{ServiceConfig: internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(grpclbConfig)}, s)
	r.InitialState(rs)

	// Connect to the test backend.
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
		grpc.WithUserAgent(testUserAgent),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()

	// Make one successful RPC.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testC := testgrpc.NewTestServiceClient(cc)
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
	}
}

// TestGRPCLB_Weighted tests weighted roundrobin. The remote balancer is
// configured to send a response with duplicate backend addresses (to simulate
// weights) to the grpclb client. The test verifies that RPCs are weighted
// roundrobin-ed across these backends.
func (s) TestGRPCLB_Weighted(t *testing.T) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 2, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()

	beServers := []*lbpb.Server{{
		IpAddress:        tss.beIPs[0],
		Port:             int32(tss.bePorts[0]),
		LoadBalanceToken: lbToken,
	}, {
		IpAddress:        tss.beIPs[1],
		Port:             int32(tss.bePorts[1]),
		LoadBalanceToken: lbToken,
	}}

	// Configure the manual resolver with an initial state containing a service
	// config with grpclb as the load balancing policy and the remote balancer
	// address specified via attributes.
	r := manual.NewBuilderWithScheme("whatever")
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs := grpclbstate.Set(resolver.State{ServiceConfig: internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(grpclbConfig)}, s)
	r.InitialState(rs)

	// Connect to test backends.
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Sequence represents the sequence of backends to be returned from the
	// remote load balancer.
	sequences := [][]int{
		{0, 0, 1, 0, 1},
		{0, 0, 0, 1, 1},
	}
	for _, seq := range sequences {
		// Push the configured sequence of backend to the remote balancer, and
		// compute the expected addresses to which RPCs should be routed.
		var backends []*lbpb.Server
		var wantAddrs []resolver.Address
		for _, s := range seq {
			backends = append(backends, beServers[s])
			wantAddrs = append(wantAddrs, resolver.Address{Addr: tss.beListeners[s].Addr().String()})
		}
		tss.ls.sls <- &lbpb.ServerList{Servers: backends}

		testC := testgrpc.NewTestServiceClient(cc)
		if err := roundrobin.CheckWeightedRoundRobinRPCs(ctx, testC, wantAddrs); err != nil {
			t.Fatal(err)
		}
	}
}

// TestGRPCLB_DropRequest tests grpclb support for dropping requests based on
// configuration received from the remote balancer.
//
// TODO: Rewrite this test to verify drop behavior using the
// ClientStats.CallsFinishedWithDrop field instead.
func (s) TestGRPCLB_DropRequest(t *testing.T) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 2, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	tss.ls.sls <- &lbpb.ServerList{
		Servers: []*lbpb.Server{{
			IpAddress:        tss.beIPs[0],
			Port:             int32(tss.bePorts[0]),
			LoadBalanceToken: lbToken,
			Drop:             false,
		}, {
			IpAddress:        tss.beIPs[1],
			Port:             int32(tss.bePorts[1]),
			LoadBalanceToken: lbToken,
			Drop:             false,
		}, {
			Drop: true,
		}},
	}

	// Configure the manual resolver with an initial state containing a service
	// config with grpclb as the load balancing policy and the remote balancer
	// address specified via attributes.
	r := manual.NewBuilderWithScheme("whatever")
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs := grpclbstate.Set(resolver.State{ServiceConfig: internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(grpclbConfig)}, s)
	r.InitialState(rs)

	// Connect to test backends.
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	var (
		i int
		p peer.Peer
	)
	const (
		// Poll to wait for something to happen. Total timeout 1 second. Sleep 1
		// ms each loop, and do at most 1000 loops.
		sleepEachLoop = time.Millisecond
		loopCount     = int(time.Second / sleepEachLoop)
	)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Make a non-fail-fast RPC and wait for it to succeed.
	for i = 0; i < loopCount; i++ {
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&p)); err == nil {
			break
		}
		time.Sleep(sleepEachLoop)
	}
	if i >= loopCount {
		t.Fatalf("timeout waiting for the first connection to become ready. EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}

	// Make RPCs until the peer is different. So we know both connections are
	// READY.
	for i = 0; i < loopCount; i++ {
		var temp peer.Peer
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&temp)); err == nil {
			if temp.Addr.(*net.TCPAddr).Port != p.Addr.(*net.TCPAddr).Port {
				break
			}
		}
		time.Sleep(sleepEachLoop)
	}
	if i >= loopCount {
		t.Fatalf("timeout waiting for the second connection to become ready")
	}

	// More RPCs until drop happens. So we know the picker index, and the
	// expected behavior of following RPCs.
	for i = 0; i < loopCount; i++ {
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) == codes.Unavailable {
			break
		}
		time.Sleep(sleepEachLoop)
	}
	if i >= loopCount {
		t.Fatalf("timeout waiting for drop. EmptyCall(_, _) = _, %v, want _, <Unavailable>", err)
	}

	select {
	case <-ctx.Done():
		t.Fatal("timed out", ctx.Err())
	default:
	}
	for _, failfast := range []bool{true, false} {
		for i := 0; i < 3; i++ {
			// 1st RPCs pick the first item in server list. They should succeed
			// since they choose the non-drop-request backend according to the
			// round robin policy.
			if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(!failfast)); err != nil {
				t.Errorf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
			}
			// 2nd RPCs pick the second item in server list. They should succeed
			// since they choose the non-drop-request backend according to the
			// round robin policy.
			if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(!failfast)); err != nil {
				t.Errorf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
			}
			// 3rd RPCs should fail, because they pick last item in server list,
			// with Drop set to true.
			if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(!failfast)); status.Code(err) != codes.Unavailable {
				t.Errorf("%v.EmptyCall(_, _) = _, %v, want _, %s", testC, err, codes.Unavailable)
			}
		}
	}

	// Make one more RPC to move the picker index one step further, so it's not
	// 0. The following RPCs will test that drop index is not reset. If picker
	// index is at 0, we cannot tell whether it's reset or not.
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Errorf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
	}

	tss.backends[0].Stop()
	// This last pick was backend 0. Closing backend 0 doesn't reset drop index
	// (for level 1 picking), so the following picks will be (backend1, drop,
	// backend1), instead of (backend, backend, drop) if drop index was reset.
	time.Sleep(time.Second)
	for i := 0; i < 3; i++ {
		var p peer.Peer
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&p)); err != nil {
			t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		if want := tss.bePorts[1]; p.Addr.(*net.TCPAddr).Port != want {
			t.Errorf("got peer: %v, want peer port: %v", p.Addr, want)
		}

		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.Unavailable {
			t.Errorf("%v.EmptyCall(_, _) = _, %v, want _, %s", testC, err, codes.Unavailable)
		}

		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&p)); err != nil {
			t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		if want := tss.bePorts[1]; p.Addr.(*net.TCPAddr).Port != want {
			t.Errorf("got peer: %v, want peer port: %v", p.Addr, want)
		}
	}
}

// TestGRPCLB_BalancerDisconnects tests the case where the remote balancer in
// use disconnects. The test verifies that grpclb connects to the next remote
// balancer address specified in attributes, and RPCs get routed to the backends
// returned by the new balancer.
func (s) TestGRPCLB_BalancerDisconnects(t *testing.T) {
	var (
		tests []*testServers
		lbs   []*grpc.Server
	)
	for i := 0; i < 2; i++ {
		tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
		if err != nil {
			t.Fatalf("failed to create new load balancer: %v", err)
		}
		defer cleanup()

		tss.ls.sls <- &lbpb.ServerList{
			Servers: []*lbpb.Server{
				{
					IpAddress:        tss.beIPs[0],
					Port:             int32(tss.bePorts[0]),
					LoadBalanceToken: lbToken,
				},
			},
		}

		tests = append(tests, tss)
		lbs = append(lbs, tss.lb)
	}

	// Configure the manual resolver with an initial state containing a service
	// config with grpclb as the load balancing policy and the remote balancer
	// addresses specified via attributes.
	r := manual.NewBuilderWithScheme("whatever")
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tests[0].lbAddr,
				ServerName: lbServerName,
			},
			{
				Addr:       tests[1].lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs := grpclbstate.Set(resolver.State{ServiceConfig: internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(grpclbConfig)}, s)
	r.InitialState(rs)

	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tests[0].beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Stop balancer[0], balancer[1] should be used by grpclb.
	// Check peer address to see if that happened.
	lbs[0].Stop()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tests[1].beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}
}

// TestGRPCLB_Fallback tests the following fallback scenarios:
//   - when the remote balancer address specified in attributes is invalid, the
//     test verifies that RPCs are routed to the fallback backend.
//   - when the remote balancer address specified in attributes is changed to a
//     valid one, the test verifies that RPCs are routed to the backend returned
//     by the remote balancer.
//   - when the configured remote balancer goes down, the test verifies that
//     RPCs are routed to the fallback backend.
func (s) TestGRPCLB_Fallback(t *testing.T) {
	balancer.Register(newLBBuilderWithFallbackTimeout(100 * time.Millisecond))
	defer balancer.Register(newLBBuilder())

	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	sl := &lbpb.ServerList{
		Servers: []*lbpb.Server{
			{
				IpAddress:        tss.beIPs[0],
				Port:             int32(tss.bePorts[0]),
				LoadBalanceToken: lbToken,
			},
		},
	}
	// Push the backend address to the remote balancer.
	tss.ls.sls <- sl

	// Start a standalone backend for fallback.
	beLis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen %v", err)
	}
	defer beLis.Close()
	standaloneBEs := startBackends(t, beServerName, true, beLis)
	defer stopBackends(standaloneBEs)

	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.Dial(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	// Push an update to the resolver with fallback backend address stored in
	// the `Addresses` field and an invalid remote balancer address stored in
	// attributes, which will cause fallback behavior to be invoked.
	rs := resolver.State{
		Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
		ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig),
	}
	rs = grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: "invalid.address", ServerName: lbServerName}}})
	r.UpdateState(rs)

	// Make an RPC and verify that it got routed to the fallback backend.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: beLis.Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Push another update to the resolver, this time with a valid balancer
	// address in the attributes field.
	rs = resolver.State{
		ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig),
		Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
	}
	rs = grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: tss.lbAddr, ServerName: lbServerName}}})
	r.UpdateState(rs)
	select {
	case <-ctx.Done():
		t.Fatalf("timeout when waiting for BalanceLoad RPC to be called on the remote balancer")
	case <-tss.ls.balanceLoadCh:
	}

	// Wait for RPCs to get routed to the backend behind the remote balancer.
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tss.beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Close backend and remote balancer connections, should use fallback.
	tss.beListeners[0].(*testutils.RestartableListener).Stop()
	tss.lbListener.(*testutils.RestartableListener).Stop()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: beLis.Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Restart backend and remote balancer, should not use fallback backend.
	tss.beListeners[0].(*testutils.RestartableListener).Restart()
	tss.lbListener.(*testutils.RestartableListener).Restart()
	tss.ls.sls <- sl
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tss.beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}
}

// TestGRPCLB_ExplicitFallback tests the case where the remote balancer sends an
// explicit fallback signal to the grpclb client, and the test verifies that
// RPCs are routed to the fallback backend.
func (s) TestGRPCLB_ExplicitFallback(t *testing.T) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	sl := &lbpb.ServerList{
		Servers: []*lbpb.Server{
			{
				IpAddress:        tss.beIPs[0],
				Port:             int32(tss.bePorts[0]),
				LoadBalanceToken: lbToken,
			},
		},
	}
	// Push the backend address to the remote balancer.
	tss.ls.sls <- sl

	// Start a standalone backend for fallback.
	beLis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen %v", err)
	}
	defer beLis.Close()
	standaloneBEs := startBackends(t, beServerName, true, beLis)
	defer stopBackends(standaloneBEs)

	// Configure the manual resolver with an initial state containing a service
	// config with grpclb as the load balancing policy and the address of the
	// fallback backend. The remote balancer address is specified via
	// attributes.
	r := manual.NewBuilderWithScheme("whatever")
	rs := resolver.State{
		Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
		ServiceConfig: internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(grpclbConfig),
	}
	rs = grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: tss.lbAddr, ServerName: lbServerName}}})
	r.InitialState(rs)

	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.NewClient(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tss.beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Send fallback signal from remote balancer; should use fallback.
	tss.ls.fallbackNow()
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: beLis.Addr().String()}}); err != nil {
		t.Fatal(err)
	}

	// Send another server list; should use backends again.
	tss.ls.sls <- sl
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tss.beListeners[0].Addr().String()}}); err != nil {
		t.Fatal(err)
	}
}

// TestGRPCLB_FallBackWithNoServerAddress tests the fallback case where no
// backend addresses are returned by the remote balancer.
func (s) TestGRPCLB_FallBackWithNoServerAddress(t *testing.T) {
	resolveNowCh := testutils.NewChannel()
	r := manual.NewBuilderWithScheme("whatever")
	r.ResolveNowCallback = func(resolver.ResolveNowOptions) {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		defer cancel()
		if err := resolveNowCh.SendContext(ctx, nil); err != nil {
			t.Error("timeout when attempting to send on resolverNowCh")
		}
	}

	// Start a remote balancer and a backend. Don't push the backend address to
	// the remote balancer yet.
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	sl := &lbpb.ServerList{
		Servers: []*lbpb.Server{
			{
				IpAddress:        tss.beIPs[0],
				Port:             int32(tss.bePorts[0]),
				LoadBalanceToken: lbToken,
			},
		},
	}

	// Start a standalone backend for fallback.
	beLis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen %v", err)
	}
	defer beLis.Close()
	standaloneBEs := startBackends(t, beServerName, true, beLis)
	defer stopBackends(standaloneBEs)

	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.Dial(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < 2; i++ {
		// Send an update with only backend address. grpclb should enter
		// fallback and use the fallback backend.
		r.UpdateState(resolver.State{
			Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
			ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig),
		})

		sCtx, sCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		defer sCancel()
		if _, err := resolveNowCh.Receive(sCtx); err != context.DeadlineExceeded {
			t.Fatalf("unexpected resolveNow when grpclb gets no balancer address 1111, %d", i)
		}

		var p peer.Peer
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true), grpc.Peer(&p)); err != nil {
			t.Fatalf("_.EmptyCall(_, _) = _, %v, want _, <nil>", err)
		}
		if p.Addr.String() != beLis.Addr().String() {
			t.Fatalf("got peer: %v, want peer: %v", p.Addr, beLis.Addr())
		}

		sCtx, sCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
		defer sCancel()
		if _, err := resolveNowCh.Receive(sCtx); err != context.DeadlineExceeded {
			t.Errorf("unexpected resolveNow when grpclb gets no balancer address 2222, %d", i)
		}

		tss.ls.sls <- sl
		// Send an update with balancer address. The backends behind grpclb should
		// be used.
		rs := resolver.State{
			Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
			ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig),
		}
		rs = grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: tss.lbAddr, ServerName: lbServerName}}})
		r.UpdateState(rs)

		select {
		case <-ctx.Done():
			t.Fatalf("timeout when waiting for BalanceLoad RPC to be called on the remote balancer")
		case <-tss.ls.balanceLoadCh:
		}

		if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, []resolver.Address{{Addr: tss.beListeners[0].Addr().String()}}); err != nil {
			t.Fatal(err)
		}
	}
}

// TestGRPCLB_PickFirst configures grpclb with pick_first as the child policy.
// The test changes the list of backend addresses returned by the remote
// balancer and verifies that RPCs are sent to the first address returned.
func (s) TestGRPCLB_PickFirst(t *testing.T) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 3, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()

	beServers := []*lbpb.Server{{
		IpAddress:        tss.beIPs[0],
		Port:             int32(tss.bePorts[0]),
		LoadBalanceToken: lbToken,
	}, {
		IpAddress:        tss.beIPs[1],
		Port:             int32(tss.bePorts[1]),
		LoadBalanceToken: lbToken,
	}, {
		IpAddress:        tss.beIPs[2],
		Port:             int32(tss.bePorts[2]),
		LoadBalanceToken: lbToken,
	}}
	beServerAddrs := []resolver.Address{}
	for _, lis := range tss.beListeners {
		beServerAddrs = append(beServerAddrs, resolver.Address{Addr: lis.Addr().String()})
	}

	// Connect to the test backends.
	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.Dial(r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()

	// Push a service config with grpclb as the load balancing policy and
	// configure pick_first as its child policy.
	rs := resolver.State{ServiceConfig: r.CC.ParseServiceConfig(`{"loadBalancingConfig":[{"grpclb":{"childPolicy":[{"pick_first":{}}]}}]}`)}

	// Push a resolver update with the remote balancer address specified via
	// attributes.
	r.UpdateState(grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: tss.lbAddr, ServerName: lbServerName}}}))

	// Push all three backend addresses to the remote balancer, and verify that
	// RPCs are routed to the first backend.
	tss.ls.sls <- &lbpb.ServerList{Servers: beServers[0:3]}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, beServerAddrs[0]); err != nil {
		t.Fatal(err)
	}

	// Update the address list with the remote balancer and verify pick_first
	// behavior based on the new backends.
	tss.ls.sls <- &lbpb.ServerList{Servers: beServers[2:]}
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, beServerAddrs[2]); err != nil {
		t.Fatal(err)
	}

	// Update the address list with the remote balancer and verify pick_first
	// behavior based on the new backends. Since the currently connected backend
	// is in the new list (even though it is not the first one on the list),
	// pick_first will continue to use it.
	tss.ls.sls <- &lbpb.ServerList{Servers: beServers[1:]}
	if err := pickfirst.CheckRPCsToBackend(ctx, cc, beServerAddrs[2]); err != nil {
		t.Fatal(err)
	}

	// Switch child policy to roundrobin.
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs = grpclbstate.Set(resolver.State{ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig)}, s)
	r.UpdateState(rs)
	testC := testgrpc.NewTestServiceClient(cc)
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, beServerAddrs[1:]); err != nil {
		t.Fatal(err)
	}

	tss.ls.sls <- &lbpb.ServerList{Servers: beServers[0:3]}
	if err := roundrobin.CheckRoundRobinRPCs(ctx, testC, beServerAddrs[0:3]); err != nil {
		t.Fatal(err)
	}
}

// TestGRPCLB_BackendConnectionErrorPropagation tests the case where grpclb
// falls back to a backend which returns an error and the test verifies that the
// error is propagated to the RPC.
func (s) TestGRPCLB_BackendConnectionErrorPropagation(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")

	// Start up an LB which will tells the client to fall back right away.
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 0, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()

	// Start a standalone backend, to be used during fallback. The creds
	// are intentionally misconfigured in order to simulate failure of a
	// security handshake.
	beLis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen %v", err)
	}
	defer beLis.Close()
	standaloneBEs := startBackends(t, "arbitrary.invalid.name", true, beLis)
	defer stopBackends(standaloneBEs)

	cc, err := grpc.Dial(r.Scheme()+":///"+beServerName,
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer))
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	rs := resolver.State{
		Addresses:     []resolver.Address{{Addr: beLis.Addr().String()}},
		ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig),
	}
	rs = grpclbstate.Set(rs, &grpclbstate.State{BalancerAddresses: []resolver.Address{{Addr: tss.lbAddr, ServerName: lbServerName}}})
	r.UpdateState(rs)

	// If https://github.com/grpc/grpc-go/blob/65cabd74d8e18d7347fecd414fa8d83a00035f5f/balancer/grpclb/grpclb_test.go#L103
	// changes, then expectedErrMsg may need to be updated.
	const expectedErrMsg = "received unexpected server name"
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		tss.ls.fallbackNow()
		wg.Done()
	}()
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}); err == nil || !strings.Contains(err.Error(), expectedErrMsg) {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, rpc error containing substring: %q", testC, err, expectedErrMsg)
	}
	wg.Wait()
}

func testGRPCLBEmptyServerList(t *testing.T, svcfg string) {
	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()

	beServers := []*lbpb.Server{{
		IpAddress:        tss.beIPs[0],
		Port:             int32(tss.bePorts[0]),
		LoadBalanceToken: lbToken,
	}}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	r := manual.NewBuilderWithScheme("whatever")
	dopts := []grpc.DialOption{
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
	}
	cc, err := grpc.DialContext(ctx, r.Scheme()+":///"+beServerName, dopts...)
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	tss.ls.sls <- &lbpb.ServerList{Servers: beServers}

	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs := grpclbstate.Set(resolver.State{ServiceConfig: r.CC.ParseServiceConfig(svcfg)}, s)
	r.UpdateState(rs)
	t.Log("Perform an initial RPC and expect it to succeed...")
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("Initial _.EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	t.Log("Now send an empty server list. Wait until we see an RPC failure to make sure the client got it...")
	tss.ls.sls <- &lbpb.ServerList{}
	gotError := false
	for ; ctx.Err() == nil; <-time.After(time.Millisecond) {
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}); err != nil {
			gotError = true
			break
		}
	}
	if !gotError {
		t.Fatalf("Expected to eventually see an RPC fail after the grpclb sends an empty server list, but none did.")
	}
	t.Log("Now send a non-empty server list. A wait-for-ready RPC should now succeed...")
	tss.ls.sls <- &lbpb.ServerList{Servers: beServers}
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("Final _.EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
}

func (s) TestGRPCLBEmptyServerListRoundRobin(t *testing.T) {
	testGRPCLBEmptyServerList(t, `{"loadBalancingConfig":[{"grpclb":{"childPolicy":[{"round_robin":{}}]}}]}`)
}

func (s) TestGRPCLBEmptyServerListPickFirst(t *testing.T) {
	testGRPCLBEmptyServerList(t, `{"loadBalancingConfig":[{"grpclb":{"childPolicy":[{"pick_first":{}}]}}]}`)
}

func (s) TestGRPCLBWithTargetNameFieldInConfig(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")

	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", nil)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	sl := &lbpb.ServerList{
		Servers: []*lbpb.Server{
			{
				IpAddress:        tss.beIPs[0],
				Port:             int32(tss.bePorts[0]),
				LoadBalanceToken: lbToken,
			},
		},
	}
	// Push the backend address to the remote balancer.
	tss.ls.sls <- sl

	cc, err := grpc.Dial(r.Scheme()+":///"+beServerName,
		grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&serverNameCheckCreds{}),
		grpc.WithContextDialer(fakeNameDialer),
		grpc.WithUserAgent(testUserAgent))
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()
	testC := testgrpc.NewTestServiceClient(cc)

	// Push a resolver update with grpclb configuration which does not contain the
	// target_name field. Our fake remote balancer is configured to always
	// expect `beServerName` as the server name in the initial request.
	rs := grpclbstate.Set(resolver.State{ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig)},
		&grpclbstate.State{BalancerAddresses: []resolver.Address{{
			Addr:       tss.lbAddr,
			ServerName: lbServerName,
		}}})
	r.UpdateState(rs)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	select {
	case <-ctx.Done():
		t.Fatalf("timeout when waiting for BalanceLoad RPC to be called on the remote balancer")
	case <-tss.ls.balanceLoadCh:
	}
	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
	}

	// When the value of target_field changes, grpclb will recreate the stream
	// to the remote balancer. So, we need to update the fake remote balancer to
	// expect a new server name in the initial request.
	const newServerName = "new-server-name"
	tss.ls.updateServerName(newServerName)
	tss.ls.sls <- sl

	// Push the resolver update with target_field changed.
	// Push a resolver update with grpclb configuration containing the
	// target_name field. Our fake remote balancer has been updated above to expect the newServerName in the initial request.
	lbCfg := fmt.Sprintf(`{"loadBalancingConfig": [{"grpclb": {"serviceName": "%s"}}]}`, newServerName)
	s := &grpclbstate.State{
		BalancerAddresses: []resolver.Address{
			{
				Addr:       tss.lbAddr,
				ServerName: lbServerName,
			},
		},
	}
	rs = grpclbstate.Set(resolver.State{ServiceConfig: r.CC.ParseServiceConfig(lbCfg)}, s)
	r.UpdateState(rs)
	select {
	case <-ctx.Done():
		t.Fatalf("timeout when waiting for BalanceLoad RPC to be called on the remote balancer")
	case <-tss.ls.balanceLoadCh:
	}

	if _, err := testC.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
	}
}

type failPreRPCCred struct{}

func (failPreRPCCred) GetRequestMetadata(_ context.Context, uri ...string) (map[string]string, error) {
	if strings.Contains(uri[0], failtosendURI) {
		return nil, fmt.Errorf("rpc should fail to send")
	}
	return nil, nil
}

func (failPreRPCCred) RequireTransportSecurity() bool {
	return false
}

func checkStats(stats, expected *rpcStats) error {
	if !stats.equal(expected) {
		return fmt.Errorf("stats not equal: got %+v, want %+v", stats, expected)
	}
	return nil
}

func runAndCheckStats(t *testing.T, drop bool, statsChan chan *lbpb.ClientStats, runRPCs func(*grpc.ClientConn), statsWant *rpcStats) error {
	r := manual.NewBuilderWithScheme("whatever")

	tss, cleanup, err := startBackendsAndRemoteLoadBalancer(t, 1, "", statsChan)
	if err != nil {
		t.Fatalf("failed to create new load balancer: %v", err)
	}
	defer cleanup()
	servers := []*lbpb.Server{{
		IpAddress:        tss.beIPs[0],
		Port:             int32(tss.bePorts[0]),
		LoadBalanceToken: lbToken,
	}}
	if drop {
		servers = append(servers, &lbpb.Server{
			LoadBalanceToken: lbToken,
			Drop:             drop,
		})
	}
	tss.ls.sls <- &lbpb.ServerList{Servers: servers}
	tss.ls.statsDura = 100 * time.Millisecond
	creds := serverNameCheckCreds{}

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	cc, err := grpc.DialContext(ctx, r.Scheme()+":///"+beServerName, grpc.WithResolvers(r),
		grpc.WithTransportCredentials(&creds),
		grpc.WithPerRPCCredentials(failPreRPCCred{}),
		grpc.WithContextDialer(fakeNameDialer))
	if err != nil {
		t.Fatalf("Failed to dial to the backend %v", err)
	}
	defer cc.Close()

	rstate := resolver.State{ServiceConfig: r.CC.ParseServiceConfig(grpclbConfig)}
	r.UpdateState(grpclbstate.Set(rstate, &grpclbstate.State{BalancerAddresses: []resolver.Address{{
		Addr:       tss.lbAddr,
		ServerName: lbServerName,
	}}}))

	runRPCs(cc)
	end := time.Now().Add(time.Second)
	for time.Now().Before(end) {
		if err := checkStats(tss.ls.stats, statsWant); err == nil {
			time.Sleep(200 * time.Millisecond) // sleep for two intervals to make sure no new stats are reported.
			break
		}
	}
	return checkStats(tss.ls.stats, statsWant)
}

const (
	countRPC      = 40
	failtosendURI = "failtosend"
)

func (s) TestGRPCLBStatsUnarySuccess(t *testing.T) {
	if err := runAndCheckStats(t, false, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for i := 0; i < countRPC-1; i++ {
			testC.EmptyCall(ctx, &testpb.Empty{})
		}
	}, &rpcStats{
		numCallsStarted:               int64(countRPC),
		numCallsFinished:              int64(countRPC),
		numCallsFinishedKnownReceived: int64(countRPC),
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsUnaryDrop(t *testing.T) {
	if err := runAndCheckStats(t, true, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for i := 0; i < countRPC-1; i++ {
			testC.EmptyCall(ctx, &testpb.Empty{})
		}
	}, &rpcStats{
		numCallsStarted:               int64(countRPC),
		numCallsFinished:              int64(countRPC),
		numCallsFinishedKnownReceived: int64(countRPC) / 2,
		numCallsDropped:               map[string]int64{lbToken: int64(countRPC) / 2},
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsUnaryFailedToSend(t *testing.T) {
	if err := runAndCheckStats(t, false, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		if _, err := testC.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
			t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for i := 0; i < countRPC-1; i++ {
			cc.Invoke(ctx, failtosendURI, &testpb.Empty{}, nil)
		}
	}, &rpcStats{
		numCallsStarted:                        int64(countRPC),
		numCallsFinished:                       int64(countRPC),
		numCallsFinishedWithClientFailedToSend: int64(countRPC) - 1,
		numCallsFinishedKnownReceived:          1,
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsStreamingSuccess(t *testing.T) {
	if err := runAndCheckStats(t, false, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		stream, err := testC.FullDuplexCall(ctx, grpc.WaitForReady(true))
		if err != nil {
			t.Fatalf("%v.FullDuplexCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for {
			if _, err = stream.Recv(); err == io.EOF {
				break
			}
		}
		for i := 0; i < countRPC-1; i++ {
			stream, err = testC.FullDuplexCall(ctx)
			if err == nil {
				// Wait for stream to end if err is nil.
				for {
					if _, err = stream.Recv(); err == io.EOF {
						break
					}
				}
			}
		}
	}, &rpcStats{
		numCallsStarted:               int64(countRPC),
		numCallsFinished:              int64(countRPC),
		numCallsFinishedKnownReceived: int64(countRPC),
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsStreamingDrop(t *testing.T) {
	if err := runAndCheckStats(t, true, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		stream, err := testC.FullDuplexCall(ctx, grpc.WaitForReady(true))
		if err != nil {
			t.Fatalf("%v.FullDuplexCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for {
			if _, err = stream.Recv(); err == io.EOF {
				break
			}
		}
		for i := 0; i < countRPC-1; i++ {
			stream, err = testC.FullDuplexCall(ctx)
			if err == nil {
				// Wait for stream to end if err is nil.
				for {
					if _, err = stream.Recv(); err == io.EOF {
						break
					}
				}
			}
		}
	}, &rpcStats{
		numCallsStarted:               int64(countRPC),
		numCallsFinished:              int64(countRPC),
		numCallsFinishedKnownReceived: int64(countRPC) / 2,
		numCallsDropped:               map[string]int64{lbToken: int64(countRPC) / 2},
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsStreamingFailedToSend(t *testing.T) {
	if err := runAndCheckStats(t, false, nil, func(cc *grpc.ClientConn) {
		testC := testgrpc.NewTestServiceClient(cc)
		ctx, cancel := context.WithTimeout(context.Background(), defaultFallbackTimeout)
		defer cancel()
		// The first non-failfast RPC succeeds, all connections are up.
		stream, err := testC.FullDuplexCall(ctx, grpc.WaitForReady(true))
		if err != nil {
			t.Fatalf("%v.FullDuplexCall(_, _) = _, %v, want _, <nil>", testC, err)
		}
		for {
			if _, err = stream.Recv(); err == io.EOF {
				break
			}
		}
		for i := 0; i < countRPC-1; i++ {
			cc.NewStream(ctx, &grpc.StreamDesc{}, failtosendURI)
		}
	}, &rpcStats{
		numCallsStarted:                        int64(countRPC),
		numCallsFinished:                       int64(countRPC),
		numCallsFinishedWithClientFailedToSend: int64(countRPC) - 1,
		numCallsFinishedKnownReceived:          1,
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestGRPCLBStatsQuashEmpty(t *testing.T) {
	ch := make(chan *lbpb.ClientStats)
	defer close(ch)
	if err := runAndCheckStats(t, false, ch, func(*grpc.ClientConn) {
		// Perform no RPCs; wait for load reports to start, which should be
		// zero, then expect no other load report within 5x the update
		// interval.
		select {
		case st := <-ch:
			if !isZeroStats(st) {
				t.Errorf("got stats %v; want all zero", st)
			}
		case <-time.After(5 * time.Second):
			t.Errorf("did not get initial stats report after 5 seconds")
			return
		}

		select {
		case st := <-ch:
			t.Errorf("got unexpected stats report: %v", st)
		case <-time.After(500 * time.Millisecond):
			// Success.
		}
		go func() {
			for range ch { // Drain statsChan until it is closed.
			}
		}()
	}, &rpcStats{
		numCallsStarted:               0,
		numCallsFinished:              0,
		numCallsFinishedKnownReceived: 0,
	}); err != nil {
		t.Fatal(err)
	}
}
