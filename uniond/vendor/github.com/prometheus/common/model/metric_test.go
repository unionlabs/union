// Copyright 2013 The Prometheus Authors
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

package model

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/proto"
)

func testMetric(t testing.TB) {
	scenarios := []struct {
		input           LabelSet
		fingerprint     Fingerprint
		fastFingerprint Fingerprint
	}{
		{
			input:           LabelSet{},
			fingerprint:     14695981039346656037,
			fastFingerprint: 14695981039346656037,
		},
		{
			input: LabelSet{
				"first_name":   "electro",
				"occupation":   "robot",
				"manufacturer": "westinghouse",
			},
			fingerprint:     5911716720268894962,
			fastFingerprint: 11310079640881077873,
		},
		{
			input: LabelSet{
				"x": "y",
			},
			fingerprint:     8241431561484471700,
			fastFingerprint: 13948396922932177635,
		},
		{
			input: LabelSet{
				"a": "bb",
				"b": "c",
			},
			fingerprint:     3016285359649981711,
			fastFingerprint: 3198632812309449502,
		},
		{
			input: LabelSet{
				"a":  "b",
				"bb": "c",
			},
			fingerprint:     7122421792099404749,
			fastFingerprint: 5774953389407657638,
		},
	}

	for i, scenario := range scenarios {
		input := Metric(scenario.input)

		if scenario.fingerprint != input.Fingerprint() {
			t.Errorf("%d. expected %d, got %d", i, scenario.fingerprint, input.Fingerprint())
		}
		if scenario.fastFingerprint != input.FastFingerprint() {
			t.Errorf("%d. expected %d, got %d", i, scenario.fastFingerprint, input.FastFingerprint())
		}
	}
}

func TestMetric(t *testing.T) {
	testMetric(t)
}

func BenchmarkMetric(b *testing.B) {
	for i := 0; i < b.N; i++ {
		testMetric(b)
	}
}

func TestMetricNameIsLegacyValid(t *testing.T) {
	scenarios := []struct {
		mn          LabelValue
		legacyValid bool
		utf8Valid   bool
	}{
		{
			mn:          "Avalid_23name",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "_Avalid_23name",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "1valid_23name",
			legacyValid: false,
			utf8Valid:   true,
		},
		{
			mn:          "avalid_23name",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "Ava:lid_23name",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "a lid_23name",
			legacyValid: false,
			utf8Valid:   true,
		},
		{
			mn:          ":leading_colon",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "colon:in:the:middle",
			legacyValid: true,
			utf8Valid:   true,
		},
		{
			mn:          "",
			legacyValid: false,
			utf8Valid:   false,
		},
		{
			mn:          "a\xc5z",
			legacyValid: false,
			utf8Valid:   false,
		},
	}

	for _, s := range scenarios {
		NameValidationScheme = LegacyValidation
		if IsValidMetricName(s.mn) != s.legacyValid {
			t.Errorf("Expected %v for %q using legacy IsValidMetricName method", s.legacyValid, s.mn)
		}
		if MetricNameRE.MatchString(string(s.mn)) != s.legacyValid {
			t.Errorf("Expected %v for %q using regexp matching", s.legacyValid, s.mn)
		}
		NameValidationScheme = UTF8Validation
		if IsValidMetricName(s.mn) != s.utf8Valid {
			t.Errorf("Expected %v for %q using utf-8 IsValidMetricName method", s.legacyValid, s.mn)
		}
	}
}

func TestMetricClone(t *testing.T) {
	m := Metric{
		"first_name":   "electro",
		"occupation":   "robot",
		"manufacturer": "westinghouse",
	}

	m2 := m.Clone()

	if len(m) != len(m2) {
		t.Errorf("expected the length of the cloned metric to be equal to the input metric")
	}

	for ln, lv := range m2 {
		expected := m[ln]
		if expected != lv {
			t.Errorf("expected label value %s but got %s for label name %s", expected, lv, ln)
		}
	}
}

func TestMetricToString(t *testing.T) {
	scenarios := []struct {
		name     string
		input    Metric
		expected string
	}{
		{
			name: "valid metric without __name__ label",
			input: Metric{
				"first_name":   "electro",
				"occupation":   "robot",
				"manufacturer": "westinghouse",
			},
			expected: `{first_name="electro", manufacturer="westinghouse", occupation="robot"}`,
		},
		{
			name: "valid metric with __name__ label",
			input: Metric{
				"__name__":     "electro",
				"occupation":   "robot",
				"manufacturer": "westinghouse",
			},
			expected: `electro{manufacturer="westinghouse", occupation="robot"}`,
		},
		{
			name: "empty metric with __name__ label",
			input: Metric{
				"__name__": "fooname",
			},
			expected: "fooname",
		},
		{
			name:     "empty metric",
			input:    Metric{},
			expected: "{}",
		},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			actual := scenario.input.String()
			if actual != scenario.expected {
				t.Errorf("expected string output %s but got %s", scenario.expected, actual)
			}
		})
	}
}

