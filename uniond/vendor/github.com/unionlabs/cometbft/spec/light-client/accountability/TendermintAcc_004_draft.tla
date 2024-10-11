-------------------- MODULE TendermintAcc_004_draft ---------------------------
(*
 A TLA+ specification of a simplified Tendermint consensus algorithm, tuned for
 fork accountability. The simplifications are as follows:

 - the protocol runs for one height, that is, it is one-shot consensus

 - this specification focuses on safety, so timeouts are modelled
   with non-determinism

 - the proposer function is non-determinstic, no fairness is assumed

 - the messages by the faulty processes are injected right in the initial states

 - every process has the voting power of 1

 - hashes are modelled as identity

 Having the above assumptions in mind, the specification follows the pseudo-code
 of the Tendermint paper: https://arxiv.org/abs/1807.04938

 Byzantine processes can demonstrate arbitrary behavior, including
 no communication. We show that if agreement is violated, then the Byzantine
 processes demonstrate one of the two behaviours:

   - Equivocation: a Byzantine process may send two different values
     in the same round.

   - Amnesia: a Byzantine process may lock a value without unlocking
     the previous value that it has locked in the past.

 * Version 5. Refactor evidence, migrate to Apalache Type System 1.2.
 * Version 4. Remove defective processes, fix bugs, collect global evidence.
 * Version 3. Modular and parameterized definitions.
 * Version 2. Bugfixes in the spec and an inductive invariant.
 * Version 1. A preliminary specification.

 Zarko Milosevic, Igor Konnov, Informal Systems, 2019-2020.
 *)

EXTENDS Integers, FiniteSets, typedefs

(********************* PROTOCOL PARAMETERS **********************************)
CONSTANTS
  \* @type: Set($process);
    Corr,          \* the set of correct processes
  \* @type: Set($process);
    Faulty,        \* the set of Byzantine processes, may be empty
  \* @type: Int;
    N,             \* the total number of processes: correct, defective, and Byzantine
  \* @type: Int;
    T,             \* an upper bound on the number of Byzantine processes
  \* @type: Set($value);
    ValidValues,   \* the set of valid values, proposed both by correct and faulty
  \* @type: Set($value);
    InvalidValues, \* the set of invalid values, never proposed by the correct ones
  \* @type: $round;
    MaxRound,      \* the maximal round number
  \* @type: $round -> $process;
    Proposer       \* the proposer function from Rounds to AllProcs

ASSUME(N = Cardinality(Corr \union Faulty))

(*************************** DEFINITIONS ************************************)
AllProcs == Corr \union Faulty      \* the set of all processes
\* @type: Set($round);
Rounds == 0..MaxRound               \* the set of potential rounds
\* @type: $round;
NilRound == -1   \* a special value to denote a nil round, outside of Rounds
RoundsOrNil == Rounds \union {NilRound}
Values == ValidValues \union InvalidValues \* the set of all values
\* @type: $value;
NilValue == "None"  \* a special value for a nil round, outside of Values
ValuesOrNil == Values \union {NilValue}

\* a value hash is modeled as identity
\* @type: (t) => t;
Id(v) == v

\* The validity predicate
IsValid(v) == v \in ValidValues

\* the two thresholds that are used in the algorithm
THRESHOLD1 == T + 1     \* at least one process is not faulty
THRESHOLD2 == 2 * T + 1 \* a quorum when having N > 3 * T

(********************* PROTOCOL STATE VARIABLES ******************************)
VARIABLES
  \* @type: $process -> $round;
  round,    \* a process round number: Corr -> Rounds
  \* @type: $process -> $step;
  step,     \* a process step: Corr -> { "PROPOSE", "PREVOTE", "PRECOMMIT", "DECIDED" }
  \* @type: $process -> $value;
  decision, \* process decision: Corr -> ValuesOrNil
  \* @type: $process -> $value;
  lockedValue,  \* a locked value: Corr -> ValuesOrNil
  \* @type: $process -> $round;
  lockedRound,  \* a locked round: Corr -> RoundsOrNil
  \* @type: $process -> $value;
  validValue,   \* a valid value: Corr -> ValuesOrNil
  \* @type: $process -> $round;
  validRound    \* a valid round: Corr -> RoundsOrNil

