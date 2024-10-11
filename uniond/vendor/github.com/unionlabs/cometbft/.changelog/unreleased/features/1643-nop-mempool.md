- `[mempool]` Add `nop` mempool ([\#1643](https://github.com/cometbft/cometbft/pull/1643)). If you want to use it, change mempool's `type` to `nop`:
```toml
      [mempool]

      # The type of mempool for this node to use.
      #
      # Possible types:
      # - "flood" : concurrent linked list mempool with flooding gossip protocol
      # (default)
      # - "nop"   : nop-mempool (short for no operation; the ABCI app is responsible
      # for storing, disseminating and proposing txs). "create_empty_blocks=false"
      # is not supported.
      type = "nop"
```
