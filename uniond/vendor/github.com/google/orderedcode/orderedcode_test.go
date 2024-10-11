// Copyright 2015 Google Inc. All Rights Reserved.
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

package orderedcode

import (
	"bytes"
	"fmt"
	"math"
	"math/rand"
	"reflect"
	"testing"
	"time"
)

// See http://en.wikipedia.org/wiki/IEEE_754-1985 for the bit-level float64 format.
var neg0 = math.Float64frombits(0x8000000000000000)

func init() {
	// IEEE 754 states that negative zero == positive zero.
	if neg0 != 0 {
		panic("neg0 != 0")
	}
}

var testCases = []struct {
	val interface{}
	enc string
}{
	// Strings.
	{"", "\x00\x01"},
	{"\x00", "\x00\xff\x00\x01"},
	{"\x00\x00", "\x00\xff\x00\xff\x00\x01"},
	{"\x01", "\x01\x00\x01"},
	{"foo", "foo\x00\x01"},
	{"foo\x00", "foo\x00\xff\x00\x01"},
	{"foo\x00\x01", "foo\x00\xff\x01\x00\x01"},
	{"foo\x01", "foo\x01\x00\x01"},
	{"foo\x01\x00", "foo\x01\x00\xff\x00\x01"},
	{"foo\xfe", "foo\xfe\x00\x01"},
	{"foo\xff", "foo\xff\x00\x00\x01"},
	{"\xff", "\xff\x00\x00\x01"},
	{"\xff\xff", "\xff\x00\xff\x00\x00\x01"},
	// Infinity.
	{Infinity, "\xff\xff"},
	// Float64s.
	{float64(math.Inf(-1)), "\x00\x3f\x80\x10\x00\x00\x00\x00\x00\x00"},
	{float64(-math.MaxFloat64), "\x00\x3f\x80\x10\x00\x00\x00\x00\x00\x01"},
	{float64(-2.71828), "\x00\x3f\xbf\xfa\x40\xf6\x6a\x55\x08\x70"},
	{float64(-1.0), "\x00\x40\x10\x00\x00\x00\x00\x00\x00"},
	{float64(-math.SmallestNonzeroFloat64), "\x7f"},
	{neg0, "\x80"},
	{float64(0), "\x80"},
	{float64(+math.SmallestNonzeroFloat64), "\x81"},
	{float64(+0.333333333), "\xff\xbf\xd5\x55\x55\x54\xf9\xb5\x16"},
	{float64(+1.0), "\xff\xbf\xf0\x00\x00\x00\x00\x00\x00"},
	{float64(+1.41421), "\xff\xbf\xf6\xa0\x9a\xaa\x3a\xd1\x8d"},
	{float64(+1.5), "\xff\xbf\xf8\x00\x00\x00\x00\x00\x00"},
	{float64(+2.0), "\xff\xc0\x40\x00\x00\x00\x00\x00\x00\x00"},
	{float64(+3.14159), "\xff\xc0\x40\x09\x21\xf9\xf0\x1b\x86\x6e"},
	{float64(+6.022e23), "\xff\xc0\x44\xdf\xe1\x54\xf4\x57\xea\x13"},
	{float64(+math.MaxFloat64), "\xff\xc0\x7f\xef\xff\xff\xff\xff\xff\xff"},
	{float64(math.Inf(+1)), "\xff\xc0\x7f\xf0\x00\x00\x00\x00\x00\x00"},
	// Int64s (values near zero).
	{int64(-8193), "\x1f\xdf\xff"}, // 00011111 11011111 11111111
	{int64(-8192), "\x20\x00"},     // 00100000 00000000
	{int64(-4097), "\x2f\xff"},     // 00101111 11111111
	{int64(-257), "\x3e\xff"},      // 00111110 11111111
	{int64(-256), "\x3f\x00"},      // 00111111 00000000
	{int64(-66), "\x3f\xbe"},       // 00111111 10111110
	{int64(-65), "\x3f\xbf"},       // 00111111 10111111
	{int64(-64), "\x40"},           // 01000000
	{int64(-63), "\x41"},           // 01000001
	{int64(-3), "\x7d"},            // 01111101
	{int64(-2), "\x7e"},            // 01111110
	{int64(-1), "\x7f"},            // 01111111
	{int64(+0), "\x80"},            // 10000000
	{int64(+1), "\x81"},            // 10000001
	{int64(+2), "\x82"},            // 10000010
	{int64(+62), "\xbe"},           // 10111110
	{int64(+63), "\xbf"},           // 10111111
	{int64(+64), "\xc0\x40"},       // 11000000 01000000
	{int64(+65), "\xc0\x41"},       // 11000000 01000001
	{int64(+255), "\xc0\xff"},      // 11000000 11111111
	{int64(+256), "\xc1\x00"},      // 11000001 00000000
	{int64(+4096), "\xd0\x00"},     // 11010000 00000000
	{int64(+8191), "\xdf\xff"},     // 11011111 11111111
	{int64(+8192), "\xe0\x20\x00"}, // 11100000 00100000 00000000
	// Int64s.
	{int64(-0x800), "\x38\x00"},
	{int64(0x424242), "\xf0\x42\x42\x42"},
	{int64(0x23), "\xa3"},
	{int64(0x10e), "\xc1\x0e"},
	{int64(-0x10f), "\x3e\xf1"},
	{int64(0x020b0c0d), "\xf2\x0b\x0c\x0d"},
	{int64(0x0a0b0c0d), "\xf8\x0a\x0b\x0c\x0d"},
	{int64(0x0102030405060708), "\xff\x81\x02\x03\x04\x05\x06\x07\x08"},
	// Int64s (edge cases).
	{int64(-1<<63 - 0), "\x00\x3f\x80\x00\x00\x00\x00\x00\x00\x00"}, // 00000000 00111111 10000000 0x00 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(-1<<62 - 1), "\x00\x3f\xbf\xff\xff\xff\xff\xff\xff\xff"}, // 00000000 00111111 10111111 0xff 0xff 0xff 0xff 0xff 0xff 0xff
	{int64(-1<<62 - 0), "\x00\x40\x00\x00\x00\x00\x00\x00\x00"},     // 00000000 01000000 00000000 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(-1<<55 - 1), "\x00\x7f\x7f\xff\xff\xff\xff\xff\xff"},     // 00000000 01111111 01111111 0xff 0xff 0xff 0xff 0xff 0xff
	{int64(-1<<55 - 0), "\x00\x80\x00\x00\x00\x00\x00\x00"},         // 00000000 10000000 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(-1<<48 - 1), "\x00\xfe\xff\xff\xff\xff\xff\xff"},         // 00000000 11111110 0xff 0xff 0xff 0xff 0xff 0xff
	{int64(-1<<48 - 0), "\x01\x00\x00\x00\x00\x00\x00"},             // 00000001 00000000 0x00 0x00 0x00 0x00 0x00
	{int64(-1<<41 - 1), "\x01\xfd\xff\xff\xff\xff\xff"},             // 00000001 11111101 0xff 0xff 0xff 0xff 0xff
	{int64(-1<<41 - 0), "\x02\x00\x00\x00\x00\x00"},                 // 00000010 00000000 0x00 0x00 0x00 0x00
	{int64(-1<<34 - 1), "\x03\xfb\xff\xff\xff\xff"},                 // 00000011 11111011 0xff 0xff 0xff 0xff
	{int64(-1<<34 - 0), "\x04\x00\x00\x00\x00"},                     // 00000100 00000000 0x00 0x00 0x00
	{int64(-1<<27 - 1), "\x07\xf7\xff\xff\xff"},                     // 00000111 11110111 0xff 0xff 0xff
	{int64(-1<<27 - 0), "\x08\x00\x00\x00"},                         // 00001000 00000000 0x00 0x00
	{int64(-1<<20 - 1), "\x0f\xef\xff\xff"},                         // 00001111 11101111 0xff 0xff
	{int64(-1<<20 - 0), "\x10\x00\x00"},                             // 00010000 00000000 0x00
	{int64(-1<<13 - 1), "\x1f\xdf\xff"},                             // 00011111 11011111 0xff
	{int64(-1<<13 - 0), "\x20\x00"},                                 // 00100000 00000000
	{int64(-1<<6 - 1), "\x3f\xbf"},                                  // 00111111 10111111
	{int64(-1<<6 - 0), "\x40"},                                      // 01000000
	{int64(+1<<6 - 1), "\xbf"},                                      // 10111111
	{int64(+1<<6 - 0), "\xc0\x40"},                                  // 11000000 01000000
	{int64(+1<<13 - 1), "\xdf\xff"},                                 // 11011111 11111111
	{int64(+1<<13 - 0), "\xe0\x20\x00"},                             // 11100000 00100000 0x00
	{int64(+1<<20 - 1), "\xef\xff\xff"},                             // 11101111 11111111 0xff
	{int64(+1<<20 - 0), "\xf0\x10\x00\x00"},                         // 11110000 00010000 0x00 0x00
	{int64(+1<<27 - 1), "\xf7\xff\xff\xff"},                         // 11110111 11111111 0xff 0xff
	{int64(+1<<27 - 0), "\xf8\x08\x00\x00\x00"},                     // 11111000 00001000 0x00 0x00 0x00
	{int64(+1<<34 - 1), "\xfb\xff\xff\xff\xff"},                     // 11111011 11111111 0xff 0xff 0xff
	{int64(+1<<34 - 0), "\xfc\x04\x00\x00\x00\x00"},                 // 11111100 00000100 0x00 0x00 0x00 0x00
	{int64(+1<<41 - 1), "\xfd\xff\xff\xff\xff\xff"},                 // 11111101 11111111 0xff 0xff 0xff 0xff
	{int64(+1<<41 - 0), "\xfe\x02\x00\x00\x00\x00\x00"},             // 11111110 00000010 0x00 0x00 0x00 0x00 0x00
	{int64(+1<<48 - 1), "\xfe\xff\xff\xff\xff\xff\xff"},             // 11111110 11111111 0xff 0xff 0xff 0xff 0xff
	{int64(+1<<48 - 0), "\xff\x01\x00\x00\x00\x00\x00\x00"},         // 11111111 00000001 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(+1<<55 - 1), "\xff\x7f\xff\xff\xff\xff\xff\xff"},         // 11111111 01111111 0xff 0xff 0xff 0xff 0xff 0xff
	{int64(+1<<55 - 0), "\xff\x80\x80\x00\x00\x00\x00\x00\x00"},     // 11111111 10000000 10000000 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(+1<<62 - 1), "\xff\xbf\xff\xff\xff\xff\xff\xff\xff"},     // 11111111 10111111 11111111 0xff 0xff 0xff 0xff 0xff 0xff
	{int64(+1<<62 - 0), "\xff\xc0\x40\x00\x00\x00\x00\x00\x00\x00"}, // 11111111 11000000 01000000 0x00 0x00 0x00 0x00 0x00 0x00 0x00
	{int64(+1<<63 - 1), "\xff\xc0\x7f\xff\xff\xff\xff\xff\xff\xff"}, // 11111111 11000000 01111111 0xff 0xff 0xff 0xff 0xff 0xff 0xff
	// Uint64s.
	{uint64(0), "\x00"},
	{uint64(1), "\x01\x01"},
	{uint64(255), "\x01\xff"},
	{uint64(256), "\x02\x01\x00"},
	{uint64(1025), "\x02\x04\x01"},
	{uint64(0x0a0b0c0d), "\x04\x0a\x0b\x0c\x0d"},
	{uint64(0x0102030405060708), "\x08\x01\x02\x03\x04\x05\x06\x07\x08"},
	{uint64(1<<64 - 1), "\x08\xff\xff\xff\xff\xff\xff\xff\xff"},
}

