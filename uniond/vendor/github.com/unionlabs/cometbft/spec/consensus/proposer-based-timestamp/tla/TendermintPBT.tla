-------------------- MODULE TendermintPBT ---------------------------
(*
 A TLA+ specification of a simplified Tendermint consensus algorithm, with added clocks
 and proposer-based timestamps. This TLA+ specification extends and modifies 
 the Tendermint TLA+ specification for fork accountability: 
    https://github.com/tendermint/spec/blob/master/spec/light-client/accountability/TendermintAcc_004_draft.tla
 
 * Version 2. A preliminary specification.

 Zarko Milosevic, Igor Konnov, Informal Systems, 2019-2020.
 Ilina Stoilkovska, Josef Widder, Informal Systems, 2021.
 Jure Kukovec, Informal Systems, 2022.
 *)

EXTENDS Integers, FiniteSets, Apalache, Sequences, typedefs

(********************* PROTOCOL PARAMETERS **********************************)
\* General protocol parameters
CONSTANTS
  \* @type: Set(PROCESS);
  Corr,          \* the set of correct processes 
  \* @type: Set(PROCESS);
  Faulty,        \* the set of Byzantine processes, may be empty
  \* @type: Int;  
  N,             \* the total number of processes: correct, defective, and Byzantine
  \* @type: Int;  
  T,             \* an upper bound on the number of Byzantine processes
  \* @type: Set(VALUE);  
  ValidValues,   \* the set of valid values, proposed both by correct and faulty
  \* @type: Set(VALUE);  
  InvalidValues, \* the set of invalid values, never proposed by the correct ones
  \* @type: ROUND;    
  MaxRound,      \* the maximal round number
  \* @type: ROUND -> PROCESS;
  Proposer       \* the proposer function from Rounds to AllProcs

\* Time-related parameters
CONSTANTS
  \* @type: TIME;
  MaxTimestamp,  \* the maximal value of the clock tick
  \* @type: TIME;
  MinTimestamp,  \* the minimal value of the clock tick
  \* @type: TIME;
  Delay,         \* message delay
  \* @type: TIME;
  Precision     \* clock precision: the maximal difference between two local clocks  

ASSUME(N = Cardinality(Corr \union Faulty))

\* Modeling parameter
CONSTANTS
  \* @type: Bool;
  PreloadAllFaultyMsgs,
  \* @type: Int;
  N_GEN

(*************************** DEFINITIONS ************************************)
\* @type: Set(PROCESS);
AllProcs == Corr \union Faulty      \* the set of all processes
\* @type: Set(ROUND);
Rounds == 0..MaxRound               \* the set of potential rounds
\* @type: Set(TIME);
Timestamps == 0..MaxTimestamp       \* the set of clock ticks
\* @type: ROUND;
NilRound == -1   \* a special value to denote a nil round, outside of Rounds
\* @type: TIME;
NilTimestamp == -1 \* a special value to denote a nil timestamp, outside of Ticks
\* @type: Set(ROUND);
RoundsOrNil == Rounds \union {NilRound}
\* @type: Set(VALUE);
Values == ValidValues \union InvalidValues \* the set of all values
\* @type: VALUE;
NilValue == "None"  \* a special value for a nil round, outside of Values
\* @type: Set(PROPOSAL);
Proposals == Values \X Timestamps \X Rounds
\* @type: PROPOSAL;
NilProposal == <<NilValue, NilTimestamp, NilRound>>
\* @type: Set(VALUE);
ValuesOrNil == Values \union {NilValue}
\* @type: Set(DECISION);
Decisions == Proposals \X Rounds
\* @type: DECISION;
NilDecision == <<NilProposal, NilRound>>

ArbitraryProposer == Proposer \in [Rounds -> AllProcs]
CorrectProposer == Proposer \in [Rounds -> Corr]
CyclicalProposer == 
  LET ProcOrder ==
    LET App(s,e) == Append(s,e)
    IN ApaFoldSet(App, <<>>, AllProcs)
  IN Proposer = [ r \in Rounds |-> ProcOrder[1 + (r % N)] ]

ValidProposals == ValidValues \X (MinTimestamp..MaxTimestamp) \X Rounds
\* a value hash is modeled as identity
\* @type: (t) => t;
Id(v) == v

\* The validity predicate
\* @type: (PROPOSAL) => Bool;
IsValid(p) == p \in ValidProposals

\* Time validity check. If we want MaxTimestamp = \infty, set ValidTime(t) == TRUE
ValidTime(t) == t < MaxTimestamp

