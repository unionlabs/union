---
order: 1
parent:
    title: gRPC services
    order: 2
---


# Fetching data from the node

One of the most important steps to create a Data Companion service is to extract the necessary data from the node.
Fortunately, CometBFT provides gRPC endpoints that allow you to fetch the data, such as `version`, `block` and
`block results`.

This documentation aims to provide a detailed explanation of CometBFT's gRPC services that can be used to retrieve
the data you need.

## Enabling the gRPC services

To utilize the gRPC services, it's necessary to enable them in CometBFT's configuration settings.

In the `[gRPC]` section of the configuration:
```
#######################################################
###       gRPC Server Configuration Options         ###
#######################################################

#
# Note that the gRPC server is exposed unauthenticated. It is critical that
# this server not be exposed directly to the public internet. If this service
# must be accessed via the public internet, please ensure that appropriate
# precautions are taken (e.g. fronting with a reverse proxy like nginx with TLS
# termination and authentication, using DDoS protection services like
# CloudFlare, etc.).
#

[grpc]
```

Add the address for the non-privileged (regular) services, for example:

```
# TCP or UNIX socket address for the RPC server to listen on. If not specified,
# the gRPC server will be disabled.
laddr = "tcp://0.0.0.0:26090"
```

The non-privileged gRPC endpoint is **enabled by default**. Each individual service exposed in this endpoint can be disabled
or enabled individually. For example, to enable the `Version` service, in the `[grpc.version_service]` section, ensure
that the `enabled` property is set to `true`:

```
#
# Each gRPC service can be turned on/off, and in some cases configured,
# individually. If the gRPC server is not enabled, all individual services'
# configurations are ignored.
#

# The gRPC version service provides version information about the node and the
# protocols it uses.
[grpc.version_service]
enabled = true
```

Do the same thing for the `block_service` and the `block_results_service` to enable them.

```
# The gRPC block service returns block information
[grpc.block_service]
enabled = true

# The gRPC block results service returns block results for a given height. If no height
# is given, it will return the block results from the latest height.
[grpc.block_results_service]
enabled = true
```

## Fetching **Block** data

In order to retrieve `block` data using the gRPC block service, ensure the service is enabled as described in the section above.

Once the service has been enabled, the Golang gRPC client provided by CometBFT can be utilized to retrieve data from the node.

This client code is a convenient option for retrieving data, as it allows for requests to be sent and responses to be
managed in a more idiomatic manner. However, if necessary and desired, the protobuf client can also be used directly.

Here is an example code to retrieve a block by its height:
```
import (
     "github.com/cometbft/cometbft/rpc/grpc/client"
)

ctx := context.Background()

// Service Client
addr := "0.0.0.0:26090"
conn, err := client.New(ctx, addr, client.WithInsecure())
if err != nil {
    // Do something with the error
}

block, err := conn.GetBlockByHeight(ctx, height)
if err != nil {
    // Do something with the error
} else {
    // Do something with the `block`
}

```

## Fetching **Block Results** data

To fetch `block results` you can use a similar code as the previous one but just invoking the method to that retrieves
block results.

Here's an example:
```
blockResults, err := conn.GetBlockResults(ctx, height)
if err != nil {
    // Do something with the error
} else {
    // Do something with the `blockResults`
}

```

## Latest height streaming

There is a new way to subscribe to a stream of new blocks with the Block service. Previously, you could connect and
subscribe to new block events using websockets through the RPC service.

One of the advantages of the new streaming service is that it allows you to opt for the latest height subscription.
This way, the gRPC endpoint will not have to transfer entire blocks to keep you updated. Instead, you can fetch the
blocks at the desired pace through the `GetBlockByHeight` method.

To receive the latest height from the stream, you need to call the method that returns the receive-only channel and then
watch for messages that come through the channel. The message sent on the channel is a `LatestHeightResult` struct.

```
// LatestHeightResult type used in GetLatestResult and send to the client
// via a channel
type LatestHeightResult struct {
    Height int64
    Error  error
}
```

Once you get a message, you can check the `Height` field for the latest height (assuming the `Error` field is nil)

Here's an example:
```
import (
"github.com/cometbft/cometbft/rpc/grpc/client"
)

ctx := context.Background()

// Service Client
addr := "0.0.0.0:26090"
conn, err := client.New(ctx, addr, client.WithInsecure())
if err != nil {
    // Do something with the error
}

stream, err := conn.GetLatestHeight(ctx)
if err != nil {
    // Do something with the error
}

for {
    select {
    case <- ctx.Done():
        return
    case latestHeight, ok := <-stream:
        if ok {
            if latestHeight.Error != nil {
                // Do something with error
            } else {
                // Latest Height -> latestHeight.Height
            }
        } else {
            return
        }
    }
}
```

The ability to monitor new blocks is attractive as it unlocks avenues for creating dynamic pipelines and ingesting services
via the producer-consumer pattern.

For instance, upon receiving a notification about a fresh block, one can activate a method to retrieve block data and
save it in a database. Subsequently, the node can set a retain height, allowing for data pruning.

## Storing the fetched data

In the Data Companion workflow, the second step involves saving the data retrieved from a blockchain onto an external
storage medium, such as a database. This external storage medium is important because it allows the data to be accessed
and utilized by custom web services that can serve the blockchain data in a more efficient way.

When choosing a database, evaluate your specific needs, including data size, user access, and budget.
For example, the [RPC Companion](https://github.com/cometbft/rpc-companion) uses Postgresql as a starting point, but there
are many other options to consider. Choose a database that meets your needs and helps you achieve your objectives.

Before proceeding to the next step, it is crucial to verify that the data has been correctly stored in the external database.
Once you have confirmed that the data has been successfully stored externally, you can proceed to update the new "retain_height"
information. This update will allow the node to prune the information that is now stored externally.

## Pruning the node data

In order to successfully execute the Data Companion workflow, the third step entails utilizing the newly introduced
gRPC pruning service API to set certain retain height values on the node. The pruning service allows the data companion
to effectively influence the pruning of blocks and state, ABCI results (if enabled), block indexer data and transaction
indexer data on the node.

For a comprehensive understanding of the pruning service, please see the document
[Pruning service](pruning.md).
