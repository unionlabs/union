package msgpack_test

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"github.com/shamaton/msgpack/v2/internal/common"
	"io"
	"math"
	"math/rand"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/shamaton/msgpack/v2"
	"github.com/shamaton/msgpack/v2/def"
	"github.com/shamaton/msgpack/v2/ext"
	extTime "github.com/shamaton/msgpack/v2/time"
)

var now time.Time

func init() {
	n := time.Now()
	now = time.Unix(n.Unix(), int64(n.Nanosecond()))
}

func TestInt(t *testing.T) {
	{
		args := []encdecArg[int]{{
			n: "FixInt",
			v: -8,
			c: func(d []byte) bool {
				return def.NegativeFixintMin <= int8(d[0]) && int8(d[0]) <= def.NegativeFixintMax
			}}, {
			n: "Int8",
			v: -108,
			c: func(d []byte) bool {
				return d[0] == def.Int8
			}}, {
			n: "Int16",
			v: -30108,
			c: func(d []byte) bool {
				return d[0] == def.Int16
			}}, {
			n: "Int32",
			v: -1030108,
			c: func(d []byte) bool {
				return d[0] == def.Int32
			}},
		}
		encdec(t, args...)
	}
	{
		arg := encdecArg[int64]{
			n: "Int64",
			v: int64(math.MinInt64 + 12345),
			c: func(d []byte) bool {
				return d[0] == def.Int64
			},
		}
		encdec(t, arg)
	}

	// error
	{
		arg := encdecArg[uint8]{
			n: "ErrorDecToUint8",
			v: -8,
			c: func(d []byte) bool {
				return def.NegativeFixintMin <= int8(d[0]) && int8(d[0]) <= def.NegativeFixintMax
			},
			e: "value different",
		}
		encdec(t, arg)
	}
	{
		arg := encdecArg[int32]{
			n: "ErrorDecToInt32",
			v: int64(math.MinInt64 + 12345),
			c: func(d []byte) bool {
				return d[0] == def.Int64
			},
			e: "value different",
		}
		encdec(t, arg)
	}
}

func TestUint(t *testing.T) {
	{
		args := []encdecArg[uint]{{
			n: "FixUint",
			v: uint(8),
			c: func(d []byte) bool {
				return def.PositiveFixIntMin <= uint8(d[0]) && uint8(d[0]) <= def.PositiveFixIntMax
			}}, {
			n: "Uint8",
			v: uint(130),
			c: func(d []byte) bool {
				return d[0] == def.Uint8
			}}, {
			n: "Uint16",
			v: uint(30130),
			c: func(d []byte) bool {
				return d[0] == def.Uint16
			}}, {
			n: "Uint32",
			v: uint(1030130),
			c: func(d []byte) bool {
				return d[0] == def.Uint32
			}},
		}
		encdec(t, args...)
	}
	{
		arg := encdecArg[uint64]{
			n: "Uint64",
			v: uint64(math.MaxUint64 - 12345),
			c: func(d []byte) bool {
				return d[0] == def.Uint64
			},
		}
		encdec(t, arg)
	}
}
func TestFloat(t *testing.T) {
	t.Run("Float32", func(t *testing.T) {
		c := func(d []byte) bool {
			return d[0] == def.Float32
		}
		args := []encdecArg[float32]{{
			n: "0",
			v: float32(0),
			c: c,
		}, {
			n: "-1",
			v: float32(-1),
			c: c,
		}, {
			n: "SmallestNonzeroFloat",
			v: float32(math.SmallestNonzeroFloat32),
			c: c,
		}, {
			n: "MaxFloat",
			v: float32(math.MaxFloat32),
			c: c,
		}}
		encdec(t, args...)
	})

	t.Run("Float64", func(t *testing.T) {
		c := func(d []byte) bool {
			return d[0] == def.Float64
		}
		args := []encdecArg[float64]{{
			n: "0",
			v: float64(0),
			c: c,
		}, {
			n: "-1",
			v: float64(-1),
			c: c,
		}, {
			n: "SmallestNonzeroFloat",
			v: math.SmallestNonzeroFloat64,
			c: c,
		}, {
			n: "MaxFloat",
			v: math.MaxFloat64,
			c: c,
		}}
		encdec(t, args...)
	})

	t.Run("FloatToInt", func(t *testing.T) {
		args := []encdecArg[int]{
			{
				n: "FromFloat32",
				v: float32(2.345),
				vc: func(v int) error {
					if v != 2 {
						return fmt.Errorf("different value: %d", v)
					}
					return nil
				},
				skipEq: true,
			},
			{
				n: "FromFloat64",
				v: float64(6.789),
				vc: func(v int) error {
					if v != 6 {
						return fmt.Errorf("different value: %d", v)
					}
					return nil
				},
				skipEq: true,
			},
		}
		encdec(t, args...)
	})

	// error
	t.Run("Float32ToFloat64", func(t *testing.T) {
		arg := encdecArg[float64]{
			n: "NotEqual",
			v: float32(math.MaxFloat32),
			c: func(d []byte) bool {
				return d[0] == def.Float32
			},
			e: "value different",
		}
		encdec(t, arg)
	})
	t.Run("Float64ToFloat32", func(t *testing.T) {
		arg := encdecArg[float32]{
			n: "ErrorDecToFloat32",
			v: math.MaxFloat64,
			c: func(d []byte) bool {
				return d[0] == def.Float64
			},
			e: "invalid code cb decoding",
		}
		encdec(t, arg)
	})
	t.Run("Float64ToString", func(t *testing.T) {
		arg := encdecArg[string]{
			n: "ErrorDecToString",
			v: math.MaxFloat64,
			c: func(d []byte) bool {
				return d[0] == def.Float64
			},
			e: "invalid code cb decoding",
		}
		encdec(t, arg)
	})
}

func TestBool(t *testing.T) {
	t.Run("Bool", func(t *testing.T) {
		args := []encdecArg[bool]{
			{
				n: "True",
				v: true,
				c: func(d []byte) bool {
					return d[0] == def.True
				},
			},
			{
				n: "False",
				v: false,
				c: func(d []byte) bool {
					return d[0] == def.False
				},
			},
		}
		encdec(t, args...)
	})

	// error
	t.Run("BoolToUint8", func(t *testing.T) {
		arg := encdecArg[uint8]{
			n: "ErrorDecToUint8",
			v: true,
			c: func(d []byte) bool {
				return d[0] == def.True
			},
			e: "invalid code c3 decoding",
		}
		encdec(t, arg)
	})
}

func TestNil(t *testing.T) {
	t.Run("Nil", func(t *testing.T) {
		args := []encdecArg[*map[any]any]{
			{
				n: "Nil",
				v: nil,
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
				vc: func(v *map[any]any) error {
					if v != nil {
						return fmt.Errorf("not nil: %v", v)
					}
					return nil
				},
			},
		}
		encdec(t, args...)
	})
}

func TestString(t *testing.T) {
	// len 31
	const base = "abcdefghijklmnopqrstuvwxyz12345"

	t.Run("String", func(t *testing.T) {
		args := []encdecArg[string]{
			{
				n: "EmptyString",
				v: "",
				c: func(d []byte) bool {
					return def.FixStr <= d[0] && d[0] < def.FixStr+32
				},
			},
			{
				n: "FixStr",
				v: strings.Repeat(base, 1),
				c: func(d []byte) bool {
					return def.FixStr <= d[0] && d[0] < def.FixStr+32
				},
			},
			{
				n: "Str8",
				v: strings.Repeat(base, 8),
				c: func(d []byte) bool {
					return d[0] == def.Str8
				},
			},
			{
				n: "Str16",
				v: strings.Repeat(base, (math.MaxUint16/len(base))-1),
				c: func(d []byte) bool {
					return d[0] == def.Str16
				},
			},
			{
				n: "Str32",
				v: strings.Repeat(base, (math.MaxUint16/len(base))+1),
				c: func(d []byte) bool {
					return d[0] == def.Str32
				},
			},
		}
		encdec(t, args...)
	})

	// type different
	t.Run("StringToBytes", func(t *testing.T) {
		arg := encdecArg[[]byte]{
			n: "TypeDifferentButSameValue",
			v: strings.Repeat(base, 8),
			c: func(d []byte) bool {
				return d[0] == def.Str8
			},
			e: "value different",
		}
		encdec(t, arg)
	})
	t.Run("BytesToString", func(t *testing.T) {
		arg := encdecArg[string]{
			n: "TypeDifferentButSameValue",
			v: []byte(base),
			c: func(d []byte) bool {
				return d[0] == def.Bin8
			},
			e: "value different",
		}
		encdec(t, arg)
	})
}

