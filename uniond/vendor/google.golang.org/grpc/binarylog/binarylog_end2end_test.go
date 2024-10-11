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

package binarylog_test

import (
	"context"
	"fmt"
	"io"
	"net"
	"sort"
	"sync"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/binarylog"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/grpclog"
	iblog "google.golang.org/grpc/internal/binarylog"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/stubserver"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"

	binlogpb "google.golang.org/grpc/binarylog/grpc_binarylog_v1"
	testgrpc "google.golang.org/grpc/interop/grpc_testing"
	testpb "google.golang.org/grpc/interop/grpc_testing"
)

var grpclogLogger = grpclog.Component("binarylog")

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func init() {
	// Setting environment variable in tests doesn't work because of the init
	// orders. Set the loggers directly here.
	iblog.SetLogger(iblog.AllLogger)
	binarylog.SetSink(testSink)
}

var testSink = &testBinLogSink{}

type testBinLogSink struct {
	mu  sync.Mutex
	buf []*binlogpb.GrpcLogEntry
}

func (s *testBinLogSink) Write(e *binlogpb.GrpcLogEntry) error {
	s.mu.Lock()
	s.buf = append(s.buf, e)
	s.mu.Unlock()
	return nil
}

func (s *testBinLogSink) Close() error { return nil }

// Returns all client entries if client is true, otherwise return all server
// entries.
func (s *testBinLogSink) logEntries(client bool) []*binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_SERVER
	if client {
		logger = binlogpb.GrpcLogEntry_LOGGER_CLIENT
	}
	var ret []*binlogpb.GrpcLogEntry
	s.mu.Lock()
	for _, e := range s.buf {
		if e.Logger == logger {
			ret = append(ret, e)
		}
	}
	s.mu.Unlock()
	return ret
}

func (s *testBinLogSink) clear() {
	s.mu.Lock()
	s.buf = nil
	s.mu.Unlock()
}

var (
	// For headers:
	testMetadata = metadata.MD{
		"key1": []string{"value1"},
		"key2": []string{"value2"},
	}
	// For trailers:
	testTrailerMetadata = metadata.MD{
		"tkey1": []string{"trailerValue1"},
		"tkey2": []string{"trailerValue2"},
	}
	// The id for which the service handler should return error.
	errorID int32 = 32202

	globalRPCID uint64 // RPC id starts with 1, but we do ++ at the beginning of each test.
)

func idToPayload(id int32) *testpb.Payload {
	return &testpb.Payload{Body: []byte{byte(id), byte(id >> 8), byte(id >> 16), byte(id >> 24)}}
}

func payloadToID(p *testpb.Payload) int32 {
	if p == nil || len(p.Body) != 4 {
		panic("invalid payload")
	}
	return int32(p.Body[0]) + int32(p.Body[1])<<8 + int32(p.Body[2])<<16 + int32(p.Body[3])<<24
}

type testServer struct {
	testgrpc.UnimplementedTestServiceServer
	te *test
}

func (s *testServer) UnaryCall(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
	md, ok := metadata.FromIncomingContext(ctx)
	if ok {
		if err := grpc.SendHeader(ctx, md); err != nil {
			return nil, status.Errorf(status.Code(err), "grpc.SendHeader(_, %v) = %v, want <nil>", md, err)
		}
		if err := grpc.SetTrailer(ctx, testTrailerMetadata); err != nil {
			return nil, status.Errorf(status.Code(err), "grpc.SetTrailer(_, %v) = %v, want <nil>", testTrailerMetadata, err)
		}
	}

	if id := payloadToID(in.Payload); id == errorID {
		return nil, fmt.Errorf("got error id: %v", id)
	}

	return &testpb.SimpleResponse{Payload: in.Payload}, nil
}

func (s *testServer) FullDuplexCall(stream testgrpc.TestService_FullDuplexCallServer) error {
	md, ok := metadata.FromIncomingContext(stream.Context())
	if ok {
		if err := stream.SendHeader(md); err != nil {
			return status.Errorf(status.Code(err), "stream.SendHeader(%v) = %v, want %v", md, err, nil)
		}
		stream.SetTrailer(testTrailerMetadata)
	}
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			return nil
		}
		if err != nil {
			return err
		}

		if id := payloadToID(in.Payload); id == errorID {
			return fmt.Errorf("got error id: %v", id)
		}

		if err := stream.Send(&testpb.StreamingOutputCallResponse{Payload: in.Payload}); err != nil {
			return err
		}
	}
}

