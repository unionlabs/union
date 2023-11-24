package app

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/binary"
	"encoding/hex"
	"errors"
	"fmt"
	"math/rand"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/protoio"
	cryptoproto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/version"
)

const (
	appVersion                 = 1
	voteExtensionKey    string = "extensionSum"
	voteExtensionMaxVal int64  = 128
	prefixReservedKey   string = "reservedTxKey_"
	suffixChainID       string = "ChainID"
	suffixVoteExtHeight string = "VoteExtensionsHeight"
	suffixInitialHeight string = "InitialHeight"
)

// Application is an ABCI application for use by end-to-end tests. It is a
// simple key/value store for strings, storing data in memory and persisting
// to disk as JSON, taking state sync snapshots if requested.
type Application struct {
	abci.BaseApplication
	logger          log.Logger
	state           *State
	snapshots       *SnapshotStore
	cfg             *Config
	restoreSnapshot *abci.Snapshot
	restoreChunks   [][]byte
}

// Config allows for the setting of high level parameters for running the e2e Application
// KeyType and ValidatorUpdates must be the same for all nodes running the same application.
type Config struct {
	// The directory with which state.json will be persisted in. Usually $HOME/.cometbft/data
	Dir string `toml:"dir"`

	// SnapshotInterval specifies the height interval at which the application
	// will take state sync snapshots. Defaults to 0 (disabled).
	SnapshotInterval uint64 `toml:"snapshot_interval"`

	// RetainBlocks specifies the number of recent blocks to retain. Defaults to
	// 0, which retains all blocks. Must be greater that PersistInterval,
	// SnapshotInterval and EvidenceAgeHeight.
	RetainBlocks uint64 `toml:"retain_blocks"`

	// KeyType sets the curve that will be used by validators.
	// Options are ed25519 & secp256k1
	KeyType string `toml:"key_type"`

	// PersistInterval specifies the height interval at which the application
	// will persist state to disk. Defaults to 1 (every height), setting this to
	// 0 disables state persistence.
	PersistInterval uint64 `toml:"persist_interval"`

	// ValidatorUpdates is a map of heights to validator names and their power,
	// and will be returned by the ABCI application. For example, the following
	// changes the power of validator01 and validator02 at height 1000:
	//
	// [validator_update.1000]
	// validator01 = 20
	// validator02 = 10
	//
	// Specifying height 0 returns the validator update during InitChain. The
	// application returns the validator updates as-is, i.e. removing a
	// validator must be done by returning it with power 0, and any validators
	// not specified are not changed.
	//
	// height <-> pubkey <-> voting power
	ValidatorUpdates map[string]map[string]uint8 `toml:"validator_update"`

	// Add artificial delays to each of the main ABCI calls to mimic computation time
	// of the application
	PrepareProposalDelay time.Duration `toml:"prepare_proposal_delay"`
	ProcessProposalDelay time.Duration `toml:"process_proposal_delay"`
	CheckTxDelay         time.Duration `toml:"check_tx_delay"`
	FinalizeBlockDelay   time.Duration `toml:"finalize_block_delay"`
	VoteExtensionDelay   time.Duration `toml:"vote_extension_delay"`
}

func DefaultConfig(dir string) *Config {
	return &Config{
		PersistInterval:  1,
		SnapshotInterval: 100,
		Dir:              dir,
	}
}

// NewApplication creates the application.
func NewApplication(cfg *Config) (*Application, error) {
	state, err := NewState(cfg.Dir, cfg.PersistInterval)
	if err != nil {
		return nil, err
	}
	snapshots, err := NewSnapshotStore(filepath.Join(cfg.Dir, "snapshots"))
	if err != nil {
		return nil, err
	}
	return &Application{
		logger:    log.NewTMLogger(log.NewSyncWriter(os.Stdout)),
		state:     state,
		snapshots: snapshots,
		cfg:       cfg,
	}, nil
}

// Info implements ABCI.
func (app *Application) Info(context.Context, *abci.RequestInfo) (*abci.ResponseInfo, error) {
	height, hash := app.state.Info()
	return &abci.ResponseInfo{
		Version:          version.ABCIVersion,
		AppVersion:       appVersion,
		LastBlockHeight:  int64(height),
		LastBlockAppHash: hash,
	}, nil
}

