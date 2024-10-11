// Package testing includes useful mocks for writing unit
// tests which depend on logic from the blockchain package.
package testing

import (
	"bytes"
	"context"
	"sync"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch/precompute"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/das"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	forkchoice2 "github.com/prysmaticlabs/prysm/v4/consensus-types/forkchoice"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/sirupsen/logrus"
)

var ErrNilState = errors.New("nil state")

// ChainService defines the mock interface for testing
type ChainService struct {
	NotFinalized                bool
	Optimistic                  bool
	ValidAttestation            bool
	ValidatorsRoot              [32]byte
	PublicKey                   [fieldparams.BLSPubkeyLength]byte
	FinalizedCheckPoint         *ethpb.Checkpoint
	CurrentJustifiedCheckPoint  *ethpb.Checkpoint
	PreviousJustifiedCheckPoint *ethpb.Checkpoint
	Slot                        *primitives.Slot // Pointer because 0 is a useful value, so checking against it can be incorrect.
	Balance                     *precompute.Balance
	CanonicalRoots              map[[32]byte]bool
	Fork                        *ethpb.Fork
	ETH1Data                    *ethpb.Eth1Data
	InitSyncBlockRoots          map[[32]byte]bool
	DB                          db.Database
	State                       state.BeaconState
	Block                       interfaces.ReadOnlySignedBeaconBlock
	VerifyBlkDescendantErr      error
	stateNotifier               statefeed.Notifier
	BlocksReceived              []interfaces.ReadOnlySignedBeaconBlock
	SyncCommitteeIndices        []primitives.CommitteeIndex
	blockNotifier               blockfeed.Notifier
	opNotifier                  opfeed.Notifier
	Root                        []byte
	SyncCommitteeDomain         []byte
	SyncSelectionProofDomain    []byte
	SyncContributionProofDomain []byte
	SyncCommitteePubkeys        [][]byte
	Genesis                     time.Time
	ForkChoiceStore             forkchoice.ForkChoicer
	ReceiveBlockMockErr         error
	OptimisticCheckRootReceived [32]byte
	FinalizedRoots              map[[32]byte]bool
	OptimisticRoots             map[[32]byte]bool
	BlockSlot                   primitives.Slot
	SyncingRoot                 [32]byte
	Blobs                       []blocks.VerifiedROBlob
	TargetRoot                  [32]byte
}

func (s *ChainService) Ancestor(ctx context.Context, root []byte, slot primitives.Slot) ([]byte, error) {
	r, err := s.ForkChoiceStore.AncestorRoot(ctx, bytesutil.ToBytes32(root), slot)
	return r[:], err
}

// StateNotifier mocks the same method in the chain service.
func (s *ChainService) StateNotifier() statefeed.Notifier {
	if s.stateNotifier == nil {
		s.stateNotifier = &MockStateNotifier{}
	}
	return s.stateNotifier
}

// BlockNotifier mocks the same method in the chain service.
func (s *ChainService) BlockNotifier() blockfeed.Notifier {
	if s.blockNotifier == nil {
		s.blockNotifier = &MockBlockNotifier{}
	}
	return s.blockNotifier
}

// MockBlockNotifier mocks the block notifier.
type MockBlockNotifier struct {
	feed *event.Feed
}

// BlockFeed returns a block feed.
func (mbn *MockBlockNotifier) BlockFeed() *event.Feed {
	if mbn.feed == nil {
		mbn.feed = new(event.Feed)
	}
	return mbn.feed
}

// MockStateNotifier mocks the state notifier.
type MockStateNotifier struct {
	feed     *event.Feed
	feedLock sync.Mutex

	recv     []*feed.Event
	recvLock sync.Mutex
	recvCh   chan *feed.Event

	RecordEvents bool
}

// ReceivedEvents returns the events received by the state feed in this mock.
func (msn *MockStateNotifier) ReceivedEvents() []*feed.Event {
	msn.recvLock.Lock()
	defer msn.recvLock.Unlock()
	return msn.recv
}

// StateFeed returns a state feed.
func (msn *MockStateNotifier) StateFeed() *event.Feed {
	msn.feedLock.Lock()
	defer msn.feedLock.Unlock()

	if msn.feed == nil && msn.recvCh == nil {
		msn.feed = new(event.Feed)
		if msn.RecordEvents {
			msn.recvCh = make(chan *feed.Event)
			sub := msn.feed.Subscribe(msn.recvCh)

			go func() {
				for {
					select {
					case evt := <-msn.recvCh:
						msn.recvLock.Lock()
						msn.recv = append(msn.recv, evt)
						msn.recvLock.Unlock()
					case <-sub.Err():
						sub.Unsubscribe()
						return
					}
				}
			}()
		}
	}
	return msn.feed
}

