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
	"encoding/json"
	"reflect"
	"regexp"
	"testing"
)

var (
	noWhitespace = regexp.MustCompile(`\s`)

	sampleHistogramPairMatrixPlain = `[
		{
			"metric":{
				"__name__":"test_metric"
			},
			"histograms":[
				[
					1234.567,
					{
					"count":"6",
					"sum":"3897",
					"buckets":[
						[
							1,
							"-4870.992343051145",
							"-4466.7196729968955",
							"1"
						],
						[
							1,
							"-861.0779292198035",
							"-789.6119426088657",
							"1"
						],
						[
							1,
							"-558.3399591246119",
							"-512",
							"1"
						],
						[
							0,
							"2048",
							"2233.3598364984477",
							"1"
						],
						[
							0,
							"2896.3093757400984",
							"3158.4477704354626",
							"1"
						],
						[
							0,
							"4466.7196729968955",
							"4870.992343051145",
							"1"
						]
					]
					}
				],
				[
					12345.678,
					{
					"count":"6",
					"sum":"3897",
					"buckets":[
						[
							1,
							"-4870.992343051145",
							"-4466.7196729968955",
							"1"
						],
						[
							1,
							"-861.0779292198035",
							"-789.6119426088657",
							"1"
						],
						[
							1,
							"-558.3399591246119",
							"-512",
							"1"
						],
						[
							0,
							"2048",
							"2233.3598364984477",
							"1"
						],
						[
							0,
							"2896.3093757400984",
							"3158.4477704354626",
							"1"
						],
						[
							0,
							"4466.7196729968955",
							"4870.992343051145",
							"1"
						]
					]
					}
				]
			]
		},
		{
			"metric":{
				"foo":"bar"
			},
			"histograms":[
				[
					2234.567,
					{
					"count":"6",
					"sum":"3897",
					"buckets":[
						[
							1,
							"-4870.992343051145",
							"-4466.7196729968955",
							"1"
						],
						[
							1,
							"-861.0779292198035",
							"-789.6119426088657",
							"1"
						],
						[
							1,
							"-558.3399591246119",
							"-512",
							"1"
						],
						[
							0,
							"2048",
							"2233.3598364984477",
							"1"
						],
						[
							0,
							"2896.3093757400984",
							"3158.4477704354626",
							"1"
						],
						[
							0,
							"4466.7196729968955",
							"4870.992343051145",
							"1"
						]
					]
					}
				],
				[
					22345.678,
					{
					"count":"6",
					"sum":"3897",
					"buckets":[
						[
							1,
							"-4870.992343051145",
							"-4466.7196729968955",
							"1"
						],
						[
							1,
							"-861.0779292198035",
							"-789.6119426088657",
							"1"
						],
						[
							1,
							"-558.3399591246119",
							"-512",
							"1"
						],
						[
							0,
							"2048",
							"2233.3598364984477",
							"1"
						],
						[
							0,
							"2896.3093757400984",
							"3158.4477704354626",
							"1"
						],
						[
							0,
							"4466.7196729968955",
							"4870.992343051145",
							"1"
						]
					]
					}
				]
			]
		}
	]`
	sampleHistogramPairMatrixValue = Matrix{
		&SampleStream{
			Metric: Metric{
				MetricNameLabel: "test_metric",
			},
			Histograms: []SampleHistogramPair{
				{
					Histogram: genSampleHistogram(),
					Timestamp: 1234567,
				},
				{
					Histogram: genSampleHistogram(),
					Timestamp: 12345678,
				},
			},
		},
		&SampleStream{
			Metric: Metric{
				"foo": "bar",
			},
			Histograms: []SampleHistogramPair{
				{
					Histogram: genSampleHistogram(),
					Timestamp: 2234567,
				},
				{
					Histogram: genSampleHistogram(),
					Timestamp: 22345678,
				},
			},
		},
	}
)

func genSampleHistogram() *SampleHistogram {
	return &SampleHistogram{
		Count: 6,
		Sum:   3897,
		Buckets: HistogramBuckets{
			{
				Boundaries: 1,
				Lower:      -4870.992343051145,
				Upper:      -4466.7196729968955,
				Count:      1,
			},
			{
				Boundaries: 1,
				Lower:      -861.0779292198035,
				Upper:      -789.6119426088657,
				Count:      1,
			},
			{
				Boundaries: 1,
				Lower:      -558.3399591246119,
				Upper:      -512,
				Count:      1,
			},
			{
				Boundaries: 0,
				Lower:      2048,
				Upper:      2233.3598364984477,
				Count:      1,
			},
			{
				Boundaries: 0,
				Lower:      2896.3093757400984,
				Upper:      3158.4477704354626,
				Count:      1,
			},
			{
				Boundaries: 0,
				Lower:      4466.7196729968955,
				Upper:      4870.992343051145,
				Count:      1,
			},
		},
	}
}

