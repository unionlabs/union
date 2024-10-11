// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package attribute_test

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
)

func TestKeyValueConstructors(t *testing.T) {
	tt := []struct {
		name     string
		actual   attribute.KeyValue
		expected attribute.KeyValue
	}{
		{
			name:   "Bool",
			actual: attribute.Bool("k1", true),
			expected: attribute.KeyValue{
				Key:   "k1",
				Value: attribute.BoolValue(true),
			},
		},
		{
			name:   "Int64",
			actual: attribute.Int64("k1", 123),
			expected: attribute.KeyValue{
				Key:   "k1",
				Value: attribute.Int64Value(123),
			},
		},
		{
			name:   "Float64",
			actual: attribute.Float64("k1", 123.5),
			expected: attribute.KeyValue{
				Key:   "k1",
				Value: attribute.Float64Value(123.5),
			},
		},
		{
			name:   "String",
			actual: attribute.String("k1", "123.5"),
			expected: attribute.KeyValue{
				Key:   "k1",
				Value: attribute.StringValue("123.5"),
			},
		},
		{
			name:   "Int",
			actual: attribute.Int("k1", 123),
			expected: attribute.KeyValue{
				Key:   "k1",
				Value: attribute.IntValue(123),
			},
		},
	}

	for _, test := range tt {
		t.Run(test.name, func(t *testing.T) {
			if diff := cmp.Diff(test.actual, test.expected, cmp.AllowUnexported(attribute.Value{})); diff != "" {
				t.Fatal(diff)
			}
		})
	}
}

func TestKeyValueValid(t *testing.T) {
	tests := []struct {
		desc  string
		valid bool
		kv    attribute.KeyValue
	}{
		{
			desc:  "uninitialized KeyValue should be invalid",
			valid: false,
			kv:    attribute.KeyValue{},
		},
		{
			desc:  "empty key value should be invalid",
			valid: false,
			kv:    attribute.Key("").Bool(true),
		},
		{
			desc:  "INVALID value type should be invalid",
			valid: false,
			kv: attribute.KeyValue{
				Key: attribute.Key("valid key"),
				// Default type is INVALID.
				Value: attribute.Value{},
			},
		},
		{
			desc:  "non-empty key with BOOL type Value should be valid",
			valid: true,
			kv:    attribute.Bool("bool", true),
		},
		{
			desc:  "non-empty key with INT64 type Value should be valid",
			valid: true,
			kv:    attribute.Int64("int64", 0),
		},
		{
			desc:  "non-empty key with FLOAT64 type Value should be valid",
			valid: true,
			kv:    attribute.Float64("float64", 0),
		},
		{
			desc:  "non-empty key with STRING type Value should be valid",
			valid: true,
			kv:    attribute.String("string", ""),
		},
	}

	for _, test := range tests {
		if got, want := test.kv.Valid(), test.valid; got != want {
			t.Error(test.desc)
		}
	}
}

func TestIncorrectCast(t *testing.T) {
	testCases := []struct {
		name string
		val  attribute.Value
	}{
		{
			name: "Float64",
			val:  attribute.Float64Value(1.0),
		},
		{
			name: "Int64",
			val:  attribute.Int64Value(2),
		},
		{
			name: "String",
			val:  attribute.BoolValue(true),
		},
		{
			name: "Float64Slice",
			val:  attribute.Float64SliceValue([]float64{1.0}),
		},
		{
			name: "Int64Slice",
			val:  attribute.Int64SliceValue([]int64{2}),
		},
		{
			name: "StringSlice",
			val:  attribute.BoolSliceValue([]bool{true}),
		},
	}
	for _, tt := range testCases {
		t.Run(tt.name, func(t *testing.T) {
			assert.NotPanics(t, func() {
				tt.val.AsBool()
				tt.val.AsBoolSlice()
				tt.val.AsFloat64()
				tt.val.AsFloat64Slice()
				tt.val.AsInt64()
				tt.val.AsInt64Slice()
				tt.val.AsInterface()
				tt.val.AsString()
				tt.val.AsStringSlice()
			})
		})
	}
}
