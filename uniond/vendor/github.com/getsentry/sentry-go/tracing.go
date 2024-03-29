package sentry

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"net/http"
	"regexp"
	"strings"
	"sync"
	"time"
)

const (
	SentryTraceHeader   = "sentry-trace"
	SentryBaggageHeader = "baggage"
)

// A Span is the building block of a Sentry transaction. Spans build up a tree
// structure of timed operations. The span tree makes up a transaction event
// that is sent to Sentry when the root span is finished.
//
// Spans must be started with either StartSpan or Span.StartChild.
type Span struct { //nolint: maligned // prefer readability over optimal memory layout (see note below *)
	TraceID      TraceID                `json:"trace_id"`
	SpanID       SpanID                 `json:"span_id"`
	ParentSpanID SpanID                 `json:"parent_span_id"`
	Name         string                 `json:"name,omitempty"`
	Op           string                 `json:"op,omitempty"`
	Description  string                 `json:"description,omitempty"`
	Status       SpanStatus             `json:"status,omitempty"`
	Tags         map[string]string      `json:"tags,omitempty"`
	StartTime    time.Time              `json:"start_timestamp"`
	EndTime      time.Time              `json:"timestamp"`
	Data         map[string]interface{} `json:"data,omitempty"`
	Sampled      Sampled                `json:"-"`
	Source       TransactionSource      `json:"-"`

	// mu protects concurrent writes to map fields
	mu sync.RWMutex
	// sample rate the span was sampled with.
	sampleRate float64
	// ctx is the context where the span was started. Always non-nil.
	ctx context.Context
	// Dynamic Sampling context
	dynamicSamplingContext DynamicSamplingContext
	// parent refers to the immediate local parent span. A remote parent span is
	// only referenced by setting ParentSpanID.
	parent *Span
	// recorder stores all spans in a transaction. Guaranteed to be non-nil.
	recorder *spanRecorder
	// span context, can only be set on transactions
	contexts map[string]Context
	// collectProfile is a function that collects a profile of the current transaction. May be nil.
	collectProfile transactionProfiler
	// a Once instance to make sure that Finish() is only called once.
	finishOnce sync.Once
}

// TraceParentContext describes the context of a (remote) parent span.
//
// The context is normally extracted from a received "sentry-trace" header and
// used to initialize a new transaction.
//
// Note: the name might be not the best one. It was taken mostly to stay aligned
// with other SDKs, and it alludes to W3C "traceparent" header (https://www.w3.org/TR/trace-context/),
// which serves a similar purpose to "sentry-trace". We should eventually consider
// making this type internal-only and give it a better name.
type TraceParentContext struct {
	TraceID      TraceID
	ParentSpanID SpanID
	Sampled      Sampled
}

// (*) Note on maligned:
//
// We prefer readability over optimal memory layout. If we ever decide to
// reorder fields, we can use a tool:
//
// go run honnef.co/go/tools/cmd/structlayout -json . Span | go run honnef.co/go/tools/cmd/structlayout-optimize
//
// Other structs would deserve reordering as well, for example Event.

// TODO: make Span.Tags and Span.Data opaque types (struct{unexported []slice}).
// An opaque type allows us to add methods and make it more convenient to use
// than maps, because maps require careful nil checks to use properly or rely on
// explicit initialization for every span, even when there might be no
// tags/data. For Span.Data, must gracefully handle values that cannot be
// marshaled into JSON (see transport.go:getRequestBodyFromEvent).

