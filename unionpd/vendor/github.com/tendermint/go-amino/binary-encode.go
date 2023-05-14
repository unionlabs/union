package amino

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"reflect"
	"time"

	"github.com/davecgh/go-spew/spew"
)

//----------------------------------------
// cdc.encodeReflectBinary

// This is the main entrypoint for encoding all types in binary form.  This
// function calls encodeReflectBinary*, and generally those functions should
// only call this one, for the prefix bytes are only written here.
// The value may be a nil interface, but not a nil pointer.
// The following contracts apply to all similar encode methods.
// CONTRACT: rv is not a pointer
// CONTRACT: rv is valid.
func (cdc *Codec) encodeReflectBinary(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions, bare bool) (err error) {
	if rv.Kind() == reflect.Ptr {
		panic("not allowed to be called with a reflect.Ptr")
	}
	if !rv.IsValid() {
		panic("not allowed to be called with invalid / zero Value")
	}
	if printLog {
		spew.Printf("(E) encodeReflectBinary(info: %v, rv: %#v (%v), fopts: %v)\n",
			info, rv.Interface(), rv.Type(), fopts)
		defer func() {
			fmt.Printf("(E) -> err: %v\n", err)
		}()
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
		err = cdc.encodeReflectBinary(w, rinfo, rrv, fopts, bare)
		return
	}

	switch info.Type.Kind() {

	//----------------------------------------
	// Complex

	case reflect.Interface:
		err = cdc.encodeReflectBinaryInterface(w, info, rv, fopts, bare)

	case reflect.Array:
		if info.Type.Elem().Kind() == reflect.Uint8 {
			err = cdc.encodeReflectBinaryByteArray(w, info, rv, fopts)
		} else {
			err = cdc.encodeReflectBinaryList(w, info, rv, fopts, bare)
		}

	case reflect.Slice:
		if info.Type.Elem().Kind() == reflect.Uint8 {
			err = cdc.encodeReflectBinaryByteSlice(w, info, rv, fopts)
		} else {
			err = cdc.encodeReflectBinaryList(w, info, rv, fopts, bare)
		}

	case reflect.Struct:
		err = cdc.encodeReflectBinaryStruct(w, info, rv, fopts, bare)

	//----------------------------------------
	// Signed

	case reflect.Int64:
		if fopts.BinFixed64 {
			err = EncodeInt64(w, rv.Int())
		} else {
			err = EncodeUvarint(w, uint64(rv.Int()))
		}

	case reflect.Int32:
		if fopts.BinFixed32 {
			err = EncodeInt32(w, int32(rv.Int()))
		} else {
			err = EncodeUvarint(w, uint64(rv.Int()))
		}

	case reflect.Int16:
		err = EncodeInt16(w, int16(rv.Int()))

	case reflect.Int8:
		err = EncodeInt8(w, int8(rv.Int()))

	case reflect.Int:
		err = EncodeUvarint(w, uint64(rv.Int()))

	//----------------------------------------
	// Unsigned

	case reflect.Uint64:
		if fopts.BinFixed64 {
			err = EncodeUint64(w, rv.Uint())
		} else {
			err = EncodeUvarint(w, rv.Uint())
		}

	case reflect.Uint32:
		if fopts.BinFixed32 {
			err = EncodeUint32(w, uint32(rv.Uint()))
		} else {
			err = EncodeUvarint(w, rv.Uint())
		}

	case reflect.Uint16:
		err = EncodeUint16(w, uint16(rv.Uint()))

	case reflect.Uint8:
		err = EncodeUint8(w, uint8(rv.Uint()))

	case reflect.Uint:
		err = EncodeUvarint(w, rv.Uint())

	//----------------------------------------
	// Misc

	case reflect.Bool:
		err = EncodeBool(w, rv.Bool())

	case reflect.Float64:
		if !fopts.Unsafe {
			err = errors.New("Amino float* support requires `amino:\"unsafe\"`.")
			return
		}
		err = EncodeFloat64(w, rv.Float())

	case reflect.Float32:
		if !fopts.Unsafe {
			err = errors.New("Amino float* support requires `amino:\"unsafe\"`.")
			return
		}
		err = EncodeFloat32(w, float32(rv.Float()))

	case reflect.String:
		err = EncodeString(w, rv.String())

	//----------------------------------------
	// Default

	default:
		panic(fmt.Sprintf("unsupported type %v", info.Type.Kind()))
	}

	return
}

