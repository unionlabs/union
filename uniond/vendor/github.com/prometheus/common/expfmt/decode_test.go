// Copyright 2015 The Prometheus Authors
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
	"bufio"
	"bytes"
	"errors"
	"io"
	"math"
	"net/http"
	"os"
	"reflect"
	"sort"
	"strings"
	"testing"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/proto"

	"github.com/prometheus/common/model"
)

func TestTextDecoder(t *testing.T) {
	var (
		ts = model.Now()
		in = `
# Only a quite simple scenario with two metric families.
# More complicated tests of the parser itself can be found in the text package.
# TYPE mf2 counter
mf2 3
mf1{label="value1"} -3.14 123456
mf1{label="value2"} 42
mf2 4
`
		out = model.Vector{
			&model.Sample{
				Metric: model.Metric{
					model.MetricNameLabel: "mf1",
					"label":               "value1",
				},
				Value:     -3.14,
				Timestamp: 123456,
			},
			&model.Sample{
				Metric: model.Metric{
					model.MetricNameLabel: "mf1",
					"label":               "value2",
				},
				Value:     42,
				Timestamp: ts,
			},
			&model.Sample{
				Metric: model.Metric{
					model.MetricNameLabel: "mf2",
				},
				Value:     3,
				Timestamp: ts,
			},
			&model.Sample{
				Metric: model.Metric{
					model.MetricNameLabel: "mf2",
				},
				Value:     4,
				Timestamp: ts,
			},
		}
	)

	dec := &SampleDecoder{
		Dec: &textDecoder{r: strings.NewReader(in)},
		Opts: &DecodeOptions{
			Timestamp: ts,
		},
	}
	var all model.Vector
	for {
		var smpls model.Vector
		err := dec.Decode(&smpls)
		if err != nil && errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			t.Fatal(err)
		}
		all = append(all, smpls...)
	}
	sort.Sort(all)
	sort.Sort(out)
	if !reflect.DeepEqual(all, out) {
		t.Fatalf("output does not match")
	}
}

