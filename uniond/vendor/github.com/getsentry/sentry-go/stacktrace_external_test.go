package sentry_test

import (
	"path/filepath"
	"testing"

	goErrors "github.com/go-errors/errors"
	pingcapErrors "github.com/pingcap/errors"
	pkgErrors "github.com/pkg/errors"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"

	"github.com/getsentry/sentry-go"
)

func f1() *sentry.Stacktrace {
	return sentry.NewStacktrace()
}

func f2() *sentry.Stacktrace {
	return f1()
}
func f3() *sentry.Stacktrace {
	return sentry.NewStacktraceForTest()
}

func RedPkgErrorsRanger() error {
	return BluePkgErrorsRanger()
}

func BluePkgErrorsRanger() error {
	return pkgErrors.New("this is bad from pkgErrors")
}

func RedPingcapErrorsRanger() error {
	return BluePingcapErrorsRanger()
}

func BluePingcapErrorsRanger() error {
	return pingcapErrors.New("this is bad from pingcapErrors")
}

func RedGoErrorsRanger() error {
	return BlueGoErrorsRanger()
}

func BlueGoErrorsRanger() error {
	return goErrors.New("this is bad from goErrors")
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestNewStacktrace(t *testing.T) {
	tests := map[string]struct {
		f    func() *sentry.Stacktrace
		want *sentry.Stacktrace
	}{
		"f1": {f1, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "f1",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   18,
					InApp:    true,
				},
			},
		}},
		"f2": {f2, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "f2",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   22,
					InApp:    true,
				},
				{
					Function: "f1",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   18,
					InApp:    true,
				},
			},
		}},
		// test that functions in the SDK that call NewStacktrace are not part
		// of the resulting Stacktrace.
		"NewStacktraceForTest": {sentry.NewStacktraceForTest, &sentry.Stacktrace{
			Frames: []sentry.Frame{},
		}},
		"f3": {f3, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "f3",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   25,
					InApp:    true,
				},
			},
		}},
		"StacktraceTestHelper": {sentry.StacktraceTestHelper{}.NewStacktrace, &sentry.Stacktrace{
			Frames: []sentry.Frame{},
		}},
	}
	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			got := tt.f()
			compareStacktrace(t, got, tt.want)
		})
	}
}

