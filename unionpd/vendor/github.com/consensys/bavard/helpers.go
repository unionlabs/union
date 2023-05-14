// Copyright 2020 ConsenSys Software Inc.
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

package bavard

import (
	"bytes"
	"errors"
	"fmt"
	"math/big"
	"math/bits"
	"reflect"
	"strconv"
	"strings"
	"sync"
	"text/template"
)

// Template helpers (txt/template)
func helpers() template.FuncMap {
	// functions used in template
	return template.FuncMap{
		"add":        add,
		"bits":       getBits,
		"bytes":      intBytes, //TODO: Do this directly
		"capitalize": strings.Title,
		"dict":       dict,
		"div":        div,
		"divides":    divides,
		"first":      first,
		"gt":		  gt,
		"interval":   interval,
		"iterate":    iterate,
		"select":	_select,
		"last":       last,
		"list":       makeSlice,
		"log":        fmt.Println,
		"lt":		  lt,
		"mod":       mod,
		"mul":       mul,
		"mul2":      mul2,
		"noFirst":   noFirst,
		"noLast":    noLast,
		"notNil":    notNil,
		"pretty":    pretty,
		"printList": printList,
		"reverse":   reverse,
		"sub":       sub,
		"supScr":    toSuperscript,
		"toInt64":   toInt64,
		"toLower":   strings.ToLower,
		"toTitle":   strings.Title,
		"toUpper":   strings.ToUpper,
		"words64":   bigIntToUint64SliceAsString,
	}
}

func _select(condition bool, ifNot, ifSo interface{}) interface{} {
	if condition {
		return ifSo
	}
	return ifNot
}

func pretty(a interface{}) interface{} {
	if res, err := printList(a); err == nil {
		return res
	}

	if s, ok := a.(big.Int); ok {
		return s.String()
	}

	return a
}

func cmp(a, b interface{}, expectedCmp int) (bool, error) {
	aI, err := toBigInt(a)
	if err != nil {
		return false, err
	}
	var bI big.Int
	bI, err = toBigInt(b)
	if err != nil {
		return false, err
	}
	return aI.Cmp(&bI) == expectedCmp, nil
}

func lt(a, b interface{}) (bool, error) {
	return cmp(a, b, -1)
}

func gt(a, b interface{}) (bool, error) {
	return cmp(a, b, +1)
}

func getBitsBig(a big.Int) ([]bool, error) {
	l := a.BitLen()
	res := make([]bool, l)
	for i := 0; i < l; i ++ {
		res[i] = a.Bit(i) == 1
	}
	return res, nil
}

func getBits(a interface{}) ([]bool, error) {

	if aBI, ok := a.(big.Int); ok {
		return getBitsBig(aBI)
	}

	var res []bool
	aI, err := toInt64(a)

	if err != nil {
		return res, err
	}

	for aI != 0 {
		res = append(res, aI%2 != 0)
		aI /= 2
	}

	return res, nil
}

func toBigInt(a interface{}) (big.Int, error) {
	switch i := a.(type) {
	case big.Int:
		return i, nil
	case *big.Int:
		return *i, nil
	/*case string:
		var res big.Int
		res.SetString(i, 0)
		return res, nil*/
	default:
		n, err := toInt64(i)
		return *big.NewInt(n), err
	}
}

func toInt64(a interface{}) (int64, error) {
	switch i := a.(type) {
	case uint8:
		return int64(i), nil
	case int8:
		return int64(i), nil
	case uint16:
		return int64(i), nil
	case int16:
		return int64(i), nil
	case uint32:
		return int64(i), nil
	case int32:
		return int64(i), nil
	case uint64:
		if i>>63 != 0 {
			return -1, fmt.Errorf("uint64 value won't fit in int64")
		}
		return int64(i), nil
	case int64:
		return i, nil
	case int:
		return int64(i), nil
	case big.Int:
		if !i.IsInt64() {
			return -1, fmt.Errorf("big.Int value won't fit in int64")
		}
		return i.Int64(), nil
	case *big.Int:
		if !i.IsInt64() {
			return -1, fmt.Errorf("big.Int value won't fit in int64")
		}
		return i.Int64(), nil
	default:
		return 0, fmt.Errorf("cannot convert to int64 from type %T", i)
	}
}

