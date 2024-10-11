package migrate

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmttypes "github.com/cometbft/cometbft/types"

	"github.com/cosmos/cosmos-sdk/client"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

type legacyAppGenesis struct {
	AppName       string                  `json:"app_name"`
	AppVersion    string                  `json:"app_version"`
	GenesisTime   time.Time               `json:"genesis_time"`
	ChainID       string                  `json:"chain_id"`
	InitialHeight int64                   `json:"initial_height"`
	AppHash       []byte                  `json:"app_hash"`
	AppState      json.RawMessage         `json:"app_state,omitempty"`
	Consensus     *legacyConsensusGenesis `json:"consensus,omitempty"`
}

type legacyConsensusGenesis struct {
	Validators []cmttypes.GenesisValidator `json:"validators,omitempty"`
	Params     *cmttypes.ConsensusParams   `json:"params,omitempty"`
}

func MigrateGenesisFile(oldGenFile string) (*types.AppGenesis, error) {
	file, err := os.Open(filepath.Clean(oldGenFile))
	if err != nil {
		return nil, err
	}
	defer file.Close()

	appGenesis, err := migrateGenesisValidator(file)
	if err != nil {
		return nil, fmt.Errorf("failed to read genesis from file %s: %w", oldGenFile, err)
	}

	return appGenesis, nil
}

// migrateGenesisValidator migrate current genesis file genesis validator to match of the
// new genesis validator type.
func migrateGenesisValidator(r io.Reader) (*types.AppGenesis, error) {
	var newAg types.AppGenesis
	var ag legacyAppGenesis
	var err error

	if rs, ok := r.(io.ReadSeeker); ok {
		err = json.NewDecoder(rs).Decode(&ag)
		if err == nil {
			vals, err := convertValidators(ag.Consensus.Validators)
			if err != nil {
				return nil, err
			}
			newAg = types.AppGenesis{
				AppName:       ag.AppName,
				AppVersion:    ag.AppVersion,
				GenesisTime:   ag.GenesisTime,
				ChainID:       ag.ChainID,
				InitialHeight: ag.InitialHeight,
				AppHash:       ag.AppHash,
				AppState:      ag.AppState,
				Consensus: &types.ConsensusGenesis{
					Validators: vals,
					Params:     ag.Consensus.Params,
				},
			}

			return &newAg, nil
		}

		err = fmt.Errorf("error unmarshalling legacy AppGenesis: %w", err)
		if _, serr := rs.Seek(0, io.SeekStart); serr != nil {
			err = errors.Join(err, fmt.Errorf("error seeking back to the front: %w", serr))
			return nil, err
		}
	}

	jsonBlob, ioerr := io.ReadAll(r)
	if ioerr != nil {
		err = errors.Join(err, fmt.Errorf("failed to read file completely: %w", ioerr))
		return nil, err
	}

	// fallback to comet genesis parsing
	var ctmGenesis cmttypes.GenesisDoc
	if uerr := cmtjson.Unmarshal(jsonBlob, &ctmGenesis); uerr != nil {
		err = errors.Join(err, fmt.Errorf("failed fallback to CometBFT GenDoc: %w", uerr))
		return nil, err
	}

	vals, err := convertValidators(ctmGenesis.Validators)
	if err != nil {
		return nil, err
	}
	newAg = types.AppGenesis{
		AppName:       version.AppName,
		GenesisTime:   ctmGenesis.GenesisTime,
		ChainID:       ctmGenesis.ChainID,
		InitialHeight: ctmGenesis.InitialHeight,
		AppHash:       ctmGenesis.AppHash,
		AppState:      ctmGenesis.AppState,
		Consensus: &types.ConsensusGenesis{
			Validators: vals,
			Params:     ctmGenesis.ConsensusParams,
		},
	}

	return &newAg, nil
}

func convertValidators(cmtVals []cmttypes.GenesisValidator) ([]sdk.GenesisValidator, error) {
	vals := make([]sdk.GenesisValidator, len(cmtVals))
	for i, cmtVal := range cmtVals {
		pk, err := cryptocodec.FromCmtPubKeyInterface(cmtVal.PubKey)
		if err != nil {
			return nil, err
		}
		jsonPk, err := cryptocodec.PubKeyFromProto(pk)
		if err != nil {
			return nil, err
		}
		vals[i] = sdk.GenesisValidator{
			Address: cmtVal.Address.Bytes(),
			PubKey:  jsonPk,
			Power:   cmtVal.Power,
			Name:    cmtVal.Name,
		}
	}
	return vals, nil
}

// CometBFT Genesis Handling for JSON,
// this is necessary for json unmarshaling of legacyConsensusGenesis
func (cs *legacyConsensusGenesis) MarshalJSON() ([]byte, error) {
	type Alias legacyConsensusGenesis
	return cmtjson.Marshal(&Alias{
		Validators: cs.Validators,
		Params:     cs.Params,
	})
}

func (cs *legacyConsensusGenesis) UnmarshalJSON(b []byte) error {
	type Alias legacyConsensusGenesis

	result := Alias{}
	if err := cmtjson.Unmarshal(b, &result); err != nil {
		return err
	}

	cs.Params = result.Params
	cs.Validators = result.Validators

	return nil
}

// since we only need migrate the consensus validators content so there is no
// exported state migration.
func Migrate(appState types.AppMap, _ client.Context) (types.AppMap, error) {
	return appState, nil
}
