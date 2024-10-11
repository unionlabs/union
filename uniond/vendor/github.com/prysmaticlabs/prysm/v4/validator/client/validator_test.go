package client

import (
	"context"
	"errors"
	"fmt"
	"io"
	"math"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/golang/protobuf/ptypes/empty"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorserviceconfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	validatorType "github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	blsmock "github.com/prysmaticlabs/prysm/v4/crypto/bls/common/mock"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	mock2 "github.com/prysmaticlabs/prysm/v4/testing/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	validatormock "github.com/prysmaticlabs/prysm/v4/testing/validator-mock"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	dbTest "github.com/prysmaticlabs/prysm/v4/validator/db/testing"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	remoteweb3signer "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer"
	"github.com/sirupsen/logrus"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
)

func init() {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)
}

var _ iface.Validator = (*validator)(nil)

const cancelledCtx = "context has been canceled"

func genMockKeymanager(t *testing.T, numKeys int) *mockKeymanager {
	pairs := make([]keypair, numKeys)
	for i := 0; i < numKeys; i++ {
		pairs[i] = randKeypair(t)
	}

	return newMockKeymanager(t, pairs...)
}

type keypair struct {
	pub [fieldparams.BLSPubkeyLength]byte
	pri bls.SecretKey
}

func randKeypair(t *testing.T) keypair {
	pri, err := bls.RandKey()
	require.NoError(t, err)
	var pub [fieldparams.BLSPubkeyLength]byte
	copy(pub[:], pri.PublicKey().Marshal())
	return keypair{pub: pub, pri: pri}
}

func newMockKeymanager(t *testing.T, pairs ...keypair) *mockKeymanager {
	m := &mockKeymanager{keysMap: make(map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey)}
	require.NoError(t, m.add(pairs...))
	return m
}

type mockKeymanager struct {
	lock                sync.RWMutex
	keysMap             map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey
	keys                [][fieldparams.BLSPubkeyLength]byte
	fetchNoKeys         bool
	accountsChangedFeed *event.Feed
}

var errMockKeyExists = errors.New("key already in mockKeymanager map")

func (m *mockKeymanager) add(pairs ...keypair) error {
	for _, kp := range pairs {
		if _, exists := m.keysMap[kp.pub]; exists {
			return errMockKeyExists
		}
		m.keys = append(m.keys, kp.pub)
		m.keysMap[kp.pub] = kp.pri
	}
	return nil
}

func (m *mockKeymanager) remove(pairs ...keypair) {
	for _, kp := range pairs {
		if _, exists := m.keysMap[kp.pub]; !exists {
			continue
		}
		m.removeOne(kp)
	}
}

func (m *mockKeymanager) removeOne(kp keypair) {
	delete(m.keysMap, kp.pub)
	if m.keys[0] == kp.pub {
		m.keys = m.keys[1:]
		return
	}
	for i := 1; i < len(m.keys); i++ {
		if m.keys[i] == kp.pub {
			m.keys = append(m.keys[0:i-1], m.keys[i:]...)
			return
		}
	}
}

func (m *mockKeymanager) FetchValidatingPublicKeys(ctx context.Context) ([][fieldparams.BLSPubkeyLength]byte, error) {
	m.lock.RLock()
	defer m.lock.RUnlock()
	if m.fetchNoKeys {
		m.fetchNoKeys = false
		return [][fieldparams.BLSPubkeyLength]byte{}, nil
	}
	return m.keys, nil
}

func (m *mockKeymanager) Sign(_ context.Context, req *validatorpb.SignRequest) (bls.Signature, error) {
	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], req.PublicKey)
	privKey, ok := m.keysMap[pubKey]
	if !ok {
		return nil, errors.New("not found")
	}
	sig := privKey.Sign(req.SigningRoot)
	return sig, nil
}

func (m *mockKeymanager) SubscribeAccountChanges(pubKeysChan chan [][fieldparams.BLSPubkeyLength]byte) event.Subscription {
	if m.accountsChangedFeed == nil {
		m.accountsChangedFeed = &event.Feed{}
	}
	return m.accountsChangedFeed.Subscribe(pubKeysChan)
}

func (m *mockKeymanager) SimulateAccountChanges(newKeys [][fieldparams.BLSPubkeyLength]byte) {
	m.accountsChangedFeed.Send(newKeys)
}

func (*mockKeymanager) ExtractKeystores(
	_ context.Context, _ []bls.PublicKey, _ string,
) ([]*keymanager.Keystore, error) {
	return nil, errors.New("extracting keys not supported on mock keymanager")
}

func (*mockKeymanager) ListKeymanagerAccounts(
	context.Context, keymanager.ListKeymanagerAccountConfig) error {
	return nil
}

func (*mockKeymanager) DeleteKeystores(context.Context, [][]byte,
) ([]*keymanager.KeyStatus, error) {
	return nil, nil
}

func generateMockStatusResponse(pubkeys [][]byte) *ethpb.ValidatorActivationResponse {
	multipleStatus := make([]*ethpb.ValidatorActivationResponse_Status, len(pubkeys))
	for i, key := range pubkeys {
		multipleStatus[i] = &ethpb.ValidatorActivationResponse_Status{
			PublicKey: key,
			Status: &ethpb.ValidatorStatusResponse{
				Status: ethpb.ValidatorStatus_UNKNOWN_STATUS,
			},
		}
	}
	return &ethpb.ValidatorActivationResponse{Statuses: multipleStatus}
}

func TestWaitForChainStart_SetsGenesisInfo(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	v := validator{
		validatorClient: client,
		db:              db,
	}

	// Make sure its clean at the start.
	savedGenValRoot, err := db.GenesisValidatorsRoot(context.Background())
	require.NoError(t, err)
	assert.DeepEqual(t, []byte(nil), savedGenValRoot, "Unexpected saved genesis validators root")

	genesis := uint64(time.Unix(1, 0).Unix())
	genesisValidatorsRoot := bytesutil.ToBytes32([]byte("validators"))
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(&ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesis,
		GenesisValidatorsRoot: genesisValidatorsRoot[:],
	}, nil)
	require.NoError(t, v.WaitForChainStart(context.Background()))
	savedGenValRoot, err = db.GenesisValidatorsRoot(context.Background())
	require.NoError(t, err)

	assert.DeepEqual(t, genesisValidatorsRoot[:], savedGenValRoot, "Unexpected saved genesis validators root")
	assert.Equal(t, genesis, v.genesisTime, "Unexpected chain start time")
	assert.NotNil(t, v.ticker, "Expected ticker to be set, received nil")

	// Make sure there are no errors running if it is the same data.
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(&ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesis,
		GenesisValidatorsRoot: genesisValidatorsRoot[:],
	}, nil)
	require.NoError(t, v.WaitForChainStart(context.Background()))
}

func TestWaitForChainStart_SetsGenesisInfo_IncorrectSecondTry(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	v := validator{
		validatorClient: client,
		db:              db,
	}
	genesis := uint64(time.Unix(1, 0).Unix())
	genesisValidatorsRoot := bytesutil.ToBytes32([]byte("validators"))
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(&ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesis,
		GenesisValidatorsRoot: genesisValidatorsRoot[:],
	}, nil)
	require.NoError(t, v.WaitForChainStart(context.Background()))
	savedGenValRoot, err := db.GenesisValidatorsRoot(context.Background())
	require.NoError(t, err)

	assert.DeepEqual(t, genesisValidatorsRoot[:], savedGenValRoot, "Unexpected saved genesis validators root")
	assert.Equal(t, genesis, v.genesisTime, "Unexpected chain start time")
	assert.NotNil(t, v.ticker, "Expected ticker to be set, received nil")

	genesisValidatorsRoot = bytesutil.ToBytes32([]byte("badvalidators"))

	// Make sure there are no errors running if it is the same data.
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(&ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesis,
		GenesisValidatorsRoot: genesisValidatorsRoot[:],
	}, nil)
	err = v.WaitForChainStart(context.Background())
	require.ErrorContains(t, "does not match root saved", err)
}

func TestWaitForChainStart_ContextCanceled(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	v := validator{
		//keyManager:      testKeyManager,
		validatorClient: client,
	}
	genesis := uint64(time.Unix(0, 0).Unix())
	genesisValidatorsRoot := bytesutil.PadTo([]byte("validators"), 32)
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(&ethpb.ChainStartResponse{
		Started:               true,
		GenesisTime:           genesis,
		GenesisValidatorsRoot: genesisValidatorsRoot,
	}, nil)
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	assert.ErrorContains(t, cancelledCtx, v.WaitForChainStart(ctx))
}

