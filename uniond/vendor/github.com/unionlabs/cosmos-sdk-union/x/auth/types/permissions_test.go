package types

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestHasPermission(t *testing.T) {
	emptyPermAddr := NewPermissionsForAddress("empty", []string{})
	has := emptyPermAddr.HasPermission(Minter)
	require.False(t, has)

	cases := []struct {
		permission string
		expectHas  bool
	}{
		{Minter, true},
		{Burner, true},
		{Staking, true},
		{"random", false},
		{"", false},
	}
	permAddr := NewPermissionsForAddress("test", []string{Minter, Burner, Staking})
	for i, tc := range cases {
		has = permAddr.HasPermission(tc.permission)
		require.Equal(t, tc.expectHas, has, "test case #%d", i)
	}
}

func TestValidatePermissions(t *testing.T) {
	cases := []struct {
		name        string
		permissions []string
		expectPass  bool
	}{
		{"no permissions", []string{}, true},
		{"valid permission", []string{Minter}, true},
		{"invalid permission", []string{""}, false},
		{"invalid and valid permission", []string{Staking, ""}, false},
	}

	for i, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			err := validatePermissions(tc.permissions...)
			if tc.expectPass {
				require.NoError(t, err, "test case #%d", i)
			} else {
				require.Error(t, err, "test case #%d", i)
			}
		})
	}
}
