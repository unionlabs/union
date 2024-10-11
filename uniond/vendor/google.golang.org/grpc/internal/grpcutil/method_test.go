/*
 *
 * Copyright 2018 gRPC authors.
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

package grpcutil

import (
	"testing"
)

func TestParseMethod(t *testing.T) {
	testCases := []struct {
		methodName  string
		wantService string
		wantMethod  string
		wantError   bool
	}{
		{methodName: "/s/m", wantService: "s", wantMethod: "m", wantError: false},
		{methodName: "/p.s/m", wantService: "p.s", wantMethod: "m", wantError: false},
		{methodName: "/p/s/m", wantService: "p/s", wantMethod: "m", wantError: false},
		{methodName: "/", wantError: true},
		{methodName: "/sm", wantError: true},
		{methodName: "", wantError: true},
		{methodName: "sm", wantError: true},
	}
	for _, tc := range testCases {
		s, m, err := ParseMethod(tc.methodName)
		if (err != nil) != tc.wantError || s != tc.wantService || m != tc.wantMethod {
			t.Errorf("ParseMethod(%s) = (%s, %s, %v), want (%s, %s, %v)", tc.methodName, s, m, err, tc.wantService, tc.wantMethod, tc.wantError)
		}
	}
}

func TestContentSubtype(t *testing.T) {
	tests := []struct {
		contentType string
		want        string
		wantValid   bool
	}{
		{"application/grpc", "", true},
		{"application/grpc+", "", true},
		{"application/grpc+blah", "blah", true},
		{"application/grpc;", "", true},
		{"application/grpc;blah", "blah", true},
		{"application/grpcd", "", false},
		{"application/grpd", "", false},
		{"application/grp", "", false},
	}
	for _, tt := range tests {
		got, gotValid := ContentSubtype(tt.contentType)
		if got != tt.want || gotValid != tt.wantValid {
			t.Errorf("contentSubtype(%q) = (%v, %v); want (%v, %v)", tt.contentType, got, gotValid, tt.want, tt.wantValid)
		}
	}
}
