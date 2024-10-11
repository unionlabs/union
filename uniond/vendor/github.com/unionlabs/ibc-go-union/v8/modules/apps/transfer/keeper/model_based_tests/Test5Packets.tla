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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 2,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "", port |-> ""],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a3",
      sender |-> "a3"],
  destChannel |-> "channel-1",
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
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 1,
      denomTrace |->
        [denom |-> "btc",
          prefix0 |-> [channel |-> "", port |-> ""],
          prefix1 |-> [channel |-> "", port |-> "cosmos-hub"]],
      receiver |-> "a3",
      sender |-> "a1"],
  destChannel |-> "channel-1",
  destPort |-> "transfer",
  sourceChannel |-> "ethereum-hub",
  sourcePort |-> "channel-0"]

(* Transition 0 to State4 *)

State4 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
/\ count = 2
/\ error = TRUE
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> TRUE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> "cosmos-hub"]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "ethereum-hub",
          sourcePort |-> "channel-0"]]
/\ p = [data |->
    [amount |-> 4,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a2",
      sender |-> "a2"],
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
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 4
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> TRUE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> "cosmos-hub"]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "ethereum-hub",
          sourcePort |-> "channel-0"]]
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 4
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 4,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |->
                    [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a2"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 4,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
          prefix1 |-> [channel |-> "", port |-> ""]],
      receiver |-> "a2",
      sender |-> ""],
  destChannel |-> "channel-1",
  destPort |-> "transfer",
  sourceChannel |-> "channel-0",
  sourcePort |-> "transfer"]

(* Transition 5 to State6 *)

State6 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 8
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
/\ count = 4
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> TRUE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> "cosmos-hub"]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "ethereum-hub",
          sourcePort |-> "channel-0"]]
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 4
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 4,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |->
                    [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a2"],
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 8
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 4
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 4,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |->
                    [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> ""],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
/\ p = [data |->
    [amount |-> 1,
      denomTrace |->
        [denom |-> "atom",
          prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
          prefix1 |-> [channel |-> "channel-0", port |-> "transfer"]],
      receiver |-> "",
      sender |-> "a1"],
  destChannel |-> "channel-0",
  destPort |-> "channel-1",
  sourceChannel |-> "cosmos-hub",
  sourcePort |-> "bitcoin-hub"]

(* Transition 0 to State7 *)

State7 ==
/\ bank = <<
    [channel |-> "", id |-> "", port |-> ""], [denom |-> "",
      prefix0 |-> [channel |-> "", port |-> ""],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 0
  @@ <<
    [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
      prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
      prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
  >>
    :> 8
  @@ <<
    [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
      prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
      prefix1 |-> [channel |-> "", port |-> ""]]
  >>
    :> 2
/\ count = 5
/\ error = TRUE
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
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
            [amount |-> 2,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a3",
              sender |-> "a3"],
          destChannel |-> "channel-1",
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> TRUE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "btc",
                  prefix0 |-> [channel |-> "", port |-> ""],
                  prefix1 |-> [channel |-> "", port |-> "cosmos-hub"]],
              receiver |-> "a3",
              sender |-> "a1"],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "ethereum-hub",
          sourcePort |-> "channel-0"]]
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 4
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 4,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |->
                    [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> "a2"],
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 8
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 4
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> FALSE,
      handler |-> "OnRecvPacket",
      packet |->
        [data |->
            [amount |-> 4,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |->
                    [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
                  prefix1 |-> [channel |-> "", port |-> ""]],
              receiver |-> "a2",
              sender |-> ""],
          destChannel |-> "channel-1",
          destPort |-> "transfer",
          sourceChannel |-> "channel-0",
          sourcePort |-> "transfer"]]
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
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 8
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
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
            [channel |-> "", id |-> "a2", port |-> ""], [denom |-> "atom",
              prefix0 |-> [channel |-> "cosmos-hub", port |-> "ethereum-hub"],
              prefix1 |-> [channel |-> "channel-1", port |-> "transfer"]]
          >>
            :> 8
          @@ <<
            [channel |-> "", id |-> "a3", port |-> ""], [denom |-> "btc",
              prefix0 |-> [channel |-> "channel-1", port |-> "transfer"],
              prefix1 |-> [channel |-> "", port |-> ""]]
          >>
            :> 2,
      error |-> TRUE,
      handler |-> "SendTransfer",
      packet |->
        [data |->
            [amount |-> 1,
              denomTrace |->
                [denom |-> "atom",
                  prefix0 |-> [channel |-> "channel-0", port |-> "transfer"],
                  prefix1 |-> [channel |-> "channel-0", port |-> "transfer"]],
              receiver |-> "",
              sender |-> "a1"],
          destChannel |-> "channel-0",
          destPort |-> "channel-1",
          sourceChannel |-> "cosmos-hub",
          sourcePort |-> "bitcoin-hub"]]
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
  count >= 5
    /\ BMC!Skolem((\E s1$2 \in DOMAIN history:
      BMC!Skolem((\E s2$2 \in DOMAIN history:
        ~(history[s1$2]["handler"] = history[s2$2]["handler"])))))

================================================================================
\* Created by Apalache on Thu Dec 10 11:52:41 CET 2020
\* https://github.com/informalsystems/apalache
