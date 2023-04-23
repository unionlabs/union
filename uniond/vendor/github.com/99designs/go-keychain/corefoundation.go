// +build darwin ios

package keychain

/*
#cgo LDFLAGS: -framework CoreFoundation

#include <CoreFoundation/CoreFoundation.h>

// Can't cast a *uintptr to *unsafe.Pointer in Go, and casting
// C.CFTypeRef to unsafe.Pointer is unsafe in Go, so have shim functions to
// do the casting in C (where it's safe).

// We add a suffix to the C functions below, because we copied this
// file from go-kext, which means that any project that depends on this
// package and go-kext would run into duplicate symbol errors otherwise.
//
// TODO: Move this file into its own package depended on by go-kext
// and this package.

CFDictionaryRef CFDictionaryCreateSafe2(CFAllocatorRef allocator, const uintptr_t *keys, const uintptr_t *values, CFIndex numValues, const CFDictionaryKeyCallBacks *keyCallBacks, const CFDictionaryValueCallBacks *valueCallBacks) {
  return CFDictionaryCreate(allocator, (const void **)keys, (const void **)values, numValues, keyCallBacks, valueCallBacks);
}

CFArrayRef CFArrayCreateSafe2(CFAllocatorRef allocator, const uintptr_t *values, CFIndex numValues, const CFArrayCallBacks *callBacks) {
  return CFArrayCreate(allocator, (const void **)values, numValues, callBacks);
}
*/
import "C"
import (
	"errors"
	"fmt"
	"math"
	"reflect"
	"unicode/utf8"
	"unsafe"
)

// Release releases memory pointed to by a CFTypeRef.
func Release(ref C.CFTypeRef) {
	C.CFRelease(ref)
}

// BytesToCFData will return a CFDataRef and if non-nil, must be released with
// Release(ref).
func BytesToCFData(b []byte) (C.CFDataRef, error) {
	if uint64(len(b)) > math.MaxUint32 {
		return 0, errors.New("Data is too large")
	}
	var p *C.UInt8
	if len(b) > 0 {
		p = (*C.UInt8)(&b[0])
	}
	cfData := C.CFDataCreate(C.kCFAllocatorDefault, p, C.CFIndex(len(b)))
	if cfData == 0 {
		return 0, fmt.Errorf("CFDataCreate failed")
	}
	return cfData, nil
}

// CFDataToBytes converts CFData to bytes.
func CFDataToBytes(cfData C.CFDataRef) ([]byte, error) {
	return C.GoBytes(unsafe.Pointer(C.CFDataGetBytePtr(cfData)), C.int(C.CFDataGetLength(cfData))), nil
}

// MapToCFDictionary will return a CFDictionaryRef and if non-nil, must be
// released with Release(ref).
func MapToCFDictionary(m map[C.CFTypeRef]C.CFTypeRef) (C.CFDictionaryRef, error) {
	var keys, values []C.uintptr_t
	for key, value := range m {
		keys = append(keys, C.uintptr_t(key))
		values = append(values, C.uintptr_t(value))
	}
	numValues := len(values)
	var keysPointer, valuesPointer *C.uintptr_t
	if numValues > 0 {
		keysPointer = &keys[0]
		valuesPointer = &values[0]
	}
	cfDict := C.CFDictionaryCreateSafe2(C.kCFAllocatorDefault, keysPointer, valuesPointer, C.CFIndex(numValues),
		&C.kCFTypeDictionaryKeyCallBacks, &C.kCFTypeDictionaryValueCallBacks) //nolint
	if cfDict == 0 {
		return 0, fmt.Errorf("CFDictionaryCreate failed")
	}
	return cfDict, nil
}

// CFDictionaryToMap converts CFDictionaryRef to a map.
func CFDictionaryToMap(cfDict C.CFDictionaryRef) (m map[C.CFTypeRef]C.CFTypeRef) {
	count := C.CFDictionaryGetCount(cfDict)
	if count > 0 {
		keys := make([]C.CFTypeRef, count)
		values := make([]C.CFTypeRef, count)
		C.CFDictionaryGetKeysAndValues(cfDict, (*unsafe.Pointer)(unsafe.Pointer(&keys[0])), (*unsafe.Pointer)(unsafe.Pointer(&values[0])))
		m = make(map[C.CFTypeRef]C.CFTypeRef, count)
		for i := C.CFIndex(0); i < count; i++ {
			m[keys[i]] = values[i]
		}
	}
	return
}

