// Package client represents a gRPC polling-based implementation
// of an Ethereum validator client.
package client

import (
	"bytes"
	"context"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"io"
	"math"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/dgraph-io/ristretto"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	lru "github.com/hashicorp/golang-lru"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorserviceconfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	accountsiface "github.com/prysmaticlabs/prysm/v4/validator/accounts/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	beacon_api "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api"
	"github.com/prysmaticlabs/prysm/v4/validator/client/iface"
	vdb "github.com/prysmaticlabs/prysm/v4/validator/db"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	"github.com/prysmaticlabs/prysm/v4/validator/graffiti"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	remoteweb3signer "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/metadata"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/emptypb"
)

// keyFetchPeriod is the frequency that we try to refetch validating keys
// in case no keys were fetched previously.
var (
	ErrBuilderValidatorRegistration = errors.New("Builder API validator registration unsuccessful")
	ErrValidatorsAllExited          = errors.New("All validators are exited, no more work to perform...")
)

var (
	msgCouldNotFetchKeys = "could not fetch validating keys"
	msgNoKeysFetched     = "No validating keys fetched. Waiting for keys..."
)

type validator struct {
	logValidatorBalances               bool
	useWeb                             bool
	emitAccountMetrics                 bool
	domainDataLock                     sync.RWMutex
	attLogsLock                        sync.Mutex
	aggregatedSlotCommitteeIDCacheLock sync.Mutex
	highestValidSlotLock               sync.Mutex
	prevBalanceLock                    sync.RWMutex
	slashableKeysLock                  sync.RWMutex
	eipImportBlacklistedPublicKeys     map[[fieldparams.BLSPubkeyLength]byte]bool
	walletInitializedFeed              *event.Feed
	attLogs                            map[[32]byte]*attSubmitted
	startBalances                      map[[fieldparams.BLSPubkeyLength]byte]uint64
	dutiesLock                         sync.RWMutex
	duties                             *ethpb.DutiesResponse
	prevBalance                        map[[fieldparams.BLSPubkeyLength]byte]uint64
	pubkeyToValidatorIndex             map[[fieldparams.BLSPubkeyLength]byte]primitives.ValidatorIndex
	signedValidatorRegistrations       map[[fieldparams.BLSPubkeyLength]byte]*ethpb.SignedValidatorRegistrationV1
	graffitiOrderedIndex               uint64
	aggregatedSlotCommitteeIDCache     *lru.Cache
	domainDataCache                    *ristretto.Cache
	highestValidSlot                   primitives.Slot
	genesisTime                        uint64
	slotFeed                           *event.Feed
	interopKeysConfig                  *local.InteropKeymanagerConfig
	wallet                             *wallet.Wallet
	graffitiStruct                     *graffiti.Graffiti
	beaconClient                       iface.BeaconChainClient
	nodeClient                         iface.NodeClient
	validatorClient                    iface.ValidatorClient
	prysmBeaconClient                  iface.PrysmBeaconChainClient
	db                                 vdb.Database
	keyManager                         keymanager.IKeymanager
	ticker                             slots.Ticker
	graffiti                           []byte
	voteStats                          voteStats
	syncCommitteeStats                 syncCommitteeStats
	Web3SignerConfig                   *remoteweb3signer.SetupConfig
	proposerSettings                   *validatorserviceconfig.ProposerSettings
	walletInitializedChannel           chan *wallet.Wallet
	validatorsRegBatchSize             int
}

type validatorStatus struct {
	publicKey []byte
	status    *ethpb.ValidatorStatusResponse
	index     primitives.ValidatorIndex
}

// Done cleans up the validator.
func (v *validator) Done() {
	v.ticker.Done()
}

// WaitForKeymanagerInitialization checks if the validator needs to wait for
func (v *validator) WaitForKeymanagerInitialization(ctx context.Context) error {
	genesisRoot, err := v.db.GenesisValidatorsRoot(ctx)
	if err != nil {
		return errors.Wrap(err, "unable to retrieve valid genesis validators root while initializing key manager")
	}

	if v.useWeb && v.wallet == nil {
		log.Info("Waiting for keymanager to initialize validator client with web UI")
		// if wallet is not set, wait for it to be set through the UI
		km, err := waitForWebWalletInitialization(ctx, v.walletInitializedFeed, v.walletInitializedChannel)
		if err != nil {
			return err
		}
		v.keyManager = km
	} else {
		if v.interopKeysConfig != nil {
			keyManager, err := local.NewInteropKeymanager(ctx, v.interopKeysConfig.Offset, v.interopKeysConfig.NumValidatorKeys)
			if err != nil {
				return errors.Wrap(err, "could not generate interop keys for key manager")
			}
			v.keyManager = keyManager
		} else if v.wallet == nil {
			return errors.New("wallet not set")
		} else {
			if v.Web3SignerConfig != nil {
				v.Web3SignerConfig.GenesisValidatorsRoot = genesisRoot
			}
			keyManager, err := v.wallet.InitializeKeymanager(ctx, accountsiface.InitKeymanagerConfig{ListenForChanges: true, Web3SignerConfig: v.Web3SignerConfig})
			if err != nil {
				return errors.Wrap(err, "could not initialize key manager")
			}
			v.keyManager = keyManager
		}
	}
	recheckKeys(ctx, v.db, v.keyManager)
	return nil
}

// subscribe to channel for when the wallet is initialized
func waitForWebWalletInitialization(
	ctx context.Context,
	walletInitializedEvent *event.Feed,
	walletChan chan *wallet.Wallet,
) (keymanager.IKeymanager, error) {
	sub := walletInitializedEvent.Subscribe(walletChan)
	defer sub.Unsubscribe()
	for {
		select {
		case w := <-walletChan:
			keyManager, err := w.InitializeKeymanager(ctx, accountsiface.InitKeymanagerConfig{ListenForChanges: true})
			if err != nil {
				return nil, errors.Wrap(err, "could not read keymanager")
			}
			return keyManager, nil
		case <-ctx.Done():
			return nil, errors.New("context canceled")
		case <-sub.Err():
			log.Error("Subscriber closed, exiting goroutine")
			return nil, nil
		}
	}
}