func TestWaitForChainStart_ReceiveErrorFromStream(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	v := validator{
		validatorClient: client,
	}
	client.EXPECT().WaitForChainStart(
		gomock.Any(),
		&emptypb.Empty{},
	).Return(nil, errors.New("fails"))
	err := v.WaitForChainStart(context.Background())
	want := "could not receive ChainStart from stream"
	assert.ErrorContains(t, want, err)
}

func TestCanonicalHeadSlot_FailedRPC(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockBeaconChainClient(ctrl)
	v := validator{
		beaconClient: client,
		genesisTime:  1,
	}
	client.EXPECT().GetChainHead(
		gomock.Any(),
		gomock.Any(),
	).Return(nil, errors.New("failed"))
	_, err := v.CanonicalHeadSlot(context.Background())
	assert.ErrorContains(t, "failed", err)
}

func TestCanonicalHeadSlot_OK(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockBeaconChainClient(ctrl)
	v := validator{
		beaconClient: client,
	}
	client.EXPECT().GetChainHead(
		gomock.Any(),
		gomock.Any(),
	).Return(&ethpb.ChainHead{HeadSlot: 0}, nil)
	headSlot, err := v.CanonicalHeadSlot(context.Background())
	require.NoError(t, err)
	assert.Equal(t, primitives.Slot(0), headSlot, "Mismatch slots")
}

func TestWaitMultipleActivation_LogsActivationEpochOK(t *testing.T) {
	ctx := context.Background()
	hook := logTest.NewGlobal()
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	validatorClient := validatormock.NewMockValidatorClient(ctrl)
	beaconClient := validatormock.NewMockBeaconChainClient(ctrl)
	prysmBeaconClient := validatormock.NewMockPrysmBeaconChainClient(ctrl)

	kp := randKeypair(t)
	v := validator{
		validatorClient:   validatorClient,
		keyManager:        newMockKeymanager(t, kp),
		beaconClient:      beaconClient,
		prysmBeaconClient: prysmBeaconClient,
	}

	resp := generateMockStatusResponse([][]byte{kp.pub[:]})
	resp.Statuses[0].Status.Status = ethpb.ValidatorStatus_ACTIVE
	clientStream := mock2.NewMockBeaconNodeValidator_WaitForActivationClient(ctrl)
	validatorClient.EXPECT().WaitForActivation(
		gomock.Any(),
		&ethpb.ValidatorActivationRequest{
			PublicKeys: [][]byte{kp.pub[:]},
		},
	).Return(clientStream, nil)
	clientStream.EXPECT().Recv().Return(
		resp,
		nil,
	)
	prysmBeaconClient.EXPECT().GetValidatorCount(
		gomock.Any(),
		"head",
		[]validatorType.Status{validatorType.Active},
	).Return([]iface.ValidatorCount{}, nil)
	require.NoError(t, v.WaitForActivation(ctx, nil), "Could not wait for activation")
	require.LogsContain(t, hook, "Validator activated")
}

func TestWaitSync_ContextCanceled(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	n := validatormock.NewMockNodeClient(ctrl)

	v := validator{
		nodeClient: n,
	}

	ctx, cancel := context.WithCancel(context.Background())
	cancel()

	n.EXPECT().GetSyncStatus(
		gomock.Any(),
		gomock.Any(),
	).Return(&ethpb.SyncStatus{Syncing: true}, nil)

	assert.ErrorContains(t, cancelledCtx, v.WaitForSync(ctx))
}

func TestWaitSync_NotSyncing(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	n := validatormock.NewMockNodeClient(ctrl)

	v := validator{
		nodeClient: n,
	}

	n.EXPECT().GetSyncStatus(
		gomock.Any(),
		gomock.Any(),
	).Return(&ethpb.SyncStatus{Syncing: false}, nil)

	require.NoError(t, v.WaitForSync(context.Background()))
}

func TestWaitSync_Syncing(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	n := validatormock.NewMockNodeClient(ctrl)

	v := validator{
		nodeClient: n,
	}

	n.EXPECT().GetSyncStatus(
		gomock.Any(),
		gomock.Any(),
	).Return(&ethpb.SyncStatus{Syncing: true}, nil)

	n.EXPECT().GetSyncStatus(
		gomock.Any(),
		gomock.Any(),
	).Return(&ethpb.SyncStatus{Syncing: false}, nil)

	require.NoError(t, v.WaitForSync(context.Background()))
}

func TestUpdateDuties_DoesNothingWhenNotEpochStart_AlreadyExistingAssignments(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	slot := primitives.Slot(1)
	v := validator{
		validatorClient: client,
		duties: &ethpb.DutiesResponse{
			CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
				{
					Committee:      []primitives.ValidatorIndex{},
					AttesterSlot:   10,
					CommitteeIndex: 20,
				},
			},
		},
	}
	client.EXPECT().GetDuties(
		gomock.Any(),
		gomock.Any(),
	).Times(0)

	assert.NoError(t, v.UpdateDuties(context.Background(), slot), "Could not update assignments")
}

func TestUpdateDuties_ReturnsError(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	v := validator{
		validatorClient: client,
		keyManager:      newMockKeymanager(t, randKeypair(t)),
		duties: &ethpb.DutiesResponse{
			CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
				{
					CommitteeIndex: 1,
				},
			},
		},
	}

	expected := errors.New("bad")

	client.EXPECT().GetDuties(
		gomock.Any(),
		gomock.Any(),
	).Return(nil, expected)

	assert.ErrorContains(t, expected.Error(), v.UpdateDuties(context.Background(), params.BeaconConfig().SlotsPerEpoch))
	assert.Equal(t, (*ethpb.DutiesResponse)(nil), v.duties, "Assignments should have been cleared on failure")
}

func TestUpdateDuties_OK(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	slot := params.BeaconConfig().SlotsPerEpoch
	resp := &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				AttesterSlot:   params.BeaconConfig().SlotsPerEpoch,
				ValidatorIndex: 200,
				CommitteeIndex: 100,
				Committee:      []primitives.ValidatorIndex{0, 1, 2, 3},
				PublicKey:      []byte("testPubKey_1"),
				ProposerSlots:  []primitives.Slot{params.BeaconConfig().SlotsPerEpoch + 1},
			},
		},
	}
	v := validator{
		keyManager:      newMockKeymanager(t, randKeypair(t)),
		validatorClient: client,
	}
	client.EXPECT().GetDuties(
		gomock.Any(),
		gomock.Any(),
	).Return(resp, nil)

	var wg sync.WaitGroup
	wg.Add(1)

	client.EXPECT().SubscribeCommitteeSubnets(
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).DoAndReturn(func(_ context.Context, _ *ethpb.CommitteeSubnetsSubscribeRequest, _ []primitives.ValidatorIndex) (*emptypb.Empty, error) {
		wg.Done()
		return nil, nil
	})

	require.NoError(t, v.UpdateDuties(context.Background(), slot), "Could not update assignments")

	util.WaitTimeout(&wg, 2*time.Second)

	assert.Equal(t, params.BeaconConfig().SlotsPerEpoch+1, v.duties.CurrentEpochDuties[0].ProposerSlots[0], "Unexpected validator assignments")
	assert.Equal(t, params.BeaconConfig().SlotsPerEpoch, v.duties.CurrentEpochDuties[0].AttesterSlot, "Unexpected validator assignments")
	assert.Equal(t, resp.CurrentEpochDuties[0].CommitteeIndex, v.duties.CurrentEpochDuties[0].CommitteeIndex, "Unexpected validator assignments")
	assert.Equal(t, resp.CurrentEpochDuties[0].ValidatorIndex, v.duties.CurrentEpochDuties[0].ValidatorIndex, "Unexpected validator assignments")
}

func TestUpdateDuties_OK_FilterBlacklistedPublicKeys(t *testing.T) {
	hook := logTest.NewGlobal()
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)
	slot := params.BeaconConfig().SlotsPerEpoch

	numValidators := 10
	km := genMockKeymanager(t, numValidators)
	blacklistedPublicKeys := make(map[[fieldparams.BLSPubkeyLength]byte]bool)
	for _, k := range km.keys {
		blacklistedPublicKeys[k] = true
	}
	v := validator{
		keyManager:                     km,
		validatorClient:                client,
		eipImportBlacklistedPublicKeys: blacklistedPublicKeys,
	}

	resp := &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{},
	}
	client.EXPECT().GetDuties(
		gomock.Any(),
		gomock.Any(),
	).Return(resp, nil)

	var wg sync.WaitGroup
	wg.Add(1)
	client.EXPECT().SubscribeCommitteeSubnets(
		gomock.Any(),
		gomock.Any(),
		gomock.Any(),
	).DoAndReturn(func(_ context.Context, _ *ethpb.CommitteeSubnetsSubscribeRequest, _ []primitives.ValidatorIndex) (*emptypb.Empty, error) {
		wg.Done()
		return nil, nil
	})

	require.NoError(t, v.UpdateDuties(context.Background(), slot), "Could not update assignments")

	util.WaitTimeout(&wg, 2*time.Second)

	for range blacklistedPublicKeys {
		assert.LogsContain(t, hook, "Not including slashable public key")
	}
}

