package cli

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"

	cfg "github.com/cometbft/cometbft/config"
	cmttypes "github.com/cometbft/cometbft/types"
	"github.com/cosmos/go-bip39"
	"github.com/spf13/cobra"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/math/unsafe"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/input"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/version"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const (
	// FlagOverwrite defines a flag to overwrite an existing genesis JSON file.
	FlagOverwrite = "overwrite"

	// FlagSeed defines a flag to initialize the private validator key from a specific seed.
	FlagRecover = "recover"

	// FlagDefaultBondDenom defines the default denom to use in the genesis file.
	FlagDefaultBondDenom = "default-denom"

	// FlagConsensusKeyAlgo defines the algorithm to use for the consensus signing key.
	FlagConsensusKeyAlgo = "consensus-key-algo"
)

type printInfo struct {
	Moniker    string          `json:"moniker" yaml:"moniker"`
	ChainID    string          `json:"chain_id" yaml:"chain_id"`
	NodeID     string          `json:"node_id" yaml:"node_id"`
	GenTxsDir  string          `json:"gentxs_dir" yaml:"gentxs_dir"`
	AppMessage json.RawMessage `json:"app_message" yaml:"app_message"`
}

func newPrintInfo(moniker, chainID, nodeID, genTxsDir string, appMessage json.RawMessage) printInfo {
	return printInfo{
		Moniker:    moniker,
		ChainID:    chainID,
		NodeID:     nodeID,
		GenTxsDir:  genTxsDir,
		AppMessage: appMessage,
	}
}

func displayInfo(dst io.Writer, info printInfo) error {
	out, err := json.MarshalIndent(info, "", " ")
	if err != nil {
		return err
	}

	_, err = fmt.Fprintf(dst, "%s\n", out)

	return err
}

// InitCmd returns a command that initializes all files needed for Tendermint
// and the respective application.
func InitCmd(mm genesisMM) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "init <moniker>",
		Short: "Initialize private validator, p2p, genesis, and application configuration files",
		Long:  `Initialize validators's and node's configuration files.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			clientCtx := client.GetClientContextFromCmd(cmd)

			config := client.GetConfigFromCmd(cmd)
			chainID, _ := cmd.Flags().GetString(flags.FlagChainID)
			switch {
			case chainID != "":
			case clientCtx.ChainID != "":
				chainID = clientCtx.ChainID
			default:
				chainID = fmt.Sprintf("test-chain-%v", unsafe.Str(6))
			}
			if config.RootDir == "" {
				config.RootDir = clientCtx.HomeDir
			}

			// Get bip39 mnemonic
			var mnemonic string
			recover, _ := cmd.Flags().GetBool(FlagRecover)
			if recover {
				inBuf := bufio.NewReader(cmd.InOrStdin())
				value, err := input.GetString("Enter your bip39 mnemonic", inBuf)
				if err != nil {
					return err
				}

				mnemonic = value
				if !bip39.IsMnemonicValid(mnemonic) {
					return errors.New("invalid mnemonic")
				}
			}

			// Get initial height
			initHeight, _ := cmd.Flags().GetInt64(flags.FlagInitHeight)
			if initHeight < 1 {
				initHeight = 1
			}

			consensusKey, err := cmd.Flags().GetString(FlagConsensusKeyAlgo)
			print(consensusKey)
			if err != nil {
				return errorsmod.Wrap(err, "Failed to get consensus key algo")
			}

			nodeID, _, err := genutil.InitializeNodeValidatorFilesFromMnemonic(config, mnemonic, consensusKey)
			if err != nil {
				return err
			}

			config.Moniker = args[0]

			genFile := config.GenesisFile()
			overwrite, _ := cmd.Flags().GetBool(FlagOverwrite)
			defaultDenom, _ := cmd.Flags().GetString(FlagDefaultBondDenom)

			// use os.Stat to check if the file exists
			_, err = os.Stat(genFile)
			if !overwrite && !os.IsNotExist(err) {
				return fmt.Errorf("genesis.json file already exists: %v", genFile)
			}

			// Overwrites the SDK default denom for side-effects
			if defaultDenom != "" {
				sdk.DefaultBondDenom = defaultDenom
			}
			appGenState := mm.DefaultGenesis()

			appState, err := json.MarshalIndent(appGenState, "", " ")
			if err != nil {
				return errorsmod.Wrap(err, "Failed to marshal default genesis state")
			}

			appGenesis := &types.AppGenesis{}
			if _, err := os.Stat(genFile); err != nil {
				if !os.IsNotExist(err) {
					return err
				}
			} else {
				appGenesis, err = types.AppGenesisFromFile(genFile)
				if err != nil {
					return errorsmod.Wrap(err, "Failed to read genesis doc from file")
				}
			}

			appGenesis.AppName = version.AppName
			appGenesis.AppVersion = version.Version
			appGenesis.ChainID = chainID
			appGenesis.AppState = appState
			appGenesis.InitialHeight = initHeight
			appGenesis.Consensus = &types.ConsensusGenesis{
				Validators: nil,
				Params:     cmttypes.DefaultConsensusParams(),
			}

			appGenesis.Consensus.Params.Validator.PubKeyTypes = []string{consensusKey}

			if err = genutil.ExportGenesisFile(appGenesis, genFile); err != nil {
				return errorsmod.Wrap(err, "Failed to export genesis file")
			}

			toPrint := newPrintInfo(config.Moniker, chainID, nodeID, "", appState)

			cfg.WriteConfigFile(filepath.Join(config.RootDir, "config", "config.toml"), config)
			return displayInfo(cmd.ErrOrStderr(), toPrint)
		},
	}

	cmd.Flags().BoolP(FlagOverwrite, "o", false, "overwrite the genesis.json file")
	cmd.Flags().Bool(FlagRecover, false, "provide seed phrase to recover existing key instead of creating")
	cmd.Flags().String(flags.FlagChainID, "", "genesis file chain-id, if left blank will be randomly created")
	cmd.Flags().String(FlagDefaultBondDenom, "", "genesis file default denomination, if left blank default value is 'stake'")
	cmd.Flags().Int64(flags.FlagInitHeight, 1, "specify the initial block height at genesis")
	cmd.Flags().String(FlagConsensusKeyAlgo, "ed25519", "algorithm to use for the consensus key")

	return cmd
}