// recheckKeys checks if the validator has any keys that need to be rechecked.
// the keymanager implements a subscription to push these updates to the validator.
func recheckKeys(ctx context.Context, valDB vdb.Database, keyManager keymanager.IKeymanager) {
	var validatingKeys [][fieldparams.BLSPubkeyLength]byte
	var err error
	validatingKeys, err = keyManager.FetchValidatingPublicKeys(ctx)
	if err != nil {
		log.WithError(err).Debug("Could not fetch validating keys")
	}
	if err := valDB.UpdatePublicKeysBuckets(validatingKeys); err != nil {
		log.WithError(err).Debug("Could not update public keys buckets")
	}
	go recheckValidatingKeysBucket(ctx, valDB, keyManager)
	for _, key := range validatingKeys {
		log.WithField(
			"publicKey", fmt.Sprintf("%#x", bytesutil.Trunc(key[:])),
		).Info("Validating for public key")
	}
}

// to accounts changes in the keymanager, then updates those keys'
// buckets in bolt DB if a bucket for a key does not exist.
func recheckValidatingKeysBucket(ctx context.Context, valDB vdb.Database, km keymanager.IKeymanager) {
	importedKeymanager, ok := km.(*local.Keymanager)
	if !ok {
		return
	}
	validatingPubKeysChan := make(chan [][fieldparams.BLSPubkeyLength]byte, 1)
	sub := importedKeymanager.SubscribeAccountChanges(validatingPubKeysChan)
	defer func() {
		sub.Unsubscribe()
		close(validatingPubKeysChan)
	}()
	for {
		select {
		case keys := <-validatingPubKeysChan:
			if err := valDB.UpdatePublicKeysBuckets(keys); err != nil {
				log.WithError(err).Debug("Could not update public keys buckets")
				continue
			}
		case <-ctx.Done():
			return
		case <-sub.Err():
			log.Error("Subscriber closed, exiting goroutine")
			return
		}
	}
}

// WaitForChainStart checks whether the beacon node has started its runtime. That is,
// it calls to the beacon node which then verifies the ETH1.0 deposit contract logs to check
// for the ChainStart log to have been emitted. If so, it starts a ticker based on the ChainStart
// unix timestamp which will be used to keep track of time within the validator client.
func (v *validator) WaitForChainStart(ctx context.Context) error {
	ctx, span := trace.StartSpan(ctx, "validator.WaitForChainStart")
	defer span.End()

	// First, check if the beacon chain has started.
	log.Info("Syncing with beacon node to align on chain genesis info")

	chainStartRes, err := v.validatorClient.WaitForChainStart(ctx, &emptypb.Empty{})
	if err == io.EOF {
		return iface.ErrConnectionIssue
	}

	if ctx.Err() == context.Canceled {
		return errors.Wrap(ctx.Err(), "context has been canceled so shutting down the loop")
	}

	if err != nil {
		return errors.Wrap(
			iface.ErrConnectionIssue,
			errors.Wrap(err, "could not receive ChainStart from stream").Error(),
		)
	}

	v.genesisTime = chainStartRes.GenesisTime

	curGenValRoot, err := v.db.GenesisValidatorsRoot(ctx)
	if err != nil {
		return errors.Wrap(err, "could not get current genesis validators root")
	}

	if len(curGenValRoot) == 0 {
		if err := v.db.SaveGenesisValidatorsRoot(ctx, chainStartRes.GenesisValidatorsRoot); err != nil {
			return errors.Wrap(err, "could not save genesis validators root")
		}

		v.setTicker()
		return nil
	}

	if !bytes.Equal(curGenValRoot, chainStartRes.GenesisValidatorsRoot) {
		log.Errorf(`The genesis validators root received from the beacon node does not match what is in
			your validator database. This could indicate that this is a database meant for another network. If
			you were previously running this validator database on another network, please run --%s to
			clear the database. If not, please file an issue at https://github.com/prysmaticlabs/prysm/issues`,
			cmd.ClearDB.Name,
		)
		return fmt.Errorf(
			"genesis validators root from beacon node (%#x) does not match root saved in validator db (%#x)",
			chainStartRes.GenesisValidatorsRoot,
			curGenValRoot,
		)
	}

	v.setTicker()
	return nil
}

func (v *validator) setTicker() {
	// Once the ChainStart log is received, we update the genesis time of the validator client
	// and begin a slot ticker used to track the current slot the beacon node is in.
	v.ticker = slots.NewSlotTicker(time.Unix(int64(v.genesisTime), 0), params.BeaconConfig().SecondsPerSlot)
	log.WithField("genesisTime", time.Unix(int64(v.genesisTime), 0)).Info("Beacon chain started")
}

// WaitForSync checks whether the beacon node has sync to the latest head.
func (v *validator) WaitForSync(ctx context.Context) error {
	ctx, span := trace.StartSpan(ctx, "validator.WaitForSync")
	defer span.End()

	s, err := v.nodeClient.GetSyncStatus(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(iface.ErrConnectionIssue, errors.Wrap(err, "could not get sync status").Error())
	}
	if !s.Syncing {
		return nil
	}

	for {
		select {
		// Poll every half slot.
		case <-time.After(slots.DivideSlotBy(2 /* twice per slot */)):
			s, err := v.nodeClient.GetSyncStatus(ctx, &emptypb.Empty{})
			if err != nil {
				return errors.Wrap(iface.ErrConnectionIssue, errors.Wrap(err, "could not get sync status").Error())
			}
			if !s.Syncing {
				return nil
			}
			log.Info("Waiting for beacon node to sync to latest chain head")
		case <-ctx.Done():
			return errors.New("context has been canceled, exiting goroutine")
		}
	}
}

// ReceiveSlots starts a stream listener to obtain
// slots from the beacon node when it imports a block. Upon receiving a slot, the service
// broadcasts it to a feed for other usages to subscribe to.
func (v *validator) ReceiveSlots(ctx context.Context, connectionErrorChannel chan<- error) {
	stream, err := v.validatorClient.StreamSlots(ctx, &ethpb.StreamSlotsRequest{VerifiedOnly: true})
	if err != nil {
		log.WithError(err).Error("Failed to retrieve slots stream, " + iface.ErrConnectionIssue.Error())
		connectionErrorChannel <- errors.Wrap(iface.ErrConnectionIssue, err.Error())
		return
	}

	for {
		if ctx.Err() == context.Canceled {
			log.WithError(ctx.Err()).Error("Context canceled - shutting down slots receiver")
			return
		}
		res, err := stream.Recv()
		if err != nil {
			log.WithError(err).Error("Could not receive slots from beacon node: " + iface.ErrConnectionIssue.Error())
			connectionErrorChannel <- errors.Wrap(iface.ErrConnectionIssue, err.Error())
			return
		}
		if res == nil {
			continue
		}
		v.setHighestSlot(res.Slot)
	}
}