// OperationNotifier mocks the same method in the chain service.
func (s *ChainService) OperationNotifier() opfeed.Notifier {
	if s.opNotifier == nil {
		s.opNotifier = &MockOperationNotifier{}
	}
	return s.opNotifier
}

// MockOperationNotifier mocks the operation notifier.
type MockOperationNotifier struct {
	feed *event.Feed
}

// OperationFeed returns an operation feed.
func (mon *MockOperationNotifier) OperationFeed() *event.Feed {
	if mon.feed == nil {
		mon.feed = new(event.Feed)
	}
	return mon.feed
}

// ReceiveBlockInitialSync mocks ReceiveBlockInitialSync method in chain service.
func (s *ChainService) ReceiveBlockInitialSync(ctx context.Context, block interfaces.ReadOnlySignedBeaconBlock, _ [32]byte) error {
	if s.State == nil {
		return ErrNilState
	}
	parentRoot := block.Block().ParentRoot()
	if !bytes.Equal(s.Root, parentRoot[:]) {
		return errors.Errorf("wanted %#x but got %#x", s.Root, block.Block().ParentRoot())
	}
	if err := s.State.SetSlot(block.Block().Slot()); err != nil {
		return err
	}
	s.BlocksReceived = append(s.BlocksReceived, block)
	signingRoot, err := block.Block().HashTreeRoot()
	if err != nil {
		return err
	}
	if s.DB != nil {
		if err := s.DB.SaveBlock(ctx, block); err != nil {
			return err
		}
		logrus.Infof("Saved block with root: %#x at slot %d", signingRoot, block.Block().Slot())
	}
	s.Root = signingRoot[:]
	s.Block = block
	return nil
}

// ReceiveBlockBatch processes blocks in batches from initial-sync.
func (s *ChainService) ReceiveBlockBatch(ctx context.Context, blks []blocks.ROBlock, _ das.AvailabilityStore) error {
	if s.State == nil {
		return ErrNilState
	}
	for _, b := range blks {
		parentRoot := b.Block().ParentRoot()
		if !bytes.Equal(s.Root, parentRoot[:]) {
			return errors.Errorf("wanted %#x but got %#x", s.Root, b.Block().ParentRoot())
		}
		if err := s.State.SetSlot(b.Block().Slot()); err != nil {
			return err
		}
		s.BlocksReceived = append(s.BlocksReceived, b)
		signingRoot, err := b.Block().HashTreeRoot()
		if err != nil {
			return err
		}
		if s.DB != nil {
			if err := s.DB.SaveBlock(ctx, b); err != nil {
				return err
			}
			logrus.Infof("Saved block with root: %#x at slot %d", signingRoot, b.Block().Slot())
		}
		s.Root = signingRoot[:]
		s.Block = b
	}
	return nil
}

// ReceiveBlock mocks ReceiveBlock method in chain service.
func (s *ChainService) ReceiveBlock(ctx context.Context, block interfaces.ReadOnlySignedBeaconBlock, _ [32]byte, _ das.AvailabilityStore) error {
	if s.ReceiveBlockMockErr != nil {
		return s.ReceiveBlockMockErr
	}
	if s.State == nil {
		return ErrNilState
	}
	parentRoot := block.Block().ParentRoot()
	if !bytes.Equal(s.Root, parentRoot[:]) {
		return errors.Errorf("wanted %#x but got %#x", s.Root, block.Block().ParentRoot())
	}
	if err := s.State.SetSlot(block.Block().Slot()); err != nil {
		return err
	}
	s.BlocksReceived = append(s.BlocksReceived, block)
	signingRoot, err := block.Block().HashTreeRoot()
	if err != nil {
		return err
	}
	if s.DB != nil {
		if err := s.DB.SaveBlock(ctx, block); err != nil {
			return err
		}
		logrus.Infof("Saved block with root: %#x at slot %d", signingRoot, block.Block().Slot())
	}
	s.Root = signingRoot[:]
	s.Block = block
	return nil
}

// HeadSlot mocks HeadSlot method in chain service.
func (s *ChainService) HeadSlot() primitives.Slot {
	if s.State == nil {
		return 0
	}
	return s.State.Slot()
}