// StartSpan starts a new span to describe an operation. The new span will be a
// child of the last span stored in ctx, if any.
//
// One or more options can be used to modify the span properties. Typically one
// option as a function literal is enough. Combining multiple options can be
// useful to define and reuse specific properties with named functions.
//
// Caller should call the Finish method on the span to mark its end. Finishing a
// root span sends the span and all of its children, recursively, as a
// transaction to Sentry.
func StartSpan(ctx context.Context, operation string, options ...SpanOption) *Span {
	parent, hasParent := ctx.Value(spanContextKey{}).(*Span)
	var span Span
	span = Span{
		// defaults
		Op:        operation,
		StartTime: time.Now(),
		Sampled:   SampledUndefined,

		ctx:    context.WithValue(ctx, spanContextKey{}, &span),
		parent: parent,
	}

	if hasParent {
		span.TraceID = parent.TraceID
	} else {
		// Only set the Source if this is a transaction
		span.Source = SourceCustom

		// Implementation note:
		//
		// While math/rand is ~2x faster than crypto/rand (exact
		// difference depends on hardware / OS), crypto/rand is probably
		// fast enough and a safer choice.
		//
		// For reference, OpenTelemetry [1] uses crypto/rand to seed
		// math/rand. AFAICT this approach does not preserve the
		// properties from crypto/rand that make it suitable for
		// cryptography. While it might be debatable whether those
		// properties are important for us here, again, we're taking the
		// safer path.
		//
		// See [2a] & [2b] for a discussion of some of the properties we
		// obtain by using crypto/rand and [3a] & [3b] for why we avoid
		// math/rand.
		//
		// Because the math/rand seed has only 64 bits (int64), if the
		// first thing we do after seeding an RNG is to read in a random
		// TraceID, there are only 2^64 possible values. Compared to
		// UUID v4 that have 122 random bits, there is a much greater
		// chance of collision [4a] & [4b].
		//
		// [1]:  https://github.com/open-telemetry/opentelemetry-go/blob/958041ddf619a128/sdk/trace/trace.go#L25-L31
		// [2a]: https://security.stackexchange.com/q/120352/246345
		// [2b]: https://security.stackexchange.com/a/120365/246345
		// [3a]: https://github.com/golang/go/issues/11871#issuecomment-126333686
		// [3b]: https://github.com/golang/go/issues/11871#issuecomment-126357889
		// [4a]: https://en.wikipedia.org/wiki/Universally_unique_identifier#Collisions
		// [4b]: https://www.wolframalpha.com/input/?i=sqrt%282*2%5E64*ln%281%2F%281-0.5%29%29%29
		_, err := rand.Read(span.TraceID[:])
		if err != nil {
			panic(err)
		}
	}
	_, err := rand.Read(span.SpanID[:])
	if err != nil {
		panic(err)
	}
	if hasParent {
		span.ParentSpanID = parent.SpanID
	}

	// Apply options to override defaults.
	for _, option := range options {
		option(&span)
	}

	span.Sampled = span.sample()

	if hasParent {
		span.recorder = parent.spanRecorder()
	} else {
		span.recorder = &spanRecorder{}
	}
	span.recorder.record(&span)

	hub := hubFromContext(ctx)

	// Update scope so that all events include a trace context, allowing
	// Sentry to correlate errors to transactions/spans.
	hub.Scope().SetContext("trace", span.traceContext().Map())

	// Start profiling only if it's a sampled root transaction.
	if span.IsTransaction() && span.Sampled.Bool() {
		span.sampleTransactionProfile()
	}

	return &span
}

// Finish sets the span's end time, unless already set. If the span is the root
// of a span tree, Finish sends the span tree to Sentry as a transaction.
//
// The logic is executed at most once per span, so that (incorrectly) calling it twice
// never double sends to Sentry.
func (s *Span) Finish() {
	s.finishOnce.Do(s.doFinish)
}

// Context returns the context containing the span.
func (s *Span) Context() context.Context { return s.ctx }

// StartChild starts a new child span.
//
// The call span.StartChild(operation, options...) is a shortcut for
// StartSpan(span.Context(), operation, options...).
func (s *Span) StartChild(operation string, options ...SpanOption) *Span {
	return StartSpan(s.Context(), operation, options...)
}

// SetTag sets a tag on the span. It is recommended to use SetTag instead of
// accessing the tags map directly as SetTag takes care of initializing the map
// when necessary.
func (s *Span) SetTag(name, value string) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.Tags == nil {
		s.Tags = make(map[string]string)
	}
	s.Tags[name] = value
}

