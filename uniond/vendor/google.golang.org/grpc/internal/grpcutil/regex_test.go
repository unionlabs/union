/*
 *
 * Copyright 2021 gRPC authors.
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
	"regexp"
	"testing"
)

func TestFullMatchWithRegex(t *testing.T) {
	tests := []struct {
		name     string
		regexStr string
		string   string
		want     bool
	}{
		{
			name:     "not match because only partial",
			regexStr: "^a+$",
			string:   "ab",
			want:     false,
		},
		{
			name:     "match because fully match",
			regexStr: "^a+$",
			string:   "aa",
			want:     true,
		},
		{
			name:     "longest",
			regexStr: "a(|b)",
			string:   "ab",
			want:     true,
		},
		{
			name:     "match all",
			regexStr: ".*",
			string:   "",
			want:     true,
		},
		{
			name:     "matches non-empty strings",
			regexStr: ".+",
			string:   "",
			want:     false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			hrm := regexp.MustCompile(tt.regexStr)
			if got := FullMatchWithRegex(hrm, tt.string); got != tt.want {
				t.Errorf("match() = %v, want %v", got, tt.want)
			}
		})
	}
}
