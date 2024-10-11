package sentry

import (
	"encoding/json"
	"errors"
	"runtime"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func NewStacktraceForTest() *Stacktrace {
	return NewStacktrace()
}

type StacktraceTestHelper struct{}

func (StacktraceTestHelper) NewStacktrace() *Stacktrace {
	return NewStacktrace()
}

func BenchmarkNewStacktrace(b *testing.B) {
	for i := 0; i < b.N; i++ {
		Trace()
	}
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestSplitQualifiedFunctionName(t *testing.T) {
	tests := []struct {
		in  string
		pkg string
		fun string
	}{
		{"", "", ""},
		{"runtime.Callers", "runtime", "Callers"},
		{"main.main.func1", "main", "main.func1"},
		{
			"github.com/getsentry/sentry-go.Init",
			"github.com/getsentry/sentry-go",
			"Init",
		},
		{
			"github.com/getsentry/sentry-go.(*Hub).Flush",
			"github.com/getsentry/sentry-go",
			"(*Hub).Flush",
		},
		{
			"github.com/getsentry/sentry-go.Test.func2.1.1",
			"github.com/getsentry/sentry-go",
			"Test.func2.1.1",
		},
		{
			"github.com/getsentry/confusing%2epkg%2ewith%2edots.Test.func1",
			"github.com/getsentry/confusing%2epkg%2ewith%2edots",
			"Test.func1",
		},
	}
	for _, tt := range tests {
		t.Run(tt.in, func(t *testing.T) {
			pkg, fun := splitQualifiedFunctionName(tt.in)
			if diff := cmp.Diff(tt.pkg, pkg); diff != "" {
				t.Errorf("Package name mismatch (-want +got):\n%s", diff)
			}
			if diff := cmp.Diff(tt.fun, fun); diff != "" {
				t.Errorf("Function name mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestCreateFrames(t *testing.T) {
	tests := []struct {
		in  []runtime.Frame
		out []Frame
	}{
		// sanity check
		{},
		// filter out go internals and SDK internals; "sentry-go_test" is
		// considered outside of the SDK and thus included (useful for testing)
		{
			in: []runtime.Frame{
				{
					Function: "runtime.goexit",
					File:     "/goroot/src/runtime/asm_amd64.s",
				},
				{
					Function: "testing.tRunner",
					File:     "/goroot/src/testing/testing.go",
				},
				{
					Function: "github.com/getsentry/sentry-go_test.TestNewStacktrace.func1",
					File:     "/somewhere/sentry/sentry-go/stacktrace_external_test.go",
				},
				{
					Function: "github.com/getsentry/sentry-go.StacktraceTestHelper.NewStacktrace",
					File:     "/somewhere/sentry/sentry-go/stacktrace_test.go",
				},
				{
					Function: "github.com/getsentry/sentry-go.NewStacktrace",
					File:     "/somewhere/sentry/sentry-go/stacktrace.go",
				},
			},
			out: []Frame{
				{
					Function: "TestNewStacktrace.func1",
					Module:   "github.com/getsentry/sentry-go_test",
					AbsPath:  "/somewhere/sentry/sentry-go/stacktrace_external_test.go",
					InApp:    true,
				},
			},
		},
		// filter out integrations; SDK subpackages
		{
			in: []runtime.Frame{
				{
					Function: "github.com/getsentry/sentry-go/http/integration.Example.Integration",
					File:     "/somewhere/sentry/sentry-go/http/integration/integration.go",
				},
				{
					Function: "github.com/getsentry/sentry-go/http.(*Handler).Handle",
					File:     "/somewhere/sentry/sentry-go/http/sentryhttp.go",
				},
				{
					Function: "main.main",
					File:     "/somewhere/example.com/pkg/main.go",
				},
			},
			out: []Frame{
				{
					Function: "main",
					Module:   "main",
					AbsPath:  "/somewhere/example.com/pkg/main.go",
					InApp:    true,
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run("", func(t *testing.T) {
			got := createFrames(tt.in)
			if diff := cmp.Diff(tt.out, got); diff != "" {
				t.Errorf("filterFrames() mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestExtractXErrorsPC(t *testing.T) {
	// This ensures that extractXErrorsPC does not break code that doesn't use
	// golang.org/x/xerrors. For tests that check that it works on the
	// appropriate type of errors, see stacktrace_external_test.go.
	if got := extractXErrorsPC(errors.New("test")); got != nil {
		t.Errorf("got %#v, want nil", got)
	}
}

func TestEventWithExceptionStacktraceMarshalJSON(t *testing.T) {
	event := NewEvent()
	event.Exception = []Exception{
		{
			Stacktrace: &Stacktrace{
				Frames: []Frame{
					{
						Function:    "gofunc",
						Symbol:      "gosym",
						Module:      "gopkg/gopath",
						Filename:    "foo.go",
						AbsPath:     "/something/foo.go",
						Lineno:      35,
						Colno:       72,
						PreContext:  []string{"pre", "context"},
						ContextLine: "contextline",
						PostContext: []string{"post", "context"},
						InApp:       true,
						Vars: map[string]interface{}{
							"foostr": "bar",
							"fooint": 25,
						},
					},
					{
						Symbol:          "nativesym",
						Package:         "my.dylib",
						InstructionAddr: "0xabcd0010",
						AddrMode:        "abs",
						SymbolAddr:      "0xabcd0000",
						ImageAddr:       "0xabc00000",
						Platform:        "native",
						StackStart:      false,
					},
				},
			},
		},
	}

	got, err := json.Marshal(event)
	if err != nil {
		t.Fatal(err)
	}

	want := `{"sdk":{},"user":{},` +
		`"exception":[{"stacktrace":{"frames":[` +
		`{"function":"gofunc",` +
		`"symbol":"gosym",` +
		`"module":"gopkg/gopath",` +
		`"filename":"foo.go",` +
		`"abs_path":"/something/foo.go",` +
		`"lineno":35,` +
		`"colno":72,` +
		`"pre_context":["pre","context"],` +
		`"context_line":"contextline",` +
		`"post_context":["post","context"],` +
		`"in_app":true,` +
		`"vars":{"fooint":25,"foostr":"bar"}` +
		`},{` +
		`"symbol":"nativesym",` +
		`"in_app":false,` +
		`"package":"my.dylib",` +
		`"instruction_addr":"0xabcd0010",` +
		`"addr_mode":"abs",` +
		`"symbol_addr":"0xabcd0000",` +
		`"image_addr":"0xabc00000",` +
		`"platform":"native"` +
		`}]}}]}`

	if diff := cmp.Diff(want, string(got)); diff != "" {
		t.Errorf("Event mismatch (-want +got):\n%s", diff)
	}
}
