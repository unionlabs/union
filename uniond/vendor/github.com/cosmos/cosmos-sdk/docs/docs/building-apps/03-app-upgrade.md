---
sidebar_position: 1
---

# Application upgrade

:::note
This document describes how to upgrade your application. If you are looking specifically for the changes to perform between SDK versions, see the [SDK migrations documentation](https://docs.cosmos.network/main/migrations/intro).
:::

:::warning
This section is currently incomplete. Track the progress of this document [here](https://github.com/cosmos/cosmos-sdk/issues/11504).
:::

## Pre-Upgrade Handling

Cosmovisor supports custom pre-upgrade handling. Use pre-upgrade handling when you need to implement application config changes that are required in the newer version before you perform the upgrade.

Using Cosmovisor pre-upgrade handling is optional. If pre-upgrade handling is not implemented, the upgrade continues.

For example, make the required new-version changes to `app.toml` settings during the pre-upgrade handling. The pre-upgrade handling process means that the file does not have to be manually updated after the upgrade.

Before the application binary is upgraded, Cosmovisor calls a `pre-upgrade` command that can  be implemented by the application.

The `pre-upgrade` command does not take in any command-line arguments and is expected to terminate with the following exit codes:

| Exit status code | How it is handled in Cosmosvisor                                                                                    |
|------------------|---------------------------------------------------------------------------------------------------------------------|
| `0`              | Assumes `pre-upgrade` command executed successfully and continues the upgrade.                                      |
| `1`              | Default exit code when `pre-upgrade` command has not been implemented.                                              |
| `30`             | `pre-upgrade` command was executed but failed. This fails the entire upgrade.                                       |
| `31`             | `pre-upgrade` command was executed but failed. But the command is retried until exit code `1` or `30` are returned. |

## Sample

Here is a sample structure of the `pre-upgrade` command:

```go
func preUpgradeCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "pre-upgrade",
		Short: "Pre-upgrade command",
        Long: "Pre-upgrade command to implement custom pre-upgrade handling",
		Run: func(cmd *cobra.Command, args []string) {

			err := HandlePreUpgrade()

			if err != nil {
				os.Exit(30)
			}

			os.Exit(0)

		},
	}

	return cmd
}
```

Ensure that the pre-upgrade command has been registered in the application:

```go
rootCmd.AddCommand(
		// ..
		preUpgradeCommand(),
		// ..
	)
```
