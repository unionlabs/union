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

func subChannelToProto(cm *channelz.SubChannel) *channelzpb.Subchannel {
	sc := &channelzpb.Subchannel{}
	sc.Ref = &channelzpb.SubchannelRef{SubchannelId: cm.ID, Name: cm.RefName}

	sc.Data = &channelzpb.ChannelData{
		State:          connectivityStateToProto(cm.ChannelMetrics.State.Load()),
		Target:         strFromPointer(cm.ChannelMetrics.Target.Load()),
		CallsStarted:   cm.ChannelMetrics.CallsStarted.Load(),
		CallsSucceeded: cm.ChannelMetrics.CallsSucceeded.Load(),
		CallsFailed:    cm.ChannelMetrics.CallsFailed.Load(),
	}
	if ts := timestamppb.New(time.Unix(0, cm.ChannelMetrics.LastCallStartedTimestamp.Load())); ts.IsValid() {
		sc.Data.LastCallStartedTimestamp = ts
	}

	skts := cm.Sockets()
	sockets := make([]*channelzpb.SocketRef, 0, len(skts))
	for id, ref := range skts {
		sockets = append(sockets, &channelzpb.SocketRef{SocketId: id, Name: ref})
	}
	sc.SocketRef = sockets
	sc.Data.Trace = channelTraceToProto(cm.Trace())
	return sc
}

// GetSubChannel returns the protobuf representation of the subchannel with the
// given ID.
func GetSubChannel(id int64) (*channelzpb.Subchannel, error) {
	subChan := channelz.GetSubChannel(id)
	if subChan == nil {
		return nil, status.Errorf(codes.NotFound, "requested sub channel %d not found", id)
	}
	return subChannelToProto(subChan), nil
}
