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
package prometheus_test

import (
	"bytes"
	"encoding/json"
	"time"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// sanitizeMetric injects expected fake created timestamp value "1970-01-01T00:00:10Z",
// so we can compare it in examples. It modifies metric in-place, the returned pointer
// is for convenience.
func sanitizeMetric(metric *dto.Metric) *dto.Metric {
	if metric.Counter != nil && metric.Counter.CreatedTimestamp != nil {
		metric.Counter.CreatedTimestamp = timestamppb.New(time.Unix(10, 0))
	}
	if metric.Summary != nil && metric.Summary.CreatedTimestamp != nil {
		metric.Summary.CreatedTimestamp = timestamppb.New(time.Unix(10, 0))
	}
	if metric.Histogram != nil && metric.Histogram.CreatedTimestamp != nil {
		metric.Histogram.CreatedTimestamp = timestamppb.New(time.Unix(10, 0))
	}
	return metric
}

// sanitizeMetricFamily is like sanitizeMetric, but for multiple metrics.
func sanitizeMetricFamily(f *dto.MetricFamily) *dto.MetricFamily {
	for _, m := range f.Metric {
		sanitizeMetric(m)
	}
	return f
}

// toNormalizedJSON removes fake random space from proto JSON original marshaller.
// It is required, so we can compare proto messages in json format.
// Read more in https://github.com/golang/protobuf/issues/1121
func toNormalizedJSON(m proto.Message) string {
	mAsJSON, err := protojson.Marshal(m)
	if err != nil {
		panic(err)
	}

	buffer := new(bytes.Buffer)
	if err := json.Compact(buffer, mAsJSON); err != nil {
		panic(err)
	}
	return buffer.String()
}
