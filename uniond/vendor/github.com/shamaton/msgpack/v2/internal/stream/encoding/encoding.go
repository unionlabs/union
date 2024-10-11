package encoding

import (
	"fmt"
	"io"
	"reflect"

	"github.com/shamaton/msgpack/v2/internal/common"
)

type encoder struct {
	w       io.Writer
	asArray bool
	buf     *common.Buffer
	common.Common
}

// Encode writes MessagePack-encoded byte array of v to writer.
func Encode(w io.Writer, v any, asArray bool) error {
	e := encoder{
		w:       w,
		buf:     common.GetBuffer(),
		asArray: asArray,
	}

	rv := reflect.ValueOf(v)
	if rv.Kind() == reflect.Ptr {
		rv = rv.Elem()
		if rv.Kind() == reflect.Ptr {
			rv = rv.Elem()
		}
	}

	err := e.create(rv)
	if err == nil {
		err = e.buf.Flush(e.w)
	}
	common.PutBuffer(e.buf)
	return err
}

func (e *encoder) create(rv reflect.Value) error {

	switch rv.Kind() {
	case reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64, reflect.Uint:
		v := rv.Uint()
		return e.writeUint(v)

	case reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64, reflect.Int:
		v := rv.Int()
		return e.writeInt(v)

	case reflect.Float32:
		return e.writeFloat32(rv.Float())

	case reflect.Float64:
		return e.writeFloat64(rv.Float())

	case reflect.Bool:
		return e.writeBool(rv.Bool())

	case reflect.String:
		return e.writeString(rv.String())

	case reflect.Complex64:
		return e.writeComplex64(complex64(rv.Complex()))

	case reflect.Complex128:
		return e.writeComplex128(rv.Complex())

	case reflect.Slice:
		if rv.IsNil() {
			return e.writeNil()
		}
		l := rv.Len()
		// bin format
		if e.isByteSlice(rv) {
			if err := e.writeByteSliceLength(l); err != nil {
				return err
			}
			return e.setBytes(rv.Bytes())
		}

		// format
		if err := e.writeSliceLength(l); err != nil {
			return err
		}

		if find, err := e.writeFixedSlice(rv); err != nil {
			return err
		} else if find {
			return nil
		}

		// func
		elem := rv.Type().Elem()
		var f structWriteFunc
		if elem.Kind() == reflect.Struct {
			f = e.getStructWriter(elem)
		} else {
			f = e.create
		}

		// objects
		for i := 0; i < l; i++ {
			if err := f(rv.Index(i)); err != nil {
				return err
			}
		}

	case reflect.Array:
		l := rv.Len()
		// bin format
		if e.isByteSlice(rv) {
			if err := e.writeByteSliceLength(l); err != nil {
				return err
			}
			// objects
			for i := 0; i < l; i++ {
				if err := e.setByte1Uint64(rv.Index(i).Uint()); err != nil {
					return err
				}
			}
			return nil
		}

		// format
		if err := e.writeSliceLength(l); err != nil {
			return err
		}

		// func
		elem := rv.Type().Elem()
		var f structWriteFunc
		if elem.Kind() == reflect.Struct {
			f = e.getStructWriter(elem)
		} else {
			f = e.create
		}

		// objects
		for i := 0; i < l; i++ {
			if err := f(rv.Index(i)); err != nil {
				return err
			}
		}

	case reflect.Map:
		if rv.IsNil() {
			return e.writeNil()
		}

		l := rv.Len()
		if err := e.writeMapLength(l); err != nil {
			return err
		}

		if find, err := e.writeFixedMap(rv); err != nil {
			return err
		} else if find {
			return nil
		}

		// key-value
		keys := rv.MapKeys()
		for _, k := range keys {
			if err := e.create(k); err != nil {
				return err
			}
			if err := e.create(rv.MapIndex(k)); err != nil {
				return err
			}
		}

	case reflect.Struct:
		return e.writeStruct(rv)

	case reflect.Ptr:
		if rv.IsNil() {
			return e.writeNil()
		}

		return e.create(rv.Elem())

	case reflect.Interface:
		return e.create(rv.Elem())

	case reflect.Invalid:
		return e.writeNil()
	default:
		return fmt.Errorf("type(%v) is unsupported", rv.Kind())
	}
	return nil
}
