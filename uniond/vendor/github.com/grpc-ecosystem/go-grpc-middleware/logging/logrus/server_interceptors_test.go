package grpc_logrus_test

import (
	"io"
	"testing"
	"time"

	grpc_middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	grpc_logrus "github.com/grpc-ecosystem/go-grpc-middleware/logging/logrus"
	grpc_ctxtags "github.com/grpc-ecosystem/go-grpc-middleware/tags"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
)

func TestLogrusServerSuite(t *testing.T) {
	for _, tcase := range []struct {
		timestampFormat string
	}{
		{
			timestampFormat: time.RFC3339,
		},
		{
			timestampFormat: "2006-01-02",
		},
	} {
		opts := []grpc_logrus.Option{
			grpc_logrus.WithLevels(customCodeToLevel),
			grpc_logrus.WithTimestampFormat(tcase.timestampFormat),
		}

		b := newLogrusBaseSuite(t)
		b.timestampFormat = tcase.timestampFormat
		b.InterceptorTestSuite.ServerOpts = []grpc.ServerOption{
			grpc_middleware.WithStreamServerChain(
				grpc_ctxtags.StreamServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
				grpc_logrus.StreamServerInterceptor(logrus.NewEntry(b.logger), opts...)),
			grpc_middleware.WithUnaryServerChain(
				grpc_ctxtags.UnaryServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
				grpc_logrus.UnaryServerInterceptor(logrus.NewEntry(b.logger), opts...)),
		}
		suite.Run(t, &logrusServerSuite{b})
	}
}

type logrusServerSuite struct {
	*logrusBaseSuite
}

func (s *logrusServerSuite) TestPing_WithCustomTags() {
	deadline := time.Now().Add(3 * time.Second)
	_, err := s.Client.Ping(s.DeadlineCtx(deadline), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 2, "two log statements should be logged")

	for _, m := range msgs {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
		assert.Equal(s.T(), m["grpc.method"], "Ping", "all lines must contain the correct method name")
		assert.Equal(s.T(), m["span.kind"], "server", "all lines must contain the kind of call (server)")
		assert.Equal(s.T(), m["custom_tags.string"], "something", "all lines must contain `custom_tags.string` with expected value")
		assert.Equal(s.T(), m["grpc.request.value"], "something", "all lines must contain the correct request value")
		assert.Equal(s.T(), m["custom_field"], "custom_value", "all lines must contain `custom_field` with the correct value")

		assert.Contains(s.T(), m, "custom_tags.int", "all lines must contain `custom_tags.int`")
		require.Contains(s.T(), m, "grpc.start_time", "all lines must contain the start time of the call")
		_, err := time.Parse(s.timestampFormat, m["grpc.start_time"].(string))
		assert.NoError(s.T(), err, "should be able to parse start time")

		require.Contains(s.T(), m, "grpc.request.deadline", "all lines must contain the deadline of the call")
		_, err = time.Parse(s.timestampFormat, m["grpc.request.deadline"].(string))
		require.NoError(s.T(), err, "should be able to parse deadline")
		assert.Equal(s.T(), m["grpc.request.deadline"], deadline.Format(s.timestampFormat), "should have the same deadline that was set by the caller")
	}

	assert.Equal(s.T(), msgs[0]["msg"], "some ping", "first message must contain the correct user message")
	assert.Equal(s.T(), msgs[1]["msg"], "finished unary call with code OK", "second message must contain the correct user message")
	assert.Equal(s.T(), msgs[1]["level"], "info", "OK codes must be logged on info level.")

	assert.Contains(s.T(), msgs[1], "grpc.time_ms", "interceptor log statement should contain execution time")
}

