package validator

import (
	"context"
	"encoding/binary"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpbv1 "github.com/prysmaticlabs/prysm/v4/proto/eth/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

// pubKey is a helper to generate a well-formed public key.
func pubKey(i uint64) []byte {
	pubKey := make([]byte, params.BeaconConfig().BLSPubkeyLength)
	binary.LittleEndian.PutUint64(pubKey, i)
	return pubKey
}

func TestGetDuties_OK(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	pubKeys := make([][]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
		indices[i] = uint64(i)
	}

	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Genesis: time.Now(),
	}
	vs := &Server{
		HeadFetcher:    chain,
		TimeFetcher:    chain,
		SyncChecker:    &mockSync.Sync{IsSyncing: false},
		PayloadIDCache: cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// Test the last validator in registry.
	lastValidatorIndex := depChainStart - 1
	req = &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[lastValidatorIndex].Data.PublicKey},
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// We request for duties for all validators.
	req = &ethpb.DutiesRequest{
		PublicKeys: pubKeys,
		Epoch:      0,
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		assert.Equal(t, primitives.ValidatorIndex(i), res.CurrentEpochDuties[i].ValidatorIndex)
	}
}

func TestGetAltairDuties_SyncCommitteeOK(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.AltairForkEpoch = primitives.Epoch(0)
	params.OverrideBeaconConfig(cfg)

	genesis := util.NewBeaconBlock()
	deposits, _, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().SyncCommitteeSize)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := util.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis bs")
	h := &ethpb.BeaconBlockHeader{
		StateRoot:  bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
		ParentRoot: bytesutil.PadTo([]byte{'b'}, fieldparams.RootLength),
		BodyRoot:   bytesutil.PadTo([]byte{'c'}, fieldparams.RootLength),
	}
	require.NoError(t, bs.SetLatestBlockHeader(h))
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	syncCommittee, err := altair.NextSyncCommittee(context.Background(), bs)
	require.NoError(t, err)
	require.NoError(t, bs.SetCurrentSyncCommittee(syncCommittee))
	pubKeys := make([][]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
		indices[i] = uint64(i)
	}
	require.NoError(t, bs.SetSlot(params.BeaconConfig().SlotsPerEpoch*primitives.Slot(params.BeaconConfig().EpochsPerSyncCommitteePeriod)-1))
	require.NoError(t, helpers.UpdateSyncCommitteeCache(bs))

	pubkeysAs48ByteType := make([][fieldparams.BLSPubkeyLength]byte, len(pubKeys))
	for i, pk := range pubKeys {
		pubkeysAs48ByteType[i] = bytesutil.ToBytes48(pk)
	}

	slot := uint64(params.BeaconConfig().SlotsPerEpoch) * uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * params.BeaconConfig().SecondsPerSlot
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Genesis: time.Now().Add(time.Duration(-1*int64(slot-1)) * time.Second),
	}
	vs := &Server{
		HeadFetcher:     chain,
		TimeFetcher:     chain,
		Eth1InfoFetcher: &mockExecution.Chain{},
		SyncChecker:     &mockSync.Sync{IsSyncing: false},
		PayloadIDCache:  cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// Test the last validator in registry.
	lastValidatorIndex := params.BeaconConfig().SyncCommitteeSize - 1
	req = &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[lastValidatorIndex].Data.PublicKey},
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// We request for duties for all validators.
	req = &ethpb.DutiesRequest{
		PublicKeys: pubKeys,
		Epoch:      0,
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		require.Equal(t, primitives.ValidatorIndex(i), res.CurrentEpochDuties[i].ValidatorIndex)
	}
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		require.Equal(t, true, res.CurrentEpochDuties[i].IsSyncCommittee)
		// Current epoch and next epoch duties should be equal before the sync period epoch boundary.
		require.Equal(t, res.CurrentEpochDuties[i].IsSyncCommittee, res.NextEpochDuties[i].IsSyncCommittee)
	}

	// Current epoch and next epoch duties should not be equal at the sync period epoch boundary.
	req = &ethpb.DutiesRequest{
		PublicKeys: pubKeys,
		Epoch:      params.BeaconConfig().EpochsPerSyncCommitteePeriod - 1,
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		require.NotEqual(t, res.CurrentEpochDuties[i].IsSyncCommittee, res.NextEpochDuties[i].IsSyncCommittee)
	}
}

