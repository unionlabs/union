//go:build windows
// +build windows

package etw

import (
	"fmt"
	"math"
	"reflect"
	"syscall"
	"time"
	"unsafe"
)

// FieldOpt defines the option function type that can be passed to
// Provider.WriteEvent to add fields to the event.
type FieldOpt func(em *eventMetadata, ed *eventData)

// WithFields returns the variadic arguments as a single slice.
func WithFields(opts ...FieldOpt) []FieldOpt {
	return opts
}

// BoolField adds a single bool field to the event.
func BoolField(name string, value bool) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeUint8, outTypeBoolean, 0)
		bool8 := uint8(0)
		if value {
			bool8 = uint8(1)
		}
		ed.writeUint8(bool8)
	}
}

// BoolArray adds an array of bool to the event.
func BoolArray(name string, values []bool) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeUint8, outTypeBoolean, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			bool8 := uint8(0)
			if v {
				bool8 = uint8(1)
			}
			ed.writeUint8(bool8)
		}
	}
}

// StringField adds a single string field to the event.
func StringField(name string, value string) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeANSIString, outTypeUTF8, 0)
		ed.writeString(value)
	}
}

// StringArray adds an array of string to the event.
func StringArray(name string, values []string) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeANSIString, outTypeUTF8, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeString(v)
		}
	}
}

// IntField adds a single int field to the event.
func IntField(name string, value int) FieldOpt {
	switch unsafe.Sizeof(value) {
	case 4:
		return Int32Field(name, int32(value))
	case 8:
		return Int64Field(name, int64(value))
	default:
		panic("Unsupported int size")
	}
}

// IntArray adds an array of int to the event.
func IntArray(name string, values []int) FieldOpt {
	inType := inTypeNull
	var writeItem func(*eventData, int)
	switch unsafe.Sizeof(values[0]) {
	case 4:
		inType = inTypeInt32
		writeItem = func(ed *eventData, item int) { ed.writeInt32(int32(item)) }
	case 8:
		inType = inTypeInt64
		writeItem = func(ed *eventData, item int) { ed.writeInt64(int64(item)) }
	default:
		panic("Unsupported int size")
	}

	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inType, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			writeItem(ed, v)
		}
	}
}

// Int8Field adds a single int8 field to the event.
func Int8Field(name string, value int8) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeInt8, outTypeDefault, 0)
		ed.writeInt8(value)
	}
}

// Int8Array adds an array of int8 to the event.
func Int8Array(name string, values []int8) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeInt8, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeInt8(v)
		}
	}
}

// Int16Field adds a single int16 field to the event.
func Int16Field(name string, value int16) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeInt16, outTypeDefault, 0)
		ed.writeInt16(value)
	}
}

// Int16Array adds an array of int16 to the event.
func Int16Array(name string, values []int16) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeInt16, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeInt16(v)
		}
	}
}

// Int32Field adds a single int32 field to the event.
func Int32Field(name string, value int32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeInt32, outTypeDefault, 0)
		ed.writeInt32(value)
	}
}

// Int32Array adds an array of int32 to the event.
func Int32Array(name string, values []int32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeInt32, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeInt32(v)
		}
	}
}

// Int64Field adds a single int64 field to the event.
func Int64Field(name string, value int64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeInt64, outTypeDefault, 0)
		ed.writeInt64(value)
	}
}

// Int64Array adds an array of int64 to the event.
func Int64Array(name string, values []int64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeInt64, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeInt64(v)
		}
	}
}

// UintField adds a single uint field to the event.
func UintField(name string, value uint) FieldOpt {
	switch unsafe.Sizeof(value) {
	case 4:
		return Uint32Field(name, uint32(value))
	case 8:
		return Uint64Field(name, uint64(value))
	default:
		panic("Unsupported uint size")
	}
}

// UintArray adds an array of uint to the event.
func UintArray(name string, values []uint) FieldOpt {
	inType := inTypeNull
	var writeItem func(*eventData, uint)
	switch unsafe.Sizeof(values[0]) {
	case 4:
		inType = inTypeUint32
		writeItem = func(ed *eventData, item uint) { ed.writeUint32(uint32(item)) }
	case 8:
		inType = inTypeUint64
		writeItem = func(ed *eventData, item uint) { ed.writeUint64(uint64(item)) }
	default:
		panic("Unsupported uint size")
	}

	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inType, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			writeItem(ed, v)
		}
	}
}

