package types_test

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
)

func TestZeroHeight(t *testing.T) {
	require.Equal(t, types.Height{}, types.ZeroHeight())
}

func TestCompareHeights(t *testing.T) {
	testCases := []struct {
		name        string
		height1     types.Height
		height2     types.Height
		compareSign int64
	}{
		{"revision number 1 is lesser", types.NewHeight(1, 3), types.NewHeight(3, 4), -1},
		{"revision number 1 is greater", types.NewHeight(7, 5), types.NewHeight(4, 5), 1},
		{"revision height 1 is lesser", types.NewHeight(3, 4), types.NewHeight(3, 9), -1},
		{"revision height 1 is greater", types.NewHeight(3, 8), types.NewHeight(3, 3), 1},
		{"revision number is MaxUint64", types.NewHeight(math.MaxUint64, 1), types.NewHeight(0, 1), 1},
		{"revision height is MaxUint64", types.NewHeight(1, math.MaxUint64), types.NewHeight(1, 0), 1},
		{"height is equal", types.NewHeight(4, 4), types.NewHeight(4, 4), 0},
	}

	for i, tc := range testCases {
		i, tc := i, tc
		t.Run(tc.name, func(t *testing.T) {
			compare := tc.height1.Compare(tc.height2)

			switch tc.compareSign {
			case -1:
				require.True(t, compare == -1, "case %d: %s should return negative value on comparison, got: %d",
					i, tc.name, compare)
			case 0:
				require.True(t, compare == 0, "case %d: %s should return zero on comparison, got: %d",
					i, tc.name, compare)
			case 1:
				require.True(t, compare == 1, "case %d: %s should return positive value on comparison, got: %d",
					i, tc.name, compare)
			}
		})
	}
}

func TestDecrement(t *testing.T) {
	validDecrement := types.NewHeight(3, 3)
	expected := types.NewHeight(3, 2)

	actual, success := validDecrement.Decrement()
	require.Equal(t, expected, actual, "decrementing %s did not return expected height: %s. got %s",
		validDecrement, expected, actual)
	require.True(t, success, "decrement failed unexpectedly")

	invalidDecrement := types.NewHeight(3, 0)
	actual, success = invalidDecrement.Decrement()

	require.Equal(t, types.ZeroHeight(), actual, "invalid decrement returned non-zero height: %s", actual)
	require.False(t, success, "invalid decrement passed")
}

func TestString(t *testing.T) {
	_, err := types.ParseHeight("height")
	require.Error(t, err, "invalid height string passed")

	_, err = types.ParseHeight("revision-10")
	require.Error(t, err, "invalid revision string passed")

	_, err = types.ParseHeight("3-height")
	require.Error(t, err, "invalid revision-height string passed")

	height := types.NewHeight(3, 4)
	recovered, err := types.ParseHeight(height.String())

	require.NoError(t, err, "valid height string could not be parsed")
	require.Equal(t, height, recovered, "recovered height not equal to original height")

	parse, err := types.ParseHeight("3-10")
	require.NoError(t, err, "parse err")
	require.Equal(t, types.NewHeight(3, 10), parse, "parse height returns wrong height")
}

func (suite *TypesTestSuite) TestMustParseHeight() {
	suite.Require().Panics(func() {
		types.MustParseHeight("height")
	})

	suite.Require().NotPanics(func() {
		types.MustParseHeight("111-1")
	})

	suite.Require().NotPanics(func() {
		types.MustParseHeight("0-0")
	})
}

func TestParseChainID(t *testing.T) {
	cases := []struct {
		chainID   string
		revision  uint64
		formatted bool
	}{
		{"gaiamainnet-3", 3, true},
		{"a-1", 1, true},
		{"gaia-mainnet-40", 40, true},
		{"gaiamainnet-3-39", 39, true},
		{"gaiamainnet--", 0, false},
		{"gaiamainnet-03", 0, false},
		{"gaiamainnet--4", 0, false},
		{"gaiamainnet-3.4", 0, false},
		{"gaiamainnet", 0, false},
		{"gaiamain\nnet-1", 0, false}, // newlines not allowed in chainID
		{"gaiamainnet-1\n", 0, false}, // newlines not allowed after dash
		{"gaiamainnet\n-3", 0, false}, // newlines not allowed before revision number
		{"a--1", 0, false},
		{"-1", 0, false},
		{"--1", 0, false},
	}

	for _, tc := range cases {
		require.Equal(t, tc.formatted, types.IsRevisionFormat(tc.chainID), "id %s does not match expected format", tc.chainID)

		revision := types.ParseChainID(tc.chainID)
		require.Equal(t, tc.revision, revision, "chainID %s returns incorrect revision", tc.chainID)
	}
}

func TestSetRevisionNumber(t *testing.T) {
	// Test SetRevisionNumber
	chainID, err := types.SetRevisionNumber("gaiamainnet", 3)
	require.Error(t, err, "invalid revision format passed SetRevisionNumber")
	require.Equal(t, "", chainID, "invalid revision format returned non-empty string on SetRevisionNumber")
	chainID = "gaiamainnet-3"

	chainID, err = types.SetRevisionNumber(chainID, 4)
	require.NoError(t, err, "valid revision format failed SetRevisionNumber")
	require.Equal(t, "gaiamainnet-4", chainID, "valid revision format returned incorrect string on SetRevisionNumber")
}

func (suite *TypesTestSuite) TestSelfHeight() {
	ctx := suite.chainA.GetContext()

	// Test default revision
	ctx = ctx.WithChainID("gaiamainnet")
	ctx = ctx.WithBlockHeight(10)
	height := types.GetSelfHeight(ctx)
	suite.Require().Equal(types.NewHeight(0, 10), height, "default self height failed")

	// Test successful revision format
	ctx = ctx.WithChainID("gaiamainnet-3")
	ctx = ctx.WithBlockHeight(18)
	height = types.GetSelfHeight(ctx)
	suite.Require().Equal(types.NewHeight(3, 18), height, "valid self height failed")
}
