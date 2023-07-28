package json

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"reflect"
	"strconv"
	"time"
)

var (
	timeType            = reflect.TypeOf(time.Time{})
	jsonMarshalerType   = reflect.TypeOf(new(json.Marshaler)).Elem()
	jsonUnmarshalerType = reflect.TypeOf(new(json.Unmarshaler)).Elem()
)

// Marshal marshals the value as JSON, using Amino-compatible JSON encoding (strings for
// 64-bit numbers, and type wrappers for registered types).
func Marshal(v interface{}) ([]byte, error) {
	buf := new(bytes.Buffer)
	err := encode(buf, v)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

// MarshalIndent marshals the value as JSON, using the given prefix and indentation.
func MarshalIndent(v interface{}, prefix, indent string) ([]byte, error) {
	bz, err := Marshal(v)
	if err != nil {
		return nil, err
	}
	buf := new(bytes.Buffer)
	err = json.Indent(buf, bz, prefix, indent)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}

func encode(w io.Writer, v interface{}) error {
	// Bare nil values can't be reflected, so we must handle them here.
	if v == nil {
		return writeStr(w, "null")
	}
	rv := reflect.ValueOf(v)

	// If this is a registered type, defer to interface encoder regardless of whether the input is
	// an interface or a bare value. This retains Amino's behavior, but is inconsistent with
	// behavior in structs where an interface field will get the type wrapper while a bare value
	// field will not.
	if typeRegistry.name(rv.Type()) != "" {
		return encodeReflectInterface(w, rv)
	}

	return encodeReflect(w, rv)
}

func encodeReflect(w io.Writer, rv reflect.Value) error {
	if !rv.IsValid() {
		return errors.New("invalid reflect value")
	}

	// Recursively dereference if pointer.
	for rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			return writeStr(w, "null")
		}
		rv = rv.Elem()
	}

	// Convert times to UTC.
	if rv.Type() == timeType {
		rv = reflect.ValueOf(rv.Interface().(time.Time).Round(0).UTC())
	}

	// If the value implements json.Marshaler, defer to stdlib directly. Since we've already
	// dereferenced, we try implementations with both value receiver and pointer receiver. We must
	// do this after the time normalization above, and thus after dereferencing.
	if rv.Type().Implements(jsonMarshalerType) {
		return encodeStdlib(w, rv.Interface())
	} else if rv.CanAddr() && rv.Addr().Type().Implements(jsonMarshalerType) {
		return encodeStdlib(w, rv.Addr().Interface())
	}

	switch rv.Type().Kind() {
	// Complex types must be recursively encoded.
	case reflect.Interface:
		return encodeReflectInterface(w, rv)

	case reflect.Array, reflect.Slice:
		return encodeReflectList(w, rv)

	case reflect.Map:
		return encodeReflectMap(w, rv)

	case reflect.Struct:
		return encodeReflectStruct(w, rv)

	// 64-bit integers are emitted as strings, to avoid precision problems with e.g.
	// Javascript which uses 64-bit floats (having 53-bit precision).
	case reflect.Int64, reflect.Int:
		return writeStr(w, `"`+strconv.FormatInt(rv.Int(), 10)+`"`)

	case reflect.Uint64, reflect.Uint:
		return writeStr(w, `"`+strconv.FormatUint(rv.Uint(), 10)+`"`)

	// For everything else, defer to the stdlib encoding/json encoder
	default:
		return encodeStdlib(w, rv.Interface())
	}
}

func encodeReflectList(w io.Writer, rv reflect.Value) error {
	// Emit nil slices as null.
	if rv.Kind() == reflect.Slice && rv.IsNil() {
		return writeStr(w, "null")
	}

	// Encode byte slices as base64 with the stdlib encoder.
	if rv.Type().Elem().Kind() == reflect.Uint8 {
		// Stdlib does not base64-encode byte arrays, only slices, so we copy to slice.
		if rv.Type().Kind() == reflect.Array {
			slice := reflect.MakeSlice(reflect.SliceOf(rv.Type().Elem()), rv.Len(), rv.Len())
			reflect.Copy(slice, rv)
			rv = slice
		}
		return encodeStdlib(w, rv.Interface())
	}

	// Anything else we recursively encode ourselves.
	length := rv.Len()
	if err := writeStr(w, "["); err != nil {
		return err
	}
	for i := 0; i < length; i++ {
		if err := encodeReflect(w, rv.Index(i)); err != nil {
			return err
		}
		if i < length-1 {
			if err := writeStr(w, ","); err != nil {
				return err
			}
		}
	}
	return writeStr(w, "]")
}

func encodeReflectMap(w io.Writer, rv reflect.Value) error {
	if rv.Type().Key().Kind() != reflect.String {
		return errors.New("map key must be string")
	}

	// nil maps are not emitted as nil, to retain Amino compatibility.

	if err := writeStr(w, "{"); err != nil {
		return err
	}
	writeComma := false
	for _, keyrv := range rv.MapKeys() {
		if writeComma {
			if err := writeStr(w, ","); err != nil {
				return err
			}
		}
		if err := encodeStdlib(w, keyrv.Interface()); err != nil {
			return err
		}
		if err := writeStr(w, ":"); err != nil {
			return err
		}
		if err := encodeReflect(w, rv.MapIndex(keyrv)); err != nil {
			return err
		}
		writeComma = true
	}
	return writeStr(w, "}")
}

func encodeReflectStruct(w io.Writer, rv reflect.Value) error {
	sInfo := makeStructInfo(rv.Type())
	if err := writeStr(w, "{"); err != nil {
		return err
	}
	writeComma := false
	for i, fInfo := range sInfo.fields {
		frv := rv.Field(i)
		if fInfo.hidden || (fInfo.omitEmpty && frv.IsZero()) {
			continue
		}

		if writeComma {
			if err := writeStr(w, ","); err != nil {
				return err
			}
		}
		if err := encodeStdlib(w, fInfo.jsonName); err != nil {
			return err
		}
		if err := writeStr(w, ":"); err != nil {
			return err
		}
		if err := encodeReflect(w, frv); err != nil {
			return err
		}
		writeComma = true
	}
	return writeStr(w, "}")
}

func encodeReflectInterface(w io.Writer, rv reflect.Value) error {
	// Get concrete value and dereference pointers.
	for rv.Kind() == reflect.Ptr || rv.Kind() == reflect.Interface {
		if rv.IsNil() {
			return writeStr(w, "null")
		}
		rv = rv.Elem()
	}

	// Look up the name of the concrete type
	name := typeRegistry.name(rv.Type())
	if name == "" {
		return fmt.Errorf("cannot encode unregistered type %v", rv.Type())
	}

	// Write value wrapped in interface envelope
	if err := writeStr(w, fmt.Sprintf(`{"type":%q,"value":`, name)); err != nil {
		return err
	}
	if err := encodeReflect(w, rv); err != nil {
		return err
	}
	return writeStr(w, "}")
}

func encodeStdlib(w io.Writer, v interface{}) error {
	// Doesn't stream the output because that adds a newline, as per:
	// https://golang.org/pkg/encoding/json/#Encoder.Encode
	blob, err := json.Marshal(v)
	if err != nil {
		return err
	}
	_, err = w.Write(blob)
	return err
}

func writeStr(w io.Writer, s string) error {
	_, err := w.Write([]byte(s))
	return err
}
