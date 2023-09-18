---
sidebar_position: 1
---

# Running in Production

:::note Synopsis
This section describes how to securely run a node in a public setting and/or on a mainnet on one of the many Cosmos SDK public blockchains. 
:::

When operating a node, full node or validator, in production it is important to set your server up securely. 

:::note
There are many different ways to secure a server and your node, the described steps here is one way. To see another way of setting up a server see the [run in production tutorial](https://tutorials.cosmos.network/hands-on-exercise/5-run-in-prod/1-overview.html).
:::

:::note
This walkthrough assumes the underlying operating system is Ubuntu. 
:::

## Sever Setup

### User

When creating a server most times it is created as user `root`. This user has heightened privileges on the server. When operating a node, it is recommended to not run your node as the root user.  

1. Create a new user

```bash
sudo adduser change_me
```

2. We want to allow this user to perform sudo tasks

```bash
sudo usermod -aG sudo change_me
```

Now when logging into the server, the non `root` user can be used. 

### Go

1. Install the [Go](https://go.dev/doc/install) version preconized by the application.

:::warning
In the past, validators [have had issues](https://github.com/cosmos/cosmos-sdk/issues/13976) when using different versions of Go. It is recommended that the whole validator set uses the version of Go that is preconized by the application.
:::

### Firewall

Nodes should not have all ports open to the public, this is a simple way to get DDOS'd. Secondly it is recommended by [CometBFT](github.com/cometbft/cometbft) to never expose ports that are not required to operate a node. 

When setting up a firewall there are a few ports that can be open when operating a Cosmos SDK node. There is the CometBFT json-RPC, prometheus, p2p, remote signer and Cosmos SDK GRPC and REST. If the node is being operated as a node that does not offer endpoints to be used for submission or querying then a max of three endpoints are needed.

Most, if not all servers come equipped with [ufw](https://help.ubuntu.com/community/UFW). Ufw will be used in this tutorial. 

1. Reset UFW to disallow all incoming connections and allow outgoing

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
```

2. Lets make sure that port 22 (ssh) stays open. 

```bash
sudo ufw allow ssh
```

or 

```bash
sudo ufw allow 22
```
Both of the above commands are the same. 

3. Allow Port 26656 (cometbft p2p port). If the node has a modified p2p port then that port must be used here.

```bash
sudo ufw allow 26656/tcp
```

4. Allow port 26660 (cometbft [prometheus](https://prometheus.io)). This acts as the applications monitoring port as well. 

```bash
sudo ufw allow 26660/tcp
```

5. IF the node which is being setup would like to expose CometBFTs jsonRPC and Cosmos SDK GRPC and REST then follow this step. (Optional)

##### CometBFT JsonRPC

```bash
sudo ufw allow 26657/tcp
```

##### Cosmos SDK GRPC

```bash
sudo ufw allow 9090/tcp
```

##### Cosmos SDK REST

```bash
sudo ufw allow 1317/tcp
```

6. Lastly, enable ufw

```bash
sudo ufw enable
```

### Signing

If the node that is being started is a validator there are multiple ways a validator could sign blocks. 

#### File

File based signing is the simplest and default approach. This approach works by storing the consensus key, generated on initialization, to sign blocks. This approach is only as safe as your server setup as if the server is compromised so is your key.  This key is located in the `config/priv_val_key.json` directory generated on initialization.

A second file exists that user must be aware of, the file is located in the data directory `data/priv_val_state.json`. This file protects your node from double signing. It keeps track of the consensus keys last sign height, round and latest signature. If the node crashes and needs to be recovered this file must be kept in order to ensure that the consensus key will not be used for signing a block that was previously signed. 

#### Remote Signer

A remote signer is a secondary server that is separate from the running node that signs blocks with the consensus key. This means that the consensus key does not live on the node itself. This increases security because your full node which is connected to the remote signer can be swapped without missing blocks. 

The two most used remote signers are [tmkms](https://github.com/iqlusioninc/tmkms) from [Iqlusion](https://www.iqlusion.io) and [horcrux](https://github.com/strangelove-ventures/horcrux) from [Strangelove](https://strange.love).

##### TMKMS 

###### Dependencies

1. Update server dependencies and install extras needed. 

```sh
sudo apt update -y && sudo apt install build-essential curl jq -y
```

2. Install Rust: 

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. Install Libusb:

```sh
sudo apt install libusb-1.0-0-dev
```

###### Setup

There are two ways to install tmkms, from source or `cargo install`. In the examples we will cover downloading or building from source and using softsign. Softsign stands for software signing, but you could use a [yubihsm](https://www.yubico.com/products/hardware-security-module/) as your signing key if you wish. 

1. Build:

From source:

```bash
cd $HOME
git clone https://github.com/iqlusioninc/tmkms.git
cd $HOME/tmkms
cargo install tmkms --features=softsign
tmkms init config
tmkms softsign keygen ./config/secrets/secret_connection_key
```
or 

Cargo install: 

```bash
cargo install tmkms --features=softsign
tmkms init config
tmkms softsign keygen ./config/secrets/secret_connection_key
```

:::note
To use tmkms with a yubikey install the binary with `--features=yubihsm`.
:::

2. Migrate the validator key from the full node to the new tmkms instance. 

```bash
scp user@123.456.32.123:~/.simd/config/priv_validator_key.json ~/tmkms/config/secrets
```

3. Import the validator key into tmkms. 

```bash
tmkms softsign import $HOME/tmkms/config/secrets/priv_validator_key.json $HOME/tmkms/config/secrets/priv_validator_key
```

At this point, it is necessary to delete the `priv_validator_key.json` from the validator node and the tmkms node. Since the key has been imported into tmkms (above) it is no longer necessary on the nodes. The key can be safely stored offline. 

4. Modifiy the `tmkms.toml`. 

```bash
vim $HOME/tmkms/config/tmkms.toml
```

This example shows a configuration that could be used for soft signing. The example has an IP of `123.456.12.345` with a port of `26659` a chain_id of `test-chain-waSDSe`. These are items that most be modified for the usecase of tmkms and the network. 

```toml
# CometBFT KMS configuration file

## Chain Configuration

[[chain]]
id = "osmosis-1"
key_format = { type = "bech32", account_key_prefix = "cosmospub", consensus_key_prefix = "cosmosvalconspub" }
state_file = "/root/tmkms/config/state/priv_validator_state.json"

## Signing Provider Configuration

### Software-based Signer Configuration

[[providers.softsign]]
chain_ids = ["test-chain-waSDSe"]
key_type = "consensus"
path = "/root/tmkms/config/secrets/priv_validator_key"

## Validator Configuration

[[validator]]
chain_id = "test-chain-waSDSe"
addr = "tcp://123.456.12.345:26659"
secret_key = "/root/tmkms/config/secrets/secret_connection_key"
protocol_version = "v0.34"
reconnect = true
```

5. Set the address of the tmkms instance. 

```bash
vim $HOME/.simd/config/config.toml

priv_validator_laddr = "tcp://0.0.0.0:26659"
```

:::tip
The above address it set to `0.0.0.0` but it is recommended to set the tmkms server to secure the startup
:::

:::tip
It is recommended to comment or delete the lines that specify the path of the validator key and validator:

```toml
# Path to the JSON file containing the private key to use as a validator in the consensus protocol
# priv_validator_key_file = "config/priv_validator_key.json"

# Path to the JSON file containing the last sign state of a validator
# priv_validator_state_file = "data/priv_validator_state.json"
```
:::

6. Start the two processes. 

```bash
tmkms start -c $HOME/tmkms/config/tmkms.toml
```

```bash
simd start
```
