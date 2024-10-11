# RFC 004: Accounts

## Changelog

* 17/03/2023: DRAFT
* 09/05/2023: DRAFT 2

## Context

The current implementation of accounts in the Cosmos SDK is limiting in terms of functionality, extensibility, and overall
architecture. This RFC aims to address the following issues with the current account system:

### 1. Accounts Representation and Authentication Mechanism

The current SDK accounts are represented as `google.Protobuf.Any`, which are then encapsulated into the account interface.
This account interface essentially represents the authentication mechanism, as it implements methods such as `GetNumber`
and `GetSequence` that serve as abstractions over the authentication system. However, this approach restricts the scope and
functionality of accounts within the SDK.

### 2. Limited Account Interface

The account interface in its current form is not versatile enough to accommodate more advanced account functionalities,
such as implementing vesting capabilities or more complex authentication and authorization systems.

### 3. Multiple Implementations of the Account Interface

There are several implementations of the account interface, like `ModuleAccount`, but the existing abstraction does not
allow for meaningful differentiation between them. This hinders the ability to create specialized accounts that cater to
specific use cases.

### 4. Primitive Authorization System

The authorization system in the `x/auth` module is basic and defines authorizations solely for the functionalities of the
`x/bank` module. Consequently, although the state transition authorization system is defined in `x/auth`, it only covers the
use cases of `x/bank`, limiting the system's overall scope and adaptability.

### 5. Cyclic Dependencies and Abstraction Leaks

The current account system leads to cyclic dependencies and abstraction leaks throughout the Cosmos SDK. For instance,
the `Vesting` functionality belongs to the `x/auth` module, which depends on the `x/bank` module. However,
the `x/bank` module depends on the `x/auth` module again to identify the account type (either `Vesting` or `Base`) during
a coin transfer. This dependency structure creates architectural issues and complicates the overall design of the SDK.

## Proposal

This proposal aims to transform the way accounts are managed within the Cosmos SDK by introducing significant changes to
their structure and functionality.

### Rethinking Account Representation and Business Logic

Instead of representing accounts as simple `google.Protobuf.Any` structures stored in state with no business logic
attached, this proposal suggests a more sophisticated account representation that is closer to module entities.
In fact, accounts should be able to receive messages and process them in the same way modules do, and be capable of storing
state in a isolated (prefixed) portion of state belonging only to them, in the same way as modules do.

### Account Message Reception

We propose that accounts should be able to receive messages in the same way modules can, allowing them to manage their
own state modifications without relying on other modules. This change would enable more advanced account functionality, such as the
`VestingAccount` example, where the x/bank module previously needed to change the vestingState by casting the abstracted 
account to `VestingAccount` and triggering the `TrackDelegation` call. Accounts are already capable of sending messages when
a state transition, originating from a transaction, is executed.

When accounts receive messages, they will be able to identify the sender of the message and decide how to process the
state transition, if at all.

### Consequences

These changes would have significant implications for the Cosmos SDK, resulting in a system of actors that are equal from
the runtime perspective. The runtime would only be responsible for propagating messages between actors and would not
manage the authorization system. Instead, actors would manage their own authorizations. For instance, there would be no
need for the `x/auth` module to manage minting or burning of coins permissions, as it would fall within the scope of the
`x/bank` module.