func (s *logrusServerSuite) TestPingError_WithCustomLevels() {
	for _, tcase := range []struct {
		code  codes.Code
		level logrus.Level
		msg   string
	}{
		{
			code:  codes.Internal,
			level: logrus.ErrorLevel,
			msg:   "Internal must remap to ErrorLevel in DefaultCodeToLevel",
		},
		{
			code:  codes.NotFound,
			level: logrus.InfoLevel,
			msg:   "NotFound must remap to InfoLevel in DefaultCodeToLevel",
		},
		{
			code:  codes.FailedPrecondition,
			level: logrus.WarnLevel,
			msg:   "FailedPrecondition must remap to WarnLevel in DefaultCodeToLevel",
		},
		{
			code:  codes.Unauthenticated,
			level: logrus.ErrorLevel,
			msg:   "Unauthenticated is overwritten to ErrorLevel with customCodeToLevel override, which probably didn't work",
		},
	} {
		s.buffer.Reset()
		_, err := s.Client.PingError(
			s.SimpleCtx(),
			&pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(tcase.code)})
		require.Error(s.T(), err, "each call here must return an error")

		msgs := s.getOutputJSONs()
		require.Len(s.T(), msgs, 1, "only the interceptor log message is printed in PingErr")
		m := msgs[0]
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
		assert.Equal(s.T(), m["grpc.method"], "PingError", "all lines must contain the correct method name")
		assert.Equal(s.T(), m["grpc.code"], tcase.code.String(), "a gRPC code must be present")
		assert.Equal(s.T(), m["level"], tcase.level.String(), tcase.msg)
		assert.Equal(s.T(), m["msg"], "finished unary call with code "+tcase.code.String(), "must have the correct finish message")

		require.Contains(s.T(), m, "grpc.start_time", "all lines must contain a start time for the call")
		_, err = time.Parse(s.timestampFormat, m["grpc.start_time"].(string))
		assert.NoError(s.T(), err, "should be able to parse the start time")

		require.Contains(s.T(), m, "grpc.request.deadline", "all lines must contain the deadline of the call")
		_, err = time.Parse(s.timestampFormat, m["grpc.request.deadline"].(string))
		require.NoError(s.T(), err, "should be able to parse deadline")
	}
}

func (s *logrusServerSuite) TestPingList_WithCustomTags() {
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
	require.Len(s.T(), msgs, 2, "two log statements should be logged")
	for _, m := range msgs {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain the correct service name")
		assert.Equal(s.T(), m["grpc.method"], "PingList", "all lines must contain the correct method name")
		assert.Equal(s.T(), m["span.kind"], "server", "all lines must contain the kind of call (server)")
		assert.Equal(s.T(), m["custom_tags.string"], "something", "all lines must contain the correct `custom_tags.string`")
		assert.Equal(s.T(), m["grpc.request.value"], "something", "all lines must contain the correct request value")

		assert.Contains(s.T(), m, "custom_tags.int", "all lines must contain `custom_tags.int`")
		require.Contains(s.T(), m, "grpc.start_time", "all lines must contain the start time for the call")
		_, err := time.Parse(s.timestampFormat, m["grpc.start_time"].(string))
		assert.NoError(s.T(), err, "should be able to parse start time as RFC3339")

		require.Contains(s.T(), m, "grpc.request.deadline", "all lines must contain the deadline of the call")
		_, err = time.Parse(s.timestampFormat, m["grpc.request.deadline"].(string))
		require.NoError(s.T(), err, "should be able to parse deadline")
	}

	assert.Equal(s.T(), msgs[0]["msg"], "some pinglist", "msg must be the correct message")
	assert.Equal(s.T(), msgs[1]["msg"], "finished streaming call with code OK", "msg must be the correct message")
	assert.Equal(s.T(), msgs[1]["level"], "info", "OK codes must be logged on info level.")

	assert.Contains(s.T(), msgs[1], "grpc.time_ms", "interceptor log statement should contain execution time")
}

func TestLogrusServerOverrideSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithDurationField(grpc_logrus.DurationToDurationField),
	}
	b := newLogrusBaseSuite(t)
	b.InterceptorTestSuite.ServerOpts = []grpc.ServerOption{
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(),
			grpc_logrus.StreamServerInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(),
			grpc_logrus.UnaryServerInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusServerOverrideSuite{b})
}

type logrusServerOverrideSuite struct {
	*logrusBaseSuite
}

func (s *logrusServerOverrideSuite) TestPing_HasOverriddenDuration() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 2, "two log statements should be logged")
	for _, m := range msgs {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), m["grpc.method"], "Ping", "all lines must contain method name")
	}

	assert.Equal(s.T(), msgs[0]["msg"], "some ping", "first message must be correct")
	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "first message must not contain default duration")
	assert.NotContains(s.T(), msgs[0], "grpc.duration", "first message must not contain overridden duration")

	assert.Equal(s.T(), msgs[1]["msg"], "finished unary call with code OK", "second message must be correct")
	assert.Equal(s.T(), msgs[1]["level"], "info", "second must be logged on info level.")
	assert.NotContains(s.T(), msgs[1], "grpc.time_ms", "second message must not contain default duration")
	assert.Contains(s.T(), msgs[1], "grpc.duration", "second message must contain overridden duration")
}