\* @type: (PROPMESSAGE) => VALUE;
MessageValue(msg) == msg.proposal[1]
\* @type: (PROPMESSAGE) => TIME;
MessageTime(msg) == msg.proposal[2]
\* @type: (PROPMESSAGE) => ROUND;
MessageRound(msg) == msg.proposal[3]

\* @type: (TIME, TIME) => Bool;
IsTimely(processTime, messageTime) ==
  /\ processTime >= messageTime - Precision
  /\ processTime <= messageTime + Precision + Delay

\* the two thresholds that are used in the algorithm
\* @type: Int;
THRESHOLD1 == T + 1     \* at least one process is not faulty
\* @type: Int;
THRESHOLD2 == 2 * T + 1 \* a quorum when having N > 3 * T

\* @type: (TIME, TIME) => TIME;
Min2(a,b) == IF a <= b THEN a ELSE b
\* @type: (Set(TIME)) => TIME;
Min(S) == ApaFoldSet( Min2, MaxTimestamp, S )
\* Min(S) == CHOOSE x \in S : \A y \in S : x <= y

\* @type: (TIME, TIME) => TIME;
Max2(a,b) == IF a >= b THEN a ELSE b
\* @type: (Set(TIME)) => TIME;
Max(S) == ApaFoldSet( Max2, NilTimestamp, S )
\* Max(S) == CHOOSE x \in S : \A y \in S : y <= x

\* @type: (Set(MESSAGE)) => Int;
Card(S) == 
  LET 
    \* @type: (Int, MESSAGE) => Int;
    PlusOne(i, m) == i + 1
  IN ApaFoldSet( PlusOne, 0, S )

(********************* PROTOCOL STATE VARIABLES ******************************)
VARIABLES
  \* @type: PROCESS -> ROUND;
  round,    \* a process round number
  \* @type: PROCESS -> STEP;
  step,     \* a process step
  \* @type: PROCESS -> DECISION;
  decision, \* process decision
  \* @type: PROCESS -> VALUE;
  lockedValue,  \* a locked value
  \* @type: PROCESS -> ROUND;
  lockedRound,  \* a locked round
  \* @type: PROCESS -> PROPOSAL;
  validValue,   \* a valid value
  \* @type: PROCESS -> ROUND;
  validRound    \* a valid round

coreVars == 
  <<round, step, decision, lockedValue, 
  lockedRound, validValue, validRound>>

\* time-related variables
VARIABLES  
  \* @type: PROCESS -> TIME;
  localClock, \* a process local clock: Corr -> Ticks
  \* @type: TIME;
  realTime   \* a reference Newtonian real time

temporalVars == <<localClock, realTime>>

\* book-keeping variables
VARIABLES
  \* @type: ROUND -> Set(PROPMESSAGE);
  msgsPropose,   \* PROPOSE messages broadcast in the system, Rounds -> Messages
  \* @type: ROUND -> Set(PREMESSAGE);
  msgsPrevote,   \* PREVOTE messages broadcast in the system, Rounds -> Messages
  \* @type: ROUND -> Set(PREMESSAGE);
  msgsPrecommit, \* PRECOMMIT messages broadcast in the system, Rounds -> Messages
  \* @type: Set(MESSAGE);
  evidence, \* the messages that were used by the correct processes to make transitions
  \* @type: ACTION;
  action,       \* we use this variable to see which action was taken
  \* @type: <<ROUND,PROCESS>> -> TIME;
  proposalReceptionTime \* used to keep track when a process receives a message
  
\* Action is excluded from the tuple, because it always changes
bookkeepingVars == 
  <<msgsPropose, msgsPrevote, msgsPrecommit, 
  evidence, (*action,*) proposalReceptionTime>>

\* Invariant support
VARIABLES
  \* @type: <<ROUND, PROCESS>> -> TIME;
  beginRound \* the minimum of the local clocks at the time any process entered a new round

(* to see a type invariant, check TendermintAccInv3 *)  

(********************* PROTOCOL INITIALIZATION ******************************)
\* @type: (ROUND) => Set(PROPMESSAGE);
FaultyProposals(r) ==
  [
    type      : {"PROPOSAL"}, 
    src       : Faulty,
    round     : {r}, 
    proposal  : Proposals, 
    validRound: RoundsOrNil
  ]

\* @type: Set(PROPMESSAGE);
AllFaultyProposals ==
  [
    type      : {"PROPOSAL"}, 
    src       : Faulty,
    round     : Rounds, 
    proposal  : Proposals, 
    validRound: RoundsOrNil
  ]

\* @type: (ROUND) => Set(PREMESSAGE);
FaultyPrevotes(r) ==
  [
    type : {"PREVOTE"}, 
    src  : Faulty, 
    round: {r}, 
    id   : Proposals
  ]