func (s *testServer) StreamingInputCall(stream testgrpc.TestService_StreamingInputCallServer) error {
	md, ok := metadata.FromIncomingContext(stream.Context())
	if ok {
		if err := stream.SendHeader(md); err != nil {
			return status.Errorf(status.Code(err), "stream.SendHeader(%v) = %v, want %v", md, err, nil)
		}
		stream.SetTrailer(testTrailerMetadata)
	}
	for {
		in, err := stream.Recv()
		if err == io.EOF {
			// read done.
			return stream.SendAndClose(&testpb.StreamingInputCallResponse{AggregatedPayloadSize: 0})
		}
		if err != nil {
			return err
		}

		if id := payloadToID(in.Payload); id == errorID {
			return fmt.Errorf("got error id: %v", id)
		}
	}
}

func (s *testServer) StreamingOutputCall(in *testpb.StreamingOutputCallRequest, stream testgrpc.TestService_StreamingOutputCallServer) error {
	md, ok := metadata.FromIncomingContext(stream.Context())
	if ok {
		if err := stream.SendHeader(md); err != nil {
			return status.Errorf(status.Code(err), "stream.SendHeader(%v) = %v, want %v", md, err, nil)
		}
		stream.SetTrailer(testTrailerMetadata)
	}

	if id := payloadToID(in.Payload); id == errorID {
		return fmt.Errorf("got error id: %v", id)
	}

	for i := 0; i < 5; i++ {
		if err := stream.Send(&testpb.StreamingOutputCallResponse{Payload: in.Payload}); err != nil {
			return err
		}
	}
	return nil
}

// test is an end-to-end test. It should be created with the newTest
// func, modified as needed, and then started with its startServer method.
// It should be cleaned up with the tearDown method.
type test struct {
	t *testing.T

	testService testgrpc.TestServiceServer // nil means none
	// srv and srvAddr are set once startServer is called.
	srv     *grpc.Server
	srvAddr string // Server IP without port.
	srvIP   net.IP
	srvPort int

	cc *grpc.ClientConn // nil until requested via clientConn

	// Fields for client address. Set by the service handler.
	clientAddrMu sync.Mutex
	clientIP     net.IP
	clientPort   int
}

func (te *test) tearDown() {
	if te.cc != nil {
		te.cc.Close()
		te.cc = nil
	}
	te.srv.Stop()
}

// newTest returns a new test using the provided testing.T and
// environment.  It is returned with default values. Tests should
// modify it before calling its startServer and clientConn methods.
func newTest(t *testing.T) *test {
	te := &test{
		t: t,
	}
	return te
}

type listenerWrapper struct {
	net.Listener
	te *test
}

func (lw *listenerWrapper) Accept() (net.Conn, error) {
	conn, err := lw.Listener.Accept()
	if err != nil {
		return nil, err
	}
	lw.te.clientAddrMu.Lock()
	lw.te.clientIP = conn.RemoteAddr().(*net.TCPAddr).IP
	lw.te.clientPort = conn.RemoteAddr().(*net.TCPAddr).Port
	lw.te.clientAddrMu.Unlock()
	return conn, nil
}

// startServer starts a gRPC server listening. Callers should defer a
// call to te.tearDown to clean up.
func (te *test) startServer(ts testgrpc.TestServiceServer) {
	te.testService = ts
	lis, err := net.Listen("tcp", "localhost:0")

	lis = &listenerWrapper{
		Listener: lis,
		te:       te,
	}

	if err != nil {
		te.t.Fatalf("Failed to listen: %v", err)
	}
	var opts []grpc.ServerOption
	s := grpc.NewServer(opts...)
	te.srv = s
	if te.testService != nil {
		testgrpc.RegisterTestServiceServer(s, te.testService)
	}

	go s.Serve(lis)
	te.srvAddr = lis.Addr().String()
	te.srvIP = lis.Addr().(*net.TCPAddr).IP
	te.srvPort = lis.Addr().(*net.TCPAddr).Port
}