// StringToCFString will return a CFStringRef and if non-nil, must be released with
// Release(ref).
func StringToCFString(s string) (C.CFStringRef, error) {
	if !utf8.ValidString(s) {
		return 0, errors.New("Invalid UTF-8 string")
	}
	if uint64(len(s)) > math.MaxUint32 {
		return 0, errors.New("String is too large")
	}

	bytes := []byte(s)
	var p *C.UInt8
	if len(bytes) > 0 {
		p = (*C.UInt8)(&bytes[0])
	}
	return C.CFStringCreateWithBytes(C.kCFAllocatorDefault, p, C.CFIndex(len(s)), C.kCFStringEncodingUTF8, C.false), nil
}

// CFStringToString converts a CFStringRef to a string.
func CFStringToString(s C.CFStringRef) string {
	p := C.CFStringGetCStringPtr(s, C.kCFStringEncodingUTF8)
	if p != nil {
		return C.GoString(p)
	}
	length := C.CFStringGetLength(s)
	if length == 0 {
		return ""
	}
	maxBufLen := C.CFStringGetMaximumSizeForEncoding(length, C.kCFStringEncodingUTF8)
	if maxBufLen == 0 {
		return ""
	}
	buf := make([]byte, maxBufLen)
	var usedBufLen C.CFIndex
	_ = C.CFStringGetBytes(s, C.CFRange{0, length}, C.kCFStringEncodingUTF8, C.UInt8(0), C.false, (*C.UInt8)(&buf[0]), maxBufLen, &usedBufLen)
	return string(buf[:usedBufLen])
}

// ArrayToCFArray will return a CFArrayRef and if non-nil, must be released with
// Release(ref).
func ArrayToCFArray(a []C.CFTypeRef) C.CFArrayRef {
	var values []C.uintptr_t
	for _, value := range a {
		values = append(values, C.uintptr_t(value))
	}
	numValues := len(values)
	var valuesPointer *C.uintptr_t
	if numValues > 0 {
		valuesPointer = &values[0]
	}
	return C.CFArrayCreateSafe2(C.kCFAllocatorDefault, valuesPointer, C.CFIndex(numValues), &C.kCFTypeArrayCallBacks) //nolint
}

// CFArrayToArray converts a CFArrayRef to an array of CFTypes.
func CFArrayToArray(cfArray C.CFArrayRef) (a []C.CFTypeRef) {
	count := C.CFArrayGetCount(cfArray)
	if count > 0 {
		a = make([]C.CFTypeRef, count)
		C.CFArrayGetValues(cfArray, C.CFRange{0, count}, (*unsafe.Pointer)(unsafe.Pointer(&a[0])))
	}
	return
}

// Convertable knows how to convert an instance to a CFTypeRef.
type Convertable interface {
	Convert() (C.CFTypeRef, error)
}

// ConvertMapToCFDictionary converts a map to a CFDictionary and if non-nil,
// must be released with Release(ref).
func ConvertMapToCFDictionary(attr map[string]interface{}) (C.CFDictionaryRef, error) {
	m := make(map[C.CFTypeRef]C.CFTypeRef)
	for key, i := range attr {
		var valueRef C.CFTypeRef
		switch val := i.(type) {
		default:
			return 0, fmt.Errorf("Unsupported value type: %v", reflect.TypeOf(i))
		case C.CFTypeRef:
			valueRef = val
		case bool:
			if val {
				valueRef = C.CFTypeRef(C.kCFBooleanTrue)
			} else {
				valueRef = C.CFTypeRef(C.kCFBooleanFalse)
			}
		case []byte:
			bytesRef, err := BytesToCFData(val)
			if err != nil {
				return 0, err
			}
			valueRef = C.CFTypeRef(bytesRef)
			defer Release(valueRef)
		case string:
			stringRef, err := StringToCFString(val)
			if err != nil {
				return 0, err
			}
			valueRef = C.CFTypeRef(stringRef)
			defer Release(valueRef)
		case Convertable:
			convertedRef, err := val.Convert()
			if err != nil {
				return 0, err
			}
			valueRef = convertedRef
			defer Release(valueRef)
		}
		keyRef, err := StringToCFString(key)
		if err != nil {
			return 0, err
		}
		m[C.CFTypeRef(keyRef)] = valueRef
		defer Release(C.CFTypeRef(keyRef))
	}

	cfDict, err := MapToCFDictionary(m)
	if err != nil {
		return 0, err
	}
	return cfDict, nil
}