\* @type: Set(PREMESSAGE);
AllFaultyPrevotes ==    
  [
    type : {"PREVOTE"}, 
    src  : Faulty, 
    round: Rounds, 
    id   : Proposals
  ]

\* @type: (ROUND) => Set(PREMESSAGE);
FaultyPrecommits(r) ==
  [
    type : {"PRECOMMIT"}, 
    src  : Faulty, 
    round: {r}, 
    id   : Proposals
  ]

\* @type: Set(PREMESSAGE);
AllFaultyPrecommits ==
  [
    type : {"PRECOMMIT"}, 
    src  : Faulty, 
    round: Rounds, 
    id   : Proposals
  ]

\* @type: Set(PROPMESSAGE);
AllProposals ==
  [
    type      : {"PROPOSAL"}, 
    src       : AllProcs,
    round     : Rounds, 
    proposal  : Proposals, 
    validRound: RoundsOrNil
  ]    

\* @type: (ROUND) => Set(PROPMESSAGE);
RoundProposals(r) ==
  [
    type      : {"PROPOSAL"}, 
    src       : AllProcs,
    round     : {r}, 
    proposal  : Proposals, 
    validRound: RoundsOrNil
  ]

\* @type: (ROUND -> Set(MESSAGE)) => Bool;
BenignRoundsInMessages(msgfun) ==
  \* the message function never contains a message for a wrong round
  \A r \in Rounds:
    \A m \in msgfun[r]:
      r = m.round

\* @type: (ROUND -> Set(MESSAGE), Set(MESSAGE)) => Bool;
BenignAndSubset(msgfun, set) == 
  /\ \A r \in Rounds: 
    \* The generated values belong to SUBSET set
    /\ msgfun[r] \subseteq set
    \* the message function never contains a message for a wrong round
    /\ \A m \in msgfun[r]: r = m.round

InitGen == 
  /\ msgsPropose \in [Rounds -> Gen(N_GEN)]
  /\ msgsPrevote \in [Rounds -> Gen(N_GEN)]
  /\ msgsPrecommit \in [Rounds -> Gen(N_GEN)]
  /\ BenignAndSubset(msgsPropose, AllFaultyProposals)
  /\ BenignAndSubset(msgsPrevote, AllFaultyPrevotes)
  /\ BenignAndSubset(msgsPrecommit, AllFaultyPrecommits)

InitPreloadAllMsgs ==
  /\ msgsPropose \in [Rounds -> SUBSET AllFaultyProposals]
  /\ msgsPrevote \in [Rounds -> SUBSET AllFaultyPrevotes]
  /\ msgsPrecommit \in [Rounds -> SUBSET AllFaultyPrecommits]
  /\ BenignRoundsInMessages(msgsPropose)
  /\ BenignRoundsInMessages(msgsPrevote)
  /\ BenignRoundsInMessages(msgsPrecommit)

InitMsgs ==
  \/ /\ PreloadAllFaultyMsgs
     \* /\ InitPreloadAllMsgs
     /\ InitGen
  \/ /\ ~PreloadAllFaultyMsgs
     /\ msgsPropose = [r \in Rounds |-> {}]
     /\ msgsPrevote = [r \in Rounds |-> {}]
     /\ msgsPrecommit = [r \in Rounds |-> {}]

\* The initial states of the protocol. Some faults can be in the system already.
Init ==
    /\ round = [p \in Corr |-> 0]
    /\ localClock \in [Corr -> MinTimestamp..(MinTimestamp + Precision)]
    /\ realTime = 0
    /\ step = [p \in Corr |-> "PROPOSE"]
    /\ decision = [p \in Corr |-> NilDecision]
    /\ lockedValue = [p \in Corr |-> NilValue]
    /\ lockedRound = [p \in Corr |-> NilRound]
    /\ validValue = [p \in Corr |-> NilProposal]
    /\ validRound = [p \in Corr |-> NilRound]
    /\ InitMsgs
    /\ proposalReceptionTime = [r \in Rounds, p \in Corr |-> NilTimestamp]
    /\ evidence = {}
    /\ action = "Init"
    /\ beginRound = 
      [r \in Rounds, c \in Corr |-> 
        IF r = 0
        THEN localClock[c]
        ELSE MaxTimestamp
      ]

lastBeginRound == [ r \in Rounds |->
  Max({beginRound[r,p] : p \in Corr})
]

firstBeginRound == [ r \in Rounds |->
  Min({beginRound[r,p] : p \in Corr})
]