// Info implements ABCI.
func (app *Application) InitChain(_ context.Context, req *abci.RequestInitChain) (*abci.ResponseInitChain, error) {
	var err error
	app.state.initialHeight = uint64(req.InitialHeight)
	if len(req.AppStateBytes) > 0 {
		err = app.state.Import(0, req.AppStateBytes)
		if err != nil {
			return nil, err
		}
	}
	app.logger.Info("setting ChainID in app_state", "chainId", req.ChainId)
	app.state.Set(prefixReservedKey+suffixChainID, req.ChainId)
	app.logger.Info("setting VoteExtensionsHeight in app_state", "height", req.ConsensusParams.Abci.VoteExtensionsEnableHeight)
	app.state.Set(prefixReservedKey+suffixVoteExtHeight, strconv.FormatInt(req.ConsensusParams.Abci.VoteExtensionsEnableHeight, 10))
	app.logger.Info("setting initial height in app_state", "initial_height", req.InitialHeight)
	app.state.Set(prefixReservedKey+suffixInitialHeight, strconv.FormatInt(req.InitialHeight, 10))
	// Get validators from genesis
	if req.Validators != nil {
		for _, val := range req.Validators {
			val := val
			if err := app.storeValidator(&val); err != nil {
				return nil, err
			}
		}
	}
	resp := &abci.ResponseInitChain{
		AppHash: app.state.GetHash(),
	}
	if resp.Validators, err = app.validatorUpdates(0); err != nil {
		return nil, err
	}
	return resp, nil
}

// CheckTx implements ABCI.
func (app *Application) CheckTx(_ context.Context, req *abci.RequestCheckTx) (*abci.ResponseCheckTx, error) {
	key, _, err := parseTx(req.Tx)
	if err != nil || key == prefixReservedKey {
		return &abci.ResponseCheckTx{
			Code: kvstore.CodeTypeEncodingError,
			Log:  err.Error(),
		}, nil
	}

	if app.cfg.CheckTxDelay != 0 {
		time.Sleep(app.cfg.CheckTxDelay)
	}

	return &abci.ResponseCheckTx{Code: kvstore.CodeTypeOK, GasWanted: 1}, nil
}

// FinalizeBlock implements ABCI.
func (app *Application) FinalizeBlock(_ context.Context, req *abci.RequestFinalizeBlock) (*abci.ResponseFinalizeBlock, error) {
	txs := make([]*abci.ExecTxResult, len(req.Txs))

	for i, tx := range req.Txs {
		key, value, err := parseTx(tx)
		if err != nil {
			panic(err) // shouldn't happen since we verified it in CheckTx and ProcessProposal
		}
		if key == prefixReservedKey {
			panic(fmt.Errorf("detected a transaction with key %q; this key is reserved and should have been filtered out", prefixReservedKey))
		}
		app.state.Set(key, value)

		txs[i] = &abci.ExecTxResult{Code: kvstore.CodeTypeOK}
	}

	valUpdates, err := app.validatorUpdates(uint64(req.Height))
	if err != nil {
		panic(err)
	}

	if app.cfg.FinalizeBlockDelay != 0 {
		time.Sleep(app.cfg.FinalizeBlockDelay)
	}

	return &abci.ResponseFinalizeBlock{
		TxResults:        txs,
		ValidatorUpdates: valUpdates,
		AppHash:          app.state.Finalize(),
		Events: []abci.Event{
			{
				Type: "val_updates",
				Attributes: []abci.EventAttribute{
					{
						Key:   "size",
						Value: strconv.Itoa(valUpdates.Len()),
					},
					{
						Key:   "height",
						Value: strconv.Itoa(int(req.Height)),
					},
				},
			},
		},
	}, nil
}

// Commit implements ABCI.
func (app *Application) Commit(_ context.Context, _ *abci.RequestCommit) (*abci.ResponseCommit, error) {
	height, err := app.state.Commit()
	if err != nil {
		panic(err)
	}
	if app.cfg.SnapshotInterval > 0 && height%app.cfg.SnapshotInterval == 0 {
		snapshot, err := app.snapshots.Create(app.state)
		if err != nil {
			panic(err)
		}
		app.logger.Info("created state sync snapshot", "height", snapshot.Height)
		err = app.snapshots.Prune(maxSnapshotCount)
		if err != nil {
			app.logger.Error("failed to prune snapshots", "err", err)
		}
	}
	retainHeight := int64(0)
	if app.cfg.RetainBlocks > 0 {
		retainHeight = int64(height - app.cfg.RetainBlocks + 1)
	}
	return &abci.ResponseCommit{
		RetainHeight: retainHeight,
	}, nil
}

