package kvstore

import (
	"bytes"
	"context"
	"encoding/base64"
	"encoding/binary"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"

	dbm "github.com/cometbft/cometbft-db"

	"github.com/cometbft/cometbft/abci/types"
	cryptoencoding "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/libs/log"
	cryptoproto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	"github.com/cometbft/cometbft/version"
)

var (
	stateKey        = []byte("stateKey")
	kvPairPrefixKey = []byte("kvPairKey:")
)

const (
	ValidatorPrefix        = "val="
	AppVersion      uint64 = 1
)

var _ types.Application = (*Application)(nil)

// Application is the kvstore state machine. It complies with the abci.Application interface.
// It takes transactions in the form of key=value and saves them in a database. This is
// a somewhat trivial example as there is no real state execution
type Application struct {
	types.BaseApplication

	state        State
	RetainBlocks int64 // blocks to retain after commit (via ResponseCommit.RetainHeight)
	stagedTxs    [][]byte
	logger       log.Logger

	// validator set
	valUpdates         []types.ValidatorUpdate
	valAddrToPubKeyMap map[string]cryptoproto.PublicKey

	// If true, the app will generate block events in BeginBlock. Used to test the event indexer
	// Should be false by default to avoid generating too much data.
	genBlockEvents bool
}

// NewApplication creates an instance of the kvstore from the provided database
func NewApplication(db dbm.DB) *Application {
	return &Application{
		logger:             log.NewNopLogger(),
		state:              loadState(db),
		valAddrToPubKeyMap: make(map[string]cryptoproto.PublicKey),
	}
}

// NewPersistentApplication creates a new application using the goleveldb database engine
func NewPersistentApplication(dbDir string) *Application {
	name := "kvstore"
	db, err := dbm.NewGoLevelDB(name, dbDir)
	if err != nil {
		panic(fmt.Errorf("failed to create persistent app at %s: %w", dbDir, err))
	}
	return NewApplication(db)
}

// NewInMemoryApplication creates a new application from an in memory database.
// Nothing will be persisted.
func NewInMemoryApplication() *Application {
	return NewApplication(dbm.NewMemDB())
}

func (app *Application) SetGenBlockEvents() {
	app.genBlockEvents = true
}

// Info returns information about the state of the application. This is generally used everytime a Tendermint instance
// begins and let's the application know what Tendermint versions it's interacting with. Based from this information,
// Tendermint will ensure it is in sync with the application by potentially replaying the blocks it has. If the
// Application returns a 0 appBlockHeight, Tendermint will call InitChain to initialize the application with consensus related data
func (app *Application) Info(context.Context, *types.RequestInfo) (*types.ResponseInfo, error) {
	// Tendermint expects the application to persist validators, on start-up we need to reload them to memory if they exist
	if len(app.valAddrToPubKeyMap) == 0 && app.state.Height > 0 {
		validators := app.getValidators()
		for _, v := range validators {
			pubkey, err := cryptoencoding.PubKeyFromProto(v.PubKey)
			if err != nil {
				panic(fmt.Errorf("can't decode public key: %w", err))
			}
			app.valAddrToPubKeyMap[string(pubkey.Address())] = v.PubKey
		}
	}

	return &types.ResponseInfo{
		Data:             fmt.Sprintf("{\"size\":%v}", app.state.Size),
		Version:          version.ABCIVersion,
		AppVersion:       AppVersion,
		LastBlockHeight:  app.state.Height,
		LastBlockAppHash: app.state.Hash(),
	}, nil
}

// InitChain takes the genesis validators and stores them in the kvstore. It returns the application hash in the
// case that the application starts prepopulated with values. This method is called whenever a new instance of the application
// starts (i.e. app height = 0).
func (app *Application) InitChain(_ context.Context, req *types.RequestInitChain) (*types.ResponseInitChain, error) {
	for _, v := range req.Validators {
		app.updateValidator(v)
	}
	appHash := make([]byte, 8)
	binary.PutVarint(appHash, app.state.Size)
	return &types.ResponseInitChain{
		AppHash: appHash,
	}, nil
}

// CheckTx handles inbound transactions or in the case of recheckTx assesses old transaction validity after a state transition.
// As this is called frequently, it's preferably to keep the check as stateless and as quick as possible.
// Here we check that the transaction has the correctly key=value format.
// For the KVStore we check that each transaction has the valid tx format:
// - Contains one and only one `=`
// - `=` is not the first or last byte.
// - if key is `val` that the validator update transaction is also valid
func (app *Application) CheckTx(_ context.Context, req *types.RequestCheckTx) (*types.ResponseCheckTx, error) {
	// If it is a validator update transaction, check that it is correctly formatted
	if isValidatorTx(req.Tx) {
		if _, _, err := parseValidatorTx(req.Tx); err != nil {
			return &types.ResponseCheckTx{Code: CodeTypeInvalidTxFormat}, nil
		}
	} else if !isValidTx(req.Tx) {
		return &types.ResponseCheckTx{Code: CodeTypeInvalidTxFormat}, nil
	}

	return &types.ResponseCheckTx{Code: CodeTypeOK, GasWanted: 1}, nil
}

