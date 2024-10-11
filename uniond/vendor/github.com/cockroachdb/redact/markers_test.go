// Copyright 2020 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package redact

import (
	"bytes"
	"errors"
	"fmt"
	"reflect"
	"regexp"
	"strings"
	"testing"

	"github.com/cockroachdb/redact/builder"
	i "github.com/cockroachdb/redact/interfaces"
	m "github.com/cockroachdb/redact/internal/markers"
)

type p = SafePrinter

func TestPrinter(t *testing.T) {
	var sn *safeNil
	var unsafeSptr *string
	ptrptr := &unsafeSptr
	ptrptrP := fmt.Sprintf("%p", ptrptr)
	ptrptrD := fmt.Sprintf("%d", ptrptr)
	ptrptrV := fmt.Sprintf("%v", ptrptr)
	var buf builder.StringBuilder
	buf.Printf("safe %s", "unsafe")

	testData := []struct {
		fn       func(p)
		expected string
	}{
		{func(w p) { w.SafeString("ab") }, `ab`},
		{func(w p) { w.SafeRune('☃') }, `☃`},
		{func(w p) { w.SafeRune(' ') }, ` `},
		{func(w p) { w.SafeInt(-123) }, `-123`},
		{func(w p) { w.SafeUint(123) }, `123`},
		{func(w p) { w.SafeFloat(3.14) }, `3.14`},
		{func(w p) { w.UnsafeString("rs") }, `‹rs›`},
		{func(w p) { w.UnsafeByte('t') }, `‹t›`},
		{func(w p) { w.UnsafeByte(m.StartS[0]) }, `‹?›`},
		{func(w p) { w.UnsafeBytes([]byte("uv")) }, `‹uv›`},
		{func(w p) { w.UnsafeRune('🛑') }, `‹🛑›`},
		{func(w p) { w.Print("fg", safe("hi")) }, `‹fg›hi`},
		{func(w p) { w.Print("fg", reflect.ValueOf(safe("hi"))) }, `‹fg›hi`},
		{func(w p) { w.Printf("jk %s %s", "lm", safe("no")) }, `jk ‹lm› no`},
		{func(w p) { w.Printf("jk %s %s", "lm", reflect.ValueOf(safe("no"))) }, `jk ‹lm› no`},
		{func(w p) { w.Print("ar", []string{"hel", "lo"}) }, `‹ar›[‹hel› ‹lo›]`},
		{func(w p) { w.Print("ar", []i.SafeValue{safe("hel"), safe("lo")}) }, `‹ar›[hel lo]`},
		{func(w p) { w.Print("ar", []interface{}{safe("hel"), safe("lo")}) }, `‹ar›[hel lo]`},
		{func(w p) { w.Print("ar", []int{123, 456}) }, `‹ar›[‹123› ‹456›]`},
		{func(w p) { w.Print("ar", []byte{55, 56}) }, `‹ar›[‹55› ‹56›]`},
		{func(w p) { w.Print("ar", Safe([]byte{55, 56})) }, `‹ar›[55 56]`},

		// Numeric values.
		{func(w p) { w.Printf("vn %+d", SafeInt(123)) }, `vn +123`},
		{func(w p) { w.Printf("vn %05d", SafeUint(123)) }, `vn 00123`},
		{func(w p) { w.Printf("vn %e", SafeFloat(3.14)) }, `vn 3.140000e+00`},

		// Pre-redactable strings.
		{func(w p) { w.Print("pr", RedactableString("hi")) }, `‹pr›hi`},
		{func(w p) { w.Print("pr", RedactableBytes("hi")) }, `‹pr›hi`},
		{func(w p) { w.Print("prr", reflect.ValueOf(RedactableString("hi"))) }, `‹prr›hi`},
		{func(w p) { w.Print("prr", reflect.ValueOf(RedactableBytes("hi"))) }, `‹prr›hi`},

		// The special string verbs are honored for plain strings.
		{func(w p) { w.Printf("fg %q", safe("hi")) }, `fg "hi"`},
		{func(w p) { w.Printf("fg %#q", safe("hi")) }, "fg `hi`"},
		{func(w p) { w.Printf("fg %x", safe("hi")) }, `fg 6869`},
		{func(w p) { w.Printf("fg %X", safe("hi")) }, "fg 6869"},
		{func(w p) { w.Printf("fg %q", Safe("hi")) }, `fg "hi"`},
		{func(w p) { w.Printf("fg %#q", Safe("hi")) }, "fg `hi`"},
		{func(w p) { w.Printf("fg %x", Safe("hi")) }, `fg 6869`},
		{func(w p) { w.Printf("fg %X", Safe("hi")) }, "fg 6869"},
		{func(w p) { w.Printf("fg %q", SafeString("hi")) }, `fg "hi"`},
		{func(w p) { w.Printf("fg %#q", SafeString("hi")) }, "fg `hi`"},
		{func(w p) { w.Printf("fg %x", SafeString("hi")) }, `fg 6869`},
		{func(w p) { w.Printf("fg %X", SafeString("hi")) }, "fg 6869"},
		{func(w p) { w.Printf("fg %q", "hi") }, `fg ‹"hi"›`},
		{func(w p) { w.Printf("fg %#q", "hi") }, "fg ‹`hi`›"},
		{func(w p) { w.Printf("fg %x", "hi") }, `fg ‹6869›`},
		{func(w p) { w.Printf("fg %X", "hi") }, "fg ‹6869›"},
		// However they are not honored for pre-redactable strings,
		// because they would unsafely mask the redaction markers
		{func(w p) { w.Printf("fg %q", RedactableString("hi")) }, `fg hi`},
		{func(w p) { w.Printf("fg %#q", RedactableString("hi")) }, "fg hi"},
		{func(w p) { w.Printf("fg %x", RedactableString("hi")) }, `fg hi`},
		{func(w p) { w.Printf("fg %X", RedactableString("hi")) }, "fg hi"},
		{func(w p) { w.Printf("fg %q", RedactableBytes("hi")) }, `fg hi`},
		{func(w p) { w.Printf("fg %#q", RedactableBytes("hi")) }, "fg hi"},
		{func(w p) { w.Printf("fg %x", RedactableBytes("hi")) }, `fg hi`},
		{func(w p) { w.Printf("fg %X", RedactableBytes("hi")) }, "fg hi"},
		// Direct access to the fmt.State.
		{func(w p) { _, _ = w.Write([]byte("pq")) }, `‹pq›`},
		// Safe strings and runes containing the delimiters get escaped.
		{func(w p) { w.SafeString("a ‹ b › c") }, `a ? b ? c`},
		{func(w p) { w.SafeRune('‹') }, `?`},
		{func(w p) { w.SafeRune('›') }, `?`},
		{func(w p) { w.Print("a ‹ b › c", safe("d ‹ e › f")) },
			`‹a ? b ? c›d ? e ? f`},
		{func(w p) { w.Printf("f %s %s", "a ‹ b › c", safe("d ‹ e › f")) },
			`f ‹a ? b ? c› d ? e ? f`},
		// Space and newlines at the end of an unsafe string get removed,
		// but not at the end of a safe string.
		{func(w p) { w.SafeString("ab \n ") }, "ab \n "},
		{func(w p) { w.UnsafeString("cd \n ") }, "‹cd ›\n‹ ›"},
		{func(w p) { w.Print("ab ", safe("cd ")) }, "‹ab ›cd "},
		{func(w p) { w.Printf("ab :%s: :%s: ", "cd ", safe("de ")) }, "ab :‹cd ›: :de : "},
		// Spaces as runes get preserved.
		{func(w p) { w.SafeRune(' ') }, ` `},
		{func(w p) { w.SafeRune('\n') }, "\n"},
		{func(w p) { w.UnsafeRune(' ') }, `‹ ›`},
		{func(w p) { w.UnsafeRune('\n') }, "\n"},
		// The Safe() API turns anything into something safe. However, the contents
		// still get escaped as needed.
		{func(w p) { w.Print("ab ", Safe("c‹d›e ")) }, "‹ab ›c?d?e "},
		{func(w p) { w.Printf("ab %03d ", Safe(12)) }, "ab 012 "},
		// Something that'd be otherwise safe, becomes unsafe with Unsafe().
		{func(w p) { w.Print(Unsafe(SafeString("abc"))) }, "‹abc›"},
		{func(w p) { w.Print(Unsafe(RedactableString("ab‹c›"))) }, "‹ab?c?›"},
		{func(w p) { w.Print(Unsafe(RedactableBytes("ab‹c›"))) }, "‹ab?c?›"},
		{func(w p) { w.Print(Unsafe(SafeRune('a'))) }, "‹97›"},
		{func(w p) { w.Print(Unsafe(SafeInt(-123))) }, "‹-123›"},
		{func(w p) { w.Print(Unsafe(SafeUint(123))) }, "‹123›"},
		{func(w p) { w.Print(Unsafe(SafeFloat(3.14))) }, "‹3.14›"},
		{func(w p) { w.Print(Unsafe(Sprint("abc"))) }, "‹?abc?›"},
		{func(w p) { w.Print(Unsafe(Safe("abc"))) }, "‹abc›"},
		{func(w p) { w.Printf("%v", Unsafe(SafeString("abc"))) }, "‹abc›"},
		{func(w p) { w.Printf("%v", Unsafe(SafeRune('a'))) }, "‹97›"},
		{func(w p) { w.Printf("%v", Unsafe(Sprint("abc"))) }, "‹?abc?›"},
		{func(w p) { w.Printf("%v", Unsafe(Safe("abc"))) }, "‹abc›"},
		{func(w p) { w.Printf("%03d", Unsafe(12)) }, "‹012›"},
		// A string that's already redactable gets included as-is;
		// in that case, the printf verb and flags are ignored.
		{func(w p) { w.Print("ab ", Sprint(12, Safe(34))) }, "‹ab ›‹12› 34"},
		{func(w p) { w.Printf("ab %q", Sprint(12, Safe(34))) }, "ab ‹12› 34"},
		{func(w p) { w.Printf("ab %d", Sprint(12, Safe(34))) }, "ab ‹12› 34"},
		// Nil untyped or interface-typed objects get formatted as safe.
		{func(w p) { w.Printf("ab %v", nil) }, "ab <nil>"},
		{func(w p) { w.Printf("ab %v", error(nil)) }, "ab <nil>"},
		{func(w p) { w.Printf("ab %v", Safe(nil)) }, "ab <nil>"},
		// Nil typed objects are unsafe.
		{func(w p) { w.Printf("ab %v", unsafeSptr) }, "ab ‹<nil>›"},
		// But a nil pointer to a type that has a SafeFormat() method is fine.
		{func(w p) { w.Printf("ab %v", sn) }, "ab hello ‹world›"},
		{func(w p) { w.Printf("ab %v", (*safeNil)(nil)) }, "ab hello ‹world›"},
		// Reflected values can be formatted too.
		{func(w p) { w.Printf("ab %.1f", reflect.ValueOf(12.3456)) }, "ab ‹12.3›"},
		{func(w p) { w.Printf("ab %.1f", Safe(reflect.ValueOf(12.3456))) }, "ab 12.3"},
		// Pointers.
		{func(w p) { w.Printf("pv %v", ptrptr) }, "pv ‹" + ptrptrV + "›"},
		{func(w p) { w.Printf("pd %d", ptrptr) }, "pd ‹" + ptrptrD + "›"},
		{func(w p) { w.Printf("pp %p", ptrptr) }, "pp ‹" + ptrptrP + "›"},
		{func(w p) { w.Printf("spv %v", Safe(ptrptr)) }, "spv " + ptrptrV},
		{func(w p) { w.Printf("spd %d", Safe(ptrptr)) }, "spd " + ptrptrD},
		{func(w p) { w.Printf("spp %p", Safe(ptrptr)) }, "spp " + ptrptrP},
		{func(w p) { w.Printf("upv %v", Unsafe(ptrptr)) }, "upv ‹" + ptrptrV + "›"},
		{func(w p) { w.Printf("upd %d", Unsafe(ptrptr)) }, "upd ‹" + ptrptrD + "›"},
		{func(w p) { w.Printf("upp %p", Unsafe(ptrptr)) }, "upp ‹" + ptrptrP + "›"},

		// Check for bad verbs.
		{func(w p) { w.Printf("ab %d", true) }, "ab %!d(bool=‹true›)"},
		{func(w p) { w.Printf("ab %d", Safe(true)) }, "ab %!d(bool=true)"},
		{func(w p) { w.Printf("ab %d") }, "ab %!d(MISSING)"},
		{func(w p) { w.Printf("ab %[2]d", 123) }, "ab %!d(BADINDEX)"},
		{func(w p) { w.Printf("ab %.*d", -1, 123) }, "ab %!(BADPREC)‹123›"},
		// A badly formed verb does not leak information.
		{func(w p) { w.Printf("ab %2", 123) }, "ab %!(NOVERB)%!(EXTRA int=‹123›)"},

		// The %T verb does what it says on the label. The type itself is
		// considered safe.
		{func(w p) { w.Printf("ab %T", 123) }, "ab int"},
		{func(w p) { w.Printf("ab %T", Safe(123)) }, "ab int"},
		{func(w p) { w.Printf("ab %T", Unsafe(123)) }, "ab ‹int›"},

		// A struct does get recursively redacted.
		{func(w p) { w.Print(SafeString("c1"), &complexObj{"somestring"}) }, "c1&{‹somestring›}"},
		{func(w p) { w.Printf("c2 %v", &complexObj{"somestring"}) }, "c2 &{‹somestring›}"},
		{func(w p) { w.Printf("c3 %+v", &complexObj{"somestring"}) }, "c3 &{v:‹somestring›}"},
		{func(w p) { w.Printf("c4 %#v", &complexObj{"somestring"}) }, `c4 &redact.complexObj{v:‹"somestring"›}`},
		{func(w p) { w.Printf("c5 %v", reflect.ValueOf(&complexObj{"somestring"})) }, "c5 &{‹somestring›}"},
		// It can also be marked safe.
		{func(w p) { w.Print(SafeString("c6"), Safe(&complexObj{"somestring"})) }, "c6&{somestring}"},
		{func(w p) { w.Printf("c7 %v", Safe(&complexObj{"somestring"})) }, "c7 &{somestring}"},
		{func(w p) { w.Printf("c8 %+v", Safe(&complexObj{"somestring"})) }, "c8 &{v:somestring}"},
		{func(w p) { w.Printf("c9 %#v", Safe(&complexObj{"somestring"})) }, `c9 &redact.complexObj{v:"somestring"}`},
		{func(w p) { w.Printf("c10 %v", Safe(reflect.ValueOf(&complexObj{"somestring"}))) }, `c10 &{somestring}`},
		// String builders are also printable.
		{func(w p) { w.Printf("%v", buf) }, "safe ‹unsafe›"},
		{func(w p) { w.Print(buf) }, "safe ‹unsafe›"},
		{func(w p) { w.Printf("%v", &buf) }, "safe ‹unsafe›"},
		{func(w p) { w.Print(&buf) }, "safe ‹unsafe›"},
	}

	var methods = []struct {
		name string
		fn   func(interface{}) string
	}{
		{"sprint", func(a interface{}) string { return string(Sprint(a)) }},
		{"sprintf", func(a interface{}) string { return string(Sprintf("%v", a)) }},
		{"fprint", func(a interface{}) string { var b strings.Builder; _, _ = Fprint(&b, a); return b.String() }},
		{"fprintf", func(a interface{}) string { var b strings.Builder; _, _ = Fprintf(&b, "%v", a); return b.String() }},
	}

	for _, m := range methods {
		t.Run(m.name, func(t *testing.T) {
			for i, tc := range testData {
				t.Run(fmt.Sprintf("%d:%q", i, tc.expected), func(t *testing.T) {
					res := m.fn(compose{fn: tc.fn})

					if res != tc.expected {
						t.Errorf("%d: expected:\n  %s\n\ngot:\n%s", i,
							strings.ReplaceAll(tc.expected, "\n", "\n  "),
							strings.ReplaceAll(res, "\n", "\n  "))
					}
				})
			}
		})
	}
}

