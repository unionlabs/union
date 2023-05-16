package amino

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"reflect"
	"time"

	"github.com/davecgh/go-spew/spew"
)

//----------------------------------------
// cdc.encodeReflectJSON

// This is the main entrypoint for encoding all types in json form.  This
// function calls encodeReflectJSON*, and generally those functions should
// only call this one, for the disfix wrapper is only written here.
// NOTE: Unlike encodeReflectBinary, rv may be a pointer.
// CONTRACT: rv is valid.
func (cdc *Codec) encodeReflectJSON(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.IsValid() {
		panic("should not happen")
	}
	if printLog {
		spew.Printf("(E) encodeReflectJSON(info: %v, rv: %#v (%v), fopts: %v)\n",
			info, rv.Interface(), rv.Type(), fopts)
		defer func() {
			fmt.Printf("(E) -> err: %v\n", err)
		}()
	}

	// Dereference value if pointer.
	var isNilPtr bool
	rv, _, isNilPtr = derefPointers(rv)

	// Write null if necessary.
	if isNilPtr {
		err = writeStr(w, `null`)
		return
	}

	// Special case:
	if rv.Type() == timeType {
		// Amino time strips the timezone.
		// NOTE: This must be done before json.Marshaler override below.
		ct := rv.Interface().(time.Time).Round(0).UTC()
		rv = reflect.ValueOf(ct)
	}

	// Handle override if rv implements MarshalAminoJSON.
	if info.IsAminoJSONMarshaler {
		// First, encode rv into repr instance.
		var (
			rrv   reflect.Value
			rinfo *TypeInfo
		)
		rrv, err = toReprJSONObject(rv)
		if err != nil {
			return
		}
		rinfo, err = cdc.getTypeInfo_wlock(info.AminoJSONMarshalReprType)
		if err != nil {
			return
		}
		// Then, encode the repr instance.
		err = cdc.encodeReflectJSON(w, rinfo, rrv, fopts)
		return
	}

	// Handle override if rv implements json.Marshaler.
	if rv.CanAddr() { // Try pointer first.
		if rv.Addr().Type().Implements(jsonMarshalerType) {
			err = invokeMarshalJSON(w, rv.Addr())
			return
		}
	} else if rv.Type().Implements(jsonMarshalerType) {
		err = invokeMarshalJSON(w, rv)
		return
	}

	// Handle override if rv implements MarshalAmino.
	if info.IsAminoMarshaler {
		// First, encode rv into repr instance.
		var rrv, rinfo = reflect.Value{}, (*TypeInfo)(nil)
		rrv, err = toReprObject(rv)
		if err != nil {
			return
		}
		rinfo, err = cdc.getTypeInfo_wlock(info.AminoMarshalReprType)
		if err != nil {
			return
		}
		// Then, encode the repr instance.
		err = cdc.encodeReflectJSON(w, rinfo, rrv, fopts)
		return
	}

	switch info.Type.Kind() {

	//----------------------------------------
	// Complex

	case reflect.Interface:
		return cdc.encodeReflectJSONInterface(w, info, rv, fopts)

	case reflect.Array, reflect.Slice:
		return cdc.encodeReflectJSONList(w, info, rv, fopts)

	case reflect.Struct:
		return cdc.encodeReflectJSONStruct(w, info, rv, fopts)

	case reflect.Map:
		return cdc.encodeReflectJSONMap(w, info, rv, fopts)

	//----------------------------------------
	// Signed, Unsigned

	case reflect.Int64, reflect.Int:
		_, err = fmt.Fprintf(w, `"%d"`, rv.Int()) // JS can't handle int64
		return

	case reflect.Uint64, reflect.Uint:
		_, err = fmt.Fprintf(w, `"%d"`, rv.Uint()) // JS can't handle uint64
		return

	case reflect.Int32, reflect.Int16, reflect.Int8,
		reflect.Uint32, reflect.Uint16, reflect.Uint8:
		return invokeStdlibJSONMarshal(w, rv.Interface())

	//----------------------------------------
	// Misc

	case reflect.Float64, reflect.Float32:
		if !fopts.Unsafe {
			return errors.New("Amino.JSON float* support requires `amino:\"unsafe\"`.")
		}
		fallthrough
	case reflect.Bool, reflect.String:
		return invokeStdlibJSONMarshal(w, rv.Interface())

	//----------------------------------------
	// Default

	default:
		panic(fmt.Sprintf("unsupported type %v", info.Type.Kind()))
	}
}