func TestEscapeName(t *testing.T) {
	scenarios := []struct {
		name                  string
		input                 string
		expectedUnderscores   string
		expectedDots          string
		expectedUnescapedDots string
		expectedValue         string
	}{
		{
			name: "empty string",
		},
		{
			name:                "legacy valid name",
			input:               "no:escaping_required",
			expectedUnderscores: "no:escaping_required",
			// Dots escaping will escape underscores even though it's not strictly
			// necessary for compatibility.
			expectedDots:          "no:escaping__required",
			expectedUnescapedDots: "no:escaping_required",
			expectedValue:         "no:escaping_required",
		},
		{
			name:                  "name with dots",
			input:                 "mysystem.prod.west.cpu.load",
			expectedUnderscores:   "mysystem_prod_west_cpu_load",
			expectedDots:          "mysystem_dot_prod_dot_west_dot_cpu_dot_load",
			expectedUnescapedDots: "mysystem.prod.west.cpu.load",
			expectedValue:         "U__mysystem_2e_prod_2e_west_2e_cpu_2e_load",
		},
		{
			name:                  "name with dots and colon",
			input:                 "http.status:sum",
			expectedUnderscores:   "http_status:sum",
			expectedDots:          "http_dot_status:sum",
			expectedUnescapedDots: "http.status:sum",
			expectedValue:         "U__http_2e_status:sum",
		},
		{
			name:                "name with unicode characters > 0x100",
			input:               "花火",
			expectedUnderscores: "__",
			expectedDots:        "__",
			// Dots-replacement does not know the difference between two replaced
			// characters and a single underscore.
			expectedUnescapedDots: "_",
			expectedValue:         "U___82b1__706b_",
		},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			got := EscapeName(scenario.input, UnderscoreEscaping)
			if got != scenario.expectedUnderscores {
				t.Errorf("expected string output %s but got %s", scenario.expectedUnderscores, got)
			}
			// Unescaping with the underscore method is a noop.
			got = UnescapeName(got, UnderscoreEscaping)
			if got != scenario.expectedUnderscores {
				t.Errorf("expected unescaped string output %s but got %s", scenario.expectedUnderscores, got)
			}

			got = EscapeName(scenario.input, DotsEscaping)
			if got != scenario.expectedDots {
				t.Errorf("expected string output %s but got %s", scenario.expectedDots, got)
			}
			got = UnescapeName(got, DotsEscaping)
			if got != scenario.expectedUnescapedDots {
				t.Errorf("expected unescaped string output %s but got %s", scenario.expectedUnescapedDots, got)
			}

			got = EscapeName(scenario.input, ValueEncodingEscaping)
			if got != scenario.expectedValue {
				t.Errorf("expected string output %s but got %s", scenario.expectedValue, got)
			}
			// Unescaped result should always be identical to the original input.
			got = UnescapeName(got, ValueEncodingEscaping)
			if got != scenario.input {
				t.Errorf("expected unescaped string output %s but got %s", scenario.input, got)
			}
		})
	}
}

func TestValueUnescapeErrors(t *testing.T) {
	scenarios := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name: "empty string",
		},
		{
			name:     "basic case, no error",
			input:    "U__no:unescapingrequired",
			expected: "no:unescapingrequired",
		},
		{
			name:     "capitals ok, no error",
			input:    "U__capitals_2E_ok",
			expected: "capitals.ok",
		},
		{
			name:     "underscores, no error",
			input:    "U__underscores__doubled__",
			expected: "underscores_doubled_",
		},
		{
			name:     "invalid single underscore",
			input:    "U__underscores_doubled_",
			expected: "U__underscores_doubled_",
		},
		{
			name:     "invalid single underscore, 2",
			input:    "U__underscores__doubled_",
			expected: "U__underscores__doubled_",
		},
		{
			name:     "giant fake utf-8 code",
			input:    "U__my__hack_2e_attempt_872348732fabdabbab_",
			expected: "U__my__hack_2e_attempt_872348732fabdabbab_",
		},
		{
			name:     "trailing utf-8",
			input:    "U__my__hack_2e",
			expected: "U__my__hack_2e",
		},
		{
			name:     "invalid utf-8 value",
			input:    "U__bad__utf_2eg_",
			expected: "U__bad__utf_2eg_",
		},
		{
			name:     "surrogate utf-8 value",
			input:    "U__bad__utf_D900_",
			expected: "U__bad__utf_D900_",
		},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			got := UnescapeName(scenario.input, ValueEncodingEscaping)
			if got != scenario.expected {
				t.Errorf("expected unescaped string output %s but got %s", scenario.expected, got)
			}
		})
	}
}

