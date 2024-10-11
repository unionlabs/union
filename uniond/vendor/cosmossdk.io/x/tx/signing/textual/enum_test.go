package textual_test

import (
	"context"
	"encoding/json"
	"os"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/testing/protocmp"

	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

type enumTest struct {
	Proto json.RawMessage
	Text  string
}

func TestEnumJSONTestcases(t *testing.T) {
	var testcases []enumTest
	raw, err := os.ReadFile("./internal/testdata/enum.json")
	require.NoError(t, err)
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	textual, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	require.NoError(t, err)

	for _, tc := range testcases {
		t.Run(tc.Text, func(t *testing.T) {
			m := &testpb.Baz{}
			err := protojson.Unmarshal(tc.Proto, m)
			require.NoError(t, err)

			fd := getFd(tc.Proto, m)
			valrend, err := textual.GetFieldValueRenderer(fd)
			require.NoError(t, err)

			val := m.ProtoReflect().Get(fd)
			screens, err := valrend.Format(context.Background(), val)
			require.NoError(t, err)
			require.Equal(t, 1, len(screens))
			require.Equal(t, tc.Text, screens[0].Content)

			// Round trip
			parsedVal, err := valrend.Parse(context.Background(), screens)
			require.NoError(t, err)
			diff := cmp.Diff(val.Interface(), parsedVal.Interface(), protocmp.Transform())
			require.Empty(t, diff)
		})
	}
}

// getFd returns the field descriptor on Baz whose value is set. Since golang
// treats empty and default values as the same, we actually parse the protojson
// encoded string to retrieve which field is set.
func getFd(proto json.RawMessage, m *testpb.Baz) protoreflect.FieldDescriptor {
	var fnum protoreflect.FieldNumber

	switch {
	case strings.Contains(string(proto), `"ee"`):
		fnum = 1
	case strings.Contains(string(proto), `"ie"`):
		fnum = 2
	default:
		fnum = 3
	}

	return m.ProtoReflect().Descriptor().Fields().ByNumber(fnum)
}