\* Faulty processes send messages
FaultyBroadcast == 
  /\ ~PreloadAllFaultyMsgs
  /\ action' = "FaultyBroadcast"
  /\ \E r \in Rounds:
    \/ \E msgs \in SUBSET FaultyProposals(r):
        /\ msgsPropose' = [msgsPropose EXCEPT ![r] = @ \union msgs]
        /\ UNCHANGED <<coreVars, temporalVars, beginRound>>
        /\ UNCHANGED 
          <<(*msgsPropose,*) msgsPrevote, msgsPrecommit, 
          evidence, (*action,*) proposalReceptionTime>>
    \/ \E msgs \in SUBSET FaultyPrevotes(r):
        /\ msgsPrevote' = [msgsPrevote EXCEPT ![r] = @ \union msgs]
        /\ UNCHANGED <<coreVars, temporalVars, beginRound>>
        /\ UNCHANGED 
          <<msgsPropose, (*msgsPrevote,*) msgsPrecommit, 
          evidence, (*action,*) proposalReceptionTime>>
    \/ \E msgs \in SUBSET FaultyPrecommits(r):
        /\ msgsPrecommit' = [msgsPrecommit EXCEPT ![r] = @ \union msgs]
        /\ UNCHANGED <<coreVars, temporalVars, beginRound>>
        /\ UNCHANGED 
          <<msgsPropose, msgsPrevote, (*msgsPrecommit,*)
          evidence, (*action,*) proposalReceptionTime>>

(************************ MESSAGE PASSING ********************************)
\* @type: (PROCESS, ROUND, PROPOSAL, ROUND) => Bool;
BroadcastProposal(pSrc, pRound, pProposal, pValidRound) ==
  LET 
    \* @type: PROPMESSAGE;
    newMsg ==
    [
      type       |-> "PROPOSAL", 
      src        |-> pSrc, 
      round      |-> pRound,
      proposal   |-> pProposal, 
      validRound |-> pValidRound
    ]
  IN
  /\ msgsPropose' = [msgsPropose EXCEPT ![pRound] = msgsPropose[pRound] \union {newMsg}]

\* @type: (PROCESS, ROUND, PROPOSAL) => Bool;
BroadcastPrevote(pSrc, pRound, pId) ==
  LET 
    \* @type: PREMESSAGE;
    newMsg == 
    [
      type  |-> "PREVOTE",
      src   |-> pSrc, 
      round |-> pRound, 
      id    |-> pId
    ]
  IN
  /\ msgsPrevote' = [msgsPrevote EXCEPT ![pRound] = msgsPrevote[pRound] \union {newMsg}]

\* @type: (PROCESS, ROUND, PROPOSAL) => Bool;
BroadcastPrecommit(pSrc, pRound, pId) ==
  LET
    \* @type: PREMESSAGE; 
    newMsg == 
    [
      type  |-> "PRECOMMIT",
      src   |-> pSrc, 
      round |-> pRound, 
      id    |-> pId
    ]
  IN
  /\ msgsPrecommit' = [msgsPrecommit EXCEPT ![pRound] = msgsPrecommit[pRound] \union {newMsg}]

(***************************** TIME **************************************)

\* [PBTS-CLOCK-PRECISION.0]
\* @type: Bool;
SynchronizedLocalClocks ==
    \A p \in Corr : \A q \in Corr : 
        p /= q => 
            \/ /\ localClock[p] >= localClock[q]
               /\ localClock[p] - localClock[q] < Precision 
            \/ /\ localClock[p] < localClock[q]
               /\ localClock[q] - localClock[p] < Precision
    
\* [PBTS-PROPOSE.0]
\* @type: (VALUE, TIME, ROUND) => PROPOSAL;
Proposal(v, t, r) ==
    <<v, t, r>>

\* [PBTS-DECISION-ROUND.0]
\* @type: (PROPOSAL, ROUND) => DECISION;
Decision(p, r) ==
    <<p, r>>

(**************** MESSAGE PROCESSING TRANSITIONS *************************)
\* lines 12-13
\* @type: (PROCESS, ROUND) => Bool;
StartRound(p, r) ==
   /\ step[p] /= "DECIDED" \* a decided process does not participate in consensus
   /\ round' = [round EXCEPT ![p] = r]
   /\ step' = [step EXCEPT ![p] = "PROPOSE"]
   \* We only need to update (last)beginRound[r] once a process enters round `r`
   /\ beginRound' = [beginRound EXCEPT ![r,p] = localClock[p]]

