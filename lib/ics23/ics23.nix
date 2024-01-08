{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, config, crane, stdenv, dbg, ... }:
    let
      ics23TestSuite = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ics23/test-suite";
      };
    in
    {
      checks = {
        ics23-test-suite = pkgs.stdenv.mkDerivation
          {
            name = "ics23-test-suite";
            src = ./test-suite;
            buildInputs = [ ics23TestSuite.packages.ics23-test-suite ];
            doCheck = true;
            checkPhase = ''
              ics23-test-suite ${inputs.ics23}/testdata
              touch $out
            '';
          };
      };
    };
}
