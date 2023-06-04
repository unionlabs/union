# Ethereum Light Client Architecture

Our [Ethereum](https://ethereum.org/) light client. To see how this is integrated within `uniond`, see [uniond/ARCHITECTURE.md](../../uniond/ARCHITECTURE.md).

## Light Client Update
This diagram roughly describes the light client state update process, starting from our [Relayer](../../relayer/ARCHITECTURE.md)
```mermaid
sequenceDiagram
    Relayer->>Uniond: MsgUpdateClient
    Uniond->>WasmClient: UpdateState
    WasmClient->>EthereumVerifier: validate_light_client_update
    WasmClient->>WasmClient: UpdateState
```