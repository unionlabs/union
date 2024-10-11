# IBC Testing Package

## Components

The testing package comprises of four parts constructed as a stack.

- coordinator
- chain
- path
- endpoint

A coordinator sits at the highest level and contains all the chains which have been initialized.
It also stores and updates the current global time. The time is manually incremented by a `TimeIncrement`.
This allows all the chains to remain in synchrony avoiding the issue of a counterparty being perceived to
be in the future. The coordinator also contains functions to do basic setup of clients, connections, and channels
between two chains.

A chain is an SDK application (as represented by an app.go file). Inside the chain is an `TestingApp` which allows
the chain to simulate block production and transaction processing. The chain contains by default a single tendermint
validator. A chain is used to process SDK messages.

A path connects two channel endpoints. It contains all the information needed to relay between two endpoints.

An endpoint represents a channel (and its associated client and connections) on some specific chain. It contains
references to the chain it is on and the counterparty endpoint it is connected to. The endpoint contains functions
to interact with initialization and updates of its associated clients, connections, and channels. It can send, receive,
and acknowledge packets.

In general:

- endpoints are used for initialization and execution of IBC logic on one side of an IBC connection
- paths are used to relay packets
- chains are used to commit SDK messages
- coordinator is used to setup a path between two chains

## Integration

To integrate the testing package into your tests, you will need to define:

- a testing application
- a function to initialize the testing application

### TestingApp

Your project will likely already have an application defined. This application
will need to be extended to fulfill the `TestingApp` interface.

```go
type TestingApp interface {
  abci.Application

  // ibc-go additions
  GetBaseApp() *baseapp.BaseApp
  GetStakingKeeper() ibctestingtypes.StakingKeeper
  GetIBCKeeper() *keeper.Keeper
  GetScopedIBCKeeper() capabilitykeeper.ScopedKeeper
  GetTxConfig() client.TxConfig

  // Implemented by SimApp
  AppCodec() codec.Codec

  // Implemented by BaseApp
  LastCommitID() sdk.CommitID
  LastBlockHeight() int64
}
```

To begin, you will need to extend your application by adding the following functions:

```go
// TestingApp functions
// Example using SimApp to implement TestingApp

// GetBaseApp implements the TestingApp interface.
func (app *SimApp) GetBaseApp() *baseapp.BaseApp {
  return app.BaseApp
}

// GetStakingKeeper implements the TestingApp interface.
func (app *SimApp) GetStakingKeeper() ibctestingtypes.Keeper {
  return app.StakingKeeper
}

// GetIBCKeeper implements the TestingApp interface.
func (app *SimApp) GetIBCKeeper() *ibckeeper.Keeper {
  return app.IBCKeeper
}

// GetScopedIBCKeeper implements the TestingApp interface.
func (app *SimApp) GetScopedIBCKeeper() capabilitykeeper.ScopedKeeper {
  return app.ScopedIBCKeeper
}

// GetTxConfig implements the TestingApp interface.
func (app *SimApp) GetTxConfig() client.TxConfig {
  return MakeTestEncodingConfig().TxConfig
}

```

Your application may need to define `AppCodec()` if it does not already exist:

```go
// AppCodec returns SimApp's app codec.
//
// NOTE: This is solely to be used for testing purposes as it may be desirable
// for modules to register their own custom testing types.
func (app *SimApp) AppCodec() codec.Codec {
  return app.appCodec
}
```

It is assumed your application contains an embedded BaseApp and thus implements the abci.Application interface, `LastCommitID()` and `LastBlockHeight()`

### Initialize TestingApp

The testing package requires that you provide a function to initialize your TestingApp. This is how ibc-go implements the initialize function with its `SimApp`:

