package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/ibc-go/v8/modules/apps/29-fee/types"
	ibcmock "github.com/cosmos/ibc-go/v8/testing/mock"
)

func TestMetadataFromVersion(t *testing.T) {
	testMetadata := types.Metadata{
		AppVersion: ibcmock.Version,
		FeeVersion: types.Version,
	}

	versionBz, err := types.ModuleCdc.MarshalJSON(&testMetadata)
	require.NoError(t, err)

	metadata, err := types.MetadataFromVersion(string(versionBz))
	require.NoError(t, err)
	require.Equal(t, ibcmock.Version, metadata.AppVersion)
	require.Equal(t, types.Version, metadata.FeeVersion)

	metadata, err = types.MetadataFromVersion("")
	require.Error(t, err)
	require.ErrorIs(t, err, types.ErrInvalidVersion)
	require.Empty(t, metadata)
}
