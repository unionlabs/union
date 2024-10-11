package sync

import (
	"context"
	"fmt"
	"reflect"
	"testing"
	"time"

	"github.com/golang/snappy"
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/libp2p/go-libp2p/core/peer"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	testingdb "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestService_ValidateSyncCommitteeMessage(t *testing.T) {
	beaconDB := testingdb.SetupDB(t)
	headRoot, keys := fillUpBlocksAndState(context.Background(), t, beaconDB)
	defaultTopic := p2p.SyncCommitteeSubnetTopicFormat
	fakeDigest := []byte{0xAB, 0x00, 0xCC, 0x9E}
	defaultTopic = defaultTopic + "/" + encoder.ProtocolSuffixSSZSnappy
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	var emptySig [96]byte
	type args struct {
		pid   peer.ID
		msg   *ethpb.SyncCommitteeMessage
		topic string
	}
	tests := []struct {
		name     string
		svcopts  []Option
		setupSvc func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock)
		args     args
		want     pubsub.ValidationResult
	}{
		{
			name: "Is syncing",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: true}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				msg.BlockRoot = headRoot[:]
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				return s, topic, startup.NewClock(time.Now(), [32]byte{})
			},
			args: args{
				pid:   "random",
				topic: "junk",
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},
		{
			name: "Bad Topic",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				msg.BlockRoot = headRoot[:]
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				return s, topic, startup.NewClock(time.Now(), [32]byte{})
			},
			args: args{
				pid:   "random",
				topic: "junk",
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Future Slot Message",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				return s, topic, startup.NewClock(time.Now(), [32]byte{})
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest, 0),
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           10,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},
		{
			name: "Already Seen Message",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				m := &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
				}
				s.setSeenSyncMessageIndexSlot(m, 0)
				return s, topic, startup.NewClock(time.Now(), [32]byte{})
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest, 0),
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},
		{
			name: "Non-existent block root",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				s.cfg.chain = &mockChain.ChainService{Genesis: time.Now()}
				incorrectRoot := [32]byte{0xBB}
				msg.BlockRoot = incorrectRoot[:]

				gt := time.Now().Add(-time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Duration(10))
				return s, topic, startup.NewClock(gt, [32]byte{'A'})
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest, 0),
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},
		{
			name: "Subnet is non-existent",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				msg.BlockRoot = headRoot[:]
				hState, err := beaconDB.State(context.Background(), headRoot)
				assert.NoError(t, err)
				s.cfg.chain = &mockChain.ChainService{
					SyncCommitteeIndices: []primitives.CommitteeIndex{0},
					Genesis:              time.Now(),
				}
				numOfVals := hState.NumValidators()

				chosenVal := numOfVals - 10
				msg.Signature = emptySig[:]
				msg.BlockRoot = headRoot[:]
				msg.ValidatorIndex = primitives.ValidatorIndex(chosenVal)
				msg.Slot = slots.PrevSlot(hState.Slot())

				gt := time.Now().Add(-time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Duration(hState.Slot()-1))
				vr := [32]byte{'A'}
				clock := startup.NewClock(gt, vr)
				digest, err := forks.CreateForkDigest(gt, vr[:])
				assert.NoError(t, err)
				actualTopic := fmt.Sprintf(defaultTopic, digest, 5)

				return s, actualTopic, clock
			},
			args: args{
				pid:   "random",
				topic: defaultTopic,
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Validator is non-existent",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				msg.BlockRoot = headRoot[:]
				hState, err := beaconDB.State(context.Background(), headRoot)
				assert.NoError(t, err)
				s.cfg.chain = &mockChain.ChainService{
					Genesis: time.Now(),
				}

				numOfVals := hState.NumValidators()

				chosenVal := numOfVals + 10
				msg.Signature = emptySig[:]
				msg.BlockRoot = headRoot[:]
				msg.ValidatorIndex = primitives.ValidatorIndex(chosenVal)
				msg.Slot = slots.PrevSlot(hState.Slot())

				gt := time.Now().Add(-time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Duration(hState.Slot()-1))
				vr := [32]byte{'A'}
				digest, err := forks.CreateForkDigest(gt, vr[:])
				assert.NoError(t, err)
				actualTopic := fmt.Sprintf(defaultTopic, digest, 5)

				return s, actualTopic, startup.NewClock(gt, vr)
			},
			args: args{
				pid:   "random",
				topic: defaultTopic,
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},
		{
			name: "Invalid Sync Committee Signature",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				msg.BlockRoot = headRoot[:]
				hState, err := beaconDB.State(context.Background(), headRoot)
				assert.NoError(t, err)

				numOfVals := hState.NumValidators()

				chosenVal := numOfVals - 10
				msg.Signature = emptySig[:]
				msg.BlockRoot = headRoot[:]
				msg.ValidatorIndex = primitives.ValidatorIndex(chosenVal)
				msg.Slot = slots.PrevSlot(hState.Slot())

				d, err := signing.Domain(hState.Fork(), slots.ToEpoch(hState.Slot()), params.BeaconConfig().DomainSyncCommittee, hState.GenesisValidatorsRoot())
				assert.NoError(t, err)
				subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
				s.cfg.chain = &mockChain.ChainService{
					SyncCommitteeIndices: []primitives.CommitteeIndex{primitives.CommitteeIndex(subCommitteeSize)},
					SyncCommitteeDomain:  d,
					PublicKey:            bytesutil.ToBytes48(keys[chosenVal].PublicKey().Marshal()),
					Genesis:              time.Now(),
				}

				// Set Topic and Subnet
				gt := time.Now().Add(-time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Duration(hState.Slot()-1))
				vr := [32]byte{'A'}
				digest, err := forks.CreateForkDigest(gt, vr[:])
				assert.NoError(t, err)
				actualTopic := fmt.Sprintf(defaultTopic, digest, 5)

				return s, actualTopic, startup.NewClock(gt, vr)
			},
			args: args{
				pid:   "random",
				topic: defaultTopic,
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Valid Sync Committee Signature",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string, *startup.Clock) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.initCaches()
				msg.BlockRoot = headRoot[:]
				hState, err := beaconDB.State(context.Background(), headRoot)
				assert.NoError(t, err)
				subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount

				numOfVals := hState.NumValidators()

				chosenVal := numOfVals - 10
				d, err := signing.Domain(hState.Fork(), slots.ToEpoch(hState.Slot()), params.BeaconConfig().DomainSyncCommittee, hState.GenesisValidatorsRoot())
				assert.NoError(t, err)
				rawBytes := p2ptypes.SSZBytes(headRoot[:])
				sigRoot, err := signing.ComputeSigningRoot(&rawBytes, d)
				assert.NoError(t, err)

				s.cfg.chain = &mockChain.ChainService{
					SyncCommitteeIndices: []primitives.CommitteeIndex{primitives.CommitteeIndex(subCommitteeSize)},
					SyncCommitteeDomain:  d,
					PublicKey:            bytesutil.ToBytes48(keys[chosenVal].PublicKey().Marshal()),
					Genesis:              time.Now(),
				}

				msg.Signature = keys[chosenVal].Sign(sigRoot[:]).Marshal()
				msg.BlockRoot = headRoot[:]
				msg.ValidatorIndex = primitives.ValidatorIndex(chosenVal)
				msg.Slot = slots.PrevSlot(hState.Slot())

				// Set Topic and Subnet
				gt := time.Now().Add(-time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot) * time.Duration(hState.Slot()-1))
				vr := [32]byte{'A'}
				digest, err := forks.CreateForkDigest(gt, vr[:])
				assert.NoError(t, err)
				actualTopic := fmt.Sprintf(defaultTopic, digest, 1)

				return s, actualTopic, startup.NewClock(gt, vr)
			},
			args: args{
				pid:   "random",
				topic: defaultTopic,
				msg: &ethpb.SyncCommitteeMessage{
					Slot:           1,
					ValidatorIndex: 1,
					BlockRoot:      params.BeaconConfig().ZeroHash[:],
					Signature:      emptySig[:],
				}},
			want: pubsub.ValidationAccept,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			ctx, cancel := context.WithCancel(ctx)
			defer cancel()

			cw := startup.NewClockSynchronizer()
			opts := []Option{WithClockWaiter(cw), WithStateNotifier(chainService.StateNotifier())}
			svc := NewService(ctx, append(opts, tt.svcopts...)...)
			var clock *startup.Clock
			svc, tt.args.topic, clock = tt.setupSvc(svc, tt.args.msg, tt.args.topic)
			go svc.Start()
			require.NoError(t, cw.SetClock(clock))
			svc.verifierWaiter = verification.NewInitializerWaiter(cw, chainService.ForkChoiceStore, svc.cfg.stateGen)

			marshalledObj, err := tt.args.msg.MarshalSSZ()
			assert.NoError(t, err)
			marshalledObj = snappy.Encode(nil, marshalledObj)
			msg := &pubsub.Message{
				Message: &pubsubpb.Message{
					Data:  marshalledObj,
					Topic: &tt.args.topic,
				},
				ReceivedFrom:  "",
				ValidatorData: nil,
			}
			for i := 0; i < 10; i++ {
				if !svc.chainIsStarted() {
					time.Sleep(100 * time.Millisecond)
				}
			}
			require.Equal(t, true, svc.chainIsStarted())
			if got, err := svc.validateSyncCommitteeMessage(ctx, tt.args.pid, msg); got != tt.want {
				_ = err
				t.Errorf("validateSyncCommitteeMessage() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestService_ignoreHasSeenSyncMsg(t *testing.T) {
	tests := []struct {
		name      string
		setupSvc  func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string)
		msg       *ethpb.SyncCommitteeMessage
		committee []primitives.CommitteeIndex
		want      pubsub.ValidationResult
	}{
		{
			name: "has seen",
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string) {
				s.initCaches()
				m := &ethpb.SyncCommitteeMessage{
					Slot:      1,
					BlockRoot: params.BeaconConfig().ZeroHash[:],
				}
				s.setSeenSyncMessageIndexSlot(m, 0)
				return s, ""
			},
			msg: &ethpb.SyncCommitteeMessage{ValidatorIndex: 0, Slot: 1,
				BlockRoot: params.BeaconConfig().ZeroHash[:]},
			committee: []primitives.CommitteeIndex{1, 2, 3},
			want:      pubsub.ValidationIgnore,
		},
		{
			name: "has not seen",
			setupSvc: func(s *Service, msg *ethpb.SyncCommitteeMessage, topic string) (*Service, string) {
				s.initCaches()
				m := &ethpb.SyncCommitteeMessage{
					Slot:      1,
					BlockRoot: params.BeaconConfig().ZeroHash[:],
				}
				s.setSeenSyncMessageIndexSlot(m, 0)
				return s, ""
			},
			msg: &ethpb.SyncCommitteeMessage{ValidatorIndex: 1, Slot: 1,
				BlockRoot: bytesutil.PadTo([]byte{'A'}, 32)},
			committee: []primitives.CommitteeIndex{1, 2, 3},
			want:      pubsub.ValidationAccept,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &Service{
				cfg: &config{chain: &mockChain.ChainService{}},
			}
			s, _ = tt.setupSvc(s, tt.msg, "")
			f := s.ignoreHasSeenSyncMsg(context.Background(), tt.msg, tt.committee)
			result, err := f(context.Background())
			_ = err
			require.Equal(t, tt.want, result)
		})
	}
}

