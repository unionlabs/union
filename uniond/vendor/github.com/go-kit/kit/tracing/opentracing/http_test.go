package opentracing_test

import (
	"context"
	"net/http"
	"reflect"
	"testing"

	opentracing "github.com/opentracing/opentracing-go"
	"github.com/opentracing/opentracing-go/ext"
	"github.com/opentracing/opentracing-go/mocktracer"

	kitot "github.com/go-kit/kit/tracing/opentracing"
	"github.com/go-kit/log"
)

func TestTraceHTTPRequestRoundtrip(t *testing.T) {
	logger := log.NewNopLogger()
	tracer := mocktracer.New()

	// Initialize the ctx with a Span to inject.
	beforeSpan := tracer.StartSpan("to_inject").(*mocktracer.MockSpan)
	defer beforeSpan.Finish()
	beforeSpan.SetBaggageItem("baggage", "check")
	beforeCtx := opentracing.ContextWithSpan(context.Background(), beforeSpan)

	toHTTPFunc := kitot.ContextToHTTP(tracer, logger)
	req, _ := http.NewRequest("GET", "http://test.biz/path", nil)
	// Call the RequestFunc.
	afterCtx := toHTTPFunc(beforeCtx, req)

	// The Span should not have changed.
	afterSpan := opentracing.SpanFromContext(afterCtx)
	if beforeSpan != afterSpan {
		t.Error("Should not swap in a new span")
	}

	// No spans should have finished yet.
	finishedSpans := tracer.FinishedSpans()
	if want, have := 0, len(finishedSpans); want != have {
		t.Errorf("Want %v span(s), found %v", want, have)
	}

	// Use HTTPToContext to verify that we can join with the trace given a req.
	fromHTTPFunc := kitot.HTTPToContext(tracer, "joined", logger)
	joinCtx := fromHTTPFunc(afterCtx, req)
	joinedSpan := opentracing.SpanFromContext(joinCtx).(*mocktracer.MockSpan)

	joinedContext := joinedSpan.Context().(mocktracer.MockSpanContext)
	beforeContext := beforeSpan.Context().(mocktracer.MockSpanContext)

	if joinedContext.SpanID == beforeContext.SpanID {
		t.Error("SpanID should have changed", joinedContext.SpanID, beforeContext.SpanID)
	}

	// Check that the parent/child relationship is as expected for the joined span.
	if want, have := beforeContext.SpanID, joinedSpan.ParentID; want != have {
		t.Errorf("Want ParentID %q, have %q", want, have)
	}
	if want, have := "joined", joinedSpan.OperationName; want != have {
		t.Errorf("Want %q, have %q", want, have)
	}
	if want, have := "check", joinedSpan.BaggageItem("baggage"); want != have {
		t.Errorf("Want %q, have %q", want, have)
	}
}

func TestContextToHTTPTags(t *testing.T) {
	tracer := mocktracer.New()
	span := tracer.StartSpan("to_inject").(*mocktracer.MockSpan)
	defer span.Finish()
	ctx := opentracing.ContextWithSpan(context.Background(), span)
	req, _ := http.NewRequest("GET", "http://test.biz/path", nil)

	kitot.ContextToHTTP(tracer, log.NewNopLogger())(ctx, req)

	expectedTags := map[string]interface{}{
		string(ext.HTTPMethod):   "GET",
		string(ext.HTTPUrl):      "http://test.biz/path",
		string(ext.PeerHostname): "test.biz",
	}
	if !reflect.DeepEqual(expectedTags, span.Tags()) {
		t.Errorf("Want %q, have %q", expectedTags, span.Tags())
	}
}

func TestHTTPToContextTags(t *testing.T) {
	tracer := mocktracer.New()
	parentSpan := tracer.StartSpan("to_extract").(*mocktracer.MockSpan)
	defer parentSpan.Finish()
	req, _ := http.NewRequest("GET", "http://test.biz/path", nil)
	tracer.Inject(parentSpan.Context(), opentracing.HTTPHeaders, opentracing.HTTPHeadersCarrier(req.Header))

	ctx := kitot.HTTPToContext(tracer, "op", log.NewNopLogger())(context.Background(), req)
	opentracing.SpanFromContext(ctx).Finish()

	childSpan := tracer.FinishedSpans()[0]
	expectedTags := map[string]interface{}{
		string(ext.HTTPMethod): "GET",
		string(ext.HTTPUrl):    "http://test.biz/path",
		string(ext.SpanKind):   ext.SpanKindRPCServerEnum,
	}
	if !reflect.DeepEqual(expectedTags, childSpan.Tags()) {
		t.Errorf("Want %q, have %q", expectedTags, childSpan.Tags())
	}
	if want, have := "op", childSpan.OperationName; want != have {
		t.Errorf("Want %q, have %q", want, have)
	}
}
