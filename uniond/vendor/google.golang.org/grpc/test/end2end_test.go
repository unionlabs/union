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

package test

import (
	"bufio"
	"bytes"
	"context"
	"crypto/tls"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"math"
	"net"
	"net/http"
	"os"
	"reflect"
	"runtime"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"testing"
	"time"

	"golang.org/x/net/http2"
	"golang.org/x/net/http2/hpack"
	"google.golang.org/grpc"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/roundrobin"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/health"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/binarylog"
	"google.golang.org/grpc/internal/channelz"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/internal/transport"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/peer"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
	"google.golang.org/grpc/serviceconfig"
	"google.golang.org/grpc/stats"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/tap"
	"google.golang.org/grpc/test/bufconn"
	"google.golang.org/grpc/testdata"

	spb "google.golang.org/genproto/googleapis/rpc/status"
	healthgrpc "google.golang.org/grpc/health/grpc_health_v1"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"

	_ "google.golang.org/grpc/encoding/gzip"
)

const defaultHealthService = "grpc.health.v1.Health"

func init() {
	channelz.TurnOn()
	balancer.Register(triggerRPCBlockPickerBalancerBuilder{})
}

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

var (
	// For headers:
	testMetadata = metadata.MD{
		"key1":     []string{"value1"},
		"key2":     []string{"value2"},
		"key3-bin": []string{"binvalue1", string([]byte{1, 2, 3})},
	}
	testMetadata2 = metadata.MD{
		"key1": []string{"value12"},
		"key2": []string{"value22"},
	}
	// For trailers:
	testTrailerMetadata = metadata.MD{
		"tkey1":     []string{"trailerValue1"},
		"tkey2":     []string{"trailerValue2"},
		"tkey3-bin": []string{"trailerbinvalue1", string([]byte{3, 2, 1})},
	}
	testTrailerMetadata2 = metadata.MD{
		"tkey1": []string{"trailerValue12"},
		"tkey2": []string{"trailerValue22"},
	}
	// capital "Key" is illegal in HTTP/2.
	malformedHTTP2Metadata = metadata.MD{
		"Key": []string{"foo"},
	}
	testAppUA     = "myApp1/1.0 myApp2/0.9"
	failAppUA     = "fail-this-RPC"
	detailedError = status.ErrorProto(&spb.Status{
		Code:    int32(codes.DataLoss),
		Message: "error for testing: " + failAppUA,
		Details: []*anypb.Any{{
			TypeUrl: "url",
			Value:   []byte{6, 0, 0, 6, 1, 3},
		}},
	})
)

var raceMode bool // set by race.go in race mode

type testServer struct {
	testgrpc.UnimplementedTestServiceServer

	security           string // indicate the authentication protocol used by this server.
	earlyFail          bool   // whether to error out the execution of a service handler prematurely.
	setAndSendHeader   bool   // whether to call setHeader and sendHeader.
	setHeaderOnly      bool   // whether to only call setHeader, not sendHeader.
	multipleSetTrailer bool   // whether to call setTrailer multiple times.
	unaryCallSleepTime time.Duration
}

func (s *testServer) EmptyCall(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
	if md, ok := metadata.FromIncomingContext(ctx); ok {
		// For testing purpose, returns an error if user-agent is failAppUA.
		// To test that client gets the correct error.
		if ua, ok := md["user-agent"]; !ok || strings.HasPrefix(ua[0], failAppUA) {
			return nil, detailedError
		}
		var str []string
		for _, entry := range md["user-agent"] {
			str = append(str, "ua", entry)
		}
		grpc.SendHeader(ctx, metadata.Pairs(str...))
	}
	return new(testpb.Empty), nil
}

func newPayload(t testpb.PayloadType, size int32) (*testpb.Payload, error) {
	if size < 0 {
		return nil, fmt.Errorf("requested a response with invalid length %d", size)
	}
	body := make([]byte, size)
	switch t {
	case testpb.PayloadType_COMPRESSABLE:
	default:
		return nil, fmt.Errorf("unsupported payload type: %d", t)
	}
	return &testpb.Payload{
		Type: t,
		Body: body,
	}, nil
}

func (s *testServer) UnaryCall(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if ok {
		if _, exists := md[":authority"]; !exists {
			return nil, status.Errorf(codes.DataLoss, "expected an :authority metadata: %v", md)
		}
		if s.setAndSendHeader {
			if err := grpc.SetHeader(ctx, md); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SetHeader(_, %v) = %v, want <nil>", md, err)
			}
			if err := grpc.SendHeader(ctx, testMetadata2); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SendHeader(_, %v) = %v, want <nil>", testMetadata2, err)
			}
		} else if s.setHeaderOnly {
			if err := grpc.SetHeader(ctx, md); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SetHeader(_, %v) = %v, want <nil>", md, err)
			}
			if err := grpc.SetHeader(ctx, testMetadata2); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SetHeader(_, %v) = %v, want <nil>", testMetadata2, err)
			}
		} else {
			if err := grpc.SendHeader(ctx, md); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SendHeader(_, %v) = %v, want <nil>", md, err)
			}
		}
		if err := grpc.SetTrailer(ctx, testTrailerMetadata); err != nil {
			return nil, status.Errorf(status.Code(err), "grpc.SetTrailer(_, %v) = %v, want <nil>", testTrailerMetadata, err)
		}
		if s.multipleSetTrailer {
			if err := grpc.SetTrailer(ctx, testTrailerMetadata2); err != nil {
				return nil, status.Errorf(status.Code(err), "grpc.SetTrailer(_, %v) = %v, want <nil>", testTrailerMetadata2, err)
			}
		}
	}
	pr, ok := peer.FromContext(ctx)
	if !ok {
		return nil, status.Error(codes.DataLoss, "failed to get peer from ctx")
	}
	if pr.Addr == net.Addr(nil) {
		return nil, status.Error(codes.DataLoss, "failed to get peer address")
	}
	if s.security != "" {
		// Check Auth info
		var authType, serverName string
		switch info := pr.AuthInfo.(type) {
		case credentials.TLSInfo:
			authType = info.AuthType()
			serverName = info.State.ServerName
		default:
			return nil, status.Error(codes.Unauthenticated, "Unknown AuthInfo type")
		}
		if authType != s.security {
			return nil, status.Errorf(codes.Unauthenticated, "Wrong auth type: got %q, want %q", authType, s.security)
		}
		if serverName != "x.test.example.com" {
			return nil, status.Errorf(codes.Unauthenticated, "Unknown server name %q", serverName)
		}
	}
	// Simulate some service delay.
	time.Sleep(s.unaryCallSleepTime)

	payload, err := newPayload(in.GetResponseType(), in.GetResponseSize())
	if err != nil {
		return nil, err
	}

	return &testpb.SimpleResponse{
		Payload: payload,
	}, nil
}

func (s *testServer) StreamingOutputCall(args *testpb.StreamingOutputCallRequest, stream testgrpc.TestService_StreamingOutputCallServer) error {
	if md, ok := metadata.FromIncomingContext(stream.Context()); ok {
		if _, exists := md[":authority"]; !exists {
			return status.Errorf(codes.DataLoss, "expected an :authority metadata: %v", md)
		}
		// For testing purpose, returns an error if user-agent is failAppUA.
		// To test that client gets the correct error.
		if ua, ok := md["user-agent"]; !ok || strings.HasPrefix(ua[0], failAppUA) {
			return status.Error(codes.DataLoss, "error for testing: "+failAppUA)
		}
	}
	cs := args.GetResponseParameters()
	for _, c := range cs {
		if us := c.GetIntervalUs(); us > 0 {
			time.Sleep(time.Duration(us) * time.Microsecond)
		}

		payload, err := newPayload(args.GetResponseType(), c.GetSize())
		if err != nil {
			return err
		}

		if err := stream.Send(&testpb.StreamingOutputCallResponse{
			Payload: payload,
		}); err != nil {
			return err
		}
	}
	return nil
}

func (s *testServer) StreamingInputCall(stream testgrpc.TestService_StreamingInputCallServer) error {
	var sum int
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			return stream.SendAndClose(&testpb.StreamingInputCallResponse{
				AggregatedPayloadSize: int32(sum),
			})
		}
		if err != nil {
			return err
		}
		p := in.GetPayload().GetBody()
		sum += len(p)
		if s.earlyFail {
			return status.Error(codes.NotFound, "not found")
		}
	}
}

func (s *testServer) FullDuplexCall(stream testgrpc.TestService_FullDuplexCallServer) error {
	md, ok := metadata.FromIncomingContext(stream.Context())
	if ok {
		if s.setAndSendHeader {
			if err := stream.SetHeader(md); err != nil {
				return status.Errorf(status.Code(err), "%v.SetHeader(_, %v) = %v, want <nil>", stream, md, err)
			}
			if err := stream.SendHeader(testMetadata2); err != nil {
				return status.Errorf(status.Code(err), "%v.SendHeader(_, %v) = %v, want <nil>", stream, testMetadata2, err)
			}
		} else if s.setHeaderOnly {
			if err := stream.SetHeader(md); err != nil {
				return status.Errorf(status.Code(err), "%v.SetHeader(_, %v) = %v, want <nil>", stream, md, err)
			}
			if err := stream.SetHeader(testMetadata2); err != nil {
				return status.Errorf(status.Code(err), "%v.SetHeader(_, %v) = %v, want <nil>", stream, testMetadata2, err)
			}
		} else {
			if err := stream.SendHeader(md); err != nil {
				return status.Errorf(status.Code(err), "%v.SendHeader(%v) = %v, want %v", stream, md, err, nil)
			}
		}
		stream.SetTrailer(testTrailerMetadata)
		if s.multipleSetTrailer {
			stream.SetTrailer(testTrailerMetadata2)
		}
	}
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			return nil
		}
		if err != nil {
			// to facilitate testSvrWriteStatusEarlyWrite
			if status.Code(err) == codes.ResourceExhausted {
				return status.Errorf(codes.Internal, "fake error for test testSvrWriteStatusEarlyWrite. true error: %s", err.Error())
			}
			return err
		}
		cs := in.GetResponseParameters()
		for _, c := range cs {
			if us := c.GetIntervalUs(); us > 0 {
				time.Sleep(time.Duration(us) * time.Microsecond)
			}

			payload, err := newPayload(in.GetResponseType(), c.GetSize())
			if err != nil {
				return err
			}

			if err := stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: payload,
			}); err != nil {
				// to facilitate testSvrWriteStatusEarlyWrite
				if status.Code(err) == codes.ResourceExhausted {
					return status.Errorf(codes.Internal, "fake error for test testSvrWriteStatusEarlyWrite. true error: %s", err.Error())
				}
				return err
			}
		}
	}
}

func (s *testServer) HalfDuplexCall(stream testgrpc.TestService_HalfDuplexCallServer) error {
	var msgBuf []*testpb.StreamingOutputCallRequest
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			break
		}
		if err != nil {
			return err
		}
		msgBuf = append(msgBuf, in)
	}
	for _, m := range msgBuf {
		cs := m.GetResponseParameters()
		for _, c := range cs {
			if us := c.GetIntervalUs(); us > 0 {
				time.Sleep(time.Duration(us) * time.Microsecond)
			}

			payload, err := newPayload(m.GetResponseType(), c.GetSize())
			if err != nil {
				return err
			}

			if err := stream.Send(&testpb.StreamingOutputCallResponse{
				Payload: payload,
			}); err != nil {
				return err
			}
		}
	}
	return nil
}

type env struct {
	name         string
	network      string // The type of network such as tcp, unix, etc.
	security     string // The security protocol such as TLS, SSH, etc.
	httpHandler  bool   // whether to use the http.Handler ServerTransport; requires TLS
	balancer     string // One of "round_robin", "pick_first", or "".
	customDialer func(string, string, time.Duration) (net.Conn, error)
}

func (e env) runnable() bool {
	if runtime.GOOS == "windows" && e.network == "unix" {
		return false
	}
	return true
}

func (e env) dialer(addr string, timeout time.Duration) (net.Conn, error) {
	if e.customDialer != nil {
		return e.customDialer(e.network, addr, timeout)
	}
	return net.DialTimeout(e.network, addr, timeout)
}

var (
	tcpClearEnv   = env{name: "tcp-clear-v1-balancer", network: "tcp"}
	tcpTLSEnv     = env{name: "tcp-tls-v1-balancer", network: "tcp", security: "tls"}
	tcpClearRREnv = env{name: "tcp-clear", network: "tcp", balancer: "round_robin"}
	tcpTLSRREnv   = env{name: "tcp-tls", network: "tcp", security: "tls", balancer: "round_robin"}
	handlerEnv    = env{name: "handler-tls", network: "tcp", security: "tls", httpHandler: true, balancer: "round_robin"}
	noBalancerEnv = env{name: "no-balancer", network: "tcp", security: "tls"}
	allEnv        = []env{tcpClearEnv, tcpTLSEnv, tcpClearRREnv, tcpTLSRREnv, handlerEnv, noBalancerEnv}
)

var onlyEnv = flag.String("only_env", "", "If non-empty, one of 'tcp-clear', 'tcp-tls', 'unix-clear', 'unix-tls', or 'handler-tls' to only run the tests for that environment. Empty means all.")

func listTestEnv() (envs []env) {
	if *onlyEnv != "" {
		for _, e := range allEnv {
			if e.name == *onlyEnv {
				if !e.runnable() {
					panic(fmt.Sprintf("--only_env environment %q does not run on %s", *onlyEnv, runtime.GOOS))
				}
				return []env{e}
			}
		}
		panic(fmt.Sprintf("invalid --only_env value %q", *onlyEnv))
	}
	for _, e := range allEnv {
		if e.runnable() {
			envs = append(envs, e)
		}
	}
	return envs
}

// test is an end-to-end test. It should be created with the newTest
// func, modified as needed, and then started with its startServer method.
// It should be cleaned up with the tearDown method.
type test struct {
	// The following are setup in newTest().
	t      *testing.T
	e      env
	ctx    context.Context // valid for life of test, before tearDown
	cancel context.CancelFunc

	// The following knobs are for the server-side, and should be set after
	// calling newTest() and before calling startServer().

	// whether or not to expose the server's health via the default health
	// service implementation.
	enableHealthServer bool
	// In almost all cases, one should set the 'enableHealthServer' flag above to
	// expose the server's health using the default health service
	// implementation. This should only be used when a non-default health service
	// implementation is required.
	healthServer            healthgrpc.HealthServer
	maxStream               uint32
	tapHandle               tap.ServerInHandle
	maxServerMsgSize        *int
	maxServerReceiveMsgSize *int
	maxServerSendMsgSize    *int
	maxServerHeaderListSize *uint32
	// Used to test the deprecated API WithCompressor and WithDecompressor.
	serverCompression           bool
	unknownHandler              grpc.StreamHandler
	unaryServerInt              grpc.UnaryServerInterceptor
	streamServerInt             grpc.StreamServerInterceptor
	serverInitialWindowSize     int32
	serverInitialConnWindowSize int32
	customServerOptions         []grpc.ServerOption

	// The following knobs are for the client-side, and should be set after
	// calling newTest() and before calling clientConn().
	maxClientMsgSize        *int
	maxClientReceiveMsgSize *int
	maxClientSendMsgSize    *int
	maxClientHeaderListSize *uint32
	userAgent               string
	// Used to test the deprecated API WithCompressor and WithDecompressor.
	clientCompression bool
	// Used to test the new compressor registration API UseCompressor.
	clientUseCompression bool
	// clientNopCompression is set to create a compressor whose type is not supported.
	clientNopCompression        bool
	unaryClientInt              grpc.UnaryClientInterceptor
	streamClientInt             grpc.StreamClientInterceptor
	clientInitialWindowSize     int32
	clientInitialConnWindowSize int32
	perRPCCreds                 credentials.PerRPCCredentials
	customDialOptions           []grpc.DialOption
	resolverScheme              string

	// These are set once startServer is called. The common case is to have
	// only one testServer.
	srv     stopper
	hSrv    healthgrpc.HealthServer
	srvAddr string

	// These are set once startServers is called.
	srvs     []stopper
	hSrvs    []healthgrpc.HealthServer
	srvAddrs []string

	cc          *grpc.ClientConn // nil until requested via clientConn
	restoreLogs func()           // nil unless declareLogNoise is used
}

type stopper interface {
	Stop()
	GracefulStop()
}

func (te *test) tearDown() {
	if te.cancel != nil {
		te.cancel()
		te.cancel = nil
	}

	if te.cc != nil {
		te.cc.Close()
		te.cc = nil
	}

	if te.restoreLogs != nil {
		te.restoreLogs()
		te.restoreLogs = nil
	}

	if te.srv != nil {
		te.srv.Stop()
	}
	for _, s := range te.srvs {
		s.Stop()
	}
}

// newTest returns a new test using the provided testing.T and
// environment.  It is returned with default values. Tests should
// modify it before calling its startServer and clientConn methods.
func newTest(t *testing.T, e env) *test {
	te := &test{
		t:         t,
		e:         e,
		maxStream: math.MaxUint32,
	}
	te.ctx, te.cancel = context.WithTimeout(context.Background(), defaultTestTimeout)
	return te
}

