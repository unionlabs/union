-------------------- MODULE TendermintPBT_001_draft ---------------------------
(*
 A TLA+ specification of a simplified Tendermint consensus algorithm, with added clocks 
 and proposer-based timestamps. This TLA+ specification extends and modifies 
 the Tendermint TLA+ specification for fork accountability: 
    https://github.com/cometbft/cometbft/blob/main/spec/light-client/accountability/TendermintAcc_004_draft.tla
 
 * Version 1. A preliminary specification.

 Zarko Milosevic, Igor Konnov, Informal Systems, 2019-2020.
 Ilina Stoilkovska, Josef Widder, Informal Systems, 2021.
 *)

EXTENDS Integers, FiniteSets

(********************* PROTOCOL PARAMETERS **********************************)
CONSTANTS
    Corr,          \* the set of correct processes 
    Faulty,        \* the set of Byzantine processes, may be empty
    N,             \* the total number of processes: correct, defective, and Byzantine
    T,             \* an upper bound on the number of Byzantine processes
    ValidValues,   \* the set of valid values, proposed both by correct and faulty
    InvalidValues, \* the set of invalid values, never proposed by the correct ones
    MaxRound,      \* the maximal round number
    MaxTimestamp,  \* the maximal value of the clock tick
    Delay,         \* message delay
    Precision,     \* clock precision: the maximal difference between two local clocks  
    Accuracy,      \* clock accuracy: the maximal difference between a local clock and the real time
    Proposer,      \* the proposer function from 0..NRounds to 1..N
    ClockDrift     \* is there clock drift between the local clocks and the global clock

ASSUME(N = Cardinality(Corr \union Faulty))

(*************************** DEFINITIONS ************************************)
AllProcs == Corr \union Faulty      \* the set of all processes
Rounds == 0..MaxRound               \* the set of potential rounds
Timestamps == 0..MaxTimestamp       \* the set of clock ticks
NilRound == -1   \* a special value to denote a nil round, outside of Rounds
NilTimestamp == -1 \* a special value to denote a nil timestamp, outside of Ticks
RoundsOrNil == Rounds \union {NilRound}
Values == ValidValues \union InvalidValues \* the set of all values
NilValue == "None"  \* a special value for a nil round, outside of Values
Proposals == Values \X Timestamps
NilProposal == <<NilValue, NilTimestamp>>
ValuesOrNil == Values \union {NilValue}
Decisions == Values \X Timestamps \X Rounds
NilDecision == <<NilValue, NilTimestamp, NilRound>>


\* a value hash is modeled as identity
Id(v) == v

\* The validity predicate
IsValid(v) == v \in ValidValues

\* the two thresholds that are used in the algorithm
THRESHOLD1 == T + 1     \* at least one process is not faulty
THRESHOLD2 == 2 * T + 1 \* a quorum when having N > 3 * T

Min(S) == CHOOSE x \in S : \A y \in S : x <= y

Max(S) == CHOOSE x \in S : \A y \in S : y <= x

(********************* TYPE ANNOTATIONS FOR APALACHE **************************)
\* the operator for type annotations
a <: b == a

\* the type of message records
MT == [type |-> STRING, src |-> STRING, round |-> Int,
       proposal |-> <<STRING, Int>>, validRound |-> Int, id |-> <<STRING, Int>>]

RP == <<STRING, MT>>  

\* a type annotation for a message
AsMsg(m) == m <: MT
\* a type annotation for a set of messages
SetOfMsgs(S) == S <: {MT}       
\* a type annotation for an empty set of messages
EmptyMsgSet == SetOfMsgs({})

SetOfRcvProp(S) == S <: {RP}
EmptyRcvProp == SetOfRcvProp({})

SetOfProc(S) == S <: {STRING}
EmptyProcSet == SetOfProc({})