func invertString(s string) string {
	b := []byte(s)
	for i := range b {
		b[i] = ^b[i]
	}
	return string(b)
}

// expect checks that decoding enc with direction dir and val's type yields
// val and exhausts the input.
func expect(enc string, dir byte, val interface{}) error {
	dst := reflect.New(reflect.TypeOf(val))
	item := dst.Interface()
	if dir == decreasing {
		item = Decr(item)
	}
	enc, err := Parse(enc, item)
	if err != nil {
		return fmt.Errorf("val=%v of type %T: got error %v", val, val, err)
	}
	if got := dst.Elem().Interface(); !reflect.DeepEqual(got, val) {
		return fmt.Errorf("val=%v of type %T: got %v, want %v", val, val, got, val)
	}
	if len(enc) != 0 {
		return fmt.Errorf("code was not exhausted, remainder has length %d", len(enc))
	}
	return nil
}

func TestIndividualEncodings(t *testing.T) {
	for _, tc := range testCases {
		// Test in-increasing-order.
		buf0, err := Append(nil, tc.val)
		if err != nil {
			t.Errorf("append incr: val=%v of type %T: %v", tc.val, tc.val, err)
			continue
		}
		enc0 := string(buf0)
		if enc0 != tc.enc {
			t.Errorf("append incr: val=%v of type %T:\ngot   % x\nwant  % x", tc.val, tc.val, enc0, tc.enc)
			continue
		}
		if err := expect(enc0, increasing, tc.val); err != nil {
			t.Errorf("parse incr: %v", err)
		}

		// Test in-decreasing-order.
		buf1, err := Append(nil, Decr(tc.val))
		if err != nil {
			t.Errorf("append decr: val=%v of type %T: %v", tc.val, tc.val, err)
			continue
		}
		enc1 := string(buf1)
		// The in-decreasing-order encoding should be the bitwise-not of the regular encoding.
		if enc1 != invertString(tc.enc) {
			t.Errorf("append decr: val=%v of type %T:\ngot   % x\nwant  % x", tc.val, tc.val, enc1, invertString(tc.enc))
			continue
		}
		if err := expect(enc1, decreasing, tc.val); err != nil {
			t.Errorf("parse decr: %v", err)
		}
	}
}