func (te *test) listenAndServe(ts testgrpc.TestServiceServer, listen func(network, address string) (net.Listener, error)) net.Listener {
	te.t.Helper()
	te.t.Logf("Running test in %s environment...", te.e.name)
	sopts := []grpc.ServerOption{grpc.MaxConcurrentStreams(te.maxStream)}
	if te.maxServerMsgSize != nil {
		sopts = append(sopts, grpc.MaxMsgSize(*te.maxServerMsgSize))
	}
	if te.maxServerReceiveMsgSize != nil {
		sopts = append(sopts, grpc.MaxRecvMsgSize(*te.maxServerReceiveMsgSize))
	}
	if te.maxServerSendMsgSize != nil {
		sopts = append(sopts, grpc.MaxSendMsgSize(*te.maxServerSendMsgSize))
	}
	if te.maxServerHeaderListSize != nil {
		sopts = append(sopts, grpc.MaxHeaderListSize(*te.maxServerHeaderListSize))
	}
	if te.tapHandle != nil {
		sopts = append(sopts, grpc.InTapHandle(te.tapHandle))
	}
	if te.serverCompression {
		sopts = append(sopts,
			grpc.RPCCompressor(grpc.NewGZIPCompressor()),
			grpc.RPCDecompressor(grpc.NewGZIPDecompressor()),
		)
	}
	if te.unaryServerInt != nil {
		sopts = append(sopts, grpc.UnaryInterceptor(te.unaryServerInt))
	}
	if te.streamServerInt != nil {
		sopts = append(sopts, grpc.StreamInterceptor(te.streamServerInt))
	}
	if te.unknownHandler != nil {
		sopts = append(sopts, grpc.UnknownServiceHandler(te.unknownHandler))
	}
	if te.serverInitialWindowSize > 0 {
		sopts = append(sopts, grpc.InitialWindowSize(te.serverInitialWindowSize))
	}
	if te.serverInitialConnWindowSize > 0 {
		sopts = append(sopts, grpc.InitialConnWindowSize(te.serverInitialConnWindowSize))
	}
	la := "localhost:0"
	switch te.e.network {
	case "unix":
		la = "/tmp/testsock" + fmt.Sprintf("%d", time.Now().UnixNano())
		syscall.Unlink(la)
	}
	lis, err := listen(te.e.network, la)
	if err != nil {
		te.t.Fatalf("Failed to listen: %v", err)
	}
	if te.e.security == "tls" {
		creds, err := credentials.NewServerTLSFromFile(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
		if err != nil {
			te.t.Fatalf("Failed to generate credentials %v", err)
		}
		sopts = append(sopts, grpc.Creds(creds))
	}
	sopts = append(sopts, te.customServerOptions...)
	s := grpc.NewServer(sopts...)
	if ts != nil {
		testgrpc.RegisterTestServiceServer(s, ts)
	}

	// Create a new default health server if enableHealthServer is set, or use
	// the provided one.
	hs := te.healthServer
	if te.enableHealthServer {
		hs = health.NewServer()
	}
	if hs != nil {
		healthgrpc.RegisterHealthServer(s, hs)
	}

	addr := la
	switch te.e.network {
	case "unix":
	default:
		_, port, err := net.SplitHostPort(lis.Addr().String())
		if err != nil {
			te.t.Fatalf("Failed to parse listener address: %v", err)
		}
		addr = "localhost:" + port
	}

	te.srv = s
	te.hSrv = hs
	te.srvAddr = addr

	if te.e.httpHandler {
		if te.e.security != "tls" {
			te.t.Fatalf("unsupported environment settings")
		}
		cert, err := tls.LoadX509KeyPair(testdata.Path("x509/server1_cert.pem"), testdata.Path("x509/server1_key.pem"))
		if err != nil {
			te.t.Fatal("tls.LoadX509KeyPair(server1.pem, server1.key) failed: ", err)
		}
		hs := &http.Server{
			Handler:   s,
			TLSConfig: &tls.Config{Certificates: []tls.Certificate{cert}},
		}
		if err := http2.ConfigureServer(hs, &http2.Server{MaxConcurrentStreams: te.maxStream}); err != nil {
			te.t.Fatal("http2.ConfigureServer(_, _) failed: ", err)
		}
		te.srv = wrapHS{hs}
		tlsListener := tls.NewListener(lis, hs.TLSConfig)
		go hs.Serve(tlsListener)
		return lis
	}

	go s.Serve(lis)
	return lis
}

type wrapHS struct {
	s *http.Server
}

func (w wrapHS) GracefulStop() {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	w.s.Shutdown(ctx)
}

func (w wrapHS) Stop() {
	w.s.Close()
	w.s.Handler.(*grpc.Server).Stop()
}

func (te *test) startServerWithConnControl(ts testgrpc.TestServiceServer) *listenerWrapper {
	l := te.listenAndServe(ts, listenWithConnControl)
	return l.(*listenerWrapper)
}

// startServer starts a gRPC server exposing the provided TestService
// implementation. Callers should defer a call to te.tearDown to clean up
func (te *test) startServer(ts testgrpc.TestServiceServer) {
	te.t.Helper()
	te.listenAndServe(ts, net.Listen)
}

// startServers starts 'num' gRPC servers exposing the provided TestService.
func (te *test) startServers(ts testgrpc.TestServiceServer, num int) {
	for i := 0; i < num; i++ {
		te.startServer(ts)
		te.srvs = append(te.srvs, te.srv.(*grpc.Server))
		te.hSrvs = append(te.hSrvs, te.hSrv)
		te.srvAddrs = append(te.srvAddrs, te.srvAddr)
		te.srv = nil
		te.hSrv = nil
		te.srvAddr = ""
	}
}

// setHealthServingStatus is a helper function to set the health status.
func (te *test) setHealthServingStatus(service string, status healthpb.HealthCheckResponse_ServingStatus) {
	hs, ok := te.hSrv.(*health.Server)
	if !ok {
		panic(fmt.Sprintf("SetServingStatus(%v, %v) called for health server of type %T", service, status, hs))
	}
	hs.SetServingStatus(service, status)
}

type nopCompressor struct {
	grpc.Compressor
}

// newNopCompressor creates a compressor to test the case that type is not supported.
func newNopCompressor() grpc.Compressor {
	return &nopCompressor{grpc.NewGZIPCompressor()}
}

func (c *nopCompressor) Type() string {
	return "nop"
}

type nopDecompressor struct {
	grpc.Decompressor
}

// newNopDecompressor creates a decompressor to test the case that type is not supported.
func newNopDecompressor() grpc.Decompressor {
	return &nopDecompressor{grpc.NewGZIPDecompressor()}
}

func (d *nopDecompressor) Type() string {
	return "nop"
}

func (te *test) configDial(opts ...grpc.DialOption) ([]grpc.DialOption, string) {
	opts = append(opts, grpc.WithDialer(te.e.dialer), grpc.WithUserAgent(te.userAgent))

	if te.clientCompression {
		opts = append(opts,
			grpc.WithCompressor(grpc.NewGZIPCompressor()),
			grpc.WithDecompressor(grpc.NewGZIPDecompressor()),
		)
	}
	if te.clientUseCompression {
		opts = append(opts, grpc.WithDefaultCallOptions(grpc.UseCompressor("gzip")))
	}
	if te.clientNopCompression {
		opts = append(opts,
			grpc.WithCompressor(newNopCompressor()),
			grpc.WithDecompressor(newNopDecompressor()),
		)
	}
	if te.unaryClientInt != nil {
		opts = append(opts, grpc.WithUnaryInterceptor(te.unaryClientInt))
	}
	if te.streamClientInt != nil {
		opts = append(opts, grpc.WithStreamInterceptor(te.streamClientInt))
	}
	if te.maxClientMsgSize != nil {
		opts = append(opts, grpc.WithMaxMsgSize(*te.maxClientMsgSize))
	}
	if te.maxClientReceiveMsgSize != nil {
		opts = append(opts, grpc.WithDefaultCallOptions(grpc.MaxCallRecvMsgSize(*te.maxClientReceiveMsgSize)))
	}
	if te.maxClientSendMsgSize != nil {
		opts = append(opts, grpc.WithDefaultCallOptions(grpc.MaxCallSendMsgSize(*te.maxClientSendMsgSize)))
	}
	if te.maxClientHeaderListSize != nil {
		opts = append(opts, grpc.WithMaxHeaderListSize(*te.maxClientHeaderListSize))
	}
	switch te.e.security {
	case "tls":
		creds, err := credentials.NewClientTLSFromFile(testdata.Path("x509/server_ca_cert.pem"), "x.test.example.com")
		if err != nil {
			te.t.Fatalf("Failed to load credentials: %v", err)
		}
		opts = append(opts, grpc.WithTransportCredentials(creds))
	case "empty":
		// Don't add any transport creds option.
	default:
		opts = append(opts, grpc.WithTransportCredentials(insecure.NewCredentials()))
	}
	// TODO(bar) switch balancer case "pick_first".
	var scheme string
	if te.resolverScheme == "" {
		scheme = "passthrough:///"
	} else {
		scheme = te.resolverScheme + ":///"
	}
	if te.e.balancer != "" {
		opts = append(opts, grpc.WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, te.e.balancer)))
	}
	if te.clientInitialWindowSize > 0 {
		opts = append(opts, grpc.WithInitialWindowSize(te.clientInitialWindowSize))
	}
	if te.clientInitialConnWindowSize > 0 {
		opts = append(opts, grpc.WithInitialConnWindowSize(te.clientInitialConnWindowSize))
	}
	if te.perRPCCreds != nil {
		opts = append(opts, grpc.WithPerRPCCredentials(te.perRPCCreds))
	}
	if te.srvAddr == "" {
		te.srvAddr = "client.side.only.test"
	}
	opts = append(opts, te.customDialOptions...)
	return opts, scheme
}

func (te *test) clientConnWithConnControl() (*grpc.ClientConn, *dialerWrapper) {
	if te.cc != nil {
		return te.cc, nil
	}
	opts, scheme := te.configDial()
	dw := &dialerWrapper{}
	// overwrite the dialer before
	opts = append(opts, grpc.WithDialer(dw.dialer))
	var err error
	te.cc, err = grpc.Dial(scheme+te.srvAddr, opts...)
	if err != nil {
		te.t.Fatalf("Dial(%q) = %v", scheme+te.srvAddr, err)
	}
	return te.cc, dw
}

func (te *test) clientConn(opts ...grpc.DialOption) *grpc.ClientConn {
	if te.cc != nil {
		return te.cc
	}
	var scheme string
	opts, scheme = te.configDial(opts...)
	var err error
	te.cc, err = grpc.Dial(scheme+te.srvAddr, opts...)
	if err != nil {
		te.t.Fatalf("Dial(%q) = %v", scheme+te.srvAddr, err)
	}
	return te.cc
}

func (te *test) declareLogNoise(phrases ...string) {
	te.restoreLogs = declareLogNoise(te.t, phrases...)
}

func (te *test) withServerTester(fn func(st *serverTester)) {
	c, err := te.e.dialer(te.srvAddr, 10*time.Second)
	if err != nil {
		te.t.Fatal(err)
	}
	defer c.Close()
	if te.e.security == "tls" {
		c = tls.Client(c, &tls.Config{
			InsecureSkipVerify: true,
			NextProtos:         []string{http2.NextProtoTLS},
		})
	}
	st := newServerTesterFromConn(te.t, c)
	st.greet()
	fn(st)
}

type lazyConn struct {
	net.Conn
	beLazy int32
}

// possible conn closed errors.
const possibleConnResetMsg = "connection reset by peer"
const possibleEOFMsg = "error reading from server: EOF"

// isConnClosedErr checks the error msg for possible conn closed messages. There
// is a raceyness in the timing of when TCP packets are sent from client to
// server, and when we tell the server to stop, so we need to check for both of
// these possible error messages:
//  1. If the call to ss.S.Stop() causes the server's sockets to close while
//     there's still in-fight data from the client on the TCP connection, then
//     the kernel can send an RST back to the client (also see
//     https://stackoverflow.com/questions/33053507/econnreset-in-send-linux-c).
//     Note that while this condition is expected to be rare due to the
//     test httpServer start synchronization, in theory it should be possible,
//     e.g. if the client sends a BDP ping at the right time.
//  2. If, for example, the call to ss.S.Stop() happens after the RPC headers
//     have been received at the server, then the TCP connection can shutdown
//     gracefully when the server's socket closes.
//  3. If there is an actual io.EOF received because the client stopped the stream.
func isConnClosedErr(err error) bool {
	errContainsConnResetMsg := strings.Contains(err.Error(), possibleConnResetMsg)
	errContainsEOFMsg := strings.Contains(err.Error(), possibleEOFMsg)

	return errContainsConnResetMsg || errContainsEOFMsg || err == io.EOF
}

func (l *lazyConn) Write(b []byte) (int, error) {
	if atomic.LoadInt32(&(l.beLazy)) == 1 {
		time.Sleep(time.Second)
	}
	return l.Conn.Write(b)
}

func (s) TestContextDeadlineNotIgnored(t *testing.T) {
	e := noBalancerEnv
	var lc *lazyConn
	e.customDialer = func(network, addr string, timeout time.Duration) (net.Conn, error) {
		conn, err := net.DialTimeout(network, addr, timeout)
		if err != nil {
			return nil, err
		}
		lc = &lazyConn{Conn: conn}
		return lc, nil
	}

	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	cancel()
	atomic.StoreInt32(&(lc.beLazy), 1)
	ctx, cancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer cancel()
	t1 := time.Now()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, context.DeadlineExceeded", err)
	}
	if time.Since(t1) > 2*time.Second {
		t.Fatalf("TestService/EmptyCall(_, _) ran over the deadline")
	}
}

func (s) TestTimeoutOnDeadServer(t *testing.T) {
	for _, e := range listTestEnv() {
		testTimeoutOnDeadServer(t, e)
	}
}

func testTimeoutOnDeadServer(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	// Wait for the client to report READY, stop the server, then wait for the
	// client to notice the connection is gone.
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)
	te.srv.Stop()
	testutils.AwaitNotState(ctx, t, cc, connectivity.Ready)
	ctx, cancel = context.WithTimeout(ctx, defaultTestShortTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(%v, _) = _, %v, want _, error code: %s", ctx, err, codes.DeadlineExceeded)
	}
	awaitNewConnLogOutput()
}

func (s) TestServerGracefulStopIdempotent(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testServerGracefulStopIdempotent(t, e)
	}
}

func testServerGracefulStopIdempotent(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	for i := 0; i < 3; i++ {
		te.srv.GracefulStop()
	}
}

func (s) TestDetailedConnectionCloseErrorPropagatesToRPCError(t *testing.T) {
	rpcStartedOnServer := make(chan struct{})
	rpcDoneOnClient := make(chan struct{})
	defer close(rpcDoneOnClient)
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			close(rpcStartedOnServer)
			<-rpcDoneOnClient
			return status.Error(codes.Internal, "arbitrary status")
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Start an RPC. Then, while the RPC is still being accepted or handled at
	// the server, abruptly stop the server, killing the connection. The RPC
	// error message should include details about the specific connection error
	// that was encountered.
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall = _, %v, want _, <nil>", ss.Client, err)
	}
	// Block until the RPC has been started on the server. This ensures that the
	// ClientConn will find a healthy connection for the RPC to go out on
	// initially, and that the TCP connection will shut down strictly after the
	// RPC has been started on it.
	<-rpcStartedOnServer
	ss.S.Stop()
	// The precise behavior of this test is subject to raceyness around the
	// timing of when TCP packets are sent from client to server, and when we
	// tell the server to stop, so we need to account for both possible error
	// messages.
	if _, err := stream.Recv(); err == io.EOF || !isConnClosedErr(err) {
		t.Fatalf("%v.Recv() = _, %v, want _, rpc error containing substring: %q OR %q", stream, err, possibleConnResetMsg, possibleEOFMsg)
	}
}

func (s) TestFailFast(t *testing.T) {
	for _, e := range listTestEnv() {
		testFailFast(t, e)
	}
}

func testFailFast(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	// Stop the server and tear down all the existing connections.
	te.srv.Stop()
	// Loop until the server teardown is propagated to the client.
	for {
		if err := ctx.Err(); err != nil {
			t.Fatalf("EmptyCall did not return UNAVAILABLE before timeout")
		}
		_, err := tc.EmptyCall(ctx, &testpb.Empty{})
		if status.Code(err) == codes.Unavailable {
			break
		}
		t.Logf("%v.EmptyCall(_, _) = _, %v", tc, err)
		time.Sleep(10 * time.Millisecond)
	}
	// The client keeps reconnecting and ongoing fail-fast RPCs should fail with code.Unavailable.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("TestService/EmptyCall(_, _, _) = _, %v, want _, error code: %s", err, codes.Unavailable)
	}
	if _, err := tc.StreamingInputCall(ctx); status.Code(err) != codes.Unavailable {
		t.Fatalf("TestService/StreamingInputCall(_) = _, %v, want _, error code: %s", err, codes.Unavailable)
	}

	awaitNewConnLogOutput()
}

func testServiceConfigSetup(t *testing.T, e env) *test {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"Failed to dial : context canceled; please retry.",
	)
	return te
}

func newInt(b int) (a *int) {
	return &b
}

func (s) TestGetMethodConfig(t *testing.T) {
	te := testServiceConfigSetup(t, tcpClearRREnv)
	defer te.tearDown()
	r := manual.NewBuilderWithScheme("whatever")

	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.UpdateState(resolver.State{
		Addresses: addrs,
		ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                }
            ],
            "waitForReady": true,
            "timeout": ".001s"
        },
        {
            "name": [
                {
                    "service": "grpc.testing.TestService"
                }
            ],
            "waitForReady": false
        }
    ]
}`)})

	tc := testgrpc.NewTestServiceClient(cc)

	// Make sure service config has been processed by grpc.
	for {
		if cc.GetMethodConfig("/grpc.testing.TestService/EmptyCall").WaitForReady != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// The following RPCs are expected to become non-fail-fast ones with 1ms deadline.
	var err error
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}

	r.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "UnaryCall"
                }
            ],
            "waitForReady": true,
            "timeout": ".001s"
        },
        {
            "name": [
                {
                    "service": "grpc.testing.TestService"
                }
            ],
            "waitForReady": false
        }
    ]
}`)})

	// Make sure service config has been processed by grpc.
	for {
		if mc := cc.GetMethodConfig("/grpc.testing.TestService/EmptyCall"); mc.WaitForReady != nil && !*mc.WaitForReady {
			break
		}
		time.Sleep(time.Millisecond)
	}
	// The following RPCs are expected to become fail-fast.
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Unavailable {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.Unavailable)
	}
}

func (s) TestServiceConfigWaitForReady(t *testing.T) {
	te := testServiceConfigSetup(t, tcpClearRREnv)
	defer te.tearDown()
	r := manual.NewBuilderWithScheme("whatever")

	// Case1: Client API set failfast to be false, and service config set wait_for_ready to be false, Client API should win, and the rpc will wait until deadline exceeds.
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.UpdateState(resolver.State{
		Addresses: addrs,
		ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                },
                {
                    "service": "grpc.testing.TestService",
                    "method": "FullDuplexCall"
                }
            ],
            "waitForReady": false,
            "timeout": ".001s"
        }
    ]
}`)})

	tc := testgrpc.NewTestServiceClient(cc)

	// Make sure service config has been processed by grpc.
	for {
		if cc.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").WaitForReady != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// The following RPCs are expected to become non-fail-fast ones with 1ms deadline.
	var err error
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}
	if _, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want %s", err, codes.DeadlineExceeded)
	}

	// Generate a service config update.
	// Case2:Client API set failfast to be false, and service config set wait_for_ready to be true, and the rpc will wait until deadline exceeds.
	r.UpdateState(resolver.State{
		Addresses: addrs,
		ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                },
                {
                    "service": "grpc.testing.TestService",
                    "method": "FullDuplexCall"
                }
            ],
            "waitForReady": true,
            "timeout": ".001s"
        }
    ]
}`)})

	// Wait for the new service config to take effect.
	for {
		if mc := cc.GetMethodConfig("/grpc.testing.TestService/EmptyCall"); mc.WaitForReady != nil && *mc.WaitForReady {
			break
		}
		time.Sleep(time.Millisecond)
	}
	// The following RPCs are expected to become non-fail-fast ones with 1ms deadline.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}
	if _, err := tc.FullDuplexCall(ctx); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want %s", err, codes.DeadlineExceeded)
	}
}

func (s) TestServiceConfigTimeout(t *testing.T) {
	te := testServiceConfigSetup(t, tcpClearRREnv)
	defer te.tearDown()
	r := manual.NewBuilderWithScheme("whatever")

	// Case1: Client API sets timeout to be 1ns and ServiceConfig sets timeout to be 1hr. Timeout should be 1ns (min of 1ns and 1hr) and the rpc will wait until deadline exceeds.
	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	addrs := []resolver.Address{{Addr: te.srvAddr}}
	r.UpdateState(resolver.State{
		Addresses: addrs,
		ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                },
                {
                    "service": "grpc.testing.TestService",
                    "method": "FullDuplexCall"
                }
            ],
            "waitForReady": true,
            "timeout": "3600s"
        }
    ]
}`)})

	tc := testgrpc.NewTestServiceClient(cc)

	// Make sure service config has been processed by grpc.
	for {
		if cc.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").Timeout != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	// The following RPCs are expected to become non-fail-fast ones with 1ns deadline.
	var err error
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}
	cancel()

	ctx, cancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	if _, err = tc.FullDuplexCall(ctx, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want %s", err, codes.DeadlineExceeded)
	}
	cancel()

	// Generate a service config update.
	// Case2: Client API sets timeout to be 1hr and ServiceConfig sets timeout to be 1ns. Timeout should be 1ns (min of 1ns and 1hr) and the rpc will wait until deadline exceeds.
	r.UpdateState(resolver.State{
		Addresses: addrs,
		ServiceConfig: parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "EmptyCall"
                },
                {
                    "service": "grpc.testing.TestService",
                    "method": "FullDuplexCall"
                }
            ],
            "waitForReady": true,
            "timeout": ".000000001s"
        }
    ]
}`)})

	// Wait for the new service config to take effect.
	for {
		if mc := cc.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall"); mc.Timeout != nil && *mc.Timeout == time.Nanosecond {
			break
		}
		time.Sleep(time.Millisecond)
	}

	ctx, cancel = context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err = tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}

	if _, err = tc.FullDuplexCall(ctx, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want %s", err, codes.DeadlineExceeded)
	}
}

func (s) TestServiceConfigMaxMsgSize(t *testing.T) {
	e := tcpClearRREnv
	r := manual.NewBuilderWithScheme("whatever")

	// Setting up values and objects shared across all test cases.
	const smallSize = 1
	const largeSize = 1024
	const extraLargeSize = 2048

	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}
	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	extraLargePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, extraLargeSize)
	if err != nil {
		t.Fatal(err)
	}

	// Case1: sc set maxReqSize to 2048 (send), maxRespSize to 2048 (recv).
	te1 := testServiceConfigSetup(t, e)
	defer te1.tearDown()

	te1.resolverScheme = r.Scheme()
	te1.startServer(&testServer{security: e.security})
	cc1 := te1.clientConn(grpc.WithResolvers(r))

	addrs := []resolver.Address{{Addr: te1.srvAddr}}
	sc := parseServiceConfig(t, r, `{
    "methodConfig": [
        {
            "name": [
                {
                    "service": "grpc.testing.TestService",
                    "method": "UnaryCall"
                },
                {
                    "service": "grpc.testing.TestService",
                    "method": "FullDuplexCall"
                }
            ],
            "maxRequestMessageBytes": 2048,
            "maxResponseMessageBytes": 2048
        }
    ]
}`)
	r.UpdateState(resolver.State{Addresses: addrs, ServiceConfig: sc})
	tc := testgrpc.NewTestServiceClient(cc1)

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(extraLargeSize),
		Payload:      smallPayload,
	}

	for {
		if cc1.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").MaxReqSize != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Test for unary RPC recv.
	if _, err = tc.UnaryCall(ctx, req, grpc.WaitForReady(true)); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for unary RPC send.
	req.Payload = extraLargePayload
	req.ResponseSize = int32(smallSize)
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for streaming RPC recv.
	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(extraLargeSize),
		},
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            smallPayload,
	}
	stream, err := tc.FullDuplexCall(te1.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err = stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test for streaming RPC send.
	respParam[0].Size = int32(smallSize)
	sreq.Payload = extraLargePayload
	stream, err = tc.FullDuplexCall(te1.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Send(%v) = %v, want _, error code: %s", stream, sreq, err, codes.ResourceExhausted)
	}

	// Case2: Client API set maxReqSize to 1024 (send), maxRespSize to 1024 (recv). Sc sets maxReqSize to 2048 (send), maxRespSize to 2048 (recv).
	te2 := testServiceConfigSetup(t, e)
	te2.resolverScheme = r.Scheme()
	te2.maxClientReceiveMsgSize = newInt(1024)
	te2.maxClientSendMsgSize = newInt(1024)

	te2.startServer(&testServer{security: e.security})
	defer te2.tearDown()
	cc2 := te2.clientConn(grpc.WithResolvers(r))
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: te2.srvAddr}}, ServiceConfig: sc})
	tc = testgrpc.NewTestServiceClient(cc2)

	for {
		if cc2.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").MaxReqSize != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	// Test for unary RPC recv.
	req.Payload = smallPayload
	req.ResponseSize = int32(largeSize)

	if _, err = tc.UnaryCall(ctx, req, grpc.WaitForReady(true)); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for unary RPC send.
	req.Payload = largePayload
	req.ResponseSize = int32(smallSize)
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for streaming RPC recv.
	stream, err = tc.FullDuplexCall(te2.ctx)
	respParam[0].Size = int32(largeSize)
	sreq.Payload = smallPayload
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err = stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test for streaming RPC send.
	respParam[0].Size = int32(smallSize)
	sreq.Payload = largePayload
	stream, err = tc.FullDuplexCall(te2.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Send(%v) = %v, want _, error code: %s", stream, sreq, err, codes.ResourceExhausted)
	}

	// Case3: Client API set maxReqSize to 4096 (send), maxRespSize to 4096 (recv). Sc sets maxReqSize to 2048 (send), maxRespSize to 2048 (recv).
	te3 := testServiceConfigSetup(t, e)
	te3.resolverScheme = r.Scheme()
	te3.maxClientReceiveMsgSize = newInt(4096)
	te3.maxClientSendMsgSize = newInt(4096)

	te3.startServer(&testServer{security: e.security})
	defer te3.tearDown()

	cc3 := te3.clientConn(grpc.WithResolvers(r))
	r.UpdateState(resolver.State{Addresses: []resolver.Address{{Addr: te3.srvAddr}}, ServiceConfig: sc})
	tc = testgrpc.NewTestServiceClient(cc3)

	for {
		if cc3.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").MaxReqSize != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	// Test for unary RPC recv.
	req.Payload = smallPayload
	req.ResponseSize = int32(largeSize)

	if _, err = tc.UnaryCall(ctx, req, grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want <nil>", err)
	}

	req.ResponseSize = int32(extraLargeSize)
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for unary RPC send.
	req.Payload = largePayload
	req.ResponseSize = int32(smallSize)
	if _, err := tc.UnaryCall(ctx, req); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want <nil>", err)
	}

	req.Payload = extraLargePayload
	if _, err = tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for streaming RPC recv.
	stream, err = tc.FullDuplexCall(te3.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam[0].Size = int32(largeSize)
	sreq.Payload = smallPayload

	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err = stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want <nil>", stream, err)
	}

	respParam[0].Size = int32(extraLargeSize)

	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err = stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test for streaming RPC send.
	respParam[0].Size = int32(smallSize)
	sreq.Payload = largePayload
	stream, err = tc.FullDuplexCall(te3.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	sreq.Payload = extraLargePayload
	if err := stream.Send(sreq); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Send(%v) = %v, want _, error code: %s", stream, sreq, err, codes.ResourceExhausted)
	}
}

