/*
 *
 * Copyright 2022 gRPC authors.
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

// Package fakegrpclb provides a fake implementation of the grpclb server.
package fakegrpclb

import (
	"errors"
	"fmt"
	"io"
	"net"
	"strconv"
	"sync"
	"time"

	"google.golang.org/grpc"
	lbgrpc "google.golang.org/grpc/balancer/grpclb/grpc_lb_v1"
	lbpb "google.golang.org/grpc/balancer/grpclb/grpc_lb_v1"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/status"
)

var logger = grpclog.Component("fake_grpclb")

// ServerParams wraps options passed while creating a Server.
type ServerParams struct {
	ListenPort    int                 // Listening port for the balancer server.
	ServerOptions []grpc.ServerOption // gRPC options for the balancer server.

	LoadBalancedServiceName string   // Service name being load balanced for.
	LoadBalancedServicePort int      // Service port being load balanced for.
	BackendAddresses        []string // Service backends to balance load across.
	ShortStream             bool     // End balancer stream after sending server list.
}

// Server is a fake implementation of the grpclb LoadBalancer service. It does
// not support stats reporting from clients, and always sends back a static list
// of backends to the client to balance load across.
//
// It is safe for concurrent access.
type Server struct {
	lbgrpc.UnimplementedLoadBalancerServer

	// Options copied over from ServerParams passed to NewServer.
	sOpts       []grpc.ServerOption // gRPC server options.
	serviceName string              // Service name being load balanced for.
	servicePort int                 // Service port being load balanced for.
	shortStream bool                // End balancer stream after sending server list.

	// Values initialized using ServerParams passed to NewServer.
	backends []*lbpb.Server // Service backends to balance load across.
	lis      net.Listener   // Listener for grpc connections to the LoadBalancer service.

	// mu guards access to below fields.
	mu         sync.Mutex
	grpcServer *grpc.Server // Underlying grpc server.
	address    string       // Actual listening address.

	stopped chan struct{} // Closed when Stop() is called.
}

// NewServer creates a new Server with passed in params. Returns a non-nil error
// if the params are invalid.
func NewServer(params ServerParams) (*Server, error) {
	var servers []*lbpb.Server
	for _, addr := range params.BackendAddresses {
		ipStr, portStr, err := net.SplitHostPort(addr)
		if err != nil {
			return nil, fmt.Errorf("failed to parse list of backend address %q: %v", addr, err)
		}
		ip := net.ParseIP(ipStr)
		if ip == nil {
			return nil, fmt.Errorf("failed to parse ip: %q", ipStr)
		}
		port, err := strconv.Atoi(portStr)
		if err != nil {
			return nil, fmt.Errorf("failed to convert port %q to int", portStr)
		}
		logger.Infof("Adding backend ip: %q, port: %d to server list", ip.String(), port)
		servers = append(servers, &lbpb.Server{
			IpAddress: ip,
			Port:      int32(port),
		})
	}

	lis, err := net.Listen("tcp", "localhost:"+strconv.Itoa(params.ListenPort))
	if err != nil {
		return nil, fmt.Errorf("failed to listen on port %q: %v", params.ListenPort, err)
	}

	return &Server{
		sOpts:       params.ServerOptions,
		serviceName: params.LoadBalancedServiceName,
		servicePort: params.LoadBalancedServicePort,
		shortStream: params.ShortStream,
		backends:    servers,
		lis:         lis,
		address:     lis.Addr().String(),
		stopped:     make(chan struct{}),
	}, nil
}

// Serve starts serving the LoadBalancer service on a gRPC server.
//
// It returns early with a non-nil error if it is unable to start serving.
// Otherwise, it blocks until Stop() is called, at which point it returns the
// error returned by the underlying grpc.Server's Serve() method.
func (s *Server) Serve() error {
	s.mu.Lock()
	if s.grpcServer != nil {
		s.mu.Unlock()
		return errors.New("Serve() called multiple times")
	}

	server := grpc.NewServer(s.sOpts...)
	s.grpcServer = server
	s.mu.Unlock()

	logger.Infof("Begin listening on %s", s.lis.Addr().String())
	lbgrpc.RegisterLoadBalancerServer(server, s)
	return server.Serve(s.lis) // This call will block.
}

// Stop stops serving the LoadBalancer service and unblocks the preceding call
// to Serve().
func (s *Server) Stop() {
	defer close(s.stopped)
	s.mu.Lock()
	if s.grpcServer != nil {
		s.grpcServer.Stop()
		s.grpcServer = nil
	}
	s.mu.Unlock()
}

// Address returns the host:port on which the LoadBalancer service is serving.
func (s *Server) Address() string {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.address
}

// BalanceLoad provides a fake implementation of the LoadBalancer service.
func (s *Server) BalanceLoad(stream lbgrpc.LoadBalancer_BalanceLoadServer) error {
	logger.Info("New BalancerLoad stream started")

	req, err := stream.Recv()
	if err == io.EOF {
		logger.Warning("Received EOF when reading from the stream")
		return nil
	}
	if err != nil {
		logger.Warning("Failed to read LoadBalanceRequest from stream: %v", err)
		return err
	}
	logger.Infof("Received LoadBalancerRequest:\n%s", pretty.ToJSON(req))

	// Initial request contains the service being load balanced for.
	initialReq := req.GetInitialRequest()
	if initialReq == nil {
		logger.Info("First message on the stream does not contain an InitialLoadBalanceRequest")
		return status.Error(codes.Unknown, "First request not an InitialLoadBalanceRequest")
	}

	// Basic validation of the service name and port from the incoming request.
	//
	// Clients targeting service:port can sometimes include the ":port" suffix in
	// their requested names; handle this case.
	serviceName, port, err := net.SplitHostPort(initialReq.Name)
	if err != nil {
		// Requested name did not contain a port. So, use the name as is.
		serviceName = initialReq.Name
	} else {
		p, err := strconv.Atoi(port)
		if err != nil {
			logger.Info("Failed to parse requested service port %q to integer", port)
			return status.Error(codes.Unknown, "Bad requested service port number")
		}
		if p != s.servicePort {
			logger.Info("Requested service port number %q does not match expected", port, s.servicePort)
			return status.Error(codes.Unknown, "Bad requested service port number")
		}
	}
	if serviceName != s.serviceName {
		logger.Info("Requested service name %q does not match expected %q", serviceName, s.serviceName)
		return status.Error(codes.NotFound, "Bad requested service name")
	}

	// Empty initial response disables stats reporting from the client. Stats
	// reporting from the client is used to determine backend load and is not
	// required for the purposes of this fake.
	initResp := &lbpb.LoadBalanceResponse{
		LoadBalanceResponseType: &lbpb.LoadBalanceResponse_InitialResponse{
			InitialResponse: &lbpb.InitialLoadBalanceResponse{},
		},
	}
	if err := stream.Send(initResp); err != nil {
		logger.Warningf("Failed to send InitialLoadBalanceResponse on the stream: %v", err)
		return err
	}

	resp := &lbpb.LoadBalanceResponse{
		LoadBalanceResponseType: &lbpb.LoadBalanceResponse_ServerList{
			ServerList: &lbpb.ServerList{Servers: s.backends},
		},
	}
	logger.Infof("Sending response with server list: %s", pretty.ToJSON(resp))
	if err := stream.Send(resp); err != nil {
		logger.Warningf("Failed to send InitialLoadBalanceResponse on the stream: %v", err)
		return err
	}

	if s.shortStream {
		logger.Info("Ending stream early as the short stream option was set")
		return nil
	}

	for {
		select {
		case <-stream.Context().Done():
			return nil
		case <-s.stopped:
			return nil
		case <-time.After(10 * time.Second):
			logger.Infof("Sending response with server list: %s", pretty.ToJSON(resp))
			if err := stream.Send(resp); err != nil {
				logger.Warningf("Failed to send InitialLoadBalanceResponse on the stream: %v", err)
				return err
			}
		}
	}
}