func TestUpdateDuties_AllValidatorsExited(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	slot := params.BeaconConfig().SlotsPerEpoch
	resp := &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				AttesterSlot:   params.BeaconConfig().SlotsPerEpoch,
				ValidatorIndex: 200,
				CommitteeIndex: 100,
				Committee:      []primitives.ValidatorIndex{0, 1, 2, 3},
				PublicKey:      []byte("testPubKey_1"),
				ProposerSlots:  []primitives.Slot{params.BeaconConfig().SlotsPerEpoch + 1},
				Status:         ethpb.ValidatorStatus_EXITED,
			},
			{
				AttesterSlot:   params.BeaconConfig().SlotsPerEpoch,
				ValidatorIndex: 201,
				CommitteeIndex: 101,
				Committee:      []primitives.ValidatorIndex{0, 1, 2, 3},
				PublicKey:      []byte("testPubKey_2"),
				ProposerSlots:  []primitives.Slot{params.BeaconConfig().SlotsPerEpoch + 1},
				Status:         ethpb.ValidatorStatus_EXITED,
			},
		},
	}
	v := validator{
		keyManager:      newMockKeymanager(t, randKeypair(t)),
		validatorClient: client,
	}
	client.EXPECT().GetDuties(
		gomock.Any(),
		gomock.Any(),
	).Return(resp, nil)

	err := v.UpdateDuties(context.Background(), slot)
	require.ErrorContains(t, ErrValidatorsAllExited.Error(), err)

}

func TestRolesAt_OK(t *testing.T) {
	v, m, validatorKey, finish := setup(t)
	defer finish()

	v.duties = &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				CommitteeIndex:  1,
				AttesterSlot:    1,
				PublicKey:       validatorKey.PublicKey().Marshal(),
				IsSyncCommittee: true,
			},
		},
		NextEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				CommitteeIndex:  1,
				AttesterSlot:    1,
				PublicKey:       validatorKey.PublicKey().Marshal(),
				IsSyncCommittee: true,
			},
		},
	}

	m.validatorClient.EXPECT().DomainData(
		gomock.Any(), // ctx
		gomock.Any(), // epoch
	).Return(&ethpb.DomainResponse{SignatureDomain: make([]byte, 32)}, nil /*err*/)

	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			PublicKey: validatorKey.PublicKey().Marshal(),
			Slot:      1,
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{}, nil /*err*/)

	roleMap, err := v.RolesAt(context.Background(), 1)
	require.NoError(t, err)

	assert.Equal(t, iface.RoleAttester, roleMap[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())][0])
	assert.Equal(t, iface.RoleAggregator, roleMap[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())][1])
	assert.Equal(t, iface.RoleSyncCommittee, roleMap[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())][2])

	// Test sync committee role at epoch boundary.
	v.duties = &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				CommitteeIndex:  1,
				AttesterSlot:    1,
				PublicKey:       validatorKey.PublicKey().Marshal(),
				IsSyncCommittee: false,
			},
		},
		NextEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				CommitteeIndex:  1,
				AttesterSlot:    1,
				PublicKey:       validatorKey.PublicKey().Marshal(),
				IsSyncCommittee: true,
			},
		},
	}

	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			PublicKey: validatorKey.PublicKey().Marshal(),
			Slot:      31,
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{}, nil /*err*/)

	roleMap, err = v.RolesAt(context.Background(), params.BeaconConfig().SlotsPerEpoch-1)
	require.NoError(t, err)
	assert.Equal(t, iface.RoleSyncCommittee, roleMap[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())][0])
}

func TestRolesAt_DoesNotAssignProposer_Slot0(t *testing.T) {
	v, m, validatorKey, finish := setup(t)
	defer finish()

	v.duties = &ethpb.DutiesResponse{
		CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
			{
				CommitteeIndex: 1,
				AttesterSlot:   0,
				ProposerSlots:  []primitives.Slot{0},
				PublicKey:      validatorKey.PublicKey().Marshal(),
			},
		},
	}

	m.validatorClient.EXPECT().DomainData(
		gomock.Any(), // ctx
		gomock.Any(), // epoch
	).Return(&ethpb.DomainResponse{SignatureDomain: make([]byte, 32)}, nil /*err*/)

	roleMap, err := v.RolesAt(context.Background(), 0)
	require.NoError(t, err)

	assert.Equal(t, iface.RoleAttester, roleMap[bytesutil.ToBytes48(validatorKey.PublicKey().Marshal())][0])
}

func TestCheckAndLogValidatorStatus_OK(t *testing.T) {
	nonexistentIndex := primitives.ValidatorIndex(^uint64(0))
	type statusTest struct {
		name   string
		status *validatorStatus
		log    string
		active bool
	}
	pubKeys := [][]byte{bytesutil.Uint64ToBytesLittleEndian(0)}
	tests := []statusTest{
		{
			name: "UNKNOWN_STATUS, no deposit found yet",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     nonexistentIndex,
				status: &ethpb.ValidatorStatusResponse{
					Status: ethpb.ValidatorStatus_UNKNOWN_STATUS,
				},
			},
			log:    "Waiting for deposit to be observed by beacon node",
			active: false,
		},
		{
			name: "DEPOSITED into state",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     30,
				status: &ethpb.ValidatorStatusResponse{
					Status:                    ethpb.ValidatorStatus_DEPOSITED,
					PositionInActivationQueue: 30,
				},
			},
			log:    "Deposit processed, entering activation queue after finalization\" index=30 positionInActivationQueue=30",
			active: false,
		},
		{
			name: "PENDING",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     50,
				status: &ethpb.ValidatorStatusResponse{
					Status:                    ethpb.ValidatorStatus_PENDING,
					ActivationEpoch:           params.BeaconConfig().FarFutureEpoch,
					PositionInActivationQueue: 6,
				},
			},
			log:    "Waiting to be assigned activation epoch\" expectedWaitingTime=12m48s index=50 positionInActivationQueue=6",
			active: false,
		},
		{
			name: "PENDING",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     89,
				status: &ethpb.ValidatorStatusResponse{
					Status:                    ethpb.ValidatorStatus_PENDING,
					ActivationEpoch:           60,
					PositionInActivationQueue: 5,
				},
			},
			log:    "Waiting for activation\" activationEpoch=60 index=89",
			active: false,
		},
		{
			name: "ACTIVE",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     89,
				status: &ethpb.ValidatorStatusResponse{
					Status: ethpb.ValidatorStatus_ACTIVE,
				},
			},
			active: true,
		},
		{
			name: "EXITING",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				index:     89,
				status: &ethpb.ValidatorStatusResponse{
					Status: ethpb.ValidatorStatus_EXITING,
				},
			},
			active: true,
		},
		{
			name: "EXITED",
			status: &validatorStatus{
				publicKey: pubKeys[0],
				status: &ethpb.ValidatorStatusResponse{
					Status: ethpb.ValidatorStatus_EXITED,
				},
			},
			log:    "Validator exited",
			active: false,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			hook := logTest.NewGlobal()
			ctrl := gomock.NewController(t)
			defer ctrl.Finish()
			client := validatormock.NewMockValidatorClient(ctrl)
			v := validator{
				validatorClient: client,
				duties: &ethpb.DutiesResponse{
					CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
						{
							CommitteeIndex: 1,
						},
					},
				},
			}

			active := v.checkAndLogValidatorStatus([]*validatorStatus{test.status}, 100)
			require.Equal(t, test.active, active)
			if test.log != "" {
				require.LogsContain(t, hook, test.log)
			}
		})
	}
}

func TestService_ReceiveSlots_SetHighest(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	client := validatormock.NewMockValidatorClient(ctrl)

	v := validator{
		validatorClient: client,
		slotFeed:        new(event.Feed),
	}
	stream := mock2.NewMockBeaconNodeValidator_StreamSlotsClient(ctrl)
	ctx, cancel := context.WithCancel(context.Background())
	client.EXPECT().StreamSlots(
		gomock.Any(),
		&ethpb.StreamSlotsRequest{VerifiedOnly: true},
	).Return(stream, nil)
	stream.EXPECT().Context().Return(ctx).AnyTimes()
	stream.EXPECT().Recv().Return(
		&ethpb.StreamSlotsResponse{Slot: 123},
		nil,
	).Do(func() {
		cancel()
	})
	connectionErrorChannel := make(chan error)
	v.ReceiveSlots(ctx, connectionErrorChannel)
	require.Equal(t, primitives.Slot(123), v.highestValidSlot)
}

