/*
 *
 * Copyright 2018 gRPC authors.
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

package test

import (
	"context"
	"crypto/tls"
	"fmt"
	"net"
	"regexp"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/net/http2"
	"google.golang.org/grpc"
	_ "google.golang.org/grpc/balancer/grpclb"
	grpclbstate "google.golang.org/grpc/balancer/grpclb/state"
	"google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/keepalive"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

func verifyResultWithDelay(f func() (bool, error)) error {
	var ok bool
	var err error
	for i := 0; i < 1000; i++ {
		if ok, err = f(); ok {
			return nil
		}
		time.Sleep(10 * time.Millisecond)
	}
	return err
}

func (s) TestCZServerRegistrationAndDeletion(t *testing.T) {
	testcases := []struct {
		total  int
		start  int64
		max    int
		length int
		end    bool
	}{
		{total: int(channelz.EntriesPerPage), start: 0, max: 0, length: channelz.EntriesPerPage, end: true},
		{total: int(channelz.EntriesPerPage) - 1, start: 0, max: 0, length: channelz.EntriesPerPage - 1, end: true},
		{total: int(channelz.EntriesPerPage) + 1, start: 0, max: 0, length: channelz.EntriesPerPage, end: false},
		{total: int(channelz.EntriesPerPage) + 1, start: int64(2*(channelz.EntriesPerPage+1) + 1), max: 0, length: 0, end: true},
		{total: int(channelz.EntriesPerPage), start: 0, max: 1, length: 1, end: false},
		{total: int(channelz.EntriesPerPage), start: 0, max: channelz.EntriesPerPage - 1, length: channelz.EntriesPerPage - 1, end: false},
	}

	for i, c := range testcases {
		// Reset channelz IDs so `start` is valid.
		channelz.IDGen.Reset()

		e := tcpClearRREnv
		te := newTest(t, e)
		te.startServers(&testServer{security: e.security}, c.total)

		ss, end := channelz.GetServers(c.start, c.max)
		if len(ss) != c.length || end != c.end {
			t.Fatalf("%d: GetServers(%d) = %+v (len of which: %d), end: %+v, want len(GetServers(%d)) = %d, end: %+v", i, c.start, ss, len(ss), end, c.start, c.length, c.end)
		}
		te.tearDown()
		ss, end = channelz.GetServers(c.start, c.max)
		if len(ss) != 0 || !end {
			t.Fatalf("%d: GetServers(0) = %+v (len of which: %d), end: %+v, want len(GetServers(0)) = 0, end: true", i, ss, len(ss), end)
		}
	}
}

func (s) TestCZGetChannel(t *testing.T) {
	e := tcpClearRREnv
	e.balancer = ""
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.InitialState(resolver.State{Addresses: addrs})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		target := tcs[0].ChannelMetrics.Target.Load()
		wantTarget := "whatever:///" + te.srvAddr
		if target == nil || *target != wantTarget {
			return false, fmt.Errorf("Got channelz target=%v; want %q", target, wantTarget)
		}
		state := tcs[0].ChannelMetrics.State.Load()
		if state == nil || *state != connectivity.Ready {
			return false, fmt.Errorf("Got channelz state=%v; want %q", state, connectivity.Ready)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZGetSubChannel(t *testing.T) {
	e := tcpClearRREnv
	e.balancer = ""
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.InitialState(resolver.State{Addresses: addrs})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		scs := tcs[0].SubChans()
		if len(scs) != 1 {
			return false, fmt.Errorf("there should be one subchannel, not %d", len(scs))
		}
		var scid int64
		for scid = range scs {
		}
		sc := channelz.GetSubChannel(scid)
		if sc == nil {
			return false, fmt.Errorf("subchannel with id %v is nil", scid)
		}
		target := sc.ChannelMetrics.Target.Load()
		if target == nil || !strings.HasPrefix(*target, "localhost") {
			t.Fatalf("subchannel target must never be set incorrectly; got: %v, want <HasPrefix('localhost')>", target)
		}
		state := sc.ChannelMetrics.State.Load()
		if state == nil || *state != connectivity.Ready {
			return false, fmt.Errorf("Got subchannel state=%v; want %q", state, connectivity.Ready)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZGetServer(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	ss, _ := channelz.GetServers(0, 0)
	if len(ss) != 1 {
		t.Fatalf("there should only be one server, not %d", len(ss))
	}

	serverID := ss[0].ID
	srv := channelz.GetServer(serverID)
	if srv == nil {
		t.Fatalf("server %d does not exist", serverID)
	}
	if srv.ID != serverID {
		t.Fatalf("server want id %d, but got %d", serverID, srv.ID)
	}

	te.tearDown()

	if err := verifyResultWithDelay(func() (bool, error) {
		srv := channelz.GetServer(serverID)
		if srv != nil {
			return false, fmt.Errorf("server %d should not exist", serverID)
		}

		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZGetSocket(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	lis := te.listenAndServe(&testServer{security: e.security}, net.Listen)
	defer te.tearDown()

	if err := verifyResultWithDelay(func() (bool, error) {
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("len(ss) = %v; want %v", len(ss), 1)
		}

		serverID := ss[0].ID
		srv := channelz.GetServer(serverID)
		if srv == nil {
			return false, fmt.Errorf("server %d does not exist", serverID)
		}
		if srv.ID != serverID {
			return false, fmt.Errorf("srv.ID = %d; want %v", srv.ID, serverID)
		}

		skts := srv.ListenSockets()
		if got, want := len(skts), 1; got != want {
			return false, fmt.Errorf("len(skts) = %v; want %v", got, want)
		}
		var sktID int64
		for sktID = range skts {
		}

		skt := channelz.GetSocket(sktID)
		if skt == nil {
			return false, fmt.Errorf("socket %v does not exist", sktID)
		}

		if got, want := skt.LocalAddr, lis.Addr(); got != want {
			return false, fmt.Errorf("socket %v LocalAddr=%v; want %v", sktID, got, want)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZTopChannelRegistrationAndDeletion(t *testing.T) {
	testcases := []struct {
		total  int
		start  int64
		max    int
		length int
		end    bool
	}{
		{total: int(channelz.EntriesPerPage), start: 0, max: 0, length: channelz.EntriesPerPage, end: true},
		{total: int(channelz.EntriesPerPage) - 1, start: 0, max: 0, length: channelz.EntriesPerPage - 1, end: true},
		{total: int(channelz.EntriesPerPage) + 1, start: 0, max: 0, length: channelz.EntriesPerPage, end: false},
		{total: int(channelz.EntriesPerPage) + 1, start: int64(2*(channelz.EntriesPerPage+1) + 1), max: 0, length: 0, end: true},
		{total: int(channelz.EntriesPerPage), start: 0, max: 1, length: 1, end: false},
		{total: int(channelz.EntriesPerPage), start: 0, max: channelz.EntriesPerPage - 1, length: channelz.EntriesPerPage - 1, end: false},
	}

	for _, c := range testcases {
		// Reset channelz IDs so `start` is valid.
		channelz.IDGen.Reset()

		e := tcpClearRREnv
		te := newTest(t, e)
		var ccs []*grpc.ClientConn
		for i := 0; i < c.total; i++ {
			cc := te.clientConn()
			te.cc = nil
			// avoid making next dial blocking
			te.srvAddr = ""
			ccs = append(ccs, cc)
		}
		if err := verifyResultWithDelay(func() (bool, error) {
			if tcs, end := channelz.GetTopChannels(c.start, c.max); len(tcs) != c.length || end != c.end {
				return false, fmt.Errorf("getTopChannels(%d) = %+v (len of which: %d), end: %+v, want len(GetTopChannels(%d)) = %d, end: %+v", c.start, tcs, len(tcs), end, c.start, c.length, c.end)
			}
			return true, nil
		}); err != nil {
			t.Fatal(err)
		}

		for _, cc := range ccs {
			cc.Close()
		}

		if err := verifyResultWithDelay(func() (bool, error) {
			if tcs, end := channelz.GetTopChannels(c.start, c.max); len(tcs) != 0 || !end {
				return false, fmt.Errorf("getTopChannels(0) = %+v (len of which: %d), end: %+v, want len(GetTopChannels(0)) = 0, end: true", tcs, len(tcs), end)
			}
			return true, nil
		}); err != nil {
			t.Fatal(err)
		}
		te.tearDown()
	}
}

func (s) TestCZTopChannelRegistrationAndDeletionWhenDialFail(t *testing.T) {
	// Make dial fails (due to no transport security specified)
	_, err := grpc.NewClient("fake.addr")
	if err == nil {
		t.Fatal("expecting dial to fail")
	}
	if tcs, end := channelz.GetTopChannels(0, 0); tcs != nil || !end {
		t.Fatalf("GetTopChannels(0, 0) = %v, %v, want <nil>, true", tcs, end)
	}
}

func (s) TestCZNestedChannelRegistrationAndDeletion(t *testing.T) {
	e := tcpClearRREnv
	// avoid calling API to set balancer type, which will void service config's change of balancer.
	e.balancer = ""
	te := newTest(t, e)
	r := manual.NewBuilderWithScheme("whatever")
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	resolvedAddrs := []resolver.Address{{Addr: "127.0.0.1:0", ServerName: "grpclb.server"}}
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	r.UpdateState(grpclbstate.Set(resolver.State{ServiceConfig: grpclbConfig}, &grpclbstate.State{BalancerAddresses: resolvedAddrs}))
	defer te.tearDown()

	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		if nestedChans := tcs[0].NestedChans(); len(nestedChans) != 1 {
			return false, fmt.Errorf("there should be one nested channel from grpclb, not %d", len(nestedChans))
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: "127.0.0.1:0"}},
		ServiceConfig: parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`),
	})

	// wait for the shutdown of grpclb balancer
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		if nestedChans := tcs[0].NestedChans(); len(nestedChans) != 0 {
			return false, fmt.Errorf("there should be 0 nested channel from grpclb, not %d", len(nestedChans))
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZClientSubChannelSocketRegistrationAndDeletion(t *testing.T) {
	e := tcpClearRREnv
	num := 3 // number of backends
	te := newTest(t, e)
	var svrAddrs []resolver.Address
	te.startServers(&testServer{security: e.security}, num)
	r := manual.NewBuilderWithScheme("whatever")
	for _, a := range te.srvAddrs {
		svrAddrs = append(svrAddrs, resolver.Address{Addr: a})
	}
	r.InitialState(resolver.State{Addresses: svrAddrs})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != num {
			return false, fmt.Errorf("there should be %d subchannel not %d", num, len(subChans))
		}
		count := 0
		for k := range subChans {
			sc := channelz.GetSubChannel(k)
			if sc == nil {
				return false, fmt.Errorf("got <nil> subchannel")
			}
			count += len(sc.Sockets())
		}
		if count != num {
			return false, fmt.Errorf("there should be %d sockets not %d", num, count)
		}

		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{Addresses: svrAddrs[:len(svrAddrs)-1]})

	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != num-1 {
			return false, fmt.Errorf("there should be %d subchannel not %d", num-1, len(subChans))
		}
		count := 0
		for k := range subChans {
			sc := channelz.GetSubChannel(k)
			if sc == nil {
				return false, fmt.Errorf("got <nil> subchannel")
			}
			count += len(sc.Sockets())
		}
		if count != num-1 {
			return false, fmt.Errorf("there should be %d sockets not %d", num-1, count)
		}

		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZServerSocketRegistrationAndDeletion(t *testing.T) {
	testcases := []struct {
		total  int
		start  int64
		max    int
		length int
		end    bool
	}{
		{total: int(channelz.EntriesPerPage), start: 0, max: 0, length: channelz.EntriesPerPage, end: true},
		{total: int(channelz.EntriesPerPage) - 1, start: 0, max: 0, length: channelz.EntriesPerPage - 1, end: true},
		{total: int(channelz.EntriesPerPage) + 1, start: 0, max: 0, length: channelz.EntriesPerPage, end: false},
		{total: int(channelz.EntriesPerPage), start: 1, max: 0, length: channelz.EntriesPerPage - 1, end: true},
		{total: int(channelz.EntriesPerPage) + 1, start: int64(channelz.EntriesPerPage) + 1, max: 0, length: 0, end: true},
		{total: int(channelz.EntriesPerPage), start: 0, max: 1, length: 1, end: false},
		{total: int(channelz.EntriesPerPage), start: 0, max: channelz.EntriesPerPage - 1, length: channelz.EntriesPerPage - 1, end: false},
	}

	for _, c := range testcases {
		// Reset channelz IDs so `start` is valid.
		channelz.IDGen.Reset()

		e := tcpClearRREnv
		te := newTest(t, e)
		te.startServer(&testServer{security: e.security})
		var ccs []*grpc.ClientConn
		for i := 0; i < c.total; i++ {
			cc := te.clientConn()
			te.cc = nil
			ccs = append(ccs, cc)
		}

		var svrID int64
		if err := verifyResultWithDelay(func() (bool, error) {
			ss, _ := channelz.GetServers(0, 0)
			if len(ss) != 1 {
				return false, fmt.Errorf("there should only be one server, not %d", len(ss))
			}
			if got := len(ss[0].ListenSockets()); got != 1 {
				return false, fmt.Errorf("there should only be one server listen socket, not %d", got)
			}

			startID := c.start
			if startID != 0 {
				ns, _ := channelz.GetServerSockets(ss[0].ID, 0, c.total)
				if int64(len(ns)) < c.start {
					return false, fmt.Errorf("there should more than %d sockets, not %d", len(ns), c.start)
				}
				startID = ns[c.start-1].ID + 1
			}

			ns, end := channelz.GetServerSockets(ss[0].ID, startID, c.max)
			if len(ns) != c.length || end != c.end {
				return false, fmt.Errorf("GetServerSockets(%d) = %+v (len of which: %d), end: %+v, want len(GetServerSockets(%d)) = %d, end: %+v", c.start, ns, len(ns), end, c.start, c.length, c.end)
			}

			svrID = ss[0].ID
			return true, nil
		}); err != nil {
			t.Fatal(err)
		}

		for _, cc := range ccs {
			cc.Close()
		}

		if err := verifyResultWithDelay(func() (bool, error) {
			ns, _ := channelz.GetServerSockets(svrID, c.start, c.max)
			if len(ns) != 0 {
				return false, fmt.Errorf("there should be %d normal sockets not %d", 0, len(ns))
			}
			return true, nil
		}); err != nil {
			t.Fatal(err)
		}
		te.tearDown()
	}
}

func (s) TestCZServerListenSocketDeletion(t *testing.T) {
	s := grpc.NewServer()
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to listen: %v", err)
	}
	go s.Serve(lis)
	if err := verifyResultWithDelay(func() (bool, error) {
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		skts := ss[0].ListenSockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one server listen socket, not %v", skts)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	lis.Close()
	if err := verifyResultWithDelay(func() (bool, error) {
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should be 1 server, not %d", len(ss))
		}
		skts := ss[0].ListenSockets()
		if len(skts) != 0 {
			return false, fmt.Errorf("there should only be %d server listen socket, not %v", 0, skts)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
	s.Stop()
}

func (s) TestCZRecursiveDeletionOfEntry(t *testing.T) {
	//           +--+TopChan+---+
	//           |              |
	//           v              v
	//    +-+SubChan1+--+   SubChan2
	//    |             |
	//    v             v
	// Socket1       Socket2

	topChan := channelz.RegisterChannel(nil, "")
	subChan1 := channelz.RegisterSubChannel(topChan, "")
	subChan2 := channelz.RegisterSubChannel(topChan, "")
	skt1 := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: subChan1})
	skt2 := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: subChan1})

	tcs, _ := channelz.GetTopChannels(0, 0)
	if tcs == nil || len(tcs) != 1 {
		t.Fatalf("There should be one TopChannel entry")
	}
	if len(tcs[0].SubChans()) != 2 {
		t.Fatalf("There should be two SubChannel entries")
	}
	sc := channelz.GetSubChannel(subChan1.ID)
	if sc == nil || len(sc.Sockets()) != 2 {
		t.Fatalf("There should be two Socket entries")
	}

	channelz.RemoveEntry(topChan.ID)
	tcs, _ = channelz.GetTopChannels(0, 0)
	if tcs == nil || len(tcs) != 1 {
		t.Fatalf("There should be one TopChannel entry")
	}

	channelz.RemoveEntry(subChan1.ID)
	channelz.RemoveEntry(subChan2.ID)
	tcs, _ = channelz.GetTopChannels(0, 0)
	if tcs == nil || len(tcs) != 1 {
		t.Fatalf("There should be one TopChannel entry")
	}
	if len(tcs[0].SubChans()) != 1 {
		t.Fatalf("There should be one SubChannel entry")
	}

	channelz.RemoveEntry(skt1.ID)
	channelz.RemoveEntry(skt2.ID)
	tcs, _ = channelz.GetTopChannels(0, 0)
	if tcs != nil {
		t.Fatalf("There should be no TopChannel entry")
	}
}

func (s) TestCZChannelMetrics(t *testing.T) {
	e := tcpClearRREnv
	num := 3 // number of backends
	te := newTest(t, e)
	te.maxClientSendMsgSize = newInt(8)
	var svrAddrs []resolver.Address
	te.startServers(&testServer{security: e.security}, num)
	r := manual.NewBuilderWithScheme("whatever")
	for _, a := range te.srvAddrs {
		svrAddrs = append(svrAddrs, resolver.Address{Addr: a})
	}
	r.InitialState(resolver.State{Addresses: svrAddrs})
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}

	const smallSize = 1
	const largeSize = 8

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(smallSize),
		Payload:      largePayload,
	}

	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	defer stream.CloseSend()
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != num {
			return false, fmt.Errorf("there should be %d subchannel not %d", num, len(subChans))
		}
		var cst, csu, cf int64
		for k := range subChans {
			sc := channelz.GetSubChannel(k)
			if sc == nil {
				return false, fmt.Errorf("got <nil> subchannel")
			}
			cst += sc.ChannelMetrics.CallsStarted.Load()
			csu += sc.ChannelMetrics.CallsSucceeded.Load()
			cf += sc.ChannelMetrics.CallsFailed.Load()
		}
		if cst != 3 {
			return false, fmt.Errorf("there should be 3 CallsStarted not %d", cst)
		}
		if csu != 1 {
			return false, fmt.Errorf("there should be 1 CallsSucceeded not %d", csu)
		}
		if cf != 1 {
			return false, fmt.Errorf("there should be 1 CallsFailed not %d", cf)
		}
		if got := tcs[0].ChannelMetrics.CallsStarted.Load(); got != 3 {
			return false, fmt.Errorf("there should be 3 CallsStarted not %d", got)
		}
		if got := tcs[0].ChannelMetrics.CallsSucceeded.Load(); got != 1 {
			return false, fmt.Errorf("there should be 1 CallsSucceeded not %d", got)
		}
		if got := tcs[0].ChannelMetrics.CallsFailed.Load(); got != 1 {
			return false, fmt.Errorf("there should be 1 CallsFailed not %d", got)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZServerMetrics(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.maxServerReceiveMsgSize = newInt(8)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}

	const smallSize = 1
	const largeSize = 8

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(smallSize),
		Payload:      largePayload,
	}
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	defer stream.CloseSend()

	if err := verifyResultWithDelay(func() (bool, error) {
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		if cs := ss[0].ServerMetrics.CallsStarted.Load(); cs != 3 {
			return false, fmt.Errorf("there should be 3 CallsStarted not %d", cs)
		}
		if cs := ss[0].ServerMetrics.CallsSucceeded.Load(); cs != 1 {
			return false, fmt.Errorf("there should be 1 CallsSucceeded not %d", cs)
		}
		if cf := ss[0].ServerMetrics.CallsFailed.Load(); cf != 1 {
			return false, fmt.Errorf("there should be 1 CallsFailed not %d", cf)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

type testServiceClientWrapper struct {
	testgrpc.TestServiceClient
	mu             sync.RWMutex
	streamsCreated int
}

func (t *testServiceClientWrapper) getCurrentStreamID() uint32 {
	t.mu.RLock()
	defer t.mu.RUnlock()
	return uint32(2*t.streamsCreated - 1)
}

func (t *testServiceClientWrapper) EmptyCall(ctx context.Context, in *testpb.Empty, opts ...grpc.CallOption) (*testpb.Empty, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.EmptyCall(ctx, in, opts...)
}

func (t *testServiceClientWrapper) UnaryCall(ctx context.Context, in *testpb.SimpleRequest, opts ...grpc.CallOption) (*testpb.SimpleResponse, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.UnaryCall(ctx, in, opts...)
}

func (t *testServiceClientWrapper) StreamingOutputCall(ctx context.Context, in *testpb.StreamingOutputCallRequest, opts ...grpc.CallOption) (testgrpc.TestService_StreamingOutputCallClient, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.StreamingOutputCall(ctx, in, opts...)
}

func (t *testServiceClientWrapper) StreamingInputCall(ctx context.Context, opts ...grpc.CallOption) (testgrpc.TestService_StreamingInputCallClient, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.StreamingInputCall(ctx, opts...)
}

func (t *testServiceClientWrapper) FullDuplexCall(ctx context.Context, opts ...grpc.CallOption) (testgrpc.TestService_FullDuplexCallClient, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.FullDuplexCall(ctx, opts...)
}

func (t *testServiceClientWrapper) HalfDuplexCall(ctx context.Context, opts ...grpc.CallOption) (testgrpc.TestService_HalfDuplexCallClient, error) {
	t.mu.Lock()
	defer t.mu.Unlock()
	t.streamsCreated++
	return t.TestServiceClient.HalfDuplexCall(ctx, opts...)
}

func doSuccessfulUnaryCall(tc testgrpc.TestServiceClient, t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
}

func doStreamingInputCallWithLargePayload(tc testgrpc.TestServiceClient, t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s, err := tc.StreamingInputCall(ctx)
	if err != nil {
		t.Fatalf("TestService/StreamingInputCall(_) = _, %v, want <nil>", err)
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 10000)
	if err != nil {
		t.Fatal(err)
	}
	s.Send(&testpb.StreamingInputCallRequest{Payload: payload})
}

func doServerSideFailedUnaryCall(tc testgrpc.TestServiceClient, t *testing.T) {
	const smallSize = 1
	const largeSize = 2000

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(smallSize),
		Payload:      largePayload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}
}

func doClientSideInitiatedFailedStream(tc testgrpc.TestServiceClient, t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}

	const smallSize = 1
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}

	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{
			{Size: smallSize},
		},
		Payload: smallPayload,
	}

	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}
	// By canceling the call, the client will send rst_stream to end the call, and
	// the stream will failed as a result.
	cancel()
}

// This func is to be used to test client side counting of failed streams.
func doServerSideInitiatedFailedStreamWithRSTStream(tc testgrpc.TestServiceClient, t *testing.T, l *listenerWrapper) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}

	const smallSize = 1
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}

	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{
			{Size: smallSize},
		},
		Payload: smallPayload,
	}

	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}

	rcw := l.getLastConn()

	if rcw != nil {
		rcw.writeRSTStream(tc.(*testServiceClientWrapper).getCurrentStreamID(), http2.ErrCodeCancel)
	}
	if _, err := stream.Recv(); err == nil {
		t.Fatalf("%v.Recv() = %v, want <non-nil>", stream, err)
	}
}

// this func is to be used to test client side counting of failed streams.
func doServerSideInitiatedFailedStreamWithGoAway(ctx context.Context, tc testgrpc.TestServiceClient, t *testing.T, l *listenerWrapper) {
	// This call is just to keep the transport from shutting down (socket will be deleted
	// in this case, and we will not be able to get metrics).
	s, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}
	if err := s.Send(&testpb.StreamingOutputCallRequest{ResponseParameters: []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}}); err != nil {
		t.Fatalf("s.Send() failed with error: %v", err)
	}
	if _, err := s.Recv(); err != nil {
		t.Fatalf("s.Recv() failed with error: %v", err)
	}

	s, err = tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}
	if err := s.Send(&testpb.StreamingOutputCallRequest{ResponseParameters: []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}}); err != nil {
		t.Fatalf("s.Send() failed with error: %v", err)
	}
	if _, err := s.Recv(); err != nil {
		t.Fatalf("s.Recv() failed with error: %v", err)
	}

	rcw := l.getLastConn()
	if rcw != nil {
		rcw.writeGoAway(tc.(*testServiceClientWrapper).getCurrentStreamID()-2, http2.ErrCodeCancel, []byte{})
	}
	if _, err := s.Recv(); err == nil {
		t.Fatalf("%v.Recv() = %v, want <non-nil>", s, err)
	}
}

func (s) TestCZClientSocketMetricsStreamsAndMessagesCount(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	e := tcpClearRREnv
	te := newTest(t, e)
	te.maxServerReceiveMsgSize = newInt(20)
	te.maxClientReceiveMsgSize = newInt(20)
	rcw := te.startServerWithConnControl(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()
	tc := &testServiceClientWrapper{TestServiceClient: testgrpc.NewTestServiceClient(cc)}

	doSuccessfulUnaryCall(tc, t)
	var scID, skID int64
	if err := verifyResultWithDelay(func() (bool, error) {
		tchan, _ := channelz.GetTopChannels(0, 0)
		if len(tchan) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tchan))
		}
		subChans := tchan[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
		}

		for scID = range subChans {
			break
		}
		sc := channelz.GetSubChannel(scID)
		if sc == nil {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not 0", scID)
		}
		skts := sc.Sockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
		}
		for skID = range skts {
			break
		}
		skt := channelz.GetSocket(skID)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 1 || sktData.StreamsSucceeded.Load() != 1 || sktData.MessagesSent.Load() != 1 || sktData.MessagesReceived.Load() != 1 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (1, 1, 1, 1), got (%d, %d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doServerSideFailedUnaryCall(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(skID)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 2 || sktData.StreamsSucceeded.Load() != 2 || sktData.MessagesSent.Load() != 2 || sktData.MessagesReceived.Load() != 1 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (2, 2, 2, 1), got (%d, %d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doClientSideInitiatedFailedStream(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(skID)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 3 || sktData.StreamsSucceeded.Load() != 2 || sktData.StreamsFailed.Load() != 1 || sktData.MessagesSent.Load() != 3 || sktData.MessagesReceived.Load() != 2 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (3, 2, 1, 3, 2), got (%d, %d, %d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doServerSideInitiatedFailedStreamWithRSTStream(tc, t, rcw)
	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(skID)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 4 || sktData.StreamsSucceeded.Load() != 2 || sktData.StreamsFailed.Load() != 2 || sktData.MessagesSent.Load() != 4 || sktData.MessagesReceived.Load() != 3 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (4, 2, 2, 4, 3), got (%d, %d, %d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doServerSideInitiatedFailedStreamWithGoAway(ctx, tc, t, rcw)
	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(skID)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 6 || sktData.StreamsSucceeded.Load() != 2 || sktData.StreamsFailed.Load() != 3 || sktData.MessagesSent.Load() != 6 || sktData.MessagesReceived.Load() != 5 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (6, 2, 3, 6, 5), got (%d, %d, %d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

// This test is to complete TestCZClientSocketMetricsStreamsAndMessagesCount and
// TestCZServerSocketMetricsStreamsAndMessagesCount by adding the test case of
// server sending RST_STREAM to client due to client side flow control violation.
// It is separated from other cases due to setup incompatibly, i.e. max receive
// size violation will mask flow control violation.
func (s) TestCZClientAndServerSocketMetricsStreamsCountFlowControlRSTStream(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.serverInitialWindowSize = 65536
	// Avoid overflowing connection level flow control window, which will lead to
	// transport being closed.
	te.serverInitialConnWindowSize = 65536 * 2
	ts := &stubserver.StubServer{FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
		stream.Send(&testpb.StreamingOutputCallResponse{})
		<-stream.Context().Done()
		return status.Errorf(codes.DeadlineExceeded, "deadline exceeded or cancelled")
	}}
	te.startServer(ts)
	defer te.tearDown()
	cc, dw := te.clientConnWithConnControl()
	tc := &testServiceClientWrapper{TestServiceClient: testgrpc.NewTestServiceClient(cc)}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("stream.Recv() = %v, want nil", err)
	}
	go func() {
		payload := make([]byte, 16384)
		for i := 0; i < 6; i++ {
			dw.getRawConnWrapper().writeDataFrame(tc.getCurrentStreamID(), payload)
		}
	}()
	if _, err := stream.Recv(); status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("stream.Recv() = %v, want error code: %v", err, codes.ResourceExhausted)
	}
	cancel()

	if err := verifyResultWithDelay(func() (bool, error) {
		tchan, _ := channelz.GetTopChannels(0, 0)
		if len(tchan) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tchan))
		}
		subChans := tchan[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
		}
		var id int64
		for id = range subChans {
			break
		}
		sc := channelz.GetSubChannel(id)
		if sc == nil {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not 0", id)
		}
		skts := sc.Sockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
		}
		for id = range skts {
			break
		}
		skt := channelz.GetSocket(id)
		sktData := &skt.SocketMetrics
		if sktData.StreamsStarted.Load() != 1 || sktData.StreamsSucceeded.Load() != 0 || sktData.StreamsFailed.Load() != 1 {
			return false, fmt.Errorf("channelz.GetSocket(%d), want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load()) = (1, 0, 1), got (%d, %d, %d)", skt.ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load())
		}
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}

		ns, _ := channelz.GetServerSockets(ss[0].ID, 0, 0)
		if len(ns) != 1 {
			return false, fmt.Errorf("there should be one server normal socket, not %d", len(ns))
		}
		sktData = &ns[0].SocketMetrics
		if sktData.StreamsStarted.Load() != 1 || sktData.StreamsSucceeded.Load() != 0 || sktData.StreamsFailed.Load() != 1 {
			return false, fmt.Errorf("server socket metric with ID %d, want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load()) = (1, 0, 1), got (%d, %d, %d)", ns[0].ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZClientAndServerSocketMetricsFlowControl(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	// disable BDP
	te.serverInitialWindowSize = 65536
	te.serverInitialConnWindowSize = 65536
	te.clientInitialWindowSize = 65536
	te.clientInitialConnWindowSize = 65536
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	for i := 0; i < 10; i++ {
		doSuccessfulUnaryCall(tc, t)
	}

	var cliSktID, svrSktID int64
	if err := verifyResultWithDelay(func() (bool, error) {
		tchan, _ := channelz.GetTopChannels(0, 0)
		if len(tchan) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tchan))
		}
		subChans := tchan[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
		}
		var id int64
		for id = range subChans {
			break
		}
		sc := channelz.GetSubChannel(id)
		if sc == nil {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not 0", id)
		}
		skts := sc.Sockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
		}
		for id = range skts {
			break
		}
		skt := channelz.GetSocket(id)
		sktData := skt.EphemeralMetrics()
		// 65536 - 5 (Length-Prefixed-Message size) * 10 = 65486
		if sktData.LocalFlowControlWindow != 65486 || sktData.RemoteFlowControlWindow != 65486 {
			return false, fmt.Errorf("client: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (65536, 65486), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		ns, _ := channelz.GetServerSockets(ss[0].ID, 0, 0)
		sktData = ns[0].EphemeralMetrics()
		if sktData.LocalFlowControlWindow != 65486 || sktData.RemoteFlowControlWindow != 65486 {
			return false, fmt.Errorf("server: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (65536, 65486), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		cliSktID, svrSktID = id, ss[0].ID
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doStreamingInputCallWithLargePayload(tc, t)

	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(cliSktID)
		sktData := skt.EphemeralMetrics()
		// Local: 65536 - 5 (Length-Prefixed-Message size) * 10 = 65486
		// Remote: 65536 - 5 (Length-Prefixed-Message size) * 10 - 10011 = 55475
		if sktData.LocalFlowControlWindow != 65486 || sktData.RemoteFlowControlWindow != 55475 {
			return false, fmt.Errorf("client: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (65486, 55475), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		ns, _ := channelz.GetServerSockets(svrSktID, 0, 0)
		sktData = ns[0].EphemeralMetrics()
		if sktData.LocalFlowControlWindow != 55475 || sktData.RemoteFlowControlWindow != 65486 {
			return false, fmt.Errorf("server: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (55475, 65486), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	// triggers transport flow control window update on server side, since unacked
	// bytes should be larger than limit now. i.e. 50 + 20022 > 65536/4.
	doStreamingInputCallWithLargePayload(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		skt := channelz.GetSocket(cliSktID)
		sktData := skt.EphemeralMetrics()
		// Local: 65536 - 5 (Length-Prefixed-Message size) * 10 = 65486
		// Remote: 65536
		if sktData.LocalFlowControlWindow != 65486 || sktData.RemoteFlowControlWindow != 65536 {
			return false, fmt.Errorf("client: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (65486, 65536), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		ns, _ := channelz.GetServerSockets(svrSktID, 0, 0)
		sktData = ns[0].EphemeralMetrics()
		if sktData.LocalFlowControlWindow != 65536 || sktData.RemoteFlowControlWindow != 65486 {
			return false, fmt.Errorf("server: (LocalFlowControlWindow, RemoteFlowControlWindow) size should be (65536, 65486), not (%d, %d)", sktData.LocalFlowControlWindow, sktData.RemoteFlowControlWindow)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZClientSocketMetricsKeepAlive(t *testing.T) {
	const keepaliveRate = 50 * time.Millisecond
	defer func(t time.Duration) { internal.KeepaliveMinPingTime = t }(internal.KeepaliveMinPingTime)
	internal.KeepaliveMinPingTime = keepaliveRate
	e := tcpClearRREnv
	te := newTest(t, e)
	te.customDialOptions = append(te.customDialOptions, grpc.WithKeepaliveParams(
		keepalive.ClientParameters{
			Time:                keepaliveRate,
			Timeout:             500 * time.Millisecond,
			PermitWithoutStream: true,
		}))
	te.customServerOptions = append(te.customServerOptions, grpc.KeepaliveEnforcementPolicy(
		keepalive.EnforcementPolicy{
			MinTime:             keepaliveRate,
			PermitWithoutStream: true,
		}))
	te.startServer(&testServer{security: e.security})
	cc := te.clientConn() // Dial the server
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)
	start := time.Now()
	// Wait for at least two keepalives to be able to occur.
	time.Sleep(2 * keepaliveRate)
	defer te.tearDown()
	if err := verifyResultWithDelay(func() (bool, error) {
		tchan, _ := channelz.GetTopChannels(0, 0)
		if len(tchan) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tchan))
		}
		subChans := tchan[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
		}
		var id int64
		for id = range subChans {
			break
		}
		sc := channelz.GetSubChannel(id)
		if sc == nil {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not 0", id)
		}
		skts := sc.Sockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
		}
		for id = range skts {
			break
		}
		skt := channelz.GetSocket(id)
		want := int64(time.Since(start) / keepaliveRate)
		if got := skt.SocketMetrics.KeepAlivesSent.Load(); got != want {
			return false, fmt.Errorf("there should be %v KeepAlives sent, not %d", want, got)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZServerSocketMetricsStreamsAndMessagesCount(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.maxServerReceiveMsgSize = newInt(20)
	te.maxClientReceiveMsgSize = newInt(20)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc, _ := te.clientConnWithConnControl()
	tc := &testServiceClientWrapper{TestServiceClient: testgrpc.NewTestServiceClient(cc)}

	var svrID int64
	if err := verifyResultWithDelay(func() (bool, error) {
		ss, _ := channelz.GetServers(0, 0)
		if len(ss) != 1 {
			return false, fmt.Errorf("there should only be one server, not %d", len(ss))
		}
		svrID = ss[0].ID
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doSuccessfulUnaryCall(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		ns, _ := channelz.GetServerSockets(svrID, 0, 0)
		sktData := &ns[0].SocketMetrics
		if sktData.StreamsStarted.Load() != 1 || sktData.StreamsSucceeded.Load() != 1 || sktData.StreamsFailed.Load() != 0 || sktData.MessagesSent.Load() != 1 || sktData.MessagesReceived.Load() != 1 {
			return false, fmt.Errorf("server socket metric with ID %d, want (StreamsStarted.Load(), StreamsSucceeded.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (1, 1, 1, 1), got (%d, %d, %d, %d, %d)", ns[0].ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doServerSideFailedUnaryCall(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		ns, _ := channelz.GetServerSockets(svrID, 0, 0)
		sktData := &ns[0].SocketMetrics
		if sktData.StreamsStarted.Load() != 2 || sktData.StreamsSucceeded.Load() != 2 || sktData.StreamsFailed.Load() != 0 || sktData.MessagesSent.Load() != 1 || sktData.MessagesReceived.Load() != 1 {
			return false, fmt.Errorf("server socket metric with ID %d, want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (2, 2, 0, 1, 1), got (%d, %d, %d, %d, %d)", ns[0].ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	doClientSideInitiatedFailedStream(tc, t)
	if err := verifyResultWithDelay(func() (bool, error) {
		ns, _ := channelz.GetServerSockets(svrID, 0, 0)
		sktData := &ns[0].SocketMetrics
		if sktData.StreamsStarted.Load() != 3 || sktData.StreamsSucceeded.Load() != 2 || sktData.StreamsFailed.Load() != 1 || sktData.MessagesSent.Load() != 2 || sktData.MessagesReceived.Load() != 2 {
			return false, fmt.Errorf("server socket metric with ID %d, want (StreamsStarted.Load(), StreamsSucceeded.Load(), StreamsFailed.Load(), MessagesSent.Load(), MessagesReceived.Load()) = (3, 2, 1, 2, 2), got (%d, %d, %d, %d, %d)", ns[0].ID, sktData.StreamsStarted.Load(), sktData.StreamsSucceeded.Load(), sktData.StreamsFailed.Load(), sktData.MessagesSent.Load(), sktData.MessagesReceived.Load())
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZServerSocketMetricsKeepAlive(t *testing.T) {
	defer func(t time.Duration) { internal.KeepaliveMinServerPingTime = t }(internal.KeepaliveMinServerPingTime)
	internal.KeepaliveMinServerPingTime = 50 * time.Millisecond

	e := tcpClearRREnv
	te := newTest(t, e)
	// We setup the server keepalive parameters to send one keepalive every
	// 50ms, and verify that the actual number of keepalives is very close to
	// Time/50ms.  We had a bug wherein the server was sending one keepalive
	// every [Time+Timeout] instead of every [Time] period, and since Timeout
	// is configured to a high value here, we should be able to verify that the
	// fix works with the above mentioned logic.
	kpOption := grpc.KeepaliveParams(keepalive.ServerParameters{
		Time:    50 * time.Millisecond,
		Timeout: 5 * time.Second,
	})
	te.customServerOptions = append(te.customServerOptions, kpOption)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)

	// Allow about 5 pings to happen (250ms/50ms).
	time.Sleep(255 * time.Millisecond)

	ss, _ := channelz.GetServers(0, 0)
	if len(ss) != 1 {
		t.Fatalf("there should be one server, not %d", len(ss))
	}
	ns, _ := channelz.GetServerSockets(ss[0].ID, 0, 0)
	if len(ns) != 1 {
		t.Fatalf("there should be one server normal socket, not %d", len(ns))
	}
	const wantMin, wantMax = 3, 7
	if got := ns[0].SocketMetrics.KeepAlivesSent.Load(); got < wantMin || got > wantMax {
		t.Fatalf("got keepalivesCount: %v, want keepalivesCount: [%v,%v]", got, wantMin, wantMax)
	}
}

var cipherSuites = []string{
	"TLS_RSA_WITH_RC4_128_SHA",
	"TLS_RSA_WITH_3DES_EDE_CBC_SHA",
	"TLS_RSA_WITH_AES_128_CBC_SHA",
	"TLS_RSA_WITH_AES_256_CBC_SHA",
	"TLS_RSA_WITH_AES_128_GCM_SHA256",
	"TLS_RSA_WITH_AES_256_GCM_SHA384",
	"TLS_ECDHE_ECDSA_WITH_RC4_128_SHA",
	"TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA",
	"TLS_ECDHE_ECDSA_WITH_AES_256_CBC_SHA",
	"TLS_ECDHE_RSA_WITH_RC4_128_SHA",
	"TLS_ECDHE_RSA_WITH_3DES_EDE_CBC_SHA",
	"TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
	"TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
	"TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
	"TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
	"TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
	"TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
	"TLS_FALLBACK_SCSV",
	"TLS_RSA_WITH_AES_128_CBC_SHA256",
	"TLS_ECDHE_ECDSA_WITH_AES_128_CBC_SHA256",
	"TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA256",
	"TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305",
	"TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305",
	"TLS_AES_128_GCM_SHA256",
	"TLS_AES_256_GCM_SHA384",
	"TLS_CHACHA20_POLY1305_SHA256",
}

func (s) TestCZSocketGetSecurityValueTLS(t *testing.T) {
	e := tcpTLSRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	te.clientConn()
	if err := verifyResultWithDelay(func() (bool, error) {
		tchan, _ := channelz.GetTopChannels(0, 0)
		if len(tchan) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tchan))
		}
		subChans := tchan[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
		}
		var id int64
		for id = range subChans {
			break
		}
		sc := channelz.GetSubChannel(id)
		if sc == nil {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not 0", id)
		}
		skts := sc.Sockets()
		if len(skts) != 1 {
			return false, fmt.Errorf("there should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
		}
		for id = range skts {
			break
		}
		skt := channelz.GetSocket(id)
		cert, _ := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
		securityVal, ok := skt.Security.(*credentials.TLSChannelzSecurityValue)
		if !ok {
			return false, fmt.Errorf("the Security is of type: %T, want: *credentials.TLSChannelzSecurityValue", skt.Security)
		}
		if !cmp.Equal(securityVal.RemoteCertificate, cert.Certificate[0]) {
			return false, fmt.Errorf("Security.RemoteCertificate got: %v, want: %v", securityVal.RemoteCertificate, cert.Certificate[0])
		}
		for _, v := range cipherSuites {
			if v == securityVal.StandardName {
				return true, nil
			}
		}
		return false, fmt.Errorf("Security.StandardName got: %v, want it to be one of %v", securityVal.StandardName, cipherSuites)
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZChannelTraceCreationDeletion(t *testing.T) {
	e := tcpClearRREnv
	// avoid calling API to set balancer type, which will void service config's change of balancer.
	e.balancer = ""
	te := newTest(t, e)
	r := manual.NewBuilderWithScheme("whatever")
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	resolvedAddrs := []resolver.Address{{Addr: "127.0.0.1:0", ServerName: "grpclb.server"}}
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	r.UpdateState(grpclbstate.Set(resolver.State{ServiceConfig: grpclbConfig}, &grpclbstate.State{BalancerAddresses: resolvedAddrs}))
	defer te.tearDown()

	var nestedConn int64
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		nestedChans := tcs[0].NestedChans()
		if len(nestedChans) != 1 {
			return false, fmt.Errorf("there should be one nested channel from grpclb, not %d", len(nestedChans))
		}
		for k := range nestedChans {
			nestedConn = k
		}
		trace := tcs[0].Trace()
		for _, e := range trace.Events {
			if e.RefID == nestedConn && e.RefType != channelz.RefChannel {
				return false, fmt.Errorf("nested channel trace event should have RefChannel as RefType")
			}
		}
		ncm := channelz.GetChannel(nestedConn)
		ncmTrace := ncm.Trace()
		if ncmTrace == nil {
			return false, fmt.Errorf("trace for nested channel should not be empty")
		}
		if len(ncmTrace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for nested channel not 0")
		}
		pattern := `Channel created`
		if ok, _ := regexp.MatchString(pattern, ncmTrace.Events[0].Desc); !ok {
			return false, fmt.Errorf("the first trace event should be %q, not %q", pattern, ncmTrace.Events[0].Desc)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: "127.0.0.1:0"}},
		ServiceConfig: parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`),
	})

	// wait for the shutdown of grpclb balancer
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		nestedChans := tcs[0].NestedChans()
		if len(nestedChans) != 0 {
			return false, fmt.Errorf("there should be 0 nested channel from grpclb, not %d", len(nestedChans))
		}
		ncm := channelz.GetChannel(nestedConn)
		if ncm == nil {
			return false, fmt.Errorf("nested channel should still exist due to parent's trace reference")
		}
		trace := ncm.Trace()
		if trace == nil {
			return false, fmt.Errorf("trace for nested channel should not be empty")
		}
		if len(trace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for nested channel not 0")
		}
		pattern := `Channel created`
		if ok, _ := regexp.MatchString(pattern, trace.Events[0].Desc); !ok {
			return false, fmt.Errorf("the first trace event should be %q, not %q", pattern, trace.Events[0].Desc)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZSubChannelTraceCreationDeletion(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: te.srvAddr}}})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	var subConn int64
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
		}
		for k := range subChans {
			subConn = k
		}
		trace := tcs[0].Trace()
		for _, e := range trace.Events {
			if e.RefID == subConn && e.RefType != channelz.RefSubChannel {
				return false, fmt.Errorf("subchannel trace event should have RefType to be RefSubChannel")
			}
		}
		scm := channelz.GetSubChannel(subConn)
		if scm == nil {
			return false, fmt.Errorf("subChannel does not exist")
		}
		scTrace := scm.Trace()
		if scTrace == nil {
			return false, fmt.Errorf("trace for subChannel should not be empty")
		}
		if len(scTrace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for subChannel not 0")
		}
		pattern := `Subchannel created`
		if ok, _ := regexp.MatchString(pattern, scTrace.Events[0].Desc); !ok {
			return false, fmt.Errorf("the first trace event should be %q, not %q", pattern, scTrace.Events[0].Desc)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, te.cc, connectivity.Ready)
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "fake address"}}})
	testutils.AwaitNotState(ctx, t, te.cc, connectivity.Ready)

	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
		}
		scm := channelz.GetSubChannel(subConn)
		if scm == nil {
			return false, fmt.Errorf("subChannel should still exist due to parent's trace reference")
		}
		trace := scm.Trace()
		if trace == nil {
			return false, fmt.Errorf("trace for SubChannel should not be empty")
		}
		if len(trace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for subChannel not 0")
		}

		pattern := `Subchannel deleted`
		desc := trace.Events[len(trace.Events)-1].Desc
		if ok, _ := regexp.MatchString(pattern, desc); !ok {
			return false, fmt.Errorf("the last trace event should be %q, not %q", pattern, desc)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZChannelAddressResolutionChange(t *testing.T) {
	e := tcpClearRREnv
	e.balancer = ""
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.InitialState(resolver.State{Addresses: addrs})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	var cid int64
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		cid = tcs[0].ID
		trace := tcs[0].Trace()
		for i := len(trace.Events) - 1; i >= 0; i-- {
			if strings.Contains(trace.Events[i].Desc, "resolver returned new addresses") {
				break
			}
			if i == 0 {
				return false, fmt.Errorf("events do not contain expected address resolution from empty address state.  Got: %+v", trace.Events)
			}
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
	r.UpdateState(resolver.State{
		Addresses:     addrs,
		ServiceConfig: parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`),
	})

	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(cid)
		trace := cm.Trace()
		for i := len(trace.Events) - 1; i >= 0; i-- {
			if strings.Contains(trace.Events[i].Desc, fmt.Sprintf("Channel switches to new LB policy %q", roundrobin.Name)) {
				break
			}
			if i == 0 {
				return false, fmt.Errorf("events do not contain expected address resolution change of LB policy")
			}
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	newSC := parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                }
            ],
            "waitForReady": false,
            "timeout": ".001s"
        }
    ]
}`)
	r.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: newSC})

	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(cid)

		var es []string
		trace := cm.Trace()
		for i := len(trace.Events) - 1; i >= 0; i-- {
			if strings.Contains(trace.Events[i].Desc, "service config updated") {
				break
			}
			es = append(es, trace.Events[i].Desc)
			if i == 0 {
				return false, fmt.Errorf("events do not contain expected address resolution of new service config\n Events:\n%v", strings.Join(es, "\n"))
			}
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{Addresses: []resolver.Address{}, ServiceConfig: newSC})

	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(cid)
		trace := cm.Trace()
		for i := len(trace.Events) - 1; i >= 0; i-- {
			if strings.Contains(trace.Events[i].Desc, "resolver returned an empty address list") {
				break
			}
			if i == 0 {
				return false, fmt.Errorf("events do not contain expected address resolution of empty address")
			}
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZSubChannelPickedNewAddress(t *testing.T) {
	e := tcpClearRREnv
	e.balancer = ""
	te := newTest(t, e)
	te.startServers(&testServer{security: e.security}, 3)
	r := manual.NewBuilderWithScheme("whatever")
	var svrAddrs []resolver.Address
	for _, a := range te.srvAddrs {
		svrAddrs = append(svrAddrs, resolver.Address{Addr: a})
	}
	r.InitialState(resolver.State{Addresses: svrAddrs})
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(cc)
	// make sure the connection is up
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	te.srvs[0].Stop()
	te.srvs[1].Stop()
	// Here, we just wait for all sockets to be up. Make several rpc calls to
	// create the sockets since we do not automatically reconnect.
	done := make(chan struct{})
	defer close(done)
	go func() {
		for {
			tc.EmptyCall(ctx, &testpb.Empty{})
			select {
			case <-time.After(10 * time.Millisecond):
			case <-done:
				return
			}
		}
	}()
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
		}
		var subConn int64
		for k := range subChans {
			subConn = k
		}
		scm := channelz.GetSubChannel(subConn)
		trace := scm.Trace()
		if trace == nil {
			return false, fmt.Errorf("trace for SubChannel should not be empty")
		}
		if len(trace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for subChannel not 0")
		}
		for i := len(trace.Events) - 1; i >= 0; i-- {
			if strings.Contains(trace.Events[i].Desc, fmt.Sprintf("Subchannel picks a new address %q to connect", te.srvAddrs[2])) {
				break
			}
			if i == 0 {
				return false, fmt.Errorf("events do not contain expected address resolution of subchannel picked new address")
			}
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZSubChannelConnectivityState(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: te.srvAddr}}})
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(cc)
	// make sure the connection is up
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	te.srv.Stop()

	var subConn int64
	if err := verifyResultWithDelay(func() (bool, error) {
		// we need to obtain the SubChannel id before it gets deleted from Channel's children list (due
		// to effect of r.UpdateState(resolver.State{Addresses:[]resolver.Address{}}))
		if subConn == 0 {
			tcs, _ := channelz.GetTopChannels(0, 0)
			if len(tcs) != 1 {
				return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
			}
			subChans := tcs[0].SubChans()
			if len(subChans) != 1 {
				return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
			}
			for k := range subChans {
				// get the SubChannel id for further trace inquiry.
				subConn = k
				t.Logf("SubChannel Id is %d", subConn)
			}
		}
		scm := channelz.GetSubChannel(subConn)
		if scm == nil {
			return false, fmt.Errorf("subChannel should still exist due to parent's trace reference")
		}
		trace := scm.Trace()
		if trace == nil {
			return false, fmt.Errorf("trace for SubChannel should not be empty")
		}
		if len(trace.Events) == 0 {
			return false, fmt.Errorf("there should be at least one trace event for subChannel not 0")
		}
		var ready, connecting, transient, shutdown int
		t.Log("SubChannel trace events seen so far...")
		for _, e := range trace.Events {
			t.Log(e.Desc)
			if strings.Contains(e.Desc, fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.TransientFailure)) {
				transient++
			}
		}
		// Make sure the SubChannel has already seen transient failure before shutting it down through
		// r.UpdateState(resolver.State{Addresses:[]resolver.Address{}}).
		if transient == 0 {
			return false, fmt.Errorf("transient failure has not happened on SubChannel yet")
		}
		transient = 0
		r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "fake address"}}})
		t.Log("SubChannel trace events seen so far...")
		for _, e := range trace.Events {
			t.Log(e.Desc)
			if strings.Contains(e.Desc, fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.Ready)) {
				ready++
			}
			if strings.Contains(e.Desc, fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.Connecting)) {
				connecting++
			}
			if strings.Contains(e.Desc, fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.TransientFailure)) {
				transient++
			}
			if strings.Contains(e.Desc, fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.Shutdown)) {
				shutdown++
			}
		}
		// example:
		// Subchannel Created
		// Subchannel's connectivity state changed to CONNECTING
		// Subchannel picked a new address: "localhost:36011"
		// Subchannel's connectivity state changed to READY
		// Subchannel's connectivity state changed to TRANSIENT_FAILURE
		// Subchannel's connectivity state changed to CONNECTING
		// Subchannel picked a new address: "localhost:36011"
		// Subchannel's connectivity state changed to SHUTDOWN
		// Subchannel Deleted
		if ready != 1 || connecting < 1 || transient < 1 || shutdown != 1 {
			return false, fmt.Errorf("got: ready = %d, connecting = %d, transient = %d, shutdown = %d, want: 1, >=1, >=1, 1", ready, connecting, transient, shutdown)
		}

		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZChannelConnectivityState(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: te.srvAddr}}})
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(cc)
	// make sure the connection is up
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	te.srv.Stop()

	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}

		var ready, connecting, transient int
		t.Log("Channel trace events seen so far...")
		for _, e := range tcs[0].Trace().Events {
			t.Log(e.Desc)
			if strings.Contains(e.Desc, fmt.Sprintf("Channel Connectivity change to %v", connectivity.Ready)) {
				ready++
			}
			if strings.Contains(e.Desc, fmt.Sprintf("Channel Connectivity change to %v", connectivity.Connecting)) {
				connecting++
			}
			if strings.Contains(e.Desc, fmt.Sprintf("Channel Connectivity change to %v", connectivity.TransientFailure)) {
				transient++
			}
		}

		// example:
		// Channel Created
		// Addresses resolved (from empty address state): "localhost:40467"
		// SubChannel (id: 4[]) Created
		// Channel's connectivity state changed to CONNECTING
		// Channel's connectivity state changed to READY
		// Channel's connectivity state changed to TRANSIENT_FAILURE
		// Channel's connectivity state changed to CONNECTING
		// Channel's connectivity state changed to TRANSIENT_FAILURE
		if ready != 1 || connecting < 1 || transient < 1 {
			return false, fmt.Errorf("got: ready = %d, connecting = %d, transient = %d, want: 1, >=1, >=1", ready, connecting, transient)
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZTraceOverwriteChannelDeletion(t *testing.T) {
	e := tcpClearRREnv
	e.balancer = ""
	te := newTest(t, e)
	channelz.SetMaxTraceEntry(1)
	defer channelz.ResetMaxTraceEntryToDefault()
	r := manual.NewBuilderWithScheme("whatever")
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	resolvedAddrs := []resolver.Address{{Addr: "127.0.0.1:0", ServerName: "grpclb.server"}}
	grpclbConfig := parseServiceConfig(t, r, `{"loadBalancingPolicy": "grpclb"}`)
	r.UpdateState(grpclbstate.Set(resolver.State{ServiceConfig: grpclbConfig}, &grpclbstate.State{BalancerAddresses: resolvedAddrs}))
	defer te.tearDown()
	var nestedConn int64
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		nestedChans := tcs[0].NestedChans()
		if len(nestedChans) != 1 {
			return false, fmt.Errorf("there should be one nested channel from grpclb, not %d", len(nestedChans))
		}
		for k := range nestedChans {
			nestedConn = k
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: "127.0.0.1:0"}},
		ServiceConfig: parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`),
	})

	// wait for the shutdown of grpclb balancer
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}

		if nestedChans := tcs[0].NestedChans(); len(nestedChans) != 0 {
			return false, fmt.Errorf("there should be 0 nested channel from grpclb, not %d", len(nestedChans))
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	// If nested channel deletion is last trace event before the next validation, it will fail, as the top channel will hold a reference to it.
	// This line forces a trace event on the top channel in that case.
	r.UpdateState(resolver.State{
		Addresses:     []resolver.Address{{Addr: "127.0.0.1:0"}},
		ServiceConfig: parseServiceConfig(t, r, `{"loadBalancingPolicy": "round_robin"}`),
	})

	// verify that the nested channel no longer exist due to trace referencing it got overwritten.
	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(nestedConn)
		if cm != nil {
			return false, fmt.Errorf("nested channel should have been deleted since its parent's trace should not contain any reference to it anymore")
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZTraceOverwriteSubChannelDeletion(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	channelz.SetMaxTraceEntry(1)
	defer channelz.ResetMaxTraceEntryToDefault()
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: te.srvAddr}}})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	defer te.tearDown()
	var subConn int64
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
		}
		for k := range subChans {
			subConn = k
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, te.cc, connectivity.Ready)
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: "fake address"}}})
	testutils.AwaitNotState(ctx, t, te.cc, connectivity.Ready)

	// verify that the subchannel no longer exist due to trace referencing it got overwritten.
	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(subConn)
		if cm != nil {
			return false, fmt.Errorf("subchannel should have been deleted since its parent's trace should not contain any reference to it anymore")
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestCZTraceTopChannelDeletionTraceClear(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: te.srvAddr}}})
	te.resolverScheme = r.Scheme()
	te.clientConn(grpc.WithResolvers(r))
	var subConn int64
	// Here, we just wait for all sockets to be up. In the future, if we implement
	// IDLE, we may need to make several rpc calls to create the sockets.
	if err := verifyResultWithDelay(func() (bool, error) {
		tcs, _ := channelz.GetTopChannels(0, 0)
		if len(tcs) != 1 {
			return false, fmt.Errorf("there should only be one top channel, not %d", len(tcs))
		}
		subChans := tcs[0].SubChans()
		if len(subChans) != 1 {
			return false, fmt.Errorf("there should be 1 subchannel not %d", len(subChans))
		}
		for k := range subChans {
			subConn = k
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
	te.tearDown()
	// verify that the subchannel no longer exist due to parent channel got deleted and its trace cleared.
	if err := verifyResultWithDelay(func() (bool, error) {
		cm := channelz.GetChannel(subConn)
		if cm != nil {
			return false, fmt.Errorf("subchannel should have been deleted since its parent's trace should not contain any reference to it anymore")
		}
		return true, nil
	}); err != nil {
		t.Fatal(err)
	}
}
