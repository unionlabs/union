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

package expfmt

import (
	"bytes"
	"math"
	"strings"
	"testing"
	"time"

	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"

	dto "github.com/prometheus/client_model/go"

	"github.com/prometheus/common/model"
)

func TestCreateOpenMetrics(t *testing.T) {
	openMetricsTimestamp := timestamppb.New(time.Unix(12345, 600000000))
	if err := openMetricsTimestamp.CheckValid(); err != nil {
		t.Error(err)
	}

	oldDefaultScheme := model.NameEscapingScheme
	model.NameEscapingScheme = model.NoEscaping
	defer func() {
		model.NameEscapingScheme = oldDefaultScheme
	}()

	scenarios := []struct {
		in      *dto.MetricFamily
		options []EncoderOption
		out     string
	}{
		// 0: Counter, timestamp given, no _total suffix.
		{
			in: &dto.MetricFamily{
				Name: proto.String("name"),
				Help: proto.String("two-line\n doc  str\\ing"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val1"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(42),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val2"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(.23),
						},
						TimestampMs: proto.Int64(1234567890),
					},
				},
			},
			out: `# HELP name two-line\n doc  str\\ing
# TYPE name unknown
name{labelname="val1",basename="basevalue"} 42.0
name{labelname="val2",basename="basevalue"} 0.23 1.23456789e+06
`,
		},
		// 1: Dots in name
		{
			in: &dto.MetricFamily{
				Name: proto.String("name.with.dots"),
				Help: proto.String("boring help"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val1"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(42),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val2"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(.23),
						},
						TimestampMs: proto.Int64(1234567890),
					},
				},
			},
			out: `# HELP "name.with.dots" boring help
# TYPE "name.with.dots" unknown
{"name.with.dots",labelname="val1",basename="basevalue"} 42.0
{"name.with.dots",labelname="val2",basename="basevalue"} 0.23 1.23456789e+06
`,
		},
		// 2: Dots in name, no labels
		{
			in: &dto.MetricFamily{
				Name: proto.String("name.with.dots"),
				Help: proto.String("boring help"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(42),
						},
					},
					{
						Counter: &dto.Counter{
							Value: proto.Float64(.23),
						},
						TimestampMs: proto.Int64(1234567890),
					},
				},
			},
			out: `# HELP "name.with.dots" boring help
# TYPE "name.with.dots" unknown
{"name.with.dots"} 42.0
{"name.with.dots"} 0.23 1.23456789e+06
`,
		},
		// 3: Gauge, some escaping required, +Inf as value, multi-byte characters in label values.
		{
			in: &dto.MetricFamily{
				Name: proto.String("gauge_name"),
				Help: proto.String("gauge\ndoc\nstr\"ing"),
				Type: dto.MetricType_GAUGE.Enum(),
				Metric: []*dto.Metric{
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name_1"),
								Value: proto.String("val with\nnew line"),
							},
							{
								Name:  proto.String("name_2"),
								Value: proto.String("val with \\backslash and \"quotes\""),
							},
						},
						Gauge: &dto.Gauge{
							Value: proto.Float64(math.Inf(+1)),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name_1"),
								Value: proto.String("Björn"),
							},
							{
								Name:  proto.String("name_2"),
								Value: proto.String("佖佥"),
							},
						},
						Gauge: &dto.Gauge{
							Value: proto.Float64(3.14e42),
						},
					},
				},
			},
			out: `# HELP gauge_name gauge\ndoc\nstr\"ing
# TYPE gauge_name gauge
gauge_name{name_1="val with\nnew line",name_2="val with \\backslash and \"quotes\""} +Inf
gauge_name{name_1="Björn",name_2="佖佥"} 3.14e+42
`,
		},
		// 4: Gauge, utf-8, some escaping required, +Inf as value, multi-byte characters in label values.
		{
			in: &dto.MetricFamily{
				Name: proto.String("gauge.name\""),
				Help: proto.String("gauge\ndoc\nstr\"ing"),
				Type: dto.MetricType_GAUGE.Enum(),
				Metric: []*dto.Metric{
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name.1"),
								Value: proto.String("val with\nnew line"),
							},
							{
								Name:  proto.String("name*2"),
								Value: proto.String("val with \\backslash and \"quotes\""),
							},
						},
						Gauge: &dto.Gauge{
							Value: proto.Float64(math.Inf(+1)),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name.1"),
								Value: proto.String("Björn"),
							},
							{
								Name:  proto.String("name*2"),
								Value: proto.String("佖佥"),
							},
						},
						Gauge: &dto.Gauge{
							Value: proto.Float64(3.14e42),
						},
					},
				},
			},
			out: `# HELP "gauge.name\"" gauge\ndoc\nstr\"ing
# TYPE "gauge.name\"" gauge
{"gauge.name\"","name.1"="val with\nnew line","name*2"="val with \\backslash and \"quotes\""} +Inf
{"gauge.name\"","name.1"="Björn","name*2"="佖佥"} 3.14e+42
`,
		},
		// 5: Unknown, no help, one sample with no labels and -Inf as value, another sample with one label.
		{
			in: &dto.MetricFamily{
				Name: proto.String("unknown_name"),
				Type: dto.MetricType_UNTYPED.Enum(),
				Metric: []*dto.Metric{
					{
						Untyped: &dto.Untyped{
							Value: proto.Float64(math.Inf(-1)),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name_1"),
								Value: proto.String("value 1"),
							},
						},
						Untyped: &dto.Untyped{
							Value: proto.Float64(-1.23e-45),
						},
					},
				},
			},
			out: `# TYPE unknown_name unknown
unknown_name -Inf
unknown_name{name_1="value 1"} -1.23e-45
`,
		},
		// 6: Summary.
		{
			in: &dto.MetricFamily{
				Name: proto.String("summary_name"),
				Help: proto.String("summary docstring"),
				Type: dto.MetricType_SUMMARY.Enum(),
				Metric: []*dto.Metric{
					{
						Summary: &dto.Summary{
							SampleCount: proto.Uint64(42),
							SampleSum:   proto.Float64(-3.4567),
							Quantile: []*dto.Quantile{
								{
									Quantile: proto.Float64(0.5),
									Value:    proto.Float64(-1.23),
								},
								{
									Quantile: proto.Float64(0.9),
									Value:    proto.Float64(.2342354),
								},
								{
									Quantile: proto.Float64(0.99),
									Value:    proto.Float64(0),
								},
							},
							CreatedTimestamp: openMetricsTimestamp,
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("name_1"),
								Value: proto.String("value 1"),
							},
							{
								Name:  proto.String("name_2"),
								Value: proto.String("value 2"),
							},
						},
						Summary: &dto.Summary{
							SampleCount: proto.Uint64(4711),
							SampleSum:   proto.Float64(2010.1971),
							Quantile: []*dto.Quantile{
								{
									Quantile: proto.Float64(0.5),
									Value:    proto.Float64(1),
								},
								{
									Quantile: proto.Float64(0.9),
									Value:    proto.Float64(2),
								},
								{
									Quantile: proto.Float64(0.99),
									Value:    proto.Float64(3),
								},
							},
							CreatedTimestamp: openMetricsTimestamp,
						},
					},
				},
			},
			options: []EncoderOption{WithCreatedLines()},
			out: `# HELP summary_name summary docstring
# TYPE summary_name summary
summary_name{quantile="0.5"} -1.23
summary_name{quantile="0.9"} 0.2342354
summary_name{quantile="0.99"} 0.0
summary_name_sum -3.4567
summary_name_count 42
summary_name_created 12345.6
summary_name{name_1="value 1",name_2="value 2",quantile="0.5"} 1.0
summary_name{name_1="value 1",name_2="value 2",quantile="0.9"} 2.0
summary_name{name_1="value 1",name_2="value 2",quantile="0.99"} 3.0
summary_name_sum{name_1="value 1",name_2="value 2"} 2010.1971
summary_name_count{name_1="value 1",name_2="value 2"} 4711
summary_name_created{name_1="value 1",name_2="value 2"} 12345.6
`,
		},
		// 7: Histogram
		{
			in: &dto.MetricFamily{
				Name: proto.String("request_duration_microseconds"),
				Help: proto.String("The response latency."),
				Type: dto.MetricType_HISTOGRAM.Enum(),
				Unit: proto.String("microseconds"),
				Metric: []*dto.Metric{
					{
						Histogram: &dto.Histogram{
							SampleCount: proto.Uint64(2693),
							SampleSum:   proto.Float64(1756047.3),
							Bucket: []*dto.Bucket{
								{
									UpperBound:      proto.Float64(100),
									CumulativeCount: proto.Uint64(123),
								},
								{
									UpperBound:      proto.Float64(120),
									CumulativeCount: proto.Uint64(412),
								},
								{
									UpperBound:      proto.Float64(144),
									CumulativeCount: proto.Uint64(592),
								},
								{
									UpperBound:      proto.Float64(172.8),
									CumulativeCount: proto.Uint64(1524),
								},
								{
									UpperBound:      proto.Float64(math.Inf(+1)),
									CumulativeCount: proto.Uint64(2693),
								},
							},
							CreatedTimestamp: openMetricsTimestamp,
						},
					},
				},
			},
			options: []EncoderOption{WithCreatedLines(), WithUnit()},
			out: `# HELP request_duration_microseconds The response latency.
# TYPE request_duration_microseconds histogram
# UNIT request_duration_microseconds microseconds
request_duration_microseconds_bucket{le="100.0"} 123
request_duration_microseconds_bucket{le="120.0"} 412
request_duration_microseconds_bucket{le="144.0"} 592
request_duration_microseconds_bucket{le="172.8"} 1524
request_duration_microseconds_bucket{le="+Inf"} 2693
request_duration_microseconds_sum 1.7560473e+06
request_duration_microseconds_count 2693
request_duration_microseconds_created 12345.6
`,
		},
		// 8: Histogram with missing +Inf bucket.
		{
			in: &dto.MetricFamily{
				Name: proto.String("request_duration_microseconds"),
				Help: proto.String("The response latency."),
				Type: dto.MetricType_HISTOGRAM.Enum(),
				Unit: proto.String("microseconds"),
				Metric: []*dto.Metric{
					{
						Histogram: &dto.Histogram{
							SampleCount: proto.Uint64(2693),
							SampleSum:   proto.Float64(1756047.3),
							Bucket: []*dto.Bucket{
								{
									UpperBound:      proto.Float64(100),
									CumulativeCount: proto.Uint64(123),
								},
								{
									UpperBound:      proto.Float64(120),
									CumulativeCount: proto.Uint64(412),
								},
								{
									UpperBound:      proto.Float64(144),
									CumulativeCount: proto.Uint64(592),
								},
								{
									UpperBound:      proto.Float64(172.8),
									CumulativeCount: proto.Uint64(1524),
								},
							},
						},
					},
				},
			},
			out: `# HELP request_duration_microseconds The response latency.
# TYPE request_duration_microseconds histogram
request_duration_microseconds_bucket{le="100.0"} 123
request_duration_microseconds_bucket{le="120.0"} 412
request_duration_microseconds_bucket{le="144.0"} 592
request_duration_microseconds_bucket{le="172.8"} 1524
request_duration_microseconds_bucket{le="+Inf"} 2693
request_duration_microseconds_sum 1.7560473e+06
request_duration_microseconds_count 2693
`,
		},
		// 9: Histogram with missing +Inf bucket but with different exemplars.
		{
			in: &dto.MetricFamily{
				Name: proto.String("request_duration_microseconds"),
				Help: proto.String("The response latency."),
				Type: dto.MetricType_HISTOGRAM.Enum(),
				Metric: []*dto.Metric{
					{
						Histogram: &dto.Histogram{
							SampleCount: proto.Uint64(2693),
							SampleSum:   proto.Float64(1756047.3),
							Bucket: []*dto.Bucket{
								{
									UpperBound:      proto.Float64(100),
									CumulativeCount: proto.Uint64(123),
								},
								{
									UpperBound:      proto.Float64(120),
									CumulativeCount: proto.Uint64(412),
									Exemplar: &dto.Exemplar{
										Label: []*dto.LabelPair{
											{
												Name:  proto.String("foo"),
												Value: proto.String("bar"),
											},
										},
										Value:     proto.Float64(119.9),
										Timestamp: openMetricsTimestamp,
									},
								},
								{
									UpperBound:      proto.Float64(144),
									CumulativeCount: proto.Uint64(592),
									Exemplar: &dto.Exemplar{
										Label: []*dto.LabelPair{
											{
												Name:  proto.String("foo"),
												Value: proto.String("baz"),
											},
											{
												Name:  proto.String("dings"),
												Value: proto.String("bums"),
											},
										},
										Value: proto.Float64(140.14),
									},
								},
								{
									UpperBound:      proto.Float64(172.8),
									CumulativeCount: proto.Uint64(1524),
								},
							},
						},
					},
				},
			},
			out: `# HELP request_duration_microseconds The response latency.
# TYPE request_duration_microseconds histogram
request_duration_microseconds_bucket{le="100.0"} 123
request_duration_microseconds_bucket{le="120.0"} 412 # {foo="bar"} 119.9 12345.6
request_duration_microseconds_bucket{le="144.0"} 592 # {foo="baz",dings="bums"} 140.14
request_duration_microseconds_bucket{le="172.8"} 1524
request_duration_microseconds_bucket{le="+Inf"} 2693
request_duration_microseconds_sum 1.7560473e+06
request_duration_microseconds_count 2693
`,
		},
		// 10: Simple Counter.
		{
			in: &dto.MetricFamily{
				Name: proto.String("foos_total"),
				Help: proto.String("Number of foos."),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value:            proto.Float64(42),
							CreatedTimestamp: openMetricsTimestamp,
						},
					},
				},
			},
			options: []EncoderOption{WithCreatedLines()},
			out: `# HELP foos Number of foos.
# TYPE foos counter
foos_total 42.0
foos_created 12345.6
`,
		},
		// 11: Simple Counter without created line.
		{
			in: &dto.MetricFamily{
				Name: proto.String("foos_total"),
				Help: proto.String("Number of foos."),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value:            proto.Float64(42),
							CreatedTimestamp: openMetricsTimestamp,
						},
					},
				},
			},
			out: `# HELP foos Number of foos.
# TYPE foos counter
foos_total 42.0
`,
		},
		// 12: No metric.
		{
			in: &dto.MetricFamily{
				Name:   proto.String("name_total"),
				Help:   proto.String("doc string"),
				Type:   dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{},
			},
			out: `# HELP name doc string
# TYPE name counter
`,
		},
		// 13: Simple Counter with exemplar that has empty label set:
		// ignore the exemplar, since OpenMetrics spec requires labels.
		{
			in: &dto.MetricFamily{
				Name: proto.String("foos_total"),
				Help: proto.String("Number of foos."),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(42),
							Exemplar: &dto.Exemplar{
								Label:     []*dto.LabelPair{},
								Value:     proto.Float64(1),
								Timestamp: openMetricsTimestamp,
							},
						},
					},
				},
			},
			out: `# HELP foos Number of foos.
# TYPE foos counter
foos_total 42.0
`,
		},
		// 14: No metric plus unit.
		{
			in: &dto.MetricFamily{
				Name:   proto.String("name_seconds_total"),
				Help:   proto.String("doc string"),
				Type:   dto.MetricType_COUNTER.Enum(),
				Unit:   proto.String("seconds"),
				Metric: []*dto.Metric{},
			},
			options: []EncoderOption{WithUnit()},
			out: `# HELP name_seconds doc string
# TYPE name_seconds counter
# UNIT name_seconds seconds
`,
		},
		// 15: Histogram plus unit, but unit not opted in.
		{
			in: &dto.MetricFamily{
				Name: proto.String("request_duration_microseconds"),
				Help: proto.String("The response latency."),
				Type: dto.MetricType_HISTOGRAM.Enum(),
				Unit: proto.String("microseconds"),
				Metric: []*dto.Metric{
					{
						Histogram: &dto.Histogram{
							SampleCount: proto.Uint64(2693),
							SampleSum:   proto.Float64(1756047.3),
							Bucket: []*dto.Bucket{
								{
									UpperBound:      proto.Float64(100),
									CumulativeCount: proto.Uint64(123),
								},
								{
									UpperBound:      proto.Float64(120),
									CumulativeCount: proto.Uint64(412),
								},
								{
									UpperBound:      proto.Float64(144),
									CumulativeCount: proto.Uint64(592),
								},
								{
									UpperBound:      proto.Float64(172.8),
									CumulativeCount: proto.Uint64(1524),
								},
								{
									UpperBound:      proto.Float64(math.Inf(+1)),
									CumulativeCount: proto.Uint64(2693),
								},
							},
						},
					},
				},
			},
			out: `# HELP request_duration_microseconds The response latency.
# TYPE request_duration_microseconds histogram
request_duration_microseconds_bucket{le="100.0"} 123
request_duration_microseconds_bucket{le="120.0"} 412
request_duration_microseconds_bucket{le="144.0"} 592
request_duration_microseconds_bucket{le="172.8"} 1524
request_duration_microseconds_bucket{le="+Inf"} 2693
request_duration_microseconds_sum 1.7560473e+06
request_duration_microseconds_count 2693
`,
		},
		// 16: No metric, unit opted in, no unit in name.
		{
			in: &dto.MetricFamily{
				Name:   proto.String("name_total"),
				Help:   proto.String("doc string"),
				Type:   dto.MetricType_COUNTER.Enum(),
				Unit:   proto.String("seconds"),
				Metric: []*dto.Metric{},
			},
			options: []EncoderOption{WithUnit()},
			out: `# HELP name_seconds doc string
# TYPE name_seconds counter
# UNIT name_seconds seconds
`,
		},
		// 17: No metric, unit opted in, BUT unit == nil.
		{
			in: &dto.MetricFamily{
				Name:   proto.String("name_total"),
				Help:   proto.String("doc string"),
				Type:   dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{},
			},
			options: []EncoderOption{WithUnit()},
			out: `# HELP name doc string
# TYPE name counter
`,
		},
		// 18: Counter, timestamp given, unit opted in, _total suffix.
		{
			in: &dto.MetricFamily{
				Name: proto.String("some_measure_total"),
				Help: proto.String("some testing measurement"),
				Type: dto.MetricType_COUNTER.Enum(),
				Unit: proto.String("seconds"),
				Metric: []*dto.Metric{
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val1"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(42),
						},
					},
					{
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("labelname"),
								Value: proto.String("val2"),
							},
							{
								Name:  proto.String("basename"),
								Value: proto.String("basevalue"),
							},
						},
						Counter: &dto.Counter{
							Value: proto.Float64(.23),
						},
						TimestampMs: proto.Int64(1234567890),
					},
				},
			},
			options: []EncoderOption{WithUnit()},
			out: `# HELP some_measure_seconds some testing measurement
# TYPE some_measure_seconds counter
# UNIT some_measure_seconds seconds
some_measure_seconds_total{labelname="val1",basename="basevalue"} 42.0
some_measure_seconds_total{labelname="val2",basename="basevalue"} 0.23 1.23456789e+06
`,
		},
	}

	for i, scenario := range scenarios {
		out := bytes.NewBuffer(make([]byte, 0, len(scenario.out)))
		n, err := MetricFamilyToOpenMetrics(out, scenario.in, scenario.options...)
		if err != nil {
			t.Errorf("%d. error: %s", i, err)
			continue
		}
		if expected, got := len(scenario.out), n; expected != got {
			t.Errorf(
				"%d. expected %d bytes written, got %d",
				i, expected, got,
			)
		}
		if expected, got := scenario.out, out.String(); expected != got {
			t.Errorf(
				"%d. expected out=%q, got %q",
				i, expected, got,
			)
		}
	}
}

