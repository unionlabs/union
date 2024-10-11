// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package attribute_test

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
)

func TestDefined(t *testing.T) {
	for _, testcase := range []struct {
		name string
		k    attribute.Key
		want bool
	}{
		{
			name: "Key.Defined() returns true when len(v.Name) != 0",
			k:    attribute.Key("foo"),
			want: true,
		},
		{
			name: "Key.Defined() returns false when len(v.Name) == 0",
			k:    attribute.Key(""),
			want: false,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// func (k attribute.Key) Defined() bool {
			have := testcase.k.Defined()
			if have != testcase.want {
				t.Errorf("Want: %v, but have: %v", testcase.want, have)
			}
		})
	}
}

func TestJSONValue(t *testing.T) {
	var kvs interface{} = [2]attribute.KeyValue{
		attribute.String("A", "B"),
		attribute.Int64("C", 1),
	}

	data, err := json.Marshal(kvs)
	require.NoError(t, err)
	require.Equal(t,
		`[{"Key":"A","Value":{"Type":"STRING","Value":"B"}},{"Key":"C","Value":{"Type":"INT64","Value":1}}]`,
		string(data))
}

func TestEmit(t *testing.T) {
	for _, testcase := range []struct {
		name string
		v    attribute.Value
		want string
	}{
		{
			name: `test Key.Emit() can emit a string representing self.BOOL`,
			v:    attribute.BoolValue(true),
			want: "true",
		},
		{
			name: `test Key.Emit() can emit a string representing self.INT64SLICE`,
			v:    attribute.Int64SliceValue([]int64{1, 42}),
			want: `[1,42]`,
		},
		{
			name: `test Key.Emit() can emit a string representing self.INT64`,
			v:    attribute.Int64Value(42),
			want: "42",
		},
		{
			name: `test Key.Emit() can emit a string representing self.FLOAT64SLICE`,
			v:    attribute.Float64SliceValue([]float64{1.0, 42.5}),
			want: `[1,42.5]`,
		},
		{
			name: `test Key.Emit() can emit a string representing self.FLOAT64`,
			v:    attribute.Float64Value(42.1),
			want: "42.1",
		},
		{
			name: `test Key.Emit() can emit a string representing self.STRING`,
			v:    attribute.StringValue("foo"),
			want: "foo",
		},
		{
			name: `test Key.Emit() can emit a string representing self.STRINGSLICE`,
			v:    attribute.StringSliceValue([]string{"foo", "bar"}),
			want: `["foo","bar"]`,
		},
	} {
		t.Run(testcase.name, func(t *testing.T) {
			// proto: func (v attribute.Value) Emit() string {
			have := testcase.v.Emit()
			if have != testcase.want {
				t.Errorf("Want: %s, but have: %s", testcase.want, have)
			}
		})
	}
}
