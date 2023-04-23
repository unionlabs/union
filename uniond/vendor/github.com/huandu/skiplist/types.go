// Copyright 2011 Huan Du. All rights reserved.
// Licensed under the MIT license that can be found in the LICENSE file.

package skiplist

import (
	"bytes"
	"fmt"
	"reflect"
)

// Key types for all built-in types.
// We can use these type as key type when creating a new skip list.
//
//     list := New(Int) // Use int as key.
const (
	Byte     = byteType
	ByteAsc  = Byte
	ByteDesc = -Byte

	Rune     = runeType
	RuneAsc  = Rune
	RuneDesc = -Rune

	Int     = intType
	IntAsc  = Int
	IntDesc = -Int

	Int8     = int8Type
	Int8Asc  = Int8
	Int8Desc = -Int8

	Int16     = int16Type
	Int16Asc  = Int16
	Int16Desc = -Int16

	Int32     = int32Type
	Int32Asc  = Int32
	Int32Desc = -Int32

	Int64     = int64Type
	Int64Asc  = Int64
	Int64Desc = -Int64

	Uint     = uintType
	UintAsc  = Uint
	UintDesc = -Uint

	Uint8     = uint8Type
	Uint8Asc  = Uint8
	Uint8Desc = -Uint8

	Uint16     = uint16Type
	Uint16Asc  = Uint16
	Uint16Desc = -Uint16

	Uint32     = uint32Type
	Uint32Asc  = Uint32
	Uint32Desc = -Uint32

	Uint64     = uint64Type
	Uint64Asc  = Uint64
	Uint64Desc = -Uint64

	Uintptr     = uintptrType
	UintptrAsc  = Uintptr
	UintptrDesc = -Uintptr

	Float32     = float32Type
	Float32Asc  = Float32
	Float32Desc = -Float32

	Float64     = float64Type
	Float64Asc  = Float64
	Float64Desc = -Float64

	String     = stringType
	StringAsc  = String
	StringDesc = -String

	Bytes     = bytesType
	BytesAsc  = Bytes
	BytesDesc = -Bytes
)

const (
	byteType    = keyType(reflect.Uint8)
	runeType    = keyType(reflect.Int32)
	intType     = keyType(reflect.Int)
	int8Type    = keyType(reflect.Int8)
	int16Type   = keyType(reflect.Int16)
	int32Type   = keyType(reflect.Int32)
	int64Type   = keyType(reflect.Int64)
	uintType    = keyType(reflect.Uint)
	uint8Type   = keyType(reflect.Uint8)
	uint16Type  = keyType(reflect.Uint16)
	uint32Type  = keyType(reflect.Uint32)
	uint64Type  = keyType(reflect.Uint64)
	uintptrType = keyType(reflect.Uintptr)
	float32Type = keyType(reflect.Float32)
	float64Type = keyType(reflect.Float64)
	stringType  = keyType(reflect.String)
	bytesType   = keyType(reflect.Slice)
)

type keyType int

var _ Comparable = keyType(0)

func (kt keyType) kind() (kind reflect.Kind, reversed bool) {
	if kt < 0 {
		reversed = true
		kt = -kt
	}

	kind = reflect.Kind(kt)
	return
}

func (kt keyType) Compare(lhs, rhs interface{}) int {
	val1 := reflect.ValueOf(lhs)
	val2 := reflect.ValueOf(rhs)
	kind, reversed := kt.kind()
	result := compareTypes(val1, val2, kind)

	if reversed {
		result = -result
	}

	return result
}

var typeOfBytes = reflect.TypeOf([]byte(nil))

func compareTypes(lhs, rhs reflect.Value, kind reflect.Kind) int {
	switch kind {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32,
		reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uintptr,
		reflect.Float32, reflect.Float64:
		v1 := calcScore(lhs)
		v2 := calcScore(rhs)

		if v1 > v2 {
			return 1
		}

		if v1 < v2 {
			return -1
		}

		return 0

	case reflect.Int64:
		v1 := lhs.Int()
		v2 := rhs.Int()

		if v1 > v2 {
			return 1
		}

		if v1 < v2 {
			return -1
		}

		return 0

	case reflect.Uint64:
		v1 := lhs.Uint()
		v2 := rhs.Uint()

		if v1 > v2 {
			return 1
		}

		if v1 < v2 {
			return -1
		}

		return 0

	case reflect.String:
		v1 := lhs.String()
		v2 := rhs.String()

		if v1 == v2 {
			return 0
		}

		if v1 > v2 {
			return 1
		}

		return -1

	case reflect.Slice:
		if lhs.Type().ConvertibleTo(typeOfBytes) && rhs.Type().ConvertibleTo(typeOfBytes) {
			bytes1 := lhs.Convert(typeOfBytes).Interface().([]byte)
			bytes2 := rhs.Convert(typeOfBytes).Interface().([]byte)
			return bytes.Compare(bytes1, bytes2)
		}
	}

	panic("never be here")
}

var numberLikeKinds = [...]bool{
	reflect.Int:     true,
	reflect.Int8:    true,
	reflect.Int16:   true,
	reflect.Int32:   true,
	reflect.Int64:   true,
	reflect.Uint:    true,
	reflect.Uint8:   true,
	reflect.Uint16:  true,
	reflect.Uint32:  true,
	reflect.Uint64:  true,
	reflect.Uintptr: true,
	reflect.Float32: true,
	reflect.Float64: true,
	reflect.String:  false,
	reflect.Slice:   false,
}

func (kt keyType) CalcScore(key interface{}) float64 {
	k := reflect.ValueOf(key)
	kind, reversed := kt.kind()

	if kk := k.Kind(); kk != kind {
		// Special case for constant values.
		// It allows us to write code like following without panic.
		//
		//     list := skiplist.New(skiplist.Float64)
		//     list.Set(123, "foo") // 123 is int instead of float64.
		if numberLikeKinds[kind] && (kk == reflect.Int || kk == reflect.Float64) {
			// By pass the check.
		} else {
			name := kind.String()

			if kind == reflect.Slice {
				name = "[]byte"
			}

			panic(fmt.Errorf("skiplist: key type must be %v, but actual type is %v", name, k.Type()))
		}
	}

	score := calcScore(k)

	if reversed {
		score = -score
	}

	return score
}

func calcScore(val reflect.Value) (score float64) {
	switch val.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		score = float64(val.Int())

	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64, reflect.Uintptr:
		score = float64(val.Uint())

	case reflect.Float32, reflect.Float64:
		score = val.Float()

	case reflect.String:
		var hash uint64
		str := val.String()
		l := len(str)

		// only use first 8 bytes
		if l > 8 {
			l = 8
		}

		// Consider str as a Big-Endian uint64.
		for i := 0; i < l; i++ {
			shift := uint(64 - 8 - i*8)
			hash |= uint64(str[i]) << shift
		}

		score = float64(hash)

	case reflect.Slice:
		if val.Type().ConvertibleTo(typeOfBytes) {
			var hash uint64
			data := val.Convert(typeOfBytes).Interface().([]byte)

			l := len(data)

			// only use first 8 bytes
			if l > 8 {
				l = 8
			}

			// Consider str as a Big-Endian uint64.
			for i := 0; i < l; i++ {
				shift := uint(64 - 8 - i*8)
				hash |= uint64(data[i]) << shift
			}

			score = float64(hash)
		}
	}

	return
}
