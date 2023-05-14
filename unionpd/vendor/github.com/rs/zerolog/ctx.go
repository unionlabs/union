package zerolog

import (
	"context"
)

var disabledLogger *Logger

func init() {
	SetGlobalLevel(TraceLevel)
	l := Nop()
	disabledLogger = &l
}

type ctxKey struct{}

// WithContext returns a copy of ctx with the receiver attached. The Logger
// attached to the provided Context (if any) will not be effected.  If the
// receiver's log level is Disabled it will only be attached to the returned
// Context if the provided Context has a previously attached Logger. If the
// provided Context has no attached Logger, a Disabled Logger will not be
// attached.
//
// Note: to modify the existing Logger attached to a Context (instead of
// replacing it in a new Context), use UpdateContext with the following
// notation:
//
//     ctx := r.Context()
//     l := zerolog.Ctx(ctx)
//     l.UpdateContext(func(c Context) Context {
//         return c.Str("bar", "baz")
//     })
//
func (l Logger) WithContext(ctx context.Context) context.Context {
	if _, ok := ctx.Value(ctxKey{}).(*Logger); !ok && l.level == Disabled {
		// Do not store disabled logger.
		return ctx
	}
	return context.WithValue(ctx, ctxKey{}, &l)
}

// Ctx returns the Logger associated with the ctx. If no logger
// is associated, DefaultContextLogger is returned, unless DefaultContextLogger
// is nil, in which case a disabled logger is returned.
func Ctx(ctx context.Context) *Logger {
	if l, ok := ctx.Value(ctxKey{}).(*Logger); ok {
		return l
	} else if l = DefaultContextLogger; l != nil {
		return l
	}
	return disabledLogger
}
