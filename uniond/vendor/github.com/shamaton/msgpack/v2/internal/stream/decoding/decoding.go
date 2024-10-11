package decoding

import (
	"fmt"
	"io"
	"reflect"

	"github.com/shamaton/msgpack/v2/internal/common"
)

type decoder struct {
	r       io.Reader
	asArray bool
	buf     *common.Buffer
	common.Common
}

// Decode analyzes the MessagePack-encoded data and stores
// the result into the pointer of v.
func Decode(r io.Reader, v interface{}, asArray bool) error {

	if r == nil {
		return fmt.Errorf("reader is nil")
	}
	rv := reflect.ValueOf(v)
	if rv.Kind() != reflect.Ptr {
		return fmt.Errorf("holder must set pointer value. but got: %t", v)
	}

	rv = rv.Elem()

	d := decoder{r: r,
		buf:     common.GetBuffer(),
		asArray: asArray,
	}
	err := d.decode(rv)
	common.PutBuffer(d.buf)
	return err
}

func (d *decoder) decode(rv reflect.Value) error {
	code, err := d.readSize1()
	if err != nil {
		return err
	}
	return d.decodeWithCode(code, rv)
}

func (d *decoder) decodeWithCode(code byte, rv reflect.Value) error {
	k := rv.Kind()
	switch k {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		v, err := d.asIntWithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetInt(v)

	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		v, err := d.asUintWithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetUint(v)

	case reflect.Float32:
		v, err := d.asFloat32WithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetFloat(float64(v))

	case reflect.Float64:
		v, err := d.asFloat64WithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetFloat(v)

	case reflect.String:
		// byte slice
		if d.isCodeBin(code) {
			v, err := d.asBinStringWithCode(code, k)
			if err != nil {
				return err
			}
			rv.SetString(v)
			return nil
		}
		v, err := d.asStringWithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetString(v)

	case reflect.Bool:
		v, err := d.asBoolWithCode(code, k)
		if err != nil {
			return err
		}
		rv.SetBool(v)

	case reflect.Slice:
		// nil
		if d.isCodeNil(code) {
			return nil
		}
		// byte slice
		if d.isCodeBin(code) {
			bs, err := d.asBinWithCode(code, k)
			if err != nil {
				return err
			}
			rv.SetBytes(bs)
			return nil
		}
		// string to bytes
		if d.isCodeString(code) {
			l, err := d.stringByteLength(code, k)
			if err != nil {
				return err
			}
			bs, err := d.asStringByteByLength(l, k)
			if err != nil {
				return err
			}
			rv.SetBytes(bs)
			return nil
		}

		// get slice length
		l, err := d.sliceLength(code, k)
		if err != nil {
			return err
		}

		// check fixed type
		found, err := d.asFixedSlice(rv, l)
		if err != nil {
			return err
		}
		if found {
			return nil
		}

		// create slice dynamically
		tmpSlice := reflect.MakeSlice(rv.Type(), l, l)
		for i := 0; i < l; i++ {
			v := tmpSlice.Index(i)
			if v.Kind() == reflect.Struct {
				structCode, err := d.readSize1()
				if err != nil {
					return err
				}
				if err = d.setStruct(structCode, v, k); err != nil {
					return err
				}
			} else {
				if err = d.decode(v); err != nil {
					return err
				}
			}
		}
		rv.Set(tmpSlice)

	case reflect.Complex64:
		v, err := d.asComplex64(code, k)
		if err != nil {
			return err
		}
		rv.SetComplex(complex128(v))

	case reflect.Complex128:
		v, err := d.asComplex128(code, k)
		if err != nil {
			return err
		}
		rv.SetComplex(v)

	case reflect.Array:
		// nil
		if d.isCodeNil(code) {
			return nil
		}
		// byte slice
		if d.isCodeBin(code) {
			bs, err := d.asBinWithCode(code, k)
			if err != nil {
				return err
			}
			if len(bs) > rv.Len() {
				return fmt.Errorf("%v len is %d, but msgpack has %d elements", rv.Type(), rv.Len(), len(bs))
			}
			for i, b := range bs {
				rv.Index(i).SetUint(uint64(b))
			}
			return nil
		}
		// string to bytes
		if d.isCodeString(code) {
			l, err := d.stringByteLength(code, k)
			if err != nil {
				return err
			}
			if l > rv.Len() {
				return fmt.Errorf("%v len is %d, but msgpack has %d elements", rv.Type(), rv.Len(), l)
			}
			bs, err := d.asStringByteByLength(l, k)
			if err != nil {
				return err
			}
			for i, b := range bs {
				rv.Index(i).SetUint(uint64(b))
			}
			return nil
		}

		// get slice length
		l, err := d.sliceLength(code, k)
		if err != nil {
			return err
		}

		if l > rv.Len() {
			return fmt.Errorf("%v len is %d, but msgpack has %d elements", rv.Type(), rv.Len(), l)
		}

		// create array dynamically
		for i := 0; i < l; i++ {
			err = d.decode(rv.Index(i))
			if err != nil {
				return err
			}
		}

	case reflect.Map:
		// nil
		if d.isCodeNil(code) {
			return nil
		}

		// get map length
		l, err := d.mapLength(code, k)
		if err != nil {
			return err
		}

		// check fixed type
		found, err := d.asFixedMap(rv, l)
		if err != nil {
			return err
		}
		if found {
			return nil
		}

		// create dynamically
		key := rv.Type().Key()
		value := rv.Type().Elem()
		if rv.IsNil() {
			rv.Set(reflect.MakeMapWithSize(rv.Type(), l))
		}
		for i := 0; i < l; i++ {
			k := reflect.New(key).Elem()
			v := reflect.New(value).Elem()
			err = d.decode(k)
			if err != nil {
				return err
			}
			err = d.decode(v)
			if err != nil {
				return err
			}

			rv.SetMapIndex(k, v)
		}

	case reflect.Struct:
		err := d.setStruct(code, rv, k)
		if err != nil {
			return err
		}

	case reflect.Ptr:
		// nil
		if d.isCodeNil(code) {
			return nil
		}

		if rv.Elem().Kind() == reflect.Invalid {
			n := reflect.New(rv.Type().Elem())
			rv.Set(n)
		}

		err := d.decodeWithCode(code, rv.Elem())
		if err != nil {
			return err
		}

	case reflect.Interface:
		if rv.Elem().Kind() == reflect.Ptr {
			err := d.decode(rv.Elem())
			if err != nil {
				return err
			}
		} else {
			v, err := d.asInterfaceWithCode(code, k)
			if err != nil {
				return err
			}
			if v != nil {
				rv.Set(reflect.ValueOf(v))
			}
		}

	default:
		return fmt.Errorf("type(%v) is unsupported", rv.Kind())
	}
	return nil
}

func (d *decoder) errorTemplate(code byte, k reflect.Kind) error {
	return fmt.Errorf("msgpack : invalid code %x decoding %v", code, k)
}