func (cdc *Codec) encodeReflectJSONInterface(w io.Writer, iinfo *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectJSONInterface")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Special case when rv is nil, just write "null".
	if rv.IsNil() {
		err = writeStr(w, `null`)
		return
	}

	// Get concrete non-pointer reflect value & type.
	var crv, isPtr, isNilPtr = derefPointers(rv.Elem())
	if isPtr && crv.Kind() == reflect.Interface {
		// See "MARKER: No interface-pointers" in codec.go
		panic("should not happen")
	}
	if isNilPtr {
		panic(fmt.Sprintf("Illegal nil-pointer of type %v for registered interface %v. "+
			"For compatibility with other languages, nil-pointer interface values are forbidden.", crv.Type(), iinfo.Type))
	}
	var crt = crv.Type()

	// Get *TypeInfo for concrete type.
	var cinfo *TypeInfo
	cinfo, err = cdc.getTypeInfo_wlock(crt)
	if err != nil {
		return
	}
	if !cinfo.Registered {
		err = fmt.Errorf("Cannot encode unregistered concrete type %v.", crt)
		return
	}

	// Write interface wrapper.
	// Part 1:
	err = writeStr(w, _fmt(`{"type":"%s","value":`, cinfo.Name))
	if err != nil {
		return
	}
	// Part 2:
	defer func() {
		if err != nil {
			return
		}
		err = writeStr(w, `}`)
	}()

	// NOTE: In the future, we may write disambiguation bytes
	// here, if it is only to be written for interface values.
	// Currently, go-amino JSON *always* writes disfix bytes for
	// all registered concrete types.

	err = cdc.encodeReflectJSON(w, cinfo, crv, fopts)
	return
}

func (cdc *Codec) encodeReflectJSONList(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectJSONList")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Special case when list is a nil slice, just write "null".
	// Empty slices and arrays are not encoded as "null".
	if rv.Kind() == reflect.Slice && rv.IsNil() {
		err = writeStr(w, `null`)
		return
	}

	ert := info.Type.Elem()
	length := rv.Len()

	switch ert.Kind() {

	case reflect.Uint8: // Special case: byte array
		// Write bytes in base64.
		// NOTE: Base64 encoding preserves the exact original number of bytes.
		// Get readable slice of bytes.
		bz := []byte(nil)
		if rv.CanAddr() {
			bz = rv.Slice(0, length).Bytes()
		} else {
			bz = make([]byte, length)
			reflect.Copy(reflect.ValueOf(bz), rv) // XXX: looks expensive!
		}
		jsonBytes := []byte(nil)
		jsonBytes, err = json.Marshal(bz) // base64 encode
		if err != nil {
			return
		}
		_, err = w.Write(jsonBytes)
		return

	default:
		// Open square bracket.
		err = writeStr(w, `[`)
		if err != nil {
			return
		}

		// Write elements with comma.
		var einfo *TypeInfo
		einfo, err = cdc.getTypeInfo_wlock(ert)
		if err != nil {
			return
		}
		for i := 0; i < length; i++ {
			// Get dereferenced element value and info.
			var erv, _, isNil = derefPointers(rv.Index(i))
			if isNil {
				err = writeStr(w, `null`)
			} else {
				err = cdc.encodeReflectJSON(w, einfo, erv, fopts)
			}
			if err != nil {
				return
			}
			// Add a comma if it isn't the last item.
			if i != length-1 {
				err = writeStr(w, `,`)
				if err != nil {
					return
				}
			}
		}

		// Close square bracket.
		defer func() {
			err = writeStr(w, `]`)
		}()
		return
	}
}

