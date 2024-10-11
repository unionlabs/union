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

// Package service provides an implementation for channelz service server.
package service

import (
	"context"

	channelzgrpc "google.golang.org/grpc/channelz/grpc_channelz_v1"
	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"

	"google.golang.org/grpc"
	"google.golang.org/grpc/channelz/internal/protoconv"
	"google.golang.org/grpc/internal/channelz"
)

func init() {
	channelz.TurnOn()
}

// RegisterChannelzServiceToServer registers the channelz service to the given server.
//
// Note: it is preferred to use the admin API
// (https://pkg.go.dev/google.golang.org/grpc/admin#Register) instead to
// register Channelz and other administrative services.
func RegisterChannelzServiceToServer(s grpc.ServiceRegistrar) {
	channelzgrpc.RegisterChannelzServer(s, newCZServer())
}

func newCZServer() channelzgrpc.ChannelzServer {
	return &serverImpl{}
}

type serverImpl struct {
	channelzgrpc.UnimplementedChannelzServer
}

func (s *serverImpl) GetChannel(_ context.Context, req *channelzpb.GetChannelRequest) (*channelzpb.GetChannelResponse, error) {
	ch, err := protoconv.GetChannel(req.GetChannelId())
	if err != nil {
		return nil, err
	}
	return &channelzpb.GetChannelResponse{Channel: ch}, nil
}

func (s *serverImpl) GetTopChannels(_ context.Context, req *channelzpb.GetTopChannelsRequest) (*channelzpb.GetTopChannelsResponse, error) {
	resp := &channelzpb.GetTopChannelsResponse{}
	resp.Channel, resp.End = protoconv.GetTopChannels(req.GetStartChannelId(), int(req.GetMaxResults()))
	return resp, nil
}

func (s *serverImpl) GetServer(_ context.Context, req *channelzpb.GetServerRequest) (*channelzpb.GetServerResponse, error) {
	srv, err := protoconv.GetServer(req.GetServerId())
	if err != nil {
		return nil, err
	}
	return &channelzpb.GetServerResponse{Server: srv}, nil
}

func (s *serverImpl) GetServers(_ context.Context, req *channelzpb.GetServersRequest) (*channelzpb.GetServersResponse, error) {
	resp := &channelzpb.GetServersResponse{}
	resp.Server, resp.End = protoconv.GetServers(req.GetStartServerId(), int(req.GetMaxResults()))
	return resp, nil
}

func (s *serverImpl) GetSubchannel(_ context.Context, req *channelzpb.GetSubchannelRequest) (*channelzpb.GetSubchannelResponse, error) {
	subChan, err := protoconv.GetSubChannel(req.GetSubchannelId())
	if err != nil {
		return nil, err
	}
	return &channelzpb.GetSubchannelResponse{Subchannel: subChan}, nil
}

func (s *serverImpl) GetServerSockets(_ context.Context, req *channelzpb.GetServerSocketsRequest) (*channelzpb.GetServerSocketsResponse, error) {
	resp := &channelzpb.GetServerSocketsResponse{}
	resp.SocketRef, resp.End = protoconv.GetServerSockets(req.GetServerId(), req.GetStartSocketId(), int(req.GetMaxResults()))
	return resp, nil
}

func (s *serverImpl) GetSocket(_ context.Context, req *channelzpb.GetSocketRequest) (*channelzpb.GetSocketResponse, error) {
	socket, err := protoconv.GetSocket(req.GetSocketId())
	if err != nil {
		return nil, err
	}
	return &channelzpb.GetSocketResponse{Socket: socket}, nil
}
