package runtime

import (
	"bytes"
	"fmt"
	"testing"

	"google.golang.org/genproto/protobuf/field_mask"
)

func fieldMasksEqual(fm1, fm2 *field_mask.FieldMask) bool {
	if fm1 == nil && fm2 == nil {
		return true
	}
	if fm1 == nil || fm2 == nil {
		return false
	}
	if len(fm1.GetPaths()) != len(fm2.GetPaths()) {
		return false
	}

	paths := make(map[string]bool)
	for _, path := range fm1.GetPaths() {
		paths[path] = true
	}
	for _, path := range fm2.GetPaths() {
		if _, ok := paths[path]; !ok {
			return false
		}
	}

	return true
}

func newFieldMask(paths ...string) *field_mask.FieldMask {
	return &field_mask.FieldMask{Paths: paths}
}

func fieldMaskString(fm *field_mask.FieldMask) string {
	if fm == nil {
		return ""
	}
	return fmt.Sprintf("%v", fm.GetPaths())
}

func TestFieldMaskFromRequestBody(t *testing.T) {
	for _, tc := range []struct {
		name        string
		input       string
		expected    *field_mask.FieldMask
		expectedErr error
	}{
		{name: "empty", expected: newFieldMask()},
		{name: "simple", input: `{"foo":1, "bar":"baz"}`, expected: newFieldMask("foo", "bar")},
		{name: "nested", input: `{"foo": {"bar":1, "baz": 2}, "qux": 3}`, expected: newFieldMask("foo.bar", "foo.baz", "qux")},
		{name: "canonical", input: `{"f": {"b": {"d": 1, "x": 2}, "c": 1}}`, expected: newFieldMask("f.b.d", "f.b.x", "f.c")},
		{name: "deeply-nested", input: `{"foo": {"bar": {"baz": {"a": 1, "b": 2}}}}`, expected: newFieldMask("foo.bar.baz.a", "foo.bar.baz.b")},
	} {
		t.Run(tc.name, func(t *testing.T) {
			actual, err := FieldMaskFromRequestBody(bytes.NewReader([]byte(tc.input)), nil)
			if !fieldMasksEqual(actual, tc.expected) {
				t.Errorf("want %v; got %v", fieldMaskString(tc.expected), fieldMaskString(actual))
			}
			if err != tc.expectedErr {
				t.Errorf("want %v; got %v", tc.expectedErr, err)
			}
		})
	}
}

// avoid compiler optimising benchmark away
var result *field_mask.FieldMask

func BenchmarkABEFieldMaskFromRequestBody(b *testing.B) {
	input := `{` +
		`"single_nested":				{"name": "bar",` +
		`                           	 "amount": 10,` +
		`                           	 "ok": "TRUE"},` +
		`"uuid":						"6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",` +
		`"nested": 						[{"name": "bar",` +
		`								  "amount": 10},` +
		`								 {"name": "baz",` +
		`								  "amount": 20}],` +
		`"float_value":             	1.5,` +
		`"double_value":            	2.5,` +
		`"int64_value":             	4294967296,` +
		`"uint64_value":            	9223372036854775807,` +
		`"int32_value":             	-2147483648,` +
		`"fixed64_value":           	9223372036854775807,` +
		`"fixed32_value":           	4294967295,` +
		`"bool_value":              	true,` +
		`"string_value":            	"strprefix/foo",` +
		`"bytes_value":					"132456",` +
		`"uint32_value":            	4294967295,` +
		`"enum_value":     		        "ONE",` +
		`"path_enum_value":	    	    "DEF",` +
		`"nested_path_enum_value":  	"JKL",` +
		`"sfixed32_value":          	2147483647,` +
		`"sfixed64_value":          	-4611686018427387904,` +
		`"sint32_value":            	2147483647,` +
		`"sint64_value":            	4611686018427387903,` +
		`"repeated_string_value": 		["a", "b", "c"],` +
		`"oneof_value":					{"oneof_string":"x"},` +
		`"map_value": 					{"a": "ONE",` +
		`								 "b": "ZERO"},` +
		`"mapped_string_value": 		{"a": "x",` +
		`								 "b": "y"},` +
		`"mapped_nested_value": 		{"a": {"name": "x", "amount": 1},` +
		`								 "b": {"name": "y", "amount": 2}},` +
		`"nonConventionalNameValue":	"camelCase",` +
		`"timestamp_value":				"2016-05-10T10:19:13.123Z",` +
		`"repeated_enum_value":			["ONE", "ZERO"],` +
		`"repeated_enum_annotation":	 ["ONE", "ZERO"],` +
		`"enum_value_annotation": 		"ONE",` +
		`"repeated_string_annotation":	["a", "b"],` +
		`"repeated_nested_annotation": 	[{"name": "hoge",` +
		`								  "amount": 10},` +
		`								 {"name": "fuga",` +
		`								  "amount": 20}],` +
		`"nested_annotation": 			{"name": "hoge",` +
		`								 "amount": 10},` +
		`"int64_override_type":			12345` +
		`}`
	var r *field_mask.FieldMask
	var err error
	for i := 0; i < b.N; i++ {
		r, err = FieldMaskFromRequestBody(bytes.NewReader([]byte(input)), nil)
	}
	if err != nil {
		b.Error(err)
	}
	result = r
}

func BenchmarkNonStandardFieldMaskFromRequestBody(b *testing.B) {
	input := `{` +
		`"id":			"foo",` +
		`"Num": 		2,` +
		`"line_num": 	3,` +
		`"langIdent":	"bar",` +
		`"STATUS": 		"baz"` +
		`}`
	var r *field_mask.FieldMask
	var err error
	for i := 0; i < b.N; i++ {
		r, err = FieldMaskFromRequestBody(bytes.NewReader([]byte(input)), nil)
	}
	if err != nil {
		b.Error(err)
	}
	result = r
}
