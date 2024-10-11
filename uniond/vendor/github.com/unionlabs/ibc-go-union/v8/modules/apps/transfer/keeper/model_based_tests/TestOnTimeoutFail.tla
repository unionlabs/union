------------------------- MODULE counterexample -------------------------

EXTENDS relay_tests

(* Initial state *)

State1 ==
TRUE
(* Transition 0 to State2 *)

State2 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
/\ count = 0
/\ error = FALSE
/\ handler = ""
/\ history = 0
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> FALSE,
      handler |-> "",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "channel-0"],
                  prefix1 |-> [channel |-> "transfer", port |-> "cosmos-hub"]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "",
          destPort |-> "",
          sourceChannel |-> "",
          sourcePort |-> ""]]
/\ p = [data |->
    [amount |-> 1,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "cosmos-hub", port |-> "channel-0"],
          prefix1 |-> [channel |-> "transfer", port |-> "cosmos-hub"]],
      receiver |-> "a2",
      sender |-> "a1"],
  destChannel |-> "",
  destPort |-> "",
  sourceChannel |-> "",
  sourcePort |-> ""]

(* Transition 6 to State3 *)

State3 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
/\ count = 1
/\ error = TRUE
/\ handler = "OnTimeoutPacket"
/\ history = 0
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> FALSE,
      handler |-> "",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "channel-0"],
                  prefix1 |-> [channel |-> "transfer", port |-> "cosmos-hub"]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "",
          destPort |-> "",
          sourceChannel |-> "",
          sourcePort |-> ""]]
  @@ 1
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> TRUE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "channel-0"],
                  prefix1 |-> [channel |-> "transfer", port |-> "cosmos-hub"]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "",
          destPort |-> "",
          sourceChannel |-> "",
          sourcePort |-> ""]]
/\ p = [data |->
    [amount |-> 0,
      denomTrace |->
        [denom |-> "",
          prefix0 |-> [channel |-> "", port |-> ""],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "",
      sender |-> ""],
  destChannel |-> "",
  destPort |-> "",
  sourceChannel |-> "",
  sourcePort |-> ""]

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  BMC!Skolem((\E s$2 \in DOMAIN history:
    history[s$2]["handler"] = "OnTimeoutPacket"
      /\ history[s$2]["error"] = TRUE
      /\ history[s$2]["packet"]["data"]["amount"] > 0))

================================================================================
\* Created by Apalache on Thu Dec 10 11:09:25 CET 2020
\* https://github.com/informalsystems/apalache
