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

// Package testhelp holds helper functions for the testing of logr and built-in
// implementations.
package testhelp

import (
	"bytes"
	"encoding/json"
	"log/slog"
	"strings"
	"testing"
	"testing/slogtest"
)

// RunSlogTests runs slogtest.TestHandler on a given slog.Handler, which is
// expected to emit JSON into the provided buffer.
func RunSlogTests(t *testing.T, createHandler func(buffer *bytes.Buffer) slog.Handler, exceptions ...string) {
	var buffer bytes.Buffer
	handler := createHandler(&buffer)
	err := slogtest.TestHandler(handler, func() []map[string]any {
		var ms []map[string]any
		for _, line := range bytes.Split(buffer.Bytes(), []byte{'\n'}) {
			if len(line) == 0 {
				continue
			}
			var m map[string]any
			if err := json.Unmarshal(line, &m); err != nil {
				t.Errorf("%v: %q", err, string(line))
			}
			ms = append(ms, m)
		}
		return ms
	})

	// Correlating failures with individual test cases is hard with the current API.
	// See https://github.com/golang/go/issues/61758
	t.Logf("Output:\n%s", buffer.String())
	if err != nil {
		if unwrappable, ok := err.(interface {
			Unwrap() []error
		}); ok {
			for _, err := range unwrappable.Unwrap() {
				if !containsOne(err.Error(), exceptions...) {
					t.Errorf("Unexpected error: %v", err)
				}
			}
		} else {
			// Shouldn't be reached, errors from errors.Join can be split up.
			t.Errorf("Unexpected errors:\n%v", err)
		}
	}
}

func containsOne(hay string, needles ...string) bool {
	for _, needle := range needles {
		if strings.Contains(hay, needle) {
			return true
		}
	}
	return false
}