func (te *test) clientConn() *grpc.ClientConn {
	if te.cc != nil {
		return te.cc
	}
	opts := []grpc.DialOption{grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock()}

	var err error
	te.cc, err = grpc.NewClient(te.srvAddr, opts...)
	if err != nil {
		te.t.Fatalf("Dial(%q) = %v", te.srvAddr, err)
	}
	return te.cc
}

type rpcType int

const (
	unaryRPC rpcType = iota
	clientStreamRPC
	serverStreamRPC
	fullDuplexStreamRPC
	cancelRPC
)

type rpcConfig struct {
	count    int     // Number of requests and responses for streaming RPCs.
	success  bool    // Whether the RPC should succeed or return error.
	callType rpcType // Type of RPC.
}

func (te *test) doUnaryCall(c *rpcConfig) (*testpb.SimpleRequest, *testpb.SimpleResponse, error) {
	var (
		resp *testpb.SimpleResponse
		req  *testpb.SimpleRequest
		err  error
	)
	tc := testgrpc.NewTestServiceClient(te.clientConn())
	if c.success {
		req = &testpb.SimpleRequest{Payload: idToPayload(errorID + 1)}
	} else {
		req = &testpb.SimpleRequest{Payload: idToPayload(errorID)}
	}
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)

	resp, err = tc.UnaryCall(ctx, req)
	return req, resp, err
}

func (te *test) doFullDuplexCallRoundtrip(c *rpcConfig) ([]proto.Message, []proto.Message, error) {
	var (
		reqs  []proto.Message
		resps []proto.Message
		err   error
	)
	tc := testgrpc.NewTestServiceClient(te.clientConn())
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)

	stream, err := tc.FullDuplexCall(ctx)
	if err != nil {
		return reqs, resps, err
	}

	if c.callType == cancelRPC {
		cancel()
		return reqs, resps, context.Canceled
	}

	var startID int32
	if !c.success {
		startID = errorID
	}
	for i := 0; i < c.count; i++ {
		req := &testpb.StreamingOutputCallRequest{
			Payload: idToPayload(int32(i) + startID),
		}
		reqs = append(reqs, req)
		if err = stream.Send(req); err != nil {
			return reqs, resps, err
		}
		var resp *testpb.StreamingOutputCallResponse
		if resp, err = stream.Recv(); err != nil {
			return reqs, resps, err
		}
		resps = append(resps, resp)
	}
	if err = stream.CloseSend(); err != nil && err != io.EOF {
		return reqs, resps, err
	}
	if _, err = stream.Recv(); err != io.EOF {
		return reqs, resps, err
	}

	return reqs, resps, nil
}

func (te *test) doClientStreamCall(c *rpcConfig) ([]proto.Message, proto.Message, error) {
	var (
		reqs []proto.Message
		resp *testpb.StreamingInputCallResponse
		err  error
	)
	tc := testgrpc.NewTestServiceClient(te.clientConn())
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)

	stream, err := tc.StreamingInputCall(ctx)
	if err != nil {
		return reqs, resp, err
	}
	var startID int32
	if !c.success {
		startID = errorID
	}
	for i := 0; i < c.count; i++ {
		req := &testpb.StreamingInputCallRequest{
			Payload: idToPayload(int32(i) + startID),
		}
		reqs = append(reqs, req)
		if err = stream.Send(req); err != nil {
			return reqs, resp, err
		}
	}
	resp, err = stream.CloseAndRecv()
	return reqs, resp, err
}

func (te *test) doServerStreamCall(c *rpcConfig) (proto.Message, []proto.Message, error) {
	var (
		req   *testpb.StreamingOutputCallRequest
		resps []proto.Message
		err   error
	)

	tc := testgrpc.NewTestServiceClient(te.clientConn())
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	ctx = metadata.NewOutgoingContext(ctx, testMetadata)

	var startID int32
	if !c.success {
		startID = errorID
	}
	req = &testpb.StreamingOutputCallRequest{Payload: idToPayload(startID)}
	stream, err := tc.StreamingOutputCall(ctx, req)
	if err != nil {
		return req, resps, err
	}
	for {
		var resp *testpb.StreamingOutputCallResponse
		resp, err := stream.Recv()
		if err == io.EOF {
			return req, resps, nil
		} else if err != nil {
			return req, resps, err
		}
		resps = append(resps, resp)
	}
}

