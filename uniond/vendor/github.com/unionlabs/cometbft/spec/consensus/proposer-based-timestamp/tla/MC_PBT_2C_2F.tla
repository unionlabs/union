----------------------------- MODULE MC_PBT_2C_2F -------------------------------
CONSTANT 
  \* @type: ROUND -> PROCESS;
  Proposer

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

\* time-related variables
VARIABLES  
  \* @type: PROCESS -> TIME;
  localClock, \* a process local clock: Corr -> Ticks
  \* @type: TIME;
  realTime   \* a reference Newtonian real time

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
  
\* Invariant support
VARIABLES
  \* @type: <<ROUND, PROCESS>> -> TIME;
  beginRound \* the minimum of the local clocks at the time any process entered a new round

INSTANCE TendermintPBT WITH
  Corr <- {"c1", "c2"},
  Faulty <- {"f3", "f4"},
  N <- 4,
  T <- 1,
  ValidValues <- { "v0", "v1" },
  InvalidValues <- {"v2"},
  MaxRound <- 3,
  MaxTimestamp <- 7,
  MinTimestamp <- 2,
  Delay <- 2,        
  Precision <- 2,
  PreloadAllFaultyMsgs <- TRUE,
  N_GEN <- 5

\* run Apalache with --cinit=CInit
CInit == \* the proposer is arbitrary -- works for safety
  ArbitraryProposer

=============================================================================    