// Query implements ABCI.
func (app *Application) Query(_ context.Context, req *abci.RequestQuery) (*abci.ResponseQuery, error) {
	value, height := app.state.Query(string(req.Data))
	return &abci.ResponseQuery{
		Height: int64(height),
		Key:    req.Data,
		Value:  []byte(value),
	}, nil
}

// ListSnapshots implements ABCI.
func (app *Application) ListSnapshots(context.Context, *abci.RequestListSnapshots) (*abci.ResponseListSnapshots, error) {
	snapshots, err := app.snapshots.List()
	if err != nil {
		panic(err)
	}
	return &abci.ResponseListSnapshots{Snapshots: snapshots}, nil
}

// LoadSnapshotChunk implements ABCI.
func (app *Application) LoadSnapshotChunk(_ context.Context, req *abci.RequestLoadSnapshotChunk) (*abci.ResponseLoadSnapshotChunk, error) {
	chunk, err := app.snapshots.LoadChunk(req.Height, req.Format, req.Chunk)
	if err != nil {
		panic(err)
	}
	return &abci.ResponseLoadSnapshotChunk{Chunk: chunk}, nil
}

// OfferSnapshot implements ABCI.
func (app *Application) OfferSnapshot(_ context.Context, req *abci.RequestOfferSnapshot) (*abci.ResponseOfferSnapshot, error) {
	if app.restoreSnapshot != nil {
		panic("A snapshot is already being restored")
	}
	app.restoreSnapshot = req.Snapshot
	app.restoreChunks = [][]byte{}
	return &abci.ResponseOfferSnapshot{Result: abci.ResponseOfferSnapshot_ACCEPT}, nil
}

// ApplySnapshotChunk implements ABCI.
func (app *Application) ApplySnapshotChunk(_ context.Context, req *abci.RequestApplySnapshotChunk) (*abci.ResponseApplySnapshotChunk, error) {
	if app.restoreSnapshot == nil {
		panic("No restore in progress")
	}
	app.restoreChunks = append(app.restoreChunks, req.Chunk)
	if len(app.restoreChunks) == int(app.restoreSnapshot.Chunks) {
		bz := []byte{}
		for _, chunk := range app.restoreChunks {
			bz = append(bz, chunk...)
		}
		err := app.state.Import(app.restoreSnapshot.Height, bz)
		if err != nil {
			panic(err)
		}
		app.restoreSnapshot = nil
		app.restoreChunks = nil
	}
	return &abci.ResponseApplySnapshotChunk{Result: abci.ResponseApplySnapshotChunk_ACCEPT}, nil
}