type doppelGangerRequestMatcher struct {
	req *ethpb.DoppelGangerRequest
}

var _ gomock.Matcher = (*doppelGangerRequestMatcher)(nil)

func (m *doppelGangerRequestMatcher) Matches(x interface{}) bool {
	r, ok := x.(*ethpb.DoppelGangerRequest)
	if !ok {
		panic("Invalid match type")
	}
	return gomock.InAnyOrder(m.req.ValidatorRequests).Matches(r.ValidatorRequests)
}

func (m *doppelGangerRequestMatcher) String() string {
	return fmt.Sprintf("%#v", m.req.ValidatorRequests)
}

func TestValidator_CheckDoppelGanger(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	flgs := features.Get()
	flgs.EnableDoppelGanger = true
	reset := features.InitWithReset(flgs)
	defer reset()
	tests := []struct {
		name            string
		validatorSetter func(t *testing.T) *validator
		err             string
	}{
		{
			name: "no doppelganger",
			validatorSetter: func(t *testing.T) *validator {
				client := validatormock.NewMockValidatorClient(ctrl)
				km := genMockKeymanager(t, 10)
				keys, err := km.FetchValidatingPublicKeys(context.Background())
				assert.NoError(t, err)
				db := dbTest.SetupDB(t, keys)
				req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				resp := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				for _, k := range keys {
					pkey := k
					att := createAttestation(10, 12)
					rt, err := att.Data.HashTreeRoot()
					assert.NoError(t, err)
					assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), pkey, rt, att))
					resp.ValidatorRequests = append(resp.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: pkey[:], Epoch: att.Data.Target.Epoch, SignedRoot: rt[:]})
					req.ValidatorRequests = append(req.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: pkey[:], Epoch: att.Data.Target.Epoch, SignedRoot: rt[:]})
				}
				v := &validator{
					validatorClient: client,
					keyManager:      km,
					db:              db,
				}
				client.EXPECT().CheckDoppelGanger(
					gomock.Any(),                     // ctx
					&doppelGangerRequestMatcher{req}, // request
				).Return(nil, nil /*err*/)

				return v
			},
		},
		{
			name: "multiple doppelganger exists",
			validatorSetter: func(t *testing.T) *validator {
				client := validatormock.NewMockValidatorClient(ctrl)
				km := genMockKeymanager(t, 10)
				keys, err := km.FetchValidatingPublicKeys(context.Background())
				assert.NoError(t, err)
				db := dbTest.SetupDB(t, keys)
				req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				resp := &ethpb.DoppelGangerResponse{Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{}}
				for i, k := range keys {
					pkey := k
					att := createAttestation(10, 12)
					rt, err := att.Data.HashTreeRoot()
					assert.NoError(t, err)
					assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), pkey, rt, att))
					if i%3 == 0 {
						resp.Responses = append(resp.Responses, &ethpb.DoppelGangerResponse_ValidatorResponse{PublicKey: pkey[:], DuplicateExists: true})
					}
					req.ValidatorRequests = append(req.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: pkey[:], Epoch: att.Data.Target.Epoch, SignedRoot: rt[:]})
				}
				v := &validator{
					validatorClient: client,
					keyManager:      km,
					db:              db,
				}
				client.EXPECT().CheckDoppelGanger(
					gomock.Any(),                     // ctx
					&doppelGangerRequestMatcher{req}, // request
				).Return(resp, nil /*err*/)
				return v
			},
			err: "Duplicate instances exists in the network for validator keys",
		},
		{
			name: "single doppelganger exists",
			validatorSetter: func(t *testing.T) *validator {
				client := validatormock.NewMockValidatorClient(ctrl)
				km := genMockKeymanager(t, 10)
				keys, err := km.FetchValidatingPublicKeys(context.Background())
				assert.NoError(t, err)
				db := dbTest.SetupDB(t, keys)
				req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				resp := &ethpb.DoppelGangerResponse{Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{}}
				for i, k := range keys {
					pkey := k
					att := createAttestation(10, 12)
					rt, err := att.Data.HashTreeRoot()
					assert.NoError(t, err)
					assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), pkey, rt, att))
					if i%9 == 0 {
						resp.Responses = append(resp.Responses, &ethpb.DoppelGangerResponse_ValidatorResponse{PublicKey: pkey[:], DuplicateExists: true})
					}
					req.ValidatorRequests = append(req.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: pkey[:], Epoch: att.Data.Target.Epoch, SignedRoot: rt[:]})
				}
				v := &validator{
					validatorClient: client,
					keyManager:      km,
					db:              db,
				}
				client.EXPECT().CheckDoppelGanger(
					gomock.Any(),                     // ctx
					&doppelGangerRequestMatcher{req}, // request
				).Return(resp, nil /*err*/)
				return v
			},
			err: "Duplicate instances exists in the network for validator keys",
		},
		{
			name: "multiple attestations saved",
			validatorSetter: func(t *testing.T) *validator {
				client := validatormock.NewMockValidatorClient(ctrl)
				km := genMockKeymanager(t, 10)
				keys, err := km.FetchValidatingPublicKeys(context.Background())
				assert.NoError(t, err)
				db := dbTest.SetupDB(t, keys)
				req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				resp := &ethpb.DoppelGangerResponse{Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{}}
				attLimit := 5
				for i, k := range keys {
					pkey := k
					for j := 0; j < attLimit; j++ {
						att := createAttestation(10+primitives.Epoch(j), 12+primitives.Epoch(j))
						rt, err := att.Data.HashTreeRoot()
						assert.NoError(t, err)
						assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), pkey, rt, att))
						if j == attLimit-1 {
							req.ValidatorRequests = append(req.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: pkey[:], Epoch: att.Data.Target.Epoch, SignedRoot: rt[:]})
						}
					}
					if i%3 == 0 {
						resp.Responses = append(resp.Responses, &ethpb.DoppelGangerResponse_ValidatorResponse{PublicKey: pkey[:], DuplicateExists: true})
					}
				}
				v := &validator{
					validatorClient: client,
					keyManager:      km,
					db:              db,
				}
				client.EXPECT().CheckDoppelGanger(
					gomock.Any(),                     // ctx
					&doppelGangerRequestMatcher{req}, // request
				).Return(resp, nil /*err*/)
				return v
			},
			err: "Duplicate instances exists in the network for validator keys",
		},
		{
			name: "no history exists",
			validatorSetter: func(t *testing.T) *validator {
				client := validatormock.NewMockValidatorClient(ctrl)
				// Use only 1 key for deterministic order.
				km := genMockKeymanager(t, 1)
				keys, err := km.FetchValidatingPublicKeys(context.Background())
				assert.NoError(t, err)
				db := dbTest.SetupDB(t, keys)
				resp := &ethpb.DoppelGangerResponse{Responses: []*ethpb.DoppelGangerResponse_ValidatorResponse{}}
				req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
				for _, k := range keys {
					resp.Responses = append(resp.Responses, &ethpb.DoppelGangerResponse_ValidatorResponse{PublicKey: k[:], DuplicateExists: false})
					req.ValidatorRequests = append(req.ValidatorRequests, &ethpb.DoppelGangerRequest_ValidatorRequest{PublicKey: k[:], SignedRoot: make([]byte, 32), Epoch: 0})
				}
				v := &validator{
					validatorClient: client,
					keyManager:      km,
					db:              db,
				}
				client.EXPECT().CheckDoppelGanger(
					gomock.Any(), // ctx
					req,          // request
				).Return(resp, nil /*err*/)
				return v
			},
			err: "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			v := tt.validatorSetter(t)
			if err := v.CheckDoppelGanger(context.Background()); tt.err != "" {
				assert.ErrorContains(t, tt.err, err)
			}
		})
	}
}

func TestValidatorAttestationsAreOrdered(t *testing.T) {
	km := genMockKeymanager(t, 10)
	keys, err := km.FetchValidatingPublicKeys(context.Background())
	assert.NoError(t, err)
	db := dbTest.SetupDB(t, keys)

	k := keys[0]
	att := createAttestation(10, 14)
	rt, err := att.Data.HashTreeRoot()
	assert.NoError(t, err)
	assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), k, rt, att))

	att = createAttestation(6, 8)
	rt, err = att.Data.HashTreeRoot()
	assert.NoError(t, err)
	assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), k, rt, att))

	att = createAttestation(10, 12)
	rt, err = att.Data.HashTreeRoot()
	assert.NoError(t, err)
	assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), k, rt, att))

	att = createAttestation(2, 3)
	rt, err = att.Data.HashTreeRoot()
	assert.NoError(t, err)
	assert.NoError(t, db.SaveAttestationForPubKey(context.Background(), k, rt, att))

	histories, err := db.AttestationHistoryForPubKey(context.Background(), k)
	assert.NoError(t, err)
	r := retrieveLatestRecord(histories)
	assert.Equal(t, r.Target, primitives.Epoch(14))
}

