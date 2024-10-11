# RFC 26: Banning peers based on ResponseCheckTx

## Changelog
- Nov 4,  2022: Initial draft (jmalicevic)
- Nov 8,  2022: Updated draft (jmalicevic)
- Nov 11, 2022: Updated based on PR comments (jmalicevic)
- Nov 14, 2022: Updated current peer banning mechanisms (jmalicevic)

## Abstract

In Tendermint, nodes receive transactions either from external clients via RPC,
or from their peers via p2p. Upon receiving a transaction, a node runs `CheckTx` on it. This is 
an application specific check whose return code with a zero value indicates the transaction
has passed this check, and can be added into the mempool. Any non-zero code indicates the transaction
is not valid. Thus, the main role of `CheckTx` is to, as early as possible, prevent invalid transactions
from entering the mempool. 

Note that there are valid scenarios in which a transaction will not
pass this check (including nodes getting different transactions at different times, meaning some
of them might be obsolete at the time of the check; state changes upon block execution etc.). 
However, Tendermint users observed that
there are transactions that can never have been or will never be valid. They thus propose
to introduce a special response code for `CheckTx` to indicate this behaviour, and ban the peers
who gossip such transactions. Additionally, users expressed a need for banning peers who
repeatedly send transactions failing `CheckTx`. 

The main goal of this document is to analyse the cases where peers could be banned when they send 
transactions failing `CheckTx`, and provide the exact conditions that a peer and transaction have
to satisfy in order to mark the peer as bad.

This document will also include a proposal for implementing these changes within the mempool, including
potential changes to the existing mempool logic and implementation.


## Background