(********************* PROTOCOL STATE VARIABLES ******************************)
VARIABLES
  round,    \* a process round number: Corr -> Rounds
  localClock,   \* a process local clock: Corr -> Ticks
  realTime, \* a reference Newtonian real time
  step,     \* a process step: Corr -> { "PROPOSE", "PREVOTE", "PRECOMMIT", "DECIDED" }
  decision, \* process decision: Corr -> ValuesOrNil
  lockedValue,  \* a locked value: Corr -> ValuesOrNil
  lockedRound,  \* a locked round: Corr -> RoundsOrNil
  validValue,   \* a valid value: Corr -> ValuesOrNil
  validRound    \* a valid round: Corr -> RoundsOrNil

\* book-keeping variables
VARIABLES
  msgsPropose,   \* PROPOSE messages broadcast in the system, Rounds -> Messages
  msgsPrevote,   \* PREVOTE messages broadcast in the system, Rounds -> Messages
  msgsPrecommit, \* PRECOMMIT messages broadcast in the system, Rounds -> Messages
  receivedTimelyProposal, \* used to keep track when a process receives a timely PROPOSAL message, {<<Corr, Messages>>}
  inspectedProposal, \* used to keep track when a process tries to receive a message, [Rounds -> <<Corr, Messages>>]
  evidence, \* the messages that were used by the correct processes to make transitions
  action,        \* we use this variable to see which action was taken
  beginConsensus, \* the minimum of the local clocks in the initial state, Int
  endConsensus, \* the local time when a decision is made, [Corr -> Int]
  lastBeginConsensus, \* the maximum of the local clocks in the initial state, Int
  proposalTime, \* the real time when a proposer proposes in a round, [Rounds -> Int]
  proposalReceivedTime \* the real time when a correct process first receives a proposal message in a round, [Rounds -> Int]

(* to see a type invariant, check TendermintAccInv3 *)  
 
\* a handy definition used in UNCHANGED
vars == <<round, step, decision, lockedValue, lockedRound,
          validValue, validRound, evidence, msgsPropose, msgsPrevote, msgsPrecommit,
          localClock, realTime, receivedTimelyProposal, inspectedProposal, action,
          beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>

(********************* PROTOCOL INITIALIZATION ******************************)
FaultyProposals(r) ==
    SetOfMsgs([type: {"PROPOSAL"}, src: Faulty,
               round: {r}, proposal: Proposals, validRound: RoundsOrNil])

AllFaultyProposals ==
    SetOfMsgs([type: {"PROPOSAL"}, src: Faulty,
               round: Rounds, proposal: Proposals, validRound: RoundsOrNil])

FaultyPrevotes(r) ==
    SetOfMsgs([type: {"PREVOTE"}, src: Faulty, round: {r}, id: Proposals])

AllFaultyPrevotes ==    
    SetOfMsgs([type: {"PREVOTE"}, src: Faulty, round: Rounds, id: Proposals])

FaultyPrecommits(r) ==
    SetOfMsgs([type: {"PRECOMMIT"}, src: Faulty, round: {r}, id: Proposals])

AllFaultyPrecommits ==
    SetOfMsgs([type: {"PRECOMMIT"}, src: Faulty, round: Rounds, id: Proposals])
   
AllProposals ==
    SetOfMsgs([type: {"PROPOSAL"}, src: AllProcs,
               round: Rounds, proposal: Proposals, validRound: RoundsOrNil])    

RoundProposals(r) ==
    SetOfMsgs([type: {"PROPOSAL"}, src: AllProcs,
               round: {r}, proposal: Proposals, validRound: RoundsOrNil])    

BenignRoundsInMessages(msgfun) ==
  \* the message function never contains a message for a wrong round
  \A r \in Rounds:
    \A m \in msgfun[r]:
      r = m.round

