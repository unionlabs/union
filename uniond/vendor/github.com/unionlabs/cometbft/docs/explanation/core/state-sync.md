--- 
order: 11
---

# State Sync

With block sync a node is downloading all of the data of an application from genesis and verifying it.
With state sync your node will download data related to the head or near the head of the chain and verify the data.
This leads to drastically shorter times for joining a network.

## Using State Sync

State sync will continuously work in the background to supply nodes with chunked data when bootstrapping.

> NOTE: Before trying to use state sync, see if the application you are operating a node for supports it.

Under the state sync section in `config.toml` you will find multiple settings that need to be configured in order for your node to use state sync.

Lets breakdown the settings:

- `enable`: Enable is to inform the node that you will be using state sync to bootstrap your node.
- `rpc_servers`: RPC servers are needed because state sync utilizes the light client for verification.
    - 2 servers are required, more is always helpful.
- `temp_dir`: Temporary directory is store the chunks in the machines local storage, If nothing is set it will create a directory in `/tmp`

The next information you will need to acquire it through publicly exposed RPC's or a block explorer which you trust.

- `trust_height`: Trusted height defines at which height your node should trust the chain.
- `trust_hash`: Trusted hash is the hash in the `BlockID` corresponding to the trusted height.
- `trust_period`: Trust period is the period in which headers can be verified.
  > :warning: This value should be significantly smaller than the unbonding period.

If you are relying on publicly exposed RPC's to get the need information, you can use `curl` and [`jq`][jq].

Example:

```bash
curl -s https://233.123.0.140:26657/commit | jq "{height: .result.signed_header.header.height, hash: .result.signed_header.commit.block_id.hash}"
```

The response will be:

```json
{
  "height": "273",
  "hash": "188F4F36CBCD2C91B57509BBF231C777E79B52EE3E0D90D06B1A25EB16E6E23D"
}
```

[jq]: https://jqlang.github.io/jq/
