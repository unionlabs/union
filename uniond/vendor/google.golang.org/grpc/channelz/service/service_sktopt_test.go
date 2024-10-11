//go:build linux && (386 || amd64)
// +build linux
// +build 386 amd64

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

// SocketOptions is only supported on linux system. The functions defined in
// this file are to parse the socket option field and the test is specifically
// to verify the behavior of socket option parsing.

package service

import (
	"context"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"golang.org/x/sys/unix"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/durationpb"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func (s) TestGetSocketOptions(t *testing.T) {
	ss := &channelz.Socket{
		SocketOptions: &channelz.SocketOptionData{
			Linger:      &unix.Linger{Onoff: 1, Linger: 2},
			RecvTimeout: &unix.Timeval{Sec: 10, Usec: 1},
			SendTimeout: &unix.Timeval{},
			TCPInfo:     &unix.TCPInfo{State: 1},
		},
	}
	svr := newCZServer()
	czServer := channelz.RegisterServer("test svr")
	defer channelz.RemoveEntry(czServer.ID)
	id := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, RefName: "0", Parent: czServer, SocketOptions: ss.SocketOptions})
	defer channelz.RemoveEntry(id.ID)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := svr.GetSocket(ctx, &channelzpb.GetSocketRequest{SocketId: id.ID})
	{
		got, want := resp.GetSocket().GetRef(), &channelzpb.SocketRef{SocketId: id.ID, Name: "0"}
		if diff := cmp.Diff(got, want, protocmp.Transform()); diff != "" {
			t.Fatal("resp.GetSocket() ref (-got +want): ", diff)
		}
	}
	{
		got := resp.GetSocket().GetData().GetOption()
		want := []*channelzpb.SocketOption{{
			Name: "SO_LINGER",
			Additional: testutils.MarshalAny(
				t,
				&channelzpb.SocketOptionLinger{Active: true, Duration: durationpb.New(2 * time.Second)},
			),
		}, {
			Name: "SO_RCVTIMEO",
			Additional: testutils.MarshalAny(
				t,
				&channelzpb.SocketOptionTimeout{Duration: durationpb.New(10*time.Second + time.Microsecond)},
			),
		}, {
			Name: "SO_SNDTIMEO",
			Additional: testutils.MarshalAny(
				t,
				&channelzpb.SocketOptionTimeout{Duration: durationpb.New(0)},
			),
		}, {
			Name: "TCP_INFO",
			Additional: testutils.MarshalAny(
				t,
				&channelzpb.SocketOptionTcpInfo{TcpiState: 1},
			),
		}}
		if diff := cmp.Diff(got, want, protocmp.Transform()); diff != "" {
			t.Fatal("resp.GetSocket() options (-got +want): ", diff)
		}
	}
}
