package sentry

import (
	"testing"

	"github.com/google/go-cmp/cmp"
)

func TestSanitizeKey(t *testing.T) {
	tests := []struct {
		name string
		key  string
		want string
	}{
		{
			name: "allowed characters",
			key:  "test.metric-1",
			want: "test.metric-1",
		},
		{
			name: "forbidden characters",
			key:  "@test.me^tri'@c-1{}[]",
			want: "_test.me_tri_c-1_",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if diff := cmp.Diff(sanitizeKey(test.key), test.want); diff != "" {
				t.Errorf("Context mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestSanitizeValue(t *testing.T) {
	tests := []struct {
		name  string
		value string
		want  string
	}{
		{
			name:  "allowed characters",
			value: "test.metric-1",
			want:  "test.metric-1",
		},
		{
			name:  "forbidden characters",
			value: "@test.me^tri'+@c-1{}[]",
			want:  "@test.metri@c-1{}[]",
		},
		{
			name:  "allow empty character",
			value: "@route /test",
			want:  "@route /test",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if diff := cmp.Diff(sanitizeValue(test.value), test.want); diff != "" {
				t.Errorf("Context mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestSerializeTags(t *testing.T) {
	tests := []struct {
		name   string
		metric abstractMetric
		want   string
	}{
		{
			name: "normal tags",
			metric: abstractMetric{
				tags: map[string]string{"tag1": "val1", "tag2": "val2"},
			},
			want: "tag1:val1,tag2:val2",
		},
		{
			name: "empty tags",
			metric: abstractMetric{
				tags: map[string]string{},
			},
			want: "",
		},
		{
			name: "un-sanitized tags",
			metric: abstractMetric{
				tags: map[string]string{"@env": "pro+d", "vers^^ion": `\release@`},
			},
			want: "_env:prod,vers_ion:release@",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if diff := cmp.Diff(test.metric.SerializeTags(), test.want); diff != "" {
				t.Errorf("Context mismatch (-want +got):\n%s", diff)
			}
		})
	}
}

func TestSerializeValue(t *testing.T) {
	tests := []struct {
		name   string
		metric Metric
		want   string
	}{
		{
			name: "distribution metric",
			metric: DistributionMetric{
				values: []float64{2.0, 4.0, 3.0, 6.0},
			},
			want: ":2:4:3:6",
		},
		{
			name: "gauge metric",
			metric: GaugeMetric{
				last:  1.0,
				min:   1.0,
				max:   1.0,
				sum:   1.0,
				count: 1.0,
			},
			want: ":1:1:1:1:1",
		},
		{
			name: "set metric with strings",
			metric: SetMetric[string]{
				values: map[string]void{"Hello": member, "World": member},
			},
			want: ":4157704578:4223024711",
		},
		{
			name: "set metric with integers",
			metric: SetMetric[int]{
				values: map[int]void{1: member, 2: member},
			},
			want: ":1:2",
		},
		{
			name: "counter metric",
			metric: CounterMetric{
				value: 2.0,
			},
			want: ":2",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if diff := cmp.Diff(test.metric.SerializeValue(), test.want); diff != "" {
				t.Errorf("Context mismatch (-want +got):\n%s", diff)
			}
		})
	}
}
