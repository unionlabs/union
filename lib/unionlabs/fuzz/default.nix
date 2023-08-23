{ ... }: {
  perSystem = { pkgs, self', crane, rust, system, ensureAtRepositoryRoot, ... }:
    let
      cargo-fuzz = crane.lib.buildPackage {
        name = "cargo-fuzz";
        version = "0.11.2";
        doCheck = false;
        src = pkgs.fetchFromGitHub {
          owner = "rust-fuzz";
          repo = "cargo-fuzz";
          rev = "a860fd92bc183f1fed45583aa78691b684e80576";
          hash = "sha256-vnOqImf3GWbDP6uM9tINGHliVDRdXnSPX1OMeT4N5qU=";
        };
      };

      rustfilt = crane.lib.buildPackage {
        name = "rustfilt";
        version = "0.2.2-alpha.0";
        doCheck = false;
        src = pkgs.fetchFromGitHub {
          owner = "luser";
          repo = "rustfilt";
          rev = "8cf08c0680ebd17e7c1ae5c67227fa7026129af6";
          hash = "sha256-9u3npksi2gBScgJA+uuCNTq6hGeaQNirC/D06vH8DC8=";
        };
      };

      rustTarget = pkgs.rust.toRustTarget pkgs.stdenv.hostPlatform;

      max_total_time = "60";

      # TODO: Continuous fuzzing in nightly CI
      # https://github.com/google/fuzzing/blob/master/tutorial/libFuzzerTutorial.md#continuous-fuzzing
      runFuzzTargets = targets: pkgs.writeShellApplication {
        name = "fuzz";
        runtimeInputs = [ cargo-fuzz rust.toolchains.dev ];
        text = ''
          ${ensureAtRepositoryRoot}

          mkdir -p fuzzing-code-coverage

          ${pkgs.lib.concatMapStrings 
            (target: ''
              cargo fuzz run \
                ${target} \
                --fuzz-dir=lib/unionlabs/fuzz/ \
                -- \
                -max_total_time=${max_total_time}

              cargo fuzz coverage \
                ${target} \
                --fuzz-dir=lib/unionlabs/fuzz/
            '')
            targets}

          echo "merging profdata"

          ${rust.toolchains.dev}/lib/rustlib/${rustTarget}/bin/llvm-profdata \
            merge \
            ${pkgs.lib.concatMapStrings 
              (target: "lib/unionlabs/fuzz/coverage/${target}/coverage.profdata ")
              targets} \
            -o fuzzing-code-coverage/merged.profdata

          echo "generating coverage"

          # TODO: Figure out what to pass for the BIN argument (duration_roundtrip is
          # used for the time being; passing multiple BINs produces strange results)
          #
          # Perhaps build the whole crate with the required llvm arguments?
          # https://llvm.org/docs/CommandGuide/llvm-cov.html
          ${rust.toolchains.dev}/lib/rustlib/${rustTarget}/bin/llvm-cov \
            show \
            -Xdemangler=${rustfilt}/bin/rustfilt \
            target/${rustTarget}/coverage/${rustTarget}/release/duration_roundtrip \
            --instr-profile fuzzing-code-coverage/merged.profdata \
            --show-line-counts-or-regions \
            --show-instantiations \
            --show-branches percent \
            --sources . \
            --format html \
            --output-dir="./fuzzing-code-coverage"
        '';
      };
    in
    {
      # TODO: Get these from `cargo fuzz list`
      # This was easier than figuring it out in bash
      packages.fuzz = runFuzzTargets [
        "duration_checked_add"
        "duration_from_str"
        "duration_roundtrip"
        "timestamp_roundtrip"
      ];
    };
}
