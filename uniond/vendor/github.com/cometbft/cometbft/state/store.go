package state

import (
	"encoding/binary"
	"errors"
	"fmt"
	"time"

	"github.com/cosmos/gogoproto/proto"
	"github.com/go-kit/kit/metrics"
	"github.com/google/orderedcode"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtstate "github.com/cometbft/cometbft/api/cometbft/state/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtos "github.com/cometbft/cometbft/internal/os"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	"github.com/cometbft/cometbft/types"
)

const (
	// persist validators every valSetCheckpointInterval blocks to avoid
	// LoadValidators taking too much time.
	// https://github.com/tendermint/tendermint/pull/3438
	// 100000 results in ~ 100ms to get 100 validators (see BenchmarkLoadValidators).
	valSetCheckpointInterval = 100000
)

var (
	ErrKeyNotFound        = errors.New("key not found")
	ErrInvalidHeightValue = errors.New("invalid height value")
)

// ------------------------------------------------------------------------.
type KeyLayout interface {
	CalcValidatorsKey(height int64) []byte

	CalcConsensusParamsKey(height int64) []byte

	CalcABCIResponsesKey(height int64) []byte
}

type v1LegacyLayout struct{}

// CalcABCIResponsesKey implements StateKeyLayout.
func (v1LegacyLayout) CalcABCIResponsesKey(height int64) []byte {
	return []byte(fmt.Sprintf("abciResponsesKey:%v", height))
}

// store.StoreOptions.DBKeyLayout.calcConsensusParamsKey implements StateKeyLayout.
func (v1LegacyLayout) CalcConsensusParamsKey(height int64) []byte {
	return []byte(fmt.Sprintf("consensusParamsKey:%v", height))
}

// store.StoreOptions.DBKeyLayout.CalcValidatorsKey implements StateKeyLayout.
func (v1LegacyLayout) CalcValidatorsKey(height int64) []byte {
	return []byte(fmt.Sprintf("validatorsKey:%v", height))
}

var _ KeyLayout = (*v1LegacyLayout)(nil)

// ----------------------

var (
	lastABCIResponseKey              = []byte("lastABCIResponseKey") // DEPRECATED
	lastABCIResponsesRetainHeightKey = []byte("lastABCIResponsesRetainHeight")
	offlineStateSyncHeight           = []byte("offlineStateSyncHeightKey")
)

var (
	// prefixes must be unique across all db's.
	prefixValidators      = int64(6)
	prefixConsensusParams = int64(7)
	prefixABCIResponses   = int64(8)
)

type v2Layout struct{}

func (v2Layout) encodeKey(prefix, height int64) []byte {
	res, err := orderedcode.Append(nil, prefix, height)
	if err != nil {
		panic(err)
	}
	return res
}

// CalcABCIResponsesKey implements StateKeyLayout.
func (v2l v2Layout) CalcABCIResponsesKey(height int64) []byte {
	return v2l.encodeKey(prefixABCIResponses, height)
}

// CalcConsensusParamsKey implements StateKeyLayout.
func (v2l v2Layout) CalcConsensusParamsKey(height int64) []byte {
	return v2l.encodeKey(prefixConsensusParams, height)
}

// CalcValidatorsKey implements StateKeyLayout.
func (v2l v2Layout) CalcValidatorsKey(height int64) []byte {
	return v2l.encodeKey(prefixValidators, height)
}

var _ KeyLayout = (*v2Layout)(nil)

//go:generate ../scripts/mockery_generate.sh Store

// Store defines the state store interface
//
// It is used to retrieve current state and save and load ABCI responses,
// validators and consensus parameters.
type Store interface {
	// LoadFromDBOrGenesisFile loads the most recent state.
	// If the chain is new it will use the genesis file from the provided genesis file path as the current state.
	LoadFromDBOrGenesisFile(filepath string) (State, error)
	// LoadFromDBOrGenesisDoc loads the most recent state.
	// If the chain is new it will use the genesis doc as the current state.
	LoadFromDBOrGenesisDoc(doc *types.GenesisDoc) (State, error)
	// Load loads the current state of the blockchain
	Load() (State, error)
	// LoadValidators loads the validator set at a given height
	LoadValidators(height int64) (*types.ValidatorSet, error)
	// LoadFinalizeBlockResponse loads the abciResponse for a given height
	LoadFinalizeBlockResponse(height int64) (*abci.FinalizeBlockResponse, error)
	// LoadLastFinalizeBlockResponse loads the last abciResponse for a given height
	LoadLastFinalizeBlockResponse(height int64) (*abci.FinalizeBlockResponse, error)
	// LoadConsensusParams loads the consensus params for a given height
	LoadConsensusParams(height int64) (types.ConsensusParams, error)
	// Save overwrites the previous state with the updated one
	Save(state State) error
	// SaveFinalizeBlockResponse saves ABCIResponses for a given height
	SaveFinalizeBlockResponse(height int64, res *abci.FinalizeBlockResponse) error
	// Bootstrap is used for bootstrapping state when not starting from a initial height.
	Bootstrap(state State) error
	// PruneStates takes the height from which to start pruning and which height stop at
	PruneStates(fromHeight, toHeight, evidenceThresholdHeight int64, previouslyPrunedStates uint64) (uint64, error)
	// PruneABCIResponses will prune all ABCI responses below the given height.
	PruneABCIResponses(targetRetainHeight int64, forceCompact bool) (int64, int64, error)
	// SaveApplicationRetainHeight persists the application retain height from the application
	SaveApplicationRetainHeight(height int64) error
	// GetApplicationRetainHeight returns the retain height set by the application
	GetApplicationRetainHeight() (int64, error)
	// SaveCompanionBlockRetainHeight saves the retain height set by the data companion
	SaveCompanionBlockRetainHeight(height int64) error
	// GetCompanionBlockRetainHeight returns the retain height set by the data companion
	GetCompanionBlockRetainHeight() (int64, error)
	// SaveABCIResRetainHeight persists the retain height for ABCI results set by the data companion
	SaveABCIResRetainHeight(height int64) error
	// GetABCIResRetainHeight returns the last saved retain height for ABCI results set by the data companion
	GetABCIResRetainHeight() (int64, error)
	// Saves the height at which the store is bootstrapped after out of band statesync
	SetOfflineStateSyncHeight(height int64) error
	// Gets the height at which the store is bootstrapped after out of band statesync
	GetOfflineStateSyncHeight() (int64, error)
	// Close closes the connection with the database
	Close() error
}

