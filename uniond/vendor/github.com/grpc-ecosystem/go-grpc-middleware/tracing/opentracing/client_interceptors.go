// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_opentracing

import (
	"context"
	"io"
	"sync"

	"github.com/grpc-ecosystem/go-grpc-middleware/util/metautils"
	opentracing "github.com/opentracing/opentracing-go"
	"github.com/opentracing/opentracing-go/ext"
	"github.com/opentracing/opentracing-go/log"
	"google.golang.org/grpc"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/metadata"
)

// UnaryClientInterceptor returns a new unary client interceptor for OpenTracing.
func UnaryClientInterceptor(opts ...Option) grpc.UnaryClientInterceptor {
	o := evaluateOptions(opts)
	return func(parentCtx context.Context, method string, req, reply interface{}, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		if o.filterOutFunc != nil && !o.filterOutFunc(parentCtx, method) {
			return invoker(parentCtx, method, req, reply, cc, opts...)
		}
		newCtx, clientSpan := newClientSpanFromContext(parentCtx, o.tracer, method)
		if o.unaryRequestHandlerFunc != nil {
			o.unaryRequestHandlerFunc(clientSpan, req)
		}
		err := invoker(newCtx, method, req, reply, cc, opts...)
		finishClientSpan(clientSpan, err)
		return err
	}
}

// StreamClientInterceptor returns a new streaming client interceptor for OpenTracing.
func StreamClientInterceptor(opts ...Option) grpc.StreamClientInterceptor {
	o := evaluateOptions(opts)
	return func(parentCtx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		if o.filterOutFunc != nil && !o.filterOutFunc(parentCtx, method) {
			return streamer(parentCtx, desc, cc, method, opts...)
		}
		newCtx, clientSpan := newClientSpanFromContext(parentCtx, o.tracer, method)
		clientStream, err := streamer(newCtx, desc, cc, method, opts...)
		if err != nil {
			finishClientSpan(clientSpan, err)
			return nil, err
		}
		return &tracedClientStream{ClientStream: clientStream, clientSpan: clientSpan}, nil
	}
}

// type serverStreamingRetryingStream is the implementation of grpc.ClientStream that acts as a
// proxy to the underlying call. If any of the RecvMsg() calls fail, it will try to reestablish
// a new ClientStream according to the retry policy.
type tracedClientStream struct {
	grpc.ClientStream
	mu              sync.Mutex
	alreadyFinished bool
	clientSpan      opentracing.Span
}

func (s *tracedClientStream) Header() (metadata.MD, error) {
	h, err := s.ClientStream.Header()
	if err != nil {
		s.finishClientSpan(err)
	}
	return h, err
}

func (s *tracedClientStream) SendMsg(m interface{}) error {
	err := s.ClientStream.SendMsg(m)
	if err != nil {
		s.finishClientSpan(err)
	}
	return err
}

func (s *tracedClientStream) CloseSend() error {
	err := s.ClientStream.CloseSend()
	s.finishClientSpan(err)
	return err
}

func (s *tracedClientStream) RecvMsg(m interface{}) error {
	err := s.ClientStream.RecvMsg(m)
	if err != nil {
		s.finishClientSpan(err)
	}
	return err
}

func (s *tracedClientStream) finishClientSpan(err error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	if !s.alreadyFinished {
		finishClientSpan(s.clientSpan, err)
		s.alreadyFinished = true
	}
}

// ClientAddContextTags returns a context with specified opentracing tags, which
// are used by UnaryClientInterceptor/StreamClientInterceptor when creating a
// new span.
func ClientAddContextTags(ctx context.Context, tags opentracing.Tags) context.Context {
	return context.WithValue(ctx, clientSpanTagKey{}, tags)
}

type clientSpanTagKey struct{}

func newClientSpanFromContext(ctx context.Context, tracer opentracing.Tracer, fullMethodName string) (context.Context, opentracing.Span) {
	var parentSpanCtx opentracing.SpanContext
	if parent := opentracing.SpanFromContext(ctx); parent != nil {
		parentSpanCtx = parent.Context()
	}
	opts := []opentracing.StartSpanOption{
		opentracing.ChildOf(parentSpanCtx),
		ext.SpanKindRPCClient,
		grpcTag,
	}
	if tagx := ctx.Value(clientSpanTagKey{}); tagx != nil {
		if opt, ok := tagx.(opentracing.StartSpanOption); ok {
			opts = append(opts, opt)
		}
	}
	clientSpan := tracer.StartSpan(fullMethodName, opts...)
	// Make sure we add this to the metadata of the call, so it gets propagated:
	md := metautils.ExtractOutgoing(ctx).Clone()
	if err := tracer.Inject(clientSpan.Context(), opentracing.HTTPHeaders, metadataTextMap(md)); err != nil {
		grpclog.Infof("grpc_opentracing: failed serializing trace information: %v", err)
	}
	ctxWithMetadata := md.ToOutgoing(ctx)
	return opentracing.ContextWithSpan(ctxWithMetadata, clientSpan), clientSpan
}

func finishClientSpan(clientSpan opentracing.Span, err error) {
	if err != nil && err != io.EOF {
		ext.Error.Set(clientSpan, true)
		clientSpan.LogFields(log.String("event", "error"), log.String("message", err.Error()))
	}
	clientSpan.Finish()
}