```go
func SetupTestingApp() (TestingApp, map[string]json.RawMessage) {
  db := dbm.NewMemDB()
  encCdc := simapp.MakeTestEncodingConfig()
  app := simapp.NewSimApp(
    log.NewNopLogger(), 
    db, 
    nil, 
    true,
    map[int64]bool{},
    simapp.DefaultNodeHome,
    5,
    encCdc, 
    simapp.EmptyAppOptions{},
  )
  return app, simapp.NewDefaultGenesisState(encCdc.Marshaler)
}
```

This function returns the TestingApp and the default genesis state used to initialize the testing app.

Change the value of `DefaultTestingAppInit` to use your function:

```go
func init() {
  ibctesting.DefaultTestingAppInit = SetupTestingApp
}
```

## Example

Here is an example of how to setup your testing environment in every package you are testing:

```go
// KeeperTestSuite is a testing suite to test keeper functions.
type KeeperTestSuite struct {
  testifysuite.Suite

  coordinator *ibctesting.Coordinator

  // testing chains used for convenience and readability
  chainA *ibctesting.TestChain
  chainB *ibctesting.TestChain
}

// TestKeeperTestSuite runs all the tests within this package.
func TestKeeperTestSuite(t *testing.T) {
  testifysuite.Run(t, new(KeeperTestSuite))
}

// SetupTest creates a coordinator with 2 test chains.
func (suite *KeeperTestSuite) SetupTest() {
  suite.coordinator = ibctesting.NewCoordinator(suite.T(), 2) // initializes 2 test chains
  suite.chainA = suite.coordinator.GetChain(ibctesting.GetChainID(1)) // convenience and readability
  suite.chainB = suite.coordinator.GetChain(ibctesting.GetChainID(2)) // convenience and readability
}

```

To create interaction between chainA and chainB, we need to contruct a `Path` these chains will use.
A path contains two endpoints, `EndpointA` and `EndpointB` (corresponding to the order of the chains passed
into the `NewPath` function). A path is a pointer and its values will be filled in as necessary during the
setup portion of testing.

Endpoint Struct:

```go
// Endpoint is a which represents a channel endpoint and its associated
// client and connections. It contains client, connection, and channel
// configuration parameters. Endpoint functions will utilize the parameters
// set in the configuration structs when executing IBC messages.
type Endpoint struct {
  Chain        *TestChain
  Counterparty *Endpoint
  ClientID     string
  ConnectionID string
  ChannelID    string

  ClientConfig     ClientConfig
  ConnectionConfig *ConnectionConfig
  ChannelConfig    *ChannelConfig
}
```

The fields empty after `NewPath` is called are `ClientID`, `ConnectionID` and
`ChannelID` as the clients, connections, and channels for these endpoints have not yet been created. The
`ClientConfig`, `ConnectionConfig` and `ChannelConfig` contain all the necessary information for clients,
connections, and channels to be initialized. If you would like to use endpoints which are intitialized to
use your Port IDs, you might add a helper function similar to the one found in transfer:

```go
func NewTransferPath(chainA, chainB *ibctesting.TestChain) *ibctesting.Path {
  path := ibctesting.NewPath(chainA, chainB)
  path.EndpointA.ChannelConfig.PortID = ibctesting.TransferPort
  path.EndpointB.ChannelConfig.PortID = ibctesting.TransferPort

  return path
}

```

Path configurations should be set to the desired values before calling any `Setup` coordinator functions.

To initialize the clients, connections, and channels for a path we can call the Setup functions of the coordinator:

- Setup() -> setup clients, connections, channels
- SetupClients() -> setup clients only
- SetupConnections() -> setup clients and connections only

Here is a basic example of the testing package being used to simulate IBC functionality:

