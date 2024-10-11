-------------------- MODULE typedefs ---------------------------
(*
  @typeAlias: process = Str;
  @typeAlias: value = Str;
  @typeAlias: step = Str;
  @typeAlias: round = Int;
  @typeAlias: action = Str;
  @typeAlias: trace = Seq(Str);
  @typeAlias: proposeMsg =
  {
    type: $step,
    src: $process,
    round: $round,
    proposal: $value,
    validRound: $round
  };
  @typeAlias: preMsg =
  {
    type: $step,
    src: $process,
    round: $round,
    id: $value
  };
*)
TypeAliases == TRUE

=============================================================================
