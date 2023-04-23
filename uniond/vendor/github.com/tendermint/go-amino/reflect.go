package amino

import (
	"encoding/json"
	"fmt"
	"reflect"
	"time"
)

//----------------------------------------
// Constants

const printLog = false

var (
	timeType            = reflect.TypeOf(time.Time{})
	jsonMarshalerType   = reflect.TypeOf(new(json.Marshaler)).Elem()
	jsonUnmarshalerType = reflect.TypeOf(new(json.Unmarshaler)).Elem()
	errorType           = reflect.TypeOf(new(error)).Elem()
)

//----------------------------------------
// encode: see binary-encode.go and json-encode.go
// decode: see binary-decode.go and json-decode.go

//----------------------------------------
// Misc.

func getTypeFromPointer(ptr interface{}) reflect.Type {
	rt := reflect.TypeOf(ptr)
	if rt.Kind() != reflect.Ptr {
		panic(fmt.Sprintf("expected pointer, got %v", rt))
	}
	return rt.Elem()
}

func checkUnsafe(field FieldInfo) {
	if field.Unsafe {
		return
	}
	switch field.Type.Kind() {
	case reflect.Float32, reflect.Float64:
		panic("floating point types are unsafe for go-amino")
	}
}

// CONTRACT: by the time this is called, len(bz) >= _n
// Returns true so you can write one-liners.
func slide(bz *[]byte, n *int, _n int) bool {
	if _n < 0 || _n > len(*bz) {
		panic(fmt.Sprintf("impossible slide: len:%v _n:%v", len(*bz), _n))
	}
	*bz = (*bz)[_n:]
	if n != nil {
		*n += _n
	}
	return true
}

// Dereference pointer recursively.
// drv: the final non-pointer value (which may be invalid).
// isPtr: whether rv.Kind() == reflect.Ptr.
// isNilPtr: whether a nil pointer at any level.
func derefPointers(rv reflect.Value) (drv reflect.Value, isPtr bool, isNilPtr bool) {
	for rv.Kind() == reflect.Ptr {
		isPtr = true
		if rv.IsNil() {
			isNilPtr = true
			return
		}
		rv = rv.Elem()
	}
	drv = rv
	return
}

// Dereference pointer recursively or return zero value.
// drv: the final non-pointer value (which is never invalid).
// isPtr: whether rv.Kind() == reflect.Ptr.
// isNilPtr: whether a nil pointer at any level.
func derefPointersZero(rv reflect.Value) (drv reflect.Value, isPtr bool, isNilPtr bool) {
	for rv.Kind() == reflect.Ptr {
		isPtr = true
		if rv.IsNil() {
			isNilPtr = true
			rt := rv.Type().Elem()
			for rt.Kind() == reflect.Ptr {
				rt = rt.Elem()
			}
			drv = reflect.New(rt).Elem()
			return
		}
		rv = rv.Elem()
	}
	drv = rv
	return
}

// Returns isDefaultValue=true iff is ultimately nil or empty
// after (recursive) dereferencing.
// If isDefaultValue=false, erv is set to the non-nil non-default
// dereferenced value.
// A zero/empty struct is not considered default for this
// function.
func isDefaultValue(rv reflect.Value) (erv reflect.Value, isDefaultValue bool) {
	rv, _, isNilPtr := derefPointers(rv)
	if isNilPtr {
		return rv, true
	}
	switch rv.Kind() {
	case reflect.Bool:
		return rv, false
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		return rv, rv.Int() == 0
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		return rv, rv.Uint() == 0
	case reflect.String:
		return rv, rv.Len() == 0
	case reflect.Chan, reflect.Map, reflect.Slice:
		return rv, rv.IsNil() || rv.Len() == 0
	case reflect.Func, reflect.Interface:
		return rv, rv.IsNil()
	default:
		return rv, false
	}
}