type expectedData struct {
	te *test
	cc *rpcConfig

	method    string
	requests  []proto.Message
	responses []proto.Message
	err       error
}

func (ed *expectedData) newClientHeaderEntry(client bool, rpcID, inRPCID uint64) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_CLIENT
	var peer *binlogpb.Address
	if !client {
		logger = binlogpb.GrpcLogEntry_LOGGER_SERVER
		ed.te.clientAddrMu.Lock()
		peer = &binlogpb.Address{
			Address: ed.te.clientIP.String(),
			IpPort:  uint32(ed.te.clientPort),
		}
		if ed.te.clientIP.To4() != nil {
			peer.Type = binlogpb.Address_TYPE_IPV4
		} else {
			peer.Type = binlogpb.Address_TYPE_IPV6
		}
		ed.te.clientAddrMu.Unlock()
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HEADER,
		Logger:               logger,
		Payload: &binlogpb.GrpcLogEntry_ClientHeader{
			ClientHeader: &binlogpb.ClientHeader{
				Metadata:   iblog.MdToMetadataProto(testMetadata),
				MethodName: ed.method,
				Authority:  ed.te.srvAddr,
			},
		},
		Peer: peer,
	}
}

func (ed *expectedData) newServerHeaderEntry(client bool, rpcID, inRPCID uint64) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_SERVER
	var peer *binlogpb.Address
	if client {
		logger = binlogpb.GrpcLogEntry_LOGGER_CLIENT
		peer = &binlogpb.Address{
			Address: ed.te.srvIP.String(),
			IpPort:  uint32(ed.te.srvPort),
		}
		if ed.te.srvIP.To4() != nil {
			peer.Type = binlogpb.Address_TYPE_IPV4
		} else {
			peer.Type = binlogpb.Address_TYPE_IPV6
		}
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_HEADER,
		Logger:               logger,
		Payload: &binlogpb.GrpcLogEntry_ServerHeader{
			ServerHeader: &binlogpb.ServerHeader{
				Metadata: iblog.MdToMetadataProto(testMetadata),
			},
		},
		Peer: peer,
	}
}

func (ed *expectedData) newClientMessageEntry(client bool, rpcID, inRPCID uint64, msg proto.Message) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_CLIENT
	if !client {
		logger = binlogpb.GrpcLogEntry_LOGGER_SERVER
	}
	data, err := proto.Marshal(msg)
	if err != nil {
		grpclogLogger.Infof("binarylogging_testing: failed to marshal proto message: %v", err)
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_MESSAGE,
		Logger:               logger,
		Payload: &binlogpb.GrpcLogEntry_Message{
			Message: &binlogpb.Message{
				Length: uint32(len(data)),
				Data:   data,
			},
		},
	}
}

func (ed *expectedData) newServerMessageEntry(client bool, rpcID, inRPCID uint64, msg proto.Message) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_CLIENT
	if !client {
		logger = binlogpb.GrpcLogEntry_LOGGER_SERVER
	}
	data, err := proto.Marshal(msg)
	if err != nil {
		grpclogLogger.Infof("binarylogging_testing: failed to marshal proto message: %v", err)
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_MESSAGE,
		Logger:               logger,
		Payload: &binlogpb.GrpcLogEntry_Message{
			Message: &binlogpb.Message{
				Length: uint32(len(data)),
				Data:   data,
			},
		},
	}
}

func (ed *expectedData) newHalfCloseEntry(client bool, rpcID, inRPCID uint64) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_CLIENT
	if !client {
		logger = binlogpb.GrpcLogEntry_LOGGER_SERVER
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CLIENT_HALF_CLOSE,
		Payload:              nil, // No payload here.
		Logger:               logger,
	}
}