func TestProtoDecoder(t *testing.T) {
	testTime := model.Now()

	scenarios := []struct {
		in             string
		expected       model.Vector
		legacyNameFail bool
		fail           bool
	}{
		{
			in: "",
		},
		{
			in:   "\x8f\x01\n\rrequest_count\x12\x12Number of requests\x18\x00\"0\n#\n\x0fsome_!abel_name\x12\x10some_label_value\x1a\t\t\x00\x00\x00\x00\x00\x00E\xc0\"6\n)\n\x12another_label_name\x12\x13another_label_value\x1a\t\t\x00\x00\x00\x00\x00\x00U@",
			fail: true,
		},
		{
			in: "\x8f\x01\n\rrequest_count\x12\x12Number of requests\x18\x00\"0\n#\n\x0fsome_label_name\x12\x10some_label_value\x1a\t\t\x00\x00\x00\x00\x00\x00E\xc0\"6\n)\n\x12another_label_name\x12\x13another_label_value\x1a\t\t\x00\x00\x00\x00\x00\x00U@",
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
						"some_label_name":     "some_label_value",
					},
					Value:     -42,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
						"another_label_name":  "another_label_value",
					},
					Value:     84,
					Timestamp: testTime,
				},
			},
		},
		{
			in: "\xb9\x01\n\rrequest_count\x12\x12Number of requests\x18\x02\"O\n#\n\x0fsome_label_name\x12\x10some_label_value\"(\x1a\x12\t\xaeG\xe1z\x14\xae\xef?\x11\x00\x00\x00\x00\x00\x00E\xc0\x1a\x12\t+\x87\x16\xd9\xce\xf7\xef?\x11\x00\x00\x00\x00\x00\x00U\xc0\"A\n)\n\x12another_label_name\x12\x13another_label_value\"\x14\x1a\x12\t\x00\x00\x00\x00\x00\x00\xe0?\x11\x00\x00\x00\x00\x00\x00$@",
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count_count",
						"some_label_name":     "some_label_value",
					},
					Value:     0,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count_sum",
						"some_label_name":     "some_label_value",
					},
					Value:     0,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
						"some_label_name":     "some_label_value",
						"quantile":            "0.99",
					},
					Value:     -42,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
						"some_label_name":     "some_label_value",
						"quantile":            "0.999",
					},
					Value:     -84,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count_count",
						"another_label_name":  "another_label_value",
					},
					Value:     0,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count_sum",
						"another_label_name":  "another_label_value",
					},
					Value:     0,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
						"another_label_name":  "another_label_value",
						"quantile":            "0.5",
					},
					Value:     10,
					Timestamp: testTime,
				},
			},
		},
		{
			in: "\x8d\x01\n\x1drequest_duration_microseconds\x12\x15The response latency.\x18\x04\"S:Q\b\x85\x15\x11\xcd\xcc\xccL\x8f\xcb:A\x1a\v\b{\x11\x00\x00\x00\x00\x00\x00Y@\x1a\f\b\x9c\x03\x11\x00\x00\x00\x00\x00\x00^@\x1a\f\b\xd0\x04\x11\x00\x00\x00\x00\x00\x00b@\x1a\f\b\xf4\v\x11\x9a\x99\x99\x99\x99\x99e@\x1a\f\b\x85\x15\x11\x00\x00\x00\x00\x00\x00\xf0\u007f",
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_bucket",
						"le":                  "100",
					},
					Value:     123,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_bucket",
						"le":                  "120",
					},
					Value:     412,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_bucket",
						"le":                  "144",
					},
					Value:     592,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_bucket",
						"le":                  "172.8",
					},
					Value:     1524,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_bucket",
						"le":                  "+Inf",
					},
					Value:     2693,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_sum",
					},
					Value:     1756047.3,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_count",
					},
					Value:     2693,
					Timestamp: testTime,
				},
			},
		},
		{
			in: "\u007f\n\x1drequest_duration_microseconds\x12\x15The response latency.\x18\x04\"E:C\b\x85\x15\x11\xcd\xcc\xccL\x8f\xcb:A\x1a\v\b{\x11\x00\x00\x00\x00\x00\x00Y@\x1a\f\b\x9c\x03\x11\x00\x00\x00\x00\x00\x00^@\x1a\f\b\xd0\x04\x11\x00\x00\x00\x00\x00\x00b@\x1a\f\b\xf4\v\x11\x9a\x99\x99\x99\x99\x99e@",
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_count",
					},
					Value:     2693,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						"le":                  "+Inf",
						model.MetricNameLabel: "request_duration_microseconds_bucket",
					},
					Value:     2693,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_duration_microseconds_sum",
					},
					Value:     1756047.3,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						"le":                  "172.8",
						model.MetricNameLabel: "request_duration_microseconds_bucket",
					},
					Value:     1524,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						"le":                  "144",
						model.MetricNameLabel: "request_duration_microseconds_bucket",
					},
					Value:     592,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						"le":                  "120",
						model.MetricNameLabel: "request_duration_microseconds_bucket",
					},
					Value:     412,
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						"le":                  "100",
						model.MetricNameLabel: "request_duration_microseconds_bucket",
					},
					Value:     123,
					Timestamp: testTime,
				},
			},
		},
		{
			// The metric type is unset in this protobuf, which needs to be handled
			// correctly by the decoder.
			in: "\x1c\n\rrequest_count\"\v\x1a\t\t\x00\x00\x00\x00\x00\x00\xf0?",
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "request_count",
					},
					Value:     1,
					Timestamp: testTime,
				},
			},
		},
		{
			in:             "\xa8\x01\n\ngauge.name\x12\x11gauge\ndoc\nstr\"ing\x18\x01\"T\n\x1b\n\x06name.1\x12\x11val with\nnew line\n*\n\x06name*2\x12 val with \\backslash and \"quotes\"\x12\t\t\x00\x00\x00\x00\x00\x00\xf0\x7f\"/\n\x10\n\x06name.1\x12\x06Björn\n\x10\n\x06name*2\x12\x06佖佥\x12\t\t\xd1\xcfD\xb9\xd0\x05\xc2H",
			legacyNameFail: true,
			expected: model.Vector{
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "gauge.name",
						"name.1":              "val with\nnew line",
						"name*2":              "val with \\backslash and \"quotes\"",
					},
					Value:     model.SampleValue(math.Inf(+1)),
					Timestamp: testTime,
				},
				&model.Sample{
					Metric: model.Metric{
						model.MetricNameLabel: "gauge.name",
						"name.1":              "Björn",
						"name*2":              "佖佥",
					},
					Value:     3.14e42,
					Timestamp: testTime,
				},
			},
		},
	}

	for i, scenario := range scenarios {
		dec := &SampleDecoder{
			Dec: &protoDecoder{r: strings.NewReader(scenario.in)},
			Opts: &DecodeOptions{
				Timestamp: testTime,
			},
		}

		var all model.Vector
		for {
			model.NameValidationScheme = model.LegacyValidation
			var smpls model.Vector
			err := dec.Decode(&smpls)
			if err != nil && errors.Is(err, io.EOF) {
				break
			}
			if scenario.legacyNameFail {
				if err == nil {
					t.Fatal("Expected error when decoding without UTF-8 support enabled but got none")
				}
				model.NameValidationScheme = model.UTF8Validation
				dec = &SampleDecoder{
					Dec: &protoDecoder{r: strings.NewReader(scenario.in)},
					Opts: &DecodeOptions{
						Timestamp: testTime,
					},
				}
				err = dec.Decode(&smpls)
				if errors.Is(err, io.EOF) {
					break
				}
				if err != nil {
					t.Fatalf("Unexpected error when decoding with UTF-8 support: %v", err)
				}
			}
			if scenario.fail {
				if err == nil {
					t.Fatal("Expected error but got none")
				}
				break
			}
			if err != nil {
				t.Fatal(err)
			}
			all = append(all, smpls...)
		}
		sort.Sort(all)
		sort.Sort(scenario.expected)
		if !reflect.DeepEqual(all, scenario.expected) {
			t.Fatalf("%d. output does not match, want: %#v, got %#v", i, scenario.expected, all)
		}
	}
}

