// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_opentracing_test

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"testing"

	"github.com/opentracing/opentracing-go"
	"github.com/opentracing/opentracing-go/log"
	"github.com/opentracing/opentracing-go/mocktracer"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"

	"github.com/grpc-ecosystem/go-grpc-middleware"
	"github.com/grpc-ecosystem/go-grpc-middleware/tags"
	"github.com/grpc-ecosystem/go-grpc-middleware/testing"
	pb_testproto "github.com/grpc-ecosystem/go-grpc-middleware/testing/testproto"
	"github.com/grpc-ecosystem/go-grpc-middleware/tracing/opentracing"
)

var (
	goodPing                = &pb_testproto.PingRequest{Value: "something", SleepTimeMs: 9999}
	fakeInboundTraceId      = 1337
	fakeInboundSpanId       = 999
	traceHeaderName         = "uber-trace-id"
	filterFunc              = func(ctx context.Context, fullMethodName string) bool { return true }
	unaryRequestHandlerFunc = func(span opentracing.Span, req interface{}) {
		span.LogFields(log.Bool("unary-request-handler", true))
	}
)

type tracingAssertService struct {
	pb_testproto.TestServiceServer
	T *testing.T
}

func (s *tracingAssertService) Ping(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.PingResponse, error) {
	assert.NotNil(s.T, opentracing.SpanFromContext(ctx), "handlers must have the spancontext in their context, otherwise propagation will fail")
	tags := grpc_ctxtags.Extract(ctx)
	assert.True(s.T, tags.Has(grpc_opentracing.TagTraceId), "tags must contain traceid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSpanId), "tags must contain spanid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSampled), "tags must contain sampled")
	assert.Equal(s.T, tags.Values()[grpc_opentracing.TagSampled], "true", "sampled must be set to true")
	return s.TestServiceServer.Ping(ctx, ping)
}

func (s *tracingAssertService) PingError(ctx context.Context, ping *pb_testproto.PingRequest) (*pb_testproto.Empty, error) {
	assert.NotNil(s.T, opentracing.SpanFromContext(ctx), "handlers must have the spancontext in their context, otherwise propagation will fail")
	return s.TestServiceServer.PingError(ctx, ping)
}

func (s *tracingAssertService) PingList(ping *pb_testproto.PingRequest, stream pb_testproto.TestService_PingListServer) error {
	assert.NotNil(s.T, opentracing.SpanFromContext(stream.Context()), "handlers must have the spancontext in their context, otherwise propagation will fail")
	tags := grpc_ctxtags.Extract(stream.Context())
	assert.True(s.T, tags.Has(grpc_opentracing.TagTraceId), "tags must contain traceid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSpanId), "tags must contain spanid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSampled), "tags must contain sampled")
	assert.Equal(s.T, tags.Values()[grpc_opentracing.TagSampled], "true", "sampled must be set to true")
	return s.TestServiceServer.PingList(ping, stream)
}

