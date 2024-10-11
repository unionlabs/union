package codec_test

import (
	"testing"

	"github.com/cosmos/gogoproto/types/any/test"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

func TestMarshalYAML(t *testing.T) {
	dog := &test.Dog{
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
  '@type': /test.Dog
  name: Spot
  size: small
x: "0"
`, string(bz))

	// amino
	aminoCdc := codec.NewAminoCodec(&codec.LegacyAmino{testdata.NewTestAmino()})
	bz, err = codec.MarshalYAML(aminoCdc, hasAnimal)
	require.NoError(t, err)
	require.Equal(t, `type: test/HasAnimal
value:
  animal:
    name: Spot
    size: small
`, string(bz))
}
