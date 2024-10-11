// Copyright 2016 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_retry_test

import (
	"context"
	"io"
	"sync"
	"testing"
	"time"

	grpc_middleware "github.com/grpc-ecosystem/go-grpc-middleware"
	grpc_retry "github.com/grpc-ecosystem/go-grpc-middleware/retry"
	"github.com/grpc-ecosystem/go-grpc-middleware/testing"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

var (
	retriableErrors = []codes.Code{codes.Unavailable, codes.DataLoss}
	goodPing        = &pb_testproto.PingRequest{Value: "something"}
	noSleep         = 0 * time.Second
	retryTimeout    = 50 * time.Millisecond
)

type failingService struct {
	pb_testproto.TestServiceServer
	mu sync.Mutex

	reqCounter uint
	reqModulo  uint
	reqSleep   time.Duration
	reqError   codes.Code
}

func (s *failingService) resetFailingConfiguration(modulo uint, errorCode codes.Code, sleepTime time.Duration) {
	s.mu.Lock()
	defer s.mu.Unlock()

	s.reqCounter = 0
	s.reqModulo = modulo
	s.reqError = errorCode
	s.reqSleep = sleepTime
}

func (s *failingService) requestCount() uint {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.reqCounter
}

func (s *failingService) maybeFailRequest() error {
	s.mu.Lock()
	s.reqCounter += 1
	reqModulo := s.reqModulo
	reqCounter := s.reqCounter
	reqSleep := s.reqSleep
	reqError := s.reqError
	s.mu.Unlock()
	if (reqModulo > 0) && (reqCounter%reqModulo == 0) {
		return nil
	}
	time.Sleep(reqSleep)
	return status.Errorf(reqError, "maybeFailRequest: failing it")
}

func (s *failingService) Ping(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.PingResponse, error) {
	if err := s.maybeFailRequest(); err != nil {
		return nil, err
	}
	return s.TestServiceServer.Ping(ctx, ping)
}

func (s *failingService) PingList(ping *pb_testproto.PingRequest, stream pb_testproto.TestService_PingListServer) error {
	if err := s.maybeFailRequest(); err != nil {
		return err
	}
	return s.TestServiceServer.PingList(ping, stream)
}

func (s *failingService) PingStream(stream pb_testproto.TestService_PingStreamServer) error {
	if err := s.maybeFailRequest(); err != nil {
		return err
	}
	return s.TestServiceServer.PingStream(stream)
}

func TestRetrySuite(t *testing.T) {
	service := &failingService{
		TestServiceServer: &grpc_testing.TestPingService{T: t},
	}
	unaryInterceptor := grpc_retry.UnaryClientInterceptor(
		grpc_retry.WithCodes(retriableErrors...),
		grpc_retry.WithMax(3),
		grpc_retry.WithBackoff(grpc_retry.BackoffLinear(retryTimeout)),
	)
	streamInterceptor := grpc_retry.StreamClientInterceptor(
		grpc_retry.WithCodes(retriableErrors...),
		grpc_retry.WithMax(3),
		grpc_retry.WithBackoff(grpc_retry.BackoffLinear(retryTimeout)),
	)
	s := &RetrySuite{
		srv: service,
		InterceptorTestSuite: &grpc_testing.InterceptorTestSuite{
			TestService: service,
			ClientOpts: []grpc.DialOption{
				grpc.WithStreamInterceptor(streamInterceptor),
				grpc.WithUnaryInterceptor(unaryInterceptor),
			},
		},
	}
	suite.Run(t, s)
}

type RetrySuite struct {
	*grpc_testing.InterceptorTestSuite
	srv *failingService
}

func (s *RetrySuite) SetupTest() {
	s.srv.resetFailingConfiguration( /* don't fail */ 0, codes.OK, noSleep)
}

func (s *RetrySuite) TestUnary_FailsOnNonRetriableError() {
	s.srv.resetFailingConfiguration(5, codes.Internal, noSleep)
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.Error(s.T(), err, "error must occur from the failing service")
	require.Equal(s.T(), codes.Internal, status.Code(err), "failure code must come from retrier")
	require.EqualValues(s.T(), 1, s.srv.requestCount(), "one request should have been made")
}

func (s *RetrySuite) TestUnary_FailsOnNonRetriableContextError() {
	s.srv.resetFailingConfiguration(5, codes.Canceled, noSleep)
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.Error(s.T(), err, "error must occur from the failing service")
	require.Equal(s.T(), codes.Canceled, status.Code(err), "failure code must come from retrier")
	require.EqualValues(s.T(), 1, s.srv.requestCount(), "one request should have been made")
}

func (s *RetrySuite) TestCallOptionsDontPanicWithoutInterceptor() {
	// Fix for https://github.com/grpc-ecosystem/go-grpc-middleware/issues/37
	// If this code doesn't panic, that's good.
	s.srv.resetFailingConfiguration(100, codes.DataLoss, noSleep) // doesn't matter all requests should fail
	nonMiddlewareClient := s.NewClient()
	_, err := nonMiddlewareClient.Ping(s.SimpleCtx(), goodPing,
		grpc_retry.WithMax(5),
		grpc_retry.WithBackoff(grpc_retry.BackoffLinear(1*time.Millisecond)),
		grpc_retry.WithCodes(codes.DataLoss),
		grpc_retry.WithPerRetryTimeout(1*time.Millisecond),
	)
	require.Error(s.T(), err)
}

func (s *RetrySuite) TestServerStream_FailsOnNonRetriableError() {
	s.srv.resetFailingConfiguration(5, codes.Internal, noSleep)
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "should not fail on establishing the stream")
	_, err = stream.Recv()
	require.Error(s.T(), err, "error must occur from the failing service")
	require.Equal(s.T(), codes.Internal, status.Code(err), "failure code must come from retrier")
}

