---
order: 1
parent:
    title: Pruning Service
    order: 3
---

# Pruning data via the pruning service

CometBFT employs a sophisticated pruning logic to eliminate unnecessary data and reduce storage requirements.

This document covers use cases where the pruning process on a CometBFT node can be influenced via the Data Companion
pruning service API.

CometBFT provides a privileged gRPC endpoint for the pruning service. This privileged endpoint is distinct from the
non-privileged (regular) gRPC endpoint and requires separate configuration and activation. These "privileged" services
have the ability to manipulate the storage on the node.

Therefore, **only operators who have privileged access to the server should be allowed to use them**.

## Privileged Services configuration

CometBFT provides "privileged" services which are not intended to be exposed to the public-facing Internet.

The privileged services offered by CometBFT can modify the data stored in the node, and hence, it's essential to keep
them off by default to avoid any unintended modifications.

However, when required, these services can be activated to set and retrieve a retained height, which can influence
the pruning mechanism on the node.

To be able to use the privileged gRPC services, they should be enabled through CometBFT's configuration.

The first step is to set the address for the privileged service, for example:
```
#
# Configuration for privileged gRPC endpoints, which should **never** be exposed
# to the public internet.
#
[grpc.privileged]
# The host/port on which to expose privileged gRPC endpoints.
laddr = "tcp://0.0.0.0:26091"
```

In the `[grpc.privileged.pruning_service]` section, ensure the value `enabled` is set to `true`

```
#
# Configuration specifically for the gRPC pruning service, which is considered a
# privileged service.
#
[grpc.privileged.pruning_service]

# Only controls whether the pruning service is accessible via the gRPC API - not
# whether a previously set pruning service retain height is honored by the
# node. See the [storage.pruning] section for control over pruning.
#
# Disabled by default.
enabled = true
```

## Pruning configuration

Ensure that the data companion pruning is enabled in the configuration to allow the data companion to influence the
node pruning mechanism.

In the `[storage.pruning.data_companion]` section of the CometBFT's configuration
file, the property `enabled` should be set to `true`:

```
[storage.pruning.data_companion]

# Whether automatic pruning respects values set by the data companion. Disabled
# by default. All other parameters in this section are ignored when this is
# disabled.
#
# If disabled, only the application retain height will influence block pruning
# (but not block results pruning). Only enabling this at a later stage will
# potentially mean that blocks below the application-set retain height at the
# time will not be available to the data companion.
enabled = true
```

In order to avoid unwanted pruning of data when the data companion is activated, it is possible to define the initial
retain height for block and block results in the configuration file. This configuration ensures that the necessary data
is retained and not removed in an undesirable way.

For example, you can change the values in this part of the configuration:
```
# The initial value for the data companion block retain height if the data
# companion has not yet explicitly set one. If the data companion has already
# set a block retain height, this is ignored.
initial_block_retain_height = 0

# The initial value for the data companion block results retain height if the
# data companion has not yet explicitly set one. If the data companion has
# already set a block results retain height, this is ignored.
initial_block_results_retain_height = 10
```

## Retain Height

One important concept that can affect the pruning of nodes is the `retain height`. The retain height determines the specific
height from which the data can be safely deleted from the node's storage. By considering the retain height,
nodes can effectively manage their storage usage and ensure that they are only retaining the data that is necessary for
their operations. This is important because storage space is a finite resource and nodes with limited storage space may
struggle to keep up with the growth of the blockchain.

## Pruning Blocks

The pruning service uses the "block retain height" parameter to specify the height to which the node will
preserve blocks. It is important to note that this parameter differs from the application block retain height, which
the application sets in response to ABCI commit messages.

> NOTE: In order to set the block retain height on the node, you have to enable the privileged services endpoint and the
pruning service in the configuration as described in section above.

Once the services are enabled, you can use the Golang client provided by CometBFT to invoke the method that sets the block retain height.

Here is an example code:
```

import (
    "github.com/cometbft/cometbft/rpc/grpc/client/privileged"
)

ctx := context.Background()

// Privileged Service Client
addr := "0.0.0.0:26091"
conn, err := privileged.New(ctx, addr, privileged.WithInsecure())
if err != nil {
    // Do something with the error
}

err := conn.SetBlockRetainHeight(ctx, height)
if err != nil {
    // Do something with the error
}
```

If you need to check what is the current value for the `Block Retain Height` you can use another method.

Here's an example:
```
retainHeight, err := conn.GetBlockRetainHeight(ctx)
if err != nil {
    // Do something with the error
} else {
    // Do something with
    // `retainHeight.App`
    // `retainHeight.PruningService`
}
```

Retaining data is crucial to data management, and the application has complete control over the application retain height.
The operator can monitor the application retain height with `GetBlockRetainHeight`, which returns a `RetainHeights`
structure with both the block retain height and the application retain height (as shown in the code above).

It's worth noting that at any given point in time, the node will only accept the lowest retain height.
If you try to set the `Block Retain Height` to a value that is lower to what is currently stored in the node, an error will
be returned informing that.