// dbStore wraps a db (github.com/cometbft/cometbft-db).
type dbStore struct {
	db dbm.DB

	DBKeyLayout KeyLayout

	StoreOptions
}

type StoreOptions struct {
	// DiscardABCIResponses determines whether or not the store
	// retains all ABCIResponses. If DiscardABCIResponses is enabled,
	// the store will maintain only the response object from the latest
	// height.
	DiscardABCIResponses bool

	Compact bool

	CompactionInterval int64

	// Metrics defines the metrics collector to use for the state store.
	// if none is specified then a NopMetrics collector is used.
	Metrics *Metrics

	Logger log.Logger

	DBKeyLayout string
}

var _ Store = (*dbStore)(nil)

func IsEmpty(store dbStore) (bool, error) {
	state, err := store.Load()
	if err != nil {
		return false, err
	}
	return state.IsEmpty(), nil
}

func setDBKeyLayout(store *dbStore, dbKeyLayoutVersion string) string {
	empty, _ := IsEmpty(*store)
	if !empty {
		version, err := store.db.Get([]byte("version"))
		if err != nil {
			// WARN: This is because currently cometBFT DB does not return an error if the key does not exist
			// If this behavior changes we need to account for that.
			panic(err)
		}
		if len(version) != 0 {
			dbKeyLayoutVersion = string(version)
		}
	}

	switch dbKeyLayoutVersion {
	case "v1", "":
		store.DBKeyLayout = &v1LegacyLayout{}
		dbKeyLayoutVersion = "v1"
	case "v2":
		store.DBKeyLayout = &v2Layout{}
		dbKeyLayoutVersion = "v2"
	default:
		panic("Unknown version. Expected v1 or v2, given " + dbKeyLayoutVersion)
	}

	if err := store.db.SetSync([]byte("version"), []byte(dbKeyLayoutVersion)); err != nil {
		panic(err)
	}
	return dbKeyLayoutVersion
}

// NewStore creates the dbStore of the state pkg.
func NewStore(db dbm.DB, options StoreOptions) Store {
	if options.Metrics == nil {
		options.Metrics = NopMetrics()
	}

	store := dbStore{
		db:           db,
		StoreOptions: options,
	}

	dbKeyLayoutVersion := setDBKeyLayout(&store, options.DBKeyLayout)

	if options.Logger != nil {
		options.Logger.Info("State store key layout version ", "version", "v"+dbKeyLayoutVersion)
	}

	return store
}

// LoadStateFromDBOrGenesisFile loads the most recent state from the database,
// or creates a new one from the given genesisFilePath.
func (store dbStore) LoadFromDBOrGenesisFile(genesisFilePath string) (State, error) {
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_from_db_or_genesis_file"), time.Now())()
	state, err := store.Load()
	if err != nil {
		return State{}, err
	}
	if state.IsEmpty() {
		var err error
		state, err = MakeGenesisStateFromFile(genesisFilePath)
		if err != nil {
			return state, err
		}
	}

	return state, nil
}

// LoadStateFromDBOrGenesisDoc loads the most recent state from the database,
// or creates a new one from the given genesisDoc.
func (store dbStore) LoadFromDBOrGenesisDoc(genesisDoc *types.GenesisDoc) (State, error) {
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_from_db_or_genesis_doc"), time.Now())()
	state, err := store.Load()
	if err != nil {
		return State{}, err
	}

	if state.IsEmpty() {
		var err error
		state, err = MakeGenesisState(genesisDoc)
		if err != nil {
			return state, err
		}
	}

	return state, nil
}