\* The initial states of the protocol. Some faults can be in the system already.
Init ==
    /\ round = [p \in Corr |-> 0]
    /\ \/ /\ ~ClockDrift
          /\ localClock \in [Corr -> 0..Accuracy]
       \/ /\ ClockDrift
          /\ localClock = [p \in Corr |-> 0]
    /\ realTime = 0
    /\ step = [p \in Corr |-> "PROPOSE"]
    /\ decision = [p \in Corr |-> NilDecision]
    /\ lockedValue = [p \in Corr |-> NilValue]
    /\ lockedRound = [p \in Corr |-> NilRound]
    /\ validValue = [p \in Corr |-> NilValue]
    /\ validRound = [p \in Corr |-> NilRound]
    /\ msgsPropose \in [Rounds -> SUBSET AllFaultyProposals]
    /\ msgsPrevote \in [Rounds -> SUBSET AllFaultyPrevotes]
    /\ msgsPrecommit \in [Rounds -> SUBSET AllFaultyPrecommits]
    /\ receivedTimelyProposal = EmptyRcvProp
    /\ inspectedProposal = [r \in Rounds |-> EmptyProcSet]
    /\ BenignRoundsInMessages(msgsPropose)
    /\ BenignRoundsInMessages(msgsPrevote)
    /\ BenignRoundsInMessages(msgsPrecommit)
    /\ evidence = EmptyMsgSet
    /\ action' = "Init"
    /\ beginConsensus = Min({localClock[p] : p \in Corr})
    /\ endConsensus = [p \in Corr |-> NilTimestamp]
    /\ lastBeginConsensus = Max({localClock[p] : p \in Corr})
    /\ proposalTime = [r \in Rounds |-> NilTimestamp]
    /\ proposalReceivedTime = [r \in Rounds |-> NilTimestamp]

(************************ MESSAGE PASSING ********************************)
BroadcastProposal(pSrc, pRound, pProposal, pValidRound) ==
  LET newMsg ==
    AsMsg([type |-> "PROPOSAL", src |-> pSrc, round |-> pRound,
           proposal |-> pProposal, validRound |-> pValidRound])
  IN
  msgsPropose' = [msgsPropose EXCEPT ![pRound] = msgsPropose[pRound] \union {newMsg}]

BroadcastPrevote(pSrc, pRound, pId) ==
  LET newMsg == AsMsg([type |-> "PREVOTE",
                       src |-> pSrc, round |-> pRound, id |-> pId])
  IN
  msgsPrevote' = [msgsPrevote EXCEPT ![pRound] = msgsPrevote[pRound] \union {newMsg}]

BroadcastPrecommit(pSrc, pRound, pId) ==
  LET newMsg == AsMsg([type |-> "PRECOMMIT",
                       src |-> pSrc, round |-> pRound, id |-> pId])
  IN
  msgsPrecommit' = [msgsPrecommit EXCEPT ![pRound] = msgsPrecommit[pRound] \union {newMsg}]


(***************************** TIME **************************************)

\* [PBTS-CLOCK-PRECISION.0]
SynchronizedLocalClocks ==
    \A p \in Corr : \A q \in Corr : 
        p /= q => 
            \/ /\ localClock[p] >= localClock[q]
               /\ localClock[p] - localClock[q] < Precision 
            \/ /\ localClock[p] < localClock[q]
               /\ localClock[q] - localClock[p] < Precision
    
\* [PBTS-PROPOSE.0]
Proposal(v, t) ==
    <<v, t>>

\* [PBTS-DECISION-ROUND.0]
Decision(v, t, r) ==
    <<v, t, r>>

(**************** MESSAGE PROCESSING TRANSITIONS *************************)
\* lines 12-13
StartRound(p, r) ==
   /\ step[p] /= "DECIDED" \* a decided process does not participate in consensus
   /\ round' = [round EXCEPT ![p] = r]
   /\ step' = [step EXCEPT ![p] = "PROPOSE"] 

\* lines 14-19, a proposal may be sent later
InsertProposal(p) == 
  LET r == round[p] IN
  /\ p = Proposer[r]
  /\ step[p] = "PROPOSE"
    \* if the proposer is sending a proposal, then there are no other proposals
    \* by the correct processes for the same round
  /\ \A m \in msgsPropose[r]: m.src /= p
  /\ \E v \in ValidValues: 
      LET proposal == IF validValue[p] /= NilValue 
                      THEN Proposal(validValue[p], localClock[p])  
                      ELSE Proposal(v, localClock[p]) IN
                     
      /\ BroadcastProposal(p, round[p], proposal, validRound[p])
      /\ proposalTime' = [proposalTime EXCEPT ![r] = realTime]
  /\ UNCHANGED <<evidence, round, decision, lockedValue, lockedRound,
                validValue, step, validRound, msgsPrevote, msgsPrecommit,
                localClock, realTime, receivedTimelyProposal, inspectedProposal,
                beginConsensus, endConsensus, lastBeginConsensus, proposalReceivedTime>>
  /\ action' = "InsertProposal"
  
