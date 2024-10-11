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
	"runtime"
	"time"
)

// A Option is an option for a transfermanager Downloader or Uploader.
type Option interface {
	apply(*transferManagerConfig)
}

// WithCallbacks returns a TransferManagerOption that allows the use of callbacks
// to process the results. If this option is set, then results will not be returned
// by [Downloader.WaitAndClose] and must be processed through the callback.
func WithCallbacks() Option {
	return &withCallbacks{}
}

type withCallbacks struct{}

func (ww withCallbacks) apply(tm *transferManagerConfig) {
	tm.asynchronous = true
}

// WithWorkers returns a TransferManagerOption that specifies the maximum number
// of concurrent goroutines that will be used to download or upload objects.
// Defaults to runtime.NumCPU()/2.
func WithWorkers(numWorkers int) Option {
	return &withWorkers{numWorkers: numWorkers}
}

type withWorkers struct {
	numWorkers int
}

func (ww withWorkers) apply(tm *transferManagerConfig) {
	tm.numWorkers = ww.numWorkers
}

// WithPerOpTimeout returns a TransferManagerOption that sets a timeout on each
// operation that is performed to download or upload an object. The timeout is
// set when the operation begins processing, not when it is added.
// By default, no timeout is set other than an overall timeout as set on the
// provided context.
func WithPerOpTimeout(timeout time.Duration) Option {
	return &withPerOpTimeout{timeout: timeout}
}

type withPerOpTimeout struct {
	timeout time.Duration
}

func (wpt withPerOpTimeout) apply(tm *transferManagerConfig) {
	tm.perOperationTimeout = wpt.timeout
}

type transferManagerConfig struct {
	// Workers in thread pool; default numCPU/2 based on previous benchmarks?
	numWorkers int

	// Timeout for a single operation (including all retries). Zero value means
	// no timeout.
	perOperationTimeout time.Duration

	// If true, callbacks are used instead of returning results synchronously
	// in a slice at the end.
	asynchronous bool
}

func defaultTransferManagerConfig() *transferManagerConfig {
	return &transferManagerConfig{
		numWorkers:          runtime.NumCPU() / 2,
		perOperationTimeout: 0, // no timeout
	}
}

// initTransferManagerConfig initializes a config with the defaults and applies
// the options passed in.
func initTransferManagerConfig(opts ...Option) *transferManagerConfig {
	config := defaultTransferManagerConfig()
	for _, o := range opts {
		o.apply(config)
	}
	return config
}
