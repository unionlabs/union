// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package attribute_test

import (
	"reflect"
	"regexp"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
)

type testCase struct {
	kvs []attribute.KeyValue

	keyRe *regexp.Regexp

	encoding string
	fullEnc  string
}

func expect(enc string, kvs ...attribute.KeyValue) testCase {
	return testCase{
		kvs:      kvs,
		encoding: enc,
	}
}

func expectFiltered(enc, filter, fullEnc string, kvs ...attribute.KeyValue) testCase {
	return testCase{
		kvs:      kvs,
		keyRe:    regexp.MustCompile(filter),
		encoding: enc,
		fullEnc:  fullEnc,
	}
}

func TestSetDedup(t *testing.T) {
	cases := []testCase{
		expect("A=B", attribute.String("A", "2"), attribute.String("A", "B")),
		expect("A=B", attribute.String("A", "2"), attribute.Int("A", 1), attribute.String("A", "B")),
		expect("A=B", attribute.String("A", "B"), attribute.String("A", "C"), attribute.String("A", "D"), attribute.String("A", "B")),

		expect("A=B,C=D", attribute.String("A", "1"), attribute.String("C", "D"), attribute.String("A", "B")),
		expect("A=B,C=D", attribute.String("A", "2"), attribute.String("A", "B"), attribute.String("C", "D")),
		expect("A=B,C=D", attribute.Float64("C", 1.2), attribute.String("A", "2"), attribute.String("A", "B"), attribute.String("C", "D")),
		expect("A=B,C=D", attribute.String("C", "D"), attribute.String("A", "B"), attribute.String("A", "C"), attribute.String("A", "D"), attribute.String("A", "B")),
		expect("A=B,C=D", attribute.String("A", "B"), attribute.String("C", "D"), attribute.String("A", "C"), attribute.String("A", "D"), attribute.String("A", "B")),
		expect("A=B,C=D", attribute.String("A", "B"), attribute.String("A", "C"), attribute.String("A", "D"), attribute.String("A", "B"), attribute.String("C", "D")),
	}
	enc := attribute.DefaultEncoder()

	s2d := map[string][]attribute.Distinct{}
	d2s := map[attribute.Distinct][]string{}

	for _, tc := range cases {
		cpy := make([]attribute.KeyValue, len(tc.kvs))
		copy(cpy, tc.kvs)
		sl := attribute.NewSet(cpy...)

		// Ensure that the input was reordered but no elements went missing.
		require.ElementsMatch(t, tc.kvs, cpy)

		str := sl.Encoded(enc)
		equ := sl.Equivalent()

		s2d[str] = append(s2d[str], equ)
		d2s[equ] = append(d2s[equ], str)

		require.Equal(t, tc.encoding, str)
	}

	for s, d := range s2d {
		// No other Distinct values are equal to this.
		for s2, d2 := range s2d {
			if s2 == s {
				continue
			}
			for _, elt := range d {
				for _, otherDistinct := range d2 {
					require.NotEqual(t, otherDistinct, elt)
				}
			}
		}
		for _, strings := range d2s {
			if strings[0] == s {
				continue
			}
			for _, otherString := range strings {
				require.NotEqual(t, otherString, s)
			}
		}
	}

	for d, s := range d2s {
		// No other Distinct values are equal to this.
		for d2, s2 := range d2s {
			if d2 == d {
				continue
			}
			for _, elt := range s {
				for _, otherDistinct := range s2 {
					require.NotEqual(t, otherDistinct, elt)
				}
			}
		}
		for _, distincts := range s2d {
			if distincts[0] == d {
				continue
			}
			for _, otherDistinct := range distincts {
				require.NotEqual(t, otherDistinct, d)
			}
		}
	}
}