func (s *tracingAssertService) PingEmpty(ctx context.Context, empty *pb_testproto.Empty) (*pb_testproto.PingResponse, error) {
	assert.NotNil(s.T, opentracing.SpanFromContext(ctx), "handlers must have the spancontext in their context, otherwise propagation will fail")
	tags := grpc_ctxtags.Extract(ctx)
	assert.True(s.T, tags.Has(grpc_opentracing.TagTraceId), "tags must contain traceid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSpanId), "tags must contain spanid")
	assert.True(s.T, tags.Has(grpc_opentracing.TagSampled), "tags must contain sampled")
	assert.Equal(s.T, tags.Values()[grpc_opentracing.TagSampled], "false", "sampled must be set to false")
	return s.TestServiceServer.PingEmpty(ctx, empty)
}

func TestTaggingSuite(t *testing.T) {
	mockTracer := mocktracer.New()
	opts := []grpc_opentracing.Option{
		grpc_opentracing.WithTracer(mockTracer),
		grpc_opentracing.WithFilterFunc(filterFunc),
		grpc_opentracing.WithTraceHeaderName(traceHeaderName),
		grpc_opentracing.WithUnaryRequestHandlerFunc(unaryRequestHandlerFunc),
	}
	s := &OpentracingSuite{
		mockTracer:           mockTracer,
		InterceptorTestSuite: makeInterceptorTestSuite(t, opts),
	}
	suite.Run(t, s)
}

func TestTaggingSuiteJaeger(t *testing.T) {
	mockTracer := mocktracer.New()
	mockTracer.RegisterInjector(opentracing.HTTPHeaders, jaegerFormatInjector{})
	mockTracer.RegisterExtractor(opentracing.HTTPHeaders, jaegerFormatExtractor{})
	opts := []grpc_opentracing.Option{
		grpc_opentracing.WithTracer(mockTracer),
		grpc_opentracing.WithUnaryRequestHandlerFunc(unaryRequestHandlerFunc),
	}
	s := &OpentracingSuite{
		mockTracer:           mockTracer,
		InterceptorTestSuite: makeInterceptorTestSuite(t, opts),
	}
	suite.Run(t, s)
}

func makeInterceptorTestSuite(t *testing.T, opts []grpc_opentracing.Option) *grpc_testing.InterceptorTestSuite {
	return &grpc_testing.InterceptorTestSuite{
		TestService: &tracingAssertService{TestServiceServer: &grpc_testing.TestPingService{T: t}, T: t},
		ClientOpts: []grpc.DialOption{
			grpc.WithUnaryInterceptor(grpc_opentracing.UnaryClientInterceptor(opts...)),
			grpc.WithStreamInterceptor(grpc_opentracing.StreamClientInterceptor(opts...)),
		},
		ServerOpts: []grpc.ServerOption{
			grpc_middleware.WithStreamServerChain(
				grpc_ctxtags.StreamServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
				grpc_opentracing.StreamServerInterceptor(opts...)),
			grpc_middleware.WithUnaryServerChain(
				grpc_ctxtags.UnaryServerInterceptor(grpc_ctxtags.WithFieldExtractor(grpc_ctxtags.CodeGenRequestFieldExtractor)),
				grpc_opentracing.UnaryServerInterceptor(opts...)),
		},
	}
}

type OpentracingSuite struct {
	*grpc_testing.InterceptorTestSuite
	mockTracer *mocktracer.MockTracer
}

func (s *OpentracingSuite) SetupTest() {
	s.mockTracer.Reset()
}

func (s *OpentracingSuite) createContextFromFakeHttpRequestParent(ctx context.Context, sampled bool, opName string) context.Context {
	jFlag := 0
	if sampled {
		jFlag = 1
	}

	if len(opName) == 0 {
		opName = "/fake/parent/http/request"
	}

	hdr := http.Header{}
	hdr.Set(traceHeaderName, fmt.Sprintf("%d:%d:%d:%d", fakeInboundTraceId, fakeInboundSpanId, fakeInboundSpanId, jFlag))
	hdr.Set("mockpfx-ids-traceid", fmt.Sprint(fakeInboundTraceId))
	hdr.Set("mockpfx-ids-spanid", fmt.Sprint(fakeInboundSpanId))
	hdr.Set("mockpfx-ids-sampled", fmt.Sprint(sampled))

	parentSpanContext, err := s.mockTracer.Extract(opentracing.HTTPHeaders, opentracing.HTTPHeadersCarrier(hdr))
	require.NoError(s.T(), err, "parsing a fake HTTP request headers shouldn't fail, ever")
	fakeSpan := s.mockTracer.StartSpan(
		opName,
		// this is magical, it attaches the new span to the parent parentSpanContext, and creates an unparented one if empty.
		opentracing.ChildOf(parentSpanContext),
	)
	fakeSpan.Finish()
	return opentracing.ContextWithSpan(ctx, fakeSpan)
}

func (s *OpentracingSuite) assertTracesCreated(methodName string) (clientSpan *mocktracer.MockSpan, serverSpan *mocktracer.MockSpan) {
	spans := s.mockTracer.FinishedSpans()
	for _, span := range spans {
		s.T().Logf("span: %v, tags: %v", span, span.Tags())
	}
	require.Len(s.T(), spans, 3, "should record 3 spans: one fake inbound, one client, one server")
	traceIdAssert := fmt.Sprintf("traceId=%d", fakeInboundTraceId)
	for _, span := range spans {
		assert.Contains(s.T(), span.String(), traceIdAssert, "not part of the fake parent trace: %v", span)
		if span.OperationName == methodName {
			kind := fmt.Sprintf("%v", span.Tag("span.kind"))
			if kind == "client" {
				clientSpan = span
			} else if kind == "server" {
				serverSpan = span
			}
			assert.EqualValues(s.T(), span.Tag("component"), "gRPC", "span must be tagged with gRPC component")
		}
	}
	require.NotNil(s.T(), clientSpan, "client span must be there")
	require.NotNil(s.T(), serverSpan, "server span must be there")
	assert.EqualValues(s.T(), serverSpan.Tag("grpc.request.value"), "something", "grpc_ctxtags must be propagated, in this case ones from request fields")
	return clientSpan, serverSpan
}

func (s *OpentracingSuite) TestPing_PropagatesTraces() {
	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, "")
	_, err := s.Client.Ping(ctx, goodPing)
	require.NoError(s.T(), err, "there must be not be an on a successful call")
	s.assertTracesCreated("/mwitkow.testproto.TestService/Ping")
}