func TestSampleHistogramPairJSON(t *testing.T) {
	input := []struct {
		plain string
		value SampleHistogramPair
	}{
		{
			plain: `[
				1234.567,
				{
					"count":"6",
					"sum":"3897",
					"buckets":[
						[
							1,
							"-4870.992343051145",
							"-4466.7196729968955",
							"1"
						],
						[
							1,
							"-861.0779292198035",
							"-789.6119426088657",
							"1"
						],
						[
							1,
							"-558.3399591246119",
							"-512",
							"1"
						],
						[
							0,
							"2048",
							"2233.3598364984477",
							"1"
						],
						[
							0,
							"2896.3093757400984",
							"3158.4477704354626",
							"1"
						],
						[
							0,
							"4466.7196729968955",
							"4870.992343051145",
							"1"
						]
					]
				}
			]`,
			value: SampleHistogramPair{
				Histogram: genSampleHistogram(),
				Timestamp: 1234567,
			},
		},
	}

	for _, test := range input {
		b, err := json.Marshal(test.value)
		if err != nil {
			t.Error(err)
			continue
		}

		trimmed := noWhitespace.ReplaceAllString(test.plain, "")
		if string(b) != trimmed {
			t.Errorf("encoding error: expected %q, got %q", trimmed, b)
			continue
		}

		var sp SampleHistogramPair
		err = json.Unmarshal(b, &sp)
		if err != nil {
			t.Error(err)
			continue
		}

		if !sp.Equal(&test.value) {
			t.Errorf("decoding error: expected %v, got %v", test.value, sp)
		}
	}
}

func TestInvalidSampleHistogramPairJSON(t *testing.T) {
	s1 := SampleHistogramPair{
		Timestamp: 1,
		Histogram: nil,
	}
	d, err := json.Marshal(s1)
	if err == nil {
		t.Errorf("expected error when trying to marshal invalid SampleHistogramPair %s", string(d))
	}

	var s2 SampleHistogramPair
	plain := "[0.001,null]"
	err = json.Unmarshal([]byte(plain), &s2)
	if err == nil {
		t.Errorf("expected error when trying to unmarshal invalid SampleHistogramPair %s", plain)
	}
}

func TestSampleHistogramJSON(t *testing.T) {
	input := []struct {
		plain string
		value Sample
	}{
		{
			plain: `{
				"metric":{
					"__name__":"test_metric"
				},
				"histogram":[
					1234.567,
					{
						"count":"6",
						"sum":"3897",
						"buckets":[
							[
								1,
								"-4870.992343051145",
								"-4466.7196729968955",
								"1"
							],
							[
								1,
								"-861.0779292198035",
								"-789.6119426088657",
								"1"
							],
							[
								1,
								"-558.3399591246119",
								"-512",
								"1"
							],
							[
								0,
								"2048",
								"2233.3598364984477",
								"1"
							],
							[
								0,
								"2896.3093757400984",
								"3158.4477704354626",
								"1"
							],
							[
								0,
								"4466.7196729968955",
								"4870.992343051145",
								"1"
							]
						]
					}
				]
			}`,
			value: Sample{
				Metric: Metric{
					MetricNameLabel: "test_metric",
				},
				Histogram: genSampleHistogram(),
				Timestamp: 1234567,
			},
		},
	}

	for _, test := range input {
		b, err := json.Marshal(test.value)
		if err != nil {
			t.Error(err)
			continue
		}

		trimmed := noWhitespace.ReplaceAllString(test.plain, "")
		if string(b) != trimmed {
			t.Errorf("encoding error: expected %q, got %q", trimmed, b)
			continue
		}

		var sv Sample
		err = json.Unmarshal(b, &sv)
		if err != nil {
			t.Error(err)
			continue
		}

		if !reflect.DeepEqual(sv, test.value) {
			t.Errorf("decoding error: expected %v, got %v", test.value, sv)
		}
	}
}