func createAttestation(source, target primitives.Epoch) *ethpb.IndexedAttestation {
	return &ethpb.IndexedAttestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{
				Epoch: source,
				Root:  make([]byte, 32),
			},
			Target: &ethpb.Checkpoint{
				Epoch: target,
				Root:  make([]byte, 32),
			},
			BeaconBlockRoot: make([]byte, 32),
		},
		Signature: make([]byte, fieldparams.BLSSignatureLength),
	}
}

func TestIsSyncCommitteeAggregator_OK(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	v, m, validatorKey, finish := setup(t)
	defer finish()

	slot := primitives.Slot(1)
	pubKey := validatorKey.PublicKey().Marshal()

	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			PublicKey: validatorKey.PublicKey().Marshal(),
			Slot:      1,
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{}, nil /*err*/)

	aggregator, err := v.isSyncCommitteeAggregator(context.Background(), slot, bytesutil.ToBytes48(pubKey))
	require.NoError(t, err)
	require.Equal(t, false, aggregator)

	c := params.BeaconConfig().Copy()
	c.TargetAggregatorsPerSyncSubcommittee = math.MaxUint64
	params.OverrideBeaconConfig(c)

	m.validatorClient.EXPECT().DomainData(
		gomock.Any(), // ctx
		gomock.Any(), // epoch
	).Return(&ethpb.DomainResponse{SignatureDomain: make([]byte, 32)}, nil /*err*/)

	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			PublicKey: validatorKey.PublicKey().Marshal(),
			Slot:      1,
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{0}}, nil /*err*/)

	aggregator, err = v.isSyncCommitteeAggregator(context.Background(), slot, bytesutil.ToBytes48(pubKey))
	require.NoError(t, err)
	require.Equal(t, true, aggregator)
}

func TestValidator_WaitForKeymanagerInitialization_web3Signer(t *testing.T) {
	ctx := context.Background()
	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	root := make([]byte, 32)
	copy(root[2:], "a")
	err := db.SaveGenesisValidatorsRoot(ctx, root)
	require.NoError(t, err)
	w := wallet.NewWalletForWeb3Signer()
	decodedKey, err := hexutil.Decode("0xa2b5aaad9c6efefe7bb9b1243a043404f3362937cfb6b31833929833173f476630ea2cfeb0d9ddf15f97ca8685948820")
	require.NoError(t, err)
	keys := [][48]byte{
		bytesutil.ToBytes48(decodedKey),
	}
	v := validator{
		db:     db,
		useWeb: false,
		wallet: w,
		Web3SignerConfig: &remoteweb3signer.SetupConfig{
			BaseEndpoint:       "http://localhost:8545",
			ProvidedPublicKeys: keys,
		},
	}
	err = v.WaitForKeymanagerInitialization(context.Background())
	require.NoError(t, err)
	km, err := v.Keymanager()
	require.NoError(t, err)
	require.NotNil(t, km)
}

func TestValidator_WaitForKeymanagerInitialization_Web(t *testing.T) {
	ctx := context.Background()
	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	root := make([]byte, 32)
	copy(root[2:], "a")
	err := db.SaveGenesisValidatorsRoot(ctx, root)
	require.NoError(t, err)
	walletChan := make(chan *wallet.Wallet, 1)
	v := validator{
		db:                       db,
		useWeb:                   true,
		walletInitializedFeed:    &event.Feed{},
		walletInitializedChannel: walletChan,
	}
	wait := make(chan struct{})
	go func() {
		defer close(wait)
		err = v.WaitForKeymanagerInitialization(ctx)
		require.NoError(t, err)
		km, err := v.Keymanager()
		require.NoError(t, err)
		require.NotNil(t, km)
	}()

	walletChan <- wallet.New(&wallet.Config{
		KeymanagerKind: keymanager.Local,
	})
	<-wait
}

func TestValidator_WaitForKeymanagerInitialization_Interop(t *testing.T) {
	ctx := context.Background()
	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	root := make([]byte, 32)
	copy(root[2:], "a")
	err := db.SaveGenesisValidatorsRoot(ctx, root)
	require.NoError(t, err)
	v := validator{
		db:     db,
		useWeb: false,
		interopKeysConfig: &local.InteropKeymanagerConfig{
			NumValidatorKeys: 2,
			Offset:           1,
		},
	}
	err = v.WaitForKeymanagerInitialization(ctx)
	require.NoError(t, err)
	km, err := v.Keymanager()
	require.NoError(t, err)
	require.NotNil(t, km)
}

