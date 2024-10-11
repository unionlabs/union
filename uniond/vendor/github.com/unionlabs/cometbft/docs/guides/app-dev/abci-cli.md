---
order: 3
---

# Using ABCI-CLI

To facilitate testing and debugging of ABCI servers and simple apps, we
built a CLI, the `abci-cli`, for sending ABCI messages from the command
line.

## Install

Make sure you [have Go installed](https://golang.org/doc/install).

Next, install the `abci-cli` tool and example applications:

```sh
git clone https://github.com/cometbft/cometbft.git
cd cometbft
make install_abci
```

Now run `abci-cli` to see the list of commands:

```sh
Usage:
  abci-cli [command]

Available Commands:
  batch            run a batch of abci commands against an application
  check_tx         validate a transaction
  commit           commit the application state and return the Merkle root hash
  completion       Generate the autocompletion script for the specified shell
  console          start an interactive ABCI console for multiple commands
  echo             have the application echo a message
  finalize_block   deliver a block of transactions to the application
  help             Help about any command
  info             get some info about the application
  kvstore          ABCI demo example
  prepare_proposal prepare proposal
  process_proposal process proposal
  query            query the application state
  test             run integration tests
  version          print ABCI console version

Flags:
      --abci string        either socket or grpc (default "socket")
      --address string     address of application socket (default "tcp://0.0.0.0:26658")
  -h, --help               help for abci-cli
      --log_level string   set the logger level (default "debug")
  -v, --verbose            print the command and results as if it were a console session

Use "abci-cli [command] --help" for more information about a command.
```

## KVStore - First Example

The `abci-cli` tool lets us send ABCI messages to our application, to
help build and debug them.

The most important messages are `deliver_tx`, `check_tx`, and `commit`,
but there are others for convenience, configuration, and information
purposes.

We'll start a kvstore application, which was installed at the same time as
`abci-cli` above. The kvstore just stores transactions in a Merkle tree. Its
code can be found
[here](https://github.com/cometbft/cometbft/blob/main/abci/example/kvstore/kvstore.go).

Start the application by running:

```sh
abci-cli kvstore
```

And in another terminal, run

```sh
abci-cli echo hello
abci-cli info
```

You'll see something like:

```sh
-> data: hello
-> data.hex: 68656C6C6F
```

and:

```sh
-> data: {"size":0}
-> data.hex: 7B2273697A65223A307D
```

An ABCI application must provide two things:

- a socket server
- a handler for ABCI messages

When we run the `abci-cli` tool we open a new connection to the
application's socket server, send the given ABCI message, and wait for a
response.

The server may be generic for a particular language, and we provide a
[reference implementation in
Golang](https://github.com/cometbft/cometbft/tree/main/abci/server). See the
[list of other ABCI implementations](https://github.com/tendermint/awesome#ecosystem) for servers in
other languages.

The handler is specific to the application, and may be arbitrary, so
long as it is deterministic and conforms to the ABCI interface
specification.

So when we run `abci-cli info`, we open a new connection to the ABCI
server, which calls the `Info()` method on the application, which tells
us the number of transactions in our Merkle tree.

Now, since every command opens a new connection, we provide the
`abci-cli console` and `abci-cli batch` commands, to allow multiple ABCI
messages to be sent over a single connection.

Running `abci-cli console` should drop you in an interactive console for
speaking ABCI messages to your application.

Try running these commands:

```sh
> echo hello
-> code: OK
-> data: hello
-> data.hex: 0x68656C6C6F

> info
-> code: OK
-> data: {"size":0}
-> data.hex: 0x7B2273697A65223A307D

> prepare_proposal "abc=123"
-> code: OK
-> log: Succeeded. Tx: abc=123

> process_proposal "abc==456"
-> code: OK
-> status: PROCESS_PROPOSAL_STATUS_REJECT

> process_proposal "abc=123"
-> code: OK
-> status: PROCESS_PROPOSAL_STATUS_ACCEPT

> finalize_block "abc=123"
-> code: OK
-> code: OK
-> data.hex: 0x0200000000000000

> commit
-> code: OK

> info
-> code: OK
-> data: {"size":1}
-> data.hex: 0x7B2273697A65223A317D

> query "abc"
-> code: OK
-> log: exists
-> height: 0
-> key: abc
-> key.hex: 616263
-> value: 123
-> value.hex: 313233

> finalize_block "def=xyz" "ghi=123"
-> code: OK
-> code: OK
-> code: OK
-> data.hex: 0x0600000000000000

> commit
-> code: OK

> query "def"
-> code: OK
-> log: exists
-> height: 0
-> key: def
-> key.hex: 646566
-> value: xyz
-> value.hex: 78797A
```

Note that if we do `finalize_block "abc" ...` it will store `(abc, abc)`, but if
we do `finalize_block "abc=efg" ...` it will store `(abc, efg)`.

You could put the commands in a file and run
`abci-cli --verbose batch < myfile`.


Note that the `abci-cli` is designed strictly for testing and debugging. In a real
deployment, the role of sending messages is taken by CometBFT, which
connects to the app using four separate connections, each with its own
pattern of messages.

For examples of running an ABCI app with CometBFT, see the
[getting started guide](getting-started.md).

## Bounties

Want to write an app in your favorite language?! We'd be happy
to add you to our [ecosystem](https://github.com/tendermint/awesome#ecosystem)!
See [funding](https://github.com/interchainio/funding) opportunities from the
[Interchain Foundation](https://interchain.io) for implementations in new languages and more.
