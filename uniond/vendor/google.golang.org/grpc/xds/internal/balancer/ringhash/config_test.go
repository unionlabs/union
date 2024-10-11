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

package ringhash

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/envconfig"
)

func (s) TestParseConfig(t *testing.T) {
	tests := []struct {
		name         string
		js           string
		envConfigCap uint64
		want         *LBConfig
		wantErr      bool
	}{
		{
			name: "OK",
			js:   `{"minRingSize": 1, "maxRingSize": 2}`,
			want: &LBConfig{MinRingSize: 1, MaxRingSize: 2},
		},
		{
			name: "OK with default min",
			js:   `{"maxRingSize": 2000}`,
			want: &LBConfig{MinRingSize: defaultMinSize, MaxRingSize: 2000},
		},
		{
			name: "OK with default max",
			js:   `{"minRingSize": 2000}`,
			want: &LBConfig{MinRingSize: 2000, MaxRingSize: defaultMaxSize},
		},
		{
			name:    "min greater than max",
			js:      `{"minRingSize": 10, "maxRingSize": 2}`,
			want:    nil,
			wantErr: true,
		},
		{
			name:    "min greater than max greater than global limit",
			js:      `{"minRingSize": 6000, "maxRingSize": 5000}`,
			want:    nil,
			wantErr: true,
		},
		{
			name: "max greater than global limit",
			js:   `{"minRingSize": 1, "maxRingSize": 6000}`,
			want: &LBConfig{MinRingSize: 1, MaxRingSize: 4096},
		},
		{
			name: "min and max greater than global limit",
			js:   `{"minRingSize": 5000, "maxRingSize": 6000}`,
			want: &LBConfig{MinRingSize: 4096, MaxRingSize: 4096},
		},
		{
			name:         "min and max less than raised global limit",
			js:           `{"minRingSize": 5000, "maxRingSize": 6000}`,
			envConfigCap: 8000,
			want:         &LBConfig{MinRingSize: 5000, MaxRingSize: 6000},
		},
		{
			name:         "min and max greater than raised global limit",
			js:           `{"minRingSize": 10000, "maxRingSize": 10000}`,
			envConfigCap: 8000,
			want:         &LBConfig{MinRingSize: 8000, MaxRingSize: 8000},
		},
		{
			name:    "min greater than upper bound",
			js:      `{"minRingSize": 8388610, "maxRingSize": 10}`,
			want:    nil,
			wantErr: true,
		},
		{
			name:    "max greater than upper bound",
			js:      `{"minRingSize": 10, "maxRingSize": 8388610}`,
			want:    nil,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if tt.envConfigCap != 0 {
				old := envconfig.RingHashCap
				defer func() { envconfig.RingHashCap = old }()
				envconfig.RingHashCap = tt.envConfigCap
			}
			got, err := parseConfig([]byte(tt.js))
			if (err != nil) != tt.wantErr {
				t.Errorf("parseConfig() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if diff := cmp.Diff(got, tt.want); diff != "" {
				t.Errorf("parseConfig() got unexpected output, diff (-got +want): %v", diff)
			}
		})
	}
}