// SetData sets a data on the span. It is recommended to use SetData instead of
// accessing the data map directly as SetData takes care of initializing the map
// when necessary.
func (s *Span) SetData(name, value string) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.Data == nil {
		s.Data = make(map[string]interface{})
	}
	s.Data[name] = value
}

// SetContext sets a context on the span. It is recommended to use SetContext instead of
// accessing the contexts map directly as SetContext takes care of initializing the map
// when necessary.
func (s *Span) SetContext(key string, value Context) {
	s.mu.Lock()
	defer s.mu.Unlock()

	if s.contexts == nil {
		s.contexts = make(map[string]Context)
	}
	s.contexts[key] = value
}

// IsTransaction checks if the given span is a transaction.
func (s *Span) IsTransaction() bool {
	return s.parent == nil
}

// GetTransaction returns the transaction that contains this span.
//
// For transaction spans it returns itself. For spans that were created manually
// the method returns "nil".
func (s *Span) GetTransaction() *Span {
	spanRecorder := s.spanRecorder()
	if spanRecorder == nil {
		// This probably means that the Span was created manually (not via
		// StartTransaction/StartSpan or StartChild).
		// Return "nil" to indicate that it's not a normal situation.
		return nil
	}
	recorderRoot := spanRecorder.root()
	if recorderRoot == nil {
		// Same as above: manually created Span.
		return nil
	}
	return recorderRoot
}

// TODO(tracing): maybe add shortcuts to get/set transaction name. Right now the
// transaction name is in the Scope, as it has existed there historically, prior
// to tracing.
//
// See Scope.Transaction() and Scope.SetTransaction().
//
// func (s *Span) TransactionName() string
// func (s *Span) SetTransactionName(name string)

// ToSentryTrace returns the seralized TraceParentContext from a transaction/span.
// Use this function to propagate the TraceParentContext to a downstream SDK,
// either as the value of the "sentry-trace" HTTP header, or as an html "sentry-trace" meta tag.
func (s *Span) ToSentryTrace() string {
	// TODO(tracing): add instrumentation for outgoing HTTP requests using
	// ToSentryTrace.
	var b strings.Builder
	fmt.Fprintf(&b, "%s-%s", s.TraceID.Hex(), s.SpanID.Hex())
	switch s.Sampled {
	case SampledTrue:
		b.WriteString("-1")
	case SampledFalse:
		b.WriteString("-0")
	}
	return b.String()
}

// ToBaggage returns the serialized DynamicSamplingContext from a transaction.
// Use this function to propagate the DynamicSamplingContext to a downstream SDK,
// either as the value of the "baggage" HTTP header, or as an html "baggage" meta tag.
func (s *Span) ToBaggage() string {
	if containingTransaction := s.GetTransaction(); containingTransaction != nil {
		// In case there is currently no frozen DynamicSamplingContext attached to the transaction,
		// create one from the properties of the transaction.
		if !s.dynamicSamplingContext.IsFrozen() {
			// This will return a frozen DynamicSamplingContext.
			s.dynamicSamplingContext = DynamicSamplingContextFromTransaction(containingTransaction)
		}

		return containingTransaction.dynamicSamplingContext.String()
	}
	return ""
}

// SetDynamicSamplingContext sets the given dynamic sampling context on the
// current transaction.
func (s *Span) SetDynamicSamplingContext(dsc DynamicSamplingContext) {
	if s.IsTransaction() {
		s.dynamicSamplingContext = dsc
	}
}

// doFinish runs the actual Span.Finish() logic.
func (s *Span) doFinish() {
	if s.EndTime.IsZero() {
		s.EndTime = monotonicTimeSince(s.StartTime)
	}

	if !s.Sampled.Bool() {
		return
	}
	event := s.toEvent()
	if event == nil {
		return
	}

	if s.collectProfile != nil {
		event.sdkMetaData.transactionProfile = s.collectProfile(s)
	}

	// TODO(tracing): add breadcrumbs
	// (see https://github.com/getsentry/sentry-python/blob/f6f3525f8812f609/sentry_sdk/tracing.py#L372)

	hub := hubFromContext(s.ctx)
	hub.CaptureEvent(event)
}

