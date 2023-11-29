package types

import (
	"bufio"
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	cmttypes "github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"

	"github.com/cosmos/cosmos-sdk/version"
)

const (
	// MaxChainIDLen is the maximum length of a chain ID.
	MaxChainIDLen = cmttypes.MaxChainIDLen
)

// AppGenesis defines the app's genesis.
type AppGenesis struct {
	AppName       string            `json:"app_name"`
	AppVersion    string            `json:"app_version"`
	GenesisTime   time.Time         `json:"genesis_time"`
	ChainID       string            `json:"chain_id"`
	InitialHeight int64             `json:"initial_height"`
	AppHash       []byte            `json:"app_hash"`
	AppState      json.RawMessage   `json:"app_state,omitempty"`
	Consensus     *ConsensusGenesis `json:"consensus,omitempty"`
}

// NewAppGenesisWithVersion returns a new AppGenesis with the app name and app version already.
func NewAppGenesisWithVersion(chainID string, appState json.RawMessage) *AppGenesis {
	return &AppGenesis{
		AppName:    version.AppName,
		AppVersion: version.Version,
		ChainID:    chainID,
		AppState:   appState,
		Consensus: &ConsensusGenesis{
			Validators: nil,
		},
	}
}

// ValidateAndComplete performs validation and completes the AppGenesis.
func (ag *AppGenesis) ValidateAndComplete() error {
	if ag.ChainID == "" {
		return errors.New("genesis doc must include non-empty chain_id")
	}

	if len(ag.ChainID) > MaxChainIDLen {
		return fmt.Errorf("chain_id in genesis doc is too long (max: %d)", MaxChainIDLen)
	}

	if ag.InitialHeight < 0 {
		return fmt.Errorf("initial_height cannot be negative (got %v)", ag.InitialHeight)
	}

	if ag.InitialHeight == 0 {
		ag.InitialHeight = 1
	}

	if ag.GenesisTime.IsZero() {
		ag.GenesisTime = cmttime.Now()
	}

	if err := ag.Consensus.ValidateAndComplete(); err != nil {
		return err
	}

	return nil
}

// SaveAs is a utility method for saving AppGenesis as a JSON file.
func (ag *AppGenesis) SaveAs(file string) error {
	appGenesisBytes, err := json.MarshalIndent(ag, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(file, appGenesisBytes, 0o600)
}

// AppGenesisFromReader reads the AppGenesis from the reader.
func AppGenesisFromReader(reader io.Reader) (*AppGenesis, error) {
	jsonBlob, err := io.ReadAll(reader)
	if err != nil {
		return nil, err
	}

	var appGenesis AppGenesis
	if err := json.Unmarshal(jsonBlob, &appGenesis); err != nil {
		// fallback to CometBFT genesis
		var ctmGenesis cmttypes.GenesisDoc
		if err2 := cmtjson.Unmarshal(jsonBlob, &ctmGenesis); err2 != nil {
			return nil, fmt.Errorf("error unmarshalling AppGenesis: %w\n failed fallback to CometBFT GenDoc: %w", err, err2)
		}

		appGenesis = AppGenesis{
			AppName: version.AppName,
			// AppVersion is not filled as we do not know it from a CometBFT genesis
			GenesisTime:   ctmGenesis.GenesisTime,
			ChainID:       ctmGenesis.ChainID,
			InitialHeight: ctmGenesis.InitialHeight,
			AppHash:       ctmGenesis.AppHash,
			AppState:      ctmGenesis.AppState,
			Consensus: &ConsensusGenesis{
				Validators: ctmGenesis.Validators,
				Params:     ctmGenesis.ConsensusParams,
			},
		}
	}

	return &appGenesis, nil
}

// AppGenesisFromFile reads the AppGenesis from the provided file.
func AppGenesisFromFile(genFile string) (*AppGenesis, error) {
	file, err := os.Open(filepath.Clean(genFile))
	if err != nil {
		return nil, err
	}

	appGenesis, err := AppGenesisFromReader(bufio.NewReader(file))
	if err != nil {
		return nil, fmt.Errorf("failed to read genesis from file %s: %w", genFile, err)
	}

	if err := file.Close(); err != nil {
		return nil, err
	}

	return appGenesis, nil
}

// --------------------------
// CometBFT Genesis Handling
// --------------------------

// ToGenesisDoc converts the AppGenesis to a CometBFT GenesisDoc.
func (ag *AppGenesis) ToGenesisDoc() (*cmttypes.GenesisDoc, error) {
	return &cmttypes.GenesisDoc{
		GenesisTime:     ag.GenesisTime,
		ChainID:         ag.ChainID,
		InitialHeight:   ag.InitialHeight,
		AppHash:         ag.AppHash,
		AppState:        ag.AppState,
		Validators:      ag.Consensus.Validators,
		ConsensusParams: ag.Consensus.Params,
	}, nil
}

// ConsensusGenesis defines the consensus layer's genesis.
// TODO(@julienrbrt) eventually abstract from CometBFT types
type ConsensusGenesis struct {
	Validators []cmttypes.GenesisValidator `json:"validators,omitempty"`
	Params     *cmttypes.ConsensusParams   `json:"params,omitempty"`
}

// NewConsensusGenesis returns a ConsensusGenesis with given values.
// It takes a proto consensus params so it can called from server export command.
func NewConsensusGenesis(params cmtproto.ConsensusParams, validators []cmttypes.GenesisValidator) *ConsensusGenesis {
	return &ConsensusGenesis{
		Params: &cmttypes.ConsensusParams{
			Block: cmttypes.BlockParams{
				MaxBytes: params.Block.MaxBytes,
				MaxGas:   params.Block.MaxGas,
			},
			Evidence: cmttypes.EvidenceParams{
				MaxAgeNumBlocks: params.Evidence.MaxAgeNumBlocks,
				MaxAgeDuration:  params.Evidence.MaxAgeDuration,
				MaxBytes:        params.Evidence.MaxBytes,
			},
			Validator: cmttypes.ValidatorParams{
				PubKeyTypes: params.Validator.PubKeyTypes,
			},
		},
		Validators: validators,
	}
}

func (cs *ConsensusGenesis) MarshalJSON() ([]byte, error) {
	type Alias ConsensusGenesis
	return cmtjson.Marshal(&Alias{
		Validators: cs.Validators,
		Params:     cs.Params,
	})
}

func (cs *ConsensusGenesis) UnmarshalJSON(b []byte) error {
	type Alias ConsensusGenesis

	result := Alias{}
	if err := cmtjson.Unmarshal(b, &result); err != nil {
		return err
	}

	cs.Params = result.Params
	cs.Validators = result.Validators

	return nil
}

func (cs *ConsensusGenesis) ValidateAndComplete() error {
	if cs == nil {
		return fmt.Errorf("consensus genesis cannot be nil")
	}

	if cs.Params == nil {
		cs.Params = cmttypes.DefaultConsensusParams()
	} else if err := cs.Params.ValidateBasic(); err != nil {
		return err
	}

	for i, v := range cs.Validators {
		if v.Power == 0 {
			return fmt.Errorf("the genesis file cannot contain validators with no voting power: %v", v)
		}
		if len(v.Address) > 0 && !bytes.Equal(v.PubKey.Address(), v.Address) {
			return fmt.Errorf("incorrect address for validator %v in the genesis file, should be %v", v, v.PubKey.Address())
		}
		if len(v.Address) == 0 {
			cs.Validators[i].Address = v.PubKey.Address()
		}
	}

	return nil
}
