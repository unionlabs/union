-------------------------- MODULE denom_record ----------------------------

(**
   The most basic implementation of denomination traces that allows only one-step sequences
   Represented via records
*)

EXTENDS identifiers

CONSTANT
  Denoms

MaxDenomLength == 3

DenomTraces == [
  port: Identifiers,
  channel: Identifiers,
  denom: Denoms
]

NullDenomTrace == [
  port |-> NullId,
  channel |-> NullId,
  denom |-> NullId
]

GetPort(trace) == trace.port
GetChannel(trace) == trace.channel
GetDenom(trace) == trace.denom

IsNativeDenomTrace(trace) == GetPort(trace) = NullId /\ GetChannel(trace) = NullId /\ GetDenom(trace) /= NullId
IsPrefixedDenomTrace(trace) == GetPort(trace) /= NullId /\ GetChannel(trace) /= NullId /\ GetDenom(trace) /= NullId

ExtendDenomTrace(port, channel, trace) ==
  IF GetPort(trace) = NullId /\ GetChannel(trace) = NullId
  THEN
      [
        port |-> port,
        channel |-> channel,
        denom |-> trace.denom
      ]
  ELSE
      NullDenomTrace


DENOM == INSTANCE denom
DenomTypeOK == DENOM!DenomTypeOK


=============================================================================
\* Modification History
\* Last modified Thu Nov 05 16:41:47 CET 2020 by andrey
\* Created Thu Nov 05 13:22:40 CET 2020 by andrey