func (s *OpentracingSuite) TestPing_CustomOpName() {
	customOpName := "customOpName"

	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, customOpName)
	_, err := s.Client.Ping(ctx, goodPing)
	require.NoError(s.T(), err, "there must be not be an error on a successful call")

	spans := s.mockTracer.FinishedSpans()
	spanOpNames := make([]string, len(spans))
	for _, span := range spans {
		spanOpNames = append(spanOpNames, span.OperationName)
	}

	require.Contains(s.T(), spanOpNames, customOpName, "finished spans must contain the custom operation name")

}

func (s *OpentracingSuite) TestPing_WithUnaryRequestHandlerFunc() {
	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, "")
	_, err := s.Client.Ping(ctx, goodPing)
	require.NoError(s.T(), err, "there must be not be an on a successful call")

	var hasLogKey bool
Loop:
	for _, span := range s.mockTracer.FinishedSpans() {
		for _, record := range span.Logs() {
			for _, field := range record.Fields {
				if field.Key == "unary-request-handler" {
					hasLogKey = true
					break Loop
				}
			}
		}
	}
	require.True(s.T(), hasLogKey, "span field 'unary-request-handler' not found")
}

func (s *OpentracingSuite) TestPing_ClientContextTags() {
	const name = "opentracing.custom"
	ctx := grpc_opentracing.ClientAddContextTags(
		s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, ""),
		opentracing.Tags{name: ""},
	)

	_, err := s.Client.Ping(ctx, goodPing)
	require.NoError(s.T(), err, "there must be not be an on a successful call")

	for _, span := range s.mockTracer.FinishedSpans() {
		if span.OperationName == "/mwitkow.testproto.TestService/Ping" {
			kind := fmt.Sprintf("%v", span.Tag("span.kind"))
			if kind == "client" {
				assert.Contains(s.T(), span.Tags(), name, "custom opentracing.Tags must be included in context")
			}
		}
	}
}

func (s *OpentracingSuite) TestPingList_PropagatesTraces() {
	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, "")
	stream, err := s.Client.PingList(ctx, goodPing)
	require.NoError(s.T(), err, "should not fail on establishing the stream")
	for {
		_, err := stream.Recv()
		if err == io.EOF {
			break
		}
		require.NoError(s.T(), err, "reading stream should not fail")
	}
	s.assertTracesCreated("/mwitkow.testproto.TestService/PingList")
}

func (s *OpentracingSuite) TestPingError_PropagatesTraces() {
	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), true, "")
	erroringPing := &pb_testproto.PingRequest{Value: "something", ErrorCodeReturned: uint32(codes.OutOfRange)}
	_, err := s.Client.PingError(ctx, erroringPing)
	require.Error(s.T(), err, "there must be an error returned here")
	clientSpan, serverSpan := s.assertTracesCreated("/mwitkow.testproto.TestService/PingError")
	assert.Equal(s.T(), true, clientSpan.Tag("error"), "client span needs to be marked as an error")
	assert.Equal(s.T(), true, serverSpan.Tag("error"), "server span needs to be marked as an error")
}

func (s *OpentracingSuite) TestPingEmpty_NotSampleTraces() {
	ctx := s.createContextFromFakeHttpRequestParent(s.SimpleCtx(), false, "")
	_, err := s.Client.PingEmpty(ctx, &pb_testproto.Empty{})
	require.NoError(s.T(), err, "there must be not be an on a successful call")
}

type jaegerFormatInjector struct{}

func (jaegerFormatInjector) Inject(ctx mocktracer.MockSpanContext, carrier interface{}) error {
	w := carrier.(opentracing.TextMapWriter)
	flags := 0
	if ctx.Sampled {
		flags = 1
	}
	w.Set(traceHeaderName, fmt.Sprintf("%d:%d::%d", ctx.TraceID, ctx.SpanID, flags))

	return nil
}

type jaegerFormatExtractor struct{}

func (jaegerFormatExtractor) Extract(carrier interface{}) (mocktracer.MockSpanContext, error) {
	rval := mocktracer.MockSpanContext{Sampled: true}
	reader, ok := carrier.(opentracing.TextMapReader)
	if !ok {
		return rval, opentracing.ErrInvalidCarrier
	}
	err := reader.ForeachKey(func(key, val string) error {
		lowerKey := strings.ToLower(key)
		switch {
		case lowerKey == traceHeaderName:
			parts := strings.Split(val, ":")
			if len(parts) != 4 {
				return errors.New("invalid trace id format")
			}
			traceId, err := strconv.Atoi(parts[0])
			if err != nil {
				return err
			}
			rval.TraceID = traceId
			spanId, err := strconv.Atoi(parts[1])
			if err != nil {
				return err
			}
			rval.SpanID = spanId
			flags, err := strconv.Atoi(parts[3])
			if err != nil {
				return err
			}
			rval.Sampled = flags%2 == 1
		}
		return nil
	})
	if rval.TraceID == 0 || rval.SpanID == 0 {
		return rval, opentracing.ErrSpanContextNotFound
	}
	if err != nil {
		return rval, err
	}
	return rval, nil
}