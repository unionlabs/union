# simapp

simapp is an application built using the Cosmos SDK for testing and educational purposes.

## Running testnets with `simd`

If you want to spin up a quick testnet with your friends, you can follow these steps.
Unless otherwise noted, every step must be done by everyone who wants to participate
in this testnet.

1. `$ make build`. This will build the `simd` binary and install it in your Cosmos SDK repo,
    inside a new `build` directory. The following instructions are run from inside
    that directory.
2. If you've run `simd` before, you may need to reset your database before starting a new
    testnet: `$ ./simd unsafe-reset-all`
3. `$ ./simd init [moniker]`. This will initialize a new working directory, by default at
    `~/.simapp`. You need a provide a "moniker," but it doesn't matter what it is.
4. `$ ./simd keys add [key_name]`. This will create a new key, with a name of your choosing.
    Save the output of this command somewhere; you'll need the address generated here later.
5. `$ ./simd add-genesis-account  $(simd keys show [key_name] -a) [amount]`, where `key_name`
    is the same key name as before; and `amount` is something like `10000000000000000000000000stake`.
6. `$ ./simd gentx [key_name] [amount] --chain-id [chain-id]`. This will create the
    genesis transaction for your new chain.
7. Now, one person needs to create the genesis file `genesis.json` using the genesis transactions
   from every participant, by gathering all the genesis transactions under `config/gentx` and then
   calling `./simd collect-gentxs`. This will create a new `genesis.json` file that includes data
   from all the validators (we sometimes call it the "super genesis file" to distinguish it from
   single-validator genesis files).
8. Once you've received the super genesis file, overwrite your original `genesis.json` file with
    the new super `genesis.json`.
9. Modify your `config/config.toml` (in the simapp working directory) to include the other participants as
    persistent peers:

    ```text
    # Comma separated list of nodes to keep persistent connections to
    persistent_peers = "[validator address]@[ip address]:[port],[validator address]@[ip address]:[port]"
    ```

    You can find `validator address` by running `./simd tendermint show-node-id`. (It will be hex-encoded.)
    By default, `port` is 26656.
10. Now you can start your nodes: `$ ./simd start`.

Now you have a small testnet that you can use to try out changes to the Cosmos SDK or Tendermint!

NOTE: Sometimes creating the network through the `collect-gentxs` will fail, and validators will start
in a funny state (and then panic). If this happens, you can try to create and start the network first
with a single validator and then add additional validators using a `create-validator` transaction.
