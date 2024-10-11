// Copyright 2020 The Prometheus Authors
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
	"fmt"

	"google.golang.org/protobuf/proto"

	dto "github.com/prometheus/client_model/go"

	"github.com/prometheus/client_golang/prometheus"
)

// Info implements an info pseudo-metric, which is modeled as a Gauge that
// always has a value of 1. In practice, you would just use a Gauge directly,
// but for this example, we pretend it would be useful to have a “native”
// implementation.
type Info struct {
	desc       *prometheus.Desc
	labelPairs []*dto.LabelPair
}

func (i Info) Desc() *prometheus.Desc {
	return i.desc
}

func (i Info) Write(out *dto.Metric) error {
	out.Label = i.labelPairs
	out.Gauge = &dto.Gauge{Value: proto.Float64(1)}
	return nil
}

// InfoVec is the vector version for Info. As an info metric never changes, we
// wouldn't really need to wrap GetMetricWithLabelValues and GetMetricWith
// because Info has no additional methods compared to the vanilla Metric that
// the unwrapped MetricVec methods return. However, to demonstrate all there is
// to do to fully implement a vector for a custom Metric implementation, we do
// it in this example anyway.
type InfoVec struct {
	*prometheus.MetricVec
}

func NewInfoVec(name, help string, labelNames []string) *InfoVec {
	desc := prometheus.NewDesc(name, help, labelNames, nil)
	return &InfoVec{
		MetricVec: prometheus.NewMetricVec(desc, func(lvs ...string) prometheus.Metric {
			if len(lvs) != len(labelNames) {
				panic("inconsistent label cardinality")
			}
			return Info{desc: desc, labelPairs: prometheus.MakeLabelPairs(desc, lvs)}
		}),
	}
}

func (v *InfoVec) GetMetricWithLabelValues(lvs ...string) (Info, error) {
	metric, err := v.MetricVec.GetMetricWithLabelValues(lvs...)
	return metric.(Info), err
}

func (v *InfoVec) GetMetricWith(labels prometheus.Labels) (Info, error) {
	metric, err := v.MetricVec.GetMetricWith(labels)
	return metric.(Info), err
}

func (v *InfoVec) WithLabelValues(lvs ...string) Info {
	i, err := v.GetMetricWithLabelValues(lvs...)
	if err != nil {
		panic(err)
	}
	return i
}

func (v *InfoVec) With(labels prometheus.Labels) Info {
	i, err := v.GetMetricWith(labels)
	if err != nil {
		panic(err)
	}
	return i
}

func (v *InfoVec) CurryWith(labels prometheus.Labels) (*InfoVec, error) {
	vec, err := v.MetricVec.CurryWith(labels)
	if vec != nil {
		return &InfoVec{vec}, err
	}
	return nil, err
}

func (v *InfoVec) MustCurryWith(labels prometheus.Labels) *InfoVec {
	vec, err := v.CurryWith(labels)
	if err != nil {
		panic(err)
	}
	return vec
}

func ExampleMetricVec() {
	infoVec := NewInfoVec(
		"library_version_info",
		"Versions of the libraries used in this binary.",
		[]string{"library", "version"},
	)

	infoVec.WithLabelValues("prometheus/client_golang", "1.7.1")
	infoVec.WithLabelValues("k8s.io/client-go", "0.18.8")

	// Just for demonstration, let's check the state of the InfoVec by
	// registering it with a custom registry and then let it collect the
	// metrics.
	reg := prometheus.NewRegistry()
	reg.MustRegister(infoVec)

	metricFamilies, err := reg.Gather()
	if err != nil || len(metricFamilies) != 1 {
		panic("unexpected behavior of custom test registry")
	}
	fmt.Println(toNormalizedJSON(metricFamilies[0]))

	// Output:
	// {"name":"library_version_info","help":"Versions of the libraries used in this binary.","type":"GAUGE","metric":[{"label":[{"name":"library","value":"k8s.io/client-go"},{"name":"version","value":"0.18.8"}],"gauge":{"value":1}},{"label":[{"name":"library","value":"prometheus/client_golang"},{"name":"version","value":"1.7.1"}],"gauge":{"value":1}}]}
}