// LoadState loads the State from the database.
func (store dbStore) Load() (State, error) {
	return store.loadState(stateKey)
}

func (store dbStore) loadState(key []byte) (state State, err error) {
	start := time.Now()
	buf, err := store.db.Get(key)
	if err != nil {
		return state, err
	}

	addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load"), start)()

	if len(buf) == 0 {
		return state, nil
	}

	sp := new(cmtstate.State)

	err = proto.Unmarshal(buf, sp)
	if err != nil {
		// DATA HAS BEEN CORRUPTED OR THE SPEC HAS CHANGED
		cmtos.Exit(fmt.Sprintf(`LoadState: Data has been corrupted or its spec has changed:
		%v\n`, err))
	}

	sm, err := FromProto(sp)
	if err != nil {
		return state, err
	}
	return *sm, nil
}

// Save persists the State, the ValidatorsInfo, and the ConsensusParamsInfo to the database.
// This flushes the writes (e.g. calls SetSync).
func (store dbStore) Save(state State) error {
	return store.save(state, stateKey)
}

func (store dbStore) save(state State, key []byte) error {
	start := time.Now()

	batch := store.db.NewBatch()
	defer func(batch dbm.Batch) {
		err := batch.Close()
		if err != nil {
			panic(err)
		}
	}(batch)
	nextHeight := state.LastBlockHeight + 1
	// If first block, save validators for the block.
	if nextHeight == 1 {
		nextHeight = state.InitialHeight
		// This extra logic due to validator set changes being delayed 1 block.
		// It may get overwritten due to InitChain validator updates.
		if err := store.saveValidatorsInfo(nextHeight, nextHeight, state.Validators, batch); err != nil {
			return err
		}
	}
	// Save next validators.
	if err := store.saveValidatorsInfo(nextHeight+1, state.LastHeightValidatorsChanged, state.NextValidators, batch); err != nil {
		return err
	}
	// Save next consensus params.
	if err := store.saveConsensusParamsInfo(nextHeight,
		state.LastHeightConsensusParamsChanged, state.ConsensusParams, batch); err != nil {
		return err
	}

	// Counting the amount of time taken to marshall the state.
	// In case the state is big this can impact the metrics reporting
	stateMarshallTime := time.Now()
	stateBytes := state.Bytes()
	stateMarshallDiff := time.Since(stateMarshallTime).Seconds()

	if err := batch.Set(key, stateBytes); err != nil {
		return err
	}
	if err := batch.WriteSync(); err != nil {
		panic(err)
	}
	store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "save").Observe(time.Since(start).Seconds() - stateMarshallDiff)
	return nil
}

// BootstrapState saves a new state, used e.g. by state sync when starting from non-zero height.
func (store dbStore) Bootstrap(state State) error {
	batch := store.db.NewBatch()
	defer func(batch dbm.Batch) {
		err := batch.Close()
		if err != nil {
			panic(err)
		}
	}(batch)
	height := state.LastBlockHeight + 1
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "bootstrap"), time.Now())()
	if height == 1 {
		height = state.InitialHeight
	}

	if height > 1 && !state.LastValidators.IsNilOrEmpty() {
		if err := store.saveValidatorsInfo(height-1, height-1, state.LastValidators, batch); err != nil {
			return err
		}
	}

	if err := store.saveValidatorsInfo(height, height, state.Validators, batch); err != nil {
		return err
	}

	if err := store.saveValidatorsInfo(height+1, height+1, state.NextValidators, batch); err != nil {
		return err
	}

	if err := store.saveConsensusParamsInfo(height,
		state.LastHeightConsensusParamsChanged, state.ConsensusParams, batch); err != nil {
		return err
	}

	if err := batch.Set(stateKey, state.Bytes()); err != nil {
		return err
	}

	if err := batch.WriteSync(); err != nil {
		panic(err)
	}

	return batch.Close()
}

