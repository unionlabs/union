package zipkin_test

import (
	"context"
	"testing"

	zipkin "github.com/openzipkin/zipkin-go"
	"github.com/openzipkin/zipkin-go/propagation/b3"
	"github.com/openzipkin/zipkin-go/reporter/recorder"
	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

	"github.com/go-kit/kit/endpoint"
	kitzipkin "github.com/go-kit/kit/tracing/zipkin"
	grpctransport "github.com/go-kit/kit/transport/grpc"
)

type dummy struct{}

func unaryInterceptor(
	ctx context.Context, method string, req, reply interface{},
	cc *grpc.ClientConn, invoker grpc.UnaryInvoker, opts ...grpc.CallOption,
) error {
	return nil
}

func TestGRPCClientTrace(t *testing.T) {
	rec := recorder.NewReporter()
	defer rec.Close()

	tr, _ := zipkin.NewTracer(rec)

	clientTracer := kitzipkin.GRPCClientTrace(tr)

	cc, err := grpc.Dial(
		"",
		grpc.WithUnaryInterceptor(unaryInterceptor),
		grpc.WithInsecure(),
	)
	if err != nil {
		t.Fatalf("unable to create gRPC dialer: %s", err.Error())
	}

	ep := grpctransport.NewClient(
		cc,
		"dummyService",
		"dummyMethod",
		func(context.Context, interface{}) (interface{}, error) { return nil, nil },
		func(context.Context, interface{}) (interface{}, error) { return nil, nil },
		dummy{},
		clientTracer,
	).Endpoint()

	parentSpan := tr.StartSpan("test")
	ctx := zipkin.NewContext(context.Background(), parentSpan)

	if _, err = ep(ctx, nil); err != nil {
		t.Errorf("unexpected error: %s", err.Error())
	}

	spans := rec.Flush()
	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
	}

	if spans[0].SpanContext.ParentID == nil {
		t.Fatalf("incorrect parent ID, want %s have nil", parentSpan.Context().ID)
	}

	if want, have := parentSpan.Context().ID, *spans[0].SpanContext.ParentID; want != have {
		t.Fatalf("incorrect parent ID, want %s, have %s", want, have)
	}
}

func TestGRPCServerTrace(t *testing.T) {
	rec := recorder.NewReporter()
	defer rec.Close()

	tr, _ := zipkin.NewTracer(rec)

	serverTracer := kitzipkin.GRPCServerTrace(tr)

	server := grpctransport.NewServer(
		endpoint.Nop,
		func(context.Context, interface{}) (interface{}, error) { return nil, nil },
		func(context.Context, interface{}) (interface{}, error) { return nil, nil },
		serverTracer,
	)

	md := metadata.MD{}
	parentSpan := tr.StartSpan("test")

	b3.InjectGRPC(&md)(parentSpan.Context())

	ctx := metadata.NewIncomingContext(context.Background(), md)
	server.ServeGRPC(ctx, nil)

	spans := rec.Flush()

	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
	}

	if want, have := parentSpan.Context().TraceID, spans[0].SpanContext.TraceID; want != have {
		t.Errorf("incorrect TraceID, want %+v, have %+v", want, have)
	}

	if want, have := parentSpan.Context().ID, spans[0].SpanContext.ID; want != have {
		t.Errorf("incorrect span ID, want %d, have %d", want, have)
	}
}
