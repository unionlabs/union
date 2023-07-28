package amino

import (
	"fmt"
	"reflect"
)

//----------------------------------------
// DeepCopy

// Deeply copies an object.
// If anything implements `.DeepCopy() <any>` along the way,
// the result of that function will be used.
// Otherwise, if it implements `.MarshalAmino() (<any>, error)` and
// `.UnmarshalAmino(<any>) error`, the pair will be used to copy.
// If .MarshalAmino() or .UnmarshalAmino() returns an error, this
// function will panic.
func DeepCopy(o interface{}) (r interface{}) {
	if o == nil {
		return nil
	}
	src := reflect.ValueOf(o)
	dst := reflect.New(src.Type()).Elem()
	deepCopy(src, dst)
	return dst.Interface()
}

func deepCopy(src, dst reflect.Value) {
	if isNil(src) {
		return
	}
	if callDeepCopy(src, dst) {
		return
	}
	if callAminoCopy(src, dst) {
		return
	}
	_deepCopy(src, dst)
}

func _deepCopy(src, dst reflect.Value) {

	switch src.Kind() {
	case reflect.Ptr:
		cpy := reflect.New(src.Type().Elem())
		_deepCopy(src.Elem(), cpy.Elem())
		dst.Set(cpy)
		return

	case reflect.Interface:
		cpy := reflect.New(src.Elem().Type())
		deepCopy(src.Elem(), cpy.Elem())
		dst.Set(cpy.Elem())
		return

	case reflect.Array:
		switch src.Type().Elem().Kind() {
		case reflect.Int64, reflect.Int32, reflect.Int16,
			reflect.Int8, reflect.Int, reflect.Uint64,
			reflect.Uint32, reflect.Uint16, reflect.Uint8,
			reflect.Uint, reflect.Bool, reflect.Float64,
			reflect.Float32, reflect.String:

			reflect.Copy(dst, src)
			return
		default:
			for i := 0; i < src.Type().Len(); i++ {
				esrc := src.Index(i)
				edst := dst.Index(i)
				deepCopy(esrc, edst)
			}
			return
		}

	case reflect.Slice:
		switch src.Type().Elem().Kind() {
		case reflect.Int64, reflect.Int32, reflect.Int16,
			reflect.Int8, reflect.Int, reflect.Uint64,
			reflect.Uint32, reflect.Uint16, reflect.Uint8,
			reflect.Uint, reflect.Bool, reflect.Float64,
			reflect.Float32, reflect.String:

			cpy := reflect.MakeSlice(
				src.Type(), src.Len(), src.Len())
			reflect.Copy(cpy, src)
			dst.Set(src)
			return
		default:
			cpy := reflect.MakeSlice(
				src.Type(), src.Len(), src.Len())
			for i := 0; i < src.Len(); i++ {
				esrc := src.Index(i)
				ecpy := cpy.Index(i)
				deepCopy(esrc, ecpy)
			}
			dst.Set(src)
			return
		}

	case reflect.Struct:
		switch src.Type() {
		case timeType:
			dst.Set(src)
			return
		default:
			for i := 0; i < src.NumField(); i++ {
				if !isExported(src.Type().Field(i)) {
					continue // field is unexported
				}
				srcf := src.Field(i)
				dstf := dst.Field(i)
				deepCopy(srcf, dstf)
			}
			return
		}

	case reflect.Map:
		cpy := reflect.MakeMapWithSize(src.Type(), src.Len())
		keys := src.MapKeys()
		for _, key := range keys {
			val := src.MapIndex(key)
			cpy.SetMapIndex(key, val)
		}
		dst.Set(cpy)
		return

	// Primitive types
	case reflect.Int64, reflect.Int32, reflect.Int16,
		reflect.Int8, reflect.Int:
		dst.SetInt(src.Int())
	case reflect.Uint64, reflect.Uint32, reflect.Uint16,
		reflect.Uint8, reflect.Uint:
		dst.SetUint(src.Uint())
	case reflect.Bool:
		dst.SetBool(src.Bool())
	case reflect.Float64, reflect.Float32:
		dst.SetFloat(src.Float())
	case reflect.String:
		dst.SetString(src.String())

	default:
		panic(fmt.Sprintf("unsupported type %v", src.Kind()))
	}

	return
}

//----------------------------------------
// misc.

// Call .DeepCopy() method if possible.
func callDeepCopy(src, dst reflect.Value) bool {
	dc := src.MethodByName("DeepCopy")
	if !dc.IsValid() {
		return false
	}
	if dc.Type().NumIn() != 0 {
		return false
	}
	if dc.Type().NumOut() != 1 {
		return false
	}
	otype := dc.Type().Out(0)
	if dst.Kind() == reflect.Ptr &&
		dst.Type().Elem() == otype {
		cpy := reflect.New(dst.Type().Elem())
		out := dc.Call(nil)[0]
		cpy.Elem().Set(out)
		dst.Set(cpy)
		return true
	}
	if dst.Type() == otype {
		out := dc.Call(nil)[0]
		dst.Set(out)
		return true
	}
	return false
}

// Call .MarshalAmino() and .UnmarshalAmino to copy if possible.
// Panics if .MarshalAmino() or .UnmarshalAmino() return an error.
// CONTRACT: src and dst are of equal types.
func callAminoCopy(src, dst reflect.Value) bool {
	if src.Type() != dst.Type() {
		panic("should not happen")
	}
	if src.Kind() == reflect.Ptr {
		cpy := reflect.New(src.Type().Elem())
		dst.Set(cpy)
	} else if src.CanAddr() {
		if !dst.CanAddr() {
			panic("should not happen")
		}
		src = src.Addr()
		dst = dst.Addr()
	} else {
		return false
	}
	if !canAminoCopy(src) {
		return false
	}
	cpy := reflect.New(src.Type().Elem())
	dst.Set(cpy)
	ma := src.MethodByName("MarshalAmino")
	ua := dst.MethodByName("UnmarshalAmino")
	outs := ma.Call(nil)
	repr, err := outs[0], outs[1]
	if !err.IsNil() {
		panic(err.Interface())
	}
	outs = ua.Call([]reflect.Value{repr})
	err = outs[0]
	if !err.IsNil() {
		panic(err.Interface())
	}
	return true
}

func canAminoCopy(rv reflect.Value) bool {
	if !rv.MethodByName("UnmarshalAmino").IsValid() {
		return false
	}
	ua := rv.MethodByName("UnmarshalAmino")
	if !ua.IsValid() {
		return false
	}
	if ua.Type().NumIn() != 1 {
		return false
	}
	if ua.Type().NumOut() != 1 {
		return false
	}
	if ua.Type().Out(0) != errorType {
		return false
	}
	ma := rv.MethodByName("MarshalAmino")
	if !ma.IsValid() {
		return false
	}
	if ma.Type().NumIn() != 0 {
		return false
	}
	if ma.Type().NumOut() != 2 {
		return false
	}
	if ma.Type().Out(1) != errorType {
		return false
	}
	if ua.Type().In(0) != ma.Type().Out(0) {
		return false
	}
	return true
}
