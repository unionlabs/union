/*
 *
 * Copyright 2019 gRPC authors.
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

package grpclb

import (
	"encoding/json"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/serviceconfig"
)

func (s) TestParse(t *testing.T) {
	tests := []struct {
		name    string
		sc      string
		want    serviceconfig.LoadBalancingConfig
		wantErr bool
	}{
		{
			name:    "empty",
			sc:      "",
			want:    nil,
			wantErr: true,
		},
		{
			name: "success1",
			sc: `
{
	"childPolicy": [
		{"pick_first":{}}
	],
	"serviceName": "foo-service"
}`,
			want: &grpclbServiceConfig{
				ChildPolicy: &[]map[string]json.RawMessage{
					{"pick_first": json.RawMessage("{}")},
				},
				ServiceName: "foo-service",
			},
		},
		{
			name: "success2",
			sc: `
{
	"childPolicy": [
		{"round_robin":{}},
		{"pick_first":{}}
	],
	"serviceName": "foo-service"
}`,
			want: &grpclbServiceConfig{
				ChildPolicy: &[]map[string]json.RawMessage{
					{"round_robin": json.RawMessage("{}")},
					{"pick_first": json.RawMessage("{}")},
				},
				ServiceName: "foo-service",
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := (&lbBuilder{}).ParseConfig(json.RawMessage(tt.sc))
			if (err != nil) != (tt.wantErr) {
				t.Fatalf("ParseConfig(%q) returned error: %v, wantErr: %v", tt.sc, err, tt.wantErr)
			}
			if diff := cmp.Diff(tt.want, got); diff != "" {
				t.Fatalf("ParseConfig(%q) returned unexpected difference (-want +got):\n%s", tt.sc, diff)
			}
		})
	}
}

func (s) TestChildIsPickFirst(t *testing.T) {
	tests := []struct {
		name string
		s    string
		want bool
	}{
		{
			name: "pickfirst_only",
			s:    `{"childPolicy":[{"pick_first":{}}]}`,
			want: true,
		},
		{
			name: "pickfirst_before_rr",
			s:    `{"childPolicy":[{"pick_first":{}},{"round_robin":{}}]}`,
			want: true,
		},
		{
			name: "rr_before_pickfirst",
			s:    `{"childPolicy":[{"round_robin":{}},{"pick_first":{}}]}`,
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			gc, err := (&lbBuilder{}).ParseConfig(json.RawMessage(tt.s))
			if err != nil {
				t.Fatalf("Parse(%v) = _, %v; want _, nil", tt.s, err)
			}
			if got := childIsPickFirst(gc.(*grpclbServiceConfig)); got != tt.want {
				t.Errorf("childIsPickFirst() = %v, want %v", got, tt.want)
			}
		})
	}
}
