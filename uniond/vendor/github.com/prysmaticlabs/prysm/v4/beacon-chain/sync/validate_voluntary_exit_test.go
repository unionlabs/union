package sync

import (
	"bytes"
	"context"
	"crypto/rand"
	"math"
	"reflect"
	"testing"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	lruwrpr "github.com/prysmaticlabs/prysm/v4/cache/lru"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func setupValidExit(t *testing.T) (*ethpb.SignedVoluntaryExit, state.BeaconState) {
	exit := &ethpb.SignedVoluntaryExit{
		Exit: &ethpb.VoluntaryExit{
			ValidatorIndex: 0,
			Epoch:          1 + params.BeaconConfig().ShardCommitteePeriod,
		},
	}
	registry := []*ethpb.Validator{
		{
			ExitEpoch:       params.BeaconConfig().FarFutureEpoch,
			ActivationEpoch: 0,
		},
	}
	st, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: registry,
		Fork: &ethpb.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
		},
		Slot: params.BeaconConfig().SlotsPerEpoch * 5,
	})
	require.NoError(t, err)
	err = st.SetSlot(st.Slot() + params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().ShardCommitteePeriod)))
	require.NoError(t, err)

	priv, err := bls.RandKey()
	require.NoError(t, err)
	exit.Signature, err = signing.ComputeDomainAndSign(st, coreTime.CurrentEpoch(st), exit.Exit, params.BeaconConfig().DomainVoluntaryExit, priv)
	require.NoError(t, err)

	val, err := st.ValidatorAtIndex(0)
	require.NoError(t, err)
	val.PublicKey = priv.PublicKey().Marshal()
	require.NoError(t, st.UpdateValidatorAtIndex(0, val))

	b := make([]byte, 32)
	_, err = rand.Read(b)
	require.NoError(t, err)

	return exit, st
}

func TestValidateVoluntaryExit_ValidExit(t *testing.T) {
	cfg := params.BeaconConfig().Copy()
	cfg.DenebForkEpoch = math.MaxUint64
	params.OverrideBeaconConfig(cfg)
	params.SetupTestConfigCleanup(t)

	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	exit, s := setupValidExit(t)

	gt := time.Now()
	mockChainService := &mock.ChainService{
		State:   s,
		Genesis: gt,
	}
	r := &Service{
		cfg: &config{
			p2p:               p,
			chain:             mockChainService,
			clock:             startup.NewClock(gt, [32]byte{}),
			initialSync:       &mockSync.Sync{IsSyncing: false},
			operationNotifier: mockChainService.OperationNotifier(),
		},
		seenExitCache: lruwrpr.New(10),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, exit)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(exit)]
	d, err := r.currentForkDigest()
	assert.NoError(t, err)
	topic = r.addDigestToTopic(topic, d)
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}

	// Subscribe to operation notifications.
	opChannel := make(chan *feed.Event, 1)
	opSub := r.cfg.operationNotifier.OperationFeed().Subscribe(opChannel)
	defer opSub.Unsubscribe()

	res, err := r.validateVoluntaryExit(ctx, "", m)
	require.NoError(t, err)
	require.Equal(t, pubsub.ValidationAccept, res, "Failed validation")
	require.NotNil(t, m.ValidatorData, "Decoded message was not set on the message validator data")

	select {
	case event := <-opChannel:
		if event.Type == opfeed.ExitReceived {
			_, ok := event.Data.(*opfeed.ExitReceivedData)
			assert.Equal(t, true, ok, "Entity is not of type *opfeed.ExitReceivedData")
		} else {
			t.Error("Unexpected event type received")
		}
	case <-opSub.Err():
		t.Error("Subscription to state notifier failed")
	case <-time.After(10 * time.Second): // Timeout to prevent hanging tests
		t.Error("Timeout waiting for exit notification")
	}
}

func TestValidateVoluntaryExit_InvalidExitSlot(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	exit, s := setupValidExit(t)
	// Set state slot to 1 to cause exit object fail to verify.
	require.NoError(t, s.SetSlot(1))
	r := &Service{
		cfg: &config{
			p2p: p,
			chain: &mock.ChainService{
				State: s,
			},
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		seenExitCache: lruwrpr.New(10),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, exit)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(exit)]
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateVoluntaryExit(ctx, "", m)
	_ = err
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "passed validation")
}

func TestValidateVoluntaryExit_ValidExit_Syncing(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	exit, s := setupValidExit(t)

	r := &Service{
		cfg: &config{
			p2p: p,
			chain: &mock.ChainService{
				State: s,
			},
			initialSync: &mockSync.Sync{IsSyncing: true},
		},
	}
	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, exit)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(exit)]
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateVoluntaryExit(ctx, "", m)
	_ = err
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "Validation should have failed")
}