func (v *validator) checkAndLogValidatorStatus(statuses []*validatorStatus, activeValCount int64) bool {
	nonexistentIndex := primitives.ValidatorIndex(^uint64(0))
	var validatorActivated bool
	for _, status := range statuses {
		fields := logrus.Fields{
			"pubKey": fmt.Sprintf("%#x", bytesutil.Trunc(status.publicKey)),
			"status": status.status.Status.String(),
		}
		if status.index != nonexistentIndex {
			fields["index"] = status.index
		}
		log := log.WithFields(fields)
		if v.emitAccountMetrics {
			fmtKey := fmt.Sprintf("%#x", status.publicKey)
			ValidatorStatusesGaugeVec.WithLabelValues(fmtKey).Set(float64(status.status.Status))
		}
		switch status.status.Status {
		case ethpb.ValidatorStatus_UNKNOWN_STATUS:
			log.Info("Waiting for deposit to be observed by beacon node")
		case ethpb.ValidatorStatus_DEPOSITED:
			if status.status.PositionInActivationQueue != 0 {
				log.WithField(
					"positionInActivationQueue", status.status.PositionInActivationQueue,
				).Info("Deposit processed, entering activation queue after finalization")
			}
		case ethpb.ValidatorStatus_PENDING:
			if activeValCount >= 0 && status.status.ActivationEpoch == params.BeaconConfig().FarFutureEpoch {
				activationsPerEpoch :=
					uint64(math.Max(float64(params.BeaconConfig().MinPerEpochChurnLimit), float64(uint64(activeValCount)/params.BeaconConfig().ChurnLimitQuotient)))
				secondsPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
				expectedWaitingTime :=
					time.Duration((status.status.PositionInActivationQueue+activationsPerEpoch)/activationsPerEpoch*secondsPerEpoch) * time.Second
				log.WithFields(logrus.Fields{
					"positionInActivationQueue": status.status.PositionInActivationQueue,
					"expectedWaitingTime":       expectedWaitingTime.String(),
				}).Info("Waiting to be assigned activation epoch")
			} else if status.status.ActivationEpoch != params.BeaconConfig().FarFutureEpoch {
				log.WithFields(logrus.Fields{
					"activationEpoch": status.status.ActivationEpoch,
				}).Info("Waiting for activation")
			}
		case ethpb.ValidatorStatus_ACTIVE, ethpb.ValidatorStatus_EXITING:
			validatorActivated = true
			log.WithFields(logrus.Fields{
				"publicKey": fmt.Sprintf("%#x", bytesutil.Trunc(status.publicKey)),
				"index":     status.index,
			}).Info("Validator activated")
		case ethpb.ValidatorStatus_EXITED:
			log.Info("Validator exited")
		case ethpb.ValidatorStatus_INVALID:
			log.Warn("Invalid Eth1 deposit")
		default:
			log.WithFields(logrus.Fields{
				"activationEpoch": status.status.ActivationEpoch,
			}).Info("Validator status")
		}
	}
	return validatorActivated
}

// CanonicalHeadSlot returns the slot of canonical block currently found in the
// beacon chain via RPC.
func (v *validator) CanonicalHeadSlot(ctx context.Context) (primitives.Slot, error) {
	ctx, span := trace.StartSpan(ctx, "validator.CanonicalHeadSlot")
	defer span.End()
	head, err := v.beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return 0, errors.Wrap(iface.ErrConnectionIssue, err.Error())
	}
	return head.HeadSlot, nil
}

// NextSlot emits the next slot number at the start time of that slot.
func (v *validator) NextSlot() <-chan primitives.Slot {
	return v.ticker.C()
}

// SlotDeadline is the start time of the next slot.
func (v *validator) SlotDeadline(slot primitives.Slot) time.Time {
	secs := time.Duration((slot + 1).Mul(params.BeaconConfig().SecondsPerSlot))
	return time.Unix(int64(v.genesisTime), 0 /*ns*/).Add(secs * time.Second)
}

// CheckDoppelGanger checks if the current actively provided keys have
// any duplicates active in the network.
func (v *validator) CheckDoppelGanger(ctx context.Context) error {
	if !features.Get().EnableDoppelGanger {
		return nil
	}
	pubkeys, err := v.keyManager.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return err
	}
	log.WithField("keys", len(pubkeys)).Info("Running doppelganger check")
	// Exit early if no validating pub keys are found.
	if len(pubkeys) == 0 {
		return nil
	}
	req := &ethpb.DoppelGangerRequest{ValidatorRequests: []*ethpb.DoppelGangerRequest_ValidatorRequest{}}
	for _, pkey := range pubkeys {
		copiedKey := pkey
		attRec, err := v.db.AttestationHistoryForPubKey(ctx, copiedKey)
		if err != nil {
			return err
		}
		if len(attRec) == 0 {
			// If no history exists we simply send in a zero
			// value for the request epoch and root.
			req.ValidatorRequests = append(req.ValidatorRequests,
				&ethpb.DoppelGangerRequest_ValidatorRequest{
					PublicKey:  copiedKey[:],
					Epoch:      0,
					SignedRoot: make([]byte, fieldparams.RootLength),
				})
			continue
		}
		r := retrieveLatestRecord(attRec)
		if copiedKey != r.PubKey {
			return errors.New("attestation record mismatched public key")
		}
		req.ValidatorRequests = append(req.ValidatorRequests,
			&ethpb.DoppelGangerRequest_ValidatorRequest{
				PublicKey:  r.PubKey[:],
				Epoch:      r.Target,
				SignedRoot: r.SigningRoot,
			})
	}
	resp, err := v.validatorClient.CheckDoppelGanger(ctx, req)
	if err != nil {
		return err
	}
	// If nothing is returned by the beacon node, we return an
	// error as it is unsafe for us to proceed.
	if resp == nil || resp.Responses == nil || len(resp.Responses) == 0 {
		return errors.New("beacon node returned 0 responses for doppelganger check")
	}
	return buildDuplicateError(resp.Responses)
}