\* book-keeping variables
VARIABLES
  \* @type: $round -> Set($proposeMsg);
  msgsPropose,   \* PROPOSE messages broadcast in the system, Rounds -> Messages
  \* @type: $round -> Set($preMsg);
  msgsPrevote,   \* PREVOTE messages broadcast in the system, Rounds -> Messages
  \* @type: $round -> Set($preMsg);
  msgsPrecommit, \* PRECOMMIT messages broadcast in the system, Rounds -> Messages
  \* @type: Set($proposeMsg);
  evidencePropose, \* the PROPOSE messages used by some correct processes to make transitions
  \* @type: Set($preMsg);
  evidencePrevote, \* the PREVOTE messages used by some correct processes to make transitions
  \* @type: Set($preMsg);
  evidencePrecommit, \* the PRECOMMIT messages used by some correct processes to make transitions
  \* @type: $action;
  action        \* we use this variable to see which action was taken

(* to see a type invariant, check TendermintAccInv3 *)

\* a handy definition used in UNCHANGED
vars == <<round, step, decision, lockedValue, lockedRound,
          validValue, validRound, msgsPropose, msgsPrevote, msgsPrecommit,
          evidencePropose, evidencePrevote, evidencePrecommit>>

(********************* PROTOCOL INITIALIZATION ******************************)
\* @type: ($round) => Set($proposeMsg);
FaultyProposals(r) ==
  [
    type      : {"PROPOSAL"},
    src       : Faulty,
    round     : {r},
    proposal  : Values,
    validRound: RoundsOrNil
  ]

\* @type: Set($proposeMsg);
AllFaultyProposals ==
  [
    type      : {"PROPOSAL"},
    src       : Faulty,
    round     : Rounds,
    proposal  : Values,
    validRound: RoundsOrNil
  ]

\* @type: ($round) => Set($preMsg);
FaultyPrevotes(r) ==
  [
    type : {"PREVOTE"},
    src  : Faulty,
    round: {r},
    id   : Values
  ]

\* @type: Set($preMsg);
AllFaultyPrevotes ==
  [
    type : {"PREVOTE"},
    src  : Faulty,
    round: Rounds,
    id   : Values
  ]

\* @type: ($round) => Set($preMsg);
FaultyPrecommits(r) ==
  [
    type : {"PRECOMMIT"},
    src  : Faulty,
    round: {r},
    id   : Values
  ]

\* @type: Set($preMsg);
AllFaultyPrecommits ==
  [
    type : {"PRECOMMIT"},
    src  : Faulty,
    round: Rounds,
    id   : Values
  ]

\* @type: ($round -> Set({ round: Int, a })) => Bool;
BenignRoundsInMessages(msgfun) ==
  \* the message function never contains a message for a wrong round
  \A r \in Rounds:
    \A m \in msgfun[r]:
      r = m.round

\* The initial states of the protocol. Some faults can be in the system already.
Init ==
    /\ round = [p \in Corr |-> 0]
    /\ step = [p \in Corr |-> "PROPOSE"]
    /\ decision = [p \in Corr |-> NilValue]
    /\ lockedValue = [p \in Corr |-> NilValue]
    /\ lockedRound = [p \in Corr |-> NilRound]
    /\ validValue = [p \in Corr |-> NilValue]
    /\ validRound = [p \in Corr |-> NilRound]
    /\ msgsPropose \in [Rounds -> SUBSET AllFaultyProposals]
    /\ msgsPrevote \in [Rounds -> SUBSET AllFaultyPrevotes]
    /\ msgsPrecommit \in [Rounds -> SUBSET AllFaultyPrecommits]
    /\ BenignRoundsInMessages(msgsPropose)
    /\ BenignRoundsInMessages(msgsPrevote)
    /\ BenignRoundsInMessages(msgsPrecommit)
    /\ evidencePropose = {}
    /\ evidencePrevote = {}
    /\ evidencePrecommit = {}
    /\ action = "Init"