func TestGetBellatrixDuties_SyncCommitteeOK(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.AltairForkEpoch = primitives.Epoch(0)
	cfg.BellatrixForkEpoch = primitives.Epoch(1)
	params.OverrideBeaconConfig(cfg)

	genesis := util.NewBeaconBlock()
	deposits, _, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().SyncCommitteeSize)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := util.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	h := &ethpb.BeaconBlockHeader{
		StateRoot:  bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
		ParentRoot: bytesutil.PadTo([]byte{'b'}, fieldparams.RootLength),
		BodyRoot:   bytesutil.PadTo([]byte{'c'}, fieldparams.RootLength),
	}
	require.NoError(t, bs.SetLatestBlockHeader(h))
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	syncCommittee, err := altair.NextSyncCommittee(context.Background(), bs)
	require.NoError(t, err)
	require.NoError(t, bs.SetCurrentSyncCommittee(syncCommittee))
	pubKeys := make([][]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
		indices[i] = uint64(i)
	}
	require.NoError(t, bs.SetSlot(params.BeaconConfig().SlotsPerEpoch*primitives.Slot(params.BeaconConfig().EpochsPerSyncCommitteePeriod)-1))
	require.NoError(t, helpers.UpdateSyncCommitteeCache(bs))

	bs, err = execution.UpgradeToBellatrix(bs)
	require.NoError(t, err)

	pubkeysAs48ByteType := make([][fieldparams.BLSPubkeyLength]byte, len(pubKeys))
	for i, pk := range pubKeys {
		pubkeysAs48ByteType[i] = bytesutil.ToBytes48(pk)
	}

	slot := uint64(params.BeaconConfig().SlotsPerEpoch) * uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * params.BeaconConfig().SecondsPerSlot
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Genesis: time.Now().Add(time.Duration(-1*int64(slot-1)) * time.Second),
	}
	vs := &Server{
		HeadFetcher:     chain,
		TimeFetcher:     chain,
		Eth1InfoFetcher: &mockExecution.Chain{},
		SyncChecker:     &mockSync.Sync{IsSyncing: false},
		PayloadIDCache:  cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// Test the last validator in registry.
	lastValidatorIndex := params.BeaconConfig().SyncCommitteeSize - 1
	req = &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[lastValidatorIndex].Data.PublicKey},
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	if res.CurrentEpochDuties[0].AttesterSlot > bs.Slot()+params.BeaconConfig().SlotsPerEpoch {
		t.Errorf("Assigned slot %d can't be higher than %d",
			res.CurrentEpochDuties[0].AttesterSlot, bs.Slot()+params.BeaconConfig().SlotsPerEpoch)
	}

	// We request for duties for all validators.
	req = &ethpb.DutiesRequest{
		PublicKeys: pubKeys,
		Epoch:      0,
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		assert.Equal(t, primitives.ValidatorIndex(i), res.CurrentEpochDuties[i].ValidatorIndex)
	}
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		assert.Equal(t, true, res.CurrentEpochDuties[i].IsSyncCommittee)
		// Current epoch and next epoch duties should be equal before the sync period epoch boundary.
		assert.Equal(t, res.CurrentEpochDuties[i].IsSyncCommittee, res.NextEpochDuties[i].IsSyncCommittee)
	}

	// Current epoch and next epoch duties should not be equal at the sync period epoch boundary.
	req = &ethpb.DutiesRequest{
		PublicKeys: pubKeys,
		Epoch:      params.BeaconConfig().EpochsPerSyncCommitteePeriod - 1,
	}
	res, err = vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	for i := 0; i < len(res.CurrentEpochDuties); i++ {
		require.NotEqual(t, res.CurrentEpochDuties[i].IsSyncCommittee, res.NextEpochDuties[i].IsSyncCommittee)
	}
}

func TestGetAltairDuties_UnknownPubkey(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.AltairForkEpoch = primitives.Epoch(0)
	params.OverrideBeaconConfig(cfg)

	genesis := util.NewBeaconBlock()
	deposits, _, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().SyncCommitteeSize)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := util.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err)
	h := &ethpb.BeaconBlockHeader{
		StateRoot:  bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
		ParentRoot: bytesutil.PadTo([]byte{'b'}, fieldparams.RootLength),
		BodyRoot:   bytesutil.PadTo([]byte{'c'}, fieldparams.RootLength),
	}
	require.NoError(t, bs.SetLatestBlockHeader(h))
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	require.NoError(t, bs.SetSlot(params.BeaconConfig().SlotsPerEpoch*primitives.Slot(params.BeaconConfig().EpochsPerSyncCommitteePeriod)-1))
	require.NoError(t, helpers.UpdateSyncCommitteeCache(bs))

	slot := uint64(params.BeaconConfig().SlotsPerEpoch) * uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * params.BeaconConfig().SecondsPerSlot
	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Genesis: time.Now().Add(time.Duration(-1*int64(slot-1)) * time.Second),
	}
	depositCache, err := depositcache.New()
	require.NoError(t, err)

	vs := &Server{
		HeadFetcher:     chain,
		TimeFetcher:     chain,
		Eth1InfoFetcher: &mockExecution.Chain{},
		SyncChecker:     &mockSync.Sync{IsSyncing: false},
		DepositFetcher:  depositCache,
		PayloadIDCache:  cache.NewPayloadIDCache(),
	}

	unknownPubkey := bytesutil.PadTo([]byte{'u'}, 48)
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{unknownPubkey},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err)
	assert.Equal(t, false, res.CurrentEpochDuties[0].IsSyncCommittee)
	assert.Equal(t, false, res.NextEpochDuties[0].IsSyncCommittee)
}

