// -*- mode: Bluespec; -*-
// The common consensus abstraction for blockchain where processes agree on a block (that is, a set of transactions).

module Consensus{

    import Spells.* from "Spells"
    import System.* from "System"

    type ConsensusState = {
      proposed: Process -> Set[Tx],
      decided: Process -> Set[Tx]
    }

    pure def newConsensus(P: Set[Process]): ConsensusState = {
      proposed: P.mapBy(p => Set()),
      decided: P.mapBy(p => Set())
    }

    //// helpers

    pure def processes(state: ConsensusState): Set[Process] = {
      state.proposed.keys()
    }

    pure def hasProposed(state: ConsensusState, p: Process): bool = {
      state.proposed.get(p).size() > 0
    }
    
    pure def hasDecided(state: ConsensusState, p: Process): bool = {
      state.decided.get(p).size() > 0
    }

    pure def isProposed(state: ConsensusState, t: Tx): bool = {
      state.proposed.keys().filter(p => state.proposed.get(p).contains(t)).size() > 0
    }

    pure def isDecided(state: ConsensusState, t: Tx): bool = {
      state.decided.keys().filter(p => state.decided.get(p).contains(t)).size() > 0
    }

    pure def hasDecision(state: ConsensusState): bool = {
      state.decided.keys().filter(p => state.decided.get(p).size()>0).size() > 0
    }

    pure def decisionOf(state: ConsensusState, p: Process): Set[Tx] = {
      state.decided.get(p)
    }

    pure def proposalOf(state: ConsensusState, p: Process): Set[Tx] = {
      state.proposed.get(p)
    }

    pure def getProposed(state: ConsensusState): Set[Tx] = {
      flatten(mapValues(state.proposed))
    }

    pure def getDecided(state: ConsensusState): Set[Tx] = {
      flatten(mapValues(state.decided))
    }

    // this function is useful when comparing two instances of consensus over time
    // (as e.g., in the stability invariant of the ledger)
    // state0 is superseded by state1 when all the processes have made more progress in state1 than in state0.
    pure def isSupersededBy(state0: ConsensusState, state1: ConsensusState): bool = {
      and {
        state0.processes() == state1.processes(),
        state0.processes().forall(
          p => not(state0.hasDecided(p) and not(state1.hasDecided(p)))
        )
      }
    }

    //// preconditions

    pure def mayPropose(state: ConsensusState, p: Process, txs: Set[Tx]): bool = {
      state.processes().contains(p) and not(state.hasProposed(p)) and txs.size() != 0
    }

    pure def mayDecide(state: ConsensusState, p: Process): bool = {
      processes(state).contains(p) and size(state.getProposed()) > 0 and not(state.hasDecided(p))
    }

    //// transitions

    pure def propose(state: ConsensusState, p: Process, txs: Set[Tx]): ConsensusState = {
      val newProposed = state.proposed.set(p, txs)
      {proposed: newProposed, ...state}
    }

    // decide any of the proposed values
    pure def decide(state: ConsensusState, p: Process): ConsensusState = {
      val proposal = state.proposed.get(setChooseSome(state.proposed.keys().filter(x => state.proposed.get(x).size()>0)))
      val decisions = state.decided.keys().filter(x => state.decided.get(x).size()>0)
      val decision = if (decisions.size()>0) {state.decided.get(setChooseSome(decisions))}  else {proposal}
      val newDecided = state.decided.set(p, decision)
      {decided: newDecided, ...state}
    }

    //// state machine

    var _consensus: ConsensusState

    action consensusInit = all {
        _consensus' = newConsensus(PROCESSES)
    }

    action consensusDoPropose = all {
      nondet p = oneOf(PROCESSES)
      nondet txs = oneOf(nonEmptyPowerset(TXS))
      all {
        require(_consensus.mayPropose(p, txs)),
        _consensus' = _consensus.propose(p, txs)
      }
    }

    action consensusDoDecide = all {
      nondet p = oneOf(PROCESSES)
      all {
        require(_consensus.mayDecide(p)),
        _consensus' = _consensus.decide(p)
      }
    }

    action consensusStep = any {
      consensusDoPropose,
      consensusDoDecide
    }

    // 5. invariants
        
    // Validity: a process may only decide proposed values
    pure def consensusValidityInv(state: ConsensusState): bool = {    
        state.processes().forall(
        p => or {
                  not(state.hasDecided(p)),
                  state.proposed.keys().filter(q => state.proposed.get(q) == state.decided.get(p)).size()>0
    	        }
        )
    }
    
    // Agreement: no two processes decide different values
    pure def consensusAgreementInv(state: ConsensusState): bool = {
      size(mapValues(state.decided).filter(x => x.size()>0)) <= 1
    }

    // Irrevocability: a decision always remains the same
    // temporal consensusIrrevocabilityInv = {
    //     _consensus.processes().forall(
    // 	  p => always(not(_consensus.hasDecided(p)) or _consensus.decisionOf(p) == next(_consensus).decisionOf(p))
    // 	)
    // }
    // FIXME. cannot be verified yet

    val consensusInvariant = {
      and {
        consensusAgreementInv(_consensus),
        consensusValidityInv(_consensus)
      }
    }

    // 6. tests
    
    run proposeTwiceErrorTest = {
      nondet p = oneOf(PROCESSES)
      nondet txs = oneOf(nonEmptyPowerset(TXS))
      consensusInit
      .then(
        all {
    	  _consensus.propose(p, txs).mayPropose(p, txs),
    	  _consensus' = _consensus
    	})
      .fail()
    }

    run decideNonProposedErrorTest = {
      consensusInit
      .then(
        all {
          _consensus.mayDecide(oneOf(PROCESSES)),
          _consensus' = _consensus
        })
      .fail()
    }

    run decideProposedSuccessTest = {
      nondet p = oneOf(PROCESSES)
      nondet q = oneOf(PROCESSES)
      nondet txs0 = oneOf(nonEmptyPowerset(TXS))
      nondet txs1 = oneOf(nonEmptyPowerset(TXS))
      consensusInit
      .then(_consensus' = _consensus.propose(p, txs0).propose(q, txs1))
      .then(
        all {
    	  _consensus' = _consensus,
    	  _consensus.mayDecide(p) and _consensus.decide(p).hasDecided(p)
        })
    }

}