func (s *RetrySuite) TestUnary_SucceedsOnRetriableError() {
	s.srv.resetFailingConfiguration(3, codes.DataLoss, noSleep) // see retriable_errors
	out, err := s.Client.Ping(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "the third invocation should succeed")
	require.NotNil(s.T(), out, "Pong must be not nil")
	require.EqualValues(s.T(), 3, s.srv.requestCount(), "three requests should have been made")
}

func (s *RetrySuite) TestUnary_OverrideFromDialOpts() {
	s.srv.resetFailingConfiguration(5, codes.ResourceExhausted, noSleep) // default is 3 and retriable_errors
	out, err := s.Client.Ping(s.SimpleCtx(), goodPing, grpc_retry.WithCodes(codes.ResourceExhausted), grpc_retry.WithMax(5))
	require.NoError(s.T(), err, "the fifth invocation should succeed")
	require.NotNil(s.T(), out, "Pong must be not nil")
	require.EqualValues(s.T(), 5, s.srv.requestCount(), "five requests should have been made")
}

func (s *RetrySuite) TestUnary_PerCallDeadline_Succeeds() {
	// This tests 5 requests, with first 4 sleeping for 10 millisecond, and the retry logic firing
	// a retry call with a 5 millisecond deadline. The 5th one doesn't sleep and succeeds.
	deadlinePerCall := 5 * time.Millisecond
	s.srv.resetFailingConfiguration(5, codes.NotFound, 2*deadlinePerCall)
	out, err := s.Client.Ping(s.SimpleCtx(), goodPing, grpc_retry.WithPerRetryTimeout(deadlinePerCall),
		grpc_retry.WithMax(5))
	require.NoError(s.T(), err, "the fifth invocation should succeed")
	require.NotNil(s.T(), out, "Pong must be not nil")
	require.EqualValues(s.T(), 5, s.srv.requestCount(), "five requests should have been made")
}

func (s *RetrySuite) TestUnary_PerCallDeadline_FailsOnParent() {
	// This tests that the parent context (passed to the invocation) takes precedence over retries.
	// The parent context has 150 milliseconds of deadline.
	// Each failed call sleeps for 100milliseconds, and there is 5 milliseconds between each one.
	// This means that unlike in TestUnary_PerCallDeadline_Succeeds, the fifth successful call won't
	// be made.
	parentDeadline := 150 * time.Millisecond
	deadlinePerCall := 50 * time.Millisecond
	// All 0-4 requests should have 10 millisecond sleeps and deadline, while the last one works.
	s.srv.resetFailingConfiguration(5, codes.NotFound, 2*deadlinePerCall)
	ctx, _ := context.WithTimeout(context.TODO(), parentDeadline)
	_, err := s.Client.Ping(ctx, goodPing, grpc_retry.WithPerRetryTimeout(deadlinePerCall),
		grpc_retry.WithMax(5))
	require.Error(s.T(), err, "the retries must fail due to context deadline exceeded")
	require.Equal(s.T(), codes.DeadlineExceeded, status.Code(err), "failre code must be a gRPC error of Deadline class")
}

func (s *RetrySuite) TestServerStream_SucceedsOnRetriableError() {
	s.srv.resetFailingConfiguration(3, codes.DataLoss, noSleep) // see retriable_errors
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing)
	require.NoError(s.T(), err, "establishing the connection must always succeed")
	s.assertPingListWasCorrect(stream)
	require.EqualValues(s.T(), 3, s.srv.requestCount(), "three requests should have been made")
}

