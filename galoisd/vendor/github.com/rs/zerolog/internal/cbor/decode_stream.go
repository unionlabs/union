package cbor

// This file contains code to decode a stream of CBOR Data into JSON.

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"math"
	"net"
	"runtime"
	"strconv"
	"strings"
	"time"
	"unicode/utf8"
)

var decodeTimeZone *time.Location

const hexTable = "0123456789abcdef"

const isFloat32 = 4
const isFloat64 = 8

func readNBytes(src *bufio.Reader, n int) []byte {
	ret := make([]byte, n)
	for i := 0; i < n; i++ {
		ch, e := src.ReadByte()
		if e != nil {
			panic(fmt.Errorf("Tried to Read %d Bytes.. But hit end of file", n))
		}
		ret[i] = ch
	}
	return ret
}

func readByte(src *bufio.Reader) byte {
	b, e := src.ReadByte()
	if e != nil {
		panic(fmt.Errorf("Tried to Read 1 Byte.. But hit end of file"))
	}
	return b
}

func decodeIntAdditionalType(src *bufio.Reader, minor byte) int64 {
	val := int64(0)
	if minor <= 23 {
		val = int64(minor)
	} else {
		bytesToRead := 0
		switch minor {
		case additionalTypeIntUint8:
			bytesToRead = 1
		case additionalTypeIntUint16:
			bytesToRead = 2
		case additionalTypeIntUint32:
			bytesToRead = 4
		case additionalTypeIntUint64:
			bytesToRead = 8
		default:
			panic(fmt.Errorf("Invalid Additional Type: %d in decodeInteger (expected <28)", minor))
		}
		pb := readNBytes(src, bytesToRead)
		for i := 0; i < bytesToRead; i++ {
			val = val * 256
			val += int64(pb[i])
		}
	}
	return val
}

func decodeInteger(src *bufio.Reader) int64 {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeUnsignedInt && major != majorTypeNegativeInt {
		panic(fmt.Errorf("Major type is: %d in decodeInteger!! (expected 0 or 1)", major))
	}
	val := decodeIntAdditionalType(src, minor)
	if major == 0 {
		return val
	}
	return (-1 - val)
}

func decodeFloat(src *bufio.Reader) (float64, int) {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeSimpleAndFloat {
		panic(fmt.Errorf("Incorrect Major type is: %d in decodeFloat", major))
	}

	switch minor {
	case additionalTypeFloat16:
		panic(fmt.Errorf("float16 is not suppported in decodeFloat"))

	case additionalTypeFloat32:
		pb := readNBytes(src, 4)
		switch string(pb) {
		case float32Nan:
			return math.NaN(), isFloat32
		case float32PosInfinity:
			return math.Inf(0), isFloat32
		case float32NegInfinity:
			return math.Inf(-1), isFloat32
		}
		n := uint32(0)
		for i := 0; i < 4; i++ {
			n = n * 256
			n += uint32(pb[i])
		}
		val := math.Float32frombits(n)
		return float64(val), isFloat32
	case additionalTypeFloat64:
		pb := readNBytes(src, 8)
		switch string(pb) {
		case float64Nan:
			return math.NaN(), isFloat64
		case float64PosInfinity:
			return math.Inf(0), isFloat64
		case float64NegInfinity:
			return math.Inf(-1), isFloat64
		}
		n := uint64(0)
		for i := 0; i < 8; i++ {
			n = n * 256
			n += uint64(pb[i])
		}
		val := math.Float64frombits(n)
		return val, isFloat64
	}
	panic(fmt.Errorf("Invalid Additional Type: %d in decodeFloat", minor))
}

func decodeStringComplex(dst []byte, s string, pos uint) []byte {
	i := int(pos)
	start := 0

	for i < len(s) {
		b := s[i]
		if b >= utf8.RuneSelf {
			r, size := utf8.DecodeRuneInString(s[i:])
			if r == utf8.RuneError && size == 1 {
				// In case of error, first append previous simple characters to
				// the byte slice if any and append a replacement character code
				// in place of the invalid sequence.
				if start < i {
					dst = append(dst, s[start:i]...)
				}
				dst = append(dst, `\ufffd`...)
				i += size
				start = i
				continue
			}
			i += size
			continue
		}
		if b >= 0x20 && b <= 0x7e && b != '\\' && b != '"' {
			i++
			continue
		}
		// We encountered a character that needs to be encoded.
		// Let's append the previous simple characters to the byte slice
		// and switch our operation to read and encode the remainder
		// characters byte-by-byte.
		if start < i {
			dst = append(dst, s[start:i]...)
		}
		switch b {
		case '"', '\\':
			dst = append(dst, '\\', b)
		case '\b':
			dst = append(dst, '\\', 'b')
		case '\f':
			dst = append(dst, '\\', 'f')
		case '\n':
			dst = append(dst, '\\', 'n')
		case '\r':
			dst = append(dst, '\\', 'r')
		case '\t':
			dst = append(dst, '\\', 't')
		default:
			dst = append(dst, '\\', 'u', '0', '0', hexTable[b>>4], hexTable[b&0xF])
		}
		i++
		start = i
	}
	if start < len(s) {
		dst = append(dst, s[start:]...)
	}
	return dst
}

