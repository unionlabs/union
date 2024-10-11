package dbus

import (
	"bytes"
	"encoding/binary"
	"reflect"
	"testing"
)

func TestEncodeArrayOfMaps(t *testing.T) {
	tests := []struct {
		name string
		vs   []interface{}
	}{
		{
			"aligned at 8 at start of array",
			[]interface{}{
				"12345",
				[]map[string]Variant{
					{
						"abcdefg": MakeVariant("foo"),
						"cdef":    MakeVariant(uint32(2)),
					},
				},
			},
		},
		{
			"not aligned at 8 for start of array",
			[]interface{}{
				"1234567890",
				[]map[string]Variant{
					{
						"abcdefg": MakeVariant("foo"),
						"cdef":    MakeVariant(uint32(2)),
					},
				},
			},
		},
	}
	for _, order := range []binary.ByteOrder{binary.LittleEndian, binary.BigEndian} {
		for _, tt := range tests {
			buf := new(bytes.Buffer)
			enc := newEncoder(buf, order)
			enc.Encode(tt.vs...)

			dec := newDecoder(buf, order)
			v, err := dec.Decode(SignatureOf(tt.vs...))
			if err != nil {
				t.Errorf("%q: decode (%v) failed: %v", tt.name, order, err)
				continue
			}
			if !reflect.DeepEqual(v, tt.vs) {
				t.Errorf("%q: (%v) not equal: got '%v', want '%v'", tt.name, order, v, tt.vs)
				continue
			}
		}
	}
}

func TestEncodeMapStringInterface(t *testing.T) {
	val := map[string]interface{}{"foo": "bar"}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := map[string]interface{}{}
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

type empty interface{}

func TestEncodeMapStringNamedInterface(t *testing.T) {
	val := map[string]empty{"foo": "bar"}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := map[string]empty{}
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

type fooer interface {
	Foo()
}

type fooimpl string

func (fooimpl) Foo() {}

func TestEncodeMapStringNonEmptyInterface(t *testing.T) {
	val := map[string]fooer{"foo": fooimpl("bar")}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := map[string]fooer{}
	err = Store(v, &out)
	if err == nil {
		t.Fatal("Shouldn't be able to convert to non empty interfaces")
	}
}

func TestEncodeSliceInterface(t *testing.T) {
	val := []interface{}{"foo", "bar"}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := []interface{}{}
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

func TestEncodeSliceNamedInterface(t *testing.T) {
	val := []empty{"foo", "bar"}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := []empty{}
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

func TestEncodeNestedInterface(t *testing.T) {
	val := map[string]interface{}{
		"foo": []interface{}{"1", "2", "3", "5",
			map[string]interface{}{
				"bar": "baz",
			},
		},
		"bar": map[string]interface{}{
			"baz":  "quux",
			"quux": "quuz",
		},
	}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	out := map[string]interface{}{}
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%#v', want '%#v'",
			out, val)
	}
}

func TestEncodeInt(t *testing.T) {
	val := 10
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	var out int
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

func TestEncodeIntToNonCovertible(t *testing.T) {
	val := 150
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	var out bool
	err = Store(v, &out)
	if err == nil {
		t.Logf("%t\n", out)
		t.Fatal("Type mismatch should have occured")
	}
}

func TestEncodeUint(t *testing.T) {
	val := uint(10)
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	var out uint
	Store(v, &out)
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

func TestEncodeUintToNonCovertible(t *testing.T) {
	val := uint(10)
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	var out bool
	err = Store(v, &out)
	if err == nil {
		t.Fatal("Type mismatch should have occured")
	}
}

type boolean bool

func TestEncodeOfAssignableType(t *testing.T) {
	val := boolean(true)
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(val)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(val))
	if err != nil {
		t.Fatal(err)
	}
	var out boolean
	err = Store(v, &out)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(out, val) {
		t.Errorf("not equal: got '%v', want '%v'",
			out, val)
	}
}

func TestEncodeVariant(t *testing.T) {
	var res map[ObjectPath]map[string]map[string]Variant
	var src = map[ObjectPath]map[string]map[string]Variant{
		ObjectPath("/foo/bar"): {
			"foo": {
				"bar": MakeVariant(10),
				"baz": MakeVariant("20"),
			},
		},
	}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(src)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(src))
	if err != nil {
		t.Fatal(err)
	}
	err = Store(v, &res)
	if err != nil {
		t.Fatal(err)
	}
	_ = res[ObjectPath("/foo/bar")]["foo"]["baz"].Value().(string)
}

func TestEncodeVariantToList(t *testing.T) {
	var res map[string]Variant
	var src = map[string]interface{}{
		"foo": []interface{}{"a", "b", "c"},
	}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(src)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(src))
	if err != nil {
		t.Fatal(err)
	}
	err = Store(v, &res)
	if err != nil {
		t.Fatal(err)
	}
	_ = res["foo"].Value().([]Variant)
}

func TestEncodeVariantToUint64(t *testing.T) {
	var res map[string]Variant
	var src = map[string]interface{}{
		"foo": uint64(10),
	}
	buf := new(bytes.Buffer)
	order := binary.LittleEndian
	enc := newEncoder(buf, binary.LittleEndian)
	err := enc.Encode(src)
	if err != nil {
		t.Fatal(err)
	}

	dec := newDecoder(buf, order)
	v, err := dec.Decode(SignatureOf(src))
	if err != nil {
		t.Fatal(err)
	}
	err = Store(v, &res)
	if err != nil {
		t.Fatal(err)
	}
	_ = res["foo"].Value().(uint64)
}