func TestConcatenation(t *testing.T) {
	// The encoding of multiple values should equal the concatenation of
	// the individual encodings.
	var (
		items []interface{}
		buf1  bytes.Buffer
	)
	for _, tc := range testCases {
		items = append(items, tc.val)
		buf1.WriteString(tc.enc)
	}
	buf0, err := Append(nil, items...)
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	if s0, s1 := string(buf0), buf1.String(); s0 != s1 {
		t.Errorf("\ngot  %q\nwant %q", s0, s1)
	}
}

func TestNaN(t *testing.T) {
	buf, err := Append(nil, math.NaN())
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	var f float64
	_, err = Parse(string(buf), &f)
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if !math.IsNaN(f) {
		t.Errorf("got %v want NaN", f)
	}
}

func TestTrailingString(t *testing.T) {
	testCases := []string{
		"",
		"\x00",
		"\x00\x01",
		"a",
		"bcd",
		"foo\x00",
		"foo\x00bar",
		"foo\x00bar\x00",
		"\xff",
		"\xff\x00",
		"\xff\xfe",
		"\xff\xff",
	}
	for _, decr := range []bool{false, true} {
		for _, tc := range testCases {
			src := interface{}(TrailingString(tc))
			if decr {
				src = Decr(src)
			}
			buf, err := Append(nil, src)
			if err != nil {
				t.Errorf("decr=%v, tc=%q: append: %v", decr, tc, err)
				continue
			}

			enc, encWant := string(buf), tc
			if decr {
				encWant = invertString(encWant)
			}
			if enc != encWant {
				t.Errorf("decr=%v, tc=%q: append: got %q want %q", decr, tc, enc, encWant)
				continue
			}

			var x TrailingString
			dst := interface{}(&x)
			if decr {
				dst = Decr(dst)
			}
			rem, err := Parse(enc, dst)
			if err != nil {
				t.Errorf("decr=%v, tc=%q: parse: %v", decr, tc, err)
				continue
			}
			if rem != "" {
				t.Errorf(`decr=%v, tc=%q: parse: got remainder %q want ""`, decr, tc, rem)
				continue
			}
			if string(x) != tc {
				t.Errorf("decr=%v, tc=%q: parse: got %q want %q", decr, tc, x, tc)
				continue
			}
		}
	}
}