\* lines 14-19, a proposal may be sent later
\* @type: (PROCESS) => Bool;
InsertProposal(p) == 
  LET r == round[p] IN
  /\ p = Proposer[r]
  /\ step[p] = "PROPOSE"
    \* if the proposer is sending a proposal, then there are no other proposals
    \* by the correct processes for the same round
  /\ \A m \in msgsPropose[r]: m.src /= p
  /\ \E v \in ValidValues: 
      LET proposal == 
        IF validValue[p] /= NilProposal 
        THEN validValue[p]
        ELSE Proposal(v, localClock[p], r)
      IN               
      /\ BroadcastProposal(p, r, proposal, validRound[p])
  /\ UNCHANGED <<temporalVars, coreVars>>
  /\ UNCHANGED 
    <<(*msgsPropose,*) msgsPrevote, msgsPrecommit, 
    evidence, proposalReceptionTime>>
  /\ UNCHANGED beginRound
  /\ action' = "InsertProposal"
  
\* a new action used to register the proposal and note the reception time.
\* [PBTS-RECEPTION-STEP.0]
\* @type: (PROCESS) => Bool;
ReceiveProposal(p) ==
  \E v \in Values, t \in Timestamps:
    /\ LET r == round[p] IN
       LET 
        \* @type: PROPMESSAGE;
        msg ==
          [
            type       |-> "PROPOSAL", 
            src        |-> Proposer[round[p]],
            round      |-> round[p], 
            proposal   |-> Proposal(v, t, r), 
            validRound |-> NilRound
          ] 
       IN
      /\ msg \in msgsPropose[round[p]] 
      /\ proposalReceptionTime[r,p] = NilTimestamp
      /\ proposalReceptionTime' = [proposalReceptionTime EXCEPT ![r,p] = localClock[p]]
      /\ UNCHANGED <<temporalVars, coreVars>>
      /\ UNCHANGED
        <<msgsPropose, msgsPrevote, msgsPrecommit, 
        evidence(*, proposalReceptionTime*)>>
      /\ UNCHANGED beginRound
      /\ action' = "ReceiveProposal"
      
