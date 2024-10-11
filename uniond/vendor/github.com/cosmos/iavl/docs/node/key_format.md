# Key Format

Nodes and fastNodes are stored under the database with different key formats to ensure there are no key collisions and a structured key from which we can extract useful information.

## Nodes

Node KeyFormat: `n|node.nodeKey.version|node.nodeKey.nonce`

Nodes are marshalled and stored under nodekey with prefix `n` to prevent collisions and then appended with the node's hash.

### FastNodes

FastNode KeyFormat: `f|node.key`

FastNodes are marshalled nodes stored with prefix `f` to prevent collisions. You can extract fast nodes from the database by iterating over the keys with prefix `f`.
