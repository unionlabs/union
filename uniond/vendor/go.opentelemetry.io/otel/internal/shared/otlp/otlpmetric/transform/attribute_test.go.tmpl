// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/otlp/otlpmetric/transform/attribute_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package transform

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
	cpb "go.opentelemetry.io/proto/otlp/common/v1"
)

var (
	attrBool         = attribute.Bool("bool", true)
	attrBoolSlice    = attribute.BoolSlice("bool slice", []bool{true, false})
	attrInt          = attribute.Int("int", 1)
	attrIntSlice     = attribute.IntSlice("int slice", []int{-1, 1})
	attrInt64        = attribute.Int64("int64", 1)
	attrInt64Slice   = attribute.Int64Slice("int64 slice", []int64{-1, 1})
	attrFloat64      = attribute.Float64("float64", 1)
	attrFloat64Slice = attribute.Float64Slice("float64 slice", []float64{-1, 1})
	attrString       = attribute.String("string", "o")
	attrStringSlice  = attribute.StringSlice("string slice", []string{"o", "n"})
	attrInvalid      = attribute.KeyValue{
		Key:   attribute.Key("invalid"),
		Value: attribute.Value{},
	}

	valBoolTrue  = &cpb.AnyValue{Value: &cpb.AnyValue_BoolValue{BoolValue: true}}
	valBoolFalse = &cpb.AnyValue{Value: &cpb.AnyValue_BoolValue{BoolValue: false}}
	valBoolSlice = &cpb.AnyValue{Value: &cpb.AnyValue_ArrayValue{
		ArrayValue: &cpb.ArrayValue{
			Values: []*cpb.AnyValue{valBoolTrue, valBoolFalse},
		},
	}}
	valIntOne   = &cpb.AnyValue{Value: &cpb.AnyValue_IntValue{IntValue: 1}}
	valIntNOne  = &cpb.AnyValue{Value: &cpb.AnyValue_IntValue{IntValue: -1}}
	valIntSlice = &cpb.AnyValue{Value: &cpb.AnyValue_ArrayValue{
		ArrayValue: &cpb.ArrayValue{
			Values: []*cpb.AnyValue{valIntNOne, valIntOne},
		},
	}}
	valDblOne   = &cpb.AnyValue{Value: &cpb.AnyValue_DoubleValue{DoubleValue: 1}}
	valDblNOne  = &cpb.AnyValue{Value: &cpb.AnyValue_DoubleValue{DoubleValue: -1}}
	valDblSlice = &cpb.AnyValue{Value: &cpb.AnyValue_ArrayValue{
		ArrayValue: &cpb.ArrayValue{
			Values: []*cpb.AnyValue{valDblNOne, valDblOne},
		},
	}}
	valStrO     = &cpb.AnyValue{Value: &cpb.AnyValue_StringValue{StringValue: "o"}}
	valStrN     = &cpb.AnyValue{Value: &cpb.AnyValue_StringValue{StringValue: "n"}}
	valStrSlice = &cpb.AnyValue{Value: &cpb.AnyValue_ArrayValue{
		ArrayValue: &cpb.ArrayValue{
			Values: []*cpb.AnyValue{valStrO, valStrN},
		},
	}}

	kvBool         = &cpb.KeyValue{Key: "bool", Value: valBoolTrue}
	kvBoolSlice    = &cpb.KeyValue{Key: "bool slice", Value: valBoolSlice}
	kvInt          = &cpb.KeyValue{Key: "int", Value: valIntOne}
	kvIntSlice     = &cpb.KeyValue{Key: "int slice", Value: valIntSlice}
	kvInt64        = &cpb.KeyValue{Key: "int64", Value: valIntOne}
	kvInt64Slice   = &cpb.KeyValue{Key: "int64 slice", Value: valIntSlice}
	kvFloat64      = &cpb.KeyValue{Key: "float64", Value: valDblOne}
	kvFloat64Slice = &cpb.KeyValue{Key: "float64 slice", Value: valDblSlice}
	kvString       = &cpb.KeyValue{Key: "string", Value: valStrO}
	kvStringSlice  = &cpb.KeyValue{Key: "string slice", Value: valStrSlice}
	kvInvalid      = &cpb.KeyValue{
		Key: "invalid",
		Value: &cpb.AnyValue{
			Value: &cpb.AnyValue_StringValue{StringValue: "INVALID"},
		},
	}
)

type attributeTest struct {
	name string
	in   []attribute.KeyValue
	want []*cpb.KeyValue
}

func TestAttributeTransforms(t *testing.T) {
	for _, test := range []attributeTest{
		{"nil", nil, nil},
		{"empty", []attribute.KeyValue{}, nil},
		{
			"invalid",
			[]attribute.KeyValue{attrInvalid},
			[]*cpb.KeyValue{kvInvalid},
		},
		{
			"bool",
			[]attribute.KeyValue{attrBool},
			[]*cpb.KeyValue{kvBool},
		},
		{
			"bool slice",
			[]attribute.KeyValue{attrBoolSlice},
			[]*cpb.KeyValue{kvBoolSlice},
		},
		{
			"int",
			[]attribute.KeyValue{attrInt},
			[]*cpb.KeyValue{kvInt},
		},
		{
			"int slice",
			[]attribute.KeyValue{attrIntSlice},
			[]*cpb.KeyValue{kvIntSlice},
		},
		{
			"int64",
			[]attribute.KeyValue{attrInt64},
			[]*cpb.KeyValue{kvInt64},
		},
		{
			"int64 slice",
			[]attribute.KeyValue{attrInt64Slice},
			[]*cpb.KeyValue{kvInt64Slice},
		},
		{
			"float64",
			[]attribute.KeyValue{attrFloat64},
			[]*cpb.KeyValue{kvFloat64},
		},
		{
			"float64 slice",
			[]attribute.KeyValue{attrFloat64Slice},
			[]*cpb.KeyValue{kvFloat64Slice},
		},
		{
			"string",
			[]attribute.KeyValue{attrString},
			[]*cpb.KeyValue{kvString},
		},
		{
			"string slice",
			[]attribute.KeyValue{attrStringSlice},
			[]*cpb.KeyValue{kvStringSlice},
		},
		{
			"all",
			[]attribute.KeyValue{
				attrBool,
				attrBoolSlice,
				attrInt,
				attrIntSlice,
				attrInt64,
				attrInt64Slice,
				attrFloat64,
				attrFloat64Slice,
				attrString,
				attrStringSlice,
				attrInvalid,
			},
			[]*cpb.KeyValue{
				kvBool,
				kvBoolSlice,
				kvInt,
				kvIntSlice,
				kvInt64,
				kvInt64Slice,
				kvFloat64,
				kvFloat64Slice,
				kvString,
				kvStringSlice,
				kvInvalid,
			},
		},
	} {
		t.Run(test.name, func(t *testing.T) {
			t.Run("KeyValues", func(t *testing.T) {
				assert.ElementsMatch(t, test.want, KeyValues(test.in))
			})
			t.Run("AttrIter", func(t *testing.T) {
				s := attribute.NewSet(test.in...)
				assert.ElementsMatch(t, test.want, AttrIter(s.Iter()))
			})
		})
	}
}
