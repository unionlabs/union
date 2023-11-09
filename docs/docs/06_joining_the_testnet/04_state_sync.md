# State Sync

If the network has produced many blocks or has seen several binary upgrades, you may find it significantly easier to join the test network via state sync.

## Joining The Network Via State Sync

To join the network with state sync, you will need to

1. Get trusted height information from an RPC
2. Configure your node to use state sync
3. Start your node

### Getting Trusted Height Information

Before joining the network using state sync, you will need to use one of our RPC nodes to obtain the current trusted height and the block hash of the trusted height.

To do this, you can run the following command:

```sh
curl -s https://rpc.cryptware.io/block | jq -r '.result.block.header.height + "\n" + .result.block_id.hash'
```

You should then see output in the form of:

```
<trusted_height>
<trusted_hash>
```

### Configuring Your Node to Use State Sync

Now, to configure your node to use state sync, you'll need to edit the TOML file `~/.union/config/config.toml`.

Find the `statesync` TOML table, and using the information from the last step, set the fields as such:

```toml
[statesync]
enable = true
rpc_servers = "https://rpc.cryptware.io:443,https://rpc.purmuzlu.cc:443"
trust_height = 11143 # <trusted_height>
trust_hash = "DAD8FE1231B030B27D36634C52DEAECCABDB6AA0AFDECC9459E507A254D4D6C9" # <trusted_hash>
trust_period = "400s"
```

### Start Your Node

Now you should be able to start your node normally. You should see log messages saying it has "Discovered a new snapshot"
