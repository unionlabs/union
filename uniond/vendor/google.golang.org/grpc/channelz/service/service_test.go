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

package service

import (
	"context"
	"fmt"
	"net"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/protobuf/encoding/prototext"
	"google.golang.org/protobuf/reflect/protodesc"
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/timestamppb"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func init() {
	channelz.TurnOn()
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const defaultTestTimeout = 10 * time.Second

func channelProtoToStruct(c *channelzpb.Channel) (*channelz.ChannelMetrics, error) {
	cm := &channelz.ChannelMetrics{}
	pdata := c.GetData()
	var s connectivity.State
	switch pdata.GetState().GetState() {
	case channelzpb.ChannelConnectivityState_UNKNOWN:
		// TODO: what should we set here?
	case channelzpb.ChannelConnectivityState_IDLE:
		s = connectivity.Idle
	case channelzpb.ChannelConnectivityState_CONNECTING:
		s = connectivity.Connecting
	case channelzpb.ChannelConnectivityState_READY:
		s = connectivity.Ready
	case channelzpb.ChannelConnectivityState_TRANSIENT_FAILURE:
		s = connectivity.TransientFailure
	case channelzpb.ChannelConnectivityState_SHUTDOWN:
		s = connectivity.Shutdown
	}
	cm.State.Store(&s)
	tgt := pdata.GetTarget()
	cm.Target.Store(&tgt)
	cm.CallsStarted.Store(pdata.CallsStarted)
	cm.CallsSucceeded.Store(pdata.CallsSucceeded)
	cm.CallsFailed.Store(pdata.CallsFailed)
	if err := pdata.GetLastCallStartedTimestamp().CheckValid(); err != nil {
		return nil, err
	}
	cm.LastCallStartedTimestamp.Store(int64(pdata.GetLastCallStartedTimestamp().AsTime().UnixNano()))
	return cm, nil
}

func convertSocketRefSliceToMap(sktRefs []*channelzpb.SocketRef) map[int64]string {
	m := make(map[int64]string)
	for _, sr := range sktRefs {
		m[sr.SocketId] = sr.Name
	}
	return m
}

func (s) TestGetTopChannels(t *testing.T) {
	tcs := []*channelz.ChannelMetrics{
		channelz.NewChannelMetricForTesting(
			connectivity.Connecting,
			"test.channelz:1234",
			6,
			2,
			3,
			time.Now().UTC().UnixNano(),
		),
		channelz.NewChannelMetricForTesting(
			connectivity.Connecting,
			"test.channelz:1234",
			1,
			2,
			3,
			time.Now().UTC().UnixNano(),
		),
		channelz.NewChannelMetricForTesting(
			connectivity.Shutdown,
			"test.channelz:8888",
			0,
			0,
			0,
			0,
		),
	}

	for _, c := range tcs {
		cz := channelz.RegisterChannel(nil, "test channel")
		cz.ChannelMetrics.CopyFrom(c)
		defer channelz.RemoveEntry(cz.ID)
	}
	s := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := s.GetTopChannels(ctx, &channelzpb.GetTopChannelsRequest{StartChannelId: 0})
	if !resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want true, got %v", resp.GetEnd())
	}
	for i, c := range resp.GetChannel() {
		channel, err := channelProtoToStruct(c)
		if err != nil {
			t.Fatal(err)
		}
		if diff := cmp.Diff(tcs[i], channel, protocmp.Transform()); diff != "" {
			t.Fatalf("unexpected channel, diff (-want +got):\n%s", diff)
		}
	}
	for i := 0; i < 50; i++ {
		cz := channelz.RegisterChannel(nil, "")
		defer channelz.RemoveEntry(cz.ID)
	}
	resp, _ = s.GetTopChannels(ctx, &channelzpb.GetTopChannelsRequest{StartChannelId: 0})
	if resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want false, got %v", resp.GetEnd())
	}
}

