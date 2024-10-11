//go:build go1.21
// +build go1.21

/*
Copyright 2021 The logr Authors.

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
	"context"
	"log/slog"
	"os"
	"testing"
)

func TestContextWithSlog(t *testing.T) {
	ctx := context.Background()

	if out := FromContextAsSlogLogger(ctx); out != nil {
		t.Errorf("expected no logger, got %#v", out)
	}

	// Write as slog...
	slogger := slog.New(slog.NewJSONHandler(os.Stderr, nil))
	sctx := NewContextWithSlogLogger(ctx, slogger)

	// ...read as logr
	if out, err := FromContext(sctx); err != nil {
		t.Errorf("unexpected error: %v", err)
	} else if _, ok := out.sink.(*slogSink); !ok {
		t.Errorf("expected output to be type *logr.slogSink, got %T", out.sink)
	}

	// ...read as slog
	if out := FromContextAsSlogLogger(sctx); out == nil {
		t.Errorf("expected a *slog.JSONHandler, got nil")
	} else if _, ok := out.Handler().(*slog.JSONHandler); !ok {
		t.Errorf("expected output to be type *slog.JSONHandler, got %T", out.Handler())
	}

	// Write as logr...
	logger := Discard()
	lctx := NewContext(ctx, logger)

	// ...read as slog
	if out := FromContextAsSlogLogger(lctx); out == nil {
		t.Errorf("expected a *log.slogHandler, got nil")
	} else if _, ok := out.Handler().(*slogHandler); !ok {
		t.Errorf("expected output to be type *logr.slogHandler, got %T", out.Handler())
	}

	// ...read as logr is covered in the non-slog test
}
