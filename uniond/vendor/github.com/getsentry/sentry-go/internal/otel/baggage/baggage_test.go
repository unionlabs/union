// Adapted from https://github.com/open-telemetry/opentelemetry-go/blob/c21b6b6bb31a2f74edd06e262f1690f3f6ea3d5c/baggage/baggage_test.go
//
// Copyright The OpenTelemetry Authors
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

//nolint:all
package baggage

import (
	"fmt"
	"math/rand"
	"sort"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/getsentry/sentry-go/internal/otel/baggage/internal/baggage"
)

var rng *rand.Rand

func init() {
	// Seed with a static value to ensure deterministic results.
	rng = rand.New(rand.NewSource(1))
}

func TestKeyRegExp(t *testing.T) {
	// ASCII only
	invalidKeyRune := []rune{
		'\x00', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07',
		'\x08', '\x09', '\x0A', '\x0B', '\x0C', '\x0D', '\x0E', '\x0F',
		'\x10', '\x11', '\x12', '\x13', '\x14', '\x15', '\x16', '\x17',
		'\x18', '\x19', '\x1A', '\x1B', '\x1C', '\x1D', '\x1E', '\x1F', ' ',
		'(', ')', '<', '>', '@', ',', ';', ':', '\\', '"', '/', '[', ']', '?',
		'=', '{', '}', '\x7F',
	}

	for _, ch := range invalidKeyRune {
		assert.NotRegexp(t, keyDef, fmt.Sprintf("%c", ch))
	}
}

func TestValueRegExp(t *testing.T) {
	// ASCII only
	invalidValueRune := []rune{
		'\x00', '\x01', '\x02', '\x03', '\x04', '\x05', '\x06', '\x07',
		'\x08', '\x09', '\x0A', '\x0B', '\x0C', '\x0D', '\x0E', '\x0F',
		'\x10', '\x11', '\x12', '\x13', '\x14', '\x15', '\x16', '\x17',
		'\x18', '\x19', '\x1A', '\x1B', '\x1C', '\x1D', '\x1E', '\x1F', ' ',
		'"', ',', ';', '\\', '\x7F',
	}

	for _, ch := range invalidValueRune {
		assert.NotRegexp(t, `^`+valueDef+`$`, fmt.Sprintf("invalid-%c-value", ch))
	}
}

func TestParseProperty(t *testing.T) {
	p := Property{key: "key", value: "value", hasValue: true}

	testcases := []struct {
		in       string
		expected Property
	}{
		{
			in:       "",
			expected: Property{},
		},
		{
			in: "key",
			expected: Property{
				key: "key",
			},
		},
		{
			in: "key=",
			expected: Property{
				key:      "key",
				hasValue: true,
			},
		},
		{
			in:       "key=value",
			expected: p,
		},
		{
			in:       " key=value ",
			expected: p,
		},
		{
			in:       "key = value",
			expected: p,
		},
		{
			in:       " key = value ",
			expected: p,
		},
		{
			in:       "\tkey=value",
			expected: p,
		},
	}

	for _, tc := range testcases {
		actual, err := parseProperty(tc.in)

		if !assert.NoError(t, err) {
			continue
		}

		assert.Equal(t, tc.expected.Key(), actual.Key(), tc.in)

		actualV, actualOk := actual.Value()
		expectedV, expectedOk := tc.expected.Value()
		assert.Equal(t, expectedOk, actualOk, tc.in)
		assert.Equal(t, expectedV, actualV, tc.in)
	}
}

func TestParsePropertyError(t *testing.T) {
	_, err := parseProperty(",;,")
	assert.ErrorIs(t, err, errInvalidProperty)
}

func TestNewKeyProperty(t *testing.T) {
	p, err := NewKeyProperty(" ")
	assert.ErrorIs(t, err, errInvalidKey)
	assert.Equal(t, Property{}, p)

	p, err = NewKeyProperty("key")
	assert.NoError(t, err)
	assert.Equal(t, Property{key: "key", hasData: true}, p)
}

