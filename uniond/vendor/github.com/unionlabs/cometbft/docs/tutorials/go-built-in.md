---
order: 4
---

# Creating a built-in application in Go

## Guide Assumptions

This guide is designed for beginners who want to get started with a CometBFT
application from scratch. It does not assume that you have any prior
experience with CometBFT.

CometBFT is a service that provides a Byzantine Fault Tolerant consensus engine
for state-machine replication. The replicated state-machine, or "application", can be written
in any language that can send and receive protocol buffer messages in a client-server model.
Applications written in Go can also use CometBFT as a library and run the service in the same
process as the application.

By following along this tutorial you will create a CometBFT application called kvstore,
a (very) simple distributed BFT key-value store.
The application will be written in Go and
some understanding of the Go programming language is expected.
If you have never written Go, you may want to go through [Learn X in Y minutes
Where X=Go](https://learnxinyminutes.com/docs/go/) first, to familiarize
yourself with the syntax.

Note: Please use the latest released version of this guide and of CometBFT.
We strongly advise against using unreleased commits for your development.

### Built-in app vs external app

On the one hand, to get maximum performance you can run your application in
the same process as the CometBFT, as long as your application is written in Go.
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) is written
this way.
This is the approach followed in this tutorial.

On the other hand, having a separate application might give you better security
guarantees as two processes would be communicating via established binary protocol.
CometBFT will not have access to application's state.
If that is the way you wish to proceed, use the [Creating an application in Go](go.md) guide instead of this one.

## 1.0 Installing Go

