// Copyright 2024 The Prometheus Authors
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
	"testing"

	"github.com/prometheus/common/model"

	"github.com/stretchr/testify/require"
)

// Test Format to Escapting Scheme conversion
// Path: expfmt/expfmt_test.go
// Compare this snippet from expfmt/expfmt.go:
func TestToFormatType(t *testing.T) {
	tests := []struct {
		format   Format
		expected FormatType
	}{
		{
			format:   FmtProtoCompact,
			expected: TypeProtoCompact,
		},
		{
			format:   FmtProtoDelim,
			expected: TypeProtoDelim,
		},
		{
			format:   FmtProtoText,
			expected: TypeProtoText,
		},
		{
			format:   FmtOpenMetrics_1_0_0,
			expected: TypeOpenMetrics,
		},
		{
			format:   FmtText,
			expected: TypeTextPlain,
		},
		{
			format:   FmtOpenMetrics_0_0_1,
			expected: TypeOpenMetrics,
		},
		{
			format:   "application/vnd.google.protobuf; proto=BadProtocol; encoding=text",
			expected: TypeUnknown,
		},
		{
			format:   "application/vnd.google.protobuf",
			expected: TypeUnknown,
		},
		{
			format:   "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily=bad",
			expected: TypeUnknown,
		},
		// encoding missing
		{
			format:   "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily",
			expected: TypeUnknown,
		},
		// invalid encoding
		{
			format:   "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=textual",
			expected: TypeUnknown,
		},
		// bad charset, must be utf-8
		{
			format:   "application/openmetrics-text; version=1.0.0; charset=ascii",
			expected: TypeUnknown,
		},
		{
			format:   "text/plain",
			expected: TypeTextPlain,
		},
		{
			format:   "text/plain; version=invalid",
			expected: TypeUnknown,
		},
		{
			format:   "gobbledygook",
			expected: TypeUnknown,
		},
	}
	for _, test := range tests {
		require.Equal(t, test.expected, test.format.FormatType())
	}
}

func TestToEscapingScheme(t *testing.T) {
	tests := []struct {
		format   Format
		expected model.EscapingScheme
	}{
		{
			format:   FmtProtoCompact,
			expected: model.UnderscoreEscaping,
		},
		{
			format:   "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=dots",
			expected: model.DotsEscaping,
		},
		{
			format:   "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=allow-utf-8",
			expected: model.NoEscaping,
		},
		// error returns default
		{
			format:   "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=invalid",
			expected: model.NameEscapingScheme,
		},
	}
	for _, test := range tests {
		require.Equal(t, test.expected, test.format.ToEscapingScheme())
	}
}

func TestWithEscapingScheme(t *testing.T) {
	tests := []struct {
		name     string
		format   Format
		scheme   model.EscapingScheme
		expected string
	}{
		{
			name:     "no existing term, append one",
			format:   FmtProtoCompact,
			scheme:   model.DotsEscaping,
			expected: "application/vnd.google.protobuf; proto=io.prometheus.client.MetricFamily; encoding=compact-text; escaping=dots",
		},
		{
			name:     "existing term at end, replace",
			format:   "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=underscores",
			scheme:   model.ValueEncodingEscaping,
			expected: "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=values",
		},
		{
			name:     "existing term in middle, replace",
			format:   "application/openmetrics-text; escaping=dots; version=1.0.0; charset=utf-8; ",
			scheme:   model.NoEscaping,
			expected: "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=allow-utf-8",
		},
		{
			name:     "multiple existing terms removed",
			format:   "application/openmetrics-text; escaping=dots; version=1.0.0; charset=utf-8; escaping=allow-utf-8",
			scheme:   model.ValueEncodingEscaping,
			expected: "application/openmetrics-text; version=1.0.0; charset=utf-8; escaping=values",
		},
	}
	for _, test := range tests {
		require.Equal(t, test.expected, string(test.format.WithEscapingScheme(test.scheme)))
	}
}
