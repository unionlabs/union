package amino

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"reflect"

	"github.com/davecgh/go-spew/spew"
)

//----------------------------------------
// cdc.decodeReflectJSON

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSON(bz []byte, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if info.Type.Kind() == reflect.Interface && rv.Kind() == reflect.Ptr {
		panic("should not happen")
	}
	if printLog {
		spew.Printf("(D) decodeReflectJSON(bz: %s, info: %v, rv: %#v (%v), fopts: %v)\n",
			bz, info, rv.Interface(), rv.Type(), fopts)
		defer func() {
			fmt.Printf("(D) -> err: %v\n", err)
		}()
	}

	// Special case for null for either interface, pointer, slice
	// NOTE: This doesn't match the binary implementation completely.
	if nullBytes(bz) {
		rv.Set(reflect.Zero(rv.Type()))
		return
	}

	// Dereference-and-construct pointers all the way.
	// This works for pointer-pointers.
	for rv.Kind() == reflect.Ptr {
		if rv.IsNil() {
			newPtr := reflect.New(rv.Type().Elem())
			rv.Set(newPtr)
		}
		rv = rv.Elem()
	}

	// Special case:
	if rv.Type() == timeType {
		// Amino time strips the timezone, so must end with Z.
		if len(bz) >= 2 && bz[0] == '"' && bz[len(bz)-1] == '"' {
			if bz[len(bz)-2] != 'Z' {
				err = fmt.Errorf("Amino:JSON time must be UTC and end with 'Z' but got %s.", bz)
				return
			}
		} else {
			err = fmt.Errorf("Amino:JSON time must be an RFC3339Nano string, but got %s.", bz)
			return
		}
	}

	// Handle override if a pointer to rv implements UnmarshalAminoJSON.
	if info.IsAminoJSONUnmarshaler {
		// First, decode repr instance from JSON.
		rrv := reflect.New(info.AminoJSONUnmarshalReprType).Elem()
		var rinfo *TypeInfo
		rinfo, err = cdc.getTypeInfo_wlock(info.AminoJSONUnmarshalReprType)
		if err != nil {
			return
		}
		err = cdc.decodeReflectJSON(bz, rinfo, rrv, fopts)
		if err != nil {
			return
		}
		// Then, decode from repr instance.
		uwrm := rv.Addr().MethodByName("UnmarshalAminoJSON")
		uwouts := uwrm.Call([]reflect.Value{rrv})
		erri := uwouts[0].Interface()
		if erri != nil {
			err = erri.(error)
		}
		return
	}

	// Handle override if a pointer to rv implements json.Unmarshaler.
	if rv.Addr().Type().Implements(jsonUnmarshalerType) {
		err = rv.Addr().Interface().(json.Unmarshaler).UnmarshalJSON(bz)
		return
	}

	// Handle override if a pointer to rv implements UnmarshalAmino.
	if info.IsAminoUnmarshaler {
		// First, decode repr instance from bytes.
		rrv, rinfo := reflect.New(info.AminoUnmarshalReprType).Elem(), (*TypeInfo)(nil)
		rinfo, err = cdc.getTypeInfo_wlock(info.AminoUnmarshalReprType)
		if err != nil {
			return
		}
		err = cdc.decodeReflectJSON(bz, rinfo, rrv, fopts)
		if err != nil {
			return
		}
		// Then, decode from repr instance.
		uwrm := rv.Addr().MethodByName("UnmarshalAmino")
		uwouts := uwrm.Call([]reflect.Value{rrv})
		erri := uwouts[0].Interface()
		if erri != nil {
			err = erri.(error)
		}
		return
	}

	switch ikind := info.Type.Kind(); ikind {

	//----------------------------------------
	// Complex

	case reflect.Interface:
		err = cdc.decodeReflectJSONInterface(bz, info, rv, fopts)

	case reflect.Array:
		err = cdc.decodeReflectJSONArray(bz, info, rv, fopts)

	case reflect.Slice:
		err = cdc.decodeReflectJSONSlice(bz, info, rv, fopts)

	case reflect.Struct:
		err = cdc.decodeReflectJSONStruct(bz, info, rv, fopts)

	case reflect.Map:
		err = cdc.decodeReflectJSONMap(bz, info, rv, fopts)

	//----------------------------------------
	// Signed, Unsigned

	case reflect.Int64, reflect.Int:
		fallthrough
	case reflect.Uint64, reflect.Uint:
		if bz[0] != '"' || bz[len(bz)-1] != '"' {
			err = fmt.Errorf("invalid character -- Amino:JSON int/int64/uint/uint64 expects quoted values for javascript numeric support, got: %v.", string(bz))
			if err != nil {
				return
			}
		}
		bz = bz[1 : len(bz)-1]
		fallthrough
	case reflect.Int32, reflect.Int16, reflect.Int8,
		reflect.Uint32, reflect.Uint16, reflect.Uint8:
		err = invokeStdlibJSONUnmarshal(bz, rv, fopts)

	//----------------------------------------
	// Misc

	case reflect.Float32, reflect.Float64:
		if !fopts.Unsafe {
			return errors.New("Amino:JSON float* support requires `amino:\"unsafe\"`.")
		}
		fallthrough
	case reflect.Bool, reflect.String:
		err = invokeStdlibJSONUnmarshal(bz, rv, fopts)

	//----------------------------------------
	// Default

	default:
		panic(fmt.Sprintf("unsupported type %v", info.Type.Kind()))
	}

	return
}

