package textual_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

func TestDispatcher(t *testing.T) {
	testcases := []struct {
		name             string
		expErr           bool
		expValueRenderer textual.ValueRenderer
	}{
		{"UINT32", false, textual.NewIntValueRenderer(fieldDescriptorFromName("UINT32"))},
		{"UINT64", false, textual.NewIntValueRenderer(fieldDescriptorFromName("UINT64"))},
		{"SDKINT", false, textual.NewIntValueRenderer(fieldDescriptorFromName("SDKINT"))},
		{"SDKDEC", false, textual.NewDecValueRenderer()},
		{"BYTES", false, textual.NewBytesValueRenderer()},
		{"TIMESTAMP", false, textual.NewTimestampValueRenderer()},
		{"DURATION", false, textual.NewDurationValueRenderer()},
		{"COIN", false, textual.NewCoinsValueRenderer(nil)},
		{"COINS", false, textual.NewCoinsValueRenderer(nil)},
		{"ENUM", false, textual.NewEnumValueRenderer(fieldDescriptorFromName("ENUM"))},
		{"ANY", false, textual.NewAnyValueRenderer(nil)},
		{"FLOAT", true, nil},
	}

	for _, tc := range testcases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			textual, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
			require.NoError(t, err)
			rend, err := textual.GetFieldValueRenderer(fieldDescriptorFromName(tc.name))

			if tc.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				require.IsType(t, tc.expValueRenderer, rend)
			}
		})
	}
}

// fieldDescriptorFromName is like GetADR050ValueRenderer, but taking a Go type
// as input instead of a protoreflect.FieldDescriptor.
func fieldDescriptorFromName(name string) protoreflect.FieldDescriptor {
	a := (&testpb.A{}).ProtoReflect().Descriptor().Fields()
	fd := a.ByName(protoreflect.Name(name))
	if fd == nil {
		panic(fmt.Errorf("no field descriptor for %s", name))
	}

	return fd
}