func (cdc *Codec) encodeReflectJSONStruct(w io.Writer, info *TypeInfo, rv reflect.Value, _ FieldOptions) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectJSONStruct")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Part 1.
	err = writeStr(w, `{`)
	if err != nil {
		return
	}
	// Part 2.
	defer func() {
		if err == nil {
			err = writeStr(w, `}`)
		}
	}()

	var writeComma = false
	for _, field := range info.Fields {
		// Get dereferenced field value and info.
		var frv, _, isNil = derefPointers(rv.Field(field.Index))
		var finfo *TypeInfo
		finfo, err = cdc.getTypeInfo_wlock(field.Type)
		if err != nil {
			return
		}
		// If frv is empty and omitempty, skip it.
		// NOTE: Unlike Amino:binary, we don't skip null fields unless "omitempty".
		if field.JSONOmitEmpty && isEmpty(frv, field.ZeroValue) {
			continue
		}
		// Now we know we're going to write something.
		// Add a comma if we need to.
		if writeComma {
			err = writeStr(w, `,`)
			if err != nil {
				return
			}
			writeComma = false
		}
		// Write field JSON name.
		err = invokeStdlibJSONMarshal(w, field.JSONName)
		if err != nil {
			return
		}
		// Write colon.
		err = writeStr(w, `:`)
		if err != nil {
			return
		}
		// Write field value.
		if isNil {
			err = writeStr(w, `null`)
		} else {
			err = cdc.encodeReflectJSON(w, finfo, frv, field.FieldOptions)
		}
		if err != nil {
			return
		}
		writeComma = true
	}
	return
}

// TODO: TEST
func (cdc *Codec) encodeReflectJSONMap(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectJSONMap")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Part 1.
	err = writeStr(w, `{`)
	if err != nil {
		return
	}
	// Part 2.
	defer func() {
		if err == nil {
			err = writeStr(w, `}`)
		}
	}()

	// Ensure that the map key type is a string.
	if rv.Type().Key().Kind() != reflect.String {
		err = errors.New("encodeReflectJSONMap: map key type must be a string")
		return
	}

	var writeComma = false
	for _, krv := range rv.MapKeys() {
		// Get dereferenced object value and info.
		var vrv, _, isNil = derefPointers(rv.MapIndex(krv))

		// Add a comma if we need to.
		if writeComma {
			err = writeStr(w, `,`)
			if err != nil {
				return
			}
			writeComma = false
		}
		// Write field name.
		err = invokeStdlibJSONMarshal(w, krv.Interface())
		if err != nil {
			return
		}
		// Write colon.
		err = writeStr(w, `:`)
		if err != nil {
			return
		}
		// Write field value.
		if isNil {
			err = writeStr(w, `null`)
		} else {
			var vinfo *TypeInfo
			vinfo, err = cdc.getTypeInfo_wlock(vrv.Type())
			if err != nil {
				return
			}
			err = cdc.encodeReflectJSON(w, vinfo, vrv, fopts) // pass through fopts
		}
		if err != nil {
			return
		}
		writeComma = true
	}
	return

}

//----------------------------------------
// Misc.

// CONTRACT: rv implements json.Marshaler.
func invokeMarshalJSON(w io.Writer, rv reflect.Value) error {
	blob, err := rv.Interface().(json.Marshaler).MarshalJSON()
	if err != nil {
		return err
	}
	_, err = w.Write(blob)
	return err
}

func invokeStdlibJSONMarshal(w io.Writer, v interface{}) error {
	// Note: Please don't stream out the output because that adds a newline
	// using json.NewEncoder(w).Encode(data)
	// as per https://golang.org/pkg/encoding/json/#Encoder.Encode
	blob, err := json.Marshal(v)
	if err != nil {
		return err
	}
	_, err = w.Write(blob)
	return err
}

func writeStr(w io.Writer, s string) (err error) {
	_, err = w.Write([]byte(s))
	return
}

func _fmt(s string, args ...interface{}) string {
	return fmt.Sprintf(s, args...)
}

// For json:",omitempty".
// Returns true for zero values, but also non-nil zero-length slices and strings.
func isEmpty(rv reflect.Value, zrv reflect.Value) bool {
	if !rv.IsValid() {
		return true
	}
	if reflect.DeepEqual(rv.Interface(), zrv.Interface()) {
		return true
	}
	switch rv.Kind() {
	case reflect.Slice, reflect.Array, reflect.String:
		if rv.Len() == 0 {
			return true
		}
	}
	return false
}
