-------------------- MODULE typedefs ---------------------------
(*
  @typeAlias: PROCESS = Str;
  @typeAlias: VALUE = Str;
  @typeAlias: STEP = Str;
  @typeAlias: ROUND = Int;
  @typeAlias: ACTION = Str;
  @typeAlias: TRACE = Seq(Str);
  @typeAlias: TIME = Int;
  @typeAlias: PROPOSAL = <<VALUE, TIME, ROUND>>;
  @typeAlias: DECISION = <<PROPOSAL, ROUND>>;
  @typeAlias: PROPMESSAGE = 
  [
    type: STEP, 
    src: PROCESS, 
    round: ROUND,
    proposal: PROPOSAL, 
    validRound: ROUND
  ];
  @typeAlias: PREMESSAGE = 
  [
    type: STEP, 
    src: PROCESS, 
    round: ROUND,
    id: PROPOSAL
  ];
  @typeAlias: MESSAGE = 
  [
    type: STEP, 
    src: PROCESS, 
    round: ROUND,
    proposal: PROPOSAL, 
    validRound: ROUND,
    id: PROPOSAL
  ];
*)
TypeAliases == TRUE

=============================================================================