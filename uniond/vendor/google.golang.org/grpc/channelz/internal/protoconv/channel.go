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
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
)

func connectivityStateToProto(s *connectivity.State) *channelzpb.ChannelConnectivityState {
	if s == nil {
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_UNKNOWN}
	}
	switch *s {
	case connectivity.Idle:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_IDLE}
	case connectivity.Connecting:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_CONNECTING}
	case connectivity.Ready:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_READY}
	case connectivity.TransientFailure:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_TRANSIENT_FAILURE}
	case connectivity.Shutdown:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_SHUTDOWN}
	default:
		return &channelzpb.ChannelConnectivityState{State: channelzpb.ChannelConnectivityState_UNKNOWN}
	}
}

func channelTraceToProto(ct *channelz.ChannelTrace) *channelzpb.ChannelTrace {
	pbt := &channelzpb.ChannelTrace{}
	if ct == nil {
		return pbt
	}
	pbt.NumEventsLogged = ct.EventNum
	if ts := timestamppb.New(ct.CreationTime); ts.IsValid() {
		pbt.CreationTimestamp = ts
	}
	events := make([]*channelzpb.ChannelTraceEvent, 0, len(ct.Events))
	for _, e := range ct.Events {
		cte := &channelzpb.ChannelTraceEvent{
			Description: e.Desc,
			Severity:    channelzpb.ChannelTraceEvent_Severity(e.Severity),
		}
		if ts := timestamppb.New(e.Timestamp); ts.IsValid() {
			cte.Timestamp = ts
		}
		if e.RefID != 0 {
			switch e.RefType {
			case channelz.RefChannel:
				cte.ChildRef = &channelzpb.ChannelTraceEvent_ChannelRef{ChannelRef: &channelzpb.ChannelRef{ChannelId: e.RefID, Name: e.RefName}}
			case channelz.RefSubChannel:
				cte.ChildRef = &channelzpb.ChannelTraceEvent_SubchannelRef{SubchannelRef: &channelzpb.SubchannelRef{SubchannelId: e.RefID, Name: e.RefName}}
			}
		}
		events = append(events, cte)
	}
	pbt.Events = events
	return pbt
}

func channelToProto(cm *channelz.Channel) *channelzpb.Channel {
	c := &channelzpb.Channel{}
	c.Ref = &channelzpb.ChannelRef{ChannelId: cm.ID, Name: cm.RefName}

	c.Data = &channelzpb.ChannelData{
		State:          connectivityStateToProto(cm.ChannelMetrics.State.Load()),
		Target:         strFromPointer(cm.ChannelMetrics.Target.Load()),
		CallsStarted:   cm.ChannelMetrics.CallsStarted.Load(),
		CallsSucceeded: cm.ChannelMetrics.CallsSucceeded.Load(),
		CallsFailed:    cm.ChannelMetrics.CallsFailed.Load(),
	}
	if ts := timestamppb.New(time.Unix(0, cm.ChannelMetrics.LastCallStartedTimestamp.Load())); ts.IsValid() {
		c.Data.LastCallStartedTimestamp = ts
	}
	ncs := cm.NestedChans()
	nestedChans := make([]*channelzpb.ChannelRef, 0, len(ncs))
	for id, ref := range ncs {
		nestedChans = append(nestedChans, &channelzpb.ChannelRef{ChannelId: id, Name: ref})
	}
	c.ChannelRef = nestedChans

	scs := cm.SubChans()
	subChans := make([]*channelzpb.SubchannelRef, 0, len(scs))
	for id, ref := range scs {
		subChans = append(subChans, &channelzpb.SubchannelRef{SubchannelId: id, Name: ref})
	}
	c.SubchannelRef = subChans

	c.Data.Trace = channelTraceToProto(cm.Trace())
	return c
}

// GetTopChannels returns the protobuf representation of the channels starting
// at startID (max of len), and returns end=true if no top channels exist with
// higher IDs.
func GetTopChannels(startID int64, len int) (channels []*channelzpb.Channel, end bool) {
	chans, end := channelz.GetTopChannels(startID, len)
	for _, ch := range chans {
		channels = append(channels, channelToProto(ch))
	}
	return channels, end
}

// GetChannel returns the protobuf representation of the channel with the given
// ID.
func GetChannel(id int64) (*channelzpb.Channel, error) {
	ch := channelz.GetChannel(id)
	if ch == nil {
		return nil, status.Errorf(codes.NotFound, "requested channel %d not found", id)
	}
	return channelToProto(ch), nil
}