func (s) TestGetServers(t *testing.T) {
	ss := []*channelz.ServerMetrics{
		channelz.NewServerMetricsForTesting(
			6,
			2,
			3,
			time.Now().UnixNano(),
		),
		channelz.NewServerMetricsForTesting(
			1,
			2,
			3,
			time.Now().UnixNano(),
		),
		channelz.NewServerMetricsForTesting(
			1,
			0,
			0,
			time.Now().UnixNano(),
		),
	}

	firstID := int64(0)
	for i, s := range ss {
		svr := channelz.RegisterServer("")
		if i == 0 {
			firstID = svr.ID
		}
		svr.ServerMetrics.CopyFrom(s)
		defer channelz.RemoveEntry(svr.ID)
	}
	svr := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := svr.GetServers(ctx, &channelzpb.GetServersRequest{StartServerId: 0})
	if !resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want true, got %v", resp.GetEnd())
	}
	serversWant := []*channelzpb.Server{
		{
			Ref: &channelzpb.ServerRef{ServerId: firstID, Name: ""},
			Data: &channelzpb.ServerData{
				CallsStarted:             6,
				CallsSucceeded:           2,
				CallsFailed:              3,
				LastCallStartedTimestamp: timestamppb.New(time.Unix(0, ss[0].LastCallStartedTimestamp.Load())),
			},
		},
		{
			Ref: &channelzpb.ServerRef{ServerId: firstID + 1, Name: ""},
			Data: &channelzpb.ServerData{
				CallsStarted:             1,
				CallsSucceeded:           2,
				CallsFailed:              3,
				LastCallStartedTimestamp: timestamppb.New(time.Unix(0, ss[1].LastCallStartedTimestamp.Load())),
			},
		},
		{
			Ref: &channelzpb.ServerRef{ServerId: firstID + 2, Name: ""},
			Data: &channelzpb.ServerData{
				CallsStarted:             1,
				CallsSucceeded:           0,
				CallsFailed:              0,
				LastCallStartedTimestamp: timestamppb.New(time.Unix(0, ss[2].LastCallStartedTimestamp.Load())),
			},
		},
	}
	if diff := cmp.Diff(serversWant, resp.GetServer(), protocmp.Transform()); diff != "" {
		t.Fatalf("unexpected server, diff (-want +got):\n%s", diff)
	}
	for i := 0; i < 50; i++ {
		id := channelz.RegisterServer("").ID
		defer channelz.RemoveEntry(id)
	}
	resp, _ = svr.GetServers(ctx, &channelzpb.GetServersRequest{StartServerId: 0})
	if resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want false, got %v", resp.GetEnd())
	}
}

func (s) TestGetServerSockets(t *testing.T) {
	svrID := channelz.RegisterServer("")
	defer channelz.RemoveEntry(svrID.ID)
	refNames := []string{"listen socket 1", "normal socket 1", "normal socket 2"}
	ids := make([]int64, 3)
	ids[0] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeListen, Parent: svrID, RefName: refNames[0]}).ID
	ids[1] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: svrID, RefName: refNames[1]}).ID
	ids[2] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: svrID, RefName: refNames[2]}).ID
	for _, id := range ids {
		defer channelz.RemoveEntry(id)
	}
	svr := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := svr.GetServerSockets(ctx, &channelzpb.GetServerSocketsRequest{ServerId: svrID.ID, StartSocketId: 0})
	if !resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want: true, got: %v", resp.GetEnd())
	}
	// GetServerSockets only return normal sockets.
	want := map[int64]string{
		ids[1]: refNames[1],
		ids[2]: refNames[2],
	}
	if got := convertSocketRefSliceToMap(resp.GetSocketRef()); !cmp.Equal(got, want) {
		t.Fatalf("GetServerSockets want: %#v, got: %#v (resp=%v)", want, got, prototext.Format(resp))
	}

	for i := 0; i < 50; i++ {
		id := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: svrID})
		defer channelz.RemoveEntry(id.ID)
	}
	resp, _ = svr.GetServerSockets(ctx, &channelzpb.GetServerSocketsRequest{ServerId: svrID.ID, StartSocketId: 0})
	if resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want false, got %v", resp.GetEnd())
	}
}

