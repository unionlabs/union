package sync

import (
	"bytes"
	"context"
	"math/rand"
	"reflect"
	"testing"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func setupValidAttesterSlashing(t *testing.T) (*ethpb.AttesterSlashing, state.BeaconState) {
	s, privKeys := util.DeterministicGenesisState(t, 5)
	vals := s.Validators()
	for _, vv := range vals {
		vv.WithdrawableEpoch = primitives.Epoch(1 * params.BeaconConfig().SlotsPerEpoch)
	}
	require.NoError(t, s.SetValidators(vals))

	att1 := util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{Epoch: 1},
		},
		AttestingIndices: []uint64{0, 1},
	})
	domain, err := signing.Domain(s.Fork(), 0, params.BeaconConfig().DomainBeaconAttester, s.GenesisValidatorsRoot())
	require.NoError(t, err)
	hashTreeRoot, err := signing.ComputeSigningRoot(att1.Data, domain)
	assert.NoError(t, err)
	sig0 := privKeys[0].Sign(hashTreeRoot[:])
	sig1 := privKeys[1].Sign(hashTreeRoot[:])
	aggregateSig := bls.AggregateSignatures([]bls.Signature{sig0, sig1})
	att1.Signature = aggregateSig.Marshal()

	att2 := util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
		AttestingIndices: []uint64{0, 1},
	})
	hashTreeRoot, err = signing.ComputeSigningRoot(att2.Data, domain)
	assert.NoError(t, err)
	sig0 = privKeys[0].Sign(hashTreeRoot[:])
	sig1 = privKeys[1].Sign(hashTreeRoot[:])
	aggregateSig = bls.AggregateSignatures([]bls.Signature{sig0, sig1})
	att2.Signature = aggregateSig.Marshal()

	slashing := &ethpb.AttesterSlashing{
		Attestation_1: att1,
		Attestation_2: att2,
	}

	currentSlot := 2 * params.BeaconConfig().SlotsPerEpoch
	require.NoError(t, s.SetSlot(currentSlot))

	b := make([]byte, 32)
	_, err = rand.Read(b)
	require.NoError(t, err)

	return slashing, s
}

func TestValidateAttesterSlashing_ValidSlashing(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	slashing, s := setupValidAttesterSlashing(t)

	chain := &mock.ChainService{State: s, Genesis: time.Now()}
	r := &Service{
		cfg: &config{
			p2p:               p,
			chain:             chain,
			clock:             startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
			initialSync:       &mockSync.Sync{IsSyncing: false},
			operationNotifier: chain.OperationNotifier(),
		},
		seenAttesterSlashingCache: make(map[uint64]bool),
		subHandler:                newSubTopicHandler(),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)

	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	d, err := r.currentForkDigest()
	assert.NoError(t, err)
	topic = r.addDigestToTopic(topic, d)
	msg := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateAttesterSlashing(ctx, "foobar", msg)
	assert.NoError(t, err)
	valid := res == pubsub.ValidationAccept

	assert.Equal(t, true, valid, "Failed Validation")
	assert.NotNil(t, msg.ValidatorData, "Decoded message was not set on the message validator data")
}

func TestValidateAttesterSlashing_InvalidSlashing_WithdrawableEpoch(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	slashing, s := setupValidAttesterSlashing(t)
	// Set only one of the  validators as withdrawn
	vals := s.Validators()
	vals[1].WithdrawableEpoch = primitives.Epoch(1)

	require.NoError(t, s.SetValidators(vals))

	chain := &mock.ChainService{State: s, Genesis: time.Now()}
	r := &Service{
		cfg: &config{
			p2p:               p,
			chain:             chain,
			clock:             startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
			initialSync:       &mockSync.Sync{IsSyncing: false},
			operationNotifier: chain.OperationNotifier(),
		},
		seenAttesterSlashingCache: make(map[uint64]bool),
		subHandler:                newSubTopicHandler(),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)

	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	d, err := r.currentForkDigest()
	assert.NoError(t, err)
	topic = r.addDigestToTopic(topic, d)
	msg := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateAttesterSlashing(ctx, "foobar", msg)
	assert.NoError(t, err)
	valid := res == pubsub.ValidationAccept

	assert.Equal(t, true, valid, "Rejected Validation")

	// Set all validators as withdrawn.
	vals = s.Validators()
	for _, vv := range vals {
		vv.WithdrawableEpoch = primitives.Epoch(1)
	}

	require.NoError(t, s.SetValidators(vals))
	res, err = r.validateAttesterSlashing(ctx, "foobar", msg)
	assert.ErrorContains(t, "none of the validators are slashable", err)
	invalid := res == pubsub.ValidationReject

	assert.Equal(t, true, invalid, "Passed Validation")
}

