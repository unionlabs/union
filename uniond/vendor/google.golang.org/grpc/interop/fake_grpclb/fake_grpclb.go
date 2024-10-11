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

// This file is for testing only. Runs a fake grpclb balancer server.
// The name of the service to load balance for and the addresses
// of that service are provided by command line flags.
package main

import (
	"flag"
	"strings"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/alts"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/internal/testutils/fakegrpclb"
	"google.golang.org/grpc/testdata"
)

var (
	port         = flag.Int("port", 10000, "Port to listen on.")
	backendAddrs = flag.String("backend_addrs", "", "Comma separated list of backend IP/port addresses.")
	useALTS      = flag.Bool("use_alts", false, "Listen on ALTS credentials.")
	useTLS       = flag.Bool("use_tls", false, "Listen on TLS credentials, using a test certificate.")
	shortStream  = flag.Bool("short_stream", false, "End the balancer stream immediately after sending the first server list.")
	serviceName  = flag.String("service_name", "UNSET", "Name of the service being load balanced for.")

	logger = grpclog.Component("interop")
)

func main() {
	flag.Parse()

	var opts []grpc.ServerOption
	if *useTLS {
		certFile := testdata.Path("server1.pem")
		keyFile := testdata.Path("server1.key")
		creds, err := credentials.NewServerTLSFromFile(certFile, keyFile)
		if err != nil {
			logger.Fatalf("Failed to generate credentials: %v", err)
		}
		opts = append(opts, grpc.Creds(creds))
	} else if *useALTS {
		altsOpts := alts.DefaultServerOptions()
		altsTC := alts.NewServerCreds(altsOpts)
		opts = append(opts, grpc.Creds(altsTC))
	}

	rawBackendAddrs := strings.Split(*backendAddrs, ",")
	server, err := fakegrpclb.NewServer(fakegrpclb.ServerParams{
		ListenPort:              *port,
		ServerOptions:           opts,
		LoadBalancedServiceName: *serviceName,
		LoadBalancedServicePort: 443, // TODO: make this configurable?
		BackendAddresses:        rawBackendAddrs,
		ShortStream:             *shortStream,
	})
	if err != nil {
		logger.Fatalf("Failed to create balancer server: %v", err)
	}

	// Serve() starts serving and blocks until Stop() is called. We don't need to
	// call Stop() here since we want the server to run until we are killed.
	if err := server.Serve(); err != nil {
		logger.Fatalf("Failed to start balancer server: %v", err)
	}
}
