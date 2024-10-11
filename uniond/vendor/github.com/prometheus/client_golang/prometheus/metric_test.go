// Copyright 2014 The Prometheus Authors
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

package prometheus

import (
	"math"
	"testing"

	dto "github.com/prometheus/client_model/go"

	"google.golang.org/protobuf/proto"
)

func TestBuildFQName(t *testing.T) {
	scenarios := []struct{ namespace, subsystem, name, result string }{
		{"a", "b", "c", "a_b_c"},
		{"", "b", "c", "b_c"},
		{"a", "", "c", "a_c"},
		{"", "", "c", "c"},
		{"a", "b", "", ""},
		{"a", "", "", ""},
		{"", "b", "", ""},
		{" ", "", "", ""},
	}

	for i, s := range scenarios {
		if want, got := s.result, BuildFQName(s.namespace, s.subsystem, s.name); want != got {
			t.Errorf("%d. want %s, got %s", i, want, got)
		}
	}
}

func TestWithExemplarsMetric(t *testing.T) {
	t.Run("histogram", func(t *testing.T) {
		// Create a constant histogram from values we got from a 3rd party telemetry system.
		h := MustNewConstHistogram(
			NewDesc("http_request_duration_seconds", "A histogram of the HTTP request durations.", nil, nil),
			4711, 403.34,
			// Four buckets, but we expect five as the +Inf bucket will be created if we see value outside of those buckets.
			map[float64]uint64{25: 121, 50: 2403, 100: 3221, 200: 4233},
		)

		m := &withExemplarsMetric{Metric: h, exemplars: []*dto.Exemplar{
			{Value: proto.Float64(2000.0)}, // Unordered exemplars.
			{Value: proto.Float64(500.0)},
			{Value: proto.Float64(42.0)},
			{Value: proto.Float64(157.0)},
			{Value: proto.Float64(100.0)},
			{Value: proto.Float64(89.0)},
			{Value: proto.Float64(24.0)},
			{Value: proto.Float64(25.1)},
		}}
		metric := dto.Metric{}
		if err := m.Write(&metric); err != nil {
			t.Fatal(err)
		}
		if want, got := 5, len(metric.GetHistogram().Bucket); want != got {
			t.Errorf("want %v, got %v", want, got)
		}

		expectedExemplarVals := []float64{24.0, 25.1, 89.0, 157.0, 500.0}
		for i, b := range metric.GetHistogram().Bucket {
			if b.Exemplar == nil {
				t.Errorf("Expected exemplar for bucket %v, got nil", i)
			}
			if want, got := expectedExemplarVals[i], *metric.GetHistogram().Bucket[i].Exemplar.Value; want != got {
				t.Errorf("%v: want %v, got %v", i, want, got)
			}
		}

		infBucket := metric.GetHistogram().Bucket[len(metric.GetHistogram().Bucket)-1]

		if want, got := math.Inf(1), infBucket.GetUpperBound(); want != got {
			t.Errorf("want %v, got %v", want, got)
		}

		if want, got := uint64(4711), infBucket.GetCumulativeCount(); want != got {
			t.Errorf("want %v, got %v", want, got)
		}
	})
}
