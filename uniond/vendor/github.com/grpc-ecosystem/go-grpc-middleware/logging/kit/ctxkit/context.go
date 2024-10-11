package ctxkit

import (
	"context"

	"github.com/go-kit/log"
	grpc_ctxtags "github.com/grpc-ecosystem/go-grpc-middleware/tags"
)

type ctxMarker struct{}

type ctxLogger struct {
	logger log.Logger
	fields []interface{}
}

var (
	ctxMarkerKey = &ctxMarker{}
)

// AddFields adds fields to the logger.
func AddFields(ctx context.Context, fields ...interface{}) {
	l, ok := ctx.Value(ctxMarkerKey).(*ctxLogger)
	if !ok || l == nil {
		return
	}
	l.fields = append(l.fields, fields...)
}

// Extract takes the call-scoped Logger from grpc_kit middleware.
//
// It always returns a Logger that has all the grpc_ctxtags updated.
func Extract(ctx context.Context) log.Logger {
	l, ok := ctx.Value(ctxMarkerKey).(*ctxLogger)
	if !ok || l == nil {
		return log.NewNopLogger()
	}
	// Add grpc_ctxtags tags metadata until now.
	fields := TagsToFields(ctx)
	return log.With(l.logger, append(fields, l.fields...)...)
}

// TagsToFields transforms the Tags on the supplied context into kit fields.
func TagsToFields(ctx context.Context) []interface{} {
	var fields []interface{}
	tags := grpc_ctxtags.Extract(ctx)
	for k, v := range tags.Values() {
		fields = append(fields, k, v)
	}
	return fields
}

// ToContext adds the kit.Logger to the context for extraction later.
// Returning the new context that has been created.
func ToContext(ctx context.Context, logger log.Logger) context.Context {
	l := &ctxLogger{
		logger: logger,
	}
	return context.WithValue(ctx, ctxMarkerKey, l)
}