func TestGetDuties_SlotOutOfUpperBound(t *testing.T) {
	chain := &mockChain.ChainService{
		Genesis: time.Now(),
	}
	vs := &Server{
		TimeFetcher: chain,
	}
	req := &ethpb.DutiesRequest{
		Epoch: primitives.Epoch(chain.CurrentSlot()/params.BeaconConfig().SlotsPerEpoch + 2),
	}
	_, err := vs.duties(context.Background(), req)
	require.ErrorContains(t, "can not be greater than next epoch", err)
}

func TestGetDuties_CurrentEpoch_ShouldNotFail(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bState, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis state")
	// Set state to non-epoch start slot.
	require.NoError(t, bState.SetSlot(5))

	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = bytesutil.ToBytes48(deposits[i].Data.PublicKey)
		indices[i] = uint64(i)
	}

	chain := &mockChain.ChainService{
		State: bState, Root: genesisRoot[:], Genesis: time.Now(),
	}
	vs := &Server{
		HeadFetcher:    chain,
		TimeFetcher:    chain,
		SyncChecker:    &mockSync.Sync{IsSyncing: false},
		PayloadIDCache: cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err)
	assert.Equal(t, 1, len(res.CurrentEpochDuties), "Expected 1 assignment")
}

func TestGetDuties_MultipleKeys_OK(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := uint64(64)

	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = bytesutil.ToBytes48(deposits[i].Data.PublicKey)
		indices[i] = uint64(i)
	}

	chain := &mockChain.ChainService{
		State: bs, Root: genesisRoot[:], Genesis: time.Now(),
	}
	vs := &Server{
		HeadFetcher:    chain,
		TimeFetcher:    chain,
		SyncChecker:    &mockSync.Sync{IsSyncing: false},
		PayloadIDCache: cache.NewPayloadIDCache(),
	}

	pubkey0 := deposits[0].Data.PublicKey
	pubkey1 := deposits[1].Data.PublicKey

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{pubkey0, pubkey1},
	}
	res, err := vs.GetDuties(context.Background(), req)
	require.NoError(t, err, "Could not call epoch committee assignment")
	assert.Equal(t, 2, len(res.CurrentEpochDuties))
	assert.Equal(t, primitives.Slot(4), res.CurrentEpochDuties[0].AttesterSlot)
	assert.Equal(t, primitives.Slot(4), res.CurrentEpochDuties[1].AttesterSlot)
}

func TestGetDuties_SyncNotReady(t *testing.T) {
	vs := &Server{
		SyncChecker: &mockSync.Sync{IsSyncing: true},
	}
	_, err := vs.GetDuties(context.Background(), &ethpb.DutiesRequest{})
	assert.ErrorContains(t, "Syncing to latest head", err)
}

func TestStreamDuties_SyncNotReady(t *testing.T) {
	vs := &Server{
		SyncChecker: &mockSync.Sync{IsSyncing: true},
	}
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mockStream := mock.NewMockBeaconNodeValidator_StreamDutiesServer(ctrl)
	assert.ErrorContains(t, "Syncing to latest head", vs.StreamDuties(&ethpb.DutiesRequest{}, mockStream))
}

