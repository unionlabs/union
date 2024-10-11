----------------------------- MODULE MC_n5_f1 -------------------------------
CONSTANT
  \* @type: $round -> $process;
  Proposer

\* the variables declared in TendermintAcc3
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
  validRound,   \* a valid round: Corr -> RoundsOrNil
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

INSTANCE TendermintAccDebug_004_draft WITH
  Corr <- {"c1", "c2", "c3", "c4"},
  Faulty <- {"f5"},
  N <- 5,
  T <- 1,
  ValidValues <- { "v0", "v1" },
  InvalidValues <- {"v2"},
  MaxRound <- 2

\* run Apalache with --cinit=ConstInit
ConstInit == \* the proposer is arbitrary -- works for safety
  Proposer \in [Rounds -> AllProcs]

=============================================================================
