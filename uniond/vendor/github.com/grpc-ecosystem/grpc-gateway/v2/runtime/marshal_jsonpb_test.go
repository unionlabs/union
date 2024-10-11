package runtime_test

import (
	"bytes"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/grpc-ecosystem/grpc-gateway/v2/runtime/internal/examplepb"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/structpb"
	"google.golang.org/protobuf/types/known/timestamppb"
	"google.golang.org/protobuf/types/known/wrapperspb"
)

func TestJSONPbMarshal(t *testing.T) {
	msg := examplepb.ABitOfEverything{
		SingleNested:        &examplepb.ABitOfEverything_Nested{},
		RepeatedStringValue: []string{},
		MappedStringValue:   map[string]string{},
		MappedNestedValue:   map[string]*examplepb.ABitOfEverything_Nested{},
		RepeatedEnumValue:   []examplepb.NumericEnum{},
		TimestampValue:      &timestamppb.Timestamp{},
		Uuid:                "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
		Nested: []*examplepb.ABitOfEverything_Nested{
			{
				Name:   "foo",
				Amount: 12345,
			},
		},
		Uint64Value: 0xFFFFFFFFFFFFFFFF,
		EnumValue:   examplepb.NumericEnum_ONE,
		OneofValue: &examplepb.ABitOfEverything_OneofString{
			OneofString: "bar",
		},
		MapValue: map[string]examplepb.NumericEnum{
			"a": examplepb.NumericEnum_ONE,
			"b": examplepb.NumericEnum_ZERO,
		},
		RepeatedEnumAnnotation:   []examplepb.NumericEnum{},
		EnumValueAnnotation:      examplepb.NumericEnum_ONE,
		RepeatedStringAnnotation: []string{},
		RepeatedNestedAnnotation: []*examplepb.ABitOfEverything_Nested{},
		NestedAnnotation:         &examplepb.ABitOfEverything_Nested{},
	}

	for i, spec := range []struct {
		useEnumNumbers, emitUnpopulated bool
		indent                          string
		useProtoNames                   bool
		verifier                        func(json string)
	}{
		{
			verifier: func(json string) {
				if !strings.Contains(json, "ONE") {
					t.Errorf(`strings.Contains(%q, "ONE") = false; want true`, json)
				}
				if want := "uint64Value"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			useEnumNumbers: true,
			verifier: func(json string) {
				if strings.Contains(json, "ONE") {
					t.Errorf(`strings.Contains(%q, "ONE") = true; want false`, json)
				}
			},
		},
		{
			emitUnpopulated: true,
			verifier: func(json string) {
				if want := `"sfixed32Value"`; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			indent: "\t\t",
			verifier: func(json string) {
				if want := "\t\t\"amount\":"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			useProtoNames: true,
			verifier: func(json string) {
				if want := "uint64_value"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
	} {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			m := runtime.JSONPb{
				MarshalOptions: protojson.MarshalOptions{
					EmitUnpopulated: spec.emitUnpopulated,
					Indent:          spec.indent,
					UseProtoNames:   spec.useProtoNames,
					UseEnumNumbers:  spec.useEnumNumbers,
				},
			}
			buf, err := m.Marshal(&msg)
			if err != nil {
				t.Errorf("m.Marshal(%v) failed with %v; want success; spec=%v", &msg, err, spec)
			}

			var got examplepb.ABitOfEverything
			unmarshaler := &protojson.UnmarshalOptions{}
			if err = unmarshaler.Unmarshal(buf, &got); err != nil {
				t.Errorf("jsonpb.UnmarshalString(%q, &got) failed with %v; want success; spec=%v", string(buf), err, spec)
			}
			if diff := cmp.Diff(&got, &msg, protocmp.Transform()); diff != "" {
				t.Errorf("case %d: spec=%v; %s", i, spec, diff)
			}
			if spec.verifier != nil {
				spec.verifier(string(buf))
			}
		})
	}
}

func TestJSONPbMarshalFields(t *testing.T) {
	var m runtime.JSONPb
	m.UseEnumNumbers = true // builtin fixtures include an enum, expected to be marshaled as int
	for _, spec := range builtinFieldFixtures {
		buf, err := m.Marshal(spec.data)
		if err != nil {
			t.Errorf("m.Marshal(%#v) failed with %v; want success", spec.data, err)
		}
		if got, want := string(buf), spec.json; got != want {
			t.Errorf("m.Marshal(%#v) = %q; want %q", spec.data, got, want)
		}
	}

	nums := []examplepb.NumericEnum{examplepb.NumericEnum_ZERO, examplepb.NumericEnum_ONE}

	buf, err := m.Marshal(nums)
	if err != nil {
		t.Errorf("m.Marshal(%#v) failed with %v; want success", nums, err)
	}
	if got, want := string(buf), `[0,1]`; got != want {
		t.Errorf("m.Marshal(%#v) = %q; want %q", nums, got, want)
	}

	m.UseEnumNumbers = false
	buf, err = m.Marshal(examplepb.NumericEnum_ONE)
	if err != nil {
		t.Errorf("m.Marshal(%#v) failed with %v; want success", examplepb.NumericEnum_ONE, err)
	}
	if got, want := string(buf), `"ONE"`; got != want {
		t.Errorf("m.Marshal(%#v) = %q; want %q", examplepb.NumericEnum_ONE, got, want)
	}

	buf, err = m.Marshal(nums)
	if err != nil {
		t.Errorf("m.Marshal(%#v) failed with %v; want success", nums, err)
	}
	if got, want := string(buf), `["ZERO","ONE"]`; got != want {
		t.Errorf("m.Marshal(%#v) = %q; want %q", nums, got, want)
	}
}

func TestJSONPbUnmarshal(t *testing.T) {
	var (
		m   runtime.JSONPb
		got examplepb.ABitOfEverything
	)
	for i, data := range []string{
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": 18446744073709551615,
			"enumValue": "ONE",
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": "18446744073709551615",
			"enumValue": "ONE",
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": 18446744073709551615,
			"enumValue": 1,
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
	} {
		if err := m.Unmarshal([]byte(data), &got); err != nil {
			t.Errorf("case %d: m.Unmarshal(%q, &got) failed with %v; want success", i, data, err)
		}

		want := examplepb.ABitOfEverything{
			Uuid: "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			Nested: []*examplepb.ABitOfEverything_Nested{
				{
					Name:   "foo",
					Amount: 12345,
				},
			},
			Uint64Value: 0xFFFFFFFFFFFFFFFF,
			EnumValue:   examplepb.NumericEnum_ONE,
			OneofValue: &examplepb.ABitOfEverything_OneofString{
				OneofString: "bar",
			},
			MapValue: map[string]examplepb.NumericEnum{
				"a": examplepb.NumericEnum_ONE,
				"b": examplepb.NumericEnum_ZERO,
			},
		}

		if diff := cmp.Diff(&got, &want, protocmp.Transform()); diff != "" {
			t.Errorf("case %d: %s", i, diff)
		}
	}
}

func TestJSONPbUnmarshalFields(t *testing.T) {
	var m runtime.JSONPb
	for _, fixt := range fieldFixtures {
		if fixt.skipUnmarshal {
			continue
		}

		dest := reflect.New(reflect.TypeOf(fixt.data))
		if err := m.Unmarshal([]byte(fixt.json), dest.Interface()); err != nil {
			t.Errorf("m.Unmarshal(%q, %T) failed with %v; want success", fixt.json, dest.Interface(), err)
		}
		if diff := cmp.Diff(dest.Elem().Interface(), fixt.data, protocmp.Transform()); diff != "" {
			t.Errorf("dest = %#v; want %#v; input = %v", dest.Elem().Interface(), fixt.data, fixt.json)
		}
	}
}

func TestJSONPbEncoder(t *testing.T) {
	msg := examplepb.ABitOfEverything{
		SingleNested:        &examplepb.ABitOfEverything_Nested{},
		RepeatedStringValue: []string{},
		MappedStringValue:   map[string]string{},
		MappedNestedValue:   map[string]*examplepb.ABitOfEverything_Nested{},
		RepeatedEnumValue:   []examplepb.NumericEnum{},
		TimestampValue:      &timestamppb.Timestamp{},
		Uuid:                "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
		Nested: []*examplepb.ABitOfEverything_Nested{
			{
				Name:   "foo",
				Amount: 12345,
			},
		},
		Uint64Value: 0xFFFFFFFFFFFFFFFF,
		OneofValue: &examplepb.ABitOfEverything_OneofString{
			OneofString: "bar",
		},
		MapValue: map[string]examplepb.NumericEnum{
			"a": examplepb.NumericEnum_ONE,
			"b": examplepb.NumericEnum_ZERO,
		},
		RepeatedEnumAnnotation:   []examplepb.NumericEnum{},
		EnumValueAnnotation:      examplepb.NumericEnum_ONE,
		RepeatedStringAnnotation: []string{},
		RepeatedNestedAnnotation: []*examplepb.ABitOfEverything_Nested{},
		NestedAnnotation:         &examplepb.ABitOfEverything_Nested{},
	}

	for i, spec := range []struct {
		useEnumNumbers, emitUnpopulated bool
		indent                          string
		useProtoNames                   bool
		verifier                        func(json string)
	}{
		{
			verifier: func(json string) {
				if !strings.Contains(json, "ONE") {
					t.Errorf(`strings.Contains(%q, "ONE") = false; want true`, json)
				}
				if want := "uint64Value"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			useEnumNumbers: true,
			verifier: func(json string) {
				if strings.Contains(json, "ONE") {
					t.Errorf(`strings.Contains(%q, "ONE") = true; want false`, json)
				}
			},
		},
		{
			emitUnpopulated: true,
			verifier: func(json string) {
				if want := `"sfixed32Value"`; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			indent: "\t\t",
			verifier: func(json string) {
				if want := "\t\t\"amount\":"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
		{
			useProtoNames: true,
			verifier: func(json string) {
				if want := "uint64_value"; !strings.Contains(json, want) {
					t.Errorf(`strings.Contains(%q, %q) = false; want true`, json, want)
				}
			},
		},
	} {
		m := runtime.JSONPb{
			MarshalOptions: protojson.MarshalOptions{
				EmitUnpopulated: spec.emitUnpopulated,
				Indent:          spec.indent,
				UseProtoNames:   spec.useProtoNames,
				UseEnumNumbers:  spec.useEnumNumbers,
			},
		}

		var buf bytes.Buffer
		enc := m.NewEncoder(&buf)
		if err := enc.Encode(&msg); err != nil {
			t.Errorf("enc.Encode(%v) failed with %v; want success; spec=%v", &msg, err, spec)
		}

		var got examplepb.ABitOfEverything
		unmarshaler := &protojson.UnmarshalOptions{}
		if err := unmarshaler.Unmarshal(buf.Bytes(), &got); err != nil {
			t.Errorf("jsonpb.UnmarshalString(%q, &got) failed with %v; want success; spec=%v", buf.String(), err, spec)
		}
		if diff := cmp.Diff(&got, &msg, protocmp.Transform()); diff != "" {
			t.Errorf("case %d: %s", i, diff)
		}
		if spec.verifier != nil {
			spec.verifier(buf.String())
		}
	}
}

func TestJSONPbEncoderFields(t *testing.T) {
	var m runtime.JSONPb
	for _, fixt := range fieldFixtures {
		var buf bytes.Buffer
		enc := m.NewEncoder(&buf)
		if err := enc.Encode(fixt.data); err != nil {
			t.Errorf("enc.Encode(%#v) failed with %v; want success", fixt.data, err)
		}
		if got, want := buf.String(), fixt.json+string(m.Delimiter()); got != want {
			t.Errorf("enc.Encode(%#v) = %q; want %q", fixt.data, got, want)
		}
	}

	m.UseEnumNumbers = true
	buf, err := m.Marshal(examplepb.NumericEnum_ONE)
	if err != nil {
		t.Errorf("m.Marshal(%#v) failed with %v; want success", examplepb.NumericEnum_ONE, err)
	}
	if got, want := string(buf), "1"; got != want {
		t.Errorf("m.Marshal(%#v) = %q; want %q", examplepb.NumericEnum_ONE, got, want)
	}
}

func TestJSONPbDecoder(t *testing.T) {
	var (
		m   runtime.JSONPb
		got examplepb.ABitOfEverything
	)
	for _, data := range []string{
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": 18446744073709551615,
			"enumValue": "ONE",
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": "18446744073709551615",
			"enumValue": "ONE",
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
		`{
			"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			"nested": [
				{"name": "foo", "amount": 12345}
			],
			"uint64Value": 18446744073709551615,
			"enumValue": 1,
			"oneofString": "bar",
			"mapValue": {
				"a": 1,
				"b": 0
			}
		}`,
	} {
		r := strings.NewReader(data)
		dec := m.NewDecoder(r)
		if err := dec.Decode(&got); err != nil {
			t.Errorf("m.Unmarshal(&got) failed with %v; want success; data=%q", err, data)
		}

		want := examplepb.ABitOfEverything{
			Uuid: "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
			Nested: []*examplepb.ABitOfEverything_Nested{
				{
					Name:   "foo",
					Amount: 12345,
				},
			},
			Uint64Value: 0xFFFFFFFFFFFFFFFF,
			EnumValue:   examplepb.NumericEnum_ONE,
			OneofValue: &examplepb.ABitOfEverything_OneofString{
				OneofString: "bar",
			},
			MapValue: map[string]examplepb.NumericEnum{
				"a": examplepb.NumericEnum_ONE,
				"b": examplepb.NumericEnum_ZERO,
			},
		}
		if diff := cmp.Diff(&got, &want, protocmp.Transform()); diff != "" {
			t.Errorf("data %q: %s", data, diff)
		}
	}
}

func TestJSONPbDecoderFields(t *testing.T) {
	var m runtime.JSONPb
	for _, fixt := range fieldFixtures {
		if fixt.skipUnmarshal {
			continue
		}

		dest := reflect.New(reflect.TypeOf(fixt.data))
		dec := m.NewDecoder(strings.NewReader(fixt.json))
		if err := dec.Decode(dest.Interface()); err != nil {
			t.Errorf("dec.Decode(%T) failed with %v; want success; input = %q", dest.Interface(), err, fixt.json)
		}
		if got, want := dest.Elem().Interface(), fixt.data; !reflect.DeepEqual(got, want) {
			t.Errorf("dest = %#v; want %#v; input = %v", got, want, fixt.json)
		}
	}
}

func TestJSONPbDecoderUnknownField(t *testing.T) {
	var (
		m = runtime.JSONPb{
			UnmarshalOptions: protojson.UnmarshalOptions{
				DiscardUnknown: false,
			},
		}
		got examplepb.ABitOfEverything
	)
	data := `{
		"uuid": "6EC2446F-7E89-4127-B3E6-5C05E6BECBA7",
		"unknownField": "111"
	}`

	r := strings.NewReader(data)
	dec := m.NewDecoder(r)
	if err := dec.Decode(&got); err == nil {
		t.Errorf("m.Unmarshal(&got) not failed; want `unknown field` error; data=%q", data)
	}
}

var (
	fieldFixtures = []struct {
		data          interface{}
		json          string
		skipUnmarshal bool
	}{
		{data: int32(1), json: "1"},
		{data: proto.Int32(1), json: "1"},
		{data: int64(1), json: "1"},
		{data: proto.Int64(1), json: "1"},
		{data: uint32(1), json: "1"},
		{data: proto.Uint32(1), json: "1"},
		{data: uint64(1), json: "1"},
		{data: proto.Uint64(1), json: "1"},
		{data: "abc", json: `"abc"`},
		{data: []byte("abc"), json: `"YWJj"`},
		{data: []byte{}, json: `""`},
		{data: proto.String("abc"), json: `"abc"`},
		{data: float32(1.5), json: "1.5"},
		{data: proto.Float32(1.5), json: "1.5"},
		{data: float64(1.5), json: "1.5"},
		{data: proto.Float64(1.5), json: "1.5"},
		{data: true, json: "true"},
		{data: false, json: "false"},
		{data: (*string)(nil), json: "null"},
		{
			data: examplepb.NumericEnum_ONE,
			json: `"ONE"`,
			// TODO(yugui) support unmarshaling of symbolic enum
			skipUnmarshal: true,
		},
		{
			data: (*examplepb.NumericEnum)(proto.Int32(int32(examplepb.NumericEnum_ONE))),
			json: `"ONE"`,
			// TODO(yugui) support unmarshaling of symbolic enum
			skipUnmarshal: true,
		},

		{
			data: map[string]int32{
				"foo": 1,
			},
			json: `{"foo":1}`,
		},
		{
			data: map[string]*examplepb.SimpleMessage{
				"foo": {Id: "bar"},
			},
			json: `{"foo":{"id":"bar"}}`,
		},
		{
			data: map[int32]*examplepb.SimpleMessage{
				1: {Id: "foo"},
			},
			json: `{"1":{"id":"foo"}}`,
		},
		{
			data: map[bool]*examplepb.SimpleMessage{
				true: {Id: "foo"},
			},
			json: `{"true":{"id":"foo"}}`,
		},
		{
			data: &durationpb.Duration{
				Seconds: 123,
				Nanos:   456000000,
			},
			json: `"123.456s"`,
		},
		{
			data: &timestamppb.Timestamp{
				Seconds: 1462875553,
				Nanos:   123000000,
			},
			json: `"2016-05-10T10:19:13.123Z"`,
		},
		{
			data: new(emptypb.Empty),
			json: "{}",
		},
		{
			data: &structpb.Value{
				Kind: new(structpb.Value_NullValue),
			},
			json:          "null",
			skipUnmarshal: true,
		},
		{
			data: &structpb.Value{
				Kind: &structpb.Value_NumberValue{
					NumberValue: 123.4,
				},
			},
			json:          "123.4",
			skipUnmarshal: true,
		},
		{
			data: &structpb.Value{
				Kind: &structpb.Value_StringValue{
					StringValue: "abc",
				},
			},
			json:          `"abc"`,
			skipUnmarshal: true,
		},
		{
			data: &structpb.Value{
				Kind: &structpb.Value_BoolValue{
					BoolValue: true,
				},
			},
			json:          "true",
			skipUnmarshal: true,
		},
		{
			data: &structpb.Struct{
				Fields: map[string]*structpb.Value{
					"foo_bar": {
						Kind: &structpb.Value_BoolValue{
							BoolValue: true,
						},
					},
				},
			},
			json:          `{"foo_bar":true}`,
			skipUnmarshal: true,
		},

		{
			data: wrapperspb.Bool(true),
			json: "true",
		},
		{
			data: wrapperspb.Double(123.456),
			json: "123.456",
		},
		{
			data: wrapperspb.Float(123.456),
			json: "123.456",
		},
		{
			data: wrapperspb.Int32(-123),
			json: "-123",
		},
		{
			data: wrapperspb.Int64(-123),
			json: `"-123"`,
		},
		{
			data: wrapperspb.UInt32(123),
			json: "123",
		},
		{
			data: wrapperspb.UInt64(123),
			json: `"123"`,
		},
		// TODO(yugui) Add other well-known types once jsonpb supports them
	}
)

func TestJSONPbUnmarshalNullField(t *testing.T) {
	var out map[string]interface{}

	const json = `{"foo": null}`
	marshaler := &runtime.JSONPb{}
	if err := marshaler.Unmarshal([]byte(json), &out); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	value, hasKey := out["foo"]
	if !hasKey {
		t.Fatalf("unmarshaled map did not have key 'foo'")
	}
	if value != nil {
		t.Fatalf("unexpected value: %v", value)
	}
}

func TestJSONPbMarshalResponseBodies(t *testing.T) {
	marshaler := &runtime.JSONPb{}
	for i, spec := range []struct {
		input           interface{}
		emitUnpopulated bool
		verifier        func(*testing.T, interface{}, []byte)
	}{
		{
			input: &examplepb.ResponseBodyOut{
				Response: &examplepb.ResponseBodyOut_Response{Data: "abcdef"},
			},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out examplepb.ResponseBodyOut
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, &out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			emitUnpopulated: true,
			input:           &examplepb.ResponseBodyOut{},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out examplepb.ResponseBodyOut
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, &out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: &examplepb.RepeatedResponseBodyOut_Response{},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, &out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			emitUnpopulated: true,
			input:           &examplepb.RepeatedResponseBodyOut_Response{},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, &out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: ([]*examplepb.RepeatedResponseBodyOut_Response)(nil),
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []*examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			emitUnpopulated: true,
			input:           ([]*examplepb.RepeatedResponseBodyOut_Response)(nil),
			verifier: func(t *testing.T, _ interface{}, json []byte) {
				var out []*examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff([]*examplepb.RepeatedResponseBodyOut_Response{}, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: []*examplepb.RepeatedResponseBodyOut_Response{},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []*examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: []string{"something"},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []string
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: []string{},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []string
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: ([]string)(nil),
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []string
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			emitUnpopulated: true,
			input:           ([]string)(nil),
			verifier: func(t *testing.T, _ interface{}, json []byte) {
				var out []string
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff([]string{}, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			input: []*examplepb.RepeatedResponseBodyOut_Response{
				{},
				{
					Data: "abc",
					Type: examplepb.RepeatedResponseBodyOut_Response_A,
				},
			},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []*examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
		{
			emitUnpopulated: true,
			input: []*examplepb.RepeatedResponseBodyOut_Response{
				{},
				{
					Data: "abc",
					Type: examplepb.RepeatedResponseBodyOut_Response_B,
				},
			},
			verifier: func(t *testing.T, input interface{}, json []byte) {
				var out []*examplepb.RepeatedResponseBodyOut_Response
				err := marshaler.Unmarshal(json, &out)
				if err != nil {
					t.Fatalf("unexpected error: %v", err)
				}
				diff := cmp.Diff(input, out, protocmp.Transform())
				if diff != "" {
					t.Errorf("json not equal:\n%s", diff)
				}
			},
		},
	} {

		t.Run(strconv.Itoa(i), func(t *testing.T) {
			m := runtime.JSONPb{
				MarshalOptions: protojson.MarshalOptions{
					EmitUnpopulated: spec.emitUnpopulated,
				},
			}
			val := spec.input
			buf, err := m.Marshal(val)
			if err != nil {
				t.Errorf("m.Marshal(%v) failed with %v; want success; spec=%v", val, err, spec)
			}
			if spec.verifier != nil {
				spec.verifier(t, spec.input, buf)
			}
		})
	}
}
