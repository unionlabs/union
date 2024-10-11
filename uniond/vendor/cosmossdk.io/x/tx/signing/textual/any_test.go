package textual_test

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/cosmos/cosmos-proto/anyutil"
	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/testing/protocmp"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/durationpb"
	"google.golang.org/protobuf/types/known/timestamppb"

	bankv1beta1 "cosmossdk.io/api/cosmos/bank/v1beta1"
	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

type anyJSONTest struct {
	Proto   json.RawMessage
	Screens []textual.Screen
}

func TestAny(t *testing.T) {
	raw, err := os.ReadFile("./internal/testdata/any.json")
	require.NoError(t, err)

	var testcases []anyJSONTest
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	require.NoError(t, err)
	for i, tc := range testcases {
		t.Run(fmt.Sprintf("%d", i), func(t *testing.T) {
			anyMsg := anypb.Any{}
			err = protojson.Unmarshal(tc.Proto, &anyMsg)
			require.NoError(t, err)

			// Format into screens and check vs expected
			rend := textual.NewAnyValueRenderer((tr))
			screens, err := rend.Format(context.Background(), protoreflect.ValueOfMessage(anyMsg.ProtoReflect()))
			require.NoError(t, err)
			require.Equal(t, tc.Screens, screens)

			// Parse back into a google.Protobuf.Any message.
			val, err := rend.Parse(context.Background(), screens)
			require.NoError(t, err)
			parsedMsg := val.Message().Interface()
			require.IsType(t, &anypb.Any{}, parsedMsg)
			parsedAny := parsedMsg.(*anypb.Any)
			diff := cmp.Diff(&anyMsg, parsedAny, protocmp.Transform())
			require.Empty(t, diff)
		})
	}
}

func TestDynamicpb(t *testing.T) {
	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{
		CoinMetadataQuerier: EmptyCoinMetadataQuerier,
		TypeResolver:        &protoregistry.Types{}, // Set to empty to force using dynamicpb
	})
	require.NoError(t, err)

	testAny, err := anyutil.New(&testpb.Foo{FullName: "foobar"})
	require.NoError(t, err)

	testcases := []struct {
		name string
		msg  proto.Message
	}{
		{"coin", &basev1beta1.Coin{Denom: "stake", Amount: "1"}},
		{"nested coins", &bankv1beta1.MsgSend{Amount: []*basev1beta1.Coin{{Denom: "stake", Amount: "1"}}}},
		{"any", testAny},
		{"nested any", &testpb.A{ANY: testAny}},
		{"duration", durationpb.New(time.Hour)},
		{"timestamp", timestamppb.New(time.Now())},
	}

	for _, tc := range testcases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			any, err := anyutil.New(tc.msg)
			require.NoError(t, err)
			val := &testpb.A{
				ANY: any,
			}
			vr, err := tr.GetMessageValueRenderer(val.ProtoReflect().Descriptor())
			require.NoError(t, err)

			// Round trip.
			screens, err := vr.Format(context.Background(), protoreflect.ValueOf(val.ProtoReflect()))
			require.NoError(t, err)
			parsedVal, err := vr.Parse(context.Background(), screens)
			require.NoError(t, err)
			diff := cmp.Diff(val, parsedVal.Message().Interface(), protocmp.Transform())
			require.Empty(t, diff)
		})
	}
}
