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
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func serverToProto(sm *channelz.Server) *channelzpb.Server {
	s := &channelzpb.Server{}
	s.Ref = &channelzpb.ServerRef{ServerId: sm.ID, Name: sm.RefName}

	s.Data = &channelzpb.ServerData{
		CallsStarted:   sm.ServerMetrics.CallsStarted.Load(),
		CallsSucceeded: sm.ServerMetrics.CallsSucceeded.Load(),
		CallsFailed:    sm.ServerMetrics.CallsFailed.Load(),
	}

	if ts := timestamppb.New(time.Unix(0, sm.ServerMetrics.LastCallStartedTimestamp.Load())); ts.IsValid() {
		s.Data.LastCallStartedTimestamp = ts
	}
	lss := sm.ListenSockets()
	sockets := make([]*channelzpb.SocketRef, 0, len(lss))
	for id, ref := range lss {
		sockets = append(sockets, &channelzpb.SocketRef{SocketId: id, Name: ref})
	}
	s.ListenSocket = sockets
	return s
}

// GetServers returns the protobuf representation of the servers starting at
// startID (max of len), and returns end=true if no servers exist with higher
// IDs.
func GetServers(startID int64, len int) (servers []*channelzpb.Server, end bool) {
	srvs, end := channelz.GetServers(startID, len)
	for _, srv := range srvs {
		servers = append(servers, serverToProto(srv))
	}
	return servers, end
}

// GetServer returns the protobuf representation of the server with the given
// ID.
func GetServer(id int64) (*channelzpb.Server, error) {
	srv := channelz.GetServer(id)
	if srv == nil {
		return nil, status.Errorf(codes.NotFound, "requested server %d not found", id)
	}
	return serverToProto(srv), nil
}
