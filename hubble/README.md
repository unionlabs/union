# Overview

Hubble is a multi-stage ETL indexer for various chains. Currently, it supports the:

- CosmosSDK
- EVM (HTTP)

## Architecture

Hubble has two distinct data objects:

- Logs: unparsed data obtained from archive nodes. Logs need further extraction before they are consumable.
- Events: JSON formatted consumable logs.

For certain chains, such as CosmosSDK-based chains, we can omit the log extraction, as they already produce JSON formatted events. For EVM-based chains, a conversion from ethabi to JSON is performed for specific contracts.

### Database Schema

Hubble uses the following tables:

- Logs: log storage for extraction, contains block and transaction data.
- Events: extracted events from logs.
- Blocks: extracted blocks from logs.
- Transactions: extracted transactions from logs.
- Chains: metadata on chains, created once on startup.