This work was triggered by issue [#7918](https://github.com/tendermint/tendermint/issues/7918) and a related
discussion in [#2185](https://github.com/tendermint/tendermint/issues/2185). Additionally,
there was a [proposal](https://github.com/tendermint/tendermint/issues/6523) 
to disconnect from peers after they send us transactions that constantly fail `CheckTx`.  While 
the actual implementation of an additional response code for `CheckTx` is straight forward there 
are certain correctness aspects to consider. The questions to answer, along with identified risks will be outlined in 
the discussion. 

### Existing issues and concerns

Before diving into the details, we collected a set of issues opened by various users, arguing for 
this behaviour and explaining their needs. 

- Celestia: [blacklisting peers that repeatedly send bad tx](https://github.com/celestiaorg/celestia-core/issues/867)
  and investigating how [Tendermint treats large Txs](https://github.com/celestiaorg/celestia-core/issues/243)
- BigChainDb: [Handling proposers who propose bad blocks](https://github.com/bigchaindb/BEPs/issues/84)
- IBC relayers: Nodes allow transactions with a wrong `minGas` transaction and gossip them, and other nodes keep rejecting them. (Problem seen with relayers)

**Acceptable duplicate transactions**

Banning peers was also mentioned within [IBC-go](https://github.com/cosmos/ibc-go/issues/853#issuecomment-1032211020). However,the crux of the issue is preventing transactions with duplicate payload. While this is indeed undesired behaviour, this is not considered behaviour that should lead to banning a peer or even disconnecting from him. Duplicate transactions are in this case prevented using an application-specific solution. 

### Current state of mempool/p2p interaction

Transactions received from a peer are handled within the [`Receive`](https://github.com/tendermint/tendermint/blob/ff0f98892f24aac11e46aeff2b6d2c0ad816701a/mempool/v0/reactor.go#L158) routine. 

Currently, the mempool triggers a disconnect from a peer in the case of the following errors:

  - [Unknown message type](https://github.com/tendermint/tendermint/blob/ff0f98892f24aac11e46aeff2b6d2c0ad816701a/mempool/v0/reactor.go#L184)

However, disconnecting from a peer is not the same as banning the peer. The p2p layer will close the connection but 
the peer can reconnect without any penalty, and if it as a persistent peer, a reconnect will be initiated
from the node. 

### Current support for peer banning

The p2p layer implements banning peers by marking them
as bad and removing them from the list of peers to connect to for *at least* a predefined amount of time. This is done by calling the `MarkBad` routine implemented by the `Switch`. If the node does not set the amount of time to be banned, a default value is used. 
Note that the timing parameter sets the lower bound for when a peer will be unbanned. 
But the p2p layer will only try to connect to banned peers if the node is not sufficiently connected. Thus the node has no
explicit control on when a reconnect attempt will be triggered.

The application can blacklist peers via ABCI if the 
[`filterPeers`](../../spec/abci/abci%2B%2B_app_requirements.md#peer-filtering) 
config flag is set, by providing a set of peers to ban to Tendermint. 

If the discussion in this RFC deems a different banning mechanism is needed,
the actual implementation and design of this mechanism will be discussed in a separate RFC.
This mechanism should be generic, designed within the p2p layer and simply provide an interface
for reactors to indicate peers to ban and for how long. It should not involve any mempool
specific design considerations. 

## Discussion

If this feature is to be implemented we need to clearly define the following:
1. What does banning a peer mean:
   1. A peer can be simply disconnected from. 
   2. Peer is disconnected from and banned.
   3. Conditions for the peer to be banned.
2. If `CheckTx` signals a peer should be banned, retrieve the ID of peers to ban.  
3. Are there possible attack scenarios or unexpected behaviours by allowing this. 

Any further mentions of `banning` will be agnostic to the actual way banning is implemented by the p2p layer. 


### 1. What does banning a peer mean 

Tendermint recognizes that peers can accept transactions into their mempool as valid but then when the state changes, they can become invalid. 
There are also transactions that are received that could never have been valid (for example due to misconfiguration on one node). 
We thus differentiate two scenarios - a) where `CheckTx` fails due to reasons already 
known and b) where `CheckTx` deems a transaction could never have been valid.

For the sake of simplicity , in the remainder of the text we will distinguish the failures due to a) as failures 
signaled with `ResponseCheckTx.code = 1` and the failures described in b), failures with `ResponseCheckTx.code > 1`, even though 
the way we actually mark them in the end might differ.

For a), a peer sends transactions that **repeatedly** fail CheckTx with `ResponseCheckTx.code = 1`, and is banned or disconnected from to avoid this. 
In this case we need to define what repeatedly means. 

For b) we need to understand what is the potential reason a transaction could never have been valid on one node, but passes `CheckTx` on another node.  
We need to understand all the possible scenarios in which this can happen:

1. What happens if a node is misconfigured and allows, for example, very large transactions into the mempool. 
This node would then gossip these transactions and they would always fail on other nodes. 
Is this a scenario where we want nodes to disconnect from this peer and ban it but do not consider it malicious?
2. Are all other reasons for this to happen sign of malicious behaviour where a node explicitly lies? How can `CheckTx` pass on a valid node, but fail on another valid node with a `ResponseCheckTx.code > 1`?
If such behaviour is only possible when a peer is malicious, should this peer be punished or banned forever? Note that 
we cannot know whether a node is a validator in order for it to be punished. Gossiping this behaviour to other peers pro-actively
also entails a different set of problems with it - how do we know we can trust peers who tell us to ban other peers. For these reasons, understanding the actual reason for these failures can be left for future work. 

For now, we will disconnect and ban the peer regardless of the exact reason a transaction 
is considered to never be valid. 

#### **Banning for frequent CheckTx failures**

If a node sends transactions that fail `CheckTx` but could be valid at some point, a peer should not be banned the first time this happens.
Only if this happens frequently enough should this be considered as spam. To define this behaviour we keep track how many times (`numFailures`) a peer 
sent us invalid transactions within a time interval (`lastFailure`). This time interval should be reset every `failureResetInterval`ms.  

For each peer, we should have a separate `numFailures` and `lastFailure` variable. There is no need to  have one per transaction.
 Whenever a transaction fails, if the `now - lastFailure <= failureResetInterval`, we increment the `numFailures` for this particular peer and set the `lastFailure` to `now`. 
 Otherwise, we set `lastFailure` to `now` and set `numFailures`  to 1. 
 Once the value for `numFailures` for a peer reaches `maxAllowedFailures`, the peer is disconnected from and banned. 

The reason for this logic is as follows: We deem it acceptable if every now and then a peer sends us an invalid transaction.
 But if this happens very frequently, then this behaviour can be considered as spamming and we want to disconnect from the peer. 
  
 **Discussion**

 The problem with supporting this scenario is the definition of the above mentioned parameters. It is very hard to estimate, at the Tendermint level, what these parameters should be. A possible solution is 
 to allow the application to set these parameters. What is unclear, how will the application know that these parameters are not well set if, due to a bug or network problems, transactions start to fail? 
The network could end up with all nodes banning everyone. How would an application developer know to debug this, what to look for?

A possible solution is to ban peers temporarily. In addition to the question on how long is temporarily, setting specific time limits for banning on a peer basis
is currently not supported by the p2p layer.

*Banning a peer in case of duplicate transactions*

Currently, a peer can send the same valid (or invalid) transaction [multiple times](https://github.com/tendermint/tendermint/blob/ff0f98892f24aac11e46aeff2b6d2c0ad816701a/mempool/v0/clist_mempool.go#L247). Peers do not 
gossip transactions to peers that have sent them that same transaction. But there is no check on whether 
a node has already sent the same transaction to this peer before. There is also no check whether the transaction
that is being gossiped is currently valid or not (assumint that invalid transactions could become valid).
The transaction broadcast logic simply loops through the mempool and tries to send the transactions currently in the pool. 

If we want to ban peers based on duplicate transactions, we should either add additional checks for the cases above, or 
not ban peers for this behaviour at the moment. It would be useful to gather metrics on how often a peer gossips the same 
transaction and whether this is cause of significant traffic. 


#### **Banning for sending *never-valid* transactions**

If a transaction fails since it could never have been valid, `CheckTx` returns a `ResponseCheckTx.code` 
value greater than 1. In this case, the peer should be disconnected from and banned immediately without keeping count on how often 
this has happened. 

The question is whether this transaction should be kept track of in the cache? We can still store it in 
the cache so that we don't run `CheckTx` on it again, but if this peer is immediately banned, maybe there is no need
to store its information.

Now, if we want to differentiate further reasons of why this transaction is sent to a node (whether it is a sign of malice or not),
we might need more information on the actual reason for rejection. This could be done by an additional set of response codes provided by the application. 

### 2. Choosing the peer to ban

Each transaction gossiped contains the ID of the peer that sent that transaction. Upon receiving a transaction,
a node saves the peer ID of the peer(s) that have sent it. As each peer had to have
run `CheckTx` on this transaction before adding it to its own mempool, we can assume this peer
can be held accountable for the validity of transactions it gossips. Invalid transactions are kept only in the mempool
cache and thus not gossiped. 
As nodes have to complete a cryptographic handshake at the p2p layer, Tendermint guarantees that a malicious peer
cannot lie about who the sender of the transaction is. 

*Transactions received from users*

For transactions submitted via `broadcastTxCommit`, the `SenderID` field is empty. 

**Note**  Do we have mechanisms in place to handle cases when `broadcastTxCommit` submits
failing transactions (can this be a form of attack)?

### 3. Attack scenarios

While an attack by simply banning peers on failing `CheckTx` is hard to imagine, as the incentive for doing so is not clear, there are considerations with regards to the current mempool gossip implementation.

Should we keep transactions that could never have been valid in the cache? Assuming that receiving such transactions is rare, and the peer that sent them is banned, do we need to occupy space in the mempool cache with these transactions?

- What if nodes run different versions of Tendermint and banning is not supported in one of the versions? 

- Reserving response codes can be problematic for existing applications that may have reserved these codes for internal purposes withtou being aware that this causes a ban now. 
 

## Implementation considerations

**Indicating a new type of `CheckTx` failure**

The initial proposal is to reserve a special response code to indicate that the transaction could never have been valid. 
Due to concerns of this being a breaking change for applications that have already reserved this code for internal
purposes, there is an alternative implementation: expanding `ResponseCheckTx` with an additional field. 
This field `neverValidTx` would be `false` by default. If a transaction could never have been valid,
in addition to indicating this with a non-zero response code from `CheckTx`, the application would set this field value. 

**Adding support for peer banning**

When a transaction fails `CheckTx`, it is not stored in the mempool but **can** be stored in the cache. If it is in the cache, it cannot be resubmitted again (as it will be discovered in the cache and not checked again). These two scenarios require a different implementation of banning in case `CheckTx` failed. 

In both cases we need to keep track of the peers that sent invalid transactions. If invalid transactions are cached, 
we also need to keep track of the `CheckTx` response code for each transaction. Currently the `ResponseCheckTx` code is checked in `resCbFirstTime` of the mempool. 
If invalid transactions are kept in the cache, the check is ran only when a transaction is 
seen for the first time. Afterwards, the  transaction is cached, to avoid running `CheckTx` on transactions already checked. 
Thus when a transaction is received from a peer, if it is in the cache,
`CheckTx` is not ran again, but the peers' ID is added to the list of peers who sent this particular transaction.
These transactions are rechecked once a block is committed to verify that they are still valid. 

If invalid transactions are not kept in the cache, they can be resubmitted multiple times, and `CheckTx` will be executed on them upon submission. 
Therefore we do not need to remember the previous response codes for these transactions.

In summary, if we want to support banning peers based on the frequency with which they submit invalid transactions, we need to have **additional datastructures**:
1. One to keep track of past invalid transactions
2. A datastructure to differentiate between valid and invalid *cached* transactions. If the `KeepInvalidTxsInCache` configuration parameter is not set, this datastructure
   is not needed. 

We propose two ways to implement peer banning based on the result of `CheckTx`:

1. Introduce banning when transactions are received
2. Adapt the recheck logic to support this



**Peer banning when transactions are received**

If a transaction fails `CheckTx` the [first time it is seen](https://github.com/tendermint/tendermint/blob/ff0f98892f24aac11e46aeff2b6d2c0ad816701a/mempool/v0/clist_mempool.go#L409),  the peer can be banned right there:

>>mempool/v0/clist_mempool.go#L409
```golang

if (r.CheckTx.Code == abci.CodeTypeOK) && postCheckErr == nil {
    // Check Tx passed
} else {
// ignore bad transaction
    mem.logger.Debug(
      "rejected bad transaction",
      "tx", types.Tx(tx).Hash(),
      "peerID", peerP2PID,
      "res", r,
      "err", postCheckErr,
    )
    mem.metrics.FailedTxs.Add(1)

    mem.banPeer(peerP2PID)

    if !mem.config.KeepInvalidTxsInCache {
      // remove from cache (it might be good later)
      mem.cache.Remove(tx)
    } else {
      // If transactins stay in the cache, remember they failed
      mem.cache.invalidCachedTx.Store(tx.Key(), true)
    }
}

```					
The `KeepInvalidTxsInCache` configuration parameter defines whether an invalid transaction stays in cache. For *never-valid*
transactions, we could apply a different approach based on what we deem to be the bigger gain:
- As we do not expect to receive frequently and from many peers, and we ban the peer that sent it immediately, we do not store it in the cache to save space. This would mean
that if we did see it again, we'd ran `CheckTx` on it again. 

```golang
if !mem.config.KeepInvalidTxsInCache || r.CheckTx.Code == abci.NeverValid {
  // remove from cache (it might be good later or is never valid, we'll most likely don't see it again)
    mem.cache.Remove(tx)
}
```

- We do keep it in the cache as long as possible to avoid running `CheckTX` on it because we know, for sure, that it will never be valid. As it is rare enough, it 
might not take that much space. In this case though, as we ban the sending peer immediately, we can save space by not storing peer information for this transaction. 

The question is which one is more costly, doing `CheckTx` more then once, or keeping an extra entry in the cache?



As said, this code will [never be executed](https://github.com/tendermint/tendermint/blob/ff0f98892f24aac11e46aeff2b6d2c0ad816701a/mempool/v0/clist_mempool.go#L239) for transactions whose signature is found
in the cache. 
Instead of remembering the cached transactions, we could have had a valid/invalid bit per transaction within the cache. As transactions themselves do not
store such information and we expect this scenario to be unlikely, instead of increasing the footprint of all transactions in the cache,
we opted to keep a map of transaction signature if the transaction is in the cache, but is invalid. Alternatively, the cache could keep two lists, one for valid, and one for invalid transactions. 
This modifies the following pieces of code as follows (this is just a prototype and does not include 
some obvious sanity checks):

```golang

// New datastructure to keep track of peer failures
type PeerFailure struct {
  lastFailure time.Time
  numFailures int8
}

type LRUTxCache struct {
// Keeping track of invalid transactions within the cache ;
  invalidCachedTx map[types.TxKey]bool
}

type CListMempool struct {
 // ..existing fields
  peerFailureMap map[nodeID]*PeerFailure

}
```

>mempool/v0/clist_mempool.go#L239
```golang

if !mem.cache.Push(tx) { // if the transaction already exists in the cache
  // Record a new sender for a tx we've already seen.
  // Note it's possible a tx is still in the cache but no longer in the mempool
  // (eg. after committing a block, txs are removed from mempool but not cache),
  // so we only record the sender for txs still in the mempool.
  if e, ok := mem.txsMap.Load(tx.Key()); ok {
    memTx := e.(*clist.CElement).Value.(*mempoolTx)
    memTx.senders.LoadOrStore(txInfo.SenderID, true)
    // TODO: consider punishing peer for dups,
    // its non-trivial since invalid txs can become valid,
    // but they can spam the same tx with little cost to them atm.
  }

  // If transaction was invalid, we need to remember the peer information
  if _, ok := mem.cache.invalidCachedTx.Load(tx.Key); ok {
      mem.banPeer(peerID)  
  }
  return mempool.ErrTxInCache
}

```


```golang

func (mem* ClistMempool) banPeer(peerID NodeID) {
    numFails := 0
      // Check whether this peer has sent us transactions that fail
    if val, ok := mem.peerFailureMap[peerID]; ok {
      lastFailureT := val.lastFailure
      numFails = val.numFails
    // if the failure was recent enough, update the number of failures and 
    // ban peer if applicable
    if time.Since(lastFailureT) <= failureResetInterval {
      if numFails == maxAllowedFailures - 1 {
          // Send Ban request to p2p 
        }
      }
    }
    // Update the time of the last failure     
    mem.peerFailureMap[peerID] = { time.Now(), numFailures + 1}
}
```
If transactions with `ResponseCheckTx.code > 1` are not deleted from the cache and we want to ban them on the first offence,
we can skip the second `if` in the code above but call `banPeer` immediately at the end of the function. The function `banPeer` should simply forward the `peerID` to the p2p layer. In fact, we do not need to store any information on this peer as the node will remove it from its peer set.

**Signaling the ban to p2p**

As it is the mempool **reactor** that has access to the p2p layer, not the actual mempool implementation, the peer banning function will most likely have to send the peer ID to a channel to inform the reactor of the fact that this peer should be banned. This would require adding a channel for communication into the `CListMempool` on construction, and adding a routine in the mempool reactor that waits on a response via this channel. However, the actual implementation of this is yet to be defined.

**Implementing peer banning on recheck**

Currently the recheck logic confirmes whether once **valid** transactions, 
, where `ResponseCheckTx.code == 0`, are still valid. 

As this logic loops through the transactions in any case, we can leverage it to check whether we can ban peers. 
However, this approach has several downsides:
- It is not timely enough. Recheck is executed after a block is committed, leaving room for a bad peer to send 
us transactions the entire time between two blocks. 
- If we want to keep track of when peers sent us a traansaction and punish them only if the misbehaviour happens 
frequently enough, this approach makes it hard to keep track of when exactly was a transaction submitted. 
- Rechecking if optional and node operators can disable it. 
- Furthermore, rechecking is a node local configuration parameter. This means that, some nodes might be performing this check,
  while others will be unaware of this. 

On the plus side this would avoid adding new logic to the mempool caching mechanism and keeping additional information about
transaction validity. But we would still have to keep the information on peers and the frequency at which they send us bad transactions.

Transactions that became invalid on recheck should not be cause for peer banning as they have not been gossiped as invalid transactions. 

#### `PreCheck` and`PostCheck`

The `PreCheck` and `PostCheck` functions are optional functions that can be executed before or after `CheckTx`. 
Following the design outlined in this RFC, their responses are not considered for banning. 


#### Checks outside `CheckTx`

There are a number of checks that the mempool performs on the transaction, that are not part of `CheckTx` itself. 
Those checks, have been mentioned in the user issues described at the beginning of this document:

- Transaction size
- Proto checks
- Receiving unknown messages via the mempool channel

The previous code snippets do not incorporate these in peer banning. If we adopt those as valid reasons for banning, we should put the corresponding logic in place. 

### Impacted mempool functionalities

- Mempool caching: remembering failed transactions and whether they come from banned peers; Removal of transactions from 
  `invalidCachedTx` when a transaction is removed from cache.
- Handling of transactions failing `CheckTx`: Keeping track of how often transactions from a particular peer have failed
  and banning them if the conditions for a ban are met.

### Impact on ABCI 

- Introduction of new response codes for CheckTx. As previously noted, this might break existing applications if they reserved codes for internal purposes. 
- Altering the specification to reflect this change

### References


> Links to external materials needed to follow the discussion may be added here.
>
> In addition, if the discussion in a request for comments leads to any design
> decisions, it may be helpful to add links to the ADR documents here after the
> discussion has settled.



- 