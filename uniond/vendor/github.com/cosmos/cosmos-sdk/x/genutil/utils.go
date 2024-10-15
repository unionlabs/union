package genutil

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"time"

	cfg "github.com/cometbft/cometbft/config"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	cmtbls12381 "github.com/cometbft/cometbft/crypto/bls12381"
	cmtbn254 "github.com/cometbft/cometbft/crypto/bn254"
	tmed25519 "github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	"github.com/cosmos/go-bip39"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// ExportGenesisFile creates and writes the genesis configuration to disk. An
// error is returned if building or writing the configuration to file fails.
func ExportGenesisFile(genesis *types.AppGenesis, genFile string) error {
	if err := genesis.ValidateAndComplete(); err != nil {
		return err
	}

	return genesis.SaveAs(genFile)
}

// ExportGenesisFileWithTime creates and writes the genesis configuration to disk.
// An error is returned if building or writing the configuration to file fails.
func ExportGenesisFileWithTime(
	genFile, chainID string, validators []sdk.GenesisValidator, appState json.RawMessage, genTime time.Time,
) error {
	appGenesis := types.NewAppGenesisWithVersion(chainID, appState)
	appGenesis.GenesisTime = genTime
	appGenesis.Consensus.Validators = validators

	if err := appGenesis.ValidateAndComplete(); err != nil {
		return err
	}

	return appGenesis.SaveAs(genFile)
}

// InitializeNodeValidatorFiles creates private validator and p2p configuration files.
func InitializeNodeValidatorFiles(config *cfg.Config, keyType string) (
	nodeID string, valPubKey cryptotypes.PubKey, err error,
) {
	return InitializeNodeValidatorFilesFromMnemonic(config, "", keyType)
}

// InitializeNodeValidatorFilesFromMnemonic creates private validator and p2p configuration files using the given mnemonic.
// If no valid mnemonic is given, a random one will be used instead.
func InitializeNodeValidatorFilesFromMnemonic(config *cfg.Config, mnemonic, keyType string) (
	nodeID string, valPubKey cryptotypes.PubKey, err error,
) {
	if len(mnemonic) > 0 && !bip39.IsMnemonicValid(mnemonic) {
		return "", nil, errors.New("invalid mnemonic")
	}
	nodeKey, err := p2p.LoadOrGenNodeKey(config.NodeKeyFile())
	if err != nil {
		return "", nil, err
	}

	nodeID = string(nodeKey.ID())

	pvKeyFile := config.PrivValidatorKeyFile()
	if err := os.MkdirAll(filepath.Dir(pvKeyFile), 0o777); err != nil {
		return "", nil, fmt.Errorf("could not create directory %q: %w", filepath.Dir(pvKeyFile), err)
	}

	pvStateFile := config.PrivValidatorStateFile()
	if err := os.MkdirAll(filepath.Dir(pvStateFile), 0o777); err != nil {
		return "", nil, fmt.Errorf("could not create directory %q: %w", filepath.Dir(pvStateFile), err)
	}

	var (
		filePV  *privval.FilePV
		privKey cmtcrypto.PrivKey
	)

	if len(mnemonic) == 0 {
		switch keyType {
		case "ed25519":
			filePV = loadOrGenFilePV(tmed25519.GenPrivKey(), pvKeyFile, pvStateFile)
		case "bls12_381":
			privKey, err = cmtbls12381.GenPrivKey()
			if err != nil {
				return "", nil, err
			}
			filePV = loadOrGenFilePV(privKey, pvKeyFile, pvStateFile)
		case "bn254":
			privKey = cmtbn254.GenPrivKey()
			filePV = loadOrGenFilePV(privKey, pvKeyFile, pvStateFile)
		default:
			privKey = cmtbn254.GenPrivKey()
			filePV = loadOrGenFilePV(privKey, pvKeyFile, pvStateFile)
		}
	} else {
		switch keyType {
		case "ed25519":
			privKey = tmed25519.GenPrivKeyFromSecret([]byte(mnemonic))
		case "bls12_381":
			// TODO: need to add support for getting from mnemonic in Comet.
			return "", nil, errors.New("BLS key type does not support mnemonic")
		case "bn254":
			privKey = cmtbn254.GenPrivKeyFromSeed([]byte(mnemonic))
		default:
			privKey = cmtbn254.GenPrivKeyFromSeed([]byte(mnemonic))
		}
		filePV = privval.NewFilePV(privKey, pvKeyFile, pvStateFile)
		filePV.Save()
	}

	tmValPubKey, err := filePV.GetPubKey()
	if err != nil {
		return "", nil, err
	}

	valPubKey, err = cryptocodec.FromCmtPubKeyInterface(tmValPubKey)
	if err != nil {
		return "", nil, err
	}

	return nodeID, valPubKey, nil
}

// loadOrGenFilePV loads a FilePV from the given filePaths
// or else generates a new one and saves it to the filePaths.
func loadOrGenFilePV(privKey cmtcrypto.PrivKey, keyFilePath, stateFilePath string) *privval.FilePV {
	_, err := os.Stat(keyFilePath)
	exists := !os.IsNotExist(err)

	var pv *privval.FilePV
	if exists {
		pv = privval.LoadFilePV(keyFilePath, stateFilePath)
	} else {
		pv = privval.NewFilePV(privKey, keyFilePath, stateFilePath)
		pv.Save()
	}
	return pv
}