func BenchmarkOpenMetricsCreate(b *testing.B) {
	mf := &dto.MetricFamily{
		Name: proto.String("request_duration_microseconds"),
		Help: proto.String("The response latency."),
		Type: dto.MetricType_HISTOGRAM.Enum(),
		Metric: []*dto.Metric{
			{
				Label: []*dto.LabelPair{
					{
						Name:  proto.String("name_1"),
						Value: proto.String("val with\nnew line"),
					},
					{
						Name:  proto.String("name_2"),
						Value: proto.String("val with \\backslash and \"quotes\""),
					},
					{
						Name:  proto.String("name_3"),
						Value: proto.String("Just a quite long label value to test performance."),
					},
				},
				Histogram: &dto.Histogram{
					SampleCount: proto.Uint64(2693),
					SampleSum:   proto.Float64(1756047.3),
					Bucket: []*dto.Bucket{
						{
							UpperBound:      proto.Float64(100),
							CumulativeCount: proto.Uint64(123),
						},
						{
							UpperBound:      proto.Float64(120),
							CumulativeCount: proto.Uint64(412),
						},
						{
							UpperBound:      proto.Float64(144),
							CumulativeCount: proto.Uint64(592),
						},
						{
							UpperBound:      proto.Float64(172.8),
							CumulativeCount: proto.Uint64(1524),
						},
						{
							UpperBound:      proto.Float64(math.Inf(+1)),
							CumulativeCount: proto.Uint64(2693),
						},
					},
				},
			},
			{
				Label: []*dto.LabelPair{
					{
						Name:  proto.String("name_1"),
						Value: proto.String("Björn"),
					},
					{
						Name:  proto.String("name_2"),
						Value: proto.String("佖佥"),
					},
					{
						Name:  proto.String("name_3"),
						Value: proto.String("Just a quite long label value to test performance."),
					},
				},
				Histogram: &dto.Histogram{
					SampleCount: proto.Uint64(5699),
					SampleSum:   proto.Float64(49484343543.4343),
					Bucket: []*dto.Bucket{
						{
							UpperBound:      proto.Float64(100),
							CumulativeCount: proto.Uint64(120),
						},
						{
							UpperBound:      proto.Float64(120),
							CumulativeCount: proto.Uint64(412),
						},
						{
							UpperBound:      proto.Float64(144),
							CumulativeCount: proto.Uint64(596),
						},
						{
							UpperBound:      proto.Float64(172.8),
							CumulativeCount: proto.Uint64(1535),
						},
					},
				},
				TimestampMs: proto.Int64(1234567890),
			},
		},
	}
	out := bytes.NewBuffer(make([]byte, 0, 1024))

	for i := 0; i < b.N; i++ {
		_, err := MetricFamilyToOpenMetrics(out, mf)
		if err != nil {
			b.Fatal(err)
		}
		out.Reset()
	}
}

