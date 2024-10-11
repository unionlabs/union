package grpc_zap_test

import (
	"io"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"

	grpc_zap "github.com/grpc-ecosystem/go-grpc-middleware/logging/zap"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"go.uber.org/zap/zapcore"
)

func customClientCodeToLevel(c codes.Code) zapcore.Level {
	if c == codes.Unauthenticated {
		// Make this a special case for tests, and an error.
		return zapcore.ErrorLevel
	}
	level := grpc_zap.DefaultClientCodeToLevel(c)
	return level
}

func TestZapClientSuite(t *testing.T) {
	opts := []grpc_zap.Option{
		grpc_zap.WithLevels(customClientCodeToLevel),
	}
	b := newBaseZapSuite(t)
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_zap.UnaryClientInterceptor(b.log, opts...)),
		grpc.WithStreamInterceptor(grpc_zap.StreamClientInterceptor(b.log, opts...)),
	}
	suite.Run(t, &zapClientSuite{b})
}

type zapClientSuite struct {
	*zapBaseSuite
}

func (s *zapClientSuite) TestPing() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client unary call", "must contain correct message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "must be logged on debug level.")
	assert.Contains(s.T(), msgs[0], "grpc.time_ms", "interceptor log statement should contain execution time")
}

func (s *zapClientSuite) TestPingList() {
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "should not fail on establishing the stream")
	for {
		_, err := stream.Recv()
		if err == io.EOF {
			break
		}
		require.NoError(s.T(), err, "reading stream should not fail")
	}
	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "PingList", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client streaming call", "handler's message must contain user message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "OK codes must be logged on debug level.")
	assert.Contains(s.T(), msgs[0], "grpc.time_ms", "handler's message must contain time in ms")
}

func (s *zapClientSuite) TestPingError_WithCustomLevels() {
	for _, tcase := range []struct {
		code  codes.Code
		level zapcore.Level
		msg   string
	}{
		{
			code:  codes.Internal,
			level: zapcore.WarnLevel,
			msg:   "Internal must remap to ErrorLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.NotFound,
			level: zapcore.DebugLevel,
			msg:   "NotFound must remap to InfoLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.FailedPrecondition,
			level: zapcore.DebugLevel,
			msg:   "FailedPrecondition must remap to WarnLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.Unauthenticated,
			level: zapcore.ErrorLevel,
			msg:   "Unauthenticated is overwritten to ErrorLevel with customClientCodeToLevel override, which probably didn't work",
		},
	} {
		s.SetupTest()
		_, err := s.Client.PingError(
			s.SimpleCtx(),
			&pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(tcase.code)})
		require.Error(s.T(), err, "each call here must return an error")

		msgs := s.getOutputJSONs()
		require.Len(s.T(), msgs, 1, "only the interceptor log message is printed in PingErr")

		assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), msgs[0]["grpc.method"], "PingError", "all lines must contain method name")
		assert.Equal(s.T(), msgs[0]["grpc.code"], tcase.code.String(), "all lines must contain the correct gRPC code")
		assert.Equal(s.T(), msgs[0]["level"], tcase.level.String(), tcase.msg)
	}
}

func TestZapClientOverrideSuite(t *testing.T) {
	opts := []grpc_zap.Option{
		grpc_zap.WithDurationField(grpc_zap.DurationToDurationField),
	}
	b := newBaseZapSuite(t)
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_zap.UnaryClientInterceptor(b.log, opts...)),
		grpc.WithStreamInterceptor(grpc_zap.StreamClientInterceptor(b.log, opts...)),
	}
	suite.Run(t, &zapClientOverrideSuite{b})
}

type zapClientOverrideSuite struct {
	*zapBaseSuite
}

func (s *zapClientOverrideSuite) TestPing_HasOverrides() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client unary call", "handler's message must contain user message")

	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "handler's message must not contain default duration")
	assert.Contains(s.T(), msgs[0], "grpc.duration", "handler's message must contain overridden duration")
}

func (s *zapClientOverrideSuite) TestPingList_HasOverrides() {
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "should not fail on establishing the stream")
	for {
		_, err := stream.Recv()
		if err == io.EOF {
			break
		}
		require.NoError(s.T(), err, "reading stream should not fail")
	}
	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "PingList", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client streaming call", "handler's message must contain user message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "must be logged on debug level.")

	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "handler's message must not contain default duration")
	assert.Contains(s.T(), msgs[0], "grpc.duration", "handler's message must contain overridden duration")
}

func TestZapLoggingClientMessageProducerSuite(t *testing.T) {
	opts := []grpc_zap.Option{
		grpc_zap.WithMessageProducer(StubMessageProducer),
	}
	b := newBaseZapSuite(t)
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_zap.UnaryClientInterceptor(b.log, opts...)),
		grpc.WithStreamInterceptor(grpc_zap.StreamClientInterceptor(b.log, opts...)),
	}
	suite.Run(t, &zapClientMessageProducerSuite{b})
}

type zapClientMessageProducerSuite struct {
	*zapBaseSuite
}

func (s *zapClientMessageProducerSuite) TestPing_HasOverriddenMessageProducer() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "custom message", "handler's message must contain user message")
}
