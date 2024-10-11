/*
 *
 * Copyright 2024 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package grpchttp2

import (
	"testing"

	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestErrorCodeString(t *testing.T) {
	tests := []struct {
		err  ErrCode
		want string
	}{
		// Known error cases
		{err: ErrCodeNoError, want: "NO_ERROR"},
		{err: ErrCodeProtocol, want: "PROTOCOL_ERROR"},
		{err: ErrCodeInternal, want: "INTERNAL_ERROR"},
		{err: ErrCodeFlowControl, want: "FLOW_CONTROL_ERROR"},
		{err: ErrCodeSettingsTimeout, want: "SETTINGS_TIMEOUT"},
		{err: ErrCodeStreamClosed, want: "STREAM_CLOSED"},
		{err: ErrCodeFrameSize, want: "FRAME_SIZE_ERROR"},
		{err: ErrCodeRefusedStream, want: "REFUSED_STREAM"},
		{err: ErrCodeCancel, want: "CANCEL"},
		{err: ErrCodeCompression, want: "COMPRESSION_ERROR"},
		{err: ErrCodeConnect, want: "CONNECT_ERROR"},
		{err: ErrCodeEnhanceYourCalm, want: "ENHANCE_YOUR_CALM"},
		{err: ErrCodeInadequateSecurity, want: "INADEQUATE_SECURITY"},
		{err: ErrCodeHTTP11Required, want: "HTTP_1_1_REQUIRED"},
		// Type casting known error case
		{err: ErrCode(0x1), want: "PROTOCOL_ERROR"},
		// Unknown error case
		{err: ErrCode(0xf), want: "unknown error code 0xf"},
	}

	for _, test := range tests {
		got := test.err.String()
		if got != test.want {
			t.Errorf("ErrCode.String(%#x) = %q, want %q", int(test.err), got, test.want)
		}
	}
}
