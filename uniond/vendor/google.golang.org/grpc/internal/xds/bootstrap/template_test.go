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
 */

package bootstrap

import "testing"

func Test_percentEncode(t *testing.T) {
	tests := []struct {
		name   string
		target string
		want   string
	}{
		{
			name:   "normal name",
			target: "server.example.com",
			want:   "server.example.com",
		},
		{
			name:   "ipv4",
			target: "0.0.0.0:8080",
			want:   "0.0.0.0:8080",
		},
		{
			name:   "ipv6",
			target: "[::1]:8080",
			want:   "%5B::1%5D:8080", // [ and ] are percent encoded.
		},
		{
			name:   "/ should not be percent encoded",
			target: "my/service/region",
			want:   "my/service/region", // "/"s are kept.
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := percentEncode(tt.target); got != tt.want {
				t.Errorf("percentEncode() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestPopulateResourceTemplate(t *testing.T) {
	tests := []struct {
		name     string
		template string
		target   string
		want     string
	}{
		{
			name:     "no %s",
			template: "/name/template",
			target:   "[::1]:8080",
			want:     "/name/template",
		},
		{
			name:     "with %s, no xdstp: prefix, ipv6",
			template: "/name/template/%s",
			target:   "[::1]:8080",
			want:     "/name/template/[::1]:8080",
		},
		{
			name:     "with %s, with xdstp: prefix",
			template: "xdstp://authority.com/%s",
			target:   "0.0.0.0:8080",
			want:     "xdstp://authority.com/0.0.0.0:8080",
		},
		{
			name:     "with %s, with xdstp: prefix, and ipv6",
			template: "xdstp://authority.com/%s",
			target:   "[::1]:8080",
			want:     "xdstp://authority.com/%5B::1%5D:8080",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := PopulateResourceTemplate(tt.template, tt.target); got != tt.want {
				t.Errorf("PopulateResourceTemplate() = %v, want %v", got, tt.want)
			}
		})
	}
}
