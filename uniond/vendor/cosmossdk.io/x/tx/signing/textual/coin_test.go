package textual_test

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/x/tx/signing/textual"
)

// Dedicated type for context Values, to avoid conflicts,
// per the (context.Context).WithValue docs.
type mockCoinMetadata string

// mockCoinMetadataKey is used in the mock coin metadata querier.
func mockCoinMetadataKey(denom string) mockCoinMetadata {
	return mockCoinMetadata(fmt.Sprintf("%s-%s", "coin-metadata", denom))
}

// mockCoinMetadataQuerier is a mock querier for coin metadata used for test
// purposes.
func mockCoinMetadataQuerier(ctx context.Context, denom string) (*bankv1beta1.Metadata, error) {
	v := ctx.Value(mockCoinMetadataKey(denom))
	if v == nil {
		return nil, nil
	}

	return v.(*bankv1beta1.Metadata), nil
}

// addMetadataToContext appends relevant coin metadata to the mock context
// used in tests.
func addMetadataToContext(ctx context.Context, metadata *bankv1beta1.Metadata) context.Context {
	if metadata == nil {
		return ctx
	}

	for _, m := range metadata.DenomUnits {
		ctx = context.WithValue(ctx, mockCoinMetadataKey(m.Denom), metadata)
	}

	return ctx
}

func TestMetadataQuerier(t *testing.T) {
	// Errors on nil metadata querier
	_, err := textual.NewSignModeHandler(textual.SignModeOptions{})
	require.Error(t, err)

	// Errors if metadata querier returns an error
	expErr := errors.New("mock error")
	txt, err := textual.NewSignModeHandler(textual.SignModeOptions{
		CoinMetadataQuerier: func(_ context.Context, _ string) (*bankv1beta1.Metadata, error) {
			return nil, expErr
		},
	})
	require.NoError(t, err)
	vr, err := txt.GetFieldValueRenderer(fieldDescriptorFromName("COIN"))
	require.NoError(t, err)
	_, err = vr.Format(context.Background(), protoreflect.ValueOf((&basev1beta1.Coin{}).ProtoReflect()))
	require.ErrorIs(t, err, expErr)
	_, err = vr.(textual.RepeatedValueRenderer).FormatRepeated(context.Background(), protoreflect.ValueOf(NewGenericList([]*basev1beta1.Coin{{}})))
	require.ErrorIs(t, err, expErr)
}

func TestCoinJSONTestcases(t *testing.T) {
	var testcases []coinJSONTest
	raw, err := os.ReadFile("./internal/testdata/coin.json")
	require.NoError(t, err)
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	textual, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: mockCoinMetadataQuerier})
	require.NoError(t, err)
	vr, err := textual.GetFieldValueRenderer(fieldDescriptorFromName("COIN"))
	require.NoError(t, err)

	for _, tc := range testcases {
		t.Run(tc.Text, func(t *testing.T) {
			if tc.Proto != nil {
				ctx := addMetadataToContext(context.Background(), tc.Metadata)

				screens, err := vr.Format(ctx, protoreflect.ValueOf(tc.Proto.ProtoReflect()))

				if tc.Error {
					require.Error(t, err)
					return
				}

				require.NoError(t, err)
				require.Equal(t, 1, len(screens))
				require.Equal(t, tc.Text, screens[0].Content)

				// Round trip.
				value, err := vr.Parse(ctx, screens)
				if tc.Error {
					require.Error(t, err)
					return
				}

				require.NoError(t, err)
				coin, ok := value.Message().Interface().(*basev1beta1.Coin)
				require.True(t, ok)

				checkCoinEqual(t, coin, tc.Proto)
			}
		})
	}
}

// coinJsonTest is the type of test cases in the testdata file.
// If the test case has a Proto, try to Format() it. If Error is set, expect
// an error, otherwise match Text, then Parse() the text and expect it to
// match (via proto.Equals()) the original Proto. If the test case has no
// Proto, try to Parse() the Text and expect an error if Error is set.
type coinJSONTest struct {
	Proto    *basev1beta1.Coin
	Metadata *bankv1beta1.Metadata
	Error    bool
	Text     string
}