func TestConversions(t *testing.T) {
	const data = `‹123› 456`
	s := RedactableString(data)

	bconv := s.ToBytes()
	expected := []byte(data)
	if !bytes.Equal(bconv, expected) {
		t.Errorf("\nexpected: %+v,\n     got: %+v", expected, bconv)
	}

	sconv := bconv.ToString()
	if s != sconv {
		t.Errorf("expected %q, got %q", s, sconv)
	}
}

func TestFormatPropagation(t *testing.T) {
	testData := []struct {
		actual   RedactableString
		expected RedactableString
	}{
		{Sprintf(":%10s:", safe("abc")), `:       abc:`},
		{Sprintf(":%10s:", "abc"), `:‹       abc›:`},
		{Sprintf(":%+#03x:", safeint(123)), `:+0x7b:`},
		{Sprintf(":%+#03x:", 123), `:‹+0x7b›:`},
	}

	for _, tc := range testData {
		if tc.actual != tc.expected {
			t.Errorf("expected %q, got %q", tc.expected, tc.actual)
		}
	}
}

type compose struct {
	fn func(p)
}

func (c compose) SafeFormat(w SafePrinter, _ rune) {
	c.fn(w)
}

type safe string

func (safe) SafeValue() {}

type safeint int

func (safeint) SafeValue() {}