// sentryTracePattern matches either
//
//	TRACE_ID - SPAN_ID
//	[[:xdigit:]]{32}-[[:xdigit:]]{16}
//
// or
//
//	TRACE_ID - SPAN_ID - SAMPLED
//	[[:xdigit:]]{32}-[[:xdigit:]]{16}-[01]
var sentryTracePattern = regexp.MustCompile(`^([[:xdigit:]]{32})-([[:xdigit:]]{16})(?:-([01]))?$`)

// updateFromSentryTrace parses a sentry-trace HTTP header (as returned by
// ToSentryTrace) and updates fields of the span. If the header cannot be
// recognized as valid, the span is left unchanged. The returned value indicates
// whether the span was updated.
func (s *Span) updateFromSentryTrace(header []byte) (updated bool) {
	m := sentryTracePattern.FindSubmatch(header)
	if m == nil {
		// no match
		return false
	}
	_, _ = hex.Decode(s.TraceID[:], m[1])
	_, _ = hex.Decode(s.ParentSpanID[:], m[2])
	if len(m[3]) != 0 {
		switch m[3][0] {
		case '0':
			s.Sampled = SampledFalse
		case '1':
			s.Sampled = SampledTrue
		}
	}
	return true
}

func (s *Span) updateFromBaggage(header []byte) {
	if s.IsTransaction() {
		dsc, err := DynamicSamplingContextFromHeader(header)
		if err != nil {
			return
		}

		s.dynamicSamplingContext = dsc
	}
}

func (s *Span) MarshalJSON() ([]byte, error) {
	// span aliases Span to allow calling json.Marshal without an infinite loop.
	// It preserves all fields while none of the attached methods.
	type span Span
	var parentSpanID string
	if s.ParentSpanID != zeroSpanID {
		parentSpanID = s.ParentSpanID.String()
	}
	return json.Marshal(struct {
		*span
		ParentSpanID string `json:"parent_span_id,omitempty"`
	}{
		span:         (*span)(s),
		ParentSpanID: parentSpanID,
	})
}

func (s *Span) clientOptions() *ClientOptions {
	client := hubFromContext(s.ctx).Client()
	if client != nil {
		return &client.options
	}
	return &ClientOptions{}
}

