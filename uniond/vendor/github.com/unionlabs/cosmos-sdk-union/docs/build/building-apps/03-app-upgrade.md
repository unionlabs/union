---
sidebar_position: 1
---

# Application Upgrade

:::note
This document describes how to upgrade your application. If you are looking specifically for the changes to perform between SDK versions, see the [SDK migrations documentation](https://docs.cosmos.network/main/migrations/intro).
:::

:::warning
This section is currently incomplete. Track the progress of this document [here](https://github.com/cosmos/cosmos-sdk/issues/11504).
:::

:::note Pre-requisite Readings

* [`x/upgrade` Documentation](https://docs.cosmos.network/main/modules/upgrade)

:::

## General Workflow

Let's assume we are running v0.38.0 of our software in our testnet and want to upgrade to v0.40.0.
How would this look in practice? First of all, we want to finalize the v0.40.0 release candidate
and then install a specially named upgrade handler (eg. "testnet-v2" or even "v0.40.0"). An upgrade
handler should be defined in a new version of the software to define what migrations
to run to migrate from the older version of the software. Naturally, this is app-specific rather
than module specific, and  must be defined in `app.go`, even if it imports logic from various
modules to perform the actions. You can register them with `upgradeKeeper.SetUpgradeHandler`
during the app initialization (before starting the abci server), and they serve not only to
perform a migration, but also to identify if this is the old or new version (eg. presence of
a handler registered for the named upgrade).

Once the release candidate along with an appropriate upgrade handler is frozen,
we can have a governance vote to approve this upgrade at some future block height (e.g. 200000).
This is known as an upgrade.Plan. The v0.38.0 code will not know of this handler, but will
continue to run until block 200000, when the plan kicks in at `BeginBlock`. It will check
for existence of the handler, and finding it missing, know that it is running the obsolete software,
and gracefully exit.

Generally the application binary will restart on exit, but then will execute this BeginBlocker
again and exit, causing a restart loop. Either the operator can manually install the new software,
or you can make use of an external watcher daemon to possibly download and then switch binaries,
also potentially doing a backup. The SDK tool for doing such, is called [Cosmovisor](https://docs.cosmos.network/main/build/tooling/cosmovisor).

When the binary restarts with the upgraded version (here v0.40.0), it will detect we have registered the
"testnet-v2" upgrade handler in the code, and realize it is the new version. It then will run the upgrade handler
and *migrate the database in-place*. Once finished, it marks the upgrade as done, and continues processing
the rest of the block as normal. Once 2/3 of the voting power has upgraded, the blockchain will immediately
resume the consensus mechanism. If the majority of operators add a custom `do-upgrade` script, this should
be a matter of minutes and not even require them to be awake at that time.

## Integrating With An App

:::tip
The following is not required for users using `depinject`, this is abstracted for them.
:::

In addition to basic module wiring, setup the upgrade Keeper for the app and then define a `PreBlocker` that calls the upgrade
keeper's PreBlocker method:

```go
func (app *myApp) PreBlocker(ctx sdk.Context, req req.RequestFinalizeBlock) (*sdk.ResponsePreBlock, error) {
      // For demonstration sake, the app PreBlocker only returns the upgrade module pre-blocker.
      // In a real app, the module manager should call all pre-blockers
      // return app.ModuleManager.PreBlock(ctx, req)
      return app.upgradeKeeper.PreBlocker(ctx, req)
}
```

The app must then integrate the upgrade keeper with its governance module as appropriate. The governance module
should call ScheduleUpgrade to schedule an upgrade and ClearUpgradePlan to cancel a pending upgrade.

## Performing Upgrades

Upgrades can be scheduled at a predefined block height. Once this block height is reached, the
existing software will cease to process ABCI messages and a new version with code that handles the upgrade must be deployed.
All upgrades are coordinated by a unique upgrade name that cannot be reused on the same blockchain. In order for the upgrade
module to know that the upgrade has been safely applied, a handler with the name of the upgrade must be installed.
Here is an example handler for an upgrade named "my-fancy-upgrade":

```go
app.upgradeKeeper.SetUpgradeHandler("my-fancy-upgrade", func(ctx context.Context, plan upgrade.Plan) {
 // Perform any migrations of the state store needed for this upgrade
})
```

This upgrade handler performs the dual function of alerting the upgrade module that the named upgrade has been applied,
as well as providing the opportunity for the upgraded software to perform any necessary state migrations. Both the halt
(with the old binary) and applying the migration (with the new binary) are enforced in the state machine. Actually
switching the binaries is an ops task and not handled inside the sdk / abci app.

Here is a sample code to set store migrations with an upgrade:

```go
// this configures a no-op upgrade handler for the "my-fancy-upgrade" upgrade
app.UpgradeKeeper.SetUpgradeHandler("my-fancy-upgrade",  func(ctx context.Context, plan upgrade.Plan) {
 // upgrade changes here
})
upgradeInfo, err := app.UpgradeKeeper.ReadUpgradeInfoFromDisk()
if err != nil {
 // handle error
}
if upgradeInfo.Name == "my-fancy-upgrade" && !app.UpgradeKeeper.IsSkipHeight(upgradeInfo.Height) {
 storeUpgrades := store.StoreUpgrades{
  Renamed: []store.StoreRename{{
   OldKey: "foo",
   NewKey: "bar",
  }},
  Deleted: []string{},
 }
 // configure store loader that checks if version == upgradeHeight and applies store upgrades
 app.SetStoreLoader(upgrade.UpgradeStoreLoader(upgradeInfo.Height, &storeUpgrades))
}
```

## Halt Behavior

Before halting the ABCI state machine in the BeginBlocker method, the upgrade module will log an error
that looks like:

```text
 UPGRADE "<Name>" NEEDED at height <NNNN>: <Info>
```

where `Name` and `Info` are the values of the respective fields on the upgrade Plan.

To perform the actual halt of the blockchain, the upgrade keeper simply panics which prevents the ABCI state machine
from proceeding but doesn't actually exit the process. Exiting the process can cause issues for other nodes that start
to lose connectivity with the exiting nodes, thus this module prefers to just halt but not exit.

## Automation

Read more about [Cosmovisor](https://docs.cosmos.network/main/build/tooling/cosmovisor), the tool for automating upgrades.

## Canceling Upgrades

There are two ways to cancel a planned upgrade - with on-chain governance or off-chain social consensus.
For the first one, there is a `CancelSoftwareUpgrade` governance proposal, which can be voted on and will
remove the scheduled upgrade plan. Of course this requires that the upgrade was known to be a bad idea
well before the upgrade itself, to allow time for a vote. If you want to allow such a possibility, you
should set the upgrade height to be `2 * (votingperiod + depositperiod) + (safety delta)` from the beginning of
the first upgrade proposal. Safety delta is the time available from the success of an upgrade proposal
and the realization it was a bad idea (due to external testing). You can also start a `CancelSoftwareUpgrade`
proposal while the original `SoftwareUpgrade` proposal is still being voted upon, as long as the voting
period ends after the `SoftwareUpgrade` proposal.

However, let's assume that we don't realize the upgrade has a bug until shortly before it will occur
(or while we try it out - hitting some panic in the migration). It would seem the blockchain is stuck,
but we need to allow an escape for social consensus to overrule the planned upgrade. To do so, there's
a `--unsafe-skip-upgrades` flag to the start command, which will cause the node to mark the upgrade
as done upon hitting the planned upgrade height(s), without halting and without actually performing a migration.
If over two-thirds run their nodes with this flag on the old binary, it will allow the chain to continue through
the upgrade with a manual override. (This must be well-documented for anyone syncing from genesis later on).

Example:

```shell
<appd> start --unsafe-skip-upgrades <height1> <optional_height_2> ... <optional_height_N>
```

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

When not using Cosmovisor, ensure to run `<appd> pre-upgrade` before starting the application binary.