func TestTransform(t *testing.T) {
	testData := []struct {
		actual   string
		expected string
	}{
		{string(StartMarker()), `‹`},
		{string(EndMarker()), `›`},
		{string(RedactedMarker()), `‹×›`},
		{string(EscapeMarkers([]byte(`a ‹ b › c`))), `a ? b ? c`},
		{string(RedactableBytes([]byte(`a ‹ b › c`)).Redact()), `a ‹×› c`},
		{string(RedactableBytes([]byte(`a ‹ b › c`)).StripMarkers()), `a  b  c`},
		{string(RedactableString(`a ‹ b › c`).Redact()), `a ‹×› c`},
		{RedactableString(`a ‹ b › c`).StripMarkers(), `a  b  c`},
	}

	for _, tc := range testData {
		if tc.actual != tc.expected {
			t.Errorf("expected %q, got %q", tc.expected, tc.actual)
		}
	}
}

// TestRedactStream verifies that the redaction logic is able to both
// add the redaction quotes and also respects the format parameters
// and verb.
func TestRedactStream(t *testing.T) {
	testData := []struct {
		f        string
		input    interface{}
		expected string
	}{
		{"%v", "", ""},
		{"%v", " ", "‹ ›"},
		{"‹› %v ›››", "abc", "?? ‹abc› ???"},
		{"%v", "abc ", "‹abc ›"},
		{"%q", "abc ", `‹"abc "›`},
		{"%v", "abc\n ", "‹abc›\n‹ ›"},
		{"%v", "abc \n\n", "‹abc ›\n\n"},
		{"%v", " \n\nabc", "‹ ›\n\n‹abc›"},
		{"%v", "‹abc›", "‹?abc?›"},
		{"%v", 123, "‹123›"},
		{"%05d", 123, "‹00123›"},
		{"%v", Safe(123), "123"},
		{"%05d", Safe(123), "00123"},
		{"%#x", 17, "‹0x11›"},
		{"%+v", &complexObj{"‹›"}, "&{v:‹??›}"},
		{"%v", &safestringer{"as"}, "as"},
		{"%v", &stringer{"as"}, "‹as›"},
		{"%v", &safefmtformatter{"af"}, "af"},
		{"%v", &fmtformatter{"af"}, "‹af›"},
		{"%v", &safemsg{"az"}, "az"},
		// Printers that cause panics during rendering.
		{"%v", &safepanicObj1{"s1-x‹y›z"}, `%!v(PANIC=String method: s1-x?y?z)`},
		{"%v", &safepanicObj2{"s2-x‹y›z"}, `%!v(PANIC=Format method: s2-x?y?z)`},
		{"%v", &panicObj1{"p1-x‹y›z"}, `%!v(PANIC=String method: ‹p1-x?y?z›)`},
		{"%v", &panicObj2{"p2-x‹y›z"}, `%!v(PANIC=Format method: ‹p2-x?y?z›)`},
		{"%v", &panicObj3{"p3-x‹y›z"}, `%!v(PANIC=SafeFormat method: ‹p3-x?y?z›)`},
		{"%v", &panicObj4{"unused"}, `%!v(PANIC=SafeMessager method: ‹woo›)`},
		{"%v", (*safestringer)(nil), `<nil>`},
		{"%v", (*safemsg)(nil), `<nil>`},
		{"%v", (*safefmtformatter)(nil), `<nil>`},
		{"%v", (*safepanicObj1)(nil), `<nil>`},
		{"%v", (*safepanicObj2)(nil), `<nil>`},
		{"%v", (*panicObj1)(nil), `<nil>`},
		{"%v", (*panicObj2)(nil), `<nil>`},
		{"%v", (*panicObj3)(nil), `<nil>`},
		{"%v", (*panicObj4)(nil), `<nil>`},
	}

	for i, tc := range testData {
		var buf strings.Builder
		n, _ := Fprintf(&buf, tc.f, tc.input)
		result := buf.String()
		if result != tc.expected {
			t.Errorf("%d: expected %q, got %q", i, tc.expected, result)
		}
		if n != len(result) {
			t.Errorf("%d: expected len %d, got %d", i, n, len(result))
		}
	}
}

