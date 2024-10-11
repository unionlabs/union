package types

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmttypes "github.com/cometbft/cometbft/types"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
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
		ag.GenesisTime = time.Now().Round(0).UTC()
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
	var ag AppGenesis
	var err error
	// check if io.ReadSeeker is implemented
	if rs, ok := reader.(io.ReadSeeker); ok {
		err = json.NewDecoder(rs).Decode(&ag)
		if err == nil {
			return &ag, nil
		}

		err = fmt.Errorf("error unmarshalling AppGenesis: %w", err)
		if _, serr := rs.Seek(0, io.SeekStart); serr != nil {
			err = errors.Join(err, fmt.Errorf("error seeking back to the front: %w", serr))
			return nil, err
		}
	}

	// TODO: once cmtjson implements incremental parsing, we can avoid storing the entire file in memory
	jsonBlob, ioerr := io.ReadAll(reader)
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

	vals := []sdk.GenesisValidator{}
	for _, cmtVal := range ctmGenesis.Validators {
		pk, err := cryptocodec.FromCmtPubKeyInterface(cmtVal.PubKey)
		if err != nil {
			return nil, err
		}
		jsonPk, err := cryptocodec.PubKeyFromProto(pk)
		if err != nil {
			return nil, err
		}
		val := sdk.GenesisValidator{
			Address: cmtVal.Address.Bytes(),
			PubKey:  jsonPk,
			Power:   cmtVal.Power,
			Name:    cmtVal.Name,
		}

		vals = append(vals, val)
	}

	ag = AppGenesis{
		AppName: version.AppName,
		// AppVersion is not filled as we do not know it from a CometBFT genesis
		GenesisTime:   ctmGenesis.GenesisTime,
		ChainID:       ctmGenesis.ChainID,
		InitialHeight: ctmGenesis.InitialHeight,
		AppHash:       ctmGenesis.AppHash,
		AppState:      ctmGenesis.AppState,
		Consensus: &ConsensusGenesis{
			Validators: vals,
			Params:     ctmGenesis.ConsensusParams,
		},
	}
	return &ag, nil
}

// AppGenesisFromFile reads the AppGenesis from the provided file.
func AppGenesisFromFile(genFile string) (*AppGenesis, error) {
	file, err := os.Open(filepath.Clean(genFile))
	if err != nil {
		return nil, err
	}

	appGenesis, err := AppGenesisFromReader(file)
	ferr := file.Close()
	if err != nil {
		return nil, fmt.Errorf("failed to read genesis from file %s: %w", genFile, err)
	}

	if ferr != nil {
		return nil, ferr
	}

	return appGenesis, nil
}

// --------------------------
// CometBFT Genesis Handling
// --------------------------

// ToGenesisDoc converts the AppGenesis to a CometBFT GenesisDoc.
func (ag *AppGenesis) ToGenesisDoc() (*cmttypes.GenesisDoc, error) {
	cmtValidators := []cmttypes.GenesisValidator{}
	for _, val := range ag.Consensus.Validators {
		pk, err := cryptocodec.PubKeyToProto(val.PubKey)
		if err != nil {
			return nil, err
		}
		cmtPk, err := cryptocodec.ToCmtPubKeyInterface(pk)
		if err != nil {
			return nil, err
		}
		cmtVal := cmttypes.GenesisValidator{
			Address: val.Address.Bytes(),
			PubKey:  cmtPk,
			Power:   val.Power,
			Name:    val.Name,
		}

		cmtValidators = append(cmtValidators, cmtVal)
	}
	// assert nil value for empty validators set
	if len(cmtValidators) == 0 {
		cmtValidators = nil
	}
	return &cmttypes.GenesisDoc{
		GenesisTime:     ag.GenesisTime,
		ChainID:         ag.ChainID,
		InitialHeight:   ag.InitialHeight,
		AppHash:         ag.AppHash,
		AppState:        ag.AppState,
		Validators:      cmtValidators,
		ConsensusParams: ag.Consensus.Params,
	}, nil
}

// ConsensusGenesis defines the consensus layer's genesis.
// TODO(@julienrbrt) eventually abstract from CometBFT types
type ConsensusGenesis struct {
	Validators []sdk.GenesisValidator    `json:"validators,omitempty"`
	Params     *cmttypes.ConsensusParams `json:"params,omitempty"`
}

// NewConsensusGenesis returns a ConsensusGenesis with given values.
// It takes a proto consensus params so it can called from server export command.
func NewConsensusGenesis(params cmtproto.ConsensusParams, validators []sdk.GenesisValidator) *ConsensusGenesis {
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

	var result Alias
	if err := cmtjson.Unmarshal(b, &result); err != nil {
		return err
	}

	cs.Params = result.Params
	cs.Validators = result.Validators

	return nil
}

func (cs *ConsensusGenesis) ValidateAndComplete() error {
	if cs == nil {
		return errors.New("consensus genesis cannot be nil")
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
			cs.Validators[i].Address = v.PubKey.Address().Bytes()
		}
	}

	return nil
}
