package blockchain

import (
	"bytes"
	"context"
	"time"

	"github.com/pkg/errors"
	"go.opencensus.io/trace"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	f "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/forkchoice"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ChainInfoFetcher defines a common interface for methods in blockchain service which
// directly retrieve chain info related data.
type ChainInfoFetcher interface {
	HeadFetcher
	FinalizationFetcher
	CanonicalFetcher
	ForkFetcher
	HeadDomainFetcher
	ForkchoiceFetcher
}

// ForkchoiceFetcher defines a common interface for methods that access directly
// forkchoice information. These typically require a lock and external callers
// are requested to call methods within this blockchain package that takes care
// of locking forkchoice
type ForkchoiceFetcher interface {
	Ancestor(context.Context, []byte, primitives.Slot) ([]byte, error)
	CachedHeadRoot() [32]byte
	GetProposerHead() [32]byte
	SetForkChoiceGenesisTime(uint64)
	UpdateHead(context.Context, primitives.Slot)
	HighestReceivedBlockSlot() primitives.Slot
	ReceivedBlocksLastEpoch() (uint64, error)
	InsertNode(context.Context, state.BeaconState, [32]byte) error
	ForkChoiceDump(context.Context) (*forkchoice.Dump, error)
	NewSlot(context.Context, primitives.Slot) error
	ProposerBoost() [32]byte
}

// TimeFetcher retrieves the Ethereum consensus data that's related to time.
type TimeFetcher interface {
	GenesisTime() time.Time
	CurrentSlot() primitives.Slot
}

// GenesisFetcher retrieves the Ethereum consensus data related to its genesis.
type GenesisFetcher interface {
	GenesisValidatorsRoot() [32]byte
}

// HeadFetcher defines a common interface for methods in blockchain service which
// directly retrieve head related data.
type HeadFetcher interface {
	HeadSlot() primitives.Slot
	HeadRoot(ctx context.Context) ([]byte, error)
	HeadBlock(ctx context.Context) (interfaces.ReadOnlySignedBeaconBlock, error)
	HeadState(ctx context.Context) (state.BeaconState, error)
	HeadStateReadOnly(ctx context.Context) (state.ReadOnlyBeaconState, error)
	HeadValidatorsIndices(ctx context.Context, epoch primitives.Epoch) ([]primitives.ValidatorIndex, error)
	HeadGenesisValidatorsRoot() [32]byte
	HeadETH1Data() *ethpb.Eth1Data
	HeadPublicKeyToValidatorIndex(pubKey [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool)
	HeadValidatorIndexToPublicKey(ctx context.Context, index primitives.ValidatorIndex) ([fieldparams.BLSPubkeyLength]byte, error)
	ChainHeads() ([][32]byte, []primitives.Slot)
	TargetRootForEpoch([32]byte, primitives.Epoch) ([32]byte, error)
	HeadSyncCommitteeFetcher
	HeadDomainFetcher
}

// ForkFetcher retrieves the current fork information of the Ethereum beacon chain.
type ForkFetcher interface {
	CurrentFork() *ethpb.Fork
	GenesisFetcher
	TimeFetcher
}

// TemporalOracle is like ForkFetcher minus CurrentFork()
type TemporalOracle interface {
	GenesisFetcher
	TimeFetcher
}

// CanonicalFetcher retrieves the current chain's canonical information.
type CanonicalFetcher interface {
	IsCanonical(ctx context.Context, blockRoot [32]byte) (bool, error)
}

// FinalizationFetcher defines a common interface for methods in blockchain service which
// directly retrieve finalization and justification related data.
type FinalizationFetcher interface {
	FinalizedCheckpt() *ethpb.Checkpoint
	CurrentJustifiedCheckpt() *ethpb.Checkpoint
	PreviousJustifiedCheckpt() *ethpb.Checkpoint
	UnrealizedJustifiedPayloadBlockHash() [32]byte
	FinalizedBlockHash() [32]byte
	InForkchoice([32]byte) bool
	IsFinalized(ctx context.Context, blockRoot [32]byte) bool
}

// OptimisticModeFetcher retrieves information about optimistic status of the node.
type OptimisticModeFetcher interface {
	IsOptimistic(ctx context.Context) (bool, error)
	IsOptimisticForRoot(ctx context.Context, root [32]byte) (bool, error)
}

// FinalizedCheckpt returns the latest finalized checkpoint from chain store.
func (s *Service) FinalizedCheckpt() *ethpb.Checkpoint {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	cp := s.cfg.ForkChoiceStore.FinalizedCheckpoint()
	return &ethpb.Checkpoint{Epoch: cp.Epoch, Root: bytesutil.SafeCopyBytes(cp.Root[:])}
}

// PreviousJustifiedCheckpt returns the current justified checkpoint from chain store.
func (s *Service) PreviousJustifiedCheckpt() *ethpb.Checkpoint {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	cp := s.cfg.ForkChoiceStore.PreviousJustifiedCheckpoint()
	return &ethpb.Checkpoint{Epoch: cp.Epoch, Root: bytesutil.SafeCopyBytes(cp.Root[:])}
}

// CurrentJustifiedCheckpt returns the current justified checkpoint from chain store.
func (s *Service) CurrentJustifiedCheckpt() *ethpb.Checkpoint {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	cp := s.cfg.ForkChoiceStore.JustifiedCheckpoint()
	return &ethpb.Checkpoint{Epoch: cp.Epoch, Root: bytesutil.SafeCopyBytes(cp.Root[:])}
}

// HeadSlot returns the slot of the head of the chain.
func (s *Service) HeadSlot() primitives.Slot {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return 0
	}

	return s.headSlot()
}

// HeadRoot returns the root of the head of the chain.
func (s *Service) HeadRoot(ctx context.Context) ([]byte, error) {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if s.head != nil && s.head.root != params.BeaconConfig().ZeroHash {
		return bytesutil.SafeCopyBytes(s.head.root[:]), nil
	}

	b, err := s.cfg.BeaconDB.HeadBlock(ctx)
	if err != nil {
		return nil, err
	}
	if b == nil || b.IsNil() {
		return params.BeaconConfig().ZeroHash[:], nil
	}

	r, err := b.Block().HashTreeRoot()
	if err != nil {
		return nil, err
	}

	return r[:], nil
}

// HeadBlock returns the head block of the chain.
// If the head is nil from service struct,
// it will attempt to get the head block from DB.
func (s *Service) HeadBlock(ctx context.Context) (interfaces.ReadOnlySignedBeaconBlock, error) {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if s.hasHeadState() {
		return s.headBlock()
	}

	return s.cfg.BeaconDB.HeadBlock(ctx)
}

// HeadState returns the head state of the chain.
// If the head is nil from service struct,
// it will attempt to get the head state from DB.
func (s *Service) HeadState(ctx context.Context) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "blockChain.HeadState")
	defer span.End()
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	ok := s.hasHeadState()
	span.AddAttributes(trace.BoolAttribute("cache_hit", ok))

	if ok {
		return s.headState(ctx), nil
	}

	return s.cfg.StateGen.StateByRoot(ctx, s.headRoot())
}