func Example_format() {
	testCases := []struct {
		format string
		args   []interface{}
	}{
		{"%d", []interface{}{123}},
		{"%v", []interface{}{[]int{123, 456}}},
		{"%v", []interface{}{[]RedactableString{"safe", "‹unsafe›"}}},
		{"%v", []interface{}{[]safe{"safe", "safe2"}}},
		{"%v", []interface{}{[]safestringer{{"safe"}, {"safe2"}}}},
		{"%v", []interface{}{makeMixedInts()}},
		{"%+v", []interface{}{makeMixedInts()}},
		{"%v", []interface{}{[]mixedInts{makeMixedInts(), makeMixedInts()}}},
		{"%+v", []interface{}{makeMixedSafe()}},
		{"%+v", []interface{}{makeMixedSafeStringer()}},
		{"%+v", []interface{}{makeMixedRedactableString()}},
		{"%+v", []interface{}{makeMixedRedactableBytes()}},
	}

	type formatterFn func(format string, args ...interface{}) string
	formatters := []struct {
		name string
		fn   formatterFn
	}{
		{"fmt.sprint", fmt.Sprintf},
		{"redact.sprint", func(format string, args ...interface{}) string { return string(Sprintf(format, args...)) }},
		{"redact.printer", func(format string, args ...interface{}) string {
			fn := func(w p) { w.Printf(format, args...) }
			return string(Sprint(compose{fn: fn}))
		}},
	}

	for _, tc := range testCases {
		base := fmt.Sprintf("%q :: %T :: %+v", tc.format, tc.args[0], tc.args)
		base = ptrRe.ReplaceAllString(base, "℘")
		fmt.Println(base)
		for _, f := range formatters {
			result := f.fn(tc.format, tc.args...)
			// Erase pointers.
			result = ptrRe.ReplaceAllString(result, "℘")
			fmt.Printf("%s:\t%s\n", f.name, result)
		}
		fmt.Println()
	}

	// Output:
	// "%d" :: int :: [123]
	// fmt.sprint:	123
	// redact.sprint:	‹123›
	// redact.printer:	‹123›
	//
	// "%v" :: []int :: [[123 456]]
	// fmt.sprint:	[123 456]
	// redact.sprint:	[‹123› ‹456›]
	// redact.printer:	[‹123› ‹456›]
	//
	// "%v" :: []markers.RedactableString :: [[safe ‹unsafe›]]
	// fmt.sprint:	[safe ‹unsafe›]
	// redact.sprint:	[safe ‹unsafe›]
	// redact.printer:	[safe ‹unsafe›]
	//
	// "%v" :: []redact.safe :: [[safe safe2]]
	// fmt.sprint:	[safe safe2]
	// redact.sprint:	[safe safe2]
	// redact.printer:	[safe safe2]
	//
	// "%v" :: []redact.safestringer :: [[{s:safe} {s:safe2}]]
	// fmt.sprint:	[{safe} {safe2}]
	// redact.sprint:	[{‹safe›} {‹safe2›}]
	// redact.printer:	[{‹safe›} {‹safe2›}]
	//
	// "%v" :: redact.mixedInts :: [{a:123 ap:℘ A:123 Ap:℘ Apn:<nil>}]
	// fmt.sprint:	{123 ℘ 123 ℘ <nil>}
	// redact.sprint:	{‹123› ‹℘› ‹123› ‹℘› ‹<nil>›}
	// redact.printer:	{‹123› ‹℘› ‹123› ‹℘› ‹<nil>›}
	//
	// "%+v" :: redact.mixedInts :: [{a:123 ap:℘ A:123 Ap:℘ Apn:<nil>}]
	// fmt.sprint:	{a:123 ap:℘ A:123 Ap:℘ Apn:<nil>}
	// redact.sprint:	{a:‹123› ap:‹℘› A:‹123› Ap:‹℘› Apn:‹<nil>›}
	// redact.printer:	{a:‹123› ap:‹℘› A:‹123› Ap:‹℘› Apn:‹<nil>›}
	//
	// "%v" :: []redact.mixedInts :: [[{a:123 ap:℘ A:123 Ap:℘ Apn:<nil>} {a:123 ap:℘ A:123 Ap:℘ Apn:<nil>}]]
	// fmt.sprint:	[{123 ℘ 123 ℘ <nil>} {123 ℘ 123 ℘ <nil>}]
	// redact.sprint:	[{‹123› ‹℘› ‹123› ‹℘› ‹<nil>›} {‹123› ‹℘› ‹123› ‹℘› ‹<nil>›}]
	// redact.printer:	[{‹123› ‹℘› ‹123› ‹℘› ‹<nil>›} {‹123› ‹℘› ‹123› ‹℘› ‹<nil>›}]
	//
	// "%+v" :: redact.mixedSafe :: [{s1:safe S1:safe s1p:℘ S1p:℘ S1pn:<nil>}]
	// fmt.sprint:	{s1:safe S1:safe s1p:℘ S1p:℘ S1pn:<nil>}
	// redact.sprint:	{s1:‹safe› S1:safe s1p:‹℘› S1p:℘ S1pn:<nil>}
	// redact.printer:	{s1:‹safe› S1:safe s1p:‹℘› S1p:℘ S1pn:<nil>}
	//
	// "%+v" :: redact.mixedSafeStringer :: [{s2:{s:safe} s2p:℘ S2:{s:safe} S2p:safe S2pn:<nil>}]
	// fmt.sprint:	{s2:{s:safe} s2p:℘ S2:{s:safe} S2p:safe S2pn:<nil>}
	// redact.sprint:	{s2:{s:‹safe›} s2p:‹℘› S2:{s:‹safe›} S2p:safe S2pn:<nil>}
	// redact.printer:	{s2:{s:‹safe›} s2p:‹℘› S2:{s:‹safe›} S2p:safe S2pn:<nil>}
	//
	// "%+v" :: redact.mixedRedactableString :: [{r:safe‹unsafe› rp:℘ R:safe‹unsafe› Rp:℘ Rpn:<nil>}]
	// fmt.sprint:	{r:safe‹unsafe› rp:℘ R:safe‹unsafe› Rp:℘ Rpn:<nil>}
	// redact.sprint:	{r:safe‹unsafe› rp:‹℘› R:safe‹unsafe› Rp:safe‹unsafe› Rpn:<nil>}
	// redact.printer:	{r:safe‹unsafe› rp:‹℘› R:safe‹unsafe› Rp:safe‹unsafe› Rpn:<nil>}
	//
	// "%+v" :: redact.mixedRedactableBytes :: [{r:[115 97 102 101 226 128 185 117 110 115 97 102 101 226 128 186] rp:℘ R:[115 97 102 101 226 128 185 117 110 115 97 102 101 226 128 186] Rp:℘ Rpn:<nil>}]
	// fmt.sprint:	{r:[115 97 102 101 226 128 185 117 110 115 97 102 101 226 128 186] rp:℘ R:[115 97 102 101 226 128 185 117 110 115 97 102 101 226 128 186] Rp:℘ Rpn:<nil>}
	// redact.sprint:	{r:safe‹unsafe› rp:‹℘› R:safe‹unsafe› Rp:safe‹unsafe› Rpn:<nil>}
	// redact.printer:	{r:safe‹unsafe› rp:‹℘› R:safe‹unsafe› Rp:safe‹unsafe› Rpn:<nil>}
}

