package grpc_zap_test

import (
	"context"
	"io"
	"runtime"
	"strings"
	"testing"

	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"

	"github.com/grpc-ecosystem/go-grpc-middleware"
	"github.com/grpc-ecosystem/go-grpc-middleware/tags"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/grpc-ecosystem/go-grpc-middleware/logging/zap"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

func TestZapPayloadSuite(t *testing.T) {
	if strings.HasPrefix(runtime.Version(), "go1.7") {
		t.Skipf("Skipping due to json.RawMessage incompatibility with go1.7")
		return
	}

	alwaysLoggingDeciderServer := func(ctx context.Context, fullMethodName string, servingObject interface{}) bool { return true }
	alwaysLoggingDeciderClient := func(ctx context.Context, fullMethodName string) bool { return true }

	b := newBaseZapSuite(t)
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_zap.PayloadUnaryClientInterceptor(b.log, alwaysLoggingDeciderClient)),
		grpc.WithStreamInterceptor(grpc_zap.PayloadStreamClientInterceptor(b.log, alwaysLoggingDeciderClient)),
	}
	noOpZap := zap.New(zapcore.NewNopCore())
	b.InterceptorTestSuite.ServerOpts = []grpc.ServerOption{
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
			grpc_zap.StreamServerInterceptor(noOpZap),
			grpc_zap.PayloadStreamServerInterceptor(b.log, alwaysLoggingDeciderServer)),
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
			grpc_zap.UnaryServerInterceptor(noOpZap),
			grpc_zap.PayloadUnaryServerInterceptor(b.log, alwaysLoggingDeciderServer)),
	}
	suite.Run(t, &zapPayloadSuite{b})
}

type zapPayloadSuite struct {
	*zapBaseSuite
}

func (s *zapPayloadSuite) getServerAndClientMessages(expectedServer int, expectedClient int) (serverMsgs []map[string]interface{}, clientMsgs []map[string]interface{}) {
	msgs := s.getOutputJSONs()
	for _, m := range msgs {
		if m["span.kind"] == "server" {
			serverMsgs = append(serverMsgs, m)
		} else if m["span.kind"] == "client" {
			clientMsgs = append(clientMsgs, m)
		}
	}
	require.Len(s.T(), serverMsgs, expectedServer, "must match expected number of server log messages")
	require.Len(s.T(), clientMsgs, expectedClient, "must match expected number of client log messages")
	return serverMsgs, clientMsgs
}

func (s *zapPayloadSuite) TestPing_LogsBothRequestAndResponse() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)

	require.NoError(s.T(), err, "there must be not be an error on a successful call")
	serverMsgs, clientMsgs := s.getServerAndClientMessages(2, 2)
	for _, m := range append(serverMsgs, clientMsgs...) {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), m["grpc.method"], "Ping", "all lines must contain method name")
		assert.Equal(s.T(), m["level"], "info", "all payloads must be logged on info level")
	}

	serverReq, serverResp := serverMsgs[0], serverMsgs[1]
	clientReq, clientResp := clientMsgs[0], clientMsgs[1]
	s.T().Log(clientReq)
	assert.Contains(s.T(), clientReq, "grpc.request.content", "request payload must be logged in a structured way")
	assert.Contains(s.T(), serverReq, "grpc.request.content", "request payload must be logged in a structured way")
	assert.Contains(s.T(), clientResp, "grpc.response.content", "response payload must be logged in a structured way")
	assert.Contains(s.T(), serverResp, "grpc.response.content", "response payload must be logged in a structured way")
}

func (s *zapPayloadSuite) TestPingError_LogsOnlyRequestsOnError() {
	_, err := s.Client.PingError(s.SimpleCtx(), &pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(4)})

	require.Error(s.T(), err, "there must be an error on an unsuccessful call")
	serverMsgs, clientMsgs := s.getServerAndClientMessages(1, 1)
	for _, m := range append(serverMsgs, clientMsgs...) {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), m["grpc.method"], "PingError", "all lines must contain method name")
		assert.Equal(s.T(), m["level"], "info", "must be logged at the info level")
	}

	assert.Contains(s.T(), clientMsgs[0], "grpc.request.content", "request payload must be logged in a structured way")
	assert.Contains(s.T(), serverMsgs[0], "grpc.request.content", "request payload must be logged in a structured way")
}

func (s *zapPayloadSuite) TestPingStream_LogsAllRequestsAndResponses() {
	messagesExpected := 20
	stream, err := s.Client.PingStream(s.SimpleCtx())

	require.NoError(s.T(), err, "no error on stream creation")
	for i := 0; i < messagesExpected; i++ {
		require.NoError(s.T(), stream.Send(goodPing), "sending must succeed")
	}
	require.NoError(s.T(), stream.CloseSend(), "no error on send stream")

	for {
		pong := &pb_testproto.PingResponse{}
		err := stream.RecvMsg(pong)
		if err == io.EOF {
			break
		}
		require.NoError(s.T(), err, "no error on receive")
	}

	serverMsgs, clientMsgs := s.getServerAndClientMessages(2*messagesExpected, 2*messagesExpected)
	for _, m := range append(serverMsgs, clientMsgs...) {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), m["grpc.method"], "PingStream", "all lines must contain method name")
		assert.Equal(s.T(), m["level"], "info", "all lines must logged at info level")

		content := m["grpc.request.content"] != nil || m["grpc.response.content"] != nil
		assert.True(s.T(), content, "all messages must contain payloads")
	}
}
