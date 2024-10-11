package cli_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/x/auth/client/cli"
)

func TestParseSigs(t *testing.T) {
	cases := []struct {
		name       string
		args       []string
		expErr     bool
		expNumSigs int
	}{
		{"no args", []string{}, true, 0},
		{"empty args", []string{""}, true, 0},
		{"too many args", []string{"foo", "bar"}, true, 0},
		{"1 sig", []string{"foo"}, false, 1},
		{"3 sigs", []string{"foo,bar,baz"}, false, 3},
	}

	for _, tc := range cases {
		sigs, err := cli.ParseSigArgs(tc.args)
		if tc.expErr {
			require.Error(t, err)
		} else {
			require.NoError(t, err)
			require.Equal(t, tc.expNumSigs, len(sigs))
		}
	}
}
