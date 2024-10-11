package beacon

import (
	"context"
	"errors"
	"fmt"
	"io"
	"math/big"
	"sort"
	"sync"
	"time"

	"github.com/patrickmn/go-cache"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	depositCache "github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// infostream is a struct for each instance of the infostream created by a client connection.
type infostream struct {
	ctx                 context.Context
	headFetcher         blockchain.HeadFetcher
	depositFetcher      depositCache.DepositFetcher
	blockFetcher        execution.POWBlockFetcher
	beaconDB            db.ReadOnlyDatabase
	pubKeys             [][]byte
	pubKeysMutex        *sync.RWMutex
	stateChannel        chan *feed.Event
	stateSub            event.Subscription
	eth1Deposits        *cache.Cache
	eth1DepositsMutex   *sync.RWMutex
	eth1Blocktimes      *cache.Cache
	eth1BlocktimesMutex *sync.RWMutex
	currentEpoch        primitives.Epoch
	stream              ethpb.BeaconChain_StreamValidatorsInfoServer
	genesisTime         uint64
}

// eth1Deposit contains information about a deposit made on the Ethereum 1 chain.
type eth1Deposit struct {
	block *big.Int
	data  *ethpb.Deposit_Data
}

var (
	eth1DepositCacheHits = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "infostream_eth1_deposit_cache_hits",
			Help: "The number of times the infostream Ethereum 1 deposit cache is hit.",
		},
	)
	eth1DepositCacheMisses = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "infostream_eth1_deposit_cache_misses",
			Help: "The number of times the infostream Ethereum 1 deposit cache is missed.",
		},
	)
	eth1BlocktimeCacheHits = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "infostream_eth1_blocktime_cache_hits",
			Help: "The number of times the infostream Ethereum 1 block time cache is hit.",
		},
	)
	eth1BlocktimeCacheMisses = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "infostream_eth1_blocktime_cache_misses",
			Help: "The number of times the infostream Ethereum 1 block time cache is missed.",
		},
	)
)

// StreamValidatorsInfo returns a stream of information for given validators.
// Validators are supplied dynamically by the client, and can be added, removed and reset at any time.
// Information about the current set of validators is supplied as soon as the end-of-epoch accounting has been processed,
// providing a near real-time view of the state of the validators.
// Note that this will stream information whilst syncing; this is intended, to allow for complete validator state capture
// over time.  If this is not required then the client can either wait until the beacon node is synced, or filter results
// based on the epoch value in the returned validator info.
// DEPRECATED: Streaming Validator Info is no longer supported.
// The Beacon API /eth/v1/beacon/states/{state_id}/validators will provide validator info in unstreamed format.
func (bs *Server) StreamValidatorsInfo(stream ethpb.BeaconChain_StreamValidatorsInfoServer) error {
	stateChannel := make(chan *feed.Event, params.BeaconConfig().SlotsPerEpoch)
	epochDuration := time.Duration(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot)) * time.Second

	// Fetch our current epoch.
	headState, err := bs.HeadFetcher.HeadState(bs.Ctx)
	if err != nil {
		return status.Error(codes.Internal, "Could not access head state")
	}
	if headState == nil || headState.IsNil() {
		return status.Error(codes.Internal, "Not ready to serve information")
	}

	// Create an infostream struct.  This will track relevant state for the stream.
	infostream := &infostream{
		ctx:                 bs.Ctx,
		headFetcher:         bs.HeadFetcher,
		depositFetcher:      bs.DepositFetcher,
		blockFetcher:        bs.BlockFetcher,
		beaconDB:            bs.BeaconDB,
		pubKeys:             make([][]byte, 0),
		pubKeysMutex:        &sync.RWMutex{},
		stateChannel:        stateChannel,
		stateSub:            bs.StateNotifier.StateFeed().Subscribe(stateChannel),
		eth1Deposits:        cache.New(epochDuration, epochDuration*2),
		eth1DepositsMutex:   &sync.RWMutex{},
		eth1Blocktimes:      cache.New(epochDuration*12, epochDuration*24),
		eth1BlocktimesMutex: &sync.RWMutex{},
		currentEpoch:        primitives.Epoch(headState.Slot() / params.BeaconConfig().SlotsPerEpoch),
		stream:              stream,
		genesisTime:         headState.GenesisTime(),
	}
	defer infostream.stateSub.Unsubscribe()

	return infostream.handleConnection()
}