// PruneStates deletes states between the given heights (including from, excluding to). It is not
// guaranteed to delete all states, since the last checkpointed state and states being pointed to by
// e.g. `LastHeightChanged` must remain. The state at to must also exist.
//
// The from parameter is necessary since we can't do a key scan in a performant way due to the key
// encoding not preserving ordering: https://github.com/tendermint/tendermint/issues/4567
// This will cause some old states to be left behind when doing incremental partial prunes,
// specifically older checkpoints and LastHeightChanged targets.
func (store dbStore) PruneStates(from int64, to int64, evidenceThresholdHeight int64, previosulyPrunedStates uint64) (uint64, error) {
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "prune_states"), time.Now())()

	if from <= 0 || to <= 0 {
		return 0, fmt.Errorf("from height %v and to height %v must be greater than 0", from, to)
	}
	if from >= to {
		return 0, fmt.Errorf("from height %v must be lower than to height %v", from, to)
	}

	valInfo, elapsedTime, err := loadValidatorsInfo(store.db, store.DBKeyLayout.CalcValidatorsKey(min(to, evidenceThresholdHeight)))
	if err != nil {
		return 0, fmt.Errorf("validators at height %v not found: %w", to, err)
	}

	paramsInfo, err := store.loadConsensusParamsInfo(to)
	if err != nil {
		return 0, fmt.Errorf("consensus params at height %v not found: %w", to, err)
	}

	keepVals := make(map[int64]bool)
	if valInfo.ValidatorSet == nil {
		keepVals[valInfo.LastHeightChanged] = true
		keepVals[lastStoredHeightFor(to, valInfo.LastHeightChanged)] = true // keep last checkpoint too
	}
	keepParams := make(map[int64]bool)
	if paramsInfo.ConsensusParams.Equal(&cmtproto.ConsensusParams{}) {
		keepParams[paramsInfo.LastHeightChanged] = true
	}

	batch := store.db.NewBatch()
	defer batch.Close()
	pruned := uint64(0)

	// We have to delete in reverse order, to avoid deleting previous heights that have validator
	// sets and consensus params that we may need to retrieve.
	for h := to - 1; h >= from; h-- {
		// For heights we keep, we must make sure they have the full validator set or consensus
		// params, otherwise they will panic if they're retrieved directly (instead of
		// indirectly via a LastHeightChanged pointer).
		if keepVals[h] {
			v, tmpTime, err := loadValidatorsInfo(store.db, store.DBKeyLayout.CalcValidatorsKey(h))
			elapsedTime += tmpTime
			if err != nil || v.ValidatorSet == nil {
				vip, err := store.LoadValidators(h)
				if err != nil {
					return pruned, err
				}

				pvi, err := vip.ToProto()
				if err != nil {
					return pruned, err
				}

				v.ValidatorSet = pvi
				v.LastHeightChanged = h

				bz, err := v.Marshal()
				if err != nil {
					return pruned, err
				}
				err = batch.Set(store.DBKeyLayout.CalcValidatorsKey(h), bz)
				if err != nil {
					return pruned, err
				}
			}
		} else if h < evidenceThresholdHeight {
			err = batch.Delete(store.DBKeyLayout.CalcValidatorsKey(h))
			if err != nil {
				return pruned, err
			}
		}
		// else we keep the validator set because we might need
		// it later on for evidence verification

		if keepParams[h] {
			p, err := store.loadConsensusParamsInfo(h)
			if err != nil {
				return pruned, err
			}

			if p.ConsensusParams.Equal(&cmtproto.ConsensusParams{}) {
				params, err := store.LoadConsensusParams(h)
				if err != nil {
					return pruned, err
				}
				p.ConsensusParams = params.ToProto()

				p.LastHeightChanged = h
				bz, err := p.Marshal()
				if err != nil {
					return pruned, err
				}

				err = batch.Set(store.DBKeyLayout.CalcConsensusParamsKey(h), bz)
				if err != nil {
					return pruned, err
				}
			}
		} else {
			err = batch.Delete(store.DBKeyLayout.CalcConsensusParamsKey(h))
			if err != nil {
				return pruned, err
			}
		}

		err = batch.Delete(store.DBKeyLayout.CalcABCIResponsesKey(h))
		if err != nil {
			return pruned, err
		}
		pruned++

		// avoid batches growing too large by flushing to database regularly
		if pruned%1000 == 0 && pruned > 0 {
			err := batch.Write()
			if err != nil {
				return pruned, err
			}
			batch.Close()
			batch = store.db.NewBatch()
			defer batch.Close()
		}
	}

	err = batch.WriteSync()
	if err != nil {
		return pruned, err
	}

	// We do not want to panic or interrupt consensus on compaction failure
	if store.StoreOptions.Compact && previosulyPrunedStates+pruned >= uint64(store.StoreOptions.CompactionInterval) {
		// When the range is nil,nil, the database will try to compact
		// ALL levels. Another option is to set a predefined range of
		// specific keys.
		err = store.db.Compact(nil, nil)
	}

	store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "pruning_load_validator_info").Observe(elapsedTime)
	return pruned, err
}

// PruneABCIResponses attempts to prune all ABCI responses up to, but not
// including, the given height. On success, returns the number of heights
// pruned and the new retain height.
func (store dbStore) PruneABCIResponses(targetRetainHeight int64, forceCompact bool) (pruned int64, newRetainHeight int64, err error) {
	if store.DiscardABCIResponses {
		return 0, 0, nil
	}
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "prune_abci_responses"), time.Now())()
	lastRetainHeight, err := store.getLastABCIResponsesRetainHeight()
	if err != nil {
		return 0, 0, fmt.Errorf("failed to look up last ABCI responses retain height: %w", err)
	}
	if lastRetainHeight == 0 {
		lastRetainHeight = 1
	}

	batch := store.db.NewBatch()
	defer batch.Close()

	batchPruned := int64(0)

	for h := lastRetainHeight; h < targetRetainHeight; h++ {
		if err := batch.Delete(store.DBKeyLayout.CalcABCIResponsesKey(h)); err != nil {
			return pruned, lastRetainHeight + pruned, fmt.Errorf("failed to delete ABCI responses at height %d: %w", h, err)
		}
		batchPruned++
		if batchPruned >= 1000 {
			if err := batch.Write(); err != nil {
				return pruned, lastRetainHeight + pruned, fmt.Errorf("failed to write ABCI responses deletion batch at height %d: %w", h, err)
			}
			batch.Close()

			pruned += batchPruned
			batchPruned = 0
			if err := store.setLastABCIResponsesRetainHeight(h); err != nil {
				return pruned, lastRetainHeight + pruned, fmt.Errorf("failed to set last ABCI responses retain height: %w", err)
			}

			batch = store.db.NewBatch()
			defer batch.Close()
		}
	}

	if err = batch.WriteSync(); err != nil {
		return pruned + batchPruned, targetRetainHeight, err
	}

	if forceCompact && store.Compact {
		if pruned+batchPruned >= store.CompactionInterval || targetRetainHeight-lastRetainHeight >= store.CompactionInterval {
			err = store.db.Compact(nil, nil)
		}
	}
	return pruned + batchPruned, targetRetainHeight, err
}

