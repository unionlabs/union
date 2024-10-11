package abci

import (
	"encoding/binary"
	"encoding/json"
	"errors"

	"github.com/dgraph-io/badger/v4"

	"github.com/cometbft/cometbft/abci/tutorials/abci-v2-forum-app/model"
)

type AppState struct {
	DB     *model.DB `json:"db"`
	Size   int64     `json:"size"`
	Height int64     `json:"height"`
}

var stateKey = "appstate"

func (s AppState) Hash() []byte {
	appHash := make([]byte, 8)
	binary.PutVarint(appHash, s.Size)
	return appHash
}

func loadState(db *model.DB) (AppState, error) {
	var state AppState
	state.DB = db
	stateBytes, err := db.Get([]byte(stateKey))
	if err != nil && !errors.Is(err, badger.ErrKeyNotFound) {
		return state, nil
	}
	if len(stateBytes) == 0 {
		return state, nil
	}
	err = json.Unmarshal(stateBytes, &state)
	state.DB = db
	if err != nil {
		return state, err
	}
	return state, nil
}

func saveState(state *AppState) error {
	stateBytes, err := json.Marshal(state)
	if err != nil {
		return err
	}
	err = state.DB.Set([]byte(stateKey), stateBytes)
	if err != nil {
		return err
	}
	return nil
}