func TestNewKeyValueProperty(t *testing.T) {
	p, err := NewKeyValueProperty(" ", "")
	assert.ErrorIs(t, err, errInvalidKey)
	assert.Equal(t, Property{}, p)

	p, err = NewKeyValueProperty("key", ";")
	assert.ErrorIs(t, err, errInvalidValue)
	assert.Equal(t, Property{}, p)

	p, err = NewKeyValueProperty("key", "value")
	assert.NoError(t, err)
	assert.Equal(t, Property{key: "key", value: "value", hasValue: true, hasData: true}, p)
}

func TestPropertyValidate(t *testing.T) {
	p := Property{}
	assert.ErrorIs(t, p.validate(), errInvalidProperty)

	p.hasData = true
	assert.ErrorIs(t, p.validate(), errInvalidKey)

	p.key = "k"
	assert.NoError(t, p.validate())

	p.value = ";"
	assert.EqualError(t, p.validate(), "invalid property: inconsistent value")

	p.hasValue = true
	assert.ErrorIs(t, p.validate(), errInvalidValue)

	p.value = "v"
	assert.NoError(t, p.validate())
}

func TestNewEmptyBaggage(t *testing.T) {
	b, err := New()
	assert.NoError(t, err)
	assert.Equal(t, Baggage{}, b)
}

func TestNewBaggage(t *testing.T) {
	b, err := New(Member{key: "k", hasData: true})
	assert.NoError(t, err)
	assert.Equal(t, Baggage{list: baggage.List{"k": {}}}, b)
}

func TestNewBaggageWithDuplicates(t *testing.T) {
	// Having this many members would normally cause this to error, but since
	// these are duplicates of the same key they will be collapsed into a
	// single entry.
	m := make([]Member, maxMembers+1)
	for i := range m {
		// Duplicates are collapsed.
		m[i] = Member{
			key:     "a",
			value:   fmt.Sprintf("%d", i),
			hasData: true,
		}
	}
	b, err := New(m...)
	assert.NoError(t, err)

	// Ensure that the last-one-wins by verifying the value.
	v := fmt.Sprintf("%d", maxMembers)
	want := Baggage{list: baggage.List{"a": {Value: v}}}
	assert.Equal(t, want, b)
}

func TestNewBaggageErrorEmptyMember(t *testing.T) {
	_, err := New(Member{})
	assert.ErrorIs(t, err, errInvalidMember)
}

func key(n int) string {
	r := []rune("0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")

	b := make([]rune, n)
	for i := range b {
		b[i] = r[rng.Intn(len(r))]
	}
	return string(b)
}

func TestNewBaggageErrorTooManyBytes(t *testing.T) {
	m := make([]Member, (maxBytesPerBaggageString/maxBytesPerMembers)+1)
	for i := range m {
		m[i] = Member{key: key(maxBytesPerMembers), hasData: true}
	}
	_, err := New(m...)
	assert.ErrorIs(t, err, errBaggageBytes)
}

func TestNewBaggageErrorTooManyMembers(t *testing.T) {
	m := make([]Member, maxMembers+1)
	for i := range m {
		m[i] = Member{key: fmt.Sprintf("%d", i), hasData: true}
	}
	_, err := New(m...)
	assert.ErrorIs(t, err, errMemberNumber)
}