func invokeStdlibJSONUnmarshal(bz []byte, rv reflect.Value, fopts FieldOptions) error {
	if !rv.CanAddr() && rv.Kind() != reflect.Ptr {
		panic("rv not addressable nor pointer")
	}

	var rrv reflect.Value = rv
	if rv.Kind() != reflect.Ptr {
		rrv = reflect.New(rv.Type())
	}

	if err := json.Unmarshal(bz, rrv.Interface()); err != nil {
		return err
	}
	rv.Set(rrv.Elem())
	return nil
}

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSONInterface(bz []byte, iinfo *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if printLog {
		fmt.Println("(d) decodeReflectJSONInterface")
		defer func() {
			fmt.Printf("(d) -> err: %v\n", err)
		}()
	}

	/*
		We don't make use of user-provided interface values because there are a
		lot of edge cases.

		* What if the type is mismatched?
		* What if the JSON field entry is missing?
		* Circular references?
	*/
	if !rv.IsNil() {
		// We don't strictly need to set it nil, but lets keep it here for a
		// while in case we forget, for defensive purposes.
		rv.Set(iinfo.ZeroValue)
	}

	// Consume type wrapper info.
	name, bz, err := decodeInterfaceJSON(bz)
	if err != nil {
		return
	}
	// XXX: Check name against interface to make sure that it actually
	// matches, and return an error if it doesn't.

	// NOTE: Unlike decodeReflectBinaryInterface, we already dealt with nil in decodeReflectJSON.
	// NOTE: We also "consumed" the interface wrapper by replacing `bz` above.

	// Get concrete type info.
	// NOTE: Unlike decodeReflectBinaryInterface, uses the full name string.
	var cinfo *TypeInfo
	cinfo, err = cdc.getTypeInfoFromName_rlock(name)
	if err != nil {
		return
	}

	// Construct the concrete type.
	var crv, irvSet = constructConcreteType(cinfo)

	// Decode into the concrete type.
	err = cdc.decodeReflectJSON(bz, cinfo, crv, fopts)
	if err != nil {
		rv.Set(irvSet) // Helps with debugging
		return
	}

	// We need to set here, for when !PointerPreferred and the type
	// is say, an array of bytes (e.g. [32]byte), then we must call
	// rv.Set() *after* the value was acquired.
	rv.Set(irvSet)
	return
}

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSONArray(bz []byte, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if printLog {
		fmt.Println("(d) decodeReflectJSONArray")
		defer func() {
			fmt.Printf("(d) -> err: %v\n", err)
		}()
	}
	ert := info.Type.Elem()
	length := info.Type.Len()

	switch ert.Kind() {

	case reflect.Uint8: // Special case: byte array
		var buf []byte
		err = json.Unmarshal(bz, &buf)
		if err != nil {
			return
		}
		if len(buf) != length {
			err = fmt.Errorf("decodeReflectJSONArray: byte-length mismatch, got %v want %v",
				len(buf), length)
		}
		reflect.Copy(rv, reflect.ValueOf(buf))
		return

	default: // General case.
		var einfo *TypeInfo
		einfo, err = cdc.getTypeInfo_wlock(ert)
		if err != nil {
			return
		}

		// Read into rawSlice.
		var rawSlice []json.RawMessage
		if err = json.Unmarshal(bz, &rawSlice); err != nil {
			return
		}
		if len(rawSlice) != length {
			err = fmt.Errorf("decodeReflectJSONArray: length mismatch, got %v want %v", len(rawSlice), length)
			return
		}

		// Decode each item in rawSlice.
		for i := 0; i < length; i++ {
			erv := rv.Index(i)
			ebz := rawSlice[i]
			err = cdc.decodeReflectJSON(ebz, einfo, erv, fopts)
			if err != nil {
				return
			}
		}
		return
	}
}

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSONSlice(bz []byte, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if printLog {
		fmt.Println("(d) decodeReflectJSONSlice")
		defer func() {
			fmt.Printf("(d) -> err: %v\n", err)
		}()
	}

	var ert = info.Type.Elem()

	switch ert.Kind() {

	case reflect.Uint8: // Special case: byte slice
		err = json.Unmarshal(bz, rv.Addr().Interface())
		if err != nil {
			return
		}
		if rv.Len() == 0 {
			// Special case when length is 0.
			// NOTE: We prefer nil slices.
			rv.Set(info.ZeroValue)
		} else {
			// NOTE: Already set via json.Unmarshal() above.
		}
		return

	default: // General case.
		var einfo *TypeInfo
		einfo, err = cdc.getTypeInfo_wlock(ert)
		if err != nil {
			return
		}

		// Read into rawSlice.
		var rawSlice []json.RawMessage
		if err = json.Unmarshal(bz, &rawSlice); err != nil {
			return
		}

		// Special case when length is 0.
		// NOTE: We prefer nil slices.
		var length = len(rawSlice)
		if length == 0 {
			rv.Set(info.ZeroValue)
			return
		}

		// Read into a new slice.
		var esrt = reflect.SliceOf(ert) // TODO could be optimized.
		var srv = reflect.MakeSlice(esrt, length, length)
		for i := 0; i < length; i++ {
			erv := srv.Index(i)
			ebz := rawSlice[i]
			err = cdc.decodeReflectJSON(ebz, einfo, erv, fopts)
			if err != nil {
				return
			}
		}

		// TODO do we need this extra step?
		rv.Set(srv)
		return
	}
}

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSONStruct(bz []byte, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if printLog {
		fmt.Println("(d) decodeReflectJSONStruct")
		defer func() {
			fmt.Printf("(d) -> err: %v\n", err)
		}()
	}

	// Map all the fields(keys) to their blobs/bytes.
	// NOTE: In decodeReflectBinaryStruct, we don't need to do this,
	// since fields are encoded in order.
	var rawMap = make(map[string]json.RawMessage)
	err = json.Unmarshal(bz, &rawMap)
	if err != nil {
		return
	}

	for _, field := range info.Fields {

		// Get field rv and info.
		var frv = rv.Field(field.Index)
		var finfo *TypeInfo
		finfo, err = cdc.getTypeInfo_wlock(field.Type)
		if err != nil {
			return
		}

		// Get value from rawMap.
		var valueBytes = rawMap[field.JSONName]
		if len(valueBytes) == 0 {
			// TODO: Since the Go stdlib's JSON codec allows case-insensitive
			// keys perhaps we need to also do case-insensitive lookups here.
			// So "Vanilla" and "vanilla" would both match to the same field.
			// It is actually a security flaw with encoding/json library
			// - See https://github.com/golang/go/issues/14750
			// but perhaps we are aiming for as much compatibility here.
			// JAE: I vote we depart from encoding/json, than carry a vuln.

			// Set to the zero value only if not omitempty
			if !field.JSONOmitEmpty {
				// Set nil/zero on frv.
				frv.Set(reflect.Zero(frv.Type()))
			}

			continue
		}

		// Decode into field rv.
		err = cdc.decodeReflectJSON(valueBytes, finfo, frv, fopts)
		if err != nil {
			return
		}
	}

	return nil
}

