package tracing

import (
	"context"
	"errors"
	"fmt"
	"runtime"
	"runtime/debug"

	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// RecoveryHandlerFunc is a function that recovers from the panic `p` by returning an `error`.
// The context can be used to extract request scoped metadata and context values.
func RecoveryHandlerFunc(ctx context.Context, p interface{}) error {
	span := trace.FromContext(ctx)
	if span != nil {
		span.AddAttributes(trace.StringAttribute("stack", string(debug.Stack())))
	}
	var err error
	switch v := p.(type) {
	case runtime.Error:
		err = errors.New(v.Error())
	default:
		err = fmt.Errorf("%v", p)
	}

	logrus.WithError(err).WithField("stack", string(debug.Stack())).Error("gRPC panicked!")
	return err
}