func TestIncrDecr(t *testing.T) {
	buf, err := Append(nil,
		uint64(0),
		Decr(uint64(1)),
		uint64(2),
		Decr(uint64(516)),
		uint64(517),
		Decr(uint64(0)),
	)
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	got := string(buf)
	want := "\x00" + "\xfe\xfe" + "\x01\x02" + "\xfd\xfd\xfb" + "\x02\x02\x05" + "\xff"
	if got != want {
		t.Errorf("\ngot  %q\nwant %q", got, want)
	}
}

func TestRoundTrip(t *testing.T) {
	key, err := Append(nil, "foo", Decr("bar"))
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	var s1, s2 string
	_, err = Parse(string(key), &s1, Decr(&s2))
	if err != nil {
		t.Fatalf("parse: %v", err)
	}
	if s1 != "foo" || s2 != "bar" {
		t.Fatalf("got s1=%q s2=%q, want s1=%q s2=%q\n", s1, s2, "foo", "bar")
	}
}

func TestRandomStrings(t *testing.T) {
	const maxStrLen = 16
	seed := time.Now().UnixNano()
	t.Logf("random seed = %v", seed)
	// generator returns a func() string that is an infinite iterator of strings.
	// Calling that func() string returns the next string and advances the iterator.
	// Calling generator twice results in two independent iterators that yield
	// the same pseudo-random sequence of strings.
	generator := func() func() string {
		r := rand.New(rand.NewSource(seed))
		return func() string {
			b := make([]byte, r.Intn(maxStrLen))
			for i := range b {
				b[i] = byte(r.Intn(256))
			}
			return string(b)
		}
	}
	const n = 1e5
	g0, g1, items := generator(), generator(), make([]interface{}, n)
	for i := 0; i < n; i++ {
		items[i] = g0()
	}
	buf, err := Append(nil, items...)
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	enc := string(buf)
	if len(enc) < n*maxStrLen/2 {
		// On average, each of the n strings has length maxStrLen/2 before encoding.
		// The encoded length is greater, due to escaping and the terminator mark.
		t.Fatalf("enc is too short, length=%d", len(enc))
	}
	for i := 0; i < n; i++ {
		var got string
		enc, err = Parse(enc, &got)
		if err != nil {
			t.Fatalf("i=%d: %v", i, err)
		}
		if want := g1(); got != want {
			t.Fatalf("i=%d: got %q, want %q", i, got, want)
		}
	}
	if len(enc) != 0 {
		t.Errorf("code was not exhausted, remainder has length %d", len(enc))
	}
}