var ptrRe = regexp.MustCompile(`0x[0-9a-f]{4,16}`)

func TestEscapeBytes(t *testing.T) {
	testCases := []struct {
		input    string
		expected string
	}{
		{"", "‹›"},
		{" ", "‹ ›"},
		{"abc", "‹abc›"},
		{"ab›‹c", "‹ab??c›"},
		{"abc\n\ncde", "‹abc›\n\n‹cde›"},
		{"\n abc ", "\n‹ abc ›"},
	}

	for _, tc := range testCases {
		input := []byte(tc.input)
		actual := EscapeBytes(input)
		actualS := string(actual)
		if actualS != tc.expected {
			t.Errorf("%q: expected %q, got %q", tc.input, tc.expected, actualS)
		}
	}
}

func TestPrinterSpaceOmission(t *testing.T) {
	// This test catches a regression introduced in v1.1.0.
	vals := []safeInt{1, 2, 3, 4, 5}
	s := Sprintfn(func(w SafePrinter) {
		t.Logf("printer type %T", w)
		for i, v := range vals {
			if i > 0 {
				w.SafeRune(' ')
			}
			if v%2 == 0 {
				w.SafeRune('e')
			} else {
				w.SafeRune('o')
			}
			w.Print(v)
		}
	})
	const expected = `o1 e2 o3 e4 o5`
	if s != expected {
		t.Errorf("expected %q, got %q", expected, s)
	}
}