// HeadRoot mocks HeadRoot method in chain service.
func (s *ChainService) HeadRoot(_ context.Context) ([]byte, error) {
	if len(s.Root) > 0 {
		return s.Root, nil
	}
	return make([]byte, 32), nil
}

// HeadBlock mocks HeadBlock method in chain service.
func (s *ChainService) HeadBlock(context.Context) (interfaces.ReadOnlySignedBeaconBlock, error) {
	return s.Block, nil
}

// HeadState mocks HeadState method in chain service.
func (s *ChainService) HeadState(context.Context) (state.BeaconState, error) {
	return s.State, nil
}

// HeadStateReadOnly mocks HeadStateReadOnly method in chain service.
func (s *ChainService) HeadStateReadOnly(context.Context) (state.ReadOnlyBeaconState, error) {
	return s.State, nil
}

// CurrentFork mocks HeadState method in chain service.
func (s *ChainService) CurrentFork() *ethpb.Fork {
	return s.Fork
}

// FinalizedCheckpt mocks FinalizedCheckpt method in chain service.
func (s *ChainService) FinalizedCheckpt() *ethpb.Checkpoint {
	return s.FinalizedCheckPoint
}

// CurrentJustifiedCheckpt mocks CurrentJustifiedCheckpt method in chain service.
func (s *ChainService) CurrentJustifiedCheckpt() *ethpb.Checkpoint {
	return s.CurrentJustifiedCheckPoint
}

// PreviousJustifiedCheckpt mocks PreviousJustifiedCheckpt method in chain service.
func (s *ChainService) PreviousJustifiedCheckpt() *ethpb.Checkpoint {
	return s.PreviousJustifiedCheckPoint
}

// ReceiveAttestation mocks ReceiveAttestation method in chain service.
func (*ChainService) ReceiveAttestation(_ context.Context, _ *ethpb.Attestation) error {
	return nil
}

// AttestationTargetState mocks AttestationTargetState method in chain service.
func (s *ChainService) AttestationTargetState(_ context.Context, _ *ethpb.Checkpoint) (state.ReadOnlyBeaconState, error) {
	return s.State, nil
}

// HeadValidatorsIndices mocks the same method in the chain service.
func (s *ChainService) HeadValidatorsIndices(ctx context.Context, epoch primitives.Epoch) ([]primitives.ValidatorIndex, error) {
	if s.State == nil {
		return []primitives.ValidatorIndex{}, nil
	}
	return helpers.ActiveValidatorIndices(ctx, s.State, epoch)
}

// HeadETH1Data provides the current ETH1Data of the head state.
func (s *ChainService) HeadETH1Data() *ethpb.Eth1Data {
	return s.ETH1Data
}

// GenesisTime mocks the same method in the chain service.
func (s *ChainService) GenesisTime() time.Time {
	return s.Genesis
}

// GenesisValidatorsRoot mocks the same method in the chain service.
func (s *ChainService) GenesisValidatorsRoot() [32]byte {
	return s.ValidatorsRoot
}

// CurrentSlot mocks the same method in the chain service.
func (s *ChainService) CurrentSlot() primitives.Slot {
	if s.Slot != nil {
		return *s.Slot
	}
	return primitives.Slot(uint64(time.Now().Unix()-s.Genesis.Unix()) / params.BeaconConfig().SecondsPerSlot)
}

// Participation mocks the same method in the chain service.
func (s *ChainService) Participation(_ uint64) *precompute.Balance {
	return s.Balance
}

// IsValidAttestation always returns true.
func (s *ChainService) IsValidAttestation(_ context.Context, _ *ethpb.Attestation) bool {
	return s.ValidAttestation
}

// IsCanonical returns and determines whether a block with the provided root is part of
// the canonical chain.
func (s *ChainService) IsCanonical(_ context.Context, r [32]byte) (bool, error) {
	if s.CanonicalRoots != nil {
		_, ok := s.CanonicalRoots[r]
		return ok, nil
	}
	return true, nil
}

// HasBlock mocks the same method in the chain service.
func (s *ChainService) HasBlock(ctx context.Context, rt [32]byte) bool {
	if s.DB == nil {
		return false
	}
	if s.DB.HasBlock(ctx, rt) {
		return true
	}
	if s.InitSyncBlockRoots == nil {
		return false
	}
	return s.InitSyncBlockRoots[rt]
}

// RecentBlockSlot mocks the same method in the chain service.
func (s *ChainService) RecentBlockSlot([32]byte) (primitives.Slot, error) {
	return s.BlockSlot, nil
}