// handleConnection handles the two-way connection between client and server.
func (is *infostream) handleConnection() error {
	// Handle messages from client.
	go func() {
		for {
			msg, err := is.stream.Recv()
			if errors.Is(err, io.EOF) {
				return
			}
			if err != nil {
				// Errors handle elsewhere
				select {
				case <-is.stream.Context().Done():
					return
				case <-is.ctx.Done():
					return
				case <-is.stateSub.Err():
					return
				default:
				}
				log.WithError(err).Debug("Receive from validators stream listener failed; client probably closed connection")
				return
			}
			is.handleMessage(msg)
		}
	}()
	// Send responses at the end of every epoch.
	for {
		select {
		case stateEvent := <-is.stateChannel:
			if stateEvent.Type == statefeed.BlockProcessed {
				is.handleBlockProcessed()
			}
		case <-is.stateSub.Err():
			return status.Error(codes.Aborted, "Subscriber closed")
		case <-is.ctx.Done():
			return status.Error(codes.Canceled, "Service context canceled")
		case <-is.stream.Context().Done():
			return status.Error(codes.Canceled, "Stream context canceled")
		}
	}
}

// handleMessage handles a message from the infostream client, updating the list of keys.
func (is *infostream) handleMessage(msg *ethpb.ValidatorChangeSet) {
	var err error
	switch msg.Action {
	case ethpb.SetAction_ADD_VALIDATOR_KEYS:
		err = is.handleAddValidatorKeys(msg.PublicKeys)
	case ethpb.SetAction_REMOVE_VALIDATOR_KEYS:
		is.handleRemoveValidatorKeys(msg.PublicKeys)
	case ethpb.SetAction_SET_VALIDATOR_KEYS:
		err = is.handleSetValidatorKeys(msg.PublicKeys)
	}
	if err != nil {
		log.WithError(err).Debug("Error handling request; closing stream")
		is.stream.Context().Done()
	}
}

// handleAddValidatorKeys handles a request to add validator keys.
func (is *infostream) handleAddValidatorKeys(reqPubKeys [][]byte) error {
	is.pubKeysMutex.Lock()
	// Create existence map to ensure we don't duplicate keys.
	pubKeysMap := make(map[[fieldparams.BLSPubkeyLength]byte]bool, len(is.pubKeys))
	for _, pubKey := range is.pubKeys {
		pubKeysMap[bytesutil.ToBytes48(pubKey)] = true
	}
	addedPubKeys := make([][]byte, 0, len(reqPubKeys))
	for _, pubKey := range reqPubKeys {
		if _, exists := pubKeysMap[bytesutil.ToBytes48(pubKey)]; !exists {
			is.pubKeys = append(is.pubKeys, pubKey)
			addedPubKeys = append(addedPubKeys, pubKey)
		}
	}
	is.pubKeysMutex.Unlock()
	// Send immediate info for the new validators.
	return is.sendValidatorsInfo(addedPubKeys)
}

// handleSetValidatorKeys handles a request to set validator keys.
func (is *infostream) handleSetValidatorKeys(reqPubKeys [][]byte) error {
	is.pubKeysMutex.Lock()
	is.pubKeys = make([][]byte, 0, len(reqPubKeys))
	is.pubKeys = append(is.pubKeys, reqPubKeys...)
	is.pubKeysMutex.Unlock()
	// Send immediate info for the new validators.
	return is.sendValidatorsInfo(is.pubKeys)
}

// handleRemoveValidatorKeys handles a request to remove validator keys.
func (is *infostream) handleRemoveValidatorKeys(reqPubKeys [][]byte) {
	is.pubKeysMutex.Lock()
	// Create existence map to track what we have to delete.
	pubKeysMap := make(map[[fieldparams.BLSPubkeyLength]byte]bool, len(reqPubKeys))
	for _, pubKey := range reqPubKeys {
		pubKeysMap[bytesutil.ToBytes48(pubKey)] = true
	}
	max := len(is.pubKeys)
	for i := 0; i < max; i++ {
		if _, exists := pubKeysMap[bytesutil.ToBytes48(is.pubKeys[i])]; exists {
			copy(is.pubKeys[i:], is.pubKeys[i+1:])
			is.pubKeys = is.pubKeys[:len(is.pubKeys)-1]
			i--
			max--
		}
	}
	is.pubKeysMutex.Unlock()
}