type safeInt int

func (safeInt) SafeValue() {}

func TestHelperForErrorf(t *testing.T) {
	origErr := errors.New("small\nuniverse")
	s, e := HelperForErrorf("hello %s", origErr)
	if actual, expected := string(s), "hello ‹small›\n‹universe›"; actual != expected {
		t.Errorf("expected %q, got %q", expected, actual)
	}
	if e != nil {
		t.Errorf("expected no error, got %v", e)
	}

	s, e = HelperForErrorf("hello %w", origErr)
	if actual, expected := string(s), "hello ‹small›\n‹universe›"; actual != expected {
		t.Errorf("expected %q, got %q", expected, actual)
	}
	if e != origErr {
		t.Errorf("expected error %v, got %v (%T)", origErr, e, e)
	}
}

type complexObj struct {
	v string
}

type stringer struct{ s string }

var _ fmt.Stringer = (*stringer)(nil)

func (s *stringer) String() string { return s.s }

type safestringer struct{ s string }

var _ SafeValue = (*safestringer)(nil)
var _ fmt.Stringer = (*safestringer)(nil)

func (*safestringer) SafeValue()       {}
func (s *safestringer) String() string { return s.s }

type fmtformatter struct{ s string }

var _ fmt.Formatter = (*fmtformatter)(nil)