func (ed *expectedData) newServerTrailerEntry(client bool, rpcID, inRPCID uint64, stErr error) *binlogpb.GrpcLogEntry {
	logger := binlogpb.GrpcLogEntry_LOGGER_SERVER
	var peer *binlogpb.Address
	if client {
		logger = binlogpb.GrpcLogEntry_LOGGER_CLIENT
		peer = &binlogpb.Address{
			Address: ed.te.srvIP.String(),
			IpPort:  uint32(ed.te.srvPort),
		}
		if ed.te.srvIP.To4() != nil {
			peer.Type = binlogpb.Address_TYPE_IPV4
		} else {
			peer.Type = binlogpb.Address_TYPE_IPV6
		}
	}
	st, ok := status.FromError(stErr)
	if !ok {
		grpclogLogger.Info("binarylogging: error in trailer is not a status error")
	}
	stProto := st.Proto()
	var (
		detailsBytes []byte
		err          error
	)
	if stProto != nil && len(stProto.Details) != 0 {
		detailsBytes, err = proto.Marshal(stProto)
		if err != nil {
			grpclogLogger.Infof("binarylogging: failed to marshal status proto: %v", err)
		}
	}
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_TRAILER,
		Logger:               logger,
		Payload: &binlogpb.GrpcLogEntry_Trailer{
			Trailer: &binlogpb.Trailer{
				Metadata: iblog.MdToMetadataProto(testTrailerMetadata),
				// st will be nil if err was not a status error, but nil is ok.
				StatusCode:    uint32(st.Code()),
				StatusMessage: st.Message(),
				StatusDetails: detailsBytes,
			},
		},
		Peer: peer,
	}
}

func (ed *expectedData) newCancelEntry(rpcID, inRPCID uint64) *binlogpb.GrpcLogEntry {
	return &binlogpb.GrpcLogEntry{
		Timestamp:            nil,
		CallId:               rpcID,
		SequenceIdWithinCall: inRPCID,
		Type:                 binlogpb.GrpcLogEntry_EVENT_TYPE_CANCEL,
		Logger:               binlogpb.GrpcLogEntry_LOGGER_CLIENT,
		Payload:              nil,
	}
}

func (ed *expectedData) toClientLogEntries() []*binlogpb.GrpcLogEntry {
	var (
		ret     []*binlogpb.GrpcLogEntry
		idInRPC uint64 = 1
	)
	ret = append(ret, ed.newClientHeaderEntry(true, globalRPCID, idInRPC))
	idInRPC++

	switch ed.cc.callType {
	case unaryRPC, fullDuplexStreamRPC:
		for i := 0; i < len(ed.requests); i++ {
			ret = append(ret, ed.newClientMessageEntry(true, globalRPCID, idInRPC, ed.requests[i]))
			idInRPC++
			if i == 0 {
				// First message, append ServerHeader.
				ret = append(ret, ed.newServerHeaderEntry(true, globalRPCID, idInRPC))
				idInRPC++
			}
			if !ed.cc.success {
				// There is no response in the RPC error case.
				continue
			}
			ret = append(ret, ed.newServerMessageEntry(true, globalRPCID, idInRPC, ed.responses[i]))
			idInRPC++
		}
		if ed.cc.success && ed.cc.callType == fullDuplexStreamRPC {
			ret = append(ret, ed.newHalfCloseEntry(true, globalRPCID, idInRPC))
			idInRPC++
		}
	case clientStreamRPC, serverStreamRPC:
		for i := 0; i < len(ed.requests); i++ {
			ret = append(ret, ed.newClientMessageEntry(true, globalRPCID, idInRPC, ed.requests[i]))
			idInRPC++
		}
		if ed.cc.callType == clientStreamRPC {
			ret = append(ret, ed.newHalfCloseEntry(true, globalRPCID, idInRPC))
			idInRPC++
		}
		ret = append(ret, ed.newServerHeaderEntry(true, globalRPCID, idInRPC))
		idInRPC++
		if ed.cc.success {
			for i := 0; i < len(ed.responses); i++ {
				ret = append(ret, ed.newServerMessageEntry(true, globalRPCID, idInRPC, ed.responses[0]))
				idInRPC++
			}
		}
	}

	if ed.cc.callType == cancelRPC {
		ret = append(ret, ed.newCancelEntry(globalRPCID, idInRPC))
		idInRPC++
	} else {
		ret = append(ret, ed.newServerTrailerEntry(true, globalRPCID, idInRPC, ed.err))
		idInRPC++
	}
	return ret
}