// ------------------------------------------------------------------------

// TxResultsHash returns the root hash of a Merkle tree of
// ExecTxResulst responses (see ABCIResults.Hash)
//
// See merkle.SimpleHashFromByteSlices.
func TxResultsHash(txResults []*abci.ExecTxResult) []byte {
	return types.NewResults(txResults).Hash()
}

// LoadFinalizeBlockResponse loads the DiscardABCIResponses for the given height from the
// database. If the node has D set to true, ErrFinalizeBlockResponsesNotPersisted
// is persisted. If not found, ErrNoABCIResponsesForHeight is returned.
func (store dbStore) LoadFinalizeBlockResponse(height int64) (*abci.FinalizeBlockResponse, error) {
	if store.DiscardABCIResponses {
		return nil, ErrFinalizeBlockResponsesNotPersisted
	}

	start := time.Now()
	buf, err := store.db.Get(store.DBKeyLayout.CalcABCIResponsesKey(height))
	if err != nil {
		return nil, err
	}

	addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_abci_responses"), start)()

	if len(buf) == 0 {
		return nil, ErrNoABCIResponsesForHeight{height}
	}

	resp := new(abci.FinalizeBlockResponse)
	err = resp.Unmarshal(buf)
	// Check for an error or if the resp.AppHash is nil if so
	// this means the unmarshalling should be a LegacyABCIResponses
	// Depending on a source message content (serialized as ABCIResponses)
	// there are instances where it can be deserialized as a FinalizeBlockResponse
	// without causing an error. But the values will not be deserialized properly
	// and, it will contain zero values, and one of them is an AppHash == nil
	// This can be verified in the /state/compatibility_test.go file
	if err != nil || resp.AppHash == nil {
		// The data might be of the legacy ABCI response type, so
		// we try to unmarshal that
		legacyResp := new(cmtstate.LegacyABCIResponses)
		if err := legacyResp.Unmarshal(buf); err != nil {
			// only return an error, this method is only invoked through the `/block_results` not for state logic and
			// some tests, so no need to exit cometbft if there's an error, just return it.
			store.Logger.Error("failed in LoadFinalizeBlockResponse", "error", ErrABCIResponseCorruptedOrSpecChangeForHeight{Height: height, Err: err})
			return nil, ErrABCIResponseCorruptedOrSpecChangeForHeight{Height: height, Err: err}
		}
		// The state store contains the old format. Migrate to
		// the new FinalizeBlockResponse format. Note that the
		// new struct expects the AppHash which we don't have.
		return responseFinalizeBlockFromLegacy(legacyResp), nil
	}

	// TODO: ensure that buf is completely read.

	// Otherwise return the FinalizeBlockResponse
	return resp, nil
}

// LoadLastFinalizeBlockResponse loads the FinalizeBlockResponses from the most recent height.
// The height parameter is used to ensure that the response corresponds to the latest height.
// If not, an error is returned.
//
// This method is used for recovering in the case that we called the Commit ABCI
// method on the application but crashed before persisting the results.
func (store dbStore) LoadLastFinalizeBlockResponse(height int64) (*abci.FinalizeBlockResponse, error) {
	start := time.Now()
	buf, err := store.db.Get(store.DBKeyLayout.CalcABCIResponsesKey(height))
	if err != nil {
		return nil, err
	}
	addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_last_abci_response"), start)()
	if len(buf) == 0 {
		// DEPRECATED lastABCIResponseKey
		// It is possible if this is called directly after an upgrade that
		// `lastABCIResponseKey` contains the last ABCI responses.
		bz, err := store.db.Get(lastABCIResponseKey)
		if err == nil && len(bz) > 0 {
			info := new(cmtstate.ABCIResponsesInfo)
			err = info.Unmarshal(bz)
			if err != nil {
				cmtos.Exit(fmt.Sprintf(`LoadLastFinalizeBlockResponse: Data has been corrupted or its spec has changed: %v\n`, err))
			}
			// Here we validate the result by comparing its height to the expected height.
			if height != info.GetHeight() {
				return nil, fmt.Errorf("expected height %d but last stored abci responses was at height %d", height, info.GetHeight())
			}
			if info.FinalizeBlock == nil {
				// sanity check
				if info.LegacyAbciResponses == nil {
					panic("state store contains last abci response but it is empty")
				}
				return responseFinalizeBlockFromLegacy(info.LegacyAbciResponses), nil
			}
			return info.FinalizeBlock, nil
		}
		// END OF DEPRECATED lastABCIResponseKey
		return nil, fmt.Errorf("expected last ABCI responses at height %d, but none are found", height)
	}
	resp := new(abci.FinalizeBlockResponse)
	err = resp.Unmarshal(buf)
	if err != nil {
		cmtos.Exit(fmt.Sprintf(`LoadLastFinalizeBlockResponse: Data has been corrupted or its spec has changed: %v\n`, err))
	}
	return resp, nil
}

