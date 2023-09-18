package codec_test

import (
	"testing"

	"github.com/cosmos/cosmos-sdk/codec"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestMarshalYAML(t *testing.T) {
	dog := &testdata.Dog{
		Size_: "small",
		Name:  "Spot",
	}
	any, err := types.NewAnyWithValue(dog)
	require.NoError(t, err)
	hasAnimal := &testdata.HasAnimal{
		Animal: any,
		X:      0,
	}

	// proto
	protoCdc := codec.NewProtoCodec(NewTestInterfaceRegistry())
	bz, err := codec.MarshalYAML(protoCdc, hasAnimal)
	require.NoError(t, err)
	require.Equal(t, `animal:
  '@type': /testpb.Dog
  name: Spot
  size: small
x: "0"
`, string(bz))

	// amino
	aminoCdc := codec.NewAminoCodec(&codec.LegacyAmino{testdata.NewTestAmino()})
	bz, err = codec.MarshalYAML(aminoCdc, hasAnimal)
	require.NoError(t, err)
	require.Equal(t, `type: testpb/HasAnimal
value:
  animal:
    type: testpb/Dog
    value:
      name: Spot
      size: small
`, string(bz))
}
