package dbus

import (
	"reflect"
	"testing"
)

func TestStoreStringToInterface(t *testing.T) {
	var dest interface{}
	err := Store([]interface{}{"foobar"}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest.(string)
}

func TestStoreVariantToInterface(t *testing.T) {
	src := MakeVariant("foobar")
	var dest interface{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest.(string)
}

func TestStoreMapStringToMapInterface(t *testing.T) {
	src := map[string]string{"foo": "bar"}
	dest := map[string]interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest["foo"].(string)
}

func TestStoreMapVariantToMapInterface(t *testing.T) {
	src := map[string]Variant{"foo": MakeVariant("foobar")}
	dest := map[string]interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest["foo"].(string)
}

func TestStoreSliceStringToSliceInterface(t *testing.T) {
	src := []string{"foo"}
	dest := []interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest[0].(string)
}

func TestStoreSliceVariantToSliceInterface(t *testing.T) {
	src := []Variant{MakeVariant("foo")}
	dest := []interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest[0].(string)
}

func TestStoreSliceVariantToSliceInterfaceMulti(t *testing.T) {
	src := []Variant{MakeVariant("foo"), MakeVariant(int32(1))}
	dest := []interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	_ = dest[0].(string)
	_ = dest[1].(int32)
}

func TestStoreNested(t *testing.T) {
	src := map[string]interface{}{
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
	dest := map[string]interface{}{}
	err := Store([]interface{}{src}, &dest)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(src, dest) {
		t.Errorf("not equal: got '%v', want '%v'",
			dest, src)
	}
}
