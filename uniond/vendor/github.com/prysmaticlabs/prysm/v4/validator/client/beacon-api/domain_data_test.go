package beacon_api

import (
	"context"
	"errors"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/beacon"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
)

func TestGetDomainData_ValidDomainData(t *testing.T) {
	const genesisValidatorRoot = "0xcf8e0d4e9587369b2301d0790347320302cc0943d5a1884560367e8208d920f2"
	forkVersion := params.BeaconConfig().AltairForkVersion
	epoch := params.BeaconConfig().AltairForkEpoch
	domainType := params.BeaconConfig().DomainBeaconProposer

	genesisValidatorRootBytes, err := hexutil.Decode(genesisValidatorRoot)
	require.NoError(t, err)

	expectedForkDataRoot, err := (&ethpb.ForkData{
		CurrentVersion:        forkVersion,
		GenesisValidatorsRoot: genesisValidatorRootBytes,
	}).HashTreeRoot()
	require.NoError(t, err)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	// Make sure that GetGenesis() is called exactly once
	genesisProvider := mock.NewMockGenesisProvider(ctrl)
	genesisProvider.EXPECT().GetGenesis(ctx).Return(
		&beacon.Genesis{GenesisValidatorsRoot: genesisValidatorRoot},
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{genesisProvider: genesisProvider}
	resp, err := validatorClient.getDomainData(ctx, epoch, domainType)
	assert.NoError(t, err)
	require.NotNil(t, resp)

	var expectedSignatureDomain []byte
	expectedSignatureDomain = append(expectedSignatureDomain, domainType[:]...)
	expectedSignatureDomain = append(expectedSignatureDomain, expectedForkDataRoot[:28]...)

	assert.Equal(t, len(expectedSignatureDomain), len(resp.SignatureDomain))
	assert.DeepEqual(t, expectedSignatureDomain, resp.SignatureDomain)
}

func TestGetDomainData_GenesisError(t *testing.T) {
	epoch := params.BeaconConfig().AltairForkEpoch
	domainType := params.BeaconConfig().DomainBeaconProposer

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	// Make sure that GetGenesis() is called exactly once
	genesisProvider := mock.NewMockGenesisProvider(ctrl)
	genesisProvider.EXPECT().GetGenesis(ctx).Return(nil, errors.New("foo error")).Times(1)

	validatorClient := &beaconApiValidatorClient{genesisProvider: genesisProvider}
	_, err := validatorClient.getDomainData(ctx, epoch, domainType)
	assert.ErrorContains(t, "failed to get genesis info", err)
	assert.ErrorContains(t, "foo error", err)
}

func TestGetDomainData_InvalidGenesisRoot(t *testing.T) {
	epoch := params.BeaconConfig().AltairForkEpoch
	domainType := params.BeaconConfig().DomainBeaconProposer

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()

	// Make sure that GetGenesis() is called exactly once
	genesisProvider := mock.NewMockGenesisProvider(ctrl)
	genesisProvider.EXPECT().GetGenesis(ctx).Return(
		&beacon.Genesis{GenesisValidatorsRoot: "foo"},
		nil,
	).Times(1)

	validatorClient := &beaconApiValidatorClient{genesisProvider: genesisProvider}
	_, err := validatorClient.getDomainData(ctx, epoch, domainType)
	assert.ErrorContains(t, "invalid genesis validators root: foo", err)
}
