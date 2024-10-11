------------------ MODULE TendermintAccDebug_004_draft -------------------------
(*
 A few definitions that we use for debugging TendermintAcc3, which do not belong
 to the specification itself.
 
 * Version 3. Modular and parameterized definitions.

 Igor Konnov, 2020.
 *)

EXTENDS TendermintAccInv_004_draft

\* make them parameters?
NFaultyProposals == 0   \* the number of injected faulty PROPOSE messages
NFaultyPrevotes == 6    \* the number of injected faulty PREVOTE messages
NFaultyPrecommits == 6  \* the number of injected faulty PRECOMMIT messages

\* Given a set of allowed messages Msgs, this operator produces a function from
\* rounds to sets of messages.
\* Importantly, there will be exactly k messages in the image of msgFun.
\* We use this action to produce k faults in an initial state.
\* @type: ($round -> Set({ round: $round, a }),
\*         Set({ round: $round, a }), Int)
\*          => Bool;
ProduceFaults(msgFun, From, k) ==
    \E f \in [1..k -> From]:
        msgFun = [r \in Rounds |-> {m \in {f[i]: i \in 1..k}: m.round = r}]

\* As TLC explodes with faults, we may have initial states without faults    
InitNoFaults ==
    /\ round = [p \in Corr |-> 0]
    /\ step = [p \in Corr |-> "PROPOSE"]
    /\ decision = [p \in Corr |-> NilValue]
    /\ lockedValue = [p \in Corr |-> NilValue]
    /\ lockedRound = [p \in Corr |-> NilRound]
    /\ validValue = [p \in Corr |-> NilValue]
    /\ validRound = [p \in Corr |-> NilRound]
    /\ msgsPropose = [r \in Rounds |-> {}]
    /\ msgsPrevote = [r \in Rounds |-> {}]
    /\ msgsPrecommit = [r \in Rounds |-> {}]
    /\ evidencePropose = {}
    /\ evidencePrevote = {}
    /\ evidencePrecommit = {}

(*
 A specialized version of Init that injects NFaultyProposals proposals,
 NFaultyPrevotes prevotes, NFaultyPrecommits precommits by the faulty processes
 *)
InitFewFaults ==
    /\ round = [p \in Corr |-> 0]
    /\ step = [p \in Corr |-> "PROPOSE"]
    /\ decision = [p \in Corr |-> NilValue]
    /\ lockedValue = [p \in Corr |-> NilValue]
    /\ lockedRound = [p \in Corr |-> NilRound]
    /\ validValue = [p \in Corr |-> NilValue]
    /\ validRound = [p \in Corr |-> NilRound]
    /\ ProduceFaults(msgsPrevote',
                     [type: {"PREVOTE"}, src: Faulty, round: Rounds, id: Values],
                     NFaultyPrevotes)
    /\ ProduceFaults(msgsPrecommit',
                     [type: {"PRECOMMIT"}, src: Faulty, round: Rounds, id: Values],
                     NFaultyPrecommits)
    /\ ProduceFaults(msgsPropose',
                     [type: {"PROPOSAL"}, src: Faulty, round: Rounds,
                                proposal: Values, validRound: Rounds \cup {NilRound}],
                     NFaultyProposals)
    /\ evidencePropose = {}
    /\ evidencePrevote = {}
    /\ evidencePrecommit = {}

\* Add faults incrementally
NextWithFaults ==
    \* either the protocol makes a step
    \/ Next
    \* or a faulty process sends a message
    \//\ UNCHANGED <<round, step, decision, lockedValue,
                     lockedRound, validValue, validRound,
                     evidencePropose, evidencePrevote, evidencePrecommit>>
      /\ \E p \in Faulty:
         \E r \in Rounds:
           \//\ UNCHANGED <<msgsPrevote, msgsPrecommit>>
             /\ \E proposal \in ValidValues \union {NilValue}:
                \E vr \in RoundsOrNil:
                  BroadcastProposal(p, r, proposal, vr)
           \//\ UNCHANGED <<msgsPropose, msgsPrecommit>>
             /\ \E id \in ValidValues \union {NilValue}:
                  BroadcastPrevote(p, r, id)
           \//\ UNCHANGED <<msgsPropose, msgsPrevote>>
             /\ \E id \in ValidValues \union {NilValue}:
                  BroadcastPrecommit(p, r, id)

(******************************** PROPERTIES  ***************************************)
\* simple reachability properties to see that the spec is progressing
NoPrevote == \A p \in Corr: step[p] /= "PREVOTE" 

NoPrecommit == \A p \in Corr: step[p] /= "PRECOMMIT"   

NoValidPrecommit ==
    \A r \in Rounds:
      \A m \in msgsPrecommit[r]:
        m.id = NilValue \/ m.src \in Faulty

NoHigherRounds == \A p \in Corr: round[p] < 1

NoDecision == \A p \in Corr: decision[p] = NilValue                    

=============================================================================    
 