func TestValidator_PushProposerSettings(t *testing.T) {
	ctrl := gomock.NewController(t)
	ctx := context.Background()
	db := dbTest.SetupDB(t, [][fieldparams.BLSPubkeyLength]byte{})
	client := validatormock.NewMockValidatorClient(ctrl)
	nodeClient := validatormock.NewMockNodeClient(ctrl)
	defaultFeeHex := "0x046Fb65722E7b2455043BFEBf6177F1D2e9738D9"
	byteValueAddress, err := hexutil.Decode("0x046Fb65722E7b2455043BFEBf6177F1D2e9738D9")
	require.NoError(t, err)

	type ExpectedValidatorRegistration struct {
		FeeRecipient []byte
		GasLimit     uint64
		Timestamp    uint64
		Pubkey       []byte
	}

	tests := []struct {
		name                 string
		validatorSetter      func(t *testing.T) *validator
		feeRecipientMap      map[primitives.ValidatorIndex]string
		mockExpectedRequests []ExpectedValidatorRegistration
		err                  string
		logMessages          []string
		doesntContainLogs    bool
	}{
		{
			name: "Happy Path proposer config not nil",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 2,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				v.pubkeyToValidatorIndex[keys[1]] = primitives.ValidatorIndex(2)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}, {Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:], keys[1][:]},
					}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9").Bytes(), ValidatorIndex: 1},
						{FeeRecipient: common.HexToAddress(defaultFeeHex).Bytes(), ValidatorIndex: 2},
					},
				}).Return(nil, nil)
				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9"),
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 40000000,
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: 35000000,
						},
					},
				})
				require.NoError(t, err)
				client.EXPECT().SubmitValidatorRegistrations(
					gomock.Any(),
					gomock.Any(),
				).Return(&empty.Empty{}, nil)
				return &v
			},
			feeRecipientMap: map[primitives.ValidatorIndex]string{
				1: "0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9",
				2: defaultFeeHex,
			},
			mockExpectedRequests: []ExpectedValidatorRegistration{

				{
					FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9").Bytes(),
					GasLimit:     40000000,
				},
				{
					FeeRecipient: byteValueAddress,
					GasLimit:     35000000,
				},
			},
		},
		{
			name: " Happy Path default doesn't send validator registration",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 2,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				v.pubkeyToValidatorIndex[keys[1]] = primitives.ValidatorIndex(2)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}, {Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:], keys[1][:]},
					}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9").Bytes(), ValidatorIndex: 1},
						{FeeRecipient: common.HexToAddress(defaultFeeHex).Bytes(), ValidatorIndex: 2},
					},
				}).Return(nil, nil)
				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9"),
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 40000000,
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  false,
							GasLimit: 35000000,
						},
					},
				})
				require.NoError(t, err)
				client.EXPECT().SubmitValidatorRegistrations(
					gomock.Any(),
					gomock.Any(),
				).Return(&empty.Empty{}, nil)
				return &v
			},
			feeRecipientMap: map[primitives.ValidatorIndex]string{
				1: "0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9",
				2: defaultFeeHex,
			},
			mockExpectedRequests: []ExpectedValidatorRegistration{

				{
					FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9").Bytes(),
					GasLimit:     uint64(40000000),
				},
			},
		},
		{
			name: " Happy Path default doesn't send any validator registrations",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 2,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				v.pubkeyToValidatorIndex[keys[1]] = primitives.ValidatorIndex(2)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}, {Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:], keys[1][:]},
					}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9").Bytes(), ValidatorIndex: 1},
						{FeeRecipient: common.HexToAddress(defaultFeeHex).Bytes(), ValidatorIndex: 2},
					},
				}).Return(nil, nil)
				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.HexToAddress("0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9"),
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
					},
				})
				require.NoError(t, err)
				return &v
			},
			feeRecipientMap: map[primitives.ValidatorIndex]string{
				1: "0x055Fb65722E7b2455043BFEBf6177F1D2e9738D9",
				2: defaultFeeHex,
			},
			logMessages:       []string{"will not be included in builder validator registration"},
			doesntContainLogs: true,
		},
		{
			name: " Happy Path",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 1,
						Offset:           1,
					},
					genesisTime: 0,
				}
				// set bellatrix as current epoch
				params.BeaconConfig().BellatrixForkEpoch = 0
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: nil,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: validatorType.Uint64(params.BeaconConfig().DefaultBuilderGasLimit),
						},
					},
				})
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:]},
					}, nil)

				client.EXPECT().SubmitValidatorRegistrations(
					gomock.Any(),
					gomock.Any(),
				).Return(&empty.Empty{}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress(defaultFeeHex).Bytes(), ValidatorIndex: 1},
					},
				}).Return(nil, nil)
				return &v
			},
			feeRecipientMap: map[primitives.ValidatorIndex]string{
				1: defaultFeeHex,
			},
			mockExpectedRequests: []ExpectedValidatorRegistration{
				{
					FeeRecipient: byteValueAddress,
					GasLimit:     params.BeaconConfig().DefaultBuilderGasLimit,
				},
			},
		},
		{
			name: " Happy Path validator index not found in cache",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 1,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: nil,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: 40000000,
						},
					},
				})
				require.NoError(t, err)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:]},
					}, nil)
				client.EXPECT().SubmitValidatorRegistrations(
					gomock.Any(),
					gomock.Any(),
				).Return(&empty.Empty{}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress(defaultFeeHex).Bytes(), ValidatorIndex: 1},
					},
				}).Return(nil, nil)
				return &v
			},
			feeRecipientMap: map[primitives.ValidatorIndex]string{
				1: defaultFeeHex,
			},
			mockExpectedRequests: []ExpectedValidatorRegistration{
				{
					FeeRecipient: byteValueAddress,
					GasLimit:     params.BeaconConfig().DefaultBuilderGasLimit,
				},
			},
		},
		{
			name: " proposer config not nil but fee recipient empty",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 1,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:]},
					}, nil)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress("0x0").Bytes(), ValidatorIndex: 1},
					},
				}).Return(nil, nil)
				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.Address{},
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
					},
				})
				require.NoError(t, err)
				return &v
			},
		},
		{
			name: "Validator index not found with proposeconfig",
			validatorSetter: func(t *testing.T) *validator {

				v := validator{
					validatorClient:              client,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 1,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				client.EXPECT().ValidatorIndex(
					gomock.Any(), // ctx
					&ethpb.ValidatorIndexRequest{PublicKey: keys[0][:]},
				).Return(nil, errors.New("could not find validator index for public key"))
				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.HexToAddress("0x046Fb65722E7b2455043BFEBf6177F1D2e9738D9"),
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
					},
				})
				require.NoError(t, err)
				return &v
			},
		},
		{
			name: "register validator batch failed",
			validatorSetter: func(t *testing.T) *validator {
				v := validator{
					validatorClient:              client,
					nodeClient:                   nodeClient,
					db:                           db,
					pubkeyToValidatorIndex:       make(map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex),
					signedValidatorRegistrations: make(map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1),
					useWeb:                       false,
					interopKeysConfig: &local.InteropKeymanagerConfig{
						NumValidatorKeys: 1,
						Offset:           1,
					},
				}
				err := v.WaitForKeymanagerInitialization(ctx)
				require.NoError(t, err)
				config := make(map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption)
				km, err := v.Keymanager()
				require.NoError(t, err)
				keys, err := km.FetchValidatingPublicKeys(ctx)
				require.NoError(t, err)
				v.pubkeyToValidatorIndex[keys[0]] = primitives.ValidatorIndex(1)
				client.EXPECT().MultipleValidatorStatus(
					gomock.Any(),
					gomock.Any()).Return(
					&ethpb.MultipleValidatorStatusResponse{
						Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}},
						PublicKeys: [][]byte{keys[0][:]},
					}, nil)

				config[keys[0]] = &validatorserviceconfig.ProposerOption{
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: common.Address{},
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 40000000,
					},
				}
				err = v.SetProposerSettings(context.Background(), &validatorserviceconfig.ProposerSettings{
					ProposeConfig: config,
					DefaultConfig: &validatorserviceconfig.ProposerOption{
						FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
							FeeRecipient: common.HexToAddress(defaultFeeHex),
						},
						BuilderConfig: &validatorserviceconfig.BuilderConfig{
							Enabled:  true,
							GasLimit: 40000000,
						},
					},
				})
				require.NoError(t, err)
				client.EXPECT().PrepareBeaconProposer(gomock.Any(), &ethpb.PrepareBeaconProposerRequest{
					Recipients: []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
						{FeeRecipient: common.HexToAddress("0x0").Bytes(), ValidatorIndex: 1},
					},
				}).Return(nil, nil)
				client.EXPECT().SubmitValidatorRegistrations(
					gomock.Any(),
					gomock.Any(),
				).Return(&empty.Empty{}, errors.New("request failed"))
				return &v
			},
			err: "could not submit signed registrations to beacon node",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			hook := logTest.NewGlobal()
			v := tt.validatorSetter(t)
			km, err := v.Keymanager()
			require.NoError(t, err)
			pubkeys, err := km.FetchValidatingPublicKeys(ctx)
			require.NoError(t, err)
			if tt.feeRecipientMap != nil {
				feeRecipients, err := v.buildPrepProposerReqs(ctx, pubkeys)
				require.NoError(t, err)
				signedRegisterValidatorRequests := v.buildSignedRegReqs(ctx, pubkeys, km.Sign)
				for _, recipient := range feeRecipients {
					require.Equal(t, strings.ToLower(tt.feeRecipientMap[recipient.ValidatorIndex]), strings.ToLower(hexutil.Encode(recipient.FeeRecipient)))
				}
				require.Equal(t, len(tt.feeRecipientMap), len(feeRecipients))
				for i, request := range tt.mockExpectedRequests {
					require.Equal(t, tt.mockExpectedRequests[i].GasLimit, request.GasLimit)
					require.Equal(t, hexutil.Encode(tt.mockExpectedRequests[i].FeeRecipient), hexutil.Encode(request.FeeRecipient))
				}
				// check if Pubkeys are always unique
				var unique = make(map[string]bool)
				for _, request := range signedRegisterValidatorRequests {
					require.Equal(t, unique[common.BytesToAddress(request.Message.Pubkey).Hex()], false)
					unique[common.BytesToAddress(request.Message.Pubkey).Hex()] = true
				}
				require.Equal(t, len(tt.mockExpectedRequests), len(signedRegisterValidatorRequests))
				require.Equal(t, len(signedRegisterValidatorRequests), len(v.signedValidatorRegistrations))
			}
			deadline := time.Now().Add(time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Second)
			if err := v.PushProposerSettings(ctx, km, 0, deadline); tt.err != "" {
				assert.ErrorContains(t, tt.err, err)
			}
			if len(tt.logMessages) > 0 {
				for _, message := range tt.logMessages {
					if tt.doesntContainLogs {
						assert.LogsDoNotContain(t, hook, message)
					} else {
						assert.LogsContain(t, hook, message)
					}
				}

			}
		})
	}
}

func getPubkeyFromString(t *testing.T, stringPubkey string) [fieldparams.BLSPubkeyLength]byte {
	pubkeyTemp, err := hexutil.Decode(stringPubkey)
	require.NoError(t, err)

	var pubkey [fieldparams.BLSPubkeyLength]byte
	copy(pubkey[:], pubkeyTemp)

	return pubkey
}

func getFeeRecipientFromString(t *testing.T, stringFeeRecipient string) common.Address {
	feeRecipientTemp, err := hexutil.Decode(stringFeeRecipient)
	require.NoError(t, err)

	var feeRecipient common.Address
	copy(feeRecipient[:], feeRecipientTemp)

	return feeRecipient
}

