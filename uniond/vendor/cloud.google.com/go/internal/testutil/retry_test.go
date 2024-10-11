// Copyright 2019 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package testutil

import (
	"testing"
	"time"
)

func TestRetry(t *testing.T) {
	Retry(t, 5, time.Millisecond, func(r *R) {
		if r.Attempt == 2 {
			return
		}
		r.Fail()
	})
}

func TestRetryAttempts(t *testing.T) {
	var attempts int
	Retry(t, 10, time.Millisecond, func(r *R) {
		r.Logf("This line should appear only once.")
		r.Logf("attempt=%d", r.Attempt)
		attempts = r.Attempt

		// Retry 5 times.
		if r.Attempt == 5 {
			return
		}
		r.Fail()
	})

	if attempts != 5 {
		t.Errorf("attempts=%d; want %d", attempts, 5)
	}
}

func TestRetryWithoutTest(t *testing.T) {
	RetryWithoutTest(5, time.Millisecond, func(r *R) {
		if r.Attempt == 2 {
			return
		}
		r.Fail()
	})
}

func TestRetryWithoutTestAttempts(t *testing.T) {
	var attempts int
	RetryWithoutTest(10, time.Millisecond, func(r *R) {
		r.Logf("This line should appear only once.")
		r.Logf("attempt=%d", r.Attempt)
		attempts = r.Attempt

		// Retry 5 times.
		if r.Attempt == 5 {
			return
		}
		r.Fail()
	})

	if attempts != 5 {
		t.Errorf("attempts=%d; want %d", attempts, 5)
	}
}