func (s *fmtformatter) Format(w fmt.State, _ rune) { fmt.Fprint(w, s.s) }

type safefmtformatter struct{ s string }

var _ SafeValue = (*safefmtformatter)(nil)
var _ fmt.Formatter = (*safefmtformatter)(nil)

func (*safefmtformatter) SafeValue()                   {}
func (s *safefmtformatter) Format(w fmt.State, _ rune) { fmt.Fprint(w, s.s) }

type panicObj1 struct{ s string }

var _ fmt.Stringer = (*panicObj1)(nil)

func (p *panicObj1) String() string { panic(p.s) }

type panicObj2 struct{ s string }

var _ fmt.Formatter = (*panicObj2)(nil)

func (p *panicObj2) Format(fmt.State, rune) { panic(p.s) }

type safepanicObj1 struct{ s string }

var _ SafeValue = (*safepanicObj1)(nil)
var _ fmt.Stringer = (*safepanicObj1)(nil)

func (*safepanicObj1) SafeValue()       {}
func (p *safepanicObj1) String() string { panic(p.s) }

type safepanicObj2 struct{ s string }

var _ SafeValue = (*safepanicObj2)(nil)
var _ fmt.Formatter = (*safepanicObj2)(nil)

func (*safepanicObj2) SafeValue()               {}
func (p *safepanicObj2) Format(fmt.State, rune) { panic(p.s) }