// HeadStateReadOnly returns the read only head state of the chain.
// If the head is nil from service struct, it will attempt to get the
// head state from DB. Any callers of this method MUST only use the
// state instance to read fields from the state. Any type assertions back
// to the concrete type and subsequent use of it could lead to corruption
// of the state.
func (s *Service) HeadStateReadOnly(ctx context.Context) (state.ReadOnlyBeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "blockChain.HeadStateReadOnly")
	defer span.End()
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	ok := s.hasHeadState()
	span.AddAttributes(trace.BoolAttribute("cache_hit", ok))

	if ok {
		return s.headStateReadOnly(ctx), nil
	}

	return s.cfg.StateGen.StateByRoot(ctx, s.headRoot())
}

// HeadValidatorsIndices returns a list of active validator indices from the head view of a given epoch.
func (s *Service) HeadValidatorsIndices(ctx context.Context, epoch primitives.Epoch) ([]primitives.ValidatorIndex, error) {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return []primitives.ValidatorIndex{}, nil
	}
	return helpers.ActiveValidatorIndices(ctx, s.headState(ctx), epoch)
}

// HeadGenesisValidatorsRoot returns genesis validators root of the head state.
func (s *Service) HeadGenesisValidatorsRoot() [32]byte {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return [32]byte{}
	}

	return s.headGenesisValidatorsRoot()
}

