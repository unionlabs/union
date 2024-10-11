package sentry

import (
	"bytes"
	"encoding/json"
	"testing"
)

func TestPropagationContextMarshalJSON(t *testing.T) {
	v := NewPropagationContext()
	b, err := json.Marshal(v)
	if err != nil {
		t.Fatal(err)
	}
	if bytes.Contains(b, []byte("parent_span_id")) {
		t.Fatalf("unwanted parent_span_id: %s", b)
	}

	v.ParentSpanID = SpanIDFromHex("b72fa28504b07285")
	b2, err := json.Marshal(v)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Contains(b2, []byte("parent_span_id")) {
		t.Fatalf("missing parent_span_id: %s", b)
	}
}

func TestPropagationContextMap(t *testing.T) {
	p := NewPropagationContext()
	assertEqual(t,
		p.Map(),
		map[string]interface{}{
			"trace_id": p.TraceID,
			"span_id":  p.SpanID,
		},
		"without parent span id")

	p.ParentSpanID = SpanIDFromHex("b72fa28504b07285")
	assertEqual(t,
		p.Map(),
		map[string]interface{}{
			"trace_id":       p.TraceID,
			"span_id":        p.SpanID,
			"parent_span_id": p.ParentSpanID,
		},
		"without praent span id")
}

func TestPropagationContextFromHeaders(t *testing.T) {
	tests := []struct {
		traceStr   string
		baggageStr string
		want       PropagationContext
	}{
		{
			// No sentry-trace or baggage => nothing to do, unfrozen DSC
			traceStr:   "",
			baggageStr: "",
			want: PropagationContext{
				DynamicSamplingContext: DynamicSamplingContext{
					Frozen:  false,
					Entries: nil,
				},
			},
		},
		{
			// Third-party baggage => nothing to do, unfrozen DSC
			traceStr:   "",
			baggageStr: "other-vendor-key1=value1;value2, other-vendor-key2=value3",
			want: PropagationContext{
				DynamicSamplingContext: DynamicSamplingContext{
					Frozen:  false,
					Entries: map[string]string{},
				},
			},
		},
		{
			// sentry-trace and no baggage => we should create a new DSC and freeze it
			// immediately.
			traceStr:   "bc6d53f15eb88f4320054569b8c553d4-b72fa28504b07285-1",
			baggageStr: "",
			want: PropagationContext{
				TraceID:      TraceIDFromHex("bc6d53f15eb88f4320054569b8c553d4"),
				ParentSpanID: SpanIDFromHex("b72fa28504b07285"),
				DynamicSamplingContext: DynamicSamplingContext{
					Frozen: true,
				},
			},
		},
		{
			traceStr:   "bc6d53f15eb88f4320054569b8c553d4-b72fa28504b07285-1",
			baggageStr: "sentry-trace_id=d49d9bf66f13450b81f65bc51cf49c03,sentry-public_key=public,sentry-sample_rate=1",
			want: PropagationContext{
				TraceID:      TraceIDFromHex("bc6d53f15eb88f4320054569b8c553d4"),
				ParentSpanID: SpanIDFromHex("b72fa28504b07285"),
				DynamicSamplingContext: DynamicSamplingContext{
					Frozen: true,
					Entries: map[string]string{
						"public_key":  "public",
						"sample_rate": "1",
						"trace_id":    "d49d9bf66f13450b81f65bc51cf49c03",
					},
				},
			},
		},
	}

	for _, tt := range tests {
		p, err := PropagationContextFromHeaders(tt.traceStr, tt.baggageStr)
		if err != nil {
			t.Fatal(err)
		}

		if tt.want.TraceID != zeroTraceID && p.TraceID != tt.want.TraceID {
			t.Errorf("got TraceID = %s, want %s", p.TraceID, tt.want.TraceID)
		}

		if p.TraceID == zeroTraceID {
			t.Errorf("got TraceID = %s, want non-zero", p.TraceID)
		}

		if p.ParentSpanID != tt.want.ParentSpanID {
			t.Errorf("got ParentSpanID = %s, want %s", p.ParentSpanID, tt.want.ParentSpanID)
		}

		if p.SpanID == zeroSpanID {
			t.Errorf("got SpanID = %s, want non-zero", p.SpanID)
		}

		assertEqual(t, p.DynamicSamplingContext, tt.want.DynamicSamplingContext)
	}
}

func TestNewPropagationContext(t *testing.T) {
	context := NewPropagationContext()

	if context.TraceID == zeroTraceID {
		t.Errorf("TraceID should not be zero")
	}

	if context.SpanID == zeroSpanID {
		t.Errorf("SpanID should not be zero")
	}
}
