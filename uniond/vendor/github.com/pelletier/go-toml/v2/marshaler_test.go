package toml_test

import (
	"bytes"
	"encoding/json"
	"fmt"
	"math"
	"math/big"
	"strings"
	"testing"
	"time"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type marshalTextKey struct {
	A string
	B string
}

func (k marshalTextKey) MarshalText() ([]byte, error) {
	return []byte(k.A + "-" + k.B), nil
}

type marshalBadTextKey struct{}

func (k marshalBadTextKey) MarshalText() ([]byte, error) {
	return nil, fmt.Errorf("error")
}

func TestMarshal(t *testing.T) {
	someInt := 42

	type structInline struct {
		A interface{} `toml:",inline"`
	}

	type comments struct {
		One   int
		Two   int   `comment:"Before kv"`
		Three []int `comment:"Before array"`
	}

	examples := []struct {
		desc     string
		v        interface{}
		expected string
		err      bool
	}{
		{
			desc: "simple map and string",
			v: map[string]string{
				"hello": "world",
			},
			expected: "hello = 'world'\n",
		},
		{
			desc: "map with new line in key",
			v: map[string]string{
				"hel\nlo": "world",
			},
			expected: "\"hel\\nlo\" = 'world'\n",
		},
		{
			desc: `map with " in key`,
			v: map[string]string{
				`hel"lo`: "world",
			},
			expected: "'hel\"lo' = 'world'\n",
		},
		{
			desc: "map in map and string",
			v: map[string]map[string]string{
				"table": {
					"hello": "world",
				},
			},
			expected: `[table]
hello = 'world'
`,
		},
		{
			desc: "map in map in map and string",
			v: map[string]map[string]map[string]string{
				"this": {
					"is": {
						"a": "test",
					},
				},
			},
			expected: `[this]
[this.is]
a = 'test'
`,
		},
		{
			desc: "map in map in map and string with values",
			v: map[string]interface{}{
				"this": map[string]interface{}{
					"is": map[string]string{
						"a": "test",
					},
					"also": "that",
				},
			},
			expected: `[this]
also = 'that'

[this.is]
a = 'test'
`,
		},
		{
			desc: `map with text key`,
			v: map[marshalTextKey]string{
				{A: "a", B: "1"}: "value 1",
				{A: "a", B: "2"}: "value 2",
				{A: "b", B: "1"}: "value 3",
			},
			expected: `a-1 = 'value 1'
a-2 = 'value 2'
b-1 = 'value 3'
`,
		},
		{
			desc: `table with text key`,
			v: map[marshalTextKey]map[string]string{
				{A: "a", B: "1"}: {"value": "foo"},
			},
			expected: `[a-1]
value = 'foo'
`,
		},
		{
			desc: `map with ptr text key`,
			v: map[*marshalTextKey]string{
				{A: "a", B: "1"}: "value 1",
				{A: "a", B: "2"}: "value 2",
				{A: "b", B: "1"}: "value 3",
			},
			expected: `a-1 = 'value 1'
a-2 = 'value 2'
b-1 = 'value 3'
`,
		},
		{
			desc: `map with bad text key`,
			v: map[marshalBadTextKey]string{
				{}: "value 1",
			},
			err: true,
		},
		{
			desc: `map with bad ptr text key`,
			v: map[*marshalBadTextKey]string{
				{}: "value 1",
			},
			err: true,
		},
		{
			desc: "simple string array",
			v: map[string][]string{
				"array": {"one", "two", "three"},
			},
			expected: `array = ['one', 'two', 'three']
`,
		},
		{
			desc:     "empty string array",
			v:        map[string][]string{},
			expected: ``,
		},
		{
			desc:     "map",
			v:        map[string][]string{},
			expected: ``,
		},
		{
			desc: "nested string arrays",
			v: map[string][][]string{
				"array": {{"one", "two"}, {"three"}},
			},
			expected: `array = [['one', 'two'], ['three']]
`,
		},
		{
			desc: "mixed strings and nested string arrays",
			v: map[string][]interface{}{
				"array": {"a string", []string{"one", "two"}, "last"},
			},
			expected: `array = ['a string', ['one', 'two'], 'last']
`,
		},
		{
			desc: "array of maps",
			v: map[string][]map[string]string{
				"top": {
					{"map1.1": "v1.1"},
					{"map2.1": "v2.1"},
				},
			},
			expected: `[[top]]
'map1.1' = 'v1.1'

[[top]]
'map2.1' = 'v2.1'
`,
		},
		{
			desc: "fixed size string array",
			v: map[string][3]string{
				"array": {"one", "two", "three"},
			},
			expected: `array = ['one', 'two', 'three']
`,
		},
		{
			desc: "fixed size nested string arrays",
			v: map[string][2][2]string{
				"array": {{"one", "two"}, {"three"}},
			},
			expected: `array = [['one', 'two'], ['three', '']]
`,
		},
		{
			desc: "mixed strings and fixed size nested string arrays",
			v: map[string][]interface{}{
				"array": {"a string", [2]string{"one", "two"}, "last"},
			},
			expected: `array = ['a string', ['one', 'two'], 'last']
`,
		},
		{
			desc: "fixed size array of maps",
			v: map[string][2]map[string]string{
				"ftop": {
					{"map1.1": "v1.1"},
					{"map2.1": "v2.1"},
				},
			},
			expected: `[[ftop]]
'map1.1' = 'v1.1'

[[ftop]]
'map2.1' = 'v2.1'
`,
		},
		{
			desc: "map with two keys",
			v: map[string]string{
				"key1": "value1",
				"key2": "value2",
			},
			expected: `key1 = 'value1'
key2 = 'value2'
`,
		},
		{
			desc: "simple struct",
			v: struct {
				A string
			}{
				A: "foo",
			},
			expected: `A = 'foo'
`,
		},
		{
			desc: "one level of structs within structs",
			v: struct {
				A interface{}
			}{
				A: struct {
					K1 string
					K2 string
				}{
					K1: "v1",
					K2: "v2",
				},
			},
			expected: `[A]
K1 = 'v1'
K2 = 'v2'
`,
		},
		{
			desc: "structs in array with interfaces",
			v: map[string]interface{}{
				"root": map[string]interface{}{
					"nested": []interface{}{
						map[string]interface{}{"name": "Bob"},
						map[string]interface{}{"name": "Alice"},
					},
				},
			},
			expected: `[root]
[[root.nested]]
name = 'Bob'

[[root.nested]]
name = 'Alice'
`,
		},
		{
			desc: "string escapes",
			v: map[string]interface{}{
				"a": "'\b\f\r\t\"\\",
			},
			expected: `a = "'\b\f\r\t\"\\"
`,
		},
		{
			desc: "string utf8 low",
			v: map[string]interface{}{
				"a": "'Ę",
			},
			expected: `a = "'Ę"
`,
		},
		{
			desc: "string utf8 low 2",
			v: map[string]interface{}{
				"a": "'\u10A85",
			},
			expected: "a = \"'\u10A85\"\n",
		},
		{
			desc: "string utf8 low 2",
			v: map[string]interface{}{
				"a": "'\u10A85",
			},
			expected: "a = \"'\u10A85\"\n",
		},
		{
			desc: "emoji",
			v: map[string]interface{}{
				"a": "'😀",
			},
			expected: "a = \"'😀\"\n",
		},
		{
			desc: "control char",
			v: map[string]interface{}{
				"a": "'\u001A",
			},
			expected: `a = "'\u001A"
`,
		},
		{
			desc: "multi-line string",
			v: map[string]interface{}{
				"a": "hello\nworld",
			},
			expected: `a = "hello\nworld"
`,
		},
		{
			desc: "multi-line forced",
			v: struct {
				A string `toml:",multiline"`
			}{
				A: "hello\nworld",
			},
			expected: `A = """
hello
world"""
`,
		},
		{
			desc: "inline field",
			v: struct {
				A map[string]string `toml:",inline"`
				B map[string]string
			}{
				A: map[string]string{
					"isinline": "yes",
				},
				B: map[string]string{
					"isinline": "no",
				},
			},
			expected: `A = {isinline = 'yes'}

[B]
isinline = 'no'
`,
		},
		{
			desc: "mutiline array int",
			v: struct {
				A []int `toml:",multiline"`
				B []int
			}{
				A: []int{1, 2, 3, 4},
				B: []int{1, 2, 3, 4},
			},
			expected: `A = [
  1,
  2,
  3,
  4
]
B = [1, 2, 3, 4]
`,
		},
		{
			desc: "mutiline array in array",
			v: struct {
				A [][]int `toml:",multiline"`
			}{
				A: [][]int{{1, 2}, {3, 4}},
			},
			expected: `A = [
  [1, 2],
  [3, 4]
]
`,
		},
		{
			desc: "nil interface not supported at root",
			v:    nil,
			err:  true,
		},
		{
			desc: "nil interface not supported in slice",
			v: map[string]interface{}{
				"a": []interface{}{"a", nil, 2},
			},
			err: true,
		},
		{
			desc: "nil pointer in slice uses zero value",
			v: struct {
				A []*int
			}{
				A: []*int{nil},
			},
			expected: `A = [0]
`,
		},
		{
			desc: "nil pointer in slice uses zero value",
			v: struct {
				A []*int
			}{
				A: []*int{nil},
			},
			expected: `A = [0]
`,
		},
		{
			desc: "pointer in slice",
			v: struct {
				A []*int
			}{
				A: []*int{&someInt},
			},
			expected: `A = [42]
`,
		},
		{
			desc: "inline table in inline table",
			v: structInline{
				A: structInline{
					A: structInline{
						A: "hello",
					},
				},
			},
			expected: `A = {A = {A = 'hello'}}
`,
		},
		{
			desc: "empty slice in map",
			v: map[string][]string{
				"a": {},
			},
			expected: `a = []
`,
		},
		{
			desc: "map in slice",
			v: map[string][]map[string]string{
				"a": {{"hello": "world"}},
			},
			expected: `[[a]]
hello = 'world'
`,
		},
		{
			desc: "newline in map in slice",
			v: map[string][]map[string]string{
				"a\n": {{"hello": "world"}},
			},
			expected: `[["a\n"]]
hello = 'world'
`,
		},
		{
			desc: "newline in map in slice",
			v: map[string][]map[string]*customTextMarshaler{
				"a": {{"hello": &customTextMarshaler{1}}},
			},
			err: true,
		},
		{
			desc: "empty slice of empty struct",
			v: struct {
				A []struct{}
			}{
				A: []struct{}{},
			},
			expected: `A = []
`,
		},
		{
			desc: "nil field is ignored",
			v: struct {
				A interface{}
			}{
				A: nil,
			},
			expected: ``,
		},
		{
			desc: "private fields are ignored",
			v: struct {
				Public  string
				private string
			}{
				Public:  "shown",
				private: "hidden",
			},
			expected: `Public = 'shown'
`,
		},
		{
			desc: "fields tagged - are ignored",
			v: struct {
				Public  string `toml:"-"`
				private string
			}{
				Public: "hidden",
			},
			expected: ``,
		},
		{
			desc: "nil value in map is ignored",
			v: map[string]interface{}{
				"A": nil,
			},
			expected: ``,
		},
		{
			desc: "new line in table key",
			v: map[string]interface{}{
				"hello\nworld": 42,
			},
			expected: `"hello\nworld" = 42
`,
		},
		{
			desc: "new line in parent of nested table key",
			v: map[string]interface{}{
				"hello\nworld": map[string]interface{}{
					"inner": 42,
				},
			},
			expected: `["hello\nworld"]
inner = 42
`,
		},
		{
			desc: "new line in nested table key",
			v: map[string]interface{}{
				"parent": map[string]interface{}{
					"in\ner": map[string]interface{}{
						"foo": 42,
					},
				},
			},
			expected: `[parent]
[parent."in\ner"]
foo = 42
`,
		},
		{
			desc: "int map key",
			v:    map[int]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "int8 map key",
			v:    map[int8]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "int64 map key",
			v:    map[int64]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "uint map key",
			v:    map[uint]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "uint8 map key",
			v:    map[uint8]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "uint64 map key",
			v:    map[uint64]interface{}{1: "a"},
			expected: `1 = 'a'
`,
		},
		{
			desc: "float32 map key",
			v: map[float32]interface{}{
				1.1:    "a",
				1.0020: "b",
			},
			expected: `'1.002' = 'b'
'1.1' = 'a'
`,
		},
		{
			desc: "float64 map key",
			v: map[float64]interface{}{
				1.1:    "a",
				1.0020: "b",
			},
			expected: `'1.002' = 'b'
'1.1' = 'a'
`,
		},
		{
			desc: "invalid map key",
			v:    map[struct{ int }]interface{}{{1}: "a"},
			err:  true,
		},
		{
			desc:     "invalid map key but empty",
			v:        map[struct{ int }]interface{}{},
			expected: "",
		},
		{
			desc: "unhandled type",
			v: struct {
				A chan int
			}{
				A: make(chan int),
			},
			err: true,
		},
		{
			desc: "time",
			v: struct {
				T time.Time
			}{
				T: time.Time{},
			},
			expected: `T = 0001-01-01T00:00:00Z
`,
		},
		{
			desc: "time nano",
			v: struct {
				T time.Time
			}{
				T: time.Date(1979, time.May, 27, 0, 32, 0, 999999000, time.UTC),
			},
			expected: `T = 1979-05-27T00:32:00.999999Z
`,
		},
		{
			desc: "bool",
			v: struct {
				A bool
				B bool
			}{
				A: false,
				B: true,
			},
			expected: `A = false
B = true
`,
		},
		{
			desc: "numbers",
			v: struct {
				A float32
				B uint64
				C uint32
				D uint16
				E uint8
				F uint
				G int64
				H int32
				I int16
				J int8
				K int
				L float64
			}{
				A: 1.1,
				B: 42,
				C: 42,
				D: 42,
				E: 42,
				F: 42,
				G: 42,
				H: 42,
				I: 42,
				J: 42,
				K: 42,
				L: 2.2,
			},
			expected: `A = 1.1
B = 42
C = 42
D = 42
E = 42
F = 42
G = 42
H = 42
I = 42
J = 42
K = 42
L = 2.2
`,
		},
		{
			desc: "comments",
			v: struct {
				Table comments `comment:"Before table"`
			}{
				Table: comments{
					One:   1,
					Two:   2,
					Three: []int{1, 2, 3},
				},
			},
			expected: `# Before table
[Table]
One = 1
# Before kv
Two = 2
# Before array
Three = [1, 2, 3]
`,
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {
			b, err := toml.Marshal(e.v)
			if e.err {
				require.Error(t, err)

				return
			}

			require.NoError(t, err)
			assert.Equal(t, e.expected, string(b))

			// make sure the output is always valid TOML
			defaultMap := map[string]interface{}{}
			err = toml.Unmarshal(b, &defaultMap)
			require.NoError(t, err)

			testWithAllFlags(t, func(t *testing.T, flags int) {
				t.Helper()

				var buf bytes.Buffer
				enc := toml.NewEncoder(&buf)
				setFlags(enc, flags)

				err := enc.Encode(e.v)
				require.NoError(t, err)

				inlineMap := map[string]interface{}{}
				err = toml.Unmarshal(buf.Bytes(), &inlineMap)
				require.NoError(t, err)

				require.Equal(t, defaultMap, inlineMap)
			})
		})
	}
}

type flagsSetters []struct {
	name string
	f    func(enc *toml.Encoder, flag bool) *toml.Encoder
}

var allFlags = flagsSetters{
	{"arrays-multiline", (*toml.Encoder).SetArraysMultiline},
	{"tables-inline", (*toml.Encoder).SetTablesInline},
	{"indent-tables", (*toml.Encoder).SetIndentTables},
}

func setFlags(enc *toml.Encoder, flags int) {
	for i := 0; i < len(allFlags); i++ {
		enabled := flags&1 > 0
		allFlags[i].f(enc, enabled)
	}
}

func testWithAllFlags(t *testing.T, testfn func(t *testing.T, flags int)) {
	t.Helper()
	testWithFlags(t, 0, allFlags, testfn)
}

func testWithFlags(t *testing.T, flags int, setters flagsSetters, testfn func(t *testing.T, flags int)) {
	t.Helper()

	if len(setters) == 0 {
		testfn(t, flags)

		return
	}

	s := setters[0]

	for _, enabled := range []bool{false, true} {
		name := fmt.Sprintf("%s=%t", s.name, enabled)
		newFlags := flags << 1

		if enabled {
			newFlags++
		}

		t.Run(name, func(t *testing.T) {
			testWithFlags(t, newFlags, setters[1:], testfn)
		})
	}
}

func TestMarshalFloats(t *testing.T) {
	v := map[string]float32{
		"nan":  float32(math.NaN()),
		"+inf": float32(math.Inf(1)),
		"-inf": float32(math.Inf(-1)),
	}

	expected := `'+inf' = inf
-inf = -inf
nan = nan
`

	actual, err := toml.Marshal(v)
	require.NoError(t, err)
	require.Equal(t, expected, string(actual))

	v64 := map[string]float64{
		"nan":  math.NaN(),
		"+inf": math.Inf(1),
		"-inf": math.Inf(-1),
	}

	actual, err = toml.Marshal(v64)
	require.NoError(t, err)
	require.Equal(t, expected, string(actual))
}

//nolint:funlen
func TestMarshalIndentTables(t *testing.T) {
	examples := []struct {
		desc     string
		v        interface{}
		expected string
	}{
		{
			desc: "one kv",
			v: map[string]interface{}{
				"foo": "bar",
			},
			expected: `foo = 'bar'
`,
		},
		{
			desc: "one level table",
			v: map[string]map[string]string{
				"foo": {
					"one": "value1",
					"two": "value2",
				},
			},
			expected: `[foo]
  one = 'value1'
  two = 'value2'
`,
		},
		{
			desc: "two levels table",
			v: map[string]interface{}{
				"root": "value0",
				"level1": map[string]interface{}{
					"one": "value1",
					"level2": map[string]interface{}{
						"two": "value2",
					},
				},
			},
			expected: `root = 'value0'

[level1]
  one = 'value1'

  [level1.level2]
    two = 'value2'
`,
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {
			var buf strings.Builder
			enc := toml.NewEncoder(&buf)
			enc.SetIndentTables(true)
			err := enc.Encode(e.v)
			require.NoError(t, err)
			assert.Equal(t, e.expected, buf.String())
		})
	}
}

type customTextMarshaler struct {
	value int64
}

func (c *customTextMarshaler) MarshalText() ([]byte, error) {
	if c.value == 1 {
		return nil, fmt.Errorf("cannot represent 1 because this is a silly test")
	}
	return []byte(fmt.Sprintf("::%d", c.value)), nil
}

func TestMarshalTextMarshaler_NoRoot(t *testing.T) {
	c := customTextMarshaler{}
	_, err := toml.Marshal(&c)
	require.Error(t, err)
}

func TestMarshalTextMarshaler_Error(t *testing.T) {
	m := map[string]interface{}{"a": &customTextMarshaler{value: 1}}
	_, err := toml.Marshal(m)
	require.Error(t, err)
}

func TestMarshalTextMarshaler_ErrorInline(t *testing.T) {
	type s struct {
		A map[string]interface{} `inline:"true"`
	}

	d := s{
		A: map[string]interface{}{"a": &customTextMarshaler{value: 1}},
	}

	_, err := toml.Marshal(d)
	require.Error(t, err)
}

func TestMarshalTextMarshaler(t *testing.T) {
	m := map[string]interface{}{"a": &customTextMarshaler{value: 2}}
	r, err := toml.Marshal(m)
	require.NoError(t, err)
	assert.Equal(t, "a = '::2'\n", string(r))
}

type brokenWriter struct{}

func (b *brokenWriter) Write([]byte) (int, error) {
	return 0, fmt.Errorf("dead")
}

func TestEncodeToBrokenWriter(t *testing.T) {
	w := brokenWriter{}
	enc := toml.NewEncoder(&w)
	err := enc.Encode(map[string]string{"hello": "world"})
	require.Error(t, err)
}

func TestEncoderSetIndentSymbol(t *testing.T) {
	var w strings.Builder
	enc := toml.NewEncoder(&w)
	enc.SetIndentTables(true)
	enc.SetIndentSymbol(">>>")
	err := enc.Encode(map[string]map[string]string{"parent": {"hello": "world"}})
	require.NoError(t, err)
	expected := `[parent]
>>>hello = 'world'
`
	assert.Equal(t, expected, w.String())
}

func TestEncoderSetMarshalJsonNumbers(t *testing.T) {
	var w strings.Builder
	enc := toml.NewEncoder(&w)
	enc.SetMarshalJsonNumbers(true)
	err := enc.Encode(map[string]interface{}{
		"A": json.Number("1.1"),
		"B": json.Number("42e-3"),
		"C": json.Number("42"),
		"D": json.Number("0"),
		"E": json.Number("0.0"),
		"F": json.Number(""),
	})
	require.NoError(t, err)
	expected := `A = 1.1
B = 0.042
C = 42
D = 0
E = 0.0
F = 0
`
	assert.Equal(t, expected, w.String())
}

func TestEncoderOmitempty(t *testing.T) {
	type doc struct {
		String  string            `toml:",omitempty,multiline"`
		Bool    bool              `toml:",omitempty,multiline"`
		Int     int               `toml:",omitempty,multiline"`
		Int8    int8              `toml:",omitempty,multiline"`
		Int16   int16             `toml:",omitempty,multiline"`
		Int32   int32             `toml:",omitempty,multiline"`
		Int64   int64             `toml:",omitempty,multiline"`
		Uint    uint              `toml:",omitempty,multiline"`
		Uint8   uint8             `toml:",omitempty,multiline"`
		Uint16  uint16            `toml:",omitempty,multiline"`
		Uint32  uint32            `toml:",omitempty,multiline"`
		Uint64  uint64            `toml:",omitempty,multiline"`
		Float32 float32           `toml:",omitempty,multiline"`
		Float64 float64           `toml:",omitempty,multiline"`
		MapNil  map[string]string `toml:",omitempty,multiline"`
		Slice   []string          `toml:",omitempty,multiline"`
		Ptr     *string           `toml:",omitempty,multiline"`
		Iface   interface{}       `toml:",omitempty,multiline"`
		Struct  struct{}          `toml:",omitempty,multiline"`
	}

	d := doc{}

	b, err := toml.Marshal(d)
	require.NoError(t, err)

	expected := ``

	assert.Equal(t, expected, string(b))
}

func TestEncoderTagFieldName(t *testing.T) {
	type doc struct {
		String string `toml:"hello"`
		OkSym  string `toml:"#"`
		Bad    string `toml:"\"`
	}

	d := doc{String: "world"}

	b, err := toml.Marshal(d)
	require.NoError(t, err)

	expected := `hello = 'world'
'#' = ''
Bad = ''
`

	assert.Equal(t, expected, string(b))
}

func TestIssue436(t *testing.T) {
	data := []byte(`{"a": [ { "b": { "c": "d" } } ]}`)

	var v interface{}
	err := json.Unmarshal(data, &v)
	require.NoError(t, err)

	var buf bytes.Buffer
	err = toml.NewEncoder(&buf).Encode(v)
	require.NoError(t, err)

	expected := `[[a]]
[a.b]
c = 'd'
`
	assert.Equal(t, expected, buf.String())
}

func TestIssue424(t *testing.T) {
	type Message1 struct {
		Text string
	}

	type Message2 struct {
		Text string `multiline:"true"`
	}

	msg1 := Message1{"Hello\\World"}
	msg2 := Message2{"Hello\\World"}

	toml1, err := toml.Marshal(msg1)
	require.NoError(t, err)

	toml2, err := toml.Marshal(msg2)
	require.NoError(t, err)

	msg1parsed := Message1{}
	err = toml.Unmarshal(toml1, &msg1parsed)
	require.NoError(t, err)
	require.Equal(t, msg1, msg1parsed)

	msg2parsed := Message2{}
	err = toml.Unmarshal(toml2, &msg2parsed)
	require.NoError(t, err)
	require.Equal(t, msg2, msg2parsed)
}

func TestIssue567(t *testing.T) {
	var m map[string]interface{}
	err := toml.Unmarshal([]byte("A = 12:08:05"), &m)
	require.NoError(t, err)
	require.IsType(t, m["A"], toml.LocalTime{})
}

func TestIssue590(t *testing.T) {
	type CustomType int
	var cfg struct {
		Option CustomType `toml:"option"`
	}
	err := toml.Unmarshal([]byte("option = 42"), &cfg)
	require.NoError(t, err)
}

func TestIssue571(t *testing.T) {
	type Foo struct {
		Float32 float32
		Float64 float64
	}

	const closeEnough = 1e-9

	foo := Foo{
		Float32: 42,
		Float64: 43,
	}
	b, err := toml.Marshal(foo)
	require.NoError(t, err)

	var foo2 Foo
	err = toml.Unmarshal(b, &foo2)
	require.NoError(t, err)

	assert.InDelta(t, 42, foo2.Float32, closeEnough)
	assert.InDelta(t, 43, foo2.Float64, closeEnough)
}

func TestIssue678(t *testing.T) {
	type Config struct {
		BigInt big.Int
	}

	cfg := &Config{
		BigInt: *big.NewInt(123),
	}

	out, err := toml.Marshal(cfg)
	require.NoError(t, err)
	assert.Equal(t, "BigInt = '123'\n", string(out))

	cfg2 := &Config{}
	err = toml.Unmarshal(out, cfg2)
	require.NoError(t, err)
	require.Equal(t, cfg, cfg2)
}

func TestIssue752(t *testing.T) {
	type Fooer interface {
		Foo() string
	}

	type Container struct {
		Fooer
	}

	c := Container{}

	out, err := toml.Marshal(c)
	require.NoError(t, err)
	require.Equal(t, "", string(out))
}

func TestIssue768(t *testing.T) {
	type cfg struct {
		Name string `comment:"This is a multiline comment.\nThis is line 2."`
	}

	out, err := toml.Marshal(&cfg{})
	require.NoError(t, err)

	expected := `# This is a multiline comment.
# This is line 2.
Name = ''
`

	require.Equal(t, expected, string(out))
}

func TestIssue786(t *testing.T) {
	type Dependencies struct {
		Dependencies         []string `toml:"dependencies,multiline,omitempty"`
		BuildDependencies    []string `toml:"buildDependencies,multiline,omitempty"`
		OptionalDependencies []string `toml:"optionalDependencies,multiline,omitempty"`
	}

	type Test struct {
		Dependencies Dependencies `toml:"dependencies,omitempty"`
	}

	x := Test{}
	b, err := toml.Marshal(x)
	require.NoError(t, err)

	require.Equal(t, "", string(b))

	type General struct {
		From      string `toml:"from,omitempty" json:"from,omitempty" comment:"from in graphite-web format, the local TZ is used"`
		Randomize bool   `toml:"randomize" json:"randomize" comment:"randomize starting time with [0,step)"`
	}

	type Custom struct {
		Name string `toml:"name" json:"name,omitempty" comment:"names for generator, braces are expanded like in shell"`
		Type string `toml:"type,omitempty" json:"type,omitempty" comment:"type of generator"`
		General
	}
	type Config struct {
		General
		Custom []Custom `toml:"custom,omitempty" json:"custom,omitempty" comment:"generators with custom parameters can be specified separately"`
	}

	buf := new(bytes.Buffer)
	config := &Config{General: General{From: "-2d", Randomize: true}}
	config.Custom = []Custom{{Name: "omit", General: General{Randomize: false}}}
	config.Custom = append(config.Custom, Custom{Name: "present", General: General{From: "-2d", Randomize: true}})
	encoder := toml.NewEncoder(buf)
	encoder.Encode(config)

	expected := `# from in graphite-web format, the local TZ is used
from = '-2d'
# randomize starting time with [0,step)
randomize = true

# generators with custom parameters can be specified separately
[[custom]]
# names for generator, braces are expanded like in shell
name = 'omit'
# randomize starting time with [0,step)
randomize = false

[[custom]]
# names for generator, braces are expanded like in shell
name = 'present'
# from in graphite-web format, the local TZ is used
from = '-2d'
# randomize starting time with [0,step)
randomize = true
`

	require.Equal(t, expected, buf.String())
}

func TestMarhsalIssue888(t *testing.T) {
	type Thing struct {
		FieldA string `comment:"my field A"`
		FieldB string `comment:"my field B"`
	}

	type Cfg struct {
		Custom []Thing `comment:"custom config"`
	}

	buf := new(bytes.Buffer)

	config := Cfg{
		Custom: []Thing{
			{FieldA: "field a 1", FieldB: "field b 1"},
			{FieldA: "field a 2", FieldB: "field b 2"},
		},
	}

	encoder := toml.NewEncoder(buf).SetIndentTables(true)
	encoder.Encode(config)

	expected := `# custom config
[[Custom]]
  # my field A
  FieldA = 'field a 1'
  # my field B
  FieldB = 'field b 1'

[[Custom]]
  # my field A
  FieldA = 'field a 2'
  # my field B
  FieldB = 'field b 2'
`

	require.Equal(t, expected, buf.String())
}

func TestMarshalNestedAnonymousStructs(t *testing.T) {
	type Embedded struct {
		Value string `toml:"value" json:"value"`
		Top   struct {
			Value string `toml:"value" json:"value"`
		} `toml:"top" json:"top"`
	}

	type Named struct {
		Value string `toml:"value" json:"value"`
	}

	var doc struct {
		Embedded
		Named     `toml:"named" json:"named"`
		Anonymous struct {
			Value string `toml:"value" json:"value"`
		} `toml:"anonymous" json:"anonymous"`
	}

	expected := `value = ''

[top]
value = ''

[named]
value = ''

[anonymous]
value = ''
`

	result, err := toml.Marshal(doc)
	require.NoError(t, err)
	require.Equal(t, expected, string(result))
}

func TestMarshalNestedAnonymousStructs_DuplicateField(t *testing.T) {
	type Embedded struct {
		Value string `toml:"value" json:"value"`
		Top   struct {
			Value string `toml:"value" json:"value"`
		} `toml:"top" json:"top"`
	}

	var doc struct {
		Value string `toml:"value" json:"value"`
		Embedded
	}
	doc.Embedded.Value = "shadowed"
	doc.Value = "shadows"

	expected := `value = 'shadows'

[top]
value = ''
`

	result, err := toml.Marshal(doc)
	require.NoError(t, err)
	require.NoError(t, err)
	require.Equal(t, expected, string(result))
}

func TestMarshalNestedAnonymousStructs_PointerEmbedded(t *testing.T) {
	type Embedded struct {
		Value   string  `toml:"value" json:"value"`
		Omitted string  `toml:"omitted,omitempty"`
		Ptr     *string `toml:"ptr"`
	}

	type Named struct {
		Value string `toml:"value" json:"value"`
	}

	type Doc struct {
		*Embedded
		*Named    `toml:"named" json:"named"`
		Anonymous struct {
			*Embedded
			Value *string `toml:"value" json:"value"`
		} `toml:"anonymous,omitempty" json:"anonymous,omitempty"`
	}

	doc := &Doc{
		Embedded: &Embedded{Value: "foo"},
	}

	expected := `value = 'foo'
`

	result, err := toml.Marshal(doc)
	require.NoError(t, err)
	require.Equal(t, expected, string(result))
}

func TestLocalTime(t *testing.T) {
	v := map[string]toml.LocalTime{
		"a": {
			Hour:       1,
			Minute:     2,
			Second:     3,
			Nanosecond: 4,
		},
	}

	expected := `a = 01:02:03.000000004
`

	out, err := toml.Marshal(v)
	require.NoError(t, err)
	require.Equal(t, expected, string(out))
}

func TestMarshalUint64Overflow(t *testing.T) {
	// The TOML spec only requires implementation to provide support for the
	// int64 range. To avoid generating TOML documents that would not be
	// supported by standard-compliant parsers, uint64 > max int64 cannot be
	// marshaled.
	x := map[string]interface{}{
		"foo": uint64(math.MaxInt64) + 1,
	}

	_, err := toml.Marshal(x)
	require.Error(t, err)
}

func TestIndentWithInlineTable(t *testing.T) {
	x := map[string][]map[string]string{
		"one": {
			{"0": "0"},
			{"1": "1"},
		},
	}
	expected := `one = [
  {0 = '0'},
  {1 = '1'}
]
`
	var buf bytes.Buffer
	enc := toml.NewEncoder(&buf)
	enc.SetIndentTables(true)
	enc.SetTablesInline(true)
	enc.SetArraysMultiline(true)
	require.NoError(t, enc.Encode(x))
	assert.Equal(t, expected, buf.String())
}

type C3 struct {
	Value  int   `toml:",commented"`
	Values []int `toml:",commented"`
}

type C2 struct {
	Int       int64
	String    string
	ArrayInts []int
	Structs   []C3
}

type C1 struct {
	Int       int64  `toml:",commented"`
	String    string `toml:",commented"`
	ArrayInts []int  `toml:",commented"`
	Structs   []C3   `toml:",commented"`
}

type Commented struct {
	Int    int64  `toml:",commented"`
	String string `toml:",commented"`

	C1 C1
	C2 C2 `toml:",commented"` // same as C1, but commented at top level
}

func TestMarshalCommented(t *testing.T) {
	c := Commented{
		Int:    42,
		String: "root",

		C1: C1{
			Int:       11,
			String:    "C1",
			ArrayInts: []int{1, 2, 3},
			Structs: []C3{
				{Value: 100},
				{Values: []int{4, 5, 6}},
			},
		},
		C2: C2{
			Int:       22,
			String:    "C2",
			ArrayInts: []int{1, 2, 3},
			Structs: []C3{
				{Value: 100},
				{Values: []int{4, 5, 6}},
			},
		},
	}

	out, err := toml.Marshal(c)
	require.NoError(t, err)

	expected := `# Int = 42
# String = 'root'

[C1]
# Int = 11
# String = 'C1'
# ArrayInts = [1, 2, 3]

# [[C1.Structs]]
# Value = 100
# Values = []

# [[C1.Structs]]
# Value = 0
# Values = [4, 5, 6]

# [C2]
# Int = 22
# String = 'C2'
# ArrayInts = [1, 2, 3]

# [[C2.Structs]]
# Value = 100
# Values = []

# [[C2.Structs]]
# Value = 0
# Values = [4, 5, 6]
`

	require.Equal(t, expected, string(out))
}

func TestMarshalIndentedCustomTypeArray(t *testing.T) {
	c := struct {
		Nested struct {
			NestedArray []struct {
				Value int
			}
		}
	}{
		Nested: struct {
			NestedArray []struct {
				Value int
			}
		}{
			NestedArray: []struct {
				Value int
			}{
				{Value: 1},
				{Value: 2},
			},
		},
	}

	expected := `[Nested]
  [[Nested.NestedArray]]
    Value = 1

  [[Nested.NestedArray]]
    Value = 2
`

	var buf bytes.Buffer
	enc := toml.NewEncoder(&buf)
	enc.SetIndentTables(true)
	require.NoError(t, enc.Encode(c))
	require.Equal(t, expected, buf.String())
}

func ExampleMarshal() {
	type MyConfig struct {
		Version int
		Name    string
		Tags    []string
	}

	cfg := MyConfig{
		Version: 2,
		Name:    "go-toml",
		Tags:    []string{"go", "toml"},
	}

	b, err := toml.Marshal(cfg)
	if err != nil {
		panic(err)
	}
	fmt.Println(string(b))

	// Output:
	// Version = 2
	// Name = 'go-toml'
	// Tags = ['go', 'toml']
}

// Example that uses the 'commented' field tag option to generate an example
// configuration file that has commented out sections (example from
// go-graphite/graphite-clickhouse).
func ExampleMarshal_commented() {
	type Common struct {
		Listen               string        `toml:"listen"                     comment:"general listener"`
		PprofListen          string        `toml:"pprof-listen"               comment:"listener to serve /debug/pprof requests. '-pprof' argument overrides it"`
		MaxMetricsPerTarget  int           `toml:"max-metrics-per-target"     comment:"limit numbers of queried metrics per target in /render requests, 0 or negative = unlimited"`
		MemoryReturnInterval time.Duration `toml:"memory-return-interval"     comment:"daemon will return the freed memory to the OS when it>0"`
	}

	type Costs struct {
		Cost       *int           `toml:"cost"        comment:"default cost (for wildcarded equalence or matched with regex, or if no value cost set)"`
		ValuesCost map[string]int `toml:"values-cost" comment:"cost with some value (for equalence without wildcards) (additional tuning, usually not needed)"`
	}

	type ClickHouse struct {
		URL string `toml:"url" comment:"default url, see https://clickhouse.tech/docs/en/interfaces/http. Can be overwritten with query-params"`

		RenderMaxQueries        int               `toml:"render-max-queries" comment:"Max queries to render queiries"`
		RenderConcurrentQueries int               `toml:"render-concurrent-queries" comment:"Concurrent queries to render queiries"`
		TaggedCosts             map[string]*Costs `toml:"tagged-costs,commented"`
		TreeTable               string            `toml:"tree-table,commented"`
		ReverseTreeTable        string            `toml:"reverse-tree-table,commented"`
		DateTreeTable           string            `toml:"date-tree-table,commented"`
		DateTreeTableVersion    int               `toml:"date-tree-table-version,commented"`
		TreeTimeout             time.Duration     `toml:"tree-timeout,commented"`
		TagTable                string            `toml:"tag-table,commented"`
		ExtraPrefix             string            `toml:"extra-prefix"             comment:"add extra prefix (directory in graphite) for all metrics, w/o trailing dot"`
		ConnectTimeout          time.Duration     `toml:"connect-timeout"          comment:"TCP connection timeout"`
		DataTableLegacy         string            `toml:"data-table,commented"`
		RollupConfLegacy        string            `toml:"rollup-conf,commented"`
		MaxDataPoints           int               `toml:"max-data-points"          comment:"max points per metric when internal-aggregation=true"`
		InternalAggregation     bool              `toml:"internal-aggregation"     comment:"ClickHouse-side aggregation, see doc/aggregation.md"`
	}

	type Tags struct {
		Rules      string `toml:"rules"`
		Date       string `toml:"date"`
		ExtraWhere string `toml:"extra-where"`
		InputFile  string `toml:"input-file"`
		OutputFile string `toml:"output-file"`
	}

	type Config struct {
		Common     Common     `toml:"common"`
		ClickHouse ClickHouse `toml:"clickhouse"`
		Tags       Tags       `toml:"tags,commented"`
	}

	cfg := &Config{
		Common: Common{
			Listen:               ":9090",
			PprofListen:          "",
			MaxMetricsPerTarget:  15000, // This is arbitrary value to protect CH from overload
			MemoryReturnInterval: 0,
		},
		ClickHouse: ClickHouse{
			URL:                 "http://localhost:8123?cancel_http_readonly_queries_on_client_close=1",
			ExtraPrefix:         "",
			ConnectTimeout:      time.Second,
			DataTableLegacy:     "",
			RollupConfLegacy:    "auto",
			MaxDataPoints:       1048576,
			InternalAggregation: true,
		},
		Tags: Tags{},
	}

	out, err := toml.Marshal(cfg)
	if err != nil {
		panic(err)
	}
	err = toml.Unmarshal(out, &cfg)
	if err != nil {
		panic(err)
	}

	fmt.Println(string(out))

	// Output:
	// [common]
	// # general listener
	// listen = ':9090'
	// # listener to serve /debug/pprof requests. '-pprof' argument overrides it
	// pprof-listen = ''
	// # limit numbers of queried metrics per target in /render requests, 0 or negative = unlimited
	// max-metrics-per-target = 15000
	// # daemon will return the freed memory to the OS when it>0
	// memory-return-interval = 0
	//
	// [clickhouse]
	// # default url, see https://clickhouse.tech/docs/en/interfaces/http. Can be overwritten with query-params
	// url = 'http://localhost:8123?cancel_http_readonly_queries_on_client_close=1'
	// # Max queries to render queiries
	// render-max-queries = 0
	// # Concurrent queries to render queiries
	// render-concurrent-queries = 0
	// # tree-table = ''
	// # reverse-tree-table = ''
	// # date-tree-table = ''
	// # date-tree-table-version = 0
	// # tree-timeout = 0
	// # tag-table = ''
	// # add extra prefix (directory in graphite) for all metrics, w/o trailing dot
	// extra-prefix = ''
	// # TCP connection timeout
	// connect-timeout = 1000000000
	// # data-table = ''
	// # rollup-conf = 'auto'
	// # max points per metric when internal-aggregation=true
	// max-data-points = 1048576
	// # ClickHouse-side aggregation, see doc/aggregation.md
	// internal-aggregation = true
	//
	// # [tags]
	// # rules = ''
	// # date = ''
	// # extra-where = ''
	// # input-file = ''
	// # output-file = ''
}

func TestReadmeComments(t *testing.T) {
	type TLS struct {
		Cipher  string `toml:"cipher"`
		Version string `toml:"version"`
	}
	type Config struct {
		Host string `toml:"host" comment:"Host IP to connect to."`
		Port int    `toml:"port" comment:"Port of the remote server."`
		Tls  TLS    `toml:"TLS,commented" comment:"Encryption parameters (optional)"`
	}
	example := Config{
		Host: "127.0.0.1",
		Port: 4242,
		Tls: TLS{
			Cipher:  "AEAD-AES128-GCM-SHA256",
			Version: "TLS 1.3",
		},
	}
	out, err := toml.Marshal(example)
	require.NoError(t, err)

	expected := `# Host IP to connect to.
host = '127.0.0.1'
# Port of the remote server.
port = 4242

# Encryption parameters (optional)
# [TLS]
# cipher = 'AEAD-AES128-GCM-SHA256'
# version = 'TLS 1.3'
`
	require.Equal(t, expected, string(out))
}