func TestProtoMultiMessageDecoder(t *testing.T) {
	data, err := os.ReadFile("testdata/protobuf-multimessage")
	if err != nil {
		t.Fatalf("Reading file failed: %v", err)
	}

	buf := bytes.NewReader(data)
	decoder := NewDecoder(buf, FmtProtoDelim)
	var metrics []*dto.MetricFamily
	for {
		var mf dto.MetricFamily
		if err := decoder.Decode(&mf); err != nil {
			if errors.Is(err, io.EOF) {
				break
			}
			t.Fatalf("Unmarshalling failed: %v", err)
		}
		metrics = append(metrics, &mf)
	}

	if len(metrics) != 6 {
		t.Fatalf("Expected %d metrics but got %d!", 6, len(metrics))
	}
}

func testDiscriminatorHTTPHeader(t testing.TB) {
	scenarios := []struct {
		input  map[string]string
		output Format
	}{
		{
			input:  map[string]string{"Content-Type": `application/vnd.google.protobuf; proto="io.prometheus.client.MetricFamily"; encoding="delimited"`},
			output: FmtProtoDelim,
		},
		{
			input:  map[string]string{"Content-Type": `application/vnd.google.protobuf; proto="illegal"; encoding="delimited"`},
			output: FmtUnknown,
		},
		{
			input:  map[string]string{"Content-Type": `application/vnd.google.protobuf; proto="io.prometheus.client.MetricFamily"; encoding="illegal"`},
			output: FmtUnknown,
		},
		{
			input:  map[string]string{"Content-Type": `text/plain; version=0.0.4`},
			output: FmtText,
		},
		{
			input:  map[string]string{"Content-Type": `text/plain`},
			output: FmtText,
		},
		{
			input:  map[string]string{"Content-Type": `text/plain; version=0.0.3`},
			output: FmtUnknown,
		},
	}

	for i, scenario := range scenarios {
		var header http.Header

		if len(scenario.input) > 0 {
			header = http.Header{}
		}

		for key, value := range scenario.input {
			header.Add(key, value)
		}

		actual := ResponseFormat(header)

		if scenario.output != actual {
			t.Errorf("%d. expected %s, got %s", i, scenario.output, actual)
		}
	}
}

