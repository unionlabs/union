/*
 *
 * Copyright 2024 gRPC authors.
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

package protoconv

import (
	"net"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"google.golang.org/protobuf/types/known/wrapperspb"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func securityToProto(se credentials.ChannelzSecurityValue) *channelzpb.Security {
	switch v := se.(type) {
	case *credentials.TLSChannelzSecurityValue:
		return &channelzpb.Security{Model: &channelzpb.Security_Tls_{Tls: &channelzpb.Security_Tls{
			CipherSuite:       &channelzpb.Security_Tls_StandardName{StandardName: v.StandardName},
			LocalCertificate:  v.LocalCertificate,
			RemoteCertificate: v.RemoteCertificate,
		}}}
	case *credentials.OtherChannelzSecurityValue:
		otherSecurity := &channelzpb.Security_OtherSecurity{
			Name: v.Name,
		}
		if anyval, err := anypb.New(v.Value); err == nil {
			otherSecurity.Value = anyval
		}
		return &channelzpb.Security{Model: &channelzpb.Security_Other{Other: otherSecurity}}
	}
	return nil
}

func addrToProto(a net.Addr) *channelzpb.Address {
	if a == nil {
		return nil
	}
	switch a.Network() {
	case "udp":
		// TODO: Address_OtherAddress{}. Need proto def for Value.
	case "ip":
		// Note zone info is discarded through the conversion.
		return &channelzpb.Address{Address: &channelzpb.Address_TcpipAddress{TcpipAddress: &channelzpb.Address_TcpIpAddress{IpAddress: a.(*net.IPAddr).IP}}}
	case "ip+net":
		// Note mask info is discarded through the conversion.
		return &channelzpb.Address{Address: &channelzpb.Address_TcpipAddress{TcpipAddress: &channelzpb.Address_TcpIpAddress{IpAddress: a.(*net.IPNet).IP}}}
	case "tcp":
		// Note zone info is discarded through the conversion.
		return &channelzpb.Address{Address: &channelzpb.Address_TcpipAddress{TcpipAddress: &channelzpb.Address_TcpIpAddress{IpAddress: a.(*net.TCPAddr).IP, Port: int32(a.(*net.TCPAddr).Port)}}}
	case "unix", "unixgram", "unixpacket":
		return &channelzpb.Address{Address: &channelzpb.Address_UdsAddress_{UdsAddress: &channelzpb.Address_UdsAddress{Filename: a.String()}}}
	default:
	}
	return &channelzpb.Address{}
}

func socketToProto(skt *channelz.Socket) *channelzpb.Socket {
	s := &channelzpb.Socket{}
	s.Ref = &channelzpb.SocketRef{SocketId: skt.ID, Name: skt.RefName}

	s.Data = &channelzpb.SocketData{
		StreamsStarted:   skt.SocketMetrics.StreamsStarted.Load(),
		StreamsSucceeded: skt.SocketMetrics.StreamsSucceeded.Load(),
		StreamsFailed:    skt.SocketMetrics.StreamsFailed.Load(),
		MessagesSent:     skt.SocketMetrics.MessagesSent.Load(),
		MessagesReceived: skt.SocketMetrics.MessagesReceived.Load(),
		KeepAlivesSent:   skt.SocketMetrics.KeepAlivesSent.Load(),
	}
	if ts := timestamppb.New(time.Unix(0, skt.SocketMetrics.LastLocalStreamCreatedTimestamp.Load())); ts.IsValid() {
		s.Data.LastLocalStreamCreatedTimestamp = ts
	}
	if ts := timestamppb.New(time.Unix(0, skt.SocketMetrics.LastRemoteStreamCreatedTimestamp.Load())); ts.IsValid() {
		s.Data.LastRemoteStreamCreatedTimestamp = ts
	}
	if ts := timestamppb.New(time.Unix(0, skt.SocketMetrics.LastMessageSentTimestamp.Load())); ts.IsValid() {
		s.Data.LastMessageSentTimestamp = ts
	}
	if ts := timestamppb.New(time.Unix(0, skt.SocketMetrics.LastMessageReceivedTimestamp.Load())); ts.IsValid() {
		s.Data.LastMessageReceivedTimestamp = ts
	}
	if skt.EphemeralMetrics != nil {
		e := skt.EphemeralMetrics()
		s.Data.LocalFlowControlWindow = wrapperspb.Int64(e.LocalFlowControlWindow)
		s.Data.RemoteFlowControlWindow = wrapperspb.Int64(e.RemoteFlowControlWindow)
	}

	s.Data.Option = sockoptToProto(skt.SocketOptions)
	s.Security = securityToProto(skt.Security)
	s.Local = addrToProto(skt.LocalAddr)
	s.Remote = addrToProto(skt.RemoteAddr)
	s.RemoteName = skt.RemoteName
	return s
}

// GetServerSockets returns the protobuf representation of the server (listen)
// sockets starting at startID (max of len), and returns end=true if no server
// sockets exist with higher IDs.
func GetServerSockets(serverID, startID int64, len int) (sockets []*channelzpb.SocketRef, end bool) {
	skts, end := channelz.GetServerSockets(serverID, startID, len)
	for _, m := range skts {
		sockets = append(sockets, &channelzpb.SocketRef{SocketId: m.ID, Name: m.RefName})
	}
	return sockets, end
}

// GetSocket returns the protobuf representation of the socket with the given
// ID.
func GetSocket(id int64) (*channelzpb.Socket, error) {
	skt := channelz.GetSocket(id)
	if skt == nil {
		return nil, status.Errorf(codes.NotFound, "requested socket %d not found", id)
	}
	return socketToProto(skt), nil
}