// This test makes a GetServerSockets with a non-zero start ID, and expect only
// sockets with ID >= the given start ID.
func (s) TestGetServerSocketsNonZeroStartID(t *testing.T) {
	svrID := channelz.RegisterServer("test server")
	defer channelz.RemoveEntry(svrID.ID)
	refNames := []string{"listen socket 1", "normal socket 1", "normal socket 2"}
	ids := make([]int64, 3)
	ids[0] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeListen, Parent: svrID, RefName: refNames[0]}).ID
	ids[1] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: svrID, RefName: refNames[1]}).ID
	ids[2] = channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: svrID, RefName: refNames[2]}).ID
	for _, id := range ids {
		defer channelz.RemoveEntry(id)
	}
	svr := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Make GetServerSockets with startID = ids[1]+1, so socket-1 won't be
	// included in the response.
	resp, _ := svr.GetServerSockets(ctx, &channelzpb.GetServerSocketsRequest{ServerId: svrID.ID, StartSocketId: ids[1] + 1})
	if !resp.GetEnd() {
		t.Fatalf("resp.GetEnd() want: true, got: %v", resp.GetEnd())
	}
	// GetServerSockets only return normal socket-2, socket-1 should be
	// filtered by start ID.
	want := map[int64]string{
		ids[2]: refNames[2],
	}
	if !cmp.Equal(convertSocketRefSliceToMap(resp.GetSocketRef()), want) {
		t.Fatalf("GetServerSockets want: %#v, got: %#v", want, resp.GetSocketRef())
	}
}

var logger = grpclog.Component("channelz")

