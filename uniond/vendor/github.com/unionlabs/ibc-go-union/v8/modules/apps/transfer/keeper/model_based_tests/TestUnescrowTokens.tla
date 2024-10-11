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
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 5,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "", port |-> ""],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a3",
      sender |-> "a1"],
  destChannel |-> "channel-0",
  destPort |-> "transfer",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* Transition 3 to State3 *)

State3 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 5
/\ count = 1
/\ error = FALSE
/\ handler = "OnRecvPacket"
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
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 1
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 5,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 3,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a1",
      sender |-> "a3"],
  destChannel |-> "channel-0",
  destPort |-> "transfer",
  sourceChannel |-> "channel-1",
  sourcePort |-> "transfer"]

(* Transition 1 to State4 *)

State4 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
  @@ <<
    [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 3
/\ count = 2
/\ error = FALSE
/\ handler = "SendTransfer"
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
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 1
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 5,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 2
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2
          @@ <<
            [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |->
                "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 5,
      error |-> FALSE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a1",
              sender |-> "a3"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-1",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 1,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
          prefix1 |-> [channel |-> "channel-0", port |-> "transfer"]],
      receiver |-> "a1",
      sender |-> "a1"],
  destChannel |-> "channel-1",
  destPort |-> "transfer",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* Transition 4 to State5 *)

State5 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a1", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 1
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
  @@ <<
    [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |-> "atom",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
/\ count = 3
/\ error = FALSE
/\ handler = "OnRecvPacket"
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
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 1
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 5,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 5,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 2
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2
          @@ <<
            [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |->
                "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 5,
      error |-> FALSE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a1",
              sender |-> "a3"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-1",
          sourcePort |-> "transfer"]]
  @@ 3
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a1", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 1
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2
          @@ <<
            [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |->
                "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2
          @@ <<
            [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |->
                "atom",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "channel-0", port |-> "transfer"]],
              receiver |-> "a1",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 0,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "transfer", port |-> "bitcoin-hub"],
          prefix1 |-> [channel |-> "channel-0", port |-> "channel-1"]],
      receiver |-> "a1",
      sender |-> ""],
  destChannel |-> "ethereum-hub",
  destPort |-> "cosmos-hub",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  history[1]["handler"] = "OnRecvPacket"
    /\ BMC!Skolem((\E s$2 \in DOMAIN history:
      ((IF history[s$2]["packet"]["data"]["denomTrace"]["prefix0"]
            = [port |-> "", channel |-> ""]
          THEN [port |-> "", channel |-> ""]
          ELSE IF history[s$2]["packet"]["data"]["denomTrace"]["prefix1"]
            = [port |-> "", channel |-> ""]
          THEN history[s$2]["packet"]["data"]["denomTrace"]["prefix0"]
          ELSE history[s$2]["packet"]["data"]["denomTrace"]["prefix1"])[
            "port"
          ]
            = history[s$2]["packet"]["sourcePort"]
          /\ (IF history[s$2]["packet"]["data"]["denomTrace"]["prefix0"]
            = [port |-> "", channel |-> ""]
          THEN [port |-> "", channel |-> ""]
          ELSE IF history[s$2]["packet"]["data"]["denomTrace"]["prefix1"]
            = [port |-> "", channel |-> ""]
          THEN history[s$2]["packet"]["data"]["denomTrace"]["prefix0"]
          ELSE history[s$2]["packet"]["data"]["denomTrace"]["prefix1"])[
            "channel"
          ]
            = history[s$2]["packet"]["sourceChannel"])
        /\ history[s$2]["handler"] = "OnRecvPacket"
        /\ history[s$2]["error"] = FALSE))

================================================================================
\* Created by Apalache on Thu Dec 10 13:38:11 CET 2020
\* https://github.com/informalsystems/apalache