func TestService_rejectIncorrectSyncCommittee(t *testing.T) {
	tests := []struct {
		name             string
		cfg              *config
		setupTopic       func(s *Service) string
		committeeIndices []primitives.CommitteeIndex
		want             pubsub.ValidationResult
	}{
		{
			name: "invalid",
			cfg: &config{
				chain: &mockChain.ChainService{},
				clock: startup.NewClock(time.Now(), [32]byte{1}),
			},
			committeeIndices: []primitives.CommitteeIndex{0},
			setupTopic: func(_ *Service) string {
				return "foobar"
			},
			want: pubsub.ValidationReject,
		},
		{
			name: "valid",
			cfg: &config{
				chain: &mockChain.ChainService{},
				clock: startup.NewClock(time.Now(), [32]byte{1}),
			},
			committeeIndices: []primitives.CommitteeIndex{0},
			setupTopic: func(s *Service) string {
				format := p2p.GossipTypeMapping[reflect.TypeOf(&ethpb.SyncCommitteeMessage{})]

				digest, err := s.currentForkDigest()
				require.NoError(t, err)
				prefix := fmt.Sprintf(format, digest, 0 /* validator index 0 */)
				topic := prefix + "foobar"
				return topic
			},
			want: pubsub.ValidationAccept,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := &Service{
				cfg: tt.cfg,
			}
			topic := tt.setupTopic(s)
			f := s.rejectIncorrectSyncCommittee(tt.committeeIndices, topic)
			result, err := f(context.Background())
			_ = err
			require.Equal(t, tt.want, result)
		})
	}
}

func Test_ignoreEmptyCommittee(t *testing.T) {
	tests := []struct {
		name      string
		committee []primitives.CommitteeIndex
		want      pubsub.ValidationResult
	}{
		{
			name:      "nil",
			committee: nil,
			want:      pubsub.ValidationIgnore,
		},
		{
			name:      "empty",
			committee: []primitives.CommitteeIndex{},
			want:      pubsub.ValidationIgnore,
		},
		{
			name:      "non-empty",
			committee: []primitives.CommitteeIndex{1},
			want:      pubsub.ValidationAccept,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			f := ignoreEmptyCommittee(tt.committee)
			result, err := f(context.Background())
			_ = err
			require.Equal(t, tt.want, result)
		})
	}
}