Verify that you have the latest version of Go installed (refer to the [official guide for installing Go](https://golang.org/doc/install)), ,
ou should see an output similar to this one (the `go version` might be a slight different depending on the Go you
have installed and the computer platform):

```bash
$ go version
go version go1.22.5 darwin/amd64

```

## 1.1 Installing CometBFT

Let's install `CometBFT` locally by running the following command:

```bash
go install github.com/cometbft/cometbft/cmd/cometbft@v1.0
```

Test the installation:

```bash
$ cometbft version
v1.0.0
```

## 1.2 Creating a new Go project

We'll start by creating a new Go project and moving into the new directory:

```bash
mkdir kvstore
cd kvstore
```

Inside the `kvstore` directory, create a `main.go` file with the following content:

```go
package main

import (
    "fmt"
)

func main() {
    fmt.Println("Hello, CometBFT")
}
```

Run the following command:

```bash
go run main.go
```

This should print "Hello, CometBFT" to the standard output.

```bash
Hello, CometBFT
```

We are going to use [Go modules](https://github.com/golang/go/wiki/Modules) for
dependency management, so let's start by including a dependency on the latest version of
CometBFT, `v1.0` in this example.

Run the commands below to create the go module file (`go.mod`)

```bash
go mod init kvstore
```

This should an output similar to this.

**NOTE**: No need to run `go mod tidy` at this time, just ignore it for now.

```bash
go: creating new go.mod: module kvstore
go: to add module requirements and sums:
	go mod tidy
```

go 1.22.5
Now, lets add `cometbft` as a dependency to our project. Run the `go get` command below:

```bash
go get github.com/cometbft/cometbft@v1.0
```

**NOTE**: This will add the latest release in the `v1.0` line, so you might a different patch release e.g. `v1.0.0`
or `v1.0.1`

```bash
go: added github.com/cometbft/cometbft v1.0.0
```

`
After running the above commands you will see two generated files, `go.mod` and `go.sum`.
The go.mod file should look similar to:

```go
module kvstore

go 1.22.5

require github.com/cometbft/cometbft v1.0.0 // indirect
```

As you write the kvstore application, you can rebuild the binary by
pulling any new dependencies and recompiling it.

```bash
go get
go build
```

At this point, if you ran the `go build` command above, you should see four files in the directory:

```bash
$ ls
go.mod	go.sum	kvstore	main.go
```

The `kvstore` file is the executable generated. You can run it again to ensure everything still works:

```bash
$ ./kvstore
Hello, CometBFT
```

## 1.3 Writing a CometBFT application

CometBFT communicates with the application through the Application
BlockChain Interface (`ABCI`). The messages exchanged through the interface are
defined in the ABCI [protobuf
file](https://github.com/cometbft/cometbft/blob/main/proto/cometbft/abci/v1/types.proto).

We begin by creating the basic scaffolding for an ABCI application by
creating a new type, `KVStoreApplication`, which implements the
methods defined by the [abcitypes.Application](https://github.com/cometbft/cometbft/blob/main/abci/types/application.go)
interface.

Create a file called `app.go` with the following contents:

```go
package main

import (
    abcitypes "github.com/cometbft/cometbft/abci/types"
    "context"
)

type KVStoreApplication struct{}

var _ abcitypes.Application = (*KVStoreApplication)(nil)

func NewKVStoreApplication() *KVStoreApplication {
    return &KVStoreApplication{}
}

func (app *KVStoreApplication) Info(_ context.Context, info *abcitypes.InfoRequest) (*abcitypes.InfoResponse, error) {
    return &abcitypes.InfoResponse{}, nil
}

func (app *KVStoreApplication) Query(_ context.Context, req *abcitypes.QueryRequest) (*abcitypes.QueryResponse, error) {
    return &abcitypes.QueryResponse{}, nil
}

func (app *KVStoreApplication) CheckTx(_ context.Context, check *abcitypes.CheckTxRequest) (*abcitypes.CheckTxResponse, error) {
	return &abcitypes.CheckTxResponse{Code: 0}, nil
}

func (app *KVStoreApplication) InitChain(_ context.Context, chain *abcitypes.InitChainRequest) (*abcitypes.InitChainResponse, error) {
    return &abcitypes.InitChainResponse{}, nil
}

func (app *KVStoreApplication) PrepareProposal(_ context.Context, proposal *abcitypes.PrepareProposalRequest) (*abcitypes.PrepareProposalResponse, error) {
    return &abcitypes.PrepareProposalResponse{}, nil
}

func (app *KVStoreApplication) ProcessProposal(_ context.Context, proposal *abcitypes.ProcessProposalRequest) (*abcitypes.ProcessProposalResponse, error) {
    return &abcitypes.ProcessProposalResponse{}, nil
}

func (app *KVStoreApplication) FinalizeBlock(_ context.Context, req *abcitypes.FinalizeBlockRequest) (*abcitypes.FinalizeBlockResponse, error) {
    return &abcitypes.FinalizeBlockResponse{}, nil
}

func (app KVStoreApplication) Commit(_ context.Context, commit *abcitypes.CommitRequest) (*abcitypes.CommitResponse, error) {
    return &abcitypes.CommitResponse{}, nil
}

func (app *KVStoreApplication) ListSnapshots(_ context.Context, snapshots *abcitypes.ListSnapshotsRequest) (*abcitypes.ListSnapshotsResponse, error) {
    return &abcitypes.ListSnapshotsResponse{}, nil
}

func (app *KVStoreApplication) OfferSnapshot(_ context.Context, snapshot *abcitypes.OfferSnapshotRequest) (*abcitypes.OfferSnapshotResponse, error) {
    return &abcitypes.OfferSnapshotResponse{}, nil
}

func (app *KVStoreApplication) LoadSnapshotChunk(_ context.Context, chunk *abcitypes.LoadSnapshotChunkRequest) (*abcitypes.LoadSnapshotChunkResponse, error) {
    return &abcitypes.LoadSnapshotChunkResponse{}, nil
}

func (app *KVStoreApplication) ApplySnapshotChunk(_ context.Context, chunk *abcitypes.ApplySnapshotChunkRequest) (*abcitypes.ApplySnapshotChunkResponse, error) {
    return &abcitypes.ApplySnapshotChunkResponse{Result: abcitypes.APPLY_SNAPSHOT_CHUNK_RESULT_ACCEPT}, nil
}

func (app KVStoreApplication) ExtendVote(_ context.Context, extend *abcitypes.ExtendVoteRequest) (*abcitypes.ExtendVoteResponse, error) {
    return &abcitypes.ExtendVoteResponse{}, nil
}

func (app *KVStoreApplication) VerifyVoteExtension(_ context.Context, verify *abcitypes.VerifyVoteExtensionRequest) (*abcitypes.VerifyVoteExtensionResponse, error) {
    return &abcitypes.VerifyVoteExtensionResponse{}, nil
}
```

The types used here are defined in the CometBFT library and were added as a dependency
to the project when you ran `go get`. If your IDE is not recognizing the types, go ahead and run the command again.

```bash
go get github.com/cometbft/cometbft@v1.0
```

Now go back to the `main.go` and modify the `main` function so it matches the following,
where an instance of the `KVStoreApplication` type is created.

```go
func main() {
    fmt.Println("Hello, CometBFT")

    _ = NewKVStoreApplication()
}
```

You can recompile and run the application now by running `go get` and `go build`, but it does
not do anything.
So let's revisit the code adding the logic needed to implement our minimal key/value store
and to start it along with the CometBFT Service.

### 1.3.1 Add a persistent data store

Our application will need to write its state out to persistent storage so that it
can stop and start without losing all of its data.

For this tutorial, we will use [BadgerDB](https://github.com/dgraph-io/badger), a
fast embedded key-value store.

First, add Badger as a dependency of your go module using the `go get` command:

`go get github.com/dgraph-io/badger/v3`

Next, let's update the application and its constructor to receive a handle to the database, as follows:

```go
type KVStoreApplication struct {
    db           *badger.DB
    onGoingBlock *badger.Txn
}

var _ abcitypes.Application = (*KVStoreApplication)(nil)

func NewKVStoreApplication(db *badger.DB) *KVStoreApplication {
    return &KVStoreApplication{db: db}
}
```

The `onGoingBlock` keeps track of the Badger transaction that will update the application's state when a block
is completed. Don't worry about it for now, we'll get to that later.

Next, update the `import` stanza at the top to include the Badger library:

```go
import(
    "context"
    abcitypes "github.com/cometbft/cometbft/abci/types"
    "github.com/dgraph-io/badger/v4"
)
```

Finally, update the `main.go` file to invoke the updated constructor:

```go
    _ = NewKVStoreApplication(nil)
```

### 1.3.2 CheckTx

When CometBFT receives a new transaction from a client, or from another full node,
CometBFT asks the application if the transaction is acceptable, using the `CheckTx` method.
Invalid transactions will not be shared with other nodes and will not become part of any blocks and, therefore, will not be executed by the application.

In our application, a transaction is a string with the form `key=value`, indicating a key and value to write to the store.

The most basic validation check we can perform is to check if the transaction conforms to the `key=value` pattern.
For that, let's add the following helper method to app.go:

```go
func (app *KVStoreApplication) isValid(tx []byte) uint32 {
    // check format
    parts := bytes.Split(tx, []byte("="))
    if len(parts) != 2 {
        return 1
    }

    return 0
}
```

Now you can rewrite the `CheckTx` method to use the helper function:

```go
func (app *KVStoreApplication) CheckTx(_ context.Context, check *abcitypes.CheckTxRequest) (*abcitypes.CheckTxResponse, error) {
    code := app.isValid(check.Tx)
    return &abcitypes.CheckTxResponse{Code: code}, nil
}
```

While this `CheckTx` is simple and only validates that the transaction is well-formed,
it is very common for `CheckTx` to make more complex use of the state of an application.
For example, you may refuse to overwrite an existing value, or you can associate
versions to the key/value pairs and allow the caller to specify a version to
perform a conditional update.

Depending on the checks and on the conditions violated, the function may return
different values, but any response with a non-zero code will be considered invalid
by CometBFT. Our `CheckTx` logic returns 0 to CometBFT when a transaction passes
its validation checks. The specific value of the code is meaningless to CometBFT.
Non-zero codes are logged by CometBFT so applications can provide more specific
information on why the transaction was rejected.

Note that `CheckTx` does not execute the transaction, it only verifies that the transaction could be executed. We do
not know yet if the rest of the network has agreed to accept this transaction into a block.

Finally, make sure to add the `bytes` package to the `import` stanza at the top of `app.go`:

```go
import(
    "bytes"
    "context"
    abcitypes "github.com/cometbft/cometbft/abci/types"
    "github.com/dgraph-io/badger/v4"
)
```

### 1.3.3 FinalizeBlock

When the CometBFT consensus engine has decided on the block, the block is transferred to the
application via `FinalizeBlock`.
`FinalizeBlock` is an ABCI method introduced in CometBFT `v0.38.0`. This replaces the functionality provided previously (pre-`v0.38.0`) by the combination of ABCI methods `BeginBlock`, `DeliverTx`, and `EndBlock`. `FinalizeBlock`'s parameters are an aggregation of those in `BeginBlock`, `DeliverTx`, and `EndBlock`.

This method is responsible for executing the block and returning a response to the consensus engine.
Providing a single `FinalizeBlock` method to signal the finalization of a block simplifies the ABCI interface and increases flexibility in the execution pipeline.

The `FinalizeBlock` method executes the block, including any necessary transaction processing and state updates, and returns a `FinalizeBlockResponse` object which contains any necessary information about the executed block.

**Note:** `FinalizeBlock` only prepares the update to be made and does not change the state of the application. The state change is actually committed in a later stage, in the `Commit` phase.

Note that, to implement these calls in our application we're going to make use of Badger's transaction mechanism. We will always refer to these as Badger transactions, not to confuse them with the transactions included in the blocks delivered by CometBFT, the _application transactions_.

First, let's create a new Badger transaction during `FinalizeBlock`. All application transactions in the current block will be executed within this Badger transaction.
Next, let's modify `FinalizeBlock` to add the `key` and `value` to the Badger transaction every time our application processes a new application transaction from the list received through `FinalizeBlockRequest`.

Note that we check the validity of the transaction _again_ during `FinalizeBlock`.

```go
func (app *KVStoreApplication) FinalizeBlock(_ context.Context, req *abcitypes.FinalizeBlockRequest) (*abcitypes.FinalizeBlockResponse, error) {
    var txs = make([]*abcitypes.ExecTxResult, len(req.Txs))

    app.onGoingBlock = app.db.NewTransaction(true)
    for i, tx := range req.Txs {
        if code := app.isValid(tx); code != 0 {
            log.Printf("Error: invalid transaction index %v", i)
            txs[i] = &abcitypes.ExecTxResult{Code: code}
        } else {
            parts := bytes.SplitN(tx, []byte("="), 2)
            key, value := parts[0], parts[1]
            log.Printf("Adding key %s with value %s", key, value)

            if err := app.onGoingBlock.Set(key, value); err != nil {
                log.Panicf("Error writing to database, unable to execute tx: %v", err)
            }

            log.Printf("Successfully added key %s with value %s", key, value)

            // Add an event for the transaction execution.
            // Multiple events can be emitted for a transaction, but we are adding only one event
            txs[i] = &abcitypes.ExecTxResult{
                Code: 0,
                Events: []abcitypes.Event{
                    {
                        Type: "app",
                        Attributes: []abcitypes.EventAttribute{
                            {Key: "key", Value: string(key), Index: true},
                            {Key: "value", Value: string(value), Index: true},
                        },
					},
				},
			}
        }
    }

    return &abcitypes.FinalizeBlockResponse{
      TxResults:        txs,
    }, nil
}
```

Transactions are not guaranteed to be valid when they are delivered to an application, even if they were valid when they were proposed.

This can happen if the application state is used to determine transaction validity.
The application state may have changed between the initial execution of `CheckTx` and the transaction delivery in `FinalizeBlock` in a way that rendered the transaction no longer valid.

**Note** that `FinalizeBlock` cannot yet commit the Badger transaction we were building during the block execution.

Other methods, such as `Query`, rely on a consistent view of the application's state, the application should only update its state by committing the Badger transactions when the full block has been delivered and the Commit method is invoked.

The `Commit` method tells the application to make permanent the effects of
the application transactions.
Let's update the method to terminate the pending Badger transaction and
persist the resulting state:

```go
func (app KVStoreApplication) Commit(_ context.Context, commit *abcitypes.CommitRequest) (*abcitypes.CommitResponse, error) {
    return &abcitypes.CommitResponse{}, app.onGoingBlock.Commit()
}
```

Finally, make sure to add the `log` and `errors` libraries to the `import` stanza as well:

```go
import (
    "context"
    "errors"
    abcitypes "github.com/cometbft/cometbft/abci/types"
    "github.com/dgraph-io/badger/v4"
    "log"
)
```

You may have noticed that the application we are writing will crash if it receives
an unexpected error from the Badger database during the `FinalizeBlock` or `Commit` methods.
This is not an accident. If the application received an error from the database, there
is no deterministic way for it to make progress so the only safe option is to terminate.
Once the application is restarted, the transactions in the block that failed execution will
be re-executed and should succeed if the Badger error was transient.

### 1.3.4 Query

When a client tries to read some information from the `kvstore`, the request will be
handled in the `Query` method. To do this, let's rewrite the `Query` method in `app.go`:

```go
func (app *KVStoreApplication) Query(_ context.Context, req *abcitypes.QueryRequest) (*abcitypes.QueryResponse, error) {
    resp := abcitypes.QueryResponse{Key: req.Data}

    dbErr := app.db.View(func(txn *badger.Txn) error {
        item, err := txn.Get(req.Data)
        if err != nil {
            if !errors.Is(err, badger.ErrKeyNotFound) {
                return err
            }
            resp.Log = "key does not exist"
            return nil
        }

        return item.Value(func(val []byte) error {
            resp.Log = "exists"
            resp.Value = val
            return nil
        })
    })
    if dbErr != nil {
        log.Panicf("Error reading database, unable to execute query: %v", dbErr)
    }
    return &resp, nil
}
```

Since it reads only committed data from the store, transactions that are part of a block
that is being processed are not reflected in the query result.

### 1.3.5 PrepareProposal and ProcessProposal

`PrepareProposal` and `ProcessProposal` are methods introduced in CometBFT v0.37.0
to give the application more control over the construction and processing of transaction blocks.

When CometBFT sees that valid transactions (validated through `CheckTx`) are available to be
included in blocks, it groups some of these transactions and then gives the application a chance
to modify the group by invoking `PrepareProposal`.

The application is free to modify the group before returning from the call, as long as the resulting set
does not use more bytes than `PrepareProposalRequest.max_tx_bytes`
For example, the application may reorder, add, or even remove transactions from the group to improve the
execution of the block once accepted.

In the following code, the application simply returns the unmodified group of transactions:

```go
func (app *KVStoreApplication) PrepareProposal(_ context.Context, proposal *abcitypes.PrepareProposalRequest) (*abcitypes.PrepareProposalResponse, error) {
    return &abcitypes.PrepareProposalResponse{Txs: proposal.Txs}, nil
}
```

Once a proposed block is received by a node, the proposal is passed to the application to give
its blessing before voting to accept the proposal.

This mechanism may be used for different reasons, for example to deal with blocks manipulated
by malicious nodes, in which case the block should not be considered valid.

The following code simply accepts all proposals:

```go
func (app *KVStoreApplication) ProcessProposal(_ context.Context, proposal *abcitypes.ProcessProposalRequest) (*abcitypes.ProcessProposalResponse, error) {
    return &abcitypes.ProcessProposalResponse{Status: abcitypes.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil
}
```

### 1.3.6 Handling errors

Please note that in the method signature for the ABCI methods, there is a response and an error return, such as
`(*abcitypes.[Method]Response, error)`. Some of the ABCI methods' responses might include a field that can return
an error in the response, such as the `Code` field in the `CheckTxResponse`. The application can use the `Code`
field to signal CometBFT that the transaction was rejected. Other examples are the `TxResults` in the
`FinalizeBlockResponse`, the `ExecTxResult` that also has a `Code` field which can be used by the application
to signal that the transactions didn't execute properly, or the `QueryResponse`. The `QueryResponse` also includes
a `Code` field to signal that a query to the application was unsuccessful or it could not find the information.

The `error` return, as in `(*abcitypes.[Method]Response, error)`, can be used if there are unrecoverable errors.
In these cases, the application should abort to prevent further unintended consequences.

## 1.4 Starting an application and a CometBFT instance in the same process

Now that we have the basic functionality of our application in place, let's put
it all together inside of our `main.go` file.

Change the contents of your `main.go` file to the following.

```go
package main

import (
    "context"
    "flag"
    "fmt"
    "github.com/cometbft/cometbft/p2p"
    "github.com/cometbft/cometbft/privval"
    "github.com/cometbft/cometbft/proxy"
    "log"
    "os"
    "os/signal"
    "path/filepath"
    "syscall"

    "github.com/dgraph-io/badger/v4"
    "github.com/spf13/viper"
    cfg "github.com/cometbft/cometbft/config"
    cmtflags "github.com/cometbft/cometbft/libs/cli/flags"
    cmtlog "github.com/cometbft/cometbft/libs/log"
    nm "github.com/cometbft/cometbft/node"
)

var homeDir string

func init() {
    flag.StringVar(&homeDir, "cmt-home", "", "Path to the CometBFT config directory (if empty, uses $HOME/.cometbft)")
}

func main() {
    flag.Parse()
    if homeDir == "" {
        homeDir = os.ExpandEnv("$HOME/.cometbft")
    }

    config := cfg.DefaultConfig()
    config.SetRoot(homeDir)
    viper.SetConfigFile(fmt.Sprintf("%s/%s", homeDir, "config/config.toml"))

    if err := viper.ReadInConfig(); err != nil {
        log.Fatalf("Reading config: %v", err)
    }
    if err := viper.Unmarshal(config); err != nil {
        log.Fatalf("Decoding config: %v", err)
    }
    if err := config.ValidateBasic(); err != nil {
        log.Fatalf("Invalid configuration data: %v", err)
    }
    dbPath := filepath.Join(homeDir, "badger")
    db, err := badger.Open(badger.DefaultOptions(dbPath))

    if err != nil {
        log.Fatalf("Opening database: %v", err)
    }
    defer func() {
        if err := db.Close(); err != nil {
            log.Printf("Closing database: %v", err)
        }
    }()

    app := NewKVStoreApplication(db)

    pv := privval.LoadFilePV(
        config.PrivValidatorKeyFile(),
        config.PrivValidatorStateFile(),
    )

    nodeKey, err := p2p.LoadNodeKey(config.NodeKeyFile())
    if err != nil {
        log.Fatalf("failed to load node's key: %v", err)
    }

    logger := cmtlog.NewTMLogger(cmtlog.NewSyncWriter(os.Stdout))
    logger, err = cmtflags.ParseLogLevel(config.LogLevel, logger, cfg.DefaultLogLevel)

    if err != nil {
        log.Fatalf("failed to parse log level: %v", err)
    }

    node, err := nm.NewNode(
        context.Background(),
        config,
        pv,
        nodeKey,
        proxy.NewLocalClientCreator(app),
        nm.DefaultGenesisDocProviderFunc(config),
        cfg.DefaultDBProvider,
        nm.DefaultMetricsProvider(config.Instrumentation),
        logger,
    )

    if err != nil {
        log.Fatalf("Creating node: %v", err)
    }

    node.Start()
    defer func() {
        node.Stop()
        node.Wait()
    }()

    c := make(chan os.Signal, 1)
    signal.Notify(c, os.Interrupt, syscall.SIGTERM)
    <-c
}
```

This is a huge blob of code, so let's break it down into pieces.

First, we use [viper](https://github.com/spf13/viper) to load the CometBFT configuration files, which we will generate later:

```go
config := cfg.DefaultValidatorConfig()

config.SetRoot(homeDir)

viper.SetConfigFile(fmt.Sprintf("%s/%s", homeDir, "config/config.toml"))
if err := viper.ReadInConfig(); err != nil {
    log.Fatalf("Reading config: %v", err)
}
if err := viper.Unmarshal(config); err != nil {
    log.Fatalf("Decoding config: %v", err)
}
if err := config.ValidateBasic(); err != nil {
    log.Fatalf("Invalid configuration data: %v", err)
}
```

Next, we initialize the Badger database and create an app instance.

```go
dbPath := filepath.Join(homeDir, "badger")
db, err := badger.Open(badger.DefaultOptions(dbPath))
if err != nil {
    log.Fatalf("Opening database: %v", err)
}
defer func() {
    if err := db.Close(); err != nil {
        log.Fatalf("Closing database: %v", err)
    }
}()

app := NewKVStoreApplication(db)
```

We use `FilePV`, which is a private validator (i.e. thing which signs consensus
messages). Normally, you would use `SignerRemote` to connect to an external
[HSM](https://kb.certus.one/hsm.html).

```go
pv := privval.LoadFilePV(
    config.PrivValidatorKeyFile(),
    config.PrivValidatorStateFile(),
)
```

`nodeKey` is needed to identify the node in a p2p network.

```go
nodeKey, err := p2p.LoadNodeKey(config.NodeKeyFile())
if err != nil {
    return nil, fmt.Errorf("failed to load node's key: %w", err)
}
```

Now we have everything set up to run the CometBFT node. We construct
a node by passing it the configuration, the logger, a handle to our application and
the genesis information:

```go
node, err := nm.NewNode(
    context.Background(),
    config,
    pv,
    nodeKey,
    proxy.NewLocalClientCreator(app),
    nm.DefaultGenesisDocProviderFunc(config),
    cfg.DefaultDBProvider,
    nm.DefaultMetricsProvider(config.Instrumentation),
    logger)

if err != nil {
    log.Fatalf("Creating node: %v", err)
}
```

Finally, we start the node, i.e., the CometBFT service inside our application:

```go
node.Start()
defer func() {
    node.Stop()
    node.Wait()
}()
```

The additional logic at the end of the file allows the program to catch SIGTERM. This means that the node can shut down gracefully when an operator tries to kill the program:

```go
c := make(chan os.Signal, 1)
signal.Notify(c, os.Interrupt, syscall.SIGTERM)
<-c
```

Try again to build the code to ensure it is all good so far:

```bash
go build
```

If there are no errors, everything worked well so far. But if you get some errors about `missing go.sum entry for module ...`,
you can try to fix them by running the commands below:

```bash
go mod tidy
go build
```

## 1.5 Initializing and Running

Our application is almost ready to run, but first we'll need to populate the CometBFT configuration files.
The following command will create a `cometbft-home` directory in your project and add a basic set of configuration files in `cometbft-home/config/`.
For more information on what these files contain see [the configuration documentation](https://docs.cometbft.com/v1.0/explanation/core/configuration).

From the root of your project, run:

```bash
cometbft init --home /tmp/cometbft-home
```

You should see an output similar to the following:

```bash
I[2023-25-04|09:06:34.444] Generated private validator                  module=main keyFile=/tmp/cometbft-home/config/priv_validator_key.json stateFile=/tmp/cometbft-home/data/priv_validator_state.json
I[2023-25-04|09:06:34.444] Generated node key                           module=main path=/tmp/cometbft-home/config/node_key.json
I[2023-25-04|09:06:34.444] Generated genesis file                       module=main path=/tmp/cometbft-home/config/genesis.json
```

Now rebuild the app:

```bash
go build -mod=mod # use -mod=mod to automatically refresh the dependencies
```

Everything is now in place to run your application. Run:

```bash
./kvstore -cmt-home /tmp/cometbft-home
```

The application will start and you should see a continuous output starting with:

```bash
badger 2023-04-25 09:08:50 INFO: All 0 tables opened in 0s
badger 2023-04-25 09:08:50 INFO: Discard stats nextEmptySlot: 0
badger 2023-04-25 09:08:50 INFO: Set nextTxnTs to 0
I[2023-04-25|09:08:50.085] service start                                module=proxy msg="Starting multiAppConn service" impl=multiAppConn
I[2023-04-25|09:08:50.085] service start                                module=abci-client connection=query msg="Starting localClient service" impl=localClient
I[2023-04-25|09:08:50.085] service start                                module=abci-client connection=snapshot msg="Starting localClient service" impl=localClient
...
```

More importantly, the application using CometBFT is producing blocks 🎉🎉 and you can see this reflected in the log output in lines like this:

```bash
I[2023-04-25|09:08:52.147] received proposal                            module=consensus proposal="Proposal{2/0 (F518444C0E348270436A73FD0F0B9DFEA758286BEB29482F1E3BEA75330E825C:1:C73D3D1273F2, -1) AD19AE292A45 @ 2023-04-25T12:08:52.143393Z}"
I[2023-04-25|09:08:52.152] received complete proposal block             module=consensus height=2 hash=F518444C0E348270436A73FD0F0B9DFEA758286BEB29482F1E3BEA75330E825C
I[2023-04-25|09:08:52.160] finalizing commit of block                   module=consensus height=2 hash=F518444C0E348270436A73FD0F0B9DFEA758286BEB29482F1E3BEA75330E825C root= num_txs=0
I[2023-04-25|09:08:52.167] executed block                               module=state height=2 num_valid_txs=0 num_invalid_txs=0
I[2023-04-25|09:08:52.171] committed state                              module=state height=2 num_txs=0 app_hash=
```

The blocks, as you can see from the `num_valid_txs=0` part, are empty, but let's remedy that next.

## 1.6 Using the application

Let's try submitting a transaction to our new application.
Open another terminal window and run the following curl command:

```bash
curl -s 'localhost:26657/broadcast_tx_commit?tx="cometbft=rocks"'
```

If everything went well, you should see a response indicating which height the
transaction was included in the blockchain.

```bash
{"jsonrpc":"2.0","id":-1,"result":{"check_tx":{"code":0,"data":null,"log":"","info":"","gas_wanted":"0","gas_used":"0","events":[],"codespace":""},"tx_result":{"code":0,"data":null,"log":"","info":"","gas_wanted":"0","gas_used":"0","events":[{"type":"app","attributes":[{"key":"key","value":"cometbft","index":true},{"key":"value","value":"rocks","index":true}]}],"codespace":""},"hash":"71276C4844CE72F6C6C868541D10923259F5F8DA5716B230555B36AD309D6FD1","height":"64"}}
```

Finally, let's make sure that transaction really was persisted by the application.
Run the following command:

```bash
curl -s 'localhost:26657/abci_query?data="cometbft"'
```

Let's examine the response object that this request returns.
The request returns a `json` object with a `key` and `value` field set.

```json
...
    "key": "Y29tZXRiZnQ=",
    "value": "cm9ja3M=",
...
```

Those values don't look like the `key` and `value` we sent to CometBFT.
What's going on here?

The response contains a `base64` encoded representation of the data we submitted.
To get the original value out of this data, we can use the `base64` command line utility:

```bash
$ echo "Y29tZXRiZnQ=" | base64 -d
cometbft
$ echo "cm9ja3M=" | base64 -d
rocks
```

If you want to search for txs, you can leverage `CometBFT` kv indexer by using the `/tx_search` RPC endpoint:

```bash
curl "localhost:26657/tx_search?query=\"app.key='cometbft'\""
```

The events (`abcitypes.Event`) added in `FinalizeBlock` are indexed by CometBFT (assuming the `kv` indexer is enabled in the CometBFT's configuration).


## Outro

Hope you could run everything smoothly. If you have any difficulties running through this tutorial, reach out to us via [discord](https://discord.com/invite/interchain) or open a new [issue](https://github.com/cometbft/cometbft/issues/new/choose) on Github.
