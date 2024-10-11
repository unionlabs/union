package opencensus_test

import (
	"context"
	"errors"
	"testing"
	"time"

	"go.opencensus.io/trace"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/kit/sd/lb"
	"github.com/go-kit/kit/tracing/opencensus"
)

const (
	span1 = ""
	span2 = "SPAN-2"
	span3 = "SPAN-3"
	span4 = "SPAN-4"
	span5 = "SPAN-5"
	span6 = "SPAN-6"
)

var (
	err1 = errors.New("some error")
	err2 = errors.New("other error")
	err3 = errors.New("some business error")
	err4 = errors.New("other business error")
)

// compile time assertion
var _ endpoint.Failer = failedResponse{}

type failedResponse struct {
	err error
}

func (r failedResponse) Failed() error { return r.err }

func passEndpoint(_ context.Context, req interface{}) (interface{}, error) {
	if err, _ := req.(error); err != nil {
		return nil, err
	}
	return req, nil
}

func TestTraceEndpoint(t *testing.T) {
	ctx := context.Background()

	e := &recordingExporter{}
	trace.RegisterExporter(e)
	trace.ApplyConfig(trace.Config{DefaultSampler: trace.AlwaysSample()})

	// span 1
	span1Attrs := []trace.Attribute{
		trace.StringAttribute("string", "value"),
		trace.Int64Attribute("int64", 42),
	}
	mw := opencensus.TraceEndpoint(
		span1, opencensus.WithEndpointAttributes(span1Attrs...),
	)
	mw(endpoint.Nop)(ctx, nil)

	// span 2
	opts := opencensus.EndpointOptions{}
	mw = opencensus.TraceEndpoint(span2, opencensus.WithEndpointConfig(opts))
	mw(passEndpoint)(ctx, err1)

	// span3
	mw = opencensus.TraceEndpoint(span3)
	ep := lb.Retry(5, 1*time.Second, lb.NewRoundRobin(sd.FixedEndpointer{passEndpoint}))
	mw(ep)(ctx, err2)

	// span4
	mw = opencensus.TraceEndpoint(span4)
	mw(passEndpoint)(ctx, failedResponse{err: err3})

	// span5
	mw = opencensus.TraceEndpoint(span5, opencensus.WithIgnoreBusinessError(true))
	mw(passEndpoint)(ctx, failedResponse{err: err4})

	// span6
	span6Attrs := []trace.Attribute{
		trace.StringAttribute("string", "value"),
		trace.Int64Attribute("int64", 42),
	}
	mw = opencensus.TraceEndpoint(
		"",
		opencensus.WithSpanName(func(ctx context.Context, name string) string {
			return span6
		}),
		opencensus.WithSpanAttributes(func(ctx context.Context) []trace.Attribute {
			return span6Attrs
		}),
	)
	mw(endpoint.Nop)(ctx, nil)

	// check span count
	spans := e.Flush()
	if want, have := 6, len(spans); want != have {
		t.Fatalf("incorrected number of spans, wanted %d, got %d", want, have)
	}

	// test span 1
	span := spans[0]
	if want, have := int32(trace.StatusCodeOK), span.Code; want != have {
		t.Errorf("incorrect status code, wanted %d, got %d", want, have)
	}

	if want, have := opencensus.TraceEndpointDefaultName, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 2, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}

	// test span 2
	span = spans[1]
	if want, have := int32(trace.StatusCodeUnknown), span.Code; want != have {
		t.Errorf("incorrect status code, wanted %d, got %d", want, have)
	}

	if want, have := span2, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 0, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}

	// test span 3
	span = spans[2]
	if want, have := int32(trace.StatusCodeUnknown), span.Code; want != have {
		t.Errorf("incorrect status code, wanted %d, got %d", want, have)
	}

	if want, have := span3, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 5, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}

	// test span 4
	span = spans[3]
	if want, have := int32(trace.StatusCodeUnknown), span.Code; want != have {
		t.Errorf("incorrect status code, wanted %d, got %d", want, have)
	}

	if want, have := span4, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 1, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}

	// test span 5
	span = spans[4]
	if want, have := int32(trace.StatusCodeOK), span.Code; want != have {
		t.Errorf("incorrect status code, wanted %d, got %d", want, have)
	}

	if want, have := span5, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 1, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}

	// test span 6
	span = spans[5]
	if want, have := span6, span.Name; want != have {
		t.Errorf("incorrect span name, wanted %q, got %q", want, have)
	}

	if want, have := 2, len(span.Attributes); want != have {
		t.Fatalf("incorrect attribute count, wanted %d, got %d", want, have)
	}
}
