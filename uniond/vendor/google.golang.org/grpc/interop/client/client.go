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
//
// See interop test case descriptions [here].
//
// [here]: https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md
package main

import (
	"context"
	"crypto/tls"
	"crypto/x509"
	"flag"
	"net"
	"os"
	"strconv"
	"strings"
	"time"

	"golang.org/x/oauth2"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/alts"
	"google.golang.org/grpc/credentials/google"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/credentials/oauth"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/interop"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/testdata"

	_ "google.golang.org/grpc/balancer/grpclb"      // Register the grpclb load balancing policy.
	_ "google.golang.org/grpc/balancer/rls"         // Register the RLS load balancing policy.
	_ "google.golang.org/grpc/xds/googledirectpath" // Register xDS resolver required for c2p directpath.

	testgrpc "google.golang.org/grpc/interop/grpc_testing"
)

const (
	googleDefaultCredsName = "google_default_credentials"
	computeEngineCredsName = "compute_engine_channel_creds"
)

var (
	caFile                                 = flag.String("ca_file", "", "The file containing the CA root cert file")
	useTLS                                 = flag.Bool("use_tls", false, "Connection uses TLS if true")
	useALTS                                = flag.Bool("use_alts", false, "Connection uses ALTS if true (this option can only be used on GCP)")
	customCredentialsType                  = flag.String("custom_credentials_type", "", "Custom creds to use, excluding TLS or ALTS")
	altsHSAddr                             = flag.String("alts_handshaker_service_address", "", "ALTS handshaker gRPC service address")
	testCA                                 = flag.Bool("use_test_ca", false, "Whether to replace platform root CAs with test CA as the CA root")
	serviceAccountKeyFile                  = flag.String("service_account_key_file", "", "Path to service account json key file")
	oauthScope                             = flag.String("oauth_scope", "", "The scope for OAuth2 tokens")
	defaultServiceAccount                  = flag.String("default_service_account", "", "Email of GCE default service account")
	serverHost                             = flag.String("server_host", "localhost", "The server host name")
	serverPort                             = flag.Int("server_port", 10000, "The server port number")
	serviceConfigJSON                      = flag.String("service_config_json", "", "Disables service config lookups and sets the provided string as the default service config.")
	soakIterations                         = flag.Int("soak_iterations", 10, "The number of iterations to use for the two soak tests: rpc_soak and channel_soak")
	soakMaxFailures                        = flag.Int("soak_max_failures", 0, "The number of iterations in soak tests that are allowed to fail (either due to non-OK status code or exceeding the per-iteration max acceptable latency).")
	soakPerIterationMaxAcceptableLatencyMs = flag.Int("soak_per_iteration_max_acceptable_latency_ms", 1000, "The number of milliseconds a single iteration in the two soak tests (rpc_soak and channel_soak) should take.")
	soakOverallTimeoutSeconds              = flag.Int("soak_overall_timeout_seconds", 10, "The overall number of seconds after which a soak test should stop and fail, if the desired number of iterations have not yet completed.")
	soakMinTimeMsBetweenRPCs               = flag.Int("soak_min_time_ms_between_rpcs", 0, "The minimum time in milliseconds between consecutive RPCs in a soak test (rpc_soak or channel_soak), useful for limiting QPS")
	soakRequestSize                        = flag.Int("soak_request_size", 271828, "The request size in a soak RPC. The default value is set based on the interop large unary test case.")
	soakResponseSize                       = flag.Int("soak_response_size", 314159, "The response size in a soak RPC. The default value is set based on the interop large unary test case.")
	tlsServerName                          = flag.String("server_host_override", "", "The server name used to verify the hostname returned by TLS handshake if it is not empty. Otherwise, --server_host is used.")
	additionalMetadata                     = flag.String("additional_metadata", "", "Additional metadata to send in each request, as a semicolon-separated list of key:value pairs.")
	testCase                               = flag.String("test_case", "large_unary",
		`Configure different test cases. Valid options are:
        empty_unary : empty (zero bytes) request and response;
        large_unary : single request and (large) response;
        client_streaming : request streaming with single response;
        server_streaming : single request with response streaming;
        ping_pong : full-duplex streaming;
        empty_stream : full-duplex streaming with zero message;
        timeout_on_sleeping_server: fullduplex streaming on a sleeping server;
        compute_engine_creds: large_unary with compute engine auth;
        service_account_creds: large_unary with service account auth;
        jwt_token_creds: large_unary with jwt token auth;
        per_rpc_creds: large_unary with per rpc token;
        oauth2_auth_token: large_unary with oauth2 token auth;
        google_default_credentials: large_unary with google default credentials
        compute_engine_channel_credentials: large_unary with compute engine creds
        cancel_after_begin: cancellation after metadata has been sent but before payloads are sent;
        cancel_after_first_response: cancellation after receiving 1st message from the server;
        status_code_and_message: status code propagated back to client;
        special_status_message: Unicode and whitespace is correctly processed in status message;
        custom_metadata: server will echo custom metadata;
        unimplemented_method: client attempts to call unimplemented method;
        unimplemented_service: client attempts to call unimplemented service;
        pick_first_unary: all requests are sent to one server despite multiple servers are resolved;
        orca_per_rpc: the client verifies ORCA per-RPC metrics are provided;
        orca_oob: the client verifies ORCA out-of-band metrics are provided.`)

	logger = grpclog.Component("interop")
)