func TestEscapeMetricFamily(t *testing.T) {
	scenarios := []struct {
		name     string
		input    *dto.MetricFamily
		scheme   EscapingScheme
		expected *dto.MetricFamily
	}{
		{
			name:     "empty",
			input:    &dto.MetricFamily{},
			scheme:   ValueEncodingEscaping,
			expected: &dto.MetricFamily{},
		},
		{
			name:   "simple, no escaping needed",
			scheme: ValueEncodingEscaping,
			input: &dto.MetricFamily{
				Name: proto.String("my_metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("my_metric"),
							},
							{
								Name:  proto.String("some_label"),
								Value: proto.String("labelvalue"),
							},
						},
					},
				},
			},
			expected: &dto.MetricFamily{
				Name: proto.String("my_metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("my_metric"),
							},
							{
								Name:  proto.String("some_label"),
								Value: proto.String("labelvalue"),
							},
						},
					},
				},
			},
		},
		{
			name:   "label name escaping needed",
			scheme: ValueEncodingEscaping,
			input: &dto.MetricFamily{
				Name: proto.String("my_metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("my_metric"),
							},
							{
								Name:  proto.String("some.label"),
								Value: proto.String("labelvalue"),
							},
						},
					},
				},
			},
			expected: &dto.MetricFamily{
				Name: proto.String("my_metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("my_metric"),
							},
							{
								Name:  proto.String("U__some_2e_label"),
								Value: proto.String("labelvalue"),
							},
						},
					},
				},
			},
		},
		{
			name:   "counter, escaping needed",
			scheme: ValueEncodingEscaping,
			input: &dto.MetricFamily{
				Name: proto.String("my.metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("my.metric"),
							},
							{
								Name:  proto.String("some?label"),
								Value: proto.String("label??value"),
							},
						},
					},
				},
			},
			expected: &dto.MetricFamily{
				Name: proto.String("U__my_2e_metric"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_COUNTER.Enum(),
				Metric: []*dto.Metric{
					{
						Counter: &dto.Counter{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("U__my_2e_metric"),
							},
							{
								Name:  proto.String("U__some_3f_label"),
								Value: proto.String("label??value"),
							},
						},
					},
				},
			},
		},
		{
			name:   "gauge, escaping needed",
			scheme: DotsEscaping,
			input: &dto.MetricFamily{
				Name: proto.String("unicode.and.dots.花火"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_GAUGE.Enum(),
				Metric: []*dto.Metric{
					{
						Gauge: &dto.Gauge{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("unicode.and.dots.花火"),
							},
							{
								Name:  proto.String("some_label"),
								Value: proto.String("label??value"),
							},
						},
					},
				},
			},
			expected: &dto.MetricFamily{
				Name: proto.String("unicode_dot_and_dot_dots_dot___"),
				Help: proto.String("some help text"),
				Type: dto.MetricType_GAUGE.Enum(),
				Metric: []*dto.Metric{
					{
						Gauge: &dto.Gauge{
							Value: proto.Float64(34.2),
						},
						Label: []*dto.LabelPair{
							{
								Name:  proto.String("__name__"),
								Value: proto.String("unicode_dot_and_dot_dots_dot___"),
							},
							{
								Name:  proto.String("some_label"),
								Value: proto.String("label??value"),
							},
						},
					},
				},
			},
		},
	}

	unexportList := []interface{}{dto.MetricFamily{}, dto.Metric{}, dto.LabelPair{}, dto.Counter{}, dto.Gauge{}}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			original := proto.Clone(scenario.input)
			got := EscapeMetricFamily(scenario.input, scenario.scheme)
			if !cmp.Equal(scenario.expected, got, cmpopts.IgnoreUnexported(unexportList...)) {
				t.Errorf("unexpected difference in escaped output:\n%s", cmp.Diff(scenario.expected, got, cmpopts.IgnoreUnexported(unexportList...)))
			}
			if !cmp.Equal(scenario.input, original, cmpopts.IgnoreUnexported(unexportList...)) {
				t.Errorf("input was mutated during escaping:\n%s", cmp.Diff(scenario.expected, got, cmpopts.IgnoreUnexported(unexportList...)))
			}
		})
	}
}

// TestProtoFormatUnchanged checks to see if the proto format changed, in which
// case EscapeMetricFamily will need to be updated.
func TestProtoFormatUnchanged(t *testing.T) {
	scenarios := []struct {
		name         string
		input        proto.Message
		expectFields []string
	}{
		{
			name:         "MetricFamily",
			input:        &dto.MetricFamily{},
			expectFields: []string{"name", "help", "type", "metric", "unit"},
		},
		{
			name:         "Metric",
			input:        &dto.Metric{},
			expectFields: []string{"label", "gauge", "counter", "summary", "untyped", "histogram", "timestamp_ms"},
		},
		{
			name:         "LabelPair",
			input:        &dto.LabelPair{},
			expectFields: []string{"name", "value"},
		},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, func(t *testing.T) {
			desc := scenario.input.ProtoReflect().Descriptor()
			fields := desc.Fields()
			if fields.Len() != len(scenario.expectFields) {
				t.Errorf("dto.MetricFamily changed length, expected %d, got %d", len(scenario.expectFields), fields.Len())
			}

			for i := 0; i < fields.Len(); i++ {
				got := fields.Get(i).TextName()
				if got != scenario.expectFields[i] {
					t.Errorf("dto.MetricFamily field mismatch, expected %s got %s", scenario.expectFields[i], got)
				}
			}
		})
	}
}
