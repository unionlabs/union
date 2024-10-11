// Copyright 2024 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package promhttp

import (
	"net/http"
	"testing"
	"time"
)

type responseWriter struct {
	flushErrorCalled       bool
	setWriteDeadlineCalled time.Time
	setReadDeadlineCalled  time.Time
}

func (rw *responseWriter) Header() http.Header {
	return nil
}

func (rw *responseWriter) Write(p []byte) (int, error) {
	return 0, nil
}

func (rw *responseWriter) WriteHeader(statusCode int) {
}

func (rw *responseWriter) FlushError() error {
	rw.flushErrorCalled = true

	return nil
}

func (rw *responseWriter) SetWriteDeadline(deadline time.Time) error {
	rw.setWriteDeadlineCalled = deadline

	return nil
}

func (rw *responseWriter) SetReadDeadline(deadline time.Time) error {
	rw.setReadDeadlineCalled = deadline

	return nil
}

func TestResponseWriterDelegatorUnwrap(t *testing.T) {
	w := &responseWriter{}
	rwd := &responseWriterDelegator{ResponseWriter: w}

	if rwd.Unwrap() != w {
		t.Error("unwrapped responsewriter must equal to the original responsewriter")
	}

	controller := http.NewResponseController(rwd)
	if err := controller.Flush(); err != nil || !w.flushErrorCalled {
		t.Error("FlushError must be propagated to the original responsewriter")
	}

	timeNow := time.Now()
	if err := controller.SetWriteDeadline(timeNow); err != nil || w.setWriteDeadlineCalled != timeNow {
		t.Error("SetWriteDeadline must be propagated to the original responsewriter")
	}

	if err := controller.SetReadDeadline(timeNow); err != nil || w.setReadDeadlineCalled != timeNow {
		t.Error("SetReadDeadline must be propagated to the original responsewriter")
	}
}