```go
  path := ibctesting.NewPath(suite.chainA, suite.chainB) // clientID, connectionID, channelID empty
  suite.coordinator.Setup(path) // clientID, connectionID, channelID filled
  suite.Require().Equal("07-tendermint-0", path.EndpointA.ClientID)
  suite.Require().Equal("connection-0", path.EndpointA.ClientID)
  suite.Require().Equal("channel-0", path.EndpointA.ClientID)

  // send on endpointA
  sequence, err := path.EndpointA.SendPacket(timeoutHeight1, timeoutTimestamp1, packet1Data)

  // create packet 1 
  packet1 := NewPacket() // NewPacket would construct your packet

  // receive on endpointB
  path.EndpointB.RecvPacket(packet1)

  // acknowledge the receipt of the packet
  path.EndpointA.AcknowledgePacket(packet1, ack)

  // we can also relay
  sequence, err := path.EndpointA.SendPacket(timeoutHeight2, timeoutTimestamp2, packet2Data)

  packet2 := NewPacket()

  path.RelayPacket(packet2)

  // if needed we can update our clients
  path.EndpointB.UpdateClient()    
```

### Transfer Testing Example

If ICS 20 had its own simapp, its testing setup might include a `testing/app.go` file with the following contents:

```go
package transfertesting

import (
  "encoding/json"

  "github.com/cometbft/cometbft/libs/log"
  dbm "github.com/cometbft/cometbft-db"

  "github.com/cosmos/ibc-go/v8/modules/apps/transfer/simapp"
  ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

func SetupTransferTestingApp() (ibctesting.TestingApp, map[string]json.RawMessage) {
  db := dbm.NewMemDB()
  encCdc := simapp.MakeTestEncodingConfig()
  app := simapp.NewSimApp(
    log.NewNopLogger(),
    db,
    nil,
    true,
    map[int64]bool{},
    simapp.DefaultNodeHome,
    5,
    encCdc,
    simapp.EmptyAppOptions{},
  )
  return app, simapp.NewDefaultGenesisState(encCdc.Marshaler)
}

func init() {
  ibctesting.DefaultTestingAppInit = SetupTransferTestingApp
}

func NewTransferPath(chainA, chainB *ibctesting.TestChain) *ibctesting.Path {
  path := ibctesting.NewPath(chainA, chainB)
  path.EndpointA.ChannelConfig.PortID = ibctesting.TransferPort
  path.EndpointB.ChannelConfig.PortID = ibctesting.TransferPort

  return path
}

func GetTransferSimApp(chain *ibctesting.TestChain) *simapp.SimApp {
  app, ok := chain.App.(*simapp.SimApp)
  if !ok {
  panic("not transfer app")
  }

  return app
}
```

### Middleware Testing

When writing IBC applications acting as middleware, it might be desirable to test integration points.
This can be done by wiring a middleware stack in the app.go file using existing applications as middleware and IBC base applications.
The mock module may also be leveraged to act as a base application in the instance that such an application is not available for testing or causes dependency concerns.

The mock IBC module contains a `MockIBCApp`. This struct contains a function field for every IBC App Module callback.
Each of these functions can be individually set to mock expected behaviour of a base application.
The portID and scoped keeper for the `MockIBCApp` should be set within `MockIBCApp` before calling `NewIBCModule`.

For example, if one wanted to test that the base application cannot affect the outcome of the `OnChanOpenTry` callback, the mock module base application callback could be updated as such:

```go
mockModule.IBCApp.OnChanOpenTry = func(ctx sdk.Context, portID, channelID, version string) error {
  return fmt.Errorf("mock base app must not be called for OnChanOpenTry")
}
```

Using a mock module as a base application in a middleware stack may require adding the module to your `SimApp`.
This is because IBC will route to the top level IBC module of a middleware stack, so a module which never
sits at the top of middleware stack will need to be accessed via a public field in `SimApp`

This might look like:

```go
suite.chainA.GetSimApp().ICAAuthModule.IBCApp.OnChanOpenInit = func(
  ctx sdk.Context, order channeltypes.Order, connectionHops []string,
  portID, channelID string, chanCap *capabilitytypes.Capability,
  counterparty channeltypes.Counterparty, version string,
) error {
  return fmt.Errorf("mock ica auth fails")
}
```
