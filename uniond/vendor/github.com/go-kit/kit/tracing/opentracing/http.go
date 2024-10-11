package opentracing

import (
	"context"
	"net"
	"net/http"
	"strconv"

	opentracing "github.com/opentracing/opentracing-go"
	"github.com/opentracing/opentracing-go/ext"

	kithttp "github.com/go-kit/kit/transport/http"
	"github.com/go-kit/log"
)

// ContextToHTTP returns an http RequestFunc that injects an OpenTracing Span
// found in `ctx` into the http headers. If no such Span can be found, the
// RequestFunc is a noop.
func ContextToHTTP(tracer opentracing.Tracer, logger log.Logger) kithttp.RequestFunc {
	return func(ctx context.Context, req *http.Request) context.Context {
		// Try to find a Span in the Context.
		if span := opentracing.SpanFromContext(ctx); span != nil {
			// Add standard OpenTracing tags.
			ext.HTTPMethod.Set(span, req.Method)
			ext.HTTPUrl.Set(span, req.URL.String())
			host, portString, err := net.SplitHostPort(req.URL.Host)
			if err == nil {
				ext.PeerHostname.Set(span, host)
				if port, err := strconv.Atoi(portString); err == nil {
					ext.PeerPort.Set(span, uint16(port))
				}
			} else {
				ext.PeerHostname.Set(span, req.URL.Host)
			}

			// There's nothing we can do with any errors here.
			if err = tracer.Inject(
				span.Context(),
				opentracing.HTTPHeaders,
				opentracing.HTTPHeadersCarrier(req.Header),
			); err != nil {
				logger.Log("err", err)
			}
		}
		return ctx
	}
}

// HTTPToContext returns an http RequestFunc that tries to join with an
// OpenTracing trace found in `req` and starts a new Span called
// `operationName` accordingly. If no trace could be found in `req`, the Span
// will be a trace root. The Span is incorporated in the returned Context and
// can be retrieved with opentracing.SpanFromContext(ctx).
func HTTPToContext(tracer opentracing.Tracer, operationName string, logger log.Logger) kithttp.RequestFunc {
	return func(ctx context.Context, req *http.Request) context.Context {
		// Try to join to a trace propagated in `req`.
		var span opentracing.Span
		wireContext, err := tracer.Extract(
			opentracing.HTTPHeaders,
			opentracing.HTTPHeadersCarrier(req.Header),
		)
		if err != nil && err != opentracing.ErrSpanContextNotFound {
			logger.Log("err", err)
		}

		span = tracer.StartSpan(operationName, ext.RPCServerOption(wireContext))
		ext.HTTPMethod.Set(span, req.Method)
		ext.HTTPUrl.Set(span, req.URL.String())
		return opentracing.ContextWithSpan(ctx, span)
	}
}
