package cli

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"

	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

const chainUpgradeGuide = "https://github.com/cosmos/cosmos-sdk/blob/main/UPGRADING.md"

// ValidateGenesisCmd takes a genesis file, and makes sure that it is valid.
func ValidateGenesisCmd(genMM genesisMM) *cobra.Command {
	return &cobra.Command{
		Use:     "validate [file]",
		Aliases: []string{"validate-genesis"},
		Args:    cobra.RangeArgs(0, 1),
		Short:   "Validates the genesis file at the default location or at the location passed as an arg",
		RunE: func(cmd *cobra.Command, args []string) (err error) {
			cfg := client.GetConfigFromCmd(cmd)

			// Load default if passed no args, otherwise load passed file
			var genesis string
			if len(args) == 0 {
				genesis = cfg.GenesisFile()
			} else {
				genesis = args[0]
			}

			appGenesis, err := types.AppGenesisFromFile(genesis)
			if err != nil {
				return enrichUnmarshalError(err)
			}

			if err := appGenesis.ValidateAndComplete(); err != nil {
				return fmt.Errorf("make sure that you have correctly migrated all CometBFT consensus params. Refer the UPGRADING.md (%s): %w", chainUpgradeGuide, err)
			}

			var genState map[string]json.RawMessage
			if err = json.Unmarshal(appGenesis.AppState, &genState); err != nil {
				if strings.Contains(err.Error(), "unexpected end of JSON input") {
					return fmt.Errorf("app_state is missing in the genesis file: %s", err.Error())
				}
				return fmt.Errorf("error unmarshalling genesis doc %s: %w", genesis, err)
			}

			if genMM != nil {
				if err = genMM.ValidateGenesis(genState); err != nil {
					errStr := fmt.Sprintf("error validating genesis file %s: %s", genesis, err.Error())
					if errors.Is(err, io.EOF) {
						errStr = fmt.Sprintf("%s: section is missing in the app_state", errStr)
					}
					return fmt.Errorf("%s", errStr)
				}
			}

			fmt.Fprintf(cmd.OutOrStdout(), "File at %s is a valid genesis file\n", genesis)
			return nil
		},
	}
}

func enrichUnmarshalError(err error) error {
	var syntaxErr *json.SyntaxError
	if errors.As(err, &syntaxErr) {
		return fmt.Errorf("error at offset %d: %s", syntaxErr.Offset, syntaxErr.Error())
	}
	return err
}
