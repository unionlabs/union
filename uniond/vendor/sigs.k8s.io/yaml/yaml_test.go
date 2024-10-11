/*
Copyright 2021 The Kubernetes Authors.

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

package yaml

import (
	"bytes"
	"encoding/json"
	"fmt"
	"math"
	"reflect"
	"sort"
	"strconv"
	"testing"

	"github.com/google/go-cmp/cmp"
	yamlv2 "sigs.k8s.io/yaml/goyaml.v2"
	yamlv3 "sigs.k8s.io/yaml/goyaml.v3"
)

/* Test helper functions */

func strPtr(str string) *string {
	return &str
}

type errorType int

const (
	noErrorsType    errorType = 0
	fatalErrorsType errorType = 1 << iota
)

type unmarshalTestCase struct {
	encoded    []byte
	decodeInto interface{}
	decoded    interface{}
	err        errorType
}

type testUnmarshalFunc = func(yamlBytes []byte, obj interface{}) error

var (
	funcUnmarshal testUnmarshalFunc = func(yamlBytes []byte, obj interface{}) error {
		return Unmarshal(yamlBytes, obj)
	}

	funcUnmarshalStrict testUnmarshalFunc = func(yamlBytes []byte, obj interface{}) error {
		return UnmarshalStrict(yamlBytes, obj)
	}
)

func testUnmarshal(t *testing.T, f testUnmarshalFunc, tests map[string]unmarshalTestCase) {
	for testName, test := range tests {
		t.Run(testName, func(t *testing.T) {
			typ := reflect.TypeOf(test.decodeInto)
			if typ.Kind() != reflect.Ptr {
				t.Errorf("unmarshalTest.ptr %T is not a pointer type", test.decodeInto)
			}

			value := reflect.New(typ.Elem())

			if !reflect.DeepEqual(test.decodeInto, value.Interface()) {
				// There's no reason for ptr to point to non-zero data,
				// as we decode into new(right-type), so the data is
				// discarded.
				// This can easily mean tests that silently don't test
				// what they should. To test decoding into existing
				// data, see TestPrefilled.
				t.Errorf("unmarshalTest.ptr %#v is not a pointer to a zero value", test.decodeInto)
			}

			err := f(test.encoded, value.Interface())
			if err != nil && test.err == noErrorsType {
				t.Errorf("error unmarshaling YAML: %v", err)
			}
			if err == nil && test.err&fatalErrorsType != 0 {
				t.Errorf("expected a fatal error, but no fatal error was returned, yaml: `%s`", test.encoded)
			}

			if test.err&fatalErrorsType != 0 {
				// Don't check output if error is fatal
				return
			}

			if !reflect.DeepEqual(value.Elem().Interface(), test.decoded) {
				t.Errorf("unmarshal YAML was unsuccessful, expected: %+#v, got: %+#v", test.decoded, value.Elem().Interface())
			}
		})
	}
}

type yamlToJSONTestcase struct {
	yaml string
	json string
	// By default we test that reversing the output == input. But if there is a
	// difference in the reversed output, you can optionally specify it here.
	yamlReverseOverwrite *string
	err                  errorType
}

type testYAMLToJSONFunc = func(yamlBytes []byte) ([]byte, error)

var (
	funcYAMLToJSON testYAMLToJSONFunc = func(yamlBytes []byte) ([]byte, error) {
		return YAMLToJSON(yamlBytes)
	}

	funcYAMLToJSONStrict testYAMLToJSONFunc = func(yamlBytes []byte) ([]byte, error) {
		return YAMLToJSONStrict(yamlBytes)
	}
)

