---
order: 2
---

# Using CometBFT

This is a guide to using the `cometbft` program from the command line.
It assumes only that you have the `cometbft` binary installed and have
some rudimentary idea of what CometBFT and ABCI are.

You can see the help menu with `cometbft --help`, and the version
number with `cometbft version`.

## Directory Root

The default directory for blockchain data is `~/.cometbft`. Override
this by setting the `CMTHOME` environment variable.

## Initialize

Initialize the root directory by running:

```sh
cometbft init
```

This will create a new private key (`priv_validator_key.json`), and a
genesis file (`genesis.json`) containing the associated public key, in
`$CMTHOME/config`. This is all that's necessary to run a local testnet
with one validator.

For more elaborate initialization, see the testnet command:

```sh
cometbft testnet --help
```

### Genesis

The `genesis.json` file in `$CMTHOME/config/` defines the initial
CometBFT state upon genesis of the blockchain ([see
definition](https://github.com/cometbft/cometbft/blob/main/types/genesis.go)).

#### Fields

- `genesis_time`: Official time of blockchain start.
- `chain_id`: ID of the blockchain. **This must be unique for
  every blockchain.** If your testnet blockchains do not have unique
  chain IDs, you will have a bad time. The ChainID must be less than 50 symbols.
- `initial_height`: Height at which CometBFT should begin at. If a blockchain is conducting a network upgrade,
    starting from the stopped height brings uniqueness to previous heights.
- `consensus_params` ([see spec](https://github.com/cometbft/cometbft/blob/main/spec/core/data_structures.md#consensusparams))
    - `block`
        - `max_bytes`: Max block size, in bytes.
        - `max_gas`: Max gas per block.
    - `evidence`
        - `max_age_num_blocks`: Max age of evidence, in blocks. The basic formula
      for calculating this is: MaxAgeDuration / {average block time}.
        - `max_age_duration`: Max age of evidence, in time. It should correspond
      with an app's "unbonding period" or other similar mechanism for handling
      [Nothing-At-Stake
      attacks](https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed).
        - `max_bytes`: This sets the maximum size in bytes of evidence that can be committed
      in a single block and should fall comfortably under the max block bytes.
    - `validator`
        - `pub_key_types`: Public key types validators can use.
    - `version`
        - `app_version`: ABCI application version.
- `validators`: List of initial validators. Note this may be overridden entirely by the
  application, and may be left empty to make explicit that the
  application will initialize the validator set upon `InitChain`.
    - `pub_key`: The first element specifies the key type,
    using the declared `PubKeyName` for the adopted
    [key type](https://github.com/cometbft/cometbft/blob/main/crypto/ed25519/ed25519.go#L36).
    The second element are the pubkey bytes.
    - `power`: The validator's voting power.
    - `name`: Name of the validator (optional).
- `app_hash`: The expected application hash (as returned by the
  `ResponseInfo` ABCI message) upon genesis. If the app's hash does
  not match, CometBFT will panic.
- `app_state`: The application state (e.g. initial distribution
  of tokens).

> :warning: **ChainID must be unique to every blockchain. Reusing old chainID can cause issues**

#### Sample genesis.json

```json
{
  "genesis_time": "2023-01-21T11:17:42.341227868Z",
  "chain_id": "test-chain-ROp9KF",
  "initial_height": "0",
  "consensus_params": {
    "block": {
      "max_bytes": "4194304",
      "max_gas": "10000000",
    },
    "evidence": {
      "max_age_num_blocks": "100000",
      "max_age_duration": "172800000000000",
      "max_bytes": 51200,
    },
    "validator": {
      "pub_key_types": [
        "ed25519"
      ]
    }
  },
  "validators": [
    {
      "address": "B547AB87E79F75A4A3198C57A8C2FDAF8628CB47",
      "pub_key": {
        "type": "tendermint/PubKeyEd25519",
        "value": "P/V6GHuZrb8rs/k1oBorxc6vyXMlnzhJmv7LmjELDys="
      },
      "power": "10",
      "name": ""
    }
  ],
  "app_hash": ""
}
```

## Run

To run a CometBFT node, use:

```bash
cometbft node
```

By default, CometBFT will try to connect to an ABCI application on
`tcp://127.0.0.1:26658`. If you have the `kvstore` ABCI app installed, run it in
another window. If you don't, kill CometBFT and run an in-process version of
the `kvstore` app:

```bash
cometbft node --proxy_app=kvstore
```

After a few seconds, you should see blocks start streaming in. Note that blocks
are produced regularly, even if there are no transactions. See [No Empty
Blocks](#no-empty-blocks), below, to modify this setting.

CometBFT supports in-process versions of the `counter`, `kvstore`, and `noop`
apps that ship as examples with `abci-cli`. It's easy to compile your app
in-process with CometBFT if it's written in Go. If your app is not written in
Go, run it in another process, and use the `--proxy_app` flag to specify the
address of the socket it is listening on, for instance:

```bash
cometbft node --proxy_app=/var/run/abci.sock
```

You can find out what flags are supported by running `cometbft node --help`.

## Transactions

To send a transaction, use `curl` to make requests to the CometBFT RPC
server, for example:

```sh
curl http://localhost:26657/broadcast_tx_commit?tx=\"abcd\"
```

We can see the chain's status at the `/status` end-point:

```sh
curl http://localhost:26657/status | json_pp
```

and the `latest_app_hash` in particular:

```sh
curl http://localhost:26657/status | json_pp | grep latest_app_hash
```

Visit `http://localhost:26657` in your browser to see the list of other
endpoints. Some take no arguments (like `/status`), while others specify
the argument name and use `_` as a placeholder.


> TIP: Find the RPC Documentation [here](https://docs.cometbft.com/main/rpc/)

### Formatting

The following nuances when sending/formatting transactions should be
taken into account:

With `GET`:

To send a UTF8 string byte array, quote the value of the tx parameter:

```sh
curl 'http://localhost:26657/broadcast_tx_commit?tx="hello"'
```

which sends a 5 byte transaction: "h e l l o" \[68 65 6c 6c 6f\].

Note the URL must be wrapped with single quotes, else bash will ignore
the double quotes. To avoid the single quotes, escape the double quotes:

```sh
curl http://localhost:26657/broadcast_tx_commit?tx=\"hello\"
```

Using a special character:

```sh
curl 'http://localhost:26657/broadcast_tx_commit?tx="€5"'
```

sends a 4 byte transaction: "€5" (UTF8) \[e2 82 ac 35\].

To send as raw hex, omit quotes AND prefix the hex string with `0x`:

```sh
curl http://localhost:26657/broadcast_tx_commit?tx=0x01020304
```

which sends a 4 byte transaction: \[01 02 03 04\].

With `POST` (using `json`), the raw hex must be `base64` encoded:

```sh
curl --data-binary '{"jsonrpc":"2.0","id":"anything","method":"broadcast_tx_commit","params": {"tx": "AQIDBA=="}}' -H 'Content-Type:text/plain;' http://localhost:26657
```

which sends the same 4 byte transaction: \[01 02 03 04\].

Note that raw hex cannot be used in `POST` transactions.

## Reset

> :warning: **UNSAFE** Only do this in development and only if you can
afford to lose all blockchain data!


To reset a blockchain, stop the node and run:

```sh
cometbft unsafe_reset_all
```

This command will remove the data directory and reset private validator and
address book files.

## Configuration

CometBFT uses a `config.toml` for configuration. For details, see [the
config specification](configuration.md).

Notable options include the socket address of the application
(`proxy_app`), the listening address of the CometBFT peer
(`p2p.laddr`), and the listening address of the RPC server
(`rpc.laddr`).

Some fields from the config file can be overwritten with flags.

## No Empty Blocks

While the default behavior of `cometbft` is still to create blocks
approximately once per second, it is possible to disable empty blocks or
set a block creation interval. In the former case, blocks will be
created when there are new transactions or when the AppHash changes.

To configure CometBFT to not produce empty blocks unless there are
transactions or the app hash changes, run CometBFT with this
additional flag:

```sh
cometbft node --consensus.create_empty_blocks=false
```

or set the configuration via the `config.toml` file:

```toml
[consensus]
create_empty_blocks = false
```

Remember: because the default is to _create empty blocks_, avoiding
empty blocks requires the config option to be set to `false`.

The block interval setting allows for a delay (in time.Duration format [ParseDuration](https://golang.org/pkg/time/#ParseDuration)) between the
creation of each new empty block. It can be set with this additional flag:

```sh
--consensus.create_empty_blocks_interval="5s"
```

or set the configuration via the `config.toml` file:

```toml
[consensus]
create_empty_blocks_interval = "5s"
```

With this setting, empty blocks will be produced every 5s if no block
has been produced otherwise, regardless of the value of
`create_empty_blocks`.

## Broadcast API

Earlier, we used the `broadcast_tx_commit` endpoint to send a
transaction. When a transaction is sent to a CometBFT node, it will
run via `CheckTx` against the application. If it passes `CheckTx`, it
will be included in the mempool, broadcasted to other peers, and
eventually included in a block.

Since there are multiple phases to processing a transaction, we offer
multiple endpoints to broadcast a transaction:

```md
/broadcast_tx_async
/broadcast_tx_sync
/broadcast_tx_commit
```

These correspond to no-processing, processing through the mempool, and
processing through a block, respectively. That is, `broadcast_tx_async`,
will return right away without waiting to hear if the transaction is
even valid, while `broadcast_tx_sync` will return with the result of
running the transaction through `CheckTx`. Using `broadcast_tx_commit`
will wait until the transaction is committed in a block or until some
timeout is reached, but will return right away if the transaction does
not pass `CheckTx`. The return value for `broadcast_tx_commit` includes
two fields, `check_tx` and `deliver_tx`, pertaining to the result of
running the transaction through those ABCI messages.

The benefit of using `broadcast_tx_commit` is that the request returns
after the transaction is committed (i.e. included in a block), but that
can take on the order of a second. For a quick result, use
`broadcast_tx_sync`, but the transaction will not be committed until
later, and by that point its effect on the state may change.

Note the mempool does not provide strong guarantees - just because a tx passed
CheckTx (ie. was accepted into the mempool), doesn't mean it will be committed,
as nodes with the tx in their mempool may crash before they get to propose.
For more information, see the [mempool
write-ahead-log](running-in-production.md#mempool-wal)

## CometBFT Networks

When `cometbft init` is run, both a `genesis.json` and
`priv_validator_key.json` are created in `~/.cometbft/config`. The
`genesis.json` might look like:

```json
{
  "validators" : [
    {
      "pub_key" : {
        "value" : "h3hk+QE8c6QLTySp8TcfzclJw/BG79ziGB/pIA+DfPE=",
        "type" : "tendermint/PubKeyEd25519"
      },
      "power" : 10,
      "name" : ""
    }
  ],
  "app_hash" : "",
  "chain_id" : "test-chain-rDlYSN",
  "genesis_time" : "0001-01-01T00:00:00Z"
}
```

And the `priv_validator_key.json`:

```json
{
  "last_step" : 0,
  "last_round" : "0",
  "address" : "B788DEDE4F50AD8BC9462DE76741CCAFF87D51E2",
  "pub_key" : {
    "value" : "h3hk+QE8c6QLTySp8TcfzclJw/BG79ziGB/pIA+DfPE=",
    "type" : "tendermint/PubKeyEd25519"
  },
  "last_height" : "0",
  "priv_key" : {
    "value" : "JPivl82x+LfVkp8i3ztoTjY6c6GJ4pBxQexErOCyhwqHeGT5ATxzpAtPJKnxNx/NyUnD8Ebv3OIYH+kgD4N88Q==",
    "type" : "tendermint/PrivKeyEd25519"
  }
}
```

The `priv_validator_key.json` actually contains a private key, and should
thus be kept absolutely secret; for now we work with the plain text.
Note the `last_` fields, which are used to prevent us from signing
conflicting messages.

Note also that the `pub_key` (the public key) in the
`priv_validator_key.json` is also present in the `genesis.json`.

The genesis file contains the list of public keys which may participate
in the consensus, and their corresponding voting power. Greater than 2/3
of the voting power must be active (i.e. the corresponding private keys
must be producing signatures) for the consensus to make progress. In our
case, the genesis file contains the public key of our
`priv_validator_key.json`, so a CometBFT node started with the default
root directory will be able to make progress. Voting power uses an int64
but must be positive, thus the range is: 0 through 9223372036854775807.
Because of how the current proposer selection algorithm works, we do not
recommend having voting powers greater than 10\^12 (ie. 1 trillion).

If we want to add more nodes to the network, we have two choices: we can
add a new validator node, who will also participate in the consensus by
proposing blocks and voting on them, or we can add a new non-validator
node, who will not participate directly, but will verify and keep up
with the consensus protocol.

### Peers

#### Seed

A seed node is a node who relays the addresses of other peers which they know
of. These nodes constantly crawl the network to try to get more peers. The
addresses which the seed node relays get saved into a local address book. Once
these are in the address book, you will connect to those addresses directly.
Basically the seed nodes job is just to relay everyones addresses. You won't
connect to seed nodes once you have received enough addresses, so typically you
only need them on the first start. The seed node will immediately disconnect
from you after sending you some addresses.

#### Persistent Peer

Persistent peers are people you want to be constantly connected with. If you
disconnect you will try to connect directly back to them as opposed to using
another address from the address book. On restarts you will always try to
connect to these peers regardless of the size of your address book.

All peers relay peers they know of by default. This is called the peer exchange
protocol (PEX). With PEX, peers will be gossiping about known peers and forming
a network, storing peer addresses in the addrbook. Because of this, you don't
have to use a seed node if you have a live persistent peer.

#### Connecting to Peers

To connect to peers on start-up, specify them in the
`$CMTHOME/config/config.toml` or on the command line. Use `seeds` to
specify seed nodes, and
`persistent_peers` to specify peers that your node will maintain
persistent connections with.

For example,

```sh
cometbft node --p2p.seeds "f9baeaa15fedf5e1ef7448dd60f46c01f1a9e9c4@1.2.3.4:26656,0491d373a8e0fcf1023aaf18c51d6a1d0d4f31bd@5.6.7.8:26656"
```

Alternatively, you can use the `/dial_seeds` endpoint of the RPC to
specify seeds for a running node to connect to:

```sh
curl 'localhost:26657/dial_seeds?seeds=\["f9baeaa15fedf5e1ef7448dd60f46c01f1a9e9c4@1.2.3.4:26656","0491d373a8e0fcf1023aaf18c51d6a1d0d4f31bd@5.6.7.8:26656"\]'
```

Note, with PEX enabled, you
should not need seeds after the first start.

If you want CometBFT to connect to specific set of addresses and
maintain a persistent connection with each, you can use the
`--p2p.persistent_peers` flag or the corresponding setting in the
`config.toml` or the `/dial_peers` RPC endpoint to do it without
stopping CometBFT instance.

```sh
cometbft node --p2p.persistent_peers "429fcf25974313b95673f58d77eacdd434402665@10.11.12.13:26656,96663a3dd0d7b9d17d4c8211b191af259621c693@10.11.12.14:26656"

curl 'localhost:26657/dial_peers?persistent=true&peers=\["429fcf25974313b95673f58d77eacdd434402665@10.11.12.13:26656","96663a3dd0d7b9d17d4c8211b191af259621c693@10.11.12.14:26656"\]'
```

### Adding a Non-Validator

Adding a non-validator is simple. Just copy the original `genesis.json`
to `~/.cometbft/config` on the new machine and start the node,
specifying seeds or persistent peers as necessary. If no seeds or
persistent peers are specified, the node won't make any blocks, because
it's not a validator, and it won't hear about any blocks, because it's
not connected to the other peer.

### Adding a Validator

The easiest way to add new validators is to do it in the `genesis.json`,
before starting the network. For instance, we could make a new
`priv_validator_key.json`, and copy it's `pub_key` into the above genesis.

We can generate a new `priv_validator_key.json` with the command:

```sh
cometbft gen_validator
```

Now we can update our genesis file. For instance, if the new
`priv_validator_key.json` looks like:

```json
{
  "address" : "5AF49D2A2D4F5AD4C7C8C4CC2FB020131E9C4902",
  "pub_key" : {
    "value" : "l9X9+fjkeBzDfPGbUM7AMIRE6uJN78zN5+lk5OYotek=",
    "type" : "tendermint/PubKeyEd25519"
  },
  "priv_key" : {
    "value" : "EDJY9W6zlAw+su6ITgTKg2nTZcHAH1NMTW5iwlgmNDuX1f35+OR4HMN88ZtQzsAwhETq4k3vzM3n6WTk5ii16Q==",
    "type" : "tendermint/PrivKeyEd25519"
  },
  "last_step" : 0,
  "last_round" : "0",
  "last_height" : "0"
}
```

then the new `genesis.json` will be:

```json
{
  "validators" : [
    {
      "pub_key" : {
        "value" : "h3hk+QE8c6QLTySp8TcfzclJw/BG79ziGB/pIA+DfPE=",
        "type" : "tendermint/PubKeyEd25519"
      },
      "power" : 10,
      "name" : ""
    },
    {
      "pub_key" : {
        "value" : "l9X9+fjkeBzDfPGbUM7AMIRE6uJN78zN5+lk5OYotek=",
        "type" : "cometbft/PubKeyEd25519"
      },
      "power" : 10,
      "name" : ""
    }
  ],
  "app_hash" : "",
  "chain_id" : "test-chain-rDlYSN",
  "genesis_time" : "0001-01-01T00:00:00Z"
}
```

Update the `genesis.json` in `~/.cometbft/config`. Copy the genesis
file and the new `priv_validator_key.json` to the `~/.cometbft/config` on
a new machine.

Now run `cometbft node` on both machines, and use either
`--p2p.persistent_peers` or the `/dial_peers` to get them to peer up.
They should start making blocks, and will only continue to do so as long
as both of them are online.

To make a CometBFT network that can tolerate one of the validators
failing, you need at least four validator nodes (e.g., 2/3).

Updating validators in a live network is supported but must be
explicitly programmed by the application developer. See the [application
developers guide](../../guides/app-dev/abci-cli.md) for more details.

### Local Network

To run a network locally, say on a single machine, you must change the `_laddr`
fields in the `config.toml` (or using the flags) so that the listening
addresses of the various sockets don't conflict. Additionally, you must set
`addr_book_strict=false` in the `config.toml`, otherwise CometBFT's p2p
library will deny making connections to peers with the same IP address.

### Upgrading

See the
[UPGRADING.md](https://github.com/cometbft/cometbft/blob/main/UPGRADING.md)
guide. You may need to reset your chain between major breaking releases.
Although, we expect CometBFT to have fewer breaking releases in the future
(especially after 1.0 release).