func TestBaggageParse(t *testing.T) {
	tooLarge := key(maxBytesPerBaggageString + 1)

	tooLargeMember := key(maxBytesPerMembers + 1)

	m := make([]string, maxMembers+1)
	for i := range m {
		m[i] = fmt.Sprintf("a%d=", i)
	}
	tooManyMembers := strings.Join(m, listDelimiter)

	testcases := []struct {
		name string
		in   string
		want baggage.List
		err  error
	}{
		{
			name: "empty value",
			in:   "",
			want: baggage.List(nil),
		},
		{
			name: "single member empty value no properties",
			in:   "foo=",
			want: baggage.List{
				"foo": {Value: ""},
			},
		},
		{
			name: "single member no properties",
			in:   "foo=1",
			want: baggage.List{
				"foo": {Value: "1"},
			},
		},
		{
			name: "single member with spaces",
			in:   " foo \t= 1\t\t ",
			want: baggage.List{
				"foo": {Value: "1"},
			},
		},
		{
			name: "single member empty value with properties",
			in:   "foo=;state=on;red",
			want: baggage.List{
				"foo": {
					Value: "",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
					},
				},
			},
		},
		{
			name: "single member with properties",
			in:   "foo=1;state=on;red",
			want: baggage.List{
				"foo": {
					Value: "1",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
					},
				},
			},
		},
		{
			name: "single member with value containing equal signs",
			in:   "foo=0=0=0",
			want: baggage.List{
				"foo": {Value: "0=0=0"},
			},
		},
		{
			name: "two members with properties",
			in:   "foo=1;state=on;red,bar=2;yellow",
			want: baggage.List{
				"foo": {
					Value: "1",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
					},
				},
				"bar": {
					Value:      "2",
					Properties: []baggage.Property{{Key: "yellow"}},
				},
			},
		},
		{
			// According to the OTel spec, last value wins.
			name: "duplicate key",
			in:   "foo=1;state=on;red,foo=2",
			want: baggage.List{
				"foo": {Value: "2"},
			},
		},
		{
			name: "url encoded value",
			in:   "key1=val%252",
			want: baggage.List{
				"key1": {Value: "val%2"},
			},
		},
		{
			name: "percent-encoded whitespace in value",
			in:   "key1=val1%20val2",
			want: baggage.List{
				"key1": {Value: "val1 val2"},
			},
		},
		{
			name: "non-ASCII character in value",
			in:   "userId=Am%C3%A9lie",
			want: baggage.List{
				"userId": {Value: "Amélie"},
			},
		},
		{
			name: "plus character in value",
			in:   "key1=1+2",
			want: baggage.List{
				"key1": {Value: "1+2"},
			},
		},
		{
			name: "percent character in value",
			in:   "key1=1%25",
			want: baggage.List{
				"key1": {Value: "1%"},
			},
		},
		{
			name: "non-encoded equal signs in value",
			in:   "foo=bar=baz",
			want: baggage.List{
				"foo": {Value: "bar=baz"},
			},
		},
		{
			name: "invalid member: empty",
			in:   "foo=,,bar=",
			err:  errInvalidMember,
		},
		{
			name: "invalid member: no key",
			in:   "=foo",
			err:  errInvalidKey,
		},
		{
			name: "invalid member: no value",
			in:   "foo",
			err:  errInvalidMember,
		},
		{
			name: "invalid member: invalid key",
			in:   "\\=value",
			err:  errInvalidKey,
		},
		{
			name: "invalid member: invalid value",
			in:   "foo=\\",
			err:  errInvalidValue,
		},
		{
			name: "invalid property: invalid key",
			in:   "foo=1;=v",
			err:  errInvalidProperty,
		},
		{
			name: "invalid property: invalid value",
			in:   "foo=1;key=\\",
			err:  errInvalidProperty,
		},
		{
			name: "invalid baggage string: too large",
			in:   tooLarge,
			err:  errBaggageBytes,
		},
		{
			name: "invalid baggage string: member too large",
			in:   tooLargeMember,
			err:  errMemberBytes,
		},
		{
			name: "invalid baggage string: too many members",
			in:   tooManyMembers,
			err:  errMemberNumber,
		},
	}

	for _, tc := range testcases {
		t.Run(tc.name, func(t *testing.T) {
			actual, err := Parse(tc.in)
			assert.ErrorIs(t, err, tc.err)
			assert.Equal(t, Baggage{list: tc.want}, actual)
		})
	}
}

