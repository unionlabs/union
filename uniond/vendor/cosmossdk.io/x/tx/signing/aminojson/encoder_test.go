package aminojson

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"
	"gotest.tools/v3/assert"
)

func TestCosmosInlineJSON(t *testing.T) {
	cases := map[string]struct {
		value      protoreflect.Value
		wantErr    bool
		wantOutput string
	}{
		"supported type - valid JSON object": {
			value:      protoreflect.ValueOfBytes([]byte(`{"test":"value"}`)),
			wantErr:    false,
			wantOutput: `{"test":"value"}`,
		},
		"supported type - valid JSON array": {
			// spaces are normalized away
			value:      protoreflect.ValueOfBytes([]byte(`[1,2,3]`)),
			wantErr:    false,
			wantOutput: `[1,2,3]`,
		},
		"supported type - valid JSON is normalized": {
			value:      protoreflect.ValueOfBytes([]byte(`[1, 2, 3]`)),
			wantErr:    false,
			wantOutput: `[1,2,3]`,
		},
		"supported type - valid JSON array (empty)": {
			value:      protoreflect.ValueOfBytes([]byte(`[]`)),
			wantErr:    false,
			wantOutput: `[]`,
		},
		"supported type - valid JSON number": {
			value:      protoreflect.ValueOfBytes([]byte(`43.72`)),
			wantErr:    false,
			wantOutput: `43.72`,
		},
		"supported type - valid JSON boolean": {
			value:      protoreflect.ValueOfBytes([]byte(`true`)),
			wantErr:    false,
			wantOutput: `true`,
		},
		"supported type - valid JSON null": {
			value:      protoreflect.ValueOfBytes([]byte(`null`)),
			wantErr:    false,
			wantOutput: `null`,
		},
		"supported type - valid JSON string": {
			value:      protoreflect.ValueOfBytes([]byte(`"hey yo"`)),
			wantErr:    false,
			wantOutput: `"hey yo"`,
		},
		"supported type - invalid JSON": {
			value:   protoreflect.ValueOfBytes([]byte(`foo`)),
			wantErr: true,
		},
		"supported type - invalid JSON (empty)": {
			value:   protoreflect.ValueOfBytes([]byte(``)),
			wantErr: true,
		},
		"supported type - invalid JSON (nil bytes)": {
			value:   protoreflect.ValueOfBytes(nil),
			wantErr: true,
		},
		"unsupported type - bool": {
			value:   protoreflect.ValueOfBool(true),
			wantErr: true,
		},
		"unsupported type - int64": {
			value:   protoreflect.ValueOfInt64(1),
			wantErr: true,
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			var buf bytes.Buffer
			err := cosmosInlineJSON(nil, tc.value, &buf)

			if tc.wantErr {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, tc.wantOutput, buf.String())
		})
	}
}

func TestSortedJSONStringify(t *testing.T) {
	tests := map[string]struct {
		input      []byte
		wantOutput string
	}{
		"leaves true unchanged": {
			input:      []byte(`true`),
			wantOutput: "true",
		},
		"leaves false unchanged": {
			input:      []byte(`false`),
			wantOutput: "false",
		},
		"leaves string unchanged": {
			input:      []byte(`"aabbccdd"`),
			wantOutput: `"aabbccdd"`,
		},
		"leaves number unchanged": {
			input:      []byte(`75`),
			wantOutput: "75",
		},
		"leaves nil unchanged": {
			input:      []byte(`null`),
			wantOutput: "null",
		},
		"leaves simple array unchanged": {
			input:      []byte(`[5, 6, 7, 1]`),
			wantOutput: "[5,6,7,1]",
		},
		"leaves complex array unchanged": {
			input:      []byte(`[5, ["a", "b"], true, null, 1]`),
			wantOutput: `[5,["a","b"],true,null,1]`,
		},
		"sorts empty object": {
			input:      []byte(`{}`),
			wantOutput: `{}`,
		},
		"sorts single key object": {
			input:      []byte(`{"a": 3}`),
			wantOutput: `{"a":3}`,
		},
		"sorts multiple keys object": {
			input:      []byte(`{"a": 3, "b": 2, "c": 1}`),
			wantOutput: `{"a":3,"b":2,"c":1}`,
		},
		"sorts unsorted object": {
			input:      []byte(`{"b": 2, "a": 3, "c": 1}`),
			wantOutput: `{"a":3,"b":2,"c":1}`,
		},
		"sorts unsorted complex object": {
			input:      []byte(`{"aaa": true, "aa": true, "a": true}`),
			wantOutput: `{"a":true,"aa":true,"aaa":true}`,
		},
		"sorts nested objects": {
			input:      []byte(`{"x": {"y": {"z": null}}}`),
			wantOutput: `{"x":{"y":{"z":null}}}`,
		},
		"sorts deeply nested unsorted objects": {
			input:      []byte(`{"b": {"z": true, "x": true, "y": true}, "a": true, "c": true}`),
			wantOutput: `{"a":true,"b":{"x":true,"y":true,"z":true},"c":true}`,
		},
		"sorts objects in array sorted": {
			input:      []byte(`[1, 2, {"x": {"y": {"z": null}}}, 4]`),
			wantOutput: `[1,2,{"x":{"y":{"z":null}}},4]`,
		},
		"sorts objects in array unsorted": {
			input:      []byte(`[1, 2, {"b": {"z": true, "x": true, "y": true}, "a": true, "c": true}, 4]`),
			wantOutput: `[1,2,{"a":true,"b":{"x":true,"y":true,"z":true},"c":true},4]`,
		},
	}

	for name, tc := range tests {
		t.Run(name, func(t *testing.T) {
			got, err := sortedJSONStringify(tc.input)
			require.NoError(t, err)
			assert.Equal(t, tc.wantOutput, string(got))
		})
	}
}
