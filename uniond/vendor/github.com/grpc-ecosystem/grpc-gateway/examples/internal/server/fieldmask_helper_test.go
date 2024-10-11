package server

import (
	"reflect"
	"testing"

	"google.golang.org/genproto/protobuf/field_mask"
)

func TestApplyFieldMask(t *testing.T) {
	for _, test := range []struct {
		name      string
		patchee   interface{}
		patcher   interface{}
		fieldMask *field_mask.FieldMask
		expected  interface{}
	}{
		{"nil fieldMask", &a{E: 64}, &a{E: 42}, nil, &a{E: 64}},
		{"empty paths", &a{E: 63}, &a{E: 42}, &field_mask.FieldMask{}, &a{E: 63}},
		{"simple path",
			&a{E: 23, Foo: "test"},
			&a{BeefCake: &b{}, E: 42},
			&field_mask.FieldMask{Paths: []string{"e"}},
			&a{E: 42, Foo: "test"}},
		{"nested",
			&a{BeefCake: &b{CowCount: 85}},
			&a{BeefCake: &b{CowCount: 58, Data: nil}},
			&field_mask.FieldMask{Paths: []string{"beef_cake.cow_count"}},
			&a{BeefCake: &b{CowCount: 58}}},
		{"multiple paths",
			&a{BeefCake: &b{CowCount: 40, Data: []int{1, 2, 3}}, E: 34, Foo: "catapult"},
			&a{BeefCake: &b{CowCount: 56}, Foo: "lettuce"},
			&field_mask.FieldMask{Paths: []string{"beef_cake.cow_count", "foo"}},
			&a{BeefCake: &b{CowCount: 56, Data: []int{1, 2, 3}}, E: 34, Foo: "lettuce"}},
	} {
		t.Run(test.name, func(t *testing.T) {
			applyFieldMask(test.patchee, test.patcher, test.fieldMask)
			if !reflect.DeepEqual(test.patchee, test.expected) {
				t.Errorf("expected %v, but was %v", test.expected, test.patchee)
			}
		})
	}
}

func TestGetValue(t *testing.T) {
	for _, test := range []struct {
		name     string
		input    interface{}
		path     string
		expected interface{}
	}{
		{"empty", &a{E: 45, Foo: "test"}, "", &a{E: 45, Foo: "test"}},
		{"pointer-simple", &a{E: 45}, "E", 45},
		{"pointer-nested", &a{BeefCake: &b{CowCount: 42}}, "beef_cake.cow_count", 42},
		{"pointer-complex type", &a{BeefCake: &b{Data: []int{1, 2}}}, "beef_cake.data", []int{1, 2}},
		{"pointer-invalid path", &a{Foo: "test"}, "x.y", nil},
		{"simple", a{E: 45}, "E", 45},
		{"nested", a{BeefCake: &b{CowCount: 42}}, "beef_cake.cow_count", 42},
		{"complex type", a{BeefCake: &b{Data: []int{1, 2}}}, "beef_cake.data", []int{1, 2}},
		{"invalid path", a{Foo: "test"}, "X.Y", nil},
	} {
		t.Run(test.name, func(t *testing.T) {
			if actual := getField(test.input, test.path); actual.IsValid() {
				if !reflect.DeepEqual(test.expected, actual.Interface()) {
					t.Errorf("expected %v, but got %v", test.expected, actual)
				}
			} else if test.expected != nil {
				t.Errorf("expected nil, but was %v", actual)
			}
		})
	}
}

func TestSetValue(t *testing.T) {
	for _, test := range []struct {
		name     string
		obj      interface{}
		newValue interface{}
		path     string
		expected interface{}
	}{
		{"simple", &a{E: 45}, 34, "e", 34},
		{"nested", &a{BeefCake: &b{CowCount: 54}}, 43, "beef_cake.cow_count", 43},
		{"complex type", &a{BeefCake: &b{Data: []int{1, 2}}}, []int{3, 4}, "beef_cake.data", []int{3, 4}},
	} {
		t.Run(test.name, func(t *testing.T) {
			setValue(test.obj, reflect.ValueOf(test.newValue), test.path)
			if actual := getField(test.obj, test.path).Interface(); !reflect.DeepEqual(actual, test.expected) {
				t.Errorf("expected %v, but got %v", test.newValue, actual)
			}
		})
	}
}

type a struct {
	BeefCake *b
	E        int
	Foo      string
}

type b struct {
	CowCount int
	Data     []int
}