func TestFiltering(t *testing.T) {
	a := attribute.String("A", "a")
	b := attribute.String("B", "b")
	c := attribute.String("C", "c")

	tests := []struct {
		name       string
		in         []attribute.KeyValue
		filter     attribute.Filter
		kept, drop []attribute.KeyValue
	}{
		{
			name:   "A",
			in:     []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool { return kv.Key == "A" },
			kept:   []attribute.KeyValue{a},
			drop:   []attribute.KeyValue{b, c},
		},
		{
			name:   "B",
			in:     []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool { return kv.Key == "B" },
			kept:   []attribute.KeyValue{b},
			drop:   []attribute.KeyValue{a, c},
		},
		{
			name:   "C",
			in:     []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool { return kv.Key == "C" },
			kept:   []attribute.KeyValue{c},
			drop:   []attribute.KeyValue{a, b},
		},
		{
			name: "A||B",
			in:   []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool {
				return kv.Key == "A" || kv.Key == "B"
			},
			kept: []attribute.KeyValue{a, b},
			drop: []attribute.KeyValue{c},
		},
		{
			name: "B||C",
			in:   []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool {
				return kv.Key == "B" || kv.Key == "C"
			},
			kept: []attribute.KeyValue{b, c},
			drop: []attribute.KeyValue{a},
		},
		{
			name: "A||C",
			in:   []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool {
				return kv.Key == "A" || kv.Key == "C"
			},
			kept: []attribute.KeyValue{a, c},
			drop: []attribute.KeyValue{b},
		},
		{
			name:   "None",
			in:     []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool { return false },
			kept:   nil,
			drop:   []attribute.KeyValue{a, b, c},
		},
		{
			name:   "All",
			in:     []attribute.KeyValue{a, b, c},
			filter: func(kv attribute.KeyValue) bool { return true },
			kept:   []attribute.KeyValue{a, b, c},
			drop:   nil,
		},
		{
			name:   "Empty",
			in:     []attribute.KeyValue{},
			filter: func(kv attribute.KeyValue) bool { return true },
			kept:   nil,
			drop:   nil,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			t.Run("NewSetWithFiltered", func(t *testing.T) {
				fltr, drop := attribute.NewSetWithFiltered(test.in, test.filter)
				assert.Equal(t, test.kept, fltr.ToSlice(), "filtered")
				assert.ElementsMatch(t, test.drop, drop, "dropped")
			})

			t.Run("Set.Filter", func(t *testing.T) {
				s := attribute.NewSet(test.in...)
				fltr, drop := s.Filter(test.filter)
				assert.Equal(t, test.kept, fltr.ToSlice(), "filtered")
				assert.ElementsMatch(t, test.drop, drop, "dropped")
			})
		})
	}
}

func TestUniqueness(t *testing.T) {
	short := []attribute.KeyValue{
		attribute.String("A", "0"),
		attribute.String("B", "2"),
		attribute.String("A", "1"),
	}
	long := []attribute.KeyValue{
		attribute.String("B", "2"),
		attribute.String("C", "5"),
		attribute.String("B", "2"),
		attribute.String("C", "1"),
		attribute.String("A", "4"),
		attribute.String("C", "3"),
		attribute.String("A", "1"),
	}
	cases := []testCase{
		expectFiltered("A=1", "^A$", "B=2", short...),
		expectFiltered("B=2", "^B$", "A=1", short...),
		expectFiltered("A=1,B=2", "^A|B$", "", short...),
		expectFiltered("", "^C", "A=1,B=2", short...),

		expectFiltered("A=1,C=3", "A|C", "B=2", long...),
		expectFiltered("B=2,C=3", "C|B", "A=1", long...),
		expectFiltered("C=3", "C", "A=1,B=2", long...),
		expectFiltered("", "D", "A=1,B=2,C=3", long...),
	}
	enc := attribute.DefaultEncoder()

	for _, tc := range cases {
		cpy := make([]attribute.KeyValue, len(tc.kvs))
		copy(cpy, tc.kvs)
		distinct, uniq := attribute.NewSetWithFiltered(cpy, func(attr attribute.KeyValue) bool {
			return tc.keyRe.MatchString(string(attr.Key))
		})

		full := attribute.NewSet(uniq...)

		require.Equal(t, tc.encoding, distinct.Encoded(enc))
		require.Equal(t, tc.fullEnc, full.Encoded(enc))
	}
}

