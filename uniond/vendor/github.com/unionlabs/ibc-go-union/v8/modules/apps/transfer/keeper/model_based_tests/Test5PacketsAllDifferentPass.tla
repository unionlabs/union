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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 3,
      denomTrace |->
        [denom |-> "eth",
          prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a2",
      sender |-> "a3"],
  destChannel |-> "channel-0",
  destPort |-> "transfer",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* Transition 6 to State3 *)

State3 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 3
/\ count = 1
/\ error = FALSE
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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            :> 0,
      error |-> FALSE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 3,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
          prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]],
      receiver |-> "a1",
      sender |-> "a2"],
  destChannel |-> "channel-0",
  destPort |-> "transfer",
  sourceChannel |-> "channel-1",
  sourcePort |-> "transfer"]

(* Transition 10 to State4 *)

State4 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 3
/\ count = 2
/\ error = FALSE
/\ handler = "OnRecvAcknowledgementError"
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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            :> 0,
      error |-> FALSE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementError",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]],
              receiver |-> "a1",
              sender |-> "a2"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-1",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 3,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a2",
      sender |-> "a1"],
  destChannel |-> "channel-1",
  destPort |-> "transfer",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* Transition 5 to State5 *)

State5 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 3
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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            :> 0,
      error |-> FALSE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementError",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]],
              receiver |-> "a1",
              sender |-> "a2"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 2,
      denomTrace |->
        [denom |-> "",
          prefix0 |-> [channel |-> "channel-1", port |-> "channel-1"],
          prefix1 |-> [channel |-> "channel-0", port |-> ""]],
      receiver |-> "",
      sender |-> "a1"],
  destChannel |-> "transfer",
  destPort |-> "cosmos-hub",
  sourceChannel |-> "cosmos-hub",
  sourcePort |-> "bitcoin-hub"]

(* Transition 12 to State6 *)

State6 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 3
/\ count = 4
/\ error = FALSE
/\ handler = "OnRecvAcknowledgementResult"
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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            :> 0,
      error |-> FALSE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementError",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]],
              receiver |-> "a1",
              sender |-> "a2"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 4
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementResult",
      packet |->
        [data |->
            [amount |-> 2,
              denomTrace |->
                [denom |-> "",
                  prefix0 |-> [channel |-> "channel-1", port |-> "channel-1"],
                  prefix1 |-> [channel |-> "channel-0", port |-> ""]],
              receiver |-> "",
              sender |-> "a1"],
          destChannel |-> "transfer",
          destPort |-> "cosmos-hub",
          sourceChannel |-> "cosmos-hub",
          sourcePort |-> "bitcoin-hub"]]
/\ p = [data |->
    [amount |-> 1,
      denomTrace |->
        [denom |-> "eth",
          prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a3",
      sender |-> "a3"],
  destChannel |-> "channel-0",
  destPort |-> "transfer",
  sourceChannel |-> "channel-1",
  sourcePort |-> "transfer"]

(* Transition 1 to State7 *)

State7 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 3
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
  @@ <<
    [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |-> "eth",
      prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 1
/\ count = 5
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
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            :> 0,
      error |-> FALSE,
      handler |-> "OnTimeoutPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a3"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementError",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]],
              receiver |-> "a1",
              sender |-> "a2"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 3,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
  @@ 4
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "OnRecvAcknowledgementResult",
      packet |->
        [data |->
            [amount |-> 2,
              denomTrace |->
                [denom |-> "",
                  prefix0 |-> [channel |-> "channel-1", port |-> "channel-1"],
                  prefix1 |-> [channel |-> "channel-0", port |-> ""]],
              receiver |-> "",
              sender |-> "a1"],
          destChannel |-> "transfer",
          destPort |-> "cosmos-hub",
          sourceChannel |-> "cosmos-hub",
          sourcePort |-> "bitcoin-hub"]]
  @@ 5
    :> [bankAfter |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2
          @@ <<
            [channel |-> "channel-1", id |-> "", port |-> "transfer"], [denom |->
                "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 1,
      bankBefore |->
        <<
            [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
              prefix0 |-> [channel |-> "", port |-> ""],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 0
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "cosmos-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 3
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "eth",
              prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 3,
      error |-> FALSE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "eth",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-0",
          destPort |-> "transfer",
          sourceChannel |-> "channel-1",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 0,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "transfer", port |-> "cosmos-hub"],
          prefix1 |-> [channel |-> "cosmos-hub", port |-> "transfer"]],
      receiver |-> "",
      sender |-> ""],
  destChannel |-> "bitcoin-hub",
  destPort |-> "ethereum-hub",
  sourceChannel |-> "transfer",
  sourcePort |-> "channel-1"]

(* The following formula holds true in the last state and violates the invariant *)

InvariantViolation ==
  (count >= 5
      /\ (\A s1$2 \in DOMAIN history:
        \A s2$2 \in DOMAIN history:
          s1$2 = s2$2 \/ ~(history[s1$2]["handler"] = history[s2$2]["handler"])))
    /\ (\A s$2 \in DOMAIN history:
      s$2 <= 0
        \/ (history[s$2]["error"] = FALSE
          /\ history[s$2]["packet"]["data"]["amount"] > 0))

================================================================================
\* Created by Apalache on Thu Dec 10 12:49:42 CET 2020
\* https://github.com/informalsystems/apalache