func buildDuplicateError(response []*ethpb.DoppelGangerResponse_ValidatorResponse) error {
	duplicates := make([][]byte, 0)
	for _, valRes := range response {
		if valRes.DuplicateExists {
			var copiedKey [fieldparams.BLSPubkeyLength]byte
			copy(copiedKey[:], valRes.PublicKey)
			duplicates = append(duplicates, copiedKey[:])
		}
	}
	if len(duplicates) == 0 {
		return nil
	}
	return errors.Errorf("Duplicate instances exists in the network for validator keys: %#x", duplicates)
}

// Ensures that the latest attestation history is retrieved.
func retrieveLatestRecord(recs []*kv.AttestationRecord) *kv.AttestationRecord {
	if len(recs) == 0 {
		return nil
	}
	lastSource := recs[len(recs)-1].Source
	chosenRec := recs[len(recs)-1]
	for i := len(recs) - 1; i >= 0; i-- {
		// Exit if we are now on a different source
		// as it is assumed that all source records are
		// byte sorted.
		if recs[i].Source != lastSource {
			break
		}
		// If we have a smaller target, we do
		// change our chosen record.
		if chosenRec.Target < recs[i].Target {
			chosenRec = recs[i]
		}
	}
	return chosenRec
}

// UpdateDuties checks the slot number to determine if the validator's
// list of upcoming assignments needs to be updated. For example, at the
// beginning of a new epoch.
func (v *validator) UpdateDuties(ctx context.Context, slot primitives.Slot) error {
	if slot%params.BeaconConfig().SlotsPerEpoch != 0 && v.duties != nil {
		// Do nothing if not epoch start AND assignments already exist.
		return nil
	}
	// Set deadline to end of epoch.
	ss, err := slots.EpochStart(slots.ToEpoch(slot) + 1)
	if err != nil {
		return err
	}
	ctx, cancel := context.WithDeadline(ctx, v.SlotDeadline(ss))
	defer cancel()
	ctx, span := trace.StartSpan(ctx, "validator.UpdateAssignments")
	defer span.End()

	validatingKeys, err := v.keyManager.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return err
	}

	// Filter out the slashable public keys from the duties request.
	filteredKeys := make([][fieldparams.BLSPubkeyLength]byte, 0, len(validatingKeys))
	v.slashableKeysLock.RLock()
	for _, pubKey := range validatingKeys {
		if ok := v.eipImportBlacklistedPublicKeys[pubKey]; !ok {
			filteredKeys = append(filteredKeys, pubKey)
		} else {
			log.WithField(
				"publicKey", fmt.Sprintf("%#x", bytesutil.Trunc(pubKey[:])),
			).Warn("Not including slashable public key from slashing protection import " +
				"in request to update validator duties")
		}
	}
	v.slashableKeysLock.RUnlock()

	req := &ethpb.DutiesRequest{
		Epoch:      primitives.Epoch(slot / params.BeaconConfig().SlotsPerEpoch),
		PublicKeys: bytesutil.FromBytes48Array(filteredKeys),
	}

	// If duties is nil it means we have had no prior duties and just started up.
	resp, err := v.validatorClient.GetDuties(ctx, req)
	if err != nil {
		v.dutiesLock.Lock()
		v.duties = nil // Clear assignments so we know to retry the request.
		v.dutiesLock.Unlock()
		log.WithError(err).Error("error getting validator duties")
		return err
	}

	v.dutiesLock.Lock()
	v.duties = resp
	v.logDuties(slot, v.duties.CurrentEpochDuties, v.duties.NextEpochDuties)
	v.dutiesLock.Unlock()

	allExitedCounter := 0
	for i := range resp.CurrentEpochDuties {
		if resp.CurrentEpochDuties[i].Status == ethpb.ValidatorStatus_EXITED {
			allExitedCounter++
		}
	}
	if allExitedCounter != 0 && allExitedCounter == len(resp.CurrentEpochDuties) {
		return ErrValidatorsAllExited
	}

	// Non-blocking call for beacon node to start subscriptions for aggregators.
	// Make sure to copy metadata into a new context
	md, exists := metadata.FromOutgoingContext(ctx)
	ctx = context.Background()
	if exists {
		ctx = metadata.NewOutgoingContext(ctx, md)
	}
	go func() {
		if err := v.subscribeToSubnets(ctx, resp); err != nil {
			log.WithError(err).Error("Failed to subscribe to subnets")
		}
	}()

	return nil
}

// subscribeToSubnets iterates through each validator duty, signs each slot, and asks beacon node
// to eagerly subscribe to subnets so that the aggregator has attestations to aggregate.
func (v *validator) subscribeToSubnets(ctx context.Context, res *ethpb.DutiesResponse) error {
	subscribeSlots := make([]primitives.Slot, 0, len(res.CurrentEpochDuties)+len(res.NextEpochDuties))
	subscribeCommitteeIndices := make([]primitives.CommitteeIndex, 0, len(res.CurrentEpochDuties)+len(res.NextEpochDuties))
	subscribeIsAggregator := make([]bool, 0, len(res.CurrentEpochDuties)+len(res.NextEpochDuties))
	subscribeValidatorIndices := make([]primitives.ValidatorIndex, 0, len(res.CurrentEpochDuties)+len(res.NextEpochDuties))
	alreadySubscribed := make(map[[64]byte]bool)

	for _, duty := range res.CurrentEpochDuties {
		pk := bytesutil.ToBytes48(duty.PublicKey)
		if duty.Status == ethpb.ValidatorStatus_ACTIVE || duty.Status == ethpb.ValidatorStatus_EXITING {
			attesterSlot := duty.AttesterSlot
			committeeIndex := duty.CommitteeIndex
			validatorIndex := duty.ValidatorIndex

			alreadySubscribedKey := validatorSubscribeKey(attesterSlot, committeeIndex)
			if _, ok := alreadySubscribed[alreadySubscribedKey]; ok {
				continue
			}

			aggregator, err := v.isAggregator(ctx, duty.Committee, attesterSlot, pk)
			if err != nil {
				return errors.Wrap(err, "could not check if a validator is an aggregator")
			}
			if aggregator {
				alreadySubscribed[alreadySubscribedKey] = true
			}

			subscribeSlots = append(subscribeSlots, attesterSlot)
			subscribeCommitteeIndices = append(subscribeCommitteeIndices, committeeIndex)
			subscribeIsAggregator = append(subscribeIsAggregator, aggregator)
			subscribeValidatorIndices = append(subscribeValidatorIndices, validatorIndex)
		}
	}

	for _, duty := range res.NextEpochDuties {
		if duty.Status == ethpb.ValidatorStatus_ACTIVE || duty.Status == ethpb.ValidatorStatus_EXITING {
			attesterSlot := duty.AttesterSlot
			committeeIndex := duty.CommitteeIndex
			validatorIndex := duty.ValidatorIndex

			alreadySubscribedKey := validatorSubscribeKey(attesterSlot, committeeIndex)
			if _, ok := alreadySubscribed[alreadySubscribedKey]; ok {
				continue
			}

			aggregator, err := v.isAggregator(ctx, duty.Committee, attesterSlot, bytesutil.ToBytes48(duty.PublicKey))
			if err != nil {
				return errors.Wrap(err, "could not check if a validator is an aggregator")
			}
			if aggregator {
				alreadySubscribed[alreadySubscribedKey] = true
			}

			subscribeSlots = append(subscribeSlots, attesterSlot)
			subscribeCommitteeIndices = append(subscribeCommitteeIndices, committeeIndex)
			subscribeIsAggregator = append(subscribeIsAggregator, aggregator)
			subscribeValidatorIndices = append(subscribeValidatorIndices, validatorIndex)
		}
	}

	_, err := v.validatorClient.SubscribeCommitteeSubnets(ctx,
		&ethpb.CommitteeSubnetsSubscribeRequest{
			Slots:        subscribeSlots,
			CommitteeIds: subscribeCommitteeIndices,
			IsAggregator: subscribeIsAggregator,
		},
		subscribeValidatorIndices,
	)

	return err
}

