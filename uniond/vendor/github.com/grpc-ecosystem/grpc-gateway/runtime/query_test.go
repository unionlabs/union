package runtime_test

import (
	"errors"
	"fmt"
	"net/url"
	"reflect"
	"testing"
	"time"

	"github.com/golang/protobuf/proto"
	"github.com/golang/protobuf/ptypes"
	"github.com/golang/protobuf/ptypes/duration"
	"github.com/golang/protobuf/ptypes/timestamp"
	"github.com/golang/protobuf/ptypes/wrappers"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/utilities"
	"google.golang.org/genproto/protobuf/field_mask"
)

func BenchmarkPopulateQueryParameters(b *testing.B) {
	timeT := time.Date(2016, time.December, 15, 12, 23, 32, 49, time.UTC)
	timeStr := timeT.Format(time.RFC3339Nano)

	durationT := 13 * time.Hour
	durationStr := durationT.String()

	fieldmaskStr := "float_value,double_value"

	msg := &proto3Message{}
	values := url.Values{
		"float_value":            {"1.5"},
		"double_value":           {"2.5"},
		"int64_value":            {"-1"},
		"int32_value":            {"-2"},
		"uint64_value":           {"3"},
		"uint32_value":           {"4"},
		"bool_value":             {"true"},
		"string_value":           {"str"},
		"bytes_value":            {"Ynl0ZXM="},
		"repeated_value":         {"a", "b", "c"},
		"enum_value":             {"1"},
		"repeated_enum":          {"1", "2", "0"},
		"timestamp_value":        {timeStr},
		"duration_value":         {durationStr},
		"fieldmask_value":        {fieldmaskStr},
		"wrapper_float_value":    {"1.5"},
		"wrapper_double_value":   {"2.5"},
		"wrapper_int64_value":    {"-1"},
		"wrapper_int32_value":    {"-2"},
		"wrapper_u_int64_value":  {"3"},
		"wrapper_u_int32_value":  {"4"},
		"wrapper_bool_value":     {"true"},
		"wrapper_string_value":   {"str"},
		"wrapper_bytes_value":    {"Ynl0ZXM="},
		"map_value[key]":         {"value"},
		"map_value[second]":      {"bar"},
		"map_value[third]":       {"zzz"},
		"map_value[fourth]":      {""},
		`map_value[~!@#$%^&*()]`: {"value"},
		"map_value2[key]":        {"-2"},
		"map_value3[-2]":         {"value"},
		"map_value4[key]":        {"-1"},
		"map_value5[-1]":         {"value"},
		"map_value6[key]":        {"3"},
		"map_value7[3]":          {"value"},
		"map_value8[key]":        {"4"},
		"map_value9[4]":          {"value"},
		"map_value10[key]":       {"1.5"},
		"map_value11[1.5]":       {"value"},
		"map_value12[key]":       {"2.5"},
		"map_value13[2.5]":       {"value"},
		"map_value14[key]":       {"true"},
		"map_value15[true]":      {"value"},
	}
	filter := utilities.NewDoubleArray([][]string{
		{"bool_value"}, {"repeated_value"},
	})

	for i := 0; i < b.N; i++ {
		_ = runtime.PopulateQueryParameters(msg, values, filter)
	}
}

