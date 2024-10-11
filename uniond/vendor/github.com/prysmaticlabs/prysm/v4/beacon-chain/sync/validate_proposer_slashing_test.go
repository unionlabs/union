package sync

import (
	"bytes"
	"context"
	"crypto/rand"
	"reflect"
	"testing"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
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
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func setupValidProposerSlashing(t *testing.T) (*ethpb.ProposerSlashing, state.BeaconState) {
	validators := make([]*ethpb.Validator, 100)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			EffectiveBalance:  params.BeaconConfig().MaxEffectiveBalance,
			Slashed:           false,
			ExitEpoch:         params.BeaconConfig().FarFutureEpoch,
			WithdrawableEpoch: params.BeaconConfig().FarFutureEpoch,
			ActivationEpoch:   0,
		}
	}
	validatorBalances := make([]uint64, len(validators))
	for i := 0; i < len(validatorBalances); i++ {
		validatorBalances[i] = params.BeaconConfig().MaxEffectiveBalance
	}

	currentSlot := primitives.Slot(0)
	st, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Validators: validators,
		Slot:       currentSlot,
		Balances:   validatorBalances,
		Fork: &ethpb.Fork{
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			Epoch:           0,
		},
		Slashings:   make([]uint64, params.BeaconConfig().EpochsPerSlashingsVector),
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),

		StateRoots:        make([][]byte, params.BeaconConfig().SlotsPerHistoricalRoot),
		BlockRoots:        make([][]byte, params.BeaconConfig().SlotsPerHistoricalRoot),
		LatestBlockHeader: &ethpb.BeaconBlockHeader{},
	})
	require.NoError(t, err)

	privKey, err := bls.RandKey()
	require.NoError(t, err)
	someRoot := [32]byte{1, 2, 3}
	someRoot2 := [32]byte{4, 5, 6}
	header1 := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			ProposerIndex: 1,
			Slot:          0,
			ParentRoot:    someRoot[:],
			StateRoot:     someRoot[:],
			BodyRoot:      someRoot[:],
		},
	}
	header1.Signature, err = signing.ComputeDomainAndSign(st, coreTime.CurrentEpoch(st), header1.Header, params.BeaconConfig().DomainBeaconProposer, privKey)
	require.NoError(t, err)

	header2 := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			ProposerIndex: 1,
			Slot:          0,
			ParentRoot:    someRoot2[:],
			StateRoot:     someRoot2[:],
			BodyRoot:      someRoot2[:],
		},
	}
	header2.Signature, err = signing.ComputeDomainAndSign(st, coreTime.CurrentEpoch(st), header2.Header, params.BeaconConfig().DomainBeaconProposer, privKey)
	require.NoError(t, err)

	slashing := &ethpb.ProposerSlashing{
		Header_1: header1,
		Header_2: header2,
	}
	val, err := st.ValidatorAtIndex(1)
	require.NoError(t, err)
	val.PublicKey = privKey.PublicKey().Marshal()
	require.NoError(t, st.UpdateValidatorAtIndex(1, val))

	b := make([]byte, 32)
	_, err = rand.Read(b)
	require.NoError(t, err)

	return slashing, st
}

func TestValidateProposerSlashing_ValidSlashing(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	slashing, s := setupValidProposerSlashing(t)

	chain := &mock.ChainService{State: s, Genesis: time.Now()}
	r := &Service{
		cfg: &config{
			p2p:               p,
			chain:             chain,
			clock:             startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
			initialSync:       &mockSync.Sync{IsSyncing: false},
			operationNotifier: chain.OperationNotifier(),
		},
		seenProposerSlashingCache: lruwrpr.New(10),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	d, err := r.currentForkDigest()
	assert.NoError(t, err)
	topic = r.addDigestToTopic(topic, d)
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}

	res, err := r.validateProposerSlashing(ctx, "", m)
	assert.NoError(t, err)
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, true, valid, "Failed validation")
	assert.NotNil(t, m.ValidatorData, "Decoded message was not set on the message validator data")
}

func TestValidateProposerSlashing_ContextTimeout(t *testing.T) {
	p := p2ptest.NewTestP2P(t)

	slashing, st := setupValidProposerSlashing(t)
	slashing.Header_1.Header.Slot = 100000000
	err := st.SetJustificationBits(bitfield.Bitvector4{0x0F}) // 0b1111
	require.NoError(t, err)
	err = st.SetPreviousJustifiedCheckpoint(&ethpb.Checkpoint{Epoch: 0, Root: []byte{}})
	require.NoError(t, err)
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	r := &Service{
		cfg: &config{
			p2p:         p,
			chain:       &mock.ChainService{State: st},
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		seenProposerSlashingCache: lruwrpr.New(10),
	}

	buf := new(bytes.Buffer)
	_, err = p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateProposerSlashing(ctx, "", m)
	assert.NotNil(t, err)
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "Slashing from the far distant future should have timed out and returned false")
}

func TestValidateProposerSlashing_Syncing(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	slashing, s := setupValidProposerSlashing(t)

	r := &Service{
		cfg: &config{
			p2p:         p,
			chain:       &mock.ChainService{State: s},
			initialSync: &mockSync.Sync{IsSyncing: true},
		},
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)
	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	m := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateProposerSlashing(ctx, "", m)
	_ = err
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "Did not fail validation")
}
