package statesync

import (
	"context"
	"fmt"
	"strings"
	"time"

	dbm "github.com/cometbft/cometbft-db"

	"github.com/cometbft/cometbft/libs/log"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/light"
	lightprovider "github.com/cometbft/cometbft/light/provider"
	lighthttp "github.com/cometbft/cometbft/light/provider/http"
	lightrpc "github.com/cometbft/cometbft/light/rpc"
	lightdb "github.com/cometbft/cometbft/light/store/db"
	cmtstate "github.com/cometbft/cometbft/proto/tendermint/state"
	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

//go:generate ../scripts/mockery_generate.sh StateProvider

// StateProvider is a provider of trusted state data for bootstrapping a node. This refers
// to the state.State object, not the state machine.
type StateProvider interface {
	// AppHash returns the app hash after the given height has been committed.
	AppHash(ctx context.Context, height uint64) ([]byte, error)
	// Commit returns the commit at the given height.
	Commit(ctx context.Context, height uint64) (*types.Commit, error)
	// State returns a state object at the given height.
	State(ctx context.Context, height uint64) (sm.State, error)
}

// lightClientStateProvider is a state provider using the light client.
type lightClientStateProvider struct {
	cmtsync.Mutex // light.Client is not concurrency-safe
	lc            *light.Client
	version       cmtstate.Version
	initialHeight int64
	providers     map[lightprovider.Provider]string
}

// NewLightClientStateProvider creates a new StateProvider using a light client and RPC clients.
func NewLightClientStateProvider(
	ctx context.Context,
	chainID string,
	version cmtstate.Version,
	initialHeight int64,
	servers []string,
	trustOptions light.TrustOptions,
	logger log.Logger,
) (StateProvider, error) {
	if len(servers) < 2 {
		return nil, fmt.Errorf("at least 2 RPC servers are required, got %v", len(servers))
	}

	providers := make([]lightprovider.Provider, 0, len(servers))
	providerRemotes := make(map[lightprovider.Provider]string)
	for _, server := range servers {
		client, err := rpcClient(server)
		if err != nil {
			return nil, fmt.Errorf("failed to set up RPC client: %w", err)
		}
		provider := lighthttp.NewWithClient(chainID, client)
		providers = append(providers, provider)
		// We store the RPC addresses keyed by provider, so we can find the address of the primary
		// provider used by the light client and use it to fetch consensus parameters.
		providerRemotes[provider] = server
	}

	lc, err := light.NewClient(ctx, chainID, trustOptions, providers[0], providers[1:],
		lightdb.New(dbm.NewMemDB(), ""), light.Logger(logger), light.MaxRetryAttempts(5))
	if err != nil {
		return nil, err
	}
	return &lightClientStateProvider{
		lc:            lc,
		version:       version,
		initialHeight: initialHeight,
		providers:     providerRemotes,
	}, nil
}

// AppHash implements StateProvider.
func (s *lightClientStateProvider) AppHash(ctx context.Context, height uint64) ([]byte, error) {
	s.Lock()
	defer s.Unlock()

	// We have to fetch the next height, which contains the app hash for the previous height.
	header, err := s.lc.VerifyLightBlockAtHeight(ctx, int64(height+1), time.Now())
	if err != nil {
		return nil, err
	}
	// We also try to fetch the blocks at height H and H+2, since we need these
	// when building the state while restoring the snapshot. This avoids the race
	// condition where we try to restore a snapshot before H+2 exists.
	//
	// FIXME This is a hack, since we can't add new methods to the interface without
	// breaking it. We should instead have a Has(ctx, height) method which checks
	// that the state provider has access to the necessary data for the height.
	// We piggyback on AppHash() since it's called when adding snapshots to the pool.
	_, err = s.lc.VerifyLightBlockAtHeight(ctx, int64(height+2), time.Now())
	if err != nil {
		return nil, err
	}
	return header.AppHash, nil
}

// Commit implements StateProvider.
func (s *lightClientStateProvider) Commit(ctx context.Context, height uint64) (*types.Commit, error) {
	s.Lock()
	defer s.Unlock()
	header, err := s.lc.VerifyLightBlockAtHeight(ctx, int64(height), time.Now())
	if err != nil {
		return nil, err
	}
	return header.Commit, nil
}

// State implements StateProvider.
func (s *lightClientStateProvider) State(ctx context.Context, height uint64) (sm.State, error) {
	s.Lock()
	defer s.Unlock()

	state := sm.State{
		ChainID:       s.lc.ChainID(),
		Version:       s.version,
		InitialHeight: s.initialHeight,
	}
	if state.InitialHeight == 0 {
		state.InitialHeight = 1
	}

	// The snapshot height maps onto the state heights as follows:
	//
	// height: last block, i.e. the snapshotted height
	// height+1: current block, i.e. the first block we'll process after the snapshot
	// height+2: next block, i.e. the second block after the snapshot
	//
	// We need to fetch the NextValidators from height+2 because if the application changed
	// the validator set at the snapshot height then this only takes effect at height+2.
	lastLightBlock, err := s.lc.VerifyLightBlockAtHeight(ctx, int64(height), time.Now())
	if err != nil {
		return sm.State{}, err
	}
	currentLightBlock, err := s.lc.VerifyLightBlockAtHeight(ctx, int64(height+1), time.Now())
	if err != nil {
		return sm.State{}, err
	}
	nextLightBlock, err := s.lc.VerifyLightBlockAtHeight(ctx, int64(height+2), time.Now())
	if err != nil {
		return sm.State{}, err
	}

	state.Version = cmtstate.Version{
		Consensus: currentLightBlock.Version,
		Software:  version.TMCoreSemVer,
	}
	state.LastBlockHeight = lastLightBlock.Height
	state.LastBlockTime = lastLightBlock.Time
	state.LastBlockID = lastLightBlock.Commit.BlockID
	state.AppHash = currentLightBlock.AppHash
	state.LastResultsHash = currentLightBlock.LastResultsHash
	state.LastValidators = lastLightBlock.ValidatorSet
	state.Validators = currentLightBlock.ValidatorSet
	state.NextValidators = nextLightBlock.ValidatorSet
	state.LastHeightValidatorsChanged = nextLightBlock.Height

	// We'll also need to fetch consensus params via RPC, using light client verification.
	primaryURL, ok := s.providers[s.lc.Primary()]
	if !ok || primaryURL == "" {
		return sm.State{}, fmt.Errorf("could not find address for primary light client provider")
	}
	primaryRPC, err := rpcClient(primaryURL)
	if err != nil {
		return sm.State{}, fmt.Errorf("unable to create RPC client: %w", err)
	}
	rpcclient := lightrpc.NewClient(primaryRPC, s.lc)
	result, err := rpcclient.ConsensusParams(ctx, &currentLightBlock.Height)
	if err != nil {
		return sm.State{}, fmt.Errorf("unable to fetch consensus parameters for height %v: %w",
			nextLightBlock.Height, err)
	}
	state.ConsensusParams = result.ConsensusParams
	state.LastHeightConsensusParamsChanged = currentLightBlock.Height

	return state, nil
}

// rpcClient sets up a new RPC client
func rpcClient(server string) (*rpchttp.HTTP, error) {
	if !strings.Contains(server, "://") {
		server = "http://" + server
	}
	c, err := rpchttp.New(server, "/websocket")
	if err != nil {
		return nil, err
	}
	return c, nil
}