func TestValidator_buildPrepProposerReqs_WithoutDefaultConfig(t *testing.T) {
	// pubkey1 => feeRecipient1 (already in `v.validatorIndex`)
	// pubkey2 => feeRecipient2 (NOT in `v.validatorIndex`, index found by beacon node)
	// pubkey3 => feeRecipient3 (NOT in `v.validatorIndex`, index NOT found by beacon node)
	// pubkey4 => Nothing (already in `v.validatorIndex`)

	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")
	pubkey2 := getPubkeyFromString(t, "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222")
	pubkey3 := getPubkeyFromString(t, "0x333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333")
	pubkey4 := getPubkeyFromString(t, "0x444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444")

	// Fee recipients
	feeRecipient1 := getFeeRecipientFromString(t, "0x1111111111111111111111111111111111111111")
	feeRecipient2 := getFeeRecipientFromString(t, "0x0000000000000000000000000000000000000000")
	feeRecipient3 := getFeeRecipientFromString(t, "0x3333333333333333333333333333333333333333")
	feeRecipient4 := common.Address{}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)
	client.EXPECT().ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubkey2[:],
		},
	).Return(&ethpb.ValidatorIndexResponse{
		Index: 2,
	}, nil)

	client.EXPECT().ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubkey3[:],
		},
	).Return(nil, status.Error(codes.NotFound, "NOT_FOUND"))

	client.EXPECT().MultipleValidatorStatus(
		gomock.Any(),
		gomock.Any()).Return(
		&ethpb.MultipleValidatorStatusResponse{
			Statuses:   []*ethpb.ValidatorStatusResponse{{Status: ethpb.ValidatorStatus_ACTIVE}, {Status: ethpb.ValidatorStatus_ACTIVE}, {Status: ethpb.ValidatorStatus_ACTIVE}},
			PublicKeys: [][]byte{pubkey1[:], pubkey2[:], pubkey4[:]},
		}, nil)
	v := validator{
		validatorClient: client,
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: nil,
			ProposeConfig: map[[48]byte]*validatorserviceconfig.ProposerOption{
				pubkey1: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient1,
					},
				},
				pubkey2: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient2,
					},
				},
				pubkey3: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient3,
					},
				},
				pubkey4: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient4,
					},
				},
			},
		},
		pubkeyToValidatorIndex: map[[48]byte]primitives.ValidatorIndex{
			pubkey1: 1,
			pubkey4: 4,
		},
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{pubkey1, pubkey2, pubkey3, pubkey4}

	expected := []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
		{
			ValidatorIndex: 1,
			FeeRecipient:   feeRecipient1[:],
		},
		{
			ValidatorIndex: 2,
			FeeRecipient:   feeRecipient2[:],
		},
		{
			ValidatorIndex: 4,
			FeeRecipient:   feeRecipient4[:],
		},
	}
	filteredKeys, err := v.filterAndCacheActiveKeys(ctx, pubkeys, 0)
	require.NoError(t, err)
	actual, err := v.buildPrepProposerReqs(ctx, filteredKeys)
	require.NoError(t, err)
	assert.DeepEqual(t, expected, actual)
}

func TestValidator_buildPrepProposerReqs_WithDefaultConfig(t *testing.T) {
	// pubkey1 => feeRecipient1 - Status: active  (already in `v.validatorIndex`)
	// pubkey2 => feeRecipient2 - Status: active  (NOT in `v.validatorIndex`, index found by beacon node)
	// pubkey3 => feeRecipient3 - Status: active  (NOT in `v.validatorIndex`, index NOT found by beacon node)
	// pubkey4 => Nothing       - Status: active  (already in `v.validatorIndex`)
	// pubkey5 => Nothing       - Status: unknown (already in `v.validatorIndex`)
	// pubkey6 => Nothing       - Status: pending (already in `v.validatorIndex`) - ActivationEpoch: 35 (current slot: 641 - current epoch: 20)
	// pubkey7 => Nothing       - Status: pending (already in `v.validatorIndex`) - ActivationEpoch: 20 (current slot: 641 - current epoch: 20)
	// pubkey8 => feeRecipient8 - Status: exiting (already in `v.validatorIndex`)

	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")
	pubkey2 := getPubkeyFromString(t, "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222")
	pubkey3 := getPubkeyFromString(t, "0x333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333")
	pubkey4 := getPubkeyFromString(t, "0x444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444444")
	pubkey5 := getPubkeyFromString(t, "0x555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555")
	pubkey6 := getPubkeyFromString(t, "0x666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666")
	pubkey7 := getPubkeyFromString(t, "0x777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777777")
	pubkey8 := getPubkeyFromString(t, "0x888888888888888888888888888888888888888888888888888888888888888888888888888888888888888888888888")

	// Fee recipients
	feeRecipient1 := getFeeRecipientFromString(t, "0x1111111111111111111111111111111111111111")
	feeRecipient2 := getFeeRecipientFromString(t, "0x0000000000000000000000000000000000000000")
	feeRecipient3 := getFeeRecipientFromString(t, "0x3333333333333333333333333333333333333333")
	feeRecipient8 := getFeeRecipientFromString(t, "0x8888888888888888888888888888888888888888")

	defaultFeeRecipient := getFeeRecipientFromString(t, "0xdddddddddddddddddddddddddddddddddddddddd")

	pubkeyToStatus := map[[fieldparams.BLSPubkeyLength]byte]ethpb.ValidatorStatus{
		pubkey1: ethpb.ValidatorStatus_ACTIVE,
		pubkey2: ethpb.ValidatorStatus_ACTIVE,
		pubkey3: ethpb.ValidatorStatus_ACTIVE,
		pubkey4: ethpb.ValidatorStatus_ACTIVE,
		pubkey5: ethpb.ValidatorStatus_UNKNOWN_STATUS,
		pubkey6: ethpb.ValidatorStatus_PENDING,
		pubkey7: ethpb.ValidatorStatus_PENDING,
		pubkey8: ethpb.ValidatorStatus_EXITING,
	}

	pubkeyToActivationEpoch := map[[fieldparams.BLSPubkeyLength]byte]primitives.Epoch{
		pubkey1: 0,
		pubkey2: 0,
		pubkey3: 0,
		pubkey4: 0,
		pubkey5: 0,
		pubkey6: 35,
		pubkey7: 20,
		pubkey8: 0,
	}

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)

	client.EXPECT().ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubkey2[:],
		},
	).Return(&ethpb.ValidatorIndexResponse{
		Index: 2,
	}, nil)

	client.EXPECT().ValidatorIndex(
		ctx,
		&ethpb.ValidatorIndexRequest{
			PublicKey: pubkey3[:],
		},
	).Return(nil, status.Error(codes.NotFound, "NOT_FOUND"))

	client.EXPECT().MultipleValidatorStatus(
		gomock.Any(),
		gomock.Any()).DoAndReturn(func(ctx context.Context, val *ethpb.MultipleValidatorStatusRequest) (*ethpb.MultipleValidatorStatusResponse, error) {
		resp := &ethpb.MultipleValidatorStatusResponse{}

		for _, k := range val.PublicKeys {
			resp.PublicKeys = append(resp.PublicKeys, bytesutil.SafeCopyBytes(k))
			resp.Statuses = append(resp.Statuses, &ethpb.ValidatorStatusResponse{
				Status:          pubkeyToStatus[bytesutil.ToBytes48(k)],
				ActivationEpoch: pubkeyToActivationEpoch[bytesutil.ToBytes48(k)],
			})
		}
		return resp, nil
	})

	v := validator{
		validatorClient: client,
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: &validatorserviceconfig.ProposerOption{
				FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
					FeeRecipient: defaultFeeRecipient,
				},
			},
			ProposeConfig: map[[fieldparams.BLSPubkeyLength]byte]*validatorserviceconfig.ProposerOption{
				pubkey1: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient1,
					},
				},
				pubkey2: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient2,
					},
				},
				pubkey3: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient3,
					},
				},
				pubkey8: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient8,
					},
				},
			},
		},
		pubkeyToValidatorIndex: map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex{
			pubkey1: 1,
			pubkey4: 4,
			pubkey5: 5,
			pubkey6: 6,
			pubkey7: 7,
			pubkey8: 8,
		},
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{
		pubkey1,
		pubkey2,
		pubkey3,
		pubkey4,
		pubkey5,
		pubkey6,
		pubkey7,
		pubkey8,
	}

	expected := []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
		{
			ValidatorIndex: 1,
			FeeRecipient:   feeRecipient1[:],
		},
		{
			ValidatorIndex: 2,
			FeeRecipient:   feeRecipient2[:],
		},
		{
			ValidatorIndex: 4,
			FeeRecipient:   defaultFeeRecipient[:],
		},
		{
			ValidatorIndex: 7,
			FeeRecipient:   defaultFeeRecipient[:],
		},
		{
			ValidatorIndex: 8,
			FeeRecipient:   feeRecipient8[:],
		},
	}
	filteredKeys, err := v.filterAndCacheActiveKeys(ctx, pubkeys, 641)
	require.NoError(t, err)
	actual, err := v.buildPrepProposerReqs(ctx, filteredKeys)
	require.NoError(t, err)
	assert.DeepEqual(t, expected, actual)
}