func testYAMLToJSON(t *testing.T, f testYAMLToJSONFunc, tests map[string]yamlToJSONTestcase) {
	for testName, test := range tests {
		t.Run(fmt.Sprintf("%s_YAMLToJSON", testName), func(t *testing.T) {
			// Convert Yaml to Json
			jsonBytes, err := f([]byte(test.yaml))
			if err != nil && test.err == noErrorsType {
				t.Errorf("Failed to convert YAML to JSON, yamlv2: `%s`, err: %v", test.yaml, err)
			}
			if err == nil && test.err&fatalErrorsType != 0 {
				t.Errorf("expected a fatal error, but no fatal error was returned, yaml: `%s`", test.yaml)
			}

			if test.err&fatalErrorsType != 0 {
				// Don't check output if error is fatal
				return
			}

			// Check it against the expected output.
			if string(jsonBytes) != test.json {
				t.Errorf("Failed to convert YAML to JSON, yaml: `%s`, expected json `%s`, got `%s`", test.yaml, test.json, string(jsonBytes))
			}
		})

		t.Run(fmt.Sprintf("%s_JSONToYAML", testName), func(t *testing.T) {
			// Convert JSON to YAML
			yamlBytes, err := JSONToYAML([]byte(test.json))
			if err != nil {
				t.Errorf("Failed to convert JSON to YAML, json: `%s`, err: %v", test.json, err)
			}

			// Set the string that we will compare the reversed output to.
			correctYamlString := test.yaml

			// If a special reverse string was specified, use that instead.
			if test.yamlReverseOverwrite != nil {
				correctYamlString = *test.yamlReverseOverwrite
			}

			// Check it against the expected output.
			if string(yamlBytes) != correctYamlString {
				t.Errorf("Failed to convert JSON to YAML, json: `%s`, expected yaml `%s`, got `%s`", test.json, correctYamlString, string(yamlBytes))
			}
		})
	}
}

/* Start tests */

type MarshalTest struct {
	A string
	B int64
	// Would like to test float64, but it's not supported in go-yaml.
	// (See https://github.com/go-yaml/yaml/issues/83.)
	C float32
}

func TestMarshal(t *testing.T) {
	f32String := strconv.FormatFloat(math.MaxFloat32, 'g', -1, 32)
	s := MarshalTest{"a", math.MaxInt64, math.MaxFloat32}
	e := []byte(fmt.Sprintf("A: a\nB: %d\nC: %s\n", math.MaxInt64, f32String))

	y, err := Marshal(s)
	if err != nil {
		t.Errorf("error marshaling YAML: %v", err)
	}

	if !reflect.DeepEqual(y, e) {
		t.Errorf("marshal YAML was unsuccessful, expected: %#v, got: %#v",
			string(e), string(y))
	}
}

type UnmarshalUntaggedStruct struct {
	A    string
	True string
}

type UnmarshalTaggedStruct struct {
	AUpper            string `json:"A"`
	ALower            string `json:"a"`
	TrueUpper         string `json:"True"`
	TrueLower         string `json:"true"`
	YesUpper          string `json:"Yes"`
	YesLower          string `json:"yes"`
	Int3              string `json:"3"`
	IntBig1           string `json:"9007199254740993"` // 2^53 + 1
	IntBig2           string `json:"1000000000000000000000000000000000000"`
	IntBig2Scientific string `json:"1e+36"`
	Float3dot3        string `json:"3.3"`
}

type UnmarshalStruct struct {
	A string  `json:"a"`
	B *string `json:"b"`
	C string  `json:"c"`
}

type UnmarshalStringMap struct {
	A map[string]string `json:"a"`
}

type UnmarshalNestedStruct struct {
	A UnmarshalStruct `json:"a"`
}

type UnmarshalSlice struct {
	A []UnmarshalStruct `json:"a"`
}

type UnmarshalEmbedStruct struct {
	UnmarshalStruct
	B string `json:"b"`
}

type UnmarshalEmbedStructPointer struct {
	*UnmarshalStruct
	B string `json:"b"`
}

type UnmarshalEmbedRecursiveStruct struct {
	*UnmarshalEmbedRecursiveStruct `json:"a"`
	B                              string `json:"b"`
}