// sendValidatorsInfo sends validator info for a specific set of public keys.
func (is *infostream) sendValidatorsInfo(pubKeys [][]byte) error {
	validators, err := is.generateValidatorsInfo(pubKeys)
	if err != nil {
		return err
	}
	for _, validator := range validators {
		if err := is.stream.Send(validator); err != nil {
			return err
		}
	}
	return nil
}

// generateValidatorsInfo generates the validator info for a set of public keys.
func (is *infostream) generateValidatorsInfo(pubKeys [][]byte) ([]*ethpb.ValidatorInfo, error) {
	if is.headFetcher == nil {
		return nil, status.Error(codes.Internal, "No head fetcher")
	}
	headState, err := is.headFetcher.HeadState(is.ctx)
	if err != nil {
		return nil, status.Error(codes.Internal, "Could not access head state")
	}
	if headState == nil || headState.IsNil() {
		return nil, status.Error(codes.Internal, "Not ready to serve information")
	}
	epoch := primitives.Epoch(headState.Slot() / params.BeaconConfig().SlotsPerEpoch)
	if epoch == 0 {
		// Not reporting, but no error.
		return nil, nil
	}
	// We are reporting on the state at the end of the *previous* epoch.
	epoch--

	res := make([]*ethpb.ValidatorInfo, 0, len(pubKeys))
	for _, pubKey := range pubKeys {
		i, e := headState.ValidatorIndexByPubkey(bytesutil.ToBytes48(pubKey))
		if !e {
			return nil, errors.New("could not find public key")
		}
		v, err := headState.ValidatorAtIndexReadOnly(i)
		if err != nil {
			return nil, status.Errorf(codes.Internal, "Could not retrieve validator: %v", err)
		}
		info, err := is.generateValidatorInfo(pubKey, v, headState, epoch)
		if err != nil {
			return nil, err
		}
		res = append(res, info)
	}

	// Calculate activation time for pending validators (if there are any).
	if err := is.calculateActivationTimeForPendingValidators(res, headState, epoch); err != nil {
		return nil, err
	}

	return res, nil
}

// generateValidatorInfo generates the validator info for a public key.
func (is *infostream) generateValidatorInfo(
	pubKey []byte,
	validator state.ReadOnlyValidator,
	headState state.ReadOnlyBeaconState,
	epoch primitives.Epoch,
) (*ethpb.ValidatorInfo, error) {
	info := &ethpb.ValidatorInfo{
		PublicKey: pubKey,
		Epoch:     epoch,
		Status:    ethpb.ValidatorStatus_UNKNOWN_STATUS,
	}

	// Index
	var ok bool
	info.Index, ok = headState.ValidatorIndexByPubkey(bytesutil.ToBytes48(pubKey))
	if !ok {
		// We don't know of this validator; it's either a pending deposit or totally unknown.
		return is.generatePendingValidatorInfo(info)
	}
	// Status and progression timestamp
	info.Status, info.TransitionTimestamp = is.calculateStatusAndTransition(validator, coreTime.CurrentEpoch(headState))

	// Balance
	if balance, err := headState.BalanceAtIndex(info.Index); err != nil {
		return nil, err
	} else {
		info.Balance = balance
	}

	// Effective balance (for attesting states)
	if info.Status == ethpb.ValidatorStatus_ACTIVE ||
		info.Status == ethpb.ValidatorStatus_SLASHING ||
		info.Status == ethpb.ValidatorStatus_EXITING {
		info.EffectiveBalance = validator.EffectiveBalance()
	}

	return info, nil
}

// generatePendingValidatorInfo generates the validator info for a pending (or unknown) key.
func (is *infostream) generatePendingValidatorInfo(info *ethpb.ValidatorInfo) (*ethpb.ValidatorInfo, error) {
	key := string(info.PublicKey)
	var deposit *eth1Deposit
	is.eth1DepositsMutex.Lock()
	if fetchedDeposit, exists := is.eth1Deposits.Get(key); exists {
		eth1DepositCacheHits.Inc()
		var ok bool
		deposit, ok = fetchedDeposit.(*eth1Deposit)
		if !ok {
			is.eth1DepositsMutex.Unlock()
			return nil, errors.New("cached eth1 deposit is not type *eth1Deposit")
		}
	} else {
		eth1DepositCacheMisses.Inc()
		fetchedDeposit, eth1BlockNumber := is.depositFetcher.DepositByPubkey(is.ctx, info.PublicKey)
		if fetchedDeposit == nil {
			deposit = &eth1Deposit{}
			is.eth1Deposits.Set(key, deposit, cache.DefaultExpiration)
		} else {
			deposit = &eth1Deposit{
				block: eth1BlockNumber,
				data:  fetchedDeposit.Data,
			}
			is.eth1Deposits.Set(key, deposit, cache.DefaultExpiration)
		}
	}
	is.eth1DepositsMutex.Unlock()
	if deposit.block != nil {
		info.Status = ethpb.ValidatorStatus_DEPOSITED
		if queueTimestamp, err := is.depositQueueTimestamp(deposit.block); err != nil {
			log.WithError(err).Error("Could not obtain queue activation timestamp")
		} else {
			info.TransitionTimestamp = queueTimestamp
		}
		info.Balance = deposit.data.Amount
	}
	return info, nil
}

