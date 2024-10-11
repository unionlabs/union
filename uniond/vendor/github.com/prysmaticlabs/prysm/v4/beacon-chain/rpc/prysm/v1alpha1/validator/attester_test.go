package validator

import (
	"context"
	"math/rand"
	"sync"
	"testing"
	"time"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	dbutil "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
)

func TestProposeAttestation_OK(t *testing.T) {
	attesterServer := &Server{
		HeadFetcher:       &mock.ChainService{},
		P2P:               &mockp2p.MockBroadcaster{},
		AttPool:           attestations.NewPool(),
		OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
	}
	head := util.NewBeaconBlock()
	head.Block.Slot = 999
	head.Block.ParentRoot = bytesutil.PadTo([]byte{'a'}, 32)
	root, err := head.Block.HashTreeRoot()
	require.NoError(t, err)

	validators := make([]*ethpb.Validator, 64)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			PublicKey:             make([]byte, 48),
			WithdrawalCredentials: make([]byte, 32),
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance:      params.BeaconConfig().MaxEffectiveBalance,
		}
	}

	state, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, state.SetSlot(params.BeaconConfig().SlotsPerEpoch+1))
	require.NoError(t, state.SetValidators(validators))

	sk, err := bls.RandKey()
	require.NoError(t, err)
	sig := sk.Sign([]byte("dummy_test_data"))
	req := &ethpb.Attestation{
		Signature: sig.Marshal(),
		Data: &ethpb.AttestationData{
			BeaconBlockRoot: root[:],
			Source:          &ethpb.Checkpoint{Root: make([]byte, 32)},
			Target:          &ethpb.Checkpoint{Root: make([]byte, 32)},
		},
	}
	_, err = attesterServer.ProposeAttestation(context.Background(), req)
	assert.NoError(t, err)
}

func TestProposeAttestation_IncorrectSignature(t *testing.T) {
	attesterServer := &Server{
		HeadFetcher:       &mock.ChainService{},
		P2P:               &mockp2p.MockBroadcaster{},
		AttPool:           attestations.NewPool(),
		OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
	}

	req := util.HydrateAttestation(&ethpb.Attestation{})
	wanted := "Incorrect attestation signature"
	_, err := attesterServer.ProposeAttestation(context.Background(), req)
	assert.ErrorContains(t, wanted, err)
}

func TestGetAttestationData_OK(t *testing.T) {
	block := util.NewBeaconBlock()
	block.Block.Slot = 3*params.BeaconConfig().SlotsPerEpoch + 1
	targetBlock := util.NewBeaconBlock()
	targetBlock.Block.Slot = 1 * params.BeaconConfig().SlotsPerEpoch
	targetRoot, err := targetBlock.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root for target block")

	justifiedBlock := util.NewBeaconBlock()
	justifiedBlock.Block.Slot = 2 * params.BeaconConfig().SlotsPerEpoch
	blockRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not hash beacon block")
	justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root for justified block")
	slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
	justifiedCheckpoint := &ethpb.Checkpoint{
		Epoch: 2,
		Root:  justifiedRoot[:],
	}
	offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
	attesterServer := &Server{
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
		CoreService: &core.Service{
			HeadFetcher: &mock.ChainService{TargetRoot: targetRoot, Root: blockRoot[:]},
			GenesisTimeFetcher: &mock.ChainService{
				Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second),
			},
			FinalizedFetcher:      &mock.ChainService{CurrentJustifiedCheckPoint: justifiedCheckpoint},
			AttestationCache:      cache.NewAttestationCache(),
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		},
	}

	req := &ethpb.AttestationDataRequest{
		CommitteeIndex: 0,
		Slot:           3*params.BeaconConfig().SlotsPerEpoch + 1,
	}
	res, err := attesterServer.GetAttestationData(context.Background(), req)
	require.NoError(t, err, "Could not get attestation info at slot")

	expectedInfo := &ethpb.AttestationData{
		Slot:            3*params.BeaconConfig().SlotsPerEpoch + 1,
		BeaconBlockRoot: blockRoot[:],
		Source: &ethpb.Checkpoint{
			Epoch: 2,
			Root:  justifiedRoot[:],
		},
		Target: &ethpb.Checkpoint{
			Epoch: 3,
			Root:  targetRoot[:],
		},
	}

	if !proto.Equal(res, expectedInfo) {
		t.Errorf("Expected attestation info to match, received %v, wanted %v", res, expectedInfo)
	}
}

