# Indexer Framework

# Defining an Indexer

Indexer implementations should be registered with the `indexer.Register` function with a unique type name. Indexers take the configuration options defined by `indexer.Config` which defines a common set of configuration options as well as indexer-specific options under the `config` sub-key. Indexers do not need to manage the common filtering options specified in `Config` - the indexer manager will manage these for the indexer. Indexer implementations just need to return a correct `InitResult` response.

# Integrating the Indexer Manager

The indexer manager should be used for managing all indexers and should be integrated directly with applications wishing to support indexing. The `StartManager` function is used to start the manager. The configuration options for the manager and all indexer targets should be passed as the ManagerOptions.Config field and should match the json structure of ManagerConfig. An example configuration section in `app.toml` might look like this:

```toml
[indexer.target.postgres]
type = "postgres"
config.database_url = "postgres://user:password@localhost:5432/dbname"
```
