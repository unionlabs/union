package version

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/spf13/cobra"
	"sigs.k8s.io/yaml"
)

const (
	flagLong   = "long"
	flagOutput = "output"
)

// NewVersionCommand returns a CLI command to interactively print the application binary version information.
// Note: When seeking to add the extra info to the context
// The below can be added to the initRootCmd to include the extraInfo field
//
// cmdContext := context.WithValue(context.Background(), version.ContextKey{}, extraInfo)
// rootCmd.SetContext(cmdContext)
func NewVersionCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "version",
		Short: "Print the application binary version information",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			verInfo := NewInfo()

			if long, _ := cmd.Flags().GetBool(flagLong); !long {
				fmt.Fprintln(cmd.OutOrStdout(), verInfo.Version)
				return nil
			}

			// Extract and set extra information from the context
			verInfo.ExtraInfo = extraInfoFromContext(cmd)

			var (
				bz  []byte
				err error
			)

			output, _ := cmd.Flags().GetString(flagOutput)
			switch strings.ToLower(output) {
			case "json":
				bz, err = json.Marshal(verInfo)

			default:
				bz, err = yaml.Marshal(&verInfo)
			}

			if err != nil {
				return err
			}

			fmt.Fprintln(cmd.OutOrStdout(), string(bz))
			return nil
		},
	}

	cmd.Flags().Bool(flagLong, false, "Print long version information")
	cmd.Flags().StringP(flagOutput, "o", "text", "Output format (text|json)")

	return cmd
}

func extraInfoFromContext(cmd *cobra.Command) ExtraInfo {
	ctx := cmd.Context()
	if ctx != nil {
		extraInfo, ok := ctx.Value(ContextKey{}).(ExtraInfo)
		if ok {
			return extraInfo
		}
	}
	return nil
}
