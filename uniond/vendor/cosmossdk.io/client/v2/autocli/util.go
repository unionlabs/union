package autocli

import (
	"context"
	"strings"

	"github.com/spf13/cobra"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/client/v2/internal/strcase"
)

// findSubCommand finds a sub-command of the provided command whose Use
// string is or begins with the provided subCmdName.
// It verifies the command's aliases as well.
func findSubCommand(cmd *cobra.Command, subCmdName string) *cobra.Command {
	for _, subCmd := range cmd.Commands() {
		use := subCmd.Use
		if use == subCmdName || strings.HasPrefix(use, subCmdName+" ") {
			return subCmd
		}

		for _, alias := range subCmd.Aliases {
			if alias == subCmdName || strings.HasPrefix(alias, subCmdName+" ") {
				return subCmd
			}
		}
	}
	return nil
}

// topLevelCmd creates a new top-level command with the provided name and
// description. The command will have DisableFlagParsing set to false and
// SuggestionsMinimumDistance set to 2.
func topLevelCmd(ctx context.Context, use, short string) *cobra.Command {
	cmd := &cobra.Command{
		Use:                        use,
		Short:                      short,
		DisableFlagParsing:         false,
		SuggestionsMinimumDistance: 2,
		RunE:                       validateCmd,
	}
	cmd.SetContext(ctx)

	return cmd
}

func protoNameToCliName(name protoreflect.Name) string {
	return strcase.ToKebab(string(name))
}
