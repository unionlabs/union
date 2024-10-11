package opencensus_test

import (
	"context"
	"errors"
	"testing"

	"go.opencensus.io/trace"
	"go.opencensus.io/trace/propagation"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"

	"github.com/go-kit/kit/endpoint"
	ockit "github.com/go-kit/kit/tracing/opencensus"
	grpctransport "github.com/go-kit/kit/transport/grpc"
)

type dummy struct{}

const traceContextKey = "grpc-trace-bin"

func unaryInterceptor(
	ctx context.Context, method string, req, reply interface{},
	cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption,
) error {
	return nil
}

func TestGRPCClientTrace(t *testing.T) {
	rec := &recordingExporter{}

	trace.RegisterExporter(rec)

	cc, err := grpc.Dial(
		"",
		grpc.WithUnaryInterceptor(unaryInterceptor),
		grpc.WithInsecure(),
	)
	if err != nil {
		t.Fatalf("unable to create gRPC dialer: %s", err.Error())
	}

	traces := []struct {
		name string
		err  error
	}{
		{"", nil},
		{"CustomName", nil},
		{"", errors.New("dummy-error")},
	}

	for _, tr := range traces {
		clientTracer := ockit.GRPCClientTrace(
			ockit.WithName(tr.name),
			ockit.WithSampler(trace.AlwaysSample()),
		)

		ep := grpctransport.NewClient(
			cc,
			"dummyService",
			"dummyMethod",
			func(context.Context, interface{}) (interface{}, error) {
				return nil, nil
			},
			func(context.Context, interface{}) (interface{}, error) {
				return nil, tr.err
			},
			dummy{},
			clientTracer,
		).Endpoint()

		ctx, parentSpan := trace.StartSpan(context.Background(), "test")

		_, err = ep(ctx, nil)
		if want, have := tr.err, err; want != have {
			t.Fatalf("unexpected error, want %s, have %s", tr.err.Error(), err.Error())
		}

		spans := rec.Flush()
		if want, have := 1, len(spans); want != have {
			t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
		}
		span := spans[0]
		if want, have := parentSpan.SpanContext().SpanID, span.ParentSpanID; want != have {
			t.Errorf("incorrect parent ID, want %s, have %s", want, have)
		}

		if want, have := tr.name, span.Name; want != have && want != "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}

		if want, have := "/dummyService/dummyMethod", span.Name; want != have && tr.name == "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}

		code := trace.StatusCodeOK
		if tr.err != nil {
			code = trace.StatusCodeUnknown

			if want, have := err.Error(), span.Status.Message; want != have {
				t.Errorf("incorrect span status msg, want %s, have %s", want, have)
			}
		}

		if want, have := int32(code), span.Status.Code; want != have {
			t.Errorf("incorrect span status code, want %d, have %d", want, have)
		}
	}
}

func TestGRPCServerTrace(t *testing.T) {
	rec := &recordingExporter{}

	trace.RegisterExporter(rec)

	traces := []struct {
		useParent bool
		name      string
		err       error
	}{
		{false, "", nil},
		{true, "", nil},
		{true, "CustomName", nil},
		{true, "", errors.New("dummy-error")},
	}

	for _, tr := range traces {
		var (
			ctx        = context.Background()
			parentSpan *trace.Span
		)

		server := grpctransport.NewServer(
			endpoint.Nop,
			func(context.Context, interface{}) (interface{}, error) {
				return nil, nil
			},
			func(context.Context, interface{}) (interface{}, error) {
				return nil, tr.err
			},
			ockit.GRPCServerTrace(
				ockit.WithName(tr.name),
				ockit.WithSampler(trace.AlwaysSample()),
			),
		)

		if tr.useParent {
			_, parentSpan = trace.StartSpan(context.Background(), "test")
			traceContextBinary := propagation.Binary(parentSpan.SpanContext())

			md := metadata.MD{}
			md.Set(traceContextKey, string(traceContextBinary))
			ctx = metadata.NewIncomingContext(ctx, md)
		}

		server.ServeGRPC(ctx, nil)

		spans := rec.Flush()

		if want, have := 1, len(spans); want != have {
			t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
		}

		if tr.useParent {
			if want, have := parentSpan.SpanContext().TraceID, spans[0].TraceID; want != have {
				t.Errorf("incorrect trace ID, want %s, have %s", want, have)
			}

			if want, have := parentSpan.SpanContext().SpanID, spans[0].ParentSpanID; want != have {
				t.Errorf("incorrect span ID, want %s, have %s", want, have)
			}
		}

		if want, have := tr.name, spans[0].Name; want != have && want != "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}

		if tr.err != nil {
			if want, have := int32(codes.Internal), spans[0].Status.Code; want != have {
				t.Errorf("incorrect span status code, want %d, have %d", want, have)
			}

			if want, have := tr.err.Error(), spans[0].Status.Message; want != have {
				t.Errorf("incorrect span status message, want %s, have %s", want, have)
			}
		}
	}
}