func (s *Span) sample() Sampled {
	clientOptions := s.clientOptions()
	// https://develop.sentry.dev/sdk/performance/#sampling
	// #1 tracing is not enabled.
	if !clientOptions.EnableTracing {
		Logger.Printf("Dropping transaction: EnableTracing is set to %t", clientOptions.EnableTracing)
		s.sampleRate = 0.0
		return SampledFalse
	}

	// #2 explicit sampling decision via StartSpan/StartTransaction options.
	if s.Sampled != SampledUndefined {
		Logger.Printf("Using explicit sampling decision from StartSpan/StartTransaction: %v", s.Sampled)
		switch s.Sampled {
		case SampledTrue:
			s.sampleRate = 1.0
		case SampledFalse:
			s.sampleRate = 0.0
		}
		return s.Sampled
	}

	// Variant for non-transaction spans: they inherit the parent decision.
	// Note: non-transaction should always have a parent, but we check both
	// conditions anyway -- the first for semantic meaning, the second to
	// avoid a nil pointer dereference.
	if !s.IsTransaction() && s.parent != nil {
		return s.parent.Sampled
	}

	// #3 use TracesSampler from ClientOptions.
	sampler := clientOptions.TracesSampler
	samplingContext := SamplingContext{
		Span:   s,
		Parent: s.parent,
	}

	if sampler != nil {
		tracesSamplerSampleRate := sampler.Sample(samplingContext)
		s.sampleRate = tracesSamplerSampleRate
		if tracesSamplerSampleRate < 0.0 || tracesSamplerSampleRate > 1.0 {
			Logger.Printf("Dropping transaction: Returned TracesSampler rate is out of range [0.0, 1.0]: %f", tracesSamplerSampleRate)
			return SampledFalse
		}
		if tracesSamplerSampleRate == 0 {
			Logger.Printf("Dropping transaction: Returned TracesSampler rate is: %f", tracesSamplerSampleRate)
			return SampledFalse
		}

		if rng.Float64() < tracesSamplerSampleRate {
			return SampledTrue
		}
		Logger.Printf("Dropping transaction: TracesSampler returned rate: %f", tracesSamplerSampleRate)
		return SampledFalse
	}
	// #4 inherit parent decision.
	if s.parent != nil {
		Logger.Printf("Using sampling decision from parent: %v", s.parent.Sampled)
		switch s.parent.Sampled {
		case SampledTrue:
			s.sampleRate = 1.0
		case SampledFalse:
			s.sampleRate = 0.0
		}
		return s.parent.Sampled
	}

	// #5 use TracesSampleRate from ClientOptions.
	sampleRate := clientOptions.TracesSampleRate
	s.sampleRate = sampleRate
	if sampleRate < 0.0 || sampleRate > 1.0 {
		Logger.Printf("Dropping transaction: TracesSamplerRate out of range [0.0, 1.0]: %f", sampleRate)
		return SampledFalse
	}
	if sampleRate == 0.0 {
		Logger.Printf("Dropping transaction: TracesSampleRate rate is: %f", sampleRate)
		return SampledFalse
	}

	if rng.Float64() < sampleRate {
		return SampledTrue
	}

	return SampledFalse
}

func (s *Span) toEvent() *Event {
	s.mu.Lock()
	defer s.mu.Unlock()

	if !s.IsTransaction() {
		return nil // only transactions can be transformed into events
	}

	children := s.recorder.children()
	finished := make([]*Span, 0, len(children))
	for _, child := range children {
		if child.EndTime.IsZero() {
			Logger.Printf("Dropped unfinished span: Op=%q TraceID=%s SpanID=%s", child.Op, child.TraceID, child.SpanID)
			continue
		}
		finished = append(finished, child)
	}

	// Create and attach a DynamicSamplingContext to the transaction.
	// If the DynamicSamplingContext is not frozen at this point, we can assume being head of trace.
	if !s.dynamicSamplingContext.IsFrozen() {
		s.dynamicSamplingContext = DynamicSamplingContextFromTransaction(s)
	}

	contexts := map[string]Context{}
	for k, v := range s.contexts {
		contexts[k] = cloneContext(v)
	}
	contexts["trace"] = s.traceContext().Map()

	// Make sure that the transaction source is valid
	transactionSource := s.Source
	if !transactionSource.isValid() {
		transactionSource = SourceCustom
	}

	return &Event{
		Type:        transactionType,
		Transaction: s.Name,
		Contexts:    contexts,
		Tags:        s.Tags,
		Extra:       s.Data,
		Timestamp:   s.EndTime,
		StartTime:   s.StartTime,
		Spans:       finished,
		TransactionInfo: &TransactionInfo{
			Source: transactionSource,
		},
		sdkMetaData: SDKMetaData{
			dsc: s.dynamicSamplingContext,
		},
	}
}

func (s *Span) traceContext() *TraceContext {
	return &TraceContext{
		TraceID:      s.TraceID,
		SpanID:       s.SpanID,
		ParentSpanID: s.ParentSpanID,
		Op:           s.Op,
		Description:  s.Description,
		Status:       s.Status,
	}
}

// spanRecorder stores the span tree. Guaranteed to be non-nil.
func (s *Span) spanRecorder() *spanRecorder { return s.recorder }

