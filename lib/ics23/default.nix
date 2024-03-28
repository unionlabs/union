{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      ics23TestSuite = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ics23";
        extraEnv = {
          ICS23_TEST_SUITE_DATA_DIR = "${inputs.ics23}/testdata";
        };
      };
    in
    {
      checks = ics23TestSuite.checks;
    };
}