func mod(a, b interface{}) (int64, error) {

	var err error
	A, err := toInt64(a)

	if err != nil {
		return 0, err
	}

	B, err := toInt64(b)

	if err != nil {
		return 0, err
	}
	return A % B, nil
}

func intBytes(i big.Int) []byte {
	return i.Bytes()
}

func interval(begin, end interface{}) ([]int64, error) {
	beginInt, err := toInt64(begin)
	if err != nil {
		return nil, err
	}
	endInt, err := toInt64(end)
	if err != nil {
		return nil, err
	}

	l := endInt - beginInt
	r := make([]int64, l)
	for i := int64(0); i < l; i++ {
		r[i] = i + beginInt
	}
	return r, nil
}

// Adopted from https://stackoverflow.com/a/50487104/5116581
func notNil(input interface{}) bool {
	isNil := input == nil || (reflect.ValueOf(input).Kind() == reflect.Ptr && reflect.ValueOf(input).IsNil())
	return !isNil
}

func AssertSlice(input interface{}) (reflect.Value, error) {
	s := reflect.ValueOf(input)
	if s.Kind() != reflect.Slice {
		return s, fmt.Errorf("value %s is not a slice", fmt.Sprint(s))
	}
	return s, nil
}

func first(input interface{}) (interface{}, error) {
	s, err := AssertSlice(input)
	if err != nil {
		return nil, err
	}
	if s.Len() == 0 {
		return nil, fmt.Errorf("empty slice")
	}
	return s.Index(0).Interface(), nil
}

func last(input interface{}) (interface{}, error) {
	s, err := AssertSlice(input)
	if err != nil {
		return nil, err
	}
	if s.Len() == 0 {
		return nil, fmt.Errorf("empty slice")
	}
	return s.Index(s.Len() - 1).Interface(), nil
}

var StringBuilderPool = sync.Pool{New: func() interface{} { return &strings.Builder{} }}

func WriteBigIntAsUint64Slice(builder *strings.Builder, input *big.Int) {
	words := input.Bits()

	if len(words) == 0 {
		builder.WriteString("0")
		return
	}

	for i := 0; i < len(words); i++ {
		w := uint64(words[i])

		if bits.UintSize == 32 && i < len(words)-1 {
			i++
			w = (w << 32) | uint64(words[i])
		}

		builder.WriteString(strconv.FormatUint(w, 10))

		if i < len(words)-1 {
			builder.WriteString(", ")
		}
	}
}

func bigIntToUint64SliceAsString(in interface{}) (string, error) {

	var input *big.Int

	switch i := in.(type) {
	case big.Int:
		input = &i
	case *big.Int:
		input = i
	default:
		return "", fmt.Errorf("unsupported type %T", in)
	}

	builder := StringBuilderPool.Get().(*strings.Builder)
	builder.Reset()
	defer StringBuilderPool.Put(builder)

	WriteBigIntAsUint64Slice(builder, input)

	return builder.String(), nil
}

func printList(input interface{}) (string, error) {

	s, err := AssertSlice(input)

	if err != nil || s.Len() == 0 {
		return "", err
	}

	builder := StringBuilderPool.Get().(*strings.Builder)
	builder.Reset()
	defer StringBuilderPool.Put(builder)

	builder.WriteString(fmt.Sprint(pretty(s.Index(0).Interface())))

	for i := 1; i < s.Len(); i++ {
		builder.WriteString(", ")
		builder.WriteString(fmt.Sprint(pretty(s.Index(i).Interface())))
	}

	return builder.String(), nil
}

func iterate(start, end interface{}) (r []int64, err error) {

	startI, err := toInt64(start)
	if err != nil {
		return nil, err
	}
	var endI int64
	endI, err = toInt64(end)

	if err != nil {
		return nil, err
	}

	for i := startI; i < endI; i++ {
		r = append(r, i)
	}
	return
}

