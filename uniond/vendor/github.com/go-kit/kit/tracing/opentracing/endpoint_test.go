package opentracing_test

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"testing"
	"time"

	"github.com/opentracing/opentracing-go"
	otext "github.com/opentracing/opentracing-go/ext"
	otlog "github.com/opentracing/opentracing-go/log"
	"github.com/opentracing/opentracing-go/mocktracer"

	"github.com/go-kit/kit/endpoint"
	"github.com/go-kit/kit/sd"
	"github.com/go-kit/kit/sd/lb"
	kitot "github.com/go-kit/kit/tracing/opentracing"
)

const (
	span1 = "SPAN-1"
	span2 = "SPAN-2"
	span3 = "SPAN-3"
	span4 = "SPAN-4"
	span5 = "SPAN-5"
	span6 = "SPAN-6"
	span7 = "SPAN-7"
	span8 = "SPAN-8"
)

var (
	err1 = errors.New("some error")
	err2 = errors.New("some business error")
	err3 = errors.New("other business error")
)

// compile time assertion
var _ endpoint.Failer = failedResponse{}

type failedResponse struct {
	err error
}

func (r failedResponse) Failed() error {
	return r.err
}

func TestTraceEndpoint(t *testing.T) {
	tracer := mocktracer.New()

	// Initialize the ctx with a parent Span.
	parentSpan := tracer.StartSpan("parent").(*mocktracer.MockSpan)
	defer parentSpan.Finish()
	ctx := opentracing.ContextWithSpan(context.Background(), parentSpan)

	tracedEndpoint := kitot.TraceEndpoint(tracer, "testOp")(endpoint.Nop)
	if _, err := tracedEndpoint(ctx, struct{}{}); err != nil {
		t.Fatal(err)
	}

	// tracedEndpoint created a new Span.
	finishedSpans := tracer.FinishedSpans()
	if want, have := 1, len(finishedSpans); want != have {
		t.Fatalf("Want %v span(s), found %v", want, have)
	}

	endpointSpan := finishedSpans[0]
	if want, have := "testOp", endpointSpan.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	parentContext := parentSpan.Context().(mocktracer.MockSpanContext)
	endpointContext := parentSpan.Context().(mocktracer.MockSpanContext)

	// ... and that the parent ID is set appropriately.
	if want, have := parentContext.SpanID, endpointContext.SpanID; want != have {
		t.Errorf("Want ParentID %q, have %q", want, have)
	}
}

func TestTraceEndpointNoContextSpan(t *testing.T) {
	tracer := mocktracer.New()

	// Empty/background context.
	tracedEndpoint := kitot.TraceEndpoint(tracer, "testOp")(endpoint.Nop)
	if _, err := tracedEndpoint(context.Background(), struct{}{}); err != nil {
		t.Fatal(err)
	}

	// tracedEndpoint created a new Span.
	finishedSpans := tracer.FinishedSpans()
	if want, have := 1, len(finishedSpans); want != have {
		t.Fatalf("Want %v span(s), found %v", want, have)
	}

	endpointSpan := finishedSpans[0]

	if want, have := "testOp", endpointSpan.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}
}

