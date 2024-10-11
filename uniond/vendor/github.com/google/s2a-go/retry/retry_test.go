/*
 *
 * Copyright 2023 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package retry

import (
	"context"
	"errors"
	"testing"
	"time"
)

var errTest error = errors.New("test error")

type constantBackoff struct{}

func (b constantBackoff) Pause() time.Duration { return 100 }

func TestS2ARetryer(t *testing.T) {
	tests := []struct {
		name            string
		err             error
		wantDelay       time.Duration
		wantShouldRetry bool
	}{
		{
			name:            "retry on err",
			err:             errTest,
			wantDelay:       100,
			wantShouldRetry: true,
		},
		{
			name:            "don't retry if err is nil",
			err:             nil,
			wantDelay:       0,
			wantShouldRetry: false,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			retryer := S2ARetryer{bo: constantBackoff{}}
			delay, shouldRetry := retryer.Retry(tc.err)
			if delay != tc.wantDelay {
				t.Fatalf("retryer.Retry(%v) = %v, want %v", tc.err, delay, tc.wantDelay)
			}
			if shouldRetry != tc.wantShouldRetry {
				t.Fatalf("retryer.Retry(%v) = %v, want %v", tc.err, shouldRetry, tc.wantShouldRetry)
			}
		})
	}
}

func TestS2ARetryerAttempts(t *testing.T) {
	retryer := S2ARetryer{bo: constantBackoff{}}
	for i := 1; i <= 5; i++ {
		_, shouldRetry := retryer.Retry(errTest)
		if !shouldRetry {
			t.Fatalf("retryer.Retry(errTest) = false, want true")
		}
	}
	_, shouldRetry := retryer.Retry(errTest)
	if shouldRetry {
		t.Fatal("an error should only be retried 5 times")
	}
}

func TestDefaultBackoff(t *testing.T) {
	bo := defaultBackoff{
		cur: 100 * time.Millisecond,
		max: 30 * time.Second,
		mul: 2,
	}
	pauseOne := bo.Pause()
	pauseTwo := bo.Pause()
	if pauseOne > 100*time.Millisecond {
		t.Fatal("first backoff should be less than 100 milli seconds")
	}
	if pauseTwo > 2*100*time.Millisecond {
		t.Fatal("second backoff should be less than 200 milli seconds")
	}
}

func TestSuccessAfterRetry(t *testing.T) {
	oldRetry := NewRetryer
	defer func() { NewRetryer = oldRetry }()
	testRetryer := NewRetryer()
	NewRetryer = func() *S2ARetryer {
		return testRetryer
	}

	cnt := 1
	f := func() error {
		if cnt == 1 {
			cnt++
			return errTest
		}
		return nil
	}
	if testRetryer.Attempts() != 0 {
		t.Fatal("before execution, retry attempt count should be 0")
	}

	Run(context.Background(), f)

	if testRetryer.Attempts() != 1 {
		t.Fatal("execution should've succeeded after 1 retry")
	}
}