\* a new action used to filter messages that are not on time
\* [PBTS-RECEPTION-STEP.0]
ReceiveProposal(p) ==
  \E v \in Values, t \in Timestamps:
    /\ LET r == round[p] IN
       LET msg ==
        AsMsg([type |-> "PROPOSAL", src |-> Proposer[round[p]],
               round |-> round[p], proposal |-> Proposal(v, t), validRound |-> NilRound]) IN
      /\ msg \in msgsPropose[round[p]] 
      /\ p \notin inspectedProposal[r]
      /\ <<p, msg>> \notin receivedTimelyProposal
      /\ inspectedProposal' = [inspectedProposal EXCEPT ![r] = @ \union {p}]
      /\ \/ /\ localClock[p] - Precision < t 
            /\ t < localClock[p] + Precision + Delay
            /\ receivedTimelyProposal' = receivedTimelyProposal \union {<<p, msg>>}
            /\ \/ /\ proposalReceivedTime[r] = NilTimestamp
                  /\ proposalReceivedTime' = [proposalReceivedTime EXCEPT ![r] = realTime]
               \/ /\ proposalReceivedTime[r] /= NilTimestamp
                  /\ UNCHANGED proposalReceivedTime
         \/ /\ \/ localClock[p] - Precision >= t
               \/ t >= localClock[p] + Precision + Delay
            /\ UNCHANGED <<receivedTimelyProposal, proposalReceivedTime>>
      /\ UNCHANGED <<round, step, decision, lockedValue, lockedRound,
          validValue, validRound, evidence, msgsPropose, msgsPrevote, msgsPrecommit,
          localClock, realTime, beginConsensus, endConsensus, lastBeginConsensus, proposalTime>>
      /\ action' = "ReceiveProposal"
      
