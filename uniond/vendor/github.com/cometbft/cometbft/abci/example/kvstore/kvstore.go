package kvstore

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"fmt"

	dbm "github.com/cometbft/cometbft-db"

	"github.com/cometbft/cometbft/abci/example/code"
	"github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/version"
)

var (
	stateKey        = []byte("stateKey")
	kvPairPrefixKey = []byte("kvPairKey:")

	ProtocolVersion uint64 = 0x1
)

type State struct {
	db      dbm.DB
	Size    int64  `json:"size"`
	Height  int64  `json:"height"`
	AppHash []byte `json:"app_hash"`
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

func prefixKey(key []byte) []byte {
	return append(kvPairPrefixKey, key...)
}

//---------------------------------------------------

var _ types.Application = (*Application)(nil)

type Application struct {
	types.BaseApplication

	state        State
	RetainBlocks int64 // blocks to retain after commit (via ResponseCommit.RetainHeight)
	txToRemove   map[string]struct{}
	// If true, the app will generate block events in BeginBlock. Used to test the event indexer
	// Should be false by default to avoid generating too much data.
	genBlockEvents bool
}

func NewApplication() *Application {
	state := loadState(dbm.NewMemDB())
	return &Application{state: state}
}

func (app *Application) SetGenBlockEvents() {
	app.genBlockEvents = true
}

func (app *Application) Info(req types.RequestInfo) (resInfo types.ResponseInfo) {
	return types.ResponseInfo{
		Data:             fmt.Sprintf("{\"size\":%v}", app.state.Size),
		Version:          version.ABCISemVer,
		AppVersion:       ProtocolVersion,
		LastBlockHeight:  app.state.Height,
		LastBlockAppHash: app.state.AppHash,
	}
}

// tx is either "key=value" or just arbitrary bytes
func (app *Application) DeliverTx(req types.RequestDeliverTx) types.ResponseDeliverTx {
	if isReplacedTx(req.Tx) {
		app.txToRemove[string(req.Tx)] = struct{}{}
	}
	var key, value string

	parts := bytes.Split(req.Tx, []byte("="))
	if len(parts) == 2 {
		key, value = string(parts[0]), string(parts[1])
	} else {
		key, value = string(req.Tx), string(req.Tx)
	}

	err := app.state.db.Set(prefixKey([]byte(key)), []byte(value))
	if err != nil {
		panic(err)
	}
	app.state.Size++

	events := []types.Event{
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
	}

	return types.ResponseDeliverTx{Code: code.CodeTypeOK, Events: events}
}

func (app *Application) CheckTx(req types.RequestCheckTx) types.ResponseCheckTx {
	if len(req.Tx) == 0 {
		return types.ResponseCheckTx{Code: code.CodeTypeRejected}
	}

	if req.Type == types.CheckTxType_Recheck {
		if _, ok := app.txToRemove[string(req.Tx)]; ok {
			return types.ResponseCheckTx{Code: code.CodeTypeExecuted, GasWanted: 1}
		}
	}
	return types.ResponseCheckTx{Code: code.CodeTypeOK, GasWanted: 1}
}

func (app *Application) Commit() types.ResponseCommit {
	// Using a memdb - just return the big endian size of the db
	appHash := make([]byte, 8)
	binary.PutVarint(appHash, app.state.Size)
	app.state.AppHash = appHash
	app.state.Height++

	// empty out the set of transactions to remove via rechecktx
	saveState(app.state)

	resp := types.ResponseCommit{Data: appHash}
	if app.RetainBlocks > 0 && app.state.Height >= app.RetainBlocks {
		resp.RetainHeight = app.state.Height - app.RetainBlocks + 1
	}
	return resp
}

// Returns an associated value or nil if missing.
func (app *Application) Query(reqQuery types.RequestQuery) (resQuery types.ResponseQuery) {
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

		return
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

	return resQuery
}

func (app *Application) BeginBlock(req types.RequestBeginBlock) types.ResponseBeginBlock {
	app.txToRemove = map[string]struct{}{}
	response := types.ResponseBeginBlock{}

	if !app.genBlockEvents {
		return response
	}

	if app.state.Height%2 == 0 {
		response = types.ResponseBeginBlock{
			Events: []types.Event{
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
			},
		}
	} else {
		response = types.ResponseBeginBlock{
			Events: []types.Event{
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
			},
		}
	}

	return response
}

func (app *Application) ProcessProposal(
	req types.RequestProcessProposal,
) types.ResponseProcessProposal {
	for _, tx := range req.Txs {
		if len(tx) == 0 {
			return types.ResponseProcessProposal{Status: types.ResponseProcessProposal_REJECT}
		}
	}
	return types.ResponseProcessProposal{Status: types.ResponseProcessProposal_ACCEPT}
}
