------------------ MODULE TendermintAccTrace_004_draft -------------------------
(*
  When Apalache is running too slow and we have an idea of a counterexample,
  we use this module to restrict the behaviors only to certain actions.
  Once the whole trace is replayed, the system deadlocks.

  Version 1.

  Igor Konnov, 2020.
 *)

EXTENDS Sequences, Apalache, typedefs, TendermintAcc_004_draft

\* a sequence of action names that should appear in the given order,
\* excluding "Init"
CONSTANT
  \* @type: $trace;
  Trace

VARIABLE
  \* @type: $trace;
  toReplay

TraceInit ==
    /\ toReplay = Trace
    /\ action' := "Init"
    /\ Init

TraceNext ==
    /\ Len(toReplay) > 0
    /\ toReplay' = Tail(toReplay)
    \* Here is the trick. We restrict the action to the expected one,
    \* so the other actions will be pruned
    /\ action' := Head(toReplay)
    /\ Next

================================================================================
