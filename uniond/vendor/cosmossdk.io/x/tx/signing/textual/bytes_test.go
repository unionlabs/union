package textual_test

import (
	"context"
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/x/tx/signing/textual"
)

func TestBytesJSONTestCases(t *testing.T) {
	var testcases []bytesTest
	// bytes.json contains bytes that are represented in base64 format, and
	// their expected results in hex.
	raw, err := os.ReadFile("./internal/testdata/bytes.json")
	require.NoError(t, err)
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	textual, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	require.NoError(t, err)

	for _, tc := range testcases {
		t.Run(tc.hex, func(t *testing.T) {
			vr, err := textual.GetFieldValueRenderer(fieldDescriptorFromName("BYTES"))
			require.NoError(t, err)

			screens, err := vr.Format(context.Background(), protoreflect.ValueOfBytes(tc.base64))
			require.NoError(t, err)
			require.Equal(t, 1, len(screens))
			require.Equal(t, tc.hex, screens[0].Content)

			// Round trip
			val, err := vr.Parse(context.Background(), screens)
			require.NoError(t, err)
			if len(tc.base64) > 35 {
				require.Equal(t, 0, len(val.Bytes()))
			} else {
				require.Equal(t, tc.base64, val.Bytes())
			}
		})
	}
}

type bytesTest struct {
	base64 []byte
	hex    string
}

func (t *bytesTest) UnmarshalJSON(b []byte) error {
	a := []interface{}{&t.base64, &t.hex}
	return json.Unmarshal(b, &a)
}
