# Zerg

Zerg is a stress test, fuzzer, benchmarker, for Voyager.

## Features

Zerg has three main features `rush`, `process`, and `analyze`.

### Rush

`zerg rush` will spawn one transaction per account per block. The number of blocks zerg does this for is configured via the Zerg configuration file. Once transfers are received on the other side of the bridge, Zerg will begin conducting round trip transactions.

The Zerg configuration file must also contain the private keys of accounts you wish to be spawning transactions from.

Zerg will then observe and record data about these transactions in the `output.csv`.

Each line of the CSV will be of the following format:

```csv
<uuid>,<address>,<execution_timestamp>,<finalization_timestamp>,<event_type>,<chain_id>
```

Where `uuid` is constructed in the following form:

```
<src_port>/<src_channel>/<sequence>
```

and `EVENT_TYPE` is one of:

```
SentFrom | ReceivedOn
```

### Process

After `zerg rush` or `zerg observe` outputs its transaction indexing, you can use `zerg process` to structure the data into a report CSV that will contain lines of the following form:

```csv
<uuid>,<src>,<executed_at?>,<execution_duration?>,<finalized_at?>,<finalization_duration?>
```

### Analyze

After `rush` and `process`, `zerg analyze` can be used to create useful statistics about a Zerg rush.

Analyze will output its statistics in CSV to the following format:

```csv
<mean_execution_duration>,<median_execution_duration>,<max_execution_duration>,<min_execution_duration>,<mean_finalization_duration>,<median_finalization_duration>,<max_finalization_duration>,<min_finalization_duration>,<incomplete_transfers>,<complete_transfers>,<from_chain_id>
```

## Configuration

A sample configuration for our devnet can be found in the root of this repository as `zerg-config.json`.