func (s *RetrySuite) TestServerStream_OverrideFromContext() {
	s.srv.resetFailingConfiguration(5, codes.ResourceExhausted, noSleep) // default is 3 and retriable_errors
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing, grpc_retry.WithCodes(codes.ResourceExhausted), grpc_retry.WithMax(5))
	require.NoError(s.T(), err, "establishing the connection must always succeed")
	s.assertPingListWasCorrect(stream)
	require.EqualValues(s.T(), 5, s.srv.requestCount(), "three requests should have been made")
}

func (s *RetrySuite) TestServerStream_PerCallDeadline_Succeeds() {
	// This tests 5 requests, with first 4 sleeping for 100 millisecond, and the retry logic firing
	// a retry call with a 50 millisecond deadline. The 5th one doesn't sleep and succeeds.
	deadlinePerCall := 50 * time.Millisecond
	s.srv.resetFailingConfiguration(5, codes.NotFound, 2*deadlinePerCall)
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing, grpc_retry.WithPerRetryTimeout(deadlinePerCall),
		grpc_retry.WithMax(5))
	require.NoError(s.T(), err, "establishing the connection must always succeed")
	s.assertPingListWasCorrect(stream)
	require.EqualValues(s.T(), 5, s.srv.requestCount(), "three requests should have been made")
}

func (s *RetrySuite) TestServerStream_PerCallDeadline_FailsOnParent() {
	// This tests that the parent context (passed to the invocation) takes precedence over retries.
	// The parent context has 150 milliseconds of deadline.
	// Each failed call sleeps for 50milliseconds, and there is 25 milliseconds between each one.
	// This means that unlike in TestServerStream_PerCallDeadline_Succeeds, the fifth successful call won't
	// be made.
	parentDeadline := 150 * time.Millisecond
	deadlinePerCall := 50 * time.Millisecond
	// All 0-4 requests should have 10 millisecond sleeps and deadline, while the last one works.
	s.srv.resetFailingConfiguration(5, codes.NotFound, 2*deadlinePerCall)
	parentCtx, _ := context.WithTimeout(context.TODO(), parentDeadline)
	stream, err := s.Client.PingList(parentCtx, goodPing, grpc_retry.WithPerRetryTimeout(deadlinePerCall),
		grpc_retry.WithMax(5))
	require.NoError(s.T(), err, "establishing the connection must always succeed")
	_, err = stream.Recv()
	require.Equal(s.T(), codes.DeadlineExceeded, status.Code(err), "failre code must be a gRPC error of Deadline class")
}

func (s *RetrySuite) TestServerStream_CallFailsOnOutOfRetries() {
	restarted := s.RestartServer(3 * retryTimeout)
	_, err := s.Client.PingList(s.SimpleCtx(), goodPing)

	require.Error(s.T(), err, "establishing the connection should not succeed")
	assert.Equal(s.T(), codes.Unavailable, status.Code(err))

	<-restarted
}

func (s *RetrySuite) TestServerStream_CallFailsOnDeadlineExceeded() {
	restarted := s.RestartServer(3 * retryTimeout)
	ctx, _ := context.WithTimeout(context.TODO(), retryTimeout)
	_, err := s.Client.PingList(ctx, goodPing)

	require.Error(s.T(), err, "establishing the connection should not succeed")
	assert.Equal(s.T(), codes.DeadlineExceeded, status.Code(err))

	<-restarted
}

func (s *RetrySuite) TestServerStream_CallRetrySucceeds() {
	restarted := s.RestartServer(retryTimeout)

	_, err := s.Client.PingList(s.SimpleCtx(), goodPing,
		grpc_retry.WithMax(40),
	)

	assert.NoError(s.T(), err, "establishing the connection should succeed")
	<-restarted
}

func (s *RetrySuite) assertPingListWasCorrect(stream pb_testproto.TestService_PingListClient) {
	count := 0
	for {
		pong, err := stream.Recv()
		if err == io.EOF {
			break
		}
		require.NotNil(s.T(), pong, "received values must not be nil")
		require.NoError(s.T(), err, "no errors during receive on client side")
		require.Equal(s.T(), goodPing.Value, pong.Value, "the returned pong contained the outgoing ping")
		count += 1
	}
	require.EqualValues(s.T(), grpc_testing.ListResponseCount, count, "should have received all ping items")
}

type trackedInterceptor struct {
	called int
}

