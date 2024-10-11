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
	"testing"
)

func TestContext(t *testing.T) {
	ctx := context.Background()

	if out, err := FromContext(ctx); err == nil {
		t.Errorf("expected error, got %#v", out)
	} else if _, ok := err.(notFoundError); !ok {
		t.Errorf("expected a notFoundError, got %#v", err)
	}

	out := FromContextOrDiscard(ctx)
	if out.sink != nil {
		t.Errorf("expected a nil sink, got %#v", out)
	}

	sink := &testLogSink{}
	logger := New(sink)
	lctx := NewContext(ctx, logger)
	if out, err := FromContext(lctx); err != nil {
		t.Errorf("unexpected error: %v", err)
	} else if p, _ := out.sink.(*testLogSink); p != sink {
		t.Errorf("expected output to be the same as input, got in=%p, out=%p", sink, p)
	}
	out = FromContextOrDiscard(lctx)
	if p, _ := out.sink.(*testLogSink); p != sink {
		t.Errorf("expected output to be the same as input, got in=%p, out=%p", sink, p)
	}
}