// Tx must have a format like key:value or key=value. That is:
// - it must have one and only one ":" or "="
// - It must not begin or end with these special characters
func isValidTx(tx []byte) bool {
	if bytes.Count(tx, []byte(":")) == 1 && bytes.Count(tx, []byte("=")) == 0 {
		if !bytes.HasPrefix(tx, []byte(":")) && !bytes.HasSuffix(tx, []byte(":")) {
			return true
		}
	} else if bytes.Count(tx, []byte("=")) == 1 && bytes.Count(tx, []byte(":")) == 0 {
		if !bytes.HasPrefix(tx, []byte("=")) && !bytes.HasSuffix(tx, []byte("=")) {
			return true
		}
	}
	return false
}

// PrepareProposal is called when the node is a proposer. Tendermint stages a set of transactions to the application. As the
// KVStore has two accepted formats, `:` and `=`, we modify all instances of `:` with `=` to make it consistent. Note: this is
// quite a trivial example of transaction modification.
// NOTE: we assume that Tendermint will never provide more transactions than can fit in a block.
func (app *Application) PrepareProposal(ctx context.Context, req *types.RequestPrepareProposal) (*types.ResponsePrepareProposal, error) {
	return &types.ResponsePrepareProposal{Txs: app.formatTxs(ctx, req.Txs)}, nil
}

// formatTxs validates and excludes invalid transactions
// also substitutes all the transactions with x:y to x=y
func (app *Application) formatTxs(ctx context.Context, blockData [][]byte) [][]byte {
	txs := make([][]byte, 0, len(blockData))
	for _, tx := range blockData {
		if resp, err := app.CheckTx(ctx, &types.RequestCheckTx{Tx: tx}); err == nil && resp.Code == CodeTypeOK {
			txs = append(txs, bytes.Replace(tx, []byte(":"), []byte("="), 1))
		}
	}
	return txs
}

// ProcessProposal is called whenever a node receives a complete proposal. It allows the application to validate the proposal.
// Only validators who can vote will have this method called. For the KVstore we reuse CheckTx.
func (app *Application) ProcessProposal(ctx context.Context, req *types.RequestProcessProposal) (*types.ResponseProcessProposal, error) {
	for _, tx := range req.Txs {
		// As CheckTx is a full validity check we can simply reuse this
		if resp, err := app.CheckTx(ctx, &types.RequestCheckTx{Tx: tx}); err != nil || resp.Code != CodeTypeOK {
			return &types.ResponseProcessProposal{Status: types.ResponseProcessProposal_REJECT}, nil
		}
	}
	return &types.ResponseProcessProposal{Status: types.ResponseProcessProposal_ACCEPT}, nil
}