func (ed *expectedData) toServerLogEntries() []*binlogpb.GrpcLogEntry {
	var (
		ret     []*binlogpb.GrpcLogEntry
		idInRPC uint64 = 1
	)
	ret = append(ret, ed.newClientHeaderEntry(false, globalRPCID, idInRPC))
	idInRPC++

	switch ed.cc.callType {
	case unaryRPC:
		ret = append(ret, ed.newClientMessageEntry(false, globalRPCID, idInRPC, ed.requests[0]))
		idInRPC++
		ret = append(ret, ed.newServerHeaderEntry(false, globalRPCID, idInRPC))
		idInRPC++
		if ed.cc.success {
			ret = append(ret, ed.newServerMessageEntry(false, globalRPCID, idInRPC, ed.responses[0]))
			idInRPC++
		}
	case fullDuplexStreamRPC:
		ret = append(ret, ed.newServerHeaderEntry(false, globalRPCID, idInRPC))
		idInRPC++
		for i := 0; i < len(ed.requests); i++ {
			ret = append(ret, ed.newClientMessageEntry(false, globalRPCID, idInRPC, ed.requests[i]))
			idInRPC++
			if !ed.cc.success {
				// There is no response in the RPC error case.
				continue
			}
			ret = append(ret, ed.newServerMessageEntry(false, globalRPCID, idInRPC, ed.responses[i]))
			idInRPC++
		}

		if ed.cc.success && ed.cc.callType == fullDuplexStreamRPC {
			ret = append(ret, ed.newHalfCloseEntry(false, globalRPCID, idInRPC))
			idInRPC++
		}
	case clientStreamRPC:
		ret = append(ret, ed.newServerHeaderEntry(false, globalRPCID, idInRPC))
		idInRPC++
		for i := 0; i < len(ed.requests); i++ {
			ret = append(ret, ed.newClientMessageEntry(false, globalRPCID, idInRPC, ed.requests[i]))
			idInRPC++
		}
		if ed.cc.success {
			ret = append(ret, ed.newHalfCloseEntry(false, globalRPCID, idInRPC))
			idInRPC++
			ret = append(ret, ed.newServerMessageEntry(false, globalRPCID, idInRPC, ed.responses[0]))
			idInRPC++
		}
	case serverStreamRPC:
		ret = append(ret, ed.newClientMessageEntry(false, globalRPCID, idInRPC, ed.requests[0]))
		idInRPC++
		ret = append(ret, ed.newServerHeaderEntry(false, globalRPCID, idInRPC))
		idInRPC++
		for i := 0; i < len(ed.responses); i++ {
			ret = append(ret, ed.newServerMessageEntry(false, globalRPCID, idInRPC, ed.responses[0]))
			idInRPC++
		}
	}

	ret = append(ret, ed.newServerTrailerEntry(false, globalRPCID, idInRPC, ed.err))
	idInRPC++

	return ret
}

func runRPCs(t *testing.T, cc *rpcConfig) *expectedData {
	te := newTest(t)
	te.startServer(&testServer{te: te})
	defer te.tearDown()

	expect := &expectedData{
		te: te,
		cc: cc,
	}

	switch cc.callType {
	case unaryRPC:
		expect.method = "/grpc.testing.TestService/UnaryCall"
		req, resp, err := te.doUnaryCall(cc)
		expect.requests = []proto.Message{req}
		expect.responses = []proto.Message{resp}
		expect.err = err
	case clientStreamRPC:
		expect.method = "/grpc.testing.TestService/StreamingInputCall"
		reqs, resp, err := te.doClientStreamCall(cc)
		expect.requests = reqs
		expect.responses = []proto.Message{resp}
		expect.err = err
	case serverStreamRPC:
		expect.method = "/grpc.testing.TestService/StreamingOutputCall"
		req, resps, err := te.doServerStreamCall(cc)
		expect.responses = resps
		expect.requests = []proto.Message{req}
		expect.err = err
	case fullDuplexStreamRPC, cancelRPC:
		expect.method = "/grpc.testing.TestService/FullDuplexCall"
		expect.requests, expect.responses, expect.err = te.doFullDuplexCallRoundtrip(cc)
	}
	if cc.success != (expect.err == nil) {
		t.Fatalf("cc.success: %v, got error: %v", cc.success, expect.err)
	}
	te.cc.Close()
	te.srv.GracefulStop() // Wait for the server to stop.

	return expect
}