// RolesAt slot returns the validator roles at the given slot. Returns nil if the
// validator is known to not have a roles at the slot. Returns UNKNOWN if the
// validator assignments are unknown. Otherwise returns a valid ValidatorRole map.
func (v *validator) RolesAt(ctx context.Context, slot primitives.Slot) (map[[fieldparams.BLSPubkeyLength]byte][]iface.ValidatorRole, error) {
	v.dutiesLock.RLock()
	defer v.dutiesLock.RUnlock()
	rolesAt := make(map[[fieldparams.BLSPubkeyLength]byte][]iface.ValidatorRole)
	for validator, duty := range v.duties.CurrentEpochDuties {
		var roles []iface.ValidatorRole

		if duty == nil {
			continue
		}
		if len(duty.ProposerSlots) > 0 {
			for _, proposerSlot := range duty.ProposerSlots {
				if proposerSlot != 0 && proposerSlot == slot {
					roles = append(roles, iface.RoleProposer)
					break
				}
			}
		}
		if duty.AttesterSlot == slot {
			roles = append(roles, iface.RoleAttester)

			aggregator, err := v.isAggregator(ctx, duty.Committee, slot, bytesutil.ToBytes48(duty.PublicKey))
			if err != nil {
				return nil, errors.Wrap(err, "could not check if a validator is an aggregator")
			}
			if aggregator {
				roles = append(roles, iface.RoleAggregator)
			}
		}

		// Being assigned to a sync committee for a given slot means that the validator produces and
		// broadcasts signatures for `slot - 1` for inclusion in `slot`. At the last slot of the epoch,
		// the validator checks whether it's in the sync committee of following epoch.
		inSyncCommittee := false
		if slots.IsEpochEnd(slot) {
			if v.duties.NextEpochDuties[validator].IsSyncCommittee {
				roles = append(roles, iface.RoleSyncCommittee)
				inSyncCommittee = true
			}
		} else {
			if duty.IsSyncCommittee {
				roles = append(roles, iface.RoleSyncCommittee)
				inSyncCommittee = true
			}
		}
		if inSyncCommittee {
			aggregator, err := v.isSyncCommitteeAggregator(ctx, slot, bytesutil.ToBytes48(duty.PublicKey))
			if err != nil {
				return nil, errors.Wrap(err, "could not check if a validator is a sync committee aggregator")
			}
			if aggregator {
				roles = append(roles, iface.RoleSyncCommitteeAggregator)
			}
		}

		if len(roles) == 0 {
			roles = append(roles, iface.RoleUnknown)
		}

		var pubKey [fieldparams.BLSPubkeyLength]byte
		copy(pubKey[:], duty.PublicKey)
		rolesAt[pubKey] = roles
	}
	return rolesAt, nil
}

// Keymanager returns the underlying validator's keymanager.
func (v *validator) Keymanager() (keymanager.IKeymanager, error) {
	if v.keyManager == nil {
		return nil, errors.New("keymanager is not initialized")
	}
	return v.keyManager, nil
}

// isAggregator checks if a validator is an aggregator of a given slot and committee,
// it uses a modulo calculated by validator count in committee and samples randomness around it.
func (v *validator) isAggregator(ctx context.Context, committee []primitives.ValidatorIndex, slot primitives.Slot, pubKey [fieldparams.BLSPubkeyLength]byte) (bool, error) {
	modulo := uint64(1)
	if len(committee)/int(params.BeaconConfig().TargetAggregatorsPerCommittee) > 1 {
		modulo = uint64(len(committee)) / params.BeaconConfig().TargetAggregatorsPerCommittee
	}

	slotSig, err := v.signSlotWithSelectionProof(ctx, pubKey, slot)
	if err != nil {
		return false, err
	}

	b := hash.Hash(slotSig)

	return binary.LittleEndian.Uint64(b[:8])%modulo == 0, nil
}

// isSyncCommitteeAggregator checks if a validator in an aggregator of a subcommittee for sync committee.
// it uses a modulo calculated by validator count in committee and samples randomness around it.
//
// Spec code:
// def is_sync_committee_aggregator(signature: BLSSignature) -> bool:
//
//	modulo = max(1, SYNC_COMMITTEE_SIZE // SYNC_COMMITTEE_SUBNET_COUNT // TARGET_AGGREGATORS_PER_SYNC_SUBCOMMITTEE)
//	return bytes_to_uint64(hash(signature)[0:8]) % modulo == 0
func (v *validator) isSyncCommitteeAggregator(ctx context.Context, slot primitives.Slot, pubKey [fieldparams.BLSPubkeyLength]byte) (bool, error) {
	res, err := v.validatorClient.GetSyncSubcommitteeIndex(ctx, &ethpb.SyncSubcommitteeIndexRequest{
		PublicKey: pubKey[:],
		Slot:      slot,
	})
	if err != nil {
		return false, err
	}

	for _, index := range res.Indices {
		subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
		subnet := uint64(index) / subCommitteeSize
		sig, err := v.signSyncSelectionData(ctx, pubKey, subnet, slot)
		if err != nil {
			return false, err
		}
		isAggregator, err := altair.IsSyncCommitteeAggregator(sig)
		if err != nil {
			return false, err
		}
		if isAggregator {
			return true, nil
		}
	}

	return false, nil
}

