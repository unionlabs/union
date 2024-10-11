/*
 *
 * Copyright 2016 gRPC authors.
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

// client starts an interop client to do stress test and a metrics server to report qps.
package main

import (
	"context"
	"flag"
	"fmt"
	"math/rand"
	"net"
	"os"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/google"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/interop"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/testdata"

	_ "google.golang.org/grpc/xds/googledirectpath" // Register xDS resolver required for c2p directpath.

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	metricspb "google.golang.org/grpc/interop/stress/grpc_testing"
)

const (
	googleDefaultCredsName = "google_default_credentials"
	computeEngineCredsName = "compute_engine_channel_creds"
)

var (
	serverAddresses       = flag.String("server_addresses", "localhost:8080", "a list of server addresses")
	testCases             = flag.String("test_cases", "", "a list of test cases along with the relative weights")
	testDurationSecs      = flag.Int("test_duration_secs", -1, "test duration in seconds")
	numChannelsPerServer  = flag.Int("num_channels_per_server", 1, "Number of channels (i.e connections) to each server")
	numStubsPerChannel    = flag.Int("num_stubs_per_channel", 1, "Number of client stubs per each connection to server")
	metricsPort           = flag.Int("metrics_port", 8081, "The port at which the stress client exposes QPS metrics")
	useTLS                = flag.Bool("use_tls", false, "Connection uses TLS if true, else plain TCP")
	testCA                = flag.Bool("use_test_ca", false, "Whether to replace platform root CAs with test CA as the CA root")
	tlsServerName         = flag.String("server_host_override", "foo.test.google.fr", "The server name use to verify the hostname returned by TLS handshake if it is not empty. Otherwise, --server_host is used.")
	caFile                = flag.String("ca_file", "", "The file containing the CA root cert file")
	customCredentialsType = flag.String("custom_credentials_type", "", "Custom credentials type to use")

	totalNumCalls int64
	logger        = grpclog.Component("stress")
)

// testCaseWithWeight contains the test case type and its weight.
type testCaseWithWeight struct {
	name   string
	weight int
}

// parseTestCases converts test case string to a list of struct testCaseWithWeight.
func parseTestCases(testCaseString string) []testCaseWithWeight {
	testCaseStrings := strings.Split(testCaseString, ",")
	testCases := make([]testCaseWithWeight, len(testCaseStrings))
	for i, str := range testCaseStrings {
		testCaseNameAndWeight := strings.Split(str, ":")
		if len(testCaseNameAndWeight) != 2 {
			panic(fmt.Sprintf("invalid test case with weight: %s", str))
		}
		// Check if test case is supported.
		testCaseName := strings.ToLower(testCaseNameAndWeight[0])
		switch testCaseName {
		case
			"empty_unary",
			"large_unary",
			"client_streaming",
			"server_streaming",
			"ping_pong",
			"empty_stream",
			"timeout_on_sleeping_server",
			"cancel_after_begin",
			"cancel_after_first_response",
			"status_code_and_message",
			"custom_metadata":
		default:
			panic(fmt.Sprintf("unknown test type: %s", testCaseNameAndWeight[0]))
		}
		testCases[i].name = testCaseName
		w, err := strconv.Atoi(testCaseNameAndWeight[1])
		if err != nil {
			panic(fmt.Sprintf("%v", err))
		}
		testCases[i].weight = w
	}
	return testCases
}

// weightedRandomTestSelector defines a weighted random selector for test case types.
type weightedRandomTestSelector struct {
	tests       []testCaseWithWeight
	totalWeight int
}

// newWeightedRandomTestSelector constructs a weightedRandomTestSelector with the given list of testCaseWithWeight.
func newWeightedRandomTestSelector(tests []testCaseWithWeight) *weightedRandomTestSelector {
	var totalWeight int
	for _, t := range tests {
		totalWeight += t.weight
	}
	return &weightedRandomTestSelector{tests, totalWeight}
}

func (selector weightedRandomTestSelector) getNextTest() string {
	random := rand.Intn(selector.totalWeight)
	var weightSofar int
	for _, test := range selector.tests {
		weightSofar += test.weight
		if random < weightSofar {
			return test.name
		}
	}
	panic("no test case selected by weightedRandomTestSelector")
}

// gauge stores the qps of one interop client (one stub).
type gauge struct {
	mutex sync.RWMutex
	val   int64
}

func (g *gauge) set(v int64) {
	g.mutex.Lock()
	defer g.mutex.Unlock()
	g.val = v
}

func (g *gauge) get() int64 {
	g.mutex.RLock()
	defer g.mutex.RUnlock()
	return g.val
}

// server implements metrics server functions.
type server struct {
	metricspb.UnimplementedMetricsServiceServer
	mutex sync.RWMutex
	// gauges is a map from /stress_test/server_<n>/channel_<n>/stub_<n>/qps to its qps gauge.
	gauges map[string]*gauge
}

// newMetricsServer returns a new metrics server.
func newMetricsServer() *server {
	return &server{gauges: make(map[string]*gauge)}
}

// GetAllGauges returns all gauges.
func (s *server) GetAllGauges(_ *metricspb.EmptyMessage, stream metricspb.MetricsService_GetAllGaugesServer) error {
	s.mutex.RLock()
	defer s.mutex.RUnlock()

	for name, gauge := range s.gauges {
		if err := stream.Send(&metricspb.GaugeResponse{Name: name, Value: &metricspb.GaugeResponse_LongValue{LongValue: gauge.get()}}); err != nil {
			return err
		}
	}
	return nil
}

// GetGauge returns the gauge for the given name.
func (s *server) GetGauge(_ context.Context, in *metricspb.GaugeRequest) (*metricspb.GaugeResponse, error) {
	s.mutex.RLock()
	defer s.mutex.RUnlock()

	if g, ok := s.gauges[in.Name]; ok {
		return &metricspb.GaugeResponse{Name: in.Name, Value: &metricspb.GaugeResponse_LongValue{LongValue: g.get()}}, nil
	}
	return nil, status.Errorf(codes.InvalidArgument, "gauge with name %s not found", in.Name)
}

// createGauge creates a gauge using the given name in metrics server.
func (s *server) createGauge(name string) *gauge {
	s.mutex.Lock()
	defer s.mutex.Unlock()

	if _, ok := s.gauges[name]; ok {
		// gauge already exists.
		panic(fmt.Sprintf("gauge %s already exists", name))
	}
	var g gauge
	s.gauges[name] = &g
	return &g
}

func startServer(server *server, port int) {
	lis, err := net.Listen("tcp", ":"+strconv.Itoa(port))
	if err != nil {
		logger.Fatalf("failed to listen: %v", err)
	}

	s := grpc.NewServer()
	metricspb.RegisterMetricsServiceServer(s, server)
	s.Serve(lis)
}

// performRPCs uses weightedRandomTestSelector to select test case and runs the tests.
func performRPCs(gauge *gauge, conn *grpc.ClientConn, selector *weightedRandomTestSelector, stop <-chan bool) {
	client := testgrpc.NewTestServiceClient(conn)
	var numCalls int64
	ctx := context.Background()
	startTime := time.Now()
	for {
		test := selector.getNextTest()
		switch test {
		case "empty_unary":
			interop.DoEmptyUnaryCall(ctx, client)
		case "large_unary":
			interop.DoLargeUnaryCall(ctx, client)
		case "client_streaming":
			interop.DoClientStreaming(ctx, client)
		case "server_streaming":
			interop.DoServerStreaming(ctx, client)
		case "ping_pong":
			interop.DoPingPong(ctx, client)
		case "empty_stream":
			interop.DoEmptyStream(ctx, client)
		case "timeout_on_sleeping_server":
			interop.DoTimeoutOnSleepingServer(ctx, client)
		case "cancel_after_begin":
			interop.DoCancelAfterBegin(ctx, client)
		case "cancel_after_first_response":
			interop.DoCancelAfterFirstResponse(ctx, client)
		case "status_code_and_message":
			interop.DoStatusCodeAndMessage(ctx, client)
		case "custom_metadata":
			interop.DoCustomMetadata(ctx, client)
		}
		numCalls++
		defer func() { atomic.AddInt64(&totalNumCalls, numCalls) }()
		gauge.set(int64(float64(numCalls) / time.Since(startTime).Seconds()))

		select {
		case <-stop:
			return
		default:
		}
	}
}

func logParameterInfo(addresses []string, tests []testCaseWithWeight) {
	logger.Infof("server_addresses: %s", *serverAddresses)
	logger.Infof("test_cases: %s", *testCases)
	logger.Infof("test_duration_secs: %d", *testDurationSecs)
	logger.Infof("num_channels_per_server: %d", *numChannelsPerServer)
	logger.Infof("num_stubs_per_channel: %d", *numStubsPerChannel)
	logger.Infof("metrics_port: %d", *metricsPort)
	logger.Infof("use_tls: %t", *useTLS)
	logger.Infof("use_test_ca: %t", *testCA)
	logger.Infof("server_host_override: %s", *tlsServerName)
	logger.Infof("custom_credentials_type: %s", *customCredentialsType)

	logger.Infoln("addresses:")
	for i, addr := range addresses {
		logger.Infof("%d. %s\n", i+1, addr)
	}
	logger.Infoln("tests:")
	for i, test := range tests {
		logger.Infof("%d. %v\n", i+1, test)
	}
}

func newConn(address string, useTLS, testCA bool, tlsServerName string) (*grpc.ClientConn, error) {
	var opts []grpc.DialOption
	if *customCredentialsType != "" {
		if *customCredentialsType == googleDefaultCredsName {
			opts = append(opts, grpc.WithCredentialsBundle(google.NewDefaultCredentials()))
		} else if *customCredentialsType == computeEngineCredsName {
			opts = append(opts, grpc.WithCredentialsBundle(google.NewComputeEngineCredentials()))
		} else {
			logger.Fatalf("Unknown custom credentials: %v", *customCredentialsType)
		}
	} else if useTLS {
		var sn string
		if tlsServerName != "" {
			sn = tlsServerName
		}
		var creds credentials.TransportCredentials
		if testCA {
			var err error
			if *caFile == "" {
				*caFile = testdata.Path("x509/server_ca_cert.pem")
			}
			creds, err = credentials.NewClientTLSFromFile(*caFile, sn)
			if err != nil {
				logger.Fatalf("Failed to create TLS credentials: %v", err)
			}
		} else {
			creds = credentials.NewClientTLSFromCert(nil, sn)
		}
		opts = append(opts, grpc.WithTransportCredentials(creds))
	} else {
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}
	return grpc.Dial(address, opts...)
}

func main() {
	flag.Parse()
	resolver.SetDefaultScheme("dns")
	addresses := strings.Split(*serverAddresses, ",")
	tests := parseTestCases(*testCases)
	logParameterInfo(addresses, tests)
	testSelector := newWeightedRandomTestSelector(tests)
	metricsServer := newMetricsServer()

	var wg sync.WaitGroup
	wg.Add(len(addresses) * *numChannelsPerServer * *numStubsPerChannel)
	stop := make(chan bool)

	for serverIndex, address := range addresses {
		for connIndex := 0; connIndex < *numChannelsPerServer; connIndex++ {
			conn, err := newConn(address, *useTLS, *testCA, *tlsServerName)
			if err != nil {
				logger.Fatalf("Fail to dial: %v", err)
			}
			defer conn.Close()
			for clientIndex := 0; clientIndex < *numStubsPerChannel; clientIndex++ {
				name := fmt.Sprintf("/stress_test/server_%d/channel_%d/stub_%d/qps", serverIndex+1, connIndex+1, clientIndex+1)
				go func() {
					defer wg.Done()
					g := metricsServer.createGauge(name)
					performRPCs(g, conn, testSelector, stop)
				}()
			}

		}
	}
	go startServer(metricsServer, *metricsPort)
	if *testDurationSecs > 0 {
		time.Sleep(time.Duration(*testDurationSecs) * time.Second)
		close(stop)
	}
	wg.Wait()
	fmt.Fprintf(os.Stdout, "Total calls made: %v\n", totalNumCalls)
	logger.Infof(" ===== ALL DONE ===== ")
}
