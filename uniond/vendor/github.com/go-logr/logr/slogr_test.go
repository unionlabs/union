//go:build go1.21
// +build go1.21

/*
Copyright 2023 The logr Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package logr

import (
	"bytes"
	"fmt"
	"io"
	"log/slog"
	"os"
	"path"
	"runtime"
	"strings"
	"testing"

	"github.com/go-logr/logr/internal/testhelp"
)

func TestToSlogHandler(t *testing.T) {
	t.Run("from simple Logger", func(t *testing.T) {
		logger := New(&testLogSink{})
		handler := ToSlogHandler(logger)
		if _, ok := handler.(*slogHandler); !ok {
			t.Errorf("expected type *slogHandler, got %T", handler)
		}
	})

	t.Run("from slog-enabled Logger", func(t *testing.T) {
		logger := New(&testSlogSink{})
		handler := ToSlogHandler(logger)
		if _, ok := handler.(*slogHandler); !ok {
			t.Errorf("expected type *slogHandler, got %T", handler)
		}
	})

	t.Run("from slogSink Logger", func(t *testing.T) {
		logger := New(&slogSink{handler: slog.NewJSONHandler(os.Stderr, nil)})
		handler := ToSlogHandler(logger)
		if _, ok := handler.(*slog.JSONHandler); !ok {
			t.Errorf("expected type *slog.JSONHandler, got %T", handler)
		}
	})
}

func TestFromSlogHandler(t *testing.T) {
	t.Run("from slog Handler", func(t *testing.T) {
		handler := slog.NewJSONHandler(os.Stderr, nil)
		logger := FromSlogHandler(handler)
		if _, ok := logger.sink.(*slogSink); !ok {
			t.Errorf("expected type *slogSink, got %T", logger.sink)
		}
	})

	t.Run("from simple slogHandler Handler", func(t *testing.T) {
		handler := &slogHandler{sink: &testLogSink{}}
		logger := FromSlogHandler(handler)
		if _, ok := logger.sink.(*testLogSink); !ok {
			t.Errorf("expected type *testSlogSink, got %T", logger.sink)
		}
	})

	t.Run("from discard slogHandler Handler", func(t *testing.T) {
		handler := &slogHandler{}
		logger := FromSlogHandler(handler)
		if logger != Discard() {
			t.Errorf("expected type *testSlogSink, got %T", logger.sink)
		}
	})
}

var debugWithoutTime = &slog.HandlerOptions{
	ReplaceAttr: func(_ []string, a slog.Attr) slog.Attr {
		if a.Key == "time" {
			return slog.Attr{}
		}
		return a
	},
	Level: slog.LevelDebug,
}

func TestWithCallDepth(t *testing.T) {
	debugWithCaller := *debugWithoutTime
	debugWithCaller.AddSource = true
	var buffer bytes.Buffer
	logger := FromSlogHandler(slog.NewTextHandler(&buffer, &debugWithCaller))

	logHelper := func(logger Logger) {
		logger.WithCallDepth(1).Info("hello")
	}

	logHelper(logger)
	_, file, line, _ := runtime.Caller(0)
	expectedSource := fmt.Sprintf("%s:%d", path.Base(file), line-1)
	actual := buffer.String()
	if !strings.Contains(actual, expectedSource) {
		t.Errorf("expected log entry with %s as caller source code location, got instead:\n%s", expectedSource, actual)
	}
}

func TestRunSlogTestsOnSlogHandlerLogSink(t *testing.T) {
	// This proves that slogHandler passes slog's own tests when given a
	// non-SlogSink LogSink.
	exceptions := []string{
		// logr sinks handle time themselves
		"a Handler should ignore a zero Record.Time",
		// slogHandler does not do groups "properly", so these all fail with
		// "missing group".  It's looking for `"G":{"a":"b"}` and getting
		// `"G.a": "b"`.
		"a Handler should handle Group attributes",
		"a Handler should handle the WithGroup method",
		"a Handler should handle multiple WithGroup and WithAttr calls",
		"a Handler should not output groups for an empty Record",
		"a Handler should not output groups if there are no attributes",
		"a Handler should call Resolve on attribute values in groups",
		"a Handler should call Resolve on attribute values in groups from WithAttrs",
	}
	testhelp.RunSlogTests(t, func(buffer *bytes.Buffer) slog.Handler {
		// We want a known-good Logger that emits JSON but is not a slogHandler
		// or SlogSink (since those get special treatment).  We can trust that
		// the slog JSONHandler works.
		handler := slog.NewJSONHandler(buffer, nil)
		sink := &passthruLogSink{handler: handler}
		logger := New(sink)
		return ToSlogHandler(logger)
	}, exceptions...)
}

func TestRunSlogTestsOnSlogHandlerSlogSink(t *testing.T) {
	// This proves that slogHandler passes slog's own tests when given a
	// SlogSink.
	exceptions := []string{}
	testhelp.RunSlogTests(t, func(buffer *bytes.Buffer) slog.Handler {
		// We want a known-good Logger that emits JSON and implements SlogSink,
		// to cover those paths.  We can trust that the slog JSONHandler works.
		handler := slog.NewJSONHandler(buffer, nil)
		sink := &passthruSlogSink{handler: handler}
		logger := New(sink)
		return ToSlogHandler(logger)
	}, exceptions...)
}

func TestSlogSinkOnDiscard(_ *testing.T) {
	// Compile-test
	logger := slog.New(ToSlogHandler(Discard()))
	logger.WithGroup("foo").With("x", 1).Info("hello")
}

func TestConversion(t *testing.T) {
	d := Discard()
	d2 := FromSlogHandler(ToSlogHandler(d))
	expectEqual(t, d, d2)

	e := Logger{}
	e2 := FromSlogHandler(ToSlogHandler(e))
	expectEqual(t, e, e2)

	text := slog.NewTextHandler(io.Discard, nil)
	text2 := ToSlogHandler(FromSlogHandler(text))
	expectEqual(t, text, text2)

	text3 := ToSlogHandler(FromSlogHandler(text).V(1))
	if handler, ok := text3.(interface {
		GetLevel() slog.Level
	}); ok {
		expectEqual(t, handler.GetLevel(), slog.Level(1))
	} else {
		t.Errorf("Expected a slogHandler which implements V(1), got instead: %T %+v", text3, text3)
	}
}

func expectEqual(t *testing.T, expected, actual any) {
	if expected != actual {
		t.Helper()
		t.Errorf("Expected %T %+v, got instead: %T %+v", expected, expected, actual, actual)
	}
}