func BenchmarkGetAttestationDataConcurrent(b *testing.B) {
	block := util.NewBeaconBlock()
	block.Block.Slot = 3*params.BeaconConfig().SlotsPerEpoch + 1
	targetBlock := util.NewBeaconBlock()
	targetBlock.Block.Slot = 1 * params.BeaconConfig().SlotsPerEpoch
	targetRoot, err := targetBlock.Block.HashTreeRoot()
	require.NoError(b, err, "Could not get signing root for target block")

	justifiedBlock := util.NewBeaconBlock()
	justifiedBlock.Block.Slot = 2 * params.BeaconConfig().SlotsPerEpoch
	blockRoot, err := block.Block.HashTreeRoot()
	require.NoError(b, err, "Could not hash beacon block")
	justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
	require.NoError(b, err, "Could not get signing root for justified block")
	slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
	justifiedCheckpoint := &ethpb.Checkpoint{
		Epoch: 2,
		Root:  justifiedRoot[:],
	}
	offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
	attesterServer := &Server{
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
		CoreService: &core.Service{
			AttestationCache: cache.NewAttestationCache(),
			HeadFetcher:      &mock.ChainService{TargetRoot: targetRoot, Root: blockRoot[:]},
			GenesisTimeFetcher: &mock.ChainService{
				Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second),
			},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
			FinalizedFetcher:      &mock.ChainService{CurrentJustifiedCheckPoint: justifiedCheckpoint},
		},
	}

	req := &ethpb.AttestationDataRequest{
		CommitteeIndex: 0,
		Slot:           3*params.BeaconConfig().SlotsPerEpoch + 1,
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		var wg sync.WaitGroup
		wg.Add(5000) // for 5000 concurrent accesses

		for j := 0; j < 5000; j++ {
			go func() {
				defer wg.Done()
				_, err := attesterServer.GetAttestationData(context.Background(), req)
				require.NoError(b, err, "Could not get attestation info at slot")
			}()
		}
		wg.Wait() // Wait for all goroutines to finish
	}

	b.Log("Elapsed time:", b.Elapsed())
}

func TestGetAttestationData_SyncNotReady(t *testing.T) {
	as := Server{
		SyncChecker: &mockSync.Sync{IsSyncing: true},
	}
	_, err := as.GetAttestationData(context.Background(), &ethpb.AttestationDataRequest{})
	assert.ErrorContains(t, "Syncing to latest head", err)
}

func TestGetAttestationData_Optimistic(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.BellatrixForkEpoch = 0
	params.OverrideBeaconConfig(cfg)

	as := &Server{
		SyncChecker:           &mockSync.Sync{},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: true},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now()},
		CoreService: &core.Service{
			GenesisTimeFetcher:    &mock.ChainService{Genesis: time.Now()},
			HeadFetcher:           &mock.ChainService{},
			AttestationCache:      cache.NewAttestationCache(),
			OptimisticModeFetcher: &mock.ChainService{Optimistic: true},
		},
	}
	_, err := as.GetAttestationData(context.Background(), &ethpb.AttestationDataRequest{})
	s, ok := status.FromError(err)
	require.Equal(t, true, ok)
	require.DeepEqual(t, codes.Unavailable, s.Code())
	require.ErrorContains(t, errOptimisticMode.Error(), err)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)
	as = &Server{
		SyncChecker:           &mockSync.Sync{},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now()},
		CoreService: &core.Service{
			AttestationCache:      cache.NewAttestationCache(),
			GenesisTimeFetcher:    &mock.ChainService{Genesis: time.Now()},
			HeadFetcher:           &mock.ChainService{Optimistic: false, State: beaconState},
			FinalizedFetcher:      &mock.ChainService{CurrentJustifiedCheckPoint: &ethpb.Checkpoint{}},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		},
	}
	_, err = as.GetAttestationData(context.Background(), &ethpb.AttestationDataRequest{})
	require.NoError(t, err)
}