type credsMode uint8

const (
	credsNone credsMode = iota
	credsTLS
	credsALTS
	credsGoogleDefaultCreds
	credsComputeEngineCreds
)

// Parses the --additional_metadata flag and returns metadata to send on each RPC,
// formatted as per https://pkg.go.dev/google.golang.org/grpc/metadata#Pairs.
// Allow any character but semicolons in values. If the flag is empty, return a nil map.
func parseAdditionalMetadataFlag() []string {
	if len(*additionalMetadata) == 0 {
		return nil
	}
	r := *additionalMetadata
	addMd := make([]string, 0)
	for len(r) > 0 {
		i := strings.Index(r, ":")
		if i < 0 {
			logger.Fatalf("Error parsing --additional_metadata flag: missing colon separator")
		}
		addMd = append(addMd, r[:i]) // append key
		r = r[i+1:]
		i = strings.Index(r, ";")
		// append value
		if i < 0 {
			addMd = append(addMd, r)
			break
		}
		addMd = append(addMd, r[:i])
		r = r[i+1:]
	}
	return addMd
}

func main() {
	flag.Parse()
	logger.Infof("Client running with test case %q", *testCase)
	var useGDC bool // use google default creds
	var useCEC bool // use compute engine creds
	if *customCredentialsType != "" {
		switch *customCredentialsType {
		case googleDefaultCredsName:
			useGDC = true
		case computeEngineCredsName:
			useCEC = true
		default:
			logger.Fatalf("If set, custom_credentials_type can only be set to one of %v or %v",
				googleDefaultCredsName, computeEngineCredsName)
		}
	}
	if (*useTLS && *useALTS) || (*useTLS && useGDC) || (*useALTS && useGDC) || (*useTLS && useCEC) || (*useALTS && useCEC) {
		logger.Fatalf("only one of TLS, ALTS, google default creds, or compute engine creds can be used")
	}

	ctx := context.Background()

	var credsChosen credsMode
	switch {
	case *useTLS:
		credsChosen = credsTLS
	case *useALTS:
		credsChosen = credsALTS
	case useGDC:
		credsChosen = credsGoogleDefaultCreds
	case useCEC:
		credsChosen = credsComputeEngineCreds
	}

	resolver.SetDefaultScheme("dns")
	serverAddr := *serverHost
	if *serverPort != 0 {
		serverAddr = net.JoinHostPort(*serverHost, strconv.Itoa(*serverPort))
	}
	var opts []grpc.DialOption
	switch credsChosen {
	case credsTLS:
		var roots *x509.CertPool
		if *testCA {
			if *caFile == "" {
				*caFile = testdata.Path("ca.pem")
			}
			b, err := os.ReadFile(*caFile)
			if err != nil {
				logger.Fatalf("Failed to read root certificate file %q: %v", *caFile, err)
			}
			roots = x509.NewCertPool()
			if !roots.AppendCertsFromPEM(b) {
				logger.Fatalf("Failed to append certificates: %s", string(b))
			}
		}
		var creds credentials.TransportCredentials
		if *tlsServerName != "" {
			creds = credentials.NewClientTLSFromCert(roots, *tlsServerName)
		} else {
			creds = credentials.NewTLS(&tls.Config{RootCAs: roots})
		}
		opts = append(opts, grpc.WithTransportCredentials(creds))
	case credsALTS:
		altsOpts := alts.DefaultClientOptions()
		if *altsHSAddr != "" {
			altsOpts.HandshakerServiceAddress = *altsHSAddr
		}
		altsTC := alts.NewClientCreds(altsOpts)
		opts = append(opts, grpc.WithTransportCredentials(altsTC))
	case credsGoogleDefaultCreds:
		opts = append(opts, grpc.WithCredentialsBundle(google.NewDefaultCredentials()))
	case credsComputeEngineCreds:
		opts = append(opts, grpc.WithCredentialsBundle(google.NewComputeEngineCredentials()))
	case credsNone:
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	default:
		logger.Fatal("Invalid creds")
	}
	if credsChosen == credsTLS {
		if *testCase == "compute_engine_creds" {
			opts = append(opts, grpc.WithPerRPCCredentials(oauth.NewComputeEngine()))
		} else if *testCase == "service_account_creds" {
			jwtCreds, err := oauth.NewServiceAccountFromFile(*serviceAccountKeyFile, *oauthScope)
			if err != nil {
				logger.Fatalf("Failed to create JWT credentials: %v", err)
			}
			opts = append(opts, grpc.WithPerRPCCredentials(jwtCreds))
		} else if *testCase == "jwt_token_creds" {
			jwtCreds, err := oauth.NewJWTAccessFromFile(*serviceAccountKeyFile)
			if err != nil {
				logger.Fatalf("Failed to create JWT credentials: %v", err)
			}
			opts = append(opts, grpc.WithPerRPCCredentials(jwtCreds))
		} else if *testCase == "oauth2_auth_token" {
			opts = append(opts, grpc.WithPerRPCCredentials(oauth.TokenSource{TokenSource: oauth2.StaticTokenSource(interop.GetToken(ctx, *serviceAccountKeyFile, *oauthScope))}))
		}
	}
	if len(*serviceConfigJSON) > 0 {
		opts = append(opts, grpc.WithDisableServiceConfig(), grpc.WithDefaultServiceConfig(*serviceConfigJSON))
	}
	if addMd := parseAdditionalMetadataFlag(); addMd != nil {
		unaryAddMd := func(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
			ctx = metadata.AppendToOutgoingContext(ctx, addMd...)
			return invoker(ctx, method, req, reply, cc, opts...)
		}
		streamingAddMd := func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
			ctx = metadata.AppendToOutgoingContext(ctx, addMd...)
			return streamer(ctx, desc, cc, method, opts...)
		}
		opts = append(opts, grpc.WithUnaryInterceptor(unaryAddMd), grpc.WithStreamInterceptor(streamingAddMd))
	}
	conn, err := grpc.Dial(serverAddr, opts...)
	if err != nil {
		logger.Fatalf("Fail to dial: %v", err)
	}
	defer conn.Close()
	tc := testgrpc.NewTestServiceClient(conn)
	ctxWithDeadline, cancel := context.WithTimeout(ctx, time.Duration(*soakOverallTimeoutSeconds)*time.Second)
	defer cancel()
	switch *testCase {
	case "empty_unary":
		interop.DoEmptyUnaryCall(ctx, tc)
		logger.Infoln("EmptyUnaryCall done")
	case "large_unary":
		interop.DoLargeUnaryCall(ctx, tc)
		logger.Infoln("LargeUnaryCall done")
	case "client_streaming":
		interop.DoClientStreaming(ctx, tc)
		logger.Infoln("ClientStreaming done")
	case "server_streaming":
		interop.DoServerStreaming(ctx, tc)
		logger.Infoln("ServerStreaming done")
	case "ping_pong":
		interop.DoPingPong(ctx, tc)
		logger.Infoln("Pingpong done")
	case "empty_stream":
		interop.DoEmptyStream(ctx, tc)
		logger.Infoln("Emptystream done")
	case "timeout_on_sleeping_server":
		interop.DoTimeoutOnSleepingServer(ctx, tc)
		logger.Infoln("TimeoutOnSleepingServer done")
	case "compute_engine_creds":
		if credsChosen != credsTLS {
			logger.Fatalf("TLS credentials need to be set for compute_engine_creds test case.")
		}
		interop.DoComputeEngineCreds(ctx, tc, *defaultServiceAccount, *oauthScope)
		logger.Infoln("ComputeEngineCreds done")
	case "service_account_creds":
		if credsChosen != credsTLS {
			logger.Fatalf("TLS credentials need to be set for service_account_creds test case.")
		}
		interop.DoServiceAccountCreds(ctx, tc, *serviceAccountKeyFile, *oauthScope)
		logger.Infoln("ServiceAccountCreds done")
	case "jwt_token_creds":
		if credsChosen != credsTLS {
			logger.Fatalf("TLS credentials need to be set for jwt_token_creds test case.")
		}
		interop.DoJWTTokenCreds(ctx, tc, *serviceAccountKeyFile)
		logger.Infoln("JWTtokenCreds done")
	case "per_rpc_creds":
		if credsChosen != credsTLS {
			logger.Fatalf("TLS credentials need to be set for per_rpc_creds test case.")
		}
		interop.DoPerRPCCreds(ctx, tc, *serviceAccountKeyFile, *oauthScope)
		logger.Infoln("PerRPCCreds done")
	case "oauth2_auth_token":
		if credsChosen != credsTLS {
			logger.Fatalf("TLS credentials need to be set for oauth2_auth_token test case.")
		}
		interop.DoOauth2TokenCreds(ctx, tc, *serviceAccountKeyFile, *oauthScope)
		logger.Infoln("Oauth2TokenCreds done")
	case "google_default_credentials":
		if credsChosen != credsGoogleDefaultCreds {
			logger.Fatalf("GoogleDefaultCredentials need to be set for google_default_credentials test case.")
		}
		interop.DoGoogleDefaultCredentials(ctx, tc, *defaultServiceAccount)
		logger.Infoln("GoogleDefaultCredentials done")
	case "compute_engine_channel_credentials":
		if credsChosen != credsComputeEngineCreds {
			logger.Fatalf("ComputeEngineCreds need to be set for compute_engine_channel_credentials test case.")
		}
		interop.DoComputeEngineChannelCredentials(ctx, tc, *defaultServiceAccount)
		logger.Infoln("ComputeEngineChannelCredentials done")
	case "cancel_after_begin":
		interop.DoCancelAfterBegin(ctx, tc)
		logger.Infoln("CancelAfterBegin done")
	case "cancel_after_first_response":
		interop.DoCancelAfterFirstResponse(ctx, tc)
		logger.Infoln("CancelAfterFirstResponse done")
	case "status_code_and_message":
		interop.DoStatusCodeAndMessage(ctx, tc)
		logger.Infoln("StatusCodeAndMessage done")
	case "special_status_message":
		interop.DoSpecialStatusMessage(ctx, tc)
		logger.Infoln("SpecialStatusMessage done")
	case "custom_metadata":
		interop.DoCustomMetadata(ctx, tc)
		logger.Infoln("CustomMetadata done")
	case "unimplemented_method":
		interop.DoUnimplementedMethod(ctx, conn)
		logger.Infoln("UnimplementedMethod done")
	case "unimplemented_service":
		interop.DoUnimplementedService(ctx, testgrpc.NewUnimplementedServiceClient(conn))
		logger.Infoln("UnimplementedService done")
	case "pick_first_unary":
		interop.DoPickFirstUnary(ctx, tc)
		logger.Infoln("PickFirstUnary done")
	case "rpc_soak":
		interop.DoSoakTest(ctxWithDeadline, tc, serverAddr, opts, false /* resetChannel */, *soakIterations, *soakMaxFailures, *soakRequestSize, *soakResponseSize, time.Duration(*soakPerIterationMaxAcceptableLatencyMs)*time.Millisecond, time.Duration(*soakMinTimeMsBetweenRPCs)*time.Millisecond)
		logger.Infoln("RpcSoak done")
	case "channel_soak":
		interop.DoSoakTest(ctxWithDeadline, tc, serverAddr, opts, true /* resetChannel */, *soakIterations, *soakMaxFailures, *soakRequestSize, *soakResponseSize, time.Duration(*soakPerIterationMaxAcceptableLatencyMs)*time.Millisecond, time.Duration(*soakMinTimeMsBetweenRPCs)*time.Millisecond)
		logger.Infoln("ChannelSoak done")
	case "orca_per_rpc":
		interop.DoORCAPerRPCTest(ctx, tc)
		logger.Infoln("ORCAPerRPC done")
	case "orca_oob":
		interop.DoORCAOOBTest(ctx, tc)
		logger.Infoln("ORCAOOB done")
	default:
		logger.Fatal("Unsupported test case: ", *testCase)
	}
}
