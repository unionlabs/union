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
	"testing"
	"time"

	"google.golang.org/grpc/internal/channelz"

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

func (s) TestCZSocketMetricsSocketOption(t *testing.T) {
	envs := []env{tcpClearRREnv, tcpTLSRREnv}
	for _, e := range envs {
		testCZSocketMetricsSocketOption(t, e)
	}
}

func testCZSocketMetricsSocketOption(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	doSuccessfulUnaryCall(tc, t)

	time.Sleep(10 * time.Millisecond)
	ss, _ := channelz.GetServers(0, 0)
	if len(ss) != 1 {
		t.Fatalf("There should be one server, not %d", len(ss))
	}
	skts := ss[0].ListenSockets()
	if len(skts) != 1 {
		t.Fatalf("There should be one listen socket, not %d", len(skts))
	}
	for id := range skts {
		sm := channelz.GetSocket(id)
		if sm == nil || sm.SocketOptions == nil {
			t.Fatalf("Unable to get server listen socket options")
		}
	}
	ns, _ := channelz.GetServerSockets(ss[0].ID, 0, 0)
	if len(ns) != 1 {
		t.Fatalf("There should be one server normal socket, not %d", len(ns))
	}
	if ns[0] == nil || ns[0].SocketOptions == nil {
		t.Fatalf("Unable to get server normal socket options")
	}

	tchan, _ := channelz.GetTopChannels(0, 0)
	if len(tchan) != 1 {
		t.Fatalf("There should only be one top channel, not %d", len(tchan))
	}
	subChans := tchan[0].SubChans()
	if len(subChans) != 1 {
		t.Fatalf("There should only be one subchannel under top channel %d, not %d", tchan[0].ID, len(subChans))
	}
	var id int64
	for id = range subChans {
		break
	}
	sc := channelz.GetSubChannel(id)
	if sc == nil {
		t.Fatalf("There should only be one socket under subchannel %d, not 0", id)
	}
	skts = sc.Sockets()
	if len(skts) != 1 {
		t.Fatalf("There should only be one socket under subchannel %d, not %d", sc.ID, len(skts))
	}
	for id = range skts {
		break
	}
	skt := channelz.GetSocket(id)
	if skt == nil || skt.SocketOptions == nil {
		t.Fatalf("Unable to get client normal socket options")
	}
}
