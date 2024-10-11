------------------------- MODULE MC4_6_faulty ---------------------------

AllNodes == {"n1", "n2", "n3", "n4"}
TRUSTED_HEIGHT == 1
TARGET_HEIGHT == 6
TRUSTING_PERIOD == 1400 \* two weeks, one day is 100 time units :-)
IS_PRCLOCK_DRIFT == 10       \* how much we assume the local clock is drifting
REAL_CLOCK_DRIFT == 3   \* how much the local clock is actually drifting
IMARY_CORRECT == FALSE
\* @type: <<Int, Int>>;
FAULTY_RATIO == <<1, 3>>    \* < 1 / 3 faulty validators

VARIABLES
  \* @type: Str;
  state,
  \* @type: Int;
  nextHeight,
  \* @type: Int -> $lightHeader;
  fetchedLightBlocks,
  \* @type: Int -> Str;
  lightBlockStatus,
  \* @type: $lightHeader;
  latestVerified,
  \* @type: Int;
  nprobes,
  \* @type: Int;
  localClock,
  \* @type: Int;
  refClock,
  \* @type: Int -> $header;
  blockchain,
  \* @type: Set($node);
  Faulty

(* the light client previous state components, used for monitoring *)
VARIABLES
  \* @type: $lightHeader;
  prevVerified,
  \* @type: $lightHeader;
  prevCurrent,
  \* @type: Int;
  prevLocalClock,
  \* @type: Str;
  prevVerdict

INSTANCE Lightclient_003_draft
============================================================================
