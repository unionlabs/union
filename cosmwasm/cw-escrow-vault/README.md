# CW Escrow Vault

## Overview

The CW Escrow Vault is a CosmWasm smart contract that acts as a solver in the UCS03 ZKGM protocol, enabling fungible cross-chain asset transfers between Cosmos and EVM chains. It manages the escrow and release of native and CW20 tokens, serving as the Cosmos-side counterparty to the UnionversalToken.sol contract on EVM chains.

## Purpose in the ZKGM Protocol

In the UCS03 ZKGM protocol, cross-chain transfers use a sophisticated filling mechanism where:

1. **Market Makers/Solvers** can fill orders by providing liquidity on the destination chain
2. **Acknowledgements** carry information about who filled the order (the beneficiary)
3. **Base tokens** are sent to the filler as compensation on the source chain

The CW Escrow Vault implements the **ISolver interface** to participate as an automated market maker in this system, creating a fungible lane between chains.

## Architecture

### Key Components

- **CW Escrow Vault (Cosmos)**: This contract - acts as a solver that fills orders with escrowed tokens
- **UnionversalToken.sol (EVM)**: The EVM counterpart - mints/burns synthetic tokens and acts as a solver
- **UCS03 ZKGM Protocol**: The cross-chain messaging protocol with open filling mechanism

### Fungible Lane Configuration

Both vaults are configured as solvers where:

- For Cosmos → EVM: Escrow Vault stores UnionversalToken.sol address as `counterparty_beneficiary`
- For EVM → Cosmos: Escrow Vault returns zero address to trigger token burning
- When filling orders, the vault returns the appropriate beneficiary in the acknowledgement
- The source chain either sends base tokens to the beneficiary or burns them (if beneficiary is 0)

## Transfer Flow

### Forward Transfer (Cosmos → EVM)

1. **User initiates transfer**: Sends native/CW20 tokens via ZKGM with a TokenOrderV2
2. **Packet sent to EVM**: ZKGM sends packet across IBC to the destination chain
3. **UnionversalToken.sol fills the order**:
   - Mints synthetic tokens to the receiver
   - Returns CW Escrow Vault address as beneficiary in acknowledgement
4. **Base tokens sent to vault**: ZKGM on Cosmos sends base tokens to the Escrow Vault
5. **Tokens held in escrow**: Vault holds tokens for future reverse transfers

### Reverse Transfer (EVM → Cosmos)

1. **User initiates transfer**: Sends synthetic tokens on EVM via ZKGM with a TokenOrderV2
2. **Packet sent to Cosmos**: ZKGM sends packet across IBC to the destination chain
3. **Escrow Vault fills the order**:
   - Releases escrowed tokens to the receiver
   - Returns zero address (0x0) as beneficiary in acknowledgement
4. **Base tokens burned**: ZKGM on EVM burns the base tokens (synthetic tokens) since beneficiary is 0x0
5. **Balance maintained**: Burning on EVM and releasing from escrow on Cosmos maintains token supply

## Implementation Details

### Solver Interface

The contract implements the ZKGM solver interface:

```rust
pub enum QueryMsg {
    IsSolver,           // Returns success to indicate solver capability
    AllowMarketMakers,  // Returns true to allow other market makers
}

pub enum ExecuteMsg {
    DoSolve {
        packet: Packet,
        order: CwTokenOrderV2,
        path: Uint256,
        caller: Addr,
        relayer: Addr,
        relayer_msg: Bytes,
        intent: bool,
    }
}
```

### Order Filling Logic

When `DoSolve` is called:

1. **Intent Validation**: If flagged as intent, checks packet hash is whitelisted
2. **Lane Validation**: Verifies a fungible lane exists for the (path, channel, token) tuple
3. **Fee Distribution**:
   - Calculates fee as `base_amount - quote_amount`
   - Sends fee to the relayer
4. **Token Transfer**: Sends `quote_amount` to the receiver
5. **Return Beneficiary**: Returns the configured `counterparty_beneficiary` address

### Configuration

#### Setting Fungible Counterparties

```rust
SetFungibleCounterparty {
    path: Uint256,                    // Routing path identifier
    channel_id: ChannelId,            // IBC channel
    base_token: Bytes,               // Token on source chain
    counterparty_beneficiary: Bytes, // UnionversalToken.sol address on EVM
    escrowed_denom: String,          // Local token to use for filling
}
```

#### Intent Whitelisting

```rust
WhitelistIntents {
    hashes_whitelist: Vec<(H256, bool)>, // Packet hashes and approval
}
```

## Security Features

- **Admin-only configuration**: Only admin can set fungible lanes and whitelist intents
- **ZKGM-only execution**: Only the ZKGM contract can call `DoSolve`
- **Lane validation**: Orders are only filled for configured fungible lanes
- **Intent protection**: Intent-based orders require whitelisting

## Benefits

1. **Capital Efficiency**: No need for separate liquidity pools - uses escrowed tokens
2. **Instant Filling**: Acts as always-available market maker for configured lanes
3. **Fee Incentives**: Relayers earn fees for submitting packets
4. **True Fungibility**: Maintains 1:1 backing between native and synthetic tokens
5. **Trustless Operation**: Smart contract automation eliminates counterparty risk

## Integration

To integrate with the Escrow Vault:

1. **Deploy the vault** with admin and ZKGM contract addresses
2. **Configure fungible lanes** for each token and channel pair
3. **Set counterparty beneficiary** to the corresponding vault on the other chain
4. **Optionally fund the vault** with tokens for filling orders
5. **Optionally whitelist intents** for pre-approved transfers

The vault will automatically participate as a solver in the ZKGM protocol, filling orders and maintaining fungibility between chains.