func decodeString(src *bufio.Reader, noQuotes bool) []byte {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeByteString {
		panic(fmt.Errorf("Major type is: %d in decodeString", major))
	}
	result := []byte{}
	if !noQuotes {
		result = append(result, '"')
	}
	length := decodeIntAdditionalType(src, minor)
	len := int(length)
	pbs := readNBytes(src, len)
	result = append(result, pbs...)
	if noQuotes {
		return result
	}
	return append(result, '"')
}

func decodeUTF8String(src *bufio.Reader) []byte {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeUtf8String {
		panic(fmt.Errorf("Major type is: %d in decodeUTF8String", major))
	}
	result := []byte{'"'}
	length := decodeIntAdditionalType(src, minor)
	len := int(length)
	pbs := readNBytes(src, len)

	for i := 0; i < len; i++ {
		// Check if the character needs encoding. Control characters, slashes,
		// and the double quote need json encoding. Bytes above the ascii
		// boundary needs utf8 encoding.
		if pbs[i] < 0x20 || pbs[i] > 0x7e || pbs[i] == '\\' || pbs[i] == '"' {
			// We encountered a character that needs to be encoded. Switch
			// to complex version of the algorithm.
			dst := []byte{'"'}
			dst = decodeStringComplex(dst, string(pbs), uint(i))
			return append(dst, '"')
		}
	}
	// The string has no need for encoding and therefore is directly
	// appended to the byte slice.
	result = append(result, pbs...)
	return append(result, '"')
}

func array2Json(src *bufio.Reader, dst io.Writer) {
	dst.Write([]byte{'['})
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeArray {
		panic(fmt.Errorf("Major type is: %d in array2Json", major))
	}
	len := 0
	unSpecifiedCount := false
	if minor == additionalTypeInfiniteCount {
		unSpecifiedCount = true
	} else {
		length := decodeIntAdditionalType(src, minor)
		len = int(length)
	}
	for i := 0; unSpecifiedCount || i < len; i++ {
		if unSpecifiedCount {
			pb, e := src.Peek(1)
			if e != nil {
				panic(e)
			}
			if pb[0] == majorTypeSimpleAndFloat|additionalTypeBreak {
				readByte(src)
				break
			}
		}
		cbor2JsonOneObject(src, dst)
		if unSpecifiedCount {
			pb, e := src.Peek(1)
			if e != nil {
				panic(e)
			}
			if pb[0] == majorTypeSimpleAndFloat|additionalTypeBreak {
				readByte(src)
				break
			}
			dst.Write([]byte{','})
		} else if i+1 < len {
			dst.Write([]byte{','})
		}
	}
	dst.Write([]byte{']'})
}

func map2Json(src *bufio.Reader, dst io.Writer) {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeMap {
		panic(fmt.Errorf("Major type is: %d in map2Json", major))
	}
	len := 0
	unSpecifiedCount := false
	if minor == additionalTypeInfiniteCount {
		unSpecifiedCount = true
	} else {
		length := decodeIntAdditionalType(src, minor)
		len = int(length)
	}
	dst.Write([]byte{'{'})
	for i := 0; unSpecifiedCount || i < len; i++ {
		if unSpecifiedCount {
			pb, e := src.Peek(1)
			if e != nil {
				panic(e)
			}
			if pb[0] == majorTypeSimpleAndFloat|additionalTypeBreak {
				readByte(src)
				break
			}
		}
		cbor2JsonOneObject(src, dst)
		if i%2 == 0 {
			// Even position values are keys.
			dst.Write([]byte{':'})
		} else {
			if unSpecifiedCount {
				pb, e := src.Peek(1)
				if e != nil {
					panic(e)
				}
				if pb[0] == majorTypeSimpleAndFloat|additionalTypeBreak {
					readByte(src)
					break
				}
				dst.Write([]byte{','})
			} else if i+1 < len {
				dst.Write([]byte{','})
			}
		}
	}
	dst.Write([]byte{'}'})
}

