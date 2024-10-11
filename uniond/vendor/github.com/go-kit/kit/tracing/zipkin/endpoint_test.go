package zipkin_test

import (
	"context"
	"testing"

	"github.com/openzipkin/zipkin-go"
	"github.com/openzipkin/zipkin-go/reporter/recorder"

	"github.com/go-kit/kit/endpoint"
	zipkinkit "github.com/go-kit/kit/tracing/zipkin"
)

const spanName = "test"

func TestTraceEndpoint(t *testing.T) {
	rec := recorder.NewReporter()
	tr, _ := zipkin.NewTracer(rec)
	mw := zipkinkit.TraceEndpoint(tr, spanName)
	mw(endpoint.Nop)(context.Background(), nil)

	spans := rec.Flush()

	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, wanted %d, got %d", want, have)
	}

	if want, have := spanName, spans[0].Name; want != have {
		t.Fatalf("incorrect span name, wanted %s, got %s", want, have)
	}
}