// Reading from a streaming RPC may fail with context canceled if timeout was
// set by service config (https://github.com/grpc/grpc-go/issues/1818). This
// test makes sure read from streaming RPC doesn't fail in this case.
func (s) TestStreamingRPCWithTimeoutInServiceConfigRecv(t *testing.T) {
	te := testServiceConfigSetup(t, tcpClearRREnv)
	te.startServer(&testServer{security: tcpClearRREnv.security})
	defer te.tearDown()
	r := manual.NewBuilderWithScheme("whatever")

	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	tc := testgrpc.NewTestServiceClient(cc)

	r.UpdateState(resolver.State{
		Addresses: []resolver.Address{{Addr: te.srvAddr}},
		ServiceConfig: parseServiceConfig(t, r, `{
	    "methodConfig": [
	        {
	            "name": [
	                {
	                    "service": "grpc.testing.TestService",
	                    "method": "FullDuplexCall"
	                }
	            ],
	            "waitForReady": true,
	            "timeout": "10s"
	        }
	    ]
	}`)})
	// Make sure service config has been processed by grpc.
	for {
		if cc.GetMethodConfig("/grpc.testing.TestService/FullDuplexCall").Timeout != nil {
			break
		}
		time.Sleep(time.Millisecond)
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("TestService/FullDuplexCall(_) = _, %v, want <nil>", err)
	}

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 0)
	if err != nil {
		t.Fatalf("failed to newPayload: %v", err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{{Size: 0}},
		Payload:            payload,
	}
	if err := stream.Send(req); err != nil {
		t.Fatalf("stream.Send(%v) = %v, want <nil>", req, err)
	}
	stream.CloseSend()
	time.Sleep(time.Second)
	// Sleep 1 second before recv to make sure the final status is received
	// before the recv.
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("stream.Recv = _, %v, want _, <nil>", err)
	}
	// Keep reading to drain the stream.
	for {
		if _, err := stream.Recv(); err != nil {
			break
		}
	}
}

func (s) TestPreloaderClientSend(t *testing.T) {
	for _, e := range listTestEnv() {
		testPreloaderClientSend(t, e)
	}
}

func testPreloaderClientSend(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"Failed to dial : context canceled; please retry.",
	)
	te.startServer(&testServer{security: e.security})

	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	// Test for streaming RPC recv.
	// Set context for send with proper RPC Information
	stream, err := tc.FullDuplexCall(te.ctx, grpc.UseCompressor("gzip"))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	var index int
	for index < len(reqSizes) {
		respParam := []*testpb.ResponseParameters{
			{
				Size: int32(respSizes[index]),
			},
		}

		payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(reqSizes[index]))
		if err != nil {
			t.Fatal(err)
		}

		req := &testpb.StreamingOutputCallRequest{
			ResponseType:       testpb.PayloadType_COMPRESSABLE,
			ResponseParameters: respParam,
			Payload:            payload,
		}
		preparedMsg := &grpc.PreparedMsg{}
		err = preparedMsg.Encode(stream, req)
		if err != nil {
			t.Fatalf("PrepareMsg failed for size %d : %v", reqSizes[index], err)
		}
		if err := stream.SendMsg(preparedMsg); err != nil {
			t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
		}
		reply, err := stream.Recv()
		if err != nil {
			t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
		}
		pt := reply.GetPayload().GetType()
		if pt != testpb.PayloadType_COMPRESSABLE {
			t.Fatalf("Got the reply of type %d, want %d", pt, testpb.PayloadType_COMPRESSABLE)
		}
		size := len(reply.GetPayload().GetBody())
		if size != int(respSizes[index]) {
			t.Fatalf("Got reply body of length %d, want %d", size, respSizes[index])
		}
		index++
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v failed to complele the ping pong test: %v", stream, err)
	}
}

func (s) TestPreloaderSenderSend(t *testing.T) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			for i := 0; i < 10; i++ {
				preparedMsg := &grpc.PreparedMsg{}
				err := preparedMsg.Encode(stream, &testpb.StreamingOutputCallResponse{
					Payload: &testpb.Payload{
						Body: []byte{'0' + uint8(i)},
					},
				})
				if err != nil {
					return err
				}
				stream.SendMsg(preparedMsg)
			}
			return nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}

	var ngot int
	var buf bytes.Buffer
	for {
		reply, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatal(err)
		}
		ngot++
		if buf.Len() > 0 {
			buf.WriteByte(',')
		}
		buf.Write(reply.GetPayload().GetBody())
	}
	if want := 10; ngot != want {
		t.Errorf("Got %d replies, want %d", ngot, want)
	}
	if got, want := buf.String(), "0,1,2,3,4,5,6,7,8,9"; got != want {
		t.Errorf("Got replies %q; want %q", got, want)
	}
}

func (s) TestMaxMsgSizeClientDefault(t *testing.T) {
	for _, e := range listTestEnv() {
		testMaxMsgSizeClientDefault(t, e)
	}
}

func testMaxMsgSizeClientDefault(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.declareLogNoise(
		"Failed to dial : context canceled; please retry.",
	)
	te.startServer(&testServer{security: e.security})

	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const smallSize = 1
	const largeSize = 4 * 1024 * 1024
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(largeSize),
		Payload:      smallPayload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Test for unary RPC recv.
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(largeSize),
		},
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            smallPayload,
	}

	// Test for streaming RPC recv.
	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
}

func (s) TestMaxMsgSizeClientAPI(t *testing.T) {
	for _, e := range listTestEnv() {
		testMaxMsgSizeClientAPI(t, e)
	}
}

func testMaxMsgSizeClientAPI(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	// To avoid error on server side.
	te.maxServerSendMsgSize = newInt(5 * 1024 * 1024)
	te.maxClientReceiveMsgSize = newInt(1024)
	te.maxClientSendMsgSize = newInt(1024)
	te.declareLogNoise(
		"Failed to dial : context canceled; please retry.",
	)
	te.startServer(&testServer{security: e.security})

	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const smallSize = 1
	const largeSize = 1024
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(largeSize),
		Payload:      smallPayload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Test for unary RPC recv.
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for unary RPC send.
	req.Payload = largePayload
	req.ResponseSize = int32(smallSize)
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(largeSize),
		},
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            smallPayload,
	}

	// Test for streaming RPC recv.
	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test for streaming RPC send.
	respParam[0].Size = int32(smallSize)
	sreq.Payload = largePayload
	stream, err = tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Send(%v) = %v, want _, error code: %s", stream, sreq, err, codes.ResourceExhausted)
	}
}

func (s) TestMaxMsgSizeServerAPI(t *testing.T) {
	for _, e := range listTestEnv() {
		testMaxMsgSizeServerAPI(t, e)
	}
}

func testMaxMsgSizeServerAPI(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.maxServerReceiveMsgSize = newInt(1024)
	te.maxServerSendMsgSize = newInt(1024)
	te.declareLogNoise(
		"Failed to dial : context canceled; please retry.",
	)
	te.startServer(&testServer{security: e.security})

	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const smallSize = 1
	const largeSize = 1024
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: int32(largeSize),
		Payload:      smallPayload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Test for unary RPC send.
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Test for unary RPC recv.
	req.Payload = largePayload
	req.ResponseSize = int32(smallSize)
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(largeSize),
		},
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            smallPayload,
	}

	// Test for streaming RPC send.
	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test for streaming RPC recv.
	respParam[0].Size = int32(smallSize)
	sreq.Payload = largePayload
	stream, err = tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
}

func (s) TestTap(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testTap(t, e)
	}
}

type myTap struct {
	cnt int
}

func (t *myTap) handle(ctx context.Context, info *tap.Info) (context.Context, error) {
	if info != nil {
		switch info.FullMethodName {
		case "/grpc.testing.TestService/EmptyCall":
			t.cnt++

			if vals := info.Header.Get("return-error"); len(vals) > 0 && vals[0] == "true" {
				return nil, status.Errorf(codes.Unknown, "tap error")
			}
		case "/grpc.testing.TestService/UnaryCall":
			return nil, fmt.Errorf("tap error")
		case "/grpc.testing.TestService/FullDuplexCall":
			return nil, status.Errorf(codes.FailedPrecondition, "test custom error")
		}
	}
	return ctx, nil
}

func testTap(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	ttap := &myTap{}
	te.tapHandle = ttap.handle
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	if ttap.cnt != 1 {
		t.Fatalf("Get the count in ttap %d, want 1", ttap.cnt)
	}

	if _, err := tc.EmptyCall(metadata.AppendToOutgoingContext(ctx, "return-error", "false"), &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	if ttap.cnt != 2 {
		t.Fatalf("Get the count in ttap %d, want 2", ttap.cnt)
	}

	if _, err := tc.EmptyCall(metadata.AppendToOutgoingContext(ctx, "return-error", "true"), &testpb.Empty{}); status.Code(err) != codes.Unknown {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.Unknown)
	}
	if ttap.cnt != 3 {
		t.Fatalf("Get the count in ttap %d, want 3", ttap.cnt)
	}

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 31)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: 45,
		Payload:      payload,
	}
	if _, err := tc.UnaryCall(ctx, req); status.Code(err) != codes.PermissionDenied {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, %s", err, codes.PermissionDenied)
	}
	str, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("Unexpected error creating stream: %v", err)
	}
	if _, err := str.Recv(); status.Code(err) != codes.FailedPrecondition {
		t.Fatalf("FullDuplexCall Recv() = _, %v, want _, %s", err, codes.FailedPrecondition)
	}
}

func (s) TestEmptyUnaryWithUserAgent(t *testing.T) {
	for _, e := range listTestEnv() {
		testEmptyUnaryWithUserAgent(t, e)
	}
}

func testEmptyUnaryWithUserAgent(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	var header metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	reply, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.Header(&header))
	if err != nil || !proto.Equal(&testpb.Empty{}, reply) {
		t.Fatalf("TestService/EmptyCall(_, _) = %v, %v, want %v, <nil>", reply, err, &testpb.Empty{})
	}
	if v, ok := header["ua"]; !ok || !strings.HasPrefix(v[0], testAppUA) {
		t.Fatalf("header[\"ua\"] = %q, %t, want string with prefix %q, true", v, ok, testAppUA)
	}

	te.srv.Stop()
}

func (s) TestFailedEmptyUnary(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			// This test covers status details, but
			// Grpc-Status-Details-Bin is not support in handler_server.
			continue
		}
		testFailedEmptyUnary(t, e)
	}
}

func testFailedEmptyUnary(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = failAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	wantErr := detailedError
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); !testutils.StatusErrEqual(err, wantErr) {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %v", err, wantErr)
	}
}

func (s) TestLargeUnary(t *testing.T) {
	for _, e := range listTestEnv() {
		testLargeUnary(t, e)
	}
}

func testLargeUnary(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const argSize = 271828
	const respSize = 314159

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	reply, err := tc.UnaryCall(ctx, req)
	if err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, <nil>", err)
	}
	pt := reply.GetPayload().GetType()
	ps := len(reply.GetPayload().GetBody())
	if pt != testpb.PayloadType_COMPRESSABLE || ps != respSize {
		t.Fatalf("Got the reply with type %d len %d; want %d, %d", pt, ps, testpb.PayloadType_COMPRESSABLE, respSize)
	}
}

// Test backward-compatibility API for setting msg size limit.
func (s) TestExceedMsgLimit(t *testing.T) {
	for _, e := range listTestEnv() {
		testExceedMsgLimit(t, e)
	}
}

func testExceedMsgLimit(t *testing.T, e env) {
	te := newTest(t, e)
	maxMsgSize := 1024
	te.maxServerMsgSize, te.maxClientMsgSize = newInt(maxMsgSize), newInt(maxMsgSize)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	largeSize := int32(maxMsgSize + 1)
	const smallSize = 1

	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}

	// Make sure the server cannot receive a unary RPC of largeSize.
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: smallSize,
		Payload:      largePayload,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}
	// Make sure the client cannot receive a unary RPC of largeSize.
	req.ResponseSize = largeSize
	req.Payload = smallPayload
	if _, err := tc.UnaryCall(ctx, req); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	}

	// Make sure the server cannot receive a streaming RPC of largeSize.
	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam := []*testpb.ResponseParameters{
		{
			Size: 1,
		},
	}

	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            largePayload,
	}
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}

	// Test on client side for streaming RPC.
	stream, err = tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	respParam[0].Size = largeSize
	sreq.Payload = smallPayload
	if err := stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
}

func (s) TestPeerClientSide(t *testing.T) {
	for _, e := range listTestEnv() {
		testPeerClientSide(t, e)
	}
}

func testPeerClientSide(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())
	peer := new(peer.Peer)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}
	pa := peer.Addr.String()
	if e.network == "unix" {
		if pa != te.srvAddr {
			t.Fatalf("peer.Addr = %v, want %v", pa, te.srvAddr)
		}
		return
	}
	_, pp, err := net.SplitHostPort(pa)
	if err != nil {
		t.Fatalf("Failed to parse address from peer.")
	}
	_, sp, err := net.SplitHostPort(te.srvAddr)
	if err != nil {
		t.Fatalf("Failed to parse address of test server.")
	}
	if pp != sp {
		t.Fatalf("peer.Addr = localhost:%v, want localhost:%v", pp, sp)
	}
}

// TestPeerNegative tests that if call fails setting peer
// doesn't cause a segmentation fault.
// issue#1141 https://github.com/grpc/grpc-go/issues/1141
func (s) TestPeerNegative(t *testing.T) {
	for _, e := range listTestEnv() {
		testPeerNegative(t, e)
	}
}

func testPeerNegative(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	peer := new(peer.Peer)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	cancel()
	tc.EmptyCall(ctx, &testpb.Empty{}, grpc.Peer(peer))
}

func (s) TestPeerFailedRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testPeerFailedRPC(t, e)
	}
}

func testPeerFailedRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.maxServerReceiveMsgSize = newInt(1 * 1024)
	te.startServer(&testServer{security: e.security})

	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// first make a successful request to the server
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, <nil>", err)
	}

	// make a second request that will be rejected by the server
	const largeSize = 5 * 1024
	largePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, largeSize)
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		Payload:      largePayload,
	}

	peer := new(peer.Peer)
	if _, err := tc.UnaryCall(ctx, req, grpc.Peer(peer)); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, error code: %s", err, codes.ResourceExhausted)
	} else {
		pa := peer.Addr.String()
		if e.network == "unix" {
			if pa != te.srvAddr {
				t.Fatalf("peer.Addr = %v, want %v", pa, te.srvAddr)
			}
			return
		}
		_, pp, err := net.SplitHostPort(pa)
		if err != nil {
			t.Fatalf("Failed to parse address from peer.")
		}
		_, sp, err := net.SplitHostPort(te.srvAddr)
		if err != nil {
			t.Fatalf("Failed to parse address of test server.")
		}
		if pp != sp {
			t.Fatalf("peer.Addr = localhost:%v, want localhost:%v", pp, sp)
		}
	}
}

func (s) TestMetadataUnaryRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testMetadataUnaryRPC(t, e)
	}
}

func testMetadataUnaryRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const argSize = 2718
	const respSize = 314

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	var header, trailer metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	if _, err := tc.UnaryCall(ctx, req, grpc.Header(&header), grpc.Trailer(&trailer)); err != nil {
		t.Fatalf("TestService.UnaryCall(%v, _, _, _) = _, %v; want _, <nil>", ctx, err)
	}
	// Ignore optional response headers that Servers may set:
	if header != nil {
		delete(header, "trailer") // RFC 2616 says server SHOULD (but optional) declare trailers
		delete(header, "date")    // the Date header is also optional
		delete(header, "user-agent")
		delete(header, "content-type")
		delete(header, "grpc-accept-encoding")
	}
	if !reflect.DeepEqual(header, testMetadata) {
		t.Fatalf("Received header metadata %v, want %v", header, testMetadata)
	}
	if !reflect.DeepEqual(trailer, testTrailerMetadata) {
		t.Fatalf("Received trailer metadata %v, want %v", trailer, testTrailerMetadata)
	}
}

func (s) TestMetadataOrderUnaryRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testMetadataOrderUnaryRPC(t, e)
	}
}

func testMetadataOrderUnaryRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	ctx = metadata.AppendToOutgoingContext(ctx, "key1", "value2")
	ctx = metadata.AppendToOutgoingContext(ctx, "key1", "value3")

	// using Join to built expected metadata instead of FromOutgoingContext
	newMetadata := metadata.Join(testMetadata, metadata.Pairs("key1", "value2", "key1", "value3"))

	var header metadata.MD
	if _, err := tc.UnaryCall(ctx, &testpb.SimpleRequest{}, grpc.Header(&header)); err != nil {
		t.Fatal(err)
	}

	// Ignore optional response headers that Servers may set:
	if header != nil {
		delete(header, "trailer") // RFC 2616 says server SHOULD (but optional) declare trailers
		delete(header, "date")    // the Date header is also optional
		delete(header, "user-agent")
		delete(header, "content-type")
		delete(header, "grpc-accept-encoding")
	}

	if !reflect.DeepEqual(header, newMetadata) {
		t.Fatalf("Received header metadata %v, want %v", header, newMetadata)
	}
}

func (s) TestMultipleSetTrailerUnaryRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testMultipleSetTrailerUnaryRPC(t, e)
	}
}

func testMultipleSetTrailerUnaryRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, multipleSetTrailer: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = 1
	)
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	var trailer metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	if _, err := tc.UnaryCall(ctx, req, grpc.Trailer(&trailer), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService.UnaryCall(%v, _, _, _) = _, %v; want _, <nil>", ctx, err)
	}
	expectedTrailer := metadata.Join(testTrailerMetadata, testTrailerMetadata2)
	if !reflect.DeepEqual(trailer, expectedTrailer) {
		t.Fatalf("Received trailer metadata %v, want %v", trailer, expectedTrailer)
	}
}

func (s) TestMultipleSetTrailerStreamingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testMultipleSetTrailerStreamingRPC(t, e)
	}
}

func testMultipleSetTrailerStreamingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, multipleSetTrailer: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	stream, err := tc.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v failed to complele the FullDuplexCall: %v", stream, err)
	}

	trailer := stream.Trailer()
	expectedTrailer := metadata.Join(testTrailerMetadata, testTrailerMetadata2)
	if !reflect.DeepEqual(trailer, expectedTrailer) {
		t.Fatalf("Received trailer metadata %v, want %v", trailer, expectedTrailer)
	}
}

func (s) TestSetAndSendHeaderUnaryRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testSetAndSendHeaderUnaryRPC(t, e)
	}
}

