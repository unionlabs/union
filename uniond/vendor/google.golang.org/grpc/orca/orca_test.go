/*
 *
 * Copyright 2022 gRPC authors.
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

package orca_test

import (
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/orca/internal"
	"google.golang.org/protobuf/proto"

	v3orcapb "github.com/cncf/xds/go/xds/data/orca/v3"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const defaultTestTimeout = 5 * time.Second

func (s) TestToLoadReport(t *testing.T) {
	goodReport := &v3orcapb.OrcaLoadReport{
		CpuUtilization: 1.0,
		MemUtilization: 50.0,
		RequestCost:    map[string]float64{"queryCost": 25.0},
		Utilization:    map[string]float64{"queueSize": 75.0},
	}
	tests := []struct {
		name    string
		md      metadata.MD
		want    *v3orcapb.OrcaLoadReport
		wantErr bool
	}{
		{
			name:    "no load report in metadata",
			md:      metadata.MD{},
			wantErr: false,
		},
		{
			name: "badly marshaled load report",
			md: func() metadata.MD {
				return metadata.Pairs("endpoint-load-metrics-bin", string("foo-bar"))
			}(),
			wantErr: true,
		},
		{
			name: "multiple load reports",
			md: func() metadata.MD {
				b, _ := proto.Marshal(goodReport)
				return metadata.Pairs("endpoint-load-metrics-bin", string(b), "endpoint-load-metrics-bin", string(b))
			}(),
			wantErr: true,
		},
		{
			name: "good load report",
			md: func() metadata.MD {
				b, _ := proto.Marshal(goodReport)
				return metadata.Pairs("endpoint-load-metrics-bin", string(b))
			}(),
			want: goodReport,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			got, err := internal.ToLoadReport(test.md)
			if (err != nil) != test.wantErr {
				t.Fatalf("orca.ToLoadReport(%v) = %v, wantErr: %v", test.md, err, test.wantErr)
			}
			if test.wantErr {
				return
			}
			if !cmp.Equal(got, test.want, cmp.Comparer(proto.Equal)) {
				t.Fatalf("Extracted load report from metadata: %s, want: %s", pretty.ToJSON(got), pretty.ToJSON(test.want))
			}
		})
	}
}