func decodeTagData(src *bufio.Reader) []byte {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeTags {
		panic(fmt.Errorf("Major type is: %d in decodeTagData", major))
	}
	switch minor {
	case additionalTypeTimestamp:
		return decodeTimeStamp(src)

	// Tag value is larger than 256 (so uint16).
	case additionalTypeIntUint16:
		val := decodeIntAdditionalType(src, minor)

		switch uint16(val) {
		case additionalTypeEmbeddedJSON:
			pb := readByte(src)
			dataMajor := pb & maskOutAdditionalType
			if dataMajor != majorTypeByteString {
				panic(fmt.Errorf("Unsupported embedded Type: %d in decodeEmbeddedJSON", dataMajor))
			}
			src.UnreadByte()
			return decodeString(src, true)

		case additionalTypeTagNetworkAddr:
			octets := decodeString(src, true)
			ss := []byte{'"'}
			switch len(octets) {
			case 6: // MAC address.
				ha := net.HardwareAddr(octets)
				ss = append(append(ss, ha.String()...), '"')
			case 4: // IPv4 address.
				fallthrough
			case 16: // IPv6 address.
				ip := net.IP(octets)
				ss = append(append(ss, ip.String()...), '"')
			default:
				panic(fmt.Errorf("Unexpected Network Address length: %d (expected 4,6,16)", len(octets)))
			}
			return ss

		case additionalTypeTagNetworkPrefix:
			pb := readByte(src)
			if pb != majorTypeMap|0x1 {
				panic(fmt.Errorf("IP Prefix is NOT of MAP of 1 elements as expected"))
			}
			octets := decodeString(src, true)
			val := decodeInteger(src)
			ip := net.IP(octets)
			var mask net.IPMask
			pfxLen := int(val)
			if len(octets) == 4 {
				mask = net.CIDRMask(pfxLen, 32)
			} else {
				mask = net.CIDRMask(pfxLen, 128)
			}
			ipPfx := net.IPNet{IP: ip, Mask: mask}
			ss := []byte{'"'}
			ss = append(append(ss, ipPfx.String()...), '"')
			return ss

		case additionalTypeTagHexString:
			octets := decodeString(src, true)
			ss := []byte{'"'}
			for _, v := range octets {
				ss = append(ss, hexTable[v>>4], hexTable[v&0x0f])
			}
			return append(ss, '"')

		default:
			panic(fmt.Errorf("Unsupported Additional Tag Type: %d in decodeTagData", val))
		}
	}
	panic(fmt.Errorf("Unsupported Additional Type: %d in decodeTagData", minor))
}

func decodeTimeStamp(src *bufio.Reader) []byte {
	pb := readByte(src)
	src.UnreadByte()
	tsMajor := pb & maskOutAdditionalType
	if tsMajor == majorTypeUnsignedInt || tsMajor == majorTypeNegativeInt {
		n := decodeInteger(src)
		t := time.Unix(n, 0)
		if decodeTimeZone != nil {
			t = t.In(decodeTimeZone)
		} else {
			t = t.In(time.UTC)
		}
		tsb := []byte{}
		tsb = append(tsb, '"')
		tsb = t.AppendFormat(tsb, IntegerTimeFieldFormat)
		tsb = append(tsb, '"')
		return tsb
	} else if tsMajor == majorTypeSimpleAndFloat {
		n, _ := decodeFloat(src)
		secs := int64(n)
		n -= float64(secs)
		n *= float64(1e9)
		t := time.Unix(secs, int64(n))
		if decodeTimeZone != nil {
			t = t.In(decodeTimeZone)
		} else {
			t = t.In(time.UTC)
		}
		tsb := []byte{}
		tsb = append(tsb, '"')
		tsb = t.AppendFormat(tsb, NanoTimeFieldFormat)
		tsb = append(tsb, '"')
		return tsb
	}
	panic(fmt.Errorf("TS format is neigther int nor float: %d", tsMajor))
}

