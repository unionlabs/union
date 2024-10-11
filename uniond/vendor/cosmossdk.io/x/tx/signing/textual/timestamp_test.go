package textual_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"
	dur "google.golang.org/protobuf/types/known/durationpb"
	tspb "google.golang.org/protobuf/types/known/timestamppb"

	"cosmossdk.io/x/tx/signing/textual"
)

// timestampJSONTest is the type of test cases in the testdata file.
// If the test case has a Proto, try to Format() it. If Error is set, expect
// an error, otherwise match Text, then Parse() the text and expect it to
// match (via proto.Equals()) the original Proto. If the test case has no
// Proto, try to Parse() the Text and expect an error if Error is set.
//
// The Timestamp proto seconds field is int64, but restricted in range
// by convention and will fit within a JSON number.
type timestampJSONTest struct {
	Proto *tspb.Timestamp
	Error bool
	Text  string
}

func TestTimestampJSONTestcasesTestData(t *testing.T) {
	raw, err := os.ReadFile("./internal/testdata/timestamp.json")
	require.NoError(t, err)
	testTimestampJSONTestcases(t, raw)
}

// Tests to ensure that we compare standardized forms of the final timestamppb.Timestamp.
// Please see issue https://github.com/cosmos/cosmos-sdk/issues/15761
func TestTimestampJsonTestcasesExtraneousNanos(t *testing.T) {
	testTimestampJSONTestcases(t, []byte(`[{"proto":{"nAnos":1000000000},"text":"1970-01-01T00:00:01Z"}]`))
}

func testTimestampJSONTestcases(t *testing.T, raw []byte) {
	t.Helper()
	var testcases []timestampJSONTest
	err := json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	for i, tc := range testcases {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			rend := textual.NewTimestampValueRenderer()

			var screens []textual.Screen
			if tc.Proto != nil {
				screens, err = rend.Format(context.Background(), protoreflect.ValueOf(tc.Proto.ProtoReflect()))
				if tc.Error {
					require.Error(t, err)
					return
				}
				require.NoError(t, err)
				require.Equal(t, 1, len(screens))
				require.Equal(t, tc.Text, screens[0].Content)
			}

			val, err := rend.Parse(context.Background(), screens)
			if tc.Error {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			msg := val.Message().Interface()
			require.IsType(t, &tspb.Timestamp{}, msg)
			timestamp := msg.(*tspb.Timestamp)
			// Please avoid using proto.Equal to compare timestamps given they aren't
			// in standardized form and will produce false positives for example given input:
			//  []byte(`[{"proto":{"nanos":1000000000}}]`)
			// Per issue: https://github.com/cosmos/cosmos-sdk/issues/15761
			require.True(t, timestamp.AsTime().Equal(tc.Proto.AsTime()))
		})
	}
}

func TestTimestampBadFormat(t *testing.T) {
	rend := textual.NewTimestampValueRenderer()
	_, err := rend.Format(context.Background(), protoreflect.ValueOf(dur.New(time.Hour).ProtoReflect()))
	require.Error(t, err)
}
