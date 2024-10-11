# Mempool

In this document, we define the notion of **mempool** and characterize its role in **CometBFT**.
First, we provide an overview of what is a mempool, and relate it to other blockchains.
Then, the interactions with the consensus and client application are detailed.
A formalization of the mempool follows.
This formalization is readable in Quint [here](./quint).

## Overview

The mempool acts as an entry point to consensus.
It permits to disseminate transactions from one node to another, for their eventual inclusion into a block.
To this end, the mempool maintains a replicated set, or _pool_, of transactions.
Transactions in the mempool are consumed by consensus to create the next proposed block.
Once a new block in the blockchain is decided, the mempool is refreshed.
We shall detail how shortly.

A transaction can be received from a local client, or a remote disseminating process.
Each transaction is subject to a test by the client application.
This test verifies that the transaction is _valid_.
Such a test provides some form of protection against byzantine agents, whether they be clients or other system nodes.
It also serves to optimize the overall utility of the blockchain.
Validity can be simply syntactical which is stateless, or a more complex verification that is state-dependent.
If the transaction is valid, the local process further propagates it in the system using a gossip (or an anti-entropy) mechanism.

_In other blockchains._
The notion of mempool appears in all blockchains, but with varying definitions and/or implementations.
For instance in Ethereum, the mempool contains two types of transactions: processable and pending ones.
To be pending, a transactions must first succeed in a series of tests.
Some of these tests are [syntactic](https://github.com/ethereum/go-ethereum/blob/281e8cd5abaac86ed3f37f98250ff147b3c9fe62/core/txpool/txpool.go#L581) ones (e.g., valid source address), while [others](https://github.com/ethereum/go-ethereum/blob/281e8cd5abaac86ed3f37f98250ff147b3c9fe62/core/txpool/txpool.go#L602) are state-dependent (e.g., enough gas, at most one pending transactions per address, etc).
[Narwhal](https://arxiv.org/abs/2105.11827.pdf) is the mempool abstraction for the Tusk and [Bullshark](https://arxiv.org/pdf/2201.05677) protocols.
It provides strong global guarantees.
In particular, once a transaction is added to the mempool, it is guaranteed to be available at any later point in time.

## Interactions

In what follows, we present the interactions of the mempool with other parts of CometBFT.
Some of the specificities of the current implementation (`CListMempool`) are also detailed.

**RPC server**
To add a new transaction to the mempool, a client submits it through an appropriate RPC endpoint.
This endpoint is offered by some of the system nodes (but not necessarily all of them).

**Gossip protocol** 
Transactions can also be received from other nodes, through a gossiping mechanism.

**ABCI application**
As pointed out above, the mempool should only store and disseminate valid transactions.
It is up to the [ABCI](./../abci/abci%2B%2B_basic_concepts.md#mempool-methods) (client) application to define whether a transaction is valid.
Transactions received locally are sent to the application to be validated, through the `checkTx` method from the mempool ABCI connection.
Such a check indicates with a flag whether it is the first time (or not) that the transaction is sent for validation.
Transactions that are validated by the application are later added to the mempool.
Transactions tagged as invalid are simply dropped.
The validity of a transaction may depend on the state of the client application.
In particular, some transactions that are valid in some state of the application may later become invalid.
The state of the application is updated when consensus commits a block of transactions.
When this happens, the transactions still in the mempool have to be validated again.
We further detail this mechanism below.

**Consensus**
The consensus protocol consumes transactions stored in the mempool to build blocks to be proposed.
To this end, consensus requests from the mempool a list of transactions.
A limit on the total number of bytes, or transactions, _may_ be specified.
In the current implementation, the mempool is stored as a list of transactions.
The call returns the longest prefix of the list that matches the imposed limits.
Notice that at this point the transactions returned to consensus are not removed from the mempool.
This comes from the fact that the block is proposed but not decided yet.

Proposing a block is the prerogative of the nodes acting as validators.
At all the full nodes (validators or not), consensus is responsible for committing blocks of transactions to the blockchain.
Once a block is committed, all the transactions included in the block are removed from the mempool.
This happens with an `update` call to the mempool.
Before doing this call, CometBFT takes a `lock` on the mempool.
Then, it `flush` the connection with the client application.
When `flush` returns, all the pending validation requests are answered and/or dropped.
Both operations aim at preventing any concurrent `checkTx` while the mempool is updated.
At the end of `update`, all the transactions still in the mempool are re-validated (asynchronously) against the new state of the client application.
This procedure is executed with a call to `recheckTxs`.
Finally, consensus removes its lock on the mempool by issuing a call to `unlock`.

## Formalization

In what follows, we formalize the notion of mempool.
To this end, we first provide a (brief) definition of what is a ledger, that is a replicated log of transactions.
At a process $p$, we shall write $p.var$ the local variable $var$ at $p$.

**Ledger.**
We use the standard definition of (BFT) SMR in the context of blockchain, where each process $p$ has a ledger, written $p.ledger$.
At process $p$, the $i$-th entry of the ledger is denoted $p.ledger[i]$.
This entry contains either a null value ($\bot$), or a set of transactions, aka., a block.
The height of the ledger at $p$ is the index of the first null entry; denoted $p.height$.
Operation $submit(txs, i)$ attempts to write the set of transactions $txs$ to the $i$-th entry of the ledger.
The (history) variable $p.submitted[i]$ holds all the transactions (if any) submitted by $p$ at height $i$.
By extension, $p.submitted$ are all the transaction submitted by $p$.
A transaction is committed when it appears in one of the entries of the ledger.
We denote by $p.committed$ the committed transactions at $p$.

As standard, the ledger ensures that:  
* _(Gap-freedom)_ There is no gap between two entries at a correct process:  
$\forall i \in 	\mathbb{N}. \forall p \in Correct. \square(p.ledger[i] \neq \bot \implies (i=0 \vee p.ledger[i-1] \neq \bot))$;  
* _(Agreement)_ No two correct processes have different ledger entries; formally:  
$\forall i \in 	\mathbb{N}. \forall p,q \in Correct. \square((p.ledger[i] = \bot) \vee (q.ledger[i] = \bot) \vee (p.ledger[i] = q.ledger[i]))$;  
* _(Validity)_ If some transaction appears at an index $i$ at a correct process, then a process submitted it at that index:  
$\forall p \in Correct. \exists q \in Processes. \forall i \in 	\mathbb{N}. \square(tx \in p.ledger[i] \implies tx \in \bigcup_q q.submitted[i]$).
* _(Termination)_ If a correct process submits a block at its current height, eventually its height get incremented:  
$\forall p \in Correct. \square((h=p.height \wedge p.submitted[h] \neq \varnothing) \implies \lozenge(p.height>h))$  

**Mempool.**
A mempool is a replicated set of transactions.
At a process $p$, we write it $p.mempool$.
We also define $p.hmempool$, the (history) variable that tracks all the transactions ever added to the mempool by process $p$.
Below, we list the invariants of the mempool (at a correct process).

Only the mempool is used as an input for the ledger:  
**INV1.** $\forall tx. \forall p \in Correct. \square(tx \in p.submitted \implies tx \in p.hmempool)$

Committed transactions are not in the mempool:  
**INV2.** $\forall tx. \forall p \in Correct. \square(tx \in p.committed \implies tx \notin p.mempool)$

In blockchain, a transaction is (or not) valid in a given state.
That is, a transaction can be valid (or not) at a given height of the ledger.
To model this, consider a transaction $tx$.
Let $p.ledger.valid(tx)$ be such a check at the current height of the ledger at process $p$ (ABCI call).
Our third invariant is that only valid transactions are present in the mempool:  
**INV3.** $\forall tx, \forall p \in Correct. \square(tx \in p.mempool \implies p.ledger.valid(tx))$

Finally, we require some progress from the mempool.
Namely, if a transaction appears at a correct process then eventually it is committed or forever invalid.  
**INV4** $\forall tx. \forall p \in Correct. \square(tx \in p.mempool \implies \lozenge\square(tx \in p.committed \vee \neg p.ledger.valid(tx)))$

The above invariant ensures that if a transaction enters the mempool (at a correct process), then it eventually leaves it at a later time.
For this to be true, the client application must ensure that the validity of a transaction converges toward some value.
This means that there exists a height after which $valid(tx)$ always returns the same value.
Such a requirement is termed _eventual non-oscillation_ in the [ABCI](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_app_requirements.md#mempool-connection-requirements) documentation.
It also appears in [Ethereum](https://github.com/ethereum/go-ethereum/blob/5c51ef8527c47268628fe9be61522816a7f1b395/light/txpool.go#L401) as a transaction is always valid until a transaction from the same address executes with the same or higher nonce.
A simple way to satisfy this for the programmer is by having $valid(tx)$ deterministic and stateless (e.g., a syntactic check).

**Practical considerations.**
Invariants INV2 and INV3 require to atomically update the mempool when transactions are newly committed.
To maintain such invariants in an implementation, standard thread-safe mechanisms (e.g., monitors and locks) can be used.

Another practical concern is that INV2 requires to traverse the whole ledger, which might be too expensive.
Instead, we would like to maintain this only over the last $\alpha$ committed transactions, for some parameter $\alpha$.
Given a process $p$, we write $p.lcommitted$ the last $\alpha$ committed transactions at $p$.
Invariant INV2 is replaced with:  
**INV2a.** $\forall tx. \forall p \in Correct. \square(tx \in p.lcommitted \implies tx \notin p.mempool)$

INV3 requires to have a green light from the client application before adding a transaction to the mempool.
For efficiency, such a validation needs to be made at most $\beta$ times per transaction at each height, for some parameter $\beta$.
Ideally, $\beta$ equals $1$.
In practice, $\beta = f(T)$ for some function $f$ of the maximal number of transactions $T$ submitted between two heights.
Given some transaction $tx$, variable $p.valid[tx]$ tracks the number of times the application was asked at the current height.
A weaker version of INV3 is as follows:  
**INV3a.** $\forall tx. \forall p \in Correct. \square(tx \in p.hmempool \implies p.valid[tx] \in [1, \beta])$

> For further information regarding the current implementation of the mempool in CometBFT, the reader may consult [this](https://github.com/cometbft/knowledge-base/blob/main/protocols/mempool/v0/mempool-v0.md) document in the knowledge base.