// Returns the default value of a type.  For a time type or a pointer(s) to
// time, the default value is not zero (or nil), but the time value of 1970.
func defaultValue(rt reflect.Type) (rv reflect.Value) {
	switch rt.Kind() {
	case reflect.Ptr:
		// Dereference all the way and see if it's a time type.
		rt_ := rt.Elem()
		for rt_.Kind() == reflect.Ptr {
			rt_ = rt_.Elem()
		}
		switch rt_ {
		case timeType:
			// Start from the top and construct pointers as needed.
			rv = reflect.New(rt).Elem()
			rt_, rv_ := rt, rv
			for rt_.Kind() == reflect.Ptr {
				newPtr := reflect.New(rt_.Elem())
				rv_.Set(newPtr)
				rt_ = rt_.Elem()
				rv_ = rv_.Elem()
			}
			// Set to 1970, the whole point of this function.
			rv_.Set(reflect.ValueOf(zeroTime))
			return rv
		}
	case reflect.Struct:
		switch rt {
		case timeType:
			// Set to 1970, the whole point of this function.
			rv = reflect.New(rt).Elem()
			rv.Set(reflect.ValueOf(zeroTime))
			return rv
		}
	}

	// Just return the default Go zero object.
	// Return an empty struct.
	return reflect.Zero(rt)
}

func isNil(rv reflect.Value) bool {
	switch rv.Kind() {
	case reflect.Interface, reflect.Chan, reflect.Func, reflect.Map, reflect.Ptr, reflect.Slice:
		return rv.IsNil()
	default:
		return false
	}
}

// constructConcreteType creates the concrete value as
// well as the corresponding settable value for it.
// Return irvSet which should be set on caller's interface rv.
func constructConcreteType(cinfo *TypeInfo) (crv, irvSet reflect.Value) {
	// Construct new concrete type.
	if cinfo.PointerPreferred {
		cPtrRv := reflect.New(cinfo.Type)
		crv = cPtrRv.Elem()
		irvSet = cPtrRv
	} else {
		crv = reflect.New(cinfo.Type).Elem()
		irvSet = crv
	}
	return
}

// CONTRACT: rt.Kind() != reflect.Ptr
func typeToTyp3(rt reflect.Type, opts FieldOptions) Typ3 {
	switch rt.Kind() {
	case reflect.Interface:
		return Typ3_ByteLength
	case reflect.Array, reflect.Slice:
		return Typ3_ByteLength
	case reflect.String:
		return Typ3_ByteLength
	case reflect.Struct, reflect.Map:
		return Typ3_ByteLength
	case reflect.Int64, reflect.Uint64:
		if opts.BinFixed64 {
			return Typ3_8Byte
		} else {
			return Typ3_Varint
		}
	case reflect.Int32, reflect.Uint32:
		if opts.BinFixed32 {
			return Typ3_4Byte
		} else {
			return Typ3_Varint
		}
	case reflect.Int16, reflect.Int8, reflect.Int,
		reflect.Uint16, reflect.Uint8, reflect.Uint, reflect.Bool:
		return Typ3_Varint
	case reflect.Float64:
		return Typ3_8Byte
	case reflect.Float32:
		return Typ3_4Byte
	default:
		panic(fmt.Sprintf("unsupported field type %v", rt))
	}
}

func toReprObject(rv reflect.Value) (rrv reflect.Value, err error) {
	var mwrm reflect.Value
	if rv.CanAddr() {
		mwrm = rv.Addr().MethodByName("MarshalAmino")
	} else {
		mwrm = rv.MethodByName("MarshalAmino")
	}
	mwouts := mwrm.Call(nil)
	if !mwouts[1].IsNil() {
		erri := mwouts[1].Interface()
		if erri != nil {
			err = erri.(error)
			return
		}
	}
	rrv = mwouts[0]
	return
}

func toReprJSONObject(rv reflect.Value) (rrv reflect.Value, err error) {
	var mwrm reflect.Value
	if rv.CanAddr() {
		mwrm = rv.Addr().MethodByName("MarshalAminoJSON")
	} else {
		mwrm = rv.MethodByName("MarshalAminoJSON")
	}
	mwouts := mwrm.Call(nil)
	if !mwouts[1].IsNil() {
		erri := mwouts[1].Interface()
		if erri != nil {
			err = erri.(error)
			return rrv, err
		}
	}
	rrv = mwouts[0]
	return
}