// HeadGenesisValidatorsRoot mocks HeadGenesisValidatorsRoot method in chain service.
func (*ChainService) HeadGenesisValidatorsRoot() [32]byte {
	return [32]byte{}
}

// VerifyLmdFfgConsistency mocks VerifyLmdFfgConsistency and always returns nil.
func (*ChainService) VerifyLmdFfgConsistency(_ context.Context, a *ethpb.Attestation) error {
	if !bytes.Equal(a.Data.BeaconBlockRoot, a.Data.Target.Root) {
		return errors.New("LMD and FFG miss matched")
	}
	return nil
}

// ChainHeads mocks ChainHeads and always return nil.
func (*ChainService) ChainHeads() ([][32]byte, []primitives.Slot) {
	return [][32]byte{
			bytesutil.ToBytes32(bytesutil.PadTo([]byte("foo"), 32)),
			bytesutil.ToBytes32(bytesutil.PadTo([]byte("bar"), 32)),
		},
		[]primitives.Slot{0, 1}
}

// HeadPublicKeyToValidatorIndex mocks HeadPublicKeyToValidatorIndex and always return 0 and true.
func (*ChainService) HeadPublicKeyToValidatorIndex(_ [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool) {
	return 0, true
}

// HeadValidatorIndexToPublicKey mocks HeadValidatorIndexToPublicKey and always return empty and nil.
func (s *ChainService) HeadValidatorIndexToPublicKey(_ context.Context, _ primitives.ValidatorIndex) ([fieldparams.BLSPubkeyLength]byte, error) {
	return s.PublicKey, nil
}

// HeadSyncCommitteeIndices mocks HeadSyncCommitteeIndices and always return `HeadNextSyncCommitteeIndices`.
func (s *ChainService) HeadSyncCommitteeIndices(_ context.Context, _ primitives.ValidatorIndex, _ primitives.Slot) ([]primitives.CommitteeIndex, error) {
	return s.SyncCommitteeIndices, nil
}

// HeadSyncCommitteePubKeys mocks HeadSyncCommitteePubKeys and always return empty nil.
func (s *ChainService) HeadSyncCommitteePubKeys(_ context.Context, _ primitives.Slot, _ primitives.CommitteeIndex) ([][]byte, error) {
	return s.SyncCommitteePubkeys, nil
}

// HeadSyncCommitteeDomain mocks HeadSyncCommitteeDomain and always return empty nil.
func (s *ChainService) HeadSyncCommitteeDomain(_ context.Context, _ primitives.Slot) ([]byte, error) {
	return s.SyncCommitteeDomain, nil
}

// HeadSyncSelectionProofDomain mocks HeadSyncSelectionProofDomain and always return empty nil.
func (s *ChainService) HeadSyncSelectionProofDomain(_ context.Context, _ primitives.Slot) ([]byte, error) {
	return s.SyncSelectionProofDomain, nil
}

// HeadSyncContributionProofDomain mocks HeadSyncContributionProofDomain and always return empty nil.
func (s *ChainService) HeadSyncContributionProofDomain(_ context.Context, _ primitives.Slot) ([]byte, error) {
	return s.SyncContributionProofDomain, nil
}

// IsOptimistic mocks the same method in the chain service.
func (s *ChainService) IsOptimistic(_ context.Context) (bool, error) {
	return s.Optimistic, nil
}

// InForkchoice mocks the same method in the chain service
func (s *ChainService) InForkchoice(_ [32]byte) bool {
	return !s.NotFinalized
}

// IsOptimisticForRoot mocks the same method in the chain service.
func (s *ChainService) IsOptimisticForRoot(_ context.Context, root [32]byte) (bool, error) {
	s.OptimisticCheckRootReceived = root
	return s.OptimisticRoots[root], nil
}

// UpdateHead mocks the same method in the chain service.
func (s *ChainService) UpdateHead(ctx context.Context, slot primitives.Slot) {
	ojc := &ethpb.Checkpoint{}
	st, root, err := prepareForkchoiceState(ctx, slot, bytesutil.ToBytes32(s.Root), [32]byte{}, [32]byte{}, ojc, ojc)
	if err != nil {
		logrus.WithError(err).Error("could not update head")
	}
	err = s.ForkChoiceStore.InsertNode(ctx, st, root)
	if err != nil {
		logrus.WithError(err).Error("could not insert node to forkchoice")
	}
}

// ReceiveAttesterSlashing mocks the same method in the chain service.
func (*ChainService) ReceiveAttesterSlashing(context.Context, *ethpb.AttesterSlashing) {}

// IsFinalized mocks the same method in the chain service.
func (s *ChainService) IsFinalized(_ context.Context, blockRoot [32]byte) bool {
	return s.FinalizedRoots[blockRoot]
}

// prepareForkchoiceState prepares a beacon state with the given data to mock
// insert into forkchoice
func prepareForkchoiceState(
	_ context.Context,
	slot primitives.Slot,
	blockRoot [32]byte,
	parentRoot [32]byte,
	payloadHash [32]byte,
	justified *ethpb.Checkpoint,
	finalized *ethpb.Checkpoint,
) (state.BeaconState, [32]byte, error) {
	blockHeader := &ethpb.BeaconBlockHeader{
		ParentRoot: parentRoot[:],
	}

	executionHeader := &enginev1.ExecutionPayloadHeader{
		BlockHash: payloadHash[:],
	}

	base := &ethpb.BeaconStateBellatrix{
		Slot:                         slot,
		RandaoMixes:                  make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		BlockRoots:                   make([][]byte, 1),
		CurrentJustifiedCheckpoint:   justified,
		FinalizedCheckpoint:          finalized,
		LatestExecutionPayloadHeader: executionHeader,
		LatestBlockHeader:            blockHeader,
	}

	base.BlockRoots[0] = append(base.BlockRoots[0], blockRoot[:]...)
	st, err := state_native.InitializeFromProtoBellatrix(base)
	return st, blockRoot, err
}

// CachedHeadRoot mocks the same method in the chain service
func (s *ChainService) CachedHeadRoot() [32]byte {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.CachedHeadRoot()
	}
	return [32]byte{}
}

