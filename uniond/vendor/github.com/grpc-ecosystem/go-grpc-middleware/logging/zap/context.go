package grpc_zap

import (
	"context"

	"github.com/grpc-ecosystem/go-grpc-middleware/logging/zap/ctxzap"
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

// AddFields adds zap fields to the logger.
// Deprecated: should use the ctxzap.AddFields instead
func AddFields(ctx context.Context, fields ...zapcore.Field) {
	ctxzap.AddFields(ctx, fields...)
}

// Extract takes the call-scoped Logger from grpc_zap middleware.
// Deprecated: should use the ctxzap.Extract instead
func Extract(ctx context.Context) *zap.Logger {
	return ctxzap.Extract(ctx)
}
