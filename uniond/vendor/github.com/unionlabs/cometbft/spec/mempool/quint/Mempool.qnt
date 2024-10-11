// -*- mode: Bluespec; -*-
// A mempool is a replicated set of transactions which is used as an input by a ledger.
// Below, we follow the specification given here:
// https://github.com/cometbft/knowledge-base/blob/main/protocols/mempool-overview.md

module ABCI {

    import System.* from "System"
    import Ledger.* from "Ledger"

    pure def isValid(l: LedgerState, p: Process, t: Tx): bool = {
      true
    }

}

module Mempool {

    import Spells.* from "Spells"
    import System.* from "System"
    import Ledger.* from "Ledger"
    import ABCI.*

    type MempoolState = {
      mempool: Process -> Set[Tx],
      hmempool: Process -> Set[Tx], // history variable
      ledger: LedgerState
    }

    pure def newMempoolState(P: Set[Process]): MempoolState = {
      ledger: newLedger(P),
      mempool: P.mapBy(p => Set()),
      hmempool: P.mapBy(p => Set())
    }

    //// helpers

    pure def txsAvailable(st: MempoolState, p: Process): Set[Tx] = {
      st.mempool.get(p)
    }

    pure def txsSize(st: MempoolState, p: Process): int = {
      st.mempool.get(p).size()
    }

    pure def reap(st: MempoolState, p: Process, max: int): Set[Tx] = {
      setSubsetOfAtMost(st.mempool.get(p), max)
    }

    pure def txsOf(st: MempoolState, p: Process): Set[Tx] = {
      st.mempool.get(p)
    }

    pure def isValidAndNotCommittedFor(ledger: LedgerState, p: Process, tx: Tx): bool = {
      isValid(ledger, p, tx) and not(ledger.getCommittedFor(p).contains(tx))
    }

    //// conditions

    pure def mayRcvFromClientAt(st: MempoolState, p: Process, txs: Set[Tx]): bool = {
      and {
        not(txs.subseteq(st.txsOf(p))), // to avoid stuttering
        txs.forall(tx => isValidAndNotCommittedFor(st.ledger, p, tx))
      }
    }

    pure def mayRcvFromProcessAt(st: MempoolState, p: Process, q: Process, txs: Set[Tx]): bool = {
      and {
        p != q,
	      not(txs.subseteq(st.txsOf(q))), // to avoid stuttering
        not(st.ledger.heightOf(p) < st.ledger.heightOf(q)),
      }
    }

    pure def maySubmitToLedger(st: MempoolState, p: Process, txs: Set[Tx]) : bool = {
      and {
        txs.forall(t => st.ledger.isValid(p, t)),
	      st.ledger.maySubmit(p, txs)
      }
    }

    //// transitions

    pure def add(st: MempoolState, p: Process, txs: Set[Tx]): MempoolState = {
      val nmempool = st.mempool.set(p, st.mempool.get(p).union(txs))
      val nhmempool = st.hmempool.set(p, st.hmempool.get(p).union(txs))
      {mempool: nmempool, hmempool: nhmempool, ...st}
    }

    pure def commitThenUpdate(st: MempoolState, p: Process): MempoolState = {
      val nledger = st.ledger.commit(p)
      val nmempool = st.mempool.set(p, st.mempool.get(p).filter(tx => isValidAndNotCommittedFor(nledger, p, tx)))
      {mempool: nmempool, ledger: nledger, ...st}
    }

    //// state machine

    var _state: MempoolState

    action init : bool = all {
      all {
        _state' = newMempoolState(PROCESSES)
      }
    }

    action doClientThenAdd(p: Process, txs: Set[Tx]): bool = all {
      all {
        require(_state.mayRcvFromClientAt(p, txs)),
	      _state' = _state.add(p, txs)        
      }
    }

    action doSubmit(p: Process): bool = all {
      val txs = reap(_state, p, 1)
      all {
        require(_state.maySubmitToLedger(p, txs)),
        _state' = {ledger: _state.ledger.submit(p, txs), ..._state}
      }
    }

    action doCommitThenUpdate(p: Process): bool = all {
      all {
        require(_state.ledger.mayCommit(p)),
	      _state' = _state.commitThenUpdate(p)
      }
    }

    action doGossipThenAdd(p: Process, q: Process): bool = all {
      val txs = _state.txsOf(p) // all txs at once
      all {
        require(_state.mayRcvFromProcessAt(p, q, txs)),
	       _state' = _state.add(p, txs)
      }
   }

    action step: bool = {
      nondet p = oneOf(PROCESSES)
      nondet q = oneOf(PROCESSES)
      nondet txs = Set(oneOf(TXS)) // one at a time
      any {
        doClientThenAdd(p,txs),
        doSubmit(p),
        doCommitThenUpdate(p),
        doGossipThenAdd(p,q)
      }	
    }

    //// invariants
    
    // INV1. the mempool is used as an input for the ledger
    val inv1 = {
      PROCESSES.forall(p => _state.ledger.getSubmittedFor(p).subseteq(_state.hmempool.get(p)))
    }

    // INV2. committed transactions are not in the mempool
    val inv2 = {
      PROCESSES.forall(p => 0.to(_state.ledger.heightOf(p)-1).forall(i =>  _state.ledger.entry(p, i).intersect(_state.txsOf(p)).size()==0))
    }

    // INV3. every transaction in the mempool is valid
    val inv3 = {
      PROCESSES.forall(p => _state.txsOf(p).forall(t => _state.ledger.isValid(p, t)))
    }
    
    // INV4. every transaction that appears in the mempool is eventually committed or forever invalid
    // temporal inv4 = {
    //   PROCESSES.forall(p => mempool.get(p).forall(tx => eventually(ledger.isCommittedFor(p, tx) or always(not(ledger.isValid(p, tx))))))
    // }
    // FIXME. cannot be verified yet

    // Instead, we use the (simpler) invariant below
    // INV4b. every transaction in hmempool is always committed or if valid still in the mempool
     val inv4() = {
       PROCESSES.forall(p => _state.hmempool.get(p).forall(tx => _state.ledger.isCommittedFor(p, tx) or not(_state.ledger.isValid(p, tx)) or _state.txsOf(p).contains(tx)))
     }

    val allInv = and {
      inv1,
      inv2,
      inv3,
      inv4
    }

    //// tests

    run moveHeightOnceTest = {
      nondet p = oneOf(PROCESSES)
      nondet txs = Set(oneOf(TXS))
      init
      .then(doClientThenAdd(p,txs))
      .then(doSubmit(p))
      .then(doCommitThenUpdate(p))
    }

}
