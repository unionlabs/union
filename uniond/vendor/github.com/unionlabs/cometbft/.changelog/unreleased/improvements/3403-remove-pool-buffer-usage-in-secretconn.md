- `[p2p/conn]` Remove the usage of a synchronous pool of buffers in secret connection, storing instead the buffer in the connection struct. This reduces the synchronization primitive usage, speeding up the code.
  ([\#3403](https://github.com/cometbft/cometbft/issues/3403))