\* lines 22-27
\* @type: (PROCESS) => Bool;
UponProposalInPropose(p) ==
  \E v \in Values, t \in Timestamps:
    LET 
      r == round[p]
    IN LET
      \* @type: PROPOSAL;
      prop == Proposal(v,t,r)
    IN
    /\ step[p] = "PROPOSE" (* line 22 *)
    /\ LET 
        \* @type: PROPMESSAGE; 
        msg ==
        [
          type       |-> "PROPOSAL", 
          src        |-> Proposer[r],
          round      |-> r, 
          proposal   |-> prop, 
          validRound |-> NilRound
        ] 
       IN
      /\ evidence' = {msg} \union evidence
    /\ LET mid == (* line 23 *)
         IF 
          \* Timeliness is checked against the process time, as was
          \* recorded in proposalReceptionTime, not as it is now.
          \* In the implementation, if the proposal is not timely, then we prevote
          \* nil. In the natural-language specification, nothing happens. 
          \* This specification maintains consistency with the implementation.
          /\ IsTimely( proposalReceptionTime[r, p], t) \* updated line 22
          /\ IsValid(prop) 
          /\ (lockedRound[p] = NilRound \/ lockedValue[p] = v)
         THEN Id(prop)
         ELSE NilProposal
       IN
       BroadcastPrevote(p, r, mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<temporalVars, beginRound>>
    /\ UNCHANGED
      <<round, (*step,*) decision, lockedValue, 
      lockedRound, validValue, validRound>>
    /\ UNCHANGED 
      <<msgsPropose, (*msgsPrevote,*) msgsPrecommit, 
      (*evidence,*) proposalReceptionTime>>
    /\ action' = "UponProposalInPropose"

\* lines 28-33        
\* [PBTS-ALG-OLD-PREVOTE.0]
\* @type: (PROCESS) => Bool;
UponProposalInProposeAndPrevote(p) ==
  \E v \in Values, t \in Timestamps, vr \in Rounds, pr \in Rounds:
    LET 
      r == round[p]
    IN LET
      \* @type: PROPOSAL;
      prop == Proposal(v,t,pr)
    IN
    /\ step[p] = "PROPOSE" /\ 0 <= vr /\ vr < r \* line 28, the while part
    /\ pr <= vr
    /\ LET
        \* @type: PROPMESSAGE; 
        msg ==
        [
          type       |-> "PROPOSAL", 
          src        |-> Proposer[r],
          round      |-> r, 
          proposal   |-> prop, 
          validRound |-> vr
        ]
       IN
       \* Changed from 001: no need to re-check timeliness
       /\ msg \in msgsPropose[r] \* line 28
       /\ LET PV == { m \in msgsPrevote[vr]: m.id = Id(prop) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 28
          /\ evidence' = PV \union {msg} \union evidence
    /\ LET mid == (* line 29 *)
         IF IsValid(prop) /\ (lockedRound[p] <= vr \/ lockedValue[p] = v)
         THEN Id(prop) 
         ELSE NilProposal 
       IN
       BroadcastPrevote(p, r, mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<temporalVars, beginRound>>
    /\ UNCHANGED
      <<round, (*step,*) decision, lockedValue, 
      lockedRound, validValue, validRound>>
    /\ UNCHANGED 
      <<msgsPropose, (*msgsPrevote,*) msgsPrecommit, 
      (*evidence,*) proposalReceptionTime>>
    /\ action' = "UponProposalInProposeAndPrevote"
                     
\* lines 34-35 + lines 61-64 (onTimeoutPrevote)
\* @type: (PROCESS) => Bool;
UponQuorumOfPrevotesAny(p) ==
  /\ step[p] = "PREVOTE" \* line 34 and 61
  /\ \E MyEvidence \in SUBSET msgsPrevote[round[p]]:
      \* find the unique voters in the evidence
      LET Voters == { m.src: m \in MyEvidence } IN
      \* compare the number of the unique voters against the threshold
      /\ Cardinality(Voters) >= THRESHOLD2 \* line 34
      /\ evidence' = MyEvidence \union evidence
      /\ BroadcastPrecommit(p, round[p], NilProposal)
      /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
      /\ UNCHANGED <<temporalVars, beginRound>>
      /\ UNCHANGED
        <<round, (*step,*) decision, lockedValue, 
        lockedRound, validValue, validRound>>
      /\ UNCHANGED 
        <<msgsPropose, msgsPrevote, (*msgsPrecommit, *)
        (*evidence,*) proposalReceptionTime>>
      /\ action' = "UponQuorumOfPrevotesAny"
                     
\* lines 36-46
\* [PBTS-ALG-NEW-PREVOTE.0]
\* @type: (PROCESS) => Bool;
UponProposalInPrevoteOrCommitAndPrevote(p) ==
  \E v \in ValidValues, t \in Timestamps, vr \in RoundsOrNil:
    LET 
      r == round[p]
    IN LET
      \* @type: PROPOSAL;
      prop == Proposal(v,t,r)
    IN
    /\ step[p] \in {"PREVOTE", "PRECOMMIT"} \* line 36
    /\ LET
        \* @type: PROPMESSAGE; 
        msg ==
        [
          type       |-> "PROPOSAL", 
          src        |-> Proposer[r],
          round      |-> r, 
          proposal   |-> prop, 
          validRound |-> vr
        ] 
       IN 
        \* Changed from 001: no need to re-check timeliness
        /\ msg \in msgsPropose[r] \* line 36
        /\ LET PV == { m \in msgsPrevote[r]: m.id = Id(prop) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 36
          /\ evidence' = PV \union {msg} \union evidence
    /\  IF step[p] = "PREVOTE"
        THEN \* lines 38-41:
          /\ lockedValue' = [lockedValue EXCEPT ![p] = v]
          /\ lockedRound' = [lockedRound EXCEPT ![p] = r]
          /\ BroadcastPrecommit(p, r, Id(prop))
          /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
        ELSE
          UNCHANGED <<lockedValue, lockedRound, msgsPrecommit, step>>
      \* lines 42-43
    /\ validValue' = [validValue EXCEPT ![p] = prop]
    /\ validRound' = [validRound EXCEPT ![p] = r]
    /\ UNCHANGED <<temporalVars, beginRound>>
    /\ UNCHANGED
      <<round, (*step,*) decision(*, lockedValue, 
      lockedRound, validValue, validRound*)>>
    /\ UNCHANGED 
      <<msgsPropose, msgsPrevote, (*msgsPrecommit, *)
      (*evidence,*) proposalReceptionTime>>
    /\ action' = "UponProposalInPrevoteOrCommitAndPrevote"

\* lines 47-48 + 65-67 (onTimeoutPrecommit)
\* @type: (PROCESS) => Bool;
UponQuorumOfPrecommitsAny(p) ==
  /\ \E MyEvidence \in SUBSET msgsPrecommit[round[p]]:
      \* find the unique committers in the evidence
      LET Committers == { m.src: m \in MyEvidence } IN
      \* compare the number of the unique committers against the threshold
      /\ Cardinality(Committers) >= THRESHOLD2 \* line 47
      /\ evidence' = MyEvidence \union evidence
      /\ round[p] + 1 \in Rounds
      /\ StartRound(p, round[p] + 1)
      /\ UNCHANGED temporalVars
      /\ UNCHANGED
        <<(*round, step,*) decision, lockedValue, 
        lockedRound, validValue, validRound>>
      /\ UNCHANGED 
        <<msgsPropose, msgsPrevote, msgsPrecommit,
        (*evidence,*) proposalReceptionTime>>
      /\ action' = "UponQuorumOfPrecommitsAny"
                     
\* lines 49-54        
\* [PBTS-ALG-DECIDE.0]
\* @type: (PROCESS) => Bool;
UponProposalInPrecommitNoDecision(p) ==
  /\ decision[p] = NilDecision \* line 49
  /\ \E v \in ValidValues, t \in Timestamps (* line 50*) , r \in Rounds, pr \in Rounds, vr \in RoundsOrNil:
    LET
      \* @type: PROPOSAL;
      prop == Proposal(v,t,pr)
    IN
    /\ LET
        \* @type: PROPMESSAGE; 
        msg == 
        [
          type       |-> "PROPOSAL", 
          src        |-> Proposer[r],
          round      |-> r, 
          proposal   |-> prop, 
          validRound |-> vr
        ] 
       IN 
     /\ msg \in msgsPropose[r] \* line 49
     /\ proposalReceptionTime[r,p] /= NilTimestamp \* Keep?
     /\ LET PV == { m \in msgsPrecommit[r]: m.id = Id(prop) } IN
       /\ Cardinality(PV) >= THRESHOLD2 \* line 49
       /\ evidence' = PV \union {msg} \union evidence
       /\ decision' = [decision EXCEPT ![p] = Decision(prop, r)] \* update the decision, line 51
    \* The original algorithm does not have 'DECIDED', but it increments the height.
    \* We introduced 'DECIDED' here to prevent the process from changing its decision.
       /\ step' = [step EXCEPT ![p] = "DECIDED"]
       /\ UNCHANGED temporalVars
       /\ UNCHANGED
         <<round, (*step, decision,*) lockedValue, 
         lockedRound, validValue, validRound>>
       /\ UNCHANGED 
         <<msgsPropose, msgsPrevote, msgsPrecommit,
         (*evidence,*) proposalReceptionTime>>
       /\ UNCHANGED beginRound
       /\ action' = "UponProposalInPrecommitNoDecision"
                                                          
\* the actions below are not essential for safety, but added for completeness

\* lines 20-21 + 57-60
\* @type: (PROCESS) => Bool;
OnTimeoutPropose(p) ==
  /\ step[p] = "PROPOSE"
  /\ p /= Proposer[round[p]]
  /\ BroadcastPrevote(p, round[p], NilProposal)
  /\ step' = [step EXCEPT ![p] = "PREVOTE"]
  /\ UNCHANGED <<temporalVars, beginRound>>
  /\ UNCHANGED
    <<round, (*step,*) decision, lockedValue, 
    lockedRound, validValue, validRound>>
  /\ UNCHANGED 
    <<msgsPropose, (*msgsPrevote,*) msgsPrecommit,
    evidence, proposalReceptionTime>>
  /\ action' = "OnTimeoutPropose"

\* lines 44-46
\* @type: (PROCESS) => Bool;
OnQuorumOfNilPrevotes(p) ==
  /\ step[p] = "PREVOTE"
  /\ LET PV == { m \in msgsPrevote[round[p]]: m.id = Id(NilProposal) } IN
    /\ Cardinality(PV) >= THRESHOLD2 \* line 36
    /\ evidence' = PV \union evidence
    /\ BroadcastPrecommit(p, round[p], Id(NilProposal))
    /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
    /\ UNCHANGED <<temporalVars, beginRound>>
    /\ UNCHANGED
      <<round, (*step,*) decision, lockedValue, 
      lockedRound, validValue, validRound>>
    /\ UNCHANGED 
      <<msgsPropose, msgsPrevote, (*msgsPrecommit,*)
      (*evidence,*) proposalReceptionTime>>
    /\ action' = "OnQuorumOfNilPrevotes"

\* lines 55-56
\* @type: (PROCESS) => Bool;
OnRoundCatchup(p) ==
  \E r \in Rounds: 
    /\ r > round[p]
    /\ LET RoundMsgs == msgsPropose[r] \union msgsPrevote[r] \union msgsPrecommit[r] IN
       \E MyEvidence \in SUBSET RoundMsgs:
           LET Faster == { m.src: m \in MyEvidence } IN
           /\ Cardinality(Faster) >= THRESHOLD1
           /\ evidence' = MyEvidence \union evidence
           /\ StartRound(p, r)
           /\ UNCHANGED temporalVars
           /\ UNCHANGED
             <<(*round, step,*) decision, lockedValue, 
             lockedRound, validValue, validRound>>
           /\ UNCHANGED 
             <<msgsPropose, msgsPrevote, msgsPrecommit,
             (*evidence,*) proposalReceptionTime>>
           /\ action' = "OnRoundCatchup"


(********************* PROTOCOL TRANSITIONS ******************************)
\* advance the global clock
\* @type: Bool;
AdvanceRealTime == 
    /\ ValidTime(realTime)
    /\ \E t \in Timestamps:
      /\ t > realTime
      /\ realTime' = t
      /\ localClock' = [p \in Corr |-> localClock[p] + (t - realTime)]  
    /\ UNCHANGED <<coreVars, bookkeepingVars, beginRound>>
    /\ action' = "AdvanceRealTime"

\* process timely messages
\* @type: (PROCESS) => Bool;
MessageProcessing(p) ==
    \* start round
    \/ InsertProposal(p)
    \* reception step
    \/ ReceiveProposal(p)
    \* processing step
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

(*
 * A system transition. In this specificatiom, the system may eventually deadlock,
 * e.g., when all processes decide. This is expected behavior, as we focus on safety.
 *)
Next == 
  \/ AdvanceRealTime
  \/ FaultyBroadcast
  \/ /\ SynchronizedLocalClocks
     /\ \E p \in Corr: MessageProcessing(p)

-----------------------------------------------------------------------------

(*************************** INVARIANTS *************************************)

\* [PBTS-INV-AGREEMENT.0]
AgreementOnValue ==
    \A p, q \in Corr:
        /\ decision[p] /= NilDecision
        /\ decision[q] /= NilDecision
        => \E v \in ValidValues, t \in Timestamps, pr \in Rounds, r1 \in Rounds, r2 \in Rounds : 
            LET prop == Proposal(v,t,pr)
            IN
            /\ decision[p] = Decision(prop, r1)
            /\ decision[q] = Decision(prop, r2)

DisagreementOnValue ==
  \E p, q \in Corr:
    \E p1 \in ValidProposals, p2 \in ValidProposals, r1 \in Rounds, r2 \in Rounds:
    /\ p1 /= p2
    /\ decision[p] = Decision(p1, r1)
    /\ decision[q] = Decision(p2, r2) 

\* [PBTS-INV-VALID.0]
ConsensusValidValue ==
    \A p \in Corr:
        \* decision[p] = Decision(Proposal(v,t,pr), r)
        LET prop == decision[p][1] IN 
          prop /= NilProposal => prop[1] \in ValidValues

\* [PBTS-INV-MONOTONICITY.0]
\* TODO: we would need to compare timestamps of blocks from different height

\* [PBTS-INV-TIMELY.0]
ConsensusTimeValid ==
    \A p \in Corr: 
    \* if a process decides on v and t
      \E v \in ValidValues, t \in Timestamps, pr \in Rounds, dr \in Rounds : 
        \* FIXME: do we need to enforce pr <= dr?
        decision[p] = Decision(Proposal(v,t,pr), dr)
        \* then a process found t timely at its proposal round (pr) 
        => \E q \in Corr:
           LET propRecvTime == proposalReceptionTime[pr, q] IN
           (
             /\ beginRound[pr, q] <= propRecvTime
             /\ beginRound[pr+1, q] >= propRecvTime
             /\ IsTimely(propRecvTime, t)
           )

IsFirstProposedInRound(prop, src, r) ==
  \E msg \in msgsPropose[r]: 
    /\ msg.proposal = prop 
    /\ msg.src = src 
    \* If a proposal is reused this changes from Nil to a valid round
    /\ msg.validRound = NilRound 

TimeLiveness == 
\A r \in Rounds \ {MaxRound}, v \in ValidValues:
  LET p == Proposer[r] IN
  p \in Corr \* Correct process is proposer in round r
  =>  
    \E t \in Timestamps:
      LET prop == Proposal(v,t,r) IN
      (
      /\ IsFirstProposedInRound(prop, p, r) \* p proposes v with some timestamp t in round r
      /\ LET tOffset == t + Delay + Precision IN
        /\ firstBeginRound[r] <= t
        /\ t <= lastBeginRound[r]
        /\ lastBeginRound[r] <= tOffset
        /\ tOffset <= firstBeginRound[r+1]
      ) =>
      \A q \in Corr: 
        \* q eventually decides prop
        LET dq == decision[q] IN
        dq /= NilDecision => dq[1] = prop


\* a conjunction of all invariants
Inv ==
    /\ AgreementOnValue 
    /\ ConsensusValidValue
    /\ ConsensusTimeValid
    /\ TimeLiveness

=============================================================================    