// GetProposerHead mocks the same method in the chain service
func (s *ChainService) GetProposerHead() [32]byte {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.GetProposerHead()
	}
	return [32]byte{}
}

// SetForkChoiceGenesisTime mocks the same method in the chain service
func (s *ChainService) SetForkChoiceGenesisTime(timestamp uint64) {
	if s.ForkChoiceStore != nil {
		s.ForkChoiceStore.SetGenesisTime(timestamp)
	}
}

// ReceivedBlocksLastEpoch mocks the same method in the chain service
func (s *ChainService) ReceivedBlocksLastEpoch() (uint64, error) {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.ReceivedBlocksLastEpoch()
	}
	return 0, nil
}

// HighestReceivedBlockSlot mocks the same method in the chain service
func (s *ChainService) HighestReceivedBlockSlot() primitives.Slot {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.HighestReceivedBlockSlot()
	}
	return 0
}

// InsertNode mocks the same method in the chain service
func (s *ChainService) InsertNode(ctx context.Context, st state.BeaconState, root [32]byte) error {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.InsertNode(ctx, st, root)
	}
	return nil
}

// ForkChoiceDump mocks the same method in the chain service
func (s *ChainService) ForkChoiceDump(ctx context.Context) (*forkchoice2.Dump, error) {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.ForkChoiceDump(ctx)
	}
	return nil, nil
}

// NewSlot mocks the same method in the chain service
func (s *ChainService) NewSlot(ctx context.Context, slot primitives.Slot) error {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.NewSlot(ctx, slot)
	}
	return nil
}

// ProposerBoost mocks the same method in the chain service
func (s *ChainService) ProposerBoost() [32]byte {
	if s.ForkChoiceStore != nil {
		return s.ForkChoiceStore.ProposerBoost()
	}
	return [32]byte{}
}

// FinalizedBlockHash mocks the same method in the chain service
func (*ChainService) FinalizedBlockHash() [32]byte {
	return [32]byte{}
}

// UnrealizedJustifiedPayloadBlockHash mocks the same method in the chain service
func (*ChainService) UnrealizedJustifiedPayloadBlockHash() [32]byte {
	return [32]byte{}
}

// BlockBeingSynced mocks the same method in the chain service
func (c *ChainService) BlockBeingSynced(root [32]byte) bool {
	return root == c.SyncingRoot
}

// ReceiveBlob implements the same method in the chain service
func (c *ChainService) ReceiveBlob(_ context.Context, b blocks.VerifiedROBlob) error {
	c.Blobs = append(c.Blobs, b)
	return nil
}

// TargetRootForEpoch mocks the same method in the chain service
func (c *ChainService) TargetRootForEpoch(_ [32]byte, _ primitives.Epoch) ([32]byte, error) {
	return c.TargetRoot, nil
}
