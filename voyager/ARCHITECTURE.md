# Voyager Architecture

```mermaid
classDiagram
    class Chain {
		    <<interface>>

		    +create_client(...State)
		    +update_client(...State)

        +connection_open_init(MsgConnection0penInit)
        +connection_open_try(MsgConnection0penTry)
        +connection_open_ack(MsgConnection0penAck)
        +connection_open_confirm(MsgConnection0penConfirm)

        +channel_open_init(MsgChannelOpenInit)
        +channel_open_try(MsgChannelOpenTry)
        +channel_open_ack(MsgChannelOpenAck)
        +channel_open_confirm(MsgChannel0penConfirm)

        +recv_packet(Packet)
    }

    Chain <|.. Cosmos
    Chain <|.. Evm
```

## Handshake

```mermaid
sequenceDiagram
    participant Cosmos
    participant Voyager
    participant Evm

    par Create WASM Client
        Voyager->>+Cosmos: create_client
        Cosmos-->>-Voyager: client_id
    and Create IBC Handler Instance
        Voyager->>+Evm: create_client
        Evm-->>-Voyager: client_id
    end

		note over Cosmos, Evm: connection handshake

    Voyager->>+Evm: MsgConnectionOpenInit
    Evm-->>-Voyager: connection_id

    Voyager->>+Cosmos: MsgConnectionOpenTry
    Cosmos-->>-Voyager: <<success>>

    Voyager->>+Evm: MsgConnectionOpenAck
    Evm-->>-Voyager: <<success>>

    Voyager->>+Cosmos: MsgConnectionOpenConfirm
    Cosmos-->>-Voyager: <<success>>

		note over Cosmos, Evm: channel handshake

    Voyager->>+Evm: MsgChannelOpenInit
    Evm-->>-Voyager: channel_id

    Voyager->>+Cosmos: MsgChannelOpenTry
    Cosmos-->>-Voyager: <<success>>

    Voyager->>+Evm: MsgChannelOpenAck
    Evm-->>-Voyager: <<success>>

    Voyager->>+Cosmos: MsgChannelOpenConfirm
    Cosmos-->>-Voyager: <<success>>
```

## Main loop

```mermaid
stateDiagram-v2
    poll : Listen for new blocks
    [*] --> poll
    poll --> onNewBlock

    note right of onNewBlock : from chain C

    onNewBlock --> connExists

    connExists : connection exists containing C?

    connExists --> poll : no
    connExists -->  updateClient : yes
    updateClient : call OtherChain#58;#58;update_client

    updateClient --> chanExists
    chanExists : channel exists and block\ncontains send packet?

    chanExists --> poll : no
    chanExists --> relayPacket : yes

    relayPacket : call OtherChain#58;#58;recv_packet

    relayPacket --> poll
```