func (s) TestGetChannel(t *testing.T) {
	refNames := []string{"top channel 1", "nested channel 1", "sub channel 2", "nested channel 3"}
	cids := make([]*channelz.Channel, 3)
	cids[0] = channelz.RegisterChannel(nil, refNames[0])
	channelz.AddTraceEvent(logger, cids[0], 0, &channelz.TraceEvent{
		Desc:     "Channel Created",
		Severity: channelz.CtInfo,
	})

	cids[1] = channelz.RegisterChannel(cids[0], refNames[1])
	channelz.AddTraceEvent(logger, cids[1], 0, &channelz.TraceEvent{
		Desc:     "Channel Created",
		Severity: channelz.CtInfo,
		Parent: &channelz.TraceEvent{
			Desc:     fmt.Sprintf("Nested Channel(id:%d) created", cids[1].ID),
			Severity: channelz.CtInfo,
		},
	})

	subChan := channelz.RegisterSubChannel(cids[0], refNames[2])
	channelz.AddTraceEvent(logger, subChan, 0, &channelz.TraceEvent{
		Desc:     "SubChannel Created",
		Severity: channelz.CtInfo,
		Parent: &channelz.TraceEvent{
			Desc:     fmt.Sprintf("SubChannel(id:%d) created", subChan.ID),
			Severity: channelz.CtInfo,
		},
	})
	defer channelz.RemoveEntry(subChan.ID)

	cids[2] = channelz.RegisterChannel(cids[1], refNames[3])
	channelz.AddTraceEvent(logger, cids[2], 0, &channelz.TraceEvent{
		Desc:     "Channel Created",
		Severity: channelz.CtInfo,
		Parent: &channelz.TraceEvent{
			Desc:     fmt.Sprintf("Nested Channel(id:%d) created", cids[2].ID),
			Severity: channelz.CtInfo,
		},
	})
	channelz.AddTraceEvent(logger, cids[0], 0, &channelz.TraceEvent{
		Desc:     fmt.Sprintf("Channel Connectivity change to %v", connectivity.Ready),
		Severity: channelz.CtInfo,
	})
	channelz.AddTraceEvent(logger, cids[0], 0, &channelz.TraceEvent{
		Desc:     "Resolver returns an empty address list",
		Severity: channelz.CtWarning,
	})

	for _, id := range cids {
		defer channelz.RemoveEntry(id.ID)
	}

	svr := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := svr.GetChannel(ctx, &channelzpb.GetChannelRequest{ChannelId: cids[0].ID})
	metrics := resp.GetChannel()
	subChans := metrics.GetSubchannelRef()
	if len(subChans) != 1 || subChans[0].GetName() != refNames[2] || subChans[0].GetSubchannelId() != subChan.ID {
		t.Fatalf("metrics.GetSubChannelRef() want %#v, got %#v", []*channelzpb.SubchannelRef{{SubchannelId: subChan.ID, Name: refNames[2]}}, subChans)
	}
	nestedChans := metrics.GetChannelRef()
	if len(nestedChans) != 1 || nestedChans[0].GetName() != refNames[1] || nestedChans[0].GetChannelId() != cids[1].ID {
		t.Fatalf("metrics.GetChannelRef() want %#v, got %#v", []*channelzpb.ChannelRef{{ChannelId: cids[1].ID, Name: refNames[1]}}, nestedChans)
	}
	trace := metrics.GetData().GetTrace()
	want := []struct {
		desc     string
		severity channelzpb.ChannelTraceEvent_Severity
		childID  int64
		childRef string
	}{
		{desc: "Channel Created", severity: channelzpb.ChannelTraceEvent_CT_INFO},
		{desc: fmt.Sprintf("Nested Channel(id:%d) created", cids[1].ID), severity: channelzpb.ChannelTraceEvent_CT_INFO, childID: cids[1].ID, childRef: refNames[1]},
		{desc: fmt.Sprintf("SubChannel(id:%d) created", subChan.ID), severity: channelzpb.ChannelTraceEvent_CT_INFO, childID: subChan.ID, childRef: refNames[2]},
		{desc: fmt.Sprintf("Channel Connectivity change to %v", connectivity.Ready), severity: channelzpb.ChannelTraceEvent_CT_INFO},
		{desc: "Resolver returns an empty address list", severity: channelzpb.ChannelTraceEvent_CT_WARNING},
	}

	for i, e := range trace.Events {
		if !strings.Contains(e.GetDescription(), want[i].desc) {
			t.Fatalf("trace: GetDescription want %#v, got %#v", want[i].desc, e.GetDescription())
		}
		if e.GetSeverity() != want[i].severity {
			t.Fatalf("trace: GetSeverity want %#v, got %#v", want[i].severity, e.GetSeverity())
		}
		if want[i].childID == 0 && (e.GetChannelRef() != nil || e.GetSubchannelRef() != nil) {
			t.Fatalf("trace: GetChannelRef() should return nil, as there is no reference")
		}
		if e.GetChannelRef().GetChannelId() != want[i].childID || e.GetChannelRef().GetName() != want[i].childRef {
			if e.GetSubchannelRef().GetSubchannelId() != want[i].childID || e.GetSubchannelRef().GetName() != want[i].childRef {
				t.Fatalf("trace: GetChannelRef/GetSubchannelRef want (child ID: %d, child name: %q), got %#v and %#v", want[i].childID, want[i].childRef, e.GetChannelRef(), e.GetSubchannelRef())
			}
		}
	}
	resp, _ = svr.GetChannel(ctx, &channelzpb.GetChannelRequest{ChannelId: cids[1].ID})
	metrics = resp.GetChannel()
	nestedChans = metrics.GetChannelRef()
	if len(nestedChans) != 1 || nestedChans[0].GetName() != refNames[3] || nestedChans[0].GetChannelId() != cids[2].ID {
		t.Fatalf("metrics.GetChannelRef() want %#v, got %#v", []*channelzpb.ChannelRef{{ChannelId: cids[2].ID, Name: refNames[3]}}, nestedChans)
	}
}

