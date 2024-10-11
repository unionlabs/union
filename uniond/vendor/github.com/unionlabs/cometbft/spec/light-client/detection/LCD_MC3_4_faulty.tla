------------------------- MODULE LCD_MC3_4_faulty ---------------------------

AllNodes == {"n1", "n2", "n3"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 4
TRUSTING_PERIOD == 1400     \* two weeks, one day is 100 time units :-)
CLOCK_DRIFT == 10       \* how much we assume the local clock is drifting
REAL_CLOCK_DRIFT == 3   \* how much the local clock is actually drifting
IS_PRIMARY_CORRECT == FALSE
IS_SECONDARY_CORRECT == TRUE
FAULTY_RATIO == <<2, 3>>    \* < 1 / 3 faulty validators

VARIABLES
  blockchain,           (* the reference blockchain *)
  localClock,           (* current time in the light client *)
  refClock,             (* current time in the reference blockchain *)
  Faulty,               (* the set of faulty validators *)
  state,                (* the state of the light client detector *)
  fetchedLightBlocks1,  (* a function from heights to LightBlocks *)
  fetchedLightBlocks2,  (* a function from heights to LightBlocks *)
  fetchedLightBlocks1b, (* a function from heights to LightBlocks *)
  commonHeight,         (* the height that is trusted in CreateEvidenceForPeer *)
  nextHeightToTry,      (* the index in CreateEvidenceForPeer *)
  evidences

INSTANCE LCDetector_003_draft
============================================================================