func TestRandomInt64s(t *testing.T) {
	seed := time.Now().UnixNano()
	t.Logf("random seed = %v", seed)
	generator := func() func() int64 {
		r := rand.New(rand.NewSource(seed))
		return func() int64 {
			x := int64(r.Uint32())
			y := int64(r.Uint32())
			return x<<32 | y
		}
	}
	const n = 1e5
	g0, g1, items := generator(), generator(), make([]interface{}, n)
	for i := 0; i < n; i++ {
		items[i] = g0()
	}
	buf, err := Append(nil, items...)
	if err != nil {
		t.Fatalf("append: %v", err)
	}
	enc := string(buf)
	for i := 0; i < n; i++ {
		var got int64
		enc, err = Parse(enc, &got)
		if err != nil {
			t.Fatalf("i=%d: %v", i, err)
		}
		if want := g1(); got != want {
			t.Fatalf("i=%d: got %d, want %d", i, got, want)
		}
	}
	if len(enc) != 0 {
		t.Errorf("code was not exhausted, remainder has length %d", len(enc))
	}
}

func TestStringOrInfinity(t *testing.T) {
	check := func(got StringOrInfinity, want interface{}) error {
		if got.String != "" && got.Infinity {
			return fmt.Errorf("StringOrInfinty has non-zero String and non-zero Infinity: %v", got)
		}
		switch v := want.(type) {
		case string:
			if got.String != v {
				return fmt.Errorf("got %q, want %q", got.String, v)
			}
		case struct{}:
			if !got.Infinity {
				return fmt.Errorf("got not-infinity, want infinity")
			}
		default:
			panic("unreachable")
		}
		return nil
	}

	vals := []interface{}{
		"foo",
		"bar",
		Infinity,
		"",
		"\x00",
		Infinity,
		Infinity,
		"\xff",
		"AB\x00\x01\x02MN\xfd\xfe\xffYZ",
	}
	buf, err := Append(nil, vals...)
	if err != nil {
		t.Fatalf("append: %v", err)
	}

	// Test parsing one at a time.
	enc := string(buf)
	for i, val := range vals {
		var x StringOrInfinity
		enc, err = Parse(enc, &x)
		if err != nil {
			t.Fatalf("parse one: i=%d: Parse: %v", i, err)
		}
		if err := check(x, val); err != nil {
			t.Fatalf("parse one: i=%d: %v", i, err)
		}
	}
	if len(enc) != 0 {
		t.Errorf("parse one: code was not exhausted, remainder=%q", enc)
	}

	// Test parsing many at a time.
	enc = string(buf)
	got := make([]interface{}, len(vals))
	for i := range got {
		got[i] = new(StringOrInfinity)
	}
	enc, err = Parse(enc, got...)
	if err != nil {
		t.Fatalf("parse many: Parse: %v", err)
	}
	for i, p := range got {
		if err := check(*p.(*StringOrInfinity), vals[i]); err != nil {
			t.Fatalf("parse many: i=%d: %v", i, err)
		}
	}
	if len(enc) != 0 {
		t.Errorf("parse many: code was not exhausted, remainder=%q", enc)
	}
}

