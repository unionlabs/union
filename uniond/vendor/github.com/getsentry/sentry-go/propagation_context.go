package sentry

import (
	"crypto/rand"
	"encoding/json"
)

type PropagationContext struct {
	TraceID                TraceID                `json:"trace_id"`
	SpanID                 SpanID                 `json:"span_id"`
	ParentSpanID           SpanID                 `json:"parent_span_id"`
	DynamicSamplingContext DynamicSamplingContext `json:"-"`
}

func (p PropagationContext) MarshalJSON() ([]byte, error) {
	type propagationContext PropagationContext
	var parentSpanID string
	if p.ParentSpanID != zeroSpanID {
		parentSpanID = p.ParentSpanID.String()
	}
	return json.Marshal(struct {
		*propagationContext
		ParentSpanID string `json:"parent_span_id,omitempty"`
	}{
		propagationContext: (*propagationContext)(&p),
		ParentSpanID:       parentSpanID,
	})
}

func (p PropagationContext) Map() map[string]interface{} {
	m := map[string]interface{}{
		"trace_id": p.TraceID,
		"span_id":  p.SpanID,
	}

	if p.ParentSpanID != zeroSpanID {
		m["parent_span_id"] = p.ParentSpanID
	}

	return m
}

func NewPropagationContext() PropagationContext {
	p := PropagationContext{}

	if _, err := rand.Read(p.TraceID[:]); err != nil {
		panic(err)
	}

	if _, err := rand.Read(p.SpanID[:]); err != nil {
		panic(err)
	}

	return p
}

func PropagationContextFromHeaders(trace, baggage string) (PropagationContext, error) {
	p := NewPropagationContext()

	if _, err := rand.Read(p.SpanID[:]); err != nil {
		panic(err)
	}

	hasTrace := false
	if trace != "" {
		if tpc, valid := ParseTraceParentContext([]byte(trace)); valid {
			hasTrace = true
			p.TraceID = tpc.TraceID
			p.ParentSpanID = tpc.ParentSpanID
		}
	}

	if baggage != "" {
		dsc, err := DynamicSamplingContextFromHeader([]byte(baggage))
		if err != nil {
			return PropagationContext{}, err
		}
		p.DynamicSamplingContext = dsc
	}

	// In case a sentry-trace header is present but there are no sentry-related
	// values in the baggage, create an empty, frozen DynamicSamplingContext.
	if hasTrace && !p.DynamicSamplingContext.HasEntries() {
		p.DynamicSamplingContext = DynamicSamplingContext{
			Frozen: true,
		}
	}

	return p, nil
}