// ParseTraceParentContext parses a sentry-trace header and builds a TraceParentContext from the
// parsed values. If the header was parsed correctly, the second returned argument
// ("valid") will be set to true, otherwise (e.g., empty or malformed header) it will
// be false.
func ParseTraceParentContext(header []byte) (traceParentContext TraceParentContext, valid bool) {
	s := Span{}
	updated := s.updateFromSentryTrace(header)
	if !updated {
		return TraceParentContext{}, false
	}
	return TraceParentContext{
		TraceID:      s.TraceID,
		ParentSpanID: s.ParentSpanID,
		Sampled:      s.Sampled,
	}, true
}

// TraceID identifies a trace.
type TraceID [16]byte

func (id TraceID) Hex() []byte {
	b := make([]byte, hex.EncodedLen(len(id)))
	hex.Encode(b, id[:])
	return b
}

func (id TraceID) String() string {
	return string(id.Hex())
}

func (id TraceID) MarshalText() ([]byte, error) {
	return id.Hex(), nil
}

// SpanID identifies a span.
type SpanID [8]byte

func (id SpanID) Hex() []byte {
	b := make([]byte, hex.EncodedLen(len(id)))
	hex.Encode(b, id[:])
	return b
}

func (id SpanID) String() string {
	return string(id.Hex())
}

func (id SpanID) MarshalText() ([]byte, error) {
	return id.Hex(), nil
}

// Zero values of TraceID and SpanID used for comparisons.
var (
	zeroTraceID TraceID
	zeroSpanID  SpanID
)

// Contains information about how the name of the transaction was determined.
type TransactionSource string

const (
	SourceCustom    TransactionSource = "custom"
	SourceURL       TransactionSource = "url"
	SourceRoute     TransactionSource = "route"
	SourceView      TransactionSource = "view"
	SourceComponent TransactionSource = "component"
	SourceTask      TransactionSource = "task"
)

// A set of all valid transaction sources.
var allTransactionSources = map[TransactionSource]struct{}{
	SourceCustom:    {},
	SourceURL:       {},
	SourceRoute:     {},
	SourceView:      {},
	SourceComponent: {},
	SourceTask:      {},
}

// isValid returns 'true' if the given transaction source is a valid
// source as recognized by the envelope protocol:
// https://develop.sentry.dev/sdk/event-payloads/transaction/#transaction-annotations
func (ts TransactionSource) isValid() bool {
	_, found := allTransactionSources[ts]
	return found
}

// SpanStatus is the status of a span.
type SpanStatus uint8

// Implementation note:
//
// In Relay (ingestion), the SpanStatus type is an enum used as
// Annotated<SpanStatus> when embedded in structs, making it effectively
// Option<SpanStatus>. It means the status is either null or one of the known
// string values.
//
// In Snuba (search), the SpanStatus is stored as an uint8 and defaulted to 2
// ("unknown") when not set. It means that Discover searches for
// `transaction.status:unknown` return both transactions/spans with status
// `null` or `"unknown"`. Searches for `transaction.status:""` return nothing.
//
// With that in mind, the Go SDK default is SpanStatusUndefined, which is
// null/omitted when serializing to JSON, but integrations may update the status
// automatically based on contextual information.

const (
	SpanStatusUndefined SpanStatus = iota
	SpanStatusOK
	SpanStatusCanceled
	SpanStatusUnknown
	SpanStatusInvalidArgument
	SpanStatusDeadlineExceeded
	SpanStatusNotFound
	SpanStatusAlreadyExists
	SpanStatusPermissionDenied
	SpanStatusResourceExhausted
	SpanStatusFailedPrecondition
	SpanStatusAborted
	SpanStatusOutOfRange
	SpanStatusUnimplemented
	SpanStatusInternalError
	SpanStatusUnavailable
	SpanStatusDataLoss
	SpanStatusUnauthenticated
	maxSpanStatus
)

func (ss SpanStatus) String() string {
	if ss >= maxSpanStatus {
		return ""
	}
	m := [maxSpanStatus]string{
		"",
		"ok",
		"cancelled", // [sic]
		"unknown",
		"invalid_argument",
		"deadline_exceeded",
		"not_found",
		"already_exists",
		"permission_denied",
		"resource_exhausted",
		"failed_precondition",
		"aborted",
		"out_of_range",
		"unimplemented",
		"internal_error",
		"unavailable",
		"data_loss",
		"unauthenticated",
	}
	return m[ss]
}

