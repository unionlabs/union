package amino

import (
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"math/bits"
	"time"
)

//----------------------------------------
// Signed

func EncodeInt8(w io.Writer, i int8) (err error) {
	return EncodeVarint(w, int64(i))
}

func EncodeInt16(w io.Writer, i int16) (err error) {
	return EncodeVarint(w, int64(i))
}

func EncodeInt32(w io.Writer, i int32) (err error) {
	var buf [4]byte
	binary.LittleEndian.PutUint32(buf[:], uint32(i))
	_, err = w.Write(buf[:])
	return
}

func EncodeInt64(w io.Writer, i int64) (err error) {
	var buf [8]byte
	binary.LittleEndian.PutUint64(buf[:], uint64(i))
	_, err = w.Write(buf[:])
	return err
}

func EncodeVarint(w io.Writer, i int64) (err error) {
	var buf [10]byte
	n := binary.PutVarint(buf[:], i)
	_, err = w.Write(buf[0:n])
	return
}

func VarintSize(i int64) int {
	return UvarintSize(uint64((uint64(i) << 1) ^ uint64(i>>63)))
}

//----------------------------------------
// Unsigned

func EncodeByte(w io.Writer, b byte) (err error) {
	return EncodeUvarint(w, uint64(b))
}

func EncodeUint8(w io.Writer, u uint8) (err error) {
	return EncodeUvarint(w, uint64(u))
}

func EncodeUint16(w io.Writer, u uint16) (err error) {
	return EncodeUvarint(w, uint64(u))
}

func EncodeUint32(w io.Writer, u uint32) (err error) {
	var buf [4]byte
	binary.LittleEndian.PutUint32(buf[:], u)
	_, err = w.Write(buf[:])
	return
}

func EncodeUint64(w io.Writer, u uint64) (err error) {
	var buf [8]byte
	binary.LittleEndian.PutUint64(buf[:], u)
	_, err = w.Write(buf[:])
	return
}

// EncodeUvarint is used to encode golang's int, int32, int64 by default. unless specified differently by the
// `binary:"fixed32"`, `binary:"fixed64"`, or `binary:"zigzag32"` `binary:"zigzag64"` tags.
// It matches protobufs varint encoding.
func EncodeUvarint(w io.Writer, u uint64) (err error) {
	var buf [10]byte
	n := binary.PutUvarint(buf[:], u)
	_, err = w.Write(buf[0:n])
	return
}

func UvarintSize(u uint64) int {
	if u == 0 {
		return 1
	}
	return (bits.Len64(u) + 6) / 7
}

//----------------------------------------
// Other

func EncodeBool(w io.Writer, b bool) (err error) {
	if b {
		err = EncodeUint8(w, 1) // same as EncodeUvarint(w, 1).
	} else {
		err = EncodeUint8(w, 0) // same as EncodeUvarint(w, 0).
	}
	return
}

// NOTE: UNSAFE
func EncodeFloat32(w io.Writer, f float32) (err error) {
	return EncodeUint32(w, math.Float32bits(f))
}

// NOTE: UNSAFE
func EncodeFloat64(w io.Writer, f float64) (err error) {
	return EncodeUint64(w, math.Float64bits(f))
}

const (
	// seconds of 01-01-0001
	minSeconds int64 = -62135596800
	// seconds of 10000-01-01
	maxSeconds int64 = 253402300800

	// nanos have to be in interval: [0, 999999999]
	maxNanos = 999999999
)

type InvalidTimeErr string

func (e InvalidTimeErr) Error() string {
	return "invalid time: " + string(e)
}

// EncodeTime writes the number of seconds (int64) and nanoseconds (int32),
// with millisecond resolution since January 1, 1970 UTC to the Writer as an
// UInt64.
// Milliseconds are used to ease compatibility with Javascript,
// which does not support finer resolution.
func EncodeTime(w io.Writer, t time.Time) (err error) {
	s := t.Unix()
	// TODO: We are hand-encoding a struct until MarshalAmino/UnmarshalAmino is supported.
	// skip if default/zero value:
	if s != 0 {
		if s < minSeconds || s >= maxSeconds {
			return InvalidTimeErr(fmt.Sprintf("seconds have to be >= %d and < %d, got: %d",
				minSeconds, maxSeconds, s))
		}
		err = encodeFieldNumberAndTyp3(w, 1, Typ3_Varint)
		if err != nil {
			return
		}
		err = EncodeUvarint(w, uint64(s))
		if err != nil {
			return
		}
	}
	ns := int32(t.Nanosecond()) // this int64 -> int32 cast is safe (nanos are in [0, 999999999])
	// skip if default/zero value:
	if ns != 0 {
		// do not encode if nanos exceed allowed interval
		if ns < 0 || ns > maxNanos {
			// we could as well panic here:
			// time.Time.Nanosecond() guarantees nanos to be in [0, 999,999,999]
			return InvalidTimeErr(fmt.Sprintf("nanoseconds have to be >= 0 and <= %v, got: %d",
				maxNanos, s))
		}
		err = encodeFieldNumberAndTyp3(w, 2, Typ3_Varint)
		if err != nil {
			return
		}
		err = EncodeUvarint(w, uint64(ns))
		if err != nil {
			return
		}
	}

	return
}

func EncodeByteSlice(w io.Writer, bz []byte) (err error) {
	err = EncodeUvarint(w, uint64(len(bz)))
	if err != nil {
		return
	}
	_, err = w.Write(bz)
	return
}

func ByteSliceSize(bz []byte) int {
	return UvarintSize(uint64(len(bz))) + len(bz)
}

func EncodeString(w io.Writer, s string) (err error) {
	return EncodeByteSlice(w, []byte(s))
}
