// Package tracing includes useful functions for opentracing annotations.
package tracing

import (
	"go.opencensus.io/trace"
)

// AnnotateError on span. This should be used any time a particular span experiences an error.
func AnnotateError(span *trace.Span, err error) {
	if err == nil {
		return
	}
	span.AddAttributes(trace.BoolAttribute("error", true))
	span.SetStatus(trace.Status{
		Code:    trace.StatusCodeUnknown,
		Message: err.Error(),
	})
}
