_: {
  perSystem =
    {
      self',
      pkgs,
      crane,
      ...
    }:
    let
      ucli = crane.buildWorkspaceMember {
        crateDirFromRoot = "ucli";
        cargoTestExtraAttrs = {
          partitions = 1;
          partitionType = "count";
        };
        extraEnv = {
          SQLX_OFFLINE = "1";
        };
      };
    in
    {
      inherit (ucli) packages;
      inherit (ucli) checks;
    };
}