func TestComplex(t *testing.T) {
	t.Run("Complex64", func(t *testing.T) {
		args := []encdecArg[complex64]{
			{
				n: "To64",
				v: complex64(complex(1, 2)),
				c: func(d []byte) bool {
					return d[0] == def.Fixext8 && int8(d[1]) == def.ComplexTypeCode()
				},
			},
			{
				n: "Nil",
				v: nil,
				e: "should not reach this line",
			},
		}
		encdec(t, args...)

		args2 := []encdecArg[complex128]{
			{
				n: "To128",
				v: complex64(complex(1, 2)),
				c: func(d []byte) bool {
					return d[0] == def.Fixext8 && int8(d[1]) == def.ComplexTypeCode()
				},
				vc: func(t complex128) error {
					if imag(t) == 0 || real(t) == 0 {
						return fmt.Errorf("somthing wrong %v", t)
					}
					return nil
				},
				skipEq: true,
			},
		}
		encdec(t, args2...)
	})

	t.Run("Complex128", func(t *testing.T) {
		args := []encdecArg[complex128]{
			{
				n: "To128",
				v: complex(math.MaxFloat64, math.SmallestNonzeroFloat64),
				c: func(d []byte) bool {
					return d[0] == def.Fixext16 && int8(d[1]) == def.ComplexTypeCode()
				},
			},
			{
				n: "Nil",
				v: nil,
				e: "should not reach this line",
			},
		}
		encdec(t, args...)

		args2 := []encdecArg[complex64]{
			{
				n: "To64",
				v: complex(math.MaxFloat64, math.SmallestNonzeroFloat64),
				c: func(d []byte) bool {
					return d[0] == def.Fixext16 && int8(d[1]) == def.ComplexTypeCode()
				},
				vc: func(t complex64) error {
					if imag(t) != 0 || real(t) == 0 {
						return fmt.Errorf("somthing wrong %v", t)
					}
					return nil
				},
				skipEq: true,
			},
		}
		encdec(t, args2...)
	})

	t.Run("ComplexTypeCode", func(t *testing.T) {
		b64, err := msgpack.Marshal(complex64(complex(3, 4)))
		NoError(t, err)

		b128, err := msgpack.Marshal(complex128(complex(5, 6)))
		NoError(t, err)

		// change complex type code
		msgpack.SetComplexTypeCode(int8(-99))

		data := []struct {
			name   string
			b      []byte
			errStr string
		}{
			{"From64", b64, "fixext8"},
			{"From128", b128, "fixext16"},
		}

		var r64 complex64
		var r128 complex128
		for _, d := range data {
			for _, u := range unmarshallers {
				t.Run(d.name+"To64"+u.name, func(t *testing.T) {
					ErrorContains(t, u.u(d.b, &r64), d.errStr)
				})
				t.Run(d.name+"To128"+u.name, func(t *testing.T) {
					ErrorContains(t, u.u(d.b, &r128), d.errStr)
				})
			}
		}
	})
}

func TestAny(t *testing.T) {
	f := func(v interface{}) error {
		b, err := msgpack.Marshal(v)
		if err != nil {
			return err
		}
		var r interface{}
		err = msgpack.Unmarshal(b, &r)
		if err != nil {
			return err
		}
		if fmt.Sprintf("%v", v) != fmt.Sprintf("%v", r) {
			return fmt.Errorf("different value %v, %v", v, r)
		}
		return err
	}

	a1 := make([]int, math.MaxUint16)
	a2 := make([]int, math.MaxUint16+1)
	m1 := map[string]int{}
	m2 := map[string]int{}

	for i := range a1 {
		a1[i] = i
		m1[fmt.Sprint(i)] = 1
	}
	for i := range a2 {
		a2[i] = i
		m2[fmt.Sprint(i)] = 1
	}

	vars := []interface{}{
		true, false,
		1, math.MaxUint8, math.MaxUint16, math.MaxUint32, math.MaxUint32 + 1,
		-1, math.MinInt8, math.MinInt16, math.MinInt32, math.MinInt32 - 1,
		math.MaxFloat32, math.MaxFloat64,
		"a", strings.Repeat("a", math.MaxUint8), strings.Repeat("a", math.MaxUint16), strings.Repeat("a", math.MaxUint16+1),
		[]byte(strings.Repeat("a", math.MaxUint8)),
		[]byte(strings.Repeat("a", math.MaxUint16)),
		[]byte(strings.Repeat("a", math.MaxUint16+1)),
		[]interface{}{1, "a", 1.23}, a1, a2,
		map[interface{}]interface{}{"1": 1, 1.23: "a"}, m1, m2,
		time.Unix(now.Unix(), int64(now.Nanosecond())),
	}

	for i, v := range vars {
		if err := f(v); err != nil {
			t.Error(i, err)
		}
	}

	t.Run("Any", func(t *testing.T) {
		args := []encdecArg[any]{
			{n: "true", v: any(true)},
			{n: "false", v: any(false)},
			{n: "1", v: any(1)},
			{n: "MaxUint8", v: any(math.MaxUint8)},
			{n: "MaxUint16", v: any(math.MaxUint16)},
			{n: "MaxUint32", v: any(math.MaxUint32)},
			{n: "MaxUint32+1", v: any(math.MaxUint32 + 1)},
			{n: "-1", v: any(-1)},
			{n: "MinInt8", v: any(math.MinInt8)},
			{n: "MinInt16", v: any(math.MinInt16)},
			{n: "MinInt32", v: any(math.MinInt32)},
			{n: "MinInt32-1", v: any(math.MinInt32 - 1)},
			{n: "MaxFloat32", v: any(math.MaxFloat32)},
			{n: "MaxFloat64", v: any(math.MaxFloat64)},
			{n: "Str1", v: any("a")},
			{n: "Str255", v: any(strings.Repeat("a", math.MaxUint8))},
			{n: "Str65535", v: any(strings.Repeat("a", math.MaxUint16))},
			{n: "Str65536", v: any(strings.Repeat("a", math.MaxUint16+1))},
			{n: "Bin255", v: any([]byte(strings.Repeat("a", math.MaxUint8)))},
			{n: "Bin65535", v: any([]byte(strings.Repeat("a", math.MaxUint16)))},
			{n: "Bin65536", v: any([]byte(strings.Repeat("a", math.MaxUint16+1)))},
			{n: "Slice3", v: any([]any{1, "a", 1.23})},
			{n: "Slice65535", v: any(a1)},
			{n: "Slice65536", v: any(a2)},
			{n: "Map3", v: any(map[any]any{"1": 1, 1.23: "a"})},
			{n: "Map65535", v: any(m1)},
			{n: "Map65536", v: any(m2)},
			{n: "Time", v: any(time.Unix(now.Unix(), int64(now.Nanosecond())))},
		}
		for i := range args {
			i := i
			args[i].skipEq = true
			args[i].vc = func(t any) error {
				if fmt.Sprintf("%v", args[i].v) != fmt.Sprintf("%v", t) {
					return fmt.Errorf("different value %v, %v", args[i].v, t)
				}
				return nil
			}
		}
		encdec(t, args...)
	})

	// error
	t.Run("AnyError", func(t *testing.T) {
		var r any
		err := msgpack.Unmarshal([]byte{def.Ext32}, &r)
		ErrorContains(t, err, "invalid code")
	})
}