// UpdateDomainDataCaches by making calls for all of the possible domain data. These can change when
// the fork version changes which can happen once per epoch. Although changing for the fork version
// is very rare, a validator should check these data every epoch to be sure the validator is
// participating on the correct fork version.
func (v *validator) UpdateDomainDataCaches(ctx context.Context, slot primitives.Slot) {
	for _, d := range [][]byte{
		params.BeaconConfig().DomainRandao[:],
		params.BeaconConfig().DomainBeaconAttester[:],
		params.BeaconConfig().DomainBeaconProposer[:],
		params.BeaconConfig().DomainSelectionProof[:],
		params.BeaconConfig().DomainAggregateAndProof[:],
		params.BeaconConfig().DomainSyncCommittee[:],
		params.BeaconConfig().DomainSyncCommitteeSelectionProof[:],
		params.BeaconConfig().DomainContributionAndProof[:],
	} {
		_, err := v.domainData(ctx, slots.ToEpoch(slot), d)
		if err != nil {
			log.WithError(err).Errorf("Failed to update domain data for domain %v", d)
		}
	}
}

func (v *validator) domainData(ctx context.Context, epoch primitives.Epoch, domain []byte) (*ethpb.DomainResponse, error) {
	v.domainDataLock.RLock()

	req := &ethpb.DomainRequest{
		Epoch:  epoch,
		Domain: domain,
	}

	key := strings.Join([]string{strconv.FormatUint(uint64(req.Epoch), 10), hex.EncodeToString(req.Domain)}, ",")

	if val, ok := v.domainDataCache.Get(key); ok {
		v.domainDataLock.RUnlock()
		return proto.Clone(val.(proto.Message)).(*ethpb.DomainResponse), nil
	}
	v.domainDataLock.RUnlock()

	// Lock as we are about to perform an expensive request to the beacon node.
	v.domainDataLock.Lock()
	defer v.domainDataLock.Unlock()

	// We check the cache again as in the event there are multiple inflight requests for
	// the same domain data, the cache might have been filled while we were waiting
	// to acquire the lock.
	if val, ok := v.domainDataCache.Get(key); ok {
		return proto.Clone(val.(proto.Message)).(*ethpb.DomainResponse), nil
	}

	res, err := v.validatorClient.DomainData(ctx, req)
	if err != nil {
		return nil, err
	}
	v.domainDataCache.Set(key, proto.Clone(res), 1)

	return res, nil
}

func (v *validator) logDuties(slot primitives.Slot, currentEpochDuties []*ethpb.DutiesResponse_Duty, nextEpochDuties []*ethpb.DutiesResponse_Duty) {
	attesterKeys := make([][]string, params.BeaconConfig().SlotsPerEpoch)
	for i := range attesterKeys {
		attesterKeys[i] = make([]string, 0)
	}
	proposerKeys := make([]string, params.BeaconConfig().SlotsPerEpoch)
	slotOffset := slot - (slot % params.BeaconConfig().SlotsPerEpoch)
	var totalAttestingKeys uint64
	for _, duty := range currentEpochDuties {
		validatorNotTruncatedKey := fmt.Sprintf("%#x", duty.PublicKey)
		if v.emitAccountMetrics {
			ValidatorStatusesGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(duty.Status))
		}

		// Only interested in validators who are attesting/proposing.
		// Note that SLASHING validators will have duties but their results are ignored by the network so we don't bother with them.
		if duty.Status != ethpb.ValidatorStatus_ACTIVE && duty.Status != ethpb.ValidatorStatus_EXITING {
			continue
		}

		validatorKey := fmt.Sprintf("%#x", bytesutil.Trunc(duty.PublicKey))
		attesterIndex := duty.AttesterSlot - slotOffset
		if attesterIndex >= params.BeaconConfig().SlotsPerEpoch {
			log.WithField("duty", duty).Warn("Invalid attester slot")
		} else {
			attesterKeys[duty.AttesterSlot-slotOffset] = append(attesterKeys[duty.AttesterSlot-slotOffset], validatorKey)
			totalAttestingKeys++
			if v.emitAccountMetrics {
				ValidatorNextAttestationSlotGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(duty.AttesterSlot))
			}
		}
		if v.emitAccountMetrics && duty.IsSyncCommittee {
			ValidatorInSyncCommitteeGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(1))
		} else if v.emitAccountMetrics && !duty.IsSyncCommittee {
			// clear the metric out if the validator is not in the current sync committee anymore otherwise it will be left at 1
			ValidatorInSyncCommitteeGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(0))
		}

		for _, proposerSlot := range duty.ProposerSlots {
			proposerIndex := proposerSlot - slotOffset
			if proposerIndex >= params.BeaconConfig().SlotsPerEpoch {
				log.WithField("duty", duty).Warn("Invalid proposer slot")
			} else {
				proposerKeys[proposerIndex] = validatorKey
			}
			if v.emitAccountMetrics {
				ValidatorNextProposalSlotGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(proposerSlot))
			}
		}
	}
	for _, duty := range nextEpochDuties {
		// for the next epoch, currently we are only interested in whether the validator is in the next sync committee or not
		validatorNotTruncatedKey := fmt.Sprintf("%#x", duty.PublicKey)

		// Only interested in validators who are attesting/proposing.
		// Note that slashed validators will have duties but their results are ignored by the network so we don't bother with them.
		if duty.Status != ethpb.ValidatorStatus_ACTIVE && duty.Status != ethpb.ValidatorStatus_EXITING {
			continue
		}

		if v.emitAccountMetrics && duty.IsSyncCommittee {
			ValidatorInNextSyncCommitteeGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(1))
		} else if v.emitAccountMetrics && !duty.IsSyncCommittee {
			// clear the metric out if the validator is now not in the next sync committee otherwise it will be left at 1
			ValidatorInNextSyncCommitteeGaugeVec.WithLabelValues(validatorNotTruncatedKey).Set(float64(0))
		}
	}
	for i := primitives.Slot(0); i < params.BeaconConfig().SlotsPerEpoch; i++ {
		startTime := slots.StartTime(v.genesisTime, slotOffset+i)
		durationTillDuty := (time.Until(startTime) + time.Second).Truncate(time.Second) // Round up to next second.

		if len(attesterKeys[i]) > 0 {
			attestationLog := log.WithFields(logrus.Fields{
				"slot":                  slotOffset + i,
				"slotInEpoch":           (slotOffset + i) % params.BeaconConfig().SlotsPerEpoch,
				"attesterDutiesAtSlot":  len(attesterKeys[i]),
				"totalAttestersInEpoch": totalAttestingKeys,
				"pubKeys":               attesterKeys[i],
			})
			if durationTillDuty > 0 {
				attestationLog = attestationLog.WithField("timeTillDuty", durationTillDuty)
			}
			attestationLog.Info("Attestation schedule")
		}
		if proposerKeys[i] != "" {
			proposerLog := log.WithField("slot", slotOffset+i).WithField("pubKey", proposerKeys[i])
			if durationTillDuty > 0 {
				proposerLog = proposerLog.WithField("timeTillDuty", durationTillDuty)
			}
			proposerLog.Info("Proposal schedule")
		}
	}
}