// To test header metadata is sent on SendHeader().
func testSetAndSendHeaderUnaryRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setAndSendHeader: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = 1
	)
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	var header metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	if _, err := tc.UnaryCall(ctx, req, grpc.Header(&header), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService.UnaryCall(%v, _, _, _) = _, %v; want _, <nil>", ctx, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")

	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}
}

func (s) TestMultipleSetHeaderUnaryRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testMultipleSetHeaderUnaryRPC(t, e)
	}
}

// To test header metadata is sent when sending response.
func testMultipleSetHeaderUnaryRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setHeaderOnly: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = 1
	)
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}

	var header metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	if _, err := tc.UnaryCall(ctx, req, grpc.Header(&header), grpc.WaitForReady(true)); err != nil {
		t.Fatalf("TestService.UnaryCall(%v, _, _, _) = _, %v; want _, <nil>", ctx, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")
	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}
}

func (s) TestMultipleSetHeaderUnaryRPCError(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testMultipleSetHeaderUnaryRPCError(t, e)
	}
}

// To test header metadata is sent when sending status.
func testMultipleSetHeaderUnaryRPCError(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setHeaderOnly: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = -1 // Invalid respSize to make RPC fail.
	)
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	var header metadata.MD
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	if _, err := tc.UnaryCall(ctx, req, grpc.Header(&header), grpc.WaitForReady(true)); err == nil {
		t.Fatalf("TestService.UnaryCall(%v, _, _, _) = _, %v; want _, <non-nil>", ctx, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")
	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}
}

func (s) TestSetAndSendHeaderStreamingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testSetAndSendHeaderStreamingRPC(t, e)
	}
}

// To test header metadata is sent on SendHeader().
func testSetAndSendHeaderStreamingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setAndSendHeader: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v failed to complele the FullDuplexCall: %v", stream, err)
	}

	header, err := stream.Header()
	if err != nil {
		t.Fatalf("%v.Header() = _, %v, want _, <nil>", stream, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")
	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}
}

func (s) TestMultipleSetHeaderStreamingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testMultipleSetHeaderStreamingRPC(t, e)
	}
}

// To test header metadata is sent when sending response.
func testMultipleSetHeaderStreamingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setHeaderOnly: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = 1
	)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.StreamingOutputCallRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{
			{Size: respSize},
		},
		Payload: payload,
	}
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
	}
	if _, err := stream.Recv(); err != nil {
		t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v failed to complele the FullDuplexCall: %v", stream, err)
	}

	header, err := stream.Header()
	if err != nil {
		t.Fatalf("%v.Header() = _, %v, want _, <nil>", stream, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")
	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}

}

func (s) TestMultipleSetHeaderStreamingRPCError(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testMultipleSetHeaderStreamingRPCError(t, e)
	}
}

// To test header metadata is sent when sending status.
func testMultipleSetHeaderStreamingRPCError(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, setHeaderOnly: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	const (
		argSize  = 1
		respSize = -1
	)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.StreamingOutputCallRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: []*testpb.ResponseParameters{
			{Size: respSize},
		},
		Payload: payload,
	}
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
	}
	if _, err := stream.Recv(); err == nil {
		t.Fatalf("%v.Recv() = %v, want <non-nil>", stream, err)
	}

	header, err := stream.Header()
	if err != nil {
		t.Fatalf("%v.Header() = _, %v, want _, <nil>", stream, err)
	}
	delete(header, "user-agent")
	delete(header, "content-type")
	delete(header, "grpc-accept-encoding")
	expectedHeader := metadata.Join(testMetadata, testMetadata2)
	if !reflect.DeepEqual(header, expectedHeader) {
		t.Fatalf("Received header metadata %v, want %v", header, expectedHeader)
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
}

// TestMalformedHTTP2Metadata verifies the returned error when the client
// sends an illegal metadata.
func (s) TestMalformedHTTP2Metadata(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			// Failed with "server stops accepting new RPCs".
			// Server stops accepting new RPCs when the client sends an illegal http2 header.
			continue
		}
		testMalformedHTTP2Metadata(t, e)
	}
}

func testMalformedHTTP2Metadata(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 2718)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: 314,
		Payload:      payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, malformedHTTP2Metadata)
	if _, err := tc.UnaryCall(ctx, req); status.Code(err) != codes.Internal {
		t.Fatalf("TestService.UnaryCall(%v, _) = _, %v; want _, %s", ctx, err, codes.Internal)
	}
}

// Tests that the client transparently retries correctly when receiving a
// RST_STREAM with code REFUSED_STREAM.
func (s) TestTransparentRetry(t *testing.T) {
	testCases := []struct {
		failFast bool
		errCode  codes.Code
	}{{
		// success attempt: 1, (stream ID 1)
	}, {
		// success attempt: 2, (stream IDs 3, 5)
	}, {
		// no success attempt (stream IDs 7, 9)
		errCode: codes.Unavailable,
	}, {
		// success attempt: 1 (stream ID 11),
		failFast: true,
	}, {
		// success attempt: 2 (stream IDs 13, 15),
		failFast: true,
	}, {
		// no success attempt (stream IDs 17, 19)
		failFast: true,
		errCode:  codes.Unavailable,
	}}

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen. Err: %v", err)
	}
	defer lis.Close()
	server := &httpServer{
		responses: []httpServerResponse{{
			trailers: [][]string{{
				":status", "200",
				"content-type", "application/grpc",
				"grpc-status", "0",
			}},
		}},
		refuseStream: func(i uint32) bool {
			switch i {
			case 1, 5, 11, 15: // these stream IDs succeed
				return false
			}
			return true // these are refused
		},
	}
	server.start(t, lis)
	cc, err := grpc.Dial(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("failed to dial due to err: %v", err)
	}
	defer cc.Close()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	client := testgrpc.NewTestServiceClient(cc)

	for i, tc := range testCases {
		stream, err := client.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("error creating stream due to err: %v", err)
		}
		code := func(err error) codes.Code {
			if err == io.EOF {
				return codes.OK
			}
			return status.Code(err)
		}
		if _, err := stream.Recv(); code(err) != tc.errCode {
			t.Fatalf("%v: stream.Recv() = _, %v, want error code: %v", i, err, tc.errCode)
		}

	}
}

func (s) TestCancel(t *testing.T) {
	for _, e := range listTestEnv() {
		t.Run(e.name, func(t *testing.T) {
			testCancel(t, e)
		})
	}
}

func testCancel(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise("grpc: the client connection is closing; please retry")
	te.startServer(&testServer{security: e.security, unaryCallSleepTime: time.Second})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	const argSize = 2718
	const respSize = 314

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	time.AfterFunc(1*time.Millisecond, cancel)
	if r, err := tc.UnaryCall(ctx, req); status.Code(err) != codes.Canceled {
		t.Fatalf("TestService/UnaryCall(_, _) = %v, %v; want _, error code: %s", r, err, codes.Canceled)
	}
	awaitNewConnLogOutput()
}

func (s) TestCancelNoIO(t *testing.T) {
	for _, e := range listTestEnv() {
		testCancelNoIO(t, e)
	}
}

func testCancelNoIO(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise("http2Client.notifyError got notified that the client transport was broken")
	te.maxStream = 1 // Only allows 1 live stream per server transport.
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	// Start one blocked RPC for which we'll never send streaming
	// input. This will consume the 1 maximum concurrent streams,
	// causing future RPCs to hang.
	ctx, cancelFirst := context.WithTimeout(context.Background(), defaultTestTimeout)
	_, err := tc.StreamingInputCall(ctx)
	if err != nil {
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want _, <nil>", tc, err)
	}

	// Loop until the ClientConn receives the initial settings
	// frame from the server, notifying it about the maximum
	// concurrent streams. We know when it's received it because
	// an RPC will fail with codes.DeadlineExceeded instead of
	// succeeding.
	// TODO(bradfitz): add internal test hook for this (Issue 534)
	for {
		ctx, cancelSecond := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		_, err := tc.StreamingInputCall(ctx)
		cancelSecond()
		if err == nil {
			continue
		}
		if status.Code(err) == codes.DeadlineExceeded {
			break
		}
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want _, %s", tc, err, codes.DeadlineExceeded)
	}
	// If there are any RPCs in flight before the client receives
	// the max streams setting, let them be expired.
	// TODO(bradfitz): add internal test hook for this (Issue 534)
	time.Sleep(50 * time.Millisecond)

	go func() {
		time.Sleep(50 * time.Millisecond)
		cancelFirst()
	}()

	// This should be blocked until the 1st is canceled, then succeed.
	ctx, cancelThird := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	if _, err := tc.StreamingInputCall(ctx); err != nil {
		t.Errorf("%v.StreamingInputCall(_) = _, %v, want _, <nil>", tc, err)
	}
	cancelThird()
}

// The following tests the gRPC streaming RPC implementations.
// TODO(zhaoq): Have better coverage on error cases.
var (
	reqSizes  = []int{27182, 8, 1828, 45904}
	respSizes = []int{31415, 9, 2653, 58979}
)

func (s) TestNoService(t *testing.T) {
	for _, e := range listTestEnv() {
		testNoService(t, e)
	}
}

func testNoService(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(nil)
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	stream, err := tc.FullDuplexCall(te.ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if _, err := stream.Recv(); status.Code(err) != codes.Unimplemented {
		t.Fatalf("stream.Recv() = _, %v, want _, error code %s", err, codes.Unimplemented)
	}
}

func (s) TestPingPong(t *testing.T) {
	for _, e := range listTestEnv() {
		testPingPong(t, e)
	}
}

func testPingPong(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	var index int
	for index < len(reqSizes) {
		respParam := []*testpb.ResponseParameters{
			{
				Size: int32(respSizes[index]),
			},
		}

		payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(reqSizes[index]))
		if err != nil {
			t.Fatal(err)
		}

		req := &testpb.StreamingOutputCallRequest{
			ResponseType:       testpb.PayloadType_COMPRESSABLE,
			ResponseParameters: respParam,
			Payload:            payload,
		}
		if err := stream.Send(req); err != nil {
			t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
		}
		reply, err := stream.Recv()
		if err != nil {
			t.Fatalf("%v.Recv() = %v, want <nil>", stream, err)
		}
		pt := reply.GetPayload().GetType()
		if pt != testpb.PayloadType_COMPRESSABLE {
			t.Fatalf("Got the reply of type %d, want %d", pt, testpb.PayloadType_COMPRESSABLE)
		}
		size := len(reply.GetPayload().GetBody())
		if size != int(respSizes[index]) {
			t.Fatalf("Got reply body of length %d, want %d", size, respSizes[index])
		}
		index++
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() got %v, want %v", stream, err, nil)
	}
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("%v failed to complele the ping pong test: %v", stream, err)
	}
}

func (s) TestMetadataStreamingRPC(t *testing.T) {
	for _, e := range listTestEnv() {
		testMetadataStreamingRPC(t, e)
	}
}

func testMetadataStreamingRPC(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx := metadata.NewOutgoingContext(te.ctx, testMetadata)
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	go func() {
		headerMD, err := stream.Header()
		if e.security == "tls" {
			delete(headerMD, "transport_security_type")
		}
		delete(headerMD, "trailer") // ignore if present
		delete(headerMD, "user-agent")
		delete(headerMD, "content-type")
		delete(headerMD, "grpc-accept-encoding")
		if err != nil || !reflect.DeepEqual(testMetadata, headerMD) {
			t.Errorf("#1 %v.Header() = %v, %v, want %v, <nil>", stream, headerMD, err, testMetadata)
		}
		// test the cached value.
		headerMD, err = stream.Header()
		delete(headerMD, "trailer") // ignore if present
		delete(headerMD, "user-agent")
		delete(headerMD, "content-type")
		delete(headerMD, "grpc-accept-encoding")
		if err != nil || !reflect.DeepEqual(testMetadata, headerMD) {
			t.Errorf("#2 %v.Header() = %v, %v, want %v, <nil>", stream, headerMD, err, testMetadata)
		}
		err = func() error {
			for index := 0; index < len(reqSizes); index++ {
				respParam := []*testpb.ResponseParameters{
					{
						Size: int32(respSizes[index]),
					},
				}

				payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(reqSizes[index]))
				if err != nil {
					return err
				}

				req := &testpb.StreamingOutputCallRequest{
					ResponseType:       testpb.PayloadType_COMPRESSABLE,
					ResponseParameters: respParam,
					Payload:            payload,
				}
				if err := stream.Send(req); err != nil {
					return fmt.Errorf("%v.Send(%v) = %v, want <nil>", stream, req, err)
				}
			}
			return nil
		}()
		// Tell the server we're done sending args.
		stream.CloseSend()
		if err != nil {
			t.Error(err)
		}
	}()
	for {
		if _, err := stream.Recv(); err != nil {
			break
		}
	}
	trailerMD := stream.Trailer()
	if !reflect.DeepEqual(testTrailerMetadata, trailerMD) {
		t.Fatalf("%v.Trailer() = %v, want %v", stream, trailerMD, testTrailerMetadata)
	}
}

func (s) TestServerStreaming(t *testing.T) {
	for _, e := range listTestEnv() {
		testServerStreaming(t, e)
	}
}

func testServerStreaming(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	respParam := make([]*testpb.ResponseParameters, len(respSizes))
	for i, s := range respSizes {
		respParam[i] = &testpb.ResponseParameters{
			Size: int32(s),
		}
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
	}

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.StreamingOutputCall(ctx, req)
	if err != nil {
		t.Fatalf("%v.StreamingOutputCall(_) = _, %v, want <nil>", tc, err)
	}
	var rpcStatus error
	var respCnt int
	var index int
	for {
		reply, err := stream.Recv()
		if err != nil {
			rpcStatus = err
			break
		}
		pt := reply.GetPayload().GetType()
		if pt != testpb.PayloadType_COMPRESSABLE {
			t.Fatalf("Got the reply of type %d, want %d", pt, testpb.PayloadType_COMPRESSABLE)
		}
		size := len(reply.GetPayload().GetBody())
		if size != int(respSizes[index]) {
			t.Fatalf("Got reply body of length %d, want %d", size, respSizes[index])
		}
		index++
		respCnt++
	}
	if rpcStatus != io.EOF {
		t.Fatalf("Failed to finish the server streaming rpc: %v, want <EOF>", rpcStatus)
	}
	if respCnt != len(respSizes) {
		t.Fatalf("Got %d reply, want %d", len(respSizes), respCnt)
	}
}

func (s) TestFailedServerStreaming(t *testing.T) {
	for _, e := range listTestEnv() {
		testFailedServerStreaming(t, e)
	}
}

func testFailedServerStreaming(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = failAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	respParam := make([]*testpb.ResponseParameters, len(respSizes))
	for i, s := range respSizes {
		respParam[i] = &testpb.ResponseParameters{
			Size: int32(s),
		}
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
	}
	ctx := metadata.NewOutgoingContext(te.ctx, testMetadata)
	stream, err := tc.StreamingOutputCall(ctx, req)
	if err != nil {
		t.Fatalf("%v.StreamingOutputCall(_) = _, %v, want <nil>", tc, err)
	}
	wantErr := status.Error(codes.DataLoss, "error for testing: "+failAppUA)
	if _, err := stream.Recv(); !equalError(err, wantErr) {
		t.Fatalf("%v.Recv() = _, %v, want _, %v", stream, err, wantErr)
	}
}

func equalError(x, y error) bool {
	return x == y || (x != nil && y != nil && x.Error() == y.Error())
}

// concurrentSendServer is a TestServiceServer whose
// StreamingOutputCall makes ten serial Send calls, sending payloads
// "0".."9", inclusive.  TestServerStreamingConcurrent verifies they
// were received in the correct order, and that there were no races.
//
// All other TestServiceServer methods crash if called.
type concurrentSendServer struct {
	testgrpc.TestServiceServer
}

func (s concurrentSendServer) StreamingOutputCall(_ *testpb.StreamingOutputCallRequest, stream testgrpc.TestService_StreamingOutputCallServer) error {
	for i := 0; i < 10; i++ {
		stream.Send(&testpb.StreamingOutputCallResponse{
			Payload: &testpb.Payload{
				Body: []byte{'0' + uint8(i)},
			},
		})
	}
	return nil
}

// Tests doing a bunch of concurrent streaming output calls.
func (s) TestServerStreamingConcurrent(t *testing.T) {
	for _, e := range listTestEnv() {
		testServerStreamingConcurrent(t, e)
	}
}

func testServerStreamingConcurrent(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(concurrentSendServer{})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	doStreamingCall := func() {
		req := &testpb.StreamingOutputCallRequest{}
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		defer cancel()
		stream, err := tc.StreamingOutputCall(ctx, req)
		if err != nil {
			t.Errorf("%v.StreamingOutputCall(_) = _, %v, want <nil>", tc, err)
			return
		}
		var ngot int
		var buf bytes.Buffer
		for {
			reply, err := stream.Recv()
			if err == io.EOF {
				break
			}
			if err != nil {
				t.Fatal(err)
			}
			ngot++
			if buf.Len() > 0 {
				buf.WriteByte(',')
			}
			buf.Write(reply.GetPayload().GetBody())
		}
		if want := 10; ngot != want {
			t.Errorf("Got %d replies, want %d", ngot, want)
		}
		if got, want := buf.String(), "0,1,2,3,4,5,6,7,8,9"; got != want {
			t.Errorf("Got replies %q; want %q", got, want)
		}
	}

	var wg sync.WaitGroup
	for i := 0; i < 20; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			doStreamingCall()
		}()
	}
	wg.Wait()

}

func generatePayloadSizes() [][]int {
	reqSizes := [][]int{
		{27182, 8, 1828, 45904},
	}

	num8KPayloads := 1024
	eightKPayloads := []int{}
	for i := 0; i < num8KPayloads; i++ {
		eightKPayloads = append(eightKPayloads, (1 << 13))
	}
	reqSizes = append(reqSizes, eightKPayloads)

	num2MPayloads := 8
	twoMPayloads := []int{}
	for i := 0; i < num2MPayloads; i++ {
		twoMPayloads = append(twoMPayloads, (1 << 21))
	}
	reqSizes = append(reqSizes, twoMPayloads)

	return reqSizes
}

func (s) TestClientStreaming(t *testing.T) {
	for _, s := range generatePayloadSizes() {
		for _, e := range listTestEnv() {
			testClientStreaming(t, e, s)
		}
	}
}

func testClientStreaming(t *testing.T, e env, sizes []int) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	ctx, cancel := context.WithTimeout(te.ctx, defaultTestTimeout)
	defer cancel()
	stream, err := tc.StreamingInputCall(ctx)
	if err != nil {
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want <nil>", tc, err)
	}

	var sum int
	for _, s := range sizes {
		payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(s))
		if err != nil {
			t.Fatal(err)
		}

		req := &testpb.StreamingInputCallRequest{
			Payload: payload,
		}
		if err := stream.Send(req); err != nil {
			t.Fatalf("%v.Send(_) = %v, want <nil>", stream, err)
		}
		sum += s
	}
	reply, err := stream.CloseAndRecv()
	if err != nil {
		t.Fatalf("%v.CloseAndRecv() got error %v, want %v", stream, err, nil)
	}
	if reply.GetAggregatedPayloadSize() != int32(sum) {
		t.Fatalf("%v.CloseAndRecv().GetAggregatePayloadSize() = %v; want %v", stream, reply.GetAggregatedPayloadSize(), sum)
	}
}

func (s) TestClientStreamingError(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.name == "handler-tls" {
			continue
		}
		testClientStreamingError(t, e)
	}
}

func testClientStreamingError(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, earlyFail: true})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	stream, err := tc.StreamingInputCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want <nil>", tc, err)
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 1)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.StreamingInputCallRequest{
		Payload: payload,
	}
	// The 1st request should go through.
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
	}
	for {
		if err := stream.Send(req); err != io.EOF {
			continue
		}
		if _, err := stream.CloseAndRecv(); status.Code(err) != codes.NotFound {
			t.Fatalf("%v.CloseAndRecv() = %v, want error %s", stream, err, codes.NotFound)
		}
		break
	}
}

func (s) TestExceedMaxStreamsLimit(t *testing.T) {
	for _, e := range listTestEnv() {
		testExceedMaxStreamsLimit(t, e)
	}
}

func testExceedMaxStreamsLimit(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise(
		"http2Client.notifyError got notified that the client transport was broken",
		"Conn.resetTransport failed to create client transport",
		"grpc: the connection is closing",
	)
	te.maxStream = 1 // Only allows 1 live stream per server transport.
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	_, err := tc.StreamingInputCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want _, <nil>", tc, err)
	}
	// Loop until receiving the new max stream setting from the server.
	for {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		defer cancel()
		_, err := tc.StreamingInputCall(ctx)
		if err == nil {
			time.Sleep(50 * time.Millisecond)
			continue
		}
		if status.Code(err) == codes.DeadlineExceeded {
			break
		}
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want _, %s", tc, err, codes.DeadlineExceeded)
	}
}

func (s) TestStreamsQuotaRecovery(t *testing.T) {
	for _, e := range listTestEnv() {
		testStreamsQuotaRecovery(t, e)
	}
}

