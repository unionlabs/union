package grpc_logrus_test

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"testing"

	grpc_logrus "github.com/grpc-ecosystem/go-grpc-middleware/logging/logrus"
	"github.com/grpc-ecosystem/go-grpc-middleware/logging/logrus/ctxlogrus"
	grpc_ctxtags "github.com/grpc-ecosystem/go-grpc-middleware/tags"
	grpc_testing "github.com/grpc-ecosystem/go-grpc-middleware/testing"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"github.com/sirupsen/logrus"
	"google.golang.org/grpc/codes"
)

var (
	goodPing = &pb_testproto.PingRequest{Value: "something", SleepTimeMs: 9999}
)

type loggingPingService struct {
	pb_testproto.TestServiceServer
}

func customCodeToLevel(c codes.Code) logrus.Level {
	if c == codes.Unauthenticated {
		// Make this a special case for tests, and an error.
		return logrus.ErrorLevel
	}
	level := grpc_logrus.DefaultCodeToLevel(c)
	return level
}

func (s *loggingPingService) Ping(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.PingResponse, error) {
	grpc_ctxtags.Extract(ctx).Set("custom_tags.string", "something").Set("custom_tags.int", 1337)
	ctxlogrus.AddFields(ctx, logrus.Fields{"custom_field": "custom_value"})
	ctxlogrus.Extract(ctx).Info("some ping")
	return s.TestServiceServer.Ping(ctx, ping)
}

func (s *loggingPingService) PingError(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.Empty, error) {
	return s.TestServiceServer.PingError(ctx, ping)
}

func (s *loggingPingService) PingList(ping *pb_testproto.PingRequest, stream pb_testproto.TestService_PingListServer) error {
	grpc_ctxtags.Extract(stream.Context()).Set("custom_tags.string", "something").Set("custom_tags.int", 1337)
	ctxlogrus.AddFields(stream.Context(), logrus.Fields{"custom_field": "custom_value"})
	ctxlogrus.Extract(stream.Context()).Info("some pinglist")
	return s.TestServiceServer.PingList(ping, stream)
}

func (s *loggingPingService) PingEmpty(ctx context.Context, empty *pb_testproto.Empty) (*pb_testproto.PingResponse, error) {
	return s.TestServiceServer.PingEmpty(ctx, empty)
}

type logrusBaseSuite struct {
	*grpc_testing.InterceptorTestSuite
	mutexBuffer     *grpc_testing.MutexReadWriter
	buffer          *bytes.Buffer
	logger          *logrus.Logger
	timestampFormat string
}

func newLogrusBaseSuite(t *testing.T) *logrusBaseSuite {
	b := &bytes.Buffer{}
	muB := grpc_testing.NewMutexReadWriter(b)
	logger := logrus.New()
	logger.Out = muB
	logger.Formatter = &logrus.JSONFormatter{DisableTimestamp: true}
	return &logrusBaseSuite{
		logger:      logger,
		buffer:      b,
		mutexBuffer: muB,
		InterceptorTestSuite: &grpc_testing.InterceptorTestSuite{
			TestService: &loggingPingService{&grpc_testing.TestPingService{T: t}},
		},
	}
}

func (s *logrusBaseSuite) SetupTest() {
	s.mutexBuffer.Lock()
	s.buffer.Reset()
	s.mutexBuffer.Unlock()
}

func (s *logrusBaseSuite) getOutputJSONs() []map[string]interface{} {
	ret := make([]map[string]interface{}, 0)
	dec := json.NewDecoder(s.mutexBuffer)

	for {
		var val map[string]interface{}
		err := dec.Decode(&val)
		if err == io.EOF {
			break
		}
		if err != nil {
			s.T().Fatalf("failed decoding output from Logrus JSON: %v", err)
		}

		ret = append(ret, val)
	}

	return ret
}

func StubMessageProducer(ctx context.Context, format string, level logrus.Level, code codes.Code, err error, fields logrus.Fields) {
	if err != nil {
		fields[logrus.ErrorKey] = err
	}
	format = "custom message"
	entry := ctxlogrus.Extract(ctx).WithContext(ctx).WithFields(fields)
	switch level {
	case logrus.DebugLevel:
		entry.Debugf(format)
	case logrus.InfoLevel:
		entry.Infof(format)
	case logrus.WarnLevel:
		entry.Warningf(format)
	case logrus.ErrorLevel:
		entry.Errorf(format)
	case logrus.FatalLevel:
		entry.Fatalf(format)
	case logrus.PanicLevel:
		entry.Panicf(format)
	}
}