func TestDiscriminatorHTTPHeader(t *testing.T) {
	testDiscriminatorHTTPHeader(t)
}

func BenchmarkDiscriminatorHTTPHeader(b *testing.B) {
	for i := 0; i < b.N; i++ {
		testDiscriminatorHTTPHeader(b)
	}
}

func TestExtractSamples(t *testing.T) {
	var (
		goodMetricFamily1 = &dto.MetricFamily{
			Name: proto.String("foo"),
			Help: proto.String("Help for foo."),
			Type: dto.MetricType_COUNTER.Enum(),
			Metric: []*dto.Metric{
				{
					Counter: &dto.Counter{
						Value: proto.Float64(4711),
					},
				},
			},
		}
		goodMetricFamily2 = &dto.MetricFamily{
			Name: proto.String("bar"),
			Help: proto.String("Help for bar."),
			Type: dto.MetricType_GAUGE.Enum(),
			Metric: []*dto.Metric{
				{
					Gauge: &dto.Gauge{
						Value: proto.Float64(3.14),
					},
				},
			},
		}
		badMetricFamily = &dto.MetricFamily{
			Name: proto.String("bad"),
			Help: proto.String("Help for bad."),
			Type: dto.MetricType(42).Enum(),
			Metric: []*dto.Metric{
				{
					Gauge: &dto.Gauge{
						Value: proto.Float64(2.7),
					},
				},
			},
		}

		opts = &DecodeOptions{
			Timestamp: 42,
		}
	)

	got, err := ExtractSamples(opts, goodMetricFamily1, goodMetricFamily2)
	if err != nil {
		t.Error("Unexpected error from ExtractSamples:", err)
	}
	want := model.Vector{
		&model.Sample{Metric: model.Metric{model.MetricNameLabel: "foo"}, Value: 4711, Timestamp: 42},
		&model.Sample{Metric: model.Metric{model.MetricNameLabel: "bar"}, Value: 3.14, Timestamp: 42},
	}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("unexpected samples extracted, got: %v, want: %v", got, want)
	}

	got, err = ExtractSamples(opts, goodMetricFamily1, badMetricFamily, goodMetricFamily2)
	if err == nil {
		t.Error("Expected error from ExtractSamples")
	}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("unexpected samples extracted, got: %v, want: %v", got, want)
	}
}

func TestTextDecoderWithBufioReader(t *testing.T) {
	example := `
	# TYPE foo gauge
	foo 0
	`

	var decoded bool
	r := bufio.NewReader(strings.NewReader(example))
	dec := NewDecoder(r, FmtText)
	for {
		var mf dto.MetricFamily
		if err := dec.Decode(&mf); err != nil {
			if errors.Is(err, io.EOF) {
				break
			}
			t.Fatalf("Unexpected error: %v", err)
		}
		if mf.GetName() != "foo" {
			t.Errorf("Unexpected metric name: got %v, expected %v", mf.GetName(), "foo")
		}
		if len(mf.Metric) != 1 {
			t.Errorf("Unexpected number of metrics: got %v, expected %v", len(mf.Metric), 1)
		}
		decoded = true
	}
	if !decoded {
		t.Fatal("Metric foo not decoded")
	}
}