func TestBaggageString(t *testing.T) {
	testcases := []struct {
		name    string
		out     string
		baggage baggage.List
	}{
		{
			name:    "empty value",
			out:     "",
			baggage: baggage.List(nil),
		},
		{
			name: "single member empty value no properties",
			out:  "foo=",
			baggage: baggage.List{
				"foo": {Value: ""},
			},
		},
		{
			name: "single member no properties",
			out:  "foo=1",
			baggage: baggage.List{
				"foo": {Value: "1"},
			},
		},
		{
			name: "value with equal signs",
			out:  "foo=1=1",
			baggage: baggage.List{
				"foo": {Value: "1=1"},
			},
		},
		{
			name: "value with whitespaces",
			out:  "foo=1%202",
			baggage: baggage.List{
				"foo": {Value: "1 2"},
			},
		},
		{
			name: "value with non-ASCII characters",
			out:  "userId=Am%C3%A9lie",
			baggage: baggage.List{
				"userId": {Value: "Amélie"},
			},
		},
		{
			name: "value with plus character",
			out:  "foo=1+2",
			baggage: baggage.List{
				"foo": {Value: "1+2"},
			},
		},
		{
			name: "value with percent character",
			out:  "foo=1%25",
			baggage: baggage.List{
				"foo": {Value: "1%"},
			},
		},
		{
			name: "single member empty value with properties",
			out:  "foo=;red;state=on",
			baggage: baggage.List{
				"foo": {
					Value: "",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
					},
				},
			},
		},
		{
			name: "single member with properties",
			// Properties are "opaque values" meaning they are sent as they
			// are set and no encoding is performed.
			out: "foo=1;red;state=on;z=z=z",
			baggage: baggage.List{
				"foo": {
					Value: "1",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
						{Key: "z", Value: "z=z", HasValue: true},
					},
				},
			},
		},
		{
			name: "two members with properties",
			out:  "bar=2;yellow,foo=1;red;state=on",
			baggage: baggage.List{
				"foo": {
					Value: "1",
					Properties: []baggage.Property{
						{Key: "state", Value: "on", HasValue: true},
						{Key: "red"},
					},
				},
				"bar": {
					Value:      "2",
					Properties: []baggage.Property{{Key: "yellow"}},
				},
			},
		},
	}

	orderer := func(s string) string {
		members := strings.Split(s, listDelimiter)
		for i, m := range members {
			parts := strings.Split(m, propertyDelimiter)
			if len(parts) > 1 {
				sort.Strings(parts[1:])
				members[i] = strings.Join(parts, propertyDelimiter)
			}
		}
		sort.Strings(members)
		return strings.Join(members, listDelimiter)
	}

	for _, tc := range testcases {
		b := Baggage{tc.baggage}
		assert.Equal(t, tc.out, orderer(b.String()))
	}
}

func TestBaggageLen(t *testing.T) {
	b := Baggage{}
	assert.Equal(t, 0, b.Len())

	b.list = make(baggage.List, 1)
	assert.Equal(t, 0, b.Len())

	b.list["k"] = baggage.Item{}
	assert.Equal(t, 1, b.Len())
}

func TestBaggageDeleteMember(t *testing.T) {
	key := "k"

	b0 := Baggage{}
	b1 := b0.DeleteMember(key)
	assert.NotContains(t, b1.list, key)

	b0 = Baggage{list: baggage.List{
		key:     {},
		"other": {},
	}}
	b1 = b0.DeleteMember(key)
	assert.Contains(t, b0.list, key)
	assert.NotContains(t, b1.list, key)
}

func TestBaggageSetMemberEmpty(t *testing.T) {
	_, err := Baggage{}.SetMember(Member{})
	assert.ErrorIs(t, err, errInvalidMember)
}