func (is *infostream) calculateActivationTimeForPendingValidators(res []*ethpb.ValidatorInfo, headState state.ReadOnlyBeaconState, epoch primitives.Epoch) error {
	// pendingValidatorsMap is map from the validator pubkey to the index in our return array
	pendingValidatorsMap := make(map[[fieldparams.BLSPubkeyLength]byte]int)
	for i, info := range res {
		if info.Status == ethpb.ValidatorStatus_PENDING {
			pendingValidatorsMap[bytesutil.ToBytes48(info.PublicKey)] = i
		}
	}
	if len(pendingValidatorsMap) == 0 {
		// Nothing to do.
		return nil
	}

	// Fetch the list of pending validators; count the number of attesting validators.
	numAttestingValidators := uint64(0)
	pendingValidators := make([]primitives.ValidatorIndex, 0, headState.NumValidators())

	err := headState.ReadFromEveryValidator(func(idx int, val state.ReadOnlyValidator) error {
		if val.IsNil() {
			return errors.New("nil validator in state")
		}
		if helpers.IsEligibleForActivationUsingTrie(headState, val) {
			pubKey := val.PublicKey()
			validatorIndex, ok := headState.ValidatorIndexByPubkey(pubKey)
			if ok {
				pendingValidators = append(pendingValidators, validatorIndex)
			}
		}
		if helpers.IsActiveValidatorUsingTrie(val, epoch) {
			numAttestingValidators++
		}
		return nil
	})
	if err != nil {
		return err
	}

	sortableIndices := &indicesSorter{
		indices: pendingValidators,
	}
	sort.Sort(sortableIndices)

	sortedIndices := sortableIndices.indices

	// Loop over epochs, roughly simulating progression.
	for curEpoch := epoch + 1; len(sortedIndices) > 0 && len(pendingValidators) > 0; curEpoch++ {
		toProcess := helpers.ValidatorActivationChurnLimit(numAttestingValidators)
		if headState.Version() >= version.Deneb {
			toProcess = helpers.ValidatorActivationChurnLimitDeneb(numAttestingValidators)
		}

		if toProcess > uint64(len(sortedIndices)) {
			toProcess = uint64(len(sortedIndices))
		}
		for i := uint64(0); i < toProcess; i++ {
			validator, err := headState.ValidatorAtIndexReadOnly(sortedIndices[i])
			if err != nil {
				return err
			}
			if index, exists := pendingValidatorsMap[validator.PublicKey()]; exists {
				res[index].TransitionTimestamp = is.epochToTimestamp(helpers.ActivationExitEpoch(curEpoch))
				delete(pendingValidatorsMap, validator.PublicKey())
			}
			numAttestingValidators++
		}
		sortedIndices = sortedIndices[toProcess:]
	}

	return nil
}

// handleBlockProcessed handles the situation where a block has been processed by the Prysm server.
func (is *infostream) handleBlockProcessed() {
	headState, err := is.headFetcher.HeadState(is.ctx)
	if err != nil {
		log.Warn("Could not access head state for infostream")
		return
	}
	if headState == nil || headState.IsNil() {
		// We aren't ready to serve information
		return
	}
	blockEpoch := primitives.Epoch(headState.Slot() / params.BeaconConfig().SlotsPerEpoch)
	if blockEpoch == is.currentEpoch {
		// Epoch hasn't changed, nothing to report yet.
		return
	}
	is.currentEpoch = blockEpoch
	if err := is.sendValidatorsInfo(is.pubKeys); err != nil {
		// Client probably disconnected.
		log.WithError(err).Debug("Could not send infostream response")
	}
}

