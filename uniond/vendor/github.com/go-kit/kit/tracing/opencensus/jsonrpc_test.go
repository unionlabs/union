package opencensus_test

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
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
	jsonrpc "github.com/go-kit/kit/transport/http/jsonrpc"
)

func TestJSONRPCClientTrace(t *testing.T) {
	t.Skip("FLAKY")

	var (
		err          error
		rec          = &recordingExporter{}
		rURL, _      = url.Parse("https://httpbin.org/anything")
		endpointName = "DummyEndpoint"
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
		clientTracer := ockit.JSONRPCClientTrace(
			ockit.WithName(tr.name),
			ockit.WithSampler(trace.AlwaysSample()),
		)
		ep := jsonrpc.NewClient(
			rURL,
			endpointName,
			jsonrpc.ClientRequestEncoder(func(ctx context.Context, i interface{}) (json.RawMessage, error) {
				return json.RawMessage(`{}`), nil
			}),
			jsonrpc.ClientResponseDecoder(func(ctx context.Context, r jsonrpc.Response) (response interface{}, err error) {
				return nil, tr.err
			}),
			clientTracer,
		).Endpoint()

		ctx, parentSpan := trace.StartSpan(context.Background(), "test")

		_, err = ep(ctx, nil)
		if want, have := tr.err, err; want != have {
			t.Fatalf("unexpected error, want %v, have %v", tr.err, err)
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

		if want, have := endpointName, span.Name; want != have && tr.name == "" {
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

func TestJSONRPCServerTrace(t *testing.T) {
	var (
		endpointName = "DummyEndpoint"
		rec          = &recordingExporter{}
	)

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

		handler := jsonrpc.NewServer(
			jsonrpc.EndpointCodecMap{
				endpointName: jsonrpc.EndpointCodec{
					Endpoint: endpoint.Nop,
					Decode:   func(context.Context, json.RawMessage) (interface{}, error) { return nil, nil },
					Encode:   func(context.Context, interface{}) (json.RawMessage, error) { return nil, tr.err },
				},
			},
			ockit.JSONRPCServerTrace(
				ockit.WithName(tr.name),
				ockit.WithSampler(trace.AlwaysSample()),
				ockit.WithHTTPPropagation(tr.propagation),
			),
		)

		server := httptest.NewServer(handler)
		defer server.Close()

		jsonStr := []byte(fmt.Sprintf(`{"method":"%s"}`, endpointName))
		req, err := http.NewRequest("POST", server.URL, bytes.NewBuffer(jsonStr))
		if err != nil {
			t.Fatalf("unable to create JSONRPC request: %v", err)
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
			t.Fatalf("unable to send JSONRPC request: %v", err)
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

		if want, have := endpointName, spans[0].Name; want != have && tr.name == "" {
			t.Errorf("incorrect span name, want %s, have %s", want, have)
		}
	}
}
