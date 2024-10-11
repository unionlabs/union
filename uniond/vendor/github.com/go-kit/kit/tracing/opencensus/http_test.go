package opencensus_test

import (
	"context"
	"errors"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"go.opencensus.io/plugin/ochttp"
	"go.opencensus.io/plugin/ochttp/propagation/b3"
	"go.opencensus.io/plugin/ochttp/propagation/tracecontext"
	"go.opencensus.io/trace"
	"go.opencensus.io/trace/propagation"

	"github.com/go-kit/kit/endpoint"
	ockit "github.com/go-kit/kit/tracing/opencensus"
	kithttp "github.com/go-kit/kit/transport/http"
)

func TestHTTPClientTrace(t *testing.T) {
	var (
		err     error
		rec     = &recordingExporter{}
		rURL, _ = url.Parse("https://httpbin.org/get")
	)

	trace.RegisterExporter(rec)

	traces := []struct {
		name string
		err  error
	}{
		{"", nil},
		{"CustomName", nil},
		{"", errors.New("dummy-error")},
	}

	for _, tr := range traces {
		clientTracer := ockit.HTTPClientTrace(
			ockit.WithName(tr.name),
			ockit.WithSampler(trace.AlwaysSample()),
		)
		ep := kithttp.NewClient(
			"GET",
			rURL,
			func(ctx context.Context, r *http.Request, i interface{}) error {
				return nil
			},
			func(ctx context.Context, r *http.Response) (response interface{}, err error) {
				return nil, tr.err
			},
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

		if want, have := "GET /get", span.Name; want != have && tr.name == "" {
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

func TestHTTPServerTrace(t *testing.T) {
	rec := &recordingExporter{}

	trace.RegisterExporter(rec)

	traces := []struct {
		useParent   bool
		name        string
		err         error
		propagation propagation.HTTPFormat
	}{
		{false, "", nil, nil},
		{true, "", nil, nil},
		{true, "CustomName", nil, &b3.HTTPFormat{}},
		{true, "", errors.New("dummy-error"), &tracecontext.HTTPFormat{}},
	}

	for _, tr := range traces {
		var client http.Client

		handler := kithttp.NewServer(
			endpoint.Nop,
			func(context.Context, *http.Request) (interface{}, error) { return nil, nil },
			func(context.Context, http.ResponseWriter, interface{}) error { return errors.New("dummy") },
			ockit.HTTPServerTrace(
				ockit.WithName(tr.name),
				ockit.WithSampler(trace.AlwaysSample()),
				ockit.WithHTTPPropagation(tr.propagation),
			),
		)

		server := httptest.NewServer(handler)
		defer server.Close()

		const httpMethod = "GET"

		req, err := http.NewRequest(httpMethod, server.URL, nil)
		if err != nil {
			t.Fatalf("unable to create HTTP request: %s", err.Error())
		}

		if tr.useParent {
			client = http.Client{
				Transport: &ochttp.Transport{
					StartOptions: trace.StartOptions{
						Sampler: trace.AlwaysSample(),
					},
					Propagation: tr.propagation,
				},
			}
		}

		resp, err := client.Do(req.WithContext(context.Background()))
		if err != nil {
			t.Fatalf("unable to send HTTP request: %s", err.Error())
		}
		resp.Body.Close()

		spans := rec.Flush()

		expectedSpans := 1
		if tr.useParent {
			expectedSpans++
		}

		if want, have := expectedSpans, len(spans); want != have {
			t.Fatalf("incorrect number of spans, want %d, have %d", want, have)
		}

		if tr.useParent {
			if want, have := spans[1].TraceID, spans[0].TraceID; want != have {
				t.Errorf("incorrect trace ID, want %s, have %s", want, have)
			}

			if want, have := spans[1].SpanID, spans[0].ParentSpanID; want != have {
				t.Errorf("incorrect span ID, want %s, have %s", want, have)
			}
		}

		if want, have := tr.name, spans[0].Name; want != have && want != "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}

		if want, have := "GET /", spans[0].Name; want != have && tr.name == "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}
	}
}
