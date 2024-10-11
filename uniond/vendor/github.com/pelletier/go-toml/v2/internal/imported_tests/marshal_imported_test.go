package imported_tests

// Those tests have been imported from v1, but adjust to match the new
// defaults of v2.

import (
	"fmt"
	"testing"
	"time"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/require"
)

func TestDocMarshal(t *testing.T) {
	type testDoc struct {
		Title       string            `toml:"title"`
		BasicLists  testDocBasicLists `toml:"basic_lists"`
		SubDocPtrs  []*testSubDoc     `toml:"subdocptrs"`
		BasicMap    map[string]string `toml:"basic_map"`
		Subdocs     testDocSubs       `toml:"subdoc"`
		Basics      testDocBasics     `toml:"basic"`
		SubDocList  []testSubDoc      `toml:"subdoclist"`
		err         int               `toml:"shouldntBeHere"`
		unexported  int               `toml:"shouldntBeHere"`
		Unexported2 int               `toml:"-"`
	}

	var docData = testDoc{
		Title:       "TOML Marshal Testing",
		unexported:  0,
		Unexported2: 0,
		Basics: testDocBasics{
			Bool:       true,
			Date:       time.Date(1979, 5, 27, 7, 32, 0, 0, time.UTC),
			Float32:    123.4,
			Float64:    123.456782132399,
			Int:        5000,
			Uint:       5001,
			String:     &biteMe,
			unexported: 0,
		},
		BasicLists: testDocBasicLists{
			Floats: []*float32{&float1, &float2, &float3},
			Bools:  []bool{true, false, true},
			Dates: []time.Time{
				time.Date(1979, 5, 27, 7, 32, 0, 0, time.UTC),
				time.Date(1980, 5, 27, 7, 32, 0, 0, time.UTC),
			},
			Ints:    []int{8001, 8001, 8002},
			Strings: []string{"One", "Two", "Three"},
			UInts:   []uint{5002, 5003},
		},
		BasicMap: map[string]string{
			"one": "one",
			"two": "two",
		},
		Subdocs: testDocSubs{
			First:  testSubDoc{"First", 0},
			Second: &subdoc,
		},
		SubDocList: []testSubDoc{
			{"List.First", 0},
			{"List.Second", 0},
		},
		SubDocPtrs: []*testSubDoc{&subdoc},
	}

	marshalTestToml := `title = 'TOML Marshal Testing'

[basic_lists]
floats = [12.3, 45.6, 78.9]
bools = [true, false, true]
dates = [1979-05-27T07:32:00Z, 1980-05-27T07:32:00Z]
ints = [8001, 8001, 8002]
uints = [5002, 5003]
strings = ['One', 'Two', 'Three']

[[subdocptrs]]
name = 'Second'

[basic_map]
one = 'one'
two = 'two'

[subdoc]
[subdoc.second]
name = 'Second'

[subdoc.first]
name = 'First'

[basic]
uint = 5001
bool = true
float = 123.4
float64 = 123.456782132399
int = 5000
string = 'Bite me'
date = 1979-05-27T07:32:00Z

[[subdoclist]]
name = 'List.First'

[[subdoclist]]
name = 'List.Second'
`

	result, err := toml.Marshal(docData)
	require.NoError(t, err)
	require.Equal(t, marshalTestToml, string(result))
}

func TestBasicMarshalQuotedKey(t *testing.T) {
	result, err := toml.Marshal(quotedKeyMarshalTestData)
	require.NoError(t, err)

	expected := `'Z.string-àéù' = 'Hello'
'Yfloat-𝟘' = 3.5

['Xsubdoc-àéù']
String2 = 'One'

[['W.sublist-𝟘']]
String2 = 'Two'

[['W.sublist-𝟘']]
String2 = 'Three'
`

	require.Equal(t, string(expected), string(result))

}

func TestEmptyMarshal(t *testing.T) {
	type emptyMarshalTestStruct struct {
		Title      string                  `toml:"title"`
		Bool       bool                    `toml:"bool"`
		Int        int                     `toml:"int"`
		String     string                  `toml:"string"`
		StringList []string                `toml:"stringlist"`
		Ptr        *basicMarshalTestStruct `toml:"ptr"`
		Map        map[string]string       `toml:"map"`
	}

	doc := emptyMarshalTestStruct{
		Title:      "Placeholder",
		Bool:       false,
		Int:        0,
		String:     "",
		StringList: []string{},
		Ptr:        nil,
		Map:        map[string]string{},
	}
	result, err := toml.Marshal(doc)
	require.NoError(t, err)

	expected := `title = 'Placeholder'
bool = false
int = 0
string = ''
stringlist = []

[map]
`

	require.Equal(t, string(expected), string(result))
}

type textMarshaler struct {
	FirstName string
	LastName  string
}

func (m textMarshaler) MarshalText() ([]byte, error) {
	fullName := fmt.Sprintf("%s %s", m.FirstName, m.LastName)
	return []byte(fullName), nil
}

func TestTextMarshaler(t *testing.T) {
	type wrap struct {
		TM textMarshaler
	}

	m := textMarshaler{FirstName: "Sally", LastName: "Fields"}

	t.Run("at root", func(t *testing.T) {
		_, err := toml.Marshal(m)
		// in v2 we do not allow TextMarshaler at root
		require.Error(t, err)
	})

	t.Run("leaf", func(t *testing.T) {
		res, err := toml.Marshal(wrap{m})
		require.NoError(t, err)

		require.Equal(t, "TM = 'Sally Fields'\n", string(res))
	})
}