func testStreamsQuotaRecovery(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise(
		"http2Client.notifyError got notified that the client transport was broken",
		"Conn.resetTransport failed to create client transport",
		"grpc: the connection is closing",
	)
	te.maxStream = 1 // Allows 1 live stream.
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.StreamingInputCall(ctx); err != nil {
		t.Fatalf("tc.StreamingInputCall(_) = _, %v, want _, <nil>", err)
	}
	// Loop until the new max stream setting is effective.
	for {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
		_, err := tc.StreamingInputCall(ctx)
		cancel()
		if err == nil {
			time.Sleep(5 * time.Millisecond)
			continue
		}
		if status.Code(err) == codes.DeadlineExceeded {
			break
		}
		t.Fatalf("tc.StreamingInputCall(_) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}

	var wg sync.WaitGroup
	for i := 0; i < 10; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 314)
			if err != nil {
				t.Error(err)
				return
			}
			req := &testpb.SimpleRequest{
				ResponseType: testpb.PayloadType_COMPRESSABLE,
				ResponseSize: 1592,
				Payload:      payload,
			}
			// No rpc should go through due to the max streams limit.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
			defer cancel()
			if _, err := tc.UnaryCall(ctx, req, grpc.WaitForReady(true)); status.Code(err) != codes.DeadlineExceeded {
				t.Errorf("tc.UnaryCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
			}
		}()
	}
	wg.Wait()

	cancel()
	// A new stream should be allowed after canceling the first one.
	ctx, cancel = context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.StreamingInputCall(ctx); err != nil {
		t.Fatalf("tc.StreamingInputCall(_) = _, %v, want _, %v", err, nil)
	}
}

func (s) TestUnaryClientInterceptor(t *testing.T) {
	for _, e := range listTestEnv() {
		testUnaryClientInterceptor(t, e)
	}
}

func failOkayRPC(ctx context.Context, method string, req, reply any, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
	err := invoker(ctx, method, req, reply, cc, opts...)
	if err == nil {
		return status.Error(codes.NotFound, "")
	}
	return err
}

func testUnaryClientInterceptor(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.unaryClientInt = failOkayRPC
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	tc := testgrpc.NewTestServiceClient(te.clientConn())
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.NotFound {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, error code %s", tc, err, codes.NotFound)
	}
}

func (s) TestStreamClientInterceptor(t *testing.T) {
	for _, e := range listTestEnv() {
		testStreamClientInterceptor(t, e)
	}
}

func failOkayStream(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
	s, err := streamer(ctx, desc, cc, method, opts...)
	if err == nil {
		return nil, status.Error(codes.NotFound, "")
	}
	return s, nil
}

func testStreamClientInterceptor(t *testing.T, e env) {
	te := newTest(t, e)
	te.streamClientInt = failOkayStream
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	tc := testgrpc.NewTestServiceClient(te.clientConn())
	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(1),
		},
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(1))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.StreamingOutputCall(ctx, req); status.Code(err) != codes.NotFound {
		t.Fatalf("%v.StreamingOutputCall(_) = _, %v, want _, error code %s", tc, err, codes.NotFound)
	}
}

func (s) TestUnaryServerInterceptor(t *testing.T) {
	for _, e := range listTestEnv() {
		testUnaryServerInterceptor(t, e)
	}
}

func errInjector(context.Context, any, *grpc.UnaryServerInfo, grpc.UnaryHandler) (any, error) {
	return nil, status.Error(codes.PermissionDenied, "")
}

func testUnaryServerInterceptor(t *testing.T, e env) {
	te := newTest(t, e)
	te.unaryServerInt = errInjector
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	tc := testgrpc.NewTestServiceClient(te.clientConn())
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.PermissionDenied {
		t.Fatalf("%v.EmptyCall(_, _) = _, %v, want _, error code %s", tc, err, codes.PermissionDenied)
	}
}

func (s) TestStreamServerInterceptor(t *testing.T) {
	for _, e := range listTestEnv() {
		// TODO(bradfitz): Temporarily skip this env due to #619.
		if e.name == "handler-tls" {
			continue
		}
		testStreamServerInterceptor(t, e)
	}
}

func fullDuplexOnly(srv any, ss grpc.ServerStream, info *grpc.StreamServerInfo, handler grpc.StreamHandler) error {
	if info.FullMethod == "/grpc.testing.TestService/FullDuplexCall" {
		return handler(srv, ss)
	}
	// Reject the other methods.
	return status.Error(codes.PermissionDenied, "")
}

func testStreamServerInterceptor(t *testing.T, e env) {
	te := newTest(t, e)
	te.streamServerInt = fullDuplexOnly
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	tc := testgrpc.NewTestServiceClient(te.clientConn())
	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(1),
		},
	}
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(1))
	if err != nil {
		t.Fatal(err)
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            payload,
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	s1, err := tc.StreamingOutputCall(ctx, req)
	if err != nil {
		t.Fatalf("%v.StreamingOutputCall(_) = _, %v, want _, <nil>", tc, err)
	}
	if _, err := s1.Recv(); status.Code(err) != codes.PermissionDenied {
		t.Fatalf("%v.StreamingInputCall(_) = _, %v, want _, error code %s", tc, err, codes.PermissionDenied)
	}
	s2, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err := s2.Send(req); err != nil {
		t.Fatalf("%v.Send(_) = %v, want <nil>", s2, err)
	}
	if _, err := s2.Recv(); err != nil {
		t.Fatalf("%v.Recv() = _, %v, want _, <nil>", s2, err)
	}
}

// funcServer implements methods of TestServiceServer using funcs,
// similar to an http.HandlerFunc.
// Any unimplemented method will crash. Tests implement the method(s)
// they need.
type funcServer struct {
	testgrpc.TestServiceServer
	unaryCall          func(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error)
	streamingInputCall func(stream testgrpc.TestService_StreamingInputCallServer) error
	fullDuplexCall     func(stream testgrpc.TestService_FullDuplexCallServer) error
}

func (s *funcServer) UnaryCall(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	return s.unaryCall(ctx, in)
}

func (s *funcServer) StreamingInputCall(stream testgrpc.TestService_StreamingInputCallServer) error {
	return s.streamingInputCall(stream)
}

func (s *funcServer) FullDuplexCall(stream testgrpc.TestService_FullDuplexCallServer) error {
	return s.fullDuplexCall(stream)
}

func (s) TestClientRequestBodyErrorUnexpectedEOF(t *testing.T) {
	for _, e := range listTestEnv() {
		testClientRequestBodyErrorUnexpectedEOF(t, e)
	}
}

func testClientRequestBodyErrorUnexpectedEOF(t *testing.T, e env) {
	te := newTest(t, e)
	ts := &funcServer{unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
		errUnexpectedCall := errors.New("unexpected call func server method")
		t.Error(errUnexpectedCall)
		return nil, errUnexpectedCall
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/UnaryCall", false)
		// Say we have 5 bytes coming, but set END_STREAM flag:
		st.writeData(1, true, []byte{0, 0, 0, 0, 5})
		st.wantAnyFrame() // wait for server to crash (it used to crash)
	})
}

func (s) TestClientRequestBodyErrorCloseAfterLength(t *testing.T) {
	for _, e := range listTestEnv() {
		testClientRequestBodyErrorCloseAfterLength(t, e)
	}
}

// Tests gRPC server's behavior when a gRPC client sends a frame with an invalid
// streamID. Per [HTTP/2 spec]: Streams initiated by a client MUST use
// odd-numbered stream identifiers. This test sets up a test server and sends a
// header frame with stream ID of 2. The test asserts that a subsequent read on
// the transport sends a GoAwayFrame with error code: PROTOCOL_ERROR.
//
// [HTTP/2 spec]: https://httpwg.org/specs/rfc7540.html#StreamIdentifiers
func (s) TestClientInvalidStreamID(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()
	s := grpc.NewServer()
	defer s.Stop()
	go s.Serve(lis)

	conn, err := net.DialTimeout("tcp", lis.Addr().String(), defaultTestTimeout)
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}
	st := newServerTesterFromConn(t, conn)
	st.greet()
	st.writeHeadersGRPC(2, "/grpc.testing.TestService/StreamingInputCall", true)
	goAwayFrame := st.wantGoAway(http2.ErrCodeProtocol)
	want := "received an illegal stream id: 2."
	if got := string(goAwayFrame.DebugData()); !strings.Contains(got, want) {
		t.Fatalf(" Received: %v, Expected error message to contain: %v.", got, want)
	}
}

// TestInvalidStreamIDSmallerThanPrevious tests the server sends a GOAWAY frame
// with error code: PROTOCOL_ERROR when the streamID of the current frame is
// lower than the previous frames.
func (s) TestInvalidStreamIDSmallerThanPrevious(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()
	s := grpc.NewServer()
	defer s.Stop()
	go s.Serve(lis)

	conn, err := net.DialTimeout("tcp", lis.Addr().String(), defaultTestTimeout)
	if err != nil {
		t.Fatalf("Failed to dial: %v", err)
	}
	st := newServerTesterFromConn(t, conn)
	st.greet()
	st.writeHeadersGRPC(3, "/grpc.testing.TestService/StreamingInputCall", true)
	st.wantAnyFrame()
	st.writeHeadersGRPC(1, "/grpc.testing.TestService/StreamingInputCall", true)
	goAwayFrame := st.wantGoAway(http2.ErrCodeProtocol)
	want := "received an illegal stream id: 1"
	if got := string(goAwayFrame.DebugData()); !strings.Contains(got, want) {
		t.Fatalf(" Received: %v, Expected error message to contain: %v.", got, want)
	}
}

func testClientRequestBodyErrorCloseAfterLength(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise("Server.processUnaryRPC failed to write status")
	ts := &funcServer{unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
		errUnexpectedCall := errors.New("unexpected call func server method")
		t.Error(errUnexpectedCall)
		return nil, errUnexpectedCall
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/UnaryCall", false)
		// say we're sending 5 bytes, but then close the connection instead.
		st.writeData(1, false, []byte{0, 0, 0, 0, 5})
		st.cc.Close()
	})
}

func (s) TestClientRequestBodyErrorCancel(t *testing.T) {
	for _, e := range listTestEnv() {
		testClientRequestBodyErrorCancel(t, e)
	}
}

func testClientRequestBodyErrorCancel(t *testing.T, e env) {
	te := newTest(t, e)
	gotCall := make(chan bool, 1)
	ts := &funcServer{unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
		gotCall <- true
		return new(testpb.SimpleResponse), nil
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/UnaryCall", false)
		// Say we have 5 bytes coming, but cancel it instead.
		st.writeRSTStream(1, http2.ErrCodeCancel)
		st.writeData(1, false, []byte{0, 0, 0, 0, 5})

		// Verify we didn't a call yet.
		select {
		case <-gotCall:
			t.Fatal("unexpected call")
		default:
		}

		// And now send an uncanceled (but still invalid), just to get a response.
		st.writeHeadersGRPC(3, "/grpc.testing.TestService/UnaryCall", false)
		st.writeData(3, true, []byte{0, 0, 0, 0, 0})
		<-gotCall
		st.wantAnyFrame()
	})
}

func (s) TestClientRequestBodyErrorCancelStreamingInput(t *testing.T) {
	for _, e := range listTestEnv() {
		testClientRequestBodyErrorCancelStreamingInput(t, e)
	}
}

func testClientRequestBodyErrorCancelStreamingInput(t *testing.T, e env) {
	te := newTest(t, e)
	recvErr := make(chan error, 1)
	ts := &funcServer{streamingInputCall: func(stream testgrpc.TestService_StreamingInputCallServer) error {
		_, err := stream.Recv()
		recvErr <- err
		return nil
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/StreamingInputCall", false)
		// Say we have 5 bytes coming, but cancel it instead.
		st.writeData(1, false, []byte{0, 0, 0, 0, 5})
		st.writeRSTStream(1, http2.ErrCodeCancel)

		var got error
		select {
		case got = <-recvErr:
		case <-time.After(3 * time.Second):
			t.Fatal("timeout waiting for error")
		}
		if grpc.Code(got) != codes.Canceled {
			t.Errorf("error = %#v; want error code %s", got, codes.Canceled)
		}
	})
}

func (s) TestClientInitialHeaderEndStream(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler {
			continue
		}
		testClientInitialHeaderEndStream(t, e)
	}
}

func testClientInitialHeaderEndStream(t *testing.T, e env) {
	// To ensure RST_STREAM is sent for illegal data write and not normal stream
	// close.
	frameCheckingDone := make(chan struct{})
	// To ensure goroutine for test does not end before RPC handler performs error
	// checking.
	handlerDone := make(chan struct{})
	te := newTest(t, e)
	ts := &funcServer{streamingInputCall: func(stream testgrpc.TestService_StreamingInputCallServer) error {
		defer close(handlerDone)
		// Block on serverTester receiving RST_STREAM. This ensures server has closed
		// stream before stream.Recv().
		<-frameCheckingDone
		data, err := stream.Recv()
		if err == nil {
			t.Errorf("unexpected data received in func server method: '%v'", data)
		} else if status.Code(err) != codes.Canceled {
			t.Errorf("expected canceled error, instead received '%v'", err)
		}
		return nil
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		// Send a headers with END_STREAM flag, but then write data.
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/StreamingInputCall", true)
		st.writeData(1, false, []byte{0, 0, 0, 0, 0})
		st.wantAnyFrame()
		st.wantAnyFrame()
		st.wantRSTStream(http2.ErrCodeStreamClosed)
		close(frameCheckingDone)
		<-handlerDone
	})
}

func (s) TestClientSendDataAfterCloseSend(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler {
			continue
		}
		testClientSendDataAfterCloseSend(t, e)
	}
}

func testClientSendDataAfterCloseSend(t *testing.T, e env) {
	// To ensure RST_STREAM is sent for illegal data write prior to execution of RPC
	// handler.
	frameCheckingDone := make(chan struct{})
	// To ensure goroutine for test does not end before RPC handler performs error
	// checking.
	handlerDone := make(chan struct{})
	te := newTest(t, e)
	ts := &funcServer{streamingInputCall: func(stream testgrpc.TestService_StreamingInputCallServer) error {
		defer close(handlerDone)
		// Block on serverTester receiving RST_STREAM. This ensures server has closed
		// stream before stream.Recv().
		<-frameCheckingDone
		for {
			_, err := stream.Recv()
			if err == io.EOF {
				break
			}
			if err != nil {
				if status.Code(err) != codes.Canceled {
					t.Errorf("expected canceled error, instead received '%v'", err)
				}
				break
			}
		}
		if err := stream.SendMsg(nil); err == nil {
			t.Error("expected error sending message on stream after stream closed due to illegal data")
		} else if status.Code(err) != codes.Canceled {
			t.Errorf("expected cancel error, instead received '%v'", err)
		}
		return nil
	}}
	te.startServer(ts)
	defer te.tearDown()
	te.withServerTester(func(st *serverTester) {
		st.writeHeadersGRPC(1, "/grpc.testing.TestService/StreamingInputCall", false)
		// Send data with END_STREAM flag, but then write more data.
		st.writeData(1, true, []byte{0, 0, 0, 0, 0})
		st.writeData(1, false, []byte{0, 0, 0, 0, 0})
		st.wantAnyFrame()
		st.wantAnyFrame()
		st.wantRSTStream(http2.ErrCodeStreamClosed)
		close(frameCheckingDone)
		<-handlerDone
	})
}

func (s) TestClientResourceExhaustedCancelFullDuplex(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler {
			// httpHandler write won't be blocked on flow control window.
			continue
		}
		testClientResourceExhaustedCancelFullDuplex(t, e)
	}
}

func testClientResourceExhaustedCancelFullDuplex(t *testing.T, e env) {
	te := newTest(t, e)
	recvErr := make(chan error, 1)
	ts := &funcServer{fullDuplexCall: func(stream testgrpc.TestService_FullDuplexCallServer) error {
		defer close(recvErr)
		_, err := stream.Recv()
		if err != nil {
			return status.Errorf(codes.Internal, "stream.Recv() got error: %v, want <nil>", err)
		}
		// create a payload that's larger than the default flow control window.
		payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 10)
		if err != nil {
			return err
		}
		resp := &testpb.StreamingOutputCallResponse{
			Payload: payload,
		}
		ce := make(chan error, 1)
		go func() {
			var err error
			for {
				if err = stream.Send(resp); err != nil {
					break
				}
			}
			ce <- err
		}()
		select {
		case err = <-ce:
		case <-time.After(10 * time.Second):
			err = errors.New("10s timeout reached")
		}
		recvErr <- err
		return err
	}}
	te.startServer(ts)
	defer te.tearDown()
	// set a low limit on receive message size to error with Resource Exhausted on
	// client side when server send a large message.
	te.maxClientReceiveMsgSize = newInt(10)
	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	req := &testpb.StreamingOutputCallRequest{}
	if err := stream.Send(req); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
	}
	if _, err := stream.Recv(); status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
	err = <-recvErr
	if status.Code(err) != codes.Canceled {
		t.Fatalf("server got error %v, want error code: %s", err, codes.Canceled)
	}
}

type clientFailCreds struct{}

func (c *clientFailCreds) ServerHandshake(rawConn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	return rawConn, nil, nil
}
func (c *clientFailCreds) ClientHandshake(context.Context, string, net.Conn) (net.Conn, credentials.AuthInfo, error) {
	return nil, nil, fmt.Errorf("client handshake fails with fatal error")
}
func (c *clientFailCreds) Info() credentials.ProtocolInfo {
	return credentials.ProtocolInfo{}
}
func (c *clientFailCreds) Clone() credentials.TransportCredentials {
	return c
}
func (c *clientFailCreds) OverrideServerName(string) error {
	return nil
}

// This test makes sure that failfast RPCs fail if client handshake fails with
// fatal errors.
func (s) TestFailfastRPCFailOnFatalHandshakeError(t *testing.T) {
	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	cc, err := grpc.Dial("passthrough:///"+lis.Addr().String(), grpc.WithTransportCredentials(&clientFailCreds{}))
	if err != nil {
		t.Fatalf("grpc.Dial(_) = %v", err)
	}
	defer cc.Close()

	tc := testgrpc.NewTestServiceClient(cc)
	// This unary call should fail, but not timeout.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}, grpc.WaitForReady(false)); status.Code(err) != codes.Unavailable {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want <Unavailable>", err)
	}
}

func (s) TestFlowControlLogicalRace(t *testing.T) {
	// Test for a regression of https://github.com/grpc/grpc-go/issues/632,
	// and other flow control bugs.

	const (
		itemCount   = 100
		itemSize    = 1 << 10
		recvCount   = 2
		maxFailures = 3
	)

	requestCount := 3000
	if raceMode {
		requestCount = 1000
	}

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to listen: %v", err)
	}
	defer lis.Close()

	s := grpc.NewServer()
	testgrpc.RegisterTestServiceServer(s, &flowControlLogicalRaceServer{
		itemCount: itemCount,
		itemSize:  itemSize,
	})
	defer s.Stop()

	go s.Serve(lis)

	cc, err := grpc.Dial(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial(%q) = %v", lis.Addr().String(), err)
	}
	defer cc.Close()
	cl := testgrpc.NewTestServiceClient(cc)

	failures := 0
	for i := 0; i < requestCount; i++ {
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		output, err := cl.StreamingOutputCall(ctx, &testpb.StreamingOutputCallRequest{})
		if err != nil {
			t.Fatalf("StreamingOutputCall; err = %q", err)
		}

		for j := 0; j < recvCount; j++ {
			if _, err := output.Recv(); err != nil {
				if err == io.EOF || status.Code(err) == codes.DeadlineExceeded {
					t.Errorf("got %d responses to request %d", j, i)
					failures++
					break
				}
				t.Fatalf("Recv; err = %q", err)
			}
		}
		cancel()

		if failures >= maxFailures {
			// Continue past the first failure to see if the connection is
			// entirely broken, or if only a single RPC was affected
			t.Fatalf("Too many failures received; aborting")
		}
	}
}

type flowControlLogicalRaceServer struct {
	testgrpc.TestServiceServer

	itemSize  int
	itemCount int
}

func (s *flowControlLogicalRaceServer) StreamingOutputCall(_ *testpb.StreamingOutputCallRequest, srv testgrpc.TestService_StreamingOutputCallServer) error {
	for i := 0; i < s.itemCount; i++ {
		err := srv.Send(&testpb.StreamingOutputCallResponse{
			Payload: &testpb.Payload{
				// Sending a large stream of data which the client reject
				// helps to trigger some types of flow control bugs.
				//
				// Reallocating memory here is inefficient, but the stress it
				// puts on the GC leads to more frequent flow control
				// failures. The GC likely causes more variety in the
				// goroutine scheduling orders.
				Body: bytes.Repeat([]byte("a"), s.itemSize),
			},
		})
		if err != nil {
			return err
		}
	}
	return nil
}

type lockingWriter struct {
	mu sync.Mutex
	w  io.Writer
}

func (lw *lockingWriter) Write(p []byte) (n int, err error) {
	lw.mu.Lock()
	defer lw.mu.Unlock()
	return lw.w.Write(p)
}

func (lw *lockingWriter) setWriter(w io.Writer) {
	lw.mu.Lock()
	defer lw.mu.Unlock()
	lw.w = w
}

var testLogOutput = &lockingWriter{w: os.Stderr}