// ProposerSettings gets the current proposer settings saved in memory validator
func (v *validator) ProposerSettings() *validatorserviceconfig.ProposerSettings {
	return v.proposerSettings
}

// SetProposerSettings sets and saves the passed in proposer settings overriding the in memory one
func (v *validator) SetProposerSettings(ctx context.Context, settings *validatorserviceconfig.ProposerSettings) error {
	if v.db == nil {
		return errors.New("db is not set")
	}
	if err := v.db.SaveProposerSettings(ctx, settings); err != nil {
		return err
	}
	v.proposerSettings = settings
	return nil
}

// PushProposerSettings calls the prepareBeaconProposer RPC to set the fee recipient and also the register validator API if using a custom builder.
func (v *validator) PushProposerSettings(ctx context.Context, km keymanager.IKeymanager, slot primitives.Slot, deadline time.Time) error {
	if km == nil {
		return errors.New("keymanager is nil when calling PrepareBeaconProposer")
	}
	nctx, cancel := context.WithDeadline(ctx, deadline)
	ctx = nctx
	defer cancel()

	pubkeys, err := km.FetchValidatingPublicKeys(ctx)
	if err != nil {
		return err
	}
	if len(pubkeys) == 0 {
		log.Info("No imported public keys. Skipping prepare proposer routine")
		return nil
	}
	filteredKeys, err := v.filterAndCacheActiveKeys(ctx, pubkeys, slot)
	if err != nil {
		return err
	}
	proposerReqs, err := v.buildPrepProposerReqs(ctx, filteredKeys)
	if err != nil {
		return err
	}
	if len(proposerReqs) == 0 {
		log.Warnf("Could not locate valid validator indices. Skipping prepare proposer routine")
		return nil
	}
	if len(proposerReqs) != len(pubkeys) {
		log.WithFields(logrus.Fields{
			"pubkeysCount":             len(pubkeys),
			"proposerSettingsReqCount": len(proposerReqs),
		}).Debugln("Request count did not match included validator count. Only keys that have been activated will be included in the request.")
	}
	if _, err := v.validatorClient.PrepareBeaconProposer(ctx, &ethpb.PrepareBeaconProposerRequest{
		Recipients: proposerReqs,
	}); err != nil {
		return err
	}

	signedRegReqs := v.buildSignedRegReqs(ctx, filteredKeys, km.Sign)
	if err := SubmitValidatorRegistrations(ctx, v.validatorClient, signedRegReqs, v.validatorsRegBatchSize); err != nil {
		return errors.Wrap(ErrBuilderValidatorRegistration, err.Error())
	}
	return nil
}

func (v *validator) StartEventStream(ctx context.Context) error {
	return v.validatorClient.StartEventStream(ctx)
}

func (v *validator) EventStreamIsRunning() bool {
	return v.validatorClient.EventStreamIsRunning()
}

func (v *validator) NodeIsHealthy(ctx context.Context) bool {
	return v.nodeClient.IsHealthy(ctx)
}

func (v *validator) filterAndCacheActiveKeys(ctx context.Context, pubkeys [][fieldparams.BLSPubkeyLength]byte, slot primitives.Slot) ([][fieldparams.BLSPubkeyLength]byte, error) {
	filteredKeys := make([][fieldparams.BLSPubkeyLength]byte, 0)
	statusRequestKeys := make([][]byte, 0)
	for _, k := range pubkeys {
		_, ok := v.pubkeyToValidatorIndex[k]
		// Get validator index from RPC server if not found.
		if !ok {
			i, ok, err := v.validatorIndex(ctx, k)
			if err != nil {
				return nil, err
			}
			if !ok { // Nothing we can do if RPC server doesn't have validator index.
				continue
			}
			v.pubkeyToValidatorIndex[k] = i
		}
		copiedk := k
		statusRequestKeys = append(statusRequestKeys, copiedk[:])
	}
	resp, err := v.validatorClient.MultipleValidatorStatus(ctx, &ethpb.MultipleValidatorStatusRequest{
		PublicKeys: statusRequestKeys,
	})
	if err != nil {
		return nil, err
	}
	for i, status := range resp.Statuses {
		currEpoch := primitives.Epoch(slot / params.BeaconConfig().SlotsPerEpoch)
		currActivating := status.Status == ethpb.ValidatorStatus_PENDING && currEpoch >= status.ActivationEpoch

		active := status.Status == ethpb.ValidatorStatus_ACTIVE
		exiting := status.Status == ethpb.ValidatorStatus_EXITING

		if currActivating || active || exiting {
			filteredKeys = append(filteredKeys, bytesutil.ToBytes48(resp.PublicKeys[i]))
		} else {
			log.WithFields(logrus.Fields{
				"publickey": hexutil.Encode(resp.PublicKeys[i]),
				"status":    status.Status.String(),
			}).Debugf("skipping non active status key.")
		}
	}

	return filteredKeys, nil
}