func reverse(input interface{}) interface{} {

	s, err := AssertSlice(input)
	if err != nil {
		return err
	}
	l := s.Len()
	toReturn := reflect.MakeSlice(s.Type(), l, l)

	l--
	for i := 0; i <= l; i++ {
		toReturn.Index(l - i).Set(s.Index(i))
	}
	return toReturn.Interface()
}

func noFirst(input interface{}) interface{} {
	s, err := AssertSlice(input)
	if s.Len() == 0 {
		return input
	}
	if err != nil {
		return err
	}
	l := s.Len() - 1
	toReturn := reflect.MakeSlice(s.Type(), l, l)
	for i := 0; i < l; i++ {
		toReturn.Index(i).Set(s.Index(i + 1))
	}
	return toReturn.Interface()
}

func noLast(input interface{}) interface{} {
	s, err := AssertSlice(input)
	if s.Len() == 0 {
		return input
	}
	if err != nil {
		return err
	}
	l := s.Len() - 1
	toReturn := reflect.MakeSlice(s.Type(), l, l)
	for i := 0; i < l; i++ {
		toReturn.Index(i).Set(s.Index(i))
	}
	return toReturn.Interface()
}

func add(a, b interface{}) (int64, error) {
	aI, err := toInt64(a)
	if err != nil {
		return 0, err
	}
	var bI int64
	if bI, err = toInt64(b); err != nil {
		return 0, err
	}
	return aI + bI, nil
}
func mul(a, b interface{}) (int64, error) {
	aI, err := toInt64(a)
	if err != nil {
		return 0, err
	}
	var bI int64
	if bI, err = toInt64(b); err != nil {
		return 0, err
	}
	return aI * bI, nil
}
func sub(a, b interface{}) (int64, error) {
	aI, err := toInt64(a)
	if err != nil {
		return 0, err
	}
	var bI int64
	if bI, err = toInt64(b); err != nil {
		return 0, err
	}
	return aI - bI, nil
}
func mul2(a interface{}) (int64, error) {
	aI, err := toInt64(a)
	if err != nil {
		return 0, err
	}

	return aI * 2, nil
}
func div(a, b interface{}) (int64, error) {
	aI, err := toInt64(a)
	if err != nil {
		return 0, err
	}
	var bI int64
	if bI, err = toInt64(b); err != nil {
		return 0, err
	}
	return aI / bI, nil
}

func makeSlice(values ...interface{}) []interface{} {
	return values
}

func dict(values ...interface{}) (map[string]interface{}, error) {
	if len(values)%2 != 0 {
		return nil, errors.New("invalid dict call")
	}
	dict := make(map[string]interface{}, len(values)/2)
	for i := 0; i < len(values); i += 2 {
		key, ok := values[i].(string)
		if !ok {
			return nil, errors.New("dict keys must be strings")
		}
		dict[key] = values[i+1]
	}
	return dict, nil
}

// return true if c1 divides c2, that is, c2 % c1 == 0
func divides(c1, c2 interface{}) (bool, error) {

	//try to convert to int64
	c1Int, err := toInt64(c1)
	if err != nil {
		return false, err
	}
	var c2Int int64
	c2Int, err = toInt64(c2)
	if err != nil {
		return false, err
	}

	return c2Int%c1Int == 0, nil
}

// Imitating supsub
var superscripts = map[rune]rune{
	'0': '⁰',
	'1': '¹',
	'2': '²',
	'3': '³',
	'4': '⁴',
	'5': '⁵',
	'6': '⁶',
	'7': '⁷',
	'8': '⁸',
	'9': '⁹',
}

// toSuperscript writes a number as a "power"
//TODO: Use https://github.com/lynn9388/supsub ?
//Copying supsub
func toSuperscript(a interface{}) (string, error) {
	i, err := toInt64(a)

	if err != nil {
		return "", err
	}

	s := strconv.FormatInt(i, 10)
	var buf bytes.Buffer
	for _, r := range s {
		sup := superscripts[r]
		buf.WriteRune(sup)
	}
	return buf.String(), nil
}