type indicesSorter struct {
	indices []primitives.ValidatorIndex
}

// Len is the number of elements in the collection.
func (s indicesSorter) Len() int { return len(s.indices) }

// Swap swaps the elements with indexes i and j.
func (s indicesSorter) Swap(i, j int) { s.indices[i], s.indices[j] = s.indices[j], s.indices[i] }

// Less reports whether the element with index i must sort before the element with index j.
func (s indicesSorter) Less(i, j int) bool {
	return s.indices[i] < s.indices[j]
}

func (is *infostream) calculateStatusAndTransition(validator state.ReadOnlyValidator, currentEpoch primitives.Epoch) (ethpb.ValidatorStatus, uint64) {
	farFutureEpoch := params.BeaconConfig().FarFutureEpoch

	if validator.IsNil() {
		return ethpb.ValidatorStatus_UNKNOWN_STATUS, 0
	}

	if currentEpoch < validator.ActivationEligibilityEpoch() {
		if validator.EffectiveBalance() == 0 {
			return ethpb.ValidatorStatus_PENDING, 0
		}
		if helpers.IsEligibleForActivationQueueUsingTrie(validator) {
			return ethpb.ValidatorStatus_DEPOSITED, is.epochToTimestamp(validator.ActivationEligibilityEpoch())
		}
		return ethpb.ValidatorStatus_DEPOSITED, 0
	}
	if currentEpoch < validator.ActivationEpoch() {
		return ethpb.ValidatorStatus_PENDING, is.epochToTimestamp(validator.ActivationEpoch())
	}
	if validator.ExitEpoch() == farFutureEpoch {
		return ethpb.ValidatorStatus_ACTIVE, 0
	}
	if currentEpoch < validator.ExitEpoch() {
		if validator.Slashed() {
			return ethpb.ValidatorStatus_SLASHING, is.epochToTimestamp(validator.ExitEpoch())
		}
		return ethpb.ValidatorStatus_EXITING, is.epochToTimestamp(validator.ExitEpoch())
	}
	return ethpb.ValidatorStatus_EXITED, is.epochToTimestamp(validator.WithdrawableEpoch())
}

// epochToTimestamp converts an epoch number to a timestamp.
func (is *infostream) epochToTimestamp(epoch primitives.Epoch) uint64 {
	return is.genesisTime + params.BeaconConfig().SecondsPerSlot*uint64(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch)))
}

// depositQueueTimestamp calculates the timestamp for exit of the validator from the deposit queue.
func (is *infostream) depositQueueTimestamp(eth1BlockNumber *big.Int) (uint64, error) {
	var blockTimestamp uint64
	key := fmt.Sprintf("%v", eth1BlockNumber)
	is.eth1BlocktimesMutex.Lock()
	if cachedTimestamp, exists := is.eth1Blocktimes.Get(key); exists {
		eth1BlocktimeCacheHits.Inc()
		var ok bool
		blockTimestamp, ok = cachedTimestamp.(uint64)
		if !ok {
			is.eth1BlocktimesMutex.Unlock()
			return 0, errors.New("cached timestamp is not type uint64")
		}
	} else {
		eth1BlocktimeCacheMisses.Inc()
		var err error
		blockTimestamp, err = is.blockFetcher.BlockTimeByHeight(is.ctx, eth1BlockNumber)
		if err != nil {
			is.eth1BlocktimesMutex.Unlock()
			return 0, err
		}
		is.eth1Blocktimes.Set(key, blockTimestamp, cache.DefaultExpiration)
	}
	is.eth1BlocktimesMutex.Unlock()

	followTime := time.Duration(params.BeaconConfig().Eth1FollowDistance*params.BeaconConfig().SecondsPerETH1Block) * time.Second
	eth1UnixTime := time.Unix(int64(blockTimestamp), 0).Add(followTime) // lint:ignore uintcast -- timestamp will not exceed int64 in your lifetime

	period := uint64(params.BeaconConfig().EpochsPerEth1VotingPeriod.Mul(uint64(params.BeaconConfig().SlotsPerEpoch)))
	votingPeriod := time.Duration(period*params.BeaconConfig().SecondsPerSlot) * time.Second
	activationTime := eth1UnixTime.Add(votingPeriod)
	eth2Genesis := time.Unix(int64(is.genesisTime), 0)

	if eth2Genesis.After(activationTime) {
		return is.genesisTime, nil
	}
	return uint64(activationTime.Unix()), nil
}