// awaitNewConnLogOutput waits for any of grpc.NewConn's goroutines to
// terminate, if they're still running. It spams logs with this
// message.  We wait for it so our log filter is still
// active. Otherwise the "defer restore()" at the top of various test
// functions restores our log filter and then the goroutine spams.
func awaitNewConnLogOutput() {
	awaitLogOutput(50*time.Millisecond, "grpc: the client connection is closing; please retry")
}

func awaitLogOutput(maxWait time.Duration, phrase string) {
	pb := []byte(phrase)

	timer := time.NewTimer(maxWait)
	defer timer.Stop()
	wakeup := make(chan bool, 1)
	for {
		if logOutputHasContents(pb, wakeup) {
			return
		}
		select {
		case <-timer.C:
			// Too slow. Oh well.
			return
		case <-wakeup:
		}
	}
}

func logOutputHasContents(v []byte, wakeup chan<- bool) bool {
	testLogOutput.mu.Lock()
	defer testLogOutput.mu.Unlock()
	fw, ok := testLogOutput.w.(*filterWriter)
	if !ok {
		return false
	}
	fw.mu.Lock()
	defer fw.mu.Unlock()
	if bytes.Contains(fw.buf.Bytes(), v) {
		return true
	}
	fw.wakeup = wakeup
	return false
}

var verboseLogs = flag.Bool("verbose_logs", false, "show all log output, without filtering")

func noop() {}

// declareLogNoise declares that t is expected to emit the following noisy
// phrases, even on success. Those phrases will be filtered from log output and
// only be shown if *verbose_logs or t ends up failing. The returned restore
// function should be called with defer to be run before the test ends.
func declareLogNoise(t *testing.T, phrases ...string) (restore func()) {
	if *verboseLogs {
		return noop
	}
	fw := &filterWriter{dst: os.Stderr, filter: phrases}
	testLogOutput.setWriter(fw)
	return func() {
		if t.Failed() {
			fw.mu.Lock()
			defer fw.mu.Unlock()
			if fw.buf.Len() > 0 {
				t.Logf("Complete log output:\n%s", fw.buf.Bytes())
			}
		}
		testLogOutput.setWriter(os.Stderr)
	}
}

type filterWriter struct {
	dst    io.Writer
	filter []string

	mu     sync.Mutex
	buf    bytes.Buffer
	wakeup chan<- bool // if non-nil, gets true on write
}

func (fw *filterWriter) Write(p []byte) (n int, err error) {
	fw.mu.Lock()
	fw.buf.Write(p)
	if fw.wakeup != nil {
		select {
		case fw.wakeup <- true:
		default:
		}
	}
	fw.mu.Unlock()

	ps := string(p)
	for _, f := range fw.filter {
		if strings.Contains(ps, f) {
			return len(p), nil
		}
	}
	return fw.dst.Write(p)
}

func (s) TestGRPCMethod(t *testing.T) {
	var method string
	var ok bool

	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			method, ok = grpc.Method(ctx)
			return &testpb.Empty{}, nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v; want _, nil", err)
	}

	if want := "/grpc.testing.TestService/EmptyCall"; !ok || method != want {
		t.Fatalf("grpc.Method(_) = %q, %v; want %q, true", method, ok, want)
	}
}

func (s) TestUnaryProxyDoesNotForwardMetadata(t *testing.T) {
	const mdkey = "somedata"

	// endpoint ensures mdkey is NOT in metadata and returns an error if it is.
	endpoint := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			if md, ok := metadata.FromIncomingContext(ctx); !ok || md[mdkey] != nil {
				return nil, status.Errorf(codes.Internal, "endpoint: md=%v; want !contains(%q)", md, mdkey)
			}
			return &testpb.Empty{}, nil
		},
	}
	if err := endpoint.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer endpoint.Stop()

	// proxy ensures mdkey IS in metadata, then forwards the RPC to endpoint
	// without explicitly copying the metadata.
	proxy := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, in *testpb.Empty) (*testpb.Empty, error) {
			if md, ok := metadata.FromIncomingContext(ctx); !ok || md[mdkey] == nil {
				return nil, status.Errorf(codes.Internal, "proxy: md=%v; want contains(%q)", md, mdkey)
			}
			return endpoint.Client.EmptyCall(ctx, in)
		},
	}
	if err := proxy.Start(nil); err != nil {
		t.Fatalf("Error starting proxy server: %v", err)
	}
	defer proxy.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	md := metadata.Pairs(mdkey, "val")
	ctx = metadata.NewOutgoingContext(ctx, md)

	// Sanity check that endpoint properly errors when it sees mdkey.
	_, err := endpoint.Client.EmptyCall(ctx, &testpb.Empty{})
	if s, ok := status.FromError(err); !ok || s.Code() != codes.Internal {
		t.Fatalf("endpoint.Client.EmptyCall(_, _) = _, %v; want _, <status with Code()=Internal>", err)
	}

	if _, err := proxy.Client.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatal(err.Error())
	}
}

func (s) TestStreamingProxyDoesNotForwardMetadata(t *testing.T) {
	const mdkey = "somedata"

	// doFDC performs a FullDuplexCall with client and returns the error from the
	// first stream.Recv call, or nil if that error is io.EOF.  Calls t.Fatal if
	// the stream cannot be established.
	doFDC := func(ctx context.Context, client testgrpc.TestServiceClient) error {
		stream, err := client.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("Unwanted error: %v", err)
		}
		if _, err := stream.Recv(); err != io.EOF {
			return err
		}
		return nil
	}

	// endpoint ensures mdkey is NOT in metadata and returns an error if it is.
	endpoint := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			ctx := stream.Context()
			if md, ok := metadata.FromIncomingContext(ctx); !ok || md[mdkey] != nil {
				return status.Errorf(codes.Internal, "endpoint: md=%v; want !contains(%q)", md, mdkey)
			}
			return nil
		},
	}
	if err := endpoint.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer endpoint.Stop()

	// proxy ensures mdkey IS in metadata, then forwards the RPC to endpoint
	// without explicitly copying the metadata.
	proxy := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			ctx := stream.Context()
			if md, ok := metadata.FromIncomingContext(ctx); !ok || md[mdkey] == nil {
				return status.Errorf(codes.Internal, "endpoint: md=%v; want !contains(%q)", md, mdkey)
			}
			return doFDC(ctx, endpoint.Client)
		},
	}
	if err := proxy.Start(nil); err != nil {
		t.Fatalf("Error starting proxy server: %v", err)
	}
	defer proxy.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	md := metadata.Pairs(mdkey, "val")
	ctx = metadata.NewOutgoingContext(ctx, md)

	// Sanity check that endpoint properly errors when it sees mdkey in ctx.
	err := doFDC(ctx, endpoint.Client)
	if s, ok := status.FromError(err); !ok || s.Code() != codes.Internal {
		t.Fatalf("stream.Recv() = _, %v; want _, <status with Code()=Internal>", err)
	}

	if err := doFDC(ctx, proxy.Client); err != nil {
		t.Fatalf("doFDC(_, proxy.Client) = %v; want nil", err)
	}
}

func (s) TestStatsTagsAndTrace(t *testing.T) {
	// Data added to context by client (typically in a stats handler).
	tags := []byte{1, 5, 2, 4, 3}
	trace := []byte{5, 2, 1, 3, 4}

	// endpoint ensures Tags() and Trace() in context match those that were added
	// by the client and returns an error if not.
	endpoint := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			md, _ := metadata.FromIncomingContext(ctx)
			if tg := stats.Tags(ctx); !reflect.DeepEqual(tg, tags) {
				return nil, status.Errorf(codes.Internal, "stats.Tags(%v)=%v; want %v", ctx, tg, tags)
			}
			if !reflect.DeepEqual(md["grpc-tags-bin"], []string{string(tags)}) {
				return nil, status.Errorf(codes.Internal, "md['grpc-tags-bin']=%v; want %v", md["grpc-tags-bin"], tags)
			}
			if tr := stats.Trace(ctx); !reflect.DeepEqual(tr, trace) {
				return nil, status.Errorf(codes.Internal, "stats.Trace(%v)=%v; want %v", ctx, tr, trace)
			}
			if !reflect.DeepEqual(md["grpc-trace-bin"], []string{string(trace)}) {
				return nil, status.Errorf(codes.Internal, "md['grpc-trace-bin']=%v; want %v", md["grpc-trace-bin"], trace)
			}
			return &testpb.Empty{}, nil
		},
	}
	if err := endpoint.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer endpoint.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	testCases := []struct {
		ctx  context.Context
		want codes.Code
	}{
		{ctx: ctx, want: codes.Internal},
		{ctx: stats.SetTags(ctx, tags), want: codes.Internal},
		{ctx: stats.SetTrace(ctx, trace), want: codes.Internal},
		{ctx: stats.SetTags(stats.SetTrace(ctx, tags), tags), want: codes.Internal},
		{ctx: stats.SetTags(stats.SetTrace(ctx, trace), tags), want: codes.OK},
	}

	for _, tc := range testCases {
		_, err := endpoint.Client.EmptyCall(tc.ctx, &testpb.Empty{})
		if tc.want == codes.OK && err != nil {
			t.Fatalf("endpoint.Client.EmptyCall(%v, _) = _, %v; want _, nil", tc.ctx, err)
		}
		if s, ok := status.FromError(err); !ok || s.Code() != tc.want {
			t.Fatalf("endpoint.Client.EmptyCall(%v, _) = _, %v; want _, <status with Code()=%v>", tc.ctx, err, tc.want)
		}
	}
}

func (s) TestTapTimeout(t *testing.T) {
	sopts := []grpc.ServerOption{
		grpc.InTapHandle(func(ctx context.Context, _ *tap.Info) (context.Context, error) {
			c, cancel := context.WithCancel(ctx)
			// Call cancel instead of setting a deadline so we can detect which error
			// occurred -- this cancellation (desired) or the client's deadline
			// expired (indicating this cancellation did not affect the RPC).
			time.AfterFunc(10*time.Millisecond, cancel)
			return c, nil
		}),
	}

	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			<-ctx.Done()
			return nil, status.Errorf(codes.Canceled, ctx.Err().Error())
		},
	}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	// This was known to be flaky; test several times.
	for i := 0; i < 10; i++ {
		// Set our own deadline in case the server hangs.
		ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
		res, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
		cancel()
		if s, ok := status.FromError(err); !ok || s.Code() != codes.Canceled {
			t.Fatalf("ss.Client.EmptyCall(ctx, _) = %v, %v; want nil, <status with Code()=Canceled>", res, err)
		}
	}

}

func (s) TestClientWriteFailsAfterServerClosesStream(t *testing.T) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(testgrpc.TestService_FullDuplexCallServer) error {
			return status.Errorf(codes.Internal, "")
		},
	}
	sopts := []grpc.ServerOption{}
	if err := ss.Start(sopts); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("Error while creating stream: %v", err)
	}
	for {
		if err := stream.Send(&testpb.StreamingOutputCallRequest{}); err == nil {
			time.Sleep(5 * time.Millisecond)
		} else if err == io.EOF {
			break // Success.
		} else {
			t.Fatalf("stream.Send(_) = %v, want io.EOF", err)
		}
	}
}

type windowSizeConfig struct {
	serverStream int32
	serverConn   int32
	clientStream int32
	clientConn   int32
}

func (s) TestConfigurableWindowSizeWithLargeWindow(t *testing.T) {
	wc := windowSizeConfig{
		serverStream: 8 * 1024 * 1024,
		serverConn:   12 * 1024 * 1024,
		clientStream: 6 * 1024 * 1024,
		clientConn:   8 * 1024 * 1024,
	}
	for _, e := range listTestEnv() {
		testConfigurableWindowSize(t, e, wc)
	}
}

func (s) TestConfigurableWindowSizeWithSmallWindow(t *testing.T) {
	wc := windowSizeConfig{
		serverStream: 1,
		serverConn:   1,
		clientStream: 1,
		clientConn:   1,
	}
	for _, e := range listTestEnv() {
		testConfigurableWindowSize(t, e, wc)
	}
}

func testConfigurableWindowSize(t *testing.T, e env, wc windowSizeConfig) {
	te := newTest(t, e)
	te.serverInitialWindowSize = wc.serverStream
	te.serverInitialConnWindowSize = wc.serverConn
	te.clientInitialWindowSize = wc.clientStream
	te.clientInitialConnWindowSize = wc.clientConn

	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	numOfIter := 11
	// Set message size to exhaust largest of window sizes.
	messageSize := max(max(wc.serverStream, wc.serverConn), max(wc.clientStream, wc.clientConn)) / int32(numOfIter-1)
	messageSize = max(messageSize, 64*1024)
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, messageSize)
	if err != nil {
		t.Fatal(err)
	}
	respParams := []*testpb.ResponseParameters{
		{
			Size: messageSize,
		},
	}
	req := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParams,
		Payload:            payload,
	}
	for i := 0; i < numOfIter; i++ {
		if err := stream.Send(req); err != nil {
			t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, req, err)
		}
		if _, err := stream.Recv(); err != nil {
			t.Fatalf("%v.Recv() = _, %v, want _, <nil>", stream, err)
		}
	}
	if err := stream.CloseSend(); err != nil {
		t.Fatalf("%v.CloseSend() = %v, want <nil>", stream, err)
	}
}

func (s) TestWaitForReadyConnection(t *testing.T) {
	for _, e := range listTestEnv() {
		testWaitForReadyConnection(t, e)
	}

}

func testWaitForReadyConnection(t *testing.T, e env) {
	te := newTest(t, e)
	te.userAgent = testAppUA
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn() // Non-blocking dial.
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testutils.AwaitState(ctx, t, cc, connectivity.Ready)
	// Make a fail-fast RPC.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("TestService/EmptyCall(_,_) = _, %v, want _, nil", err)
	}
}

func (s) TestSvrWriteStatusEarlyWrite(t *testing.T) {
	for _, e := range listTestEnv() {
		testSvrWriteStatusEarlyWrite(t, e)
	}
}

func testSvrWriteStatusEarlyWrite(t *testing.T, e env) {
	te := newTest(t, e)
	const smallSize = 1024
	const largeSize = 2048
	const extraLargeSize = 4096
	te.maxServerReceiveMsgSize = newInt(largeSize)
	te.maxServerSendMsgSize = newInt(largeSize)
	smallPayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, smallSize)
	if err != nil {
		t.Fatal(err)
	}
	extraLargePayload, err := newPayload(testpb.PayloadType_COMPRESSABLE, extraLargeSize)
	if err != nil {
		t.Fatal(err)
	}
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())
	respParam := []*testpb.ResponseParameters{
		{
			Size: int32(smallSize),
		},
	}
	sreq := &testpb.StreamingOutputCallRequest{
		ResponseType:       testpb.PayloadType_COMPRESSABLE,
		ResponseParameters: respParam,
		Payload:            extraLargePayload,
	}
	// Test recv case: server receives a message larger than maxServerReceiveMsgSize.
	stream, err := tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send() = _, %v, want <nil>", stream, err)
	}
	if _, err = stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
	// Test send case: server sends a message larger than maxServerSendMsgSize.
	sreq.Payload = smallPayload
	respParam[0].Size = int32(extraLargeSize)

	stream, err = tc.FullDuplexCall(te.ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want <nil>", tc, err)
	}
	if err = stream.Send(sreq); err != nil {
		t.Fatalf("%v.Send(%v) = %v, want <nil>", stream, sreq, err)
	}
	if _, err = stream.Recv(); err == nil || status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("%v.Recv() = _, %v, want _, error code: %s", stream, err, codes.ResourceExhausted)
	}
}

// TestMalformedStreamMethod starts a test server and sends an RPC with a
// malformed method name. The server should respond with an UNIMPLEMENTED status
// code in this case.
func (s) TestMalformedStreamMethod(t *testing.T) {
	const testMethod = "a-method-name-without-any-slashes"
	te := newTest(t, tcpClearRREnv)
	te.startServer(nil)
	defer te.tearDown()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	err := te.clientConn().Invoke(ctx, testMethod, nil, nil)
	if gotCode := status.Code(err); gotCode != codes.Unimplemented {
		t.Fatalf("Invoke with method %q, got code %s, want %s", testMethod, gotCode, codes.Unimplemented)
	}
}

func (s) TestMethodFromServerStream(t *testing.T) {
	const testMethod = "/package.service/method"
	e := tcpClearRREnv
	te := newTest(t, e)
	var method string
	var ok bool
	te.unknownHandler = func(_ any, stream grpc.ServerStream) error {
		method, ok = grpc.MethodFromServerStream(stream)
		return nil
	}

	te.startServer(nil)
	defer te.tearDown()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	_ = te.clientConn().Invoke(ctx, testMethod, nil, nil)
	if !ok || method != testMethod {
		t.Fatalf("Invoke with method %q, got %q, %v, want %q, true", testMethod, method, ok, testMethod)
	}
}

func (s) TestInterceptorCanAccessCallOptions(t *testing.T) {
	e := tcpClearRREnv
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	type observedOptions struct {
		headers     []*metadata.MD
		trailers    []*metadata.MD
		peer        []*peer.Peer
		creds       []credentials.PerRPCCredentials
		failFast    []bool
		maxRecvSize []int
		maxSendSize []int
		compressor  []string
		subtype     []string
	}
	var observedOpts observedOptions
	populateOpts := func(opts []grpc.CallOption) {
		for _, o := range opts {
			switch o := o.(type) {
			case grpc.HeaderCallOption:
				observedOpts.headers = append(observedOpts.headers, o.HeaderAddr)
			case grpc.TrailerCallOption:
				observedOpts.trailers = append(observedOpts.trailers, o.TrailerAddr)
			case grpc.PeerCallOption:
				observedOpts.peer = append(observedOpts.peer, o.PeerAddr)
			case grpc.PerRPCCredsCallOption:
				observedOpts.creds = append(observedOpts.creds, o.Creds)
			case grpc.FailFastCallOption:
				observedOpts.failFast = append(observedOpts.failFast, o.FailFast)
			case grpc.MaxRecvMsgSizeCallOption:
				observedOpts.maxRecvSize = append(observedOpts.maxRecvSize, o.MaxRecvMsgSize)
			case grpc.MaxSendMsgSizeCallOption:
				observedOpts.maxSendSize = append(observedOpts.maxSendSize, o.MaxSendMsgSize)
			case grpc.CompressorCallOption:
				observedOpts.compressor = append(observedOpts.compressor, o.CompressorType)
			case grpc.ContentSubtypeCallOption:
				observedOpts.subtype = append(observedOpts.subtype, o.ContentSubtype)
			}
		}
	}

	te.unaryClientInt = func(_ context.Context, _ string, _, _ any, _ *grpc.ClientConn, _ grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		populateOpts(opts)
		return nil
	}
	te.streamClientInt = func(_ context.Context, _ *grpc.StreamDesc, _ *grpc.ClientConn, _ string, _ grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		populateOpts(opts)
		return nil, nil
	}

	defaults := []grpc.CallOption{
		grpc.WaitForReady(true),
		grpc.MaxCallRecvMsgSize(1010),
	}
	tc := testgrpc.NewTestServiceClient(te.clientConn(grpc.WithDefaultCallOptions(defaults...)))

	var headers metadata.MD
	var trailers metadata.MD
	var pr peer.Peer
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	tc.UnaryCall(ctx, &testpb.SimpleRequest{},
		grpc.MaxCallRecvMsgSize(100),
		grpc.MaxCallSendMsgSize(200),
		grpc.PerRPCCredentials(testPerRPCCredentials{}),
		grpc.Header(&headers),
		grpc.Trailer(&trailers),
		grpc.Peer(&pr))
	expected := observedOptions{
		failFast:    []bool{false},
		maxRecvSize: []int{1010, 100},
		maxSendSize: []int{200},
		creds:       []credentials.PerRPCCredentials{testPerRPCCredentials{}},
		headers:     []*metadata.MD{&headers},
		trailers:    []*metadata.MD{&trailers},
		peer:        []*peer.Peer{&pr},
	}

	if !reflect.DeepEqual(expected, observedOpts) {
		t.Errorf("unary call did not observe expected options: expected %#v, got %#v", expected, observedOpts)
	}

	observedOpts = observedOptions{} // reset

	tc.StreamingInputCall(ctx,
		grpc.WaitForReady(false),
		grpc.MaxCallSendMsgSize(2020),
		grpc.UseCompressor("comp-type"),
		grpc.CallContentSubtype("json"))
	expected = observedOptions{
		failFast:    []bool{false, true},
		maxRecvSize: []int{1010},
		maxSendSize: []int{2020},
		compressor:  []string{"comp-type"},
		subtype:     []string{"json"},
	}

	if !reflect.DeepEqual(expected, observedOpts) {
		t.Errorf("streaming call did not observe expected options: expected %#v, got %#v", expected, observedOpts)
	}
}