// PrepareProposal will take the given transactions and attempt to prepare a
// proposal from them when it's our turn to do so. If the current height has
// vote extension enabled, this method will use vote extensions from the previous
// height, passed from CometBFT as parameters to construct a special transaction
// whose value is the sum of all of the vote extensions from the previous round.
//
// Additionally, we verify the vote extension signatures passed from CometBFT and
// include all data necessary for such verification in the special transaction's
// payload so that ProcessProposal at other nodes can also verify the proposer
// constructed the special transaction correctly.
//
// If vote extensions are enabled for the current height, PrepareProposal makes
// sure there was at least one non-empty vote extension whose signature it could verify.
// If vote extensions are not enabled for the current height, PrepareProposal makes
// sure non-empty vote extensions are not present.
//
// The special vote extension-generated transaction must fit within an empty block
// and takes precedence over all other transactions coming from the mempool.
func (app *Application) PrepareProposal(
	_ context.Context, req *abci.RequestPrepareProposal,
) (*abci.ResponsePrepareProposal, error) {
	_, areExtensionsEnabled := app.checkHeightAndExtensions(true, req.Height, "PrepareProposal")

	txs := make([][]byte, 0, len(req.Txs)+1)
	var totalBytes int64
	extTxPrefix := fmt.Sprintf("%s=", voteExtensionKey)
	sum, err := app.verifyAndSum(areExtensionsEnabled, req.Height, &req.LocalLastCommit, "prepare_proposal")
	if err != nil {
		panic(fmt.Errorf("failed to sum and verify in PrepareProposal; err %w", err))
	}
	if areExtensionsEnabled {
		extCommitBytes, err := req.LocalLastCommit.Marshal()
		if err != nil {
			panic("unable to marshall extended commit")
		}
		extCommitHex := hex.EncodeToString(extCommitBytes)
		extTx := []byte(fmt.Sprintf("%s%d|%s", extTxPrefix, sum, extCommitHex))
		extTxLen := int64(len(extTx))
		app.logger.Info("preparing proposal with special transaction from vote extensions", "extTxLen", extTxLen)
		if extTxLen > req.MaxTxBytes {
			panic(fmt.Errorf("serious problem in the e2e app configuration; "+
				"the tx conveying the vote extension data does not fit in an empty block(%d > %d); "+
				"please review the app's configuration",
				extTxLen, req.MaxTxBytes))
		}
		txs = append(txs, extTx)
		// Coherence: No need to call parseTx, as the check is stateless and has been performed by CheckTx
		totalBytes = extTxLen
	}
	for _, tx := range req.Txs {
		if areExtensionsEnabled && strings.HasPrefix(string(tx), extTxPrefix) {
			// When vote extensions are enabled, our generated transaction takes precedence
			// over any supplied transaction that attempts to modify the "extensionSum" value.
			continue
		}
		if strings.HasPrefix(string(tx), prefixReservedKey) {
			app.logger.Error("detected tx that should not come from the mempool", "tx", tx)
			continue
		}
		if totalBytes+int64(len(tx)) > req.MaxTxBytes {
			break
		}
		totalBytes += int64(len(tx))
		// Coherence: No need to call parseTx, as the check is stateless and has been performed by CheckTx
		txs = append(txs, tx)
	}

	if app.cfg.PrepareProposalDelay != 0 {
		time.Sleep(app.cfg.PrepareProposalDelay)
	}

	return &abci.ResponsePrepareProposal{Txs: txs}, nil
}

// ProcessProposal implements part of the Application interface.
// It accepts any proposal that does not contain a malformed transaction.
// NOTE It is up to real Applications to effect punitive behavior in the cases ProcessProposal
// returns ResponseProcessProposal_REJECT, as it is evidence of misbehavior.
func (app *Application) ProcessProposal(_ context.Context, req *abci.RequestProcessProposal) (*abci.ResponseProcessProposal, error) {
	_, areExtensionsEnabled := app.checkHeightAndExtensions(true, req.Height, "ProcessProposal")

	for _, tx := range req.Txs {
		k, v, err := parseTx(tx)
		if err != nil {
			app.logger.Error("malformed transaction in ProcessProposal", "tx", tx, "err", err)
			return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}, nil
		}
		switch {
		case areExtensionsEnabled && k == voteExtensionKey:
			// Additional check for vote extension-related txs
			if err := app.verifyExtensionTx(req.Height, v); err != nil {
				app.logger.Error("vote extension transaction failed verification, rejecting proposal", k, v, "err", err)
				return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}, nil
			}
		case strings.HasPrefix(k, prefixReservedKey):
			app.logger.Error("key prefix %q is reserved and cannot be used in transactions, rejecting proposal", k)
			return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_REJECT}, nil
		}
	}

	if app.cfg.ProcessProposalDelay != 0 {
		time.Sleep(app.cfg.ProcessProposalDelay)
	}

	return &abci.ResponseProcessProposal{Status: abci.ResponseProcessProposal_ACCEPT}, nil
}