// CONTRACT: rv.CanAddr() is true.
func (cdc *Codec) decodeReflectJSONMap(bz []byte, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if !rv.CanAddr() {
		panic("rv not addressable")
	}
	if printLog {
		fmt.Println("(d) decodeReflectJSONMap")
		defer func() {
			fmt.Printf("(d) -> err: %v\n", err)
		}()
	}

	// Map all the fields(keys) to their blobs/bytes.
	// NOTE: In decodeReflectBinaryMap, we don't need to do this,
	// since fields are encoded in order.
	var rawMap = make(map[string]json.RawMessage)
	err = json.Unmarshal(bz, &rawMap)
	if err != nil {
		return
	}

	var krt = rv.Type().Key()
	if krt.Kind() != reflect.String {
		err = fmt.Errorf("decodeReflectJSONMap: key type must be string") // TODO also support []byte and maybe others
		return
	}
	var vinfo *TypeInfo
	vinfo, err = cdc.getTypeInfo_wlock(rv.Type().Elem())
	if err != nil {
		return
	}

	var mrv = reflect.MakeMapWithSize(rv.Type(), len(rawMap))
	for key, valueBytes := range rawMap {

		// Get map value rv.
		vrv := reflect.New(mrv.Type().Elem()).Elem()

		// Decode valueBytes into vrv.
		err = cdc.decodeReflectJSON(valueBytes, vinfo, vrv, fopts)
		if err != nil {
			return
		}

		// And set.
		krv := reflect.New(reflect.TypeOf("")).Elem()
		krv.SetString(key)
		mrv.SetMapIndex(krv, vrv)
	}
	rv.Set(mrv)

	return nil
}

//----------------------------------------
// Misc.

type disfixWrapper struct {
	Name string          `json:"type"`
	Data json.RawMessage `json:"value"`
}

// decodeInterfaceJSON helps unravel the type name and
// the stored data, which are expected in the form:
// {
//    "type": "<canonical concrete type name>",
//    "value":  {}
// }
func decodeInterfaceJSON(bz []byte) (name string, data []byte, err error) {
	dfw := new(disfixWrapper)
	err = json.Unmarshal(bz, dfw)
	if err != nil {
		err = fmt.Errorf("cannot parse disfix JSON wrapper: %v", err)
		return
	}

	// Get name.
	if dfw.Name == "" {
		err = errors.New("JSON encoding of interfaces require non-empty type field.")
		return
	}
	name = dfw.Name

	// Get data.
	if len(dfw.Data) == 0 {
		err = errors.New("interface JSON wrapper should have non-empty value field")
		return
	}
	data = dfw.Data
	return
}

func nullBytes(b []byte) bool {
	return bytes.Equal(b, []byte(`null`))
}