func (s) TestServeExitsWhenListenerClosed(t *testing.T) {
	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return &testpb.Empty{}, nil
		},
	}

	s := grpc.NewServer()
	defer s.Stop()
	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}

	done := make(chan struct{})
	go func() {
		s.Serve(lis)
		close(done)
	}()

	cc, err := grpc.Dial(lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Failed to dial server: %v", err)
	}
	defer cc.Close()
	c := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := c.EmptyCall(ctx, &testpb.Empty{}); err != nil {
		t.Fatalf("Failed to send test RPC to server: %v", err)
	}

	if err := lis.Close(); err != nil {
		t.Fatalf("Failed to close listener: %v", err)
	}
	const timeout = 5 * time.Second
	timer := time.NewTimer(timeout)
	select {
	case <-done:
		return
	case <-timer.C:
		t.Fatalf("Serve did not return after %v", timeout)
	}
}

// Service handler returns status with invalid utf8 message.
func (s) TestStatusInvalidUTF8Message(t *testing.T) {
	var (
		origMsg = string([]byte{0xff, 0xfe, 0xfd})
		wantMsg = "���"
	)

	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			return nil, status.Errorf(codes.Internal, origMsg)
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); status.Convert(err).Message() != wantMsg {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v (msg %q); want _, err with msg %q", err, status.Convert(err).Message(), wantMsg)
	}
}

// Service handler returns status with details and invalid utf8 message. Proto
// will fail to marshal the status because of the invalid utf8 message. Details
// will be dropped when sending.
func (s) TestStatusInvalidUTF8Details(t *testing.T) {
	grpctest.TLogger.ExpectError("Failed to marshal rpc status")

	var (
		origMsg = string([]byte{0xff, 0xfe, 0xfd})
		wantMsg = "���"
	)

	ss := &stubserver.StubServer{
		EmptyCallF: func(context.Context, *testpb.Empty) (*testpb.Empty, error) {
			st := status.New(codes.Internal, origMsg)
			st, err := st.WithDetails(&testpb.Empty{})
			if err != nil {
				return nil, err
			}
			return nil, st.Err()
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	_, err := ss.Client.EmptyCall(ctx, &testpb.Empty{})
	st := status.Convert(err)
	if st.Message() != wantMsg {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v (msg %q); want _, err with msg %q", err, st.Message(), wantMsg)
	}
	if len(st.Details()) != 0 {
		// Details should be dropped on the server side.
		t.Fatalf("RPC status contain details: %v, want no details", st.Details())
	}
}

func (s) TestRPCTimeout(t *testing.T) {
	for _, e := range listTestEnv() {
		testRPCTimeout(t, e)
	}
}

func testRPCTimeout(t *testing.T, e env) {
	te := newTest(t, e)
	te.startServer(&testServer{security: e.security, unaryCallSleepTime: 500 * time.Millisecond})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)

	const argSize = 2718
	const respSize = 314

	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, argSize)
	if err != nil {
		t.Fatal(err)
	}

	req := &testpb.SimpleRequest{
		ResponseType: testpb.PayloadType_COMPRESSABLE,
		ResponseSize: respSize,
		Payload:      payload,
	}
	for i := -1; i <= 10; i++ {
		ctx, cancel := context.WithTimeout(context.Background(), time.Duration(i)*time.Millisecond)
		if _, err := tc.UnaryCall(ctx, req); status.Code(err) != codes.DeadlineExceeded {
			t.Fatalf("TestService/UnaryCallv(_, _) = _, %v; want <nil>, error code: %s", err, codes.DeadlineExceeded)
		}
		cancel()
	}
}

func (s) TestDisabledIOBuffers(t *testing.T) {
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, int32(60000))
	if err != nil {
		t.Fatalf("Failed to create payload: %v", err)
	}
	req := &testpb.StreamingOutputCallRequest{
		Payload: payload,
	}
	resp := &testpb.StreamingOutputCallResponse{
		Payload: payload,
	}

	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			for {
				in, err := stream.Recv()
				if err == io.EOF {
					return nil
				}
				if err != nil {
					t.Errorf("stream.Recv() = _, %v, want _, <nil>", err)
					return err
				}
				if !reflect.DeepEqual(in.Payload.Body, payload.Body) {
					t.Errorf("Received message(len: %v) on server not what was expected(len: %v).", len(in.Payload.Body), len(payload.Body))
					return err
				}
				if err := stream.Send(resp); err != nil {
					t.Errorf("stream.Send(_)= %v, want <nil>", err)
					return err
				}

			}
		},
	}

	s := grpc.NewServer(grpc.WriteBufferSize(0), grpc.ReadBufferSize(0))
	testgrpc.RegisterTestServiceServer(s, ss)

	lis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("Failed to create listener: %v", err)
	}

	go func() {
		s.Serve(lis)
	}()
	defer s.Stop()
	dctx, dcancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer dcancel()
	cc, err := grpc.DialContext(dctx, lis.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithWriteBufferSize(0), grpc.WithReadBufferSize(0))
	if err != nil {
		t.Fatalf("Failed to dial server")
	}
	defer cc.Close()
	c := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := c.FullDuplexCall(ctx, grpc.WaitForReady(true))
	if err != nil {
		t.Fatalf("Failed to send test RPC to server")
	}
	for i := 0; i < 10; i++ {
		if err := stream.Send(req); err != nil {
			t.Fatalf("stream.Send(_) = %v, want <nil>", err)
		}
		in, err := stream.Recv()
		if err != nil {
			t.Fatalf("stream.Recv() = _, %v, want _, <nil>", err)
		}
		if !reflect.DeepEqual(in.Payload.Body, payload.Body) {
			t.Fatalf("Received message(len: %v) on client not what was expected(len: %v).", len(in.Payload.Body), len(payload.Body))
		}
	}
	stream.CloseSend()
	if _, err := stream.Recv(); err != io.EOF {
		t.Fatalf("stream.Recv() = _, %v, want _, io.EOF", err)
	}
}

func (s) TestServerMaxHeaderListSizeClientUserViolation(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler {
			continue
		}
		testServerMaxHeaderListSizeClientUserViolation(t, e)
	}
}

func testServerMaxHeaderListSizeClientUserViolation(t *testing.T, e env) {
	te := newTest(t, e)
	te.maxServerHeaderListSize = new(uint32)
	*te.maxServerHeaderListSize = 216
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	metadata.AppendToOutgoingContext(ctx, "oversize", string(make([]byte, 216)))
	var err error
	if err = verifyResultWithDelay(func() (bool, error) {
		if _, err = tc.EmptyCall(ctx, &testpb.Empty{}); err != nil && status.Code(err) == codes.Internal {
			return true, nil
		}
		return false, fmt.Errorf("tc.EmptyCall() = _, err: %v, want _, error code: %v", err, codes.Internal)
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientMaxHeaderListSizeServerUserViolation(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler {
			continue
		}
		testClientMaxHeaderListSizeServerUserViolation(t, e)
	}
}

func testClientMaxHeaderListSizeServerUserViolation(t *testing.T, e env) {
	te := newTest(t, e)
	te.maxClientHeaderListSize = new(uint32)
	*te.maxClientHeaderListSize = 1 // any header server sends will violate
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc := te.clientConn()
	tc := testgrpc.NewTestServiceClient(cc)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	var err error
	if err = verifyResultWithDelay(func() (bool, error) {
		if _, err = tc.EmptyCall(ctx, &testpb.Empty{}); err != nil && status.Code(err) == codes.Internal {
			return true, nil
		}
		return false, fmt.Errorf("tc.EmptyCall() = _, err: %v, want _, error code: %v", err, codes.Internal)
	}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerMaxHeaderListSizeClientIntentionalViolation(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler || e.security == "tls" {
			continue
		}
		testServerMaxHeaderListSizeClientIntentionalViolation(t, e)
	}
}

func testServerMaxHeaderListSizeClientIntentionalViolation(t *testing.T, e env) {
	te := newTest(t, e)
	te.maxServerHeaderListSize = new(uint32)
	*te.maxServerHeaderListSize = 512
	te.startServer(&testServer{security: e.security})
	defer te.tearDown()

	cc, dw := te.clientConnWithConnControl()
	tc := &testServiceClientWrapper{TestServiceClient: testgrpc.NewTestServiceClient(cc)}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want _, <nil>", tc, err)
	}
	rcw := dw.getRawConnWrapper()
	val := make([]string, 512)
	for i := range val {
		val[i] = "a"
	}
	// allow for client to send the initial header
	time.Sleep(100 * time.Millisecond)
	rcw.writeHeaders(http2.HeadersFrameParam{
		StreamID:      tc.getCurrentStreamID(),
		BlockFragment: rcw.encodeHeader("oversize", strings.Join(val, "")),
		EndStream:     false,
		EndHeaders:    true,
	})
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.Internal {
		t.Fatalf("stream.Recv() = _, %v, want _, error code: %v", err, codes.Internal)
	}
}

func (s) TestClientMaxHeaderListSizeServerIntentionalViolation(t *testing.T) {
	for _, e := range listTestEnv() {
		if e.httpHandler || e.security == "tls" {
			continue
		}
		testClientMaxHeaderListSizeServerIntentionalViolation(t, e)
	}
}

func testClientMaxHeaderListSizeServerIntentionalViolation(t *testing.T, e env) {
	te := newTest(t, e)
	te.maxClientHeaderListSize = new(uint32)
	*te.maxClientHeaderListSize = 200
	lw := te.startServerWithConnControl(&testServer{security: e.security, setHeaderOnly: true})
	defer te.tearDown()
	cc, _ := te.clientConnWithConnControl()
	tc := &testServiceClientWrapper{TestServiceClient: testgrpc.NewTestServiceClient(cc)}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("%v.FullDuplexCall(_) = _, %v, want _, <nil>", tc, err)
	}
	var i int
	var rcw *rawConnWrapper
	for i = 0; i < 100; i++ {
		rcw = lw.getLastConn()
		if rcw != nil {
			break
		}
		time.Sleep(10 * time.Millisecond)
		continue
	}
	if i == 100 {
		t.Fatalf("failed to create server transport after 1s")
	}

	val := make([]string, 200)
	for i := range val {
		val[i] = "a"
	}
	// allow for client to send the initial header.
	time.Sleep(100 * time.Millisecond)
	rcw.writeHeaders(http2.HeadersFrameParam{
		StreamID:      tc.getCurrentStreamID(),
		BlockFragment: rcw.encodeRawHeader("oversize", strings.Join(val, "")),
		EndStream:     false,
		EndHeaders:    true,
	})
	if _, err := stream.Recv(); err == nil || status.Code(err) != codes.Internal {
		t.Fatalf("stream.Recv() = _, %v, want _, error code: %v", err, codes.Internal)
	}
}

func (s) TestNetPipeConn(t *testing.T) {
	// This test will block indefinitely if grpc writes both client and server
	// prefaces without either reading from the Conn.
	pl := testutils.NewPipeListener()
	s := grpc.NewServer()
	defer s.Stop()
	ts := &funcServer{unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
		return &testpb.SimpleResponse{}, nil
	}}
	testgrpc.RegisterTestServiceServer(s, ts)
	go s.Serve(pl)
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	cc, err := grpc.DialContext(ctx, "", grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithDialer(pl.Dialer()))
	if err != nil {
		t.Fatalf("Error creating client: %v", err)
	}
	defer cc.Close()
	client := testgrpc.NewTestServiceClient(cc)
	if _, err := client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("UnaryCall(_) = _, %v; want _, nil", err)
	}
}

func (s) TestLargeTimeout(t *testing.T) {
	for _, e := range listTestEnv() {
		testLargeTimeout(t, e)
	}
}

func testLargeTimeout(t *testing.T, e env) {
	te := newTest(t, e)
	te.declareLogNoise("Server.processUnaryRPC failed to write status")

	ts := &funcServer{}
	te.startServer(ts)
	defer te.tearDown()
	tc := testgrpc.NewTestServiceClient(te.clientConn())

	timeouts := []time.Duration{
		time.Duration(math.MaxInt64), // will be (correctly) converted to
		// 2562048 hours, which overflows upon converting back to an int64
		2562047 * time.Hour, // the largest timeout that does not overflow
	}

	for i, maxTimeout := range timeouts {
		ts.unaryCall = func(ctx context.Context, _ *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			deadline, ok := ctx.Deadline()
			timeout := time.Until(deadline)
			minTimeout := maxTimeout - 5*time.Second
			if !ok || timeout < minTimeout || timeout > maxTimeout {
				t.Errorf("ctx.Deadline() = (now+%v), %v; want [%v, %v], true", timeout, ok, minTimeout, maxTimeout)
				return nil, status.Error(codes.OutOfRange, "deadline error")
			}
			return &testpb.SimpleResponse{}, nil
		}

		ctx, cancel := context.WithTimeout(context.Background(), maxTimeout)
		defer cancel()

		if _, err := tc.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
			t.Errorf("case %v: UnaryCall(_) = _, %v; want _, nil", i, err)
		}
	}
}

func listenWithNotifyingListener(network, address string, event *grpcsync.Event) (net.Listener, error) {
	lis, err := net.Listen(network, address)
	if err != nil {
		return nil, err
	}
	return notifyingListener{connEstablished: event, Listener: lis}, nil
}

type notifyingListener struct {
	connEstablished *grpcsync.Event
	net.Listener
}

func (lis notifyingListener) Accept() (net.Conn, error) {
	defer lis.connEstablished.Fire()
	return lis.Listener.Accept()
}

func (s) TestRPCWaitsForResolver(t *testing.T) {
	te := testServiceConfigSetup(t, tcpClearRREnv)
	te.startServer(&testServer{security: tcpClearRREnv.security})
	defer te.tearDown()
	r := manual.NewBuilderWithScheme("whatever")

	te.resolverScheme = r.Scheme()
	cc := te.clientConn(grpc.WithResolvers(r))
	tc := testgrpc.NewTestServiceClient(cc)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer cancel()
	// With no resolved addresses yet, this will timeout.
	if _, err := tc.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.DeadlineExceeded {
		t.Fatalf("TestService/EmptyCall(_, _) = _, %v, want _, %s", err, codes.DeadlineExceeded)
	}

	ctx, cancel = context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	go func() {
		time.Sleep(time.Second)
		r.UpdateState(resolver.State{
			Addresses: []resolver.Address{{Addr: te.srvAddr}},
			ServiceConfig: parseServiceConfig(t, r, `{
		    "methodConfig": [
		        {
		            "name": [
		                {
		                    "service": "grpc.testing.TestService",
		                    "method": "UnaryCall"
		                }
		            ],
                    "maxRequestMessageBytes": 0
		        }
		    ]
		}`)})
	}()
	// We wait a second before providing a service config and resolving
	// addresses.  So this will wait for that and then honor the
	// maxRequestMessageBytes it contains.
	payload, err := newPayload(testpb.PayloadType_COMPRESSABLE, 1)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tc.UnaryCall(ctx, &testpb.SimpleRequest{Payload: payload}); status.Code(err) != codes.ResourceExhausted {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, nil", err)
	}
	if got := ctx.Err(); got != nil {
		t.Fatalf("ctx.Err() = %v; want nil (deadline should be set short by service config)", got)
	}
	if _, err := tc.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("TestService/UnaryCall(_, _) = _, %v, want _, nil", err)
	}
}

type httpServerResponse struct {
	headers  [][]string
	payload  []byte
	trailers [][]string
}

type httpServer struct {
	// If waitForEndStream is set, wait for the client to send a frame with end
	// stream in it before sending a response/refused stream.
	waitForEndStream bool
	refuseStream     func(uint32) bool
	responses        []httpServerResponse
}

func (s *httpServer) writeHeader(framer *http2.Framer, sid uint32, headerFields []string, endStream bool) error {
	if len(headerFields)%2 == 1 {
		panic("odd number of kv args")
	}

	var buf bytes.Buffer
	henc := hpack.NewEncoder(&buf)
	for len(headerFields) > 0 {
		k, v := headerFields[0], headerFields[1]
		headerFields = headerFields[2:]
		henc.WriteField(hpack.HeaderField{Name: k, Value: v})
	}

	return framer.WriteHeaders(http2.HeadersFrameParam{
		StreamID:      sid,
		BlockFragment: buf.Bytes(),
		EndStream:     endStream,
		EndHeaders:    true,
	})
}

func (s *httpServer) writePayload(framer *http2.Framer, sid uint32, payload []byte) error {
	return framer.WriteData(sid, false, payload)
}

func (s *httpServer) start(t *testing.T, lis net.Listener) {
	// Launch an HTTP server to send back header.
	go func() {
		conn, err := lis.Accept()
		if err != nil {
			t.Errorf("Error accepting connection: %v", err)
			return
		}
		defer conn.Close()
		// Read preface sent by client.
		if _, err = io.ReadFull(conn, make([]byte, len(http2.ClientPreface))); err != nil {
			t.Errorf("Error at server-side while reading preface from client. Err: %v", err)
			return
		}
		reader := bufio.NewReader(conn)
		writer := bufio.NewWriter(conn)
		framer := http2.NewFramer(writer, reader)
		if err = framer.WriteSettingsAck(); err != nil {
			t.Errorf("Error at server-side while sending Settings ack. Err: %v", err)
			return
		}
		writer.Flush() // necessary since client is expecting preface before declaring connection fully setup.
		var sid uint32
		// Loop until framer returns possible conn closed errors.
		for requestNum := 0; ; requestNum = (requestNum + 1) % len(s.responses) {
			// Read frames until a header is received.
			for {
				frame, err := framer.ReadFrame()
				if err != nil {
					if !isConnClosedErr(err) {
						t.Errorf("Error at server-side while reading frame. got: %q, want: rpc error containing substring %q OR %q", err, possibleConnResetMsg, possibleEOFMsg)
					}
					return
				}
				sid = 0
				switch fr := frame.(type) {
				case *http2.HeadersFrame:
					// Respond after this if we are not waiting for an end
					// stream or if this frame ends it.
					if !s.waitForEndStream || fr.StreamEnded() {
						sid = fr.Header().StreamID
					}

				case *http2.DataFrame:
					// Respond after this if we were waiting for an end stream
					// and this frame ends it.  (If we were not waiting for an
					// end stream, this stream was already responded to when
					// the headers were received.)
					if s.waitForEndStream && fr.StreamEnded() {
						sid = fr.Header().StreamID
					}
				}
				if sid != 0 {
					if s.refuseStream == nil || !s.refuseStream(sid) {
						break
					}
					framer.WriteRSTStream(sid, http2.ErrCodeRefusedStream)
					writer.Flush()
				}
			}

			response := s.responses[requestNum]
			for _, header := range response.headers {
				if err = s.writeHeader(framer, sid, header, false); err != nil {
					t.Errorf("Error at server-side while writing headers. Err: %v", err)
					return
				}
				writer.Flush()
			}
			if response.payload != nil {
				if err = s.writePayload(framer, sid, response.payload); err != nil {
					t.Errorf("Error at server-side while writing payload. Err: %v", err)
					return
				}
				writer.Flush()
			}
			for i, trailer := range response.trailers {
				if err = s.writeHeader(framer, sid, trailer, i == len(response.trailers)-1); err != nil {
					t.Errorf("Error at server-side while writing trailers. Err: %v", err)
					return
				}
				writer.Flush()
			}
		}
	}()
}

func (s) TestClientCancellationPropagatesUnary(t *testing.T) {
	wg := &sync.WaitGroup{}
	called, done := make(chan struct{}), make(chan struct{})
	ss := &stubserver.StubServer{
		EmptyCallF: func(ctx context.Context, _ *testpb.Empty) (*testpb.Empty, error) {
			close(called)
			<-ctx.Done()
			err := ctx.Err()
			if err != context.Canceled {
				t.Errorf("ctx.Err() = %v; want context.Canceled", err)
			}
			close(done)
			return nil, err
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)

	wg.Add(1)
	go func() {
		if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.Canceled {
			t.Errorf("ss.Client.EmptyCall() = _, %v; want _, Code()=codes.Canceled", err)
		}
		wg.Done()
	}()

	select {
	case <-called:
	case <-time.After(5 * time.Second):
		t.Fatalf("failed to perform EmptyCall after 10s")
	}
	cancel()
	select {
	case <-done:
	case <-time.After(5 * time.Second):
		t.Fatalf("server failed to close done chan due to cancellation propagation")
	}
	wg.Wait()
}

// When an RPC is canceled, it's possible that the last Recv() returns before
// all call options' after are executed.
func (s) TestCanceledRPCCallOptionRace(t *testing.T) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			err := stream.Send(&testpb.StreamingOutputCallResponse{})
			if err != nil {
				return err
			}
			<-stream.Context().Done()
			return nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	const count = 1000
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	var wg sync.WaitGroup
	wg.Add(count)
	for i := 0; i < count; i++ {
		go func() {
			defer wg.Done()
			var p peer.Peer
			ctx, cancel := context.WithCancel(ctx)
			defer cancel()
			stream, err := ss.Client.FullDuplexCall(ctx, grpc.Peer(&p))
			if err != nil {
				t.Errorf("_.FullDuplexCall(_) = _, %v", err)
				return
			}
			if err := stream.Send(&testpb.StreamingOutputCallRequest{}); err != nil {
				t.Errorf("_ has error %v while sending", err)
				return
			}
			if _, err := stream.Recv(); err != nil {
				t.Errorf("%v.Recv() = %v", stream, err)
				return
			}
			cancel()
			if _, err := stream.Recv(); status.Code(err) != codes.Canceled {
				t.Errorf("%v compleled with error %v, want %s", stream, err, codes.Canceled)
				return
			}
			// If recv returns before call options are executed, peer.Addr is not set,
			// fail the test.
			if p.Addr == nil {
				t.Errorf("peer.Addr is nil, want non-nil")
				return
			}
		}()
	}
	wg.Wait()
}

