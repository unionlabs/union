// Copyright 2017 Michal Witkowski. All Rights Reserved.
// See LICENSE for licensing terms.

package grpc_zap

import (
	"context"
	"path"
	"time"

	"github.com/grpc-ecosystem/go-grpc-middleware/logging/zap/ctxzap"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
	"google.golang.org/grpc"
)

var (
	// ClientField is used in every client-side log statement made through grpc_zap. Can be overwritten before initialization.
	ClientField = zap.String("span.kind", "client")
)

// UnaryClientInterceptor returns a new unary client interceptor that optionally logs the execution of external gRPC calls.
func UnaryClientInterceptor(logger *zap.Logger, opts ...Option) grpc.UnaryClientInterceptor {
	o := evaluateClientOpt(opts)
	return func(ctx context.Context, method string, req, reply interface{}, cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption) error {
		fields := newClientLoggerFields(ctx, method)
		startTime := time.Now()
		err := invoker(ctx, method, req, reply, cc, opts...)
		newCtx := ctxzap.ToContext(ctx, logger.With(fields...))
		logFinalClientLine(newCtx, o, startTime, err, "finished client unary call")
		return err
	}
}

// StreamClientInterceptor returns a new streaming client interceptor that optionally logs the execution of external gRPC calls.
func StreamClientInterceptor(logger *zap.Logger, opts ...Option) grpc.StreamClientInterceptor {
	o := evaluateClientOpt(opts)
	return func(ctx context.Context, desc *grpc.StreamDesc, cc *grpc.ClientConn, method string, streamer grpc.Streamer, opts ...grpc.CallOption) (grpc.ClientStream, error) {
		fields := newClientLoggerFields(ctx, method)
		startTime := time.Now()
		clientStream, err := streamer(ctx, desc, cc, method, opts...)
		newCtx := ctxzap.ToContext(ctx, logger.With(fields...))
		logFinalClientLine(newCtx, o, startTime, err, "finished client streaming call")
		return clientStream, err
	}
}

func logFinalClientLine(ctx context.Context, o *options, startTime time.Time, err error, msg string) {
	code := o.codeFunc(err)
	level := o.levelFunc(code)
	duration := o.durationFunc(time.Now().Sub(startTime))
	o.messageFunc(ctx, msg, level, code, err, duration)
}

func newClientLoggerFields(ctx context.Context, fullMethodString string) []zapcore.Field {
	service := path.Dir(fullMethodString)[1:]
	method := path.Base(fullMethodString)
	return []zapcore.Field{
		SystemField,
		ClientField,
		zap.String("grpc.service", service),
		zap.String("grpc.method", method),
	}
}
