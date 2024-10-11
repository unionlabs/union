package glog

import (
	"context"
	"flag"
	"testing"

	"github.com/golang/glog/internal/logsink"
)

type contextKey string
type fakeLogSink struct {
	context context.Context
}

var ctxKey = contextKey("key")
var ctxValue = "some-value"
var originalSinks = logsink.StructuredSinks

func (s *fakeLogSink) Printf(meta *logsink.Meta, format string, args ...any) (int, error) {
	s.context = meta.Context
	return 0, nil
}

// Test that log.(Info|Error|Warning)Context functions behave the same as non context variants
// and pass right context.
func TestLogContext(t *testing.T) {
	fakeLogSink := &fakeLogSink{}
	logsink.StructuredSinks = append([]logsink.Structured{fakeLogSink}, originalSinks...)

	funcs := map[string]func(ctx context.Context, args ...any){
		"InfoContext":      InfoContext,
		"InfoContextDepth": func(ctx context.Context, args ...any) { InfoContextDepth(ctx, 2, args) },
		"ErrorContext":     ErrorContext,
		"WarningContext":   WarningContext,
	}

	ctx := context.WithValue(context.Background(), ctxKey, ctxValue)
	for name, f := range funcs {
		f(ctx, "test")
		want := ctxValue
		if got := fakeLogSink.context.Value(ctxKey); got != want {
			t.Errorf("%s: context value unexpectedly missing: got %q, want %q", name, got, want)
		}
	}
}

// Test that V.InfoContext behaves the same as V.Info and passes right context.
func TestVInfoContext(t *testing.T) {
	fakeLogSink := &fakeLogSink{}
	logsink.StructuredSinks = append([]logsink.Structured{fakeLogSink}, originalSinks...)
	if err := flag.Lookup("v").Value.Set("2"); err != nil {
		t.Fatalf("Failed to set -v=2: %v", err)
	}
	defer flag.Lookup("v").Value.Set("0")
	ctx := context.WithValue(context.Background(), ctxKey, ctxValue)
	V(2).InfoContext(ctx, "test")
	want := ctxValue
	if got := fakeLogSink.context.Value(ctxKey); got != want {
		t.Errorf("V.InfoContext: context value unexpectedly missing: got %q, want %q", got, want)
	}
}