(************************ MESSAGE PASSING ********************************)
\* @type: ($process, $round, $value, $round) => Bool;
BroadcastProposal(pSrc, pRound, pProposal, pValidRound) ==
  LET newMsg == [
      type       |-> "PROPOSAL",
      src        |-> pSrc,
      round      |-> pRound,
      proposal   |-> pProposal,
      validRound |-> pValidRound
  ]
  IN
  msgsPropose' = [msgsPropose EXCEPT ![pRound] = msgsPropose[pRound] \union {newMsg}]

\* @type: ($process, $round, $value) => Bool;
BroadcastPrevote(pSrc, pRound, pId) ==
  LET newMsg == [
      type  |-> "PREVOTE",
      src   |-> pSrc,
      round |-> pRound,
      id    |-> pId
  ]
  IN
  msgsPrevote' = [msgsPrevote EXCEPT ![pRound] = msgsPrevote[pRound] \union {newMsg}]

\* @type: ($process, $round, $value) => Bool;
BroadcastPrecommit(pSrc, pRound, pId) ==
  LET newMsg == [
      type  |-> "PRECOMMIT",
      src   |-> pSrc,
      round |-> pRound,
      id    |-> pId
  ]
  IN
  msgsPrecommit' = [msgsPrecommit EXCEPT ![pRound] = msgsPrecommit[pRound] \union {newMsg}]


(********************* PROTOCOL TRANSITIONS ******************************)
\* lines 12-13
StartRound(p, r) ==
   /\ step[p] /= "DECIDED" \* a decided process does not participate in consensus
   /\ round' = [round EXCEPT ![p] = r]
   /\ step' = [step EXCEPT ![p] = "PROPOSE"]

\* lines 14-19, a proposal may be sent later
\* @type: $process => Bool;
InsertProposal(p) ==
  LET r == round[p] IN
  /\ p = Proposer[r]
  /\ step[p] = "PROPOSE"
    \* if the proposer is sending a proposal, then there are no other proposals
    \* by the correct processes for the same round
  /\ \A m \in msgsPropose[r]: m.src /= p
  /\ \E v \in ValidValues:
      LET proposal ==
          IF validValue[p] /= NilValue
          THEN validValue[p]
          ELSE v
      IN BroadcastProposal(p, round[p], proposal, validRound[p])
  /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                 validValue, step, validRound, msgsPrevote, msgsPrecommit,
                 evidencePropose, evidencePrevote, evidencePrecommit>>
  /\ action' = "InsertProposal"

