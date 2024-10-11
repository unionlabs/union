/*
 *
 * Copyright 2014 gRPC authors.
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

// Binary client is an interop client.
package main

import (
	"context"
	"flag"
	"strings"
	"sync"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/google"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/interop"

	_ "google.golang.org/grpc/balancer/grpclb"      // Register the grpclb load balancing policy.
	_ "google.golang.org/grpc/balancer/rls"         // Register the RLS load balancing policy.
	_ "google.golang.org/grpc/xds/googledirectpath" // Register xDS resolver required for c2p directpath.

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

const (
	computeEngineCredsName = "compute_engine_channel_creds"
	insecureCredsName      = "INSECURE_CREDENTIALS"
)

var (
	serverURIs                             = flag.String("server_uris", "", "Comma-separated list of sever URIs to make RPCs to")
	credentialsTypes                       = flag.String("credentials_types", "", "Comma-separated list of credentials, each entry is used for the server of the corresponding index in server_uris. Supported values: compute_engine_channel_creds, INSECURE_CREDENTIALS")
	soakIterations                         = flag.Int("soak_iterations", 10, "The number of iterations to use for the two soak tests: rpc_soak and channel_soak")
	soakMaxFailures                        = flag.Int("soak_max_failures", 0, "The number of iterations in soak tests that are allowed to fail (either due to non-OK status code or exceeding the per-iteration max acceptable latency).")
	soakPerIterationMaxAcceptableLatencyMs = flag.Int("soak_per_iteration_max_acceptable_latency_ms", 1000, "The number of milliseconds a single iteration in the two soak tests (rpc_soak and channel_soak) should take.")
	soakOverallTimeoutSeconds              = flag.Int("soak_overall_timeout_seconds", 10, "The overall number of seconds after which a soak test should stop and fail, if the desired number of iterations have not yet completed.")
	soakMinTimeMsBetweenRPCs               = flag.Int("soak_min_time_ms_between_rpcs", 0, "The minimum time in milliseconds between consecutive RPCs in a soak test (rpc_soak or channel_soak), useful for limiting QPS")
	soakRequestSize                        = flag.Int("soak_request_size", 271828, "The request size in a soak RPC. The default value is set based on the interop large unary test case.")
	soakResponseSize                       = flag.Int("soak_response_size", 314159, "The response size in a soak RPC. The default value is set based on the interop large unary test case.")
	testCase                               = flag.String("test_case", "rpc_soak",
		`Configure different test cases. Valid options are:
        rpc_soak: sends --soak_iterations large_unary RPCs;
        channel_soak: sends --soak_iterations RPCs, rebuilding the channel each time`)

	logger = grpclog.Component("interop")
)

type clientConfig struct {
	tc   testgrpc.TestServiceClient
	opts []grpc.DialOption
	uri  string
}

func main() {
	flag.Parse()
	// validate flags
	uris := strings.Split(*serverURIs, ",")
	creds := strings.Split(*credentialsTypes, ",")
	if len(uris) != len(creds) {
		logger.Fatalf("Number of entries in --server_uris (%d) != number of entries in --credentials_types (%d)", len(uris), len(creds))
	}
	for _, c := range creds {
		if c != computeEngineCredsName && c != insecureCredsName {
			logger.Fatalf("Unsupported credentials type: %v", c)
		}
	}
	var resetChannel bool
	switch *testCase {
	case "rpc_soak":
		resetChannel = false
	case "channel_soak":
		resetChannel = true
	default:
		logger.Fatal("Unsupported test case: ", *testCase)
	}

	// create clients as specified in flags
	var clients []clientConfig
	for i := range uris {
		var opts []grpc.DialOption
		switch creds[i] {
		case computeEngineCredsName:
			opts = append(opts, grpc.WithCredentialsBundle(google.NewComputeEngineCredentials()))
		case insecureCredsName:
			opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
		}
		cc, err := grpc.Dial(uris[i], opts...)
		if err != nil {
			logger.Fatalf("Fail to dial %v: %v", uris[i], err)
		}
		defer cc.Close()
		clients = append(clients, clientConfig{
			tc:   testgrpc.NewTestServiceClient(cc),
			opts: opts,
			uri:  uris[i],
		})
	}

	// run soak tests with the different clients
	logger.Infof("Clients running with test case %q", *testCase)
	var wg sync.WaitGroup
	ctx := context.Background()
	for i := range clients {
		wg.Add(1)
		go func(c clientConfig) {
			ctxWithDeadline, cancel := context.WithTimeout(ctx, time.Duration(*soakOverallTimeoutSeconds)*time.Second)
			defer cancel()
			interop.DoSoakTest(ctxWithDeadline, c.tc, c.uri, c.opts, resetChannel, *soakIterations, *soakMaxFailures, *soakRequestSize, *soakResponseSize, time.Duration(*soakPerIterationMaxAcceptableLatencyMs)*time.Millisecond, time.Duration(*soakMinTimeMsBetweenRPCs)*time.Millisecond)
			logger.Infof("%s test done for server: %s", *testCase, c.uri)
			wg.Done()
		}(clients[i])
	}
	wg.Wait()
	logger.Infoln("All clients done!")
}
