# ADR 035: Rosetta API Support

## Authors

* Jonathan Gimeno (@jgimeno)
* David Grierson (@senormonito)
* Alessio Treglia (@alessio)
* Frojdy Dymylja (@fdymylja)

## Changelog

* 2021-05-12: the external library  [cosmos-rosetta-gateway](https://github.com/tendermint/cosmos-rosetta-gateway) has been moved within the Cosmos SDK.

## Context

[Rosetta API](https://www.rosetta-api.org/) is an open-source specification and set of tools developed by Coinbase to
standardise blockchain interactions.

Through the use of a standard API for integrating blockchain applications it will

* Be easier for a user to interact with a given blockchain
* Allow exchanges to integrate new blockchains quickly and easily
* Enable application developers to build cross-blockchain applications such as block explorers, wallets and dApps at
  considerably lower cost and effort.

## Decision

It is clear that adding Rosetta API support to the Cosmos SDK will bring value to all the developers and
Cosmos SDK based chains in the ecosystem. How it is implemented is key.

The driving principles of the proposed design are:

1. **Extensibility:** it must be as riskless and painless as possible for application developers to set-up network
   configurations to expose Rosetta API-compliant services.
2. **Long term support:** This proposal aims to provide support for all the supported Cosmos SDK release series.
3. **Cost-efficiency:** Backporting changes to Rosetta API specifications from `master` to the various stable
   branches of Cosmos SDK is a cost that needs to be reduced.

We will achieve these delivering on these principles by the following:

1. There will be a package `rosetta/lib`
   for the implementation of the core Rosetta API features, particularly:
   a. The types and interfaces (`Client`, `OfflineClient`...), this separates design from implementation detail.
   b. The `Server` functionality as this is independent of the Cosmos SDK version.
   c. The `Online/OfflineNetwork`, which is not exported, and implements the rosetta API using the `Client` interface to query the node, build tx and so on.
   d. The `errors` package to extend rosetta errors.
2. Due to differences between the Cosmos release series, each series will have its own specific implementation of `Client` interface.
3. There will be two options for starting an API service in applications:
   a. API shares the application process
   b. API-specific process.

## Architecture

### The External Repo

As section will describe the proposed external library, including the service implementation, plus the defined types and interfaces.

#### Server

`Server` is a simple `struct` that is started and listens to the port specified in the settings. This is meant to be used across all the Cosmos SDK versions that are actively supported.

The constructor follows:

`func NewServer(settings Settings) (Server, error)`

`Settings`, which are used to construct a new server, are the following:

```go
// Settings define the rosetta server settings
type Settings struct {
	// Network contains the information regarding the network
	Network *types.NetworkIdentifier
	// Client is the online API handler
	Client crgtypes.Client
	// Listen is the address the handler will listen at
	Listen string
	// Offline defines if the rosetta service should be exposed in offline mode
	Offline bool
	// Retries is the number of readiness checks that will be attempted when instantiating the handler
	// valid only for online API
	Retries int
	// RetryWait is the time that will be waited between retries
	RetryWait time.Duration
}
```

#### Types

Package types uses a mixture of rosetta types and custom defined type wrappers, that the client must parse and return while executing operations.

##### Interfaces

Every SDK version uses a different format to connect (rpc, gRPC, etc), query and build transactions, we have abstracted this in what is the `Client` interface.
The client uses rosetta types, whilst the `Online/OfflineNetwork` takes care of returning correctly parsed rosetta responses and errors.

Each Cosmos SDK release series will have their own `Client` implementations.
Developers can implement their own custom `Client`s as required.

```go
// Client defines the API the client implementation should provide.
type Client interface {
	// Needed if the client needs to perform some action before connecting.
	Bootstrap() error
	// Ready checks if the servicer constraints for queries are satisfied
	// for example the node might still not be ready, it's useful in process
	// when the rosetta instance might come up before the node itself
	// the servicer must return nil if the node is ready
	Ready() error

	// Data API

	// Balances fetches the balance of the given address
	// if height is not nil, then the balance will be displayed
	// at the provided height, otherwise last block balance will be returned
	Balances(ctx context.Context, addr string, height *int64) ([]*types.Amount, error)
	// BlockByHashAlt gets a block and its transaction at the provided height
	BlockByHash(ctx context.Context, hash string) (BlockResponse, error)
	// BlockByHeightAlt gets a block given its height, if height is nil then last block is returned
	BlockByHeight(ctx context.Context, height *int64) (BlockResponse, error)
	// BlockTransactionsByHash gets the block, parent block and transactions
	// given the block hash.
	BlockTransactionsByHash(ctx context.Context, hash string) (BlockTransactionsResponse, error)
	// BlockTransactionsByHash gets the block, parent block and transactions
	// given the block hash.
	BlockTransactionsByHeight(ctx context.Context, height *int64) (BlockTransactionsResponse, error)
	// GetTx gets a transaction given its hash
	GetTx(ctx context.Context, hash string) (*types.Transaction, error)
	// GetUnconfirmedTx gets an unconfirmed Tx given its hash
	// NOTE(fdymylja): NOT IMPLEMENTED YET!
	GetUnconfirmedTx(ctx context.Context, hash string) (*types.Transaction, error)
	// Mempool returns the list of the current non confirmed transactions
	Mempool(ctx context.Context) ([]*types.TransactionIdentifier, error)
	// Peers gets the peers currently connected to the node
	Peers(ctx context.Context) ([]*types.Peer, error)
	// Status returns the node status, such as sync data, version etc
	Status(ctx context.Context) (*types.SyncStatus, error)

	// Construction API

	// PostTx posts txBytes to the node and returns the transaction identifier plus metadata related
	// to the transaction itself.
	PostTx(txBytes []byte) (res *types.TransactionIdentifier, meta map[string]interface{}, err error)
	// ConstructionMetadataFromOptions
	ConstructionMetadataFromOptions(ctx context.Context, options map[string]interface{}) (meta map[string]interface{}, err error)
	OfflineClient
}

// OfflineClient defines the functionalities supported without having access to the node
type OfflineClient interface {
	NetworkInformationProvider
	// SignedTx returns the signed transaction given the tx bytes (msgs) plus the signatures
	SignedTx(ctx context.Context, txBytes []byte, sigs []*types.Signature) (signedTxBytes []byte, err error)
	// TxOperationsAndSignersAccountIdentifiers returns the operations related to a transaction and the account
	// identifiers if the transaction is signed
	TxOperationsAndSignersAccountIdentifiers(signed bool, hexBytes []byte) (ops []*types.Operation, signers []*types.AccountIdentifier, err error)
	// ConstructionPayload returns the construction payload given the request
	ConstructionPayload(ctx context.Context, req *types.ConstructionPayloadsRequest) (resp *types.ConstructionPayloadsResponse, err error)
	// PreprocessOperationsToOptions returns the options given the preprocess operations
	PreprocessOperationsToOptions(ctx context.Context, req *types.ConstructionPreprocessRequest) (options map[string]interface{}, err error)
	// AccountIdentifierFromPublicKey returns the account identifier given the public key
	AccountIdentifierFromPublicKey(pubKey *types.PublicKey) (*types.AccountIdentifier, error)
}
```

### 2. Cosmos SDK Implementation

The Cosmos SDK implementation, based on version, takes care of satisfying the `Client` interface.
In Stargate, Launchpad and 0.37, we have introduced the concept of rosetta.Msg, this message is not in the shared repository as the sdk.Msg type differs between Cosmos SDK versions.

The rosetta.Msg interface follows:

```go
// Msg represents a cosmos-sdk message that can be converted from and to a rosetta operation.
type Msg interface {
	sdk.Msg
	ToOperations(withStatus, hasError bool) []*types.Operation
	FromOperations(ops []*types.Operation) (sdk.Msg, error)
}
```

Hence developers who want to extend the rosetta set of supported operations just need to extend their module's sdk.Msgs with the `ToOperations` and `FromOperations` methods.

### 3. API service invocation

As stated at the start, application developers will have two methods for invocation of the Rosetta API service:

1. Shared process for both application and API
2. Standalone API service

#### Shared Process (Only Stargate)

Rosetta API service could run within the same execution process as the application. This would be enabled via app.toml settings, and if gRPC is not enabled the rosetta instance would be spinned in offline mode (tx building capabilities only).

#### Separate API service

Client application developers can write a new command to launch a Rosetta API server as a separate process too, using the rosetta command contained in the `/server/rosetta` package. Construction of the command depends on Cosmos SDK version. Examples can be found inside `simd` for stargate, and `contrib/rosetta/simapp` for other release series.

## Status

Proposed

## Consequences

### Positive

* Out-of-the-box Rosetta API support within Cosmos SDK.
* Blockchain interface standardisation

## References

* https://www.rosetta-api.org/