func (s) TestGetSubChannel(t *testing.T) {
	var (
		subchanCreated            = "SubChannel Created"
		subchanConnectivityChange = fmt.Sprintf("Subchannel Connectivity change to %v", connectivity.Ready)
		subChanPickNewAddress     = fmt.Sprintf("Subchannel picks a new address %q to connect", "0.0.0.0")
	)

	refNames := []string{"top channel 1", "sub channel 1", "socket 1", "socket 2"}
	chann := channelz.RegisterChannel(nil, refNames[0])
	defer channelz.RemoveEntry(chann.ID)
	channelz.AddTraceEvent(logger, chann, 0, &channelz.TraceEvent{
		Desc:     "Channel Created",
		Severity: channelz.CtInfo,
	})
	subChan := channelz.RegisterSubChannel(chann, refNames[1])
	defer channelz.RemoveEntry(subChan.ID)
	channelz.AddTraceEvent(logger, subChan, 0, &channelz.TraceEvent{
		Desc:     subchanCreated,
		Severity: channelz.CtInfo,
		Parent: &channelz.TraceEvent{
			Desc:     fmt.Sprintf("Nested Channel(id:%d) created", chann.ID),
			Severity: channelz.CtInfo,
		},
	})
	skt1 := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: subChan, RefName: refNames[2]})
	defer channelz.RemoveEntry(skt1.ID)
	skt2 := channelz.RegisterSocket(&channelz.Socket{SocketType: channelz.SocketTypeNormal, Parent: subChan, RefName: refNames[3]})
	defer channelz.RemoveEntry(skt2.ID)
	channelz.AddTraceEvent(logger, subChan, 0, &channelz.TraceEvent{
		Desc:     subchanConnectivityChange,
		Severity: channelz.CtInfo,
	})
	channelz.AddTraceEvent(logger, subChan, 0, &channelz.TraceEvent{
		Desc:     subChanPickNewAddress,
		Severity: channelz.CtInfo,
	})
	svr := newCZServer()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	resp, _ := svr.GetSubchannel(ctx, &channelzpb.GetSubchannelRequest{SubchannelId: subChan.ID})
	metrics := resp.GetSubchannel()
	want := map[int64]string{
		skt1.ID: refNames[2],
		skt2.ID: refNames[3],
	}
	if !cmp.Equal(convertSocketRefSliceToMap(metrics.GetSocketRef()), want) {
		t.Fatalf("metrics.GetSocketRef() want %#v: got: %#v", want, metrics.GetSocketRef())
	}

	trace := metrics.GetData().GetTrace()
	wantTrace := []struct {
		desc     string
		severity channelzpb.ChannelTraceEvent_Severity
		childID  int64
		childRef string
	}{
		{desc: subchanCreated, severity: channelzpb.ChannelTraceEvent_CT_INFO},
		{desc: subchanConnectivityChange, severity: channelzpb.ChannelTraceEvent_CT_INFO},
		{desc: subChanPickNewAddress, severity: channelzpb.ChannelTraceEvent_CT_INFO},
	}
	for i, e := range trace.Events {
		if e.GetDescription() != wantTrace[i].desc {
			t.Fatalf("trace: GetDescription want %#v, got %#v", wantTrace[i].desc, e.GetDescription())
		}
		if e.GetSeverity() != wantTrace[i].severity {
			t.Fatalf("trace: GetSeverity want %#v, got %#v", wantTrace[i].severity, e.GetSeverity())
		}
		if wantTrace[i].childID == 0 && (e.GetChannelRef() != nil || e.GetSubchannelRef() != nil) {
			t.Fatalf("trace: GetChannelRef() should return nil, as there is no reference")
		}
		if e.GetChannelRef().GetChannelId() != wantTrace[i].childID || e.GetChannelRef().GetName() != wantTrace[i].childRef {
			if e.GetSubchannelRef().GetSubchannelId() != wantTrace[i].childID || e.GetSubchannelRef().GetName() != wantTrace[i].childRef {
				t.Fatalf("trace: GetChannelRef/GetSubchannelRef want (child ID: %d, child name: %q), got %#v and %#v", wantTrace[i].childID, wantTrace[i].childRef, e.GetChannelRef(), e.GetSubchannelRef())
			}
		}
	}
}

type czSocket struct {
	streamsStarted                   int64
	streamsSucceeded                 int64
	streamsFailed                    int64
	messagesSent                     int64
	messagesReceived                 int64
	keepAlivesSent                   int64
	lastLocalStreamCreatedTimestamp  time.Time
	lastRemoteStreamCreatedTimestamp time.Time
	lastMessageSentTimestamp         time.Time
	lastMessageReceivedTimestamp     time.Time
	localFlowControlWindow           int64
	remoteFlowControlWindow          int64

	localAddr     net.Addr
	remoteAddr    net.Addr
	remoteName    string
	socketOptions *channelz.SocketOptionData
	security      credentials.ChannelzSecurityValue
}

