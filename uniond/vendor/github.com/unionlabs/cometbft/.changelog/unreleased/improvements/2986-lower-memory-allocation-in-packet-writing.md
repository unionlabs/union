- `[p2p/conn]` Speedup connection.WritePacketMsgTo, by reusing internal buffers rather than re-allocating.
  ([\#2986](https://github.com/cometbft/cometbft/pull/2986))