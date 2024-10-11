package grpc_logrus_test

import (
	"io"
	"testing"

	grpc_logrus "github.com/grpc-ecosystem/go-grpc-middleware/logging/logrus"
	"github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"

	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
)

func customClientCodeToLevel(c codes.Code) logrus.Level {
	if c == codes.Unauthenticated {
		// Make this a special case for tests, and an error.
		return logrus.ErrorLevel
	}
	level := grpc_logrus.DefaultClientCodeToLevel(c)
	return level
}

func TestLogrusClientSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithLevels(customClientCodeToLevel),
	}
	b := newLogrusBaseSuite(t)
	b.logger.Level = logrus.DebugLevel // a lot of our stuff is on debug level by default
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_logrus.UnaryClientInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc.WithStreamInterceptor(grpc_logrus.StreamClientInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusClientSuite{b})
}

type logrusClientSuite struct {
	*logrusBaseSuite
}

func (s *logrusClientSuite) TestPing() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	assert.NoError(s.T(), err, "there must be not be an on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain the correct method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client unary call", "handler's message must contain the correct message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "OK codes must be logged on debug level.")

	assert.Contains(s.T(), msgs[0], "grpc.time_ms", "interceptor log statement should contain execution time (duration in ms)")
}

func (s *logrusClientSuite) TestPingList() {
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

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "PingList", "all lines must contain the correct method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client streaming call", "handler's message must contain the correct message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "OK codes must be logged on debug level.")
	assert.Contains(s.T(), msgs[0], "grpc.time_ms", "interceptor log statement should contain execution time (duration in ms)")
}

func (s *logrusClientSuite) TestPingError_WithCustomLevels() {
	for _, tcase := range []struct {
		code  codes.Code
		level logrus.Level
		msg   string
	}{
		{
			code:  codes.Internal,
			level: logrus.WarnLevel,
			msg:   "Internal must remap to ErrorLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.NotFound,
			level: logrus.DebugLevel,
			msg:   "NotFound must remap to InfoLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.FailedPrecondition,
			level: logrus.DebugLevel,
			msg:   "FailedPrecondition must remap to WarnLevel in DefaultClientCodeToLevel",
		},
		{
			code:  codes.Unauthenticated,
			level: logrus.ErrorLevel,
			msg:   "Unauthenticated is overwritten to ErrorLevel with customClientCodeToLevel override, which probably didn't work",
		},
	} {
		s.SetupTest()
		_, err := s.Client.PingError(
			s.SimpleCtx(),
			&pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(tcase.code)})

		assert.Error(s.T(), err, "each call here must return an error")

		msgs := s.getOutputJSONs()
		require.Len(s.T(), msgs, 1, "only a single log message is printed")

		assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
		assert.Equal(s.T(), msgs[0]["grpc.method"], "PingError", "all lines must contain the correct method name")
		assert.Equal(s.T(), msgs[0]["grpc.code"], tcase.code.String(), "all lines must contain a grpc code")
		assert.Equal(s.T(), msgs[0]["level"], tcase.level.String(), tcase.msg)
	}
}

func TestLogrusClientOverrideSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithDurationField(grpc_logrus.DurationToDurationField),
	}
	b := newLogrusBaseSuite(t)
	b.logger.Level = logrus.DebugLevel // a lot of our stuff is on debug level by default
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_logrus.UnaryClientInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc.WithStreamInterceptor(grpc_logrus.StreamClientInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusClientOverrideSuite{b})
}

type logrusClientOverrideSuite struct {
	*logrusBaseSuite
}

func (s *logrusClientOverrideSuite) TestPing_HasOverrides() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	assert.NoError(s.T(), err, "there must be not be an on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")
	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain the correct method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client unary call", "handler's message must contain the correct message")

	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "message must not contain default duration")
	assert.Contains(s.T(), msgs[0], "grpc.duration", "message must contain overridden duration")
}

func (s *logrusClientOverrideSuite) TestPingList_HasOverrides() {
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

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "PingList", "all lines must contain the correct method name")
	assert.Equal(s.T(), msgs[0]["msg"], "finished client streaming call", "log message must be correct")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "OK codes must be logged on debug level.")

	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "message must not contain default duration")
	assert.Contains(s.T(), msgs[0], "grpc.duration", "message must contain overridden duration")
}

func TestZapLoggingClientMessageProducerSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithMessageProducer(StubMessageProducer),
	}
	b := newLogrusBaseSuite(t)
	b.logger.Level = logrus.DebugLevel // a lot of our stuff is on debug level by default
	b.InterceptorTestSuite.ClientOpts = []grpc.DialOption{
		grpc.WithUnaryInterceptor(grpc_logrus.UnaryClientInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc.WithStreamInterceptor(grpc_logrus.StreamClientInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusClientMessageProducerSuite{b})
}

type logrusClientMessageProducerSuite struct {
	*logrusBaseSuite
}

func (s *logrusClientMessageProducerSuite) TestPing_HasOverriddenMessageProducer() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	assert.NoError(s.T(), err, "there must be not be an on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "one log statement should be logged")

	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain the correct method name")
	assert.Equal(s.T(), msgs[0]["msg"], "custom message", "handler's message must contain the correct message")
	assert.Equal(s.T(), msgs[0]["span.kind"], "client", "all lines must contain the kind of call (client)")
	assert.Equal(s.T(), msgs[0]["level"], "debug", "OK codes must be logged on debug level.")

	assert.Contains(s.T(), msgs[0], "grpc.time_ms", "interceptor log statement should contain execution time (duration in ms)")
}