func TestVectorHistogramJSON(t *testing.T) {
	input := []struct {
		plain string
		value Vector
	}{
		{
			plain: `[
				{
					"metric":{
						"__name__":"test_metric"
					},
					"histogram":[
						1234.567,
						{
							"count":"6",
							"sum":"3897",
							"buckets":[
							[
								1,
								"-4870.992343051145",
								"-4466.7196729968955",
								"1"
							],
							[
								1,
								"-861.0779292198035",
								"-789.6119426088657",
								"1"
							],
							[
								1,
								"-558.3399591246119",
								"-512",
								"1"
							],
							[
								0,
								"2048",
								"2233.3598364984477",
								"1"
							],
							[
								0,
								"2896.3093757400984",
								"3158.4477704354626",
								"1"
							],
							[
								0,
								"4466.7196729968955",
								"4870.992343051145",
								"1"
							]
							]
						}
					]
				}
			]`,
			value: Vector{&Sample{
				Metric: Metric{
					MetricNameLabel: "test_metric",
				},
				Histogram: genSampleHistogram(),
				Timestamp: 1234567,
			}},
		},
		{
			plain: `[
				{
					"metric":{
						"__name__":"test_metric"
					},
					"histogram":[
						1234.567,
						{
							"count":"6",
							"sum":"3897",
							"buckets":[
							[
								1,
								"-4870.992343051145",
								"-4466.7196729968955",
								"1"
							],
							[
								1,
								"-861.0779292198035",
								"-789.6119426088657",
								"1"
							],
							[
								1,
								"-558.3399591246119",
								"-512",
								"1"
							],
							[
								0,
								"2048",
								"2233.3598364984477",
								"1"
							],
							[
								0,
								"2896.3093757400984",
								"3158.4477704354626",
								"1"
							],
							[
								0,
								"4466.7196729968955",
								"4870.992343051145",
								"1"
							]
							]
						}
					]
				},
				{
					"metric":{
						"foo":"bar"
					},
					"histogram":[
						1.234,
						{
							"count":"6",
							"sum":"3897",
							"buckets":[
							[
								1,
								"-4870.992343051145",
								"-4466.7196729968955",
								"1"
							],
							[
								1,
								"-861.0779292198035",
								"-789.6119426088657",
								"1"
							],
							[
								1,
								"-558.3399591246119",
								"-512",
								"1"
							],
							[
								0,
								"2048",
								"2233.3598364984477",
								"1"
							],
							[
								0,
								"2896.3093757400984",
								"3158.4477704354626",
								"1"
							],
							[
								0,
								"4466.7196729968955",
								"4870.992343051145",
								"1"
							]
							]
						}
					]
				}
			]`,
			value: Vector{
				&Sample{
					Metric: Metric{
						MetricNameLabel: "test_metric",
					},
					Histogram: genSampleHistogram(),
					Timestamp: 1234567,
				},
				&Sample{
					Metric: Metric{
						"foo": "bar",
					},
					Histogram: genSampleHistogram(),
					Timestamp: 1234,
				},
			},
		},
	}

	for _, test := range input {
		b, err := json.Marshal(test.value)
		if err != nil {
			t.Error(err)
			continue
		}

		trimmed := noWhitespace.ReplaceAllString(test.plain, "")
		if string(b) != trimmed {
			t.Errorf("encoding error: expected %q, got %q", trimmed, b)
			continue
		}

		var vec Vector
		err = json.Unmarshal(b, &vec)
		if err != nil {
			t.Error(err)
			continue
		}

		if !reflect.DeepEqual(vec, test.value) {
			t.Errorf("decoding error: expected %v, got %v", test.value, vec)
		}
	}
}

func TestMatrixHistogramJSON(t *testing.T) {
	input := []struct {
		plain string
		value Matrix
	}{
		{
			plain: `[]`,
			value: Matrix{},
		},
		{
			plain: sampleHistogramPairMatrixPlain,
			value: sampleHistogramPairMatrixValue,
		},
	}

	for _, test := range input {
		b, err := json.Marshal(test.value)
		if err != nil {
			t.Error(err)
			continue
		}

		trimmed := noWhitespace.ReplaceAllString(test.plain, "")
		if string(b) != trimmed {
			t.Errorf("encoding error: expected %q, got %q", trimmed, b)
			continue
		}

		var mat Matrix
		err = json.Unmarshal(b, &mat)
		if err != nil {
			t.Error(err)
			continue
		}

		if !reflect.DeepEqual(mat, test.value) {
			t.Errorf("decoding error: expected %v, got %v", test.value, mat)
		}
	}
}

func BenchmarkJSONMarshallingSampleHistogramPairMatrix(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_, err := json.Marshal(sampleHistogramPairMatrixValue)
		if err != nil {
			b.Fatal("error marshalling")
		}
	}
}
