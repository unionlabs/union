package cli

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"os"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/version"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const (
	flagTraceStore       = "trace-store"
	flagHeight           = "height"
	flagForZeroHeight    = "for-zero-height"
	flagJailAllowedAddrs = "jail-allowed-addrs"
	flagModulesToExport  = "modules-to-export"
)

// ExportCmd dumps app state to JSON.
func ExportCmd(appExporter servertypes.AppExporter) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "export",
		Short: "Export state to JSON",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			config := client.GetConfigFromCmd(cmd)
			viper := client.GetViperFromCmd(cmd)
			logger := client.GetLoggerFromCmd(cmd)

			if _, err := os.Stat(config.GenesisFile()); os.IsNotExist(err) {
				return err
			}

			db, err := server.OpenDB(config.RootDir, server.GetAppDBBackend(viper))
			if err != nil {
				return err
			}

			if appExporter == nil {
				if _, err := fmt.Fprintln(cmd.ErrOrStderr(), "WARNING: App exporter not defined. Returning genesis file."); err != nil {
					return err
				}

				// Open file in read-only mode so we can copy it to stdout.
				// It is possible that the genesis file is large,
				// so we don't need to read it all into memory
				// before we stream it out.
				f, err := os.OpenFile(config.GenesisFile(), os.O_RDONLY, 0)
				if err != nil {
					return err
				}
				defer f.Close()

				if _, err := io.Copy(cmd.OutOrStdout(), f); err != nil {
					return err
				}

				return nil
			}

			traceWriterFile, _ := cmd.Flags().GetString(flagTraceStore)
			traceWriter, cleanup, err := server.SetupTraceWriter(logger, traceWriterFile) //resleak:notresource
			if err != nil {
				return err
			}
			defer cleanup()

			height, _ := cmd.Flags().GetInt64(flagHeight)
			forZeroHeight, _ := cmd.Flags().GetBool(flagForZeroHeight)
			jailAllowedAddrs, _ := cmd.Flags().GetStringSlice(flagJailAllowedAddrs)
			modulesToExport, _ := cmd.Flags().GetStringSlice(flagModulesToExport)
			outputDocument, _ := cmd.Flags().GetString(flags.FlagOutputDocument)

			exported, err := appExporter(logger, db, traceWriter, height, forZeroHeight, jailAllowedAddrs, viper, modulesToExport)
			if err != nil {
				return fmt.Errorf("error exporting state: %w", err)
			}

			appGenesis, err := genutiltypes.AppGenesisFromFile(config.GenesisFile())
			if err != nil {
				return err
			}

			// set current binary version
			appGenesis.AppName = version.AppName
			appGenesis.AppVersion = version.Version

			appGenesis.AppState = exported.AppState
			appGenesis.InitialHeight = exported.Height
			appGenesis.Consensus = genutiltypes.NewConsensusGenesis(exported.ConsensusParams, exported.Validators)

			out, err := json.Marshal(appGenesis)
			if err != nil {
				return err
			}

			if outputDocument == "" {
				// Copy the entire genesis file to stdout.
				_, err := io.Copy(cmd.OutOrStdout(), bytes.NewReader(out))
				return err
			}

			if err = appGenesis.SaveAs(outputDocument); err != nil {
				return err
			}

			return nil
		},
	}

	cmd.Flags().Int64(flagHeight, -1, "Export state from a particular height (-1 means latest height)")
	cmd.Flags().Bool(flagForZeroHeight, false, "Export state to start at height zero (perform preproccessing)")
	cmd.Flags().StringSlice(flagJailAllowedAddrs, []string{}, "Comma-separated list of operator addresses of jailed validators to unjail")
	cmd.Flags().StringSlice(flagModulesToExport, []string{}, "Comma-separated list of modules to export. If empty, will export all modules")
	cmd.Flags().String(flags.FlagOutputDocument, "", "Exported state is written to the given file instead of STDOUT")

	return cmd
}
