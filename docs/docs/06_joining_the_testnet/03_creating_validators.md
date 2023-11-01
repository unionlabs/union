---
title: "Creating a Validator"
---

Before trying to create a validator, ensure you have the following task complete:

- Initialized your node
- Completed our intake form
- Received UNO tokens
- Started your node
- Ensured your node has caught up to the networks current height

Once all of these tasks are finished, you can continue with creating your validator.

## How Delegations Will Work

To ensure quick update cycles of the testnet, we ask that you only have a self delegation of 1 UNO. The Union team will delegate more to you to ensure you and the other validators have a similar delegation. This will enable us to maintain the majority of the total voting power so that we can quickly conduct updates to the network.

As we get closer to a mainnet release, we will update staking and slashing parameters and ask for more realistic self delegations. For the meantime, we will ensure to testnet is configured to change quickly with minimum friction.

## Creating your validator

Now that you're ready to create your validator, you can simply run the following command

```sh
uniond tx staking create-validator \
  --amount 1000000muno \
  --pubkey $(uniond tendermint show-validator) \
  --moniker $MONIKER \
  --chain-id union-testnet-4 \
  --from $KEY_NAME \
  --commission-max-change-rate "0.1" \
  --commission-max-rate "0.20" \
  --commission-rate "0.1" \
  --min-self-delegation "1"
```

:::note
If your own node isn't set up to accept RPC request, you can send them to another node such as `https://rpc.wakey.uno:443` via the `--node` option.
:::
