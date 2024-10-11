/*
 *
 * Copyright 2020 gRPC authors.
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

package weightedtarget

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/balancer"
	_ "google.golang.org/grpc/balancer/grpclb"
	"google.golang.org/grpc/balancer/roundrobin"
	internalserviceconfig "google.golang.org/grpc/internal/serviceconfig"
)

const (
	testJSONConfig = `{
  "targets": {
	"cluster_1": {
	  "weight": 75,
	  "childPolicy": [{
        "grpclb": {
          "childPolicy": [{"pick_first":{}}],
          "targetName": "foo-service"
        }
      }]
	},
	"cluster_2": {
	  "weight": 25,
	  "childPolicy": [{"round_robin": ""}]
	}
  }
}`
)

var (
	grpclbConfigParser = balancer.Get("grpclb").(balancer.ConfigParser)
	grpclbConfigJSON   = `{"childPolicy": [{"pick_first":{}}], "targetName": "foo-service"}`
	grpclbConfig, _    = grpclbConfigParser.ParseConfig([]byte(grpclbConfigJSON))
)

func (s) TestParseConfig(t *testing.T) {
	tests := []struct {
		name    string
		js      string
		want    *LBConfig
		wantErr bool
	}{
		{
			name:    "empty json",
			js:      "",
			want:    nil,
			wantErr: true,
		},
		{
			name: "OK",
			js:   testJSONConfig,
			want: &LBConfig{
				Targets: map[string]Target{
					"cluster_1": {
						Weight: 75,
						ChildPolicy: &internalserviceconfig.BalancerConfig{
							Name:   "grpclb",
							Config: grpclbConfig,
						},
					},
					"cluster_2": {
						Weight: 25,
						ChildPolicy: &internalserviceconfig.BalancerConfig{
							Name: roundrobin.Name,
						},
					},
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := parseConfig([]byte(tt.js))
			if (err != nil) != tt.wantErr {
				t.Fatalf("parseConfig() error = %v, wantErr %v", err, tt.wantErr)
			}
			if !cmp.Equal(got, tt.want) {
				t.Errorf("parseConfig() got unexpected result, diff: %v", cmp.Diff(got, tt.want))
			}
		})
	}
}
