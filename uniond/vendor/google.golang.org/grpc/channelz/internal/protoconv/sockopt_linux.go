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

package protoconv

import (
	"time"

	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal/channelz"

	channelzpb "google.golang.org/grpc/channelz/grpc_channelz_v1"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
)

var logger = grpclog.Component("channelz")

func convertToPbDuration(sec int64, usec int64) *durationpb.Duration {
	return durationpb.New(time.Duration(sec*1e9 + usec*1e3))
}

func sockoptToProto(skopts *channelz.SocketOptionData) []*channelzpb.SocketOption {
	if skopts == nil {
		return nil
	}
	var opts []*channelzpb.SocketOption
	if skopts.Linger != nil {
		additional, err := anypb.New(&channelzpb.SocketOptionLinger{
			Active:   skopts.Linger.Onoff != 0,
			Duration: convertToPbDuration(int64(skopts.Linger.Linger), 0),
		})
		if err == nil {
			opts = append(opts, &channelzpb.SocketOption{
				Name:       "SO_LINGER",
				Additional: additional,
			})
		} else {
			logger.Warningf("Failed to marshal socket options linger %+v: %v", skopts.Linger, err)
		}
	}
	if skopts.RecvTimeout != nil {
		additional, err := anypb.New(&channelzpb.SocketOptionTimeout{
			Duration: convertToPbDuration(int64(skopts.RecvTimeout.Sec), int64(skopts.RecvTimeout.Usec)),
		})
		if err == nil {
			opts = append(opts, &channelzpb.SocketOption{
				Name:       "SO_RCVTIMEO",
				Additional: additional,
			})
		} else {
			logger.Warningf("Failed to marshal socket options receive timeout %+v: %v", skopts.RecvTimeout, err)
		}
	}
	if skopts.SendTimeout != nil {
		additional, err := anypb.New(&channelzpb.SocketOptionTimeout{
			Duration: convertToPbDuration(int64(skopts.SendTimeout.Sec), int64(skopts.SendTimeout.Usec)),
		})
		if err == nil {
			opts = append(opts, &channelzpb.SocketOption{
				Name:       "SO_SNDTIMEO",
				Additional: additional,
			})
		} else {
			logger.Warningf("Failed to marshal socket options send timeout %+v: %v", skopts.SendTimeout, err)
		}
	}
	if skopts.TCPInfo != nil {
		additional, err := anypb.New(&channelzpb.SocketOptionTcpInfo{
			TcpiState:       uint32(skopts.TCPInfo.State),
			TcpiCaState:     uint32(skopts.TCPInfo.Ca_state),
			TcpiRetransmits: uint32(skopts.TCPInfo.Retransmits),
			TcpiProbes:      uint32(skopts.TCPInfo.Probes),
			TcpiBackoff:     uint32(skopts.TCPInfo.Backoff),
			TcpiOptions:     uint32(skopts.TCPInfo.Options),
			// https://golang.org/pkg/syscall/#TCPInfo
			// TCPInfo struct does not contain info about TcpiSndWscale and TcpiRcvWscale.
			TcpiRto:          skopts.TCPInfo.Rto,
			TcpiAto:          skopts.TCPInfo.Ato,
			TcpiSndMss:       skopts.TCPInfo.Snd_mss,
			TcpiRcvMss:       skopts.TCPInfo.Rcv_mss,
			TcpiUnacked:      skopts.TCPInfo.Unacked,
			TcpiSacked:       skopts.TCPInfo.Sacked,
			TcpiLost:         skopts.TCPInfo.Lost,
			TcpiRetrans:      skopts.TCPInfo.Retrans,
			TcpiFackets:      skopts.TCPInfo.Fackets,
			TcpiLastDataSent: skopts.TCPInfo.Last_data_sent,
			TcpiLastAckSent:  skopts.TCPInfo.Last_ack_sent,
			TcpiLastDataRecv: skopts.TCPInfo.Last_data_recv,
			TcpiLastAckRecv:  skopts.TCPInfo.Last_ack_recv,
			TcpiPmtu:         skopts.TCPInfo.Pmtu,
			TcpiRcvSsthresh:  skopts.TCPInfo.Rcv_ssthresh,
			TcpiRtt:          skopts.TCPInfo.Rtt,
			TcpiRttvar:       skopts.TCPInfo.Rttvar,
			TcpiSndSsthresh:  skopts.TCPInfo.Snd_ssthresh,
			TcpiSndCwnd:      skopts.TCPInfo.Snd_cwnd,
			TcpiAdvmss:       skopts.TCPInfo.Advmss,
			TcpiReordering:   skopts.TCPInfo.Reordering,
		})
		if err == nil {
			opts = append(opts, &channelzpb.SocketOption{
				Name:       "TCP_INFO",
				Additional: additional,
			})
		} else {
			logger.Warningf("Failed to marshal socket options TCP info %+v: %v", skopts.TCPInfo, err)
		}
	}
	return opts
}
