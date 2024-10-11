- `[p2p]` Remove `p2p_peer_send_bytes_total` and `p2p_peer_receive_bytes_total`
  metrics as they are costly to track, and not that informative in debugging
  ([\#3184](https://github.com/cometbft/cometbft/issues/3184))