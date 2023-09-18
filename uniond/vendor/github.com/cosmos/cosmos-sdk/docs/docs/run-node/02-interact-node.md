---
sidebar_position: 1
---

# Interacting with the Node

:::note Synopsis
There are multiple ways to interact with a node: using the CLI, using gRPC or using the REST endpoints.
:::

:::note Pre-requisite Readings

* [gRPC, REST and CometBFT Endpoints](../core/06-grpc_rest.md)
* [Running a Node](./01-run-node.md)

:::

## Using the CLI

Now that your chain is running, it is time to try sending tokens from the first account you created to a second account. In a new terminal window, start by running the following query command:

```bash
simd query bank balances $MY_VALIDATOR_ADDRESS
```

You should see the current balance of the account you created, equal to the original balance of `stake` you granted it minus the amount you delegated via the `gentx`. Now, create a second account:

```bash
simd keys add recipient --keyring-backend test

# Put the generated address in a variable for later use.
RECIPIENT=$(simd keys show recipient -a --keyring-backend test)
```

The command above creates a local key-pair that is not yet registered on the chain. An account is created the first time it receives tokens from another account. Now, run the following command to send tokens to the `recipient` account:

```bash
simd tx bank send $MY_VALIDATOR_ADDRESS $RECIPIENT 1000000stake --chain-id my-test-chain --keyring-backend test

# Check that the recipient account did receive the tokens.
simd query bank balances $RECIPIENT
```

Finally, delegate some of the stake tokens sent to the `recipient` account to the validator:

```bash
simd tx staking delegate $(simd keys show my_validator --bech val -a --keyring-backend test) 500stake --from recipient --chain-id my-test-chain --keyring-backend test

# Query the total delegations to `validator`.
simd query staking delegations-to $(simd keys show my_validator --bech val -a --keyring-backend test)
```

You should see two delegations, the first one made from the `gentx`, and the second one you just performed from the `recipient` account.

## Using gRPC

The Protobuf ecosystem developed tools for different use cases, including code-generation from `*.proto` files into various languages. These tools allow the building of clients easily. Often, the client connection (i.e. the transport) can be plugged and replaced very easily. Let's explore one of the most popular transport: [gRPC](../core/06-grpc_rest.md).

Since the code generation library largely depends on your own tech stack, we will only present three alternatives:

* `grpcurl` for generic debugging and testing,
* programmatically via Go,
* CosmJS for JavaScript/TypeScript developers.

### grpcurl