func (ss SpanStatus) MarshalJSON() ([]byte, error) {
	s := ss.String()
	if s == "" {
		return []byte("null"), nil
	}
	return json.Marshal(s)
}

// A TraceContext carries information about an ongoing trace and is meant to be
// stored in Event.Contexts (as *TraceContext).
type TraceContext struct {
	TraceID      TraceID    `json:"trace_id"`
	SpanID       SpanID     `json:"span_id"`
	ParentSpanID SpanID     `json:"parent_span_id"`
	Op           string     `json:"op,omitempty"`
	Description  string     `json:"description,omitempty"`
	Status       SpanStatus `json:"status,omitempty"`
}

func (tc *TraceContext) MarshalJSON() ([]byte, error) {
	// traceContext aliases TraceContext to allow calling json.Marshal without
	// an infinite loop. It preserves all fields while none of the attached
	// methods.
	type traceContext TraceContext
	var parentSpanID string
	if tc.ParentSpanID != zeroSpanID {
		parentSpanID = tc.ParentSpanID.String()
	}
	return json.Marshal(struct {
		*traceContext
		ParentSpanID string `json:"parent_span_id,omitempty"`
	}{
		traceContext: (*traceContext)(tc),
		ParentSpanID: parentSpanID,
	})
}

func (tc TraceContext) Map() map[string]interface{} {
	m := map[string]interface{}{
		"trace_id": tc.TraceID,
		"span_id":  tc.SpanID,
	}

	if tc.ParentSpanID != [8]byte{} {
		m["parent_span_id"] = tc.ParentSpanID
	}

	if tc.Op != "" {
		m["op"] = tc.Op
	}

	if tc.Description != "" {
		m["description"] = tc.Description
	}

	if tc.Status > 0 && tc.Status < maxSpanStatus {
		m["status"] = tc.Status
	}

	return m
}

// Sampled signifies a sampling decision.
type Sampled int8

// The possible trace sampling decisions are: SampledFalse, SampledUndefined
// (default) and SampledTrue.
const (
	SampledFalse     Sampled = -1
	SampledUndefined Sampled = 0
	SampledTrue      Sampled = 1
)

func (s Sampled) String() string {
	switch s {
	case SampledFalse:
		return "SampledFalse"
	case SampledUndefined:
		return "SampledUndefined"
	case SampledTrue:
		return "SampledTrue"
	default:
		return fmt.Sprintf("SampledInvalid(%d)", s)
	}
}

// Bool returns true if the sample decision is SampledTrue, false otherwise.
func (s Sampled) Bool() bool {
	return s == SampledTrue
}

// A SpanOption is a function that can modify the properties of a span.
type SpanOption func(s *Span)

// WithTransactionName option sets the name of the current transaction.
//
// A span tree has a single transaction name, therefore using this option when
// starting a span affects the span tree as a whole, potentially overwriting a
// name set previously.
func WithTransactionName(name string) SpanOption {
	return func(s *Span) {
		s.Name = name
	}
}

// WithDescription sets the description of a span.
func WithDescription(description string) SpanOption {
	return func(s *Span) {
		s.Description = description
	}
}

// WithOpName sets the operation name for a given span.
func WithOpName(name string) SpanOption {
	return func(s *Span) {
		s.Op = name
	}
}

// WithTransactionSource sets the source of the transaction name.
//
// Note: if the transaction source is not a valid source (as described
// by the spec https://develop.sentry.dev/sdk/event-payloads/transaction/#transaction-annotations),
// it will be corrected to "custom" eventually, before the transaction is sent.
func WithTransactionSource(source TransactionSource) SpanOption {
	return func(s *Span) {
		s.Source = source
	}
}