func TestValidator_buildSignedRegReqs_DefaultConfigDisabled(t *testing.T) {
	// pubkey1 => feeRecipient1, builder enabled
	// pubkey2 => feeRecipient2, builder disabled
	// pubkey3 => Nothing, builder enabled

	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")
	pubkey2 := getPubkeyFromString(t, "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222")
	pubkey3 := getPubkeyFromString(t, "0x333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333")

	// Fee recipients
	feeRecipient1 := getFeeRecipientFromString(t, "0x0000000000000000000000000000000000000000")
	feeRecipient2 := getFeeRecipientFromString(t, "0x2222222222222222222222222222222222222222")

	defaultFeeRecipient := getFeeRecipientFromString(t, "0xdddddddddddddddddddddddddddddddddddddddd")

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)

	signature := blsmock.NewSignature(ctrl)
	signature.EXPECT().Marshal().Return([]byte{})

	v := validator{
		signedValidatorRegistrations: map[[48]byte]*ethpb.SignedValidatorRegistrationV1{},
		validatorClient:              client,
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: &validatorserviceconfig.ProposerOption{
				FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
					FeeRecipient: defaultFeeRecipient,
				},
				BuilderConfig: &validatorserviceconfig.BuilderConfig{
					Enabled:  false,
					GasLimit: 9999,
				},
			},
			ProposeConfig: map[[48]byte]*validatorserviceconfig.ProposerOption{
				pubkey1: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient1,
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 1111,
					},
				},
				pubkey2: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient2,
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  false,
						GasLimit: 2222,
					},
				},
				pubkey3: {
					FeeRecipientConfig: nil,
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 3333,
					},
				},
			},
		},
		pubkeyToValidatorIndex: make(map[[48]byte]primitives.ValidatorIndex),
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{pubkey1, pubkey2, pubkey3}

	var signer = func(_ context.Context, _ *validatorpb.SignRequest) (bls.Signature, error) {
		return signature, nil
	}
	v.pubkeyToValidatorIndex[pubkey1] = primitives.ValidatorIndex(1)
	v.pubkeyToValidatorIndex[pubkey2] = primitives.ValidatorIndex(2)
	v.pubkeyToValidatorIndex[pubkey3] = primitives.ValidatorIndex(3)
	actual := v.buildSignedRegReqs(ctx, pubkeys, signer)

	assert.Equal(t, 1, len(actual))
	assert.DeepEqual(t, feeRecipient1[:], actual[0].Message.FeeRecipient)
	assert.Equal(t, uint64(1111), actual[0].Message.GasLimit)
	assert.DeepEqual(t, pubkey1[:], actual[0].Message.Pubkey)
}

func TestValidator_buildSignedRegReqs_DefaultConfigEnabled(t *testing.T) {
	// pubkey1 => feeRecipient1, builder enabled
	// pubkey2 => feeRecipient2, builder disabled
	// pubkey3 => Nothing, builder enabled

	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")
	pubkey2 := getPubkeyFromString(t, "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222")
	pubkey3 := getPubkeyFromString(t, "0x333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333")

	// Fee recipients
	feeRecipient1 := getFeeRecipientFromString(t, "0x0000000000000000000000000000000000000000")
	feeRecipient2 := getFeeRecipientFromString(t, "0x2222222222222222222222222222222222222222")

	defaultFeeRecipient := getFeeRecipientFromString(t, "0xdddddddddddddddddddddddddddddddddddddddd")

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)

	signature := blsmock.NewSignature(ctrl)
	signature.EXPECT().Marshal().Return([]byte{}).Times(2)

	v := validator{
		signedValidatorRegistrations: map[[48]byte]*ethpb.SignedValidatorRegistrationV1{},
		validatorClient:              client,
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: &validatorserviceconfig.ProposerOption{
				FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
					FeeRecipient: defaultFeeRecipient,
				},
				BuilderConfig: &validatorserviceconfig.BuilderConfig{
					Enabled:  true,
					GasLimit: 9999,
				},
			},
			ProposeConfig: map[[48]byte]*validatorserviceconfig.ProposerOption{
				pubkey1: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient1,
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 1111,
					},
				},
				pubkey2: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient2,
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  false,
						GasLimit: 2222,
					},
				},
				pubkey3: {
					FeeRecipientConfig: nil,
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 3333,
					},
				},
			},
		},
		pubkeyToValidatorIndex: make(map[[48]byte]primitives.ValidatorIndex),
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{pubkey1, pubkey2, pubkey3}

	var signer = func(_ context.Context, _ *validatorpb.SignRequest) (bls.Signature, error) {
		return signature, nil
	}
	v.pubkeyToValidatorIndex[pubkey1] = primitives.ValidatorIndex(1)
	v.pubkeyToValidatorIndex[pubkey2] = primitives.ValidatorIndex(2)
	v.pubkeyToValidatorIndex[pubkey3] = primitives.ValidatorIndex(3)
	actual := v.buildSignedRegReqs(ctx, pubkeys, signer)

	assert.Equal(t, 2, len(actual))

	assert.DeepEqual(t, feeRecipient1[:], actual[0].Message.FeeRecipient)
	assert.Equal(t, uint64(1111), actual[0].Message.GasLimit)
	assert.DeepEqual(t, pubkey1[:], actual[0].Message.Pubkey)

	assert.DeepEqual(t, defaultFeeRecipient[:], actual[1].Message.FeeRecipient)
	assert.Equal(t, uint64(9999), actual[1].Message.GasLimit)
	assert.DeepEqual(t, pubkey3[:], actual[1].Message.Pubkey)
}

func TestValidator_buildSignedRegReqs_SignerOnError(t *testing.T) {
	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")

	// Fee recipients
	defaultFeeRecipient := getFeeRecipientFromString(t, "0xdddddddddddddddddddddddddddddddddddddddd")

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)

	v := validator{
		signedValidatorRegistrations: map[[48]byte]*ethpb.SignedValidatorRegistrationV1{},
		validatorClient:              client,
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: &validatorserviceconfig.ProposerOption{
				FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
					FeeRecipient: defaultFeeRecipient,
				},
				BuilderConfig: &validatorserviceconfig.BuilderConfig{
					Enabled:  true,
					GasLimit: 9999,
				},
			},
		},
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{pubkey1}

	var signer = func(_ context.Context, _ *validatorpb.SignRequest) (bls.Signature, error) {
		return nil, errors.New("custom error")
	}

	actual := v.buildSignedRegReqs(ctx, pubkeys, signer)
	assert.Equal(t, 0, len(actual))
}

func TestValidator_buildSignedRegReqs_TimestampBeforeGenesis(t *testing.T) {
	// Public keys
	pubkey1 := getPubkeyFromString(t, "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111")

	// Fee recipients
	feeRecipient1 := getFeeRecipientFromString(t, "0x0000000000000000000000000000000000000000")

	defaultFeeRecipient := getFeeRecipientFromString(t, "0xdddddddddddddddddddddddddddddddddddddddd")

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	ctx := context.Background()
	client := validatormock.NewMockValidatorClient(ctrl)

	signature := blsmock.NewSignature(ctrl)

	v := validator{
		signedValidatorRegistrations: map[[48]byte]*ethpb.SignedValidatorRegistrationV1{},
		validatorClient:              client,
		genesisTime:                  uint64(time.Now().UTC().Unix() + 1000),
		proposerSettings: &validatorserviceconfig.ProposerSettings{
			DefaultConfig: &validatorserviceconfig.ProposerOption{
				FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
					FeeRecipient: defaultFeeRecipient,
				},
				BuilderConfig: &validatorserviceconfig.BuilderConfig{
					Enabled:  true,
					GasLimit: 9999,
				},
			},
			ProposeConfig: map[[48]byte]*validatorserviceconfig.ProposerOption{
				pubkey1: {
					FeeRecipientConfig: &validatorserviceconfig.FeeRecipientConfig{
						FeeRecipient: feeRecipient1,
					},
					BuilderConfig: &validatorserviceconfig.BuilderConfig{
						Enabled:  true,
						GasLimit: 1111,
					},
				},
			},
		},
		pubkeyToValidatorIndex: make(map[[48]byte]primitives.ValidatorIndex),
	}

	pubkeys := [][fieldparams.BLSPubkeyLength]byte{pubkey1}

	var signer = func(_ context.Context, _ *validatorpb.SignRequest) (bls.Signature, error) {
		return signature, nil
	}
	v.pubkeyToValidatorIndex[pubkey1] = primitives.ValidatorIndex(1)
	actual := v.buildSignedRegReqs(ctx, pubkeys, signer)
	assert.Equal(t, 0, len(actual))
}
