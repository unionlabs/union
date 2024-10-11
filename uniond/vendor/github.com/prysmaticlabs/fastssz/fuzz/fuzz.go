package fuzz

import (
	"fmt"
	"math/rand"
	"reflect"
	"strconv"
	"strings"
	"time"
)

// Based on https://github.com/google/gofuzz

// Fuzzer knows how to fill any object with random fields.
type Fuzzer struct {
	r         *rand.Rand
	failRatio float64
}

func randomInt(min, max int) int {
	return min + rand.Intn(max-min)
}

// New returns a new Fuzzer.
func New() *Fuzzer {
	return NewWithSeed(time.Now().UnixNano())
}

// NewWithSeed returns a new Fuzzer with a specific seed.
func NewWithSeed(seed int64) *Fuzzer {
	f := &Fuzzer{
		r: rand.New(rand.NewSource(seed)),
	}
	return f
}

// SetFailureRatio sets the failure ratio for the fuzzer
func (f *Fuzzer) SetFailureRatio(failRatio float64) {
	f.failRatio = failRatio
}

// Fuzz recursively fills all of obj's fields with something random
func (f *Fuzzer) Fuzz(obj interface{}) bool {
	v := reflect.ValueOf(obj)
	if v.Kind() != reflect.Ptr {
		panic("needed ptr!")
	}
	v = v.Elem()
	fc := &fuzzerContext{fuzzer: f}
	fc.doFuzz(v, "")
	return fc.failed
}

type fuzzerContext struct {
	fuzzer *Fuzzer
	failed bool
}

func convertNum(str string) int {
	num, err := strconv.Atoi(str)
	if err != nil {
		panic(err)
	}
	return num
}

func (f *Fuzzer) getShoudlFail() bool {
	return f.r.Float64() < f.failRatio
}

func (fc *fuzzerContext) getRandomNum(maxStr string, isMax bool) int {
	max := convertNum(maxStr)
	if max > 5000 {
		// hard cap for long values in Beacon state
		return 1000
	}
	if !fc.failed {
		if fc.fuzzer.getShoudlFail() {
			fc.failed = true
			if isMax {
				nn := randomInt(max+1, max+10)
				return nn
			}
			// fixed size, return lower and higher values and avoid the
			// input value since we already set ourselves as failed
			num := randomInt(max-10, max+10)
			if num == max {
				return num + 1
			}
			if num < 0 {
				num = 1
			}
			return num
		}
	}
	return max
}

func (fc *fuzzerContext) genElementCount(tag reflect.StructTag) (reflect.StructTag, int) {
	if size := tag.Get("ssz-size"); size != "" {
		indx := strings.Index(size, ",")
		if indx == -1 {
			// just one size
			return "", fc.getRandomNum(size, false)
		}

		var num int
		if size[:indx] == "?" {
			// search for ssz-max tag
			max := tag.Get("ssz-max")
			if max == "" {
				panic("BUG: Max tag expected after ?")
			}
			num = fc.getRandomNum(max, true)
		} else {
			// its a number
			num = fc.getRandomNum(size[:indx], false)
		}

		// a,b
		return reflect.StructTag("ssz-size:\"" + size[indx+1:] + "\""), num
	}
	if max := tag.Get("ssz-max"); max != "" {
		return "", fc.getRandomNum(max, true)
	}
	if typ := tag.Get("ssz"); typ != "" {
		if typ == "bitlist" {
			return "", randomInt(1, 10)
		}
	}
	panic("BUG: Tags not expected")
}

func (fc *fuzzerContext) doFuzz(v reflect.Value, tag reflect.StructTag) {
	if !v.CanSet() {
		return
	}

	switch v.Kind() {
	case reflect.Uint, reflect.Uint8, reflect.Uint16, reflect.Uint32, reflect.Uint64:
		fuzzUint(v, fc.fuzzer.r)

	case reflect.Bool:
		v.SetBool(randBool(fc.fuzzer.r))

	case reflect.String:
		v.SetString(randString(fc.fuzzer.r.Int(), letters))

	case reflect.Ptr:
		v.Set(reflect.New(v.Type().Elem()))
		fc.doFuzz(v.Elem(), "")
		return

	case reflect.Slice:
		subTag, n := fc.genElementCount(tag)
		v.Set(reflect.MakeSlice(v.Type(), n, n))
		for i := 0; i < n; i++ {
			fc.doFuzz(v.Index(i), subTag)
		}

	case reflect.Struct:
		typ := v.Type()
		for i := 0; i < v.NumField(); i++ {
			// fuzz nil values if the field of the struct is
			// another struct
			if isPtrToStruct(v.Field(i)) {
				if fc.addNil(v.Field(i)) {
					continue
				}
			}
			fc.doFuzz(v.Field(i), typ.Field(i).Tag)
		}

	case reflect.Array:
		n := v.Len()
		for i := 0; i < n; i++ {
			fc.doFuzz(v.Index(i), tag)
		}

	default:
		panic(fmt.Sprintf("Can't handle %#v", v.Interface()))
	}
}

func (fc *fuzzerContext) addNil(v reflect.Value) bool {
	if !fc.failed {
		if fc.fuzzer.getShoudlFail() {
			// set to nil, we dont fail because marshal fills empty values
			v.Set(reflect.Zero(v.Type()))
			return true
		}
	}
	return false
}

func isPtrToStruct(v reflect.Value) bool {
	if v.Kind() == reflect.Ptr {
		if v.Type().Elem().Kind() == reflect.Struct {
			return true
		}
	}
	return false
}

func fuzzInt(v reflect.Value, r *rand.Rand) {
	v.SetInt(int64(randUint64(r)))
}

func fuzzUint(v reflect.Value, r *rand.Rand) {
	v.SetUint(randUint64(r))
}

func randBool(r *rand.Rand) bool {
	if r.Int()&1 == 1 {
		return true
	}
	return false
}

const letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

func randString(n int, dict string) string {
	b := make([]byte, n)
	for i := range b {
		b[i] = dict[rand.Intn(len(dict))]
	}
	return string(b)
}

func randUint64(r *rand.Rand) uint64 {
	return uint64(r.Uint32())<<32 | uint64(r.Uint32())
}
