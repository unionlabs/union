# Running a Chain Software Upgrade

Running a chain software upgrade is a multistep process that will require synchronous organization between all validators of the network.

## 1) Writing the Migration

Before cutting a release, it's important that the necessary migration code has been added to the upgrade module (`uniond/app/upgrades`).

At a minimum this should detail the upgrade name and provide a `CreateUpgradeHandler`. See our existing upgrades for an example.

## 2) Cutting The release

Before anyone starts upgrading their node, a release will need to be conducted so that binaries may be obtained by all validators.

To cut a release, please refer to our `VERSIONING.md`

## 3) Creating the Proposal

After a release artifact has been generated, a proposal will need to be created to conduct an upgrade.

To do this run:

```sh
uniond tx gov draft-proposal
```

Then select `software-upgrade` and fill out the requested information for this release. Make sure to provide an upgrade height that will allow enough time for the proposal to be voted on and provide ample time for validators to prepare.

After this, it will generate a new `.json` file that contains the proposal. You can then run:

```sh
tx gov submit-proposal `$PATH_TO_PROPOSAL`
```

After the proposal has reached its minimum deposit, validators can begin voting on it.

To vote use the `uniond tx gov vote` command providing the proposal ID and `yes` or `no` to cast your vote.

## 4) Migrating Your Node

Finally, you will need to wait until the upgrade height and migrate your node binaries and restart your service. Once this is done, validators will begin producing blocks again after enough have finished the upgrade.

For help in migrating your node, see our [unionvisor](https://github.com/unionlabs/union/tree/3aa2d2ff5e72ba3b4b3a83d898715202c500ea52/unionvisor).
