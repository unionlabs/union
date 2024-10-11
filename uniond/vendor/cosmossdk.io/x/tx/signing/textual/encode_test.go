package textual

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

type encodingJSONTest struct {
	Screens  []Screen
	Encoding string
}

func TestEncodingJson(t *testing.T) {
	raw, err := os.ReadFile("./internal/testdata/encode.json")
	require.NoError(t, err)

	var testcases []encodingJSONTest
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	for i, tc := range testcases {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			var buf bytes.Buffer
			err := encode(tc.Screens, &buf)
			require.NoError(t, err)
			want, err := hex.DecodeString(tc.Encoding)
			require.NoError(t, err)
			require.Equal(t, want, buf.Bytes())
		})
	}
}
