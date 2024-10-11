package opencensus

import (
	"context"
	"strconv"

	"go.opencensus.io/trace"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd/lb"
)

// TraceEndpointDefaultName is the default endpoint span name to use.
const TraceEndpointDefaultName = "gokit/endpoint"

// TraceEndpoint returns an Endpoint middleware, tracing a Go kit endpoint.
// This endpoint tracer should be used in combination with a Go kit Transport
// tracing middleware, generic OpenCensus transport middleware or custom before
// and after transport functions as service propagation of SpanContext is not
// provided in this middleware.
func TraceEndpoint(name string, options ...EndpointOption) endpoint.Middleware {
	if name == "" {
		name = TraceEndpointDefaultName
	}

	cfg := &EndpointOptions{}

	for _, o := range options {
		o(cfg)
	}

	return func(next endpoint.Endpoint) endpoint.Endpoint {
		return func(ctx context.Context, request interface{}) (response interface{}, err error) {
			if cfg.GetName != nil {
				if newName := cfg.GetName(ctx, name); newName != "" {
					name = newName
				}
			}

			ctx, span := trace.StartSpan(ctx, name)
			if len(cfg.Attributes) > 0 {
				span.AddAttributes(cfg.Attributes...)
			}
			defer span.End()

			if cfg.GetAttributes != nil {
				if attrs := cfg.GetAttributes(ctx); len(attrs) > 0 {
					span.AddAttributes(attrs...)
				}
			}

			defer func() {
				if err != nil {
					if lberr, ok := err.(lb.RetryError); ok {
						// handle errors originating from lb.Retry
						attrs := make([]trace.Attribute, 0, len(lberr.RawErrors))
						for idx, rawErr := range lberr.RawErrors {
							attrs = append(attrs, trace.StringAttribute(
								"gokit.retry.error."+strconv.Itoa(idx+1), rawErr.Error(),
							))
						}
						span.AddAttributes(attrs...)
						span.SetStatus(trace.Status{
							Code:    trace.StatusCodeUnknown,
							Message: lberr.Final.Error(),
						})
						return
					}
					// generic error
					span.SetStatus(trace.Status{
						Code:    trace.StatusCodeUnknown,
						Message: err.Error(),
					})
					return
				}

				// test for business error
				if res, ok := response.(endpoint.Failer); ok && res.Failed() != nil {
					span.AddAttributes(
						trace.StringAttribute("gokit.business.error", res.Failed().Error()),
					)
					if cfg.IgnoreBusinessError {
						span.SetStatus(trace.Status{Code: trace.StatusCodeOK})
						return
					}
					// treating business error as real error in span.
					span.SetStatus(trace.Status{
						Code:    trace.StatusCodeUnknown,
						Message: res.Failed().Error(),
					})
					return
				}

				// no errors identified
				span.SetStatus(trace.Status{Code: trace.StatusCodeOK})
			}()
			response, err = next(ctx, request)
			return
		}
	}
}