type panicObj3 struct{ s string }

var _ SafeFormatter = (*panicObj3)(nil)

func (p *panicObj3) SafeFormat(SafePrinter, rune) { panic(p.s) }

type safemsg struct {
	s string
}

type panicObj4 struct{ unused string }

var _ SafeMessager = (*panicObj4)(nil)

func (p *panicObj4) SafeMessage() string { panic("woo") }

var _ SafeMessager = (*safemsg)(nil)

func (p *safemsg) SafeMessage() string { return p.s }

// TestFormatRedirect checks that the count return from the
// (*escapeWriter).Write() method is correct when there are newline
// characters.
func TestFormatRedirect(t *testing.T) {
	v := &fmter{}
	if expected, actual := "‹hello›\n‹world›", string(Sprintf("%v", v)); expected != actual {
		t.Errorf("expected %q, got %q", expected, actual)
	}
	if expected, actual := "‹hello›\n‹world›", string(Sprintf("%+v", v)); expected != actual {
		t.Errorf("expected %q, got %q", expected, actual)
	}
}

type fmter struct{}

// Format implements the fmt.Formatter interface.
func (ef *fmter) Format(s fmt.State, verb rune) {
	var buf bytes.Buffer
	buf.WriteString("hello\nworld")
	_, _ = buf.WriteTo(s)
}

type safeNil struct {
	unused int
}

func (s *safeNil) SafeFormat(p SafePrinter, _ rune) {
	p.Printf("hello %v", "world")
}

type mixedInts struct {
	a   int
	ap  *int
	A   int
	Ap  *int
	Apn *int
}

type mixedSafe struct {
	s1   safe
	S1   safe
	s1p  *safe
	S1p  *safe
	S1pn *safe
}

type mixedSafeStringer struct {
	s2   safestringer
	s2p  *safestringer
	S2   safestringer
	S2p  *safestringer
	S2pn *safestringer
}

type mixedRedactableString struct {
	r   RedactableString
	rp  *RedactableString
	R   RedactableString
	Rp  *RedactableString
	Rpn *RedactableString
}

type mixedRedactableBytes struct {
	r   RedactableBytes
	rp  *RedactableBytes
	R   RedactableBytes
	Rp  *RedactableBytes
	Rpn *RedactableBytes
}

func makeMixedInts() mixedInts {
	i := 123
	return mixedInts{
		a:  i,
		ap: &i,
		A:  i,
		Ap: &i,
	}
}
func makeMixedSafe() mixedSafe {
	s := safe("safe")
	return mixedSafe{
		s1:  s,
		s1p: &s,
		S1:  s,
		S1p: &s,
	}
}
func makeMixedSafeStringer() mixedSafeStringer {
	ss := safestringer{"safe"}
	return mixedSafeStringer{
		s2:  ss,
		s2p: &ss,
		S2:  ss,
		S2p: &ss,
	}
}
func makeMixedRedactableString() mixedRedactableString {
	r := RedactableString("safe‹unsafe›")
	return mixedRedactableString{
		r:  r,
		rp: &r,
		R:  r,
		Rp: &r,
	}
}

func makeMixedRedactableBytes() mixedRedactableBytes {
	r := RedactableBytes("safe‹unsafe›")
	return mixedRedactableBytes{
		r:  r,
		rp: &r,
		R:  r,
		Rp: &r,
	}
}