// Uint8Field adds a single uint8 field to the event.
func Uint8Field(name string, value uint8) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeUint8, outTypeDefault, 0)
		ed.writeUint8(value)
	}
}

// Uint8Array adds an array of uint8 to the event.
func Uint8Array(name string, values []uint8) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeUint8, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint8(v)
		}
	}
}

// Uint16Field adds a single uint16 field to the event.
func Uint16Field(name string, value uint16) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeUint16, outTypeDefault, 0)
		ed.writeUint16(value)
	}
}

// Uint16Array adds an array of uint16 to the event.
func Uint16Array(name string, values []uint16) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeUint16, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint16(v)
		}
	}
}

// Uint32Field adds a single uint32 field to the event.
func Uint32Field(name string, value uint32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeUint32, outTypeDefault, 0)
		ed.writeUint32(value)
	}
}

// Uint32Array adds an array of uint32 to the event.
func Uint32Array(name string, values []uint32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeUint32, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint32(v)
		}
	}
}

// Uint64Field adds a single uint64 field to the event.
func Uint64Field(name string, value uint64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeUint64, outTypeDefault, 0)
		ed.writeUint64(value)
	}
}

// Uint64Array adds an array of uint64 to the event.
func Uint64Array(name string, values []uint64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeUint64, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint64(v)
		}
	}
}

// UintptrField adds a single uintptr field to the event.
func UintptrField(name string, value uintptr) FieldOpt {
	inType := inTypeNull
	var writeItem func(*eventData, uintptr)
	switch unsafe.Sizeof(value) {
	case 4:
		inType = inTypeHexInt32
		writeItem = func(ed *eventData, item uintptr) { ed.writeUint32(uint32(item)) }
	case 8:
		inType = inTypeHexInt64
		writeItem = func(ed *eventData, item uintptr) { ed.writeUint64(uint64(item)) }
	default:
		panic("Unsupported uintptr size")
	}

	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inType, outTypeDefault, 0)
		writeItem(ed, value)
	}
}

// UintptrArray adds an array of uintptr to the event.
func UintptrArray(name string, values []uintptr) FieldOpt {
	inType := inTypeNull
	var writeItem func(*eventData, uintptr)
	switch unsafe.Sizeof(values[0]) {
	case 4:
		inType = inTypeHexInt32
		writeItem = func(ed *eventData, item uintptr) { ed.writeUint32(uint32(item)) }
	case 8:
		inType = inTypeHexInt64
		writeItem = func(ed *eventData, item uintptr) { ed.writeUint64(uint64(item)) }
	default:
		panic("Unsupported uintptr size")
	}

	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inType, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			writeItem(ed, v)
		}
	}
}

// Float32Field adds a single float32 field to the event.
func Float32Field(name string, value float32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeFloat, outTypeDefault, 0)
		ed.writeUint32(math.Float32bits(value))
	}
}

// Float32Array adds an array of float32 to the event.
func Float32Array(name string, values []float32) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeFloat, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint32(math.Float32bits(v))
		}
	}
}

// Float64Field adds a single float64 field to the event.
func Float64Field(name string, value float64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeDouble, outTypeDefault, 0)
		ed.writeUint64(math.Float64bits(value))
	}
}

// Float64Array adds an array of float64 to the event.
func Float64Array(name string, values []float64) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeArray(name, inTypeDouble, outTypeDefault, 0)
		ed.writeUint16(uint16(len(values)))
		for _, v := range values {
			ed.writeUint64(math.Float64bits(v))
		}
	}
}

// Struct adds a nested struct to the event, the FieldOpts in the opts argument
// are used to specify the fields of the struct.
func Struct(name string, opts ...FieldOpt) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeStruct(name, uint8(len(opts)), 0)
		for _, opt := range opts {
			opt(em, ed)
		}
	}
}

// Time adds a time to the event.
func Time(name string, value time.Time) FieldOpt {
	return func(em *eventMetadata, ed *eventData) {
		em.writeField(name, inTypeFileTime, outTypeDateTimeUTC, 0)
		ed.writeFiletime(syscall.NsecToFiletime(value.UTC().UnixNano()))
	}
}

