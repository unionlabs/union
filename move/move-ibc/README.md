# Move IBC

Our Move-based implementation is designed to be permissionless, where any smart contract can plug in their
implementation and start doing general message passing and asset transfers by using our IBC contract without
any permission or fee.

Instead of explaining the IBC implementation in detail, this document aims to give app developers all they
need to implement and understand how an IBC app can be implemented on Union.

## Adding the IBC protocol as a dependency

Please refer to [the Aptos documentation](https://aptos.dev/en/build/smart-contracts/third-party-dependencies) to learn more about
how to integrate third-party dependencies.

## Integrating an app with Union-IBC

Our relayer a.k.a [Voyager](https://github.com/unionlabs/union/tree/main/voyager) requires the IBC entry points
to be defined under the module `ibc`. So to start with your implementation, make sure to define all the described
entry functions under `ibc`:

```move
module my_app::ibc {

}
```

In IBC, every IBC app talks to each other via IBC channels. The IBC contract only lets the corresponding IBC
app send and receive a packet on the channel that it owns. This means a channel can only be owned by a single app
but a single app can own multiple channels. This architecture ensures that apps have exclusive channels for different
chains with different configurations.

Here is the definition of `channel_open_init` function in IBC core:

```move
public fun channel_open_init(
    ibc_app: &signer, // this is the caller which should be the `ibc_app`
    port_id: address,
    connection_hops: vector<String>,
    ordering: u8,
    counterparty: channel::Counterparty,
    version: String,
): (Channel, u64);
```

As you can see, IBC expects this function to be called with `ibc_app` `signer` and will later use this
information for authentication. The authentication is done with the following check:

```move
assert!(object::create_object_address(&port_id, IBC_APP_SEED) == signer::address_of(ibc_app), E_UNAUTHORIZED);
```

`port_id` should be the address of the ibc app. And the `signer` must be the `object signer` that is generated
in the `init_module` with the correct seed. Here is an example setup in the app:

```move
module my_app::ibc {
    // Make sure to use this exact seed
    const IBC_APP_SEED: vector<u8> = b"union-ibc-app-v1";

    fun init_module(deployer: &signer) {
        assert!(signer::address_of(deployer) == @my_app, 1);

        let vault_constructor_ref = &object::create_named_object(deployer, IBC_APP_SEED);
        let vault_signer = &object::generate_signer(vault_constructor_ref);

        move_to(vault_signer, SignerRef {
            self_ref: object::generate_extend_ref(vault_constructor_ref),
            self_address: signer::address_of(deployer),
        });
    }

    public fun get_signer(): signer acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        object::generate_signer_for_extending(&vault.self_ref)
    }

    public fun get_self_address(): address acquires SignerRef {
        let vault = borrow_global<SignerRef>(get_vault_addr());
        vault.self_address
    }
}
```

In this function, we are saving an [ExtendRef](https://legacy.aptos.dev/reference/move/?branch=mainnet&page=aptos-framework/doc/object.md#0x1_object_ExtendRef)
which is used later to provide the `signer` to the IBC app via `get_signer` method. `get_self_address` is also implemented to
get the contract address.

Note that IBC also uses the same `IBC_APP_SEED` to generate the correct address. Since the address generation function
is deterministic, the app and IBC can calculate the same address.

Now to the channel opening part. In all of other IBC implementations, entry functions for the channel handshake are
defined under the core protocol. But since Move doesn't let the contracts call arbitrary contracts (a.k.a dynamic dispatch),
in this implementation, the app is responsible for defining the handshake entry functions and calling the IBC contract.

```move
module my_app::ibc {
    public entry fun channel_open_init(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        version: String,
    ) acquires SignerRef {
        ibc::channel_open_init(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            version,
        );
    }

    public entry fun channel_open_try(
        connection_hops: vector<String>,
        ordering: u8,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        version: String,
        proof_init: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires SignerRef {
        ibc::channel_open_try(
            &get_signer(),
            get_self_address(),
            connection_hops,
            ordering,
            channel::new_counterparty(counterparty_port_id, counterparty_channel_id),
            counterparty_version,
            version,
            proof_init,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
    }

    public entry fun channel_open_ack(
        channel_id: String,
        counterparty_channel_id: String,
        counterparty_version: String,
        proof_try: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires LocalStorage, SignerRef {
        save_channel(channel_id)

        ibc::channel_open_ack(
            &get_signer(),
            get_self_address(),
            channel_id,
            counterparty_channel_id,
            counterparty_version,
            proof_try,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
        borrow_global_mut<PingPong>(get_vault_addr()).channel_id = channel_id;
    }

    public entry fun channel_open_confirm(
        channel_id: String,
        proof_ack: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires LocalStorage, SignerRef {
        save_channel(channel_id);

        ibc::channel_open_confirm(
            &get_signer(),
            get_self_address(),
            channel_id,
            proof_ack,
            height::new(proof_height_revision_num, proof_height_revision_height),
        );
    }
}
```

Most of these functions are just there in order to pass the execution to IBC. Also most of the time,
your app will require doing some bookkeeping. In this case, you can save the channel information during
`channel_open_ack` and `channel_open_confirm`. During a channel handshake, depending on where the channel
handshake is initiated, either `channel_open_init` then `channel_open_ack` or `channel_open_try`
then `channel_open_confirm` is called. This means `init` and `try` can have identical implementations
and the same applies to `ack` and `confirm`.

Now the exciting part, packet send and receive. The flow of two chains communicating works like this:

1. IBC app on chain A calls `ibc::send_packet` to send a packet to chain B. The packet is relayed by the relayers.
2. `ibc::recv_packet` function of IBC app on chain B is being called. The app receives a packet and calls `ibc::recv_packet` with an acknowledgment indicating whether the operation
   is successful or not.
3. `ibc::acknowledge_packet` function of IBC app on chain A is being called. This is called to let the sender app whether the
   packet transfer succeeded. For example, if this were to be an asset transfer app, the locked assets could be refunded when the transfer
   fails.

Let's first do the packet sending:

```move
module my_app::ibc {
  public entry fun transfer_funds(caller: &signer, channel: String, timeout_seconds: u64) {
    let transfer_packet = do_transfer(caller);
    let encoded_packet = encode_packet(transfer_packet);

    ibc::send_packet(
      &get_signer(),
      get_self_address(),
      channel,
      height::default(), // no height timeout
      (std::timestamp::now_seconds() + timeout_seconds) * 1_000_000_000, // timeout in nanoseconds
      encoded_packet
    )
  }
}
```

Here we define whatever entry function we like. This example app defines a function to transfer some funds
from the `caller` account using `channel`. Note that a `timeout` is also defined. IBC lets you define a height
or timestamp-based timeout. Our app chose to work with timestamps here.

The app did its transfer logic, and returned a packet. Then it encoded that packet however it liked and sent that packet via IBC.

Now the receiving part:

```move

module my_app::ibc {
    public entry fun recv_packet(
        packet_sequence: u64,
        packet_source_port: String,
        packet_source_channel: String,
        packet_destination_port: String,
        packet_destination_channel: String,
        packet_data: vector<u8>,
        packet_timeout_revision_num: u64,
        packet_timeout_revision_height: u64,
        packet_timeout_timestamp: u64,
        proof: vector<u8>,
        proof_height_revision_num: u64,
        proof_height_revision_height: u64,
    ) acquires PingPong, SignerRef {
        let my_packet = decode_packet(&packet_data);

        let ack = if (receive_transfer(my_packet) == STATUS_SUCCESS) {
          ACK_SUCCESS // vector[1]
        } else
          ACK_FAILURE // vector[0]
        };

        ibc::recv_packet(
            &get_signer(),
            get_self_address(),
            packet::new(
                packet_sequence,
                packet_source_port,
                packet_source_channel,
                packet_destination_port,
                packet_destination_channel,
                packet_data,
                height::new(
                    packet_timeout_revision_num,
                    packet_timeout_revision_height,
                ),
                packet_timeout_timestamp,
            ),
            proof,
            height::new(proof_height_revision_num, proof_height_revision_height),
            ack,
        );
    }
}
```

As you can see, `decode_packet` is called with `packet_data`. This is intentionally left to the app developers
because IBC has no assumption on what the packet is. So you can use whatever packet type you want.

Next, `receive_transfer` is called which should be the receiving logic defined by the protocol.

Finally, to let IBC know a packet is received, `recv_packet` function of IBC is being called. The important part here is the last
parameter (`ack`). This is the arbitrary acknowledgment that is used for letting the counterparty chain know whether the transaction is
succeeded or not. In this implementation, we chose to use `1` and `0` but you can also put the error message to the failure case.

The final part of the flow is the acknowledgement. When we sent some funds, `do_transfer` function possibly
locked some funds, burned some tokens, and/or changed the balance of `caller`. Now, in the success case, we don't
need to do anything but if the action failed, we need to undo the things that we did.

```move
module my_app::ibc {
  public entry fun acknowledge_packet(
      packet_sequence: u64,
      packet_source_port: String,
      packet_source_channel: String,
      packet_destination_port: String,
      packet_destination_channel: String,
      packet_data: vector<u8>,
      packet_timeout_revision_num: u64,
      packet_timeout_revision_height: u64,
      packet_timeout_timestamp: u64,
      acknowledgement: vector<u8>,
      proof: vector<u8>,
      proof_height_revision_num: u64,
      proof_height_revision_height: u64,
  ) acquires SignerRef {
    if (acknowledgement == ACK_FAILURE) {
      undo_transfer(packet_data);
    };

    ibc::acknowledge_packet(
      &get_signer(),
      get_self_address(),
      packet::new(
          packet_sequence,
          packet_source_port,
          packet_source_channel,
          packet_destination_port,
          packet_destination_channel,
          packet_data,
          height::new(
              packet_timeout_revision_num,
              packet_timeout_revision_height,
          ),
          packet_timeout_timestamp,
      ),
      acknowledgement,
      proof,
      height::new(proof_height_revision_num, proof_height_revision_height),
    );
  }
}
```

Finally, we have the `timeout_packet` entry function. This is being called when the packet that we sent is timed out.
Note that only the sender app is being notified when this happens. Here is an example implementation:

```move
module my_app::ibc {
  public entry fun timeout_packet(
      packet_sequence: u64,
      packet_source_port: String,
      packet_source_channel: String,
      packet_destination_port: String,
      packet_destination_channel: String,
      packet_data: vector<u8>,
      packet_timeout_revision_num: u64,
      packet_timeout_revision_height: u64,
      packet_timeout_timestamp: u64,
      proof: vector<u8>,
      proof_height_revision_num: u64,
      proof_height_revision_height: u64,
      next_sequence_receive: u64,
  ) acquires RelayStore, SignerRef {
    let packet = decode_packet(packet_data);

    // refund the sender
    refund_tokens(packet_sequence, packet_source_channel, &packet);

    ibc::timeout_packet(
      &get_signer(),
      get_self_address(),
      packet::new(
        packet_sequence,
        packet_source_port,
        packet_source_channel,
        packet_destination_port,
        packet_destination_channel,
        packet_data,
        height::new(
          packet_timeout_revision_num,
          packet_timeout_revision_height,
        ),
        packet_timeout_timestamp,
      ),
      proof,
      height::new(proof_height_revision_num, proof_height_revision_height),
      next_sequence_receive,
    );
  }
}
```