func (s *logrusServerOverrideSuite) TestPingList_HasOverriddenDuration() {
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
	require.Len(s.T(), msgs, 2, "two log statements should be logged")

	for _, m := range msgs {
		assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
		assert.Equal(s.T(), m["grpc.method"], "PingList", "all lines must contain method name")
	}

	assert.Equal(s.T(), msgs[0]["msg"], "some pinglist", "first message must contain user message")
	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "first message must not contain default duration")
	assert.NotContains(s.T(), msgs[0], "grpc.duration", "first message must not contain overridden duration")

	assert.Equal(s.T(), msgs[1]["msg"], "finished streaming call with code OK", "second message must contain correct message")
	assert.Equal(s.T(), msgs[1]["level"], "info", "second message must be logged on info level.")
	assert.NotContains(s.T(), msgs[1], "grpc.time_ms", "second message must not contain default duration")
	assert.Contains(s.T(), msgs[1], "grpc.duration", "second message must contain overridden duration")
}

func TestLogrusServerOverrideDeciderSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithDecider(func(method string, err error) bool {
			if err != nil && method == "/mwitkow.testproto.TestService/PingError" {
				return true
			}

			return false
		}),
	}
	b := newLogrusBaseSuite(t)
	b.InterceptorTestSuite.ServerOpts = []grpc.ServerOption{
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(),
			grpc_logrus.StreamServerInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(),
			grpc_logrus.UnaryServerInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusServerOverrideDeciderSuite{b})
}

type logrusServerOverrideDeciderSuite struct {
	*logrusBaseSuite
}

func (s *logrusServerOverrideDeciderSuite) TestPing_HasOverriddenDecider() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "single log statements should be logged")
	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "some ping", "handler's message must contain user message")
}

func (s *logrusServerOverrideDeciderSuite) TestPingError_HasOverriddenDecider() {
	code := codes.NotFound
	level := logrus.InfoLevel
	msg := "NotFound must remap to InfoLevel in DefaultCodeToLevel"

	s.buffer.Reset()
	_, err := s.Client.PingError(
		s.SimpleCtx(),
		&pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(code)})
	require.Error(s.T(), err, "each call here must return an error")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 1, "only the interceptor log message is printed in PingErr")
	m := msgs[0]
	assert.Equal(s.T(), m["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), m["grpc.method"], "PingError", "all lines must contain method name")
	assert.Equal(s.T(), m["grpc.code"], code.String(), "all lines must correct gRPC code")
	assert.Equal(s.T(), m["level"], level.String(), msg)
}

func (s *logrusServerOverrideDeciderSuite) TestPingList_HasOverriddenDecider() {
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
	require.Len(s.T(), msgs, 1, "single log statements should be logged")
	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "PingList", "all lines must contain method name")
	assert.Equal(s.T(), msgs[0]["msg"], "some pinglist", "handler's message must contain user message")

	assert.NotContains(s.T(), msgs[0], "grpc.time_ms", "handler's message must not contain default duration")
	assert.NotContains(s.T(), msgs[0], "grpc.duration", "handler's message must not contain overridden duration")
}

func TestLogrusServerMessageProducerSuite(t *testing.T) {
	opts := []grpc_logrus.Option{
		grpc_logrus.WithMessageProducer(StubMessageProducer),
	}
	b := newLogrusBaseSuite(t)
	b.InterceptorTestSuite.ServerOpts = []grpc.ServerOption{
		grpc_middleware.WithStreamServerChain(
			grpc_ctxtags.StreamServerInterceptor(),
			grpc_logrus.StreamServerInterceptor(logrus.NewEntry(b.logger), opts...)),
		grpc_middleware.WithUnaryServerChain(
			grpc_ctxtags.UnaryServerInterceptor(),
			grpc_logrus.UnaryServerInterceptor(logrus.NewEntry(b.logger), opts...)),
	}
	suite.Run(t, &logrusServerMessageProducerSuite{b})
}

type logrusServerMessageProducerSuite struct {
	*logrusBaseSuite
}

func (s *logrusServerMessageProducerSuite) TestPing_HasMessageProducer() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	msgs := s.getOutputJSONs()
	require.Len(s.T(), msgs, 2, "single log statements should be logged")
	assert.Equal(s.T(), msgs[0]["grpc.service"], "mwitkow.testproto.TestService", "all lines must contain service name")
	assert.Equal(s.T(), msgs[0]["grpc.method"], "Ping", "all lines must contain method name")
	assert.Equal(s.T(), msgs[1]["msg"], "custom message", "user defined message producer must be used")

	assert.Equal(s.T(), msgs[0]["msg"], "some ping", "handler's message must contain user message")
}
