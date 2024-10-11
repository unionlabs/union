---------------------------- MODULE counterexample ----------------------------

EXTENDS MC_PBT_2C_2F

(* Constant initialization state *)
ConstInit ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })

(* Initial state *)
State0 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "Init"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence = {}
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "None">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, -1>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, -1>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PROPOSE">>, <<"c2", "PROPOSE">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"None", -1, -1>>>> })

(* Transition 3 to State1 *)
State1 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "ReceiveProposal"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence = {}
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "None">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, -1>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PROPOSE">>, <<"c2", "PROPOSE">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"None", -1, -1>>>> })

(* Transition 12 to State2 *)
State2 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPropose"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence
      = {[proposal |-> <<"v0", 3, 0>>,
        round |-> 0,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1]}
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "None">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, -1>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PROPOSE">>, <<"c2", "PREVOTE">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"None", -1, -1>>>> })

(* Transition 3 to State3 *)
State3 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "ReceiveProposal"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence
      = {[proposal |-> <<"v0", 3, 0>>,
        round |-> 0,
        src |-> "f4",
        type |-> "PROPOSAL",
        validRound |-> -1]}
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "None">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PROPOSE">>, <<"c2", "PREVOTE">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", -1>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"None", -1, -1>>>> })

(* Transition 9 to State4 *)
State4 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPrevoteOrCommitAndPrevote"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence
      = { [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "c2", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> 2] }
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "v0">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PROPOSE">>, <<"c2", "PRECOMMIT">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"v0", 3, 0>>>> })

(* Transition 12 to State5 *)
State5 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPropose"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"None", -1, -1>>, -1>>>> })
    /\ evidence
      = { [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "c2", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> 2],
        [proposal |-> <<"v1", 2, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1] }
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "v0">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PREVOTE">>, <<"c2", "PRECOMMIT">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"v0", 3, 0>>>> })

(* Transition 5 to State6 *)
State6 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPrecommitNoDecision"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"v0", 3, 0>>, 0>>>> })
    /\ evidence
      = { [id |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "c2",
          type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "c2", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> 2],
        [proposal |-> <<"v1", 2, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1] }
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ lockedValue = SetAsFun({ <<"c1", "None">>, <<"c2", "v0">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PREVOTE">>, <<"c2", "DECIDED">> })
    /\ validRound = SetAsFun({ <<"c1", -1>>, <<"c2", 0>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"None", -1, -1>>>>, <<"c2", <<"v0", 3, 0>>>> })

(* Transition 9 to State7 *)
State7 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPrevoteOrCommitAndPrevote"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"None", -1, -1>>, -1>>>>,
        <<"c2", <<<<"v0", 3, 0>>, 0>>>> })
    /\ evidence
      = { [id |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "c2",
          type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "c2", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "c1", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> 2],
        [proposal |-> <<"v1", 2, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1] }
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ lockedValue = SetAsFun({ <<"c1", "v1">>, <<"c2", "v0">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "PRECOMMIT">>, <<"c2", "DECIDED">> })
    /\ validRound = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"v1", 2, 0>>>>, <<"c2", <<"v0", 3, 0>>>> })

(* Transition 5 to State8 *)
State8 ==
  Proposer = SetAsFun({ <<0, "f4">>, <<1, "f4">>, <<2, "f4">>, <<3, "f4">> })
    /\ action = "UponProposalInPrecommitNoDecision"
    /\ beginRound
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, 7>>,
        <<<<1, "c1">>, 7>>,
        <<<<2, "c2">>, 7>>,
        <<<<1, "c2">>, 7>>,
        <<<<3, "c2">>, 7>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, 7>> })
    /\ decision
      = SetAsFun({ <<"c1", <<<<"v1", 2, 0>>, 0>>>>,
        <<"c2", <<<<"v0", 3, 0>>, 0>>>> })
    /\ evidence
      = { [id |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "c2",
          type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "c2", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
        [id |-> <<"v0", 3, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "c1", type |-> "PRECOMMIT"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "c1", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f3", type |-> "PRECOMMIT"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f3", type |-> "PREVOTE"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f4", type |-> "PRECOMMIT"],
        [id |-> <<"v1", 2, 0>>, round |-> 0, src |-> "f4", type |-> "PREVOTE"],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1],
        [proposal |-> <<"v0", 3, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> 2],
        [proposal |-> <<"v1", 2, 0>>,
          round |-> 0,
          src |-> "f4",
          type |-> "PROPOSAL",
          validRound |-> -1] }
    /\ localClock = SetAsFun({ <<"c1", 3>>, <<"c2", 2>> })
    /\ lockedRound = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ lockedValue = SetAsFun({ <<"c1", "v1">>, <<"c2", "v0">> })
    /\ msgsPrecommit
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PRECOMMIT"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PRECOMMIT"] }
        >>,
        <<1, {}>>,
        <<
          2, {[id |-> <<"v2", 3, 2>>,
            round |-> 2,
            src |-> "f3",
            type |-> "PRECOMMIT"]}
        >>,
        <<
          3, {[id |-> <<"v2", 7, 3>>,
            round |-> 3,
            src |-> "f4",
            type |-> "PRECOMMIT"]}
        >> })
    /\ msgsPrevote
      = SetAsFun({ <<
          0, { [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "c2",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "c1",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f3",
              type |-> "PREVOTE"],
            [id |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PREVOTE"] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ msgsPropose
      = SetAsFun({ <<
          0, { [proposal |-> <<"v0", 3, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> 2],
            [proposal |-> <<"v1", 2, 0>>,
              round |-> 0,
              src |-> "f4",
              type |-> "PROPOSAL",
              validRound |-> -1] }
        >>,
        <<1, {}>>,
        <<2, {}>>,
        <<3, {}>> })
    /\ proposalReceptionTime
      = SetAsFun({ <<<<0, "c1">>, 3>>,
        <<<<2, "c1">>, -1>>,
        <<<<1, "c1">>, -1>>,
        <<<<2, "c2">>, -1>>,
        <<<<1, "c2">>, -1>>,
        <<<<3, "c2">>, -1>>,
        <<<<0, "c2">>, 2>>,
        <<<<3, "c1">>, -1>> })
    /\ realTime = 0
    /\ round = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ step = SetAsFun({ <<"c1", "DECIDED">>, <<"c2", "DECIDED">> })
    /\ validRound = SetAsFun({ <<"c1", 0>>, <<"c2", 0>> })
    /\ validValue
      = SetAsFun({ <<"c1", <<"v1", 2, 0>>>>, <<"c2", <<"v0", 3, 0>>>> })

(* The following formula holds true in the last state and violates the invariant *)
InvariantViolation ==
  Skolem((\E p$41 \in { "c1", "c2" }:
    Skolem((\E q$14 \in { "c1", "c2" }:
      (~(decision[p$41] = <<<<"None", -1, -1>>, -1>>)
          /\ ~(decision[q$14] = <<<<"None", -1, -1>>, -1>>))
        /\ (\A v$9 \in { "v0", "v1" }:
          \A t$9 \in 0 .. 7:
            \A pr$4 \in 0 .. 3:
              \A r1$2 \in 0 .. 3:
                \A r2$2 \in 0 .. 3:
                  LET prop_si_7 == <<v$9, t$9, pr$4>> IN
                  ~(decision[p$41] = <<(prop_si_7), r1$2>>)
                    \/ ~(decision[q$14] = <<(prop_si_7), r2$2>>))))))

================================================================================
(* Created by Apalache on Wed May 18 11:06:20 UTC 2022 *)
(* https://github.com/informalsystems/apalache *)