By default, both the application retain height and the data companion retain height are set to zero. This is done to prevent
either one of them from prematurely pruning the data while the other has not indicated that it's okay to do so.

In essence, the node will preserve blocks up to the lowest value between data companion block retain height and the application
block retain height. This way, data can be reliably preserved and maintained for the necessary amount of time, ensuring
that it is not lost or prematurely deleted.

## Pruning Block Results

The "block results retain height" pruning parameter determines the height up to which the node will keep block results.
By retaining block results to a certain height, the node can efficiently manage its storage and optimize its performance.

> NOTE: In order to set the block results retain height on the node, you have to enable the privileged services endpoint and the
pruning service in the configuration as described in the section above.

Once the services are enabled, you can use the Golang client provided by CometBFT to invoke the method that sets the block results retain height.

Here is an example code:
```

import (
    "github.com/cometbft/cometbft/rpc/grpc/client/privileged"
)

ctx := context.Background()

// Privileged Service Client
addr := "0.0.0.0:26091"
conn, err := privileged.New(ctx, addr, privileged.WithInsecure())
if err != nil {
    // Do something with the error
}

err := conn.SetBlockResultsRetainHeight(ctx, height)
if err != nil {
    // Do something with the error
}

```

> NOTE: If you try to set the `Block Results Retain Height` to a value that is lower to what is currently stored in the node, an error will
be returned informing that.

If you need to check what is the current value for the `Block Results Retain Height` you can use another method.

Here's an example:
```
retainHeight, err := conn.GetBlockResultsRetainHeight(ctx)
if err != nil {
    // Do something with the error
} else {
    // Do something with the `retainHeight` value
}

```

> NOTE: Please note that if the `discard_abci_responses` in the `[storage]` section of the configuration file is set to `true`, then
block results are **not stored** on the node and the `Block Results Retain Height` will be ignored. In order to have block
results pruned the value should be set to `false` (default)

```
#######################################################
###         Storage Configuration Options           ###
#######################################################
[storage]

# Set to true to discard ABCI responses from the state store, which can save a
# considerable amount of disk space. Set to false to ensure ABCI responses are
# persisted. ABCI responses are required for /block_results RPC queries, and to
# reindex events in the command-line tool.
discard_abci_responses = false
```

## Pruning Block Indexed Data

The "block indexer retain height" pruning parameter determines the height up to which the node will keep block indexed data.

> NOTE: In order to set the block indexer retain height on the node, you have to enable the privileged services endpoint and the
pruning service in the configuration as described in the section above.

Once the services are enabled, you can use the Golang client provided by CometBFT to invoke the method that sets the block
indexer retain height.

Here is an example code:
```

import (
"github.com/cometbft/cometbft/rpc/grpc/client/privileged"
)

ctx := context.Background()

// Privileged Service Client
addr := "0.0.0.0:26091"
conn, err := privileged.New(ctx, addr, privileged.WithInsecure())
if err != nil {
    // Do something with the error
}

err := conn.SetBlockIndexerRetainHeight(ctx, height)
if err != nil {
    // Do something with the error
}

```

> NOTE: If you try to set the `Block Indexer Retain Height` to a value that is lower to what is currently stored in the node, an error will
be returned informing that.

If you need to check what is the current value for the `Block Indexer Retain Height` you can use another method.

Here's an example:
```
retainHeight, err := conn.GetBlockIndexerRetainHeight(ctx)
if err != nil {
    // Do something with the error
} else {
    // Do something with the `retainHeight` value
}

```

## Pruning Transaction Indexed Data

The "tx indexer retain height" pruning parameter determines the height up to which the node will keep transaction indexed data.

> NOTE: In order to set the tx indexer retain height on the node, you have to enable the privileged services endpoint and the
pruning service in the configuration as described in the section above.

Once the services are enabled, you can use the Golang client provided by CometBFT to invoke the method that sets the tx
indexer retain height.

Here is an example code:
```

import (
"github.com/cometbft/cometbft/rpc/grpc/client/privileged"
)

ctx := context.Background()

// Privileged Service Client
addr := "0.0.0.0:26091"
conn, err := privileged.New(ctx, addr, privileged.WithInsecure())
if err != nil {
    // Do something with the error
}

err := conn.SetTxIndexerRetainHeight(ctx, height)
if err != nil {
    // Do something with the error
}

```

> NOTE: If you try to set the `Tx Indexer Retain Height` to a value that is lower to what is currently stored in the node, an error will
be returned informing that.

If you need to check what is the current value for the `Tx Indexer Retain Height` you can use another method.

Here's an example:
```
retainHeight, err := conn.GetTxIndexerRetainHeight(ctx)
if err != nil {
    // Do something with the error
} else {
    // Do something with the `retainHeight` value
}

```

## Conclusion

Utilizing the pruning service can unlock remarkable benefits for your node. Whether used with a Data Companion service
or as a standalone solution, it can greatly enhance the pruning mechanism on your node, leading to significant cost
savings in node operation.