// HeadETH1Data returns the eth1data of the current head state.
func (s *Service) HeadETH1Data() *ethpb.Eth1Data {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return &ethpb.Eth1Data{}
	}
	return s.head.state.Eth1Data()
}

// GenesisTime returns the genesis time of beacon chain.
func (s *Service) GenesisTime() time.Time {
	return s.genesisTime
}

// GenesisValidatorsRoot returns the genesis validators
// root of the chain.
func (s *Service) GenesisValidatorsRoot() [32]byte {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return [32]byte{}
	}
	return bytesutil.ToBytes32(s.head.state.GenesisValidatorsRoot())
}

// CurrentFork retrieves the latest fork information of the beacon chain.
func (s *Service) CurrentFork() *ethpb.Fork {
	s.headLock.RLock()
	defer s.headLock.RUnlock()

	if !s.hasHeadState() {
		return &ethpb.Fork{
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
		}
	}
	return s.head.state.Fork()
}

// IsCanonical returns true if the input block root is part of the canonical chain.
func (s *Service) IsCanonical(ctx context.Context, blockRoot [32]byte) (bool, error) {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	// If the block has not been finalized, check fork choice store to see if the block is canonical
	if s.cfg.ForkChoiceStore.HasNode(blockRoot) {
		return s.cfg.ForkChoiceStore.IsCanonical(blockRoot), nil
	}

	// If the block has been finalized, the block will always be part of the canonical chain.
	return s.cfg.BeaconDB.IsFinalizedBlock(ctx, blockRoot), nil
}