func TestLookup(t *testing.T) {
	set := attribute.NewSet(attribute.Int("C", 3), attribute.Int("A", 1), attribute.Int("B", 2))

	value, has := set.Value("C")
	require.True(t, has)
	require.Equal(t, int64(3), value.AsInt64())

	value, has = set.Value("B")
	require.True(t, has)
	require.Equal(t, int64(2), value.AsInt64())

	value, has = set.Value("A")
	require.True(t, has)
	require.Equal(t, int64(1), value.AsInt64())

	_, has = set.Value("D")
	require.False(t, has)
}

func TestZeroSetExportedMethodsNoPanic(t *testing.T) {
	rType := reflect.TypeOf((*attribute.Set)(nil))
	rVal := reflect.ValueOf(&attribute.Set{})
	for n := 0; n < rType.NumMethod(); n++ {
		mType := rType.Method(n)
		if !mType.IsExported() {
			t.Logf("ignoring unexported %s", mType.Name)
			continue
		}
		t.Run(mType.Name, func(t *testing.T) {
			m := rVal.MethodByName(mType.Name)
			if !m.IsValid() {
				t.Errorf("unknown method: %s", mType.Name)
			}
			assert.NotPanics(t, func() { _ = m.Call(args(mType)) })
		})
	}
}

func args(m reflect.Method) []reflect.Value {
	numIn := m.Type.NumIn() - 1 // Do not include the receiver arg.
	if numIn <= 0 {
		return nil
	}
	if m.Type.IsVariadic() {
		numIn--
	}
	out := make([]reflect.Value, numIn)
	for i := range out {
		aType := m.Type.In(i + 1) // Skip receiver arg.
		out[i] = reflect.New(aType).Elem()
	}
	return out
}

func BenchmarkFiltering(b *testing.B) {
	var kvs [26]attribute.KeyValue
	buf := [1]byte{'A' - 1}
	for i := range kvs {
		buf[0]++ // A, B, C ... Z
		kvs[i] = attribute.String(string(buf[:]), "")
	}

	var result struct {
		set     attribute.Set
		dropped []attribute.KeyValue
	}

	benchFn := func(fltr attribute.Filter) func(*testing.B) {
		return func(b *testing.B) {
			b.Helper()
			b.Run("Set.Filter", func(b *testing.B) {
				s := attribute.NewSet(kvs[:]...)
				b.ResetTimer()
				b.ReportAllocs()
				for n := 0; n < b.N; n++ {
					result.set, result.dropped = s.Filter(fltr)
				}
			})

			b.Run("NewSetWithFiltered", func(b *testing.B) {
				attrs := kvs[:]
				b.ResetTimer()
				b.ReportAllocs()
				for n := 0; n < b.N; n++ {
					result.set, result.dropped = attribute.NewSetWithFiltered(attrs, fltr)
				}
			})
		}
	}

	b.Run("NoFilter", benchFn(nil))
	b.Run("NoFiltered", benchFn(func(attribute.KeyValue) bool { return true }))
	b.Run("Filtered", benchFn(func(kv attribute.KeyValue) bool { return kv.Key == "A" }))
	b.Run("AllDropped", benchFn(func(attribute.KeyValue) bool { return false }))
}

var sinkSet attribute.Set

func BenchmarkNewSet(b *testing.B) {
	attrs := []attribute.KeyValue{
		attribute.String("B1", "2"),
		attribute.String("C2", "5"),
		attribute.String("B3", "2"),
		attribute.String("C4", "1"),
		attribute.String("A5", "4"),
		attribute.String("C6", "3"),
		attribute.String("A7", "1"),
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		sinkSet = attribute.NewSet(attrs...)
	}
}