// SaveFinalizeBlockResponse persists the FinalizeBlockResponse to the database.
// This is useful in case we crash after app.Commit and before s.Save().
// Responses are indexed by height so they can also be loaded later to produce
// Merkle proofs.
//
// CONTRACT: height must be monotonically increasing every time this is called.
func (store dbStore) SaveFinalizeBlockResponse(height int64, resp *abci.FinalizeBlockResponse) error {
	var dtxs []*abci.ExecTxResult
	// strip nil values,
	for _, tx := range resp.TxResults {
		if tx != nil {
			dtxs = append(dtxs, tx)
		}
	}
	resp.TxResults = dtxs

	bz, err := resp.Marshal()
	if err != nil {
		return err
	}

	// Save the ABCI response.
	//
	// We always save the last ABCI response for crash recovery.
	// If `store.DiscardABCIResponses` is true, then we delete the previous ABCI response.
	start := time.Now()
	if store.DiscardABCIResponses && height > 1 {
		if err := store.db.Delete(store.DBKeyLayout.CalcABCIResponsesKey(height - 1)); err != nil {
			return err
		}
	}
	if err := store.db.SetSync(store.DBKeyLayout.CalcABCIResponsesKey(height), bz); err != nil {
		return err
	}
	addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "save_abci_responses"), start)()
	return nil
}

func (store dbStore) getValue(key []byte) ([]byte, error) {
	bz, err := store.db.Get(key)
	if err != nil {
		return nil, err
	}

	if len(bz) == 0 {
		return nil, ErrKeyNotFound
	}
	return bz, nil
}

// ApplicationRetainHeight.
func (store dbStore) SaveApplicationRetainHeight(height int64) error {
	return store.db.SetSync(AppRetainHeightKey, int64ToBytes(height))
}

func (store dbStore) GetApplicationRetainHeight() (int64, error) {
	buf, err := store.getValue(AppRetainHeightKey)
	if err != nil {
		return 0, err
	}
	height := int64FromBytes(buf)

	if height < 0 {
		return 0, ErrInvalidHeightValue
	}

	return height, nil
}

// DataCompanionRetainHeight.
func (store dbStore) SaveCompanionBlockRetainHeight(height int64) error {
	return store.db.SetSync(CompanionBlockRetainHeightKey, int64ToBytes(height))
}

func (store dbStore) GetCompanionBlockRetainHeight() (int64, error) {
	buf, err := store.getValue(CompanionBlockRetainHeightKey)
	if err != nil {
		return 0, err
	}
	height := int64FromBytes(buf)

	if height < 0 {
		return 0, ErrInvalidHeightValue
	}

	return height, nil
}

// DataCompanionRetainHeight.
func (store dbStore) SaveABCIResRetainHeight(height int64) error {
	return store.db.SetSync(ABCIResultsRetainHeightKey, int64ToBytes(height))
}

func (store dbStore) GetABCIResRetainHeight() (int64, error) {
	buf, err := store.getValue(ABCIResultsRetainHeightKey)
	if err != nil {
		return 0, err
	}
	height := int64FromBytes(buf)

	if height < 0 {
		return 0, ErrInvalidHeightValue
	}

	return height, nil
}

func (store dbStore) getLastABCIResponsesRetainHeight() (int64, error) {
	bz, err := store.getValue(lastABCIResponsesRetainHeightKey)
	if errors.Is(err, ErrKeyNotFound) {
		return 0, nil
	}
	height := int64FromBytes(bz)
	if height < 0 {
		return 0, ErrInvalidHeightValue
	}
	return height, nil
}

func (store dbStore) setLastABCIResponsesRetainHeight(height int64) error {
	return store.db.SetSync(lastABCIResponsesRetainHeightKey, int64ToBytes(height))
}

// -----------------------------------------------------------------------------