func (s) TestClientSettingsFloodCloseConn(t *testing.T) {
	// Tests that the server properly closes its transport if the client floods
	// settings frames and then closes the connection.

	// Minimize buffer sizes to stimulate failure condition more quickly.
	s := grpc.NewServer(grpc.WriteBufferSize(20))
	l := bufconn.Listen(20)
	go s.Serve(l)

	// Dial our server and handshake.
	conn, err := l.Dial()
	if err != nil {
		t.Fatalf("Error dialing bufconn: %v", err)
	}

	n, err := conn.Write([]byte(http2.ClientPreface))
	if err != nil || n != len(http2.ClientPreface) {
		t.Fatalf("Error writing client preface: %v, %v", n, err)
	}

	fr := http2.NewFramer(conn, conn)
	f, err := fr.ReadFrame()
	if err != nil {
		t.Fatalf("Error reading initial settings frame: %v", err)
	}
	if _, ok := f.(*http2.SettingsFrame); ok {
		if err := fr.WriteSettingsAck(); err != nil {
			t.Fatalf("Error writing settings ack: %v", err)
		}
	} else {
		t.Fatalf("Error reading initial settings frame: type=%T", f)
	}

	// Confirm settings can be written, and that an ack is read.
	if err = fr.WriteSettings(); err != nil {
		t.Fatalf("Error writing settings frame: %v", err)
	}
	if f, err = fr.ReadFrame(); err != nil {
		t.Fatalf("Error reading frame: %v", err)
	}
	if sf, ok := f.(*http2.SettingsFrame); !ok || !sf.IsAck() {
		t.Fatalf("Unexpected frame: %v", f)
	}

	// Flood settings frames until a timeout occurs, indicating the server has
	// stopped reading from the connection, then close the conn.
	for {
		conn.SetWriteDeadline(time.Now().Add(50 * time.Millisecond))
		if err := fr.WriteSettings(); err != nil {
			if to, ok := err.(interface{ Timeout() bool }); !ok || !to.Timeout() {
				t.Fatalf("Received unexpected write error: %v", err)
			}
			break
		}
	}
	conn.Close()

	// If the server does not handle this situation correctly, it will never
	// close the transport.  This is because its loopyWriter.run() will have
	// exited, and thus not handle the goAway the draining process initiates.
	// Also, we would see a goroutine leak in this case, as the reader would be
	// blocked on the controlBuf's throttle() method indefinitely.

	timer := time.AfterFunc(5*time.Second, func() {
		t.Errorf("Timeout waiting for GracefulStop to return")
		s.Stop()
	})
	s.GracefulStop()
	timer.Stop()
}

func unaryInterceptorVerifyConn(ctx context.Context, _ any, _ *grpc.UnaryServerInfo, _ grpc.UnaryHandler) (any, error) {
	conn := transport.GetConnection(ctx)
	if conn == nil {
		return nil, status.Error(codes.NotFound, "connection was not in context")
	}
	return nil, status.Error(codes.OK, "")
}

// TestUnaryServerInterceptorGetsConnection tests whether the accepted conn on
// the server gets to any unary interceptors on the server side.
func (s) TestUnaryServerInterceptorGetsConnection(t *testing.T) {
	ss := &stubserver.StubServer{}
	if err := ss.Start([]grpc.ServerOption{grpc.UnaryInterceptor(unaryInterceptorVerifyConn)}); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	if _, err := ss.Client.EmptyCall(ctx, &testpb.Empty{}); status.Code(err) != codes.OK {
		t.Fatalf("ss.Client.EmptyCall(_, _) = _, %v, want _, error code %s", err, codes.OK)
	}
}

func streamingInterceptorVerifyConn(_ any, ss grpc.ServerStream, _ *grpc.StreamServerInfo, _ grpc.StreamHandler) error {
	conn := transport.GetConnection(ss.Context())
	if conn == nil {
		return status.Error(codes.NotFound, "connection was not in context")
	}
	return status.Error(codes.OK, "")
}

// TestStreamingServerInterceptorGetsConnection tests whether the accepted conn on
// the server gets to any streaming interceptors on the server side.
func (s) TestStreamingServerInterceptorGetsConnection(t *testing.T) {
	ss := &stubserver.StubServer{}
	if err := ss.Start([]grpc.ServerOption{grpc.StreamInterceptor(streamingInterceptorVerifyConn)}); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	s, err := ss.Client.StreamingOutputCall(ctx, &testpb.StreamingOutputCallRequest{})
	if err != nil {
		t.Fatalf("ss.Client.StreamingOutputCall(_) = _, %v, want _, <nil>", err)
	}
	if _, err := s.Recv(); err != io.EOF {
		t.Fatalf("ss.Client.StreamingInputCall(_) = _, %v, want _, %v", err, io.EOF)
	}
}

// unaryInterceptorVerifyAuthority verifies there is an unambiguous :authority
// once the request gets to an interceptor. An unambiguous :authority is defined
// as at most a single :authority header, and no host header according to A41.
func unaryInterceptorVerifyAuthority(ctx context.Context, _ any, _ *grpc.UnaryServerInfo, _ grpc.UnaryHandler) (any, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if !ok {
		return nil, status.Error(codes.NotFound, "metadata was not in context")
	}
	authority := md.Get(":authority")
	if len(authority) > 1 { // Should be an unambiguous authority by the time it gets to interceptor.
		return nil, status.Error(codes.NotFound, ":authority value had more than one value")
	}
	// Host header shouldn't be present by the time it gets to the interceptor
	// level (should either be renamed to :authority or explicitly deleted).
	host := md.Get("host")
	if len(host) != 0 {
		return nil, status.Error(codes.NotFound, "host header should not be present in metadata")
	}
	// Pass back the authority for verification on client - NotFound so
	// grpc-message will be available to read for verification.
	if len(authority) == 0 {
		// Represent no :authority header present with an empty string.
		return nil, status.Error(codes.NotFound, "")
	}
	return nil, status.Error(codes.NotFound, authority[0])
}

// TestAuthorityHeader tests that the eventual :authority that reaches the grpc
// layer is unambiguous due to logic added in A41.
func (s) TestAuthorityHeader(t *testing.T) {
	tests := []struct {
		name          string
		headers       []string
		wantAuthority string
	}{
		// "If :authority is missing, Host must be renamed to :authority." - A41
		{
			name: "Missing :authority",
			// Codepath triggered by incoming headers with no authority but with
			// a host.
			headers: []string{
				":method", "POST",
				":path", "/grpc.testing.TestService/UnaryCall",
				"content-type", "application/grpc",
				"te", "trailers",
				"host", "localhost",
			},
			wantAuthority: "localhost",
		},
		{
			name: "Missing :authority and host",
			// Codepath triggered by incoming headers with no :authority and no
			// host.
			headers: []string{
				":method", "POST",
				":path", "/grpc.testing.TestService/UnaryCall",
				"content-type", "application/grpc",
				"te", "trailers",
			},
			wantAuthority: "",
		},
		// "If :authority is present, Host must be discarded." - A41
		{
			name: ":authority and host present",
			// Codepath triggered by incoming headers with both an authority
			// header and a host header.
			headers: []string{
				":method", "POST",
				":path", "/grpc.testing.TestService/UnaryCall",
				":authority", "localhost",
				"content-type", "application/grpc",
				"host", "localhost2",
			},
			wantAuthority: "localhost",
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			te := newTest(t, tcpClearRREnv)
			ts := &funcServer{unaryCall: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
				return &testpb.SimpleResponse{}, nil
			}}
			te.unaryServerInt = unaryInterceptorVerifyAuthority
			te.startServer(ts)
			defer te.tearDown()
			success := testutils.NewChannel()
			te.withServerTester(func(st *serverTester) {
				st.writeHeaders(http2.HeadersFrameParam{
					StreamID:      1,
					BlockFragment: st.encodeHeader(test.headers...),
					EndStream:     false,
					EndHeaders:    true,
				})
				st.writeData(1, true, []byte{0, 0, 0, 0, 0})

				for {
					frame := st.wantAnyFrame()
					f, ok := frame.(*http2.MetaHeadersFrame)
					if !ok {
						continue
					}
					for _, header := range f.Fields {
						if header.Name == "grpc-message" {
							success.Send(header.Value)
							return
						}
					}
				}
			})

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			gotAuthority, err := success.Receive(ctx)
			if err != nil {
				t.Fatalf("Error receiving from channel: %v", err)
			}
			if gotAuthority != test.wantAuthority {
				t.Fatalf("gotAuthority: %v, wantAuthority %v", gotAuthority, test.wantAuthority)
			}
		})
	}
}

// wrapCloseListener tracks Accepts/Closes and maintains a counter of the
// number of open connections.
type wrapCloseListener struct {
	net.Listener
	connsOpen int32
}

// wrapCloseListener is returned by wrapCloseListener.Accept and decrements its
// connsOpen when Close is called.
type wrapCloseConn struct {
	net.Conn
	lis       *wrapCloseListener
	closeOnce sync.Once
}

func (w *wrapCloseListener) Accept() (net.Conn, error) {
	conn, err := w.Listener.Accept()
	if err != nil {
		return nil, err
	}
	atomic.AddInt32(&w.connsOpen, 1)
	return &wrapCloseConn{Conn: conn, lis: w}, nil
}

func (w *wrapCloseConn) Close() error {
	defer w.closeOnce.Do(func() { atomic.AddInt32(&w.lis.connsOpen, -1) })
	return w.Conn.Close()
}

// TestServerClosesConn ensures conn.Close is always closed even if the client
// doesn't complete the HTTP/2 handshake.
func (s) TestServerClosesConn(t *testing.T) {
	lis := bufconn.Listen(20)
	wrapLis := &wrapCloseListener{Listener: lis}

	s := grpc.NewServer()
	go s.Serve(wrapLis)
	defer s.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	for i := 0; i < 10; i++ {
		conn, err := lis.DialContext(ctx)
		if err != nil {
			t.Fatalf("Dial = _, %v; want _, nil", err)
		}
		conn.Close()
	}
	for ctx.Err() == nil {
		if atomic.LoadInt32(&wrapLis.connsOpen) == 0 {
			return
		}
		time.Sleep(50 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for conns to be closed by server; still open: %v", atomic.LoadInt32(&wrapLis.connsOpen))
}

// TestNilStatsHandler ensures we do not panic as a result of a nil stats
// handler.
func (s) TestNilStatsHandler(t *testing.T) {
	grpctest.TLogger.ExpectErrorN("ignoring nil parameter", 2)
	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{grpc.StatsHandler(nil)}, grpc.WithStatsHandler(nil)); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("Unexpected error from UnaryCall: %v", err)
	}
}

// TestUnexpectedEOF tests a scenario where a client invokes two unary RPC
// calls. The first call receives a payload which exceeds max grpc receive
// message length, and the second gets a large response. This second RPC should
// not fail with unexpected.EOF.
func (s) TestUnexpectedEOF(t *testing.T) {
	ss := &stubserver.StubServer{
		UnaryCallF: func(_ context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{
				Payload: &testpb.Payload{
					Body: bytes.Repeat([]byte("a"), int(in.ResponseSize)),
				},
			}, nil
		},
	}
	if err := ss.Start([]grpc.ServerOption{}); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < 10; i++ {
		// exceeds grpc.DefaultMaxRecvMessageSize, this should error with
		// RESOURCE_EXHAUSTED error.
		_, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{ResponseSize: 4194304})
		if code := status.Code(err); code != codes.ResourceExhausted {
			t.Fatalf("UnaryCall RPC returned error: %v, want status code %v", err, codes.ResourceExhausted)
		}
		// Larger response that doesn't exceed DefaultMaxRecvMessageSize, this
		// should work normally.
		if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{ResponseSize: 275075}); err != nil {
			t.Fatalf("UnaryCall RPC failed: %v", err)
		}
	}
}

// TestRecvWhileReturningStatus performs a Recv in a service handler while the
// handler returns its status.  A race condition could result in the server
// sending the first headers frame without the HTTP :status header.  This can
// happen when the failed Recv (due to the handler returning) and the handler's
// status both attempt to write the status, which would be the first headers
// frame sent, simultaneously.
func (s) TestRecvWhileReturningStatus(t *testing.T) {
	ss := &stubserver.StubServer{
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			// The client never sends, so this Recv blocks until the server
			// returns and causes stream operations to return errors.
			go stream.Recv()
			return nil
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for i := 0; i < 100; i++ {
		stream, err := ss.Client.FullDuplexCall(ctx)
		if err != nil {
			t.Fatalf("Error while creating stream: %v", err)
		}
		if _, err := stream.Recv(); err != io.EOF {
			t.Fatalf("stream.Recv() = %v, want io.EOF", err)
		}
	}
}

type mockBinaryLogger struct {
	mml *mockMethodLogger
}

func newMockBinaryLogger() *mockBinaryLogger {
	return &mockBinaryLogger{
		mml: &mockMethodLogger{},
	}
}

func (mbl *mockBinaryLogger) GetMethodLogger(string) binarylog.MethodLogger {
	return mbl.mml
}

type mockMethodLogger struct {
	events uint64
}

func (mml *mockMethodLogger) Log(context.Context, binarylog.LogEntryConfig) {
	atomic.AddUint64(&mml.events, 1)
}

// TestGlobalBinaryLoggingOptions tests the binary logging options for client
// and server side. The test configures a binary logger to be plumbed into every
// created ClientConn and server. It then makes a unary RPC call, and a
// streaming RPC call. A certain amount of logging calls should happen as a
// result of the stream operations on each of these calls.
func (s) TestGlobalBinaryLoggingOptions(t *testing.T) {
	csbl := newMockBinaryLogger()
	ssbl := newMockBinaryLogger()

	internal.AddGlobalDialOptions.(func(opt ...grpc.DialOption))(internal.WithBinaryLogger.(func(bl binarylog.Logger) grpc.DialOption)(csbl))
	internal.AddGlobalServerOptions.(func(opt ...grpc.ServerOption))(internal.BinaryLogger.(func(bl binarylog.Logger) grpc.ServerOption)(ssbl))
	defer func() {
		internal.ClearGlobalDialOptions()
		internal.ClearGlobalServerOptions()
	}()
	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
		FullDuplexCallF: func(stream testgrpc.TestService_FullDuplexCallServer) error {
			_, err := stream.Recv()
			if err == io.EOF {
				return nil
			}
			return status.Errorf(codes.Unknown, "expected client to call CloseSend")
		},
	}

	// No client or server options specified, because should pick up configured
	// global options.
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	// Make a Unary RPC. This should cause Log calls on the MethodLogger.
	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("Unexpected error from UnaryCall: %v", err)
	}
	if csbl.mml.events != 5 {
		t.Fatalf("want 5 client side binary logging events, got %v", csbl.mml.events)
	}
	if ssbl.mml.events != 5 {
		t.Fatalf("want 5 server side binary logging events, got %v", ssbl.mml.events)
	}

	// Make a streaming RPC. This should cause Log calls on the MethodLogger.
	stream, err := ss.Client.FullDuplexCall(ctx)
	if err != nil {
		t.Fatalf("ss.Client.FullDuplexCall failed: %f", err)
	}

	stream.CloseSend()
	if _, err = stream.Recv(); err != io.EOF {
		t.Fatalf("unexpected error: %v, expected an EOF error", err)
	}

	if csbl.mml.events != 8 {
		t.Fatalf("want 8 client side binary logging events, got %v", csbl.mml.events)
	}
	if ssbl.mml.events != 8 {
		t.Fatalf("want 8 server side binary logging events, got %v", ssbl.mml.events)
	}
}

type statsHandlerRecordEvents struct {
	mu sync.Mutex
	s  []stats.RPCStats
}

func (*statsHandlerRecordEvents) TagRPC(ctx context.Context, _ *stats.RPCTagInfo) context.Context {
	return ctx
}
func (h *statsHandlerRecordEvents) HandleRPC(_ context.Context, s stats.RPCStats) {
	h.mu.Lock()
	defer h.mu.Unlock()
	h.s = append(h.s, s)
}
func (*statsHandlerRecordEvents) TagConn(ctx context.Context, _ *stats.ConnTagInfo) context.Context {
	return ctx
}
func (*statsHandlerRecordEvents) HandleConn(context.Context, stats.ConnStats) {}

type triggerRPCBlockPicker struct {
	pickDone func()
}

func (bp *triggerRPCBlockPicker) Pick(balancer.PickInfo) (balancer.PickResult, error) {
	bp.pickDone()
	return balancer.PickResult{}, balancer.ErrNoSubConnAvailable
}

const name = "triggerRPCBlockBalancer"

type triggerRPCBlockPickerBalancerBuilder struct{}

func (triggerRPCBlockPickerBalancerBuilder) Build(cc balancer.ClientConn, bOpts balancer.BuildOptions) balancer.Balancer {
	b := &triggerRPCBlockBalancer{
		blockingPickerDone: grpcsync.NewEvent(),
		ClientConn:         cc,
	}
	// round_robin child to complete balancer tree with a usable leaf policy and
	// have RPCs actually work.
	builder := balancer.Get(roundrobin.Name)
	rr := builder.Build(b, bOpts)
	if rr == nil {
		panic("round robin builder returned nil")
	}
	b.Balancer = rr
	return b
}

func (triggerRPCBlockPickerBalancerBuilder) ParseConfig(json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	return &bpbConfig{}, nil
}

func (triggerRPCBlockPickerBalancerBuilder) Name() string {
	return name
}

type bpbConfig struct {
	serviceconfig.LoadBalancingConfig
}

// triggerRPCBlockBalancer uses a child RR balancer, but blocks all UpdateState
// calls until the first Pick call. That first Pick returns
// ErrNoSubConnAvailable to make the RPC block and trigger the appropriate stats
// handler callout. After the first Pick call, it will forward at least one
// READY picker update from the child, causing RPCs to proceed as normal using a
// round robin balancer's picker if it updates with a READY picker.
type triggerRPCBlockBalancer struct {
	stateMu    sync.Mutex
	childState balancer.State

	blockingPickerDone *grpcsync.Event
	// embed a ClientConn to wrap only UpdateState() operation
	balancer.ClientConn
	// embed a Balancer to wrap only UpdateClientConnState() operation
	balancer.Balancer
}

func (bpb *triggerRPCBlockBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	err := bpb.Balancer.UpdateClientConnState(s)
	bpb.ClientConn.UpdateState(balancer.State{
		ConnectivityState: connectivity.Connecting,
		Picker: &triggerRPCBlockPicker{
			pickDone: func() {
				bpb.stateMu.Lock()
				defer bpb.stateMu.Unlock()
				bpb.blockingPickerDone.Fire()
				if bpb.childState.ConnectivityState == connectivity.Ready {
					bpb.ClientConn.UpdateState(bpb.childState)
				}
			},
		},
	})
	return err
}

func (bpb *triggerRPCBlockBalancer) UpdateState(state balancer.State) {
	bpb.stateMu.Lock()
	defer bpb.stateMu.Unlock()
	bpb.childState = state
	if bpb.blockingPickerDone.HasFired() { // guard first one to get a picker sending ErrNoSubConnAvailable first
		if state.ConnectivityState == connectivity.Ready {
			bpb.ClientConn.UpdateState(state) // after the first rr picker update, only forward once READY for deterministic picker counts
		}
	}
}

// TestRPCBlockingOnPickerStatsCall tests the emission of a stats handler call
// that represents the RPC had to block waiting for a new picker due to
// ErrNoSubConnAvailable being returned from the first picker call.
func (s) TestRPCBlockingOnPickerStatsCall(t *testing.T) {
	sh := &statsHandlerRecordEvents{}
	ss := &stubserver.StubServer{
		UnaryCallF: func(context.Context, *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			return &testpb.SimpleResponse{}, nil
		},
	}

	if err := ss.StartServer(); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	lbCfgJSON := `{
  		"loadBalancingConfig": [
    		{
      			"triggerRPCBlockBalancer": {}
    		}
		]
	}`

	sc := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(lbCfgJSON)
	mr := manual.NewBuilderWithScheme("pickerupdatedbalancer")
	defer mr.Close()
	mr.InitialState(resolver.State{
		Addresses: []resolver.Address{
			{Addr: ss.Address},
		},
		ServiceConfig: sc,
	})

	cc, err := grpc.Dial(mr.Scheme()+":///", grpc.WithResolvers(mr), grpc.WithStatsHandler(sh), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("grpc.Dial() failed: %v", err)
	}
	defer cc.Close()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	testServiceClient := testgrpc.NewTestServiceClient(cc)
	if _, err := testServiceClient.UnaryCall(ctx, &testpb.SimpleRequest{}); err != nil {
		t.Fatalf("Unexpected error from UnaryCall: %v", err)
	}

	var pickerUpdatedCount uint
	for _, stat := range sh.s {
		if _, ok := stat.(*stats.PickerUpdated); ok {
			pickerUpdatedCount++
		}
	}
	if pickerUpdatedCount != 1 {
		t.Fatalf("sh.pickerUpdated count: %v, want: %v", pickerUpdatedCount, 2)
	}
}