func TestTraceEndpointWithOptions(t *testing.T) {
	tracer := mocktracer.New()

	// span 1 without options
	mw := kitot.TraceEndpoint(tracer, span1)
	tracedEndpoint := mw(endpoint.Nop)
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 2 with options
	mw = kitot.TraceEndpoint(
		tracer,
		span2,
		kitot.WithOptions(kitot.EndpointOptions{}),
	)
	tracedEndpoint = mw(func(context.Context, interface{}) (interface{}, error) {
		return nil, err1
	})
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 3 with lb error
	mw = kitot.TraceEndpoint(
		tracer,
		span3,
		kitot.WithOptions(kitot.EndpointOptions{}),
	)
	tracedEndpoint = mw(
		lb.Retry(
			5,
			1*time.Second,
			lb.NewRoundRobin(
				sd.FixedEndpointer{
					func(context.Context, interface{}) (interface{}, error) {
						return nil, err1
					},
				},
			),
		),
	)
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 4 with disabled IgnoreBusinessError option
	mw = kitot.TraceEndpoint(
		tracer,
		span4,
		kitot.WithIgnoreBusinessError(false),
	)
	tracedEndpoint = mw(func(context.Context, interface{}) (interface{}, error) {
		return failedResponse{
			err: err2,
		}, nil
	})
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 5 with enabled IgnoreBusinessError option
	mw = kitot.TraceEndpoint(tracer, span5, kitot.WithIgnoreBusinessError(true))
	tracedEndpoint = mw(func(context.Context, interface{}) (interface{}, error) {
		return failedResponse{
			err: err3,
		}, nil
	})
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 6 with OperationNameFunc option
	mw = kitot.TraceEndpoint(
		tracer,
		span6,
		kitot.WithOperationNameFunc(func(ctx context.Context, name string) string {
			return fmt.Sprintf("%s-%s", "new", name)
		}),
	)
	tracedEndpoint = mw(endpoint.Nop)
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 7 with Tags options
	mw = kitot.TraceEndpoint(
		tracer,
		span7,
		kitot.WithTags(map[string]interface{}{
			"tag1": "tag1",
			"tag2": "tag2",
		}),
		kitot.WithTags(map[string]interface{}{
			"tag3": "tag3",
		}),
	)
	tracedEndpoint = mw(endpoint.Nop)
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	// span 8 with TagsFunc options
	mw = kitot.TraceEndpoint(
		tracer,
		span8,
		kitot.WithTags(map[string]interface{}{
			"tag1": "tag1",
			"tag2": "tag2",
		}),
		kitot.WithTags(map[string]interface{}{
			"tag3": "tag3",
		}),
		kitot.WithTagsFunc(func(ctx context.Context) opentracing.Tags {
			return map[string]interface{}{
				"tag4": "tag4",
			}
		}),
	)
	tracedEndpoint = mw(endpoint.Nop)
	_, _ = tracedEndpoint(context.Background(), struct{}{})

	finishedSpans := tracer.FinishedSpans()
	if want, have := 8, len(finishedSpans); want != have {
		t.Fatalf("Want %v span(s), found %v", want, have)
	}

	// test span 1
	span := finishedSpans[0]

	if want, have := span1, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 2
	span = finishedSpans[1]

	if want, have := span2, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := true, span.Tag("error"); want != have {
		t.Fatalf("Want %v, have %v", want, have)
	}

	// test span 3
	span = finishedSpans[2]

	if want, have := span3, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := true, span.Tag("error"); want != have {
		t.Fatalf("Want %v, have %v", want, have)
	}

	if want, have := 1, len(span.Logs()); want != have {
		t.Fatalf("incorrect logs count, wanted %d, got %d", want, have)
	}

	if want, have := []otlog.Field{
		otlog.String("event", "error"),
		otlog.String("error.object", "some error (previously: some error; some error; some error; some error)"),
		otlog.String("gokit.retry.error.1", "some error"),
		otlog.String("gokit.retry.error.2", "some error"),
		otlog.String("gokit.retry.error.3", "some error"),
		otlog.String("gokit.retry.error.4", "some error"),
		otlog.String("gokit.retry.error.5", "some error"),
	}, span.Logs()[0].Fields; reflect.DeepEqual(want, have) {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 4
	span = finishedSpans[3]

	if want, have := span4, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := true, span.Tag("error"); want != have {
		t.Fatalf("Want %v, have %v", want, have)
	}

	if want, have := 2, len(span.Logs()); want != have {
		t.Fatalf("incorrect logs count, wanted %d, got %d", want, have)
	}

	if want, have := []otlog.Field{
		otlog.String("gokit.business.error", "some business error"),
	}, span.Logs()[0].Fields; reflect.DeepEqual(want, have) {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := []otlog.Field{
		otlog.String("event", "error"),
		otlog.String("error.object", "some business error"),
	}, span.Logs()[1].Fields; reflect.DeepEqual(want, have) {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 5
	span = finishedSpans[4]

	if want, have := span5, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := (interface{})(nil), span.Tag("error"); want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := 1, len(span.Logs()); want != have {
		t.Fatalf("incorrect logs count, wanted %d, got %d", want, have)
	}

	if want, have := []otlog.Field{
		otlog.String("gokit.business.error", "some business error"),
	}, span.Logs()[0].Fields; reflect.DeepEqual(want, have) {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 6
	span = finishedSpans[5]

	if want, have := fmt.Sprintf("%s-%s", "new", span6), span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 7
	span = finishedSpans[6]

	if want, have := span7, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := map[string]interface{}{
		"tag1": "tag1",
		"tag2": "tag2",
		"tag3": "tag3",
	}, span.Tags(); fmt.Sprint(want) != fmt.Sprint(have) {
		t.Fatalf("Want %q, have %q", want, have)
	}

	// test span 8
	span = finishedSpans[7]

	if want, have := span8, span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := map[string]interface{}{
		"tag1": "tag1",
		"tag2": "tag2",
		"tag3": "tag3",
		"tag4": "tag4",
	}, span.Tags(); fmt.Sprint(want) != fmt.Sprint(have) {
		t.Fatalf("Want %q, have %q", want, have)
	}
}

func TestTraceServer(t *testing.T) {
	tracer := mocktracer.New()

	// Empty/background context.
	tracedEndpoint := kitot.TraceServer(tracer, "testOp")(endpoint.Nop)
	if _, err := tracedEndpoint(context.Background(), struct{}{}); err != nil {
		t.Fatal(err)
	}

	// tracedEndpoint created a new Span.
	finishedSpans := tracer.FinishedSpans()
	if want, have := 1, len(finishedSpans); want != have {
		t.Fatalf("Want %v span(s), found %v", want, have)
	}

	span := finishedSpans[0]

	if want, have := "testOp", span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := map[string]interface{}{
		otext.SpanKindRPCServer.Key: otext.SpanKindRPCServer.Value,
	}, span.Tags(); fmt.Sprint(want) != fmt.Sprint(have) {
		t.Fatalf("Want %q, have %q", want, have)
	}
}

func TestTraceClient(t *testing.T) {
	tracer := mocktracer.New()

	// Empty/background context.
	tracedEndpoint := kitot.TraceClient(tracer, "testOp")(endpoint.Nop)
	if _, err := tracedEndpoint(context.Background(), struct{}{}); err != nil {
		t.Fatal(err)
	}

	// tracedEndpoint created a new Span.
	finishedSpans := tracer.FinishedSpans()
	if want, have := 1, len(finishedSpans); want != have {
		t.Fatalf("Want %v span(s), found %v", want, have)
	}

	span := finishedSpans[0]

	if want, have := "testOp", span.OperationName; want != have {
		t.Fatalf("Want %q, have %q", want, have)
	}

	if want, have := map[string]interface{}{
		otext.SpanKindRPCClient.Key: otext.SpanKindRPCClient.Value,
	}, span.Tags(); fmt.Sprint(want) != fmt.Sprint(have) {
		t.Fatalf("Want %q, have %q", want, have)
	}
}
