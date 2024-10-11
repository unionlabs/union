package textual_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

type repeatedJSONTest struct {
	Proto   *testpb.Qux
	Screens []textual.Screen
}

func TestRepeatedJSONTestcases(t *testing.T) {
	raw, err := os.ReadFile("./internal/testdata/repeated.json")
	require.NoError(t, err)

	var testcases []repeatedJSONTest
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: mockCoinMetadataQuerier})
	for i, tc := range testcases {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			// Create a context.Context containing all coins metadata, to simulate
			// that they are in state.
			ctx := context.Background()
			rend := textual.NewMessageValueRenderer(tr, (&testpb.Qux{}).ProtoReflect().Descriptor())
			require.NoError(t, err)

			screens, err := rend.Format(ctx, protoreflect.ValueOf(tc.Proto.ProtoReflect()))
			require.NoError(t, err)
			require.Equal(t, tc.Screens, screens)

			val, err := rend.Parse(context.Background(), screens)
			require.NoError(t, err)
			msg := val.Message().Interface()
			require.IsType(t, &testpb.Qux{}, msg)
			baz := msg.(*testpb.Qux)
			require.True(t, proto.Equal(baz, tc.Proto))
		})
	}
}
