package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"union/x/tokenfactory/types"
)

func TestDeconstructDenom(t *testing.T) {
	// Note: this seems to be used in osmosis to add some more checks (only 20 or 32 byte addresses),
	// which is good, but not required for these tests as they make code less reuable
	// appparams.SetAddressPrefixes()

	for _, tc := range []struct {
		desc             string
		denom            string
		expectedSubdenom string
		err              error
	}{
		{
			desc:  "empty is invalid",
			denom: "",
			err:   types.ErrInvalidDenom,
		},
		{
			desc:             "normal",
			denom:            "factory/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/bitcoin",
			expectedSubdenom: "bitcoin",
		},
		{
			desc:             "multiple slashes in subdenom",
			denom:            "factory/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/bitcoin/1",
			expectedSubdenom: "bitcoin/1",
		},
		{
			desc:             "no subdenom",
			denom:            "factory/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/",
			expectedSubdenom: "",
		},
		{
			desc:  "incorrect prefix",
			denom: "ibc/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/bitcoin",
			err:   types.ErrInvalidDenom,
		},
		{
			desc:             "subdenom of only slashes",
			denom:            "factory/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/////",
			expectedSubdenom: "////",
		},
		{
			desc:  "too long name",
			denom: "factory/cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8/adsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsf",
			err:   types.ErrInvalidDenom,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			expectedCreator := "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8"
			creator, subdenom, err := types.DeconstructDenom(tc.denom)
			if tc.err != nil {
				require.ErrorContains(t, err, tc.err.Error())
			} else {
				require.NoError(t, err)
				require.Equal(t, expectedCreator, creator)
				require.Equal(t, tc.expectedSubdenom, subdenom)
			}
		})
	}
}

func TestGetTokenDenom(t *testing.T) {
	// appparams.SetAddressPrefixes()
	for _, tc := range []struct {
		desc     string
		creator  string
		subdenom string
		valid    bool
	}{
		{
			desc:     "normal",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "bitcoin",
			valid:    true,
		},
		{
			desc:     "multiple slashes in subdenom",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "bitcoin/1",
			valid:    true,
		},
		{
			desc:     "no subdenom",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "",
			valid:    true,
		},
		{
			desc:     "subdenom of only slashes",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "/////",
			valid:    true,
		},
		{
			desc:     "too long name",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "adsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsfadsf",
			valid:    false,
		},
		{
			desc:     "subdenom is exactly max length",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8",
			subdenom: "bitcoinfsadfsdfeadfsafwefsefsefsdfsdafasefsf",
			valid:    true,
		},
		{
			desc:     "creator is exactly max length",
			creator:  "cosmos1t7egva48prqmzl59x5ngv4zx0dtrwewcdqdjr8jhgjhgkhjklhkjhkjhgjhgjgjghelu",
			subdenom: "bitcoin",
			valid:    true,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			_, err := types.GetTokenDenom(tc.creator, tc.subdenom)
			if tc.valid {
				require.NoError(t, err)
			} else {
				require.Error(t, err)
			}
		})
	}
}