// ExtendVote will produce vote extensions in the form of random numbers to
// demonstrate vote extension nondeterminism.
//
// In the next block, if there are any vote extensions from the previous block,
// a new transaction will be proposed that updates a special value in the
// key/value store ("extensionSum") with the sum of all of the numbers collected
// from the vote extensions.
func (app *Application) ExtendVote(_ context.Context, req *abci.RequestExtendVote) (*abci.ResponseExtendVote, error) {
	appHeight, areExtensionsEnabled := app.checkHeightAndExtensions(false, req.Height, "ExtendVote")
	if !areExtensionsEnabled {
		panic(fmt.Errorf("received call to ExtendVote at height %d, when vote extensions are disabled", appHeight))
	}

	ext := make([]byte, binary.MaxVarintLen64)
	// We don't care that these values are generated by a weak random number
	// generator. It's just for test purposes.
	//nolint:gosec // G404: Use of weak random number generator
	num := rand.Int63n(voteExtensionMaxVal)
	extLen := binary.PutVarint(ext, num)

	if app.cfg.VoteExtensionDelay != 0 {
		time.Sleep(app.cfg.VoteExtensionDelay)
	}

	app.logger.Info("generated vote extension", "num", num, "ext", fmt.Sprintf("%x", ext[:extLen]), "height", appHeight)
	return &abci.ResponseExtendVote{
		VoteExtension: ext[:extLen],
	}, nil
}

// VerifyVoteExtension simply validates vote extensions from other validators
// without doing anything about them. In this case, it just makes sure that the
// vote extension is a well-formed integer value.
func (app *Application) VerifyVoteExtension(_ context.Context, req *abci.RequestVerifyVoteExtension) (*abci.ResponseVerifyVoteExtension, error) {
	appHeight, areExtensionsEnabled := app.checkHeightAndExtensions(false, req.Height, "VerifyVoteExtension")
	if !areExtensionsEnabled {
		panic(fmt.Errorf("received call to VerifyVoteExtension at height %d, when vote extensions are disabled", appHeight))
	}
	// We don't allow vote extensions to be optional
	if len(req.VoteExtension) == 0 {
		app.logger.Error("received empty vote extension")
		return &abci.ResponseVerifyVoteExtension{
			Status: abci.ResponseVerifyVoteExtension_REJECT,
		}, nil
	}

	num, err := parseVoteExtension(req.VoteExtension)
	if err != nil {
		app.logger.Error("failed to parse vote extension", "vote_extension", req.VoteExtension, "err", err)
		return &abci.ResponseVerifyVoteExtension{
			Status: abci.ResponseVerifyVoteExtension_REJECT,
		}, nil
	}

	if app.cfg.VoteExtensionDelay != 0 {
		time.Sleep(app.cfg.VoteExtensionDelay)
	}

	app.logger.Info("verified vote extension value", "height", req.Height, "vote_extension", req.VoteExtension, "num", num)
	return &abci.ResponseVerifyVoteExtension{
		Status: abci.ResponseVerifyVoteExtension_ACCEPT,
	}, nil
}

func (app *Application) Rollback() error {
	return app.state.Rollback()
}

func (app *Application) getAppHeight() int64 {
	initialHeightStr, height := app.state.Query(prefixReservedKey + suffixInitialHeight)
	if len(initialHeightStr) == 0 {
		panic("initial height not set in database")
	}
	initialHeight, err := strconv.ParseInt(initialHeightStr, 10, 64)
	if err != nil {
		panic(fmt.Errorf("malformed initial height %q in database", initialHeightStr))
	}

	appHeight := int64(height)
	if appHeight == 0 {
		appHeight = initialHeight - 1
	}
	return appHeight + 1
}

func (app *Application) checkHeightAndExtensions(isPrepareProcessProposal bool, height int64, callsite string) (int64, bool) {
	appHeight := app.getAppHeight()
	if height != appHeight {
		panic(fmt.Errorf(
			"got unexpected height in %s request; expected %d, actual %d",
			callsite, appHeight, height,
		))
	}

	voteExtHeightStr := app.state.Get(prefixReservedKey + suffixVoteExtHeight)
	if len(voteExtHeightStr) == 0 {
		panic("vote extension height not set in database")
	}
	voteExtHeight, err := strconv.ParseInt(voteExtHeightStr, 10, 64)
	if err != nil {
		panic(fmt.Errorf("malformed vote extension height %q in database", voteExtHeightStr))
	}
	currentHeight := appHeight
	if isPrepareProcessProposal {
		currentHeight-- // at exactly voteExtHeight, PrepareProposal still has no extensions, see RFC100
	}

	return appHeight, voteExtHeight != 0 && currentHeight >= voteExtHeight
}