// FinalizeBlock executes the block against the application state. It punishes validators who equivocated and
// updates validators according to transactions in a block. The rest of the transactions are regular key value
// updates and are cached in memory and will be persisted once Commit is called.
// ConsensusParams are never changed.
func (app *Application) FinalizeBlock(_ context.Context, req *types.RequestFinalizeBlock) (*types.ResponseFinalizeBlock, error) {
	// reset valset changes
	app.valUpdates = make([]types.ValidatorUpdate, 0)
	app.stagedTxs = make([][]byte, 0)

	// Punish validators who committed equivocation.
	for _, ev := range req.Misbehavior {
		if ev.Type == types.MisbehaviorType_DUPLICATE_VOTE {
			addr := string(ev.Validator.Address)
			if pubKey, ok := app.valAddrToPubKeyMap[addr]; ok {
				app.valUpdates = append(app.valUpdates, types.ValidatorUpdate{
					PubKey: pubKey,
					Power:  ev.Validator.Power - 1,
				})
				app.logger.Info("Decreased val power by 1 because of the equivocation",
					"val", addr)
			} else {
				panic(fmt.Errorf("wanted to punish val %q but can't find it", addr))
			}
		}
	}

	respTxs := make([]*types.ExecTxResult, len(req.Txs))
	for i, tx := range req.Txs {
		if isValidatorTx(tx) {
			pubKey, power, err := parseValidatorTx(tx)
			if err != nil {
				panic(err)
			}
			app.valUpdates = append(app.valUpdates, types.UpdateValidator(pubKey, power, ""))
		} else {
			app.stagedTxs = append(app.stagedTxs, tx)
		}

		var key, value string
		parts := bytes.Split(tx, []byte("="))
		if len(parts) == 2 {
			key, value = string(parts[0]), string(parts[1])
		} else {
			key, value = string(tx), string(tx)
		}
		respTxs[i] = &types.ExecTxResult{
			Code: CodeTypeOK,
			// With every transaction we can emit a series of events. To make it simple, we just emit the same events.
			Events: []types.Event{
				{
					Type: "app",
					Attributes: []types.EventAttribute{
						{Key: "creator", Value: "Cosmoshi Netowoko", Index: true},
						{Key: "key", Value: key, Index: true},
						{Key: "index_key", Value: "index is working", Index: true},
						{Key: "noindex_key", Value: "index is working", Index: false},
					},
				},
				{
					Type: "app",
					Attributes: []types.EventAttribute{
						{Key: "creator", Value: "Cosmoshi", Index: true},
						{Key: "key", Value: value, Index: true},
						{Key: "index_key", Value: "index is working", Index: true},
						{Key: "noindex_key", Value: "index is working", Index: false},
					},
				},
			},
		}
		app.state.Size++
	}

	app.state.Height = req.Height

	response := &types.ResponseFinalizeBlock{TxResults: respTxs, ValidatorUpdates: app.valUpdates, AppHash: app.state.Hash()}
	if !app.genBlockEvents {
		return response, nil
	}
	if app.state.Height%2 == 0 {
		response.Events = []types.Event{
			{
				Type: "begin_event",
				Attributes: []types.EventAttribute{
					{
						Key:   "foo",
						Value: "100",
						Index: true,
					},
					{
						Key:   "bar",
						Value: "200",
						Index: true,
					},
				},
			},
			{
				Type: "begin_event",
				Attributes: []types.EventAttribute{
					{
						Key:   "foo",
						Value: "200",
						Index: true,
					},
					{
						Key:   "bar",
						Value: "300",
						Index: true,
					},
				},
			},
		}
	} else {
		response.Events = []types.Event{
			{
				Type: "begin_event",
				Attributes: []types.EventAttribute{
					{
						Key:   "foo",
						Value: "400",
						Index: true,
					},
					{
						Key:   "bar",
						Value: "300",
						Index: true,
					},
				},
			},
		}
	}
	return response, nil
}

// Commit is called after FinalizeBlock and after Tendermint state which includes the updates to
// AppHash, ConsensusParams and ValidatorSet has occurred.
// The KVStore persists the validator updates and the new key values
func (app *Application) Commit(context.Context, *types.RequestCommit) (*types.ResponseCommit, error) {
	// apply the validator updates to state (note this is really the validator set at h + 2)
	for _, valUpdate := range app.valUpdates {
		app.updateValidator(valUpdate)
	}

	// persist all the staged txs in the kvstore
	for _, tx := range app.stagedTxs {
		parts := bytes.Split(tx, []byte("="))
		if len(parts) != 2 {
			panic(fmt.Sprintf("unexpected tx format. Expected 2 got %d: %s", len(parts), parts))
		}
		key, value := string(parts[0]), string(parts[1])
		err := app.state.db.Set(prefixKey([]byte(key)), []byte(value))
		if err != nil {
			panic(err)
		}
	}

	// persist the state (i.e. size and height)
	saveState(app.state)

	resp := &types.ResponseCommit{}
	if app.RetainBlocks > 0 && app.state.Height >= app.RetainBlocks {
		resp.RetainHeight = app.state.Height - app.RetainBlocks + 1
	}
	return resp, nil
}

// Returns an associated value or nil if missing.
func (app *Application) Query(_ context.Context, reqQuery *types.RequestQuery) (*types.ResponseQuery, error) {
	resQuery := &types.ResponseQuery{}

	if reqQuery.Path == "/val" {
		key := []byte(ValidatorPrefix + string(reqQuery.Data))
		value, err := app.state.db.Get(key)
		if err != nil {
			panic(err)
		}

		return &types.ResponseQuery{
			Key:   reqQuery.Data,
			Value: value,
		}, nil
	}

	if reqQuery.Prove {
		value, err := app.state.db.Get(prefixKey(reqQuery.Data))
		if err != nil {
			panic(err)
		}

		if value == nil {
			resQuery.Log = "does not exist"
		} else {
			resQuery.Log = "exists"
		}
		resQuery.Index = -1 // TODO make Proof return index
		resQuery.Key = reqQuery.Data
		resQuery.Value = value
		resQuery.Height = app.state.Height

		return resQuery, nil
	}

	resQuery.Key = reqQuery.Data
	value, err := app.state.db.Get(prefixKey(reqQuery.Data))
	if err != nil {
		panic(err)
	}
	if value == nil {
		resQuery.Log = "does not exist"
	} else {
		resQuery.Log = "exists"
	}
	resQuery.Value = value
	resQuery.Height = app.state.Height

	return resQuery, nil
}

