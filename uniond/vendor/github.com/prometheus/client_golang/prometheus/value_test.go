// Copyright 2018 The Prometheus Authors
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
	"fmt"
	"testing"
	"time"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestNewConstMetricInvalidLabelValues(t *testing.T) {
	testCases := []struct {
		desc   string
		labels Labels
	}{
		{
			desc:   "non utf8 label value",
			labels: Labels{"a": "\xFF"},
		},
		{
			desc:   "not enough label values",
			labels: Labels{},
		},
		{
			desc:   "too many label values",
			labels: Labels{"a": "1", "b": "2"},
		},
	}

	for _, test := range testCases {
		metricDesc := NewDesc(
			"sample_value",
			"sample value",
			[]string{"a"},
			Labels{},
		)

		expectPanic(t, func() {
			MustNewConstMetric(metricDesc, CounterValue, 0.3, "\xFF")
		}, fmt.Sprintf("WithLabelValues: expected panic because: %s", test.desc))

		if _, err := NewConstMetric(metricDesc, CounterValue, 0.3, "\xFF"); err == nil {
			t.Errorf("NewConstMetric: expected error because: %s", test.desc)
		}
	}
}

func TestNewConstMetricWithCreatedTimestamp(t *testing.T) {
	now := time.Now()

	for _, tcase := range []struct {
		desc             string
		metricType       ValueType
		createdTimestamp time.Time
		expecErr         bool
		expectedCt       *timestamppb.Timestamp
	}{
		{
			desc:             "gauge with CT",
			metricType:       GaugeValue,
			createdTimestamp: now,
			expecErr:         true,
			expectedCt:       nil,
		},
		{
			desc:             "counter with CT",
			metricType:       CounterValue,
			createdTimestamp: now,
			expecErr:         false,
			expectedCt:       timestamppb.New(now),
		},
	} {
		t.Run(tcase.desc, func(t *testing.T) {
			metricDesc := NewDesc(
				"sample_value",
				"sample value",
				nil,
				nil,
			)
			m, err := NewConstMetricWithCreatedTimestamp(metricDesc, tcase.metricType, float64(1), tcase.createdTimestamp)
			if tcase.expecErr && err == nil {
				t.Errorf("Expected error is test %s, got no err", tcase.desc)
			}
			if !tcase.expecErr && err != nil {
				t.Errorf("Didn't expect error in test %s, got %s", tcase.desc, err.Error())
			}

			if tcase.expectedCt != nil {
				var metric dto.Metric
				m.Write(&metric)
				if metric.Counter.CreatedTimestamp.AsTime() != tcase.expectedCt.AsTime() {
					t.Errorf("Expected timestamp %v, got %v", tcase.expectedCt, &metric.Counter.CreatedTimestamp)
				}
			}
		})
	}
}
