package sync

import (
	"context"
	"fmt"
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
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestService_ValidateBlsToExecutionChange(t *testing.T) {
	beaconDB := testingdb.SetupDB(t)
	defaultTopic := p2p.BlsToExecutionChangeSubnetTopicFormat + "/" + encoder.ProtocolSuffixSSZSnappy
	fakeDigest := []byte{0xAB, 0x00, 0xCC, 0x9E}
	wantedExecAddress := []byte{0xd8, 0xdA, 0x6B, 0xF2, 0x69, 0x64, 0xaF, 0x9D, 0x7e, 0xEd, 0x9e, 0x03, 0xE5, 0x34, 0x15, 0xD3, 0x7a, 0xA9, 0x60, 0x45}
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	var emptySig [96]byte
	type args struct {
		pid   peer.ID
		msg   *ethpb.SignedBLSToExecutionChange
		topic string
	}
	tests := []struct {
		name     string
		svcopts  []Option
		setupSvc func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string)
		clock    *startup.Clock
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
				WithStateNotifier(chainService.StateNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: "junk",
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
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
				WithStateNotifier(chainService.StateNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: "junk",
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Already Seen Message",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				s.cfg.blsToExecPool.InsertBLSToExecChange(&ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     10,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				})
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     10,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationIgnore,
		},

		{
			name: "Non-Capella HeadState Valid Execution Change Message",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot*10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, keys := util.DeterministicGenesisStateBellatrix(t, 128)
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 50
				// Provide invalid withdrawal key for validator
				msg.Message.FromBlsPubkey = keys[51].PublicKey().Marshal()
				msg.Message.ToExecutionAddress = wantedExecAddress
				epoch := slots.ToEpoch(st.Slot())
				domain, err := signing.Domain(st.Fork(), epoch, params.BeaconConfig().DomainBLSToExecutionChange, st.GenesisValidatorsRoot())
				assert.NoError(t, err)
				htr, err := signing.Data(msg.Message.HashTreeRoot, domain)
				assert.NoError(t, err)
				msg.Signature = keys[51].Sign(htr[:]).Marshal()
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationAccept,
		},
		{
			name: "Non-existent Validator Index",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot)*time.Duration(10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, _ := util.DeterministicGenesisStateCapella(t, 128)
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 130
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Invalid Withdrawal Pubkey",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot)*time.Duration(10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, keys := util.DeterministicGenesisStateCapella(t, 128)
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 50
				// Provide invalid withdrawal key for validator
				msg.Message.FromBlsPubkey = keys[0].PublicKey().Marshal()
				msg.Message.ToExecutionAddress = wantedExecAddress
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Invalid Credentials in State",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot)*time.Duration(10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, keys := util.DeterministicGenesisStateCapella(t, 128)
				assert.NoError(t, st.ApplyToEveryValidator(func(idx int, val *ethpb.Validator) (bool, *ethpb.Validator, error) {
					newCreds := make([]byte, 32)
					newCreds[0] = params.BeaconConfig().ETH1AddressWithdrawalPrefixByte
					copy(newCreds[12:], wantedExecAddress)
					val.WithdrawalCredentials = newCreds
					return true, val, nil
				}))
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 50
				// Provide Correct withdrawal pubkey
				msg.Message.FromBlsPubkey = keys[51].PublicKey().Marshal()
				msg.Message.ToExecutionAddress = wantedExecAddress
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Invalid Execution Change Signature",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot)*time.Duration(10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, keys := util.DeterministicGenesisStateCapella(t, 128)
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 50
				// Provide invalid withdrawal key for validator
				msg.Message.FromBlsPubkey = keys[51].PublicKey().Marshal()
				msg.Message.ToExecutionAddress = wantedExecAddress
				badSig := make([]byte, 96)
				copy(badSig, []byte{'j', 'u', 'n', 'k'})
				msg.Signature = badSig
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
				}},
			want: pubsub.ValidationReject,
		},
		{
			name: "Valid Execution Change Message",
			svcopts: []Option{
				WithP2P(mockp2p.NewTestP2P(t)),
				WithInitialSync(&mockSync.Sync{IsSyncing: false}),
				WithChainService(chainService),
				WithOperationNotifier(chainService.OperationNotifier()),
				WithBlsToExecPool(blstoexec.NewPool()),
				WithStateNotifier(chainService.StateNotifier()),
			},
			clock: startup.NewClock(time.Now().Add(-time.Second*time.Duration(params.BeaconConfig().SecondsPerSlot)*time.Duration(10)), [32]byte{'A'}),
			setupSvc: func(s *Service, msg *ethpb.SignedBLSToExecutionChange, topic string) (*Service, string) {
				s.cfg.stateGen = stategen.New(beaconDB, doublylinkedtree.New())
				s.cfg.beaconDB = beaconDB
				s.cfg.clock = startup.NewClock(time.Now(), [32]byte{'A'})
				s.initCaches()
				st, keys := util.DeterministicGenesisStateCapella(t, 128)
				s.cfg.chain = &mockChain.ChainService{
					State:   st,
					Genesis: time.Now(),
				}

				msg.Message.ValidatorIndex = 50
				// Provide invalid withdrawal key for validator
				msg.Message.FromBlsPubkey = keys[51].PublicKey().Marshal()
				msg.Message.ToExecutionAddress = wantedExecAddress
				epoch := slots.ToEpoch(st.Slot())
				domain, err := signing.Domain(st.Fork(), epoch, params.BeaconConfig().DomainBLSToExecutionChange, st.GenesisValidatorsRoot())
				assert.NoError(t, err)
				htr, err := signing.Data(msg.Message.HashTreeRoot, domain)
				assert.NoError(t, err)
				msg.Signature = keys[51].Sign(htr[:]).Marshal()
				return s, topic
			},
			args: args{
				pid:   "random",
				topic: fmt.Sprintf(defaultTopic, fakeDigest),
				msg: &ethpb.SignedBLSToExecutionChange{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     0,
						FromBlsPubkey:      make([]byte, 48),
						ToExecutionAddress: make([]byte, 20),
					},
					Signature: emptySig[:],
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
			opts := []Option{WithClockWaiter(cw)}
			svc := NewService(ctx, append(opts, tt.svcopts...)...)
			svc, tt.args.topic = tt.setupSvc(svc, tt.args.msg, tt.args.topic)
			go svc.Start()
			if tt.clock == nil {
				tt.clock = startup.NewClock(time.Now(), [32]byte{})
			}
			require.NoError(t, cw.SetClock(tt.clock))
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
			if got, err := svc.validateBlsToExecutionChange(ctx, tt.args.pid, msg); got != tt.want {
				_ = err
				t.Errorf("validateBlsToExecutionChange() = %v, want %v", got, tt.want)
			}
		})
	}
}