func TestServer_GetAttestationData_InvalidRequestSlot(t *testing.T) {
	ctx := context.Background()

	slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
	offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
	attesterServer := &Server{
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
		CoreService: &core.Service{
			GenesisTimeFetcher:    &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		},
	}

	req := &ethpb.AttestationDataRequest{
		Slot: 1000000000000,
	}
	_, err := attesterServer.GetAttestationData(ctx, req)
	assert.ErrorContains(t, "invalid request", err)
}

func TestServer_GetAttestationData_RequestSlotIsDifferentThanCurrentSlot(t *testing.T) {
	ctx := context.Background()
	db := dbutil.SetupDB(t)

	slot := 3*params.BeaconConfig().SlotsPerEpoch + 1
	block := util.NewBeaconBlock()
	block.Block.Slot = slot
	block2 := util.NewBeaconBlock()
	block2.Block.Slot = slot - 1
	targetBlock := util.NewBeaconBlock()
	targetBlock.Block.Slot = 1 * params.BeaconConfig().SlotsPerEpoch
	justifiedBlock := util.NewBeaconBlock()
	justifiedBlock.Block.Slot = 2 * params.BeaconConfig().SlotsPerEpoch
	blockRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not hash beacon block")
	blockRoot2, err := block2.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, db, block2)
	justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root for justified block")
	justifiedCheckpoint := &ethpb.Checkpoint{
		Epoch: 2,
		Root:  justifiedRoot[:],
	}
	offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
	attesterServer := &Server{
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
		CoreService: &core.Service{
			HeadFetcher:           &mock.ChainService{TargetRoot: blockRoot2, Root: blockRoot[:]},
			GenesisTimeFetcher:    &mock.ChainService{Genesis: time.Now().Add(time.Duration(-1*offset) * time.Second)},
			StateGen:              stategen.New(db, doublylinkedtree.New()),
			FinalizedFetcher:      &mock.ChainService{CurrentJustifiedCheckPoint: justifiedCheckpoint},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		},
	}
	util.SaveBlock(t, ctx, db, block)

	req := &ethpb.AttestationDataRequest{
		CommitteeIndex: 0,
		Slot:           slot - 1,
	}
	_, err = attesterServer.GetAttestationData(ctx, req)
	require.ErrorContains(t, "invalid request: slot 24 is not the current slot 25", err)
}

func TestGetAttestationData_SucceedsInFirstEpoch(t *testing.T) {
	slot := primitives.Slot(5)
	block := util.NewBeaconBlock()
	block.Block.Slot = slot
	targetBlock := util.NewBeaconBlock()
	targetBlock.Block.Slot = 0
	justifiedBlock := util.NewBeaconBlock()
	justifiedBlock.Block.Slot = 0
	blockRoot, err := block.Block.HashTreeRoot()
	require.NoError(t, err, "Could not hash beacon block")
	justifiedRoot, err := justifiedBlock.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root for justified block")
	targetRoot, err := targetBlock.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root for target block")

	justifiedCheckpoint := &ethpb.Checkpoint{
		Epoch: 0,
		Root:  justifiedRoot[:],
	}
	offset := int64(slot.Mul(params.BeaconConfig().SecondsPerSlot))
	attesterServer := &Server{
		SyncChecker:           &mockSync.Sync{IsSyncing: false},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		TimeFetcher:           &mock.ChainService{Genesis: prysmTime.Now().Add(time.Duration(-1*offset) * time.Second)},
		CoreService: &core.Service{
			AttestationCache: cache.NewAttestationCache(),
			HeadFetcher: &mock.ChainService{
				TargetRoot: targetRoot, Root: blockRoot[:],
			},
			GenesisTimeFetcher:    &mock.ChainService{Genesis: prysmTime.Now().Add(time.Duration(-1*offset) * time.Second)},
			FinalizedFetcher:      &mock.ChainService{CurrentJustifiedCheckPoint: justifiedCheckpoint},
			OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
		},
	}

	req := &ethpb.AttestationDataRequest{
		CommitteeIndex: 0,
		Slot:           5,
	}
	res, err := attesterServer.GetAttestationData(context.Background(), req)
	require.NoError(t, err, "Could not get attestation info at slot")

	expectedInfo := &ethpb.AttestationData{
		Slot:            slot,
		BeaconBlockRoot: blockRoot[:],
		Source: &ethpb.Checkpoint{
			Epoch: 0,
			Root:  justifiedRoot[:],
		},
		Target: &ethpb.Checkpoint{
			Epoch: 0,
			Root:  targetRoot[:],
		},
	}

	if !proto.Equal(res, expectedInfo) {
		t.Errorf("Expected attestation info to match, received %v, wanted %v", res, expectedInfo)
	}
}