func TestPopulateParameters(t *testing.T) {
	timeT := time.Date(2016, time.December, 15, 12, 23, 32, 49, time.UTC)
	timeStr := timeT.Format(time.RFC3339Nano)
	timePb, err := ptypes.TimestampProto(timeT)
	if err != nil {
		t.Fatalf("Couldn't setup timestamp in Protobuf format: %v", err)
	}

	durationT := 13 * time.Hour
	durationStr := durationT.String()
	durationPb := ptypes.DurationProto(durationT)

	fieldmaskStr := "float_value,double_value"
	fieldmaskPb := &field_mask.FieldMask{Paths: []string{"float_value", "double_value"}}

	for _, spec := range []struct {
		values  url.Values
		filter  *utilities.DoubleArray
		want    proto.Message
		wanterr error
	}{
		{
			values: url.Values{
				"float_value":            {"1.5"},
				"double_value":           {"2.5"},
				"int64_value":            {"-1"},
				"int32_value":            {"-2"},
				"uint64_value":           {"3"},
				"uint32_value":           {"4"},
				"bool_value":             {"true"},
				"string_value":           {"str"},
				"bytes_value":            {"Ynl0ZXM="},
				"repeated_value":         {"a", "b", "c"},
				"enum_value":             {"1"},
				"repeated_enum":          {"1", "2", "0"},
				"timestamp_value":        {timeStr},
				"duration_value":         {durationStr},
				"fieldmask_value":        {fieldmaskStr},
				"wrapper_float_value":    {"1.5"},
				"wrapper_double_value":   {"2.5"},
				"wrapper_int64_value":    {"-1"},
				"wrapper_int32_value":    {"-2"},
				"wrapper_u_int64_value":  {"3"},
				"wrapper_u_int32_value":  {"4"},
				"wrapper_bool_value":     {"true"},
				"wrapper_string_value":   {"str"},
				"wrapper_bytes_value":    {"Ynl0ZXM="},
				"map_value[key]":         {"value"},
				"map_value[second]":      {"bar"},
				"map_value[third]":       {"zzz"},
				"map_value[fourth]":      {""},
				`map_value[~!@#$%^&*()]`: {"value"},
				"map_value2[key]":        {"-2"},
				"map_value3[-2]":         {"value"},
				"map_value4[key]":        {"-1"},
				"map_value5[-1]":         {"value"},
				"map_value6[key]":        {"3"},
				"map_value7[3]":          {"value"},
				"map_value8[key]":        {"4"},
				"map_value9[4]":          {"value"},
				"map_value10[key]":       {"1.5"},
				"map_value11[1.5]":       {"value"},
				"map_value12[key]":       {"2.5"},
				"map_value13[2.5]":       {"value"},
				"map_value14[key]":       {"true"},
				"map_value15[true]":      {"value"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				FloatValue:         1.5,
				DoubleValue:        2.5,
				Int64Value:         -1,
				Int32Value:         -2,
				Uint64Value:        3,
				Uint32Value:        4,
				BoolValue:          true,
				StringValue:        "str",
				BytesValue:         []byte("bytes"),
				RepeatedValue:      []string{"a", "b", "c"},
				EnumValue:          EnumValue_Y,
				RepeatedEnum:       []EnumValue{EnumValue_Y, EnumValue_Z, EnumValue_X},
				TimestampValue:     timePb,
				DurationValue:      durationPb,
				FieldMaskValue:     fieldmaskPb,
				WrapperFloatValue:  &wrappers.FloatValue{Value: 1.5},
				WrapperDoubleValue: &wrappers.DoubleValue{Value: 2.5},
				WrapperInt64Value:  &wrappers.Int64Value{Value: -1},
				WrapperInt32Value:  &wrappers.Int32Value{Value: -2},
				WrapperUInt64Value: &wrappers.UInt64Value{Value: 3},
				WrapperUInt32Value: &wrappers.UInt32Value{Value: 4},
				WrapperBoolValue:   &wrappers.BoolValue{Value: true},
				WrapperStringValue: &wrappers.StringValue{Value: "str"},
				WrapperBytesValue:  &wrappers.BytesValue{Value: []byte("bytes")},
				MapValue: map[string]string{
					"key":         "value",
					"second":      "bar",
					"third":       "zzz",
					"fourth":      "",
					`~!@#$%^&*()`: "value",
				},
				MapValue2:  map[string]int32{"key": -2},
				MapValue3:  map[int32]string{-2: "value"},
				MapValue4:  map[string]int64{"key": -1},
				MapValue5:  map[int64]string{-1: "value"},
				MapValue6:  map[string]uint32{"key": 3},
				MapValue7:  map[uint32]string{3: "value"},
				MapValue8:  map[string]uint64{"key": 4},
				MapValue9:  map[uint64]string{4: "value"},
				MapValue10: map[string]float32{"key": 1.5},
				MapValue11: map[float32]string{1.5: "value"},
				MapValue12: map[string]float64{"key": 2.5},
				MapValue13: map[float64]string{2.5: "value"},
				MapValue14: map[string]bool{"key": true},
				MapValue15: map[bool]string{true: "value"},
			},
		},
		{
			values: url.Values{
				"floatValue":         {"1.5"},
				"doubleValue":        {"2.5"},
				"int64Value":         {"-1"},
				"int32Value":         {"-2"},
				"uint64Value":        {"3"},
				"uint32Value":        {"4"},
				"boolValue":          {"true"},
				"stringValue":        {"str"},
				"bytesValue":         {"Ynl0ZXM="},
				"repeatedValue":      {"a", "b", "c"},
				"enumValue":          {"1"},
				"repeatedEnum":       {"1", "2", "0"},
				"timestampValue":     {timeStr},
				"durationValue":      {durationStr},
				"fieldmaskValue":     {fieldmaskStr},
				"wrapperFloatValue":  {"1.5"},
				"wrapperDoubleValue": {"2.5"},
				"wrapperInt64Value":  {"-1"},
				"wrapperInt32Value":  {"-2"},
				"wrapperUInt64Value": {"3"},
				"wrapperUInt32Value": {"4"},
				"wrapperBoolValue":   {"true"},
				"wrapperStringValue": {"str"},
				"wrapperBytesValue":  {"Ynl0ZXM="},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				FloatValue:         1.5,
				DoubleValue:        2.5,
				Int64Value:         -1,
				Int32Value:         -2,
				Uint64Value:        3,
				Uint32Value:        4,
				BoolValue:          true,
				StringValue:        "str",
				BytesValue:         []byte("bytes"),
				RepeatedValue:      []string{"a", "b", "c"},
				EnumValue:          EnumValue_Y,
				RepeatedEnum:       []EnumValue{EnumValue_Y, EnumValue_Z, EnumValue_X},
				TimestampValue:     timePb,
				DurationValue:      durationPb,
				FieldMaskValue:     fieldmaskPb,
				WrapperFloatValue:  &wrappers.FloatValue{Value: 1.5},
				WrapperDoubleValue: &wrappers.DoubleValue{Value: 2.5},
				WrapperInt64Value:  &wrappers.Int64Value{Value: -1},
				WrapperInt32Value:  &wrappers.Int32Value{Value: -2},
				WrapperUInt64Value: &wrappers.UInt64Value{Value: 3},
				WrapperUInt32Value: &wrappers.UInt32Value{Value: 4},
				WrapperBoolValue:   &wrappers.BoolValue{Value: true},
				WrapperStringValue: &wrappers.StringValue{Value: "str"},
				WrapperBytesValue:  &wrappers.BytesValue{Value: []byte("bytes")},
			},
		},
		{
			values: url.Values{
				"enum_value":    {"EnumValue_Z"},
				"repeated_enum": {"EnumValue_X", "2", "0"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				EnumValue:    EnumValue_Z,
				RepeatedEnum: []EnumValue{EnumValue_X, EnumValue_Z, EnumValue_X},
			},
		},
		{
			values: url.Values{
				"float_value":    {"1.5"},
				"double_value":   {"2.5"},
				"int64_value":    {"-1"},
				"int32_value":    {"-2"},
				"uint64_value":   {"3"},
				"uint32_value":   {"4"},
				"bool_value":     {"true"},
				"string_value":   {"str"},
				"repeated_value": {"a", "b", "c"},
				"enum_value":     {"1"},
				"repeated_enum":  {"1", "2", "0"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto2Message{
				FloatValue:    proto.Float32(1.5),
				DoubleValue:   proto.Float64(2.5),
				Int64Value:    proto.Int64(-1),
				Int32Value:    proto.Int32(-2),
				Uint64Value:   proto.Uint64(3),
				Uint32Value:   proto.Uint32(4),
				BoolValue:     proto.Bool(true),
				StringValue:   proto.String("str"),
				RepeatedValue: []string{"a", "b", "c"},
				EnumValue:     EnumValue_Y,
				RepeatedEnum:  []EnumValue{EnumValue_Y, EnumValue_Z, EnumValue_X},
			},
		},
		{
			values: url.Values{
				"floatValue":    {"1.5"},
				"doubleValue":   {"2.5"},
				"int64Value":    {"-1"},
				"int32Value":    {"-2"},
				"uint64Value":   {"3"},
				"uint32Value":   {"4"},
				"boolValue":     {"true"},
				"stringValue":   {"str"},
				"repeatedValue": {"a", "b", "c"},
				"enumValue":     {"1"},
				"repeatedEnum":  {"1", "2", "0"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto2Message{
				FloatValue:    proto.Float32(1.5),
				DoubleValue:   proto.Float64(2.5),
				Int64Value:    proto.Int64(-1),
				Int32Value:    proto.Int32(-2),
				Uint64Value:   proto.Uint64(3),
				Uint32Value:   proto.Uint32(4),
				BoolValue:     proto.Bool(true),
				StringValue:   proto.String("str"),
				RepeatedValue: []string{"a", "b", "c"},
				EnumValue:     EnumValue_Y,
				RepeatedEnum:  []EnumValue{EnumValue_Y, EnumValue_Z, EnumValue_X},
			},
		},
		{
			values: url.Values{
				"nested.nested.nested.repeated_value": {"a", "b", "c"},
				"nested.nested.nested.string_value":   {"s"},
				"nested.nested.string_value":          {"t"},
				"nested.string_value":                 {"u"},
				"nested_non_null.string_value":        {"v"},
				"nested.nested.map_value[first]":      {"foo"},
				"nested.nested.map_value[second]":     {"bar"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				Nested: &proto2Message{
					Nested: &proto3Message{
						MapValue: map[string]string{
							"first":  "foo",
							"second": "bar",
						},
						Nested: &proto2Message{
							RepeatedValue: []string{"a", "b", "c"},
							StringValue:   proto.String("s"),
						},
						StringValue: "t",
					},
					StringValue: proto.String("u"),
				},
				NestedNonNull: proto2Message{
					StringValue: proto.String("v"),
				},
			},
		},
		{
			values: url.Values{
				"uint64_value": {"1", "2", "3", "4", "5"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				Uint64Value: 1,
			},
		},
		{
			values: url.Values{
				"oneof_string_value": {"foobar"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				OneofValue: &proto3Message_OneofStringValue{"foobar"},
			},
		},
		{
			values: url.Values{
				"oneofStringValue": {"foobar"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				OneofValue: &proto3Message_OneofStringValue{"foobar"},
			},
		},
		{
			values: url.Values{
				"oneof_bool_value": {"true"},
			},
			filter: utilities.NewDoubleArray(nil),
			want: &proto3Message{
				OneofValue: &proto3Message_OneofBoolValue{true},
			},
		},
		{
			// Don't allow setting a oneof more than once
			values: url.Values{
				"oneof_bool_value":   {"true"},
				"oneof_string_value": {"foobar"},
			},
			filter:  utilities.NewDoubleArray(nil),
			want:    &proto3Message{},
			wanterr: errors.New("field already set for oneof_value oneof"),
		},
	} {
		msg := proto.Clone(spec.want)
		msg.Reset()
		err := runtime.PopulateQueryParameters(msg, spec.values, spec.filter)
		if spec.wanterr != nil {
			if !reflect.DeepEqual(err, spec.wanterr) {
				t.Errorf("runtime.PopulateQueryParameters(msg, %v, %v) failed with %v; want error %v", spec.values, spec.filter, err, spec.wanterr)
			}
			continue
		}

		if err != nil {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, %v) failed with %v; want success", spec.values, spec.filter, err)
			continue
		}
		if got, want := msg, spec.want; !proto.Equal(got, want) {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, %v = %v; want %v", spec.values, spec.filter, got, want)
		}
	}
}

func TestPopulateParametersWithNativeTypes(t *testing.T) {
	timeT := time.Date(2016, time.December, 15, 12, 23, 32, 49, time.UTC)
	timeStr := timeT.Format(time.RFC3339Nano)

	durationT := 13 * time.Hour
	durationStr := durationT.String()

	for _, spec := range []struct {
		values url.Values
		want   *nativeProto3Message
	}{
		{
			values: url.Values{
				"native_timestamp_value": {timeStr},
				"native_duration_value":  {durationStr},
			},
			want: &nativeProto3Message{
				NativeTimeValue:     &timeT,
				NativeDurationValue: &durationT,
			},
		},
		{
			values: url.Values{
				"nativeTimestampValue": {timeStr},
				"nativeDurationValue":  {durationStr},
			},
			want: &nativeProto3Message{
				NativeTimeValue:     &timeT,
				NativeDurationValue: &durationT,
			},
		},
	} {
		msg := new(nativeProto3Message)
		err := runtime.PopulateQueryParameters(msg, spec.values, utilities.NewDoubleArray(nil))

		if err != nil {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, utilities.NewDoubleArray(nil)) failed with %v; want success", spec.values, err)
			continue
		}
		if got, want := msg, spec.want; !proto.Equal(got, want) {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, utilities.NewDoubleArray(nil)) = %v; want %v", spec.values, got, want)
		}
	}
}

func TestPopulateParametersWithFilters(t *testing.T) {
	for _, spec := range []struct {
		values url.Values
		filter *utilities.DoubleArray
		want   proto.Message
	}{
		{
			values: url.Values{
				"bool_value":     {"true"},
				"string_value":   {"str"},
				"repeated_value": {"a", "b", "c"},
			},
			filter: utilities.NewDoubleArray([][]string{
				{"bool_value"}, {"repeated_value"},
			}),
			want: &proto3Message{
				StringValue: "str",
			},
		},
		{
			values: url.Values{
				"nested.nested.bool_value":   {"true"},
				"nested.nested.string_value": {"str"},
				"nested.string_value":        {"str"},
				"string_value":               {"str"},
			},
			filter: utilities.NewDoubleArray([][]string{
				{"nested"},
			}),
			want: &proto3Message{
				StringValue: "str",
			},
		},
		{
			values: url.Values{
				"nested.nested.bool_value":   {"true"},
				"nested.nested.string_value": {"str"},
				"nested.string_value":        {"str"},
				"string_value":               {"str"},
			},
			filter: utilities.NewDoubleArray([][]string{
				{"nested", "nested"},
			}),
			want: &proto3Message{
				Nested: &proto2Message{
					StringValue: proto.String("str"),
				},
				StringValue: "str",
			},
		},
		{
			values: url.Values{
				"nested.nested.bool_value":   {"true"},
				"nested.nested.string_value": {"str"},
				"nested.string_value":        {"str"},
				"string_value":               {"str"},
			},
			filter: utilities.NewDoubleArray([][]string{
				{"nested", "nested", "string_value"},
			}),
			want: &proto3Message{
				Nested: &proto2Message{
					StringValue: proto.String("str"),
					Nested: &proto3Message{
						BoolValue: true,
					},
				},
				StringValue: "str",
			},
		},
	} {
		msg := proto.Clone(spec.want)
		msg.Reset()
		err := runtime.PopulateQueryParameters(msg, spec.values, spec.filter)
		if err != nil {
			t.Errorf("runtime.PoplateQueryParameters(msg, %v, %v) failed with %v; want success", spec.values, spec.filter, err)
			continue
		}
		if got, want := msg, spec.want; !proto.Equal(got, want) {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, %v = %v; want %v", spec.values, spec.filter, got, want)
		}
	}
}

func TestPopulateQueryParametersWithInvalidNestedParameters(t *testing.T) {
	for _, spec := range []struct {
		msg    proto.Message
		values url.Values
		filter *utilities.DoubleArray
	}{
		{
			msg: &proto3Message{},
			values: url.Values{
				"float_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"double_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"int64_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"int32_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"uint64_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"uint32_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"bool_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"string_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"repeated_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"enum_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"enum_value.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
		{
			msg: &proto3Message{},
			values: url.Values{
				"repeated_enum.nested": {"test"},
			},
			filter: utilities.NewDoubleArray(nil),
		},
	} {
		spec.msg.Reset()
		err := runtime.PopulateQueryParameters(spec.msg, spec.values, spec.filter)
		if err == nil {
			t.Errorf("runtime.PopulateQueryParameters(msg, %v, %v) did not fail; want error", spec.values, spec.filter)
		}
	}
}

type proto3Message struct {
	Nested             *proto2Message           `protobuf:"bytes,1,opt,name=nested,json=nested" json:"nested,omitempty"`
	NestedNonNull      proto2Message            `protobuf:"bytes,15,opt,name=nested_non_null,json=nestedNonNull" json:"nested_non_null,omitempty"`
	FloatValue         float32                  `protobuf:"fixed32,2,opt,name=float_value,json=floatValue" json:"float_value,omitempty"`
	DoubleValue        float64                  `protobuf:"fixed64,3,opt,name=double_value,json=doubleValue" json:"double_value,omitempty"`
	Int64Value         int64                    `protobuf:"varint,4,opt,name=int64_value,json=int64Value" json:"int64_value,omitempty"`
	Int32Value         int32                    `protobuf:"varint,5,opt,name=int32_value,json=int32Value" json:"int32_value,omitempty"`
	Uint64Value        uint64                   `protobuf:"varint,6,opt,name=uint64_value,json=uint64Value" json:"uint64_value,omitempty"`
	Uint32Value        uint32                   `protobuf:"varint,7,opt,name=uint32_value,json=uint32Value" json:"uint32_value,omitempty"`
	BoolValue          bool                     `protobuf:"varint,8,opt,name=bool_value,json=boolValue" json:"bool_value,omitempty"`
	StringValue        string                   `protobuf:"bytes,9,opt,name=string_value,json=stringValue" json:"string_value,omitempty"`
	BytesValue         []byte                   `protobuf:"bytes,25,opt,name=bytes_value,json=bytesValue" json:"bytes_value,omitempty"`
	RepeatedValue      []string                 `protobuf:"bytes,10,rep,name=repeated_value,json=repeatedValue" json:"repeated_value,omitempty"`
	EnumValue          EnumValue                `protobuf:"varint,11,opt,name=enum_value,json=enumValue,enum=runtime_test_api.EnumValue" json:"enum_value,omitempty"`
	RepeatedEnum       []EnumValue              `protobuf:"varint,12,rep,packed,name=repeated_enum,json=repeatedEnum,enum=runtime_test_api.EnumValue" json:"repeated_enum,omitempty"`
	TimestampValue     *timestamp.Timestamp     `protobuf:"bytes,16,opt,name=timestamp_value,json=timestampValue" json:"timestamp_value,omitempty"`
	DurationValue      *duration.Duration       `protobuf:"bytes,42,opt,name=duration_value,json=durationValue" json:"duration_value,omitempty"`
	FieldMaskValue     *field_mask.FieldMask    `protobuf:"bytes,27,opt,name=fieldmask_value,json=fieldmaskValue" json:"fieldmask_value,omitempty"`
	OneofValue         proto3Message_OneofValue `protobuf_oneof:"oneof_value"`
	WrapperDoubleValue *wrappers.DoubleValue    `protobuf:"bytes,17,opt,name=wrapper_double_value,json=wrapperDoubleValue" json:"wrapper_double_value,omitempty"`
	WrapperFloatValue  *wrappers.FloatValue     `protobuf:"bytes,18,opt,name=wrapper_float_value,json=wrapperFloatValue" json:"wrapper_float_value,omitempty"`
	WrapperInt64Value  *wrappers.Int64Value     `protobuf:"bytes,19,opt,name=wrapper_int64_value,json=wrapperInt64Value" json:"wrapper_int64_value,omitempty"`
	WrapperInt32Value  *wrappers.Int32Value     `protobuf:"bytes,20,opt,name=wrapper_int32_value,json=wrapperInt32Value" json:"wrapper_int32_value,omitempty"`
	WrapperUInt64Value *wrappers.UInt64Value    `protobuf:"bytes,21,opt,name=wrapper_u_int64_value,json=wrapperUInt64Value" json:"wrapper_u_int64_value,omitempty"`
	WrapperUInt32Value *wrappers.UInt32Value    `protobuf:"bytes,22,opt,name=wrapper_u_int32_value,json=wrapperUInt32Value" json:"wrapper_u_int32_value,omitempty"`
	WrapperBoolValue   *wrappers.BoolValue      `protobuf:"bytes,23,opt,name=wrapper_bool_value,json=wrapperBoolValue" json:"wrapper_bool_value,omitempty"`
	WrapperStringValue *wrappers.StringValue    `protobuf:"bytes,24,opt,name=wrapper_string_value,json=wrapperStringValue" json:"wrapper_string_value,omitempty"`
	WrapperBytesValue  *wrappers.BytesValue     `protobuf:"bytes,26,opt,name=wrapper_bytes_value,json=wrapperBytesValue" json:"wrapper_bytes_value,omitempty"`
	MapValue           map[string]string        `protobuf:"bytes,27,opt,name=map_value,json=mapValue" json:"map_value,omitempty"`
	MapValue2          map[string]int32         `protobuf:"bytes,28,opt,name=map_value2,json=mapValue2" json:"map_value2,omitempty"`
	MapValue3          map[int32]string         `protobuf:"bytes,29,opt,name=map_value3,json=mapValue3" json:"map_value3,omitempty"`
	MapValue4          map[string]int64         `protobuf:"bytes,30,opt,name=map_value4,json=mapValue4" json:"map_value4,omitempty"`
	MapValue5          map[int64]string         `protobuf:"bytes,31,opt,name=map_value5,json=mapValue5" json:"map_value5,omitempty"`
	MapValue6          map[string]uint32        `protobuf:"bytes,32,opt,name=map_value6,json=mapValue6" json:"map_value6,omitempty"`
	MapValue7          map[uint32]string        `protobuf:"bytes,33,opt,name=map_value7,json=mapValue7" json:"map_value7,omitempty"`
	MapValue8          map[string]uint64        `protobuf:"bytes,34,opt,name=map_value8,json=mapValue8" json:"map_value8,omitempty"`
	MapValue9          map[uint64]string        `protobuf:"bytes,35,opt,name=map_value9,json=mapValue9" json:"map_value9,omitempty"`
	MapValue10         map[string]float32       `protobuf:"bytes,36,opt,name=map_value10,json=mapValue10" json:"map_value10,omitempty"`
	MapValue11         map[float32]string       `protobuf:"bytes,37,opt,name=map_value11,json=mapValue11" json:"map_value11,omitempty"`
	MapValue12         map[string]float64       `protobuf:"bytes,38,opt,name=map_value12,json=mapValue12" json:"map_value12,omitempty"`
	MapValue13         map[float64]string       `protobuf:"bytes,39,opt,name=map_value13,json=mapValue13" json:"map_value13,omitempty"`
	MapValue14         map[string]bool          `protobuf:"bytes,40,opt,name=map_value14,json=mapValue14" json:"map_value14,omitempty"`
	MapValue15         map[bool]string          `protobuf:"bytes,41,opt,name=map_value15,json=mapValue15" json:"map_value15,omitempty"`
}

func (m *proto3Message) Reset()         { *m = proto3Message{} }
func (m *proto3Message) String() string { return proto.CompactTextString(m) }
func (*proto3Message) ProtoMessage()    {}

func (m *proto3Message) GetNested() *proto2Message {
	if m != nil {
		return m.Nested
	}
	return nil
}

type proto3Message_OneofValue interface {
	proto3Message_OneofValue()
}

type proto3Message_OneofBoolValue struct {
	OneofBoolValue bool `protobuf:"varint,13,opt,name=oneof_bool_value,json=oneofBoolValue,oneof"`
}
type proto3Message_OneofStringValue struct {
	OneofStringValue string `protobuf:"bytes,14,opt,name=oneof_string_value,json=oneofStringValue,oneof"`
}

func (*proto3Message_OneofBoolValue) proto3Message_OneofValue()   {}
func (*proto3Message_OneofStringValue) proto3Message_OneofValue() {}

func (m *proto3Message) GetOneofValue() proto3Message_OneofValue {
	if m != nil {
		return m.OneofValue
	}
	return nil
}

func (m *proto3Message) GetOneofBoolValue() bool {
	if x, ok := m.GetOneofValue().(*proto3Message_OneofBoolValue); ok {
		return x.OneofBoolValue
	}
	return false
}

func (m *proto3Message) GetOneofStringValue() string {
	if x, ok := m.GetOneofValue().(*proto3Message_OneofStringValue); ok {
		return x.OneofStringValue
	}
	return ""
}

// XXX_OneofFuncs is for the internal use of the proto package.
func (*proto3Message) XXX_OneofFuncs() (func(msg proto.Message, b *proto.Buffer) error, func(msg proto.Message, tag, wire int, b *proto.Buffer) (bool, error), func(msg proto.Message) (n int), []interface{}) {
	return _proto3Message_OneofMarshaler, _proto3Message_OneofUnmarshaler, _proto3Message_OneofSizer, []interface{}{
		(*proto3Message_OneofBoolValue)(nil),
		(*proto3Message_OneofStringValue)(nil),
	}
}

func _proto3Message_OneofMarshaler(msg proto.Message, b *proto.Buffer) error {
	m := msg.(*proto3Message)
	// oneof_value
	switch x := m.OneofValue.(type) {
	case *proto3Message_OneofBoolValue:
		t := uint64(0)
		if x.OneofBoolValue {
			t = 1
		}
		b.EncodeVarint(13<<3 | proto.WireVarint)
		b.EncodeVarint(t)
	case *proto3Message_OneofStringValue:
		b.EncodeVarint(14<<3 | proto.WireBytes)
		b.EncodeStringBytes(x.OneofStringValue)
	case nil:
	default:
		return fmt.Errorf("proto3Message.OneofValue has unexpected type %T", x)
	}
	return nil
}

func _proto3Message_OneofUnmarshaler(msg proto.Message, tag, wire int, b *proto.Buffer) (bool, error) {
	m := msg.(*proto3Message)
	switch tag {
	case 14: // oneof_value.oneof_bool_value
		if wire != proto.WireVarint {
			return true, proto.ErrInternalBadWireType
		}
		x, err := b.DecodeVarint()
		m.OneofValue = &proto3Message_OneofBoolValue{x != 0}
		return true, err
	case 15: // oneof_value.oneof_string_value
		if wire != proto.WireBytes {
			return true, proto.ErrInternalBadWireType
		}
		x, err := b.DecodeStringBytes()
		m.OneofValue = &proto3Message_OneofStringValue{x}
		return true, err
	default:
		return false, nil
	}
}

func _proto3Message_OneofSizer(msg proto.Message) (n int) {
	m := msg.(*proto3Message)
	// oneof_value
	switch x := m.OneofValue.(type) {
	case *proto3Message_OneofBoolValue:
		n += proto.SizeVarint(14<<3 | proto.WireVarint)
		n += 1
	case *proto3Message_OneofStringValue:
		n += proto.SizeVarint(15<<3 | proto.WireBytes)
		n += proto.SizeVarint(uint64(len(x.OneofStringValue)))
		n += len(x.OneofStringValue)
	case nil:
	default:
		panic(fmt.Sprintf("proto: unexpected type %T in oneof", x))
	}
	return n
}

type nativeProto3Message struct {
	NativeTimeValue     *time.Time     `protobuf:"bytes,1,opt,name=native_timestamp_value,json=nativeTimestampValue" json:"native_timestamp_value,omitempty"`
	NativeDurationValue *time.Duration `protobuf:"bytes,2,opt,name=native_duration_value,json=nativeDurationValue" json:"native_duration_value,omitempty"`
}

func (m *nativeProto3Message) Reset()         { *m = nativeProto3Message{} }
func (m *nativeProto3Message) String() string { return proto.CompactTextString(m) }
func (*nativeProto3Message) ProtoMessage()    {}

type proto2Message struct {
	Nested           *proto3Message `protobuf:"bytes,1,opt,name=nested,json=nested" json:"nested,omitempty"`
	FloatValue       *float32       `protobuf:"fixed32,2,opt,name=float_value,json=floatValue" json:"float_value,omitempty"`
	DoubleValue      *float64       `protobuf:"fixed64,3,opt,name=double_value,json=doubleValue" json:"double_value,omitempty"`
	Int64Value       *int64         `protobuf:"varint,4,opt,name=int64_value,json=int64Value" json:"int64_value,omitempty"`
	Int32Value       *int32         `protobuf:"varint,5,opt,name=int32_value,json=int32Value" json:"int32_value,omitempty"`
	Uint64Value      *uint64        `protobuf:"varint,6,opt,name=uint64_value,json=uint64Value" json:"uint64_value,omitempty"`
	Uint32Value      *uint32        `protobuf:"varint,7,opt,name=uint32_value,json=uint32Value" json:"uint32_value,omitempty"`
	BoolValue        *bool          `protobuf:"varint,8,opt,name=bool_value,json=boolValue" json:"bool_value,omitempty"`
	StringValue      *string        `protobuf:"bytes,9,opt,name=string_value,json=stringValue" json:"string_value,omitempty"`
	RepeatedValue    []string       `protobuf:"bytes,10,rep,name=repeated_value,json=repeatedValue" json:"repeated_value,omitempty"`
	EnumValue        EnumValue      `protobuf:"varint,11,opt,name=enum_value,json=enumValue,enum=runtime_test_api.EnumValue" json:"enum_value,omitempty"`
	RepeatedEnum     []EnumValue    `protobuf:"varint,12,rep,packed,name=repeated_enum,json=repeatedEnum,enum=runtime_test_api.EnumValue" json:"repeated_enum,omitempty"`
	XXX_unrecognized []byte         `json:"-"`
}

func (m *proto2Message) Reset()         { *m = proto2Message{} }
func (m *proto2Message) String() string { return proto.CompactTextString(m) }
func (*proto2Message) ProtoMessage()    {}

func (m *proto2Message) GetNested() *proto3Message {
	if m != nil {
		return m.Nested
	}
	return nil
}

func (m *proto2Message) GetFloatValue() float32 {
	if m != nil && m.FloatValue != nil {
		return *m.FloatValue
	}
	return 0
}

func (m *proto2Message) GetDoubleValue() float64 {
	if m != nil && m.DoubleValue != nil {
		return *m.DoubleValue
	}
	return 0
}

func (m *proto2Message) GetInt64Value() int64 {
	if m != nil && m.Int64Value != nil {
		return *m.Int64Value
	}
	return 0
}

func (m *proto2Message) GetInt32Value() int32 {
	if m != nil && m.Int32Value != nil {
		return *m.Int32Value
	}
	return 0
}

func (m *proto2Message) GetUint64Value() uint64 {
	if m != nil && m.Uint64Value != nil {
		return *m.Uint64Value
	}
	return 0
}

func (m *proto2Message) GetUint32Value() uint32 {
	if m != nil && m.Uint32Value != nil {
		return *m.Uint32Value
	}
	return 0
}

func (m *proto2Message) GetBoolValue() bool {
	if m != nil && m.BoolValue != nil {
		return *m.BoolValue
	}
	return false
}

func (m *proto2Message) GetStringValue() string {
	if m != nil && m.StringValue != nil {
		return *m.StringValue
	}
	return ""
}

func (m *proto2Message) GetRepeatedValue() []string {
	if m != nil {
		return m.RepeatedValue
	}
	return nil
}

type EnumValue int32

const (
	EnumValue_X EnumValue = 0
	EnumValue_Y EnumValue = 1
	EnumValue_Z EnumValue = 2
)

var EnumValue_name = map[int32]string{
	0: "EnumValue_X",
	1: "EnumValue_Y",
	2: "EnumValue_Z",
}
var EnumValue_value = map[string]int32{
	"EnumValue_X": 0,
	"EnumValue_Y": 1,
	"EnumValue_Z": 2,
}

func init() {
	proto.RegisterEnum("runtime_test_api.EnumValue", EnumValue_name, EnumValue_value)
}