func TestBaggageSetMember(t *testing.T) {
	b0 := Baggage{}

	key := "k"
	m := Member{key: key, hasData: true}
	b1, err := b0.SetMember(m)
	assert.NoError(t, err)
	assert.NotContains(t, b0.list, key)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, 0, len(b0.list))
	assert.Equal(t, 1, len(b1.list))

	m.value = "v"
	b2, err := b1.SetMember(m)
	assert.NoError(t, err)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, baggage.Item{Value: "v"}, b2.list[key])
	assert.Equal(t, 1, len(b1.list))
	assert.Equal(t, 1, len(b2.list))

	p := properties{{key: "p", hasData: true}}
	m.properties = p
	b3, err := b2.SetMember(m)
	assert.NoError(t, err)
	assert.Equal(t, baggage.Item{Value: "v"}, b2.list[key])
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	assert.Equal(t, 1, len(b2.list))
	assert.Equal(t, 1, len(b3.list))

	// The returned baggage needs to be immutable and should use a copy of the
	// properties slice.
	p[0] = Property{key: "different", hasData: true}
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	// Reset for below.
	p[0] = Property{key: "p", hasData: true}

	m = Member{key: "another", hasData: true}
	b4, err := b3.SetMember(m)
	assert.NoError(t, err)
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	assert.NotContains(t, b3.list, m.key)
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b4.list[key])
	assert.Equal(t, baggage.Item{}, b4.list[m.key])
	assert.Equal(t, 1, len(b3.list))
	assert.Equal(t, 2, len(b4.list))
}

func TestBaggageSetFalseMember(t *testing.T) {
	b0 := Baggage{}

	key := "k"
	m := Member{key: key, hasData: false}
	b1, err := b0.SetMember(m)
	assert.Error(t, err)
	assert.NotContains(t, b0.list, key)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, 0, len(b0.list))
	assert.Equal(t, 0, len(b1.list))

	m.value = "v"
	b2, err := b1.SetMember(m)
	assert.Error(t, err)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, baggage.Item{Value: ""}, b2.list[key])
	assert.Equal(t, 0, len(b1.list))
	assert.Equal(t, 0, len(b2.list))
}

func TestBaggageSetFalseMembers(t *testing.T) {
	b0 := Baggage{}

	key := "k"
	m := Member{key: key, hasData: true}
	b1, err := b0.SetMember(m)
	assert.NoError(t, err)
	assert.NotContains(t, b0.list, key)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, 0, len(b0.list))
	assert.Equal(t, 1, len(b1.list))

	m.value = "v"
	b2, err := b1.SetMember(m)
	assert.NoError(t, err)
	assert.Equal(t, baggage.Item{}, b1.list[key])
	assert.Equal(t, baggage.Item{Value: "v"}, b2.list[key])
	assert.Equal(t, 1, len(b1.list))
	assert.Equal(t, 1, len(b2.list))

	p := properties{{key: "p", hasData: false}}
	m.properties = p
	b3, err := b2.SetMember(m)
	assert.NoError(t, err)
	assert.Equal(t, baggage.Item{Value: "v"}, b2.list[key])
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	assert.Equal(t, 1, len(b2.list))
	assert.Equal(t, 1, len(b3.list))

	// The returned baggage needs to be immutable and should use a copy of the
	// properties slice.
	p[0] = Property{key: "different", hasData: false}
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	// Reset for below.
	p[0] = Property{key: "p", hasData: false}

	m = Member{key: "another", hasData: false}
	b4, err := b3.SetMember(m)
	assert.Error(t, err)
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b3.list[key])
	assert.NotContains(t, b3.list, m.key)
	assert.Equal(t, baggage.Item{Value: "v", Properties: []baggage.Property{{Key: "p"}}}, b4.list[key])
	assert.Equal(t, baggage.Item{}, b4.list[m.key])
	assert.Equal(t, 1, len(b3.list))
	assert.Equal(t, 1, len(b4.list))
}

func TestNilBaggageMembers(t *testing.T) {
	assert.Nil(t, Baggage{}.Members())
}

func TestBaggageMembers(t *testing.T) {
	members := []Member{
		{
			key:   "foo",
			value: "1",
			properties: properties{
				{key: "state", value: "on", hasValue: true},
				{key: "red"},
			},
			hasData: true,
		},
		{
			key:   "bar",
			value: "2",
			properties: properties{
				{key: "yellow"},
			},
			hasData: true,
		},
	}

	bag := Baggage{list: baggage.List{
		"foo": {
			Value: "1",
			Properties: []baggage.Property{
				{Key: "state", Value: "on", HasValue: true},
				{Key: "red"},
			},
		},
		"bar": {
			Value:      "2",
			Properties: []baggage.Property{{Key: "yellow"}},
		},
	}}

	assert.ElementsMatch(t, members, bag.Members())
}