[grpcurl](https://github.com/fullstorydev/grpcurl) is like `curl` but for gRPC. It is also available as a Go library, but we will use it only as a CLI command for debugging and testing purposes. Follow the instructions in the previous link to install it.

Assuming you have a local node running (either a localnet, or connected a live network), you should be able to run the following command to list the Protobuf services available (you can replace `localhost:9000` by the gRPC server endpoint of another node, which is configured under the `grpc.address` field inside [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml)):

```bash
grpcurl -plaintext localhost:9090 list
```

You should see a list of gRPC services, like `cosmos.bank.v1beta1.Query`. This is called reflection, which is a Protobuf endpoint returning a description of all available endpoints. Each of these represents a different Protobuf service, and each service exposes multiple RPC methods you can query against.

In order to get a description of the service you can run the following command:

```bash
grpcurl -plaintext \
    localhost:9090 \
    describe cosmos.bank.v1beta1.Query                  # Service we want to inspect
```

It's also possible to execute an RPC call to query the node for information:

```bash
grpcurl \
    -plaintext \
    -d "{\"address\":\"$MY_VALIDATOR_ADDRESS\"}" \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/AllBalances
```

The list of all available gRPC query endpoints is [coming soon](https://github.com/cosmos/cosmos-sdk/issues/7786).

#### Query for historical state using grpcurl

You may also query for historical data by passing some [gRPC metadata](https://github.com/grpc/grpc-go/blob/master/Documentation/grpc-metadata.md) to the query: the `x-cosmos-block-height` metadata should contain the block to query. Using grpcurl as above, the command looks like:

```bash
grpcurl \
    -plaintext \
    -H "x-cosmos-block-height: 123" \
    -d "{\"address\":\"$MY_VALIDATOR_ADDRESS\"}" \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/AllBalances
```

Assuming the state at that block has not yet been pruned by the node, this query should return a non-empty response.

### Programmatically via Go

The following snippet shows how to query the state using gRPC inside a Go program. The idea is to create a gRPC connection, and use the Protobuf-generated client code to query the gRPC server.

#### Install Cosmos SDK


```bash
go get github.com/cosmos/cosmos-sdk@main
```

```go
package main

import (
    "context"
    "fmt"

    "google.golang.org/grpc"

    "github.com/cosmos/cosmos-sdk/codec"
    sdk "github.com/cosmos/cosmos-sdk/types"
    banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func queryState() error {
    myAddress, err := sdk.AccAddressFromBech32("cosmos1...") // the my_validator or recipient address.
    if err != nil {
        return err
    }

    // Create a connection to the gRPC server.
    grpcConn, err := grpc.Dial(
        "127.0.0.1:9090", // your gRPC server address.
        grpc.WithInsecure(), // The Cosmos SDK doesn't support any transport security mechanism. 
        // This instantiates a general gRPC codec which handles proto bytes. We pass in a nil interface registry
        // if the request/response types contain interface instead of 'nil' you should pass the application specific codec.
		grpc.WithDefaultCallOptions(grpc.ForceCodec(codec.NewProtoCodec(nil).GRPCCodec())),
	)
    if err != nil {
        return err
    }
    defer grpcConn.Close()

    // This creates a gRPC client to query the x/bank service.
    bankClient := banktypes.NewQueryClient(grpcConn)
    bankRes, err := bankClient.Balance(
        context.Background(),
        &banktypes.QueryBalanceRequest{Address: myAddress.String(), Denom: "stake"},
    )
    if err != nil {
        return err
    }

    fmt.Println(bankRes.GetBalance()) // Prints the account balance

    return nil
}

func main() {
    if err := queryState(); err != nil {
        panic(err)
    }
}
```

You can replace the query client (here we are using `x/bank`'s) with one generated from any other Protobuf service. The list of all available gRPC query endpoints is [coming soon](https://github.com/cosmos/cosmos-sdk/issues/7786).

#### Query for historical state using Go

Querying for historical blocks is done by adding the block height metadata in the gRPC request.

```go
package main

import (
	"context"
	"fmt"

	"google.golang.org/grpc"
	"google.golang.org/grpc/metadata"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	grpctypes "github.com/cosmos/cosmos-sdk/types/grpc"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func queryState() error {
	myAddress, err := sdk.AccAddressFromBech32("cosmos1yerherx4d43gj5wa3zl5vflj9d4pln42n7kuzu") // the my_validator or recipient address.
	if err != nil {
		return err
	}

	// Create a connection to the gRPC server.
	grpcConn, err := grpc.Dial(
		"127.0.0.1:9090",    // your gRPC server address.
		grpc.WithInsecure(), // The Cosmos SDK doesn't support any transport security mechanism.
		// This instantiates a general gRPC codec which handles proto bytes. We pass in a nil interface registry
		// if the request/response types contain interface instead of 'nil' you should pass the application specific codec.
		grpc.WithDefaultCallOptions(grpc.ForceCodec(codec.NewProtoCodec(nil).GRPCCodec())),
	)
	if err != nil {
		return err
	}
	defer grpcConn.Close()

	// This creates a gRPC client to query the x/bank service.
	bankClient := banktypes.NewQueryClient(grpcConn)

	var header metadata.MD
	_, err = bankClient.Balance(
		metadata.AppendToOutgoingContext(context.Background(), grpctypes.GRPCBlockHeightHeader, "12"), // Add metadata to request
		&banktypes.QueryBalanceRequest{Address: myAddress.String(), Denom: "stake"},
		grpc.Header(&header), // Retrieve header from response
	)
	if err != nil {
		return err
	}
	blockHeight := header.Get(grpctypes.GRPCBlockHeightHeader)

	fmt.Println(blockHeight) // Prints the block height (12)

	return nil
}

func main() {
    if err := queryState(); err != nil {
        panic(err)
    }
}
```

### CosmJS

CosmJS documentation can be found at [https://cosmos.github.io/cosmjs](https://cosmos.github.io/cosmjs). As of January 2021, CosmJS documentation is still work in progress.

## Using the REST Endpoints

As described in the [gRPC guide](../core/06-grpc_rest.md), all gRPC services on the Cosmos SDK are made available for more convenient REST-based queries through gRPC-gateway. The format of the URL path is based on the Protobuf service method's full-qualified name, but may contain small customizations so that final URLs look more idiomatic. For example, the REST endpoint for the `cosmos.bank.v1beta1.Query/AllBalances` method is `GET /cosmos/bank/v1beta1/balances/{address}`. Request arguments are passed as query parameters.

Note that the REST endpoints are not enabled by default. To enable them, edit the `api` section of your  `~/.simapp/config/app.toml` file:

```toml
# Enable defines if the API server should be enabled.
enable = true
```

As a concrete example, the `curl` command to make balances request is:

```bash
curl \
    -X GET \
    -H "Content-Type: application/json" \
    http://localhost:1317/cosmos/bank/v1beta1/balances/$MY_VALIDATOR_ADDRESS
```

Make sure to replace `localhost:1317` with the REST endpoint of your node, configured under the `api.address` field.

The list of all available REST endpoints is available as a Swagger specification file, it can be viewed at `localhost:1317/swagger`. Make sure that the `api.swagger` field is set to true in your [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml) file.

### Query for historical state using REST

Querying for historical state is done using the HTTP header `x-cosmos-block-height`. For example, a curl command would look like:

```bash
curl \
    -X GET \
    -H "Content-Type: application/json" \
    -H "x-cosmos-block-height: 123" \
    http://localhost:1317/cosmos/bank/v1beta1/balances/$MY_VALIDATOR_ADDRESS
```

Assuming the state at that block has not yet been pruned by the node, this query should return a non-empty response.

### Cross-Origin Resource Sharing (CORS)

[CORS policies](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) are not enabled by default to help with security. If you would like to use the rest-server in a public environment we recommend you provide a reverse proxy, this can be done with [nginx](https://www.nginx.com/). For testing and development purposes there is an `enabled-unsafe-cors` field inside [`app.toml`](../run-node/01-run-node.md#configuring-the-node-using-apptoml).
