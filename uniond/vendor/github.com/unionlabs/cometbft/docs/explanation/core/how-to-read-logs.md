---
order: 7
---

# How to read logs

## Walkabout example

We first create three connections (mempool, consensus and query) to the
application (running `kvstore` locally in this case).

```sh
I[10-04|13:54:27.364] Starting multiAppConn                        module=proxy impl=multiAppConn
I[10-04|13:54:27.366] Starting localClient                         module=abci-client connection=query impl=localClient
I[10-04|13:54:27.366] Starting localClient                         module=abci-client connection=mempool impl=localClient
I[10-04|13:54:27.367] Starting localClient                         module=abci-client connection=consensus impl=localClient
```

Then CometBFT and the application perform a handshake.

```sh
I[10-04|13:54:27.367] ABCI Handshake                               module=consensus appHeight=90 appHash=E0FBAFBF6FCED8B9786DDFEB1A0D4FA2501BADAD
I[10-04|13:54:27.368] ABCI Replay Blocks                           module=consensus appHeight=90 storeHeight=90 stateHeight=90
I[10-04|13:54:27.368] Completed ABCI Handshake - CometBFT and App are synced module=consensus appHeight=90 appHash=E0FBAFBF6FCED8B9786DDFEB1A0D4FA2501BADAD
```

After that, we start a few more things like the event switch and reactors.

```sh
I[10-04|13:54:27.374] Starting EventSwitch                         module=types impl=EventSwitch
I[10-04|13:54:27.375] This node is a validator                     module=consensus
I[10-04|13:54:27.379] Starting Node                                module=main impl=Node
I[10-04|13:54:27.381] Local listener                               module=p2p ip=:: port=26656
I[10-04|13:54:30.386] Starting DefaultListener                     module=p2p impl=Listener(@10.0.2.15:26656)
I[10-04|13:54:30.387] Starting P2P Switch                          module=p2p impl="P2P Switch"
I[10-04|13:54:30.387] Starting MempoolReactor                      module=mempool impl=MempoolReactor
I[10-04|13:54:30.387] Starting BlockchainReactor                   module=blockchain impl=BlockchainReactor
I[10-04|13:54:30.387] Starting ConsensusReactor                    module=consensus impl=ConsensusReactor
I[10-04|13:54:30.387] ConsensusReactor                             module=consensus fastSync=false
I[10-04|13:54:30.387] Starting ConsensusState                      module=consensus impl=ConsensusState
I[10-04|13:54:30.387] Starting WAL                                 module=consensus wal=/home/vagrant/.cometbft/data/cs.wal/wal impl=WAL
I[10-04|13:54:30.388] Starting TimeoutTicker                       module=consensus impl=TimeoutTicker
```

Notice the second row where CometBFT reports that "This node is a
validator". It also could be just an observer (regular node).

Next we replay all the messages from the WAL.

```sh
I[10-04|13:54:30.390] Catchup by replaying consensus messages      module=consensus height=91
I[10-04|13:54:30.390] Replay: New Step                             module=consensus height=91 round=0 step=RoundStepNewHeight
I[10-04|13:54:30.390] Replay: Done                                 module=consensus
```

"Started node" message signals that everything is ready for work.

```sh
I[10-04|13:54:30.391] Starting RPC HTTP server on tcp socket 0.0.0.0:26657 module=rpc-server
I[10-04|13:54:30.392] Started node                                 module=main nodeInfo="NodeInfo{id: DF22D7C92C91082324A1312F092AA1DA197FA598DBBFB6526E, moniker: anonymous, network: test-chain-3MNw2N [remote , listen 10.0.2.15:26656], version: 0.11.0-10f361fc ([wire_version=0.6.2 p2p_version=0.5.0 consensus_version=v1/0.2.2 rpc_version=0.7.0/3 tx_index=on rpc_addr=tcp://0.0.0.0:26657])}"
```

