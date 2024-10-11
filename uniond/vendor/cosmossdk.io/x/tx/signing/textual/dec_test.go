package textual_test

import (
	"context"
	"encoding/json"
	"math/big"
	"os"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/math"
	"cosmossdk.io/x/tx/signing/textual"
)

func TestDecJSONTestcases(t *testing.T) {
	type decimalTest []string
	var testcases []decimalTest
	raw, err := os.ReadFile("./internal/testdata/decimals.json")
	require.NoError(t, err)
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	textual, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	require.NoError(t, err)

	for _, tc := range testcases {
		tc := tc
		t.Run(tc[0], func(t *testing.T) {
			r, err := textual.GetFieldValueRenderer(fieldDescriptorFromName("SDKDEC"))
			require.NoError(t, err)

			checkDecTest(t, r, protoreflect.ValueOf(tc[0]), tc[1])
		})
	}
}

func checkDecTest(t *testing.T, r textual.ValueRenderer, pv protoreflect.Value, expected string) {
	t.Helper()
	screens, err := r.Format(context.Background(), pv)
	require.NoError(t, err)
	require.Len(t, screens, 1)
	require.Zero(t, screens[0].Indent)
	require.False(t, screens[0].Expert)

	require.Equal(t, expected, screens[0].Content)

	// Round trip.
	value, err := r.Parse(context.Background(), screens)
	require.NoError(t, err)

	v1, err := math.LegacyNewDecFromStr(value.String())
	require.NoError(t, err)

	decStr := pv.String()
	if !strings.Contains(decStr, ".") {
		n, ok := new(big.Int).SetString(decStr, 10)
		require.True(t, ok)
		decStr = math.LegacyNewDecFromBigIntWithPrec(n, 18).String()
	}

	v, err := math.LegacyNewDecFromStr(decStr)
	require.NoError(t, err)

	require.Truef(t, v.Equal(v1), "%s != %s", v, v1)
}
