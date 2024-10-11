-------------------------- MODULE denom_record2 ----------------------------

(**
   The implementation of denomination traces that allows one- or two-step sequences
   Represented via records
*)

EXTENDS identifiers

CONSTANT
  Denoms

MaxDenomLength == 5

DenomPrefixes == [
  port: Identifiers,
  channel: Identifiers
]

NullDenomPrefix == [
  port |-> NullId,
  channel |-> NullId
]

MakeDenomPrefix(port, channel) == [
    port |-> port,
    channel |-> channel
]

IsValidDenomPrefix(prefix) ==
  /\ prefix.port /= NullId
  /\ prefix.channel /= NullId

DenomTraces == [
  prefix1: DenomPrefixes, \* the most recent prefix
  prefix0: DenomPrefixes, \* the deepest prefix
  denom: Denoms
]

NullDenomTrace == [
  prefix1 |-> NullDenomPrefix,
  prefix0 |-> NullDenomPrefix,
  denom |-> NullId
]


TraceLen(trace) ==
   IF trace.prefix0 = NullDenomPrefix
   THEN 1
   ELSE IF trace.prefix1 = NullDenomPrefix
   THEN 3
   ELSE 5

LatestPrefix(trace) ==
   IF trace.prefix0 = NullDenomPrefix
   THEN NullDenomPrefix
   ELSE IF trace.prefix1 = NullDenomPrefix
   THEN  trace.prefix0
   ELSE trace.prefix1


ExtendDenomTrace(port, channel, trace) ==
  IF trace.prefix0 = NullDenomPrefix
  THEN [
      prefix1 |-> NullDenomPrefix,
      prefix0 |-> MakeDenomPrefix(port, channel),
      denom |-> trace.denom
    ]
  ELSE IF trace.prefix1 = NullDenomPrefix
  THEN [
        prefix1 |-> MakeDenomPrefix(port, channel),
        prefix0 |-> trace.prefix0,
        denom |-> trace.denom
    ]
  ELSE NullDenomTrace \* can extend only for two steps

ReduceDenomTrace(trace) ==
  IF trace.prefix1 /= NullDenomPrefix
  THEN [
      prefix1 |-> NullDenomPrefix,
      prefix0 |-> trace.prefix0,
      denom |-> trace.denom
    ]
  ELSE IF trace.prefix0 /= NullDenomPrefix
  THEN [
        prefix1 |-> NullDenomPrefix,
        prefix0 |-> NullDenomPrefix,
        denom |-> trace.denom
    ]
  ELSE NullDenomTrace \* cannot reduce further

GetPort(trace) == LatestPrefix(trace).port
GetChannel(trace) == LatestPrefix(trace).channel
GetDenom(trace) == trace.denom

IsValidDenomTrace(trace) ==
  /\ GetDenom(trace) /= NullId
  /\ IF IsValidDenomPrefix(trace.prefix1)
     THEN IsValidDenomPrefix(trace.prefix0)
     ELSE
       /\ trace.prefix1 = NullDenomPrefix
       /\ (IsValidDenomPrefix(trace.prefix0) \/ trace.prefix0 = NullDenomPrefix)

IsNativeDenomTrace(trace) == LatestPrefix(trace) = NullDenomPrefix /\ GetDenom(trace) /= NullId
IsPrefixedDenomTrace(trace) == LatestPrefix(trace) /= NullDenomPrefix /\ GetDenom(trace) /= NullId

DENOM == INSTANCE denom
DenomTypeOK == DENOM!DenomTypeOK


=============================================================================
\* Modification History
\* Last modified Fri Dec 04 10:38:10 CET 2020 by andrey
\* Created Fri Dec 04 10:22:10 CET 2020 by andrey