func TestStreamDuties_OK(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	pubKeys := make([][]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
		indices[i] = uint64(i)
	}

	pubkeysAs48ByteType := make([][fieldparams.BLSPubkeyLength]byte, len(pubKeys))
	for i, pk := range pubKeys {
		pubkeysAs48ByteType[i] = bytesutil.ToBytes48(pk)
	}

	ctx, cancel := context.WithCancel(context.Background())
	c := &mockChain.ChainService{
		Genesis: time.Now(),
	}
	vs := &Server{
		Ctx:            ctx,
		HeadFetcher:    &mockChain.ChainService{State: bs, Root: genesisRoot[:]},
		SyncChecker:    &mockSync.Sync{IsSyncing: false},
		TimeFetcher:    c,
		StateNotifier:  &mockChain.MockStateNotifier{},
		PayloadIDCache: cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	wantedRes, err := vs.duties(ctx, req)
	require.NoError(t, err)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	exitRoutine := make(chan bool)
	mockStream := mock.NewMockBeaconNodeValidator_StreamDutiesServer(ctrl)
	mockStream.EXPECT().Send(wantedRes).Do(func(arg0 interface{}) {
		exitRoutine <- true
	})
	mockStream.EXPECT().Context().Return(ctx).AnyTimes()
	go func(tt *testing.T) {
		assert.ErrorContains(t, "context canceled", vs.StreamDuties(req, mockStream))
	}(t)
	<-exitRoutine
	cancel()
}

func TestStreamDuties_OK_ChainReorg(t *testing.T) {
	genesis := util.NewBeaconBlock()
	depChainStart := params.BeaconConfig().MinGenesisActiveValidatorCount
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(t, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(t, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(t, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err, "Could not get signing root")

	pubKeys := make([][]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = deposits[i].Data.PublicKey
		indices[i] = uint64(i)
	}

	pubkeysAs48ByteType := make([][fieldparams.BLSPubkeyLength]byte, len(pubKeys))
	for i, pk := range pubKeys {
		pubkeysAs48ByteType[i] = bytesutil.ToBytes48(pk)
	}

	ctx, cancel := context.WithCancel(context.Background())
	c := &mockChain.ChainService{
		Genesis: time.Now(),
	}
	vs := &Server{
		Ctx:            ctx,
		HeadFetcher:    &mockChain.ChainService{State: bs, Root: genesisRoot[:]},
		SyncChecker:    &mockSync.Sync{IsSyncing: false},
		TimeFetcher:    c,
		StateNotifier:  &mockChain.MockStateNotifier{},
		PayloadIDCache: cache.NewPayloadIDCache(),
	}

	// Test the first validator in registry.
	req := &ethpb.DutiesRequest{
		PublicKeys: [][]byte{deposits[0].Data.PublicKey},
	}
	wantedRes, err := vs.duties(ctx, req)
	require.NoError(t, err)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	exitRoutine := make(chan bool)
	mockStream := mock.NewMockBeaconNodeValidator_StreamDutiesServer(ctrl)
	mockStream.EXPECT().Send(wantedRes).Return(nil)
	mockStream.EXPECT().Send(wantedRes).Do(func(arg0 interface{}) {
		exitRoutine <- true
	})
	mockStream.EXPECT().Context().Return(ctx).AnyTimes()
	go func(tt *testing.T) {
		assert.ErrorContains(t, "context canceled", vs.StreamDuties(req, mockStream))
	}(t)
	// Fire a reorg event. This needs to trigger
	// a recomputation and resending of duties over the stream.
	for sent := 0; sent == 0; {
		sent = vs.StateNotifier.StateFeed().Send(&feed.Event{
			Type: statefeed.Reorg,
			Data: &ethpbv1.EventChainReorg{Depth: uint64(params.BeaconConfig().SlotsPerEpoch), Slot: 0},
		})
	}
	<-exitRoutine
	cancel()
}

func BenchmarkCommitteeAssignment(b *testing.B) {

	genesis := util.NewBeaconBlock()
	depChainStart := uint64(8192 * 2)
	deposits, _, err := util.DeterministicDepositsAndKeys(depChainStart)
	require.NoError(b, err)
	eth1Data, err := util.DeterministicEth1Data(len(deposits))
	require.NoError(b, err)
	bs, err := transition.GenesisBeaconState(context.Background(), deposits, 0, eth1Data)
	require.NoError(b, err, "Could not setup genesis bs")
	genesisRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(b, err, "Could not get signing root")

	pubKeys := make([][fieldparams.BLSPubkeyLength]byte, len(deposits))
	indices := make([]uint64, len(deposits))
	for i := 0; i < len(deposits); i++ {
		pubKeys[i] = bytesutil.ToBytes48(deposits[i].Data.PublicKey)
		indices[i] = uint64(i)
	}

	chain := &mockChain.ChainService{State: bs, Root: genesisRoot[:]}
	vs := &Server{
		HeadFetcher: chain,
		TimeFetcher: chain,
		SyncChecker: &mockSync.Sync{IsSyncing: false},
	}

	// Create request for all validators in the system.
	pks := make([][]byte, len(deposits))
	for i, deposit := range deposits {
		pks[i] = deposit.Data.PublicKey
	}
	req := &ethpb.DutiesRequest{
		PublicKeys: pks,
		Epoch:      0,
	}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_, err := vs.GetDuties(context.Background(), req)
		assert.NoError(b, err)
	}
}