func decodeSimpleFloat(src *bufio.Reader) []byte {
	pb := readByte(src)
	major := pb & maskOutAdditionalType
	minor := pb & maskOutMajorType
	if major != majorTypeSimpleAndFloat {
		panic(fmt.Errorf("Major type is: %d in decodeSimpleFloat", major))
	}
	switch minor {
	case additionalTypeBoolTrue:
		return []byte("true")
	case additionalTypeBoolFalse:
		return []byte("false")
	case additionalTypeNull:
		return []byte("null")
	case additionalTypeFloat16:
		fallthrough
	case additionalTypeFloat32:
		fallthrough
	case additionalTypeFloat64:
		src.UnreadByte()
		v, bc := decodeFloat(src)
		ba := []byte{}
		switch {
		case math.IsNaN(v):
			return []byte("\"NaN\"")
		case math.IsInf(v, 1):
			return []byte("\"+Inf\"")
		case math.IsInf(v, -1):
			return []byte("\"-Inf\"")
		}
		if bc == isFloat32 {
			ba = strconv.AppendFloat(ba, v, 'f', -1, 32)
		} else if bc == isFloat64 {
			ba = strconv.AppendFloat(ba, v, 'f', -1, 64)
		} else {
			panic(fmt.Errorf("Invalid Float precision from decodeFloat: %d", bc))
		}
		return ba
	default:
		panic(fmt.Errorf("Invalid Additional Type: %d in decodeSimpleFloat", minor))
	}
}

func cbor2JsonOneObject(src *bufio.Reader, dst io.Writer) {
	pb, e := src.Peek(1)
	if e != nil {
		panic(e)
	}
	major := (pb[0] & maskOutAdditionalType)

	switch major {
	case majorTypeUnsignedInt:
		fallthrough
	case majorTypeNegativeInt:
		n := decodeInteger(src)
		dst.Write([]byte(strconv.Itoa(int(n))))

	case majorTypeByteString:
		s := decodeString(src, false)
		dst.Write(s)

	case majorTypeUtf8String:
		s := decodeUTF8String(src)
		dst.Write(s)

	case majorTypeArray:
		array2Json(src, dst)

	case majorTypeMap:
		map2Json(src, dst)

	case majorTypeTags:
		s := decodeTagData(src)
		dst.Write(s)

	case majorTypeSimpleAndFloat:
		s := decodeSimpleFloat(src)
		dst.Write(s)
	}
}

func moreBytesToRead(src *bufio.Reader) bool {
	_, e := src.ReadByte()
	if e == nil {
		src.UnreadByte()
		return true
	}
	return false
}

// Cbor2JsonManyObjects decodes all the CBOR Objects read from src
// reader. It keeps on decoding until reader returns EOF (error when reading).
// Decoded string is written to the dst. At the end of every CBOR Object
// newline is written to the output stream.
//
// Returns error (if any) that was encountered during decode.
// The child functions will generate a panic when error is encountered and
// this function will recover non-runtime Errors and return the reason as error.
func Cbor2JsonManyObjects(src io.Reader, dst io.Writer) (err error) {
	defer func() {
		if r := recover(); r != nil {
			if _, ok := r.(runtime.Error); ok {
				panic(r)
			}
			err = r.(error)
		}
	}()
	bufRdr := bufio.NewReader(src)
	for moreBytesToRead(bufRdr) {
		cbor2JsonOneObject(bufRdr, dst)
		dst.Write([]byte("\n"))
	}
	return nil
}

// Detect if the bytes to be printed is Binary or not.
func binaryFmt(p []byte) bool {
	if len(p) > 0 && p[0] > 0x7F {
		return true
	}
	return false
}

func getReader(str string) *bufio.Reader {
	return bufio.NewReader(strings.NewReader(str))
}

// DecodeIfBinaryToString converts a binary formatted log msg to a
// JSON formatted String Log message - suitable for printing to Console/Syslog.
func DecodeIfBinaryToString(in []byte) string {
	if binaryFmt(in) {
		var b bytes.Buffer
		Cbor2JsonManyObjects(strings.NewReader(string(in)), &b)
		return b.String()
	}
	return string(in)
}

// DecodeObjectToStr checks if the input is a binary format, if so,
// it will decode a single Object and return the decoded string.
func DecodeObjectToStr(in []byte) string {
	if binaryFmt(in) {
		var b bytes.Buffer
		cbor2JsonOneObject(getReader(string(in)), &b)
		return b.String()
	}
	return string(in)
}

// DecodeIfBinaryToBytes checks if the input is a binary format, if so,
// it will decode all Objects and return the decoded string as byte array.
func DecodeIfBinaryToBytes(in []byte) []byte {
	if binaryFmt(in) {
		var b bytes.Buffer
		Cbor2JsonManyObjects(bytes.NewReader(in), &b)
		return b.Bytes()
	}
	return in
}