func TestValidateAttesterSlashing_CanFilter(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	chain := &mock.ChainService{Genesis: time.Now()}
	r := &Service{
		cfg: &config{
			p2p:         p,
			initialSync: &mockSync.Sync{IsSyncing: false},
			chain:       chain,
			clock:       startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		seenAttesterSlashingCache: make(map[uint64]bool),
		subHandler:                newSubTopicHandler(),
	}

	r.setAttesterSlashingIndicesSeen([]uint64{1, 2, 3, 4}, []uint64{3, 4, 5, 6})

	// The below attestations should be filtered hence bad signature is ok.
	topic := p2p.GossipTypeMapping[reflect.TypeOf(&ethpb.AttesterSlashing{})]
	d, err := r.currentForkDigest()
	assert.NoError(t, err)
	topic = r.addDigestToTopic(topic, d)
	buf := new(bytes.Buffer)
	_, err = p.Encoding().EncodeGossip(buf, &ethpb.AttesterSlashing{
		Attestation_1: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
			AttestingIndices: []uint64{3},
		}),
		Attestation_2: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
			AttestingIndices: []uint64{3},
		}),
	})
	require.NoError(t, err)
	msg := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateAttesterSlashing(ctx, "foobar", msg)
	_ = err
	ignored := res == pubsub.ValidationIgnore
	assert.Equal(t, true, ignored)

	buf = new(bytes.Buffer)
	_, err = p.Encoding().EncodeGossip(buf, &ethpb.AttesterSlashing{
		Attestation_1: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
			AttestingIndices: []uint64{4, 3},
		}),
		Attestation_2: util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
			AttestingIndices: []uint64{3, 4},
		}),
	})
	require.NoError(t, err)
	msg = &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err = r.validateAttesterSlashing(ctx, "foobar", msg)
	_ = err
	ignored = res == pubsub.ValidationIgnore
	assert.Equal(t, true, ignored)
}

func TestValidateAttesterSlashing_ContextTimeout(t *testing.T) {
	p := p2ptest.NewTestP2P(t)

	slashing, s := setupValidAttesterSlashing(t)
	slashing.Attestation_1.Data.Target.Epoch = 100000000

	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()

	chain := &mock.ChainService{State: s}
	r := &Service{
		cfg: &config{
			p2p:         p,
			chain:       chain,
			clock:       startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		seenAttesterSlashingCache: make(map[uint64]bool),
	}

	buf := new(bytes.Buffer)
	_, err := p.Encoding().EncodeGossip(buf, slashing)
	require.NoError(t, err)

	topic := p2p.GossipTypeMapping[reflect.TypeOf(slashing)]
	msg := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateAttesterSlashing(ctx, "foobar", msg)
	_ = err
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "slashing from the far distant future should have timed out and returned false")
}

func TestValidateAttesterSlashing_Syncing(t *testing.T) {
	p := p2ptest.NewTestP2P(t)
	ctx := context.Background()

	slashing, s := setupValidAttesterSlashing(t)

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
	msg := &pubsub.Message{
		Message: &pubsubpb.Message{
			Data:  buf.Bytes(),
			Topic: &topic,
		},
	}
	res, err := r.validateAttesterSlashing(ctx, "foobar", msg)
	_ = err
	valid := res == pubsub.ValidationAccept
	assert.Equal(t, false, valid, "Passed validation")
}

func TestSeenAttesterSlashingIndices(t *testing.T) {
	tt := []struct {
		saveIndices1  []uint64
		saveIndices2  []uint64
		checkIndices1 []uint64
		checkIndices2 []uint64
		seen          bool
	}{
		{
			saveIndices1:  []uint64{0, 1, 2},
			saveIndices2:  []uint64{0},
			checkIndices1: []uint64{0, 1, 2},
			checkIndices2: []uint64{0},
			seen:          true,
		},
		{
			saveIndices1:  []uint64{100, 99, 98},
			saveIndices2:  []uint64{99, 98, 97},
			checkIndices1: []uint64{99, 98},
			checkIndices2: []uint64{99, 98},
			seen:          true,
		},
		{
			saveIndices1:  []uint64{100},
			saveIndices2:  []uint64{100},
			checkIndices1: []uint64{100, 101},
			checkIndices2: []uint64{100, 101},
			seen:          false,
		},
		{
			saveIndices1:  []uint64{100, 99, 98},
			saveIndices2:  []uint64{99, 98, 97},
			checkIndices1: []uint64{99, 98, 97},
			checkIndices2: []uint64{99, 98, 97},
			seen:          false,
		},
		{
			saveIndices1:  []uint64{100, 99, 98},
			saveIndices2:  []uint64{99, 98, 97},
			checkIndices1: []uint64{101, 100},
			checkIndices2: []uint64{101},
			seen:          false,
		},
	}
	for _, tc := range tt {
		r := &Service{
			seenAttesterSlashingCache: map[uint64]bool{},
		}
		r.setAttesterSlashingIndicesSeen(tc.saveIndices1, tc.saveIndices2)
		slashedVals := slice.IntersectionUint64(tc.checkIndices1, tc.checkIndices2)
		assert.Equal(t, tc.seen, r.hasSeenAttesterSlashingIndices(slashedVals))
	}
}
