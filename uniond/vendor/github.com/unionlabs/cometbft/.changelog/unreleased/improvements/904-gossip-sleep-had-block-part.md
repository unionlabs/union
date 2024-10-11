- `[consensus]` Optimize vote and block part gossip with new message `HasProposalBlockPartMessage`,
  which is similar to `HasVoteMessage`; and random sleep in the loop broadcasting those messages.
  The sleep can be configured with new config `peer_gossip_intraloop_sleep_duration`, which is set to 0
  by default as this is experimental.
  Our scale tests show substantial bandwidth improvement with a value of 50 ms.
  ([\#904](https://github.com/cometbft/cometbft/pull/904))