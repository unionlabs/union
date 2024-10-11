package validator

import (
	"context"
	"testing"
	"time"

	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestProposeExit_Notification(t *testing.T) {
	ctx := context.Background()

	deposits, keys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	require.NoError(t, err)
	beaconState, err := transition.GenesisBeaconState(ctx, deposits, 0, &ethpb.Eth1Data{BlockHash: make([]byte, 32)})
	require.NoError(t, err)
	epoch := primitives.Epoch(2048)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch))))
	block := util.NewBeaconBlock()
	genesisRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	// Set genesis time to be 100 epochs ago.
	offset := int64(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	genesisTime := time.Now().Add(time.Duration(-100*offset) * time.Second)
	mockChainService := &mockChain.ChainService{State: beaconState, Root: genesisRoot[:], Genesis: genesisTime}
	server := &Server{
		HeadFetcher:       mockChainService,
		SyncChecker:       &mockSync.Sync{IsSyncing: false},
		TimeFetcher:       mockChainService,
		StateNotifier:     mockChainService.StateNotifier(),
		OperationNotifier: mockChainService.OperationNotifier(),
		ExitPool:          voluntaryexits.NewPool(),
		P2P:               mockp2p.NewTestP2P(t),
	}

	// Subscribe to operation notifications.
	opChannel := make(chan *feed.Event, 1024)
	opSub := server.OperationNotifier.OperationFeed().Subscribe(opChannel)
	defer opSub.Unsubscribe()

	// Send the request, expect a result on the state feed.
	validatorIndex := primitives.ValidatorIndex(0)
	req := &ethpb.SignedVoluntaryExit{
		Exit: &ethpb.VoluntaryExit{
			Epoch:          epoch,
			ValidatorIndex: validatorIndex,
		},
	}
	req.Signature, err = signing.ComputeDomainAndSign(beaconState, epoch, req.Exit, params.BeaconConfig().DomainVoluntaryExit, keys[0])
	require.NoError(t, err)

	resp, err := server.ProposeExit(context.Background(), req)
	require.NoError(t, err)
	expectedRoot, err := req.Exit.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedRoot[:], resp.ExitRoot)

	// Ensure the state notification was broadcast.
	notificationFound := false
	for !notificationFound {
		select {
		case event := <-opChannel:
			if event.Type == opfeed.ExitReceived {
				notificationFound = true
				data, ok := event.Data.(*opfeed.ExitReceivedData)
				assert.Equal(t, true, ok, "Entity is of the wrong type")
				assert.NotNil(t, data.Exit)
			}
		case <-opSub.Err():
			t.Error("Subscription to state notifier failed")
			return
		}
	}
}

func TestProposeExit_NoPanic(t *testing.T) {
	ctx := context.Background()

	deposits, keys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	require.NoError(t, err)
	beaconState, err := transition.GenesisBeaconState(ctx, deposits, 0, &ethpb.Eth1Data{BlockHash: make([]byte, 32)})
	require.NoError(t, err)
	epoch := primitives.Epoch(2048)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch))))
	block := util.NewBeaconBlock()
	genesisRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	// Set genesis time to be 100 epochs ago.
	offset := int64(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	genesisTime := time.Now().Add(time.Duration(-100*offset) * time.Second)
	mockChainService := &mockChain.ChainService{State: beaconState, Root: genesisRoot[:], Genesis: genesisTime}
	server := &Server{
		HeadFetcher:       mockChainService,
		SyncChecker:       &mockSync.Sync{IsSyncing: false},
		TimeFetcher:       mockChainService,
		StateNotifier:     mockChainService.StateNotifier(),
		OperationNotifier: mockChainService.OperationNotifier(),
		ExitPool:          voluntaryexits.NewPool(),
		P2P:               mockp2p.NewTestP2P(t),
	}

	// Subscribe to operation notifications.
	opChannel := make(chan *feed.Event, 1024)
	opSub := server.OperationNotifier.OperationFeed().Subscribe(opChannel)
	defer opSub.Unsubscribe()

	req := &ethpb.SignedVoluntaryExit{}
	_, err = server.ProposeExit(context.Background(), req)
	require.ErrorContains(t, "voluntary exit does not exist", err, "Expected error for no exit existing")

	// Send the request, expect a result on the state feed.
	validatorIndex := primitives.ValidatorIndex(0)
	req = &ethpb.SignedVoluntaryExit{
		Exit: &ethpb.VoluntaryExit{
			Epoch:          epoch,
			ValidatorIndex: validatorIndex,
		},
	}

	_, err = server.ProposeExit(context.Background(), req)
	require.ErrorContains(t, "invalid signature provided", err, "Expected error for no signature exists")
	req.Signature = bytesutil.FromBytes48([fieldparams.BLSPubkeyLength]byte{})

	_, err = server.ProposeExit(context.Background(), req)
	require.ErrorContains(t, "invalid signature provided", err, "Expected error for invalid signature length")
	req.Signature, err = signing.ComputeDomainAndSign(beaconState, epoch, req.Exit, params.BeaconConfig().DomainVoluntaryExit, keys[0])
	require.NoError(t, err)
	resp, err := server.ProposeExit(context.Background(), req)
	require.NoError(t, err)
	expectedRoot, err := req.Exit.HashTreeRoot()
	require.NoError(t, err)
	assert.DeepEqual(t, expectedRoot[:], resp.ExitRoot)
}