func TestCorruptStringOrInfinity(t *testing.T) {
	var dst0, dst1, dst2 StringOrInfinity

	// Parse one StringOrInfinity value.
	input := "\x00" // The "\x00" is neither a valid string nor a valid infinity.
	if _, err := Parse(input, &dst0); err != errCorrupt {
		t.Errorf("parse one: got %v, want errCorrupt", err)
	}

	// Parse many StringOrInfinity values.
	input = "foo\x00\x01" + "\xff\xff" + "\x00"
	if _, err := Parse(input, &dst0, &dst1, &dst2); err != errCorrupt {
		t.Errorf("parse many: got %v, want errCorrupt", err)
	}
}

func TestCorrupt(t *testing.T) {
	testCases := []struct {
		dst    interface{}
		inputs []string
	}{
		{
			new(string),
			[]string{
				"",
				"\x00", // A valid uint64, but not a valid string.
				"\x00\x00",
				"\x00\x00\x01",
				"\x00\x02",
				"abc",
				"abc\xff\xff",
				"foo\x00",
				"\xa3", // A valid float64 or int64, but not a valid string.
				"\xff",
				"\xff\x00",
				"\xff\xfe",
				"\xff\xff", // A valid infinity, but not a valid string.
			},
		},
		{
			&Infinity,
			[]string{
				"",
				"\x00", // A valid uint64, but not a valid infinity.
				"abc",
				"foo\x00\x01", // A valid string, but not a valid infinity.
				"\xa3",        // A valid float64 or int64, but not a valid infinity.
				"\xff",
				"\xff\x00",
				"\xff\xfe",
			},
		},
		{
			new(float64),
			[]string{
				"",
				"\x00", // A valid uint64, but not a valid float64.
				"\x00\x00",
				"\x00\x00abcdefghijklmnopqrst",
				"\x00\x01", // A valid string, but not a valid float64.
				"\xc0",
				"\xf0\x00",
				"\xff\xffabcdefghijklmnopqrst",
				"\xff\xff", // A valid infinity, but not a valid float64.
			},
		},
		{
			new(int64),
			[]string{
				"",
				"\x00", // A valid uint64, but not a valid int64.
				"\x00\x00",
				"\x00\x00abcdefghijklmnopqrst",
				"\x00\x01", // A valid string, but not a valid int64.
				"\xc0",
				"\xf0\x00",
				"\xff\xffabcdefghijklmnopqrst",
				"\xff\xff", // A valid infinity, but not a valid int64.
			},
		},
		{
			new(uint64),
			[]string{
				"",
				"\x01",
				"\x08abcd",
				"\x09abcdefghijklmnopqrst",
				"abc",
				"abc\xff\xff",
				"foo\x00\x01", // A valid string, but not a valid uint64.
				"\xa3",        // A valid float64 or int64, but not a valid uint64.
				"\xff",
				"\xff\x00",
				"\xff\xfe",
				"\xff\xff", // A valid infinity, but not a valid uint64.
			},
		},
	}
	for _, tc := range testCases {
		for _, input := range tc.inputs {
			if _, err := Parse(input, tc.dst); err != errCorrupt {
				t.Errorf("dst has type %T, input=%q: got %v want errCorrupt", tc.dst, input, err)
			}
		}
	}
}
