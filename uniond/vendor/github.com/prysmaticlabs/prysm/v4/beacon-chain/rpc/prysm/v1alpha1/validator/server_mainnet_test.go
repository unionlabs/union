package validator

import (
	"context"
	"testing"

	"github.com/golang/mock/gomock"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/container/trie"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestWaitForActivation_ValidatorOriginallyExists(t *testing.T) {
	// This test breaks if it doesn't use mainnet config
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.MainnetConfig().Copy())
	ctx := context.Background()

	priv1, err := bls.RandKey()
	require.NoError(t, err)
	priv2, err := bls.RandKey()
	require.NoError(t, err)

	pubKey1 := priv1.PublicKey().Marshal()
	pubKey2 := priv2.PublicKey().Marshal()

	beaconState := &ethpb.BeaconState{
		Slot: 4000,
		Validators: []*ethpb.Validator{
			{
				ActivationEpoch:       0,
				ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
				PublicKey:             pubKey1,
				WithdrawalCredentials: make([]byte, 32),
			},
		},
	}
	block := util.NewBeaconBlock()
	genesisRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")
	depData := &ethpb.Deposit_Data{
		PublicKey:             pubKey1,
		WithdrawalCredentials: bytesutil.PadTo([]byte("hey"), 32),
		Signature:             make([]byte, 96),
	}
	domain, err := signing.ComputeDomain(params.BeaconConfig().DomainDeposit, nil, nil)
	require.NoError(t, err)
	signingRoot, err := signing.ComputeSigningRoot(depData, domain)
	require.NoError(t, err)
	depData.Signature = priv1.Sign(signingRoot[:]).Marshal()

	deposit := &ethpb.Deposit{
		Data: depData,
	}
	depositTrie, err := trie.NewTrie(params.BeaconConfig().DepositContractTreeDepth)
	require.NoError(t, err, "Could not setup deposit trie")
	depositCache, err := depositcache.New()
	require.NoError(t, err)

	root, err := depositTrie.HashTreeRoot()
	require.NoError(t, err)
	assert.NoError(t, depositCache.InsertDeposit(ctx, deposit, 10 /*blockNum*/, 0, root))
	s, err := state_native.InitializeFromProtoUnsafePhase0(beaconState)
	require.NoError(t, err)
	vs := &Server{
		Ctx:               context.Background(),
		ChainStartFetcher: &mockExecution.Chain{},
		BlockFetcher:      &mockExecution.Chain{},
		Eth1InfoFetcher:   &mockExecution.Chain{},
		DepositFetcher:    depositCache,
		HeadFetcher:       &mockChain.ChainService{State: s, Root: genesisRoot[:]},
	}
	req := &ethpb.ValidatorActivationRequest{
		PublicKeys: [][]byte{pubKey1, pubKey2},
	}
	ctrl := gomock.NewController(t)

	defer ctrl.Finish()
	mockChainStream := mock.NewMockBeaconNodeValidator_WaitForActivationServer(ctrl)
	mockChainStream.EXPECT().Context().Return(context.Background())
	mockChainStream.EXPECT().Send(
		&ethpb.ValidatorActivationResponse{
			Statuses: []*ethpb.ValidatorActivationResponse_Status{
				{
					PublicKey: pubKey1,
					Status: &ethpb.ValidatorStatusResponse{
						Status: ethpb.ValidatorStatus_ACTIVE,
					},
					Index: 0,
				},
				{
					PublicKey: pubKey2,
					Status: &ethpb.ValidatorStatusResponse{
						ActivationEpoch: params.BeaconConfig().FarFutureEpoch,
					},
					Index: nonExistentIndex,
				},
			},
		},
	).Return(nil)

	require.NoError(t, vs.WaitForActivation(req, mockChainStream), "Could not setup wait for activation stream")
}
