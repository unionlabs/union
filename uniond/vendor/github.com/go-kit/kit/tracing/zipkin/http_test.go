package zipkin_test

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"net/url"
	"reflect"
	"testing"

	zipkin "github.com/openzipkin/zipkin-go"
	"github.com/openzipkin/zipkin-go/model"
	"github.com/openzipkin/zipkin-go/propagation/b3"
	"github.com/openzipkin/zipkin-go/reporter/recorder"

	"github.com/go-kit/kit/endpoint"
	zipkinkit "github.com/go-kit/kit/tracing/zipkin"
	kithttp "github.com/go-kit/kit/transport/http"
)

const (
	testName     = "test"
	testBody     = "test_body"
	testTagKey   = "test_key"
	testTagValue = "test_value"
)

func TestHTTPClientTracePropagatesParentSpan(t *testing.T) {
	rec := recorder.NewReporter()
	defer rec.Close()

	tr, _ := zipkin.NewTracer(rec)

	rURL, _ := url.Parse("https://httpbin.org/get")

	clientTracer := zipkinkit.HTTPClientTrace(tr)
	ep := kithttp.NewClient(
		"GET",
		rURL,
		func(ctx context.Context, r *http.Request, i interface{}) error {
			return nil
		},
		func(ctx context.Context, r *http.Response) (response interface{}, err error) {
			return nil, nil
		},
		clientTracer,
	).Endpoint()

	parentSpan := tr.StartSpan("test")

	ctx := zipkin.NewContext(context.Background(), parentSpan)

	_, err := ep(ctx, nil)
	if err != nil {
		t.Fatalf("unexpected error: %s", err.Error())
	}

	spans := rec.Flush()
	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
	}

	span := spans[0]
	if span.SpanContext.ParentID == nil {
		t.Fatalf("incorrect parent ID, want %s have nil", parentSpan.Context().ID)
	}

	if want, have := parentSpan.Context().ID, *span.SpanContext.ParentID; want != have {
		t.Fatalf("incorrect parent ID, want %s, have %s", want, have)
	}
}

func TestHTTPClientTraceAddsExpectedTags(t *testing.T) {
	dataProvider := []struct {
		ResponseStatusCode int
		ErrorTagValue      string
	}{
		{http.StatusOK, ""},
		{http.StatusForbidden, fmt.Sprint(http.StatusForbidden)},
	}

	for _, data := range dataProvider {
		testHTTPClientTraceCase(t, data.ResponseStatusCode, data.ErrorTagValue)
	}
}

func testHTTPClientTraceCase(t *testing.T, responseStatusCode int, errTagValue string) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(responseStatusCode)
		w.Write([]byte(testBody))
	}))
	defer ts.Close()

	rec := recorder.NewReporter()
	defer rec.Close()

	tr, err := zipkin.NewTracer(rec)
	if err != nil {
		t.Errorf("Unwanted error: %s", err.Error())
	}

	rMethod := "GET"
	rURL, _ := url.Parse(ts.URL)

	clientTracer := zipkinkit.HTTPClientTrace(
		tr,
		zipkinkit.Name(testName),
		zipkinkit.Tags(map[string]string{testTagKey: testTagValue}),
	)

	ep := kithttp.NewClient(
		rMethod,
		rURL,
		func(ctx context.Context, r *http.Request, i interface{}) error {
			return nil
		},
		func(ctx context.Context, r *http.Response) (response interface{}, err error) {
			return nil, nil
		},
		clientTracer,
	).Endpoint()

	_, err = ep(context.Background(), nil)
	if err != nil {
		t.Fatalf("unwanted error: %s", err.Error())
	}

	spans := rec.Flush()
	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, wanted %d, got %d", want, have)
	}

	span := spans[0]
	if span.SpanContext.ParentID != nil {
		t.Fatalf("incorrect parentID, wanted nil, got %s", span.SpanContext.ParentID)
	}

	if want, have := testName, span.Name; want != have {
		t.Fatalf("incorrect span name, wanted %s, got %s", want, have)
	}

	if want, have := model.Client, span.Kind; want != have {
		t.Fatalf("incorrect span kind, wanted %s, got %s", want, have)
	}

	tags := map[string]string{
		testTagKey:                         testTagValue,
		string(zipkin.TagHTTPStatusCode):   fmt.Sprint(responseStatusCode),
		string(zipkin.TagHTTPMethod):       rMethod,
		string(zipkin.TagHTTPUrl):          rURL.String(),
		string(zipkin.TagHTTPResponseSize): fmt.Sprint(len(testBody)),
	}

	if errTagValue != "" {
		tags[string(zipkin.TagError)] = fmt.Sprint(errTagValue)
	}

	if !reflect.DeepEqual(span.Tags, tags) {
		t.Fatalf("invalid tags set, wanted %+v, got %+v", tags, span.Tags)
	}
}

func TestHTTPServerTrace(t *testing.T) {
	rec := recorder.NewReporter()
	defer rec.Close()

	// explicitly show we use the default of RPC shared spans in Zipkin as it
	// is idiomatic for Zipkin to share span identifiers between client and
	// server side.
	tr, _ := zipkin.NewTracer(rec, zipkin.WithSharedSpans(true))

	handler := kithttp.NewServer(
		endpoint.Nop,
		func(context.Context, *http.Request) (interface{}, error) { return nil, nil },
		func(context.Context, http.ResponseWriter, interface{}) error { return errors.New("dummy") },
		zipkinkit.HTTPServerTrace(tr),
	)

	server := httptest.NewServer(handler)
	defer server.Close()

	const httpMethod = "GET"

	req, err := http.NewRequest(httpMethod, server.URL, nil)
	if err != nil {
		t.Fatalf("unable to create HTTP request: %s", err.Error())
	}

	parentSpan := tr.StartSpan("Dummy")

	b3.InjectHTTP(req)(parentSpan.Context())

	client := http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		t.Fatalf("unable to send HTTP request: %s", err.Error())
	}
	resp.Body.Close()

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

	if want, have := httpMethod, spans[0].Name; want != have {
		t.Errorf("incorrect span name, want %s, have %s", want, have)
	}

	if want, have := http.StatusText(500), spans[0].Tags["error"]; want != have {
		t.Fatalf("incorrect error tag, want %s, have %s", want, have)
	}
}

func TestHTTPServerTraceIsRequestBasedSampled(t *testing.T) {
	rec := recorder.NewReporter()
	defer rec.Close()

	const httpMethod = "DELETE"

	tr, _ := zipkin.NewTracer(rec)

	handler := kithttp.NewServer(
		endpoint.Nop,
		func(context.Context, *http.Request) (interface{}, error) { return nil, nil },
		func(context.Context, http.ResponseWriter, interface{}) error { return nil },
		zipkinkit.HTTPServerTrace(tr, zipkinkit.RequestSampler(func(r *http.Request) bool {
			return r.Method == httpMethod
		})),
	)

	server := httptest.NewServer(handler)
	defer server.Close()

	req, err := http.NewRequest(httpMethod, server.URL, nil)
	if err != nil {
		t.Fatalf("unable to create HTTP request: %s", err.Error())
	}

	client := http.Client{}
	resp, err := client.Do(req)
	if err != nil {
		t.Fatalf("unable to send HTTP request: %s", err.Error())
	}
	resp.Body.Close()

	spans := rec.Flush()
	if want, have := 1, len(spans); want != have {
		t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
	}
}
