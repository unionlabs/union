// -*- mode: Bluespec; -*-
// A Ledger is a replicated log of blocks.
// The specification below considers that
// - replication is made with consensus (see Consensus.qnt)
// - idempotence is not granted (that is, two log entries might have transactions in common).

module Ledger{

    import Spells.* from "Spells"
    import System.* from "System"
    import Consensus.* from "Consensus"

    type LedgerState = {
      log: List[ConsensusState],
    }

    pure def newLedger(P: Set[Process]): LedgerState = {
      log: List(newConsensus(P))
    }

    //// helpers

    // index of the first null entry
    pure def height(state: LedgerState): int = {
      length(state.log.select(s => s.hasDecision()))
    }

    pure def heightOf(state: LedgerState, p: Process): int = {
      length(state.log.select(s => s.hasDecided(p)))
    }

    pure def entry(state: LedgerState, p: Process, i: int): Set[Tx] = {
      state.log[i].decisionOf(p)
    }

    pure def lastEntry(state: LedgerState, p: Process): Set[Tx] = {
      entry(state, p, state.heightOf(p)-1)
    }

    pure def getSubmittedFor(state: LedgerState, p: Process): Set[Tx] = {
      0.to(state.heightOf(p)).fold(Set(), (s, i) => s.union(state.log[i].proposalOf(p)))
    }

    pure def getCommittedFor(state: LedgerState, p: Process): Set[Tx] = {
      0.to(state.heightOf(p)).fold(Set(), (s, i) => s.union(state.log[i].decisionOf(p)))
    }

    pure def isSubmitted(state: LedgerState, t: Tx): bool = {
      0.to(state.height()).exists(h => state.log[h].isProposed(t))
    }

    pure def isCommitted(state: LedgerState, t: Tx): bool = {
      if (state.height()==0) false else 0.to(state.height()-1).exists(i => state.log[i].isDecided(t))
    }

    pure def isCommittedFor(state: LedgerState, p: Process, t: Tx): bool = {
      if (state.heightOf(p)==0) false else 0.to(state.heightOf(p)-1).exists(h => state.log[h].isDecided(t))
    }

    //// preconditions

    pure def maySubmit(state: LedgerState, p: Process, txs: Set[Tx]): bool = {
      state.log[state.heightOf(p)].mayPropose(p, txs)
    }

    pure def mayCommit(state: LedgerState, p: Process): bool = {
      state.log[state.heightOf(p)].mayDecide(p)
    }

    //// transitions

    pure def submit(state: LedgerState, p: Process, txs: Set[Tx]): LedgerState = {
      val currentConsensus = state.log[heightOf(state,p)]
      val nextLog = state.log.replaceAt(heightOf(state,p), currentConsensus.propose(p, txs))
      {log: nextLog, ...state}
    }

    pure def commit(state: LedgerState, p: Process): LedgerState = {
      val currentConsensus = state.log[state.heightOf(p)]
      val nextLog = if (state.heightOf(p) == state.height()) {
          state.log.append(newConsensus(processes(state.log[0])))
      } else {
    	  state.log
      }
      {log: nextLog.replaceAt(heightOf(state,p), currentConsensus.decide(p)), ...state}
    }

    //// state machine

    var _ledger: LedgerState

    action ledgerInit = all {
        _ledger' = newLedger(PROCESSES)
    }

    action ledgerDoSubmit = all {
      nondet p = oneOf(PROCESSES)
      nondet txs = oneOf(nonEmptyPowerset(TXS))
      all {
        require(_ledger.maySubmit(p, txs)),
        _ledger' = _ledger.submit(p, txs)
      }
    }

    action ledgerDoCommit = all {
      nondet p = oneOf(PROCESSES)
      all {
        require(_ledger.mayCommit(p)),
        _ledger' = _ledger.commit(p)
      }
    }

    action ledgerStep = any {
      ledgerDoSubmit,
      ledgerDoCommit
    }

    //// invariants

    // Validity: every non-null entry is submitted.
    pure def ledgerValidityInv(state: LedgerState): bool = {
      0.to(state.height()).forall(h => consensusValidityInv(state.log[h]))
    }
    
    // Total Order: for any two processes, entries are prefix one from another.
    pure def ledgerOrdertInv(state: LedgerState): bool = {
      0.to(state.height()-1).forall(h => consensusAgreementInv(state.log[h]))
    }

    // Stability: for every process, its entry always grows.
    // temporal ledgerStabilityInv = {
    //   and {
    //     _ledger.height() <= next(_ledger.height()),	
    //     0.to(_ledger.height()).forall(h => _ledger.log[h].isSupersededBy(next(_ledger.log[h])))
    //   }
    // }
    // FIXME. cannot be verified yet

    val ledgerInvariant = {
      and {
        ledgerValidityInv(_ledger),
        ledgerOrdertInv(_ledger)
      }
    }

    //// tests
    
    run submitTwiceErrorTest = {
      nondet p = oneOf(PROCESSES)
      nondet txs = oneOf(nonEmptyPowerset(TXS))
      ledgerInit
      .then(
        all {
          _ledger.submit(p, txs).maySubmit(p, txs),
          _ledger'=_ledger
       }
      )
      .fail()
    }

    run commitNonSubmittedErrorTest = {
      nondet p = oneOf(PROCESSES)
      ledgerInit
      .then(
        all {
          _ledger.mayCommit(p),
          _ledger'=_ledger
        }
      )
      .fail()
    }

    run commitSubmittedSuccessTest = {
      nondet p = oneOf(PROCESSES)
      nondet q = oneOf(PROCESSES)
      nondet txs0 = oneOf(nonEmptyPowerset(TXS))
      nondet txs1 = oneOf(nonEmptyPowerset(TXS))
      ledgerInit
      .then(_ledger' = _ledger.submit(p, txs0).submit(q, txs1))
      .then(
        all {
          _ledger.mayCommit(p) and heightOf(_ledger.commit(p), p)==1,
          _ledger'=_ledger
       }
      )
    }

}
