package sentry

// A SamplingContext is passed to a TracesSampler to determine a sampling
// decision.
//
// TODO(tracing): possibly expand SamplingContext to include custom /
// user-provided data.
type SamplingContext struct {
	Span   *Span // The current span, always non-nil.
	Parent *Span // The parent span, may be nil.
}

// The TracesSample type is an adapter to allow the use of ordinary
// functions as a TracesSampler.
type TracesSampler func(ctx SamplingContext) float64

func (f TracesSampler) Sample(ctx SamplingContext) float64 {
	return f(ctx)
}