func (v *validator) buildPrepProposerReqs(ctx context.Context, pubkeys [][fieldparams.BLSPubkeyLength]byte /* only active pubkeys */) ([]*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer, error) {
	var prepareProposerReqs []*ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer
	for _, k := range pubkeys {
		// Default case: Define fee recipient to burn address
		var feeRecipient common.Address

		// If fee recipient is defined in default configuration, use it
		if v.ProposerSettings() != nil && v.ProposerSettings().DefaultConfig != nil && v.ProposerSettings().DefaultConfig.FeeRecipientConfig != nil {
			feeRecipient = v.ProposerSettings().DefaultConfig.FeeRecipientConfig.FeeRecipient // Use cli config for fee recipient.
		}

		// If fee recipient is defined for this specific pubkey in proposer configuration, use it
		if v.ProposerSettings() != nil && v.ProposerSettings().ProposeConfig != nil {
			config, ok := v.ProposerSettings().ProposeConfig[k]

			if ok && config != nil && config.FeeRecipientConfig != nil {
				feeRecipient = config.FeeRecipientConfig.FeeRecipient // Use file config for fee recipient.
			}
		}

		validatorIndex, ok := v.pubkeyToValidatorIndex[k]
		if !ok {
			continue
		}

		prepareProposerReqs = append(prepareProposerReqs, &ethpb.PrepareBeaconProposerRequest_FeeRecipientContainer{
			ValidatorIndex: validatorIndex,
			FeeRecipient:   feeRecipient[:],
		})
	}
	return prepareProposerReqs, nil
}

func (v *validator) buildSignedRegReqs(ctx context.Context, pubkeys [][fieldparams.BLSPubkeyLength]byte /* only active pubkeys */, signer iface.SigningFunc) []*ethpb.SignedValidatorRegistrationV1 {
	var signedValRegRegs []*ethpb.SignedValidatorRegistrationV1
	if v.ProposerSettings() == nil {
		return signedValRegRegs
	}
	// if the timestamp is pre-genesis, don't create registrations
	if v.genesisTime > uint64(time.Now().UTC().Unix()) {
		return signedValRegRegs
	}
	for i, k := range pubkeys {
		feeRecipient := common.HexToAddress(params.BeaconConfig().EthBurnAddressHex)
		gasLimit := params.BeaconConfig().DefaultBuilderGasLimit
		enabled := false

		if v.ProposerSettings().DefaultConfig != nil && v.ProposerSettings().DefaultConfig.FeeRecipientConfig != nil {
			defaultConfig := v.ProposerSettings().DefaultConfig
			feeRecipient = defaultConfig.FeeRecipientConfig.FeeRecipient // Use cli defaultBuilderConfig for fee recipient.
			defaultBuilderConfig := defaultConfig.BuilderConfig

			if defaultBuilderConfig != nil && defaultBuilderConfig.Enabled {
				gasLimit = uint64(defaultBuilderConfig.GasLimit) // Use cli config for gas limit.
				enabled = true
			}
		}

		if v.ProposerSettings().ProposeConfig != nil {
			config, ok := v.ProposerSettings().ProposeConfig[k]
			if ok && config != nil && config.FeeRecipientConfig != nil {
				feeRecipient = config.FeeRecipientConfig.FeeRecipient // Use file config for fee recipient.
				builderConfig := config.BuilderConfig
				if builderConfig != nil {
					if builderConfig.Enabled {
						gasLimit = uint64(builderConfig.GasLimit) // Use file config for gas limit.
						enabled = true
					} else {
						enabled = false // Custom config can disable validator from register.
					}
				}
			}
		}

		if !enabled {
			continue
		}

		// map is populated before this function in buildPrepProposerReq
		_, ok := v.pubkeyToValidatorIndex[k]
		if !ok {
			continue
		}

		req := &ethpb.ValidatorRegistrationV1{
			FeeRecipient: feeRecipient[:],
			GasLimit:     gasLimit,
			Timestamp:    uint64(time.Now().UTC().Unix()),
			Pubkey:       pubkeys[i][:],
		}

		signedReq, err := v.SignValidatorRegistrationRequest(ctx, signer, req)
		if err != nil {
			log.WithFields(logrus.Fields{
				"pubkey":       fmt.Sprintf("%#x", req.Pubkey),
				"feeRecipient": feeRecipient,
			}).Error(err)
			continue
		}

		signedValRegRegs = append(signedValRegRegs, signedReq)

		if hexutil.Encode(feeRecipient.Bytes()) == params.BeaconConfig().EthBurnAddressHex {
			log.WithFields(logrus.Fields{
				"pubkey":       fmt.Sprintf("%#x", req.Pubkey),
				"feeRecipient": feeRecipient,
			}).Warn("Fee recipient is burn address")
		}
	}
	return signedValRegRegs
}

func (v *validator) validatorIndex(ctx context.Context, pubkey [fieldparams.BLSPubkeyLength]byte) (primitives.ValidatorIndex, bool, error) {
	resp, err := v.validatorClient.ValidatorIndex(ctx, &ethpb.ValidatorIndexRequest{PublicKey: pubkey[:]})
	switch {
	case status.Code(err) == codes.NotFound:
		log.Debugf("Could not find validator index for public key %#x. "+
			"Perhaps the validator is not yet active.", pubkey)
		return 0, false, nil
	case err != nil:
		notFoundErr := &beacon_api.IndexNotFoundError{}
		if errors.As(err, &notFoundErr) {
			log.Debugf("Could not find validator index for public key %#x. "+
				"Perhaps the validator is not yet active.", pubkey)
			return 0, false, nil
		}
		return 0, false, err
	}
	return resp.Index, true, nil
}

// This constructs a validator subscribed key, it's used to track
// which subnet has already been pending requested.
func validatorSubscribeKey(slot primitives.Slot, committeeID primitives.CommitteeIndex) [64]byte {
	return bytesutil.ToBytes64(append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(committeeID))...))
}

// This tracks all validators' voting status.
type voteStats struct {
	startEpoch          primitives.Epoch
	totalAttestedCount  uint64
	totalRequestedCount uint64
	totalDistance       primitives.Slot
	totalCorrectSource  uint64
	totalCorrectTarget  uint64
	totalCorrectHead    uint64
}

// This tracks all validators' submissions for sync committees.
type syncCommitteeStats struct {
	totalMessagesSubmitted uint64
}
