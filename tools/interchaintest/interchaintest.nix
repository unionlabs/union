{ ... }: {
  perSystem = { pkgs, rust, system, lib, dbg, inputs', goPkgs,  ensureAtRepositoryRoot,...}: {
    packages = let
      # interchaintest = goPkgs.buildGoModule {
      #   pname = "interchaintest";
      #   version = "v8.0.0";
      #   src = pkgs.fetchFromGitHub {
      #     owner = "strangelove-ventures";
      #     repo = "interchaintest";
      #     rev = "2f014d308bea4429169c94c4ba08759ce5e7be03";
      #     sha256 = "sha256-PY2S1ieVBmtb9OlF8YKke/Qlk/xZZdOe0TthmJlJyWg=";
      #   };
      #   vendorHash = "sha256-hJZ6klBzD6sbh6G7nwX+rEkh2e7Tq/3nLlOD4dlAvXk=";
      #   buildFlags = ["-c" "-o ./bin/interchaintest ./cmd/interchaintest"];
      #   ldflags = [ "-X github.com/strangelove-ventures/interchaintest/v8/internal/version.GitSha=$(shell git describe --always --dirty)" ];
      #   preBuild = ''
      #     export GOWORK=off
      #     rm -rf local-interchain
      #   '';
      #   doCheck = false;
      # };
      # };
      interchaintest = goPkgs.buildGoModule {
        pname = "interchaintest";
        version = "v8.0.0";
        src = pkgs.fetchFromGitHub {
          owner = "strangelove-ventures";
          repo = "interchaintest";
          rev = "2f014d308bea4429169c94c4ba08759ce5e7be03";
          sha256 = "sha256-PY2S1ieVBmtb9OlF8YKke/Qlk/xZZdOe0TthmJlJyWg=";
        };
        vendorHash = "sha256-hJZ6klBzD6sbh6G7nwX+rEkh2e7Tq/3nLlOD4dlAvXk=";
        buildInputs = [ pkgs.git ];
        buildPhase = ''
          runHook preBuild
          runHook renameImports

          export GOWORK=off
          echo "LINENO: $LINENO"

          echo "pwd: $(pwd)"
          echo "ls: $(ls -a)"
          echo "ls nixbuildtop/go/bin: $(ls $NIX_BUILD_TOP/go/bin)"

        	go test -c -o $out/bin/interchaintest ./cmd/interchaintest

          echo "ls bin: $(ls ./bin)"

          runHook postBuild
        '';
        doCheck = false;
      };
      union_matrix = {
        Relayers = [ "rly" ];
        ChainSets = [
          [
            {
              Name = "union";
              Version = "latest";
            }
            {
              Name = "osmosis";
              Version = "latest";
            }
          ]
        ];
      };
      union_interchaintest_config = pkgs.linkFarm "union_interchaintest_config" [
        {
          name = "union_matrix.json";
          path = pkgs.writeText "union_matrix.json" (builtins.toJSON union_matrix);
        }
      ];
    in
    {
      interchaintest-uniond-conformance = pkgs.writeShellApplication {
        name = "interchaintest-uniond-conformance";
        runtimeInputs = [ interchaintest ];
        text = ''
          set -eo pipefail
          ${ensureAtRepositoryRoot}
          cd uniond

          echo "Running interchaintest conformance check for uniond..."

        '';
      };
    };
  };
}