func (cdc *Codec) encodeReflectBinaryInterface(w io.Writer, iinfo *TypeInfo, rv reflect.Value, fopts FieldOptions, bare bool) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectBinaryInterface")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Special case when rv is nil, write 0x00 to denote an empty byteslice.
	if rv.IsNil() {
		_, err = w.Write([]byte{0x00})
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

	// For Proto3 compatibility, encode interfaces as ByteLength.
	buf := bytes.NewBuffer(nil)

	// Write disambiguation bytes if needed.
	var needDisamb bool = false
	if iinfo.AlwaysDisambiguate {
		needDisamb = true
	} else if len(iinfo.Implementers[cinfo.Prefix]) > 1 {
		needDisamb = true
	}
	if needDisamb {
		_, err = buf.Write(append([]byte{0x00}, cinfo.Disamb[:]...))
		if err != nil {
			return
		}
	}

	// Write prefix bytes.
	_, err = buf.Write(cinfo.Prefix.Bytes())
	if err != nil {
		return
	}

	// Write actual concrete value.
	err = cdc.encodeReflectBinary(buf, cinfo, crv, fopts, true)
	if err != nil {
		return
	}

	if bare {
		// Write byteslice without byte-length prefixing.
		_, err = w.Write(buf.Bytes())
	} else {
		// Write byte-length prefixed byteslice.
		err = EncodeByteSlice(w, buf.Bytes())
	}
	return
}

func (cdc *Codec) encodeReflectBinaryByteArray(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	ert := info.Type.Elem()
	if ert.Kind() != reflect.Uint8 {
		panic("should not happen")
	}
	length := info.Type.Len()

	// Get byteslice.
	var byteslice = []byte(nil)
	if rv.CanAddr() {
		byteslice = rv.Slice(0, length).Bytes()
	} else {
		byteslice = make([]byte, length)
		reflect.Copy(reflect.ValueOf(byteslice), rv) // XXX: looks expensive!
	}

	// Write byte-length prefixed byteslice.
	err = EncodeByteSlice(w, byteslice)
	return
}

func (cdc *Codec) encodeReflectBinaryList(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions, bare bool) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectBinaryList")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}
	ert := info.Type.Elem()
	if ert.Kind() == reflect.Uint8 {
		panic("should not happen")
	}
	einfo, err := cdc.getTypeInfo_wlock(ert)
	if err != nil {
		return
	}

	// Proto3 byte-length prefixing incurs alloc cost on the encoder.
	// Here we incur it for unpacked form for ease of dev.
	buf := bytes.NewBuffer(nil)

	// If elem is not already a ByteLength type, write in packed form.
	// This is a Proto wart due to Proto backwards compatibility issues.
	// Amino2 will probably migrate to use the List typ3.  Please?  :)
	typ3 := typeToTyp3(einfo.Type, fopts)
	if typ3 != Typ3_ByteLength {
		// Write elems in packed form.
		for i := 0; i < rv.Len(); i++ {
			// Get dereferenced element value (or zero).
			var erv, _, _ = derefPointersZero(rv.Index(i))
			// Write the element value.
			err = cdc.encodeReflectBinary(buf, einfo, erv, fopts, false)
			if err != nil {
				return
			}
		}
	} else {
		// NOTE: ert is for the element value, while einfo.Type is dereferenced.
		isErtStructPointer := ert.Kind() == reflect.Ptr && einfo.Type.Kind() == reflect.Struct

		// Write elems in unpacked form.
		for i := 0; i < rv.Len(); i++ {
			// Write elements as repeated fields of the parent struct.
			err = encodeFieldNumberAndTyp3(buf, fopts.BinFieldNum, Typ3_ByteLength)
			if err != nil {
				return
			}
			// Get dereferenced element value and info.
			var erv, isDefault = isDefaultValue(rv.Index(i))
			if isDefault {
				// Special case if:
				//  - erv is a struct pointer and
				//  - field option has EmptyElements set
				if isErtStructPointer && fopts.EmptyElements {
					// NOTE: Not sure what to do here, but for future-proofing,
					// we explicitly fail on nil pointers, just like
					// Proto3's Golang client does.
					// This also makes it easier to upgrade to Amino2
					// which would enable the encoding of nil structs.
					return errors.New("nil struct pointers not supported when empty_elements field tag is set")
				}
				// Nothing to encode, so the length is 0.
				err = EncodeByte(buf, byte(0x00))
				if err != nil {
					return
				}
			} else {
				// Write the element value as a ByteLength.
				// In case of any inner lists in unpacked form.
				efopts := fopts
				efopts.BinFieldNum = 1
				err = cdc.encodeReflectBinary(buf, einfo, erv, efopts, false)
				if err != nil {
					return
				}
			}
		}
	}

	if bare {
		// Write byteslice without byte-length prefixing.
		_, err = w.Write(buf.Bytes())
	} else {
		// Write byte-length prefixed byteslice.
		err = EncodeByteSlice(w, buf.Bytes())
	}
	return
}