func TestOpenMetricsCreateError(t *testing.T) {
	scenarios := []struct {
		in  *dto.MetricFamily
		err string
	}{
		// 0: No metric name.
		{
			in: &dto.MetricFamily{
				Help: proto.String("doc string"),
				Type: dto.MetricType_UNTYPED.Enum(),
				Metric: []*dto.Metric{
					{
						Untyped: &dto.Untyped{
							Value: proto.Float64(math.Inf(-1)),
						},
					},
				},
			},
			err: "MetricFamily has no name",
		},
		// 1: Wrong type.
		{
			in: &dto.MetricFamily{
				Name: proto.String("name"),
				Help: proto.String("doc string"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Untyped: &dto.Untyped{
							Value: proto.Float64(math.Inf(-1)),
						},
					},
				},
			},
			err: "expected counter in metric",
		},
	}

	for i, scenario := range scenarios {
		var out bytes.Buffer
		_, err := MetricFamilyToOpenMetrics(&out, scenario.in)
		if err == nil {
			t.Errorf("%d. expected error, got nil", i)
			continue
		}
		if expected, got := scenario.err, err.Error(); strings.Index(got, expected) != 0 {
			t.Errorf(
				"%d. expected error starting with %q, got %q",
				i, expected, got,
			)
		}
	}
}
