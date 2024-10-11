// Copyright 2024 Google LLC
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

package transfermanager

import (
	"context"
	"strings"
	"testing"
	"time"
)

func TestApply(t *testing.T) {
	opts := []Option{
		WithWorkers(3),
		WithPerOpTimeout(time.Hour),
		WithCallbacks(),
	}
	var got transferManagerConfig
	for _, opt := range opts {
		opt.apply(&got)
	}
	want := transferManagerConfig{
		numWorkers:          3,
		perOperationTimeout: time.Hour,
		asynchronous:        true,
	}

	if got != want {
		t.Errorf("got: %+v, want: %+v", got, want)
	}
}

func TestWithCallbacks(t *testing.T) {
	for _, test := range []struct {
		desc          string
		withCallbacks bool
		callback      func(*DownloadOutput)
		expectedErr   string
	}{
		{
			desc:          "cannot use callbacks without the option",
			withCallbacks: false,
			callback:      func(*DownloadOutput) {},
			expectedErr:   "transfermanager: input.Callback must be nil unless the WithCallbacks option is set",
		},
		{
			desc:          "must provide callback when option is set",
			withCallbacks: true,
			expectedErr:   "transfermanager: input.Callback must not be nil when the WithCallbacks option is set",
		},
	} {
		t.Run(test.desc, func(t *testing.T) {
			var opts []Option
			if test.withCallbacks {
				opts = append(opts, WithCallbacks())
			}
			d, err := NewDownloader(nil, opts...)
			if err != nil {
				t.Fatalf("NewDownloader: %v", err)
			}

			err = d.DownloadObject(context.Background(), &DownloadObjectInput{
				Callback: test.callback,
			})
			if !strings.Contains(err.Error(), test.expectedErr) {
				t.Errorf("expected err %q, got: %v", test.expectedErr, err.Error())
			}
		})
	}
}