// WithSpanSampled updates the sampling flag for a given span.
func WithSpanSampled(sampled Sampled) SpanOption {
	return func(s *Span) {
		s.Sampled = sampled
	}
}

// ContinueFromRequest returns a span option that updates the span to continue
// an existing trace. If it cannot detect an existing trace in the request, the
// span will be left unchanged.
//
// ContinueFromRequest is an alias for:
//
// ContinueFromHeaders(r.Header.Get(SentryTraceHeader), r.Header.Get(SentryBaggageHeader)).
func ContinueFromRequest(r *http.Request) SpanOption {
	return ContinueFromHeaders(r.Header.Get(SentryTraceHeader), r.Header.Get(SentryBaggageHeader))
}

// ContinueFromHeaders returns a span option that updates the span to continue
// an existing TraceID and propagates the Dynamic Sampling context.
func ContinueFromHeaders(trace, baggage string) SpanOption {
	return func(s *Span) {
		if trace != "" {
			s.updateFromSentryTrace([]byte(trace))
		}
		if baggage != "" {
			s.updateFromBaggage([]byte(baggage))
		}

		// In case a sentry-trace header is present but there are no sentry-related
		// values in the baggage, create an empty, frozen DynamicSamplingContext.
		if trace != "" && !s.dynamicSamplingContext.HasEntries() {
			s.dynamicSamplingContext = DynamicSamplingContext{
				Frozen: true,
			}
		}
	}
}

// ContinueFromTrace returns a span option that updates the span to continue
// an existing TraceID.
func ContinueFromTrace(trace string) SpanOption {
	return func(s *Span) {
		if trace == "" {
			return
		}
		s.updateFromSentryTrace([]byte(trace))
	}
}

// spanContextKey is used to store span values in contexts.
type spanContextKey struct{}

// TransactionFromContext returns the root span of the current transaction. It
// returns nil if no transaction is tracked in the context.
func TransactionFromContext(ctx context.Context) *Span {
	if span, ok := ctx.Value(spanContextKey{}).(*Span); ok {
		return span.recorder.root()
	}
	return nil
}

// SpanFromContext returns the last span stored in the context, or nil if no span
// is set on the context.
func SpanFromContext(ctx context.Context) *Span {
	if span, ok := ctx.Value(spanContextKey{}).(*Span); ok {
		return span
	}
	return nil
}

// StartTransaction will create a transaction (root span) if there's no existing
// transaction in the context otherwise, it will return the existing transaction.
func StartTransaction(ctx context.Context, name string, options ...SpanOption) *Span {
	currentTransaction, exists := ctx.Value(spanContextKey{}).(*Span)
	if exists {
		return currentTransaction
	}

	options = append(options, WithTransactionName(name))
	return StartSpan(
		ctx,
		"",
		options...,
	)
}

// HTTPtoSpanStatus converts an HTTP status code to a SpanStatus.
func HTTPtoSpanStatus(code int) SpanStatus {
	if code < http.StatusBadRequest {
		return SpanStatusOK
	}
	if http.StatusBadRequest <= code && code < http.StatusInternalServerError {
		switch code {
		case http.StatusForbidden:
			return SpanStatusPermissionDenied
		case http.StatusNotFound:
			return SpanStatusNotFound
		case http.StatusTooManyRequests:
			return SpanStatusResourceExhausted
		case http.StatusRequestEntityTooLarge:
			return SpanStatusFailedPrecondition
		case http.StatusUnauthorized:
			return SpanStatusUnauthenticated
		case http.StatusConflict:
			return SpanStatusAlreadyExists
		default:
			return SpanStatusInvalidArgument
		}
	}
	if http.StatusInternalServerError <= code && code < 600 {
		switch code {
		case http.StatusGatewayTimeout:
			return SpanStatusDeadlineExceeded
		case http.StatusNotImplemented:
			return SpanStatusUnimplemented
		case http.StatusServiceUnavailable:
			return SpanStatusUnavailable
		default:
			return SpanStatusInternalError
		}
	}
	return SpanStatusUnknown
}
