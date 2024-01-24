# App

The app module of `uniond` initializes the structure and tooling of the `uniond` node.

## Design

The structure of our app module is based off the app_v1 design from the cosmos-sdk simapp.

### Custom Query

The `custom_query` submodule is used for native BLS aggregation and verification of custom queries from light clients.

### IBC

The `ibc` submodule contains a keeper and functions used for maintaining the client and consensus state for IBC connections.

### Params

The `params` submodule contains default parameters used when initializing the application.

### Upgrades

The `upgrades` submodule contains runtime migrations used while upgrading the network.
