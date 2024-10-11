package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/core/03-connection/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func TestValidateVersion(t *testing.T) {
	testCases := []struct {
		name    string
		version *types.Version
		expPass bool
	}{
		{"valid version", types.DefaultIBCVersion, true},
		{"valid empty feature set", types.NewVersion(types.DefaultIBCVersionIdentifier, []string{}), true},
		{"empty version identifier", types.NewVersion("       ", []string{"ORDER_UNORDERED"}), false},
		{"empty feature", types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_UNORDERED", "   "}), false},
	}

	for i, tc := range testCases {
		i, tc := i, tc

		err := types.ValidateVersion(tc.version)

		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestIsSupportedVersion(t *testing.T) {
	testCases := []struct {
		name    string
		version *types.Version
		expPass bool
	}{
		{
			"version is supported",
			types.GetCompatibleVersions()[0],
			true,
		},
		{
			"version is not supported",
			&types.Version{},
			false,
		},
		{
			"version feature is not supported",
			types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_DAG"}),
			false,
		},
	}

	for _, tc := range testCases {
		tc := tc

		require.Equal(t, tc.expPass, types.IsSupportedVersion(types.GetCompatibleVersions(), tc.version))
	}
}

func TestFindSupportedVersion(t *testing.T) {
	testCases := []struct {
		name              string
		version           *types.Version
		supportedVersions []*types.Version
		expVersion        *types.Version
		expFound          bool
	}{
		{"valid supported version", types.DefaultIBCVersion, types.GetCompatibleVersions(), types.DefaultIBCVersion, true},
		{"empty (invalid) version", &types.Version{}, types.GetCompatibleVersions(), &types.Version{}, false},
		{"empty supported versions", types.DefaultIBCVersion, []*types.Version{}, &types.Version{}, false},
		{"desired version is last", types.DefaultIBCVersion, []*types.Version{types.NewVersion("1.1", nil), types.NewVersion("2", []string{"ORDER_UNORDERED"}), types.NewVersion("3", nil), types.DefaultIBCVersion}, types.DefaultIBCVersion, true},
		{"desired version identifier with different feature set", types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_DAG"}), types.GetCompatibleVersions(), types.DefaultIBCVersion, true},
		{"version not supported", types.NewVersion("2", []string{"ORDER_DAG"}), types.GetCompatibleVersions(), &types.Version{}, false},
	}

	for i, tc := range testCases {
		tc := tc

		version, found := types.FindSupportedVersion(tc.version, tc.supportedVersions)
		if tc.expFound {
			require.Equal(t, tc.expVersion.GetIdentifier(), version.GetIdentifier(), "test case %d: %s", i, tc.name)
			require.True(t, found, "test case %d: %s", i, tc.name)
		} else {
			require.False(t, found, "test case: %s", tc.name)
			require.Nil(t, version, "test case: %s", tc.name)
		}
	}
}

func TestPickVersion(t *testing.T) {
	testCases := []struct {
		name                 string
		supportedVersions    []*types.Version
		counterpartyVersions []*types.Version
		expVer               *types.Version
		expPass              bool
	}{
		{"valid default ibc version", types.GetCompatibleVersions(), types.GetCompatibleVersions(), types.DefaultIBCVersion, true},
		{"valid version in counterparty versions", types.GetCompatibleVersions(), []*types.Version{types.NewVersion("version1", nil), types.NewVersion("2.0.0", []string{"ORDER_UNORDERED-ZK"}), types.DefaultIBCVersion}, types.DefaultIBCVersion, true},
		{"valid identifier match but empty feature set not allowed", types.GetCompatibleVersions(), []*types.Version{types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"DAG", "ORDERED-ZK", "UNORDERED-zk]"})}, types.NewVersion(types.DefaultIBCVersionIdentifier, nil), false},
		{"empty counterparty versions", types.GetCompatibleVersions(), []*types.Version{}, &types.Version{}, false},
		{"non-matching counterparty versions", types.GetCompatibleVersions(), []*types.Version{types.NewVersion("2.0.0", nil)}, &types.Version{}, false},
		{"non-matching counterparty versions (uses ordered channels only) contained in supported versions (uses unordered channels only)", []*types.Version{types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_UNORDERED"})}, []*types.Version{types.NewVersion(types.DefaultIBCVersionIdentifier, []string{"ORDER_ORDERED"})}, &types.Version{}, false},
	}

	for i, tc := range testCases {
		i, tc := i, tc

		version, err := types.PickVersion(tc.supportedVersions, tc.counterpartyVersions)

		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
			var emptyVersion *types.Version
			require.Equal(t, emptyVersion, version, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

func TestVerifyProposedVersion(t *testing.T) {
	testCases := []struct {
		name             string
		proposedVersion  *types.Version
		supportedVersion *types.Version
		expPass          bool
	}{
		{"entire feature set supported", types.DefaultIBCVersion, types.NewVersion("1", []string{"ORDER_ORDERED", "ORDER_UNORDERED", "ORDER_DAG"}), true},
		{"empty feature sets not supported", types.NewVersion("1", []string{}), types.DefaultIBCVersion, false},
		{"one feature missing", types.DefaultIBCVersion, types.NewVersion("1", []string{"ORDER_UNORDERED", "ORDER_DAG"}), false},
		{"both features missing", types.DefaultIBCVersion, types.NewVersion("1", []string{"ORDER_DAG"}), false},
		{"identifiers do not match", types.NewVersion("2", []string{"ORDER_UNORDERED", "ORDER_ORDERED"}), types.DefaultIBCVersion, false},
	}

	for i, tc := range testCases {
		i, tc := i, tc

		err := tc.supportedVersion.VerifyProposedVersion(tc.proposedVersion)

		if tc.expPass {
			require.NoError(t, err, "test case %d: %s", i, tc.name)
		} else {
			require.Error(t, err, "test case %d: %s", i, tc.name)
		}
	}
}

func TestVerifySupportedFeature(t *testing.T) {
	nilFeatures := types.NewVersion(types.DefaultIBCVersionIdentifier, nil)

	testCases := []struct {
		name    string
		version *types.Version
		feature string
		expPass bool
	}{
		{"check ORDERED supported", ibctesting.ConnectionVersion, "ORDER_ORDERED", true},
		{"check UNORDERED supported", ibctesting.ConnectionVersion, "ORDER_UNORDERED", true},
		{"check DAG unsupported", ibctesting.ConnectionVersion, "ORDER_DAG", false},
		{"check empty feature set returns false", nilFeatures, "ORDER_ORDERED", false},
	}

	for i, tc := range testCases {
		i, tc := i, tc

		supported := types.VerifySupportedFeature(tc.version, tc.feature)

		require.Equal(t, tc.expPass, supported, "test case %d: %s", i, tc.name)
	}
}
