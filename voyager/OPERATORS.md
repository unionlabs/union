# Running Voyager

## Requirements

- Linux
- PostgreSQL

Voyager uses a postgres database for it's internal message queue. The database will be initialized with the necessary tables on startup.

## Setup

To initialize a config:

```sh
voyager config default
```

This will print a skeleton of a config to a terminal. You can also look at our [devnet config](https://github.com/unionlabs/union/blob/main/voyager/devnet-config.json) for examples.

The config also supports a JSON Schema, which is hosted at <https://zkgm.uno/voyager/config.schema.json>. To generate a local copy:

```sh
voyager config schema
```
