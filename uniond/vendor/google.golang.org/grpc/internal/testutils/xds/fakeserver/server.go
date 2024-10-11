/*
 *
 * Copyright 2019 gRPC authors.
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

// Package fakeserver provides a fake implementation of the management server.
//
// This package is recommended only for scenarios which cannot be tested using
// the xDS management server (which uses envoy-go-control-plane) provided by the
// `internal/testutils/xds/e2e` package.
package fakeserver

import (
	"fmt"
	"io"
	"net"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	v3discoverygrpc "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
	v3lrsgrpc "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
	v3lrspb "github.com/envoyproxy/go-control-plane/envoy/service/load_stats/v3"
)

const (
	// TODO: Make this a var or a field in the server if there is a need to use a
	// value other than this default.
	defaultChannelBufferSize = 50
	defaultDialTimeout       = 5 * time.Second
)

// Request wraps the request protobuf (xds/LRS) and error received by the
// Server in a call to stream.Recv().
type Request struct {
	Req proto.Message
	Err error
}

// Response wraps the response protobuf (xds/LRS) and error that the Server
// should send out to the client through a call to stream.Send()
type Response struct {
	Resp proto.Message
	Err  error
}

// Server is a fake implementation of xDS and LRS protocols. It listens on the
// same port for both services and exposes a bunch of channels to send/receive
// messages.
//
// This server is recommended only for scenarios which cannot be tested using
// the xDS management server (which uses envoy-go-control-plane) provided by the
// `internal/testutils/xds/e2e` package.
type Server struct {
	// XDSRequestChan is a channel on which received xDS requests are made
	// available to the users of this Server.
	XDSRequestChan *testutils.Channel
	// XDSResponseChan is a channel on which the Server accepts xDS responses
	// to be sent to the client.
	XDSResponseChan chan *Response
	// LRSRequestChan is a channel on which received LRS requests are made
	// available to the users of this Server.
	LRSRequestChan *testutils.Channel
	// LRSResponseChan is a channel on which the Server accepts the LRS
	// response to be sent to the client.
	LRSResponseChan chan *Response
	// LRSStreamOpenChan is a channel on which the Server sends notifications
	// when a new LRS stream is created.
	LRSStreamOpenChan *testutils.Channel
	// LRSStreamCloseChan is a channel on which the Server sends notifications
	// when an existing LRS stream is closed.
	LRSStreamCloseChan *testutils.Channel
	// NewConnChan is a channel on which the fake server notifies receipt of new
	// connection attempts. Tests can gate on this event before proceeding to
	// other actions which depend on a connection to the fake server being up.
	NewConnChan *testutils.Channel
	// Address is the host:port on which the Server is listening for requests.
	Address string

	// The underlying fake implementation of xDS and LRS.
	*xdsServerV3
	*lrsServerV3
}

type wrappedListener struct {
	net.Listener
	server *Server
}

func (wl *wrappedListener) Accept() (net.Conn, error) {
	c, err := wl.Listener.Accept()
	if err != nil {
		return nil, err
	}
	wl.server.NewConnChan.Send(struct{}{})
	return c, err
}

// StartServer makes a new Server and gets it to start listening on the given
// net.Listener. If the given net.Listener is nil, a new one is created on a
// local port for gRPC requests. The returned cancel function should be invoked
// by the caller upon completion of the test.
func StartServer(lis net.Listener) (*Server, func(), error) {
	if lis == nil {
		var err error
		lis, err = net.Listen("tcp", "localhost:0")
		if err != nil {
			return nil, func() {}, fmt.Errorf("net.Listen() failed: %v", err)
		}
	}

	s := NewServer(lis.Addr().String())
	wp := &wrappedListener{
		Listener: lis,
		server:   s,
	}

	server := grpc.NewServer()
	v3lrsgrpc.RegisterLoadReportingServiceServer(server, s)
	v3discoverygrpc.RegisterAggregatedDiscoveryServiceServer(server, s)
	go server.Serve(wp)

	return s, func() { server.Stop() }, nil
}

// NewServer returns a new instance of Server, set to accept requests on addr.
// It is the responsibility of the caller to register the exported ADS and LRS
// services on an appropriate gRPC server. Most usages should prefer
// StartServer() instead of this.
func NewServer(addr string) *Server {
	s := &Server{
		XDSRequestChan:     testutils.NewChannelWithSize(defaultChannelBufferSize),
		LRSRequestChan:     testutils.NewChannelWithSize(defaultChannelBufferSize),
		NewConnChan:        testutils.NewChannelWithSize(defaultChannelBufferSize),
		XDSResponseChan:    make(chan *Response, defaultChannelBufferSize),
		LRSResponseChan:    make(chan *Response, 1), // The server only ever sends one response.
		LRSStreamOpenChan:  testutils.NewChannelWithSize(defaultChannelBufferSize),
		LRSStreamCloseChan: testutils.NewChannelWithSize(defaultChannelBufferSize),
		Address:            addr,
	}
	s.xdsServerV3 = &xdsServerV3{reqChan: s.XDSRequestChan, respChan: s.XDSResponseChan}
	s.lrsServerV3 = &lrsServerV3{reqChan: s.LRSRequestChan, respChan: s.LRSResponseChan, streamOpenChan: s.LRSStreamOpenChan, streamCloseChan: s.LRSStreamCloseChan}
	return s
}

type xdsServerV3 struct {
	reqChan  *testutils.Channel
	respChan chan *Response
}

func (xdsS *xdsServerV3) StreamAggregatedResources(s v3discoverygrpc.AggregatedDiscoveryService_StreamAggregatedResourcesServer) error {
	errCh := make(chan error, 2)
	go func() {
		for {
			req, err := s.Recv()
			if err != nil {
				errCh <- err
				return
			}
			xdsS.reqChan.Send(&Request{req, err})
		}
	}()
	go func() {
		var retErr error
		defer func() {
			errCh <- retErr
		}()

		for {
			select {
			case r := <-xdsS.respChan:
				if r.Err != nil {
					retErr = r.Err
					return
				}
				if err := s.Send(r.Resp.(*v3discoverypb.DiscoveryResponse)); err != nil {
					retErr = err
					return
				}
			case <-s.Context().Done():
				retErr = s.Context().Err()
				return
			}
		}
	}()

	if err := <-errCh; err != nil {
		return err
	}
	return nil
}

func (xdsS *xdsServerV3) DeltaAggregatedResources(v3discoverygrpc.AggregatedDiscoveryService_DeltaAggregatedResourcesServer) error {
	return status.Error(codes.Unimplemented, "")
}

type lrsServerV3 struct {
	reqChan         *testutils.Channel
	respChan        chan *Response
	streamOpenChan  *testutils.Channel
	streamCloseChan *testutils.Channel
}

func (lrsS *lrsServerV3) StreamLoadStats(s v3lrsgrpc.LoadReportingService_StreamLoadStatsServer) error {
	lrsS.streamOpenChan.Send(nil)
	defer lrsS.streamCloseChan.Send(nil)

	req, err := s.Recv()
	lrsS.reqChan.Send(&Request{req, err})
	if err != nil {
		return err
	}

	select {
	case r := <-lrsS.respChan:
		if r.Err != nil {
			return r.Err
		}
		if err := s.Send(r.Resp.(*v3lrspb.LoadStatsResponse)); err != nil {
			return err
		}
	case <-s.Context().Done():
		return s.Context().Err()
	}

	for {
		req, err := s.Recv()
		lrsS.reqChan.Send(&Request{req, err})
		if err != nil {
			if err == io.EOF {
				return nil
			}
			return err
		}
	}
}