func TestServer_SubscribeCommitteeSubnets_NoSlots(t *testing.T) {
	attesterServer := &Server{
		HeadFetcher:       &mock.ChainService{},
		P2P:               &mockp2p.MockBroadcaster{},
		AttPool:           attestations.NewPool(),
		OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
	}

	_, err := attesterServer.SubscribeCommitteeSubnets(context.Background(), &ethpb.CommitteeSubnetsSubscribeRequest{
		Slots:        nil,
		CommitteeIds: nil,
		IsAggregator: nil,
	})
	assert.ErrorContains(t, "no attester slots provided", err)
}

func TestServer_SubscribeCommitteeSubnets_DifferentLengthSlots(t *testing.T) {
	// fixed seed
	s := rand.NewSource(10)
	randGen := rand.New(s)

	attesterServer := &Server{
		HeadFetcher:       &mock.ChainService{},
		P2P:               &mockp2p.MockBroadcaster{},
		AttPool:           attestations.NewPool(),
		OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
	}

	var ss []primitives.Slot
	var comIdxs []primitives.CommitteeIndex
	var isAggregator []bool

	for i := primitives.Slot(100); i < 200; i++ {
		ss = append(ss, i)
		comIdxs = append(comIdxs, primitives.CommitteeIndex(randGen.Int63n(64)))
		boolVal := randGen.Uint64()%2 == 0
		isAggregator = append(isAggregator, boolVal)
	}

	ss = append(ss, 321)

	_, err := attesterServer.SubscribeCommitteeSubnets(context.Background(), &ethpb.CommitteeSubnetsSubscribeRequest{
		Slots:        ss,
		CommitteeIds: comIdxs,
		IsAggregator: isAggregator,
	})
	assert.ErrorContains(t, "request fields are not the same length", err)
}

func TestServer_SubscribeCommitteeSubnets_MultipleSlots(t *testing.T) {
	// fixed seed
	s := rand.NewSource(10)
	randGen := rand.New(s)

	validators := make([]*ethpb.Validator, 64)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:        params.BeaconConfig().FarFutureEpoch,
			EffectiveBalance: params.BeaconConfig().MaxEffectiveBalance,
		}
	}

	state, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, state.SetValidators(validators))

	attesterServer := &Server{
		HeadFetcher:       &mock.ChainService{State: state},
		P2P:               &mockp2p.MockBroadcaster{},
		AttPool:           attestations.NewPool(),
		OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
	}

	var ss []primitives.Slot
	var comIdxs []primitives.CommitteeIndex
	var isAggregator []bool

	for i := primitives.Slot(100); i < 200; i++ {
		ss = append(ss, i)
		comIdxs = append(comIdxs, primitives.CommitteeIndex(randGen.Int63n(64)))
		boolVal := randGen.Uint64()%2 == 0
		isAggregator = append(isAggregator, boolVal)
	}

	_, err = attesterServer.SubscribeCommitteeSubnets(context.Background(), &ethpb.CommitteeSubnetsSubscribeRequest{
		Slots:        ss,
		CommitteeIds: comIdxs,
		IsAggregator: isAggregator,
	})
	require.NoError(t, err)
	for i := primitives.Slot(100); i < 200; i++ {
		subnets := cache.SubnetIDs.GetAttesterSubnetIDs(i)
		assert.Equal(t, 1, len(subnets))
		if isAggregator[i-100] {
			subnets = cache.SubnetIDs.GetAggregatorSubnetIDs(i)
			assert.Equal(t, 1, len(subnets))
		}
	}
}