\* lines 22-27
UponProposalInPropose(p) ==
  \E v \in Values:
    /\ step[p] = "PROPOSE" (* line 22 *)
    /\ LET msg == [
          type       |-> "PROPOSAL",
          src        |-> Proposer[round[p]],
          round      |-> round[p],
          proposal   |-> v,
          validRound |-> NilRound
       ]
       IN
      /\ msg \in msgsPropose[round[p]] \* line 22
      /\ evidencePropose' = {msg} \union evidencePropose
    /\ LET mid == (* line 23 *)
         IF IsValid(v) /\ (lockedRound[p] = NilRound \/ lockedValue[p] = v)
         THEN Id(v)
         ELSE NilValue
       IN
       BroadcastPrevote(p, round[p], mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                   validValue, validRound, msgsPropose, msgsPrecommit,
                   evidencePrevote, evidencePrecommit>>
    /\ action' = "UponProposalInPropose"

\* lines 28-33
UponProposalInProposeAndPrevote(p) ==
  \E v \in Values, vr \in Rounds:
    /\ step[p] = "PROPOSE" /\ 0 <= vr /\ vr < round[p] \* line 28, the while part
    /\ LET msg == [
          type       |-> "PROPOSAL",
          src        |-> Proposer[round[p]],
          round      |-> round[p],
          proposal   |-> v,
          validRound |-> vr
       ]
       IN
       /\ msg \in msgsPropose[round[p]] \* line 28
       /\ LET PV == { m \in msgsPrevote[vr]: m.id = Id(v) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 28
          /\ evidencePropose' = {msg} \union evidencePropose
          /\ evidencePrevote' = PV \union evidencePrevote
    /\ LET mid == (* line 29 *)
         IF IsValid(v) /\ (lockedRound[p] <= vr \/ lockedValue[p] = v)
         THEN Id(v)
         ELSE NilValue
       IN
       BroadcastPrevote(p, round[p], mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                   validValue, validRound, msgsPropose, msgsPrecommit,
                   evidencePrecommit>>
    /\ action' = "UponProposalInProposeAndPrevote"

 \* lines 34-35 + lines 61-64 (onTimeoutPrevote)
UponQuorumOfPrevotesAny(p) ==
  /\ step[p] = "PREVOTE" \* line 34 and 61
  /\ \E MyEvidence \in SUBSET msgsPrevote[round[p]]:
      \* find the unique voters in the evidence
      LET Voters == { m.src: m \in MyEvidence } IN
      \* compare the number of the unique voters against the threshold
      /\ Cardinality(Voters) >= THRESHOLD2 \* line 34
      /\ evidencePrevote' = MyEvidence \union evidencePrevote
      /\ BroadcastPrecommit(p, round[p], NilValue)
      /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
      /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                     validValue, validRound, msgsPropose, msgsPrevote,
                     evidencePropose, evidencePrecommit>>
      /\ action' = "UponQuorumOfPrevotesAny"

\* lines 36-46
UponProposalInPrevoteOrCommitAndPrevote(p) ==
  \E v \in ValidValues, vr \in RoundsOrNil:
    /\ step[p] \in {"PREVOTE", "PRECOMMIT"} \* line 36
    /\ LET msg == [
          type       |-> "PROPOSAL",
          src        |-> Proposer[round[p]],
          round      |-> round[p],
          proposal   |-> v,
          validRound |-> vr
       ]
       IN
        /\ msg \in msgsPropose[round[p]] \* line 36
        /\ LET PV == { m \in msgsPrevote[round[p]]: m.id = Id(v) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 36
          /\ evidencePrevote' = PV \union evidencePrevote
          /\ evidencePropose' = {msg} \union evidencePropose
    /\  IF step[p] = "PREVOTE"
        THEN \* lines 38-41:
          /\ lockedValue' = [lockedValue EXCEPT ![p] = v]
          /\ lockedRound' = [lockedRound EXCEPT ![p] = round[p]]
          /\ BroadcastPrecommit(p, round[p], Id(v))
          /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
        ELSE
          UNCHANGED <<lockedValue, lockedRound, msgsPrecommit, step>>
      \* lines 42-43
    /\ validValue' = [validValue EXCEPT ![p] = v]
    /\ validRound' = [validRound EXCEPT ![p] = round[p]]
    /\ UNCHANGED <<round, decision, msgsPropose, msgsPrevote,
                   evidencePrecommit>>
    /\ action' = "UponProposalInPrevoteOrCommitAndPrevote"

\* lines 47-48 + 65-67 (onTimeoutPrecommit)
UponQuorumOfPrecommitsAny(p) ==
  /\ \E MyEvidence \in SUBSET msgsPrecommit[round[p]]:
      \* find the unique committers in the evidence
      LET Committers == { m.src: m \in MyEvidence } IN
      \* compare the number of the unique committers against the threshold
      /\ Cardinality(Committers) >= THRESHOLD2 \* line 47
      /\ evidencePrecommit' = MyEvidence \union evidencePrecommit
      /\ round[p] + 1 \in Rounds
      /\ StartRound(p, round[p] + 1)
      /\ UNCHANGED <<decision, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                     evidencePropose, evidencePrevote>>
      /\ action' = "UponQuorumOfPrecommitsAny"

\* lines 49-54
UponProposalInPrecommitNoDecision(p) ==
  /\ decision[p] = NilValue \* line 49
  /\ \E v \in ValidValues (* line 50*) , r \in Rounds, vr \in RoundsOrNil:
    /\ LET msg == [
          type       |-> "PROPOSAL",
          src        |-> Proposer[r],
          round      |-> r,
          proposal   |-> v,
          validRound |-> vr
       ]
       IN
     /\ msg \in msgsPropose[r] \* line 49
     /\ LET PV == { m \in msgsPrecommit[r]: m.id = Id(v) } IN
       /\ Cardinality(PV) >= THRESHOLD2 \* line 49
       /\ evidencePrecommit' = PV \union evidencePrecommit
       /\ evidencePropose' = evidencePropose \union { msg }
       /\ decision' = [decision EXCEPT ![p] = v] \* update the decision, line 51
    \* The original algorithm does not have 'DECIDED', but it increments the height.
    \* We introduced 'DECIDED' here to prevent the process from changing its decision.
       /\ step' = [step EXCEPT ![p] = "DECIDED"]
       /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                      validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                      evidencePrevote>>
       /\ action' = "UponProposalInPrecommitNoDecision"

\* the actions below are not essential for safety, but added for completeness

\* lines 20-21 + 57-60
OnTimeoutPropose(p) ==
  /\ step[p] = "PROPOSE"
  /\ p /= Proposer[round[p]]
  /\ BroadcastPrevote(p, round[p], NilValue)
  /\ step' = [step EXCEPT ![p] = "PREVOTE"]
  /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                 validRound, decision, msgsPropose, msgsPrecommit,
                 evidencePropose, evidencePrevote, evidencePrecommit>>
  /\ action' = "OnTimeoutPropose"

\* lines 44-46
OnQuorumOfNilPrevotes(p) ==
  /\ step[p] = "PREVOTE"
  /\ LET PV == { m \in msgsPrevote[round[p]]: m.id = Id(NilValue) } IN
    /\ Cardinality(PV) >= THRESHOLD2 \* line 36
    /\ evidencePrevote' = PV \union evidencePrevote
    /\ BroadcastPrecommit(p, round[p], Id(NilValue))
    /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
    /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                   validRound, decision, msgsPropose, msgsPrevote,
                   evidencePropose, evidencePrecommit>>
    /\ action' = "OnQuorumOfNilPrevotes"

\* lines 55-56
OnRoundCatchup(p) ==
  \E r \in {rr \in Rounds: rr > round[p]}:
    \E EvPropose \in SUBSET msgsPropose[r],
          EvPrevote \in SUBSET msgsPrevote[r],
          EvPrecommit \in SUBSET msgsPrecommit[r]:
        LET \* @type: Set({ src: $process, a }) => Set($process);
            Src(E) == { m.src: m \in E }
        IN
        LET Faster ==
            Src(EvPropose) \union Src(EvPrevote) \union Src(EvPrecommit)
        IN
        /\ Cardinality(Faster) >= THRESHOLD1
        /\ evidencePropose' = EvPropose \union evidencePropose
        /\ evidencePrevote' = EvPrevote \union evidencePrevote
        /\ evidencePrecommit' = EvPrecommit \union evidencePrecommit
        /\ StartRound(p, r)
        /\ UNCHANGED <<decision, lockedValue, lockedRound, validValue,
                      validRound, msgsPropose, msgsPrevote, msgsPrecommit>>
        /\ action' = "OnRoundCatchup"

(*
 * A system transition. In this specificatiom, the system may eventually deadlock,
 * e.g., when all processes decide. This is expected behavior, as we focus on safety.
 *)
Next ==
  \E p \in Corr:
    \/ InsertProposal(p)
    \/ UponProposalInPropose(p)
    \/ UponProposalInProposeAndPrevote(p)
    \/ UponQuorumOfPrevotesAny(p)
    \/ UponProposalInPrevoteOrCommitAndPrevote(p)
    \/ UponQuorumOfPrecommitsAny(p)
    \/ UponProposalInPrecommitNoDecision(p)
    \* the actions below are not essential for safety, but added for completeness
    \/ OnTimeoutPropose(p)
    \/ OnQuorumOfNilPrevotes(p)
    \/ OnRoundCatchup(p)


(**************************** FORK SCENARIOS  ***************************)

\* equivocation by a process p
EquivocationBy(p) ==
    LET
        \* @type: Set({ src: $process, a }) => Bool;
        EquivocationIn(S) ==
            \E m1, m2 \in S:
              /\ m1 /= m2
              /\ m1.src = p
              /\ m2.src = p
              /\ m1.round = m2.round
              /\ m1.type = m2.type
    IN
    \/ EquivocationIn(evidencePropose)
    \/ EquivocationIn(evidencePrevote)
    \/ EquivocationIn(evidencePrecommit)

\* amnesic behavior by a process p
AmnesiaBy(p) ==
    \E r1, r2 \in Rounds:
      /\ r1 < r2
      /\ \E v1, v2 \in ValidValues:
        /\ v1 /= v2
        /\ [
            type  |-> "PRECOMMIT",
            src   |-> p,
            round |-> r1,
            id    |-> Id(v1)
           ] \in evidencePrecommit
        /\ [
            type  |-> "PREVOTE",
            src   |-> p,
            round |-> r2,
            id    |-> Id(v2)
           ] \in evidencePrevote
        /\ \A r \in { rnd \in Rounds: r1 <= rnd /\ rnd < r2 }:
            LET prevotes ==
                { m \in evidencePrevote: m.round = r /\ m.id = Id(v2) }
            IN
            Cardinality(prevotes) < THRESHOLD2

(******************************** PROPERTIES  ***************************************)

\* the safety property -- agreement
Agreement ==
  \A p, q \in Corr:
    \/ decision[p] = NilValue
    \/ decision[q] = NilValue
    \/ decision[p] = decision[q]

\* the protocol validity
Validity ==
    \A p \in Corr: decision[p] \in ValidValues \union {NilValue}

(*
  The protocol safety. Two cases are possible:
     1. There is no fork, that is, Agreement holds true.
     2. A subset of faulty processes demonstrates equivocation or amnesia.
 *)
Accountability ==
    \/ Agreement
    \/ \E Detectable \in SUBSET Faulty:
        /\ Cardinality(Detectable) >= THRESHOLD1
        /\ \A p \in Detectable:
            EquivocationBy(p) \/ AmnesiaBy(p)

(****************** FALSE INVARIANTS TO PRODUCE EXAMPLES ***********************)

\* This property is violated. You can check it to see how amnesic behavior
\* appears in the evidence variable.
NoAmnesia ==
    \A p \in Faulty: ~AmnesiaBy(p)

\* This property is violated. You can check it to see an example of equivocation.
NoEquivocation ==
    \A p \in Faulty: ~EquivocationBy(p)

\* This property is violated. You can check it to see an example of agreement.
\* It is not exactly ~Agreement, as we do not want to see the states where
\* decision[p] = NilValue
NoAgreement ==
  \A p, q \in Corr:
    (p /= q /\ decision[p] /= NilValue /\ decision[q] /= NilValue)
        => decision[p] /= decision[q]

\* Either agreement holds, or the faulty processes indeed demonstrate amnesia.
\* This property is violated. A counterexample should demonstrate equivocation.
AgreementOrAmnesia ==
    Agreement \/ (\A p \in Faulty: AmnesiaBy(p))

\* We expect this property to be violated. It shows us a protocol run,
\* where one faulty process demonstrates amnesia without equivocation.
\* However, the absence of amnesia
\* is a tough constraint for Apalache. It has not reported a counterexample
\* for n=4,f=2, length <= 5.
ShowMeAmnesiaWithoutEquivocation ==
    (~Agreement /\ \E p \in Faulty: ~EquivocationBy(p))
        => \A p \in Faulty: ~AmnesiaBy(p)

\* This property is violated on n=4,f=2, length=4 in less than 10 min.
\* Two faulty processes may demonstrate amnesia without equivocation.
AmnesiaImpliesEquivocation ==
    (\E p \in Faulty: AmnesiaBy(p)) => (\E q \in Faulty: EquivocationBy(q))

(*
  This property is violated. You can check it to see that all correct processes
  may reach MaxRound without making a decision.
 *)
NeverUndecidedInMaxRound ==
    LET AllInMax == \A p \in Corr: round[p] = MaxRound
        AllDecided == \A p \in Corr: decision[p] /= NilValue
    IN
    AllInMax => AllDecided

=============================================================================

