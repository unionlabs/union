// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package trace

import (
	"bytes"
	"context"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
)

func TestSpanContextIsValid(t *testing.T) {
	for _, testcase := range []struct {
		name string
		tid  TraceID
		sid  SpanID
		want bool
	}{
		{
			name: "SpanContext.IsValid() returns true if sc has both an Trace ID and Span ID",
			tid:  [16]byte{1},
			sid:  [8]byte{42},
			want: true,
		}, {
			name: "SpanContext.IsValid() returns false if sc has neither an Trace ID nor Span ID",
			tid:  TraceID([16]byte{}),
			sid:  [8]byte{},
			want: false,
		}, {
			name: "SpanContext.IsValid() returns false if sc has a Span ID but not a Trace ID",
			tid:  TraceID([16]byte{}),
			sid:  [8]byte{42},
			want: false,
		}, {
			name: "SpanContext.IsValid() returns false if sc has a Trace ID but not a Span ID",
			tid:  TraceID([16]byte{1}),
			sid:  [8]byte{},
			want: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			sc := SpanContext{
				traceID: testcase.tid,
				spanID:  testcase.sid,
			}
			have := sc.IsValid()
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestSpanContextEqual(t *testing.T) {
	a := SpanContext{
		traceID: [16]byte{1},
		spanID:  [8]byte{42},
	}

	b := SpanContext{
		traceID: [16]byte{1},
		spanID:  [8]byte{42},
	}

	c := SpanContext{
		traceID: [16]byte{2},
		spanID:  [8]byte{42},
	}

	if !a.Equal(b) {
		t.Error("Want: true, but have: false")
	}

	if a.Equal(c) {
		t.Error("Want: false, but have: true")
	}
}

func TestSpanContextIsSampled(t *testing.T) {
	for _, testcase := range []struct {
		name string
		tf   TraceFlags
		want bool
	}{
		{
			name: "SpanContext.IsSampled() returns false if sc is not sampled",
			want: false,
		}, {
			name: "SpanContext.IsSampled() returns true if sc is sampled",
			tf:   FlagsSampled,
			want: true,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			sc := SpanContext{
				traceFlags: testcase.tf,
			}

			have := sc.IsSampled()

			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestSpanContextIsRemote(t *testing.T) {
	for _, testcase := range []struct {
		name   string
		remote bool
		want   bool
	}{
		{
			name: "SpanContext.IsRemote() returns false if sc is not remote",
			want: false,
		}, {
			name:   "SpanContext.IsRemote() returns true if sc is remote",
			remote: true,
			want:   true,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			sc := SpanContext{
				remote: testcase.remote,
			}

			have := sc.IsRemote()

			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestSpanContextMarshalJSON(t *testing.T) {
	for _, testcase := range []struct {
		name     string
		tid      TraceID
		sid      SpanID
		tstate   TraceState
		tflags   TraceFlags
		isRemote bool
		want     []byte
	}{
		{
			name: "SpanContext.MarshalJSON() returns json with partial data",
			tid:  [16]byte{1},
			sid:  [8]byte{42},
			want: []byte(`{"TraceID":"01000000000000000000000000000000","SpanID":"2a00000000000000","TraceFlags":"00","TraceState":"","Remote":false}`),
		},
		{
			name:     "SpanContext.MarshalJSON() returns json with full data",
			tid:      [16]byte{1},
			sid:      [8]byte{42},
			tflags:   FlagsSampled,
			isRemote: true,
			tstate: TraceState{list: []member{
				{Key: "foo", Value: "1"},
			}},
			want: []byte(`{"TraceID":"01000000000000000000000000000000","SpanID":"2a00000000000000","TraceFlags":"01","TraceState":"foo=1","Remote":true}`),
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			sc := SpanContext{
				traceID:    testcase.tid,
				spanID:     testcase.sid,
				traceFlags: testcase.tflags,
				traceState: testcase.tstate,
				remote:     testcase.isRemote,
			}
			have, err := sc.MarshalJSON()
			if err != nil {
				t.Errorf("Marshaling failed: %v", err)
			}

			if !bytes.Equal(have, testcase.want) {
				t.Errorf("Want: %v, but have: %v", string(testcase.want), string(have))
			}
		})
	}
}

func TestSpanIDFromHex(t *testing.T) {
	for _, testcase := range []struct {
		name  string
		hex   string
		sid   SpanID
		valid bool
	}{
		{
			name:  "Valid SpanID",
			sid:   SpanID([8]byte{42}),
			hex:   "2a00000000000000",
			valid: true,
		}, {
			name:  "Invalid SpanID with invalid length",
			hex:   "80f198ee56343ba",
			valid: false,
		}, {
			name:  "Invalid SpanID with invalid char",
			hex:   "80f198ee563433g7",
			valid: false,
		}, {
			name:  "Invalid SpanID with uppercase",
			hex:   "80f198ee53ba86F7",
			valid: false,
		}, {
			name:  "Invalid SpanID with zero value",
			hex:   "0000000000000000",
			valid: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			sid, err := SpanIDFromHex(testcase.hex)

			if testcase.valid && err != nil {
				t.Errorf("Expected SpanID %s to be valid but end with error %s", testcase.hex, err.Error())
			} else if !testcase.valid && err == nil {
				t.Errorf("Expected SpanID %s to be invalid but end no error", testcase.hex)
			}

			if sid != testcase.sid {
				t.Errorf("Want: %v, but have: %v", testcase.sid, sid)
			}
		})
	}
}

func TestIsValidFromHex(t *testing.T) {
	for _, testcase := range []struct {
		name  string
		hex   string
		tid   TraceID
		valid bool
	}{
		{
			name:  "Valid TraceID",
			tid:   TraceID([16]byte{128, 241, 152, 238, 86, 52, 59, 168, 100, 254, 139, 42, 87, 211, 239, 247}),
			hex:   "80f198ee56343ba864fe8b2a57d3eff7",
			valid: true,
		}, {
			name:  "Invalid TraceID with invalid length",
			hex:   "80f198ee56343ba864fe8b2a57d3eff",
			valid: false,
		}, {
			name:  "Invalid TraceID with invalid char",
			hex:   "80f198ee56343ba864fe8b2a57d3efg7",
			valid: false,
		}, {
			name:  "Invalid TraceID with uppercase",
			hex:   "80f198ee56343ba864fe8b2a57d3efF7",
			valid: false,
		}, {
			name:  "Invalid TraceID with zero value",
			hex:   "00000000000000000000000000000000",
			valid: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			tid, err := TraceIDFromHex(testcase.hex)

			if testcase.valid && err != nil {
				t.Errorf("Expected TraceID %s to be valid but end with error %s", testcase.hex, err.Error())
			}

			if !testcase.valid && err == nil {
				t.Errorf("Expected TraceID %s to be invalid but end no error", testcase.hex)
			}

			if tid != testcase.tid {
				t.Errorf("Want: %v, but have: %v", testcase.tid, tid)
			}
		})
	}
}

func TestSpanContextHasTraceID(t *testing.T) {
	for _, testcase := range []struct {
		name string
		tid  TraceID
		want bool
	}{
		{
			name: "SpanContext.HasTraceID() returns true if both Low and High are nonzero",
			tid:  TraceID([16]byte{1}),
			want: true,
		}, {
			name: "SpanContext.HasTraceID() returns false if neither Low nor High are nonzero",
			tid:  TraceID{},
			want: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// proto: func (sc SpanContext) HasTraceID() bool{}
			sc := SpanContext{traceID: testcase.tid}
			have := sc.HasTraceID()
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestSpanContextHasSpanID(t *testing.T) {
	for _, testcase := range []struct {
		name string
		sc   SpanContext
		want bool
	}{
		{
			name: "SpanContext.HasSpanID() returns true if self.SpanID != 0",
			sc:   SpanContext{spanID: [8]byte{42}},
			want: true,
		}, {
			name: "SpanContext.HasSpanID() returns false if self.SpanID == 0",
			sc:   SpanContext{},
			want: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// proto: func (sc SpanContext) HasSpanID() bool {}
			have := testcase.sc.HasSpanID()
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestTraceFlagsIsSampled(t *testing.T) {
	for _, testcase := range []struct {
		name string
		tf   TraceFlags
		want bool
	}{
		{
			name: "sampled",
			tf:   FlagsSampled,
			want: true,
		}, {
			name: "unused bits are ignored, still not sampled",
			tf:   ^FlagsSampled,
			want: false,
		}, {
			name: "unused bits are ignored, still sampled",
			tf:   FlagsSampled | ^FlagsSampled,
			want: true,
		}, {
			name: "not sampled/default",
			want: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			have := testcase.tf.IsSampled()
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestTraceFlagsWithSampled(t *testing.T) {
	for _, testcase := range []struct {
		name   string
		start  TraceFlags
		sample bool
		want   TraceFlags
	}{
		{
			name:   "sampled unchanged",
			start:  FlagsSampled,
			want:   FlagsSampled,
			sample: true,
		}, {
			name:   "become sampled",
			want:   FlagsSampled,
			sample: true,
		}, {
			name:   "unused bits are ignored, still not sampled",
			start:  ^FlagsSampled,
			want:   ^FlagsSampled,
			sample: false,
		}, {
			name:   "unused bits are ignored, becomes sampled",
			start:  ^FlagsSampled,
			want:   FlagsSampled | ^FlagsSampled,
			sample: true,
		}, {
			name:   "not sampled/default",
			sample: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			have := testcase.start.WithSampled(testcase.sample)
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestStringTraceID(t *testing.T) {
	for _, testcase := range []struct {
		name string
		tid  TraceID
		want string
	}{
		{
			name: "TraceID.String returns string representation of self.TraceID values > 0",
			tid:  TraceID([16]byte{255}),
			want: "ff000000000000000000000000000000",
		},
		{
			name: "TraceID.String returns string representation of self.TraceID values == 0",
			tid:  TraceID([16]byte{}),
			want: "00000000000000000000000000000000",
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// proto: func (t TraceID) String() string {}
			have := testcase.tid.String()
			if have != testcase.want {
				t.Errorf("Want: %s, but have: %s", testcase.want, have)
			}
		})
	}
}

func TestStringSpanID(t *testing.T) {
	for _, testcase := range []struct {
		name string
		sid  SpanID
		want string
	}{
		{
			name: "SpanID.String returns string representation of self.SpanID values > 0",
			sid:  SpanID([8]byte{255}),
			want: "ff00000000000000",
		},
		{
			name: "SpanID.String returns string representation of self.SpanID values == 0",
			sid:  SpanID([8]byte{}),
			want: "0000000000000000",
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// proto: func (t TraceID) String() string {}
			have := testcase.sid.String()
			if have != testcase.want {
				t.Errorf("Want: %s, but have: %s", testcase.want, have)
			}
		})
	}
}

func TestValidateSpanKind(t *testing.T) {
	tests := []struct {
		in   SpanKind
		want SpanKind
	}{
		{
			SpanKindUnspecified,
			SpanKindInternal,
		},
		{
			SpanKindInternal,
			SpanKindInternal,
		},
		{
			SpanKindServer,
			SpanKindServer,
		},
		{
			SpanKindClient,
			SpanKindClient,
		},
		{
			SpanKindProducer,
			SpanKindProducer,
		},
		{
			SpanKindConsumer,
			SpanKindConsumer,
		},
	}
	for _, test := range tests {
		if got := ValidateSpanKind(test.in); got != test.want {
			t.Errorf("ValidateSpanKind(%#v) = %#v, want %#v", test.in, got, test.want)
		}
	}
}

func TestSpanKindString(t *testing.T) {
	tests := []struct {
		in   SpanKind
		want string
	}{
		{
			SpanKindUnspecified,
			"unspecified",
		},
		{
			SpanKindInternal,
			"internal",
		},
		{
			SpanKindServer,
			"server",
		},
		{
			SpanKindClient,
			"client",
		},
		{
			SpanKindProducer,
			"producer",
		},
		{
			SpanKindConsumer,
			"consumer",
		},
	}
	for _, test := range tests {
		if got := test.in.String(); got != test.want {
			t.Errorf("%#v.String() = %#v, want %#v", test.in, got, test.want)
		}
	}
}

func assertSpanContextEqual(got SpanContext, want SpanContext) bool {
	return got.spanID == want.spanID &&
		got.traceID == want.traceID &&
		got.traceFlags == want.traceFlags &&
		got.remote == want.remote &&
		got.traceState.String() == want.traceState.String()
}

func TestNewSpanContext(t *testing.T) {
	testCases := []struct {
		name                string
		config              SpanContextConfig
		expectedSpanContext SpanContext
	}{
		{
			name: "Complete SpanContext",
			config: SpanContextConfig{
				TraceID:    TraceID([16]byte{1}),
				SpanID:     SpanID([8]byte{42}),
				TraceFlags: 0x1,
				TraceState: TraceState{list: []member{
					{"foo", "bar"},
				}},
			},
			expectedSpanContext: SpanContext{
				traceID:    TraceID([16]byte{1}),
				spanID:     SpanID([8]byte{42}),
				traceFlags: 0x1,
				traceState: TraceState{list: []member{
					{"foo", "bar"},
				}},
			},
		},
		{
			name:                "Empty SpanContext",
			config:              SpanContextConfig{},
			expectedSpanContext: SpanContext{},
		},
		{
			name: "Partial SpanContext",
			config: SpanContextConfig{
				TraceID: TraceID([16]byte{1}),
				SpanID:  SpanID([8]byte{42}),
			},
			expectedSpanContext: SpanContext{
				traceID:    TraceID([16]byte{1}),
				spanID:     SpanID([8]byte{42}),
				traceFlags: 0x0,
				traceState: TraceState{},
			},
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			sctx := NewSpanContext(tc.config)
			if !assertSpanContextEqual(sctx, tc.expectedSpanContext) {
				t.Fatalf("%s: Unexpected context created: %s", tc.name, cmp.Diff(sctx, tc.expectedSpanContext))
			}
		})
	}
}

func TestSpanContextDerivation(t *testing.T) {
	from := SpanContext{}
	to := SpanContext{traceID: TraceID([16]byte{1})}

	modified := from.WithTraceID(to.TraceID())
	if !assertSpanContextEqual(modified, to) {
		t.Fatalf("WithTraceID: Unexpected context created: %s", cmp.Diff(modified, to))
	}

	from = to
	to.spanID = SpanID([8]byte{42})

	modified = from.WithSpanID(to.SpanID())
	if !assertSpanContextEqual(modified, to) {
		t.Fatalf("WithSpanID: Unexpected context created: %s", cmp.Diff(modified, to))
	}

	from = to
	to.traceFlags = 0x13

	modified = from.WithTraceFlags(to.TraceFlags())
	if !assertSpanContextEqual(modified, to) {
		t.Fatalf("WithTraceFlags: Unexpected context created: %s", cmp.Diff(modified, to))
	}

	from = to
	to.traceState = TraceState{list: []member{{"foo", "bar"}}}

	modified = from.WithTraceState(to.TraceState())
	if !assertSpanContextEqual(modified, to) {
		t.Fatalf("WithTraceState: Unexpected context created: %s", cmp.Diff(modified, to))
	}
}

func TestLinkFromContext(t *testing.T) {
	k1v1 := attribute.String("key1", "value1")
	spanCtx := SpanContext{traceID: TraceID([16]byte{1}), remote: true}

	receiverCtx := ContextWithRemoteSpanContext(context.Background(), spanCtx)
	link := LinkFromContext(receiverCtx, k1v1)

	if !assertSpanContextEqual(link.SpanContext, spanCtx) {
		t.Fatalf("LinkFromContext: Unexpected context created: %s", cmp.Diff(link.SpanContext, spanCtx))
	}
	assert.Equal(t, link.Attributes[0], k1v1)
}