func newSocket(cs czSocket) *channelz.Socket {
	if cs.lastLocalStreamCreatedTimestamp.IsZero() {
		cs.lastLocalStreamCreatedTimestamp = time.Unix(0, 0)
	}
	if cs.lastRemoteStreamCreatedTimestamp.IsZero() {
		cs.lastRemoteStreamCreatedTimestamp = time.Unix(0, 0)
	}
	if cs.lastMessageSentTimestamp.IsZero() {
		cs.lastMessageSentTimestamp = time.Unix(0, 0)
	}
	if cs.lastMessageReceivedTimestamp.IsZero() {
		cs.lastMessageReceivedTimestamp = time.Unix(0, 0)
	}

	s := &channelz.Socket{
		LocalAddr:     cs.localAddr,
		RemoteAddr:    cs.remoteAddr,
		RemoteName:    cs.remoteName,
		SocketOptions: cs.socketOptions,
		Security:      cs.security,
	}
	s.SocketMetrics.StreamsStarted.Store(cs.streamsStarted)
	s.SocketMetrics.StreamsSucceeded.Store(cs.streamsSucceeded)
	s.SocketMetrics.StreamsFailed.Store(cs.streamsFailed)
	s.SocketMetrics.MessagesSent.Store(cs.messagesSent)
	s.SocketMetrics.MessagesReceived.Store(cs.messagesReceived)
	s.SocketMetrics.KeepAlivesSent.Store(cs.keepAlivesSent)
	s.SocketMetrics.LastLocalStreamCreatedTimestamp.Store(cs.lastLocalStreamCreatedTimestamp.UnixNano())
	s.SocketMetrics.LastRemoteStreamCreatedTimestamp.Store(cs.lastRemoteStreamCreatedTimestamp.UnixNano())
	s.SocketMetrics.LastMessageSentTimestamp.Store(cs.lastMessageSentTimestamp.UnixNano())
	s.SocketMetrics.LastMessageReceivedTimestamp.Store(cs.lastMessageReceivedTimestamp.UnixNano())
	s.EphemeralMetrics = func() *channelz.EphemeralSocketMetrics {
		return &channelz.EphemeralSocketMetrics{
			LocalFlowControlWindow:  cs.localFlowControlWindow,
			RemoteFlowControlWindow: cs.remoteFlowControlWindow,
		}
	}
	return s
}

type OtherChannelzSecurityValue struct {
	LocalCertificate  []byte `protobuf:"bytes,1,opt,name=local_certificate,json=localCertificate,proto3" json:"local_certificate,omitempty"`
	RemoteCertificate []byte `protobuf:"bytes,2,opt,name=remote_certificate,json=remoteCertificate,proto3" json:"remote_certificate,omitempty"`
}

func (x *OtherChannelzSecurityValue) Reset() {
	*x = OtherChannelzSecurityValue{}
}

func (x *OtherChannelzSecurityValue) String() string {
	return prototext.Format(x)
}

func (*OtherChannelzSecurityValue) ProtoMessage() {}

func (x OtherChannelzSecurityValue) ProtoReflect() protoreflect.Message {
	const s = `
		name:   "service_test.proto"
		syntax: "proto3"
		package: "grpc.credentials",
		message_type: [{
			name: "OtherChannelzSecurityValue"
			field: [
				{name:"local_certificate"  number:1 type:TYPE_BYTES},
				{name:"remote_certificate"  number:2 type:TYPE_BYTES}
			]
		}]
	`
	pb := new(descriptorpb.FileDescriptorProto)
	if err := prototext.Unmarshal([]byte(s), pb); err != nil {
		panic(err)
	}
	fd, err := protodesc.NewFile(pb, nil)
	if err != nil {
		panic(err)
	}
	md := fd.Messages().Get(0)
	mt := dynamicpb.NewMessageType(md)
	return mt.New()
}