func (app *Application) storeValidator(valUpdate *abci.ValidatorUpdate) error {
	// Store validator data to verify extensions
	pubKey, err := cryptoenc.PubKeyFromProto(valUpdate.PubKey)
	if err != nil {
		return err
	}
	addr := pubKey.Address().String()
	if valUpdate.Power > 0 {
		pubKeyBytes, err := valUpdate.PubKey.Marshal()
		if err != nil {
			return err
		}
		app.logger.Info("setting validator in app_state", "addr", addr)
		app.state.Set(prefixReservedKey+addr, hex.EncodeToString(pubKeyBytes))
	}
	return nil
}

// validatorUpdates generates a validator set update.
func (app *Application) validatorUpdates(height uint64) (abci.ValidatorUpdates, error) {
	updates := app.cfg.ValidatorUpdates[fmt.Sprintf("%v", height)]
	if len(updates) == 0 {
		return nil, nil
	}

	valUpdates := abci.ValidatorUpdates{}
	for keyString, power := range updates {

		keyBytes, err := base64.StdEncoding.DecodeString(keyString)
		if err != nil {
			return nil, fmt.Errorf("invalid base64 pubkey value %q: %w", keyString, err)
		}
		valUpdate := abci.UpdateValidator(keyBytes, int64(power), app.cfg.KeyType)
		valUpdates = append(valUpdates, valUpdate)
		if err := app.storeValidator(&valUpdate); err != nil {
			return nil, err
		}
	}
	return valUpdates, nil
}

// parseTx parses a tx in 'key=value' format into a key and value.
func parseTx(tx []byte) (string, string, error) {
	parts := bytes.Split(tx, []byte("="))
	if len(parts) != 2 {
		return "", "", fmt.Errorf("invalid tx format: %q", string(tx))
	}
	if len(parts[0]) == 0 {
		return "", "", errors.New("key cannot be empty")
	}
	return string(parts[0]), string(parts[1]), nil
}

func (app *Application) verifyAndSum(
	areExtensionsEnabled bool,
	currentHeight int64,
	extCommit *abci.ExtendedCommitInfo,
	callsite string,
) (int64, error) {
	var sum int64
	var extCount int
	for _, vote := range extCommit.Votes {
		if vote.BlockIdFlag == cmtproto.BlockIDFlagUnknown || vote.BlockIdFlag > cmtproto.BlockIDFlagNil {
			return 0, fmt.Errorf("vote with bad blockID flag value at height %d; blockID flag %d", currentHeight, vote.BlockIdFlag)
		}
		if vote.BlockIdFlag == cmtproto.BlockIDFlagAbsent || vote.BlockIdFlag == cmtproto.BlockIDFlagNil {
			if len(vote.VoteExtension) != 0 {
				return 0, fmt.Errorf("non-empty vote extension at height %d, for a vote with blockID  flag %d",
					currentHeight, vote.BlockIdFlag)
			}
			if len(vote.ExtensionSignature) != 0 {
				return 0, fmt.Errorf("non-empty vote extension signature at height %d, for a vote with blockID flag %d",
					currentHeight, vote.BlockIdFlag)
			}
			// Only interested in votes that can have extensions
			continue
		}
		if !areExtensionsEnabled {
			if len(vote.VoteExtension) != 0 {
				return 0, fmt.Errorf("non-empty vote extension at height %d, which has extensions disabled",
					currentHeight)
			}
			if len(vote.ExtensionSignature) != 0 {
				return 0, fmt.Errorf("non-empty vote extension signature at height %d, which has extensions disabled",
					currentHeight)
			}
			continue
		}
		if len(vote.VoteExtension) == 0 {
			return 0, fmt.Errorf("received empty vote extension from %X at height %d (extensions enabled); "+
				"e2e app's logic does not allow it", vote.Validator, currentHeight)
		}
		// Vote extension signatures are always provided. Apps can use them to verify the integrity of extensions
		if len(vote.ExtensionSignature) == 0 {
			return 0, fmt.Errorf("empty vote extension signature at height %d (extensions enabled)", currentHeight)
		}

		// Reconstruct vote extension's signed bytes...
		chainID := app.state.Get(prefixReservedKey + suffixChainID)
		if len(chainID) == 0 {
			panic("chainID not set in database")
		}
		cve := cmtproto.CanonicalVoteExtension{
			Extension: vote.VoteExtension,
			Height:    currentHeight - 1, // the vote extension was signed in the previous height
			Round:     int64(extCommit.Round),
			ChainId:   chainID,
		}
		extSignBytes, err := protoio.MarshalDelimited(&cve)
		if err != nil {
			return 0, fmt.Errorf("error when marshaling signed bytes: %w", err)
		}

		//... and verify
		valAddr := crypto.Address(vote.Validator.Address).String()
		pubKeyHex := app.state.Get(prefixReservedKey + valAddr)
		if len(pubKeyHex) == 0 {
			return 0, fmt.Errorf("received vote from unknown validator with address %q", valAddr)
		}
		pubKeyBytes, err := hex.DecodeString(pubKeyHex)
		if err != nil {
			return 0, fmt.Errorf("could not hex-decode public key for validator address %s, err %w", valAddr, err)
		}
		var pubKeyProto cryptoproto.PublicKey
		err = pubKeyProto.Unmarshal(pubKeyBytes)
		if err != nil {
			return 0, fmt.Errorf("unable to unmarshal public key for validator address %s, err %w", valAddr, err)
		}
		pubKey, err := cryptoenc.PubKeyFromProto(pubKeyProto)
		if err != nil {
			return 0, fmt.Errorf("could not obtain a public key from its proto for validator address %s, err %w", valAddr, err)
		}
		if !pubKey.VerifySignature(extSignBytes, vote.ExtensionSignature) {
			return 0, errors.New("received vote with invalid signature")
		}

		extValue, err := parseVoteExtension(vote.VoteExtension)
		// The extension's format should have been verified in VerifyVoteExtension
		if err != nil {
			return 0, fmt.Errorf("failed to parse vote extension: %w", err)
		}
		app.logger.Info(
			"received and verified vote extension value",
			"height", currentHeight,
			"valAddr", valAddr,
			"value", extValue,
			"callsite", callsite,
		)
		sum += extValue
		extCount++
	}

	if areExtensionsEnabled && (extCount == 0) {
		return 0, errors.New("bad extension data, at least one extended vote should be present when extensions are enabled")
	}
	return sum, nil
}