// equalLogEntry sorts the metadata entries by key (to compare metadata).
//
// This function is typically called with only two entries. It's written in this
// way so the code can be put in a for loop instead of copied twice.
func equalLogEntry(entries ...*binlogpb.GrpcLogEntry) (equal bool) {
	for i, e := range entries {
		// Clear out some fields we don't compare.
		e.Timestamp = nil
		e.CallId = 0 // CallID is global to the binary, hard to compare.
		if h := e.GetClientHeader(); h != nil {
			h.Timeout = nil
			tmp := append(h.Metadata.Entry[:0], h.Metadata.Entry...)
			h.Metadata.Entry = tmp
			sort.Slice(h.Metadata.Entry, func(i, j int) bool { return h.Metadata.Entry[i].Key < h.Metadata.Entry[j].Key })
		}
		if h := e.GetServerHeader(); h != nil {
			tmp := append(h.Metadata.Entry[:0], h.Metadata.Entry...)
			h.Metadata.Entry = tmp
			sort.Slice(h.Metadata.Entry, func(i, j int) bool { return h.Metadata.Entry[i].Key < h.Metadata.Entry[j].Key })
		}
		if h := e.GetTrailer(); h != nil {
			sort.Slice(h.Metadata.Entry, func(i, j int) bool { return h.Metadata.Entry[i].Key < h.Metadata.Entry[j].Key })
		}

		if i > 0 && !proto.Equal(e, entries[i-1]) {
			return false
		}
	}
	return true
}

func testClientBinaryLog(t *testing.T, c *rpcConfig) error {
	defer testSink.clear()
	expect := runRPCs(t, c)
	want := expect.toClientLogEntries()
	var got []*binlogpb.GrpcLogEntry
	// In racy cases, some entries are not logged when the RPC is finished (e.g.
	// context.Cancel).
	//
	// Check 10 times, with a sleep of 1/100 seconds between each check. Makes
	// it an 1-second wait in total.
	for i := 0; i < 10; i++ {
		got = testSink.logEntries(true) // all client entries.
		if len(want) == len(got) {
			break
		}
		time.Sleep(100 * time.Millisecond)
	}
	if len(want) != len(got) {
		for i, e := range want {
			t.Errorf("in want: %d, %s", i, e.GetType())
		}
		for i, e := range got {
			t.Errorf("in got: %d, %s", i, e.GetType())
		}
		return fmt.Errorf("didn't get same amount of log entries, want: %d, got: %d", len(want), len(got))
	}
	var errored bool
	for i := 0; i < len(got); i++ {
		if !equalLogEntry(want[i], got[i]) {
			t.Errorf("entry: %d, want %+v, got %+v", i, want[i], got[i])
			errored = true
		}
	}
	if errored {
		return fmt.Errorf("test failed")
	}
	return nil
}