Next follows a standard block creation cycle, where we enter a new
round, propose a block, receive more than 2/3 of prevotes, then
precommits and finally have a chance to commit a block. For details,
please refer to [Byzantine Consensus Algorithm](https://github.com/cometbft/cometbft/blob/main/spec/consensus/consensus.md).

```sh
I[10-04|13:54:30.393] enterNewRound(91/0). Current: 91/0/RoundStepNewHeight module=consensus
I[10-04|13:54:30.393] enterPropose(91/0). Current: 91/0/RoundStepNewRound module=consensus
I[10-04|13:54:30.393] enterPropose: Our turn to propose            module=consensus proposer=125B0E3C5512F5C2B0E1109E31885C4511570C42 privValidator="PrivValidator{125B0E3C5512F5C2B0E1109E31885C4511570C42 LH:90, LR:0, LS:3}"
I[10-04|13:54:30.394] Signed proposal                              module=consensus height=91 round=0 proposal="Proposal{91/0 1:21B79872514F (-1,:0:000000000000) {/10EDEDD7C84E.../}}"
I[10-04|13:54:30.397] Received complete proposal block             module=consensus height=91 hash=F671D562C7B9242900A286E1882EE64E5556FE9E
I[10-04|13:54:30.397] enterPrevote(91/0). Current: 91/0/RoundStepPropose module=consensus
I[10-04|13:54:30.397] enterPrevote: ProposalBlock is valid         module=consensus height=91 round=0
I[10-04|13:54:30.398] Signed and pushed vote                       module=consensus height=91 round=0 vote="Vote{0:125B0E3C5512 91/00/1(Prevote) F671D562C7B9 {/89047FFC21D8.../}}" err=null
I[10-04|13:54:30.401] Added to prevote                             module=consensus vote="Vote{0:125B0E3C5512 91/00/1(Prevote) F671D562C7B9 {/89047FFC21D8.../}}" prevotes="VoteSet{H:91 R:0 T:1 +2/3:F671D562C7B9242900A286E1882EE64E5556FE9E:1:21B79872514F BA{1:X} map[]}"
I[10-04|13:54:30.401] enterPrecommit(91/0). Current: 91/0/RoundStepPrevote module=consensus
I[10-04|13:54:30.401] enterPrecommit: +2/3 prevoted proposal block. Locking module=consensus hash=F671D562C7B9242900A286E1882EE64E5556FE9E
I[10-04|13:54:30.402] Signed and pushed vote                       module=consensus height=91 round=0 vote="Vote{0:125B0E3C5512 91/00/2(Precommit) F671D562C7B9 {/80533478E41A.../}}" err=null
I[10-04|13:54:30.404] Added to precommit                           module=consensus vote="Vote{0:125B0E3C5512 91/00/2(Precommit) F671D562C7B9 {/80533478E41A.../}}" precommits="VoteSet{H:91 R:0 T:2 +2/3:F671D562C7B9242900A286E1882EE64E5556FE9E:1:21B79872514F BA{1:X} map[]}"
I[10-04|13:54:30.404] enterCommit(91/0). Current: 91/0/RoundStepPrecommit module=consensus
I[10-04|13:54:30.405] Finalizing commit of block with 0 txs        module=consensus height=91 hash=F671D562C7B9242900A286E1882EE64E5556FE9E root=E0FBAFBF6FCED8B9786DDFEB1A0D4FA2501BADAD
I[10-04|13:54:30.405] Block{
  Header{
    ChainID:        test-chain-3MNw2N
    Height:         91
    Time:           2017-10-04 13:54:30.393 +0000 UTC
    NumTxs:         0
    LastBlockID:    F15AB8BEF9A6AAB07E457A6E16BC410546AA4DC6:1:D505DA273544
    LastCommit:     56FEF2EFDB8B37E9C6E6D635749DF3169D5F005D
    Data:
    Validators:     CE25FBFF2E10C0D51AA1A07C064A96931BC8B297
    App:            E0FBAFBF6FCED8B9786DDFEB1A0D4FA2501BADAD
  }#F671D562C7B9242900A286E1882EE64E5556FE9E
  Data{

  }#
  Commit{
    BlockID:    F15AB8BEF9A6AAB07E457A6E16BC410546AA4DC6:1:D505DA273544
    Precommits: Vote{0:125B0E3C5512 90/00/2(Precommit) F15AB8BEF9A6 {/FE98E2B956F0.../}}
  }#56FEF2EFDB8B37E9C6E6D635749DF3169D5F005D
}#F671D562C7B9242900A286E1882EE64E5556FE9E module=consensus
I[10-04|13:54:30.408] Executed block                               module=state height=91 validTxs=0 invalidTxs=0
I[10-04|13:54:30.410] Committed state                              module=state height=91 txs=0 hash=E0FBAFBF6FCED8B9786DDFEB1A0D4FA2501BADAD
I[10-04|13:54:30.410] Recheck txs                                  module=mempool numtxs=0 height=91
```

## List of modules

Here is the list of modules you may encounter in CometBFT's log and a
little overview what they do.

- `abci-client` As mentioned in [Application Development Guide](../../guides/app-dev/abci-cli.md), CometBFT acts as an ABCI
  client with respect to the application and maintains 3 connections:
  mempool, consensus and query. The code used by CometBFT can
  be found [here](https://github.com/cometbft/cometbft/blob/main/abci/client).
- `blockchain` Provides storage, pool (a group of peers), and reactor
  for both storing and exchanging blocks between peers.
- `consensus` The heart of CometBFT, which is the
  implementation of the consensus algorithm. Includes two
  "submodules": `wal` (write-ahead logging) for ensuring data
  integrity and `replay` to replay blocks and messages on recovery
  from a crash.
- `events` Simple event notification system. The list of events can be
  found
  [here](https://github.com/cometbft/cometbft/blob/main/types/events.go).
  You can subscribe to them by calling `subscribe` RPC method. Refer
  to [RPC docs](rpc.md) for additional information.
- `mempool` Mempool module handles all incoming transactions, whenever
  they are coming from peers or the application.
- `p2p` Provides an abstraction around peer-to-peer communication. For
  more details, please check out the
  [README](https://github.com/cometbft/cometbft/blob/main/p2p/README.md).
- `rpc` [CometBFT's RPC](rpc.md).
- `rpc-server` RPC server. For implementation details, please read the
  [doc.go](https://github.com/cometbft/cometbft/blob/main/rpc/jsonrpc/doc.go).
- `state` Represents the latest state and execution submodule, which
  executes blocks against the application.
- `types` A collection of the publicly exposed types and methods to
  work with them.