// CFTypeDescription returns type string for CFTypeRef.
func CFTypeDescription(ref C.CFTypeRef) string {
	typeID := C.CFGetTypeID(ref)
	typeDesc := C.CFCopyTypeIDDescription(typeID)
	defer Release(C.CFTypeRef(typeDesc))
	return CFStringToString(typeDesc)
}

// Convert converts a CFTypeRef to a go instance.
func Convert(ref C.CFTypeRef) (interface{}, error) {
	typeID := C.CFGetTypeID(ref)
	if typeID == C.CFStringGetTypeID() {
		return CFStringToString(C.CFStringRef(ref)), nil
	} else if typeID == C.CFDictionaryGetTypeID() {
		return ConvertCFDictionary(C.CFDictionaryRef(ref))
	} else if typeID == C.CFArrayGetTypeID() {
		arr := CFArrayToArray(C.CFArrayRef(ref))
		results := make([]interface{}, 0, len(arr))
		for _, ref := range arr {
			v, err := Convert(ref)
			if err != nil {
				return nil, err
			}
			results = append(results, v)
		}
		return results, nil
	} else if typeID == C.CFDataGetTypeID() {
		b, err := CFDataToBytes(C.CFDataRef(ref))
		if err != nil {
			return nil, err
		}
		return b, nil
	} else if typeID == C.CFNumberGetTypeID() {
		return CFNumberToInterface(C.CFNumberRef(ref)), nil
	} else if typeID == C.CFBooleanGetTypeID() {
		if C.CFBooleanGetValue(C.CFBooleanRef(ref)) != 0 {
			return true, nil
		}
		return false, nil
	}

	return nil, fmt.Errorf("Invalid type: %s", CFTypeDescription(ref))
}

// ConvertCFDictionary converts a CFDictionary to map (deep).
func ConvertCFDictionary(d C.CFDictionaryRef) (map[interface{}]interface{}, error) {
	m := CFDictionaryToMap(d)
	result := make(map[interface{}]interface{})

	for k, v := range m {
		gk, err := Convert(k)
		if err != nil {
			return nil, err
		}
		gv, err := Convert(v)
		if err != nil {
			return nil, err
		}
		result[gk] = gv
	}
	return result, nil
}

// CFNumberToInterface converts the CFNumberRef to the most appropriate numeric
// type.
// This code is from github.com/kballard/go-osx-plist.
func CFNumberToInterface(cfNumber C.CFNumberRef) interface{} {
	typ := C.CFNumberGetType(cfNumber)
	switch typ {
	case C.kCFNumberSInt8Type:
		var sint C.SInt8
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&sint)) //nolint
		return int8(sint)
	case C.kCFNumberSInt16Type:
		var sint C.SInt16
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&sint)) //nolint
		return int16(sint)
	case C.kCFNumberSInt32Type:
		var sint C.SInt32
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&sint)) //nolint
		return int32(sint)
	case C.kCFNumberSInt64Type:
		var sint C.SInt64
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&sint)) //nolint
		return int64(sint)
	case C.kCFNumberFloat32Type:
		var float C.Float32
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&float)) //nolint
		return float32(float)
	case C.kCFNumberFloat64Type:
		var float C.Float64
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&float)) //nolint
		return float64(float)
	case C.kCFNumberCharType:
		var char C.char
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&char)) //nolint
		return byte(char)
	case C.kCFNumberShortType:
		var short C.short
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&short)) //nolint
		return int16(short)
	case C.kCFNumberIntType:
		var i C.int
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&i)) //nolint
		return int32(i)
	case C.kCFNumberLongType:
		var long C.long
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&long)) //nolint
		return int(long)
	case C.kCFNumberLongLongType:
		// This is the only type that may actually overflow us
		var longlong C.longlong
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&longlong)) //nolint
		return int64(longlong)
	case C.kCFNumberFloatType:
		var float C.float
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&float)) //nolint
		return float32(float)
	case C.kCFNumberDoubleType:
		var double C.double
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&double)) //nolint
		return float64(double)
	case C.kCFNumberCFIndexType:
		// CFIndex is a long
		var index C.CFIndex
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&index)) //nolint
		return int(index)
	case C.kCFNumberNSIntegerType:
		// We don't have a definition of NSInteger, but we know it's either an int or a long
		var nsInt C.long
		C.CFNumberGetValue(cfNumber, typ, unsafe.Pointer(&nsInt)) //nolint
		return int(nsInt)
	}
	panic("Unknown CFNumber type")
}
