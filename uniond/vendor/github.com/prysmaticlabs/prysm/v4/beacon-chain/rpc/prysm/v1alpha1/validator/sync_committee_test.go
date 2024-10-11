package validator

import (
	"context"
	"testing"
	"time"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/emptypb"
)

func TestGetSyncMessageBlockRoot_OK(t *testing.T) {
	r := []byte{'a'}
	server := &Server{
		HeadFetcher: &mock.ChainService{Root: r},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}
	res, err := server.GetSyncMessageBlockRoot(context.Background(), &emptypb.Empty{})
	require.NoError(t, err)
	require.DeepEqual(t, r, res.Root)
}

func TestGetSyncMessageBlockRoot_Optimistic(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig().Copy()
	cfg.BellatrixForkEpoch = 0
	params.OverrideBeaconConfig(cfg)

	server := &Server{
		HeadFetcher:           &mock.ChainService{},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now()},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: true},
	}
	_, err := server.GetSyncMessageBlockRoot(context.Background(), &emptypb.Empty{})
	s, ok := status.FromError(err)
	require.Equal(t, true, ok)
	require.DeepEqual(t, codes.Unavailable, s.Code())
	require.ErrorContains(t, errOptimisticMode.Error(), err)

	server = &Server{
		HeadFetcher:           &mock.ChainService{},
		TimeFetcher:           &mock.ChainService{Genesis: time.Now()},
		OptimisticModeFetcher: &mock.ChainService{Optimistic: false},
	}
	_, err = server.GetSyncMessageBlockRoot(context.Background(), &emptypb.Empty{})
	require.NoError(t, err)
}

func TestSubmitSyncMessage_OK(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, 10)
	server := &Server{
		CoreService: &core.Service{
			SyncCommitteePool: synccommittee.NewStore(),
			P2P:               &mockp2p.MockBroadcaster{},
			HeadFetcher: &mock.ChainService{
				State: st,
			},
		},
	}
	msg := &ethpb.SyncCommitteeMessage{
		Slot:           1,
		ValidatorIndex: 2,
	}
	_, err := server.SubmitSyncMessage(context.Background(), msg)
	require.NoError(t, err)
	savedMsgs, err := server.CoreService.SyncCommitteePool.SyncCommitteeMessages(1)
	require.NoError(t, err)
	require.DeepEqual(t, []*ethpb.SyncCommitteeMessage{msg}, savedMsgs)
}

func TestGetSyncSubcommitteeIndex_Ok(t *testing.T) {
	transition.SkipSlotCache.Disable()
	defer transition.SkipSlotCache.Enable()

	server := &Server{
		HeadFetcher: &mock.ChainService{
			SyncCommitteeIndices: []primitives.CommitteeIndex{0},
		},
	}
	var pubKey [fieldparams.BLSPubkeyLength]byte
	// Request slot 0, should get the index 0 for validator 0.
	res, err := server.GetSyncSubcommitteeIndex(context.Background(), &ethpb.SyncSubcommitteeIndexRequest{
		PublicKey: pubKey[:], Slot: primitives.Slot(0),
	})
	require.NoError(t, err)
	require.DeepEqual(t, []primitives.CommitteeIndex{0}, res.Indices)
}

func TestGetSyncCommitteeContribution_FiltersDuplicates(t *testing.T) {
	st, _ := util.DeterministicGenesisStateAltair(t, 10)
	syncCommitteePool := synccommittee.NewStore()
	headFetcher := &mock.ChainService{
		State:                st,
		SyncCommitteeIndices: []primitives.CommitteeIndex{10},
	}
	server := &Server{
		CoreService: &core.Service{
			SyncCommitteePool: syncCommitteePool,
			HeadFetcher:       headFetcher,
			P2P:               &mockp2p.MockBroadcaster{},
		},
		SyncCommitteePool: syncCommitteePool,
		HeadFetcher:       headFetcher,
		P2P:               &mockp2p.MockBroadcaster{},
		TimeFetcher:       &mock.ChainService{Genesis: time.Now()},
	}
	secKey, err := bls.RandKey()
	require.NoError(t, err)
	sig := secKey.Sign([]byte{'A'}).Marshal()
	msg := &ethpb.SyncCommitteeMessage{
		Slot:           1,
		ValidatorIndex: 2,
		BlockRoot:      make([]byte, 32),
		Signature:      sig,
	}
	_, err = server.SubmitSyncMessage(context.Background(), msg)
	require.NoError(t, err)
	_, err = server.SubmitSyncMessage(context.Background(), msg)
	require.NoError(t, err)
	val, err := st.ValidatorAtIndex(2)
	require.NoError(t, err)

	contr, err := server.GetSyncCommitteeContribution(context.Background(),
		&ethpb.SyncCommitteeContributionRequest{
			Slot:      1,
			PublicKey: val.PublicKey,
			SubnetId:  1})
	require.NoError(t, err)
	assert.DeepEqual(t, sig, contr.Signature)
}

func TestSubmitSignedContributionAndProof_OK(t *testing.T) {
	server := &Server{
		CoreService: &core.Service{
			SyncCommitteePool: synccommittee.NewStore(),
			Broadcaster:       &mockp2p.MockBroadcaster{},
			OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
		},
	}
	contribution := &ethpb.SignedContributionAndProof{
		Message: &ethpb.ContributionAndProof{
			Contribution: &ethpb.SyncCommitteeContribution{
				Slot:              1,
				SubcommitteeIndex: 2,
			},
		},
	}
	_, err := server.SubmitSignedContributionAndProof(context.Background(), contribution)
	require.NoError(t, err)
	savedMsgs, err := server.CoreService.SyncCommitteePool.SyncCommitteeContributions(1)
	require.NoError(t, err)
	require.DeepEqual(t, []*ethpb.SyncCommitteeContribution{contribution.Message.Contribution}, savedMsgs)
}

func TestSubmitSignedContributionAndProof_Notification(t *testing.T) {
	server := &Server{
		CoreService: &core.Service{
			SyncCommitteePool: synccommittee.NewStore(),
			Broadcaster:       &mockp2p.MockBroadcaster{},
			OperationNotifier: (&mock.ChainService{}).OperationNotifier(),
		},
	}

	// Subscribe to operation notifications.
	opChannel := make(chan *feed.Event, 1024)
	opSub := server.CoreService.OperationNotifier.OperationFeed().Subscribe(opChannel)
	defer opSub.Unsubscribe()

	contribution := &ethpb.SignedContributionAndProof{
		Message: &ethpb.ContributionAndProof{
			Contribution: &ethpb.SyncCommitteeContribution{
				Slot:              1,
				SubcommitteeIndex: 2,
			},
		},
	}
	_, err := server.SubmitSignedContributionAndProof(context.Background(), contribution)
	require.NoError(t, err)

	// Ensure the state notification was broadcast.
	notificationFound := false
	for !notificationFound {
		select {
		case event := <-opChannel:
			if event.Type == opfeed.SyncCommitteeContributionReceived {
				notificationFound = true
				data, ok := event.Data.(*opfeed.SyncCommitteeContributionReceivedData)
				assert.Equal(t, true, ok, "Entity is of the wrong type")
				assert.NotNil(t, data.Contribution)
			}
		case <-opSub.Err():
			t.Error("Subscription to state notifier failed")
			return
		}
	}
}