func (app *Application) Close() error {
	return app.state.db.Close()
}

func isValidatorTx(tx []byte) bool {
	return strings.HasPrefix(string(tx), ValidatorPrefix)
}

func parseValidatorTx(tx []byte) ([]byte, int64, error) {
	tx = tx[len(ValidatorPrefix):]

	//  get the pubkey and power
	pubKeyAndPower := strings.Split(string(tx), "!")
	if len(pubKeyAndPower) != 2 {
		return nil, 0, fmt.Errorf("expected 'pubkey!power'. Got %v", pubKeyAndPower)
	}
	pubkeyS, powerS := pubKeyAndPower[0], pubKeyAndPower[1]

	// decode the pubkey
	pubkey, err := base64.StdEncoding.DecodeString(pubkeyS)
	if err != nil {
		return nil, 0, fmt.Errorf("pubkey (%s) is invalid base64", pubkeyS)
	}

	// decode the power
	power, err := strconv.ParseInt(powerS, 10, 64)
	if err != nil {
		return nil, 0, fmt.Errorf("power (%s) is not an int", powerS)
	}

	if power < 0 {
		return nil, 0, fmt.Errorf("power can not be less than 0, got %d", power)
	}

	return pubkey, power, nil
}

// add, update, or remove a validator
func (app *Application) updateValidator(v types.ValidatorUpdate) {
	pubkey, err := cryptoencoding.PubKeyFromProto(v.PubKey)
	if err != nil {
		panic(fmt.Errorf("can't decode public key: %w", err))
	}
	key := []byte(ValidatorPrefix + string(pubkey.Bytes()))

	if v.Power == 0 {
		// remove validator
		hasKey, err := app.state.db.Has(key)
		if err != nil {
			panic(err)
		}
		if !hasKey {
			pubStr := base64.StdEncoding.EncodeToString(pubkey.Bytes())
			app.logger.Info("tried to remove non existent validator. Skipping...", "pubKey", pubStr)
		}
		if err = app.state.db.Delete(key); err != nil {
			panic(err)
		}
		delete(app.valAddrToPubKeyMap, string(pubkey.Address()))
	} else {
		// add or update validator
		value := bytes.NewBuffer(make([]byte, 0))
		if err := types.WriteMessage(&v, value); err != nil {
			panic(err)
		}
		if err = app.state.db.Set(key, value.Bytes()); err != nil {
			panic(err)
		}
		app.valAddrToPubKeyMap[string(pubkey.Address())] = v.PubKey
	}
}

func (app *Application) getValidators() (validators []types.ValidatorUpdate) {
	itr, err := app.state.db.Iterator(nil, nil)
	if err != nil {
		panic(err)
	}
	for ; itr.Valid(); itr.Next() {
		if isValidatorTx(itr.Key()) {
			validator := new(types.ValidatorUpdate)
			err := types.ReadMessage(bytes.NewBuffer(itr.Value()), validator)
			if err != nil {
				panic(err)
			}
			validators = append(validators, *validator)
		}
	}
	if err = itr.Error(); err != nil {
		panic(err)
	}
	return
}

// -----------------------------

type State struct {
	db dbm.DB
	// Size is essentially the amount of transactions that have been processes.
	// This is used for the appHash
	Size   int64 `json:"size"`
	Height int64 `json:"height"`
}

func loadState(db dbm.DB) State {
	var state State
	state.db = db
	stateBytes, err := db.Get(stateKey)
	if err != nil {
		panic(err)
	}
	if len(stateBytes) == 0 {
		return state
	}
	err = json.Unmarshal(stateBytes, &state)
	if err != nil {
		panic(err)
	}
	return state
}

func saveState(state State) {
	stateBytes, err := json.Marshal(state)
	if err != nil {
		panic(err)
	}
	err = state.db.Set(stateKey, stateBytes)
	if err != nil {
		panic(err)
	}
}

// Hash returns the hash of the application state. This is computed
// as the size or number of transactions processed within the state. Note that this isn't
// a strong guarantee of state machine replication because states could
// have different kv values but still have the same size.
// This function is used as the "AppHash"
func (s State) Hash() []byte {
	appHash := make([]byte, 8)
	binary.PutVarint(appHash, s.Size)
	return appHash
}

func prefixKey(key []byte) []byte {
	return append(kvPairPrefixKey, key...)
}