// LoadValidators loads the ValidatorSet for a given height.
// Returns ErrNoValSetForHeight if the validator set can't be found for this height.
func (store dbStore) LoadValidators(height int64) (*types.ValidatorSet, error) {
	valInfo, elapsedTime, err := loadValidatorsInfo(store.db, store.DBKeyLayout.CalcValidatorsKey(height))
	if err != nil {
		return nil, ErrNoValSetForHeight{height}
	}
	// (WARN) This includes time to unmarshal the validator info
	if valInfo.ValidatorSet == nil {
		lastStoredHeight := lastStoredHeightFor(height, valInfo.LastHeightChanged)
		valInfo2, tmpTime, err := loadValidatorsInfo(store.db, store.DBKeyLayout.CalcValidatorsKey(lastStoredHeight))
		elapsedTime += tmpTime
		if err != nil || valInfo2.ValidatorSet == nil {
			return nil,
				fmt.Errorf("couldn't find validators at height %d (height %d was originally requested): %w",
					lastStoredHeight,
					height,
					err,
				)
		}

		vs, err := types.ValidatorSetFromProto(valInfo2.ValidatorSet)
		if err != nil {
			return nil, err
		}

		vs.IncrementProposerPriority(cmtmath.SafeConvertInt32(height - lastStoredHeight)) // mutate
		vi2, err := vs.ToProto()
		if err != nil {
			return nil, err
		}

		valInfo2.ValidatorSet = vi2
		valInfo = valInfo2
	}

	vip, err := types.ValidatorSetFromProto(valInfo.ValidatorSet)
	if err != nil {
		return nil, err
	}
	store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_validators").Observe(elapsedTime)
	return vip, nil
}

func lastStoredHeightFor(height, lastHeightChanged int64) int64 {
	checkpointHeight := height - height%valSetCheckpointInterval
	return cmtmath.MaxInt64(checkpointHeight, lastHeightChanged)
}

// CONTRACT: Returned ValidatorsInfo can be mutated.
func loadValidatorsInfo(db dbm.DB, valInfoKey []byte) (*cmtstate.ValidatorsInfo, float64, error) {
	start := time.Now()
	buf, err := db.Get(valInfoKey)
	if err != nil {
		return nil, 0, err
	}

	elapsedTime := time.Since(start).Seconds()

	if len(buf) == 0 {
		return nil, 0, errors.New("value retrieved from db is empty")
	}

	v := new(cmtstate.ValidatorsInfo)
	err = v.Unmarshal(buf)
	if err != nil {
		// DATA HAS BEEN CORRUPTED OR THE SPEC HAS CHANGED
		cmtos.Exit(fmt.Sprintf(`LoadValidators: Data has been corrupted or its spec has changed:
        %v\n`, err))
	}
	// TODO: ensure that buf is completely read.

	return v, elapsedTime, nil
}

// saveValidatorsInfo persists the validator set.
//
// `height` is the effective height for which the validator is responsible for
// signing. It should be called from s.Save(), right before the state itself is
// persisted.
func (store dbStore) saveValidatorsInfo(height, lastHeightChanged int64, valSet *types.ValidatorSet, batch dbm.Batch) error {
	if lastHeightChanged > height {
		return errors.New("lastHeightChanged cannot be greater than ValidatorsInfo height")
	}
	valInfo := &cmtstate.ValidatorsInfo{
		LastHeightChanged: lastHeightChanged,
	}
	// Only persist validator set if it was updated or checkpoint height (see
	// valSetCheckpointInterval) is reached.
	if height == lastHeightChanged || height%valSetCheckpointInterval == 0 {
		pv, err := valSet.ToProto()
		if err != nil {
			return err
		}
		valInfo.ValidatorSet = pv
	}

	bz, err := valInfo.Marshal()
	if err != nil {
		return err
	}
	start := time.Now()
	err = batch.Set(store.DBKeyLayout.CalcValidatorsKey(height), bz)
	if err != nil {
		return err
	}
	defer addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "saveValidatorsInfo"), start)()

	return nil
}

// -----------------------------------------------------------------------------

// ConsensusParamsInfo represents the latest consensus params, or the last height it changed

// LoadConsensusParams loads the ConsensusParams for a given height.
func (store dbStore) LoadConsensusParams(height int64) (types.ConsensusParams, error) {
	var (
		empty   = types.ConsensusParams{}
		emptypb = cmtproto.ConsensusParams{}
	)
	paramsInfo, err := store.loadConsensusParamsInfo(height)
	if err != nil {
		return empty, fmt.Errorf("could not find consensus params for height #%d: %w", height, err)
	}

	if paramsInfo.ConsensusParams.Equal(&emptypb) {
		paramsInfo2, err := store.loadConsensusParamsInfo(paramsInfo.LastHeightChanged)
		if err != nil {
			return empty, fmt.Errorf(
				"couldn't find consensus params at height %d as last changed from height %d: %w",
				paramsInfo.LastHeightChanged,
				height,
				err,
			)
		}

		paramsInfo = paramsInfo2
	}

	return types.ConsensusParamsFromProto(paramsInfo.ConsensusParams), nil
}

