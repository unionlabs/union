# Zerg

Zerg is a stress test, fuzzer, benchmarker, for Voyager.

## Features

Zerg has two main features `rush` and `observe`.

### Rush

`zerg rush` will spawn one transaction per account per block. The number of blocks zerg does this for is configured via the zerg configuration file.

The Zerg configuration file must also contain the private keys of accounts you wish to be spawning transactions from.

Zerg will then observe and record data about these transactions in the `output.csv`.

Each line of the CSV will be of the following format:
```csv
<uuid>, <address>, <timestamp>, <EVENT_TYPE>, <chain_id>
```

Where `uuid` is constructed in the following form:
```
<src_port>/<src_channel>/<sequence>
```

and `EVENT_TYPE` is one of:
```
SentFrom | RReceivedOn
```

### Observe

Like `zerg rush`, `zerg observe` will benchmark transactions from Voyager. However, it will not spawn transactions of its own.

## Configuration

A sample configuration for our devnet can be found in the root of this repository as `zerg-config.json`.
