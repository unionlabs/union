package integration_test

import (
	"bytes"
	"fmt"
	"testing"

	"github.com/golang/protobuf/descriptor"
	"github.com/grpc-ecosystem/grpc-gateway/examples/internal/proto/examplepb"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
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

// N.B. These tests are here rather than in the runtime package because they need
// to import examplepb for the descriptor, which would result in a circular
// dependency since examplepb imports runtime from the pb.gw.go files
func TestFieldMaskFromRequestBodyWithDescriptor(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	_, md := descriptor.ForMessage(new(examplepb.NonStandardMessage))
	jsonInput := `{"id":"foo", "thing":{"subThing":{"sub_value":"bar"}}}`
	expected := newFieldMask("id", "thing.subThing.sub_value")

	actual, err := runtime.FieldMaskFromRequestBody(bytes.NewReader([]byte(jsonInput)), md)
	if !fieldMasksEqual(actual, expected) {
		t.Errorf("want %v; got %v", fieldMaskString(expected), fieldMaskString(actual))
	}
	if err != nil {
		t.Errorf("err %v", err)
	}
}

func TestFieldMaskFromRequestBodyWithJsonNames(t *testing.T) {
	if testing.Short() {
		t.Skip()
		return
	}

	_, md := descriptor.ForMessage(new(examplepb.NonStandardMessageWithJSONNames))
	jsonInput := `{"ID":"foo", "Thingy":{"SubThing":{"sub_Value":"bar"}}}`
	expected := newFieldMask("id", "thing.subThing.sub_value")

	actual, err := runtime.FieldMaskFromRequestBody(bytes.NewReader([]byte(jsonInput)), md)
	if !fieldMasksEqual(actual, expected) {
		t.Errorf("want %v; got %v", fieldMaskString(expected), fieldMaskString(actual))
	}
	if err != nil {
		t.Errorf("err %v", err)
	}
}

// avoid compiler optimising benchmark away
var result *field_mask.FieldMask

func BenchmarkABEFieldMaskFromRequestBodyWithDescriptor(b *testing.B) {
	if testing.Short() {
		b.Skip()
		return
	}

	_, md := descriptor.ForMessage(new(examplepb.ABitOfEverything))
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
		r, err = runtime.FieldMaskFromRequestBody(bytes.NewReader([]byte(input)), md)
	}
	if err != nil {
		b.Error(err)
	}
	result = r
}

func BenchmarkNonStandardFieldMaskFromRequestBodyWithDescriptor(b *testing.B) {
	if testing.Short() {
		b.Skip()
		return
	}

	_, md := descriptor.ForMessage(new(examplepb.NonStandardMessage))
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
		r, err = runtime.FieldMaskFromRequestBody(bytes.NewReader([]byte(input)), md)
	}
	if err != nil {
		b.Error(err)
	}
	result = r
}