// nolint: scopelint // false positive https://github.com/kyoh86/scopelint/issues/4
func TestExtractStacktrace(t *testing.T) {
	tests := map[string]struct {
		f    func() error
		want *sentry.Stacktrace
	}{
		// https://github.com/pkg/errors
		"pkg/errors": {RedPkgErrorsRanger, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "RedPkgErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   29,
					InApp:    true,
				},
				{
					Function: "BluePkgErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   33,
					InApp:    true,
				},
			},
		}},
		// https://github.com/pingcap/errors
		"pingcap/errors": {RedPingcapErrorsRanger, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "RedPingcapErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   37,
					InApp:    true,
				},
				{
					Function: "BluePingcapErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   41,
					InApp:    true,
				},
			},
		}},
		// https://github.com/go-errors/errors
		"go-errors/errors": {RedGoErrorsRanger, &sentry.Stacktrace{
			Frames: []sentry.Frame{
				{
					Function: "RedGoErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   45,
					InApp:    true,
				},
				{
					Function: "BlueGoErrorsRanger",
					Module:   "github.com/getsentry/sentry-go_test",
					Lineno:   49,
					InApp:    true,
				},
			},
		}},
		// FIXME: The tests below are commented out to avoid introducing a
		// dependency on golang.org/x/xerrors. We should enable them when we
		// move tests with external dependencies to a separate module.
		// See https://github.com/getsentry/sentry-go/issues/238.
		//
		// https://golang.org/x/xerrors
		// "xerrors.errorString": {
		// 	func() error {
		// 		err := xerrors.New("xerror")
		// 		errType := reflect.TypeOf(err).String()
		// 		if errType != "*xerrors.errorString" {
		// 			panic("unexpected error type: " + errType)
		// 		}
		// 		return err
		// 	},
		// 	&sentry.Stacktrace{
		// 		Frames: []sentry.Frame{
		// 			{
		// 				Function: "TestExtractStacktrace.func1",
		// 				Module:   "github.com/getsentry/sentry-go_test",
		// 				Lineno:   178,
		// 				InApp:    true,
		// 			},
		// 		},
		// 	},
		// },
		// "xerrors.wrapError": {
		// 	func() error {
		// 		err := xerrors.Errorf("new error: %w", xerrors.New("xerror"))
		// 		errType := reflect.TypeOf(err).String()
		// 		if errType != "*xerrors.wrapError" {
		// 			panic("unexpected error type: " + errType)
		// 		}
		// 		return err
		// 	},
		// 	&sentry.Stacktrace{
		// 		Frames: []sentry.Frame{
		// 			{
		// 				Function: "TestExtractStacktrace.func2",
		// 				Module:   "github.com/getsentry/sentry-go_test",
		// 				Lineno:   198,
		// 				InApp:    true,
		// 			},
		// 		},
		// 	},
		// },
		// "xerrors.noWrapError": {
		// 	func() error {
		// 		err := xerrors.Errorf("new error: %w", "not an xerror")
		// 		errType := reflect.TypeOf(err).String()
		// 		if errType != "*xerrors.noWrapError" {
		// 			panic("unexpected error type: " + errType)
		// 		}
		// 		return err
		// 	},
		// 	&sentry.Stacktrace{
		// 		Frames: []sentry.Frame{
		// 			{
		// 				Function: "TestExtractStacktrace.func3",
		// 				Module:   "github.com/getsentry/sentry-go_test",
		// 				Lineno:   218,
		// 				InApp:    true,
		// 			},
		// 		},
		// 	},
		// },
	}
	for name, tt := range tests {
		t.Run(name, func(t *testing.T) {
			err := tt.f()
			if err == nil {
				t.Fatal("got nil error")
			}
			got := sentry.ExtractStacktrace(err)
			compareStacktrace(t, got, tt.want)
			// We ignore paths in compareStacktrace because they depend on the
			// environment where tests are run. However, Frame.Filename should
			// be a relative path and Frame.AbsPath should be an absolute path.
			for _, frame := range got.Frames {
				if !filepath.IsAbs(frame.AbsPath) {
					t.Errorf("got %q, want absolute path", frame.AbsPath)
				}
				if filepath.IsAbs(frame.Filename) {
					t.Errorf("got %q, want relative path", frame.Filename)
				}
			}
		})
	}
}

func BenchmarkExtractStacktrace(b *testing.B) {
	var fs = []func() error{
		RedPkgErrorsRanger,
		RedPingcapErrorsRanger,
		RedGoErrorsRanger,
	}

	for i := 0; i < b.N; i++ {
		f := fs[i%len(fs)]
		err := f()

		sentry.ExtractStacktrace(err)
	}
}

func compareStacktrace(t *testing.T, got, want *sentry.Stacktrace) {
	t.Helper()

	if got == nil {
		t.Fatal("got nil stack trace")
	}

	if len(got.Frames) == 0 {
		t.Fatal("got no frames")
	}
	// Skip anonymous function passed to t.Run.
	got.Frames = got.Frames[1:]

	if diff := stacktraceDiff(want, got); diff != "" {
		t.Fatalf("Stacktrace mismatch (-want +got):\n%s", diff)
	}

	// Because stacktraceDiff ignores Frame.AbsPath, sanity check that the
	// values we got are actually absolute paths pointing to this test file.
	for _, frame := range got.Frames {
		if !filepath.IsAbs(frame.AbsPath) {
			t.Errorf("Frame{Function: %q}.AbsPath = %q, want absolute path", frame.Function, frame.AbsPath)
		}
		if filepath.Base(frame.AbsPath) != "stacktrace_external_test.go" {
			t.Errorf(`Frame{Function: %q}.AbsPath = %q, want ".../stacktrace_external_test.go"`, frame.Function, frame.AbsPath)
		}
	}
}

func stacktraceDiff(x, y *sentry.Stacktrace) string {
	return cmp.Diff(
		x, y,
		cmpopts.IgnoreFields(sentry.Frame{}, "AbsPath", "Filename"),
	)
}
