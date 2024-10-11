package verification

import (
	"context"
	"sync"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/kzg"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// Forkchoicer represents the forkchoice methods that the verifiers need.
// Note that forkchoice is used here in a lock-free fashion, assuming that a version of forkchoice
// is given that internally handles the details of locking the underlying store.
type Forkchoicer interface {
	FinalizedCheckpoint() *forkchoicetypes.Checkpoint
	HasNode([32]byte) bool
	IsCanonical(root [32]byte) bool
	Slot([32]byte) (primitives.Slot, error)
	TargetRootForEpoch([32]byte, primitives.Epoch) ([32]byte, error)
}

// StateByRooter describes a stategen-ish type that can produce arbitrary states by their root
type StateByRooter interface {
	StateByRoot(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error)
}

// sharedResources provides access to resources that are required by different verification types.
// for example, sidecar verification and block verification share the block signature verification cache.
type sharedResources struct {
	clock *startup.Clock
	fc    Forkchoicer
	sc    SignatureCache
	pc    ProposerCache
	sr    StateByRooter
}

// Initializer is used to create different Verifiers.
// Verifiers require access to stateful data structures, like caches,
// and it is Initializer's job to provides access to those.
type Initializer struct {
	shared *sharedResources
}

// NewBlobVerifier creates a BlobVerifier for a single blob, with the given set of requirements.
func (ini *Initializer) NewBlobVerifier(b blocks.ROBlob, reqs []Requirement) *ROBlobVerifier {
	return &ROBlobVerifier{
		sharedResources:      ini.shared,
		blob:                 b,
		results:              newResults(reqs...),
		verifyBlobCommitment: kzg.Verify,
	}
}

// InitializerWaiter provides an Initializer once all dependent resources are ready
// via the WaitForInitializer method.
type InitializerWaiter struct {
	sync.RWMutex
	ready bool
	cw    startup.ClockWaiter
	ini   *Initializer
}

// NewInitializerWaiter creates an InitializerWaiter which can be used to obtain an Initializer once async dependencies are ready.
func NewInitializerWaiter(cw startup.ClockWaiter, fc Forkchoicer, sr StateByRooter) *InitializerWaiter {
	pc := newPropCache()
	// signature cache is initialized in WaitForInitializer, since we need the genesis validators root, which can be obtained from startup.Clock.
	shared := &sharedResources{
		fc: fc,
		pc: pc,
		sr: sr,
	}
	return &InitializerWaiter{cw: cw, ini: &Initializer{shared: shared}}
}

// WaitForInitializer ensures that asynchronous initialization of the shared resources the initializer
// depends on has completed before the underlying Initializer is accessible by client code.
func (w *InitializerWaiter) WaitForInitializer(ctx context.Context) (*Initializer, error) {
	if err := w.waitForReady(ctx); err != nil {
		return nil, err
	}
	// We wait until this point to initialize the signature cache because here we have access to the genesis validator root.
	vr := w.ini.shared.clock.GenesisValidatorsRoot()
	sc := newSigCache(vr[:], DefaultSignatureCacheSize)
	w.ini.shared.sc = sc
	return w.ini, nil
}

func (w *InitializerWaiter) waitForReady(ctx context.Context) error {
	w.Lock()
	defer w.Unlock()
	if w.ready {
		return nil
	}

	clock, err := w.cw.WaitForClock(ctx)
	if err != nil {
		return err
	}
	w.ini.shared.clock = clock
	w.ready = true
	return nil
}
