package beacon_api

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestGetGenesis_ValidGenesis(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	genesisResponseJson := beacon.GetGenesisResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/beacon/genesis",
		&genesisResponseJson,
	).Return(
		nil,
	).SetArg(
		2,
		beacon.GetGenesisResponse{
			Data: &beacon.Genesis{
				GenesisTime:           "1234",
				GenesisValidatorsRoot: "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2",
			},
		},
	).Times(1)

	genesisProvider := &beaconApiGenesisProvider{jsonRestHandler: jsonRestHandler}
	resp, err := genesisProvider.GetGenesis(ctx)
	assert.NoError(t, err)
	require.NotNil(t, resp)
	assert.Equal(t, "1234", resp.GenesisTime)
	assert.Equal(t, "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2", resp.GenesisValidatorsRoot)
}

func TestGetGenesis_NilData(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	genesisResponseJson := beacon.GetGenesisResponse{}
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)
	jsonRestHandler.EXPECT().Get(
		ctx,
		"/eth/v1/beacon/genesis",
		&genesisResponseJson,
	).Return(
		nil,
	).SetArg(
		2,
		beacon.GetGenesisResponse{Data: nil},
	).Times(1)

	genesisProvider := &beaconApiGenesisProvider{jsonRestHandler: jsonRestHandler}
	_, err := genesisProvider.GetGenesis(ctx)
	assert.ErrorContains(t, "genesis data is nil", err)
}