func (ti *trackedInterceptor) UnaryClientInterceptor(ctx context.Context, method string, req, reply interface{}, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
	ti.called++
	return invoker(ctx, method, req, reply, cc, opts...)
}

func (ti *trackedInterceptor) StreamClientInterceptor(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
	ti.called++
	return streamer(ctx, desc, cc, method, opts...)
}

func TestChainedRetrySuite(t *testing.T) {
	service := &failingService{
		TestServiceServer: &grpc_testing.TestPingService{T: t},
	}
	preRetryInterceptor := &trackedInterceptor{}
	postRetryInterceptor := &trackedInterceptor{}
	s := &ChainedRetrySuite{
		srv:                  service,
		preRetryInterceptor:  preRetryInterceptor,
		postRetryInterceptor: postRetryInterceptor,
		InterceptorTestSuite: &grpc_testing.InterceptorTestSuite{
			TestService: service,
			ClientOpts: []grpc.DialOption{
				grpc.WithUnaryInterceptor(grpc_middleware.ChainUnaryClient(preRetryInterceptor.UnaryClientInterceptor, grpc_retry.UnaryClientInterceptor(), postRetryInterceptor.UnaryClientInterceptor)),
				grpc.WithStreamInterceptor(grpc_middleware.ChainStreamClient(preRetryInterceptor.StreamClientInterceptor, grpc_retry.StreamClientInterceptor(), postRetryInterceptor.StreamClientInterceptor)),
			},
		},
	}
	suite.Run(t, s)
}

type ChainedRetrySuite struct {
	*grpc_testing.InterceptorTestSuite
	srv                  *failingService
	preRetryInterceptor  *trackedInterceptor
	postRetryInterceptor *trackedInterceptor
}

func (s *ChainedRetrySuite) SetupTest() {
	s.srv.resetFailingConfiguration( /* don't fail */ 0, codes.OK, noSleep)
	s.preRetryInterceptor.called = 0
	s.postRetryInterceptor.called = 0
}

func (s *ChainedRetrySuite) TestUnaryWithChainedInterceptors_NoFailure() {
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing, grpc_retry.WithMax(2))
	require.NoError(s.T(), err, "the invocation should succeed")
	require.EqualValues(s.T(), 1, s.srv.requestCount(), "one request should have been made")
	require.EqualValues(s.T(), 1, s.preRetryInterceptor.called, "pre-retry interceptor should be called once")
	require.EqualValues(s.T(), 1, s.postRetryInterceptor.called, "post-retry interceptor should be called once")
}

func (s *ChainedRetrySuite) TestUnaryWithChainedInterceptors_WithRetry() {
	s.srv.resetFailingConfiguration(2, codes.Unavailable, noSleep)
	_, err := s.Client.Ping(s.SimpleCtx(), goodPing, grpc_retry.WithMax(2))
	require.NoError(s.T(), err, "the second invocation should succeed")
	require.EqualValues(s.T(), 2, s.srv.requestCount(), "two requests should have been made")
	require.EqualValues(s.T(), 1, s.preRetryInterceptor.called, "pre-retry interceptor should be called once")
	require.EqualValues(s.T(), 2, s.postRetryInterceptor.called, "post-retry interceptor should be called twice")
}

func (s *ChainedRetrySuite) TestStreamWithChainedInterceptors_NoFailure() {
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing, grpc_retry.WithMax(2))
	require.NoError(s.T(), err, "the invocation should succeed")
	_, err = stream.Recv()
	require.NoError(s.T(), err, "the Recv should succeed")
	require.EqualValues(s.T(), 1, s.srv.requestCount(), "one request should have been made")
	require.EqualValues(s.T(), 1, s.preRetryInterceptor.called, "pre-retry interceptor should be called once")
	require.EqualValues(s.T(), 1, s.postRetryInterceptor.called, "post-retry interceptor should be called once")
}

func (s *ChainedRetrySuite) TestStreamWithChainedInterceptors_WithRetry() {
	s.srv.resetFailingConfiguration(2, codes.Unavailable, noSleep)
	stream, err := s.Client.PingList(s.SimpleCtx(), goodPing, grpc_retry.WithMax(2))
	require.NoError(s.T(), err, "the second invocation should succeed")
	_, err = stream.Recv()
	require.NoError(s.T(), err, "the Recv should succeed")
	require.EqualValues(s.T(), 2, s.srv.requestCount(), "two requests should have been made")
	require.EqualValues(s.T(), 1, s.preRetryInterceptor.called, "pre-retry interceptor should be called once")
	require.EqualValues(s.T(), 2, s.postRetryInterceptor.called, "post-retry interceptor should be called twice")
}
