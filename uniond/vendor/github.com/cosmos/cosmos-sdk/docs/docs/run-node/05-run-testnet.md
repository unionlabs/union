---
sidebar_position: 1
---

# Running a Testnet

:::note Synopsis
The `simd testnet` subcommand makes it easy to initialize and start a simulated test network for testing purposes.
:::

In addition to the commands for [running a node](./01-run-node.md), the `simd` binary also includes a `testnet` command that allows you to start a simulated test network in-process or to initialize files for a simulated test network that runs in a separate process.

## Initialize Files

First, let's take a look at the `init-files` subcommand.

This is similar to the `init` command when initializing a single node, but in this case we are initializing multiple nodes, generating the genesis transactions for each node, and then collecting those transactions.

The `init-files` subcommand initializes the necessary files to run a test network in a separate process (i.e. using a Docker container). Running this command is not a prerequisite for the `start` subcommand ([see below](#start-testnet)).

In order to initialize the files for a test network, run the following command:

```bash
simd testnet init-files
```

You should see the following output in your terminal:

```bash
Successfully initialized 4 node directories
```

The default output directory is a relative `.testnets` directory. Let's take a look at the files created within the `.testnets` directory.

### gentxs

The `gentxs` directory includes a genesis transaction for each validator node. Each file includes a JSON encoded genesis transaction used to register a validator node at the time of genesis. The genesis transactions are added to the `genesis.json` file within each node directory during the initilization process.

### nodes

A node directory is created for each validator node. Within each node directory is a `simd` directory. The `simd` directory is the home directory for each node, which includes the configuration and data files for that node (i.e. the same files included in the default `~/.simapp` directory when running a single node).

## Start Testnet

Now, let's take a look at the `start` subcommand.

The `start` subcommand both initializes and starts an in-process test network. This is the fastest way to spin up a local test network for testing purposes.

You can start the local test network by running the following command:

```bash
simd testnet start
```

You should see something similar to the following:

```bash
acquiring test network lock
preparing test network with chain-id "chain-mtoD9v"


+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
++       THIS MNEMONIC IS FOR TESTING PURPOSES ONLY        ++
++                DO NOT USE IN PRODUCTION                 ++
++                                                         ++
++  sustain know debris minute gate hybrid stereo custom   ++
++  divorce cross spoon machine latin vibrant term oblige  ++
++   moment beauty laundry repeat grab game bronze truly   ++
+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++


starting test network...
started test network
press the Enter Key to terminate
```

The first validator node is now running in-process, which means the test network will terminate once you either close the terminal window or you press the Enter key. In the output, the mnemonic phrase for the first validator node is provided for testing purposes. The validator node is using the same default addresses being used when initializing and starting a single node (no need to provide a `--node` flag).

Check the status of the first validator node:

```shell
simd status
```

Import the key from the provided mnemonic:

```shell
simd keys add test --recover --keyring-backend test
```

Check the balance of the account address:

```shell
simd q bank balances [address]
```

Use this test account to manually test against the test network.

## Testnet Options

You can customize the configuration of the test network with flags. In order to see all flag options, append the `--help` flag to each command.