func (s) TestClientBinaryLogUnaryRPC(t *testing.T) {
	if err := testClientBinaryLog(t, &rpcConfig{success: true, callType: unaryRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogUnaryRPCError(t *testing.T) {
	if err := testClientBinaryLog(t, &rpcConfig{success: false, callType: unaryRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogClientStreamRPC(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: true, callType: clientStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogClientStreamRPCError(t *testing.T) {
	count := 1
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: false, callType: clientStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogServerStreamRPC(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: true, callType: serverStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogServerStreamRPCError(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: false, callType: serverStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogFullDuplexRPC(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: true, callType: fullDuplexStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogFullDuplexRPCError(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: false, callType: fullDuplexStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestClientBinaryLogCancel(t *testing.T) {
	count := 5
	if err := testClientBinaryLog(t, &rpcConfig{count: count, success: false, callType: cancelRPC}); err != nil {
		t.Fatal(err)
	}
}

func testServerBinaryLog(t *testing.T, c *rpcConfig) error {
	defer testSink.clear()
	expect := runRPCs(t, c)
	want := expect.toServerLogEntries()
	var got []*binlogpb.GrpcLogEntry
	// In racy cases, some entries are not logged when the RPC is finished (e.g.
	// context.Cancel). This is unlikely to happen on server side, but it does
	// no harm to retry.
	//
	// Check 10 times, with a sleep of 1/100 seconds between each check. Makes
	// it an 1-second wait in total.
	for i := 0; i < 10; i++ {
		got = testSink.logEntries(false) // all server entries.
		if len(want) == len(got) {
			break
		}
		time.Sleep(100 * time.Millisecond)
	}

	if len(want) != len(got) {
		for i, e := range want {
			t.Errorf("in want: %d, %s", i, e.GetType())
		}
		for i, e := range got {
			t.Errorf("in got: %d, %s", i, e.GetType())
		}
		return fmt.Errorf("didn't get same amount of log entries, want: %d, got: %d", len(want), len(got))
	}
	var errored bool
	for i := 0; i < len(got); i++ {
		if !equalLogEntry(want[i], got[i]) {
			t.Errorf("entry: %d, want %+v, got %+v", i, want[i], got[i])
			errored = true
		}
	}
	if errored {
		return fmt.Errorf("test failed")
	}
	return nil
}

func (s) TestServerBinaryLogUnaryRPC(t *testing.T) {
	if err := testServerBinaryLog(t, &rpcConfig{success: true, callType: unaryRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogUnaryRPCError(t *testing.T) {
	if err := testServerBinaryLog(t, &rpcConfig{success: false, callType: unaryRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogClientStreamRPC(t *testing.T) {
	count := 5
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: true, callType: clientStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogClientStreamRPCError(t *testing.T) {
	count := 1
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: false, callType: clientStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogServerStreamRPC(t *testing.T) {
	count := 5
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: true, callType: serverStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogServerStreamRPCError(t *testing.T) {
	count := 5
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: false, callType: serverStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogFullDuplex(t *testing.T) {
	count := 5
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: true, callType: fullDuplexStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

func (s) TestServerBinaryLogFullDuplexError(t *testing.T) {
	count := 5
	if err := testServerBinaryLog(t, &rpcConfig{count: count, success: false, callType: fullDuplexStreamRPC}); err != nil {
		t.Fatal(err)
	}
}

// TestCanceledStatus ensures a server that responds with a Canceled status has
// its trailers logged appropriately and is not treated as a canceled RPC.
func (s) TestCanceledStatus(t *testing.T) {
	defer testSink.clear()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	const statusMsgWant = "server returned Canceled"
	ss := &stubserver.StubServer{
		UnaryCallF: func(ctx context.Context, in *testpb.SimpleRequest) (*testpb.SimpleResponse, error) {
			grpc.SetTrailer(ctx, metadata.Pairs("key", "value"))
			return nil, status.Error(codes.Canceled, statusMsgWant)
		},
	}
	if err := ss.Start(nil); err != nil {
		t.Fatalf("Error starting endpoint server: %v", err)
	}
	defer ss.Stop()

	if _, err := ss.Client.UnaryCall(ctx, &testpb.SimpleRequest{}); status.Code(err) != codes.Canceled {
		t.Fatalf("Received unexpected error from UnaryCall: %v; want Canceled", err)
	}

	got := testSink.logEntries(true)
	last := got[len(got)-1]
	if last.Type != binlogpb.GrpcLogEntry_EVENT_TYPE_SERVER_TRAILER ||
		last.GetTrailer().GetStatusCode() != uint32(codes.Canceled) ||
		last.GetTrailer().GetStatusMessage() != statusMsgWant ||
		len(last.GetTrailer().GetMetadata().GetEntry()) != 1 ||
		last.GetTrailer().GetMetadata().GetEntry()[0].GetKey() != "key" ||
		string(last.GetTrailer().GetMetadata().GetEntry()[0].GetValue()) != "value" {
		t.Fatalf("Got binary log: %+v; want last entry is server trailing with status Canceled", got)
	}
}
