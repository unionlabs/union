// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package metadata

import (
	"io"
	"testing"
	"time"
)

type constantBackoff struct{}

func (b constantBackoff) Pause() time.Duration { return 100 }

type errTemp struct{}

func (e errTemp) Error() string { return "temporary error" }

func (e errTemp) Temporary() bool { return true }

type errWrapped struct {
	e error
}

func (e errWrapped) Error() string { return "unwrap me to get more context" }

func (e errWrapped) Unwrap() error { return e.e }

func TestMetadataRetryer(t *testing.T) {
	tests := []struct {
		name            string
		code            int
		err             error
		wantDelay       time.Duration
		wantShouldRetry bool
	}{
		{
			name:            "retry on 500",
			code:            500,
			wantDelay:       100,
			wantShouldRetry: true,
		},
		{
			name:            "don't retry on 400",
			code:            400,
			err:             io.EOF,
			wantDelay:       0,
			wantShouldRetry: false,
		},
		{
			name:            "retry on io.ErrUnexpectedEOF",
			code:            400,
			err:             io.ErrUnexpectedEOF,
			wantDelay:       100,
			wantShouldRetry: true,
		},
		{
			name:            "retry on temporary error",
			code:            400,
			err:             errTemp{},
			wantDelay:       100,
			wantShouldRetry: true,
		},
		{
			name:            "retry on wrapped temporary error",
			code:            400,
			err:             errWrapped{errTemp{}},
			wantDelay:       100,
			wantShouldRetry: true,
		},
		{
			name:            "don't retry on wrapped io.EOF",
			code:            400,
			err:             errWrapped{io.EOF},
			wantDelay:       0,
			wantShouldRetry: false,
		},
		{
			name:            "don't retry 200",
			code:            200,
			err:             nil,
			wantDelay:       0,
			wantShouldRetry: false,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			retryer := metadataRetryer{bo: constantBackoff{}}
			delay, shouldRetry := retryer.Retry(tc.code, tc.err)
			if delay != tc.wantDelay {
				t.Fatalf("retryer.Retry(%v, %v) = %v, want %v", tc.code, tc.err, delay, tc.wantDelay)
			}
			if shouldRetry != tc.wantShouldRetry {
				t.Fatalf("retryer.Retry(%v, %v) = %v, want %v", tc.code, tc.err, shouldRetry, tc.wantShouldRetry)
			}
		})
	}
}

func TestMetadataRetryerAttempts(t *testing.T) {
	retryer := metadataRetryer{bo: constantBackoff{}}
	for i := 1; i <= 6; i++ {
		_, shouldRetry := retryer.Retry(500, nil)
		if i == 6 {
			if shouldRetry {
				t.Fatal("an error should only be retried 5 times")
			}
			break
		}
		if !shouldRetry {
			t.Fatalf("retryer.Retry(500, nil) = false, want true")
		}
	}
}
