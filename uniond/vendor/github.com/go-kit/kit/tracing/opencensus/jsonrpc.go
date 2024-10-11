package opencensus

import (
	"context"
	"net/http"

	"go.opencensus.io/plugin/ochttp"
	"go.opencensus.io/plugin/ochttp/propagation/b3"
	"go.opencensus.io/trace"

	kithttp "github.com/go-kit/kit/transport/http"
	jsonrpc "github.com/go-kit/kit/transport/http/jsonrpc"
)

// JSONRPCClientTrace enables OpenCensus tracing of a Go kit JSONRPC transport client.
func JSONRPCClientTrace(options ...TracerOption) jsonrpc.ClientOption {
	cfg := TracerOptions{}

	for _, option := range options {
		option(&cfg)
	}

	if !cfg.Public && cfg.HTTPPropagate == nil {
		cfg.HTTPPropagate = &b3.HTTPFormat{}
	}

	clientBefore := jsonrpc.ClientBefore(
		func(ctx context.Context, req *http.Request) context.Context {
			var name string

			if cfg.Name != "" {
				name = cfg.Name
			} else {
				// OpenCensus states Path being default naming for a client span
				name = ctx.Value(jsonrpc.ContextKeyRequestMethod).(string)
			}

			ctx, span := trace.StartSpan(
				ctx,
				name,
				trace.WithSampler(cfg.Sampler),
				trace.WithSpanKind(trace.SpanKindClient),
			)

			span.AddAttributes(
				trace.StringAttribute(ochttp.HostAttribute, req.URL.Host),
				trace.StringAttribute(ochttp.MethodAttribute, req.Method),
				trace.StringAttribute(ochttp.PathAttribute, req.URL.Path),
				trace.StringAttribute(ochttp.UserAgentAttribute, req.UserAgent()),
			)

			if !cfg.Public {
				cfg.HTTPPropagate.SpanContextToRequest(span.SpanContext(), req)
			}

			return ctx
		},
	)

	clientAfter := jsonrpc.ClientAfter(
		func(ctx context.Context, res *http.Response) context.Context {
			if span := trace.FromContext(ctx); span != nil {
				span.SetStatus(ochttp.TraceStatus(res.StatusCode, http.StatusText(res.StatusCode)))
				span.AddAttributes(
					trace.Int64Attribute(ochttp.StatusCodeAttribute, int64(res.StatusCode)),
				)
			}
			return ctx
		},
	)

	clientFinalizer := jsonrpc.ClientFinalizer(
		func(ctx context.Context, err error) {
			if span := trace.FromContext(ctx); span != nil {
				if err != nil {
					span.SetStatus(trace.Status{
						Code:    trace.StatusCodeUnknown,
						Message: err.Error(),
					})
				}
				span.End()
			}
		},
	)

	return func(c *jsonrpc.Client) {
		clientBefore(c)
		clientAfter(c)
		clientFinalizer(c)
	}
}

// JSONRPCServerTrace enables OpenCensus tracing of a Go kit JSONRPC transport server.
func JSONRPCServerTrace(options ...TracerOption) jsonrpc.ServerOption {
	cfg := TracerOptions{}

	for _, option := range options {
		option(&cfg)
	}

	if !cfg.Public && cfg.HTTPPropagate == nil {
		cfg.HTTPPropagate = &b3.HTTPFormat{}
	}

	serverBeforeCodec := jsonrpc.ServerBeforeCodec(
		func(ctx context.Context, httpReq *http.Request, req jsonrpc.Request) context.Context {
			var (
				spanContext trace.SpanContext
				span        *trace.Span
				name        string
				ok          bool
			)

			if cfg.Name != "" {
				name = cfg.Name
			} else {
				name = ctx.Value(jsonrpc.ContextKeyRequestMethod).(string)
				if name == "" {
					// we can't find the rpc method. probably the
					// unaryInterceptor was not wired up.
					name = "unknown jsonrpc method"
				}
			}

			spanContext, ok = cfg.HTTPPropagate.SpanContextFromRequest(httpReq)
			if ok && !cfg.Public {
				ctx, span = trace.StartSpanWithRemoteParent(
					ctx,
					name,
					spanContext,
					trace.WithSpanKind(trace.SpanKindServer),
					trace.WithSampler(cfg.Sampler),
				)
			} else {
				ctx, span = trace.StartSpan(
					ctx,
					name,
					trace.WithSpanKind(trace.SpanKindServer),
					trace.WithSampler(cfg.Sampler),
				)
				if ok {
					span.AddLink(trace.Link{
						TraceID:    spanContext.TraceID,
						SpanID:     spanContext.SpanID,
						Type:       trace.LinkTypeChild,
						Attributes: nil,
					})
				}
			}

			span.AddAttributes(
				trace.StringAttribute(ochttp.MethodAttribute, httpReq.Method),
				trace.StringAttribute(ochttp.PathAttribute, httpReq.URL.Path),
			)

			return ctx
		},
	)

	serverFinalizer := jsonrpc.ServerFinalizer(
		func(ctx context.Context, code int, r *http.Request) {
			if span := trace.FromContext(ctx); span != nil {
				span.SetStatus(ochttp.TraceStatus(code, http.StatusText(code)))

				if rs, ok := ctx.Value(kithttp.ContextKeyResponseSize).(int64); ok {
					span.AddAttributes(
						trace.Int64Attribute("http.response_size", rs),
					)
				}

				span.End()
			}
		},
	)

	return func(s *jsonrpc.Server) {
		serverBeforeCodec(s)
		serverFinalizer(s)
	}
}
