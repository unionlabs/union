---
sidebar_position: 1
---

# Rosetta

The `rosetta` package implements Coinbase's [Rosetta API](https://www.rosetta-api.org). This document provides instructions on how to use the Rosetta API integration. For information about the motivation and design choices, refer to [ADR 035](https://docs.cosmos.network/main/architecture/adr-035-rosetta-api-support).

## Add Rosetta Command

The Rosetta API server is a stand-alone server that connects to a node of a chain developed with Cosmos SDK.

To enable Rosetta API support, it's required to add the `RosettaCommand` to your application's root command file (e.g. `simd/cmd/root.go`).

Import the `server` package:

```go
import "github.com/cosmos/cosmos-sdk/server"
```

Find the following line:

```go
initRootCmd(rootCmd, encodingConfig)
```

After that line, add the following:

```go
rootCmd.AddCommand(
  server.RosettaCommand(encodingConfig.InterfaceRegistry, encodingConfig.Codec)
)
```

The `RosettaCommand` function builds the `rosetta` root command and is defined in the `server` package within Cosmos SDK.

Since we’ve updated the Cosmos SDK to work with the Rosetta API, updating the application's root command file is all you need to do.

An implementation example can be found in `simapp` package.

## Use Rosetta Command

To run Rosetta in your application CLI, use the following command:

```shell
simd rosetta --help
```

To test and run Rosetta API endpoints for applications that are running and exposed, use the following command:

```shell
simd rosetta
     --blockchain "your application name (ex: gaia)"
     --network "your chain identifier (ex: testnet-1)"
     --tendermint "tendermint endpoint (ex: localhost:26657)"
     --grpc "gRPC endpoint (ex: localhost:9090)"
     --addr "rosetta binding address (ex: :8080)"
```

## Extensions

There are two ways in which you can customize and extend the implementation with your custom settings.

### Message extension

In order to make an `sdk.Msg` understandable by rosetta the only thing which is required is adding the methods to your messages that satisfy the `rosetta.Msg` interface. Examples on how to do so can be found in the staking types such as `MsgDelegate`, or in bank types such as `MsgSend`.

### Client interface override

In case more customization is required, it's possible to embed the Client type and override the methods which require customizations.

Example:

```go
package custom_client
import (

"context"
"github.com/coinbase/rosetta-sdk-go/types"
"cosmossdk.io/tools/rosetta/lib"
)

// CustomClient embeds the standard cosmos client
// which means that it implements the cosmos-rosetta-gateway Client
// interface while at the same time allowing to customize certain methods
type CustomClient struct {
    *rosetta.Client
}

func (c *CustomClient) ConstructionPayload(_ context.Context, request *types.ConstructionPayloadsRequest) (resp *types.ConstructionPayloadsResponse, err error) {
    // provide custom signature bytes
    panic("implement me")
}
```

NOTE: when using a customized client, the command cannot be used as the constructors required **may** differ, so it's required to create a new one. We intend to provide a way to init a customized client without writing extra code in the future.

### Error extension

Since rosetta requires to provide 'returned' errors to network options. In order to declare a new rosetta error, we use the `errors` package in cosmos-rosetta-gateway.

Example:

```go
package custom_errors
import crgerrs "cosmossdk.io/tools/rosetta/lib/errors"

var customErrRetriable = true
var CustomError = crgerrs.RegisterError(100, "custom message", customErrRetriable, "description")
```

Note: errors must be registered before cosmos-rosetta-gateway's `Server`.`Start` method is called. Otherwise the registration will be ignored. Errors with same code will be ignored too.