// CONTRACT: info.Type.Elem().Kind() == reflect.Uint8
func (cdc *Codec) encodeReflectBinaryByteSlice(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectBinaryByteSlice")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}
	ert := info.Type.Elem()
	if ert.Kind() != reflect.Uint8 {
		panic("should not happen")
	}

	// Write byte-length prefixed byte-slice.
	var byteslice = rv.Bytes()
	err = EncodeByteSlice(w, byteslice)
	return
}

func (cdc *Codec) encodeReflectBinaryStruct(w io.Writer, info *TypeInfo, rv reflect.Value, fopts FieldOptions, bare bool) (err error) {
	if printLog {
		fmt.Println("(e) encodeReflectBinaryBinaryStruct")
		defer func() {
			fmt.Printf("(e) -> err: %v\n", err)
		}()
	}

	// Proto3 incurs a cost in writing non-root structs.
	// Here we incur it for root structs as well for ease of dev.
	buf := bytes.NewBuffer(nil)

	switch info.Type {

	case timeType:
		// Special case: time.Time
		err = EncodeTime(buf, rv.Interface().(time.Time))
		if err != nil {
			return
		}

	default:
		for _, field := range info.Fields {
			// Get type info for field.
			var finfo *TypeInfo
			finfo, err = cdc.getTypeInfo_wlock(field.Type)
			if err != nil {
				return
			}
			// Get dereferenced field value and info.
			var frv = rv.Field(field.Index)
			var frvIsPtr = frv.Kind() == reflect.Ptr
			var dfrv, isDefault = isDefaultValue(frv)
			if isDefault && !fopts.WriteEmpty {
				// Do not encode default value fields
				// (except when `amino:"write_empty"` is set).
				continue
			}
			if field.UnpackedList {
				// Write repeated field entries for each list item.
				err = cdc.encodeReflectBinaryList(buf, finfo, dfrv, field.FieldOptions, true)
				if err != nil {
					return
				}
			} else {
				lBeforeKey := buf.Len()
				// Write field key (number and type).
				err = encodeFieldNumberAndTyp3(buf, field.BinFieldNum, typeToTyp3(finfo.Type, field.FieldOptions))
				if err != nil {
					return
				}
				lBeforeValue := buf.Len()

				// Write field value from rv.
				err = cdc.encodeReflectBinary(buf, finfo, dfrv, field.FieldOptions, false)
				if err != nil {
					return
				}
				lAfterValue := buf.Len()

				if !frvIsPtr && !fopts.WriteEmpty && lBeforeValue == lAfterValue-1 && buf.Bytes()[buf.Len()-1] == 0x00 {
					// rollback typ3/fieldnum and last byte if
					// not a pointer and empty:
					buf.Truncate(lBeforeKey)
				}

			}
		}
	}

	if bare {
		// Write byteslice without byte-length prefixing.
		_, err = w.Write(buf.Bytes())
	} else {
		// Write byte-length prefixed byteslice.
		err = EncodeByteSlice(w, buf.Bytes())
	}
	return
}

//----------------------------------------
// Misc.

// Write field key.
func encodeFieldNumberAndTyp3(w io.Writer, num uint32, typ Typ3) (err error) {
	if (typ & 0xF8) != 0 {
		panic(fmt.Sprintf("invalid Typ3 byte %v", typ))
	}
	if num < 0 || num > (1<<29-1) {
		panic(fmt.Sprintf("invalid field number %v", num))
	}

	// Pack Typ3 and field number.
	var value64 = (uint64(num) << 3) | uint64(typ)

	// Write uvarint value for field and Typ3.
	var buf [10]byte
	n := binary.PutUvarint(buf[:], value64)
	_, err = w.Write(buf[0:n])
	return
}
