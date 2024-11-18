_: {
  perSystem =
    {
      crane,
      lib,
      ...
    }:
    let
      lc = crane.buildWasmContract {
        crateDirFromRoot = "light-clients/movement/ics08-movement";
      };
    in
    {
      inherit (lc) packages checks;
    };
}
