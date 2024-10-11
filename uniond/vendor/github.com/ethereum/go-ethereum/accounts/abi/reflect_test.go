// Copyright 2019 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package abi

import (
	"math/big"
	"reflect"
	"testing"
)

type reflectTest struct {
	name  string
	args  []string
	struc interface{}
	want  map[string]string
	err   string
}

var reflectTests = []reflectTest{
	{
		name: "OneToOneCorrespondence",
		args: []string{"fieldA"},
		struc: struct {
			FieldA int `abi:"fieldA"`
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
		},
	},
	{
		name: "MissingFieldsInStruct",
		args: []string{"fieldA", "fieldB"},
		struc: struct {
			FieldA int `abi:"fieldA"`
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
		},
	},
	{
		name: "MoreFieldsInStructThanArgs",
		args: []string{"fieldA"},
		struc: struct {
			FieldA int `abi:"fieldA"`
			FieldB int
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
		},
	},
	{
		name: "MissingFieldInArgs",
		args: []string{"fieldA"},
		struc: struct {
			FieldA int `abi:"fieldA"`
			FieldB int `abi:"fieldB"`
		}{},
		err: "struct: abi tag 'fieldB' defined but not found in abi",
	},
	{
		name: "NoAbiDescriptor",
		args: []string{"fieldA"},
		struc: struct {
			FieldA int
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
		},
	},
	{
		name: "NoArgs",
		args: []string{},
		struc: struct {
			FieldA int `abi:"fieldA"`
		}{},
		err: "struct: abi tag 'fieldA' defined but not found in abi",
	},
	{
		name: "DifferentName",
		args: []string{"fieldB"},
		struc: struct {
			FieldA int `abi:"fieldB"`
		}{},
		want: map[string]string{
			"fieldB": "FieldA",
		},
	},
	{
		name: "DifferentName",
		args: []string{"fieldB"},
		struc: struct {
			FieldA int `abi:"fieldB"`
		}{},
		want: map[string]string{
			"fieldB": "FieldA",
		},
	},
	{
		name: "MultipleFields",
		args: []string{"fieldA", "fieldB"},
		struc: struct {
			FieldA int `abi:"fieldA"`
			FieldB int `abi:"fieldB"`
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
			"fieldB": "FieldB",
		},
	},
	{
		name: "MultipleFieldsABIMissing",
		args: []string{"fieldA", "fieldB"},
		struc: struct {
			FieldA int `abi:"fieldA"`
			FieldB int
		}{},
		want: map[string]string{
			"fieldA": "FieldA",
			"fieldB": "FieldB",
		},
	},
	{
		name: "NameConflict",
		args: []string{"fieldB"},
		struc: struct {
			FieldA int `abi:"fieldB"`
			FieldB int
		}{},
		err: "abi: multiple variables maps to the same abi field 'fieldB'",
	},
	{
		name: "Underscored",
		args: []string{"_"},
		struc: struct {
			FieldA int
		}{},
		err: "abi: purely underscored output cannot unpack to struct",
	},
	{
		name: "DoubleMapping",
		args: []string{"fieldB", "fieldC", "fieldA"},
		struc: struct {
			FieldA int `abi:"fieldC"`
			FieldB int
		}{},
		err: "abi: multiple outputs mapping to the same struct field 'FieldA'",
	},
	{
		name: "AlreadyMapped",
		args: []string{"fieldB", "fieldB"},
		struc: struct {
			FieldB int `abi:"fieldB"`
		}{},
		err: "struct: abi tag in 'FieldB' already mapped",
	},
}

func TestReflectNameToStruct(t *testing.T) {
	for _, test := range reflectTests {
		t.Run(test.name, func(t *testing.T) {
			m, err := mapArgNamesToStructFields(test.args, reflect.ValueOf(test.struc))
			if len(test.err) > 0 {
				if err == nil || err.Error() != test.err {
					t.Fatalf("Invalid error: expected %v, got %v", test.err, err)
				}
			} else {
				if err != nil {
					t.Fatalf("Unexpected error: %v", err)
				}
				for fname := range test.want {
					if m[fname] != test.want[fname] {
						t.Fatalf("Incorrect value for field %s: expected %v, got %v", fname, test.want[fname], m[fname])
					}
				}
			}
		})
	}
}

func TestConvertType(t *testing.T) {
	// Test Basic Struct
	type T struct {
		X *big.Int
		Y *big.Int
	}
	// Create on-the-fly structure
	var fields []reflect.StructField
	fields = append(fields, reflect.StructField{
		Name: "X",
		Type: reflect.TypeOf(new(big.Int)),
		Tag:  "json:\"" + "x" + "\"",
	})
	fields = append(fields, reflect.StructField{
		Name: "Y",
		Type: reflect.TypeOf(new(big.Int)),
		Tag:  "json:\"" + "y" + "\"",
	})
	val := reflect.New(reflect.StructOf(fields))
	val.Elem().Field(0).Set(reflect.ValueOf(big.NewInt(1)))
	val.Elem().Field(1).Set(reflect.ValueOf(big.NewInt(2)))
	// ConvertType
	out := *ConvertType(val.Interface(), new(T)).(*T)
	if out.X.Cmp(big.NewInt(1)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out.X, big.NewInt(1))
	}
	if out.Y.Cmp(big.NewInt(2)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out.Y, big.NewInt(2))
	}
	// Slice Type
	val2 := reflect.MakeSlice(reflect.SliceOf(reflect.StructOf(fields)), 2, 2)
	val2.Index(0).Field(0).Set(reflect.ValueOf(big.NewInt(1)))
	val2.Index(0).Field(1).Set(reflect.ValueOf(big.NewInt(2)))
	val2.Index(1).Field(0).Set(reflect.ValueOf(big.NewInt(3)))
	val2.Index(1).Field(1).Set(reflect.ValueOf(big.NewInt(4)))
	out2 := *ConvertType(val2.Interface(), new([]T)).(*[]T)
	if out2[0].X.Cmp(big.NewInt(1)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out2[0].X, big.NewInt(1))
	}
	if out2[0].Y.Cmp(big.NewInt(2)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out2[1].Y, big.NewInt(2))
	}
	if out2[1].X.Cmp(big.NewInt(3)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out2[0].X, big.NewInt(1))
	}
	if out2[1].Y.Cmp(big.NewInt(4)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out2[1].Y, big.NewInt(2))
	}
	// Array Type
	val3 := reflect.New(reflect.ArrayOf(2, reflect.StructOf(fields)))
	val3.Elem().Index(0).Field(0).Set(reflect.ValueOf(big.NewInt(1)))
	val3.Elem().Index(0).Field(1).Set(reflect.ValueOf(big.NewInt(2)))
	val3.Elem().Index(1).Field(0).Set(reflect.ValueOf(big.NewInt(3)))
	val3.Elem().Index(1).Field(1).Set(reflect.ValueOf(big.NewInt(4)))
	out3 := *ConvertType(val3.Interface(), new([2]T)).(*[2]T)
	if out3[0].X.Cmp(big.NewInt(1)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out3[0].X, big.NewInt(1))
	}
	if out3[0].Y.Cmp(big.NewInt(2)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out3[1].Y, big.NewInt(2))
	}
	if out3[1].X.Cmp(big.NewInt(3)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out3[0].X, big.NewInt(1))
	}
	if out3[1].Y.Cmp(big.NewInt(4)) != 0 {
		t.Errorf("ConvertType failed, got %v want %v", out3[1].Y, big.NewInt(2))
	}
}
