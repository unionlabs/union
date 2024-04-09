{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      ssz = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ssz";
        extraEnv = {
          ETHEREUM_CONSENSUS_SPECS_DIR = "${inputs.ethereum-consensus-specs}";
        };
      };
    in
    {
      checks = ssz.checks;
    };
}