func TestBaggageMember(t *testing.T) {
	bag := Baggage{list: baggage.List{"foo": {Value: "1"}}}
	assert.Equal(t, Member{key: "foo", value: "1", hasData: true}, bag.Member("foo"))
	assert.Equal(t, Member{}, bag.Member("bar"))
}

func TestMemberKey(t *testing.T) {
	m := Member{}
	assert.Equal(t, "", m.Key(), "even invalid values should be returned")

	key := "k"
	m.key = key
	assert.Equal(t, key, m.Key())
}

func TestMemberValue(t *testing.T) {
	m := Member{key: "k", value: "\\"}
	assert.Equal(t, "\\", m.Value(), "even invalid values should be returned")

	value := "v"
	m.value = value
	assert.Equal(t, value, m.Value())
}

func TestMemberProperties(t *testing.T) {
	m := Member{key: "k", value: "v"}
	assert.Nil(t, m.Properties())

	p := []Property{{key: "foo"}}
	m.properties = properties(p)
	got := m.Properties()
	assert.Equal(t, p, got)

	// Returned slice needs to be a copy so the original is immutable.
	got[0] = Property{key: "bar"}
	assert.NotEqual(t, m.properties, got)
}

func TestMemberValidation(t *testing.T) {
	m := Member{hasData: false}
	assert.ErrorIs(t, m.validate(), errInvalidMember)

	m.hasData = true
	assert.ErrorIs(t, m.validate(), errInvalidKey)

	m.key, m.value = "k", "\\"
	assert.NoError(t, m.validate())

	m.value = "v"
	assert.NoError(t, m.validate())
}

func TestNewMember(t *testing.T) {
	m, err := NewMember("", "")
	assert.ErrorIs(t, err, errInvalidKey)
	assert.Equal(t, Member{hasData: false}, m)

	key, val := "k", "v"
	p := Property{key: "foo", hasData: true}
	m, err = NewMember(key, val, p)
	assert.NoError(t, err)
	expected := Member{
		key:        key,
		value:      val,
		properties: properties{{key: "foo", hasData: true}},
		hasData:    true,
	}
	assert.Equal(t, expected, m)

	// value should not be decoded
	val = "%3B"
	m, err = NewMember(key, val, p)
	expected = Member{
		key:        key,
		value:      "%3B",
		properties: properties{{key: "foo", hasData: true}},
		hasData:    true,
	}
	assert.NoError(t, err)
	assert.Equal(t, expected, m)

	// Ensure new member is immutable.
	p.key = "bar"
	assert.Equal(t, expected, m)
}

func TestPropertiesValidate(t *testing.T) {
	p := properties{{hasData: true}}
	assert.ErrorIs(t, p.validate(), errInvalidKey)

	p[0].key = "foo"
	assert.NoError(t, p.validate())

	p = append(p, Property{key: "bar", hasData: true})
	assert.NoError(t, p.validate())
}

func TestMemberString(t *testing.T) {
	// normal key value pair
	member, _ := NewMember("key", "value")
	memberStr := member.String()
	assert.Equal(t, memberStr, "key=value")
	// percent sign should be percent-encoded
	member, _ = NewMember("key", "%3B")
	memberStr = member.String()
	assert.Equal(t, memberStr, "key=%253B")
}

var benchBaggage Baggage

func BenchmarkNew(b *testing.B) {
	mem1, _ := NewMember("key1", "val1")
	mem2, _ := NewMember("key2", "val2")
	mem3, _ := NewMember("key3", "val3")
	mem4, _ := NewMember("key4", "val4")

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		benchBaggage, _ = New(mem1, mem2, mem3, mem4)
	}
}

var benchMember Member

func BenchmarkNewMember(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		benchMember, _ = NewMember("key", "value")
	}
}

func BenchmarkParse(b *testing.B) {
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		benchBaggage, _ = Parse(`userId=alice,serverNode = DF28 , isProduction = false,hasProp=stuff;propKey;propWValue=value`)
	}
}
