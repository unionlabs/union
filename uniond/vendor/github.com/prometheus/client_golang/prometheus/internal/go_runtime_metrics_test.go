// Copyright 2021 The Prometheus Authors
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

//go:build go1.17
// +build go1.17

package internal

import (
	"runtime/metrics"
	"testing"
)

func TestRuntimeMetricsToProm(t *testing.T) {
	tests := []struct {
		got    metrics.Description
		expect string
	}{
		{
			metrics.Description{
				Name: "/memory/live:bytes",
				Kind: metrics.KindUint64,
			},
			"go_memory_live_bytes",
		},
		{
			metrics.Description{
				Name:       "/memory/allocs:bytes",
				Kind:       metrics.KindUint64,
				Cumulative: true,
			},
			"go_memory_allocs_bytes_total",
		},
		{
			metrics.Description{
				Name: "/memory/alloc-rate:bytes/second",
				Kind: metrics.KindFloat64,
			},
			"go_memory_alloc_rate_bytes_per_second",
		},
		{
			metrics.Description{
				Name:       "/gc/time:cpu*seconds",
				Kind:       metrics.KindFloat64,
				Cumulative: true,
			},
			"go_gc_time_cpu_seconds_total",
		},
		{
			metrics.Description{
				Name: "/this/is/a/very/deep/metric:metrics",
				Kind: metrics.KindFloat64,
			},
			"go_this_is_a_very_deep_metric_metrics",
		},
		{
			metrics.Description{
				Name: "/this*is*an*invalid...:µname",
				Kind: metrics.KindUint64,
			},
			"",
		},
		{
			metrics.Description{
				Name: "/this/is/a/valid/name:objects",
				Kind: metrics.KindBad,
			},
			"",
		},
	}
	for _, test := range tests {
		ns, ss, n, ok := RuntimeMetricsToProm(&test.got)
		name := ns + "_" + ss + "_" + n
		if test.expect == "" && ok {
			t.Errorf("bad input expected a bad output: input %s, got %s", test.got.Name, name)
			continue
		}
		if test.expect != "" && !ok {
			t.Errorf("unexpected bad output on good input: input %s", test.got.Name)
			continue
		}
		if test.expect != "" && name != test.expect {
			t.Errorf("expected %s, got %s", test.expect, name)
			continue
		}
	}
}
