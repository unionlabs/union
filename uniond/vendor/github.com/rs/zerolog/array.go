package zerolog

import (
	"net"
	"sync"
	"time"
)

var arrayPool = &sync.Pool{
	New: func() interface{} {
		return &Array{
			buf: make([]byte, 0, 500),
		}
	},
}

// Array is used to prepopulate an array of items
// which can be re-used to add to log messages.
type Array struct {
	buf []byte
}

func putArray(a *Array) {
	// Proper usage of a sync.Pool requires each entry to have approximately
	// the same memory cost. To obtain this property when the stored type
	// contains a variably-sized buffer, we add a hard limit on the maximum buffer
	// to place back in the pool.
	//
	// See https://golang.org/issue/23199
	const maxSize = 1 << 16 // 64KiB
	if cap(a.buf) > maxSize {
		return
	}
	arrayPool.Put(a)
}

// Arr creates an array to be added to an Event or Context.
func Arr() *Array {
	a := arrayPool.Get().(*Array)
	a.buf = a.buf[:0]
	return a
}

// MarshalZerologArray method here is no-op - since data is
// already in the needed format.
func (*Array) MarshalZerologArray(*Array) {
}

func (a *Array) write(dst []byte) []byte {
	dst = enc.AppendArrayStart(dst)
	if len(a.buf) > 0 {
		dst = append(dst, a.buf...)
	}
	dst = enc.AppendArrayEnd(dst)
	putArray(a)
	return dst
}

// Object marshals an object that implement the LogObjectMarshaler
// interface and appends it to the array.
func (a *Array) Object(obj LogObjectMarshaler) *Array {
	e := Dict()
	obj.MarshalZerologObject(e)
	e.buf = enc.AppendEndMarker(e.buf)
	a.buf = append(enc.AppendArrayDelim(a.buf), e.buf...)
	putEvent(e)
	return a
}

// Str appends the val as a string to the array.
func (a *Array) Str(val string) *Array {
	a.buf = enc.AppendString(enc.AppendArrayDelim(a.buf), val)
	return a
}

// Bytes appends the val as a string to the array.
func (a *Array) Bytes(val []byte) *Array {
	a.buf = enc.AppendBytes(enc.AppendArrayDelim(a.buf), val)
	return a
}

// Hex appends the val as a hex string to the array.
func (a *Array) Hex(val []byte) *Array {
	a.buf = enc.AppendHex(enc.AppendArrayDelim(a.buf), val)
	return a
}

// RawJSON adds already encoded JSON to the array.
func (a *Array) RawJSON(val []byte) *Array {
	a.buf = appendJSON(enc.AppendArrayDelim(a.buf), val)
	return a
}

// Err serializes and appends the err to the array.
func (a *Array) Err(err error) *Array {
	switch m := ErrorMarshalFunc(err).(type) {
	case LogObjectMarshaler:
		e := newEvent(nil, 0)
		e.buf = e.buf[:0]
		e.appendObject(m)
		a.buf = append(enc.AppendArrayDelim(a.buf), e.buf...)
		putEvent(e)
	case error:
		if m == nil || isNilValue(m) {
			a.buf = enc.AppendNil(enc.AppendArrayDelim(a.buf))
		} else {
			a.buf = enc.AppendString(enc.AppendArrayDelim(a.buf), m.Error())
		}
	case string:
		a.buf = enc.AppendString(enc.AppendArrayDelim(a.buf), m)
	default:
		a.buf = enc.AppendInterface(enc.AppendArrayDelim(a.buf), m)
	}

	return a
}

// Bool appends the val as a bool to the array.
func (a *Array) Bool(b bool) *Array {
	a.buf = enc.AppendBool(enc.AppendArrayDelim(a.buf), b)
	return a
}

// Int appends i as a int to the array.
func (a *Array) Int(i int) *Array {
	a.buf = enc.AppendInt(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Int8 appends i as a int8 to the array.
func (a *Array) Int8(i int8) *Array {
	a.buf = enc.AppendInt8(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Int16 appends i as a int16 to the array.
func (a *Array) Int16(i int16) *Array {
	a.buf = enc.AppendInt16(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Int32 appends i as a int32 to the array.
func (a *Array) Int32(i int32) *Array {
	a.buf = enc.AppendInt32(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Int64 appends i as a int64 to the array.
func (a *Array) Int64(i int64) *Array {
	a.buf = enc.AppendInt64(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Uint appends i as a uint to the array.
func (a *Array) Uint(i uint) *Array {
	a.buf = enc.AppendUint(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Uint8 appends i as a uint8 to the array.
func (a *Array) Uint8(i uint8) *Array {
	a.buf = enc.AppendUint8(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Uint16 appends i as a uint16 to the array.
func (a *Array) Uint16(i uint16) *Array {
	a.buf = enc.AppendUint16(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Uint32 appends i as a uint32 to the array.
func (a *Array) Uint32(i uint32) *Array {
	a.buf = enc.AppendUint32(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Uint64 appends i as a uint64 to the array.
func (a *Array) Uint64(i uint64) *Array {
	a.buf = enc.AppendUint64(enc.AppendArrayDelim(a.buf), i)
	return a
}

// Float32 appends f as a float32 to the array.
func (a *Array) Float32(f float32) *Array {
	a.buf = enc.AppendFloat32(enc.AppendArrayDelim(a.buf), f)
	return a
}

// Float64 appends f as a float64 to the array.
func (a *Array) Float64(f float64) *Array {
	a.buf = enc.AppendFloat64(enc.AppendArrayDelim(a.buf), f)
	return a
}

// Time appends t formatted as string using zerolog.TimeFieldFormat.
func (a *Array) Time(t time.Time) *Array {
	a.buf = enc.AppendTime(enc.AppendArrayDelim(a.buf), t, TimeFieldFormat)
	return a
}

// Dur appends d to the array.
func (a *Array) Dur(d time.Duration) *Array {
	a.buf = enc.AppendDuration(enc.AppendArrayDelim(a.buf), d, DurationFieldUnit, DurationFieldInteger)
	return a
}

// Interface appends i marshaled using reflection.
func (a *Array) Interface(i interface{}) *Array {
	if obj, ok := i.(LogObjectMarshaler); ok {
		return a.Object(obj)
	}
	a.buf = enc.AppendInterface(enc.AppendArrayDelim(a.buf), i)
	return a
}

// IPAddr adds IPv4 or IPv6 address to the array
func (a *Array) IPAddr(ip net.IP) *Array {
	a.buf = enc.AppendIPAddr(enc.AppendArrayDelim(a.buf), ip)
	return a
}

// IPPrefix adds IPv4 or IPv6 Prefix (IP + mask) to the array
func (a *Array) IPPrefix(pfx net.IPNet) *Array {
	a.buf = enc.AppendIPPrefix(enc.AppendArrayDelim(a.buf), pfx)
	return a
}

// MACAddr adds a MAC (Ethernet) address to the array
func (a *Array) MACAddr(ha net.HardwareAddr) *Array {
	a.buf = enc.AppendMACAddr(enc.AppendArrayDelim(a.buf), ha)
	return a
}

// Dict adds the dict Event to the array
func (a *Array) Dict(dict *Event) *Array {
	dict.buf = enc.AppendEndMarker(dict.buf)
	a.buf = append(enc.AppendArrayDelim(a.buf), dict.buf...)
	return a
}
