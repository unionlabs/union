package keys

import (
	"github.com/spf13/cobra"

	"github.com/cosmos/cosmos-sdk/client"
)

// MigrateCommand migrates key information from legacy keybase to OS secret store.
func MigrateCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate",
		Short: "Migrate keys from amino to proto serialization format",
		Long: `Migrate keys from Amino to Protocol Buffers records.
For each key material entry, the command will check if the key can be deserialized using proto.
If this is the case, the key is already migrated. Therefore, we skip it and continue with a next one. 
Otherwise, we try to deserialize it using Amino into LegacyInfo. If this attempt is successful, we serialize 
LegacyInfo to Protobuf serialization format and overwrite the keyring entry. If any error occurred, it will be 
outputted in CLI and migration will be continued until all keys in the keyring DB are exhausted.
See https://github.com/cosmos/cosmos-sdk/pull/9695 for more details.
`,
		Args: cobra.NoArgs,
		RunE: runMigrateCmd,
	}

	return cmd
}

func runMigrateCmd(cmd *cobra.Command, _ []string) error {
	clientCtx, err := client.GetClientQueryContext(cmd)
	if err != nil {
		return err
	}

	if _, err = clientCtx.Keyring.MigrateAll(); err != nil {
		return err
	}

	cmd.Println("Keys migration has been successfully executed.")
	return nil
}