func TestUnmarshal(t *testing.T) {
	tests := map[string]unmarshalTestCase{
		// casematched / non-casematched untagged keys
		"untagged casematched string key": {
			encoded:    []byte("A: test"),
			decodeInto: new(UnmarshalUntaggedStruct),
			decoded:    UnmarshalUntaggedStruct{A: "test"},
		},
		"untagged non-casematched string key": {
			encoded:    []byte("a: test"),
			decodeInto: new(UnmarshalUntaggedStruct),
			decoded:    UnmarshalUntaggedStruct{A: "test"},
		},
		"untagged casematched boolean key": {
			encoded:    []byte("True: test"),
			decodeInto: new(UnmarshalUntaggedStruct),
			decoded:    UnmarshalUntaggedStruct{True: "test"},
		},
		"untagged non-casematched boolean key": {
			encoded:    []byte("true: test"),
			decodeInto: new(UnmarshalUntaggedStruct),
			decoded:    UnmarshalUntaggedStruct{True: "test"},
		},

		// casematched / non-casematched tagged keys
		"tagged casematched string key": {
			encoded:    []byte("A: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{AUpper: "test"},
		},
		"tagged non-casematched string key": {
			encoded:    []byte("a: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{ALower: "test"},
		},
		"tagged casematched boolean key": {
			encoded:    []byte("True: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{TrueLower: "test"},
		},
		"tagged non-casematched boolean key": {
			encoded:    []byte("true: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{TrueLower: "test"},
		},
		"tagged casematched boolean key (yes)": {
			encoded:    []byte("Yes: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{TrueLower: "test"},
		},
		"tagged non-casematched boolean key (yes)": {
			encoded:    []byte("yes: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{TrueLower: "test"},
		},
		"tagged integer key": {
			encoded:    []byte("3: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{Int3: "test"},
		},
		"tagged big integer key 2^53 + 1": {
			encoded:    []byte("9007199254740993: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{IntBig1: "test"},
		},
		"tagged big integer key 1000000000000000000000000000000000000": {
			encoded:    []byte("1000000000000000000000000000000000000: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{IntBig2Scientific: "test"},
		},
		"tagged float key": {
			encoded:    []byte("3.3: test"),
			decodeInto: new(UnmarshalTaggedStruct),
			decoded:    UnmarshalTaggedStruct{Float3dot3: "test"},
		},

		// decode into string field
		"string value into string field": {
			encoded:    []byte("a: test"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "test"},
		},
		"integer value into string field": {
			encoded:    []byte("a: 1"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "1"},
		},
		"boolean value into string field": {
			encoded:    []byte("a: true"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "true"},
		},
		"boolean value (no) into string field": {
			encoded:    []byte("a: no"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "false"},
		},

		// decode into complex fields
		"decode into nested struct": {
			encoded:    []byte("a:\n  a: 1"),
			decodeInto: new(UnmarshalNestedStruct),
			decoded:    UnmarshalNestedStruct{UnmarshalStruct{A: "1"}},
		},
		"decode into slice": {
			encoded:    []byte("a:\n  - a: abc\n    b: def\n  - a: 123"),
			decodeInto: new(UnmarshalSlice),
			decoded:    UnmarshalSlice{[]UnmarshalStruct{{A: "abc", B: strPtr("def")}, {A: "123"}}},
		},
		"decode into string map": {
			encoded:    []byte("a:\n  b: 1"),
			decodeInto: new(UnmarshalStringMap),
			decoded:    UnmarshalStringMap{map[string]string{"b": "1"}},
		},
		"decode into struct pointer map": {
			encoded:    []byte("a:\n  a: TestA\nb:\n  a: TestB\n  b: TestC"),
			decodeInto: new(map[string]*UnmarshalStruct),
			decoded: map[string]*UnmarshalStruct{
				"a": {A: "TestA"},
				"b": {A: "TestB", B: strPtr("TestC")},
			},
		},

		// decoding into string map
		"string map: decode string key": {
			encoded:    []byte("a:"),
			decodeInto: new(map[string]struct{}),
			decoded: map[string]struct{}{
				"a": {},
			},
		},
		"string map: decode boolean key": {
			encoded:    []byte("True:"),
			decodeInto: new(map[string]struct{}),
			decoded: map[string]struct{}{
				"true": {},
			},
		},
		"string map: decode boolean key (yes)": {
			encoded:    []byte("Yes:"),
			decodeInto: new(map[string]struct{}),
			decoded: map[string]struct{}{
				"true": {},
			},
		},
		"string map: decode integer key": {
			encoded:    []byte("44:"),
			decodeInto: new(map[string]struct{}),
			decoded: map[string]struct{}{
				"44": {},
			},
		},
		"string map: decode float key": {
			encoded:    []byte("444.444:"),
			decodeInto: new(map[string]struct{}),
			decoded: map[string]struct{}{
				"444.444": {},
			},
		},

		// decoding integers
		"decode 2^53 + 1 into int": {
			encoded:    []byte("9007199254740993"),
			decodeInto: new(int),
			decoded:    9007199254740993,
		},
		"decode 2^53 + 1 into interface": {
			encoded:    []byte("9007199254740993"),
			decodeInto: new(interface{}),
			decoded:    9.007199254740992e+15,
		},

		// decode into interface
		"float into interface": {
			encoded:    []byte("3.0"),
			decodeInto: new(interface{}),
			decoded:    float64(3),
		},
		"integer into interface": {
			encoded:    []byte("3"),
			decodeInto: new(interface{}),
			decoded:    float64(3),
		},
		"empty vs empty string into interface": {
			encoded:    []byte("a: \"\"\nb: \n"),
			decodeInto: new(interface{}),
			decoded: map[string]interface{}{
				"a": "",
				"b": nil,
			},
		},

		// duplicate (non-casematched) keys (NOTE: this is very non-ideal behaviour!)
		"decode duplicate (non-casematched) into nested struct 1": {
			encoded:    []byte("a:\n  a: 1\n  b: 1\n  c: test\n\nA:\n  a: 2"),
			decodeInto: new(UnmarshalNestedStruct),
			decoded:    UnmarshalNestedStruct{A: UnmarshalStruct{A: "1", B: strPtr("1"), C: "test"}},
		},
		"decode duplicate (non-casematched) into nested struct 2": {
			encoded:    []byte("A:\n  a: 1\n  b: 1\n  c: test\na:\n  a: 2"),
			decodeInto: new(UnmarshalNestedStruct),
			decoded:    UnmarshalNestedStruct{A: UnmarshalStruct{A: "2", B: strPtr("1"), C: "test"}},
		},
		"decode duplicate (non-casematched) into nested slice 1": {
			encoded:    []byte("a:\n  - a: abc\n    b: def\nA:\n  - a: 123"),
			decodeInto: new(UnmarshalSlice),
			decoded:    UnmarshalSlice{[]UnmarshalStruct{{A: "abc", B: strPtr("def")}}},
		},
		"decode duplicate (non-casematched) into nested slice 2": {
			encoded:    []byte("A:\n  - a: abc\n    b: def\na:\n  - a: 123"),
			decodeInto: new(UnmarshalSlice),
			decoded:    UnmarshalSlice{[]UnmarshalStruct{{A: "123", B: strPtr("def")}}},
		},
		"decode duplicate (non-casematched) into nested string map 1": {
			encoded:    []byte("a:\n  b: 1\nA:\n  c: 1"),
			decodeInto: new(UnmarshalStringMap),
			decoded:    UnmarshalStringMap{map[string]string{"b": "1", "c": "1"}},
		},
		"decode duplicate (non-casematched) into nested string map 2": {
			encoded:    []byte("A:\n  b: 1\na:\n  c: 1"),
			decodeInto: new(UnmarshalStringMap),
			decoded:    UnmarshalStringMap{map[string]string{"b": "1", "c": "1"}},
		},
		"decode duplicate (non-casematched) into string map": {
			encoded:    []byte("a: test\nb: test\nA: test2"),
			decodeInto: new(map[string]string),
			decoded: map[string]string{
				"a": "test",
				"A": "test2",
				"b": "test",
			},
		},

		// decoding embeded structs
		"decode embeded struct": {
			encoded:    []byte("a: testA\nb: testB"),
			decodeInto: new(UnmarshalEmbedStruct),
			decoded: UnmarshalEmbedStruct{
				UnmarshalStruct: UnmarshalStruct{
					A: "testA",
				},
				B: "testB",
			},
		},
		"decode embeded structpointer": {
			encoded:    []byte("a: testA\nb: testB"),
			decodeInto: new(UnmarshalEmbedStructPointer),
			decoded: UnmarshalEmbedStructPointer{
				UnmarshalStruct: &UnmarshalStruct{
					A: "testA",
				},
				B: "testB",
			},
		},
		"decode recursive embeded structpointer": {
			encoded:    []byte("b: testB\na:\n  b: testA"),
			decodeInto: new(UnmarshalEmbedRecursiveStruct),
			decoded: UnmarshalEmbedRecursiveStruct{
				UnmarshalEmbedRecursiveStruct: &UnmarshalEmbedRecursiveStruct{
					B: "testA",
				},
				B: "testB",
			},
		},

		// BUG: type info gets lost (#58)
		"decode embeded struct and cast integer to string": {
			encoded:    []byte("a: 11\nb: testB"),
			decodeInto: new(UnmarshalEmbedStruct),
			decoded: UnmarshalEmbedStruct{
				UnmarshalStruct: UnmarshalStruct{
					A: "11",
				},
				B: "testB",
			},
			err: fatalErrorsType,
		},
		"decode embeded structpointer and cast integer to string": {
			encoded:    []byte("a: 11\nb: testB"),
			decodeInto: new(UnmarshalEmbedStructPointer),
			decoded: UnmarshalEmbedStructPointer{
				UnmarshalStruct: &UnmarshalStruct{
					A: "11",
				},
				B: "testB",
			},
			err: fatalErrorsType,
		},

		// decoding into incompatible type
		"decode into stringmap with incompatible type": {
			encoded:    []byte("a:\n  a:\n    a: 3"),
			decodeInto: new(UnmarshalStringMap),
			err:        fatalErrorsType,
		},
	}

	t.Run("Unmarshal", func(t *testing.T) {
		testUnmarshal(t, funcUnmarshal, tests)
	})

	t.Run("UnmarshalStrict", func(t *testing.T) {
		testUnmarshal(t, funcUnmarshalStrict, tests)
	})
}

func TestUnmarshalStrictFails(t *testing.T) {
	tests := map[string]unmarshalTestCase{
		// decoding with duplicate values
		"decode into struct pointer map with duplicate string value": {
			encoded:    []byte("a:\n  a: TestA\n  b: ID-A\n  b: ID-1"),
			decodeInto: new(map[string]*UnmarshalStruct),
			decoded: map[string]*UnmarshalStruct{
				"a": {A: "TestA", B: strPtr("ID-1")},
			},
		},
		"decode into string field with duplicate boolean value": {
			encoded:    []byte("a: true\na: false"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "false"},
		},
		"decode into slice with duplicate string-boolean value": {
			encoded:    []byte("a:\n- b: abc\n  a: 32\n  b: 123"),
			decodeInto: new(UnmarshalSlice),
			decoded:    UnmarshalSlice{[]UnmarshalStruct{{A: "32", B: strPtr("123")}}},
		},

		// decoding with unknown fields
		"decode into struct with unknown field": {
			encoded:    []byte("a: TestB\nb: ID-B\nunknown: Some-Value"),
			decodeInto: new(UnmarshalStruct),
			decoded:    UnmarshalStruct{A: "TestB", B: strPtr("ID-B")},
		},

		// decoding with duplicate complex values
		"decode duplicate into nested struct": {
			encoded:    []byte("a:\n  a: 1\na:\n  a: 2"),
			decodeInto: new(UnmarshalNestedStruct),
			decoded:    UnmarshalNestedStruct{A: UnmarshalStruct{A: "2"}},
		},
		"decode duplicate into nested slice": {
			encoded:    []byte("a:\n  - a: abc\n    b: def\na:\n  - a: 123"),
			decodeInto: new(UnmarshalSlice),
			decoded:    UnmarshalSlice{[]UnmarshalStruct{{A: "123"}}},
		},
		"decode duplicate into nested string map": {
			encoded:    []byte("a:\n  b: 1\na:\n  c: 1"),
			decodeInto: new(UnmarshalStringMap),
			decoded:    UnmarshalStringMap{map[string]string{"c": "1"}},
		},
		"decode duplicate into string map": {
			encoded:    []byte("a: test\nb: test\na: test2"),
			decodeInto: new(map[string]string),
			decoded: map[string]string{
				"a": "test2",
				"b": "test",
			},
		},
	}

	t.Run("Unmarshal", func(t *testing.T) {
		testUnmarshal(t, funcUnmarshal, tests)
	})

	t.Run("UnmarshalStrict", func(t *testing.T) {
		failTests := map[string]unmarshalTestCase{}
		for name, test := range tests {
			test.err = fatalErrorsType
			failTests[name] = test
		}
		testUnmarshal(t, funcUnmarshalStrict, failTests)
	})
}

func TestYAMLToJSON(t *testing.T) {
	tests := map[string]yamlToJSONTestcase{
		"string value": {
			yaml: "t: a\n",
			json: `{"t":"a"}`,
		},
		"null value": {
			yaml: "t: null\n",
			json: `{"t":null}`,
		},
		"boolean value": {
			yaml:                 "t: True\n",
			json:                 `{"t":true}`,
			yamlReverseOverwrite: strPtr("t: true\n"),
		},
		"boolean value (no)": {
			yaml:                 "t: no\n",
			json:                 `{"t":false}`,
			yamlReverseOverwrite: strPtr("t: false\n"),
		},
		"integer value (2^53 + 1)": {
			yaml:                 "t: 9007199254740993\n",
			json:                 `{"t":9007199254740993}`,
			yamlReverseOverwrite: strPtr("t: 9007199254740993\n"),
		},
		"integer value (1000000000000000000000000000000000000)": {
			yaml:                 "t: 1000000000000000000000000000000000000\n",
			json:                 `{"t":1e+36}`,
			yamlReverseOverwrite: strPtr("t: 1e+36\n"),
		},
		"line-wrapped string value": {
			yaml: "t: this is very long line with spaces and it must be longer than 80 so we will repeat\n  that it must be longer that 80\n",
			json: `{"t":"this is very long line with spaces and it must be longer than 80 so we will repeat that it must be longer that 80"}`,
		},
		"empty yaml value": {
			yaml:                 "t: ",
			json:                 `{"t":null}`,
			yamlReverseOverwrite: strPtr("t: null\n"),
		},
		"boolean key": {
			yaml:                 "True: a",
			json:                 `{"true":"a"}`,
			yamlReverseOverwrite: strPtr("\"true\": a\n"),
		},
		"boolean key (no)": {
			yaml:                 "no: a",
			json:                 `{"false":"a"}`,
			yamlReverseOverwrite: strPtr("\"false\": a\n"),
		},
		"integer key": {
			yaml:                 "1: a",
			json:                 `{"1":"a"}`,
			yamlReverseOverwrite: strPtr("\"1\": a\n"),
		},
		"float key": {
			yaml:                 "1.2: a",
			json:                 `{"1.2":"a"}`,
			yamlReverseOverwrite: strPtr("\"1.2\": a\n"),
		},
		"large integer key": {
			yaml:                 "1000000000000000000000000000000000000: a",
			json:                 `{"1e+36":"a"}`,
			yamlReverseOverwrite: strPtr("\"1e+36\": a\n"),
		},
		"large integer key (scientific notation)": {
			yaml:                 "1e+36: a",
			json:                 `{"1e+36":"a"}`,
			yamlReverseOverwrite: strPtr("\"1e+36\": a\n"),
		},
		"string key (large integer as string)": {
			yaml: "\"1e+36\": a\n",
			json: `{"1e+36":"a"}`,
		},
		"string key (float as string)": {
			yaml: "\"1.2\": a\n",
			json: `{"1.2":"a"}`,
		},
		"array": {
			yaml: "- t: a\n",
			json: `[{"t":"a"}]`,
		},
		"nested struct array": {
			yaml: "- t: a\n- t:\n    b: 1\n    c: 2\n",
			json: `[{"t":"a"},{"t":{"b":1,"c":2}}]`,
		},
		"nested struct array (json notation)": {
			yaml:                 `[{t: a}, {t: {b: 1, c: 2}}]`,
			json:                 `[{"t":"a"},{"t":{"b":1,"c":2}}]`,
			yamlReverseOverwrite: strPtr("- t: a\n- t:\n    b: 1\n    c: 2\n"),
		},
		"empty struct value": {
			yaml:                 "- t: ",
			json:                 `[{"t":null}]`,
			yamlReverseOverwrite: strPtr("- t: null\n"),
		},
		"null struct value": {
			yaml: "- t: null\n",
			json: `[{"t":null}]`,
		},
		"binary data": {
			yaml:                 "a: !!binary gIGC",
			json:                 `{"a":"\ufffd\ufffd\ufffd"}`,
			yamlReverseOverwrite: strPtr("a: \ufffd\ufffd\ufffd\n"),
		},

		// Cases that should produce errors.
		"~ key": {
			yaml:                 "~: a",
			json:                 `{"null":"a"}`,
			yamlReverseOverwrite: strPtr("\"null\": a\n"),
			err:                  fatalErrorsType,
		},
		"null key": {
			yaml:                 "null: a",
			json:                 `{"null":"a"}`,
			yamlReverseOverwrite: strPtr("\"null\": a\n"),
			err:                  fatalErrorsType,
		},
	}

	t.Run("YAMLToJSON", func(t *testing.T) {
		testYAMLToJSON(t, funcYAMLToJSON, tests)
	})

	t.Run("YAMLToJSONStrict", func(t *testing.T) {
		testYAMLToJSON(t, funcYAMLToJSONStrict, tests)
	})
}

func TestYAMLToJSONStrictFails(t *testing.T) {
	tests := map[string]yamlToJSONTestcase{
		// expect YAMLtoJSON to pass on duplicate field names
		"duplicate struct value": {
			yaml:                 "foo: bar\nfoo: baz\n",
			json:                 `{"foo":"baz"}`,
			yamlReverseOverwrite: strPtr("foo: baz\n"),
		},
	}

	t.Run("YAMLToJSON", func(t *testing.T) {
		testYAMLToJSON(t, funcYAMLToJSON, tests)
	})

	t.Run("YAMLToJSONStrict", func(t *testing.T) {
		failTests := map[string]yamlToJSONTestcase{}
		for name, test := range tests {
			test.err = fatalErrorsType
			failTests[name] = test
		}
		testYAMLToJSON(t, funcYAMLToJSONStrict, failTests)
	})
}

func TestJSONObjectToYAMLObject(t *testing.T) {
	const bigUint64 = ((uint64(1) << 63) + 500) / 1000 * 1000
	intOrInt64 := func(i64 int64) interface{} {
		if i := int(i64); i64 == int64(i) {
			return i
		}
		return i64
	}

	tests := []struct {
		name     string
		input    map[string]interface{}
		expected yamlv2.MapSlice
	}{
		{name: "nil", expected: yamlv2.MapSlice(nil)},
		{name: "empty", input: map[string]interface{}{}, expected: yamlv2.MapSlice(nil)},
		{
			name: "values",
			input: map[string]interface{}{
				"nil slice":          []interface{}(nil),
				"nil map":            map[string]interface{}(nil),
				"empty slice":        []interface{}{},
				"empty map":          map[string]interface{}{},
				"bool":               true,
				"float64":            float64(42.1),
				"fractionless":       float64(42),
				"int":                int(42),
				"int64":              int64(42),
				"int64 big":          float64(math.Pow(2, 62)),
				"negative int64 big": -float64(math.Pow(2, 62)),
				"map":                map[string]interface{}{"foo": "bar"},
				"slice":              []interface{}{"foo", "bar"},
				"string":             string("foo"),
				"uint64 big":         bigUint64,
			},
			expected: yamlv2.MapSlice{
				{Key: "nil slice"},
				{Key: "nil map"},
				{Key: "empty slice", Value: []interface{}{}},
				{Key: "empty map", Value: yamlv2.MapSlice(nil)},
				{Key: "bool", Value: true},
				{Key: "float64", Value: float64(42.1)},
				{Key: "fractionless", Value: int(42)},
				{Key: "int", Value: int(42)},
				{Key: "int64", Value: int(42)},
				{Key: "int64 big", Value: intOrInt64(int64(1) << 62)},
				{Key: "negative int64 big", Value: intOrInt64(-(1 << 62))},
				{Key: "map", Value: yamlv2.MapSlice{{Key: "foo", Value: "bar"}}},
				{Key: "slice", Value: []interface{}{"foo", "bar"}},
				{Key: "string", Value: string("foo")},
				{Key: "uint64 big", Value: bigUint64},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := JSONObjectToYAMLObject(tt.input)
			sortMapSlicesInPlace(tt.expected)
			sortMapSlicesInPlace(got)
			if !reflect.DeepEqual(tt.expected, got) {
				t.Errorf("jsonToYAML() returned unexpected results (-want+got):\n%v", cmp.Diff(tt.expected, got))
			}

			jsonBytes, err := json.Marshal(tt.input)
			if err != nil {
				t.Fatalf("unexpected json.Marshal error: %v", err)
			}
			var gotByRoundtrip yamlv2.MapSlice
			if err := yamlv2.Unmarshal(jsonBytes, &gotByRoundtrip); err != nil {
				t.Fatalf("unexpected yaml.Unmarshal error: %v", err)
			}

			// yamlv2.Unmarshal loses precision, it's rounding to the 4th last digit.
			// Replicate this here in the test, but don't change the type.
			for i := range got {
				switch got[i].Key {
				case "int64 big", "uint64 big", "negative int64 big":
					switch v := got[i].Value.(type) {
					case int64:
						d := int64(500)
						if v < 0 {
							d = -500
						}
						got[i].Value = int64((v+d)/1000) * 1000
					case uint64:
						got[i].Value = uint64((v+500)/1000) * 1000
					case int:
						d := int(500)
						if v < 0 {
							d = -500
						}
						got[i].Value = int((v+d)/1000) * 1000
					default:
						t.Fatalf("unexpected type for key %s: %v:%T", got[i].Key, v, v)
					}
				}
			}

			if !reflect.DeepEqual(got, gotByRoundtrip) {
				t.Errorf("yaml.Unmarshal(json.Marshal(tt.input)) returned unexpected results (-want+got):\n%v", cmp.Diff(got, gotByRoundtrip))
				t.Errorf("json: %s", string(jsonBytes))
			}
		})
	}
}

func sortMapSlicesInPlace(x interface{}) {
	switch x := x.(type) {
	case []interface{}:
		for i := range x {
			sortMapSlicesInPlace(x[i])
		}
	case yamlv2.MapSlice:
		sort.Slice(x, func(a, b int) bool {
			return x[a].Key.(string) < x[b].Key.(string)
		})
	}
}

func TestPatchedYamlV3AndUpstream(t *testing.T) {
	input := `group: apps
apiVersion: v1
kind: Deployment
metadata:
  name: deploy1
spec:
  template:
    spec:
      containers:
      - image: nginx:1.7.9
        name: nginx-tagged
      - image: nginx:latest
        name: nginx-latest
      - image: foobar:1
        name: replaced-with-digest
      - image: postgres:1.8.0
        name: postgresdb
      initContainers:
      - image: nginx
        name: nginx-notag
      - image: nginx@sha256:111111111111111111
        name: nginx-sha256
      - image: alpine:1.8.0
        name: init-alpine
`

	var v3Map map[string]interface{}
	var v2Map map[string]interface{}

	// unmarshal the input into the two maps
	if err := yamlv3.Unmarshal([]byte(input), &v3Map); err != nil {
		t.Fatal(err)
	}
	if err := yamlv2.Unmarshal([]byte(input), &v2Map); err != nil {
		t.Fatal(err)
	}

	// marshal using non-default settings from the yaml v3 fork
	var buf bytes.Buffer
	enc := yamlv3.NewEncoder(&buf)
	enc.CompactSeqIndent()
	enc.SetIndent(2)
	err := enc.Encode(v3Map)
	v3output := buf.String()

	// marshal using the yaml v2 fork
	v2output, err := yamlv2.Marshal(v2Map)
	if err != nil {
		t.Fatal(err)
	}
	if v3output != string(v2output) {
		t.Fatalf("expected\n%s\ngot\n%s", string(v2output), v3output)
	}
}
