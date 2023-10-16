// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package utils

import (
	"math"
	"math/big"
	"reflect"
)

type toBigIntInterface interface {
	ToBigIntRegular(res *big.Int) *big.Int
}

// FromInterface converts an interface to a big.Int element
//
// input must be primitive (uintXX, intXX, []byte, string) or implement
// BigInt(res *big.Int) (which is the case for gnark-crypto field elements)
//
// if the input is a string, it calls (big.Int).SetString(input, 0). In particular:
// The number prefix determines the actual base: A prefix of
// ”0b” or ”0B” selects base 2, ”0”, ”0o” or ”0O” selects base 8,
// and ”0x” or ”0X” selects base 16. Otherwise, the selected base is 10
// and no prefix is accepted.
//
// panics if the input is invalid
func FromInterface(input interface{}) big.Int {
	var r big.Int

	switch v := input.(type) {
	case big.Int:
		r.Set(&v)
	case *big.Int:
		r.Set(v)
	case uint8:
		r.SetUint64(uint64(v))
	case uint16:
		r.SetUint64(uint64(v))
	case uint32:
		r.SetUint64(uint64(v))
	case uint64:
		r.SetUint64(v)
	case uint:
		r.SetUint64(uint64(v))
	case int8:
		r.SetInt64(int64(v))
	case int16:
		r.SetInt64(int64(v))
	case int32:
		r.SetInt64(int64(v))
	case int64:
		r.SetInt64(v)
	case int:
		r.SetInt64(int64(v))
	case string:
		if _, ok := r.SetString(v, 0); !ok {
			panic("unable to set big.Int from string " + v)
		}
	case []byte:
		r.SetBytes(v)
	default:
		if v, ok := input.(toBigIntInterface); ok {
			v.ToBigIntRegular(&r)
			return r
		} else if reflect.ValueOf(input).Kind() == reflect.Pointer {
			vv := reflect.ValueOf(input)
			if vv.CanInterface() {
				if v, ok := vv.Interface().(toBigIntInterface); ok {
					v.ToBigIntRegular(&r)
					return r
				}
			}
		}
		panic(reflect.TypeOf(input).String() + " to big.Int not supported")
	}

	return r
}

func IntSliceSliceToUint64SliceSlice(in [][]int) [][]uint64 {
	res := make([][]uint64, len(in))
	for i := range in {
		res[i] = make([]uint64, len(in[i]))
		for j := range in[i] {
			if in[i][j] < 0 {
				panic("negative value in int slice")
			}
			res[i][j] = uint64(in[i][j])
		}
	}
	return res
}

func Uint64SliceSliceToIntSliceSlice(in [][]uint64) [][]int {
	res := make([][]int, len(in))
	for i := range in {
		res[i] = make([]int, len(in[i]))
		for j := range in[i] {
			if in[i][j] >= math.MaxInt {
				panic("too large")
			}
			res[i][j] = int(in[i][j])
		}
	}
	return res
}