// verifyExtensionTx parses and verifies the payload of a vote extension-generated tx
func (app *Application) verifyExtensionTx(height int64, payload string) error {
	parts := strings.Split(payload, "|")
	if len(parts) != 2 {
		return fmt.Errorf("invalid payload format")
	}
	expSumStr := parts[0]
	if len(expSumStr) == 0 {
		return fmt.Errorf("sum cannot be empty in vote extension payload")
	}

	expSum, err := strconv.Atoi(expSumStr)
	if err != nil {
		return fmt.Errorf("malformed sum %q in vote extension payload", expSumStr)
	}

	extCommitHex := parts[1]
	if len(extCommitHex) == 0 {
		return fmt.Errorf("extended commit data cannot be empty in vote extension payload")
	}

	extCommitBytes, err := hex.DecodeString(extCommitHex)
	if err != nil {
		return fmt.Errorf("could not hex-decode vote extension payload")
	}

	var extCommit abci.ExtendedCommitInfo
	if extCommit.Unmarshal(extCommitBytes) != nil {
		return fmt.Errorf("unable to unmarshal extended commit")
	}

	sum, err := app.verifyAndSum(true, height, &extCommit, "process_proposal")
	if err != nil {
		return fmt.Errorf("failed to sum and verify in process proposal: %w", err)
	}

	// Final check that the proposer behaved correctly
	if int64(expSum) != sum {
		return fmt.Errorf("sum is not consistent with vote extension payload: %d!=%d", expSum, sum)
	}
	return nil
}

// parseVoteExtension attempts to parse the given extension data into a positive
// integer value.
func parseVoteExtension(ext []byte) (int64, error) {
	num, errVal := binary.Varint(ext)
	if errVal == 0 {
		return 0, errors.New("vote extension is too small to parse")
	}
	if errVal < 0 {
		return 0, errors.New("vote extension value is too large")
	}
	if num >= voteExtensionMaxVal {
		return 0, fmt.Errorf("vote extension value must be smaller than %d (was %d)", voteExtensionMaxVal, num)
	}
	return num, nil
}