// HeadPublicKeyToValidatorIndex returns the validator index of the `pubkey` in current head state.
func (s *Service) HeadPublicKeyToValidatorIndex(pubKey [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool) {
	s.headLock.RLock()
	defer s.headLock.RUnlock()
	if !s.hasHeadState() {
		return 0, false
	}
	return s.headValidatorIndexAtPubkey(pubKey)
}

// HeadValidatorIndexToPublicKey returns the pubkey of the validator `index`  in current head state.
func (s *Service) HeadValidatorIndexToPublicKey(_ context.Context, index primitives.ValidatorIndex) ([fieldparams.BLSPubkeyLength]byte, error) {
	s.headLock.RLock()
	defer s.headLock.RUnlock()
	if !s.hasHeadState() {
		return [fieldparams.BLSPubkeyLength]byte{}, nil
	}
	v, err := s.headValidatorAtIndex(index)
	if err != nil {
		return [fieldparams.BLSPubkeyLength]byte{}, err
	}
	return v.PublicKey(), nil
}

// ForkChoicer returns the forkchoice interface.
func (s *Service) ForkChoicer() f.ForkChoicer {
	return s.cfg.ForkChoiceStore
}

// IsOptimistic returns true if the current head is optimistic.
func (s *Service) IsOptimistic(_ context.Context) (bool, error) {
	if slots.ToEpoch(s.CurrentSlot()) < params.BeaconConfig().BellatrixForkEpoch {
		return false, nil
	}
	s.headLock.RLock()
	if s.head == nil {
		s.headLock.RUnlock()
		return false, ErrNilHead
	}
	headRoot := s.head.root
	headSlot := s.head.slot
	headOptimistic := s.head.optimistic
	s.headLock.RUnlock()
	// we trust the head package for recent head slots, otherwise fallback to forkchoice
	if headSlot+2 >= s.CurrentSlot() {
		return headOptimistic, nil
	}

	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	optimistic, err := s.cfg.ForkChoiceStore.IsOptimistic(headRoot)
	if err == nil {
		return optimistic, nil
	}
	if !errors.Is(err, doublylinkedtree.ErrNilNode) {
		return true, err
	}
	// If fockchoice does not have the headroot, then the node is considered
	// optimistic
	return true, nil
}

// IsFinalized returns true if the input root is finalized.
// It first checks latest finalized root then checks finalized root index in DB.
func (s *Service) IsFinalized(ctx context.Context, root [32]byte) bool {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	if s.cfg.ForkChoiceStore.FinalizedCheckpoint().Root == root {
		return true
	}
	// If node exists in our store, then it is not
	// finalized.
	if s.cfg.ForkChoiceStore.HasNode(root) {
		return false
	}
	return s.cfg.BeaconDB.IsFinalizedBlock(ctx, root)
}

// InForkchoice returns true if the given root is found in forkchoice
// This in particular means that the blockroot is a descendant of the
// finalized checkpoint
func (s *Service) InForkchoice(root [32]byte) bool {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	return s.cfg.ForkChoiceStore.HasNode(root)
}

// IsViableForCheckpoint returns whether the given checkpoint is a checkpoint in any
// chain known to forkchoice
func (s *Service) IsViableForCheckpoint(cp *forkchoicetypes.Checkpoint) (bool, error) {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	return s.cfg.ForkChoiceStore.IsViableForCheckpoint(cp)
}

// IsOptimisticForRoot takes the root as argument instead of the current head
// and returns true if it is optimistic.
func (s *Service) IsOptimisticForRoot(ctx context.Context, root [32]byte) (bool, error) {
	s.cfg.ForkChoiceStore.RLock()
	optimistic, err := s.cfg.ForkChoiceStore.IsOptimistic(root)
	s.cfg.ForkChoiceStore.RUnlock()
	if err == nil {
		return optimistic, nil
	}
	if !errors.Is(err, doublylinkedtree.ErrNilNode) {
		return false, err
	}
	// if the requested root is the headroot and the root is not found in
	// forkchoice, the node should respond that it is optimistic
	headRoot, err := s.HeadRoot(ctx)
	if err != nil {
		return true, err
	}
	if bytes.Equal(headRoot, root[:]) {
		return true, nil
	}

	ss, err := s.cfg.BeaconDB.StateSummary(ctx, root)
	if err != nil {
		return false, err
	}

	if ss == nil {
		ss, err = s.recoverStateSummary(ctx, root)
		if err != nil {
			return true, err
		}
	}
	validatedCheckpoint, err := s.cfg.BeaconDB.LastValidatedCheckpoint(ctx)
	if err != nil {
		return false, err
	}
	if slots.ToEpoch(ss.Slot) > validatedCheckpoint.Epoch {
		return true, nil
	}

	// Historical non-canonical blocks here are returned as optimistic for safety.
	isCanonical, err := s.IsCanonical(ctx, root)
	if err != nil {
		return false, err
	}

	if slots.ToEpoch(ss.Slot)+1 < validatedCheckpoint.Epoch {
		return !isCanonical, nil
	}

	// Checkpoint root could be zeros before the first finalized epoch. Use genesis root if the case.
	lastValidated, err := s.cfg.BeaconDB.StateSummary(ctx, s.ensureRootNotZeros(bytesutil.ToBytes32(validatedCheckpoint.Root)))
	if err != nil {
		return false, err
	}
	if lastValidated == nil {
		lastValidated, err = s.recoverStateSummary(ctx, root)
		if err != nil {
			return false, err
		}
	}

	if ss.Slot > lastValidated.Slot {
		return true, nil
	}
	return !isCanonical, nil
}

// TargetRootForEpoch wraps the corresponding method in forkchoice
func (s *Service) TargetRootForEpoch(root [32]byte, epoch primitives.Epoch) ([32]byte, error) {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	return s.cfg.ForkChoiceStore.TargetRootForEpoch(root, epoch)
}

// Ancestor returns the block root of an ancestry block from the input block root.
//
// Spec pseudocode definition:
//
//	def get_ancestor(store: Store, root: Root, slot: Slot) -> Root:
//	 block = store.blocks[root]
//	 if block.slot > slot:
//	     return get_ancestor(store, block.parent_root, slot)
//	 elif block.slot == slot:
//	     return root
//	 else:
//	     # root is older than queried slot, thus a skip slot. Return most recent root prior to slot
//	     return root
func (s *Service) Ancestor(ctx context.Context, root []byte, slot primitives.Slot) ([]byte, error) {
	ctx, span := trace.StartSpan(ctx, "blockChain.ancestor")
	defer span.End()

	r := bytesutil.ToBytes32(root)
	// Get ancestor root from fork choice store instead of recursively looking up blocks in DB.
	// This is most optimal outcome.
	s.cfg.ForkChoiceStore.RLock()
	ar, err := s.cfg.ForkChoiceStore.AncestorRoot(ctx, r, slot)
	s.cfg.ForkChoiceStore.RUnlock()
	if err != nil {
		// Try getting ancestor root from DB when failed to retrieve from fork choice store.
		// This is the second line of defense for retrieving ancestor root.
		ar, err = s.ancestorByDB(ctx, r, slot)
		if err != nil {
			return nil, err
		}
	}

	return ar[:], nil
}

// SetOptimisticToInvalid wraps the corresponding method in forkchoice
func (s *Service) SetOptimisticToInvalid(ctx context.Context, root, parent, lvh [32]byte) ([][32]byte, error) {
	s.cfg.ForkChoiceStore.Lock()
	defer s.cfg.ForkChoiceStore.Unlock()
	return s.cfg.ForkChoiceStore.SetOptimisticToInvalid(ctx, root, parent, lvh)
}

// SetGenesisTime sets the genesis time of beacon chain.
func (s *Service) SetGenesisTime(t time.Time) {
	s.genesisTime = t
}

func (s *Service) recoverStateSummary(ctx context.Context, blockRoot [32]byte) (*ethpb.StateSummary, error) {
	if s.cfg.BeaconDB.HasBlock(ctx, blockRoot) {
		b, err := s.cfg.BeaconDB.Block(ctx, blockRoot)
		if err != nil {
			return nil, err
		}
		summary := &ethpb.StateSummary{Slot: b.Block().Slot(), Root: blockRoot[:]}
		if err := s.cfg.BeaconDB.SaveStateSummary(ctx, summary); err != nil {
			return nil, err
		}
		return summary, nil
	}
	return nil, errBlockDoesNotExist
}

// BlockBeingSynced returns whether the block with the given root is currently being synced
func (s *Service) BlockBeingSynced(root [32]byte) bool {
	return s.blockBeingSynced.isSyncing(root)
}

// RecentBlockSlot returns block slot form fork choice store
func (s *Service) RecentBlockSlot(root [32]byte) (primitives.Slot, error) {
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	return s.cfg.ForkChoiceStore.Slot(root)
}

// inRegularSync applies the following heuristics to decide if the node is in
// regular sync mode vs init sync mode using only forkchoice.
// It checks that the highest received block is behind the current time by at least 2 epochs
// and that it was imported at least one epoch late if both of these
// tests pass then the node is in init sync. The caller of this function MUST
// have a lock on forkchoice
func (s *Service) inRegularSync() bool {
	currentSlot := s.CurrentSlot()
	fc := s.cfg.ForkChoiceStore
	if currentSlot-fc.HighestReceivedBlockSlot() < 2*params.BeaconConfig().SlotsPerEpoch {
		return true
	}
	return fc.HighestReceivedBlockDelay() < params.BeaconConfig().SlotsPerEpoch
}