func TestBin(t *testing.T) {
	makeByteSlice := func(len int) []byte {
		v := make([]byte, len)
		for i := range v {
			v[i] = byte(rand.Intn(0xff))
		}
		return v
	}

	t.Run("Slice", func(t *testing.T) {
		args := []encdecArg[[]byte]{
			{
				n: "Bin8",
				v: makeByteSlice(128),
				c: func(d []byte) bool {
					return d[0] == def.Bin8
				},
			},
			{
				n: "Bin16",
				v: makeByteSlice(31280),
				c: func(d []byte) bool {
					return d[0] == def.Bin16
				},
			},
			{
				n: "Bin32",
				v: makeByteSlice(1031280),
				c: func(d []byte) bool {
					return d[0] == def.Bin32
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("Array", func(t *testing.T) {
		var (
			a128     [128]byte
			a31280   [31280]byte
			a1031280 [1031280]byte
		)
		for i := range a128 {
			a128[i] = byte(rand.Intn(0xff))
		}
		for i := range a31280 {
			a31280[i] = byte(rand.Intn(0xff))
		}
		for i := range a1031280 {
			a1031280[i] = byte(rand.Intn(0xff))
		}
		args128 := []encdecArg[[128]byte]{
			{
				n: "Bin8",
				v: a128,
				c: func(d []byte) bool {
					return d[0] == def.Bin8
				},
			},
		}
		encdec(t, args128...)

		args31280 := []encdecArg[[31280]byte]{
			{
				n: "Bin16",
				v: a31280,
				c: func(d []byte) bool {
					return d[0] == def.Bin16
				},
			},
		}
		encdec(t, args31280...)

		args1031280 := []encdecArg[[1031280]byte]{
			{
				n: "Bin32",
				v: a1031280,
				c: func(d []byte) bool {
					return d[0] == def.Bin32
				},
			},
		}
		encdec(t, args1031280...)

		args := []encdecArg[[1]byte]{
			{
				n: "Nil",
				v: nil,
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
				e: "value different",
			},
		}
		encdec(t, args...)
	})

	t.Run("Error", func(t *testing.T) {
		args1 := []encdecArg[[1]byte]{
			{
				n: "Nil",
				v: nil,
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
				e: "value different",
			},
		}
		encdec(t, args1...)

		var a128 [128]byte
		for i := range a128 {
			a128[i] = byte(rand.Intn(0xff))
		}
		args2 := []encdecArg[[127]byte]{
			{
				n: "Len",
				v: a128,
				c: func(d []byte) bool {
					return d[0] == def.Bin8
				},
				e: "[127]uint8 len is 127, but msgpack has 128 elements",
			},
		}
		encdec(t, args2...)
	})
}

func TestArray(t *testing.T) {
	t.Run("IntSlice", func(t *testing.T) {
		args := []encdecArg[[]int]{
			{
				n: "Nil",
				v: ([]int)(nil),
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
			},
			{
				n: "FixArray",
				v: make([]int, 15),
				c: func(d []byte) bool {
					return def.FixArray <= d[0] && d[0] <= def.FixArray+0x0f
				},
			},
			{
				n: "Array16",
				v: make([]int, 30015),
				c: func(d []byte) bool {
					return d[0] == def.Array16
				},
			},
			{
				n: "Array32",
				v: make([]int, 1030015),
				c: func(d []byte) bool {
					return d[0] == def.Array32
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("FloatArray", func(t *testing.T) {
		var v [8]float32
		for i := range v {
			v[i] = float32(rand.Intn(0xff))
		}
		args := []encdecArg[[8]float32]{
			{
				n: "FixArray",
				v: v,
				c: func(d []byte) bool {
					return def.FixArray <= d[0] && d[0] <= def.FixArray+0x0f
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("StringArray", func(t *testing.T) {
		var v [31280]string
		for i := range v {
			v[i] = "a"
		}
		args := []encdecArg[[31280]string]{
			{
				n: "Array16",
				v: v,
				c: func(d []byte) bool {
					return d[0] == def.Array16
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("BoolArray", func(t *testing.T) {
		var v [1031280]bool
		for i := range v {
			v[i] = rand.Intn(0xff) > 0x7f
		}
		args := []encdecArg[[1031280]bool]{
			{
				n: "Array32",
				v: v,
				c: func(d []byte) bool {
					return d[0] == def.Array32
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("SliceToArray", func(t *testing.T) {
		args := []encdecArg[[1]int]{
			{
				n: "Int",
				v: ([]int)(nil),
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
				e: "value different",
			},
		}
		encdec(t, args...)
	})

	t.Run("StringToBytes", func(t *testing.T) {
		const v = "abcde"
		args := []encdecArg[[5]byte]{
			{
				n:      "Test",
				v:      v,
				skipEq: true,
				vc: func(t [5]byte) error {
					if v != string(t[:]) {
						return fmt.Errorf("value different %v, %v", v, string(t[:]))
					}
					return nil
				},
			},
		}
		encdec(t, args...)
	})
}

func TestFixedSlice(t *testing.T) {
	c := func(d []byte) bool {
		return def.FixArray <= d[0] && d[0] <= def.FixArray+0x0f
	}

	t.Run("IntSlice", func(t *testing.T) {
		args := []encdecArg[[]int]{
			{
				v: []int{-1, 1},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("UintSlice", func(t *testing.T) {
		args := []encdecArg[[]uint]{
			{
				v: []uint{0, 100},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Int8Slice", func(t *testing.T) {
		args := []encdecArg[[]int8]{
			{
				v: []int8{math.MinInt8, math.MaxInt8},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Int16Slice", func(t *testing.T) {
		args := []encdecArg[[]int16]{
			{
				v: []int16{math.MinInt16, math.MaxInt16},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Int32Slice", func(t *testing.T) {
		args := []encdecArg[[]int32]{
			{
				v: []int32{math.MinInt32, math.MaxInt32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Int64Slice", func(t *testing.T) {
		args := []encdecArg[[]int64]{
			{
				v: []int64{math.MinInt64, math.MaxInt64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Uint8Slice", func(t *testing.T) {
		args := []encdecArg[[]uint8]{
			{
				v: []uint8{0, math.MaxUint8},
				c: func(d []byte) bool {
					// byte array
					return def.Bin8 == d[0]
				},
			},
		}
		encdec(t, args...)
	})
	t.Run("Uint16Slice", func(t *testing.T) {
		args := []encdecArg[[]uint16]{
			{
				v: []uint16{0, math.MaxUint16},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Uint32Slice", func(t *testing.T) {
		args := []encdecArg[[]uint32]{
			{
				v: []uint32{0, math.MaxUint32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Uint64Slice", func(t *testing.T) {
		args := []encdecArg[[]uint64]{
			{
				v: []uint64{0, math.MaxUint64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Float32Slice", func(t *testing.T) {
		args := []encdecArg[[]float32]{
			{
				v: []float32{math.SmallestNonzeroFloat32, math.MaxFloat32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("Float64Slice", func(t *testing.T) {
		args := []encdecArg[[]float64]{
			{
				v: []float64{math.SmallestNonzeroFloat64, math.MaxFloat64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("StringSlice", func(t *testing.T) {
		args := []encdecArg[[]string]{
			{
				v: []string{"aaa", "bbb"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("BoolSlice", func(t *testing.T) {
		args := []encdecArg[[]bool]{
			{
				v: []bool{true, false},
				c: c,
			},
		}
		encdec(t, args...)
	})
}

func TestFixedMap(t *testing.T) {
	c := func(d []byte) bool {
		return def.FixMap <= d[0] && d[0] <= def.FixMap+0x0f
	}

	t.Run("MapStringInt", func(t *testing.T) {
		args := []encdecArg[map[string]int]{
			{
				v: map[string]int{"a": 1, "b": 2},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringUint", func(t *testing.T) {
		args := []encdecArg[map[string]uint]{
			{
				v: map[string]uint{"a": math.MaxUint32, "b": 0},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringString", func(t *testing.T) {
		args := []encdecArg[map[string]string]{
			{
				v: map[string]string{"a": "12345", "abcdefghijklmnopqrstuvwxyz": "abcdefghijklmnopqrstuvwxyz"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringFloat32", func(t *testing.T) {
		args := []encdecArg[map[string]float32]{
			{
				v: map[string]float32{"a": math.MaxFloat32, "b": math.SmallestNonzeroFloat32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringFloat64", func(t *testing.T) {
		args := []encdecArg[map[string]float64]{
			{
				v: map[string]float64{"a": math.MaxFloat64, "b": math.SmallestNonzeroFloat64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringBool", func(t *testing.T) {
		args := []encdecArg[map[string]bool]{
			{
				v: map[string]bool{"a": true, "b": false},
				c: c,
			},
		}
		encdec(t, args...)
	})

	t.Run("MapStringInt8", func(t *testing.T) {
		args := []encdecArg[map[string]int8]{
			{
				v: map[string]int8{"a": math.MinInt8, "b": math.MaxInt8},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringInt16", func(t *testing.T) {
		args := []encdecArg[map[string]int16]{
			{
				v: map[string]int16{"a": math.MaxInt16, "b": math.MinInt16},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringInt32", func(t *testing.T) {
		args := []encdecArg[map[string]int32]{
			{
				v: map[string]int32{"a": math.MaxInt32, "b": math.MinInt32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringInt64", func(t *testing.T) {
		args := []encdecArg[map[string]int64]{
			{
				v: map[string]int64{"a": math.MinInt64, "b": math.MaxInt64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringUint8", func(t *testing.T) {
		args := []encdecArg[map[string]uint8]{
			{
				v: map[string]uint8{"a": 0, "b": math.MaxUint8},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringUint16", func(t *testing.T) {
		args := []encdecArg[map[string]uint16]{
			{
				v: map[string]uint16{"a": 0, "b": math.MaxUint16},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringUint32", func(t *testing.T) {
		args := []encdecArg[map[string]uint32]{
			{
				v: map[string]uint32{"a": 0, "b": math.MaxUint32},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapStringUint64", func(t *testing.T) {
		args := []encdecArg[map[string]uint64]{
			{
				v: map[string]uint64{"a": 0, "b": math.MaxUint64},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapIntString", func(t *testing.T) {
		args := []encdecArg[map[int]string]{
			{
				v: map[int]string{0: "a", 1: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapIntBool", func(t *testing.T) {
		args := []encdecArg[map[int]bool]{
			{
				v: map[int]bool{1: true, 2: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUintString", func(t *testing.T) {
		args := []encdecArg[map[uint]string]{
			{
				v: map[uint]string{0: "a", 1: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUintBool", func(t *testing.T) {
		args := []encdecArg[map[uint]bool]{
			{
				v: map[uint]bool{0: true, 255: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapFloat32String", func(t *testing.T) {
		args := []encdecArg[map[float32]string]{
			{
				v: map[float32]string{math.MaxFloat32: "a", math.SmallestNonzeroFloat32: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapFloat32Bool", func(t *testing.T) {
		args := []encdecArg[map[float32]bool]{
			{
				v: map[float32]bool{math.SmallestNonzeroFloat32: true, math.MaxFloat32: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapFloat64String", func(t *testing.T) {
		args := []encdecArg[map[float64]string]{
			{
				v: map[float64]string{math.MaxFloat64: "a", math.SmallestNonzeroFloat64: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapFloat64Bool", func(t *testing.T) {
		args := []encdecArg[map[float64]bool]{
			{
				v: map[float64]bool{math.SmallestNonzeroFloat64: true, math.MaxFloat64: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt8String", func(t *testing.T) {
		args := []encdecArg[map[int8]string]{
			{
				v: map[int8]string{math.MinInt8: "a", math.MaxInt8: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt8Bool", func(t *testing.T) {
		args := []encdecArg[map[int8]bool]{
			{
				v: map[int8]bool{math.MinInt8: true, math.MaxInt8: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt16String", func(t *testing.T) {
		args := []encdecArg[map[int16]string]{
			{
				v: map[int16]string{math.MaxInt16: "a", math.MinInt16: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt16Bool", func(t *testing.T) {
		args := []encdecArg[map[int16]bool]{
			{
				v: map[int16]bool{math.MaxInt16: true, math.MinInt16: false},
				c: c,
			},
		}
		encdec(t, args...)
	})

	t.Run("MapInt32String", func(t *testing.T) {
		args := []encdecArg[map[int32]string]{
			{
				v: map[int32]string{math.MinInt32: "a", math.MaxInt32: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt32Bool", func(t *testing.T) {
		args := []encdecArg[map[int32]bool]{
			{
				v: map[int32]bool{math.MinInt32: true, math.MaxInt32: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt64String", func(t *testing.T) {
		args := []encdecArg[map[int64]string]{
			{
				v: map[int64]string{math.MaxInt64: "a", math.MinInt64: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapInt64Bool", func(t *testing.T) {
		args := []encdecArg[map[int64]bool]{
			{
				v: map[int64]bool{math.MaxInt64: true, math.MinInt64: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint8String", func(t *testing.T) {
		args := []encdecArg[map[uint8]string]{
			{
				v: map[uint8]string{0: "a", math.MaxUint8: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint8Bool", func(t *testing.T) {
		args := []encdecArg[map[uint8]bool]{
			{
				v: map[uint8]bool{0: true, math.MaxUint8: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint16String", func(t *testing.T) {
		args := []encdecArg[map[uint16]string]{
			{
				v: map[uint16]string{0: "a", math.MaxUint16: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint16Bool", func(t *testing.T) {
		args := []encdecArg[map[uint16]bool]{
			{
				v: map[uint16]bool{0: true, math.MaxUint16: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint32String", func(t *testing.T) {
		args := []encdecArg[map[uint32]string]{
			{
				v: map[uint32]string{0: "a", math.MaxUint32: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint32Bool", func(t *testing.T) {
		args := []encdecArg[map[uint32]bool]{
			{
				v: map[uint32]bool{0: true, math.MaxUint32: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint64String", func(t *testing.T) {
		args := []encdecArg[map[uint64]string]{
			{
				v: map[uint64]string{0: "a", math.MaxUint64: "b"},
				c: c,
			},
		}
		encdec(t, args...)
	})
	t.Run("MapUint64Bool", func(t *testing.T) {
		args := []encdecArg[map[uint64]bool]{
			{
				v: map[uint64]bool{0: true, math.MaxUint64: false},
				c: c,
			},
		}
		encdec(t, args...)
	})
}

func TestTime(t *testing.T) {
	t.Run("Time", func(t *testing.T) {
		args := []encdecArg[time.Time]{
			{
				n: "Fixext4",
				v: time.Unix(now.Unix(), 0),
				c: func(d []byte) bool {
					return d[0] == def.Fixext4
				},
			},
			{
				n: "Fixext8",
				v: time.Unix(now.Unix(), int64(now.Nanosecond())),
				c: func(d []byte) bool {
					return d[0] == def.Fixext8
				},
			},
		}
		encdec(t, args...)
	})

	t.Run("Error", func(t *testing.T) {
		now := time.Now().Unix()
		nowByte := make([]byte, 4)
		binary.BigEndian.PutUint32(nowByte, uint32(now))

		var r time.Time
		c := def.TimeStamp

		for _, u := range unmarshallers {
			t.Run(u.name+"64formats", func(t *testing.T) {
				nanoByte := make([]byte, 64)
				for i := range nanoByte[:30] {
					nanoByte[i] = 0xff
				}
				b := append([]byte{def.Fixext8, byte(c)}, nanoByte...)
				err := u.u(b, &r)
				ErrorContains(t, err, "timestamp 64 formats")
			})
			t.Run(u.name+"96formats", func(t *testing.T) {
				nanoByte := make([]byte, 96)
				for i := range nanoByte[:32] {
					nanoByte[i] = 0xff
				}
				b := append([]byte{def.Ext8, byte(12), byte(c)}, nanoByte...)
				err := u.u(b, &r)
				ErrorContains(t, err, "timestamp 96 formats")
			})
		}

		t.Run("ShouldNotReach", func(t *testing.T) {
			notReach := []byte{def.Fixext1}
			_, _, err := extTime.Decoder.AsValue(0, reflect.Bool, &notReach)
			ErrorContains(t, err, "should not reach this line")
		})

		t.Run("ShouldNotReachStream", func(t *testing.T) {
			_, err := extTime.StreamDecoder.ToValue(def.Fixext1, nil, reflect.Bool)
			ErrorContains(t, err, "should not reach this line")
		})
	})
}

func TestMap(t *testing.T) {
	mapIntInt := func(l int) map[int]int {
		m := make(map[int]int, l)
		for i := 0; i < l; i++ {
			m[i] = i + 1
		}
		return m
	}
	t.Run("MapCode", func(t *testing.T) {
		args := []encdecArg[map[int]int]{
			{
				n: "FixMap",
				v: map[int]int{1: 2, 3: 4, 5: 6, 7: 8, 9: 10},
				c: func(d []byte) bool {
					return def.FixMap <= d[0] && d[0] <= def.FixMap+0x0f
				},
			},
			{
				n: "Map16",
				v: mapIntInt(1000),
				c: func(d []byte) bool {
					return d[0] == def.Map16
				},
			},
			{
				n: "Map32",
				v: mapIntInt(math.MaxUint16 + 1),
				c: func(d []byte) bool {
					return d[0] == def.Map32
				},
			},
		}
		encdec(t, args...)
	})
	t.Run("DiffType", func(t *testing.T) {
		v := mapIntInt(100)
		args := []encdecArg[map[uint]uint]{
			{
				n: "IntIntToUintUint",
				v: v,
				c: func(d []byte) bool {
					return d[0] == def.Map16
				},
				skipEq: true,
				vc: func(r map[uint]uint) error {
					for k, vv := range v {
						if rv, ok := r[uint(k)]; !ok || rv != uint(vv) {
							return fmt.Errorf("value differentee")
						}
					}
					if len(v) != len(r) {
						return fmt.Errorf("value different. v:%d, r:%d", len(v), len(r))
					}
					return nil
				},
			},
		}
		encdec(t, args...)
	})

	// error
	t.Run("Error", func(t *testing.T) {
		v1 := make(map[string]int, 100)
		for i := 0; i < 100; i++ {
			v1[fmt.Sprintf("%03d", i)] = i
		}
		v2 := make(map[int]string, 100)
		for i := 0; i < 100; i++ {
			v2[i] = fmt.Sprint(i % 10)
		}
		args := []encdecArg[map[int]int]{
			{
				n: "InvalidKey",
				v: v1,
				c: func(d []byte) bool {
					return d[0] == def.Map16
				},
				e: "invalid code a3 decoding",
			},
			{
				n: "InvalidValue",
				v: v2,
				c: func(d []byte) bool {
					return d[0] == def.Map16
				},
				e: "invalid code a1 decoding",
			},
		}
		encdec(t, args...)
	})
}

func TestPointer(t *testing.T) {
	t.Run("Pointer", func(t *testing.T) {
		v := 250
		vv := &v
		var vvv *int
		args := []encdecArg[*int]{
			{
				n: "Int",
				v: vv,
				c: func(d []byte) bool {
					return d[0] == def.Uint8
				},
			},
			{
				n: "Nil",
				v: vvv,
				c: func(d []byte) bool {
					return d[0] == def.Nil
				},
			},
		}
		encdec(t, args...)
	})

	// error
	t.Run("ReceiverMustBePointer", func(t *testing.T) {
		for _, u := range unmarshallers {
			var r int
			t.Run(u.name, func(t *testing.T) {
				err := u.u([]byte{def.Nil}, r)
				ErrorContains(t, err, "holder must set pointer value. but got:")
			})
		}
	})
}

func TestUnsupported(t *testing.T) {
	b := []byte{0xc0}

	t.Run("Uintptr", func(t *testing.T) {
		for _, m := range marshallers {
			t.Run(m.name, func(t *testing.T) {
				var v uintptr
				_, err := m.m(v)
				ErrorContains(t, err, "type(uintptr) is unsupported")
			})
		}
		for _, u := range unmarshallers {
			t.Run(u.name, func(t *testing.T) {
				var r uintptr
				err := u.u(b, &r)
				ErrorContains(t, err, "type(uintptr) is unsupported")
			})
		}
	})
	t.Run("Chan", func(t *testing.T) {
		for _, m := range marshallers {
			t.Run(m.name, func(t *testing.T) {
				var v chan string
				_, err := m.m(v)
				ErrorContains(t, err, "type(chan) is unsupported")
			})
		}
		for _, u := range unmarshallers {
			t.Run(u.name, func(t *testing.T) {
				var r chan string
				err := u.u(b, &r)
				ErrorContains(t, err, "type(chan) is unsupported")
			})
		}
	})
	t.Run("Func", func(t *testing.T) {
		for _, m := range marshallers {
			t.Run(m.name, func(t *testing.T) {
				var v func()
				_, err := m.m(v)
				ErrorContains(t, err, "type(func) is unsupported")
			})
		}
		for _, u := range unmarshallers {
			t.Run(u.name, func(t *testing.T) {
				var r func()
				err := u.u(b, &r)
				ErrorContains(t, err, "type(func) is unsupported")
			})
		}
	})
	t.Run("Error", func(t *testing.T) {
		// error reflect kind is invalid. current version set nil (0xc0)
		for _, m := range marshallers {
			for _, u := range unmarshallers {
				t.Run(m.name+u.name, func(t *testing.T) {
					var v, r error
					bb, err := m.m(v)
					NoError(t, err)
					if bb[0] != def.Nil {
						t.Fatalf("code is different %d, %d", b[0], def.Nil)
					}

					err = u.u(b, &r)
					NoError(t, err)
				})
			}
		}
	})
	{
		// error reflect kind is invalid. current version set nil (0xc0)
		var v, r error
		bb, err := msgpack.Marshal(v)
		if err != nil {
			t.Error(err)
		}
		if bb[0] != def.Nil {
			t.Errorf("code is different %d, %d", bb[0], def.Nil)
		}
		err = msgpack.Unmarshal(b, &r)
		if err != nil {
			t.Error(err)
		}
		if r != nil {
			t.Error("error should be nil")
		}
	}
}

/////////////////////////////////////////////////////////////////

func TestStruct(t *testing.T) {
	t.Run("Code", func(t *testing.T) {
		testStructCode(t)
	})
	t.Run("Tag", func(t *testing.T) {
		testStructTag(t)
	})
	t.Run("Array", func(t *testing.T) {
		testStructArray(t)
	})
	t.Run("Embedded", func(t *testing.T) {
		testEmbedded(t)
	})
	t.Run("Jump", func(t *testing.T) {
		testStructJump(t)
	})

	t.Run("UseCase", func(t *testing.T) {
		testStructUseCase(t)
	})
}

func testEmbedded(t *testing.T) {
	type Emb struct {
		Int int
	}
	type A struct {
		Emb
	}
	v := A{Emb: Emb{Int: 2}}

	arg := encdecArg[A]{
		v: v,
		vc: func(t A) error {
			if v.Int != t.Int {
				return fmt.Errorf("value is different %v, %v", v, t)
			}
			return nil
		},
	}
	encdec(t, arg)
}

func testStructTag(t *testing.T) {
	type vSt struct {
		One int    `msgpack:"Three"`
		Two string `msgpack:"four"`
		Hfn bool   `msgpack:"-"`
	}
	type rSt struct {
		Three int
		Four  string `msgpack:"four"`
		Hfn   bool
	}
	v := vSt{One: 1, Two: "2", Hfn: true}

	msgpack.StructAsArray = false

	arg := encdecArg[rSt]{
		v: v,
		c: func(d []byte) bool {
			return d[0] == def.FixMap+0x02
		},
		skipEq: true,
		vc: func(r rSt) error {
			if v.One != r.Three || v.Two != r.Four || r.Hfn != false {
				return fmt.Errorf("error: %v, %v", v, r)
			}
			return nil
		},
	}
	encdec(t, arg)
}

func testStructArray(t *testing.T) {
	type vSt struct {
		One  int
		Two  string
		Ten  float32
		Skip float32
	}
	type rSt struct {
		Three int
		Four  string
		Tem   float32
	}
	v := vSt{One: 1, Two: "2", Ten: 1.234}

	msgpack.StructAsArray = true

	arg := encdecArg[rSt]{
		v: v,
		c: func(d []byte) bool {
			return d[0] == def.FixArray+0x04
		},
		skipEq: true,
		vc: func(r rSt) error {
			if v.One != r.Three || v.Two != r.Four || v.Ten != r.Tem {
				return fmt.Errorf("error: %v, %v", v, r)
			}
			return nil
		},
	}
	encdec(t, arg)
}

func testStructCode(t *testing.T) {
	type st1 struct {
		Int int
	}
	type st16 struct {
		I1  int
		I2  int
		I3  int
		I4  int
		I5  int
		I6  int
		I7  int
		I8  int
		I9  int
		I10 int
		I11 int
		I12 int
		I13 int
		I14 int
		I15 int
		I16 int
	}
	v1 := st1{Int: math.MinInt32}
	v16 := st16{I1: 1, I2: 2, I3: 3, I4: 4, I5: 5, I6: 6, I7: 7, I8: 8, I9: 9, I10: 10,
		I11: 11, I12: 12, I13: 13, I14: 14, I15: 15, I16: 16}

	t.Run("Map", func(t *testing.T) {
		t.Run("st1", func(t *testing.T) {
			arg := encdecArg[st1]{
				v: v1,
				c: func(d []byte) bool {
					return def.FixMap <= d[0] && d[0] <= def.FixMap+0x0f
				},
			}
			encdec(t, arg)
		})
		t.Run("st16", func(t *testing.T) {
			arg := encdecArg[st16]{
				v: v16,
				c: func(d []byte) bool {
					return d[0] == def.Map16
				},
			}
			encdec(t, arg)
		})
	})

	msgpack.StructAsArray = true
	defer func() {
		msgpack.StructAsArray = false
	}()

	t.Run("Array", func(t *testing.T) {
		t.Run("st1", func(t *testing.T) {
			arg := encdecArg[st1]{
				v: v1,
				c: func(d []byte) bool {
					return def.FixArray <= d[0] && d[0] <= def.FixArray+0x0f
				},
			}
			encdec(t, arg)
		})
		t.Run("st16", func(t *testing.T) {
			arg := encdecArg[st16]{
				v: v16,
				c: func(d []byte) bool {
					return d[0] == def.Array16
				},
			}
			encdec(t, arg)
		})
	})
}

func testStructUseCase(t *testing.T) {
	type child3 struct {
		Int int
	}
	type child2 struct {
		Int2Uint        map[int]uint
		Float2Bool      map[float32]bool
		Duration2Struct map[time.Duration]child3
	}
	type child struct {
		IntArray      []int
		UintArray     []uint
		FloatArray    []float32
		BoolArray     []bool
		StringArray   []string
		TimeArray     []time.Time
		DurationArray []time.Duration
		Child         child2
	}
	type st struct {
		Int8        int8
		Int16       int16
		Int32       int32
		Int64       int64
		Uint8       byte
		Uint16      uint16
		Uint32      uint32
		Uint64      uint64
		Float       float32
		Double      float64
		Bool        bool
		String      string
		Pointer     *int
		Nil         *int
		Time        time.Time
		Duration    time.Duration
		Child       child
		Child3Array []child3
	}
	p := rand.Int()
	v := &st{
		Int32:    -32,
		Int8:     -8,
		Int16:    -16,
		Int64:    -64,
		Uint32:   32,
		Uint8:    8,
		Uint16:   16,
		Uint64:   64,
		Float:    1.23,
		Double:   2.3456,
		Bool:     true,
		String:   "Parent",
		Pointer:  &p,
		Nil:      nil,
		Time:     now,
		Duration: 123 * time.Second,

		// child
		Child: child{
			IntArray:      []int{-1, -2, -3, -4, -5},
			UintArray:     []uint{1, 2, 3, 4, 5},
			FloatArray:    []float32{-1.2, -3.4, -5.6, -7.8},
			BoolArray:     []bool{true, true, false, false, true},
			StringArray:   []string{"str", "ing", "arr", "ay"},
			TimeArray:     []time.Time{now, now, now},
			DurationArray: []time.Duration{1 * time.Nanosecond, 2 * time.Nanosecond},

			// childchild
			Child: child2{
				Int2Uint:        map[int]uint{-1: 2, -3: 4},
				Float2Bool:      map[float32]bool{-1.1: true, -2.2: false},
				Duration2Struct: map[time.Duration]child3{1 * time.Hour: {Int: 1}, 2 * time.Hour: {Int: 2}},
			},
		},

		Child3Array: []child3{{Int: 100}, {Int: 1000000}, {Int: 100000000}},
	}

	msgpack.StructAsArray = false
	encdec(t, encdecArg[st]{n: "Map", v: v})

	msgpack.StructAsArray = true
	encdec(t, encdecArg[st]{n: "Array", v: v})

	msgpack.StructAsArray = false
}

func testStructJump(t *testing.T) {
	type v1 struct{ A interface{} }
	type r1 struct{ B interface{} }

	a1 := make([]int, math.MaxUint16)
	a2 := make([]int, math.MaxUint16+1)
	m1 := map[string]int{}
	m2 := map[string]int{}

	for i := range a1 {
		a1[i] = i
		m1[fmt.Sprint(i)] = 1
	}
	for i := range a2 {
		a2[i] = i
		m2[fmt.Sprint(i)] = 1
	}

	vs := []v1{
		{A: true},
		{A: 1}, {A: -1},
		{A: math.MaxUint8}, {A: math.MinInt8},
		{A: math.MaxUint16}, {A: math.MinInt16},
		{A: math.MaxUint32 + 1}, {A: math.MinInt32 - 1}, {A: math.MaxFloat64},
		{A: "a"},
		{A: strings.Repeat("b", math.MaxUint8)}, {A: []byte(strings.Repeat("c", math.MaxUint8))},
		{A: strings.Repeat("e", math.MaxUint16)}, {A: []byte(strings.Repeat("d", math.MaxUint16))},
		{A: strings.Repeat("f", math.MaxUint16+1)}, {A: []byte(strings.Repeat("g", math.MaxUint16+1))},
		{A: []int{1}}, {A: a1}, {A: a2},
		{A: map[string]int{"a": 1}}, {A: m1}, {A: m2},
		{A: time.Unix(now.Unix(), int64(now.Nanosecond()))},
	}
	msgpack.StructAsArray = false

	for _, v := range vs {
		n := fmt.Sprint(v)
		if len(n) > 32 {
			n = n[:32]
		}
		arg := encdecArg[r1]{
			n:      n,
			v:      v,
			skipEq: true,
			vc: func(r r1) error {
				if fmt.Sprint(v.A) == fmt.Sprint(r.B) {
					return fmt.Errorf("value equal %v, %v", v, r)
				}
				return nil
			},
		}
		encdec(t, arg)
	}
}

/////////////////////////////////////////////////////////////

func TestExt(t *testing.T) {
	err := msgpack.AddExtCoder(encoder, decoder)
	NoError(t, err)
	err = msgpack.AddExtStreamCoder(streamEncoder, streamDecoder)
	NoError(t, err)

	{
		v := ExtInt{
			Int8:        math.MinInt8,
			Int16:       math.MinInt16,
			Int32:       math.MinInt32,
			Int64:       math.MinInt32 - 1,
			Uint8:       math.MaxUint8,
			Uint16:      math.MaxUint16,
			Uint32:      math.MaxUint32,
			Uint64:      math.MaxUint32 + 1,
			Byte2Int:    rand.Intn(math.MaxUint16),
			Byte4Int:    rand.Intn(math.MaxInt32) - rand.Intn(math.MaxInt32-1),
			Byte4Uint32: math.MaxUint32 - 1,
			Bytes:       []byte{1, 2, 3}, // 3
		}
		encdec(t, encdecArg[ExtInt]{n: "AddCoder", v: v})
	}

	err = msgpack.RemoveExtCoder(encoder, decoder)
	NoError(t, err)
	err = msgpack.RemoveExtStreamCoder(streamEncoder, streamDecoder)
	NoError(t, err)

	{
		v := ExtInt{
			Int8:        math.MinInt8,
			Int16:       math.MinInt16,
			Int32:       math.MinInt32,
			Int64:       math.MinInt32 - 1,
			Uint8:       math.MaxUint8,
			Uint16:      math.MaxUint16,
			Uint32:      math.MaxUint32,
			Uint64:      math.MaxUint32 + 1,
			Byte2Int:    rand.Intn(math.MaxUint16),
			Byte4Int:    rand.Intn(math.MaxInt32) - rand.Intn(math.MaxInt32-1),
			Byte4Uint32: math.MaxUint32 - 1,
			Bytes:       []byte{4, 5, 6}, // 3
		}
		encdec(t, encdecArg[ExtInt]{n: "RemoveCoder", v: v})
	}

	t.Run("ErrorExtCoder", func(t *testing.T) {
		err = msgpack.AddExtCoder(&testExt2Encoder{}, &testExt2Decoder{})
		ErrorContains(t, err, "code different")

		err = msgpack.RemoveExtCoder(&testExt2Encoder{}, &testExt2Decoder{})
		ErrorContains(t, err, "code different")

		err = msgpack.AddExtStreamCoder(&testExt2StreamEncoder{}, &testExt2StreamDecoder{})
		ErrorContains(t, err, "code different")

		err = msgpack.RemoveExtStreamCoder(&testExt2StreamEncoder{}, &testExt2StreamDecoder{})
		ErrorContains(t, err, "code different")
	})
}

type ExtStruct struct {
	Int8        int8
	Int16       int16
	Int32       int32
	Int64       int64
	Uint8       uint8
	Uint16      uint16
	Uint32      uint32
	Uint64      uint64
	Byte2Int    int
	Byte4Int    int
	Byte4Uint32 uint32
	Bytes       []byte // 3
}
type ExtInt ExtStruct

var decoder = new(testDecoder)

type testDecoder struct {
	ext.DecoderCommon
}

var _ ext.Decoder = (*testDecoder)(nil)

var extIntCode = int8(-2)

func (td *testDecoder) Code() int8 {
	return extIntCode
}

func (td *testDecoder) IsType(offset int, d *[]byte) bool {
	code, offset := td.ReadSize1(offset, d)
	if code == def.Ext8 {
		c, offset := td.ReadSize1(offset, d)
		t, _ := td.ReadSize1(offset, d)
		return c == 15+15+10+3 && int8(t) == td.Code()
	}
	return false
}

func (td *testDecoder) AsValue(offset int, k reflect.Kind, d *[]byte) (interface{}, int, error) {
	code, offset := td.ReadSize1(offset, d)

	switch code {
	case def.Ext8:
		// size
		_, offset = td.ReadSize1(offset, d)
		// code
		_, offset = td.ReadSize1(offset, d)
		i8, offset := td.ReadSize1(offset, d)
		i16, offset := td.ReadSize2(offset, d)
		i32, offset := td.ReadSize4(offset, d)
		i64, offset := td.ReadSize8(offset, d)
		u8, offset := td.ReadSize1(offset, d)
		u16, offset := td.ReadSize2(offset, d)
		u32, offset := td.ReadSize4(offset, d)
		u64, offset := td.ReadSize8(offset, d)
		b16, offset := td.ReadSize2(offset, d)
		b32, offset := td.ReadSize4(offset, d)
		bu32, offset := td.ReadSize4(offset, d)
		bs, offset := td.ReadSizeN(offset, 3, d)
		return ExtInt{
			Int8:        int8(i8),
			Int16:       int16(binary.BigEndian.Uint16(i16)),
			Int32:       int32(binary.BigEndian.Uint32(i32)),
			Int64:       int64(binary.BigEndian.Uint64(i64)),
			Uint8:       u8,
			Uint16:      binary.BigEndian.Uint16(u16),
			Uint32:      binary.BigEndian.Uint32(u32),
			Uint64:      binary.BigEndian.Uint64(u64),
			Byte2Int:    int(binary.BigEndian.Uint16(b16)),
			Byte4Int:    int(int32(binary.BigEndian.Uint32(b32))),
			Byte4Uint32: binary.BigEndian.Uint32(bu32),
			Bytes:       bs,
		}, offset, nil
	}

	return ExtInt{}, 0, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}

var streamDecoder = new(testStreamDecoder)

type testStreamDecoder struct {
	ext.DecoderStreamCommon
}

var _ ext.StreamDecoder = (*testStreamDecoder)(nil)

func (td *testStreamDecoder) Code() int8 {
	return extIntCode
}

func (td *testStreamDecoder) IsType(code byte, innerType int8, dataLength int) bool {
	if code == def.Ext8 {
		return dataLength == 15+15+10+3 && innerType == td.Code()
	}
	return false
}

func (td *testStreamDecoder) ToValue(code byte, data []byte, k reflect.Kind) (any, error) {
	switch code {
	case def.Ext8:
		i8 := data[:1]
		i16 := data[1:3]
		i32 := data[3:7]
		i64 := data[7:15]
		u8 := data[15:16]
		u16 := data[16:18]
		u32 := data[18:22]
		u64 := data[22:30]
		b16 := data[30:32]
		b32 := data[32:36]
		bu32 := data[36:40]
		bs := data[40:43]
		return ExtInt{
			Int8:        int8(i8[0]),
			Int16:       int16(binary.BigEndian.Uint16(i16)),
			Int32:       int32(binary.BigEndian.Uint32(i32)),
			Int64:       int64(binary.BigEndian.Uint64(i64)),
			Uint8:       u8[0],
			Uint16:      binary.BigEndian.Uint16(u16),
			Uint32:      binary.BigEndian.Uint32(u32),
			Uint64:      binary.BigEndian.Uint64(u64),
			Byte2Int:    int(binary.BigEndian.Uint16(b16)),
			Byte4Int:    int(int32(binary.BigEndian.Uint32(b32))),
			Byte4Uint32: binary.BigEndian.Uint32(bu32),
			Bytes:       bs,
		}, nil
	}

	return ExtInt{}, fmt.Errorf("should not reach this line!! code %x decoding %v", code, k)
}

var encoder = new(testEncoder)

type testEncoder struct {
	ext.EncoderCommon
}

var _ ext.Encoder = (*testEncoder)(nil)

func (s *testEncoder) Code() int8 {
	return extIntCode
}

func (s *testEncoder) Type() reflect.Type {
	return reflect.TypeOf(ExtInt{})
}

func (s *testEncoder) CalcByteSize(value reflect.Value) (int, error) {
	t := value.Interface().(ExtInt)
	return def.Byte1 + def.Byte1 + 15 + 15 + 10 + len(t.Bytes), nil
}

func (s *testEncoder) WriteToBytes(value reflect.Value, offset int, bytes *[]byte) int {
	t := value.Interface().(ExtInt)
	offset = s.SetByte1Int(def.Ext8, offset, bytes)
	offset = s.SetByte1Int(15+15+10+len(t.Bytes), offset, bytes)
	offset = s.SetByte1Int(int(s.Code()), offset, bytes)

	offset = s.SetByte1Int64(int64(t.Int8), offset, bytes)
	offset = s.SetByte2Int64(int64(t.Int16), offset, bytes)
	offset = s.SetByte4Int64(int64(t.Int32), offset, bytes)
	offset = s.SetByte8Int64(t.Int64, offset, bytes)

	offset = s.SetByte1Uint64(uint64(t.Uint8), offset, bytes)
	offset = s.SetByte2Uint64(uint64(t.Uint16), offset, bytes)
	offset = s.SetByte4Uint64(uint64(t.Uint32), offset, bytes)
	offset = s.SetByte8Uint64(t.Uint64, offset, bytes)

	offset = s.SetByte2Int(t.Byte2Int, offset, bytes)
	offset = s.SetByte4Int(t.Byte4Int, offset, bytes)

	offset = s.SetByte4Uint32(t.Byte4Uint32, offset, bytes)
	offset = s.SetBytes(t.Bytes, offset, bytes)
	return offset
}

var streamEncoder = new(testStreamEncoder)

type testStreamEncoder struct {
	ext.StreamEncoderCommon
}

var _ ext.StreamEncoder = (*testStreamEncoder)(nil)

func (s *testStreamEncoder) Code() int8 {
	return extIntCode
}

func (s *testStreamEncoder) Type() reflect.Type {
	return reflect.TypeOf(ExtInt{})
}

func (s *testStreamEncoder) Write(w io.Writer, value reflect.Value, buf *common.Buffer) error {
	t := value.Interface().(ExtInt)
	if err := s.WriteByte1Int(w, def.Ext8, buf); err != nil {
		return err
	}
	if err := s.WriteByte1Int(w, 15+15+10+len(t.Bytes), buf); err != nil {
		return err
	}
	if err := s.WriteByte1Int(w, int(s.Code()), buf); err != nil {
		return err
	}

	if err := s.WriteByte1Int64(w, int64(t.Int8), buf); err != nil {
		return err
	}
	if err := s.WriteByte2Int64(w, int64(t.Int16), buf); err != nil {
		return err
	}
	if err := s.WriteByte4Int64(w, int64(t.Int32), buf); err != nil {
		return err
	}
	if err := s.WriteByte8Int64(w, t.Int64, buf); err != nil {
		return err
	}

	if err := s.WriteByte1Uint64(w, uint64(t.Uint8), buf); err != nil {
		return err
	}
	if err := s.WriteByte2Uint64(w, uint64(t.Uint16), buf); err != nil {
		return err
	}
	if err := s.WriteByte4Uint64(w, uint64(t.Uint32), buf); err != nil {
		return err
	}
	if err := s.WriteByte8Uint64(w, t.Uint64, buf); err != nil {
		return err
	}

	if err := s.WriteByte2Int(w, t.Byte2Int, buf); err != nil {
		return err
	}
	if err := s.WriteByte4Int(w, t.Byte4Int, buf); err != nil {
		return err
	}

	if err := s.WriteByte4Uint32(w, t.Byte4Uint32, buf); err != nil {
		return err
	}
	if err := s.WriteBytes(w, t.Bytes, buf); err != nil {
		return err
	}
	return nil
}

/////////////////////////////////////////////////////////

type Ext2Struct struct {
	V int
}
type Ext2Int Ext2Struct

const (
	testExt2DecoderCode = 3
	testExt2EncoderCode = -3
)

type testExt2Decoder struct{}

var _ ext.Decoder = (*testExt2Decoder)(nil)

func (td *testExt2Decoder) Code() int8 {
	return testExt2DecoderCode
}

func (td *testExt2Decoder) IsType(_ int, _ *[]byte) bool {
	return false
}

func (td *testExt2Decoder) AsValue(_ int, k reflect.Kind, _ *[]byte) (interface{}, int, error) {
	return Ext2Int{}, 0, fmt.Errorf("should not reach this line!! code %x decoding %v", td.Code(), k)
}

type testExt2StreamDecoder struct{}

var _ ext.StreamDecoder = (*testExt2StreamDecoder)(nil)

func (td *testExt2StreamDecoder) Code() int8 {
	return testExt2DecoderCode
}

func (td *testExt2StreamDecoder) IsType(_ byte, _ int8, _ int) bool {
	return false
}

func (td *testExt2StreamDecoder) ToValue(_ byte, _ []byte, k reflect.Kind) (any, error) {
	return Ext2Int{}, fmt.Errorf("should not reach this line!! code %x decoding %v", td.Code(), k)
}

type testExt2Encoder struct {
	ext.EncoderCommon
}

var _ ext.Encoder = (*testExt2Encoder)(nil)

func (s *testExt2Encoder) Code() int8 {
	return testExt2EncoderCode
}

func (s *testExt2Encoder) Type() reflect.Type {
	return reflect.TypeOf(ExtInt{})
}

func (s *testExt2Encoder) CalcByteSize(_ reflect.Value) (int, error) {
	return 0, nil
}

func (s *testExt2Encoder) WriteToBytes(_ reflect.Value, offset int, _ *[]byte) int {
	return offset
}

type testExt2StreamEncoder struct{}

var _ ext.StreamEncoder = (*testExt2StreamEncoder)(nil)

func (s *testExt2StreamEncoder) Code() int8 {
	return testExt2EncoderCode
}

func (s *testExt2StreamEncoder) Type() reflect.Type {
	return reflect.TypeOf(ExtInt{})
}

func (s *testExt2StreamEncoder) Write(_ io.Writer, _ reflect.Value, _ *common.Buffer) error {
	return fmt.Errorf("should not reach this line!! code %x", s.Code())
}

/////////////////////////////////////////////////////////

type (
	checker      func(data []byte) bool
	marshaller   func(v any) ([]byte, error)
	unmarshaller func(data []byte, v any) error
)

var marshallers = []struct {
	name string
	m    marshaller
}{
	{"Marshal", msgpack.Marshal},
	{"MarshalWrite", func(v any) ([]byte, error) {
		buf := bytes.Buffer{}
		err := msgpack.MarshalWrite(&buf, v)
		return buf.Bytes(), err
	}},
}
var unmarshallers = []struct {
	name string
	u    unmarshaller
}{
	{"Unmarshal", msgpack.Unmarshal},
	{"UnmarshalRead", func(data []byte, v any) error {
		return msgpack.UnmarshalRead(bytes.NewReader(data), v)
	}},
}

type encdecArg[T any] struct {
	n      string
	v      any
	r      T
	c      checker
	skipEq bool
	vc     func(t T) error
	e      string
}

func encdec[T any](t *testing.T, args ...encdecArg[T]) {
	t.Helper()

	for _, arg := range args {
		t.Run(arg.n, func(t *testing.T) {
			for _, m := range marshallers {
				for _, u := range unmarshallers {
					t.Run(m.name+"-"+u.name, func(t *testing.T) {
						var e error
						defer func() {
							if e != nil {
								if len(arg.e) < 1 {
									t.Fatalf("unexpected error: %v", e)
								}
								if !strings.Contains(e.Error(), arg.e) {
									t.Fatalf("error does not contain '%s'. err: %v", arg.e, e)
								}
							} else {
								if len(arg.e) > 0 {
									t.Fatalf("error should occur, but nil. expected: %s", arg.e)
								}
							}
						}()

						d, err := m.m(arg.v)
						if err != nil {
							e = err
							return
						}
						if arg.c != nil && !arg.c(d) {
							e = fmt.Errorf("different %s", hex.Dump(d))
							return
						}
						if err = u.u(d, &arg.r); err != nil {
							e = err
							return
						}
						if !arg.skipEq {
							if err = equalCheck(arg.v, arg.r); err != nil {
								e = err
								return
							}
						}
						if arg.vc != nil {
							if err = arg.vc(arg.r); err != nil {
								e = err
								return
							}
						}
					})
				}
			}
		})
	}
}

// for check value
func getValue(v interface{}) interface{} {
	rv := reflect.ValueOf(v)
	if rv.Kind() == reflect.Ptr {
		rv = rv.Elem()
	}
	if rv.Kind() == reflect.Ptr {
		rv = rv.Elem()
	}
	if rv.Kind() == reflect.Invalid {
		return nil
	}
	return rv.Interface()
}

func equalCheck(in, out interface{}) error {
	i := getValue(in)
	o := getValue(out)
	if !reflect.DeepEqual(i, o) {
		return errors.New(fmt.Sprint("value different \n[in]:", i, " \n[out]:", o))
	}
	return nil
}

func NoError(t *testing.T, err error) {
	t.Helper()
	if err != nil {
		t.Fatal(err)
	}
}

func ErrorContains(t *testing.T, err error, errStr string) {
	if err == nil {
		t.Fatal("error should occur")
	}
	if !strings.Contains(err.Error(), errStr) {
		t.Fatalf("error does not contain '%s'. err: %v", errStr, err)
	}
}