func (s) TestGetSocket(t *testing.T) {
	ss := []*channelz.Socket{newSocket(czSocket{
		streamsStarted:                   10,
		streamsSucceeded:                 2,
		streamsFailed:                    3,
		messagesSent:                     20,
		messagesReceived:                 10,
		keepAlivesSent:                   2,
		lastLocalStreamCreatedTimestamp:  time.Unix(0, 0),
		lastRemoteStreamCreatedTimestamp: time.Unix(1, 0),
		lastMessageSentTimestamp:         time.Unix(2, 0),
		lastMessageReceivedTimestamp:     time.Unix(3, 0),
		localFlowControlWindow:           65536,
		remoteFlowControlWindow:          1024,
		localAddr:                        &net.TCPAddr{IP: net.ParseIP("1.0.0.1"), Port: 10001},
		remoteAddr:                       &net.TCPAddr{IP: net.ParseIP("12.0.0.1"), Port: 10002},
		remoteName:                       "remote.remote",
	}), newSocket(czSocket{
		streamsStarted:                   10,
		streamsSucceeded:                 2,
		streamsFailed:                    3,
		messagesSent:                     20,
		messagesReceived:                 10,
		keepAlivesSent:                   2,
		lastLocalStreamCreatedTimestamp:  time.Unix(0, 0),
		lastRemoteStreamCreatedTimestamp: time.Unix(5, 0),
		lastMessageSentTimestamp:         time.Unix(6, 0),
		lastMessageReceivedTimestamp:     time.Unix(7, 0),
		localFlowControlWindow:           65536,
		remoteFlowControlWindow:          1024,
		localAddr:                        &net.UnixAddr{Name: "file.path", Net: "unix"},
		remoteAddr:                       &net.UnixAddr{Name: "another.path", Net: "unix"},
		remoteName:                       "remote.remote",
	}), newSocket(czSocket{
		streamsStarted:                   5,
		streamsSucceeded:                 2,
		streamsFailed:                    3,
		messagesSent:                     20,
		messagesReceived:                 10,
		keepAlivesSent:                   2,
		lastLocalStreamCreatedTimestamp:  time.Unix(10, 10),
		lastRemoteStreamCreatedTimestamp: time.Unix(0, 0),
		lastMessageSentTimestamp:         time.Unix(0, 0),
		lastMessageReceivedTimestamp:     time.Unix(0, 0),
		localFlowControlWindow:           65536,
		remoteFlowControlWindow:          10240,
		localAddr:                        &net.IPAddr{IP: net.ParseIP("1.0.0.1")},
		remoteAddr:                       &net.IPAddr{IP: net.ParseIP("9.0.0.1")},
		remoteName:                       "",
	}), newSocket(czSocket{
		localAddr: &net.TCPAddr{IP: net.ParseIP("127.0.0.1"), Port: 10001},
	}), newSocket(czSocket{
		security: &credentials.TLSChannelzSecurityValue{
			StandardName:      "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
			RemoteCertificate: []byte{48, 130, 2, 156, 48, 130, 2, 5, 160},
		},
	}), newSocket(czSocket{
		security: &credentials.OtherChannelzSecurityValue{
			Name: "XXXX",
		},
	}), newSocket(czSocket{
		security: &credentials.OtherChannelzSecurityValue{
			Name: "YYYY",
			Value: OtherChannelzSecurityValue{
				LocalCertificate:  []byte{1, 2, 3},
				RemoteCertificate: []byte{4, 5, 6},
			},
		},
	}),
	}
	otherSecVal, err := anypb.New(ss[6].Security.(*credentials.OtherChannelzSecurityValue).Value)
	if err != nil {
		t.Fatal("Error marshalling proto:", err)
	}

	svr := newCZServer()
	skts := make([]*channelz.Socket, len(ss))
	svrID := channelz.RegisterServer("")
	defer channelz.RemoveEntry(svrID.ID)
	for i, s := range ss {
		s.Parent = svrID
		s.RefName = strconv.Itoa(i)
		skts[i] = channelz.RegisterSocket(s)
		defer channelz.RemoveEntry(skts[i].ID)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	emptyData := `data: {
		last_local_stream_created_timestamp: {seconds: 0 nanos: 0}
		last_remote_stream_created_timestamp: {seconds: 0 nanos: 0}
		last_message_sent_timestamp: {seconds: 0 nanos: 0}
		last_message_received_timestamp: {seconds: 0 nanos: 0}
		local_flow_control_window: { value: 0 }
		remote_flow_control_window: { value: 0 }
	}`
	want := []string{`
		ref: {socket_id: ` + fmt.Sprint(skts[0].ID) + ` name: "0" }
		data: {
			streams_started: 10
			streams_succeeded: 2
			streams_failed: 3
			messages_sent: 20
			messages_received: 10
			keep_alives_sent: 2
			last_local_stream_created_timestamp: {seconds: 0 nanos: 0}
			last_remote_stream_created_timestamp: {seconds: 1 nanos: 0}
			last_message_sent_timestamp: {seconds: 2 nanos: 0}
			last_message_received_timestamp: {seconds: 3 nanos: 0}
			local_flow_control_window: { value: 65536 }
			remote_flow_control_window: { value: 1024 }
		}
		local: { tcpip_address: { ip_address: "` + addr(skts[0].LocalAddr) + `" port: 10001 } }
		remote: { tcpip_address: { ip_address: "` + addr(skts[0].RemoteAddr) + `" port: 10002 } }
		remote_name: "remote.remote"`,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[1].ID) + ` name: "1" }
		data: {
			streams_started: 10
			streams_succeeded: 2
			streams_failed: 3
			messages_sent: 20
			messages_received: 10
			keep_alives_sent: 2
			last_local_stream_created_timestamp: {seconds: 0 nanos: 0}
			last_remote_stream_created_timestamp: {seconds: 5 nanos: 0}
			last_message_sent_timestamp: {seconds: 6 nanos: 0}
			last_message_received_timestamp: {seconds: 7 nanos: 0}
			local_flow_control_window: { value: 65536 }
			remote_flow_control_window: { value: 1024 }
		}
		local: { uds_address { filename: "file.path" } }
		remote: { uds_address { filename: "another.path" } }
		remote_name: "remote.remote"`,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[2].ID) + ` name: "2" }
		data: {
			streams_started: 5
			streams_succeeded: 2
			streams_failed: 3
			messages_sent: 20
			messages_received: 10
			keep_alives_sent: 2
			last_local_stream_created_timestamp: {seconds: 10 nanos: 10}
			last_remote_stream_created_timestamp: {seconds: 0 nanos: 0}
			last_message_sent_timestamp: {seconds: 0 nanos: 0}
			last_message_received_timestamp: {seconds: 0 nanos: 0}
			local_flow_control_window: { value: 65536 }
			remote_flow_control_window: { value: 10240 }
		}
		local: { tcpip_address: { ip_address: "` + addr(skts[2].LocalAddr) + `" } }
		remote: { tcpip_address: { ip_address: "` + addr(skts[2].RemoteAddr) + `" } }
		remote_name: ""`,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[3].ID) + ` name: "3" }
		local: { tcpip_address: { ip_address: "` + addr(skts[3].LocalAddr) + `" port: 10001 } }
		` + emptyData,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[4].ID) + ` name: "4" }
		security: { tls: {
			standard_name: "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
			remote_certificate: "\x30\x82\x02\x9c\x30\x82\x02\x05\xa0"
		} }
		` + emptyData,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[5].ID) + ` name: "5" }
		security: { other: { name: "XXXX" } }
		` + emptyData,
		`
		ref: {socket_id: ` + fmt.Sprint(skts[6].ID) + ` name: "6" }
		security: { other: {
			name: "YYYY"
			value: {
				type_url: "type.googleapis.com/grpc.credentials.OtherChannelzSecurityValue"
				value: "` + escape(otherSecVal.Value) + `"
			}
		} }
		` + emptyData,
	}

	for i := range ss {
		resp, _ := svr.GetSocket(ctx, &channelzpb.GetSocketRequest{SocketId: skts[i].ID})
		w := &channelzpb.Socket{}
		if err := prototext.Unmarshal([]byte(want[i]), w); err != nil {
			t.Fatalf("Error unmarshalling %q: %v", want[i], err)
		}
		if diff := cmp.Diff(resp.GetSocket(), w, protocmp.Transform()); diff != "" {
			t.Fatalf("Socket %v did not match expected.  -got +want: %v", i, diff)
		}
	}
}

func escape(bs []byte) string {
	ret := ""
	for _, b := range bs {
		ret += fmt.Sprintf("\\x%02x", b)
	}
	return ret
}

func addr(a net.Addr) string {
	switch a := a.(type) {
	case *net.TCPAddr:
		return escape([]byte(a.IP))
	case *net.IPAddr:
		return escape([]byte(a.IP))
	}
	return ""
}
