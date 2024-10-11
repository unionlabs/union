/*
Copyright 2014 Google Inc. All rights reserved.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package fuzz

import (
	"math/rand"
	"reflect"
	"regexp"
	"strings"
	"testing"
	"time"
)

func TestFuzz_basic(t *testing.T) {
	obj := &struct {
		I    int
		I8   int8
		I16  int16
		I32  int32
		I64  int64
		U    uint
		U8   uint8
		U16  uint16
		U32  uint32
		U64  uint64
		Uptr uintptr
		S    string
		B    bool
		T    time.Time
		C64  complex64
		C128 complex128
	}{}

	failed := map[string]int{}
	for i := 0; i < 10; i++ {
		New().Fuzz(obj)

		if n, v := "i", obj.I; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "i8", obj.I8; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "i16", obj.I16; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "i32", obj.I32; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "i64", obj.I64; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "u", obj.U; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "u8", obj.U8; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "u16", obj.U16; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "u32", obj.U32; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "u64", obj.U64; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "uptr", obj.Uptr; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "s", obj.S; v == "" {
			failed[n] = failed[n] + 1
		}
		if n, v := "b", obj.B; v == false {
			failed[n] = failed[n] + 1
		}
		if n, v := "t", obj.T; v.IsZero() {
			failed[n] = failed[n] + 1
		}
		if n, v := "c64", obj.C64; v == 0 {
			failed[n] = failed[n] + 1
		}
		if n, v := "c128", obj.C128; v == 0 {
			failed[n] = failed[n] + 1
		}
	}
	checkFailed(t, failed)
}

func checkFailed(t *testing.T, failed map[string]int) {
	for k, v := range failed {
		if v > 8 {
			t.Errorf("%v seems to not be getting set, was zero value %v times", k, v)
		}
	}
}

func TestFuzz_structptr(t *testing.T) {
	obj := &struct {
		A *struct {
			S string
		}
	}{}

	f := New().NilChance(.5)
	failed := map[string]int{}
	for i := 0; i < 10; i++ {
		f.Fuzz(obj)

		if n, v := "a not nil", obj.A; v == nil {
			failed[n] = failed[n] + 1
		}
		if n, v := "a nil", obj.A; v != nil {
			failed[n] = failed[n] + 1
		}
		if n, v := "as", obj.A; v == nil || v.S == "" {
			failed[n] = failed[n] + 1
		}
	}
	checkFailed(t, failed)
}

// tryFuzz tries fuzzing up to 20 times. Fail if check() never passes, report the highest
// stage it ever got to.
func tryFuzz(t *testing.T, f *Fuzzer, obj interface{}, check func() (stage int, passed bool)) {
	maxStage := 0
	for i := 0; i < 20; i++ {
		f.Fuzz(obj)
		stage, passed := check()
		if stage > maxStage {
			maxStage = stage
		}
		if passed {
			return
		}
	}
	t.Errorf("Only ever got to stage %v", maxStage)
}

func TestFuzz_structmap(t *testing.T) {
	obj := &struct {
		A map[struct {
			S string
		}]struct {
			S2 string
		}
		B map[string]string
	}{}

	tryFuzz(t, New(), obj, func() (int, bool) {
		if obj.A == nil {
			return 1, false
		}
		if len(obj.A) == 0 {
			return 2, false
		}
		for k, v := range obj.A {
			if k.S == "" {
				return 3, false
			}
			if v.S2 == "" {
				return 4, false
			}
		}

		if obj.B == nil {
			return 5, false
		}
		if len(obj.B) == 0 {
			return 6, false
		}
		for k, v := range obj.B {
			if k == "" {
				return 7, false
			}
			if v == "" {
				return 8, false
			}
		}
		return 9, true
	})
}

func TestFuzz_structslice(t *testing.T) {
	obj := &struct {
		A []struct {
			S string
		}
		B []string
	}{}

	tryFuzz(t, New(), obj, func() (int, bool) {
		if obj.A == nil {
			return 1, false
		}
		if len(obj.A) == 0 {
			return 2, false
		}
		for _, v := range obj.A {
			if v.S == "" {
				return 3, false
			}
		}

		if obj.B == nil {
			return 4, false
		}
		if len(obj.B) == 0 {
			return 5, false
		}
		for _, v := range obj.B {
			if v == "" {
				return 6, false
			}
		}
		return 7, true
	})
}

func TestFuzz_structarray(t *testing.T) {
	obj := &struct {
		A [3]struct {
			S string
		}
		B [2]int
	}{}

	tryFuzz(t, New(), obj, func() (int, bool) {
		for _, v := range obj.A {
			if v.S == "" {
				return 1, false
			}
		}

		for _, v := range obj.B {
			if v == 0 {
				return 2, false
			}
		}
		return 3, true
	})
}

func TestFuzz_custom(t *testing.T) {
	obj := &struct {
		A string
		B *string
		C map[string]string
		D *map[string]string
	}{}

	testPhrase := "gotcalled"
	testMap := map[string]string{"C": "D"}
	f := New().Funcs(
		func(s *string, c Continue) {
			*s = testPhrase
		},
		func(m map[string]string, c Continue) {
			m["C"] = "D"
		},
	)

	tryFuzz(t, f, obj, func() (int, bool) {
		if obj.A != testPhrase {
			return 1, false
		}
		if obj.B == nil {
			return 2, false
		}
		if *obj.B != testPhrase {
			return 3, false
		}
		if e, a := testMap, obj.C; !reflect.DeepEqual(e, a) {
			return 4, false
		}
		if obj.D == nil {
			return 5, false
		}
		if e, a := testMap, *obj.D; !reflect.DeepEqual(e, a) {
			return 6, false
		}
		return 7, true
	})
}

type SelfFuzzer string

// Implement fuzz.Interface.
func (sf *SelfFuzzer) Fuzz(c Continue) {
	*sf = selfFuzzerTestPhrase
}

const selfFuzzerTestPhrase = "was fuzzed"

func TestFuzz_interface(t *testing.T) {
	f := New()

	var obj1 SelfFuzzer
	tryFuzz(t, f, &obj1, func() (int, bool) {
		if obj1 != selfFuzzerTestPhrase {
			return 1, false
		}
		return 1, true
	})

	var obj2 map[int]SelfFuzzer
	tryFuzz(t, f, &obj2, func() (int, bool) {
		for _, v := range obj2 {
			if v != selfFuzzerTestPhrase {
				return 1, false
			}
		}
		return 1, true
	})
}

func TestFuzz_interfaceAndFunc(t *testing.T) {
	const privateTestPhrase = "private phrase"
	f := New().Funcs(
		// This should take precedence over SelfFuzzer.Fuzz().
		func(s *SelfFuzzer, c Continue) {
			*s = privateTestPhrase
		},
	)

	var obj1 SelfFuzzer
	tryFuzz(t, f, &obj1, func() (int, bool) {
		if obj1 != privateTestPhrase {
			return 1, false
		}
		return 1, true
	})

	var obj2 map[int]SelfFuzzer
	tryFuzz(t, f, &obj2, func() (int, bool) {
		for _, v := range obj2 {
			if v != privateTestPhrase {
				return 1, false
			}
		}
		return 1, true
	})
}

func TestFuzz_noCustom(t *testing.T) {
	type Inner struct {
		Str string
	}
	type Outer struct {
		Str string
		In  Inner
	}

	testPhrase := "gotcalled"
	f := New().Funcs(
		func(outer *Outer, c Continue) {
			outer.Str = testPhrase
			c.Fuzz(&outer.In)
		},
		func(inner *Inner, c Continue) {
			inner.Str = testPhrase
		},
	)
	c := Continue{fc: &fuzzerContext{fuzzer: f}, Rand: f.r}

	// Fuzzer.Fuzz()
	obj1 := Outer{}
	f.Fuzz(&obj1)
	if obj1.Str != testPhrase {
		t.Errorf("expected Outer custom function to have been called")
	}
	if obj1.In.Str != testPhrase {
		t.Errorf("expected Inner custom function to have been called")
	}

	// Continue.Fuzz()
	obj2 := Outer{}
	c.Fuzz(&obj2)
	if obj2.Str != testPhrase {
		t.Errorf("expected Outer custom function to have been called")
	}
	if obj2.In.Str != testPhrase {
		t.Errorf("expected Inner custom function to have been called")
	}

	// Fuzzer.FuzzNoCustom()
	obj3 := Outer{}
	f.FuzzNoCustom(&obj3)
	if obj3.Str == testPhrase {
		t.Errorf("expected Outer custom function to not have been called")
	}
	if obj3.In.Str != testPhrase {
		t.Errorf("expected Inner custom function to have been called")
	}

	// Continue.FuzzNoCustom()
	obj4 := Outer{}
	c.FuzzNoCustom(&obj4)
	if obj4.Str == testPhrase {
		t.Errorf("expected Outer custom function to not have been called")
	}
	if obj4.In.Str != testPhrase {
		t.Errorf("expected Inner custom function to have been called")
	}
}

func TestFuzz_NumElements(t *testing.T) {
	f := New().NilChance(0).NumElements(0, 1)
	obj := &struct {
		A []int
	}{}

	tryFuzz(t, f, obj, func() (int, bool) {
		if obj.A == nil {
			return 1, false
		}
		return 2, len(obj.A) == 0
	})
	tryFuzz(t, f, obj, func() (int, bool) {
		if obj.A == nil {
			return 3, false
		}
		return 4, len(obj.A) == 1
	})
}

func TestFuzz_Maxdepth(t *testing.T) {
	type S struct {
		S *S
	}

	f := New().NilChance(0)

	f.MaxDepth(1)
	for i := 0; i < 100; i++ {
		obj := S{}
		f.Fuzz(&obj)

		if obj.S != nil {
			t.Errorf("Expected nil")
		}
	}

	f.MaxDepth(3) // field, ptr
	for i := 0; i < 100; i++ {
		obj := S{}
		f.Fuzz(&obj)

		if obj.S == nil {
			t.Errorf("Expected obj.S not nil")
		} else if obj.S.S != nil {
			t.Errorf("Expected obj.S.S nil")
		}
	}

	f.MaxDepth(5) // field, ptr, field, ptr
	for i := 0; i < 100; i++ {
		obj := S{}
		f.Fuzz(&obj)

		if obj.S == nil {
			t.Errorf("Expected obj.S not nil")
		} else if obj.S.S == nil {
			t.Errorf("Expected obj.S.S not nil")
		} else if obj.S.S.S != nil {
			t.Errorf("Expected obj.S.S.S nil")
		}
	}
}

func TestFuzz_SkipPattern(t *testing.T) {
	obj := &struct {
		S1    string
		S2    string
		XXX_S string
		S_XXX string
		In    struct {
			Str    string
			XXX_S1 string
			S2_XXX string
		}
	}{}

	f := New().NilChance(0).SkipFieldsWithPattern(regexp.MustCompile(`^XXX_`))
	f.Fuzz(obj)

	tryFuzz(t, f, obj, func() (int, bool) {
		if obj.XXX_S != "" {
			return 1, false
		}
		if obj.S_XXX == "" {
			return 2, false
		}
		if obj.In.XXX_S1 != "" {
			return 3, false
		}
		if obj.In.S2_XXX == "" {
			return 4, false
		}
		return 5, true
	})
}

func TestFuzz_NilChanceZero(t *testing.T) {
	// This data source for random will result in the following four values
	// being sampled (the first, 0, being the most interesting case):
	//   0; 0.8727288671879787; 0.5547307616625858; 0.021885026049502695
	data := []byte("H0000000\x00")
	f := NewFromGoFuzz(data).NilChance(0)

	var fancyStruct struct {
		A, B, C, D *string
	}
	f.Fuzz(&fancyStruct) // None of the pointers should be nil, as NilChance is 0

	if fancyStruct.A == nil {
		t.Error("First value in struct was nil")
	}

	if fancyStruct.B == nil {
		t.Error("Second value in struct was nil")
	}

	if fancyStruct.C == nil {
		t.Error("Third value in struct was nil")
	}

	if fancyStruct.D == nil {
		t.Error("Fourth value in struct was nil")
	}
}

type int63mode int

const (
	modeRandom int63mode = iota
	modeFirst
	modeLast
)

type customInt63 struct {
	mode int63mode
}

func (c customInt63) Int63n(n int64) int64 {
	switch c.mode {
	case modeFirst:
		return 0
	case modeLast:
		return n - 1
	default:
		return rand.Int63n(n)
	}
}

func Test_charRange_choose(t *testing.T) {
	lowercaseLetters := UnicodeRange{'a', 'z'}

	t.Run("Picks first", func(t *testing.T) {
		r := customInt63{mode: modeFirst}
		letter := lowercaseLetters.choose(r)
		if letter != 'a' {
			t.Errorf("Expected a, got %v", letter)
		}
	})

	t.Run("Picks last", func(t *testing.T) {
		r := customInt63{mode: modeLast}
		letter := lowercaseLetters.choose(r)
		if letter != 'z' {
			t.Errorf("Expected z, got %v", letter)
		}
	})
}

func Test_UnicodeRange_CustomStringFuzzFunc(t *testing.T) {
	a2z := "abcdefghijklmnopqrstuvwxyz"

	unicodeRange := UnicodeRange{'a', 'z'}
	f := New().Funcs(unicodeRange.CustomStringFuzzFunc())
	var myString string
	f.Fuzz(&myString)

	t.Run("Picks a-z string", func(t *testing.T) {
		for i := range myString {
			if !strings.ContainsRune(a2z, rune(myString[i])) {
				t.Errorf("Expected a-z, got %v", string(myString[i]))
			}
		}
	})
}

func Test_UnicodeRange_Check(t *testing.T) {
	unicodeRange := UnicodeRange{'a', 'z'}

	unicodeRange.check()
}

func Test_UnicodeRanges_CustomStringFuzzFunc(t *testing.T) {
	a2z0to9 := "abcdefghijklmnopqrstuvwxyz0123456789"

	unicodeRanges := UnicodeRanges{
		{'a', 'z'},
		{'0', '9'},
	}
	f := New().Funcs(unicodeRanges.CustomStringFuzzFunc())
	var myString string
	f.Fuzz(&myString)

	t.Run("Picks a-z0-9 string", func(t *testing.T) {
		for i := range myString {
			if !strings.ContainsRune(a2z0to9, rune(myString[i])) {
				t.Errorf("Expected a-z0-9, got %v", string(myString[i]))
			}
		}
	})
}

func TestNewFromGoFuzz(t *testing.T) {
	t.Parallel()

	input := []byte{1, 2, 3}

	var got int
	NewFromGoFuzz(input).Fuzz(&got)

	if want := 5563767293437588600; want != got {
		t.Errorf("Fuzz(%q) = %d, want: %d", input, got, want)
	}
}

func BenchmarkRandBool(b *testing.B) {
	rs := rand.New(rand.NewSource(123))

	for i := 0; i < b.N; i++ {
		randBool(rs)
	}
}

func BenchmarkRandString(b *testing.B) {
	rs := rand.New(rand.NewSource(123))

	for i := 0; i < b.N; i++ {
		randString(rs)
	}
}

func BenchmarkUnicodeRangeRandString(b *testing.B) {
	unicodeRange := UnicodeRange{'a', 'z'}

	rs := rand.New(rand.NewSource(123))

	for i := 0; i < b.N; i++ {
		unicodeRange.randString(rs)
	}
}

func BenchmarkUnicodeRangesRandString(b *testing.B) {
	unicodeRanges := UnicodeRanges{
		{'a', 'z'},
		{'0', '9'},
	}

	rs := rand.New(rand.NewSource(123))

	for i := 0; i < b.N; i++ {
		unicodeRanges.randString(rs)
	}
}
