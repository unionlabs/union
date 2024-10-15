package cli

import (
	"encoding/json"
	"path/filepath"

	"github.com/spf13/cobra"

	"cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const flagGenTxDir = "gentx-dir"

// CollectGenTxsCmd - return the cobra command to collect genesis transactions
func CollectGenTxsCmd(validator types.MessageValidator) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "collect-gentxs",
		Short: "Collect genesis txs and output a genesis.json file",
		RunE: func(cmd *cobra.Command, _ []string) error {
			config := client.GetConfigFromCmd(cmd)

			clientCtx := client.GetClientContextFromCmd(cmd)
			cdc := clientCtx.Codec

			consensusKey, err := cmd.Flags().GetString(FlagConsensusKeyAlgo)
			if err != nil {
				return errors.Wrap(err, "Failed to get consensus key algo")
			}

			nodeID, valPubKey, err := genutil.InitializeNodeValidatorFiles(config, consensusKey)
			if err != nil {
				return errors.Wrap(err, "failed to initialize node validator files")
			}

			appGenesis, err := types.AppGenesisFromFile(config.GenesisFile())
			if err != nil {
				return errors.Wrap(err, "failed to read genesis doc from file")
			}

			genTxDir, _ := cmd.Flags().GetString(flagGenTxDir)
			genTxsDir := genTxDir
			if genTxsDir == "" {
				genTxsDir = filepath.Join(config.RootDir, "config", "gentx")
			}

			toPrint := newPrintInfo(config.Moniker, appGenesis.ChainID, nodeID, genTxsDir, json.RawMessage(""))
			initCfg := types.NewInitConfig(appGenesis.ChainID, genTxsDir, nodeID, valPubKey)

			appMessage, err := genutil.GenAppStateFromConfig(cdc, clientCtx.TxConfig, config, initCfg, appGenesis, validator, clientCtx.ValidatorAddressCodec, clientCtx.AddressCodec)
			if err != nil {
				return errors.Wrap(err, "failed to get genesis app state from config")
			}

			toPrint.AppMessage = appMessage

			return displayInfo(cmd.ErrOrStderr(), toPrint)
		},
	}
	cmd.Flags().String(FlagConsensusKeyAlgo, "ed25519", "algorithm to use for the consensus key")
	cmd.Flags().String(flagGenTxDir, "", "override default \"gentx\" directory from which collect and execute genesis transactions; default [--home]/config/gentx/")

	return cmd
}