// Currently, we support logging basic builtin types (int, string, etc), slices
// of basic builtin types, error, types derived from the basic types (e.g. "type
// foo int"), and structs (recursively logging their fields). We do not support
// slices of derived types (e.g. "[]foo").
//
// For types that we don't support, the value is formatted via fmt.Sprint, and
// we also log a message that the type is unsupported along with the formatted
// type. The intent of this is to make it easier to see which types are not
// supported in traces, so we can evaluate adding support for more types in the
// future.
func SmartField(name string, v interface{}) FieldOpt {
	switch v := v.(type) {
	case bool:
		return BoolField(name, v)
	case []bool:
		return BoolArray(name, v)
	case string:
		return StringField(name, v)
	case []string:
		return StringArray(name, v)
	case int:
		return IntField(name, v)
	case []int:
		return IntArray(name, v)
	case int8:
		return Int8Field(name, v)
	case []int8:
		return Int8Array(name, v)
	case int16:
		return Int16Field(name, v)
	case []int16:
		return Int16Array(name, v)
	case int32:
		return Int32Field(name, v)
	case []int32:
		return Int32Array(name, v)
	case int64:
		return Int64Field(name, v)
	case []int64:
		return Int64Array(name, v)
	case uint:
		return UintField(name, v)
	case []uint:
		return UintArray(name, v)
	case uint8:
		return Uint8Field(name, v)
	case []uint8:
		return Uint8Array(name, v)
	case uint16:
		return Uint16Field(name, v)
	case []uint16:
		return Uint16Array(name, v)
	case uint32:
		return Uint32Field(name, v)
	case []uint32:
		return Uint32Array(name, v)
	case uint64:
		return Uint64Field(name, v)
	case []uint64:
		return Uint64Array(name, v)
	case uintptr:
		return UintptrField(name, v)
	case []uintptr:
		return UintptrArray(name, v)
	case float32:
		return Float32Field(name, v)
	case []float32:
		return Float32Array(name, v)
	case float64:
		return Float64Field(name, v)
	case []float64:
		return Float64Array(name, v)
	case error:
		return StringField(name, v.Error())
	case time.Time:
		return Time(name, v)
	default:
		switch rv := reflect.ValueOf(v); rv.Kind() {
		case reflect.Bool:
			return SmartField(name, rv.Bool())
		case reflect.Int:
			return SmartField(name, int(rv.Int()))
		case reflect.Int8:
			return SmartField(name, int8(rv.Int()))
		case reflect.Int16:
			return SmartField(name, int16(rv.Int()))
		case reflect.Int32:
			return SmartField(name, int32(rv.Int()))
		case reflect.Int64:
			return SmartField(name, int64(rv.Int())) //nolint:unconvert // make look consistent
		case reflect.Uint:
			return SmartField(name, uint(rv.Uint()))
		case reflect.Uint8:
			return SmartField(name, uint8(rv.Uint()))
		case reflect.Uint16:
			return SmartField(name, uint16(rv.Uint()))
		case reflect.Uint32:
			return SmartField(name, uint32(rv.Uint()))
		case reflect.Uint64:
			return SmartField(name, uint64(rv.Uint())) //nolint:unconvert // make look consistent
		case reflect.Uintptr:
			return SmartField(name, uintptr(rv.Uint()))
		case reflect.Float32:
			return SmartField(name, float32(rv.Float()))
		case reflect.Float64:
			return SmartField(name, float64(rv.Float())) //nolint:unconvert // make look consistent
		case reflect.String:
			return SmartField(name, rv.String())
		case reflect.Struct:
			fields := make([]FieldOpt, 0, rv.NumField())
			for i := 0; i < rv.NumField(); i++ {
				field := rv.Field(i)
				if field.CanInterface() {
					fields = append(fields, SmartField(name, field.Interface()))
				}
			}
			return Struct(name, fields...)
		case reflect.Array, reflect.Chan, reflect.Complex128, reflect.Complex64,
			reflect.Func, reflect.Interface, reflect.Invalid, reflect.Map, reflect.Ptr,
			reflect.Slice, reflect.UnsafePointer:
		}
	}

	return StringField(name, fmt.Sprintf("(Unsupported: %T) %v", v, v))
}
