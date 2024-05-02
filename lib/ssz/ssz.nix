{ inputs, ... }: {
  perSystem = { self', inputs', pkgs, system, config, crane, stdenv, dbg, ensureAtRepositoryRoot, mkCi, ... }:
    let
      spec_compliance_tests_dir = "lib/ssz/tests/spec_conformance";
      spec_compliance_tests_file = "lib/ssz/tests/spec_conformance.rs";

      tests-generator = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ssz/tests-generator";
      };

      ssz-compliance-tests = mkCi false (pkgs.stdenv.mkDerivation {
        name = "ssz-compliance-tests";
        src = crane.cargoWorkspaceSrc;
        buildInputs = [ tests-generator.packages.ssz-tests-generator config.treefmt.build.programs.rustfmt ];
        buildPhase = ''
          mkdir -p $out/tests

          ssz-tests-generator "${inputs.ethereum-consensus-specs}" $out/spec_conformance.rs $out/tests

          rustfmt --config-path ${../../rustfmt.toml} --config skip_children=true $out/tests/* $out/spec_conformance.rs
        '';
      });

      generate-ssz-compliance-tests = mkCi (system == "x86_64") (pkgs.writeShellApplication {
        name = "generate-ssz-compliance-tests";
        text = ''
          ${ensureAtRepositoryRoot}

          cp --no-preserve=mode ${ssz-compliance-tests}/tests/* ${spec_compliance_tests_dir}
          cp --no-preserve=mode ${ssz-compliance-tests}/spec_conformance.rs ${spec_compliance_tests_file}
        '';
      });

      ssz = crane.buildWorkspaceMember {
        crateDirFromRoot = "lib/ssz";
      };
    in
    {
      checks = ssz.checks // {
        ssz-tests-up-to-date = mkCi (system == "x86_64") (pkgs.stdenv.mkDerivation {
          name = "ssz-tests-up-to-date";
          src = crane.cargoWorkspaceSrc;
          doCheck = true;
          checkPhase = ''
            diff ${ssz-compliance-tests}/tests ${spec_compliance_tests_dir}
            diff ${ssz-compliance-tests}/spec_conformance.rs ${spec_compliance_tests_file}

            touch $out
          '';
        });
      };

      packages = {
        inherit generate-ssz-compliance-tests ssz-compliance-tests;
      };
    };
}
