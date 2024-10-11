------------------------- MODULE MC_5_3 -------------------------------------

AllNodes == {"n1", "n2", "n3", "n4", "n5"}
COMMON_HEIGHT == 1
CONFLICT_HEIGHT == 3
TRUSTING_PERIOD == 1400     \* two weeks, one day is 100 time units :-)
FAULTY_RATIO == <<1, 2>>    \* < 1 / 2 faulty validators

VARIABLES
  blockchain,           \* the reference blockchain
  refClock,             \* current time in the reference blockchain
  Faulty,               \* the set of faulty validators
  state,                \* the state of the light client detector
  conflictingBlock,     \* an evidence that two peers reported conflicting blocks
  attackers

INSTANCE Isolation_001_draft
============================================================================
