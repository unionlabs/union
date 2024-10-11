// Copyright 2023, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package callctx

import (
	"context"
	"sync"
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestAll(t *testing.T) {
	testCases := []struct {
		name  string
		pairs []string
		want  map[string][]string
	}{
		{
			name:  "standard",
			pairs: []string{"key", "value"},
			want:  map[string][]string{"key": {"value"}},
		},
		{
			name:  "multiple values",
			pairs: []string{"key", "value", "key2", "value2"},
			want:  map[string][]string{"key": {"value"}, "key2": {"value2"}},
		},
		{
			name:  "multiple values with same key",
			pairs: []string{"key", "value", "key", "value2"},
			want:  map[string][]string{"key": {"value", "value2"}},
		},
	}
	for _, tc := range testCases {
		ctx := context.Background()
		ctx = SetHeaders(ctx, tc.pairs...)
		got := HeadersFromContext(ctx)
		if diff := cmp.Diff(tc.want, got); diff != "" {
			t.Errorf("HeadersFromContext() mismatch (-want +got):\n%s", diff)
		}
	}
}

func TestSetHeaders_panics(t *testing.T) {
	defer func() {
		if r := recover(); r == nil {
			t.Errorf("expected panic with odd key value pairs")
		}
	}()
	ctx := context.Background()
	SetHeaders(ctx, "1", "2", "3")
}

func TestSetHeaders_reuse(t *testing.T) {
	c := SetHeaders(context.Background(), "key", "value1")
	v1 := HeadersFromContext(c)
	c = SetHeaders(c, "key", "value2")
	v2 := HeadersFromContext(c)

	if cmp.Diff(v2, v1) == "" {
		t.Errorf("Second header set did not differ from first header set as expected")
	}
}

func TestSetHeaders_race(t *testing.T) {
	key := "key"
	value := "value"
	want := map[string][]string{
		key: {value, value},
	}

	// Init the ctx so a value already exists to be "shared".
	cctx := SetHeaders(context.Background(), key, value)

	// Reusing the same cctx and adding to the same header key
	// should *not* produce a race condition when run with -race.
	var wg sync.WaitGroup
	for i := 0; i < 3; i++ {
		wg.Add(1)
		go func(ctx context.Context) {
			defer wg.Done()
			c := SetHeaders(ctx, key, value)
			h := HeadersFromContext(c)

			// Additionally, if there was a race condition,
			// we may see that one instance of these headers
			// contains extra values.
			if diff := cmp.Diff(h, want); diff != "" {
				t.Errorf("got(-),want(+):\n%s", diff)
			}
		}(cctx)
	}
	wg.Wait()
}