The key difference between accounts and modules would lie in the origin of the message (state transition). Accounts
(ExternallyOwnedAccount), which have credentials (e.g., a public/private key pairing), originate state transitions from
transactions. In contrast, module state transitions do not have authentication credentials backing them and can be 
caused by two factors: either as a consequence of a state transition coming from a transaction or triggered by a scheduler
(e.g., the runtime's Begin/EndBlock).

By implementing these proposed changes, the Cosmos SDK will benefit from a more extensible, versatile, and efficient account
management system that is better suited to address the requirements of the Cosmos ecosystem.

#### Standardization

With `x/accounts` allowing a modular api there becomes a need for standardization of accounts or the interfaces wallets and other clients should expect to use. For this reason we will be using the [`CIP` repo](https://github.com/cosmos/cips) in order to standardize interfaces in order for wallets to know what to expect when interacting with accounts.

## Implementation

### Account Definition

We define the new `Account` type, which is what an account needs to implement to be treated as such.
An `Account` type is defined at APP level, so it cannot be dynamically loaded as the chain is running without upgrading the
node code, unless we create something like a `CosmWasmAccount` which is an account backed by an `x/wasm` contract.

```go
// Account is what the developer implements to define an account.
type Account[InitMsg proto.Message] interface {
	// Init is the function that initialises an account instance of a given kind.
	// InitMsg is used to initialise the initial state of an account.
	Init(ctx *Context, msg InitMsg) error
	// RegisterExecuteHandlers registers an account's execution messages.
	RegisterExecuteHandlers(executeRouter *ExecuteRouter)
	// RegisterQueryHandlers registers an account's query messages.
	RegisterQueryHandlers(queryRouter *QueryRouter)
	// RegisterMigrationHandlers registers an account's migration messages.
	RegisterMigrationHandlers(migrationRouter *MigrationRouter)
}
```

### The InternalAccount definition

The public `Account` interface implementation is then converted by the runtime into an `InternalAccount` implementation,
which contains all the information and business logic needed to operate the account.

```go
type Schema struct {
	state StateSchema // represents the state of an account
	init InitSchema // represents the init msg schema
	exec ExecSchema // represents the multiple execution msg schemas, containing also responses
	query QuerySchema // represents the multiple query msg schemas, containing also responses
	migrate *MigrateSchema // represents the multiple migrate msg schemas, containing also responses, it's optional
}

type InternalAccount struct {
	init    func(ctx *Context, msg proto.Message) (*InitResponse, error)
	execute func(ctx *Context, msg proto.Message) (*ExecuteResponse, error)
	query   func(ctx *Context, msg proto.Message) (proto.Message, error)
    schema  func() *Schema
    migrate func(ctx *Context, msg proto.Message) (*MigrateResponse, error)
}
```

This is an internal view of the account as intended by the system. It is not meant to be what developers implement. An
example implementation of the `InternalAccount` type can be found in [this](https://github.com/testinginprod/accounts-poc/blob/main/examples/recover/recover.go)
example of account whose credentials can be recovered. In fact, even if the `Internal` implementation is untyped (with
respect to `proto.Message`), the concrete implementation is fully typed.

During any of the execution methods of `InternalAccount`, `schema` excluded, the account is given a `Context` which provides:

* A namespaced `KVStore` for the account, which isolates the account state from others (NOTE: no `store keys` needed,
  the account address serves as `store key`).
* Information regarding itself (its address)
* Information regarding the sender.
* ...

#### Init

Init defines the entrypoint that allows for a new account instance of a given kind to be initialised.
The account is passed some opaque protobuf message which is then interpreted and contains the instructions that
constitute the initial state of an account once it is deployed.

An `Account` code can be deployed multiple times through the `Init` function, similar to how a `CosmWasm` contract code
can be deployed (Instantiated) multiple times.

#### Execute

Execute defines the entrypoint that allows an `Account` to process a state transition, the account can decide then how to
process the state transition based on the message provided and the sender of the transition.

#### Query

Query defines a read-only entrypoint that provides a stable interface that links an account with its state. The reason for
which `Query` is still being preferred as an addition to raw state reflection is to:

* Provide a stable interface for querying (state can be optimised and change more frequently than a query)
* Provide a way to define an account `Interface` with respect to its `Read/Write` paths.
* Provide a way to query information that cannot be processed from raw state reflection, ex: compute information from lazy
  state that has not been yet concretely processed (eg: balances with respect to lazy inputs/outputs)

#### Schema

Schema provides the definition of an account from `API` perspective, and it's the only thing that should be taken into account
when interacting with an account from another account or module, for example: an account is an `authz-interface` account if
it has the following message in its execution messages `MsgProxyStateTransition{ state_transition: google.Protobuf.Any }`.

### Migrate

Migrate defines the entrypoint that allows an `Account` to migrate its state from a previous version to a new one. Migrations
can be initiated only by the account itself, concretely this means that the migrate action sender can only be the account address
itself, if the account wants to allow another address to migrate it on its behalf then it could create an execution message
that makes the account migrate itself.

### x/accounts module

In order to create accounts we define a new module `x/accounts`, note that `x/accounts` deploys account with no authentication
credentials attached to it which means no action of an account can be incepted from a TX, we will later explore how the
`x/authn` module uses `x/accounts` to deploy authenticated accounts.

This also has another important implication for which account addresses are now fully decoupled from the authentication mechanism
which makes in turn off-chain operations a little more complex, as the chain becomes the real link between account identifier
and credentials.

We could also introduce a way to deterministically compute the account address.

Note, from the transaction point of view, the `init_message` and `execute_message` are opaque `google.Protobuf.Any`.

The module protobuf definition for `x/accounts` are the following:

```protobuf
// Msg defines the Msg service.
service Msg {
  rpc Deploy(MsgDeploy) returns (MsgDeployResponse);
  rpc Execute(MsgExecute) returns (MsgExecuteResponse);
  rpc Migrate(MsgMigrate) returns (MsgMigrateResponse);
}

message MsgDeploy {
  string sender = 1;
  string kind = 2;
  google.Protobuf.Any init_message = 3;
  repeated google.Protobuf.Any authorize_messages = 4 [(gogoproto.nullable) = false];
}

message MsgDeployResponse {
  string address = 1;
  uint64 id = 2;
  google.Protobuf.Any data = 3;
}

message MsgExecute {
  string sender = 1;
  string address = 2;
  google.Protobuf.Any message = 3;
  repeated google.Protobuf.Any authorize_messages = 4 [(gogoproto.nullable) = false];
}

message MsgExecuteResponse {
  google.Protobuf.Any data = 1;
}

message MsgMigrate {
  string sender = 1;
  string new_account_kind = 2;
  google.Protobuf.Any migrate_message = 3;
}

message MsgMigrateResponse {
  google.Protobuf.Any data = 1;
}

```

#### MsgDeploy

Deploys a new instance of the given account `kind` with initial settings represented by the `init_message` which is a `google.Protobuf.Any`.
Of course the `init_message` can be empty. A response is returned containing the account ID and humanised address, alongside some response
that the account instantiation might produce.

#### Address derivation

In order to decouple public keys from account addresses, we introduce a new address derivation mechanism which is


#### MsgExecute

Sends a `StateTransition` execution request, where the state transition is represented by the `message` which is a `google.Protobuf.Any`.
The account can then decide if to process it or not based on the `sender`.

### MsgMigrate

Migrates an account to a new version of itself, the new version is represented by the `new_account_kind`. The state transition
can only be incepted by the account itself, which means that the `sender` must be the account address itself. During the migration
the account current state is given to the new version of the account, which then executes the migration logic using the `migrate_message`,
it might change state or not, it's up to the account to decide. The response contains possible data that the account might produce
after the migration.

#### Authorize Messages

The `Deploy` and `Execute` messages have a field in common called `authorize_messages`, these messages are messages that the account
can execute on behalf of the sender. For example, in case an account is expecting some funds to be sent from the sender,
the sender can attach a `MsgSend` that the account can execute on the sender's behalf. These authorizations are short-lived,
they live only for the duration of the `Deploy` or `Execute` message execution, or until they are consumed.

An alternative would have been to add a `funds` field, like it happens in cosmwasm, which guarantees the called contract that
the funds are available and sent in the context of the message execution. This would have been a simpler approach, but it would
have been limited to the context of `MsgSend` only, where the asset is `sdk.Coins`. The proposed generic way, instead, allows
the account to execute any message on behalf of the sender, which is more flexible, it could include NFT send execution, or
more complex things like `MsgMultiSend` or `MsgDelegate`, etc.


### Further discussion

#### Sub-accounts

We could provide a way to link accounts to other accounts. Maybe during deployment the sender could decide to link the
newly created to its own account, although there might be use-cases for which the deployer is different from the account
that needs to be linked, in this case a handshake protocol on linking would need to be defined.

#### Predictable address creation

We need to provide a way to create an account with a predictable address, this might serve a lot of purposes, like accounts
wanting to generate an address that:

* nobody else can claim besides the account used to generate the new account
* is predictable

For example:

```protobuf

message MsgDeployPredictable {
  string sender = 1;
  uint32 nonce = 2; 
  ...
}
```

And then the address becomes `bechify(concat(sender, nonce))`

`x/accounts` would still use the monotonically increasing sequence as account number.

#### Joining Multiple Accounts

As developers are building new kinds of accounts, it becomes necessary to provide a default way to combine the
functionalities of different account types. This allows developers to avoid duplicating code and enables end-users to
create or migrate to accounts with multiple functionalities without requiring custom development.

To address this need, we propose the inclusion of a default account type called "MultiAccount". The MultiAccount type is
designed to merge the functionalities of other accounts by combining their execution, query, and migration APIs.
The account joining process would only fail in the case of API (intended as non-state Schema APIs) conflicts, ensuring
compatibility and consistency.

With the introduction of the MultiAccount type, users would have the option to either migrate their existing accounts to
a MultiAccount type or extend an existing MultiAccount with newer APIs. This flexibility empowers users to leverage
various account functionalities without compromising compatibility or resorting to manual code duplication.

The MultiAccount type serves as a standardized solution for combining different account functionalities within the
cosmos-sdk ecosystem. By adopting this approach, developers can streamline the development process and users can benefit
from a modular and extensible account system.