func (store dbStore) loadConsensusParamsInfo(height int64) (*cmtstate.ConsensusParamsInfo, error) {
	start := time.Now()
	buf, err := store.db.Get(store.DBKeyLayout.CalcConsensusParamsKey(height))
	if err != nil {
		return nil, err
	}

	addTimeSample(store.StoreOptions.Metrics.StoreAccessDurationSeconds.With("method", "load_consensus_params"), start)()

	if len(buf) == 0 {
		return nil, errors.New("value retrieved from db is empty")
	}

	paramsInfo := new(cmtstate.ConsensusParamsInfo)
	if err = paramsInfo.Unmarshal(buf); err != nil {
		// DATA HAS BEEN CORRUPTED OR THE SPEC HAS CHANGED
		cmtos.Exit(fmt.Sprintf(`LoadConsensusParams: Data has been corrupted or its spec has changed:
                %v\n`, err))
	}
	// TODO: ensure that buf is completely read.

	return paramsInfo, nil
}

// saveConsensusParamsInfo persists the consensus params for the next block to disk.
// It should be called from s.Save(), right before the state itself is persisted.
// If the consensus params did not change after processing the latest block,
// only the last height for which they changed is persisted.
func (store dbStore) saveConsensusParamsInfo(nextHeight, changeHeight int64, params types.ConsensusParams, batch dbm.Batch) error {
	paramsInfo := &cmtstate.ConsensusParamsInfo{
		LastHeightChanged: changeHeight,
	}

	if changeHeight == nextHeight {
		paramsInfo.ConsensusParams = params.ToProto()
	}
	bz, err := paramsInfo.Marshal()
	if err != nil {
		return err
	}

	err = batch.Set(store.DBKeyLayout.CalcConsensusParamsKey(nextHeight), bz)
	if err != nil {
		return err
	}

	return nil
}

func (store dbStore) SetOfflineStateSyncHeight(height int64) error {
	err := store.db.SetSync(offlineStateSyncHeight, int64ToBytes(height))
	if err != nil {
		return err
	}
	return nil
}

// Gets the height at which the store is bootstrapped after out of band statesync.
func (store dbStore) GetOfflineStateSyncHeight() (int64, error) {
	buf, err := store.db.Get(offlineStateSyncHeight)
	if err != nil {
		return 0, err
	}

	if len(buf) == 0 {
		return 0, errors.New("value empty")
	}

	height := int64FromBytes(buf)
	if height < 0 {
		return 0, errors.New("invalid value for height: height cannot be negative")
	}
	return height, nil
}

func (store dbStore) Close() error {
	return store.db.Close()
}

func min(a int64, b int64) int64 {
	if a < b {
		return a
	}
	return b
}

// responseFinalizeBlockFromLegacy is a convenience function that takes the old abci responses and morphs
// it to the finalize block response. Note that the app hash is missing.
func responseFinalizeBlockFromLegacy(legacyResp *cmtstate.LegacyABCIResponses) *abci.FinalizeBlockResponse {
	var response abci.FinalizeBlockResponse
	events := make([]abci.Event, 0)

	if legacyResp.DeliverTxs != nil {
		response.TxResults = legacyResp.DeliverTxs
	}

	// Check for begin block and end block and only append events or assign values if they are not nil
	if legacyResp.BeginBlock != nil {
		if legacyResp.BeginBlock.Events != nil {
			// Add BeginBlock attribute to BeginBlock events
			for idx := range legacyResp.BeginBlock.Events {
				legacyResp.BeginBlock.Events[idx].Attributes = append(legacyResp.BeginBlock.Events[idx].Attributes, abci.EventAttribute{
					Key:   "mode",
					Value: "BeginBlock",
					Index: false,
				})
			}
			events = append(events, legacyResp.BeginBlock.Events...)
		}
	}
	if legacyResp.EndBlock != nil {
		if legacyResp.EndBlock.ValidatorUpdates != nil {
			response.ValidatorUpdates = legacyResp.EndBlock.ValidatorUpdates
		}
		if legacyResp.EndBlock.ConsensusParamUpdates != nil {
			response.ConsensusParamUpdates = legacyResp.EndBlock.ConsensusParamUpdates
		}
		if legacyResp.EndBlock.Events != nil {
			// Add EndBlock attribute to BeginBlock events
			for idx := range legacyResp.EndBlock.Events {
				legacyResp.EndBlock.Events[idx].Attributes = append(legacyResp.EndBlock.Events[idx].Attributes, abci.EventAttribute{
					Key:   "mode",
					Value: "EndBlock",
					Index: false,
				})
			}
			events = append(events, legacyResp.EndBlock.Events...)
		}
	}

	response.Events = events

	// NOTE: AppHash is missing in the response but will
	// be caught and filled in consensus/replay.go
	return &response
}

// ----- Util.
func int64FromBytes(bz []byte) int64 {
	v, _ := binary.Varint(bz)
	return v
}

func int64ToBytes(i int64) []byte {
	buf := make([]byte, binary.MaxVarintLen64)
	n := binary.PutVarint(buf, i)
	return buf[:n]
}

// addTimeSample returns a function that, when called, adds an observation to m.
// The observation added to m is the number of seconds elapsed since addTimeSample
// was initially called. addTimeSample is meant to be called in a defer to calculate
// the amount of time a function takes to complete.
func addTimeSample(m metrics.Histogram, start time.Time) func() {
	return func() { m.Observe(time.Since(start).Seconds()) }
}