\* lines 22-27
UponProposalInPropose(p) ==
  \E v \in Values, t \in Timestamps:
    /\ step[p] = "PROPOSE" (* line 22 *)
    /\ LET msg ==
        AsMsg([type |-> "PROPOSAL", src |-> Proposer[round[p]],
               round |-> round[p], proposal |-> Proposal(v, t), validRound |-> NilRound]) IN
      /\ <<p, msg>> \in receivedTimelyProposal \* updated line 22
      /\ evidence' = {msg} \union evidence
    /\ LET mid == (* line 23 *)
         IF IsValid(v) /\ (lockedRound[p] = NilRound \/ lockedValue[p] = v)
         THEN Id(Proposal(v, t))
         ELSE NilProposal
       IN
       BroadcastPrevote(p, round[p], mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                   validValue, validRound, msgsPropose, msgsPrecommit,
                   localClock, realTime, receivedTimelyProposal, inspectedProposal,
                   beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
    /\ action' = "UponProposalInPropose"

\* lines 28-33        
\* [PBTS-ALG-OLD-PREVOTE.0]
UponProposalInProposeAndPrevote(p) ==
  \E v \in Values, t1 \in Timestamps, t2 \in Timestamps, vr \in Rounds:
    /\ step[p] = "PROPOSE" /\ 0 <= vr /\ vr < round[p] \* line 28, the while part
    /\ LET msg ==
         AsMsg([type |-> "PROPOSAL", src |-> Proposer[round[p]],
                round |-> round[p], proposal |-> Proposal(v, t1), validRound |-> vr])
       IN
       /\ <<p, msg>> \in receivedTimelyProposal \* updated line 28
       /\ LET PV == { m \in msgsPrevote[vr]: m.id = Id(Proposal(v, t2)) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 28
          /\ evidence' = PV \union {msg} \union evidence
    /\ LET mid == (* line 29 *)
         IF IsValid(v) /\ (lockedRound[p] <= vr \/ lockedValue[p] = v)
         THEN Id(Proposal(v, t1)) 
         ELSE NilProposal 
       IN
       BroadcastPrevote(p, round[p], mid) \* lines 24-26
    /\ step' = [step EXCEPT ![p] = "PREVOTE"]
    /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                   validValue, validRound, msgsPropose, msgsPrecommit,
                   localClock, realTime, receivedTimelyProposal, inspectedProposal,
                   beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
    /\ action' = "UponProposalInProposeAndPrevote"
                     
 \* lines 34-35 + lines 61-64 (onTimeoutPrevote)
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
      /\ UNCHANGED <<round, decision, lockedValue, lockedRound,
                    validValue, validRound, msgsPropose, msgsPrevote,
                    localClock, realTime, receivedTimelyProposal, inspectedProposal,
                    beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
      /\ action' = "UponQuorumOfPrevotesAny"
                     
\* lines 36-46
\* [PBTS-ALG-NEW-PREVOTE.0]
UponProposalInPrevoteOrCommitAndPrevote(p) ==
  \E v \in ValidValues, t \in Timestamps, vr \in RoundsOrNil:
    /\ step[p] \in {"PREVOTE", "PRECOMMIT"} \* line 36
    /\ LET msg ==
         AsMsg([type |-> "PROPOSAL", src |-> Proposer[round[p]],
                round |-> round[p], proposal |-> Proposal(v, t), validRound |-> vr]) IN
        /\ <<p, msg>> \in receivedTimelyProposal \* updated line 36
        /\ LET PV == { m \in msgsPrevote[round[p]]: m.id = Id(Proposal(v, t)) } IN
          /\ Cardinality(PV) >= THRESHOLD2 \* line 36
          /\ evidence' = PV \union {msg} \union evidence
    /\  IF step[p] = "PREVOTE"
        THEN \* lines 38-41:
          /\ lockedValue' = [lockedValue EXCEPT ![p] = v]
          /\ lockedRound' = [lockedRound EXCEPT ![p] = round[p]]
          /\ BroadcastPrecommit(p, round[p], Id(Proposal(v, t)))
          /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
        ELSE
          UNCHANGED <<lockedValue, lockedRound, msgsPrecommit, step>>
      \* lines 42-43
    /\ validValue' = [validValue EXCEPT ![p] = v]
    /\ validRound' = [validRound EXCEPT ![p] = round[p]]
    /\ UNCHANGED <<round, decision, msgsPropose, msgsPrevote,
                  localClock, realTime, receivedTimelyProposal, inspectedProposal,
                  beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
    /\ action' = "UponProposalInPrevoteOrCommitAndPrevote"

\* lines 47-48 + 65-67 (onTimeoutPrecommit)
UponQuorumOfPrecommitsAny(p) ==
  /\ \E MyEvidence \in SUBSET msgsPrecommit[round[p]]:
      \* find the unique committers in the evidence
      LET Committers == { m.src: m \in MyEvidence } IN
      \* compare the number of the unique committers against the threshold
      /\ Cardinality(Committers) >= THRESHOLD2 \* line 47
      /\ evidence' = MyEvidence \union evidence
      /\ round[p] + 1 \in Rounds
      /\ StartRound(p, round[p] + 1)   
      /\ UNCHANGED <<decision, lockedValue, lockedRound, validValue,
                    validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                    localClock, realTime, receivedTimelyProposal, inspectedProposal,
                    beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
      /\ action' = "UponQuorumOfPrecommitsAny"
                     
\* lines 49-54        
\* [PBTS-ALG-DECIDE.0]
UponProposalInPrecommitNoDecision(p) ==
  /\ decision[p] = NilDecision \* line 49
  /\ \E v \in ValidValues, t \in Timestamps (* line 50*) , r \in Rounds, vr \in RoundsOrNil:
    /\ LET msg == AsMsg([type |-> "PROPOSAL", src |-> Proposer[r],
                           round |-> r, proposal |-> Proposal(v, t), validRound |-> vr]) IN
     /\ msg \in msgsPropose[r] \* line 49
     /\ p \in inspectedProposal[r]
     /\ LET PV == { m \in msgsPrecommit[r]: m.id = Id(Proposal(v, t)) } IN
       /\ Cardinality(PV) >= THRESHOLD2 \* line 49
       /\ evidence' = PV \union {msg} \union evidence
       /\ decision' = [decision EXCEPT ![p] = Decision(v, t, round[p])] \* update the decision, line 51
    \* The original algorithm does not have 'DECIDED', but it increments the height.
    \* We introduced 'DECIDED' here to prevent the process from changing its decision.
       /\ endConsensus' = [endConsensus EXCEPT ![p] = localClock[p]]
       /\ step' = [step EXCEPT ![p] = "DECIDED"]
       /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                     validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                     localClock, realTime, receivedTimelyProposal, inspectedProposal,
                     beginConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
       /\ action' = "UponProposalInPrecommitNoDecision"
                                                          
\* the actions below are not essential for safety, but added for completeness

\* lines 20-21 + 57-60
OnTimeoutPropose(p) ==
  /\ step[p] = "PROPOSE"
  /\ p /= Proposer[round[p]]
  /\ BroadcastPrevote(p, round[p], NilProposal)
  /\ step' = [step EXCEPT ![p] = "PREVOTE"]
  /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                validRound, decision, evidence, msgsPropose, msgsPrecommit,
                localClock, realTime, receivedTimelyProposal, inspectedProposal,
                beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
  /\ action' = "OnTimeoutPropose"

\* lines 44-46
OnQuorumOfNilPrevotes(p) ==
  /\ step[p] = "PREVOTE"
  /\ LET PV == { m \in msgsPrevote[round[p]]: m.id = Id(NilProposal) } IN
    /\ Cardinality(PV) >= THRESHOLD2 \* line 36
    /\ evidence' = PV \union evidence
    /\ BroadcastPrecommit(p, round[p], Id(NilProposal))
    /\ step' = [step EXCEPT ![p] = "PRECOMMIT"]
    /\ UNCHANGED <<round, lockedValue, lockedRound, validValue,
                  validRound, decision, msgsPropose, msgsPrevote,
                  localClock, realTime, receivedTimelyProposal, inspectedProposal,
                  beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
    /\ action' = "OnQuorumOfNilPrevotes"

\* lines 55-56
OnRoundCatchup(p) ==
  \E r \in {rr \in Rounds: rr > round[p]}:
    LET RoundMsgs == msgsPropose[r] \union msgsPrevote[r] \union msgsPrecommit[r] IN
    \E MyEvidence \in SUBSET RoundMsgs:
        LET Faster == { m.src: m \in MyEvidence } IN
        /\ Cardinality(Faster) >= THRESHOLD1
        /\ evidence' = MyEvidence \union evidence
        /\ StartRound(p, r)
        /\ UNCHANGED <<decision, lockedValue, lockedRound, validValue,
                      validRound, msgsPropose, msgsPrevote, msgsPrecommit,
                      localClock, realTime, receivedTimelyProposal, inspectedProposal,
                      beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
        /\ action' = "OnRoundCatchup"


(********************* PROTOCOL TRANSITIONS ******************************)
\* advance the global clock
AdvanceRealTime == 
    /\ realTime < MaxTimestamp
    /\ realTime' = realTime + 1
    /\ \/ /\ ~ClockDrift
          /\ localClock' = [p \in Corr |-> localClock[p] + 1]  
       \/ /\ ClockDrift
          /\ UNCHANGED localClock
    /\ UNCHANGED <<round, step, decision, lockedValue, lockedRound,
                  validValue, validRound, evidence, msgsPropose, msgsPrevote, msgsPrecommit,
                  localClock, receivedTimelyProposal, inspectedProposal,
                  beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>> 
    /\ action' = "AdvanceRealTime"
    
\* advance the local clock of node p
AdvanceLocalClock(p) ==
    /\ localClock[p] < MaxTimestamp
    /\ localClock' = [localClock EXCEPT ![p] = @ + 1]
    /\ UNCHANGED <<round, step, decision, lockedValue, lockedRound,
                  validValue, validRound, evidence, msgsPropose, msgsPrevote, msgsPrecommit,
                  realTime, receivedTimelyProposal, inspectedProposal,
                  beginConsensus, endConsensus, lastBeginConsensus, proposalTime, proposalReceivedTime>>
    /\ action' = "AdvanceLocalClock"

\* process timely messages
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
  \/ /\ ClockDrift
     /\ \E p \in Corr: AdvanceLocalClock(p)
  \/ /\ SynchronizedLocalClocks
     /\ \E p \in Corr: MessageProcessing(p)

-----------------------------------------------------------------------------

(*************************** INVARIANTS *************************************)

\* [PBTS-INV-AGREEMENT.0]
AgreementOnValue ==
    \A p, q \in Corr:
        /\ decision[p] /= NilDecision
        /\ decision[q] /= NilDecision
        => \E v \in ValidValues, t1 \in Timestamps, t2 \in Timestamps, r1 \in Rounds, r2 \in Rounds : 
            /\ decision[p] = Decision(v, t1, r1)
            /\ decision[q] = Decision(v, t2, r2)

\* [PBTS-INV-TIME-AGR.0]
AgreementOnTime ==
    \A p, q \in Corr: 
        \A v1 \in ValidValues, v2 \in ValidValues, t1 \in Timestamps, t2 \in Timestamps, r \in Rounds :
            /\ decision[p] = Decision(v1, t1, r)
            /\ decision[q] = Decision(v2, t2, r)
            => t1 = t2

\* [PBTS-CONSENSUS-TIME-VALID.0]
ConsensusTimeValid ==
    \A p \in Corr, t \in Timestamps : 
    \* if a process decides on v and t
    (\E v \in ValidValues, r \in Rounds : decision[p] = Decision(v, t, r))
        \* then 
        => /\ beginConsensus - Precision <= t 
           /\ t < endConsensus[p] + Precision + Delay

\* [PBTS-CONSENSUS-SAFE-VALID-CORR-PROP.0]
ConsensusSafeValidCorrProp ==
    \A v \in ValidValues, t \in Timestamps :
        \* if the proposer in the first round is correct
       (/\ Proposer[0] \in Corr
        \* and there exists a process that decided on v, t 
        /\ \E p \in Corr, r \in Rounds : decision[p] = Decision(v, t, r))
            \* then t is between the minimal and maximal initial local time
            => /\ beginConsensus <= t 
               /\ t <= lastBeginConsensus

\* [PBTS-CONSENSUS-REALTIME-VALID-CORR.0]
ConsensusRealTimeValidCorr ==
    \A t \in Timestamps, r \in Rounds :
       (/\ \E p \in Corr, v \in ValidValues : decision[p] = Decision(v, t, r) 
        /\ proposalTime[r] /= NilTimestamp)
        => /\ proposalTime[r] - Accuracy < t
           /\ t < proposalTime[r] + Accuracy

\* [PBTS-CONSENSUS-REALTIME-VALID.0]
ConsensusRealTimeValid ==
    \A t \in Timestamps, r \in Rounds :
       (\E p \in Corr, v \in ValidValues : decision[p] = Decision(v, t, r)) 
        => /\ proposalReceivedTime[r] - Accuracy - Precision < t
           /\ t < proposalReceivedTime[r] + Accuracy + Precision + Delay

\* [PBTS-MSG-FAIR.0]
BoundedDelay ==
    \A r \in Rounds : 
        (/\ proposalTime[r] /= NilTimestamp
         /\ proposalTime[r] + Delay < realTime)
            => inspectedProposal[r] = Corr

\* [PBTS-CONSENSUS-TIME-LIVE.0]
ConsensusTimeLive ==
    \A r \in Rounds, p \in Corr : 
       (/\ proposalTime[r] /= NilTimestamp
        /\ proposalTime[r] + Delay < realTime 
        /\ Proposer[r] \in Corr
        /\ round[p] <= r)
            => \E msg \in RoundProposals(r) : <<p, msg>> \in receivedTimelyProposal

\* a conjunction of all invariants
Inv ==
    /\ AgreementOnValue 
    /\ AgreementOnTime
    /\ ConsensusTimeValid
    /\ ConsensusSafeValidCorrProp
    /\ ConsensusRealTimeValid
    /\ ConsensusRealTimeValidCorr
    /\ BoundedDelay

Liveness ==
    ConsensusTimeLive    

=============================================================================    
