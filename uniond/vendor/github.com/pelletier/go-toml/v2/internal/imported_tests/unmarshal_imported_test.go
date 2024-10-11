package imported_tests

// Those tests were imported directly from go-toml v1
// https://raw.githubusercontent.com/pelletier/go-toml/a2e52561804c6cd9392ebf0048ca64fe4af67a43/marshal_test.go
// They have been cleaned up to only include Unmarshal tests, and only depend
// on the public API. Tests related to strict mode have been commented out and
// marked as skipped until we figure out if that's something we want in v2.

import (
	"bytes"
	"errors"
	"fmt"
	"reflect"
	"strconv"
	"testing"
	"time"

	"github.com/pelletier/go-toml/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

type basicMarshalTestStruct struct {
	String     string   `toml:"Zstring"`
	StringList []string `toml:"Ystrlist"`
	BasicMarshalTestSubAnonymousStruct
	Sub     basicMarshalTestSubStruct   `toml:"Xsubdoc"`
	SubList []basicMarshalTestSubStruct `toml:"Wsublist"`
}

type basicMarshalTestSubStruct struct {
	String2 string
}

type BasicMarshalTestSubAnonymousStruct struct {
	String3 string
}

var basicTestData = basicMarshalTestStruct{
	String:                             "Hello",
	StringList:                         []string{"Howdy", "Hey There"},
	BasicMarshalTestSubAnonymousStruct: BasicMarshalTestSubAnonymousStruct{"One"},
	Sub:                                basicMarshalTestSubStruct{"Two"},
	SubList:                            []basicMarshalTestSubStruct{{"Three"}, {"Four"}},
}

var basicTestToml = []byte(`String3 = "One"
Ystrlist = ["Howdy", "Hey There"]
Zstring = "Hello"

[[Wsublist]]
  String2 = "Three"

[[Wsublist]]
  String2 = "Four"

[Xsubdoc]
  String2 = "Two"
`)

var marshalTestToml = []byte(`title = "TOML Marshal Testing"

[basic]
  bool = true
  date = 1979-05-27T07:32:00Z
  float = 123.4
  float64 = 123.456782132399
  int = 5000
  string = "Bite me"
  uint = 5001

[basic_lists]
  bools = [true, false, true]
  dates = [1979-05-27T07:32:00Z, 1980-05-27T07:32:00Z]
  floats = [12.3, 45.6, 78.9]
  ints = [8001, 8001, 8002]
  strings = ["One", "Two", "Three"]
  uints = [5002, 5003]

[basic_map]
  one = "one"
  two = "two"

[subdoc]

  [subdoc.first]
    name = "First"

  [subdoc.second]
    name = "Second"

[[subdoclist]]
  name = "List.First"

[[subdoclist]]
  name = "List.Second"

[[subdocptrs]]
  name = "Second"
`)

type Conf struct {
	Name  string
	Age   int
	Inter interface{}
}

type NestedStruct struct {
	FirstName string
	LastName  string
	Age       int
}

var doc = []byte(`Name = "rui"
Age = 18

[Inter]
  FirstName = "wang"
  LastName = "jl"
  Age = 100`)

func TestInterface(t *testing.T) {
	var config Conf
	config.Inter = &NestedStruct{}
	err := toml.Unmarshal(doc, &config)
	require.NoError(t, err)
	expected := Conf{
		Name: "rui",
		Age:  18,
		Inter: map[string]interface{}{
			"FirstName": "wang",
			"LastName":  "jl",
			"Age":       int64(100),
		},
	}
	assert.Equal(t, expected, config)
}

func TestBasicUnmarshal(t *testing.T) {
	result := basicMarshalTestStruct{}
	err := toml.Unmarshal(basicTestToml, &result)
	require.NoError(t, err)
	require.Equal(t, basicTestData, result)
}

type quotedKeyMarshalTestStruct struct {
	String  string                      `toml:"Z.string-àéù"`
	Float   float64                     `toml:"Yfloat-𝟘"`
	Sub     basicMarshalTestSubStruct   `toml:"Xsubdoc-àéù"`
	SubList []basicMarshalTestSubStruct `toml:"W.sublist-𝟘"`
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var quotedKeyMarshalTestData = quotedKeyMarshalTestStruct{
	String:  "Hello",
	Float:   3.5,
	Sub:     basicMarshalTestSubStruct{"One"},
	SubList: []basicMarshalTestSubStruct{{"Two"}, {"Three"}},
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var quotedKeyMarshalTestToml = []byte(`"Yfloat-𝟘" = 3.5
"Z.string-àéù" = "Hello"

[["W.sublist-𝟘"]]
  String2 = "Two"

[["W.sublist-𝟘"]]
  String2 = "Three"

["Xsubdoc-àéù"]
  String2 = "One"
`)

type testDoc struct {
	Title       string            `toml:"title"`
	BasicLists  testDocBasicLists `toml:"basic_lists"`
	SubDocPtrs  []*testSubDoc     `toml:"subdocptrs"`
	BasicMap    map[string]string `toml:"basic_map"`
	Subdocs     testDocSubs       `toml:"subdoc"`
	Basics      testDocBasics     `toml:"basic"`
	SubDocList  []testSubDoc      `toml:"subdoclist"`
	err         int               `toml:"shouldntBeHere"` // nolint:structcheck,unused
	unexported  int               `toml:"shouldntBeHere"`
	Unexported2 int               `toml:"-"`
}

type testMapDoc struct {
	Title    string            `toml:"title"`
	BasicMap map[string]string `toml:"basic_map"`
	LongMap  map[string]string `toml:"long_map"`
}

type testDocBasics struct {
	Uint       uint      `toml:"uint"`
	Bool       bool      `toml:"bool"`
	Float32    float32   `toml:"float"`
	Float64    float64   `toml:"float64"`
	Int        int       `toml:"int"`
	String     *string   `toml:"string"`
	Date       time.Time `toml:"date"`
	unexported int       `toml:"shouldntBeHere"`
}

type testDocBasicLists struct {
	Floats  []*float32  `toml:"floats"`
	Bools   []bool      `toml:"bools"`
	Dates   []time.Time `toml:"dates"`
	Ints    []int       `toml:"ints"`
	UInts   []uint      `toml:"uints"`
	Strings []string    `toml:"strings"`
}

type testDocSubs struct {
	Second *testSubDoc `toml:"second"`
	First  testSubDoc  `toml:"first"`
}

type testSubDoc struct {
	Name       string `toml:"name"`
	unexported int    `toml:"shouldntBeHere"`
}

var (
	biteMe         = "Bite me"
	float1 float32 = 12.3
	float2 float32 = 45.6
	float3 float32 = 78.9
	subdoc         = testSubDoc{"Second", 0}
)

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
		Bools: []bool{true, false, true},
		Dates: []time.Time{
			time.Date(1979, 5, 27, 7, 32, 0, 0, time.UTC),
			time.Date(1980, 5, 27, 7, 32, 0, 0, time.UTC),
		},
		Floats:  []*float32{&float1, &float2, &float3},
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

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var mapTestDoc = testMapDoc{
	Title: "TOML Marshal Testing",
	BasicMap: map[string]string{
		"one": "one",
		"two": "two",
	},
	LongMap: map[string]string{
		"h1":  "8",
		"i2":  "9",
		"b3":  "2",
		"d4":  "4",
		"f5":  "6",
		"e6":  "5",
		"a7":  "1",
		"c8":  "3",
		"j9":  "10",
		"g10": "7",
	},
}

func TestDocUnmarshal(t *testing.T) {
	result := testDoc{}
	err := toml.Unmarshal(marshalTestToml, &result)
	expected := docData
	require.NoError(t, err)
	assert.Equal(t, expected, result)
}

type unexportedMarshalTestStruct struct {
	String      string                      `toml:"string"`
	StringList  []string                    `toml:"strlist"`
	Sub         basicMarshalTestSubStruct   `toml:"subdoc"`
	SubList     []basicMarshalTestSubStruct `toml:"sublist"`
	unexported  int                         `toml:"shouldntBeHere"`
	Unexported2 int                         `toml:"-"`
}

var unexportedTestData = unexportedMarshalTestStruct{
	String:      "Hello",
	StringList:  []string{"Howdy", "Hey There"},
	Sub:         basicMarshalTestSubStruct{"One"},
	SubList:     []basicMarshalTestSubStruct{{"Two"}, {"Three"}},
	unexported:  0,
	Unexported2: 0,
}

var unexportedTestToml = []byte(`string = "Hello"
strlist = ["Howdy","Hey There"]
unexported = 1
shouldntBeHere = 2

[subdoc]
  String2 = "One"

[[sublist]]
  String2 = "Two"

[[sublist]]
  String2 = "Three"
`)

func TestUnexportedUnmarshal(t *testing.T) {
	result := unexportedMarshalTestStruct{}
	err := toml.Unmarshal(unexportedTestToml, &result)
	require.NoError(t, err)
	assert.Equal(t, unexportedTestData, result)
}

type errStruct struct {
	Bool   bool      `toml:"bool"`
	Date   time.Time `toml:"date"`
	Float  float64   `toml:"float"`
	Int    int16     `toml:"int"`
	String *string   `toml:"string"`
}

type mapErr struct {
	Vals map[string]float64
}

type intErr struct {
	Int1  int
	Int2  int8
	Int3  int16
	Int4  int32
	Int5  int64
	UInt1 uint
	UInt2 uint8
	UInt3 uint16
	UInt4 uint32
	UInt5 uint64
	Flt1  float32
	Flt2  float64
}

var intErrTomls = []string{
	"Int1 = []\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = []\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = []\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = []\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = []\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = []\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = []\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = []\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = []\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = []\nFlt1 = 1.0\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = []\nFlt2 = 2.0",
	"Int1 = 1\nInt2 = 2\nInt3 = 3\nInt4 = 4\nInt5 = 5\nUInt1 = 1\nUInt2 = 2\nUInt3 = 3\nUInt4 = 4\nUInt5 = 5\nFlt1 = 1.0\nFlt2 = []",
}

func TestErrUnmarshal(t *testing.T) {
	errTomls := []string{
		"bool = truly\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:3200Z\nfloat = 123.4\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123a4\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = j000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = 5000\nstring = Bite me",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = 5000\nstring = Bite me",
		"bool = 1\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1\nfloat = 123.4\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\n\"sorry\"\nint = 5000\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = \"sorry\"\nstring = \"Bite me\"",
		"bool = true\ndate = 1979-05-27T07:32:00Z\nfloat = 123.4\nint = 5000\nstring = 1",
	}

	for ind, x := range errTomls {
		t.Run(fmt.Sprintf("Base Case %d", ind), func(t *testing.T) {
			result := errStruct{}
			err := toml.Unmarshal([]byte(x), &result)
			if err == nil {
				t.Errorf("Expected err from case %d\n", ind)
			}
		})
	}
	result2 := mapErr{}
	err := toml.Unmarshal([]byte("[Vals]\nfred=\"1.2\""), &result2)
	if err == nil {
		t.Errorf("Expected err from map")
	}
	for ind, x := range intErrTomls {
		result3 := intErr{}
		err := toml.Unmarshal([]byte(x), &result3)
		if err == nil {
			t.Errorf("Expected int err from case %d\n", ind)
		}
	}
}

var emptyTestToml = []byte(`bool = false
int = 0
string = ""
stringlist = []
title = "Placeholder"

[map]
`)

func TestEmptytomlUnmarshal(t *testing.T) {
	type emptyMarshalTestStruct struct {
		Title      string                  `toml:"title"`
		Bool       bool                    `toml:"bool"`
		Int        int                     `toml:"int"`
		String     string                  `toml:"string"`
		StringList []string                `toml:"stringlist"`
		Ptr        *basicMarshalTestStruct `toml:"ptr"`
		Map        map[string]string       `toml:"map"`
	}

	emptyTestData := emptyMarshalTestStruct{
		Title:      "Placeholder",
		Bool:       false,
		Int:        0,
		String:     "",
		StringList: []string{},
		Ptr:        nil,
		Map:        nil,
	}

	result := emptyMarshalTestStruct{}
	err := toml.Unmarshal(emptyTestToml, &result)
	require.NoError(t, err)
	assert.Equal(t, emptyTestData, result)
}

type pointerMarshalTestStruct struct {
	Str       *string
	List      *[]string
	ListPtr   *[]*string
	Map       *map[string]string
	MapPtr    *map[string]*string
	EmptyStr  *string
	EmptyList *[]string
	EmptyMap  *map[string]string
	DblPtr    *[]*[]*string
}

var (
	pointerStr      = "Hello"
	pointerList     = []string{"Hello back"}
	pointerListPtr  = []*string{&pointerStr}
	pointerMap      = map[string]string{"response": "Goodbye"}
	pointerMapPtr   = map[string]*string{"alternate": &pointerStr}
	pointerTestData = pointerMarshalTestStruct{
		Str:       &pointerStr,
		List:      &pointerList,
		ListPtr:   &pointerListPtr,
		Map:       &pointerMap,
		MapPtr:    &pointerMapPtr,
		EmptyStr:  nil,
		EmptyList: nil,
		EmptyMap:  nil,
	}
)

var pointerTestToml = []byte(`List = ["Hello back"]
ListPtr = ["Hello"]
Str = "Hello"

[Map]
  response = "Goodbye"

[MapPtr]
  alternate = "Hello"
`)

func TestPointerUnmarshal(t *testing.T) {
	result := pointerMarshalTestStruct{}
	err := toml.Unmarshal(pointerTestToml, &result)
	require.NoError(t, err)
	assert.Equal(t, pointerTestData, result)
}

func TestUnmarshalTypeMismatch(t *testing.T) {
	result := pointerMarshalTestStruct{}
	err := toml.Unmarshal([]byte("List = 123"), &result)
	assert.Error(t, err)
}

type nestedMarshalTestStruct struct {
	String [][]string
	// Struct [][]basicMarshalTestSubStruct
	StringPtr *[]*[]*string
	// StructPtr *[]*[]*basicMarshalTestSubStruct
}

var (
	str1    = "Three"
	str2    = "Four"
	strPtr  = []*string{&str1, &str2}
	strPtr2 = []*[]*string{&strPtr}
)

var nestedTestData = nestedMarshalTestStruct{
	String:    [][]string{{"Five", "Six"}, {"One", "Two"}},
	StringPtr: &strPtr2,
}

var nestedTestToml = []byte(`String = [["Five", "Six"], ["One", "Two"]]
StringPtr = [["Three", "Four"]]
`)

func TestNestedUnmarshal(t *testing.T) {
	result := nestedMarshalTestStruct{}
	err := toml.Unmarshal(nestedTestToml, &result)
	require.NoError(t, err)
	assert.Equal(t, nestedTestData, result)
}

type customMarshalerParent struct {
	Self    customMarshaler   `toml:"me"`
	Friends []customMarshaler `toml:"friends"`
}

type customMarshaler struct {
	FirstName string
	LastName  string
}

func (c customMarshaler) MarshalTOML() ([]byte, error) {
	fullName := fmt.Sprintf("%s %s", c.FirstName, c.LastName)
	return []byte(fullName), nil
}

var customMarshalerData = customMarshaler{FirstName: "Sally", LastName: "Fields"}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var customMarshalerToml = []byte(`Sally Fields`)

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var nestedCustomMarshalerData = customMarshalerParent{
	Self:    customMarshaler{FirstName: "Maiku", LastName: "Suteda"},
	Friends: []customMarshaler{customMarshalerData},
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var nestedCustomMarshalerToml = []byte(`friends = ["Sally Fields"]
me = "Maiku Suteda"
`)

var nestedCustomMarshalerTomlForUnmarshal = []byte(`[friends]
FirstName = "Sally"
LastName = "Fields"`)

type IntOrString string

func (x *IntOrString) MarshalTOML() ([]byte, error) {
	s := *(*string)(x)
	_, err := strconv.Atoi(s)
	if err != nil {
		return []byte(fmt.Sprintf(`"%s"`, s)), nil
	}
	return []byte(s), nil
}

func TestUnmarshalTextMarshaler(t *testing.T) {
	nested := struct {
		Friends textMarshaler `toml:"friends"`
	}{}

	expected := struct {
		Friends textMarshaler `toml:"friends"`
	}{
		Friends: textMarshaler{FirstName: "Sally", LastName: "Fields"},
	}

	err := toml.Unmarshal(nestedCustomMarshalerTomlForUnmarshal, &nested)
	if err != nil {
		t.Fatal(err)
	}
	if !reflect.DeepEqual(nested, expected) {
		t.Errorf("Bad unmarshal: expected %v, got %v", expected, nested)
	}
}

// TODO: Remove nolint once type and methods are used by a test
//
//nolint:unused
type precedentMarshaler struct {
	FirstName string
	LastName  string
}

//nolint:unused
func (m precedentMarshaler) MarshalText() ([]byte, error) {
	return []byte("shadowed"), nil
}

//nolint:unused
func (m precedentMarshaler) MarshalTOML() ([]byte, error) {
	fullName := fmt.Sprintf("%s %s", m.FirstName, m.LastName)
	return []byte(fullName), nil
}

// TODO: Remove nolint once type and method are used by a test
//
//nolint:unused
type customPointerMarshaler struct {
	FirstName string
	LastName  string
}

//nolint:unused
func (m *customPointerMarshaler) MarshalTOML() ([]byte, error) {
	return []byte(`"hidden"`), nil
}

// TODO: Remove nolint once type and method are used by a test
//
//nolint:unused
type textPointerMarshaler struct {
	FirstName string
	LastName  string
}

//nolint:unused
func (m *textPointerMarshaler) MarshalText() ([]byte, error) {
	return []byte("hidden"), nil
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var commentTestToml = []byte(`
# it's a comment on type
[postgres]
  # isCommented = "dvalue"
  noComment = "cvalue"

  # A comment on AttrB with a
  # break line
  password = "bvalue"

  # A comment on AttrA
  user = "avalue"

  [[postgres.My]]

    # a comment on my on typeC
    My = "Foo"

  [[postgres.My]]

    # a comment on my on typeC
    My = "Baar"
`)

type mapsTestStruct struct {
	Simple map[string]string
	Paths  map[string]string
	Other  map[string]float64
	X      struct {
		Y struct {
			Z map[string]bool
		}
	}
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var mapsTestData = mapsTestStruct{
	Simple: map[string]string{
		"one plus one": "two",
		"next":         "three",
	},
	Paths: map[string]string{
		"/this/is/a/path": "/this/is/also/a/path",
		"/heloo.txt":      "/tmp/lololo.txt",
	},
	Other: map[string]float64{
		"testing": 3.9999,
	},
	X: struct{ Y struct{ Z map[string]bool } }{
		Y: struct{ Z map[string]bool }{
			Z: map[string]bool{
				"is.Nested": true,
			},
		},
	},
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var mapsTestToml = []byte(`
[Other]
  "testing" = 3.9999

[Paths]
  "/heloo.txt" = "/tmp/lololo.txt"
  "/this/is/a/path" = "/this/is/also/a/path"

[Simple]
  "next" = "three"
  "one plus one" = "two"

[X]

  [X.Y]

    [X.Y.Z]
      "is.Nested" = true
`)

// TODO: Remove nolint once type is used by a test
//
//nolint:deadcode,unused
type structArrayNoTag struct {
	A struct {
		B []int64
		C []int64
	}
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var customTagTestToml = []byte(`
[postgres]
  password = "bvalue"
  user = "avalue"

  [[postgres.My]]
    My = "Foo"

  [[postgres.My]]
    My = "Baar"
`)

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var customCommentTagTestToml = []byte(`
# db connection
[postgres]

  # db pass
  password = "bvalue"

  # db user
  user = "avalue"
`)

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var customCommentedTagTestToml = []byte(`
[postgres]
  # password = "bvalue"
  # user = "avalue"
`)

func TestUnmarshalTabInStringAndQuotedKey(t *testing.T) {
	type Test struct {
		Field1 string `toml:"Fie	ld1"`
		Field2 string
	}

	type TestCase struct {
		desc     string
		input    []byte
		expected Test
	}

	testCases := []TestCase{
		{
			desc:  "multiline string with tab",
			input: []byte("Field2 = \"\"\"\nhello\tworld\"\"\""),
			expected: Test{
				Field2: "hello\tworld",
			},
		},
		{
			desc:  "quoted key with tab",
			input: []byte("\"Fie\tld1\" = \"key with tab\""),
			expected: Test{
				Field1: "key with tab",
			},
		},
		{
			desc:  "basic string tab",
			input: []byte("Field2 = \"hello\tworld\""),
			expected: Test{
				Field2: "hello\tworld",
			},
		},
	}

	for _, test := range testCases {
		t.Run(test.desc, func(t *testing.T) {
			result := Test{}
			err := toml.Unmarshal(test.input, &result)
			require.NoError(t, err)
			assert.Equal(t, test.expected, result)
		})
	}
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var customMultilineTagTestToml = []byte(`int_slice = [
  1,
  2,
  3,
]
`)

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var testDocBasicToml = []byte(`
[document]
  bool_val = true
  date_val = 1979-05-27T07:32:00Z
  float_val = 123.4
  int_val = 5000
  string_val = "Bite me"
  uint_val = 5001
`)

// TODO: Remove nolint once type is used by a test
//
//nolint:deadcode
type testDocCustomTag struct {
	Doc testDocBasicsCustomTag `file:"document"`
}

// TODO: Remove nolint once type is used by a test
//
//nolint:deadcode
type testDocBasicsCustomTag struct {
	Bool       bool      `file:"bool_val"`
	Date       time.Time `file:"date_val"`
	Float      float32   `file:"float_val"`
	Int        int       `file:"int_val"`
	Uint       uint      `file:"uint_val"`
	String     *string   `file:"string_val"`
	unexported int       `file:"shouldntBeHere"`
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,varcheck
var testDocCustomTagData = testDocCustomTag{
	Doc: testDocBasicsCustomTag{
		Bool:       true,
		Date:       time.Date(1979, 5, 27, 7, 32, 0, 0, time.UTC),
		Float:      123.4,
		Int:        5000,
		Uint:       5001,
		String:     &biteMe,
		unexported: 0,
	},
}

func TestUnmarshalMap(t *testing.T) {
	testToml := []byte(`
		a = 1
		b = 2
		c = 3
		`)
	var result map[string]int
	err := toml.Unmarshal(testToml, &result)
	if err != nil {
		t.Errorf("Received unexpected error: %s", err)
		return
	}

	expected := map[string]int{
		"a": 1,
		"b": 2,
		"c": 3,
	}

	if !reflect.DeepEqual(result, expected) {
		t.Errorf("Bad unmarshal: expected %v, got %v", expected, result)
	}
}

func TestUnmarshalMapWithTypedKey(t *testing.T) {
	testToml := []byte(`
		a = 1
		b = 2
		c = 3
		`)

	type letter string
	var result map[letter]int
	err := toml.Unmarshal(testToml, &result)
	if err != nil {
		t.Errorf("Received unexpected error: %s", err)
		return
	}

	expected := map[letter]int{
		"a": 1,
		"b": 2,
		"c": 3,
	}

	if !reflect.DeepEqual(result, expected) {
		t.Errorf("Bad unmarshal: expected %v, got %v", expected, result)
	}
}

func TestUnmarshalTypeTableHeader(t *testing.T) {
	testToml := []byte(`
		[test]
		a = 1
		`)

	type header string
	var result map[header]map[string]int
	err := toml.Unmarshal(testToml, &result)
	if err != nil {
		t.Errorf("Received unexpected error: %s", err)
		return
	}

	expected := map[header]map[string]int{
		"test": map[string]int{"a": 1},
	}

	if !reflect.DeepEqual(result, expected) {
		t.Errorf("Bad unmarshal: expected %v, got %v", expected, result)
	}
}

func TestUnmarshalNonPointer(t *testing.T) {
	a := 1
	err := toml.Unmarshal([]byte{}, a)
	if err == nil {
		t.Fatal("unmarshal should err when given a non pointer")
	}
}

func TestUnmarshalInvalidPointerKind(t *testing.T) {
	t.Skipf("should this really be an error?")
	a := 1
	err := toml.Unmarshal([]byte{}, &a)
	assert.Error(t, err)
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused
type testDuration struct {
	Nanosec   time.Duration  `toml:"nanosec"`
	Microsec1 time.Duration  `toml:"microsec1"`
	Microsec2 *time.Duration `toml:"microsec2"`
	Millisec  time.Duration  `toml:"millisec"`
	Sec       time.Duration  `toml:"sec"`
	Min       time.Duration  `toml:"min"`
	Hour      time.Duration  `toml:"hour"`
	Mixed     time.Duration  `toml:"mixed"`
	AString   string         `toml:"a_string"`
}

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var testDurationToml = []byte(`
nanosec = "1ns"
microsec1 = "1us"
microsec2 = "1µs"
millisec = "1ms"
sec = "1s"
min = "1m"
hour = "1h"
mixed = "1h1m1s1ms1µs1ns"
a_string = "15s"
`)

// TODO: Remove nolint once var is used by a test
//
//nolint:deadcode,unused,varcheck
var testDurationToml2 = []byte(`a_string = "15s"
hour = "1h0m0s"
microsec1 = "1µs"
microsec2 = "1µs"
millisec = "1ms"
min = "1m0s"
mixed = "1h1m1.001001001s"
nanosec = "1ns"
sec = "1s"
`)

// TODO: Remove nolint once type is used by a test
//
//nolint:deadcode,unused
type testBadDuration struct {
	Val time.Duration `toml:"val"`
}

// TODO: add back camelCase test
var testCamelCaseKeyToml = []byte(`fooBar = 10`) //nolint:unused

//nolint:unused
func TestUnmarshalCamelCaseKey(t *testing.T) {
	t.Skipf("don't know if it is a good idea to automatically convert like that yet")
	var x struct {
		FooBar int
		B      int
	}

	if err := toml.Unmarshal(testCamelCaseKeyToml, &x); err != nil {
		t.Fatal(err)
	}

	if x.FooBar != 10 {
		t.Fatal("Did not set camelCase'd key")
	}
}

func TestUnmarshalNegativeUint(t *testing.T) {
	t.Skipf("not sure if we this should always error")
	type check struct{ U uint } // nolint:unused
	err := toml.Unmarshal([]byte("U = -1"), &check{})
	assert.Error(t, err)
}

func TestUnmarshalCheckConversionFloatInt(t *testing.T) {
	type conversionCheck struct {
		U uint
		I int
		F float64
	}

	type TestCase struct {
		desc  string
		input string
	}

	testCases := []TestCase{
		{
			desc:  "unsigned int",
			input: `U = 1e300`,
		},
		{
			desc:  "int",
			input: `I = 1e300`,
		},
	}

	for _, test := range testCases {
		t.Run(test.desc, func(t *testing.T) {
			err := toml.Unmarshal([]byte(test.input), &conversionCheck{})
			require.Error(t, err)
		})
	}
}

func TestUnmarshalOverflow(t *testing.T) {
	type overflow struct {
		U8  uint8
		I8  int8
		F32 float32
	}

	type TestCase struct {
		desc  string
		input string
	}

	testCases := []TestCase{
		{
			desc:  "byte",
			input: `U8 = 300`,
		},
		{
			desc:  "int8",
			input: `I8 = 300`,
		},
		{
			desc:  "float32",
			input: `F32 = 1e300`,
		},
	}

	for _, test := range testCases {
		t.Run(test.desc, func(t *testing.T) {
			err := toml.Unmarshal([]byte(test.input), &overflow{})
			require.Error(t, err)
		})
	}
}

func TestUnmarshalDefault(t *testing.T) {
	t.Skipf("don't know if it is a good idea to have `default`")
	t.Run("main", func(t *testing.T) {
		type EmbeddedStruct struct {
			StringField string `default:"c"`
		}

		type aliasUint uint

		var doc struct {
			StringField       string        `default:"a"`
			BoolField         bool          `default:"true"`
			UintField         uint          `default:"1"`
			Uint8Field        uint8         `default:"8"`
			Uint16Field       uint16        `default:"16"`
			Uint32Field       uint32        `default:"32"`
			Uint64Field       uint64        `default:"64"`
			IntField          int           `default:"-1"`
			Int8Field         int8          `default:"-8"`
			Int16Field        int16         `default:"-16"`
			Int32Field        int32         `default:"-32"`
			Int64Field        int64         `default:"-64"`
			Float32Field      float32       `default:"32.1"`
			Float64Field      float64       `default:"64.1"`
			DurationField     time.Duration `default:"120ms"`
			DurationField2    time.Duration `default:"120000000"`
			NonEmbeddedStruct struct {
				StringField string `default:"b"`
			}
			EmbeddedStruct
			AliasUintField aliasUint `default:"1000"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err != nil {
			t.Fatal(err)
		}
		if doc.BoolField != true {
			t.Errorf("BoolField should be true, not %t", doc.BoolField)
		}
		if doc.StringField != "a" {
			t.Errorf("StringField should be \"a\", not %s", doc.StringField)
		}
		if doc.UintField != 1 {
			t.Errorf("UintField should be 1, not %d", doc.UintField)
		}
		if doc.Uint8Field != 8 {
			t.Errorf("Uint8Field should be 8, not %d", doc.Uint8Field)
		}
		if doc.Uint16Field != 16 {
			t.Errorf("Uint16Field should be 16, not %d", doc.Uint16Field)
		}
		if doc.Uint32Field != 32 {
			t.Errorf("Uint32Field should be 32, not %d", doc.Uint32Field)
		}
		if doc.Uint64Field != 64 {
			t.Errorf("Uint64Field should be 64, not %d", doc.Uint64Field)
		}
		if doc.IntField != -1 {
			t.Errorf("IntField should be -1, not %d", doc.IntField)
		}
		if doc.Int8Field != -8 {
			t.Errorf("Int8Field should be -8, not %d", doc.Int8Field)
		}
		if doc.Int16Field != -16 {
			t.Errorf("Int16Field should be -16, not %d", doc.Int16Field)
		}
		if doc.Int32Field != -32 {
			t.Errorf("Int32Field should be -32, not %d", doc.Int32Field)
		}
		if doc.Int64Field != -64 {
			t.Errorf("Int64Field should be -64, not %d", doc.Int64Field)
		}
		if doc.Float32Field != 32.1 {
			t.Errorf("Float32Field should be 32.1, not %f", doc.Float32Field)
		}
		if doc.Float64Field != 64.1 {
			t.Errorf("Float64Field should be 64.1, not %f", doc.Float64Field)
		}
		if doc.DurationField != 120*time.Millisecond {
			t.Errorf("DurationField should be 120ms, not %s", doc.DurationField.String())
		}
		if doc.DurationField2 != 120*time.Millisecond {
			t.Errorf("DurationField2 should be 120000000, not %d", doc.DurationField2)
		}
		if doc.NonEmbeddedStruct.StringField != "b" {
			t.Errorf("StringField should be \"b\", not %s", doc.NonEmbeddedStruct.StringField)
		}
		if doc.EmbeddedStruct.StringField != "c" {
			t.Errorf("StringField should be \"c\", not %s", doc.EmbeddedStruct.StringField)
		}
		if doc.AliasUintField != 1000 {
			t.Errorf("AliasUintField should be 1000, not %d", doc.AliasUintField)
		}
	})

	t.Run("failure bool", func(t *testing.T) {
		var doc struct {
			Field bool `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})

	t.Run("failure int", func(t *testing.T) {
		var doc struct {
			Field int `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})

	t.Run("failure int64", func(t *testing.T) {
		var doc struct {
			Field int64 `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})

	t.Run("failure float64", func(t *testing.T) {
		var doc struct {
			Field float64 `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})

	t.Run("failure duration", func(t *testing.T) {
		var doc struct {
			Field time.Duration `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})

	t.Run("failure unsupported", func(t *testing.T) {
		var doc struct {
			Field struct{} `default:"blah"`
		}

		err := toml.Unmarshal([]byte(``), &doc)
		if err == nil {
			t.Fatal("should error")
		}
	})
}

func TestUnmarshalNestedAnonymousStructs(t *testing.T) {
	type Nested struct {
		Value string `toml:"nested_field"`
	}
	type Deep struct {
		Nested
	}
	type Document struct {
		Deep
		Value string `toml:"own_field"`
	}

	var doc Document

	err := toml.Unmarshal([]byte(`nested_field = "nested value"`+"\n"+`own_field = "own value"`), &doc)
	if err != nil {
		t.Fatal("should not error")
	}
	if doc.Value != "own value" || doc.Nested.Value != "nested value" {
		t.Fatal("unexpected values")
	}
}

func TestUnmarshalNestedAnonymousStructs_Controversial(t *testing.T) {
	t.Skipf("TODO: what does encoding/json do?")
	type Nested struct {
		Value string `toml:"nested"`
	}
	type Deep struct {
		Nested
	}
	type Document struct {
		Deep
		Value string `toml:"own"`
	}

	var doc Document

	err := toml.Unmarshal([]byte(`nested = "nested value"`+"\n"+`own = "own value"`), &doc)
	if err == nil {
		t.Fatal("should error")
	}
}

type unexportedFieldPreservationTest struct {
	Exported   string `toml:"exported"`
	unexported string
	Nested1    unexportedFieldPreservationTestNested    `toml:"nested1"`
	Nested2    *unexportedFieldPreservationTestNested   `toml:"nested2"`
	Nested3    *unexportedFieldPreservationTestNested   `toml:"nested3"`
	Slice1     []unexportedFieldPreservationTestNested  `toml:"slice1"`
	Slice2     []*unexportedFieldPreservationTestNested `toml:"slice2"`
}

type unexportedFieldPreservationTestNested struct {
	Exported1   string `toml:"exported1"`
	unexported1 string
}

func TestUnmarshalPreservesUnexportedFields(t *testing.T) {
	doc := `
	exported = "visible"
	unexported = "ignored"

	[nested1]
	exported1 = "visible1"
	unexported1 = "ignored1"

	[nested2]
	exported1 = "visible2"
	unexported1 = "ignored2"

	[nested3]
	exported1 = "visible3"
	unexported1 = "ignored3"

	[[slice1]]
	exported1 = "visible3"

	[[slice1]]
	exported1 = "visible4"

	[[slice2]]
	exported1 = "visible5"
	`

	t.Run("unexported field should not be set from toml", func(t *testing.T) {
		var actual unexportedFieldPreservationTest
		err := toml.Unmarshal([]byte(doc), &actual)
		if err != nil {
			t.Fatal("did not expect an error")
		}

		expect := unexportedFieldPreservationTest{
			Exported:   "visible",
			unexported: "",
			Nested1:    unexportedFieldPreservationTestNested{"visible1", ""},
			Nested2:    &unexportedFieldPreservationTestNested{"visible2", ""},
			Nested3:    &unexportedFieldPreservationTestNested{"visible3", ""},
			Slice1: []unexportedFieldPreservationTestNested{
				{Exported1: "visible3"},
				{Exported1: "visible4"},
			},
			Slice2: []*unexportedFieldPreservationTestNested{
				{Exported1: "visible5"},
			},
		}

		if !reflect.DeepEqual(actual, expect) {
			t.Fatalf("%+v did not equal %+v", actual, expect)
		}
	})

	t.Run("unexported field should be preserved", func(t *testing.T) {
		actual := unexportedFieldPreservationTest{
			Exported:   "foo",
			unexported: "bar",
			Nested1:    unexportedFieldPreservationTestNested{"baz", "bax"},
			Nested2:    nil,
			Nested3:    &unexportedFieldPreservationTestNested{"baz", "bax"},
		}
		err := toml.Unmarshal([]byte(doc), &actual)
		if err != nil {
			t.Fatal("did not expect an error")
		}

		expect := unexportedFieldPreservationTest{
			Exported:   "visible",
			unexported: "bar",
			Nested1:    unexportedFieldPreservationTestNested{"visible1", "bax"},
			Nested2:    &unexportedFieldPreservationTestNested{"visible2", ""},
			Nested3:    &unexportedFieldPreservationTestNested{"visible3", "bax"},
			Slice1: []unexportedFieldPreservationTestNested{
				{Exported1: "visible3"},
				{Exported1: "visible4"},
			},
			Slice2: []*unexportedFieldPreservationTestNested{
				{Exported1: "visible5"},
			},
		}

		if !reflect.DeepEqual(actual, expect) {
			t.Fatalf("%+v did not equal %+v", actual, expect)
		}
	})
}

func TestUnmarshalLocalDate(t *testing.T) {
	t.Run("ToLocalDate", func(t *testing.T) {
		type dateStruct struct {
			Date toml.LocalDate
		}

		doc := `date = 1979-05-27`

		var obj dateStruct

		err := toml.Unmarshal([]byte(doc), &obj)
		if err != nil {
			t.Fatal(err)
		}

		if obj.Date.Year != 1979 {
			t.Errorf("expected year 1979, got %d", obj.Date.Year)
		}
		if obj.Date.Month != 5 {
			t.Errorf("expected month 5, got %d", obj.Date.Month)
		}
		if obj.Date.Day != 27 {
			t.Errorf("expected day 27, got %d", obj.Date.Day)
		}
	})

	t.Run("ToLocalDate", func(t *testing.T) {
		type dateStruct struct {
			Date time.Time
		}

		doc := `date = 1979-05-27`

		var obj dateStruct

		err := toml.Unmarshal([]byte(doc), &obj)
		if err != nil {
			t.Fatal(err)
		}

		if obj.Date.Year() != 1979 {
			t.Errorf("expected year 1979, got %d", obj.Date.Year())
		}
		if obj.Date.Month() != 5 {
			t.Errorf("expected month 5, got %d", obj.Date.Month())
		}
		if obj.Date.Day() != 27 {
			t.Errorf("expected day 27, got %d", obj.Date.Day())
		}
	})
}

func TestUnmarshalLocalDateTime(t *testing.T) {
	examples := []struct {
		name string
		in   string
		out  toml.LocalDateTime
	}{
		{
			name: "normal",
			in:   "1979-05-27T07:32:00",
			out: toml.LocalDateTime{
				LocalDate: toml.LocalDate{
					Year:  1979,
					Month: 5,
					Day:   27,
				},
				LocalTime: toml.LocalTime{
					Hour:       7,
					Minute:     32,
					Second:     0,
					Nanosecond: 0,
				},
			},
		},
		{
			name: "with nanoseconds",
			in:   "1979-05-27T00:32:00.999999",
			out: toml.LocalDateTime{
				LocalDate: toml.LocalDate{
					Year:  1979,
					Month: 5,
					Day:   27,
				},
				LocalTime: toml.LocalTime{
					Hour:       0,
					Minute:     32,
					Second:     0,
					Nanosecond: 999999000,
					Precision:  6,
				},
			},
		},
	}

	for i, example := range examples {
		doc := fmt.Sprintf(`date = %s`, example.in)

		t.Run(fmt.Sprintf("ToLocalDateTime_%d_%s", i, example.name), func(t *testing.T) {
			type dateStruct struct {
				Date toml.LocalDateTime
			}

			var obj dateStruct

			err := toml.Unmarshal([]byte(doc), &obj)
			if err != nil {
				t.Fatal(err)
			}

			if obj.Date != example.out {
				t.Errorf("expected '%s', got '%s'", example.out, obj.Date)
			}
		})

		t.Run(fmt.Sprintf("ToTime_%d_%s", i, example.name), func(t *testing.T) {
			type dateStruct struct {
				Date time.Time
			}

			var obj dateStruct

			err := toml.Unmarshal([]byte(doc), &obj)
			if err != nil {
				t.Fatal(err)
			}

			if obj.Date.Year() != example.out.Year {
				t.Errorf("expected year %d, got %d", example.out.Year, obj.Date.Year())
			}
			if obj.Date.Month() != time.Month(example.out.Month) {
				t.Errorf("expected month %d, got %d", example.out.Month, obj.Date.Month())
			}
			if obj.Date.Day() != example.out.Day {
				t.Errorf("expected day %d, got %d", example.out.Day, obj.Date.Day())
			}
			if obj.Date.Hour() != example.out.Hour {
				t.Errorf("expected hour %d, got %d", example.out.Hour, obj.Date.Hour())
			}
			if obj.Date.Minute() != example.out.Minute {
				t.Errorf("expected minute %d, got %d", example.out.Minute, obj.Date.Minute())
			}
			if obj.Date.Second() != example.out.Second {
				t.Errorf("expected second %d, got %d", example.out.Second, obj.Date.Second())
			}
			if obj.Date.Nanosecond() != example.out.Nanosecond {
				t.Errorf("expected nanoseconds %d, got %d", example.out.Nanosecond, obj.Date.Nanosecond())
			}
		})
	}
}

func TestUnmarshalLocalTime(t *testing.T) {
	examples := []struct {
		name string
		in   string
		out  toml.LocalTime
	}{
		{
			name: "normal",
			in:   "07:32:00",
			out: toml.LocalTime{
				Hour:       7,
				Minute:     32,
				Second:     0,
				Nanosecond: 0,
			},
		},
		{
			name: "with nanoseconds",
			in:   "00:32:00.999999",
			out: toml.LocalTime{
				Hour:       0,
				Minute:     32,
				Second:     0,
				Nanosecond: 999999000,
				Precision:  6,
			},
		},
	}

	for i, example := range examples {
		doc := fmt.Sprintf(`Time = %s`, example.in)

		t.Run(fmt.Sprintf("ToLocalTime_%d_%s", i, example.name), func(t *testing.T) {
			type dateStruct struct {
				Time toml.LocalTime
			}

			var obj dateStruct

			err := toml.Unmarshal([]byte(doc), &obj)
			if err != nil {
				t.Fatal(err)
			}

			if obj.Time != example.out {
				t.Errorf("expected '%s', got '%s'", example.out, obj.Time)
			}
		})
	}
}

// test case for issue #339
func TestUnmarshalSameInnerField(t *testing.T) {
	type InterStruct2 struct {
		Test string
		Name string
		Age  int
	}
	type Inter2 struct {
		Name         string
		Age          int
		InterStruct2 InterStruct2
	}
	type Server struct {
		Name   string `toml:"name"`
		Inter2 Inter2 `toml:"inter2"`
	}

	var server Server

	if err := toml.Unmarshal([]byte(`name = "123"
[inter2]
name = "inter2"
age = 222`), &server); err == nil {
		expected := Server{
			Name: "123",
			Inter2: Inter2{
				Name: "inter2",
				Age:  222,
			},
		}
		if !reflect.DeepEqual(server, expected) {
			t.Errorf("Bad unmarshal: expected %v, got %v", expected, server)
		}
	} else {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestUnmarshalToNilInterface(t *testing.T) {
	doc := []byte(`
PrimitiveField = "Hello"
ArrayField = [1,2,3]
InterfacePointerField = "World"

[StructField]
Field1 = 123
Field2 = "Field2"

[MapField]
MapField1 = [4,5,6]
MapField2 = {A = "A"}
MapField3 = false

[[StructArrayField]]
Name = "Allen"
Age = 20

[[StructArrayField]]
Name = "Jack"
Age = 23
`)

	type OuterStruct struct {
		PrimitiveField        interface{}
		ArrayField            interface{}
		StructArrayField      interface{}
		MapField              map[string]interface{}
		StructField           interface{}
		NilField              interface{}
		InterfacePointerField *interface{}
	}

	var s interface{} = "World"
	expected := OuterStruct{
		PrimitiveField: "Hello",
		ArrayField:     []interface{}{int64(1), int64(2), int64(3)},
		StructField: map[string]interface{}{
			"Field1": int64(123),
			"Field2": "Field2",
		},
		MapField: map[string]interface{}{
			"MapField1": []interface{}{int64(4), int64(5), int64(6)},
			"MapField2": map[string]interface{}{
				"A": "A",
			},
			"MapField3": false,
		},
		NilField:              nil,
		InterfacePointerField: &s,
		StructArrayField: []interface{}{
			map[string]interface{}{
				"Name": "Allen",
				"Age":  int64(20),
			},
			map[string]interface{}{
				"Name": "Jack",
				"Age":  int64(23),
			},
		},
	}
	actual := OuterStruct{}
	err := toml.Unmarshal(doc, &actual)
	require.NoError(t, err)
	assert.Equal(t, expected, actual)
}

func TestUnmarshalToNonNilInterface(t *testing.T) {
	doc := []byte(`
PrimitiveField = "Allen"
ArrayField = [1,2,3]

[StructField]
InnerField = "After1"

[PointerField]
InnerField = "After2"

[InterfacePointerField]
InnerField = "After"

[MapField]
MapField1 = [4,5,6]
MapField2 = {A = "A"}
MapField3 = false

[[StructArrayField]]
InnerField = "After3"

[[StructArrayField]]
InnerField = "After4"
`)
	type InnerStruct struct {
		InnerField interface{}
	}

	type OuterStruct struct {
		PrimitiveField        interface{}
		ArrayField            interface{}
		StructArrayField      interface{}
		MapField              map[string]interface{}
		StructField           interface{}
		PointerField          interface{}
		NilField              interface{}
		InterfacePointerField *interface{}
	}

	var s interface{} = InnerStruct{"After"}
	expected := OuterStruct{
		PrimitiveField: "Allen",
		ArrayField:     []interface{}{int64(1), int64(2), int64(3)},
		StructField:    map[string]interface{}{"InnerField": "After1"},
		MapField: map[string]interface{}{
			"MapField1": []interface{}{int64(4), int64(5), int64(6)},
			"MapField2": map[string]interface{}{
				"A": "A",
			},
			"MapField3": false,
		},
		PointerField:          map[string]interface{}{"InnerField": "After2"},
		NilField:              nil,
		InterfacePointerField: &s,
		StructArrayField: []interface{}{
			map[string]interface{}{"InnerField": "After3"},
			map[string]interface{}{"InnerField": "After4"},
		},
	}
	actual := OuterStruct{
		PrimitiveField: "aaa",
		ArrayField:     []int{100, 200, 300, 400},
		StructField:    InnerStruct{InnerField: "Before1"},
		MapField: map[string]interface{}{
			"MapField1": []int{4, 5, 6},
			"MapField2": map[string]string{
				"B": "BBB",
			},
			"MapField3": true,
		},
		PointerField:          &InnerStruct{InnerField: "Before2"},
		NilField:              nil,
		InterfacePointerField: &s,
		StructArrayField: []InnerStruct{
			{InnerField: "Before3"},
			{InnerField: "Before4"},
		},
	}

	err := toml.Unmarshal(doc, &actual)
	require.NoError(t, err)
	assert.Equal(t, expected, actual)
}

func TestUnmarshalNil(t *testing.T) {
	assert.Error(t, toml.Unmarshal([]byte(`whatever = "whatever"`), nil))
	assert.Error(t, toml.Unmarshal([]byte(`whatever = "whatever"`), (*struct{})(nil)))
}

var sliceTomlDemo = []byte(`str_slice = ["Howdy","Hey There"]
str_slice_ptr= ["Howdy","Hey There"]
int_slice=[1,2]
int_slice_ptr=[1,2]
[[struct_slice]]
String2="1"
[[struct_slice]]
String2="2"
[[struct_slice_ptr]]
String2="1"
[[struct_slice_ptr]]
String2="2"
`)

type sliceStruct struct {
	Slice          []string                     `  toml:"str_slice"  `
	SlicePtr       *[]string                    `  toml:"str_slice_ptr"  `
	IntSlice       []int                        `  toml:"int_slice"  `
	IntSlicePtr    *[]int                       `  toml:"int_slice_ptr"  `
	StructSlice    []basicMarshalTestSubStruct  `  toml:"struct_slice"  `
	StructSlicePtr *[]basicMarshalTestSubStruct `  toml:"struct_slice_ptr"  `
}

type arrayStruct struct {
	Slice          [4]string                     `  toml:"str_slice"  `
	SlicePtr       *[4]string                    `  toml:"str_slice_ptr"  `
	IntSlice       [4]int                        `  toml:"int_slice"  `
	IntSlicePtr    *[4]int                       `  toml:"int_slice_ptr"  `
	StructSlice    [4]basicMarshalTestSubStruct  `  toml:"struct_slice"  `
	StructSlicePtr *[4]basicMarshalTestSubStruct `  toml:"struct_slice_ptr"  `
}

type arrayTooSmallStruct struct {
	Slice       [1]string                    `  toml:"str_slice"  `
	StructSlice [1]basicMarshalTestSubStruct `  toml:"struct_slice"  `
}

func TestUnmarshalSlice(t *testing.T) {
	var actual sliceStruct
	err := toml.Unmarshal(sliceTomlDemo, &actual)
	require.NoError(t, err)
	expected := sliceStruct{
		Slice:          []string{"Howdy", "Hey There"},
		SlicePtr:       &[]string{"Howdy", "Hey There"},
		IntSlice:       []int{1, 2},
		IntSlicePtr:    &[]int{1, 2},
		StructSlice:    []basicMarshalTestSubStruct{{"1"}, {"2"}},
		StructSlicePtr: &[]basicMarshalTestSubStruct{{"1"}, {"2"}},
	}
	assert.Equal(t, expected, actual)
}

func TestUnmarshalSliceFail(t *testing.T) {
	var actual sliceStruct
	assert.Error(t, toml.Unmarshal([]byte(`str_slice = [1, 2]`), &actual))
}

func TestUnmarshalSliceFail2(t *testing.T) {
	doc := `str_slice=[1,2]`
	var actual sliceStruct
	assert.Error(t, toml.Unmarshal([]byte(doc), &actual))
}

func TestUnmarshalMixedTypeSlice(t *testing.T) {
	type TestStruct struct {
		ArrayField []interface{}
	}

	//doc := []byte(`ArrayField = [3.14,100,true,"hello world",{Field = "inner1"},[{Field = "inner2"},{Field = "inner3"}]]
	//`)

	doc := []byte(`ArrayField = [{Field = "inner1"},[{Field = "inner2"},{Field = "inner3"}]]
`)

	actual := TestStruct{}
	expected := TestStruct{
		ArrayField: []interface{}{
			//3.14,
			//int64(100),
			//true,
			//"hello world",
			map[string]interface{}{
				"Field": "inner1",
			},
			[]interface{}{
				map[string]interface{}{"Field": "inner2"},
				map[string]interface{}{"Field": "inner3"},
			},
		},
	}
	err := toml.Unmarshal(doc, &actual)
	require.NoError(t, err)
	assert.Equal(t, expected, actual)
}

func TestUnmarshalArray(t *testing.T) {
	var err error

	var actual arrayStruct
	err = toml.Unmarshal(sliceTomlDemo, &actual)
	require.NoError(t, err)

	expected := arrayStruct{
		Slice:          [4]string{"Howdy", "Hey There"},
		SlicePtr:       &[4]string{"Howdy", "Hey There"},
		IntSlice:       [4]int{1, 2},
		IntSlicePtr:    &[4]int{1, 2},
		StructSlice:    [4]basicMarshalTestSubStruct{{"1"}, {"2"}},
		StructSlicePtr: &[4]basicMarshalTestSubStruct{{"1"}, {"2"}},
	}
	assert.Equal(t, expected, actual)
}

func TestUnmarshalArrayFail3(t *testing.T) {
	doc := `[[struct_slice]]
String2="1"
[[struct_slice]]
String2="2"`

	var actual arrayTooSmallStruct
	err := toml.Unmarshal([]byte(doc), &actual)
	assert.Error(t, err)
}

func decoder(doc string) *toml.Decoder {
	return toml.NewDecoder(bytes.NewReader([]byte(doc)))
}

func strictDecoder(doc string) *toml.Decoder {
	d := decoder(doc)
	d.DisallowUnknownFields()
	return d
}

func TestDecoderStrict(t *testing.T) {
	input := `
	[decoded]
	 key = ""
	
	[undecoded]
	 key = ""
	
	 [undecoded.inner]
		key = ""
	
	 [[undecoded.array]]
		key = ""
	
	 [[undecoded.array]]
		key = ""
	
	`
	var doc struct {
		Decoded struct {
			Key string
		}
	}

	err := strictDecoder(input).Decode(&doc)
	require.Error(t, err)
	require.IsType(t, &toml.StrictMissingError{}, err)
	se := err.(*toml.StrictMissingError)

	keys := []toml.Key{}

	for _, e := range se.Errors {
		keys = append(keys, e.Key())
	}

	expectedKeys := []toml.Key{
		{"undecoded"},
		{"undecoded", "inner"},
		{"undecoded", "array"},
		{"undecoded", "array"},
	}

	require.Equal(t, expectedKeys, keys)

	err = decoder(input).Decode(&doc)
	require.NoError(t, err)

	var m map[string]interface{}
	err = decoder(input).Decode(&m)
}

func TestDecoderStrictValid(t *testing.T) {
	input := `
	[decoded]
	 key = ""
	`
	var doc struct {
		Decoded struct {
			Key string
		}
	}

	err := strictDecoder(input).Decode(&doc)
	require.NoError(t, err)
}

type docUnmarshalTOML struct {
	Decoded struct {
		Key string
	}
}

func (d *docUnmarshalTOML) UnmarshalTOML(i interface{}) error {
	if iMap, ok := i.(map[string]interface{}); !ok {
		return fmt.Errorf("type assertion error: wants %T, have %T", map[string]interface{}{}, i)
	} else if key, ok := iMap["key"]; !ok {
		return fmt.Errorf("key '%s' not in map", "key")
	} else if keyString, ok := key.(string); !ok {
		return fmt.Errorf("type assertion error: wants %T, have %T", "", key)
	} else {
		d.Decoded.Key = keyString
	}
	return nil
}

func TestDecoderStrictCustomUnmarshal(t *testing.T) {
	t.Skip()
	//input := `key = "ok"`
	//var doc docUnmarshalTOML
	//err := NewDecoder(bytes.NewReader([]byte(input))).Strict(true).Decode(&doc)
	//if err != nil {
	//	t.Fatal("unexpected error:", err)
	//}
	//if doc.Decoded.Key != "ok" {
	//	t.Errorf("Bad unmarshal: expected ok, got %v", doc.Decoded.Key)
	//}
}

type parent struct {
	Doc        docUnmarshalTOML
	DocPointer *docUnmarshalTOML
}

func TestCustomUnmarshal(t *testing.T) {
	t.Skip("not sure if UnmarshalTOML is a good idea")
	input := `
[Doc]
    key = "ok1"
[DocPointer]
    key = "ok2"
`

	var d parent
	err := toml.Unmarshal([]byte(input), &d)
	require.NoError(t, err)
	assert.Equal(t, "ok1", d.Doc.Decoded.Key)
	assert.Equal(t, "ok2", d.DocPointer.Decoded.Key)
}

func TestCustomUnmarshalError(t *testing.T) {
	t.Skip("not sure if UnmarshalTOML is a good idea")

	input := `
[Doc]
    key = 1
[DocPointer]
    key = "ok2"
`

	expected := "(2, 1): unmarshal toml: type assertion error: wants string, have int64"

	var d parent
	err := toml.Unmarshal([]byte(input), &d)
	if err == nil {
		t.Error("expected error, got none")
	} else if err.Error() != expected {
		t.Errorf("expect err: %s, got: %s", expected, err.Error())
	}
}

type intWrapper struct {
	Value int
}

func (w *intWrapper) UnmarshalText(text []byte) error {
	var err error
	if w.Value, err = strconv.Atoi(string(text)); err == nil {
		return nil
	}
	if b, err := strconv.ParseBool(string(text)); err == nil {
		if b {
			w.Value = 1
		}
		return nil
	}
	if f, err := strconv.ParseFloat(string(text), 32); err == nil {
		w.Value = int(f)
		return nil
	}
	return fmt.Errorf("unsupported: %s", text)
}

func TestTextUnmarshal(t *testing.T) {
	var doc struct {
		UnixTime intWrapper
		Version  *intWrapper

		Bool  intWrapper
		Int   intWrapper
		Float intWrapper
	}

	input := `
UnixTime = "12"
Version = "42"
Bool = true
Int = 21
Float = 2.0
`
	err := toml.Unmarshal([]byte(input), &doc)
	require.NoError(t, err)
	assert.Equal(t, 12, doc.UnixTime.Value)
	assert.Equal(t, 42, doc.Version.Value)
	assert.Equal(t, 1, doc.Bool.Value)
	assert.Equal(t, 21, doc.Int.Value)
	assert.Equal(t, 2, doc.Float.Value)
}

func TestTextUnmarshalError(t *testing.T) {
	var doc struct {
		Failer intWrapper
	}

	input := `Failer = "hello"`
	if err := toml.Unmarshal([]byte(input), &doc); err == nil {
		t.Fatalf("expected err, got none")
	}
}

// issue406
func TestPreserveNotEmptyField(t *testing.T) {
	doc := []byte(`Field1 = "ccc"`)
	type Inner struct {
		InnerField1 string
		InnerField2 int
	}
	type TestStruct struct {
		Field1 string
		Field2 int
		Field3 Inner
	}

	actual := TestStruct{
		"aaa",
		100,
		Inner{
			"bbb",
			200,
		},
	}

	expected := TestStruct{
		"ccc",
		100,
		Inner{
			"bbb",
			200,
		},
	}

	err := toml.Unmarshal(doc, &actual)
	if err != nil {
		t.Fatal(err)
	}

	if !reflect.DeepEqual(actual, expected) {
		t.Errorf("Bad unmarshal: expected %+v, got %+v", expected, actual)
	}
}

// github issue 432
func TestUnmarshalEmptyInterface(t *testing.T) {
	doc := []byte(`User = "pelletier"`)

	var v interface{}

	err := toml.Unmarshal(doc, &v)
	if err != nil {
		t.Fatal(err)
	}
	require.IsType(t, map[string]interface{}{}, v)

	x := v.(map[string]interface{})
	assert.Equal(t, "pelletier", x["User"])
}

func TestUnmarshalEmptyInterfaceDeep(t *testing.T) {
	t.Skipf("TODO")
	doc := []byte(`
User = "pelletier"
Age = 99

[foo]
bar = 42
`)

	var v interface{}

	err := toml.Unmarshal(doc, &v)
	if err != nil {
		t.Fatal(err)
	}

	x, ok := v.(map[string]interface{})
	if !ok {
		t.Fatal(err)
	}

	expected := map[string]interface{}{
		"User": "pelletier",
		"Age":  99,
		"foo": map[string]interface{}{
			"bar": 42,
		},
	}

	reflect.DeepEqual(x, expected)
}

type Config struct {
	Key string `toml:"key"`
	Obj Custom `toml:"obj"`
}

type Custom struct {
	v string
}

func (c *Custom) UnmarshalTOML(v interface{}) error {
	c.v = "called"
	return nil
}

func TestGithubIssue431(t *testing.T) {
	doc := `key = "value"`
	var c Config
	if err := toml.Unmarshal([]byte(doc), &c); err != nil {
		t.Fatalf("unexpected error: %s", err)
	}

	if c.Key != "value" {
		t.Errorf("expected c.Key='value', not '%s'", c.Key)
	}

	if c.Obj.v == "called" {
		t.Errorf("UnmarshalTOML should not have been called")
	}
}

type durationString struct {
	time.Duration
}

func (d *durationString) UnmarshalTOML(v interface{}) error {
	d.Duration = 10 * time.Second
	return nil
}

type config437Error struct{}

func (e *config437Error) UnmarshalTOML(v interface{}) error {
	return errors.New("expected")
}

type config437 struct {
	HTTP struct {
		PingTimeout durationString `toml:"PingTimeout"`
		ErrorField  config437Error
	} `toml:"HTTP"`
}

func TestGithubIssue437(t *testing.T) {
	t.Skipf("unmarshalTOML not implemented")
	src := `
[HTTP]
PingTimeout = "32m"
`
	cfg := &config437{}
	cfg.HTTP.PingTimeout = durationString{time.Second}

	err := toml.Unmarshal([]byte(src), cfg)
	if err != nil {
		t.Fatalf("unexpected errors %s", err)
	}
	expected := durationString{10 * time.Second}
	if cfg.HTTP.PingTimeout != expected {
		t.Fatalf("expected '%s', got '%s'", expected, cfg.HTTP.PingTimeout)
	}
}

func TestLeafUnmarshalerError(t *testing.T) {
	src := `
[HTTP]
ErrorField = "foo"
`
	cfg := &config437{}

	err := toml.Unmarshal([]byte(src), cfg)
	if err == nil {
		t.Fatalf("error was expected")
	}
}
